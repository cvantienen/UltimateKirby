//! Execute the production native sampler on the host with copied ABI data and read-only stubs.
#![allow(dead_code, unused_imports)]
use kirby_assist_core::{native_states as native, telemetry::Sample};
use std::{cell::RefCell, sync::Mutex, time::Duration};

#[path = "../../../src/game_ffi.rs"]
mod native_abi;
#[path = "../../../src/live/telemetry.rs"]
mod sampler;
mod performance {
    pub fn ray() {}
}
mod native_params {
    pub const PARAM_GROUND_BRAKE: u64 = 0x0c8cc9db76;
    pub const PARAM_AIR_ACCEL_Y: u64 = 0xba18057d9;
    pub const PARAM_AIR_SPEED_Y_STABLE: u64 = 0x12ec5626fe;
    pub const PARAM_DAMAGE_FLY_TOP_GRAVITY: u64 = 0x1a58828323;
    pub const PARAM_DAMAGE_FLY_TOP_TERMINAL_SPEED: u64 = 0x1d71e21b0b;
}

static SERIAL: Mutex<()> = Mutex::new(());
thread_local! {
    static WORLD: RefCell<World> = RefCell::new(World::new());
}
#[repr(C)]
struct Actor {
    header: u64,
    id: u32,
    status: i32,
    position: [f32; 3],
    velocity: [f32; 2],
    frame: f32,
    stopped: bool,
    damage_stopped: bool,
    active: bool,
    attack: bool,
    geometry: native_abi::AttackGeometry,
}
impl Actor {
    fn new(id: u32) -> Self {
        let mut geometry: native_abi::AttackGeometry = unsafe { std::mem::zeroed() };
        geometry.x = 1.;
        geometry.y = 2.;
        geometry.z = 3.;
        geometry.x2 = 4.;
        geometry.y2 = 5.;
        geometry.z2 = 6.;
        geometry.size = 2.;
        geometry.power = 12.;
        geometry.angle = 45;
        geometry.kbg = 90;
        geometry.fkb = 3;
        geometry.bkb = 20;
        geometry.hitlag = 1.5;
        geometry.sdi = 0.8;
        geometry.slip = 0.25;
        geometry.node = 0x123456789;
        geometry.check_type = 2;
        geometry.situation = 3;
        geometry.capability_padding = [0xff; 17];
        geometry.reflectable = 1;
        geometry.absorbable = 1;
        geometry.direct = 1;
        Self {
            header: 0,
            id,
            status: native::status::WAIT,
            position: [10., 20., 30.],
            velocity: [0.5, -0.75],
            frame: 7.5,
            stopped: false,
            damage_stopped: false,
            active: true,
            attack: false,
            geometry,
        }
    }
    fn object(&mut self) -> *mut native_abi::Object {
        (self as *mut Self).cast()
    }
}
struct World {
    actor: Box<Actor>,
    enabled: bool,
    ready: bool,
    session: u64,
    training: bool,
    training_reads_left: Option<usize>,
    human: bool,
    roster: Option<kirby_assist_core::battle_objects::FightersInGame>,
    reads: usize,
    work_reads: Vec<i32>,
    flag_reads: Vec<i32>,
    samples: Vec<Sample>,
    revoked: usize,
    null_accessor: bool,
    active_id: Option<u32>,
    null_geometry: bool,
    world_geometry: bool,
    invalidate_after_reads: Option<usize>,
}
impl World {
    fn new() -> Self {
        Self {
            actor: Box::new(Actor::new(1)),
            enabled: true,
            ready: true,
            session: 1,
            training: true,
            training_reads_left: None,
            human: true,
            roster: Some({
                let mut ids = [None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY];
                ids[0] = Some(1);
                ids
            }),
            reads: 0,
            work_reads: vec![],
            flag_reads: vec![],
            samples: vec![],
            revoked: 0,
            null_accessor: false,
            active_id: None,
            null_geometry: false,
            world_geometry: true,
            invalidate_after_reads: None,
        }
    }
}
fn with_world<R>(f: impl FnOnce(&mut World) -> R) -> R {
    WORLD.with(|world| f(&mut world.borrow_mut()))
}
fn reset() {
    with_world(|w| *w = World::new());
}
fn observe() {
    // The native sampler's real monotonic throttle is shared across test threads.
    std::thread::sleep(Duration::from_millis(17));
    let object = with_world(|w| w.actor.object());
    unsafe {
        sampler::observe(object);
    }
}
fn base() -> Sample {
    with_world(|w| {
        *w.samples
            .iter()
            .find(|s| s.group == 0)
            .expect("base sample")
    })
}
fn attack() -> Sample {
    with_world(|w| {
        *w.samples
            .iter()
            .find(|s| s.group == 1)
            .expect("attack sample")
    })
}
fn float(s: &Sample, index: usize) -> Option<f32> {
    s.values[index].map(|bits| f32::from_bits(bits as u32))
}
fn human_fighter(_: *mut game::Object) -> bool {
    with_world(|w| w.human)
}
fn fighter_ids() -> Option<kirby_assist_core::battle_objects::FightersInGame> {
    with_world(|w| w.roster)
}

