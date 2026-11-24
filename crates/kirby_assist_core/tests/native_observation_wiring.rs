//! Typecheck the complete production observation/frame modules on the host.
//! Execute copied-ID snapshots and native adapters against local read-only stubs.
#![allow(dead_code, unexpected_cfgs, unused_imports)]
use kirby_assist_core::{
    battle_objects::FightersInGame, fighter_profiles::FighterProfiles,
    ledge_position::LedgePosition, ledge_trap::LedgeTrap, sequence_config::SequenceLibrary,
};
use std::sync::{atomic::AtomicUsize, Mutex, OnceLock};
#[path = "../../../src/game_ffi.rs"]
mod native_abi;
mod native_params {
    pub const PARAM_GROUND_BRAKE: u64 = 0x0c8cc9db76;
    pub const PARAM_AIR_ACCEL_Y: u64 = 0xba18057d9;
    pub const PARAM_AIR_SPEED_Y_STABLE: u64 = 0x12ec5626fe;
}
thread_local! {
    static LINKS:std::cell::Cell<u32>=const{std::cell::Cell::new(0)};
    static LINK_READS:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};
    static INVALIDATE_GEOMETRY_ON_SCALE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static SOURCES: std::cell::RefCell<Vec<Box<Actor>>> = const { std::cell::RefCell::new(Vec::new()) };
}
#[repr(C)]
struct Actor {
    header: u64,
    id: u32,
    status: i32,
    situation: i32,
    position: [f32; 3],
    velocity: [f32; 2],
    stopped: bool,
    remaining_stop: u64,
    reaction: f32,
    guard: u32,
    passive_disabled: bool,
    air_escape: bool,
    floor_enabled: bool,
    active: bool,
    motion: u64,
    frame: f32,
    geometry: native_abi::AttackGeometry,
}
impl Actor {
    fn new(id: u32) -> Self {
        let mut geometry: native_abi::AttackGeometry = unsafe { std::mem::zeroed() };
        geometry.y = 8.;
        geometry.size = 2.;
        geometry.situation = kirby_assist_core::native_states::situation::ATTACK_GROUND_MASK;
        Self {
            header: 0,
            id,
            status: kirby_assist_core::native_states::status::WAIT,
            situation: kirby_assist_core::native_states::situation::GROUND,
            position: [0.; 3],
            velocity: [0.; 2],
            stopped: false,
            remaining_stop: 0,
            reaction: 60.,
            guard: 0,
            passive_disabled: false,
            air_escape: false,
            floor_enabled: true,
            active: true,
            motion: kirby_assist_core::timing::motion_hash("attack_lw3"),
            frame: 1.,
            geometry,
        }
    }
    fn object(&mut self) -> *mut native_abi::Object {
        (self as *mut Self).cast()
    }
}
mod game_ffi {
    pub use super::native_abi::*;
    use super::{Actor, SOURCES};
    unsafe fn actor<'a>(o: *mut Object) -> &'a Actor {
        &*o.cast::<Actor>()
    }
    pub unsafe fn status_kind(o: *mut Object) -> i32 {
        actor(o).status
    }
    pub unsafe fn situation_kind(o: *mut Object) -> i32 {
        actor(o).situation
    }
    pub unsafe fn is_stop(o: *mut Object) -> bool {
        actor(o).stopped
    }
    pub unsafe fn is_damage_stop(o: *mut Object) -> bool {
        actor(o).stopped
    }
    pub unsafe fn hitstop_remaining(o: *mut Object) -> u64 {
        actor(o).remaining_stop
    }
    pub unsafe fn pos_x(o: *mut Object) -> f32 {
        actor(o).position[0]
    }
    pub unsafe fn pos_y(o: *mut Object) -> f32 {
        actor(o).position[1]
    }
    pub unsafe fn pos_z(o: *mut Object) -> f32 {
        actor(o).position[2]
    }
    pub unsafe fn speed_x(o: *mut Object, _: i32) -> f32 {
        actor(o).velocity[0]
    }
    pub unsafe fn speed_y(o: *mut Object, _: i32) -> f32 {
        actor(o).velocity[1]
    }
    pub unsafe fn team(o: *mut Object) -> u64 {
        u64::from(actor(o).id != 1)
    }
    pub unsafe fn scale(o: *mut Object) -> f32 {
        if super::INVALIDATE_GEOMETRY_ON_SCALE.with(|flag| flag.get()) {
            (*o.cast::<super::Actor>()).geometry.size = 0.;
        }
        1.
    }
    pub unsafe fn motion_kind(o: *mut Object) -> u64 {
        actor(o).motion
    }
    pub unsafe fn motion_frame(o: *mut Object) -> f32 {
        actor(o).frame
    }
    pub unsafe fn motion_rate(_: *mut Object) -> f32 {
        1.
    }
    pub unsafe fn previous_status(_: *mut Object, _: u32) -> i32 {
        0
    }
    pub unsafe fn fighter_kind(_: u32) -> i32 {
        kirby_assist_core::fighter_kinds::KIRBY
    }
    pub unsafe fn is_active(id: u32) -> bool {
        !module_accessor(id).is_null()
    }
    pub unsafe fn module_accessor(id: u32) -> *mut Object {
        SOURCES.with(|sources| {
            sources
                .borrow_mut()
                .iter_mut()
                .find(|a| a.id == id)
                .map_or(std::ptr::null_mut(), |a| a.object())
        })
    }
    pub unsafe fn attack_active(o: *mut Object, slot: i32, _: bool) -> bool {
        actor(o).active && slot == 0
    }
    pub unsafe fn attack_geometry(o: *mut Object, _: i32, _: bool) -> *const AttackGeometry {
        &actor(o).geometry
    }
    pub unsafe fn joint_position(
        o: *mut Object,
        _: u64,
        p: *const Vector3,
        out: *mut Vector3,
        _: bool,
    ) {
        let a = actor(o);
        *out = Vector3 {
            x: a.position[0] + (*p).x,
            y: a.position[1] + (*p).y,
            z: a.position[2] + (*p).z,
        };
    }
    pub unsafe fn check_button_on(o: *mut Object, button: i32) -> bool {
        assert!(matches!(button, 0x3 | 0x10));
        actor(o).guard & (1 << button) != 0
    }
    pub unsafe fn is_flag(o: *mut Object, _: i32) -> bool {
        actor(o).passive_disabled
    }
    pub unsafe fn transition_enabled(o: *mut Object, term: i32) -> bool {
        use kirby_assist_core::native_states::transition_terms as t;
        match term {
            t::CONT_ESCAPE_AIR => actor(o).air_escape,
            t::PASSIVE | t::PASSIVE_FB => actor(o).floor_enabled,
            _ => false,
        }
    }
    pub unsafe fn work_float(o: *mut Object, key: i32) -> f32 {
        use kirby_assist_core::native_states::work_ids as w;
        match key {
            w::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME => actor(o).reaction,
            w::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X => actor(o).velocity[0],
            w::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y => actor(o).velocity[1],
            _ => panic!("unexpected work field {key}"),
        }
    }
    pub unsafe fn ray_check(
        _: *mut Object,
        from: *const Vector2,
        delta: *const Vector2,
        _: bool,
    ) -> u64 {
        u64::from((*delta).y < 0. && (*from).y >= 0. && (*from).y + (*delta).y <= 0.)
    }
    pub unsafe fn is_link(_: *mut Object, slot: i32) -> bool {
        assert!(matches!(slot, 0 | 2));
        super::LINK_READS.with(|n| n.set(n.get() + 1));
        super::LINKS.with(|links| links.get() & (1 << slot) != 0)
    }
}
use game_ffi as game;
#[cfg(feature = "diagnostic-logging")]
#[path = "../../../src/diagnostics/performance.rs"]
mod performance;
struct Menu;
impl Menu {
    fn preview_slot(&self) -> Option<kirby_assist_core::fighter_profiles::presets::PresetSlot> {
        None
    }
}
struct State {
    ledge: Option<LedgeTrap>,
    ledge_position: Option<LedgePosition>,
    ledge_roster: Option<(FightersInGame, i64)>,
    settings_menu: Option<Menu>,
}
static STATE: Mutex<State> = Mutex::new(State {
    ledge: None,
    ledge_position: None,
    ledge_roster: None,
    settings_menu: None,
});
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static SEQUENCES: OnceLock<Mutex<SequenceLibrary>> = OnceLock::new();
static MANAGER_SLOT: AtomicUsize = AtomicUsize::new(0);
mod attack_sources {
    use super::*;
    pub fn sources() -> [Option<u32>; 32] {
        SOURCES.with(|sources| {
            let sources = sources.borrow();
            std::array::from_fn(|i| sources.get(i).map(|a| a.id))
        })
    }
}
#[path = "../../../src/live/frame.rs"]
mod frame;
#[path = "../../../src/live/observations.rs"]
mod observations;
#[path = "../../../src/live/tech.rs"]
mod tech;

