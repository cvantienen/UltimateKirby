use kirby_assist_core::{
    combat_assist::{TripContext, TripFollowup, TripTarget},
    native_input::MAPPER_A,
};
fn context(tilt: i32, target: i32, previous: i32) -> TripContext {
    let mut targets = [None; 8];
    targets[0] = Some(TripTarget {
        id: 2,
        status: target,
        previous_status: previous,
    });
    TripContext {
        object: 1,
        kirby: true,
        status: tilt,
        motion: 123,
        frame: 10.,
        grounded: true,
        facing: -1.,
        stopped: false,
        targets,
    }
}
#[test]
fn down_tilt_recognizes_trip_rehit_damage_and_buffers_forward_smash() {
    for (status, previous) in [
        (0x86, 0),
        (0x87, 0x88),
        (0x88, 0x86),
        (0x47, 0x88),
        (0x5a, 0x87),
    ] {
        let c = context(0x2c, status, previous);
        let mut tool = TripFollowup::default();
        let prime = tool.sample(true, c);
        assert_eq!(prime.requested, 0);
        assert_ne!(prime.owned, 0);
        assert_eq!(
            tool.sample(true, c),
            prime,
            "duplicate observations do not spend release"
        );
        let press = tool.sample(true, TripContext { frame: 11., ..c });
        assert_eq!(press.requested, MAPPER_A);
        assert_eq!(press.stick, Some([-1., 0.]));
        let accepted = tool.sample(
            true,
            TripContext {
                status: 0x2d,
                frame: 0.,
                ..c
            },
        );
        assert_eq!(accepted.requested, 0);
        assert!(!tool.watching());
    }
}
#[test]
fn new_down_tilt_rearms_even_when_opponent_never_leaves_trip_wait() {
    let c = context(0x2c, 0x88, 0x86);
    let mut tool = TripFollowup::default();
    tool.sample(true, c);
    tool.sample(true, TripContext { frame: 11., ..c });
    tool.sample(
        true,
        TripContext {
            status: 0x2d,
            frame: 0.,
            ..c
        },
    );
    tool.sample(
        true,
        TripContext {
            status: 0,
            frame: 0.,
            ..c
        },
    );
    tool.sample(true, TripContext { frame: 1., ..c });
    let retrip = tool.sample(true, TripContext { frame: 2., ..c });
    assert_eq!(retrip.requested, MAPPER_A);
    assert_eq!(retrip.stick, Some([-1., 0.]));
}
#[test]
fn forward_tilt_keeps_forward_smash_and_unrelated_damage_is_not_a_trip() {
    let c = context(0x2a, 0x88, 0);
    let mut tool = TripFollowup::default();
    tool.sample(true, c);
    assert_eq!(
        tool.sample(true, TripContext { frame: 11., ..c }).stick,
        Some([-1., 0.])
    );
    assert_eq!(
        TripFollowup::default()
            .sample(true, context(0x2c, 0x47, 0))
            .owned,
        0
    );
    assert_eq!(
        TripFollowup::default()
            .sample(true, context(0x2c, 0x48, 0x88))
            .owned,
        0
    );
    assert_eq!(tool.sample(false, c).owned, 0);
}