mod diagnostics {
    pub mod telemetry {
        use super::super::{with_world, Sample};
        pub fn enabled() -> bool {
            with_world(|w| w.enabled)
        }
        pub fn ready() -> bool {
            with_world(|w| w.enabled && w.ready)
        }
        pub fn session() -> u64 {
            with_world(|w| w.session)
        }
        pub fn observe(training: bool) {
            if !training {
                revoke();
            }
        }
        pub fn revoke() {
            with_world(|w| {
                w.ready = false;
                w.revoked += 1;
                w.samples.clear();
            });
        }
        pub fn publish(sample: Sample) {
            with_world(|w| w.samples.push(sample));
        }
    }
}
mod game {
    pub use super::native_abi::*;
    use super::{native, with_world, Actor};
    unsafe fn read<R>(o: *mut Object, f: impl FnOnce(&Actor) -> R) -> R {
        assert!(!o.is_null());
        with_world(|w| w.reads += 1);
        f(&*o.cast::<Actor>())
    }
    pub unsafe fn is_training_mode() -> bool {
        with_world(|w| {
            w.reads += 1;
            if let Some(left) = w.training_reads_left.as_mut() {
                if *left == 0 {
                    return false;
                }
                *left -= 1;
            }
            w.training
        })
    }
    pub unsafe fn is_active(id: u32) -> bool {
        with_world(|w| {
            w.active_id.unwrap_or(w.actor.id) == id
                && w.actor.active
                && w.invalidate_after_reads.is_none_or(|limit| w.reads < limit)
        })
    }
    pub unsafe fn module_accessor(id: u32) -> *mut Object {
        with_world(|w| {
            if w.null_accessor || id != w.active_id.unwrap_or(w.actor.id) {
                std::ptr::null_mut()
            } else {
                w.actor.object()
            }
        })
    }
    pub unsafe fn fighter_kind(_: u32) -> i32 {
        6
    }
    pub unsafe fn status_kind(o: *mut Object) -> i32 {
        read(o, |a| a.status)
    }
    pub unsafe fn previous_status(o: *mut Object, index: u32) -> i32 {
        assert_eq!(index, LATEST_PREVIOUS_STATUS_HISTORY_INDEX);
        read(o, |_| -2)
    }
    pub unsafe fn situation_kind(o: *mut Object) -> i32 {
        read(o, |_| 0)
    }
    pub unsafe fn motion_kind(o: *mut Object) -> u64 {
        read(o, |_| 0x123456789abcdef)
    }
    pub unsafe fn motion_frame(o: *mut Object) -> f32 {
        read(o, |a| a.frame)
    }
    pub unsafe fn motion_rate(o: *mut Object) -> f32 {
        read(o, |_| 1.25)
    }
    pub unsafe fn motion_end_frame(o: *mut Object) -> f32 {
        read(o, |_| 40.)
    }
    pub unsafe fn pos_x(o: *mut Object) -> f32 {
        read(o, |a| a.position[0])
    }
    pub unsafe fn pos_y(o: *mut Object) -> f32 {
        read(o, |a| a.position[1])
    }
    pub unsafe fn pos_z(o: *mut Object) -> f32 {
        read(o, |a| a.position[2])
    }
    pub unsafe fn facing(o: *mut Object) -> f32 {
        read(o, |_| -1.)
    }
    pub unsafe fn scale(o: *mut Object) -> f32 {
        read(o, |_| 2.)
    }
    pub unsafe fn speed_x(o: *mut Object, reserve: i32) -> f32 {
        assert_eq!(reserve, ALL_KINETIC_ENERGY_RESERVES);
        read(o, |a| a.velocity[0])
    }
    pub unsafe fn speed_y(o: *mut Object, reserve: i32) -> f32 {
        assert_eq!(reserve, ALL_KINETIC_ENERGY_RESERVES);
        read(o, |a| a.velocity[1])
    }
    pub unsafe fn team(o: *mut Object) -> u64 {
        read(o, |_| 3)
    }
    pub unsafe fn team_owner_id(o: *mut Object) -> u64 {
        read(o, |_| 7)
    }
    pub unsafe fn is_stop(o: *mut Object) -> bool {
        read(o, |a| a.stopped)
    }
    pub unsafe fn is_damage_stop(o: *mut Object) -> bool {
        read(o, |a| a.damage_stopped)
    }
    pub unsafe fn hitstop_remaining(o: *mut Object) -> u64 {
        read(o, |_| 4)
    }
    pub unsafe fn damage_percent(o: *mut Object, slot: i32) -> f32 {
        assert_eq!(slot, 0);
        read(o, |_| 62.5)
    }
    pub unsafe fn hit_status(o: *mut Object, slot: i32) -> u64 {
        assert_eq!(slot, 0);
        read(o, |_| 2)
    }
    pub unsafe fn cancel_enabled(o: *mut Object) -> bool {
        read(o, |_| true)
    }
    pub unsafe fn cancel_frame(o: *mut Object, motion: u64, alternate: bool) -> f32 {
        assert_eq!(motion, 0x123456789abcdef);
        assert!(!alternate);
        read(o, |_| 20.)
    }
    pub unsafe fn normal_hit_frame(o: *mut Object, _: u64, alternate: bool) -> f32 {
        assert!(!alternate);
        read(o, |_| 16.)
    }
    pub unsafe fn passable_ground(o: *mut Object) -> bool {
        read(o, |_| true)
    }
    pub unsafe fn is_link(o: *mut Object, slot: i32) -> bool {
        assert!(slot == 0 || slot == 2);
        read(o, |_| slot == 2)
    }
    pub unsafe fn clatter_time(o: *mut Object, slot: i32) -> f32 {
        assert_eq!(slot, 0);
        read(o, |_| 9.)
    }
    pub unsafe fn param_float(o: *mut Object, key: u64, field: u64) -> f32 {
        if field != 0 {
            assert_eq!(key, kirby_assist_core::timing::motion_hash("common"));
            assert!([
                kirby_assist_core::timing::motion_hash("walk_stick_x"),
                kirby_assist_core::timing::motion_hash("dash_stick_x")
            ]
            .contains(&field));
        } else {
            assert!([
                super::native_params::PARAM_AIR_ACCEL_Y,
                super::native_params::PARAM_AIR_SPEED_Y_STABLE,
                super::native_params::PARAM_GROUND_BRAKE,
                super::native_params::PARAM_DAMAGE_FLY_TOP_GRAVITY,
                super::native_params::PARAM_DAMAGE_FLY_TOP_TERMINAL_SPEED,
            ]
            .contains(&key));
        }
        read(o, |_| 0.125)
    }
    pub unsafe fn link_node_object_id(o: *mut Object, slot: i32) -> u64 {
        assert_eq!(slot, 0);
        read(o, |_| 123)
    }
    pub unsafe fn work_float(o: *mut Object, key: i32) -> f32 {
        with_world(|w| w.work_reads.push(key));
        read(o, |_| 13.5)
    }
    pub unsafe fn work_int(o: *mut Object, key: i32) -> i32 {
        with_world(|w| w.work_reads.push(key));
        read(o, |_| 8)
    }
    pub unsafe fn is_flag(o: *mut Object, key: i32) -> bool {
        with_world(|w| w.flag_reads.push(key));
        read(o, |_| true)
    }
    pub unsafe fn transition_enabled(o: *mut Object, _: i32) -> bool {
        read(o, |_| true)
    }
    pub unsafe fn check_button_on(o: *mut Object, button: i32) -> bool {
        assert!(button == 0x3 || button == 0x10);
        read(o, |_| button == 0x3)
    }
    pub unsafe fn attack_infliction_now(o: *mut Object, mask: i32) -> bool {
        assert_eq!(mask, COLLISION_KIND_MASK_HIT);
        read(o, |_| true)
    }
    pub unsafe fn attack_active(o: *mut Object, slot: i32, alternate: bool) -> bool {
        assert!(!alternate);
        assert!((0..8).contains(&slot));
        read(o, |a| a.attack && slot == 0)
    }
    pub unsafe fn attack_geometry(
        o: *mut Object,
        slot: i32,
        alternate: bool,
    ) -> *const AttackGeometry {
        assert_eq!(slot, 0);
        assert!(!alternate);
        if with_world(|w| w.null_geometry) {
            std::ptr::null()
        } else {
            read(o, |a| &a.geometry as *const _)
        }
    }
}
mod hitboxes {
    pub struct WorldGeometry {
        pub start: [f32; 3],
        pub end: [f32; 3],
        pub radius: f32,
    }
    pub unsafe fn world_geometry(
        _: *mut super::game::Object,
        data: super::game::AttackGeometry,
        scale: f32,
        velocity: [f32; 2],
    ) -> Option<WorldGeometry> {
        // Opaque capability bytes must never be forwarded even to the geometry adapter.
        assert_eq!(data.capability_padding, [0; 17]);
        assert_eq!((data.reflectable, data.absorbable, data.direct), (0, 0, 0));
        assert_eq!(scale, 2.);
        if !super::with_world(|w| w.world_geometry) {
            return None;
        }
        Some(WorldGeometry {
            start: [
                data.x * scale + velocity[0],
                data.y * scale + velocity[1],
                data.z * scale,
            ],
            end: [data.x2 * scale, data.y2 * scale, data.z2 * scale],
            radius: data.size * scale,
        })
    }
}

