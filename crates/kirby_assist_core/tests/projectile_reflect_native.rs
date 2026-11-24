//! Compile the production read-only adapter and its real ABI prefix on the host.
#![allow(dead_code, unexpected_cfgs)]
use std::cell::{Cell, RefCell};
mod performance {
    pub fn ray() {}
}
#[path = "../../../src/game_ffi.rs"]
mod abi;
thread_local! {
    static OBJECTS:RefCell<Vec<Box<game_ffi::Object>>>=const{RefCell::new(Vec::new())};
    static READS:Cell<usize>=const{Cell::new(0)};
    static GEOMETRY:Cell<usize>=const{Cell::new(0)};
    static JOINTS:Cell<usize>=const{Cell::new(0)};
}
fn read() {
    READS.with(|v| v.set(v.get() + 1));
}
mod game_ffi {
    pub use super::abi::{AttackGeometry, Vector3};
    use super::*;
    #[repr(C)]
    pub struct Object {
        pub header: u64,
        pub id: u32,
        pub generation: u32,
        pub kind: i32,
        pub status: i32,
        pub situation: i32,
        pub stopped: bool,
        pub linked: bool,
        pub ready: bool,
        pub gauge: f32,
        pub facing: f32,
        pub position: [f32; 3],
        pub velocity: [f32; 2],
        pub team: u64,
        pub scale: f32,
        pub active: bool,
        pub attacks: bool,
        pub geometry: AttackGeometry,
        pub motion: u64,
        pub frame: f32,
        pub rate: f32,
    }
    impl Object {
        pub fn new(id: u32) -> Self {
            let mut geometry: AttackGeometry = unsafe { std::mem::zeroed() };
            geometry.y = 8.;
            geometry.size = 1.;
            geometry.situation = kirby_assist_core::native_states::situation::ATTACK_GROUND_MASK
                | kirby_assist_core::native_states::situation::ATTACK_AIR_MASK;
            geometry.reflectable = 1;
            Self {
                header: 0,
                id,
                generation: 1,
                kind: 0,
                status: kirby_assist_core::native_states::status::WAIT,
                situation: kirby_assist_core::native_states::situation::GROUND,
                stopped: false,
                linked: false,
                ready: true,
                gauge: 0.,
                facing: 1.,
                position: [0.; 3],
                velocity: [0.; 2],
                team: 1,
                scale: 1.,
                active: true,
                attacks: true,
                geometry,
                motion: 1,
                frame: 0.,
                rate: 1.,
            }
        }
    }
    pub unsafe fn is_active(id: u32) -> bool {
        read();
        let p = module_accessor(id);
        !p.is_null() && (*p).active
    }
    pub unsafe fn module_accessor(id: u32) -> *mut Object {
        read();
        OBJECTS.with(|v| {
            v.borrow()
                .iter()
                .find(|o| o.id == id)
                .map_or(std::ptr::null_mut(), |o| (&**o as *const Object).cast_mut())
        })
    }
    pub unsafe fn battle_object_id(object: *mut Object) -> u32 {
        (*object).id
    }
    pub unsafe fn work_float(o: *mut Object, key: i32) -> f32 {
        read();
        assert_eq!(key, 0x4c);
        (*o).gauge
    }
    pub unsafe fn transition_enabled(o: *mut Object, key: i32) -> bool {
        read();
        assert_eq!(
            key,
            kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_LW
        );
        (*o).ready
    }
    pub unsafe fn scale(o: *mut Object) -> f32 {
        read();
        (*o).scale
    }
    pub unsafe fn pos_z(o: *mut Object) -> f32 {
        read();
        (*o).position[2]
    }
    pub unsafe fn attack_active(o: *mut Object, _: i32, _: bool) -> bool {
        read();
        (*o).attacks
    }
    pub unsafe fn attack_geometry(o: *mut Object, _: i32, _: bool) -> *const AttackGeometry {
        GEOMETRY.with(|v| v.set(v.get() + 1));
        &(*o).geometry
    }
    pub unsafe fn joint_position(
        o: *mut Object,
        _: u64,
        local: *const Vector3,
        out: *mut Vector3,
        _: bool,
    ) {
        JOINTS.with(|v| v.set(v.get() + 1));
        (*out).x = (*o).position[0] + (*local).x;
        (*out).y = (*o).position[1] + (*local).y;
        (*out).z = (*o).position[2] + (*local).z;
    }
}
mod frame {
    use super::*;
    pub struct FrameReads;
    impl FrameReads {
        pub unsafe fn fighter_kind(&self, o: *mut game_ffi::Object) -> i32 {
            read();
            (*o).kind
        }
        pub unsafe fn status_kind(&self, o: *mut game_ffi::Object) -> i32 {
            read();
            (*o).status
        }
        pub unsafe fn situation_kind(&self, o: *mut game_ffi::Object) -> i32 {
            read();
            (*o).situation
        }
        pub unsafe fn capture_link(&self, o: *mut game_ffi::Object) -> bool {
            read();
            (*o).linked
        }
        pub unsafe fn is_stop(&self, o: *mut game_ffi::Object) -> bool {
            read();
            (*o).stopped
        }
        pub unsafe fn facing(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).facing
        }
        pub unsafe fn motion_kind(&self, o: *mut game_ffi::Object) -> u64 {
            read();
            (*o).motion
        }
        pub unsafe fn motion_frame(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).frame
        }
        pub unsafe fn motion_rate(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).rate
        }
        pub fn roster(&self) -> Option<kirby_assist_core::battle_objects::FightersInGame> {
            Some(OBJECTS.with(|v| std::array::from_fn(|i| v.borrow().get(i).map(|o| o.id))))
        }
        pub unsafe fn pos_x(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).position[0]
        }
        pub unsafe fn pos_y(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).position[1]
        }
        pub unsafe fn speed_x(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).velocity[0]
        }
        pub unsafe fn speed_y(&self, o: *mut game_ffi::Object) -> f32 {
            read();
            (*o).velocity[1]
        }
        pub unsafe fn team(&self, o: *mut game_ffi::Object) -> u64 {
            read();
            (*o).team
        }
    }
}
mod attack_sources {
    pub fn forget(_: u32) {}
    pub fn projectiles() -> [Option<kirby_assist_core::projectile_reflect::ProjectileKey>; 32] {
        use kirby_assist_core::projectile_reflect::ProjectileKey;
        super::OBJECTS.with(|v| {
            std::array::from_fn(|i| {
                v.borrow().get(i).map(|o| ProjectileKey {
                    id: o.id,
                    generation: o.generation,
                })
            })
        })
    }
}
#[path = "../../../src/live/reflect.rs"]
mod reflect;
#[path = "../../../src/live/counter.rs"]
mod counter;
use kirby_assist_core::{fighter_tuning::Tuning, projectile_reflect::Config};
fn config() -> Config {
    let mut t = Tuning::default();
    t.set("reflect_default", "true").unwrap();
    Config::for_fighter(28, t).unwrap()
}
fn reset() -> Box<game_ffi::Object> {
    OBJECTS.with(|v| v.borrow_mut().clear());
    READS.with(|v| v.set(0));
    GEOMETRY.with(|v| v.set(0));
    JOINTS.with(|v| v.set(0));
    Box::new(game_ffi::Object::new(1))
}
fn projectile(edit: impl FnOnce(&mut game_ffi::Object)) {
    let mut p = Box::new(game_ffi::Object::new(0x10000002));
    p.team = 2;
    p.position[0] = 41.;
    p.velocity[0] = -5.;
    edit(&mut p);
    OBJECTS.with(|v| v.borrow_mut().push(p));
}
fn counter_config() -> kirby_assist_core::auto_counter::Config {
    kirby_assist_core::auto_counter::Config {
        startup: 5., input_delay: 0., margin: 2.,
        half_width: 5., height: 16., half_depth: 5.,
    }
}
fn counter_enemy(edit: impl FnOnce(&mut game_ffi::Object)) {
    let mut enemy = Box::new(game_ffi::Object::new(7));
    enemy.kind = kirby_assist_core::fighter_kinds::MARIO;
    enemy.team = 2;
    enemy.status = kirby_assist_core::native_states::status::ATTACK_AIR;
    enemy.motion = kirby_assist_core::timing::motion_hash("attack_air_n");
    enemy.frame = 6.;
    enemy.position[0] = 12.;
    enemy.velocity[0] = -1.;
    enemy.geometry.direct = 1;
    enemy.geometry.power = 10.;
    edit(&mut enemy);
    OBJECTS.with(|v| v.borrow_mut().push(enemy));
}
#[test]
fn counter_native_permission_is_required_in_movement_and_air() {
    use kirby_assist_core::native_states::{status, situation};
    for (state, air) in [(status::WAIT, false), (status::WALK, false),
        (status::DASH, false), (status::RUN, false), (status::TURN_RUN_BRAKE, false),
        (status::JUMP_SQUAT, false), (status::LANDING, false),
        (status::JUMP, true), (status::FALL, true)] {
        let mut own = reset();
        own.status = state;
        own.situation = if air { situation::AIR } else { situation::GROUND };
        own.ready = false;
        counter_enemy(|_| {});
        let (_, context, threat) = unsafe {
            counter::observe(&frame::FrameReads, &mut *own, 1,
                kirby_assist_core::fighter_kinds::LUCINA, Some(counter_config()), true)
        }.unwrap();
        assert!(!context.can_special);
        assert!(threat.is_none());
        assert_eq!(GEOMETRY.with(Cell::get), 0);
    }
}
#[test]
fn counter_observation_requires_live_geometry_and_current_window() {
    for scenario in 0..7 {
        let mut own = reset();
        counter_enemy(|e| match scenario {
            1 => e.attacks = false,
            2 => e.stopped = true,
            3 => e.frame = 100.,
            4 => e.geometry.size = f32::NAN,
            5 => e.motion = 0,
            6 => e.active = false,
            _ => {},
        });
        let (_, _, threat) = unsafe {
            counter::observe(&frame::FrameReads, &mut *own, 1,
                kirby_assist_core::fighter_kinds::LUCINA, Some(counter_config()), true)
        }.unwrap();
        assert_eq!(threat.is_some(), scenario == 0, "scenario {scenario}: {threat:?}");
    }
}
#[test]
fn truncated_counter_scan_withholds_instead_of_claiming_safe_contact() {
    let mut own = reset();
    for id in 7..10 { counter_enemy(|e| e.id = id); }
    let (_, _, threat) = unsafe {
        counter::observe(&frame::FrameReads, &mut *own, 1,
            kirby_assist_core::fighter_kinds::LUCINA, Some(counter_config()), true)
    }.unwrap();
    assert!(threat.is_none());
    assert_eq!(GEOMETRY.with(Cell::get), 16);
}
#[test]
fn earlier_uncounterable_attacker_is_not_hidden_by_a_later_candidate() {
    let mut own = reset();
    counter_enemy(|_| {});
    counter_enemy(|e| { e.id = 8; e.position[0] = 8.; });
    let (_, _, threat) = unsafe {
        counter::observe(&frame::FrameReads, &mut *own, 1,
            kirby_assist_core::fighter_kinds::LUCINA, Some(counter_config()), true)
    }.unwrap();
    assert!(threat.is_none());
}
#[test]
fn counter_contact_respects_delay_approach_and_defender_situation() {
    use kirby_assist_core::native_states::situation;
    for air in [false, true] {
        for delay in [0., 2.] {
            let mut own = reset();
            if air {
                own.situation = situation::AIR;
                own.status = kirby_assist_core::native_states::status::FALL;
            }
            counter_enemy(|e| e.facing = -1.);
            let mut config = counter_config();
            config.input_delay = delay;
            let (_, _, threat) = unsafe {
                counter::observe(&frame::FrameReads, &mut *own, 1,
                    kirby_assist_core::fighter_kinds::LUCINA, Some(config), true)
            }.unwrap();
            assert_eq!(threat.is_some(), delay == 0., "air={air}, delay={delay}");
        }
    }
}
#[test]
fn disabled_unsupported_and_unconfigured_modes_make_zero_native_queries() {
    let mut own = reset();
    for _ in 0..1000 {
        unsafe {
            assert!(
                reflect::observe(&frame::FrameReads, &mut *own, 1, 28, None, None, true).is_none()
            );
            assert!(reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                22,
                Some(config()),
                None,
                true
            )
            .is_none());
            assert!(reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                28,
                Some(config()),
                None,
                false
            )
            .is_none());
        }
    }
    assert_eq!(READS.with(Cell::get), 0);
    // An enabled fighter with no remembered projectile also has no native work.
    for _ in 0..1000 {
        assert!(unsafe {
            reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                28,
                Some(config()),
                None,
                true,
            )
        }
        .is_none());
    }
    assert_eq!(READS.with(Cell::get), 0);
}
#[test]
fn busy_captured_full_unready_and_shielding_states_never_read_projectile_geometry() {
    for status in [
        kirby_assist_core::native_states::status::GUARD_ON,
        kirby_assist_core::native_states::status::GUARD,
        kirby_assist_core::native_states::status::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ESCAPE_F,
        kirby_assist_core::native_states::status::CATCH,
        kirby_assist_core::native_states::status::CATCH_JUMP,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE,
        kirby_assist_core::native_states::status::SPECIAL_HI,
        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_SHOOT,
    ] {
        let mut own = reset();
        own.status = status;
        projectile(|_| {});
        assert!(unsafe {
            reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                28,
                Some(config()),
                None,
                true,
            )
        }
        .is_none());
        assert_eq!(GEOMETRY.with(Cell::get), 0);
    }
    for case in 0..3 {
        let mut own = reset();
        match case {
            0 => own.gauge = 3.,
            1 => own.linked = true,
            _ => own.ready = false,
        };
        projectile(|_| {});
        assert!(unsafe {
            reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                28,
                Some(config()),
                None,
                true,
            )
        }
        .is_none());
        assert_eq!(GEOMETRY.with(Cell::get), 0);
    }
}
#[test]
fn actual_capability_team_situation_and_active_geometry_filter_the_incoming_shot() {
    for rejected in 0..8 {
        let mut own = reset();
        projectile(|p| match rejected {
            0 => p.geometry.direct = 1,
            1 => p.geometry.reflectable = 0,
            2 => p.team = 1,
            3 => {
                p.geometry.situation = kirby_assist_core::native_states::situation::ATTACK_AIR_MASK
            }
            4 => p.active = false,
            5 => p.attacks = false,
            6 => p.geometry.size = f32::NAN,
            _ => p.position[2] = 50.,
        });
        let (_, _, threat) = unsafe {
            reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                28,
                Some(config()),
                None,
                true,
            )
        }
        .unwrap();
        assert!(threat.is_none(), "rejected source {rejected}");
    }
    let mut own = reset();
    projectile(|p| p.generation = 17);
    let (_, _, threat) = unsafe {
        reflect::observe(
            &frame::FrameReads,
            &mut *own,
            1,
            28,
            Some(config()),
            None,
            true,
        )
    }
    .unwrap();
    assert_eq!(threat.unwrap().in_frames, 7.);
    assert_eq!(
        threat.unwrap().generation,
        17,
        "native attack identity must reach admission"
    );
    let mut own = reset();
    projectile(|p| {
        p.geometry.reflectable = 0;
        p.geometry.absorbable = 1;
    });
    assert!(unsafe {
        reflect::observe(
            &frame::FrameReads,
            &mut *own,
            1,
            28,
            Some(config()),
            None,
            true,
        )
    }
    .unwrap()
    .2
    .is_some());
}
#[test]
fn crowded_projectiles_and_hitstop_keep_native_work_bounded() {
    let mut own = reset();
    OBJECTS.with(|v| {
        for index in 0..32 {
            let mut p = Box::new(game_ffi::Object::new(0x10000002 + index));
            p.team = 2;
            p.position[0] = 41.;
            p.velocity[0] = -5.;
            v.borrow_mut().push(p);
        }
    });
    unsafe {
        reflect::observe(
            &frame::FrameReads,
            &mut *own,
            1,
            28,
            Some(config()),
            None,
            true,
        )
    };
    assert_eq!(GEOMETRY.with(Cell::get), 16);
    assert!(JOINTS.with(Cell::get) <= 32);
    let mut own = reset();
    own.status = kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT;
    own.stopped = true;
    projectile(|_| {});
    let (_, context, threat) = unsafe {
        reflect::observe(
            &frame::FrameReads,
            &mut *own,
            1,
            28,
            Some(config()),
            attack_sources::projectiles().into_iter().flatten().find(|key| key.id == 0x10000002),
            true,
        )
    }
    .unwrap();
    assert!(context.stopped);
    assert!(threat.is_none());
    assert_eq!(GEOMETRY.with(Cell::get), 0);
}

