use super::*;
use kirby_assist_core::{
    fighter_profiles::presets::PresetSlot,
    gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
    ledge_trap::NativeRule,
    settings::AssistSettings,
    timing::motion_hash,
};
use std::time::Duration;

/// Builds a baseline calibration observation for a selected option.
fn frame(slot: PresetSlot) -> CalibrationFrame {
    let context = GameplayContext {
        held: true,
        gameplay: true,
        manual_input_epoch: 0,
        can_act: true,
        can_control: true,
        stopped: false,
        status: kirby_assist_core::native_states::status::WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion_frame: 1.0,
        motion_kind: motion_hash("wait"),
        motion_rate: 1.0,
        position: [0.0, 0.0],
        velocity: [0.0; 2],
        ground_brake: Some(0.1),
        walk_stick: None,
        facing: 1.0,
        previous_status: kirby_assist_core::native_states::status::WAIT,
        can_grab: true,
        can_tilt: true,
        can_down_tilt: true,
        can_down_special: true,
        can_dash_attack: true,
        can_down_smash: true,
        input_delay: 0.0,
        sequence_input_delay: 0.0,
        can_jump: true,
        can_air_attack: true,
        air_physics: None,
        short_hop_speed: None,
        full_hop_speed: None,
        shield_stun_frames: None,
        shield_threat: false,
        shield_attacker: None,
        hit_confirmed: false,
        safe_ground: [true; 2],
    };
    let enemy = Opponent {
        id: 42,
        status: kirby_assist_core::native_states::status::CLIFF_ESCAPE,
        previous_status: kirby_assist_core::native_states::status::CLIFF_WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
        position: [5.0, 0.0],
        vulnerable: false,
        velocity: [0.0; 2],
        cliff_position: Some([0.0; 2]),
        vulnerable_in: Some(10.0),
        motion: Some(MotionTiming {
            kind: motion_hash("cliff_escape"),
            frame: 12.0,
            rate: 1.0,
            remaining: Some(30.0),
        }),
        attack_active: None,
        air_physics: None,
        actionable_in: Some(30.0),
    };
    CalibrationFrame {
        target: 42,
        slot,
        native_attack_active: None,
        timeline_end: None,
        expected: CalibrationMove {
            kind: CalibrationKind::Attack,
            native: NativeRule::Aerial {
                motion: motion_hash("attack_air_n"),
                custom_status: None,
            },
            first: Some(8),
            last: Some(32),
            ..CalibrationMove::unknown()
        },
        context,
        enemy: Some(enemy),
        output: ControllerOutput {
            requested: MAPPER_X | MAPPER_A,
            ..ControllerOutput::default()
        },
        phase: "prepare",
        complete: false,
        captured: false,
        blocked: false,
        endpoint: None,
        configured_range: Some(12.0),
        lead: None,
        distance_agnostic: true,
        grab_button: AssistSettings::default().grab_button,
    }
}

#[test]
/// Verifies raw recipes do not fabricate readiness or opponent failures.
fn raw_recipe_diagnostics_do_not_invent_readiness_or_opponent_failures() {
    let mut f = frame(PresetSlot::Drop);
    f.expected = CalibrationMove::unknown();
    f.enemy = None;
    f.context.can_act = false;
    f.context.can_control = false;
    f.output.requested = MAPPER_ZR;
    let mut recorder = Recorder::new();
    recorder.observe(Some(f), ("yoshi", 5), "Shield", Instant::now());
    recorder.mapped(MAPPER_ZR, MAPPER_ZR, native_input::MAPPED_GUARD, "ledge");
    let report = recorder.active.as_ref().unwrap();
    assert_eq!(report.result(), "INPUT requested; move model unavailable");
    assert!(report.explanation().contains("raw-input recipe"));
    assert!(!report.explanation().contains("No opponent was available"));
    assert!(!report.explanation().contains("controller did not start"));
    assert_ne!(report.raw_applied, 0);
}
#[test]
/// Verifies each option channel creates one report without live publication.
fn all_five_options_record_once_without_live_report_updates() {
    let now = Instant::now();
    for slot in PresetSlot::ALL {
        let mut recorder = Recorder::new();
        for _ in 0..1000 {
            recorder.observe(Some(frame(slot)), ("kirby", 6), "Nair", now);
        }
        let report = recorder.active.as_ref().unwrap();
        assert_eq!(report.first.slot, slot);
        assert_eq!(report.requests.iter().flatten().count(), 2);
        assert!(report.move_start.is_none());
        assert!(!report.format().contains("CAPTURE confirmed"));
    }
}