#[test]
fn copies_basic_native_values_without_assist_configuration() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    observe();
    let sample = base();
    assert_eq!((sample.object_id, sample.category, sample.kind), (1, 0, 6));
    assert_eq!(sample.values[1], Some((-2i64) as u64));
    assert_eq!(sample.values[3], Some(0x123456789abcdef));
    for (index, expected) in [
        (4, 7.5),
        (5, 1.25),
        (6, 40.),
        (7, 10.),
        (8, 20.),
        (9, 30.),
        (10, -1.),
        (11, 2.),
        (12, 0.5),
        (13, -0.75),
        (19, 62.5),
    ] {
        assert_eq!(float(&sample, index), Some(expected));
    }
    assert_eq!(sample.values[14], Some(3));
    assert_eq!(sample.values[15], Some(7));
    assert_eq!(sample.values[18], Some(0));
    assert_eq!(sample.values[25], Some(0));
    assert_eq!(sample.values[26], Some(1));
    with_world(|w| w.actor.position[0] = 99.);
    assert_eq!(float(&sample, 7), Some(10.));
    assert!(with_world(|w| w.work_reads
        == [
            native::work_ids::FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID
        ]
        && w.flag_reads.is_empty()));
    assert_eq!(float(&sample, 54), Some(0.125));
    assert_eq!(sample.values[56], None);
    assert_eq!(sample.values[57], Some(8));
}

