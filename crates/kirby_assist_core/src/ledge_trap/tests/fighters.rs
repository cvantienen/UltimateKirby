//! Real catalog dispatch with native-like contexts, rather than label-only tests.
use super::*;
use crate::{fighter_profiles::FighterProfiles, ledge_menu::MenuSelection};

fn fighter_settings(kind: i32) -> LedgeSettings {
    let profiles = FighterProfiles::default();
    let p = profiles.profile(kind).unwrap();
    let base = p.apply(LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    });
    MenuSelection::new(&p.menu_options).settings(&p.menu_options, base)
}
#[test]
fn new_fighter_neutral_defaults_dispatch_their_selected_attack() {
    for (kind, vulnerable_in, buttons, stick) in [
        (9, 14.0, MAPPER_X | MAPPER_A, [0.0, 0.0]),
        (0x29, 17.0, MAPPER_X | MAPPER_A, [0.0, 0.0]),
        (11, 7.0, MAPPER_A, [0.6, -0.3]),
    ] {
        let s = fighter_settings(kind);
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            vulnerable_in: Some(vulnerable_in),
            actionable_in: Some(vulnerable_in + 3.0),
            ..enemy(CLIFF_CLIMB, false)
        };
        let out = trap.sample(s, 0, context(), Some(target));
        assert_eq!(out.requested, buttons, "fighter {kind}");
        assert_eq!(out.stick, Some(stick));
    }
}
#[test]
fn luigi_blocks_then_nairs_in_front_while_falcon_keeps_the_grab_default() {
    for (kind, punish) in [
        (9, MAPPER_X | MAPPER_A),
        (0x29, MAPPER_X | MAPPER_A),
        (11, MAPPER_ZR | MAPPER_R),
    ] {
        let s = fighter_settings(kind);
        let mut trap = LedgeTrap::default();
        let guard = LedgeContext {
            status: GUARD,
            can_act: false,
            ..context()
        };
        let target = enemy(CLIFF_ATTACK, false);
        assert_eq!(trap.sample(s, 0, guard, Some(target)).requested, MAPPER_ZR);
        trap.plan = OptionPlan::Shield { blocked: true };
        assert_eq!(trap.sample(s, 1, guard, Some(target)).requested, punish);
    }
}
#[test]
fn new_roll_responses_do_not_count_inactive_gaps_as_hitbox_coverage() {
    for (kind, last_active) in [(9, 7.0), (11, 20.0)] {
        let s = fighter_settings(kind);
        let mut trap = LedgeTrap::default();
        let ctx = LedgeContext {
            position: [25.0, 0.0],
            facing: -1.0,
            ..context()
        };
        let mut target = Opponent {
            position: [25.0, 0.0],
            cliff_position: Some([0.0, 0.0]),
            vulnerable_in: Some(last_active + 1.0),
            actionable_in: Some(last_active + 5.0),
            ..enemy(CLIFF_ESCAPE, false)
        };
        assert_eq!(trap.sample(s, 0, ctx, Some(target)).requested, 0);
        target.vulnerable_in = Some(last_active);
        assert_eq!(trap.sample(s, 1, ctx, Some(target)).requested, MAPPER_A);
    }
}
