use kirby_assist_core::{
    gameplay::GameplayContext,
    native_input::*,
    option_read::{Kind, Target},
    reaction_rules::{Candidate, Resolved, Trigger},
    sequence_config::Catalog,
    settings::AssistSettings,
    spot_punish::Punisher,
};
fn rules() -> Resolved {
    let c=Catalog::parse("[bowser.attack.spot_grab]\nstart_delay=0\nreaction=spot_dodge:grounded:0:12:18:front:1\nactions=input:r:0:0:4\n[bowser.attack.spot_smash]\nstart_delay=0\nreaction=spot_dodge:grounded:0:20:18:any:2\nactions=down_smash:4\n").unwrap();
    let mut r = Resolved::default();
    for (i, e) in c.entries().iter().enumerate() {
        r.add(Candidate {
            slot: 3,
            index: i,
            sequence: e.sequence,
        })
        .unwrap();
    }
    r
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
fn target(frame: f32) -> Target {
    Target {
        id: 2,
        kind: Kind::Spot,
        status: kirby_assist_core::native_states::ESCAPE,
        motion: 22,
        frame,
        rate: 1.,
        position: [8., 0.],
        velocity: [0.; 2],
        opens: 0.,
        closes: 10.,
    }
}
fn sample(
    p: &mut Punisher,
    r: &Resolved,
    frame: f32,
    t: Target,
) -> kirby_assist_core::gameplay::ControllerOutput {
    p.sample(
        true,
        1,
        Some(r),
        AssistSettings::default(),
        ctx(frame),
        [Some(t), None, None, None, None, None, None, None],
        None,
    )
}
#[test]
fn waits_for_vulnerability_then_selects_priority_and_attempts_once() {
    let mut p = Punisher::default();
    let r = rules();
    assert!(r.has_defense(Trigger::SpotDodge));
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                opens: 2.,
                ..target(10.)
            }
        )
        .requested,
        0
    );
    assert_eq!(sample(&mut p, &r, 1., target(12.)).requested, MAPPER_R);
    assert!(p.committed());
    for i in 2..5 {
        assert_eq!(
            sample(&mut p, &r, i as f32, target(i as f32 + 11.)).requested,
            MAPPER_R
        );
    }
    for i in 5..9 {
        assert_eq!(
            sample(&mut p, &r, i as f32, target(i as f32 + 11.)).requested,
            0
        );
    }
    assert_eq!(sample(&mut p, &r, 9., target(0.)).requested, MAPPER_R);
}
#[test]
fn side_range_and_priority_fallbacks_are_applied() {
    let r = rules();
    let mut p = Punisher::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                position: [16., 0.],
                ..target(12.)
            }
        )
        .requested,
        MAPPER_A
    );
    let mut p = Punisher::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                position: [-8., 0.],
                ..target(12.)
            }
        )
        .requested,
        MAPPER_A
    );
    let mut p = Punisher::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                position: [25., 0.],
                ..target(12.)
            }
        )
        .requested,
        0
    );
    let mut p = Punisher::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                position: [8., 25.],
                ..target(12.)
            }
        )
        .requested,
        0
    );
}
#[test]
fn ordinary_rolls_unknown_windows_and_safety_cancellation_do_not_fire_or_retry() {
    let r = rules();
    let mut p = Punisher::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                kind: Kind::Roll,
                status: kirby_assist_core::native_states::ESCAPE_F,
                ..target(12.)
            }
        )
        .requested,
        0
    );
    assert_eq!(
        sample(
            &mut p,
            &r,
            1.,
            Target {
                opens: f32::NAN,
                ..target(12.)
            }
        )
        .requested,
        0
    );
    assert_eq!(sample(&mut p, &r, 2., target(12.)).requested, MAPPER_R);
    p.cancel_pending();
    assert_eq!(sample(&mut p, &r, 3., target(13.)).requested, 0);
    assert_eq!(
        p.sample(
            false,
            1,
            Some(&r),
            AssistSettings::default(),
            ctx(4.),
            [None; 8],
            None
        )
        .requested,
        0
    );
}
#[test]
fn pending_automatic_ground_grab_cancels_on_takeoff_and_spatial_actions_require_fresh_target() {
    let r = rules();
    let mut p = Punisher::default();
    sample(&mut p, &r, 0., target(12.));
    let air = GameplayContext {
        situation: kirby_assist_core::native_states::situation::AIR,
        ..ctx(1.)
    };
    assert_eq!(
        p.sample(
            true,
            1,
            Some(&r),
            AssistSettings::default(),
            air,
            [None; 8],
            None
        )
        .requested,
        0
    );
    assert!(
        !p.committed(),
        "a pending grounded grab must not become an airborne dodge"
    );
    let c=Catalog::parse("[bowser.attack.spatial]\nstart_delay=0\nreaction=spot_dodge:any:0:20:18:any:1\nactions=face:opponent,walk_to_opponent:50%:10:1,input:r:0:0:4\n").unwrap();
    let mut r = Resolved::default();
    r.add(Candidate {
        slot: 3,
        index: 0,
        sequence: c.entries()[0].sequence,
    })
    .unwrap();
    let mut p = Punisher::default();
    sample(&mut p, &r, 0., target(12.));
    assert!(p.committed());
    p.sample(
        true,
        1,
        Some(&r),
        AssistSettings::default(),
        ctx(1.),
        [None; 8],
        None,
    );
    assert!(!p.committed());
}

#[test]
fn explicit_any_spot_rule_can_start_before_vulnerability_without_changing_legacy_rules() {
    let c=Catalog::parse("[bowser.attack.grab]\nstart_delay=0\nreaction=spot_dodge:grounded:0:22:12:front:1\nactions=input:r:0:0:4\n[bowser.attack.chase]\nstart_delay=0\nreaction=spot_dodge_any:grounded:0:30:12:any:2\nactions=up_b:4,drift:opponent:1:40\n").unwrap();
    let mut r = Resolved::default();
    for (i, e) in c.entries().iter().enumerate() {
        r.add(Candidate {
            slot: 3,
            index: i,
            sequence: e.sequence,
        })
        .unwrap();
    }
    assert!(r.has_defense(Trigger::SpotDodge));
    assert!(r.has_defense(Trigger::SpotDodgeAny));
    let mut p = Punisher::default();
    assert_ne!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                opens: 5.,
                ..target(1.)
            }
        )
        .requested
            & kirby_assist_core::native_input::MAPPER_B,
        0
    );
    let mut p = Punisher::default();
    assert_eq!(sample(&mut p, &r, 0., target(20.)).requested, MAPPER_R);
    let mut p = Punisher::default();
    assert_ne!(
        sample(
            &mut p,
            &r,
            0.,
            Target {
                opens: f32::NAN,
                closes: f32::NAN,
                ..target(1.)
            }
        )
        .requested
            & kirby_assist_core::native_input::MAPPER_B,
        0
    );
}