#[test]
fn nonfinite_values_and_missing_geometry_remain_unknown() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    with_world(|w| {
        w.actor.frame = f32::NAN;
        w.actor.position[0] = f32::INFINITY;
        w.actor.velocity[1] = f32::NEG_INFINITY;
        w.actor.attack = true;
        w.null_geometry = true;
    });
    observe();
    let sample = base();
    assert_eq!(sample.values[4], None);
    assert_eq!(sample.values[7], None);
    assert_eq!(sample.values[13], None);
    assert_eq!(float(&sample, 8), Some(20.));
    let sample = attack();
    assert_eq!(sample.values[0], Some(1));
    assert!(sample.values[1..].iter().all(Option::is_none));
}

#[test]
fn policy_publication_is_bounded_read_only_and_requires_an_active_lease() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    let snapshot = kirby_assist_core::diagnostics::modes::Snapshot {
        winner: None,
        present: 0,
        committed: false,
        requested: 0,
        stick: None,
        status: 0,
        motion: 0,
        native_frame: 0,
        stopped: false,
        can_act: false,
        hit: None,
        valid: false,
        safety: 0,
    };
    with_world(|w| {
        w.ready = false;
        w.session = 1234;
    });
    sampler::publish_policy(42, 6, snapshot, [None; 18]);
    with_world(|w| {
        assert!(w.samples.is_empty());
        w.ready = true;
    });
    sampler::publish_policy(42, 6, snapshot, [None; 18]);
    sampler::publish_policy(42, 6, snapshot, [None; 18]);
    with_world(|w| {
        assert_eq!(w.samples.len(), 1);
        assert_eq!(
            w.samples[0].group,
            kirby_assist_core::telemetry::POLICY_GROUP
        );
        assert_eq!(w.reads, 0);
        w.session += 1;
    });
    sampler::publish_policy(42, 6, snapshot, [None; 18]);
    with_world(|w| assert_eq!(w.samples.len(), 2));
}

