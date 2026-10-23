//! Existing regression cases for `live_input`. Kept outside the runtime code.
use super::*;

#[test]
fn direct_config_selects_one_controller_and_takeover_tools() {
    let config = LiveConfig::parse(include_str!(
        "../../../../deployment/ultimate/kirby_assist/config.txt"
    ))
    .unwrap();
    assert_eq!(config.player_index, 0);
    assert!(config.controls.takeover);
    assert!(config.controls.auto_mash && config.controls.auto_tech);
}

#[test]
fn analog_mash_composes_with_button_takeover_and_restores_fresh_physical_input() {
    use crate::manual_controls::{
        ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS, RIGHT_STICK_DIRECTIONS, STICK_CLICKS,
    };
    use crate::native_input::{mapper_buttons, ButtonTrace, MapperButtonOverlay};
    let settings = LiveSettings {
        takeover: true,
        ..LiveSettings::default()
    };
    let mut controls = RawAutoControls::default();
    let mut buttons = MapperButtonOverlay::default();
    let mut manual = ManualOverlay::default();
    let blocked = RIGHT_STICK_DIRECTIONS | STICK_CLICKS;
    let held = mapper_buttons(ASSIST_BUTTONS) | blocked | LEFT_STICK_DIRECTIONS | (1 << 18);
    let human = ButtonTrace {
        held,
        previous: held,
        ..ButtonTrace::default()
    };
    let physical_stick = [0.2, -0.3];
    for tick in 0..12 {
        let requested = controls.update(settings, tick, 7, true, false);
        assert_eq!(
            requested, 0,
            "analog mash must never press an action button"
        );
        assert_eq!(controls.update(settings, tick, 7, true, false), requested);
        let stick = controls.mash_stick().unwrap();
        let assisted = buttons.apply(human, requested, ASSIST_BUTTONS);
        let frame = manual.apply_controls(
            assisted,
            physical_stick,
            TurboOutput {
                consume_dpad: false,
                stick: Some(stick),
            },
            0,
            0,
            blocked,
        );
        assert_eq!(frame.stick, stick);
        assert_eq!(
            frame.buttons.held & (mapper_buttons(ASSIST_BUTTONS) | blocked),
            0
        );
        assert_eq!((frame.buttons.held & LEFT_STICK_DIRECTIONS).count_ones(), 1);
        assert_ne!(frame.buttons.held & (1 << 18), 0, "menu is never owned");
    }
    assert_eq!(controls.update(settings, 12, 7, false, false), 0);
    assert_eq!(controls.mash_stick(), None);
    let assisted = buttons.apply(human, 0, 0);
    let frame = manual.apply_controls(assisted, physical_stick, TurboOutput::default(), 0, 0, 0);
    assert_eq!(frame.stick, physical_stick);
    assert_eq!(frame.buttons.held, human.held);
    assert_eq!(
        manual
            .apply_controls(human, physical_stick, TurboOutput::default(), 0, 0, 0)
            .buttons,
        human
    );
}

#[test]
fn bundled_config_keeps_analog_mash_and_extended_prediction() {
    let config = LiveConfig::parse(include_str!(
        "../../../../deployment/ultimate/kirby_assist/config.txt"
    ))
    .unwrap();
    assert!(config.ledge.enabled);
    assert_eq!(config.ledge.horizontal_range, 20.0);
    assert_eq!(config.ledge.vertical_range, 10.0);
    assert_eq!(config.ledge.tilt_range, 15.0);
    assert_eq!(config.ledge.grab_range, 9.0);
    assert_eq!(config.ledge.follow_range, 40.0);
    assert_eq!(config.ledge.follow_vertical_range, 48.0);
    assert!(!config.ledge.auto_position);
    assert!(config.ledge.manual_spacing);
    assert_eq!(config.ledge.walk_strength, 0.35);
    assert_eq!(config.ledge.roll_dash_range, 10.0);
    assert_eq!(config.ledge.stand_distance, 10.0);
    assert_eq!(config.ledge.move_strength, 0.65);
    assert_eq!(config.ledge.grab_buffer_frames, 3);
    assert_eq!(config.ledge.grab_button, crate::native_input::MAPPER_R);
    assert_eq!(config.ledge.anti_air_range, 12.0);
    assert_eq!(config.ledge.anti_air_height, 18.0);
    assert!(config.turbo.enabled);
    assert!(config.dpad_slide);
    assert_eq!(config.turbo.phase_frames, 2);
    assert_eq!(config.turbo.walk_strength, 0.35);
    assert_eq!(config.lookahead, 5.0);
}