#[test]
/// Verifies jump mapping is not misclassified as attack/aerial confirmation.
fn jump_mapping_does_not_confirm_attack_or_wrong_aerial() {
    let now = Instant::now();
    let mut recorder = Recorder::new();
    let mut f = frame(PresetSlot::Neutral);
    recorder.observe(Some(f), ("kirby", 6), "Nair", now);
    recorder.mapped(
        MAPPER_X | MAPPER_A,
        MAPPER_X,
        native_input::MAPPED_JUMP,
        "ledge",
    );
    assert!(recorder.active.as_ref().unwrap().mapped[0]);
    assert!(!recorder.active.as_ref().unwrap().mapped[1]);
    f.output = ControllerOutput::default();
    f.context.situation = kirby_assist_core::native_states::situation::AIR;
    f.context.status = kirby_assist_core::native_states::status::ATTACK_AIR;
    f.context.motion_kind = motion_hash("attack_air_f");
    recorder.observe(
        Some(f),
        ("kirby", 6),
        "Nair",
        now + Duration::from_millis(16),
    );
    assert!(recorder.active.as_ref().unwrap().move_start.is_none());
    f.context.motion_kind = motion_hash("attack_air_n");
    f.context.motion_frame = 0.0;
    recorder.observe(
        Some(f),
        ("kirby", 6),
        "Nair",
        now + Duration::from_millis(32),
    );
    let report = recorder.active.as_ref().unwrap();
    assert!(report.move_start.is_some());
    assert!(report.hit.is_none());
    assert!(report.activity_frame.is_none());
    assert!(report
        .format()
        .contains("START observed; hit not confirmed"));
}

#[test]
/// Verifies grounded startup/charge does not count as active attack pose.
fn grounded_startup_or_charge_is_not_an_active_attack_pose() {
    let now = Instant::now();
    let mut recorder = Recorder::new();
    let mut f = frame(PresetSlot::Roll);
    f.expected = CalibrationMove {
        kind: CalibrationKind::Attack,
        native: NativeRule::Ground {
            startup: kirby_assist_core::native_states::status::ATTACK_LW4_START,
            attack: kirby_assist_core::native_states::status::ATTACK_LW4,
        },
        first: Some(7),
        last: Some(19),
        ..CalibrationMove::unknown()
    };
    recorder.observe(Some(f), ("kirby", 6), "Dsmash", now);
    f.context.status = kirby_assist_core::native_states::status::ATTACK_LW4_HOLD;
    f.context.motion_frame = 9.0;
    f.context.hit_confirmed = true;
    recorder.observe(
        Some(f),
        ("kirby", 6),
        "Dsmash",
        now + Duration::from_millis(16),
    );
    let report = recorder.active.as_ref().unwrap();
    assert!(report.move_start.is_some());
    assert!(report.activity_frame.is_none());
    assert!(report.hit.is_none());
}

#[test]
/// Verifies shield preparation retains independent grab-request spacing.
fn shield_preparation_does_not_replace_grab_request_spacing() {
    let now = Instant::now();
    let mut recorder = Recorder::new();
    let mut f = frame(PresetSlot::Attack);
    f.expected = CalibrationMove {
        kind: CalibrationKind::Grab,
        native: NativeRule::Statuses(&[kirby_assist_core::native_states::CATCH]),
        ..CalibrationMove::unknown()
    };
    f.output.requested = MAPPER_ZR;
    recorder.observe(Some(f), ("kirby", 6), "Grab", now);
    assert!(recorder.active.as_ref().unwrap().first_request.is_none());
    f.context.position[0] = 2.0;
    f.output.requested = MAPPER_R | MAPPER_ZR;
    recorder.observe(
        Some(f),
        ("kirby", 6),
        "Grab",
        now + Duration::from_millis(16),
    );
    assert_eq!(
        recorder
            .active
            .as_ref()
            .unwrap()
            .first_request
            .unwrap()
            .context
            .position[0],
        2.0
    );
    recorder.mapped(
        f.output.requested,
        f.output.requested,
        native_input::MAPPED_GUARD | native_input::MAPPED_CATCH,
        "ledge",
    );
    assert!(recorder.active.as_ref().unwrap().mapped[2]);
}