#[test]
fn feature_detail_batches_are_throttled_and_revocation_blocks_publication() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    with_world(|w| {
        w.session = 9876;
        w.ready = false;
    });
    assert!(!sampler::feature_details_due());
    with_world(|w| w.ready = true);
    assert!(sampler::feature_details_due());
    assert!(!sampler::feature_details_due());
    let details =
        [kirby_assist_core::telemetry::FeatureDetail::new(Some(true), "Held", "Example"); 18];
    sampler::publish_feature_details(42, 6, details);
    with_world(|w| {
        assert_eq!(w.samples.len(), 18);
        assert_eq!(w.reads, 0);
        w.ready = false;
    });
    sampler::publish_feature_details(42, 6, details);
    with_world(|w| {
        assert_eq!(w.samples.len(), 18);
        w.ready = true;
        w.session += 1;
    });
    assert!(sampler::feature_details_due());
}

#[test]
fn work_reads_are_scoped_to_damage_guard_cliff_and_tumble_states() {
    let _serial = SERIAL.lock().unwrap();
    use native::work_ids::*;
    let cases = [
        (native::status::WAIT, true, false, vec![], vec![]),
        (
            native::status::WAIT,
            true,
            true,
            vec![
                FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X,
                FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y,
            ],
            vec![],
        ),
        (
            native::GUARD_DAMAGE,
            false,
            false,
            vec![FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME],
            vec![],
        ),
        (
            native::status::CLIFF_CATCH,
            false,
            false,
            vec![FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME],
            vec![],
        ),
        (
            native::status::CLIFF_WAIT,
            false,
            false,
            vec![FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME],
            vec![],
        ),
        (
            native::status::DAMAGE_FLY,
            false,
            false,
            vec![FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME],
            vec![FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE],
        ),
    ];
    for (status, stopped, damage, work, flags) in cases {
        reset();
        with_world(|w| {
            w.actor.status = status;
            w.actor.stopped = stopped;
            w.actor.damage_stopped = damage;
        });
        observe();
        with_world(|w| {
            let mut expected = work;
            expected.push(FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID);
            assert_eq!(w.work_reads, expected, "status {status}");
            assert_eq!(w.flag_reads, flags);
        });
        let sample = base();
        assert_eq!(
            sample.values[18],
            Some(if stopped && damage { 4 } else { 0 })
        );
        assert_eq!(sample.values[34].is_some(), stopped && damage);
        assert_eq!(sample.values[37].is_some(), status == native::GUARD_DAMAGE);
        assert_eq!(
            sample.values[38].is_some(),
            [native::status::CLIFF_CATCH, native::status::CLIFF_WAIT].contains(&status)
        );
        assert_eq!(
            sample.values[39].is_some(),
            status == native::status::DAMAGE_FLY
        );
    }
}

#[test]
fn attack_groups_copy_verified_scalars_and_optional_world_geometry_only() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    with_world(|w| w.actor.attack = true);
    observe();
    assert_eq!(base().values[30], Some(1));
    let sample = attack();
    assert_eq!(sample.values[1], Some(0x123456789));
    for (index, expected) in [
        (2, 1.),
        (3, 2.),
        (4, 3.),
        (5, 4.),
        (6, 5.),
        (7, 6.),
        (8, 2.),
        (9, 12.),
        (14, 1.5),
        (15, 0.8),
        (17, 0.25),
        (25, 2.5),
        (26, 3.25),
        (27, 6.),
        (28, 8.),
        (29, 10.),
        (30, 12.),
        (31, 4.),
    ] {
        assert_eq!(float(&sample, index), Some(expected));
    }
    assert_eq!(
        sample.values[10..14],
        [Some(45), Some(90), Some(3), Some(20)]
    );
    assert_eq!(sample.values[23..25], [Some(2), Some(3)]);
    assert!(sample.values[18..23].iter().all(Option::is_none));
    with_world(|w| {
        for sample in w.samples.iter().filter(|s| (2..=8).contains(&s.group)) {
            assert_eq!(sample.values[0], Some(0));
            assert!(sample.values[1..].iter().all(Option::is_none));
        }
    });
    reset();
    with_world(|w| {
        w.actor.attack = true;
        w.world_geometry = false;
    });
    observe();
    assert!(attack().values[25..32].iter().all(Option::is_none));
    assert_eq!(float(&attack(), 9), Some(12.));
}

