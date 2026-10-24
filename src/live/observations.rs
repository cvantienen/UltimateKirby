//! Read-only native observations: physics, floor support, roster and opponents.
//!
//! Resolve an object ID to a fresh game accessor before reading it; never store
//! accessor pointers in policy state. Native calls remain outside STATE locks.
//! Opponent get-up timings come from that opponent's native animation metadata;
//! these are separate from our fighter's attack startup and input recipes.
use super::{MANAGER_SLOT, PROFILES, STATE};
use crate::game_ffi as game;
use kirby_assist_core::autopilot_guard::duck::DefenseThreat;
use kirby_assist_core::combat_assist::TripTarget;
use kirby_assist_core::ledge_position::LedgePosition;
use kirby_assist_core::ledge_trap::{
    frames_until, in_tracking_range, LedgeSettings, LedgeTrap, MotionTiming, Opponent,
    CLIFF_CATCH_MOVE, CLIFF_JUMP3,
};
use std::sync::atomic::Ordering;

/// L shield defense uses the cached fighter roster. Held-Y duck defense uses
/// bounded active hitbox geometry, including remembered weapon/item sources.
pub(super) unsafe fn incoming_fighter_attack(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    position: [f32; 2],
    velocity: [f32; 2],
    allow_duck: bool,
) -> DefenseThreat {
    if allow_duck {
        return super::duck_hits::observe(reads, own, own_id, position, velocity);
    }
    let mut threats = DefenseThreat::default();
    let roster = STATE.lock().ok().and_then(|s| s.ledge_roster.map(|r| r.0));
    let Some(roster) = roster else {
        return threats;
    };
    let team = reads.team(own);
    for id in roster.into_iter().flatten() {
        if id == own_id || id >> 28 != 0 || !game::is_active(id) {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let target = [reads.pos_x(enemy), reads.pos_y(enemy)];
        let target_velocity = [reads.speed_x(enemy, -1), reads.speed_y(enemy, -1)];
        if kirby_assist_core::autopilot_guard::approaching_active_attack(
            position,
            velocity,
            target,
            target_velocity,
        ) {
            let active = (0..8).any(|slot| game::attack_active(enemy, slot, false));
            if active && !allow_duck {
                threats.shield = true;
                return threats;
            }
            let status = reads.status_kind(enemy);
            if active {
                threats.shield = true;
                return threats;
            }
            if !matches!(status, 0x2a | 0x2c | 0x32 | 0x36) {
                continue;
            }
            let kind = game::fighter_kind(id);
            let imminent = PROFILES
                .get()
                .and_then(|profiles| profiles.profile(kind))
                .filter(|profile| profile.fighter_kind == kind)
                .is_some_and(|profile| {
                    kirby_assist_core::autopilot_guard::imminent_attack(
                        profile.ledge,
                        status,
                        reads.motion_kind(enemy),
                        reads.motion_frame(enemy),
                        reads.motion_rate(enemy),
                    )
                });
            if imminent {
                threats.shield = true;
                return threats;
            }
        }
    }
    threats
}

// Fighter instance parameters are read only while ledge assist is observing.
// Missing/invalid keys stay None; no character-wide gravity constants are invented.
pub(super) unsafe fn native_air_physics(
    object: *mut game::Object,
) -> Option<kirby_assist_core::ledge_trap::AirPhysics> {
    kirby_assist_core::ledge_trap::AirPhysics::new(
        game::param_float(object, 0xba18057d9, 0),  // air_accel_y
        game::param_float(object, 0x12ec5626fe, 0), // air_speed_y_stable
    )
}

pub(super) unsafe fn native_short_hop_speed(object: *mut game::Object) -> Option<f32> {
    // Hash40("mini_jump_speed_y"), verified against ultimate-research/param-labels.
    let speed = game::param_float(object, 0x112ab24d60, 0);
    (speed.is_finite() && (0.1..=8.0).contains(&speed)).then_some(speed)
}

// Hash40("jump_speed_y"), verified in ultimate-research/param-labels.
pub(super) unsafe fn native_full_hop_speed(object: *mut game::Object) -> Option<f32> {
    let speed = game::param_float(object, 0x0cbed5c24c, 0);
    (speed.is_finite() && (0.1..=8.0).contains(&speed)).then_some(speed)
}

/// Bounded downward rays only while an airborne Kirby starter is acquiring.
pub(super) unsafe fn starter_air_timing(
    object: *mut game::Object,
    position: [f32; 2],
    vy: f32,
) -> Option<kirby_assist_core::finisher::AirTiming> {
    if !position.iter().all(|v| v.is_finite()) {
        return None;
    }
    let origin = game::Vector2 {
        x: position[0],
        y: position[1],
    };
    let hits = |depth: f32| {
        game::ray_check(object, &origin, &game::Vector2 { x: 0.0, y: -depth }, true) != 0
    };
    if !hits(256.0) {
        return None;
    } // No known landing surface: leave recovery physical.
    let (mut lo, mut hi) = (0.0, 256.0);
    for _ in 0..8 {
        let mid = (lo + hi) * 0.5;
        if hits(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let physics = native_air_physics(object)?;
    let dive = game::param_float(
        object,
        kirby_assist_core::timing::motion_hash("dive_speed_y"),
        0,
    );
    kirby_assist_core::finisher::landing_frames(lo, vy, physics.gravity, dive)
        .map(|landing_frames| kirby_assist_core::finisher::AirTiming { landing_frames })
}

// Sample support along each possible ground move. Probe a short swept
// lookahead, including current momentum, to stop walking across a visible edge.
// This is intentionally conservative on sloped/moving/passable platforms.
pub(super) unsafe fn ground_support(
    object: *mut game::Object,
    position: [f32; 2],
    vx: f32,
) -> [bool; 2] {
    // A partially initialized/reset pose must never reach native collision rays.
    if !position
        .iter()
        .chain(std::iter::once(&vx))
        .all(|value| value.is_finite())
    {
        return [false; 2];
    }
    let distance = (vx.abs() * 4.0 + 4.0).clamp(4.0, 16.0);
    [-1.0, 1.0].map(|direction| {
        [1.5, distance].iter().all(|step| {
            let origin = game::Vector2 {
                x: position[0] + direction * step,
                y: position[1] + 2.0,
            };
            let delta = game::Vector2 { x: 0.0, y: -6.0 };
            game::ray_check(object, &origin, &delta, true) != 0
        })
    })
}

// Reuse callback-discovered IDs. Never enumerate the roster from the mapper.
// Trip recognition needs only opponent status. Last-attacker attribution is
// used exclusively for SDI direction, never to gate the trip follow-up.
pub(super) unsafe fn posture_target(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> Option<f32> {
    // Only called when Kirby's optional Dtilt-facing row is enabled. At most
    // eight known fighters; no stage/item scan and no retained native pointer.
    let roster = STATE
        .lock()
        .ok()
        .and_then(|s| s.ledge_roster.map(|r| r.0))?;
    let x = reads.pos_x(own);
    let y = reads.pos_y(own);
    let team = reads.team(own);
    let mut nearest: Option<(f32, f32)> = None;
    for id in roster.into_iter().flatten() {
        if id == own_id || id >> 28 != 0 || !game::is_active(id) {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let dx = reads.pos_x(enemy) - x;
        let dy = reads.pos_y(enemy) - y;
        let distance = dx * dx + dy * dy;
        if dx.abs() > 0.5 && distance.is_finite() && nearest.is_none_or(|old| distance < old.0) {
            nearest = Some((distance, dx.signum()));
        }
    }
    nearest.map(|(_, direction)| direction)
}

/// Fixed roster only; full motion data is read only for damage/tech/get-up states.
pub(super) unsafe fn jab_lock_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> [Option<kirby_assist_core::jab_lock::Target>; 8] {
    let mut targets = [None; 8];
    let Some(roster) = STATE.lock().ok().and_then(|s| s.ledge_roster.map(|r| r.0)) else {
        return targets;
    };
    let team = reads.team(own);
    let entry = game_entry_id(own_id);
    for (slot, id) in roster.into_iter().enumerate() {
        let Some(id) = id.filter(|id| *id != own_id && *id >> 28 == 0 && game::is_active(*id))
        else {
            continue;
        };
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let status = reads.status_kind(enemy);
        if !matches!(status,0x47..=0x54 | 0x57..=0x5b | 0x67..=0x68) {
            continue;
        }
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        let motion = reads.motion_kind(enemy);
        let remaining = kirby_assist_core::ledge_trap::recovery_frames(
            frame,
            reads.motion_end_frame(enemy),
            game::cancel_frame(enemy, motion, false),
            rate,
        );
        let vulnerable_in = if game::hit_status(enemy, 0) == 0 {
            Some(0.0)
        } else {
            let normal = game::normal_hit_frame(enemy, motion, false);
            (normal.is_finite() && normal > frame && rate.is_finite() && rate > 0.0)
                .then(|| (normal - frame) / rate)
        };
        targets[slot] = Some(kirby_assist_core::jab_lock::Target {
            id,
            status,
            frame,
            position: [reads.pos_x(enemy), reads.pos_y(enemy)],
            velocity_x: reads.speed_x(enemy, -1),
            grounded: reads.situation_kind(enemy) == 0,
            // Pinned instance work ID; status-local slots are not read in damage.
            lock_count: game::work_int(enemy, 0x1000006a),
            tech_available: matches!(status, 0x50..=0x54)
                && (game::transition_enabled(enemy, 0x1e000013)
                    || game::transition_enabled(enemy, 0x1e000014)),
            vulnerable_in,
            remaining,
            hit_by_us: (0..8).contains(&entry)
                && game::work_int(enemy, 0x100000b3) == entry
                && matches!(status, 0x47..=0x54 | 0x5a),
        });
    }
    targets
}

pub(super) unsafe fn combat_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    watch_trip: bool,
    watch_sdi: bool,
) -> ([Option<TripTarget>; 8], f32) {
    let roster = STATE
        .lock()
        .ok()
        .and_then(|state| state.ledge_roster.map(|r| r.0));
    let own_x = reads.pos_x(own);
    let mut outward = 0.0;
    let mut opposing_count = 0;
    let mut sole_opponent_direction = 0.0;
    let mut attacker_found = false;
    let mut targets = [None; 8];
    let Some(roster) = roster else {
        return (targets, outward);
    };
    let attacker_entry = if watch_sdi {
        game::work_int(own, 0x100000b3)
    } else {
        -1
    };
    let own_team = reads.team(own);
    for (slot, id) in roster.into_iter().enumerate() {
        let Some(id) = id.filter(|id| *id != own_id && *id >> 28 == 0 && game::is_active(*id))
        else {
            continue;
        };
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == own_team
        {
            continue;
        }
        if watch_sdi {
            let dx = own_x - reads.pos_x(enemy);
            let direction = if dx.is_finite() && dx.abs() > 0.5 {
                dx.signum()
            } else {
                0.0
            };
            opposing_count += 1;
            sole_opponent_direction = direction;
            if game_entry_id(id) == attacker_entry {
                attacker_found = true;
                outward = direction;
            }
        }
        if watch_trip {
            targets[slot] = Some(TripTarget {
                id,
                status: reads.status_kind(enemy),
                previous_status: reads.previous_status(enemy, 0),
            });
        }
    }
    if watch_sdi && !attacker_found && opposing_count == 1 {
        outward = sole_opponent_direction;
    }
    (targets, outward)
}

/// Optional onstage rolls only. Cheap status filtering precedes motion/position
/// reads; existing cached fighter IDs bound this to eight entries, with no items.
pub(super) unsafe fn roll_read_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> [Option<kirby_assist_core::roll_read::Target>; 8] {
    use kirby_assist_core::roll_read::{Target, ESCAPE_B, ESCAPE_F};
    let mut targets = [None; 8];
    let roster = STATE.lock().ok().and_then(|s| s.ledge_roster.map(|r| r.0));
    let Some(roster) = roster else {
        return targets;
    };
    let team = reads.team(own);
    for (slot, id) in roster.into_iter().enumerate() {
        let Some(id) = id.filter(|id| *id != own_id && *id >> 28 == 0 && game::is_active(*id))
        else {
            continue;
        };
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let status = reads.status_kind(enemy);
        if !matches!(status, ESCAPE_F | ESCAPE_B) || reads.situation_kind(enemy) != 0 {
            continue;
        }
        // Pinned ESCAPE work integers reflect this instance's dodge staleness.
        // Use the end of intangibility, even during the roll's vulnerable startup.
        let age = game::work_int(enemy, 0x11000005);
        let normal = game::work_int(enemy, 0x1100000a);
        let opens = if age >= 0 && normal > 0 && normal <= 120 {
            if normal > age {
                Some((normal - age) as f32)
            } else if game::hit_status(enemy, 0) == 0 {
                Some(0.0)
            } else {
                None
            }
        } else {
            None
        };
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        let kind = reads.motion_kind(enemy);
        targets[slot] = Some(Target {
            id,
            status,
            frame,
            rate,
            position: [reads.pos_x(enemy), reads.pos_y(enemy)],
            velocity_x: reads.speed_x(enemy, -1),
            vulnerable_in: opens,
            actionable_in: if kind != 0 {
                kirby_assist_core::ledge_trap::recovery_frames(
                    frame,
                    reads.motion_end_frame(enemy),
                    game::cancel_frame(enemy, kind, false),
                    rate,
                )
            } else {
                None
            },
        });
    }
    targets
}

// Read the roster from the manager instead of waiting for opponents to query
// command flags. IDs are resolved each call; no native pointer is retained.
// `Option<[Option<u32>; 8]>` has two levels: None means no usable roster;
// Some(array) can still contain empty slots. Callers use flatten() to visit
// only present fighter IDs. IDs must be resolved to fresh accessors each time.
unsafe fn fighter_manager_instance() -> Option<*mut game::Object> {
    let slot = MANAGER_SLOT.load(Ordering::Relaxed);
    if slot == 0 {
        return None;
    }
    let manager = (slot as *const *mut game::Object).read();
    (!manager.is_null()).then_some(manager)
}

unsafe fn active_fighter_manager() -> Option<*mut game::Object> {
    let manager = fighter_manager_instance()?;
    // A bounded roster and non-result scene are the mode-independent checks.
    if game::is_result(manager) || !(1..=8).contains(&game::entry_count(manager)) {
        return None;
    }
    Some(manager)
}

/// Menu lifetime only. A retained roster is insufficient on character select:
/// require GO, or let the caller prove a fresh, local ENTRY callback for intro.
/// None rejects missing/result/invalid managers without querying fighter data.
pub(super) unsafe fn battle_menu_ready_go() -> Option<bool> {
    let manager = active_fighter_manager()?;
    Some(game::is_ready_go(manager))
}

pub(super) unsafe fn fighter_ids() -> Option<[Option<u32>; 8]> {
    let manager = active_fighter_manager()?;
    let count = game::entry_count(manager);
    if !(0..=8).contains(&count) {
        return None;
    }
    let mut ids = [None; 8];
    for ordinal in 0..count {
        let entry_id = game::entry_id(manager, ordinal);
        if !(0..8).contains(&entry_id) {
            continue;
        }
        let entry = game::fighter_entry(manager, entry_id) as *mut game::Object;
        if entry.is_null() {
            continue;
        }
        let id = game::current_fighter_id(entry) as u32;
        if id >> 28 == 0 && game::is_active(id) {
            ids[ordinal as usize] = Some(id);
        }
    }
    Some(ids)
}

// Prefer the tracked opponent through its get-up; selection uses the existing
// bounded follow box. All stage preparation queries are ground-only.
pub(super) unsafe fn nearest_ledge_opponent(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    own_id: u32,
    settings: LedgeSettings,
) -> Option<Opponent> {
    let (tracked, locked_spot, roster) = {
        let state = STATE.lock().ok()?;
        (
            state.ledge.as_ref().and_then(LedgeTrap::target),
            state.ledge_position.as_ref().and_then(LedgePosition::spot),
            state.ledge_roster.map(|(ids, _)| ids)?,
        )
    };
    let position = [reads.pos_x(object), reads.pos_y(object)];
    let team = reads.team(object);
    let mut nearest: Option<(f32, Opponent)> = None;
    for id in roster.into_iter().flatten() {
        if id == own_id {
            continue;
        }
        if !game::is_active(id) {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let status = reads.status_kind(enemy);
        let previous_status = reads.previous_status(enemy, 0);
        let on_ledge = (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&status);
        let recent_option = (0x78..=CLIFF_JUMP3).contains(&previous_status)
            || settings.drop_defense
                && reads.situation_kind(enemy) == 2
                && (CLIFF_CATCH_MOVE..=0x77).contains(&previous_status);
        if !on_ledge && !recent_option && tracked != Some(id) {
            continue;
        }
        let vulnerable = game::hit_status(enemy, 0) == 0;
        let cliff_position = if status == 0x77 {
            let p = game::cliff_position(enemy);
            (p.x.is_finite()
                && p.y.is_finite()
                && (p.x - reads.pos_x(enemy)).abs() <= 12.0
                && (p.y - reads.pos_y(enemy)).abs() <= 64.0)
                .then_some([p.x, p.y])
        } else {
            None
        };
        if let (Some(spot), Some(cliff)) = (locked_spot, cliff_position) {
            if (spot.edge[0] - cliff[0]).abs() > 4.0 || (spot.edge[1] - cliff[1]).abs() > 6.0 {
                continue;
            }
        }
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        let kind = reads.motion_kind(enemy);
        let cancel = if (0x78..=0x7d).contains(&status) && kind != 0 {
            game::cancel_frame(enemy, kind, false)
        } else {
            0.0
        };
        let motion =
            (frame.is_finite() && frame >= 0.0 && kind != 0 && rate.is_finite() && rate > 0.0)
                .then(|| MotionTiming {
                    kind,
                    frame,
                    rate,
                    remaining: kirby_assist_core::ledge_trap::recovery_frames(
                        frame,
                        reads.motion_end_frame(enemy),
                        cancel,
                        rate,
                    ),
                });
        // None means unknown, not "vulnerable immediately". Divide animation
        // time by its native rate in frames_until before comparing attack startup.
        let vulnerable_in = if !vulnerable && motion.is_some() && (0x78..=0x7d).contains(&status) {
            frames_until(frame, game::normal_hit_frame(enemy, kind, false), rate).or_else(|| {
                // Grounded ledge options keep an instance-specific normal-hit
                // deadline in their shared cliff status work. Never read this
                // status-local slot from jump/fall, which use different work.
                if (0x78..=0x7a).contains(&status) {
                    frames_until(frame, game::work_float(enemy, 0x1000007), rate)
                } else {
                    None
                }
            })
        } else {
            None
        };
        let opponent = Opponent {
            id,
            status,
            previous_status,
            situation: reads.situation_kind(enemy),
            position: [reads.pos_x(enemy), reads.pos_y(enemy)],
            vulnerable,
            velocity: [reads.speed_x(enemy, -1), reads.speed_y(enemy, -1)],
            cliff_position,
            vulnerable_in,
            motion,
            attack_active: Some((0..8).any(|index| game::attack_active(enemy, index, false))),
            air_physics: if reads.situation_kind(enemy) == 2 && !matches!(status, 0x7b | 0x7c) {
                native_air_physics(enemy)
            } else {
                None
            },
            actionable_in: if status == 0x7d && cancel.is_finite() && cancel > 0.0 {
                if cancel <= frame {
                    Some(0.0)
                } else {
                    frames_until(frame, cancel, rate)
                }
            } else if matches!(status, 0x78..=0x7a) {
                motion.and_then(|motion| motion.remaining)
            } else if matches!(status, 0xb..=0xf | 0x36 | 0x22..=0x23) {
                // These are ordinary actionable air states, not forced ledge recovery.
                Some(0.0)
            } else {
                None
            },
        };
        if !in_tracking_range(settings, position, opponent, tracked == Some(id)) {
            continue;
        }
        let dx = opponent.position[0] - position[0];
        let dy = opponent.position[1] - position[1];
        if tracked == Some(id) {
            return Some(opponent);
        }
        // Squared distance ranks candidates without a square root. A tracked
        // target returned above wins, preventing mid-option target switching.
        let distance = dx * dx + dy * dy;
        if nearest.as_ref().is_none_or(|(best, _)| distance < *best) {
            nearest = Some((distance, opponent));
        }
    }
    nearest.map(|(_, opponent)| opponent)
}

// Input ownership is selected by mapper index, independently of battle slot.
// Only use FighterInformation to reject CPU actors sharing a default input
// index; roster enumeration/current-fighter cross-checks are not prerequisites
// for a human controller's independent tools.
pub(super) unsafe fn human_fighter(object: *mut game::Object) -> bool {
    let Some(manager) = fighter_manager_instance() else {
        return false;
    };
    let id = object.cast::<u8>().add(8).cast::<u32>().read();
    // Resolve an actual entry ID from the object's identity, not the input index.
    let entry = game_entry_id(id);
    if !(0..8).contains(&entry) {
        return false;
    }
    let information = game::fighter_information(manager, entry);
    !information.is_null() && !game::is_cpu(information)
}
extern "C" {
    #[link_name = "\u{1}_ZN3app7fighter20get_fighter_entry_idEj"]
    fn game_entry_id(object_id: u32) -> i32;
}

/// Kirby's held-L recovery reader. Reuse the bounded cached roster only near
/// a locked ledge; no discovery, damage or hitbox scans here.
pub(super) unsafe fn nearest_recovery_opponent(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    new_spot: Option<kirby_assist_core::ledge_position::LedgeSpot>,
) -> Option<kirby_assist_core::two_frame::RecoveryTarget> {
    use kirby_assist_core::two_frame::RecoveryTarget;
    let (tracked, spot, roster) = {
        let state = STATE.lock().ok()?;
        (
            state.two_frame.as_ref().and_then(|p| p.target()),
            new_spot.or_else(|| state.ledge_position.as_ref().and_then(LedgePosition::spot))?,
            state.ledge_roster.map(|r| r.0)?,
        )
    };
    let stand = (reads.pos_x(own) - spot.edge[0]) * spot.inward;
    if !(1.0..=10.0).contains(&stand) || (reads.pos_y(own) - spot.edge[1]).abs() > 4.0 {
        return None;
    }
    let team = reads.team(own);
    let mut nearest: Option<(f32, RecoveryTarget)> = None;
    for id in roster.into_iter().flatten() {
        if id == own_id || !game::is_active(id) {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let position = [reads.pos_x(enemy), reads.pos_y(enemy)];
        if !position.iter().all(|p| p.is_finite()) {
            continue;
        }
        let situation = reads.situation_kind(enemy);
        let status = reads.status_kind(enemy);
        let outside = (spot.edge[0] - position[0]) * spot.inward;
        let height = position[1] - spot.edge[1];
        let near = (-8.0..=90.0).contains(&outside) && (-120.0..=10.0).contains(&height);
        if tracked != Some(id) && (!near || situation != 2 && !(0x75..=0x7d).contains(&status)) {
            continue;
        }
        let target = RecoveryTarget {
            id,
            status,
            previous_status: reads.previous_status(enemy, 0),
            situation,
            position,
            velocity: [reads.speed_x(enemy, -1), reads.speed_y(enemy, -1)],
            stopped: reads.is_stop(enemy),
            vulnerable: game::hit_status(enemy, 0) == 0,
        };
        if tracked == Some(id) {
            return Some(target);
        }
        let distance = outside * outside + height * height;
        if nearest.as_ref().is_none_or(|(best, _)| distance < *best) {
            nearest = Some((distance, target));
        }
    }
    nearest.map(|(_, target)| target)
}

/// Bounded ordinary-grab warning pass: existing roster, status filter first.
/// No hitbox scans, allocated lists or retained game pointers. Command grabs
/// are excluded until their timing/geometry has a verified model.
pub(super) unsafe fn grab_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> [Option<kirby_assist_core::spot_dodge::Target>; 8] {
    let mut targets = [None; 8];
    let Some(roster) = STATE.lock().ok().and_then(|s| s.ledge_roster.map(|r| r.0)) else {
        return targets;
    };
    let Some(profiles) = PROFILES.get() else {
        return targets;
    };
    let team = reads.team(own);
    for (slot, id) in roster.into_iter().enumerate() {
        let Some(id) = id.filter(|id| *id != own_id && *id >> 28 == 0 && game::is_active(*id))
        else {
            continue;
        };
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let status = reads.status_kind(enemy);
        if !matches!(status, 0x37 | 0x39 | 0x3b) || reads.is_stop(enemy) {
            continue;
        }
        let Some(profile) = profiles.profile(game::fighter_kind(id)) else {
            continue;
        };
        let grab = profile.ledge.grab;
        let first = match status {
            0x39 => grab.dash,
            0x3b => grab.turn_or_shield,
            _ => grab.standing,
        };
        targets[slot] = Some(kirby_assist_core::spot_dodge::Target {
            id,
            status,
            motion: reads.motion_kind(enemy),
            frame: reads.motion_frame(enemy),
            rate: reads.motion_rate(enemy),
            position: [reads.pos_x(enemy), reads.pos_y(enemy)],
            velocity: [reads.speed_x(enemy, -1), reads.speed_y(enemy, -1)],
            facing: reads.facing(enemy),
            first,
            last: first.saturating_add(grab.active_frames.saturating_sub(1)),
            reach: grab.standing_range,
        });
    }
    targets
}

pub(super) fn reaction_config(kind: i32) -> Option<kirby_assist_core::option_read::Config> {
    let p = PROFILES.get()?.profile(kind)?;
    let mut moves = p
        .reference()?
        .moves
        .iter()
        .filter(|m| m.name == "Forward Smash");
    let f = moves.next()?;
    if moves.next().is_some() {
        return None;
    }
    let span = f.active.first()?;
    if span.last > 120 {
        return None;
    }
    Some(kirby_assist_core::option_read::Config {
        down: [p.ledge.down_smash.first, p.ledge.down_smash.last],
        forward: [span.first as u8, span.last as u8],
        down_reach: if kind == 6 {
            14.0
        } else if kind == 5 {
            13.0
        } else {
            12.0
        },
        forward_reach: if kind == 6 { 22.0 } else { 21.0 },
    })
}
/// Shares the cached roster and ESCAPE timers with roll reads. No new hitbox scans.
pub(super) unsafe fn reaction_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    spot: bool,
    grab: Option<u32>,
    roll: bool,
) -> [Option<kirby_assist_core::option_read::Target>; 8] {
    use kirby_assist_core::option_read::{Kind, Target};
    let mut out = [None; 8];
    let Some(roster) = STATE.lock().ok().and_then(|s| s.ledge_roster.map(|r| r.0)) else {
        return out;
    };
    let team = reads.team(own);
    for (slot, id) in roster.into_iter().enumerate() {
        let Some(id) = id.filter(|id| *id != own_id && *id >> 28 == 0 && game::is_active(*id))
        else {
            continue;
        };
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || reads.team(enemy) == team
        {
            continue;
        }
        let status = reads.status_kind(enemy);
        let kind = if status == 0x1f && spot {
            Kind::Spot
        } else if matches!(status, 0x20 | 0x21) && roll {
            Kind::Roll
        } else if Some(id) == grab && matches!(status, 0x37 | 0x39 | 0x3b) {
            Kind::Grab
        } else {
            continue;
        };
        if reads.situation_kind(enemy) != 0 || reads.is_stop(enemy) {
            continue;
        }
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        let motion = reads.motion_kind(enemy);
        let Some(closes) = kirby_assist_core::ledge_trap::recovery_frames(
            frame,
            reads.motion_end_frame(enemy),
            game::cancel_frame(enemy, motion, false),
            rate,
        ) else {
            continue;
        };
        let opens = if kind == Kind::Grab {
            let Some(p) = PROFILES
                .get()
                .and_then(|p| p.profile(game::fighter_kind(id)))
            else {
                continue;
            };
            let g = p.ledge.grab;
            let first = match status {
                0x39 => g.dash,
                0x3b => g.turn_or_shield,
                _ => g.standing,
            };
            ((f32::from(first) + f32::from(g.active_frames) - frame) / rate).max(0.0)
        } else {
            let age = game::work_int(enemy, 0x11000005);
            let normal = game::work_int(enemy, 0x1100000a);
            if age < 0 || normal <= 0 || normal > 120 {
                continue;
            }
            if normal > age {
                (normal - age) as f32
            } else if game::hit_status(enemy, 0) == 0 {
                0.0
            } else {
                continue;
            }
        };
        out[slot] = Some(Target {
            id,
            kind,
            status,
            motion,
            frame,
            rate,
            position: [reads.pos_x(enemy), reads.pos_y(enemy)],
            velocity: [reads.speed_x(enemy, -1), reads.speed_y(enemy, -1)],
            opens,
            closes,
        });
    }
    out
}
