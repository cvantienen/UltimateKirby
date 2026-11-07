//! Compile the actual native tech detector against copied read-only game facts.
#![allow(dead_code, unexpected_cfgs)]
use std::cell::Cell;
mod game_ffi {
    use super::Cell;
    #[repr(C)]
    pub struct Object {
        pub header: u64,
        pub id: u32,
        pub status: i32,
        pub situation: i32,
        pub previous: i32,
        pub previous_situation: i32,
        pub floor_tech: bool,
        pub wall_tech: bool,
        pub ceil_tech: bool,
        pub timing: [i32; 3],
        pub disabled: bool,
        pub damage_stop: bool,
        pub air_escape: bool,
        pub stop_remaining: u64,
        pub reaction: f32,
        pub frame: f32,
        pub rate: f32,
        pub position: [f32; 2],
        pub velocity: [f32; 2],
        pub rays: Cell<u32>,
        pub kind: i32,
        pub wall_plane: Option<f32>,
        pub ceiling_plane: Option<f32>,
        pub touches: u32,
        pub ecb_offset: [f32; 2],
        pub scale: f32,
    }
    impl Default for Object {
        fn default() -> Self {
            Self {
                header: 0,
                id: 1,
                status: kirby_assist_core::native_states::status::DAMAGE_FLY,
                situation: kirby_assist_core::native_states::situation::AIR,
                previous: 0,
                previous_situation: kirby_assist_core::native_states::situation::AIR,
                floor_tech: true,
                wall_tech: true,
                ceil_tech: true,
                timing: [28, 12, 7],
                disabled: false,
                damage_stop: false,
                air_escape: false,
                stop_remaining: 0,
                reaction: 60.,
                frame: 1.,
                rate: 1.,
                position: [0., 10.],
                velocity: [0., -4.],
                rays: Cell::new(0),
                kind: kirby_assist_core::fighter_kinds::BOWSER,
                wall_plane: None,
                ceiling_plane: None,
                touches: 0,
                ecb_offset: [0.; 2],
                scale: 1.,
            }
        }
    }
    pub unsafe fn ground_touch(o: *mut Object, mask: u32) -> bool {
        (*o).touches & mask != 0
    }
    pub unsafe fn passive_ground_touch(o: *mut Object, mask: u32) -> bool {
        ground_touch(o, mask)
    }
    pub unsafe fn ground_offset_x(o: *mut Object) -> f32 {
        (*o).ecb_offset[0]
    }
    pub unsafe fn ground_offset_y(o: *mut Object) -> f32 {
        (*o).ecb_offset[1]
    }
    pub unsafe fn scale(o: *mut Object) -> f32 {
        (*o).scale
    }
    pub struct Vector2 {
        pub x: f32,
        pub y: f32,
    }
    pub unsafe fn is_link(_: *mut Object, _: i32) -> bool {
        false
    }
    pub unsafe fn link_object_id(_: *mut Object, _: i32) -> u64 {
        0
    }
    pub unsafe fn is_active(_: u32) -> bool {
        false
    }
    pub unsafe fn module_accessor(_: u32) -> *mut Object {
        std::ptr::null_mut()
    }
    pub unsafe fn previous_status(o: *mut Object, _: i32) -> i32 {
        (*o).previous
    }
    pub unsafe fn previous_situation_kind(o: *mut Object) -> i32 {
        (*o).previous_situation
    }
    pub unsafe fn is_damage_stop(o: *mut Object) -> bool {
        (*o).damage_stop
    }
    pub unsafe fn hitstop_remaining(o: *mut Object) -> u64 {
        (*o).stop_remaining
    }
    pub unsafe fn is_flag(o: *mut Object, key: i32) -> bool {
        match key {
            kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE => (*o).disabled,
            kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_END_REACTION => (*o).air_escape,
            _ => false,
        }
    }
    pub unsafe fn transition_enabled(o: *mut Object, key: i32) -> bool {
        match key {
            kirby_assist_core::native_states::transition_terms::PASSIVE
            | kirby_assist_core::native_states::transition_terms::PASSIVE_FB => (*o).floor_tech,
            kirby_assist_core::native_states::transition_terms::PASSIVE_WALL => (*o).wall_tech,
            kirby_assist_core::native_states::transition_terms::PASSIVE_CEIL => (*o).ceil_tech,
            kirby_assist_core::native_states::transition_terms::CONT_ESCAPE_AIR => (*o).air_escape,
            _ => false,
        }
    }
    pub unsafe fn param_int(o: *mut Object, parent: u64, child: u64) -> i32 {
        assert_eq!(parent, 0x6e5ec7051);
        (*o).timing[match child {
            0x14459066dd => 0,
            0x15ba3cdb02 => 1,
            0x2535742c20 => 2,
            _ => panic!("unexpected common parameter"),
        }]
    }
    pub unsafe fn work_float(o: *mut Object, key: i32) -> f32 {
        assert_eq!(key, 0x13);
        (*o).reaction
    }
    pub unsafe fn ray_check(
        o: *mut Object,
        origin: &Vector2,
        path: &Vector2,
        passable: bool,
    ) -> u64 {
        (*o).rays.set((*o).rays.get() + 1);
        let wall = (*o).wall_plane.is_some_and(|plane| {
            let t = (plane - origin.x) / path.x;
            (0.0..=1.).contains(&t)
        });
        let ceiling = (*o).ceiling_plane.is_some_and(|plane| {
            let t = (plane - origin.y) / path.y;
            (0.0..=1.).contains(&t)
        });
        u64::from(if passable {
            origin.y >= 0. && origin.y + path.y <= 0.
        } else {
            wall || ceiling
        })
    }
}
#[cfg(feature = "diagnostic-logging")]
mod performance {
    pub enum Part {
        Tech,
    }
    pub struct Scope;
    impl Scope {
        pub fn new(_: Part) -> Self {
            Self
        }
    }
}
#[path = "."]
mod live {
    pub mod frame {
        use crate::game_ffi::Object;
        #[derive(Default)]
        pub struct FrameReads;
        impl FrameReads {
            pub unsafe fn status_kind(&self, o: *mut Object) -> i32 {
                (*o).status
            }
            pub unsafe fn situation_kind(&self, o: *mut Object) -> i32 {
                (*o).situation
            }
            pub unsafe fn fighter_kind(&self, o: *mut Object) -> i32 {
                (*o).kind
            }
            pub unsafe fn motion_kind(&self, _: *mut Object) -> u64 {
                1
            }
            pub unsafe fn motion_frame(&self, o: *mut Object) -> f32 {
                (*o).frame
            }
            pub unsafe fn motion_rate(&self, o: *mut Object) -> f32 {
                (*o).rate
            }
            pub unsafe fn speed_x(&self, o: *mut Object, _: i32) -> f32 {
                (*o).velocity[0]
            }
            pub unsafe fn speed_y(&self, o: *mut Object, _: i32) -> f32 {
                (*o).velocity[1]
            }
            pub unsafe fn pos_x(&self, o: *mut Object) -> f32 {
                (*o).position[0]
            }
            pub unsafe fn pos_y(&self, o: *mut Object) -> f32 {
                (*o).position[1]
            }
            pub unsafe fn damage_launch(&self, o: *mut Object) -> [f32; 2] {
                (*o).velocity
            }
        }
    }
    #[path = "../../../src/live/tech.rs"]
    pub mod tech;
    use kirby_assist_core::{
        native_input::MAPPED_GUARD,
        tech_input::{TechContext, TechInput},
        tech_prediction::{Body, FloorMotionHistory},
    };
    fn assess(
        o: &mut crate::game_ffi::Object,
        stopped: bool,
        enabled: bool,
    ) -> tech::TechAssessment {
        unsafe {
            tech::assess_tech(
                &frame::FrameReads,
                o,
                5.,
                stopped,
                0.,
                enabled,
                &mut FloorMotionHistory::default(),
                Body::default(),
            )
        }
    }
    #[test]
    fn actual_detector_accepts_native_ground_contact_without_rays_and_keeps_native_restrictions() {
        let mut o = crate::game_ffi::Object {
            situation: kirby_assist_core::native_states::situation::GROUND,
            floor_tech: true,
            ..Default::default()
        };
        let a = assess(&mut o, false, true);
        assert_eq!(a.reason, "collision_predicted");
        assert_eq!(a.contact_in, Some(0.));
        assert!(a.prepare_guard);
        assert_eq!(o.rays.get(), 0);
        o.previous_situation = kirby_assist_core::native_states::situation::GROUND;
        assert_eq!(
            assess(&mut o, false, true).contact_in,
            None,
            "a grounded launch is not an impact"
        );
        o.previous_situation = kirby_assist_core::native_states::situation::AIR;
        o.floor_tech = false;
        assert_eq!(assess(&mut o, false, true).reason, "not_airborne");
        o.floor_tech = true;
        o.disabled = true;
        assert_eq!(assess(&mut o, false, true).contact_in, None);
        o.disabled = false;
        o.status = kirby_assist_core::native_states::status::DOWN;
        assert_eq!(
            assess(&mut o, false, true).reason,
            "outside_supported_tumble"
        );
        o.status = kirby_assist_core::native_states::status::DAMAGE_FLY;
        assert_eq!(assess(&mut o, false, false).reason, "auto_tech_disabled");
    }
    #[test]
    fn grounded_global_stop_does_not_fake_contact_but_known_damage_stop_retains_it() {
        let mut o = crate::game_ffi::Object {
            situation: kirby_assist_core::native_states::situation::GROUND,
            floor_tech: true,
            ..Default::default()
        };
        assert_eq!(assess(&mut o, true, true).contact_in, None);
        o.damage_stop = true;
        o.stop_remaining = 3;
        let a = assess(&mut o, true, true);
        assert_eq!(a.contact_in, Some(3.));
        assert!(a.prepare_guard);
    }
    #[test]
    fn actual_airborne_prediction_keeps_ray_and_air_dodge_gates() {
        let mut o = crate::game_ffi::Object::default();
        let a = assess(&mut o, false, true);
        assert_eq!(a.reason, "collision_predicted");
        assert!(a.contact_in.is_some_and(|f| (1.0..=5.0).contains(&f)));
        assert!(o.rays.get() > 0 && o.rays.get() <= 60);
        o.air_escape = true;
        o.rays.set(0);
        let a = assess(&mut o, false, true);
        assert_eq!(a.reason, "air_dodge_available");
        assert!(!a.prepare_guard);
        assert_eq!(o.rays.get(), 0);
    }
    #[test]
    fn native_reaction_timer_reaches_scheduler_even_when_damage_animation_is_frozen() {
        let mut o = crate::game_ffi::Object {
            rate: 0.,
            ..Default::default()
        };
        let mut input = TechInput::default();
        for remaining in (20..=60).rev() {
            o.reaction = remaining as f32;
            let a = assess(&mut o, false, true);
            let out = input.sample_with_reaction_clock(
                true,
                true,
                40,
                TechContext {
                    object: o.id,
                    status: a.status,
                    motion: 1,
                    frame: o.frame,
                    rate: o.rate,
                    stopped: false,
                    remaining_stop: a.remaining_stop,
                    contact_in: a.contact_in,
                    input_delay: 0.,
                    previous_guard: false,
                    prepare_guard: a.prepare_guard,
                },
                a.reaction_remaining,
            );
            if remaining == 60 {
                assert!(out.press);
                input.mapped_press();
            } else if remaining == 20 {
                assert!(out.press);
                assert_ne!(
                    kirby_assist_core::tech_input::mapped_guard(0, out) & MAPPED_GUARD,
                    0
                );
            } else {
                assert!(!out.press);
            }
        }
    }
    #[test]
    fn native_surface_stop_states_require_the_matching_transition_term_and_a_live_clock() {
        for status in [
            kirby_assist_core::native_states::status::STOP_WALL,
            kirby_assist_core::native_states::status::STOP_CEIL,
        ] {
            let mut o = crate::game_ffi::Object {
                status,
                ..Default::default()
            };
            let a = assess(&mut o, false, true);
            assert_eq!(a.contact_in, Some(0.));
            assert!(a.prepare_guard);
            assert_eq!(o.rays.get(), 0);
            assert!(
                assess(&mut o, true, true).contact_in.is_none(),
                "global stop cannot create an input opportunity"
            );
            if status == kirby_assist_core::native_states::status::STOP_WALL {
                o.wall_tech = false;
            } else {
                o.ceil_tech = false;
            }
            let a = assess(&mut o, false, true);
            assert_eq!(a.contact_in, None);
            assert!(!a.prepare_guard);
        }
    }
    #[test]
    fn native_windows_and_lockout_are_read_from_current_common_params() {
        let mut o = crate::game_ffi::Object::default();
        assert_eq!(unsafe { tech::native_timing(&mut o, 40) }, (28, 12, 7));
        o.timing = [19, 8, 6];
        assert_eq!(unsafe { tech::native_timing(&mut o, 40) }, (19, 8, 6));
        o.timing = [-1, 0, 200];
        assert_eq!(unsafe { tech::native_timing(&mut o, 40) }, (40, 12, 7));
    }
    #[test]
    fn a_geometric_hit_without_native_passive_eligibility_cannot_request_tech() {
        let mut o = crate::game_ffi::Object {
            floor_tech: false,
            wall_tech: false,
            ceil_tech: false,
            ..Default::default()
        };
        assert_eq!(assess(&mut o, false, true).contact_in, None);
        assert_eq!(o.rays.get(), 0);
    }

