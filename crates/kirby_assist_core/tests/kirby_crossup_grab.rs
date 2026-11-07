//! Cross-up is now a file-authored spatial tape, not a compiled Kirby option.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    gameplay::{GameplayContext, Opponent},
    ledge_menu::MenuSelection,
    ledge_trap::{LedgeSettings, LedgeTrap, CLIFF_CLIMB},
    native_input::MAPPER_R,
    sequence_config::Catalog,
};
fn context(frame: f32, x: f32, facing: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.0,
        position: [x, 0.0],
        facing,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
#[test]
fn file_crossup_uses_authored_distance_then_observed_turn_before_one_grab() {
    let file=Catalog::parse("[kirby.neutral.crossup_grab]\nstart_delay=0\ndefault=true\nactions=walk:facing:22:80%,face:opponent,input:r:0:0:1\n").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    let p = profiles.profile(6).unwrap();
    let mut menu = MenuSelection::new(&p.menu_options);
    menu.choices[0] = p.menu_options.lists[0]
        .options
        .iter()
        .position(|c| c.id == "crossup_grab")
        .unwrap();
    let settings = menu.settings(
        &p.menu_options,
        p.apply(LedgeSettings {
            enabled: true,
            ..Default::default()
        }),
    );
    let enemy = Opponent {
        id: 2,
        status: CLIFF_CLIMB,
        position: [5.0, 0.0],
        ..Default::default()
    };
    let mut trap = LedgeTrap::default();
    assert_eq!(
        trap.sample(settings, 0, context(0.0, 0.0, 1.0), Some(enemy))
            .stick,
        Some([0.8, 0.0])
    );
    assert_eq!(
        trap.sample(settings, 1, context(1.0, 21.0, 1.0), Some(enemy))
            .requested,
        0
    );
    assert_eq!(
        trap.sample(settings, 2, context(2.0, 22.0, 1.0), Some(enemy))
            .stick,
        Some([-0.3, 0.0])
    );
    assert_eq!(
        trap.sample(settings, 3, context(3.0, 22.0, -1.0), Some(enemy))
            .requested,
        MAPPER_R
    );
    assert_eq!(
        trap.sample(settings, 4, context(4.0, 22.0, -1.0), None)
            .requested,
        0
    );
    assert!(trap.manual_after_response());
}