#[test]
fn analog_mash_to_tech_keeps_the_existing_zr_release_and_cooldown() {
    let settings = LiveSettings {
        takeover: true,
        ..LiveSettings::default()
    };
    let mut controls = RawAutoControls::default();
    let mut overlay = HidButtonOverlay::default();
    for tick in 0..8 {
        assert_eq!(controls.update(settings, tick, 1, true, false), 0);
        assert!(controls.mash_stick().is_some());
    }
    assert_eq!(
        controls.update_with_buttons(settings, 8, 1, false, true, HID_ZR),
        0
    );
    assert_eq!(controls.mash_stick(), None);
    assert_eq!(overlay.apply_owned(9, HID_ZR, 0, HID_ZR, true), 0);
    assert_eq!(controls.update(settings, 9, 1, false, true), HID_ZR);
    assert_eq!(
        overlay.apply_owned(10, HID_ZR, HID_ZR, HID_ZR, true),
        HID_ZR
    );
    assert_eq!(controls.update(settings, 10, 1, false, true), 0);
    assert_eq!(overlay.apply_owned(11, HID_ZR, 0, HID_ZR, true), 0);
}

#[test]
fn one_update_tech_opportunity_is_not_delayed_when_zr_is_released() {
    let settings = LiveSettings {
        takeover: true,
        ..LiveSettings::default()
    };
    let mut controls = RawAutoControls::default();
    assert_eq!(
        controls.update_with_buttons(settings, 0, 1, false, true, 0),
        HID_ZR
    );
    assert_eq!(
        controls.update_with_buttons(settings, 1, 1, false, false, HID_ZR),
        0
    );
    assert_eq!(
        controls.update_with_buttons(settings, 2, 1, false, true, 0),
        0
    );
}

#[test]
fn grounded_getup_move_is_selectable_without_changing_other_responses() {
    let default = LiveConfig::parse("ledge_trap = true").unwrap();
    assert_eq!(
        default.ledge.getup_attack,
        crate::ledge_trap::GetupAttack::NeutralAir
    );
    let forward =
        LiveConfig::parse("ledge_trap = true\nledge_getup_attack = FORWARD_TILT").unwrap();
    assert_eq!(
        forward.ledge.getup_attack,
        crate::ledge_trap::GetupAttack::ForwardTilt
    );
    assert_eq!(forward.ledge.anti_air_range, default.ledge.anti_air_range);
    assert_eq!(forward.ledge.grab_button, default.ledge.grab_button);
}

#[test]
fn legacy_throw_setting_stays_parseable_and_bundled_capture_is_manual() {
    assert!(!LiveConfig::default().ledge.throw_outward);
    assert!(
        !LiveConfig::parse("ledge_throw_outward = false")
            .unwrap()
            .ledge
            .throw_outward
    );
    assert!(LiveConfig::parse("ledge_throw_outward = maybe").is_err());
    let supplied = LiveConfig::parse(include_str!(
        "../../../../deployment/ultimate/kirby_assist/config.txt"
    ))
    .unwrap();
    assert!(!supplied.ledge.throw_outward);
    assert_eq!(
        supplied.ledge.getup_attack,
        crate::ledge_trap::GetupAttack::NeutralAir
    );
}

