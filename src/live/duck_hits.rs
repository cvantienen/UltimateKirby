//! Active hitbox observations for optional grounded Kirby defense. One normal ATTACK
//! command hook remembers object IDs only; it never edits Lua arguments,
//! attacks or statuses. Native geometry is read only while defense is enabled.
use crate::game_ffi as game;
use kirby_assist_core::autopilot_guard::{
    duck::{self, DefenseThreat},
    duck_geometry::{self, Contact, Hitbox},
};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

const EMPTY: u32 = u32::MAX;
const CAPACITY: usize = 32;
static SOURCES: [AtomicU32; CAPACITY] = [const { AtomicU32::new(EMPTY) }; CAPACITY];
static NEXT: AtomicUsize = AtomicUsize::new(0);
static TRACKING: AtomicBool = AtomicBool::new(false);
pub(super) fn tracking(enabled: bool) {
    TRACKING.store(enabled, Ordering::Relaxed);
}

#[skyline::hook(replace = game::attack_command)]
unsafe fn remember_attack(lua_state: u64) {
    original!()(lua_state);
    if !TRACKING.load(Ordering::Relaxed) {
        return;
    }
    let object = game::lua_object(lua_state);
    if object.is_null() {
        return;
    }
    let id = object.cast::<u8>().add(8).cast::<u32>().read();
    // Native category nibbles: Weapon=1, Item=4. Fighters come from the
    // existing roster, so their many hitboxes do not evict projectiles.
    if !matches!(id >> 28, 1 | 4) {
        return;
    }
    if SOURCES
        .iter()
        .any(|slot| slot.load(Ordering::Relaxed) == id)
    {
        return;
    }
    SOURCES[NEXT.fetch_add(1, Ordering::Relaxed) % CAPACITY].store(id, Ordering::Relaxed);
}

pub(super) unsafe fn install() {
    skyline::install_hook!(remember_attack);
}
pub(super) fn clear() {
    tracking(false);
    for slot in &SOURCES {
        slot.store(EMPTY, Ordering::Relaxed);
    }
}

pub(super) unsafe fn observe(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    position: [f32; 2],
    velocity: [f32; 2],
) -> DefenseThreat {
    let mut threats = DefenseThreat::default();
    // Crouching on a passable platform can drop through it. Nonstandard sizes
    // cannot use the calibrated body envelope.
    if game::passable_ground(own) || !(0.99..=1.01).contains(&game::scale(own)) {
        return threats;
    }
    let origin = [position[0], position[1], game::pos_z(own)];
    let team = reads.team(own);
    let roster = super::STATE
        .lock()
        .ok()
        .and_then(|s| s.ledge_roster.map(|r| r.0));
    let fighters = roster.unwrap_or([None; 8]);
    let mut active_slots = 0;
    for id in fighters
        .into_iter()
        .flatten()
        .chain(SOURCES.iter().filter_map(|slot| {
            let id = slot.load(Ordering::Relaxed);
            (id != EMPTY).then_some(id)
        }))
    {
        if id == own_id || !game::is_active(id) {
            continue;
        }
        let object = game::module_accessor(id);
        if object.is_null()
            || object.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(object) == team
        {
            continue;
        }
        let root = [reads.pos_x(object), reads.pos_y(object)];
        // Cheap prefilter before up to eight native slot/joint queries. A
        // projectile can travel up to six frames; its speed expands the radius.
        let speed = [reads.speed_x(object, -1), reads.speed_y(object, -1)];
        if !root.iter().chain(speed.iter()).all(|v| v.is_finite())
            || (root[0] - position[0]).abs() > 96.0 + (speed[0] - velocity[0]).abs() * 6.0
            || (root[1] - position[1]).abs() > 96.0 + (speed[1] - velocity[1]).abs() * 6.0
        {
            continue;
        }
        let scale = game::scale(object);
        if !scale.is_finite() || !(0.01..=10.0).contains(&scale) {
            continue;
        }
        for slot in 0..8 {
            if !game::attack_active(object, slot, false) {
                continue;
            }
            active_slots += 1;
            if active_slots > 32 {
                // Bound expensive geometry transforms in crowded battles.
                // Unexamined attacks must not be treated as safe to duck.
                threats.shield = true;
                return threats;
            }
            let data = game::attack_geometry(object, slot, false);
            if data.is_null() {
                continue;
            }
            let data = data.read();
            if data.situation & 1 == 0 || !data.size.is_finite() || data.size <= 0.0 {
                continue;
            }
            let local = game::Vector3 {
                x: data.x,
                y: data.y,
                z: data.z,
            };
            if ![data.x, data.y, data.z, data.x2, data.y2, data.z2]
                .iter()
                .all(|v| v.is_finite())
            {
                continue;
            }
            let mut start = game::Vector3 {
                x: f32::NAN,
                y: f32::NAN,
                z: f32::NAN,
            };
            game::joint_position(object, data.node, &local, &mut start, true);
            let mut end = game::Vector3 {
                x: start.x,
                y: start.y,
                z: start.z,
            };
            if data.x2 != 0.0 || data.y2 != 0.0 || data.z2 != 0.0 {
                let offset = game::Vector3 {
                    x: data.x2,
                    y: data.y2,
                    z: data.z2,
                };
                game::joint_position(object, data.node, &offset, &mut end, true);
            }
            match duck_geometry::contact(
                origin,
                velocity,
                Hitbox {
                    start: [start.x, start.y, start.z],
                    end: [end.x, end.y, end.z],
                    radius: data.size * scale,
                    velocity: speed,
                },
            ) {
                Contact::None => {}
                Contact::Duck(lead) => {
                    threats.add_duck(duck::geometry_threat(id, reads.motion_frame(object), lead))
                }
                Contact::Shield => {
                    threats.shield = true;
                    return threats;
                }
            }
        }
    }
    threats
}
