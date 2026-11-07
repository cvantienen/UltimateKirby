//! Compile the production read-only damage adapter against copied native facts.
#![allow(dead_code)]
mod game_ffi {
    use std::cell::Cell;
    pub struct Object {
        pub position: [f32; 2],
        pub launch: [f32; 2],
        pub hitstun: f32,
    }
    pub struct Vector3 {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }
    thread_local! {pub static QUERIES:Cell<usize>=const{Cell::new(0)};}
    pub unsafe fn pos_z(_: *mut Object) -> f32 {
        0.
    }
    pub unsafe fn check_dead_area(p: &Vector3) -> i32 {
        QUERIES.with(|n| n.set(n.get() + 1));
        if p.y >= 180. {
            0
        } else if p.y <= -140. {
            1
        } else if p.x <= -240. {
            2
        } else if p.x >= 240. {
            3
        } else {
            -1
        }
    }
    pub unsafe fn param_float(_: *mut Object, parent: u64, child: u64) -> f32 {
        assert_eq!(child, 0);
        match parent {
            0xba18057d9 => 0.08,
            0x12ec5626fe => 1.6,
            0x1a58828323 => 0.06,
            0x1d71e21b0b => 1.2,
            _ => panic!("unverified native parameter"),
        }
    }
    pub unsafe fn work_float(o: *mut Object, key: i32) -> f32 {
        assert_eq!(key, 0x13);
        (*o).hitstun
    }
}
#[path = "."]
mod live {
    pub mod native_params {
        pub const PARAM_AIR_ACCEL_Y: u64 = 0xba18057d9;
        pub const PARAM_AIR_SPEED_Y_STABLE: u64 = 0x12ec5626fe;
        pub const PARAM_DAMAGE_FLY_TOP_GRAVITY: u64 = 0x1a58828323;
        pub const PARAM_DAMAGE_FLY_TOP_TERMINAL_SPEED: u64 = 0x1d71e21b0b;
    }
    pub mod frame {
        use crate::game_ffi::Object;
        pub struct FrameReads;
        impl FrameReads {
            pub unsafe fn pos_x(&self, o: *mut Object) -> f32 {
                (*o).position[0]
            }
            pub unsafe fn pos_y(&self, o: *mut Object) -> f32 {
                (*o).position[1]
            }
            pub unsafe fn damage_launch(&self, o: *mut Object) -> [f32; 2] {
                (*o).launch
            }
        }
    }
    #[path = "../../../src/live/damage.rs"]
    mod damage;
    use crate::game_ffi::{Object, QUERIES};
    fn object() -> Object {
        Object {
            position: [0.; 2],
            launch: [4., 1.],
            hitstun: 55.,
        }
    }
    fn count() -> usize {
        QUERIES.with(|n| n.get())
    }
    #[test]
    fn hit_bounds_are_reused_for_polls_and_refreshed_after_sdi_before_final_di() {
        let mut o = object();
        let (mut cache, t) =
            unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 12, None) };
        let first_queries = count();
        assert!(first_queries > 0 && first_queries <= 90);
        assert!(t.unwrap().valid());
        for remaining in [12, 12, 11, 10, 8, 4] {
            let result =
                unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, remaining, cache) };
            cache = result.0;
            assert_eq!(count(), first_queries);
        }
        o.position = [10., 5.];
        let (cache, t) = unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 3, cache) };
        assert_eq!(t.unwrap().position, [10., 5.]);
        assert!(count() > first_queries);
        let final_queries = count();
        let (cache, _) = unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 0, cache) };
        assert_eq!(count(), final_queries);
        assert_eq!(
            unsafe { damage::observe(&frame::FrameReads, &mut o, 1, false, 0, cache) },
            (None, None)
        );
    }
    #[test]
    fn new_victims_and_increased_hitstop_cannot_inherit_an_earlier_hits_geometry() {
        let mut o = object();
        let (cache, _) = unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 8, None) };
        let initial = count();
        let (cache, _) = unsafe { damage::observe(&frame::FrameReads, &mut o, 2, true, 8, cache) };
        assert!(count() > initial);
        let next = count();
        unsafe { damage::observe(&frame::FrameReads, &mut o, 2, true, 12, cache) };
        assert!(count() > next);
    }
    #[test]
    fn upward_top_launch_uses_native_damage_fly_top_physics_and_unknown_geometry_fails_closed() {
        let mut o = object();
        o.launch = [0., 5.];
        let (_, t) = unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 3, None) };
        let t = t.unwrap();
        assert_eq!(t.gravity, 0.06);
        assert_eq!(t.fall_speed, 1.2);
        o.launch = [4., 1.];
        let (_, t) = unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 3, None) };
        let t = t.unwrap();
        assert_eq!(t.gravity, 0.08);
        assert_eq!(t.fall_speed, 1.6);
        o.position = [f32::NAN, 0.];
        assert!(
            unsafe { damage::observe(&frame::FrameReads, &mut o, 1, true, 3, None) }
                .1
                .is_none()
        );
    }
}
