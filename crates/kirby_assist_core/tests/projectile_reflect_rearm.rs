//! Real native source-cache + reflector lifecycle regressions. No game writes.
#[path = "../../../src/live/projectile_sources.rs"]
mod projectile_sources;
use kirby_assist_core::{
    fighter_tuning::Tuning,
    native_input::MAPPER_B,
    projectile_reflect::{Config, Context, ProjectileKey, Reflector, Threat},
    timing::AnimationSample,
};
use projectile_sources::Sources;
const SHOT: u32 = 0x10000021;
fn config() -> Config {
    let mut tuning = Tuning::default();
    tuning.set("reflect_default", "true").unwrap();
    Config::for_fighter(28, tuning).unwrap()
}
fn own(status: i32, gauge: f32) -> Context {
    Context {
        kind: 28,
        situation: 0,
        facing: 1.,
        can_special: true,
        gauge,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status,
            kind: status as u64 + 1,
            frame: 0.,
            auxiliary: 0.,
        },
    }
}
fn incoming(key: ProjectileKey) -> Option<Threat> {
    Some(Threat {
        id: key.id,
        generation: key.generation,
        in_frames: 7.,
        direction: 1.,
    })
}
fn shot(sources: &Sources) -> ProjectileKey {
    sources.remember(SHOT);
    sources
        .snapshot()
        .into_iter()
        .flatten()
        .find(|key| key.id == SHOT)
        .unwrap()
}
fn complete(engine: &mut Reflector, key: ProjectileKey, status: i32, gauge: f32) {
    engine.mapped(i64::from(key.generation));
    assert_eq!(
        engine
            .sample(true, Some(config()), own(0x1e3, gauge), incoming(key))
            .requested,
        MAPPER_B
    );
    assert_eq!(
        engine
            .sample(true, Some(config()), own(status, gauge), incoming(key))
            .owned,
        0
    );
    assert!(!engine.active());
}

#[test]
fn repeated_samus_missile_articles_rearm_on_the_first_legal_observation_without_cooldown() {
    let sources = Sources::new();
    let mut engine = Reflector::new();
    for _ in 0..80 {
        let key = shot(&sources);
        assert_eq!(
            sources.snapshot().iter().flatten().count(),
            1,
            "repeated attack does not fill/rotate the source ring"
        );
        let output = engine.sample(true, Some(config()), own(0, 0.), incoming(key));
        assert_eq!(
            output.requested, MAPPER_B,
            "fresh shot {} must not wait for 32 other IDs",
            key.generation
        );
        assert_eq!(output.stick, Some([0., -1.]));
        complete(&mut engine, key, 0x1e6, 0.);
        assert_eq!(
            engine
                .sample(true, Some(config()), own(0, 0.), incoming(key))
                .owned,
            0,
            "a duplicate of the already reflected attack remains blocked"
        );
    }
}

#[test]
fn successive_charge_shots_rearm_with_partial_gauge_and_full_bucket_never_dumps_oil() {
    let sources = Sources::new();
    let mut engine = Reflector::new();
    for gauge in [0., 1., 2.] {
        let key = shot(&sources);
        let mut recovery = own(0, gauge);
        recovery.can_special = false;
        assert_eq!(
            engine
                .sample(true, Some(config()), recovery, incoming(key))
                .owned,
            0
        );
        assert_eq!(
            engine
                .sample(true, Some(config()), own(0, gauge), incoming(key))
                .requested,
            MAPPER_B,
            "no extra rearm frame once the game's transition is legal"
        );
        complete(&mut engine, key, 0x1e5, gauge);
    }
    let key = shot(&sources);
    assert_eq!(
        engine
            .sample(true, Some(config()), own(0, 3.), incoming(key))
            .owned,
        0
    );
    assert_eq!(
        engine
            .sample(true, Some(config()), own(0, 0.), incoming(key))
            .requested,
        MAPPER_B,
        "manually emptying Bucket makes the untouched new shot immediately eligible"
    );
}

#[test]
fn tracker_retirement_and_reuse_have_fresh_generations_without_static_history_collisions() {
    let sources = Sources::new();
    let first = shot(&sources);
    sources.forget(SHOT);
    assert!(sources.snapshot().iter().all(Option::is_none));
    let second = shot(&sources);
    assert_ne!(first, second);
    sources.clear();
    let third = shot(&sources);
    assert_ne!(second, third);
    sources.remember(1);
    assert_eq!(
        sources.snapshot().iter().flatten().count(),
        1,
        "melee cannot enter the ring"
    );
    for index in 0..50 {
        sources.remember(0x10000100 + index);
    }
    assert_eq!(sources.snapshot().iter().flatten().count(), 32);
    let old = sources.snapshot();
    sources.forget(old[10].unwrap().id);
    sources.remember(0x40000100);
    let current = sources.snapshot();
    assert_eq!(current.iter().flatten().count(), 32);
    assert!(
        old.iter()
            .flatten()
            .filter(|key| key.id != old[10].unwrap().id)
            .all(|key| current.contains(&Some(*key))),
        "reuse vacant slots before evicting a live projectile"
    );
}

#[test]
fn a_fresh_attack_on_the_tracked_object_does_not_restart_an_inflight_bucket() {
    let sources = Sources::new();
    let mut engine = Reflector::new();
    let first = shot(&sources);
    engine.sample(true, Some(config()), own(0, 0.), incoming(first));
    engine.mapped(1);
    let next = shot(&sources);
    assert_eq!(
        engine
            .sample(true, Some(config()), own(0x1e3, 0.), incoming(next))
            .requested,
        MAPPER_B
    );
    assert_eq!(engine.target(), Some(SHOT));
    engine.sample(true, Some(config()), own(0x1e6, 0.), incoming(next));
    assert_eq!(
        engine
            .sample(true, Some(config()), own(0, 0.), incoming(first))
            .owned,
        0
    );
    assert_eq!(
        engine
            .sample(true, Some(config()), own(0, 0.), incoming(next))
            .requested,
        MAPPER_B
    );
}