    // Identity coverage with varied copied native facts; these are not dumps of
    // every fighter's animation/ECB and do not replace on-device roster tests.
    #[test]
    fn roster_contacts_ignore_model_size_and_frozen_animations() {
        let mut count = 0;
        for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
            count += 1;
            for (touch, velocity) in [
                (0x8, [0., -0.001]),
                (0x2, [0., 0.]),
                (0x4, [0., 0.]),
                (0x1, [0., 0.]),
            ] {
                for status in [
                    kirby_assist_core::native_states::status::DAMAGE_FLY,
                    kirby_assist_core::native_states::status::DAMAGE_FLY_ROLL,
                    kirby_assist_core::native_states::status::DAMAGE_FLY_METEOR,
                    kirby_assist_core::native_states::status::DAMAGE_FALL,
                ] {
                    let mut o = crate::game_ffi::Object {
                        kind,
                        status,
                        touches: touch,
                        velocity,
                        position: [if kind % 2 == 0 { 100. } else { -100. }, 100.],
                        scale: [0.5, 1., 2., 3.][kind as usize % 4],
                        ecb_offset: [kind as f32 % 5. - 2., kind as f32 % 9. - 4.],
                        frame: 5.,
                        rate: 0.,
                        ..Default::default()
                    };
                    let a = assess(&mut o, false, true);
                    assert_eq!(
                        a.reason, "native_collision_contact",
                        "kind={kind}, status={status:#x}, mask={touch:#x}"
                    );
                    assert_eq!(a.contact_in, Some(0.));
                    assert!(a.prepare_guard);
                    assert_eq!(o.rays.get(), 0);
                    let mut input = TechInput::default();
                    assert!(
                        input
                            .sample_with_native_delivery(
                                true,
                                true,
                                28,
                                TechContext {
                                    object: o.id,
                                    status,
                                    motion: 1,
                                    frame: o.frame,
                                    rate: 0.,
                                    stopped: false,
                                    remaining_stop: 0,
                                    contact_in: a.contact_in,
                                    input_delay: 0.,
                                    previous_guard: false,
                                    prepare_guard: a.prepare_guard
                                },
                                a.reaction_remaining,
                                12,
                                7,
                                false
                            )
                            .press
                    );
                }
            }
        }
        assert_eq!(
            count, 0x5e,
            "contact eligibility must not depend on a fighter menu/profile whitelist"
        );
    }
    #[test]
    fn final_zero_count_victim_hitlag_retains_contact_while_global_stop_cannot_invent_it() {
        for (status, touches) in [
            (0x49, 0x8),
            (0x49, 0x2),
            (0x49, 0x1),
            (0x6c, 0x2),
            (0x6d, 0x1),
        ] {
            let mut o = crate::game_ffi::Object {
                status,
                touches,
                damage_stop: true,
                stop_remaining: 0,
                rate: 0.,
                ..Default::default()
            };
            let a = assess(&mut o, true, true);
            assert_eq!(a.contact_in, Some(0.));
            assert!(a.prepare_guard);
            o.damage_stop = false;
            assert_eq!(assess(&mut o, true, true).contact_in, None);
        }
    }
    #[test]
    fn actual_contacts_obey_native_disable_air_escape_and_surface_terms() {
        let mut o = crate::game_ffi::Object {
            touches: 0x8,
            position: [0., 100.],
            ..Default::default()
        };
        assert_eq!(assess(&mut o, false, true).contact_in, Some(0.));
        o.disabled = true;
        assert_eq!(assess(&mut o, false, true).contact_in, None);
        o.disabled = false;
        o.floor_tech = false;
        assert_eq!(assess(&mut o, false, true).contact_in, None);
        o.floor_tech = true;
        o.air_escape = true;
        assert_eq!(assess(&mut o, false, true).contact_in, None);
    }
    #[test]
    fn a_grounded_upward_launch_is_not_a_floor_impact_but_downward_and_reflect_contacts_are() {
        let mut o = crate::game_ffi::Object {
            situation: kirby_assist_core::native_states::situation::GROUND,
            previous_situation: kirby_assist_core::native_states::situation::GROUND,
            touches: 0x8,
            velocity: [0., 4.],
            ..Default::default()
        };
        assert_eq!(assess(&mut o, false, true).contact_in, None);
        o.previous_situation = kirby_assist_core::native_states::situation::AIR;
        assert_eq!(
            assess(&mut o, false, true).contact_in,
            None,
            "a just-landed upward launch must not spend the upcoming fall's tech window"
        );
        o.previous_situation = kirby_assist_core::native_states::situation::GROUND;
        o.velocity = [0., -0.01];
        assert_eq!(assess(&mut o, false, true).contact_in, Some(0.));
        o.velocity = [0., 4.];
        o.status = kirby_assist_core::native_states::status::DAMAGE_FLY_REFLECT_D;
        assert_eq!(assess(&mut o, false, true).contact_in, Some(0.));
    }
    #[test]
    fn forecasts_follow_animated_ecb_offsets_and_current_fighter_scale() {
        let mut o = crate::game_ffi::Object::default();
        assert!(assess(&mut o, false, true).contact_in.is_some());
        o.ecb_offset = [0., 20.];
        assert_eq!(
            assess(&mut o, false, true).contact_in,
            None,
            "raised ECB is not at the old root position"
        );
        o.ecb_offset = [0., -8.];
        assert_eq!(assess(&mut o, false, true).contact_in, Some(1.));
        o.ecb_offset = [0.; 2];
        o.position = [0., 0.];
        o.velocity = [0., 0.2];
        o.ceiling_plane = Some(20.);
        assert_eq!(assess(&mut o, false, true).contact_in, None);
        o.scale = 3.;
        assert!(
            assess(&mut o, false, true).contact_in.is_some(),
            "scaled ceiling probes must reach the same fighter's larger collision envelope"
        );
        o.scale = f32::NAN;
        assert_eq!(
            assess(&mut o, false, true).contact_in,
            None,
            "invalid scales must not leave a previous giant envelope cached"
        );
    }
}
