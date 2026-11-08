//! Exercise native-facing fighter adapters against bounded read-only stubs.
#![allow(dead_code)]
use std::cell::RefCell;

use kirby_assist_core::{
    auto_duck::DefenseThreat,
    autopilot_guard::Envelope,
    fighter_kinds::{KIRBY, YOSHI},
    fighter_tuning::Tuning,
    gameplay::ControllerOutput,
    jab_lock::Target,
    native_states::{situation, status},
};

#[derive(Default)]
struct Calls {
    hit: usize,
    transitions: usize,
    targets: usize,
    ground: usize,
    threats: usize,
    include_damage: bool,
    transitions_enabled: bool,
    hit_result: bool,
}

thread_local! {
    static CALLS: RefCell<Calls> = RefCell::new(Calls::default());
}

mod game_ffi {
    pub type Object = u8;
    pub const COLLISION_KIND_MASK_HIT: i32 = 1;
    pub unsafe fn attack_infliction_now(_: *mut Object, _: i32) -> bool {
        super::CALLS.with(|calls| {
            let mut calls = calls.borrow_mut();
            calls.hit += 1;
            calls.hit_result
        })
    }
    pub unsafe fn transition_enabled(_: *mut Object, _: i32) -> bool {
        super::CALLS.with(|calls| {
            let mut calls = calls.borrow_mut();
            calls.transitions += 1;
            calls.transitions_enabled
        })
    }
}

mod frame {
    pub struct FrameReads;
    impl FrameReads {
        pub unsafe fn pos_x(&self, _: *mut u8) -> f32 {
            3.0
        }
        pub unsafe fn pos_y(&self, _: *mut u8) -> f32 {
            4.0
        }
        pub unsafe fn speed_x(&self, _: *mut u8) -> f32 {
            0.5
        }
        pub unsafe fn speed_y(&self, _: *mut u8) -> f32 {
            -0.5
        }
        pub unsafe fn facing(&self, _: *mut u8) -> f32 {
            -1.0
        }
        pub unsafe fn ground_support(&self, _: *mut u8, _: [f32; 2], _: f32) -> [bool; 2] {
            super::CALLS.with(|calls| calls.borrow_mut().ground += 1);
            [true, false]
        }
    }
}

mod observations {
    use super::*;
    pub unsafe fn jab_lock_targets(
        _: &frame::FrameReads,
        _: *mut u8,
        _: u32,
        include_damage: bool,
        _: f32,
        _: f32,
    ) -> [Option<Target>; 8] {
        CALLS.with(|calls| {
            let mut calls = calls.borrow_mut();
            calls.targets += 1;
            calls.include_damage = include_damage;
        });
        [None; 8]
    }
    pub unsafe fn incoming_fighter_attack(
        _: &frame::FrameReads,
        _: *mut u8,
        _: u32,
        _: [f32; 2],
        _: [f32; 2],
        _: bool,
        _: Envelope,
    ) -> DefenseThreat {
        CALLS.with(|calls| calls.borrow_mut().threats += 1);
        DefenseThreat::default()
    }
}

#[path = "../../../src/live/fighters/kirby/defense.rs"]
mod kirby;
#[path = "../../../src/live/fighters/yoshi/jab_lock.rs"]
mod yoshi;

fn yoshi_observation() -> yoshi::Observation {
    yoshi::Observation {
        object: 1,
        kind: YOSHI,
        status: status::WAIT,
        situation: situation::GROUND,
        motion: 7,
        frame: 2.0,
        rate: 1.0,
        stopped: false,
        clatter: false,
        tech: false,
    }
}

fn kirby_observation() -> kirby::Observation {
    kirby::Observation {
        enabled: true,
        kind: KIRBY,
        object: 1,
        status: status::WAIT,
        situation: situation::GROUND,
        motion: 7,
        frame: 2.0,
        rate: 1.0,
        stopped: false,
        clatter: false,
        tech: false,
    }
}

