use super::*;

#[test]
fn trajectory_cache_tracks_position_and_survival_window() {
    let settings = Settings::default();
    let context = Context {
        object: 1,
        damage_stop: true,
        remaining: 2,
        launch: [8., 0.5],
        outward: 1.,
    };
    let blast = BlastZone {
        left: -100.,
        right: 100.,
        bottom: -100.,
        top: 100.,
    };
    let first = Trajectory {
        position: [80., 0.],
        blast,
        hitstun: 10.,
        gravity: 0.1,
        fall_speed: 2.,
    };
    let mut di = AutoDi::default();
    assert_eq!(
        di.cached_trajectory_decision(settings, context, first),
        Some(choose_trajectory(settings, context, first).unwrap())
    );
    let moved = Trajectory {
        position: [-80., 0.],
        ..first
    };
    assert_ne!(
        choose_trajectory(settings, context, first).unwrap().stick,
        choose_trajectory(settings, context, moved).unwrap().stick
    );
    assert_eq!(
        di.cached_trajectory_decision(settings, context, moved),
        choose_trajectory(settings, context, moved)
    );
    let shorter = Trajectory {
        hitstun: 2.,
        ..moved
    };
    assert_eq!(
        di.cached_trajectory_decision(settings, context, shorter),
        choose_trajectory(settings, context, shorter)
    );
    let reversed = Context {
        outward: -1.,
        ..context
    };
    assert_eq!(
        di.cached_trajectory_decision(settings, reversed, shorter),
        choose_trajectory(settings, reversed, shorter)
    );
}
