use kirby_assist_core::{
    fighter_profiles::presets::PresetSlot,
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{
        InputTimeline, LedgeTrap, LedgeVisits, CLIFF_CATCH_MOVE, CLIFF_CLIMB, CLIFF_WAIT,
    },
    manual_controls::ManualTakeover,
    native_input::*,
    reaction_rules::{Candidate, Resolved},
    sequence_config::{persist, Action, Catalog},
    settings::AssistSettings,
};
fn sequence(actions: &str) -> kirby_assist_core::sequence_config::Sequence {
    Catalog::parse(&format!(
        "[bowser.drop.test]\nstart_delay=0\ntimeout=10\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
fn ctx(frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status: kirby_assist_core::native_states::status::WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn enemy(status: i32) -> Opponent {
    Opponent {
        id: 2,
        status,
        vulnerable: status == CLIFF_CATCH_MOVE,
        situation: kirby_assist_core::native_states::situation::AIR,
        position: [6., -2.],
        cliff_position: Some([10., 0.]),
        ..Default::default()
    }
}
#[test]
fn shield_until_manual_ignores_duplicates_and_timeout_then_hands_back_without_next_attack() {
    let mut tape = InputTimeline::new(sequence("shield_until_manual,jab:4"), enemy(CLIFF_WAIT));
    let mut c = ctx(0.);
    c.manual_input_epoch = 7;
    assert_eq!(
        tape.sample(AssistSettings::default(), c, None, None, 0.)
            .requested,
        MAPPER_ZR
    );
    for _ in 0..20 {
        c.motion_frame += 100.;
        assert_eq!(
            tape.sample(AssistSettings::default(), c, None, None, 0.)
                .requested,
            MAPPER_ZR
        );
    }
    c.manual_input_epoch = 8;
    let out = tape.sample(AssistSettings::default(), c, None, None, 0.);
    assert_eq!(out.requested, 0);
    assert_eq!(out.owned, 0);
    assert!(tape.finished());
}
#[test]
fn until_manual_cannot_create_an_air_dodge_and_timed_shield_stays_timed() {
    let mut c = ctx(0.);
    c.situation = kirby_assist_core::native_states::situation::AIR;
    let mut tape = InputTimeline::new(sequence("shield_until_manual"), enemy(CLIFF_WAIT));
    assert_eq!(
        tape.sample(AssistSettings::default(), c, None, None, 0.)
            .requested,
        0
    );
    let mut timed = InputTimeline::new(sequence("shield:2"), enemy(CLIFF_WAIT));
    assert_eq!(
        timed
            .sample(AssistSettings::default(), ctx(0.), None, None, 0.)
            .requested,
        MAPPER_ZR
    );
    timed.sample(AssistSettings::default(), ctx(1.), None, None, 0.);
    assert_eq!(
        timed
            .sample(AssistSettings::default(), ctx(2.), None, None, 0.)
            .requested,
        0
    );
}
#[test]
fn physical_shield_or_stick_change_takes_over_but_held_assist_and_analog_noise_do_not() {
    let mut t = ManualTakeover::new();
    t.sample(MAPPER_L, [0.; 4]);
    t.sample(MAPPER_L, [0.02, 0., 0., 0.]);
    assert_eq!(t.epoch(), 0);
    t.sample(MAPPER_L | MAPPER_ZR, [0.; 4]);
    assert_eq!(t.epoch(), 1);
    t.sample(MAPPER_L | MAPPER_ZR, [0.; 4]);
    assert_eq!(t.epoch(), 1);
    t.sample(MAPPER_L | MAPPER_ZR, [0.7, 0., 0., 0.]);
    assert_eq!(t.epoch(), 2);
}
fn settings() -> AssistSettings {
    let mut s = AssistSettings {
        enabled: true,
        ..Default::default()
    };
    s.sequence_overrides[PresetSlot::Drop as usize] = Some(sequence("shield:90"));
    s.sequence_overrides[0] = Some(sequence("jab:1"));
    s
}
fn drop() -> Opponent {
    Opponent {
        status: kirby_assist_core::native_states::status::FALL,
        previous_status: CLIFF_WAIT,
        ..enemy(0xe)
    }
}
fn sample(
    t: &mut LedgeTrap,
    v: &mut LedgeVisits,
    n: i64,
    e: Opponent,
    _r: Option<&Resolved>,
) -> ControllerOutput {
    t.sample_with_visits(v, settings(), n, ctx(n as f32), Some(e))
}
#[test]
fn quick_regrab_interrupts_long_drop_shield_and_keeps_next_getup_available() {
    let mut t = LedgeTrap::default();
    let mut v = LedgeVisits::default();
    sample(&mut t, &mut v, 0, enemy(CLIFF_WAIT), None);
    assert_eq!(sample(&mut t, &mut v, 1, drop(), None).requested, MAPPER_ZR);
    assert!(t.file_timeline_active());
    assert_eq!(
        sample(&mut t, &mut v, 2, enemy(CLIFF_CATCH_MOVE), None).requested,
        0
    );
    assert_eq!(v.visit_epoch(), 2);
    assert_eq!(
        sample(&mut t, &mut v, 3, enemy(CLIFF_CLIMB), None).requested,
        MAPPER_A
    );
}
fn rules() -> Resolved {
    let seq=Catalog::parse("[bowser.drop.regrab]\nstart_delay=0\nreaction=ledge_regrab:grounded:0:18:30:any:1\nactions=down_smash:4\n").unwrap().entries()[0].sequence;
    let mut r = Resolved::default();
    r.add(Candidate {
        slot: 4,
        index: 0,
        sequence: seq,
    })
    .unwrap();
    r
}

#[test]
fn landing_between_visits_is_not_a_regrab_and_new_syntax_round_trips() {
    let mut t = LedgeTrap::default();
    let mut v = LedgeVisits::default();
    let r = rules();
    sample(&mut t, &mut v, 0, enemy(CLIFF_WAIT), Some(&r));
    sample(&mut t, &mut v, 1, drop(), Some(&r));
    sample(
        &mut t,
        &mut v,
        2,
        Opponent {
            situation: kirby_assist_core::native_states::situation::GROUND,
            status: kirby_assist_core::native_states::status::WAIT,
            ..enemy(0)
        },
        Some(&r),
    );
    assert_eq!(
        sample(&mut t, &mut v, 3, enemy(CLIFF_WAIT), Some(&r)).requested,
        0
    );
    assert_eq!(v.visit_epoch(), 2);
    let s = sequence("shield_until_manual");
    let text = persist::update_named(
        "[bowser.drop.manual]\nstart_delay=0\nactions=shield:4\n",
        15,
        PresetSlot::Drop,
        "manual",
        s,
        false,
        Some("Hold shield"),
    )
    .unwrap();
    assert_eq!(
        Catalog::parse(&text).unwrap().entries()[0].sequence.actions[0],
        Action::ShieldUntilManual
    );
}

#[test]
fn neutral_b_holds_across_animation_loops_until_physical_takeover_without_a_release_frame() {
    let mut tape = InputTimeline::new(sequence("hold_until_manual:b:0:0,jab:4"), enemy(CLIFF_WAIT));
    let mut c = ctx(0.);
    c.manual_input_epoch = 3;
    assert_eq!(
        tape.sample(AssistSettings::default(), c, None, None, 0.)
            .requested,
        MAPPER_B
    );
    for n in 0..200 {
        c.motion_frame = (n % 20) as f32;
        assert_eq!(
            tape.sample(AssistSettings::default(), c, None, None, 0.)
                .requested,
            MAPPER_B
        );
    }
    c.manual_input_epoch = 4;
    let out = tape.sample(AssistSettings::default(), c, None, None, 0.);
    assert_eq!(out.requested, 0);
    assert_eq!(out.owned, 0);
    assert!(tape.finished());
    let s = sequence("facing_hold_until_manual:b:1:0");
    let source = "[bowser.drop.manual]\nstart_delay=0\nactions=b:4\n";
    let output =
        persist::update_named(source, 15, PresetSlot::Drop, "manual", s, false, None).unwrap();
    assert_eq!(Catalog::parse(&output).unwrap().entries()[0].sequence, s);
}

#[test]
fn vulnerable_hanging_opponent_has_no_automatic_punish() {
    let mut t = LedgeTrap::default();
    let mut v = LedgeVisits::default();
    for n in 0..20 {
        assert_eq!(
            sample(
                &mut t,
                &mut v,
                n,
                Opponent {
                    vulnerable: true,
                    ..enemy(CLIFF_WAIT)
                },
                None
            )
            .requested,
            0
        );
    }
    assert_eq!(
        sample(&mut t, &mut v, 20, enemy(CLIFF_CLIMB), None).requested,
        MAPPER_A
    );
}
