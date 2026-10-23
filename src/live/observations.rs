//! Read-only native observations: physics, floor support, roster and opponents.
//!
//! Resolve an object ID to a fresh game accessor before reading it; never store
//! accessor pointers in policy state. Native calls remain outside STATE locks.
//! Opponent get-up timings come from that opponent's native animation metadata;
//! these are separate from our fighter's attack startup and input recipes.
use super::{MANAGER_SLOT, STATE};
use crate::game_ffi as game;
use kirby_assist_core::combat_assist::TripTarget;
use kirby_assist_core::ledge_position::LedgePosition;
use kirby_assist_core::ledge_trap::{
    frames_until, in_tracking_range, LedgeSettings, LedgeTrap, MotionTiming, Opponent,
    CLIFF_CATCH_MOVE, CLIFF_JUMP3,
};
use std::sync::atomic::Ordering;

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
pub(super) unsafe fn combat_targets(
    own: *mut game::Object,
    own_id: u32,
    watch_trip: bool,
    watch_sdi: bool,
) -> ([Option<TripTarget>; 8], f32) {
    let roster = STATE
        .lock()
        .ok()
        .and_then(|state| state.ledge_roster.map(|r| r.0));
    let own_x = game::pos_x(own);
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
    let own_team = game::team(own);
    for (slot, id) in roster.into_iter().enumerate() {
        let Some(id) = id.filter(|id| *id != own_id && *id >> 28 == 0 && game::is_active(*id))
        else {
            continue;
        };
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || game::team(enemy) == own_team
        {
            continue;
        }
        if watch_sdi {
            let dx = own_x - game::pos_x(enemy);
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
                status: game::status_kind(enemy),
            });
        }
    }
    if watch_sdi && !attacker_found && opposing_count == 1 {
        outward = sole_opponent_direction;
    }
    (targets, outward)
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
    let position = [game::pos_x(object), game::pos_y(object)];
    let team = game::team(object);
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
            || game::team(enemy) == team
        {
            continue;
        }
        let status = game::status_kind(enemy);
        let previous_status = game::previous_status(enemy, 0);
        let on_ledge = (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&status);
        let recent_option = (0x78..=CLIFF_JUMP3).contains(&previous_status)
            || settings.drop_defense
                && game::situation_kind(enemy) == 2
                && (CLIFF_CATCH_MOVE..=0x77).contains(&previous_status);
        if !on_ledge && !recent_option && tracked != Some(id) {
            continue;
        }
        let vulnerable = game::hit_status(enemy, 0) == 0;
        let cliff_position = if status == 0x77 {
            let p = game::cliff_position(enemy);
            (p.x.is_finite()
                && p.y.is_finite()
                && (p.x - game::pos_x(enemy)).abs() <= 12.0
                && (p.y - game::pos_y(enemy)).abs() <= 64.0)
                .then_some([p.x, p.y])
        } else {
            None
        };
        if let (Some(spot), Some(cliff)) = (locked_spot, cliff_position) {
            if (spot.edge[0] - cliff[0]).abs() > 4.0 || (spot.edge[1] - cliff[1]).abs() > 6.0 {
                continue;
            }
        }
        let frame = game::motion_frame(enemy);
        let rate = game::motion_rate(enemy);
        let kind = game::motion_kind(enemy);
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
                        game::motion_end_frame(enemy),
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
            situation: game::situation_kind(enemy),
            position: [game::pos_x(enemy), game::pos_y(enemy)],
            vulnerable,
            velocity: [game::speed_x(enemy, -1), game::speed_y(enemy, -1)],
            cliff_position,
            vulnerable_in,
            motion,
            attack_active: Some((0..8).any(|index| game::attack_active(enemy, index, false))),
            air_physics: if game::situation_kind(enemy) == 2 && !matches!(status, 0x7b | 0x7c) {
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

/// Recovery-only reader used exclusively while the selected two-frame mode is
/// held. Reuse the bounded cached roster; no discovery, damage or hitbox scans.
pub(super) unsafe fn nearest_recovery_opponent(
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
    let team = game::team(own);
    let mut nearest: Option<(f32, RecoveryTarget)> = None;
    for id in roster.into_iter().flatten() {
        if id == own_id || !game::is_active(id) {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != id
            || game::team(enemy) == team
        {
            continue;
        }
        let position = [game::pos_x(enemy), game::pos_y(enemy)];
        if !position.iter().all(|p| p.is_finite()) {
            continue;
        }
        let situation = game::situation_kind(enemy);
        let status = game::status_kind(enemy);
        let outside = (spot.edge[0] - position[0]) * spot.inward;
        let height = position[1] - spot.edge[1];
        let near = (-8.0..=90.0).contains(&outside) && (-120.0..=10.0).contains(&height);
        if tracked != Some(id) && (!near || situation != 2 && !(0x75..=0x7d).contains(&status)) {
            continue;
        }
        let target = RecoveryTarget {
            id,
            status,
            previous_status: game::previous_status(enemy, 0),
            situation,
            position,
            velocity: [game::speed_x(enemy, -1), game::speed_y(enemy, -1)],
            stopped: game::is_stop(enemy),
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