#[test]
fn nearby_hitboxes_use_geometry_and_situation_instead_of_root_or_startup() {
    use kirby_assist_core::{autopilot_guard::Envelope, native_states::situation};
    let mut own = Actor::new(1);
    for id in [2, 0x10000002, 0x40000002] {
        for (offset, mask, expected) in [
            ([0., 8., 0.], situation::ATTACK_GROUND_MASK, true),
            ([0., 8., 50.], situation::ATTACK_GROUND_MASK, false),
            ([0., 80., 0.], situation::ATTACK_GROUND_MASK, false),
            ([0., 8., 0.], situation::ATTACK_AIR_MASK, false),
        ] {
            let mut enemy = Box::new(Actor::new(id));
            enemy.geometry.x = offset[0];
            enemy.geometry.y = offset[1];
            enemy.geometry.z = offset[2];
            enemy.geometry.situation = mask;
            SOURCES.with(|s| *s.borrow_mut() = vec![enemy]);
            assert_eq!(
                unsafe {
                    observations::incoming_fighter_attack(
                        &frame::FrameReads::default(),
                        own.object(),
                        1,
                        [0.; 2],
                        [0.; 2],
                        Envelope::default(),
                    )
                },
                expected,
                "source={id:#x} offset={offset:?} mask={mask}"
            );
        }
    }
    SOURCES.with(|s| s.borrow_mut().clear());
}

