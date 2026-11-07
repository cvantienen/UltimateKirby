//! File scheduling replaces native Kirby/Yoshi move timing and hop policies.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    ledge_menu::MenuSelection,
    ledge_trap::{LedgeContext, LedgeSettings, LedgeTrap, Opponent, CLIFF_CLIMB},
    native_input::{MAPPER_A, MAPPER_X},
    sequence_config::Catalog,
};
fn ctx(frame: f32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.0,
        facing: 1.0,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn target() -> Opponent {
    Opponent {
        id: 2,
        status: CLIFF_CLIMB,
        position: [5.0, 0.0],
        ..Default::default()
    }
}
#[test]
fn kirby_authored_nair_waits_exactly_five_frames_after_the_jump_input() {
    let file = Catalog::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    )))
    .unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    let p = profiles.profile(6).unwrap();
    let mut menu = MenuSelection::new(&p.menu_options);
    menu.choices[0] = p.menu_options.lists[0]
        .options
        .iter()
        .position(|c| c.id == "nair_wait")
        .unwrap();
    let s = menu.settings(
        &p.menu_options,
        p.apply(LedgeSettings {
            enabled: true,
            ..Default::default()
        }),
    );
    let mut trap = LedgeTrap::default();
    let jump = trap.sample(s, 0, ctx(0.0), Some(target()));
    assert_eq!(jump.requested, MAPPER_X);
    assert!(
        !jump.native_short_hop,
        "raw file inputs never force an engine jump flag"
    );
    for frame in 1..=5 {
        assert_eq!(trap.sample(s, frame, ctx(frame as f32), None).requested, 0);
    }
    assert_eq!(trap.sample(s, 6, ctx(6.0), None).requested, MAPPER_A);
}
#[test]
fn changing_only_the_file_changes_the_start_and_each_inter_action_delay() {
    for (start, wait) in [(0, 0), (1, 1), (2, 2)] {
        let file = Catalog::parse(
            &format!(
                "[kirby.neutral.test]\\nstart_delay={start}\\nactions=short_hop,wait:{wait},nair"
            )
            .replace("\\n", "\n"),
        )
        .unwrap();
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&file).unwrap();
        let p = profiles.profile(6).unwrap();
        let s = MenuSelection::new(&p.menu_options).settings(
            &p.menu_options,
            p.apply(LedgeSettings {
                enabled: true,
                ..Default::default()
            }),
        );
        let mut trap = LedgeTrap::default();
        for frame in 0..=start + wait + 2 {
            let out = trap.sample(s, frame as i64, ctx(frame as f32), Some(target()));
            let expected = if frame == start {
                MAPPER_X
            } else if frame == start + 1 + wait {
                MAPPER_A
            } else {
                0
            };
            assert_eq!(
                out.requested, expected,
                "start={start}, wait={wait}, frame={frame}"
            );
        }
    }
}