#[test]
fn an_existing_target_is_not_replaced_by_another_projectile_mid_bucket() {
    let mut own = reset();
    own.status = kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT;
    OBJECTS.with(|v| {
        let mut nearer = Box::new(game_ffi::Object::new(0x10000003));
        nearer.team = 2;
        nearer.position[0] = 21.;
        nearer.velocity[0] = -5.;
        v.borrow_mut().push(nearer);
    });
    projectile(|_| {});
    let (_, _, threat) = unsafe {
        reflect::observe(
            &frame::FrameReads,
            &mut *own,
            1,
            28,
            Some(config()),
            attack_sources::projectiles().into_iter().flatten().find(|key| key.id == 0x10000002),
            true,
        )
    }
    .unwrap();
    assert_eq!(threat.unwrap().id, 0x10000002);
}

#[test]
fn attached_pikmin_and_luma_never_trigger_bucket_but_incoming_reflectable_pikmin_can() {
    for (kind, status, allowed) in [
        (
            kirby_assist_core::battle_objects::WEAPON_KIND_PIKMIN_PIKMIN,
            kirby_assist_core::native_states::status::LANDING_LIGHT,
            false,
        ),
        (
            kirby_assist_core::battle_objects::WEAPON_KIND_PIKMIN_PIKMIN,
            kirby_assist_core::native_states::status::LANDING_ATTACK_AIR,
            false,
        ),
        (
            kirby_assist_core::battle_objects::WEAPON_KIND_PIKMIN_PIKMIN,
            kirby_assist_core::native_states::status::LANDING_FALL_SPECIAL,
            false,
        ),
        (
            kirby_assist_core::battle_objects::WEAPON_KIND_ROSETTA_TICO,
            kirby_assist_core::native_states::status::SQUAT_RV,
            false,
        ),
        (
            kirby_assist_core::battle_objects::WEAPON_KIND_PIKMIN_PIKMIN,
            kirby_assist_core::native_states::status::SQUAT_RV,
            true,
        ),
    ] {
        let mut own = reset();
        projectile(|p| {
            p.kind = kind;
            p.status = status;
        });
        let (_, _, threat) = unsafe {
            reflect::observe(
                &frame::FrameReads,
                &mut *own,
                1,
                28,
                Some(config()),
                None,
                true,
            )
        }
        .unwrap();
        assert_eq!(
            threat.is_some(),
            allowed,
            "kind {kind:#x} status {status:#x}"
        );
    }
}