#[test]
fn removed_aerial_settings_do_not_disable_existing_sd_configs() {
    let config = LiveConfig::parse("ledge_trap = true\nledge_grab_button = l\nledge_jump_response = fair\nledge_fair_range = 11.0\nledge_fair_vertical_range = 10.0\nledge_fair_delay_frames = 0.0\nledge_fair_lead_frames = 4.0\nledge_fair_jump_squat_buffer = true\nledge_fair_buffer_frame = 1.0").unwrap();
    assert!(config.ledge.enabled);
    assert_eq!(config.ledge.grab_button, crate::native_input::MAPPER_L);
    assert_eq!(config.ledge.anti_air_range, 12.0);
    assert!(config.controls.auto_mash);
    assert!(config.controls.auto_tech);
}
#[test]
fn old_button_mash_settings_are_ignored_without_disabling_other_tools() {
    let config = LiveConfig::parse("auto_mash = true\nalternate_mash = true\nmash_buttons_per_frame = 4\nmash_interval_frames = 3\ntakeover = true").unwrap();
    let mut controls = RawAutoControls::default();
    for tick in 0..4 {
        assert_eq!(controls.update(config.controls, tick, 1, true, false), 0);
        assert!(controls.mash_stick().is_some());
    }
    assert!(config.controls.auto_tech);
}

#[test]
fn startup_config_handles_comments_toggles_and_rejects_mistakes() {
    let config = LiveConfig::parse(
        "# Pro Controller\nenabled = false\nauto_tech = false\nnpad_id = 2\ntech_lookahead_frames = 4 # prediction\n",
    ).unwrap();
    assert!(!config.controls.enabled);
    assert!(!config.controls.auto_tech);
    assert!(config.controls.auto_mash);
    assert_eq!(config.player_index, 2);
    assert_eq!(config.lookahead, 4.0);
    for invalid in [
        "enabled = yes",
        "ledge_trap = yes",
        "ledge_getup_attack = smash",
        "ledge_follow_range = NaN",
        "ledge_follow_vertical_range = 101",
        "ledge_auto_position = yes",
        "ledge_stand_distance = 1",
        "ledge_move_strength = 1.1",
        "ledge_grab_buffer_frames = 7",
        "ledge_grab_button = ZR",
        "ledge_anti_air_range = NaN",
        "ledge_anti_air_height = 41",
        "ledge_grab_button = A",
        "ledge_trap_range = NaN",
        "ledge_tilt_range = NaN",
        "ledge_grab_range = 21",
        "ledge_trap_range = 21",
        "ledge_trap_vertical_range = 0",
        "turbo_phase_frames = 0",
        "turbo_phase_frames = 11",
        "turbo_walk_strength = NaN",
        "turbo_walk_strength = 0.9",
        "npad_id = 8",
        "npad_id = -1",
        "tech_lookahead_frames = NaN",
        "tech_lookahead_frames = 6",
        "auto_tehc = false",
        "enabled",
    ] {
        assert!(LiveConfig::parse(invalid).is_err(), "accepted {invalid}");
    }
}

#[test]
fn analog_cycle_advances_once_per_update_and_restarts_on_exit_or_reassignment() {
    let mut controls = RawAutoControls::default();
    let settings = LiveSettings::default();
    let expected = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
    for tick in 0..12 {
        assert_eq!(controls.update(settings, tick, 10, true, false), 0);
        assert_eq!(controls.mash_stick(), Some(expected[(tick % 4) as usize]));
        controls.update(settings, tick, 10, true, false);
        assert_eq!(controls.mash_stick(), Some(expected[(tick % 4) as usize]));
    }
    controls.update(settings, 12, 10, false, false);
    assert_eq!(controls.mash_stick(), None);
    controls.update(settings, 13, 10, true, false);
    assert_eq!(controls.mash_stick(), Some(expected[0]));
    controls.update(settings, 14, 10, true, false);
    assert_eq!(controls.mash_stick(), Some(expected[1]));
    controls.update(settings, 14, 11, true, false);
    assert_eq!(controls.mash_stick(), Some(expected[0]));
}