#[test]
fn yoshi_admission_and_observation_preserve_lazy_native_read_gates() {
    assert!(!yoshi::enabled(true, YOSHI, true, || panic!(
        "menu should not be read"
    )));
    assert!(!yoshi::enabled(false, KIRBY, true, || panic!(
        "menu should not be read"
    )));
    assert!(!yoshi::enabled(false, YOSHI, false, || panic!(
        "menu should not be read"
    )));
    assert!(yoshi::enabled(false, YOSHI, true, || true));
    CALLS.with(|calls| *calls.borrow_mut() = Calls::default());
    let reads = frame::FrameReads;
    let object = std::ptr::null_mut();
    let tuning = Tuning::default();
    let (context, _) =
        unsafe { yoshi::observe(&reads, object, &yoshi_observation(), None, tuning, || false) };
    assert!(!context.hit && !context.can_jab && !context.can_smash && !context.can_grab);
    CALLS.with(|calls| {
        let calls = calls.borrow();
        assert_eq!(
            (calls.hit, calls.targets, calls.transitions, calls.ground),
            (0, 1, 0, 0)
        );
        assert!(!calls.include_damage);
    });
    CALLS.with(|calls| {
        *calls.borrow_mut() = Calls {
            hit_result: true,
            transitions_enabled: true,
            ..Default::default()
        }
    });
    let mut observation = yoshi_observation();
    observation.status = status::ATTACK_LW3;
    let (context, _) =
        unsafe { yoshi::observe(&reads, object, &observation, None, tuning, || false) };
    assert!(context.hit && context.can_jab && context.can_smash && context.can_grab);
    assert_eq!(context.safe_ground, [true, false]);
    CALLS.with(|calls| {
        let calls = calls.borrow();
        assert_eq!(
            (calls.hit, calls.targets, calls.transitions, calls.ground),
            (1, 1, 3, 1)
        );
        assert!(calls.include_damage);
    });
    CALLS.with(|calls| {
        *calls.borrow_mut() = Calls {
            transitions_enabled: true,
            ..Default::default()
        }
    });
    observation.status = status::WAIT;
    observation.situation = situation::AIR;
    let (context, _) =
        unsafe { yoshi::observe(&reads, object, &observation, None, tuning, || true) };
    assert!(!context.grounded && !context.can_jab && !context.can_smash && !context.can_grab);
    CALLS.with(|calls| {
        let calls = calls.borrow();
        assert!(calls.include_damage);
        assert_eq!((calls.transitions, calls.ground), (0, 0));
    });
    let mut state = Some(Default::default());
    assert_eq!(
        yoshi::sample(&mut state, None, true, None, tuning),
        ControllerOutput::default()
    );
    assert!(state.is_none());
}

#[test]
fn kirby_defense_gates_fighter_ground_and_expensive_observations() {
    let reads = frame::FrameReads;
    let object = std::ptr::null_mut();
    let tuning = Tuning::default();
    CALLS.with(|calls| *calls.borrow_mut() = Calls::default());
    let mut observation = kirby_observation();
    observation.kind = YOSHI;
    assert!(
        unsafe { kirby::observe(&reads, object, &observation, || panic!("no envelope")) }.is_none()
    );
    observation.kind = KIRBY;
    observation.situation = situation::AIR;
    assert!(
        unsafe { kirby::observe(&reads, object, &observation, || panic!("no envelope")) }.is_none()
    );
    observation.situation = situation::GROUND;
    observation.clatter = true;
    assert!(
        unsafe { kirby::observe(&reads, object, &observation, || panic!("no envelope")) }.is_none()
    );
    observation.clatter = false;
    observation.tech = true;
    assert!(
        unsafe { kirby::observe(&reads, object, &observation, || panic!("no envelope")) }.is_none()
    );
    observation.tech = false;
    observation.stopped = true;
    assert!(
        unsafe { kirby::observe(&reads, object, &observation, || panic!("no envelope")) }.is_some()
    );
    CALLS.with(|calls| assert_eq!((calls.borrow().transitions, calls.borrow().threats), (0, 0)));
    observation.stopped = false;
    CALLS.with(|calls| calls.borrow_mut().transitions_enabled = true);
    let (context, _) =
        unsafe { kirby::observe(&reads, object, &observation, || tuning.shield()).unwrap() };
    assert!(context.can_shield && context.can_crouch);
    CALLS.with(|calls| assert_eq!((calls.borrow().transitions, calls.borrow().threats), (2, 1)));
}