#[test]
/// Verifies completed reports stay closed until the option is revisited.
fn completed_result_stays_closed_until_next_option_visit() {
    let now = Instant::now();
    let mut recorder = Recorder::new();
    let f = frame(PresetSlot::Jump);
    recorder.observe(Some(f), ("yoshi", 5), "Nair", now);
    recorder.stop("test completion");
    recorder.observe(Some(f), ("yoshi", 5), "Nair", now);
    assert!(!recorder.active());
    recorder.observe(None, ("yoshi", 5), "Nair", now);
    recorder.observe(Some(f), ("yoshi", 5), "Nair", now);
    assert!(recorder.active());
}
#[test]
/// Verifies neutral raw reports include actual active/protection/spacing facts.
fn raw_neutral_report_survives_recipe_end_and_captures_real_active_protection_and_spacing() {
    let now = Instant::now();
    let mut recorder = Recorder::new();
    let mut f = frame(PresetSlot::Neutral);
    f.expected = CalibrationMove::unknown();
    f.complete = true;
    f.context.status = kirby_assist_core::native_states::status::JUMP_SQUAT;
    f.native_attack_active = Some(false);
    let mut enemy = f.enemy.unwrap();
    enemy.status = kirby_assist_core::ledge_trap::CLIFF_CLIMB;
    f.enemy = Some(enemy);
    recorder.observe(Some(f), ("yoshi", 5), "Raw Nair", now);
    assert!(
        recorder.active(),
        "recipe completion is not attack completion"
    );
    f.context.status = kirby_assist_core::native_states::status::ATTACK_AIR;
    f.context.motion_kind = motion_hash("attack_air_n");
    f.context.motion_frame = 3.0;
    f.context.position = [0.0, 8.0];
    f.output = ControllerOutput::default();
    f.native_attack_active = Some(true);
    recorder.observe(
        Some(f),
        ("yoshi", 5),
        "Raw Nair",
        now + Duration::from_millis(16),
    );
    let report = recorder.active.as_ref().unwrap();
    assert_eq!(report.neutral_first_active.unwrap().vulnerable, Some(false));
    assert_eq!(report.neutral_first_active.unwrap().gap, Some([5.0, -8.0]));
    assert!(report.neutral_vulnerable_active.is_none());
    enemy.vulnerable = true;
    enemy.actionable_in = Some(1.0);
    f.enemy = Some(enemy);
    f.context.motion_frame = 4.0;
    recorder.observe(
        Some(f),
        ("yoshi", 5),
        "Raw Nair",
        now + Duration::from_millis(32),
    );
    let report = recorder.active.as_ref().unwrap();
    assert_eq!(
        report.neutral_vulnerable_active.unwrap().recovery,
        Some(1.0)
    );
    let text = report.format();
    assert!(text.contains("Neutral first native active:"));
    assert!(text.contains("recovery_left=1.0"));
    assert!(text.contains("ACTIVE attack observed"));
    assert!(report.hit.is_none());
    f.context.status = kirby_assist_core::native_states::status::LANDING_ATTACK_AIR;
    f.native_attack_active = Some(false);
    recorder.observe(
        Some(f),
        ("yoshi", 5),
        "Raw Nair",
        now + Duration::from_millis(48),
    );
    assert!(!recorder.active());
}
#[test]
/// Verifies only raw neutral attack traces continue after recipe completion.
fn only_raw_neutral_attack_reports_extend_past_recipe_completion() {
    for slot in [PresetSlot::Neutral, PresetSlot::Jump] {
        let mut recorder = Recorder::new();
        let mut f = frame(slot);
        f.complete = true;
        if slot == PresetSlot::Neutral {
            f.expected = CalibrationMove::unknown();
            f.output.requested = MAPPER_X; // No attack to watch.
        }
        recorder.observe(Some(f), ("kirby", 6), "Input", Instant::now());
        assert!(!recorder.active());
    }
}
#[test]
/// Requires matching target-clock evidence before calculating native frame delta.
fn input_path_only_computes_native_delta_with_matching_clock_evidence() {
    let a = Stamp::new(frame(PresetSlot::Neutral), 0);
    let mut b = a;
    b.target_frame = Some(a.target_frame.unwrap() + 4.0);
    b.ms = 90;
    assert_eq!(target_delta(a, b), Some(4.0));
    b.target_motion = Some(99);
    assert_eq!(target_delta(a, b), None);
    b = a;
    b.target_rate = Some(f32::NAN);
    assert_eq!(target_delta(a, b), None);
    b = a;
    b.target_frame = Some(0.0);
    assert_eq!(target_delta(a, b), None);
}
#[test]
/// Verifies mapper acknowledgment is timestamped separately and retains trial label.
fn mapper_ack_has_its_own_point_and_report_uses_stored_trial_label() {
    let mut r = Recorder::new();
    let f = frame(PresetSlot::Jump);
    r.observe(Some(f), ("kirby", 6), "Nair", Instant::now());
    r.mapped(
        f.output.requested,
        f.output.requested,
        native_input::MAPPED_ATTACK | native_input::MAPPED_JUMP,
        "test",
    );
    r.active.as_mut().unwrap().test_latency = Some(4);
    let report = r.active.as_ref().unwrap();
    assert_eq!(report.test_latency, Some(4));
    assert!(report.mapped_first[1].is_some());
    assert!(report.format().contains("INPUT PATH [Attack]"));
    assert!(report.format().contains("external +4f"));
    assert!(report.format().contains("MAPPED ACK LIMIT"));
}