#[test]
fn attack_geometry_is_copied_before_the_next_native_call() {
    let mut own = Actor::new(1);
    SOURCES.with(|s| *s.borrow_mut() = vec![Box::new(Actor::new(2))]);
    INVALIDATE_GEOMETRY_ON_SCALE.with(|flag| flag.set(true));
    let threat = unsafe {
        observations::incoming_fighter_attack(
            &frame::FrameReads::default(), own.object(), 1, [0.; 2], [0.; 2],
            kirby_assist_core::autopilot_guard::Envelope::default(),
        )
    };
    INVALIDATE_GEOMETRY_ON_SCALE.with(|flag| flag.set(false));
    SOURCES.with(|s| s.borrow_mut().clear());
    assert!(threat);
}

#[test]
fn only_moving_fighters_use_fighter_startup_tables() {
    use kirby_assist_core::autopilot_guard::Envelope;
    let mut own = Actor::new(1);
    for (id, stopped, expected) in [
        (2, false, true),
        (2, true, false),
        (0x10000002, false, false),
        (0x40000002, false, false),
    ] {
        let mut enemy = Box::new(Actor::new(id));
        enemy.active = false;
        enemy.stopped = stopped;
        SOURCES.with(|s| *s.borrow_mut() = vec![enemy]);
        assert_eq!(
            unsafe {
                observations::incoming_fighter_attack(
                    &frame::FrameReads::default(),
                    own.object(),
                    1,
                    [0.; 2],
                    [0.; 2],
                    Envelope::default(),
                )
            },
            expected
        );
    }
    SOURCES.with(|s| s.borrow_mut().clear());
}

#[test]
fn stopped_projectile_keeps_current_geometry_without_projecting_frozen_speed() {
    use kirby_assist_core::autopilot_guard::Envelope;
    let mut own = Actor::new(1);
    for (x, stopped, expected) in [(50., false, true), (50., true, false), (0., true, true)] {
        let mut enemy = Box::new(Actor::new(0x10000002));
        enemy.position[0] = x;
        enemy.velocity[0] = -8.;
        enemy.stopped = stopped;
        SOURCES.with(|s| *s.borrow_mut() = vec![enemy]);
        assert_eq!(
            unsafe {
                observations::incoming_fighter_attack(
                    &frame::FrameReads::default(),
                    own.object(),
                    1,
                    [0.; 2],
                    [0.; 2],
                    Envelope::default(),
                )
            },
            expected
        );
    }
    SOURCES.with(|s| s.borrow_mut().clear());
}