#[test]
fn rejects_invalid_objects_and_revokes_on_roster_loss_or_training_exit() {
    let _serial = SERIAL.lock().unwrap();
    for invalid in 0..5 {
        reset();
        with_world(|w| match invalid {
            0 => w.actor.active = false,
            1 => w.null_accessor = true,
            2 => {
                w.roster.as_mut().unwrap()[0] = Some(2);
                w.active_id = Some(2);
            }
            3 => w.invalidate_after_reads = Some(10),
            _ => {
                w.actor.id = 0x30000001;
                w.roster.as_mut().unwrap()[0] = Some(w.actor.id);
            }
        });
        observe();
        assert!(
            with_world(|w| w.samples.is_empty()),
            "invalid case {invalid}"
        );
    }
    reset();
    with_world(|w| w.roster = None);
    observe();
    assert_eq!(with_world(|w| w.revoked), 1);
    reset();
    observe();
    assert!(!with_world(|w| w.samples.is_empty()));
    with_world(|w| w.training = false);
    observe();
    assert_eq!(with_world(|w| w.revoked), 1);
    assert!(!with_world(|w| w.ready));
    assert!(with_world(|w| w.samples.is_empty()));
}

#[test]
fn remembered_projectiles_sample_geometry_without_fighter_work_and_clear_on_training_exit() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    let weapon_id = (kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON << 28) | 5;
    with_world(|w| {
        w.actor = Box::new(Actor::new(weapon_id));
        w.actor.status = native::GUARD_DAMAGE;
        w.actor.stopped = true;
        w.actor.damage_stopped = true;
        w.actor.attack = true;
        w.roster = Some([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    });
    let object = with_world(|w| w.actor.object());
    unsafe {
        sampler::remember_attack(object);
        sampler::remember_attack(object);
    }
    observe();
    let sample = base();
    assert_eq!(sample.object_id, weapon_id);
    assert_eq!(
        sample.category,
        kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
    );
    assert!(sample.values[17..28].iter().all(Option::is_none));
    assert!(with_world(
        |w| w.work_reads.is_empty() && w.flag_reads.is_empty()
    ));
    assert_eq!(float(&attack(), 9), Some(12.));
    assert_eq!(
        with_world(|w| w.samples.iter().filter(|s| s.group == 0).count()),
        1
    );
    with_world(|w| w.training = false);
    observe();
    with_world(|w| {
        w.training = true;
        w.ready = true;
    });
    observe();
    assert!(with_world(|w| w.samples.is_empty()));
}

#[test]
fn samples_the_last_validated_partner_roster_slot() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    with_world(|w| {
        let mut ids = [None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY];
        ids[ids.len() - 1] = Some(w.actor.id);
        w.roster = Some(ids);
    });
    observe();
    assert_eq!(base().object_id, 1);
}

#[test]
fn training_exit_during_sampling_revokes_before_more_publication() {
    let _serial = SERIAL.lock().unwrap();
    for reads in [1, 2] {
        reset();
        with_world(|w| w.training_reads_left = Some(reads));
        observe();
        assert_eq!(with_world(|w| w.revoked), 1);
        assert!(with_world(|w| w.samples.is_empty()));
        assert!(!with_world(|w| w.ready));
    }
}

#[test]
fn new_session_does_not_reuse_remembered_projectiles() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    let id = (kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON << 28) | 7;
    with_world(|w| {
        w.actor = Box::new(Actor::new(id));
        w.actor.attack = true;
        w.roster = Some([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    });
    let object = with_world(|w| w.actor.object());
    unsafe { sampler::remember_attack(object) };
    observe();
    assert_eq!(base().object_id, id);
    with_world(|w| {
        w.samples.clear();
        w.session += 1;
    });
    observe();
    assert!(with_world(|w| w.samples.is_empty()));
    unsafe { sampler::remember_attack(object) };
    observe();
    assert_eq!(base().object_id, id);
}

#[test]
fn disabled_transport_performs_no_native_reads_and_null_or_nonhuman_callbacks_do_not_sample() {
    let _serial = SERIAL.lock().unwrap();
    reset();
    with_world(|w| w.enabled = false);
    observe();
    assert_eq!(with_world(|w| w.reads), 0);
    assert!(with_world(|w| w.samples.is_empty()));
    reset();
    unsafe {
        sampler::observe(std::ptr::null_mut());
    }
    assert!(with_world(|w| w.samples.is_empty()));
    reset();
    with_world(|w| w.human = false);
    observe();
    assert!(with_world(|w| w.samples.is_empty()));
    reset();
    with_world(|w| w.ready = false);
    observe();
    assert!(with_world(|w| w.samples.is_empty()));
}