#[test]
fn tech_is_one_press_with_cooldown_and_no_timer_bypass() {
    let mut controls = RawAutoControls::default();
    let settings = LiveSettings {
        tech_cooldown: 4,
        ..LiveSettings::default()
    };
    assert_eq!(controls.update(settings, 0, 1, false, true), HID_ZR);
    assert_eq!(controls.update(settings, 1, 1, false, false), 0);
    assert_eq!(controls.update(settings, 2, 1, false, true), 0);
    assert_eq!(controls.update(settings, 3, 1, false, false), 0);
    assert_eq!(controls.update(settings, 4, 1, false, true), HID_ZR);
    assert_eq!(controls.update(settings, 5, 1, false, true), 0);
}

#[test]
fn tech_opportunity_that_starts_during_cooldown_fires_when_ready() {
    let settings = LiveSettings {
        tech_cooldown: 4,
        ..LiveSettings::default()
    };
    let mut controls = RawAutoControls::default();
    assert_eq!(controls.update(settings, 0, 1, false, true), HID_ZR);
    assert_eq!(controls.update(settings, 1, 1, false, false), 0);
    assert_eq!(controls.update(settings, 2, 1, false, true), 0);
    assert_eq!(controls.update(settings, 3, 1, false, true), 0);
    assert_eq!(controls.update(settings, 4, 1, false, true), HID_ZR);
    assert_eq!(controls.update(settings, 4, 1, false, true), HID_ZR);
    for tick in 5..8 {
        assert_eq!(controls.update(settings, tick, 1, false, true), 0);
    }
    assert_eq!(controls.update(settings, 8, 1, false, true), HID_ZR);
}

#[test]
fn disabling_and_input_clock_reset_cancel_old_stick_output() {
    let mut controls = RawAutoControls::default();
    let settings = LiveSettings::default();
    controls.update(settings, 20, 1, true, false);
    assert_eq!(controls.mash_stick(), Some([1.0, 0.0]));
    controls.update(
        LiveSettings {
            enabled: false,
            ..settings
        },
        20,
        1,
        true,
        false,
    );
    assert_eq!(controls.mash_stick(), None);
    controls.update(settings, 20, 1, true, false);
    controls.update(settings, 21, 1, true, false);
    assert_eq!(controls.mash_stick(), Some([0.0, 1.0]));
    controls.update(settings, 0, 1, true, false);
    assert_eq!(controls.mash_stick(), Some([1.0, 0.0]));
}

#[test]
fn per_tool_settings_and_tech_priority_cancel_mash_stick() {
    let mut controls = RawAutoControls::default();
    let settings = LiveSettings {
        auto_tech: false,
        ..LiveSettings::default()
    };
    assert_eq!(controls.update(settings, 0, 1, false, true), 0);
    assert_eq!(controls.mash_stick(), None);
    controls.update(settings, 1, 1, true, false);
    assert_eq!(controls.mash_stick(), Some([1.0, 0.0]));
    controls.update(
        LiveSettings {
            auto_mash: false,
            ..settings
        },
        2,
        1,
        true,
        false,
    );
    assert_eq!(controls.mash_stick(), None);
    let mut controls = RawAutoControls::default();
    assert_eq!(
        controls.update(LiveSettings::default(), 0, 1, true, true),
        HID_ZR
    );
    assert_eq!(controls.mash_stick(), None);
}

#[test]
fn raw_overlay_matches_button_trace_and_never_removes_human_input() {
    let mut overlay = HidButtonOverlay::default();
    let human = (1 << 1) | (1 << 7) | (1 << 63);
    // Equivalent button values to physically holding B+R while pressing A.
    assert_eq!(overlay.apply(1, human, HID_A, true), human | HID_A);
    // A policy change waits for a new hardware sample.
    assert_eq!(overlay.apply(1, human, HID_ZR, true), human | HID_A);
    assert_eq!(overlay.apply(2, human, HID_ZR, true), human | HID_ZR);
    // A release request also waits for the next native sample. Two reads
    // of the same sampling number must not fabricate a button transition.
    assert_eq!(overlay.apply(2, human, 0, true), human | HID_ZR);
    assert_eq!(overlay.apply(3, human, 0, true), human);
    // User-held A remains held during an assist-release gap.
    assert_eq!(overlay.apply(4, human | HID_A, 0, true), human | HID_A);
    assert_eq!(
        overlay.apply(5, human, u64::MAX, true),
        human | ASSIST_BUTTONS
    );
    assert_eq!(
        overlay.apply(6, human | HID_PLUS, HID_A, true),
        human | HID_PLUS
    );
    assert_eq!(overlay.apply(7, human, HID_A, false), human);
}