#[test]
fn native_tech_feedback_retains_delivery_and_advances_frozen_pose_lockout() {
    use kirby_assist_core::{
        native_states::{situation, status},
        tech_input::{TechContext, TechInput},
    };
    let mut own = Actor::new(1);
    own.status = status::DAMAGE_FLY;
    own.situation = situation::AIR;
    own.position[1] = 2.;
    own.velocity[1] = -2.;
    let mut input = TechInput::default();
    let sample = |input: &mut TechInput, own: &mut Actor| {
        let assessment = unsafe {
            tech::assess_tech(&frame::FrameReads::default(), own.object(), true, 5., false)
        };
        let (guard, reaction) = unsafe { tech::input_feedback(own.object(), own.status) };
        input.sample_with_native_delivery(
            true,
            true,
            2,
            TechContext {
                object: 1,
                status: own.status,
                motion: 1,
                frame: 0.,
                rate: 0.,
                stopped: false,
                remaining_stop: 0,
                contact_in: assessment.contact_in,
                input_delay: tech::TECH_INPUT_DELAY_FRAMES,
                previous_guard: false,
                prepare_guard: assessment.contact_in.is_some(),
            },
            reaction,
            12,
            7,
            guard,
        )
    };
    assert!(sample(&mut input, &mut own).press);
    input.mapped_press();
    for _ in 0..10 {
        assert!(sample(&mut input, &mut own).press);
    }
    own.guard = 1 << 0x10;
    assert!(!sample(&mut input, &mut own).press);
    own.guard = 0;
    own.reaction -= 1.;
    assert!(!sample(&mut input, &mut own).press);
    own.reaction -= 1.;
    assert!(sample(&mut input, &mut own).press);
    own.air_escape = true;
    assert!(!sample(&mut input, &mut own).press);
    own.air_escape = false;
    own.passive_disabled = true;
    assert!(!sample(&mut input, &mut own).press);
}

#[test]
fn native_hitstop_forecast_includes_frozen_frames_before_contact() {
    use kirby_assist_core::native_states::{situation, status};
    let mut own = Actor::new(1);
    own.status = status::DAMAGE_FLY;
    own.situation = situation::AIR;
    own.position[1] = 2.;
    own.velocity[1] = -2.;
    let assess = |own: &mut Actor| unsafe {
        let stopped = own.stopped;
        tech::assess_tech(
            &frame::FrameReads::default(),
            own.object(),
            true,
            5.,
            stopped,
        )
    };
    let moving = assess(&mut own).contact_in.unwrap();
    own.stopped = true;
    own.remaining_stop = 5;
    assert_eq!(assess(&mut own).contact_in, Some(moving + 5.));
    own.remaining_stop = 20;
    assert_eq!(assess(&mut own).contact_in, None);
}
#[test]
fn real_frame_cache_keeps_sixteen_actor_roster_copies_scoped_to_one_observation() {
    let roster = std::array::from_fn(|i| Some(i as u32));
    STATE.lock().unwrap().ledge_roster = Some((roster, 1));
    let reads = frame::FrameReads::default();
    assert_eq!(reads.roster(), Some(roster));
    STATE.lock().unwrap().ledge_roster = None;
    assert_eq!(reads.roster(), Some(roster));
    assert_eq!(frame::FrameReads::default().roster(), None);
    assert_eq!(reads.known_statuses(), [None; 16]);
}

#[test]
fn capture_stop_link_is_protected_but_partner_tether_does_not_look_like_a_capture() {
    for (links, captured) in [(0, false), (1, true), (1 << 2, true), (1 << 12, false)] {
        LINKS.with(|v| v.set(links));
        LINK_READS.with(|v| v.set(0));
        let mut header = [0u64, 1];
        let object = header.as_mut_ptr().cast::<game::Object>();
        let reads = frame::FrameReads::default();
        for _ in 0..100 {
            assert_eq!(unsafe { reads.capture_link(object) }, captured);
        }
        assert!(
            LINK_READS.with(|v| v.get()) <= 2,
            "same observation caches both capture-link checks"
        );
    }
}