#[test]
fn instruction_scan_requires_a_single_aligned_match() {
    let pattern = [1, 2, 3, 4];
    assert_eq!(
        unique_instruction_offset(&[0, 0, 0, 0, 1, 2, 3, 4], &pattern),
        Some(4)
    );
    assert_eq!(
        unique_instruction_offset(&[1, 2, 3, 4, 1, 2, 3, 4], &pattern),
        None
    );
    assert_eq!(unique_instruction_offset(&[0, 1, 2, 3, 4], &pattern), None);
    assert_eq!(unique_instruction_offset(&pattern, &[]), None);
    assert_eq!(unique_instruction_offset(&[], &pattern), None);
}
#[test]
fn roll_distance_accepts_auto_or_a_measured_finite_endpoint_distance() {
    assert_eq!(
        LiveConfig::parse("ledge_roll_distance = auto")
            .unwrap()
            .ledge
            .roll_distance,
        None
    );
    assert_eq!(
        LiveConfig::parse("ledge_roll_distance = 25.0")
            .unwrap()
            .ledge
            .roll_distance,
        Some(25.0)
    );
    for value in ["NaN", "inf", "1", "101", "wrong"] {
        assert!(LiveConfig::parse(&format!("ledge_roll_distance = {value}")).is_err());
    }
}
#[test]
fn manual_spacing_config_rejects_nonfinite_and_dash_strength_settings() {
    for source in [
        "ledge_walk_strength = NaN",
        "ledge_walk_strength = 0.46",
        "ledge_walk_strength = 0.0",
        "ledge_roll_smash_range = inf",
        "ledge_roll_smash_range = 0.0",
        "ledge_manual_spacing = maybe",
    ] {
        assert!(LiveConfig::parse(source).is_err(), "{source}");
    }
    let config = LiveConfig::parse(
        "ledge_manual_spacing = true\nledge_walk_strength = 0.4\nledge_roll_smash_range = 8.5",
    )
    .unwrap();
    assert!(config.ledge.manual_spacing);
    assert_eq!(config.ledge.walk_strength, 0.4);
    assert_eq!(config.ledge.roll_dash_range, 8.5);
}
#[test]
fn player_slot_defaults_to_player_one_and_accepts_legacy_config_key() {
    assert_eq!(LiveConfig::default().player_index, 0);
    assert_eq!(
        LiveConfig::parse("player_index = 0").unwrap().player_index,
        0
    );
    assert_eq!(
        LiveConfig::parse("player_index = 3").unwrap().player_index,
        3
    );
    assert_eq!(LiveConfig::parse("npad_id = 2").unwrap().player_index, 2);
    for invalid in ["player_index = 8", "player_index = -1"] {
        assert!(LiveConfig::parse(invalid).is_err());
    }
}

#[test]
fn r3_turbo_key_and_legacy_l3_key_enable_the_same_tool() {
    for key in ["r3_turbo", "l3_turbo"] {
        assert!(
            LiveConfig::parse(&format!("{key} = true"))
                .unwrap()
                .turbo
                .enabled
        );
        assert!(
            !LiveConfig::parse(&format!("{key} = false"))
                .unwrap()
                .turbo
                .enabled
        );
    }
}

#[test]
fn fair_first_hit_followup_has_a_startup_disable_option() {
    assert!(LiveConfig::default().kirby_fair1_smash);
    assert!(
        !LiveConfig::parse("kirby_fair1_smash = false")
            .unwrap()
            .kirby_fair1_smash
    );
}
