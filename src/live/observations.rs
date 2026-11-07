//! Read-only native observations: physics, floor support, roster and opponents.
//!
//! Resolve an object ID to a fresh game accessor before reading it; never store
//! accessor pointers in policy state. Native calls remain outside STATE locks.
//! Opponent get-up timings come from that opponent's native animation metadata;
//! these are separate from our fighter's attack startup and input recipes.
use super::{MANAGER_SLOT, STATE};
use crate::game_ffi as game;
use kirby_assist_core::autopilot_guard::duck::DefenseThreat;
use kirby_assist_core::combat_assist::TripTarget;
use kirby_assist_core::gameplay::{frames_until, MotionTiming, Opponent};
use kirby_assist_core::ledge_position::LedgePosition;
use kirby_assist_core::ledge_trap::{
    in_tracking_range, LedgeSettings, LedgeTrap, CLIFF_CATCH_MOVE, CLIFF_JUMP3,
};
use std::sync::atomic::Ordering;

/// L shield defense uses the cached roster and bounded remembered weapon/item
/// sources. Held-Y duck defense uses its separately calibrated body geometry.
pub(super) unsafe fn incoming_fighter_attack(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    position: [f32; 2],
    velocity: [f32; 2],
    allow_duck: bool,
    envelope: kirby_assist_core::autopilot_guard::Envelope,
) -> DefenseThreat {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Shield);
    if allow_duck {
        return super::duck_hits::observe(reads, own, own_id, position, velocity);
    }
    let mut threats = DefenseThreat::default();
    let roster = reads.roster();
    let roster = roster.unwrap_or([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    let team = reads.team(own);
    let mut geometry_budget = 16;
    let mut own_z = None;
    for id in roster
        .into_iter()
        .chain(super::duck_hits::sources())
        .flatten()
    {
        if id == own_id
            || !matches!(
                id >> 28,
                kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
                    | kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
                    | kirby_assist_core::battle_objects::OBJECT_CATEGORY_ITEM
            )
            || !game::is_active(id)
        {
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
        if !target
            .iter()
            .chain(target_velocity.iter())
            .all(|x| x.is_finite())
            || (target[0] - position[0]).abs()
                > envelope.scan_x.max(envelope.warning_x)
                    + (target_velocity[0] - velocity[0]).abs() * envelope.forecast
            || (target[1] - position[1]).abs()
                > envelope.scan_y.max(envelope.warning_y)
                    + (target_velocity[1] - velocity[1]).abs() * envelope.forecast
        {
            continue;
        }
        let near = kirby_assist_core::autopilot_guard::approaching_active_attack_with(
            position,
            velocity,
            target,
            target_velocity,
            envelope,
        );
        if id >> 28 == kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER {
            let status = reads.status_kind(enemy);
            if status >= kirby_assist_core::native_states::status::SPECIAL_HI {
                let previous = if status >= kirby_assist_core::native_states::status::FINAL {
                    reads.previous_status(enemy, 0)
                } else {
                    0
                };
                if kirby_assist_core::autopilot_guard::recovery_warning(
                    reads.fighter_kind(enemy),
                    status,
                    previous,
                    position,
                    velocity,
                    target,
                    target_velocity,
                    envelope,
                ) {
                    threats.shield = true;
                    return threats;
                }
            }
        }
        // Near attacks return immediately and idle opponents have no geometry
        // to transform. Read scale only for an active geometry candidate.
        let mut scale = None;
        for slot in 0..8 {
            if !game::attack_active(enemy, slot, false) {
                continue;
            }
            if near {
                threats.shield = true;
                return threats;
            }
            if geometry_budget == 0 {
                threats.shield = true;
                return threats;
            }
            geometry_budget -= 1;
            let data = game::attack_geometry(enemy, slot, false);
            if data.is_null() {
                continue;
            }
            let scale = *scale.get_or_insert_with(|| game::scale(enemy));
            if !scale.is_finite() || !(0.01..=10.0).contains(&scale) {
                continue;
            }
            let data = data.read();
            if data.situation & 1 == 0
                || ![data.x, data.y, data.z, data.x2, data.y2, data.z2, data.size]
                    .iter()
                    .all(|x| x.is_finite())
                || data.size <= 0.0
            {
                continue;
            }
            let mut from = game::Vector3 {
                x: f32::NAN,
                y: f32::NAN,
                z: f32::NAN,
            };
            let local = game::Vector3 {
                x: data.x,
                y: data.y,
                z: data.z,
            };
            game::joint_position(enemy, data.node, &local, &mut from, true);
            let mut to = game::Vector3 {
                x: from.x,
                y: from.y,
                z: from.z,
            };
            if data.x2 != 0.0 || data.y2 != 0.0 || data.z2 != 0.0 {
                let local = game::Vector3 {
                    x: data.x2,
                    y: data.y2,
                    z: data.z2,
                };
                game::joint_position(enemy, data.node, &local, &mut to, true);
            }
            let z = *own_z.get_or_insert_with(|| game::pos_z(own));
            if kirby_assist_core::autopilot_guard::active_hitbox_threat_with(
                [position[0], position[1], z],
                velocity,
                kirby_assist_core::autopilot_guard::duck_geometry::Hitbox {
                    start: [from.x, from.y, from.z],
                    end: [to.x, to.y, to.z],
                    radius: data.size * scale,
                    velocity: target_velocity,
                },
                envelope,
            ) {
                threats.shield = true;
                return threats;
            }
        }
        if !near || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER {
            continue;
        }
        let kind = reads.fighter_kind(enemy);
        let motion = reads.motion_kind(enemy);
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        let imminent = kirby_assist_core::autopilot_guard::referenced_imminent_attack_with(
            kind,
            motion,
            frame,
            rate,
            envelope.forecast,
        );
        if imminent {
            threats.shield = true;
            return threats;
        }
    }
    threats
}

// Fighter instance parameters are read only while ledge assist is observing.
// Missing/invalid keys stay None; no character-wide gravity constants are invented.
pub(super) unsafe fn native_air_physics(
    object: *mut game::Object,
) -> Option<kirby_assist_core::gameplay::AirPhysics> {
    kirby_assist_core::gameplay::AirPhysics::new(
        game::param_float(object, 0xba18057d9, 0),  // air_accel_y
        game::param_float(object, 0x12ec5626fe, 0), // air_speed_y_stable
    )
}

// Sample support along each possible ground move. Probe a short swept
// lookahead, including current momentum, to stop walking across a visible edge.
// This is intentionally conservative on sloped/moving/passable platforms.
pub(super) unsafe fn ground_support(
    object: *mut game::Object,
    position: [f32; 2],
    vx: f32,
) -> [bool; 2] {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Ground);
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

// Reuse callback-discovered IDs; never enumerate a roster from the mapper.
pub(super) unsafe fn posture_opponent(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> Option<[f32; 2]> {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Posture);
    // Shared by optional crouch proximity and Dtilt facing. At most
    // eight known fighters; no stage/item scan and no retained native pointer.
    let roster = reads.roster()?;
    let x = reads.pos_x(own);
    let y = reads.pos_y(own);
    let team = reads.team(own);
    let mut nearest: Option<(f32, [f32; 2])> = None;
    for id in roster.into_iter().flatten() {
        if id == own_id
            || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
            || !game::is_active(id)
        {
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
        if distance.is_finite() && nearest.is_none_or(|old| distance < old.0) {
            nearest = Some((distance, [dx, dy]));
        }
    }
    nearest.map(|(_, offset)| offset)
}

/// Fixed roster only; full motion data is read only for damage/tech/get-up states.
pub(super) unsafe fn jab_lock_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    include_damage: bool,
    chase_distance: f32,
    chase_height: f32,
) -> [Option<kirby_assist_core::jab_lock::Target>; 8] {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Jab);
    let mut targets = [None; 8];
    let Some(roster) = reads.roster() else {
        return targets;
    };
    let own_position = [reads.pos_x(own), reads.pos_y(own)];
    let team = reads.team(own);
    let entry = game_entry_id(own_id);
    for id in roster.into_iter() {
        let Some(id) = id.filter(|id| {
            *id != own_id
                && *id >> 28 == kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
                && game::is_active(*id)
        }) else {
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
        let candidate = kirby_assist_core::jab_lock::tech_chase_status(status)
            || include_damage
                && matches!(status,kirby_assist_core::native_states::status::DAMAGE..=kirby_assist_core::native_states::status::DOWN_WAIT_CONTINUE | kirby_assist_core::native_states::status::DOWN_STAND..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR | kirby_assist_core::native_states::status::PASSIVE..=kirby_assist_core::native_states::status::PASSIVE_FB);
        if !candidate {
            continue;
        }
        let position = [reads.pos_x(enemy), reads.pos_y(enemy)];
        if !position.iter().all(|v| v.is_finite())
            || (position[0] - own_position[0]).abs() > chase_distance
            || (position[1] - own_position[1]).abs() > chase_height
        {
            continue;
        }
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        let motion = reads.motion_kind(enemy);
        let remaining = kirby_assist_core::gameplay::recovery_frames(
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
        kirby_assist_core::battle_objects::insert_nearest(
            &mut targets,
            kirby_assist_core::jab_lock::Target {
                id,
                status,
                frame,
                position,
                velocity_x: reads.speed_x(enemy, -1),
                grounded: reads.situation_kind(enemy) == 0,
                // Pinned instance work ID; status-local slots are not read in damage.
                lock_count: game::work_int(enemy, kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_INT_DOWN_DAMAGE_COUNT),
                tech_available: matches!(
                    status,
                    kirby_assist_core::native_states::status::DOWN
                        ..=kirby_assist_core::native_states::status::DOWN_WAIT_CONTINUE
                ) && (game::transition_enabled(enemy, kirby_assist_core::native_states::transition_terms::PASSIVE)
                    || game::transition_enabled(enemy, kirby_assist_core::native_states::transition_terms::PASSIVE_FB)),
                vulnerable_in,
                remaining,
                hit_by_us: (0..8).contains(&entry)
                    && game::work_int(enemy, kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID) == entry
                    && matches!(
                        status,
                        kirby_assist_core::native_states::status::DAMAGE
                            ..=kirby_assist_core::native_states::status::DOWN_WAIT_CONTINUE
                                | kirby_assist_core::native_states::status::DOWN_DAMAGE
                    ),
            },
            |target| {
                kirby_assist_core::battle_objects::distance_squared(own_position, target.position)
            },
        );
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
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Trip);
    let roster = reads.roster();
    let own_x = reads.pos_x(own);
    let mut outward = 0.0;
    let mut opposing_count = 0;
    let mut sole_opponent_direction = 0.0;
    let mut attacker_found = false;
    let mut attacker_direction = kirby_assist_core::battle_objects::EntryDirection::default();
    let mut targets = [None; 8];
    let Some(roster) = roster else {
        return (targets, outward);
    };
    let attacker_entry = if watch_sdi {
        game::work_int(own, kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID)
    } else {
        -1
    };
    let own_team = reads.team(own);
    for id in roster.into_iter() {
        let Some(id) = id.filter(|id| {
            *id != own_id
                && *id >> 28 == kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
                && game::is_active(*id)
        }) else {
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
                attacker_direction.observe(direction);
                outward = attacker_direction.direction().unwrap_or(0.0);
            }
        }
        if watch_trip {
            if let Some(slot) = targets.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(TripTarget {
                    id,
                    status: reads.status_kind(enemy),
                    previous_status: reads.previous_status(enemy, 0),
                });
            }
        }
    }
    if watch_sdi && !attacker_found && opposing_count == 1 {
        outward = sole_opponent_direction;
    }
    (targets, outward)
}

/// Optional onstage reactions, front grabs and either-side shield fallbacks. Status and
/// spatial filters precede detailed reads; sixteen copied actor IDs bound the pass.
pub(super) unsafe fn rule_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    follow: Option<u32>,
    grab_range: Option<[f32; 2]>,
    shield_fallback: bool,
    watch_events: bool,
    spin_follow: bool,
) -> (
    [Option<kirby_assist_core::roll_read::Target>; 8],
    Option<Opponent>,
) {
    #[cfg(feature = "diagnostic-logging")]
    let _scope = crate::performance::Scope::new(crate::performance::Part::Roll);
    let mut targets = [None; 8];
    let mut following = None;
    // A committed response follows only its locked actor during recovery.
    // It never searches other fighters for a fresh punish in that state.
    if !watch_events && grab_range.is_none() {
        return (
            targets,
            follow.and_then(|id| rule_follow(reads, own, own_id, id, spin_follow)),
        );
    }

    let Some(roster) = reads.roster() else {
        return (targets, following);
    };
    let team = reads.team(own);
    let own_position = [reads.pos_x(own), reads.pos_y(own)];
    let grab_from = grab_range
        .filter(|r| r[0].is_finite() && r[0] > 0. && r[1].is_finite() && r[1] >= 0.)
        .map(|r| (own_position, reads.facing(own), r));
    for id in roster.into_iter() {
        let Some(id) = id.filter(|id| {
            *id != own_id
                && *id >> 28 == kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
                && game::is_active(*id)
        }) else {
            continue;
        };
        let e = game::module_accessor(id);
        if e.is_null() || e.cast::<u8>().add(8).cast::<u32>().read() != id || reads.team(e) == team
        {
            continue;
        }
        let status = reads.status_kind(e);
        if status >= kirby_assist_core::native_states::status::FINAL
            && kirby_assist_core::native_states::fighter_noncombat(reads.fighter_kind(e), status)
        {
            continue;
        }
        let available = kirby_assist_core::reaction_rules::automatic_target_available(status);
        if !available
            && !(spin_follow
                && follow == Some(id)
                && kirby_assist_core::reaction_rules::automatic_tracking_target_available(status))
        {
            continue;
        }
        let event = available
            && watch_events
            && matches!(
                status,
                kirby_assist_core::native_states::ESCAPE
                    ..=kirby_assist_core::native_states::ESCAPE_AIR_SLIDE
            );
        let grab = available
            && grab_from.is_some()
            && if shield_fallback {
                kirby_assist_core::out_of_shield::automatic_shield_target_status(status)
            } else {
                kirby_assist_core::out_of_shield::automatic_grab_target_status(status)
            };
        if !grab && !event && follow != Some(id) {
            continue;
        }
        if reads.capture_link(e) {
            continue;
        }
        let situation = reads.situation_kind(e);
        let eligible_event = event
            && !((kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE
                ..=kirby_assist_core::native_states::status::CLIFF_JUMP3)
                .contains(&reads.previous_status(e, 0))
                || matches!(
                    status,
                    kirby_assist_core::native_states::ESCAPE
                        ..=kirby_assist_core::native_states::ESCAPE_B
                ) && situation != 0
                || matches!(
                    status,
                    kirby_assist_core::native_states::ESCAPE_AIR
                        | kirby_assist_core::native_states::ESCAPE_AIR_SLIDE
                ) && situation != 2);
        let position = [reads.pos_x(e), reads.pos_y(e)];
        let eligible_grab =
            grab && grab_from.is_some_and(|(from, facing, range)| {
                facing.is_finite()
                    && facing.abs() >= 0.5
                    && (shield_fallback || (position[0] - from[0]) * facing > 0.)
                    && (position[0] - from[0]).abs() <= range[0]
                    && (position[1] - from[1]).abs() <= range[1]
            }) && game::hit_status(e, 0) == 0
                && !(event
                    && (kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE
                        ..=kirby_assist_core::native_states::status::CLIFF_JUMP3)
                        .contains(&reads.previous_status(e, 0)));
        if !eligible_event && !eligible_grab && follow != Some(id) {
            continue;
        }
        let motion = MotionTiming {
            kind: reads.motion_kind(e),
            frame: reads.motion_frame(e),
            rate: reads.motion_rate(e),
            remaining: None,
        };
        // Only current grab candidates need a velocity read for startup reach.
        let velocity_x = if eligible_grab {
            reads.speed_x(e, -1)
        } else {
            0.
        };
        if follow == Some(id) {
            following = Some(Opponent {
                id,
                status,
                situation,
                position,
                velocity: [velocity_x, 0.],
                motion: Some(motion),
                vulnerable: eligible_grab,
                ..Default::default()
            });
        }
        if !eligible_event && !eligible_grab {
            continue;
        }
        kirby_assist_core::battle_objects::insert_nearest(
            &mut targets,
            kirby_assist_core::roll_read::Target {
                id,
                status,
                motion: motion.kind,
                frame: motion.frame,
                rate: motion.rate,
                position,
                velocity_x,
                vulnerable_in: eligible_grab.then_some(0.),
                actionable_in: None,
            },
            |target| {
                kirby_assist_core::battle_objects::distance_squared(own_position, target.position)
            },
        );
    }
    (targets, following)
}

unsafe fn rule_follow(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    id: u32,
    spin: bool,
) -> Option<Opponent> {
    if id == own_id
        || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(id)
    {
        return None;
    }
    let e = game::module_accessor(id);
    if e.is_null()
        || e.cast::<u8>().add(8).cast::<u32>().read() != id
        || reads.team(e) == reads.team(own)
    {
        return None;
    }
    let status = reads.status_kind(e);
    if status >= kirby_assist_core::native_states::status::FINAL
        && kirby_assist_core::native_states::fighter_noncombat(reads.fighter_kind(e), status)
    {
        return None;
    }
    if !(if spin {
        kirby_assist_core::reaction_rules::automatic_tracking_target_available(status)
    } else {
        kirby_assist_core::reaction_rules::automatic_target_available(status)
    }) || reads.capture_link(e)
    {
        return None;
    }
    Some(Opponent {
        id,
        status,
        situation: reads.situation_kind(e),
        position: [reads.pos_x(e), reads.pos_y(e)],
        motion: Some(MotionTiming {
            kind: reads.motion_kind(e),
            frame: reads.motion_frame(e),
            rate: reads.motion_rate(e),
            remaining: None,
        }),
        ..Default::default()
    })
}

// Read the roster from the manager instead of waiting for opponents to query
// command flags. IDs are resolved each call; no native pointer is retained.
// `Option<FighterRoster>` has two levels: None means no usable roster;
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
#[path = "fighter_roster.rs"]
mod fighter_roster;
pub(super) use fighter_roster::fighter_ids;

// Prefer the tracked opponent through its get-up; selection uses the existing
// bounded follow box. All stage preparation queries are ground-only.
/// Clock-only observation for a committed literal tape whose own pose has zero
/// motion rate or a held edge-balancing pose. Read the already locked actor; no roster scan, hitbox/physics
/// lookup, ray query or action-readiness query. Never retain its native pointer.
pub(super) unsafe fn tracked_ledge_clock(reads: &super::frame::FrameReads) -> Option<Opponent> {
    let id = STATE.lock().ok()?.ledge.as_ref()?.observation_target()?;
    if !game::is_active(id) {
        return None;
    }
    let object = game::module_accessor(id);
    if object.is_null() || object.cast::<u8>().add(8).cast::<u32>().read() != id {
        return None;
    }
    let motion = MotionTiming {
        kind: reads.motion_kind(object),
        frame: reads.motion_frame(object),
        rate: reads.motion_rate(object),
        remaining: None,
    };
    let motion = (motion.kind != 0
        && motion.frame.is_finite()
        && motion.frame >= 0.0
        && motion.rate.is_finite()
        && motion.rate > 0.0)
        .then_some(motion);
    let status = reads.status_kind(object);
    let cliff_position = if status == kirby_assist_core::ledge_trap::CLIFF_WAIT {
        let p = game::cliff_position(object);
        (p.x.is_finite()
            && p.y.is_finite()
            && (p.x - reads.pos_x(object)).abs() <= 12.0
            && (p.y - reads.pos_y(object)).abs() <= 64.0)
            .then_some([p.x, p.y])
    } else {
        None
    };
    Some(Opponent {
        id,
        status,
        previous_status: reads.previous_status(object, 0),
        cliff_position,
        situation: reads.situation_kind(object),
        // One read on the already locked actor, only while actually at ledge.
        vulnerable: (kirby_assist_core::ledge_trap::CLIFF_CATCH_MOVE
            ..=kirby_assist_core::ledge_trap::CLIFF_WAIT)
            .contains(&status)
            && game::hit_status(object, 0) == 0,
        position: [reads.pos_x(object), reads.pos_y(object)],
        motion,
        ..Opponent::default()
    })
}

pub(super) unsafe fn nearest_ledge_opponent(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    own_id: u32,
    settings: LedgeSettings,
) -> Option<Opponent> {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Ledge);
    let (tracked, locked_spot, roster, preview) = {
        let state = STATE.lock().ok()?;
        (
            state.ledge.as_ref().and_then(LedgeTrap::observation_target),
            state.ledge_position.as_ref().and_then(LedgePosition::spot),
            state.ledge_roster.map(|(ids, _)| ids)?,
            state
                .ledge_menu
                .as_ref()
                .is_some_and(|m| m.preview_slot().is_some()),
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
        let recent_option = (kirby_assist_core::native_states::status::CLIFF_ATTACK..=CLIFF_JUMP3)
            .contains(&previous_status)
            || settings.drop_defense
                && reads.situation_kind(enemy) == 2
                && (CLIFF_CATCH_MOVE..=kirby_assist_core::native_states::status::CLIFF_WAIT)
                    .contains(&previous_status);
        if !preview && !on_ledge && !recent_option && tracked != Some(id) {
            continue;
        }
        let vulnerable = game::hit_status(enemy, 0) == 0;
        let cliff_position = if status == kirby_assist_core::native_states::status::CLIFF_WAIT {
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
        let cancel = if (kirby_assist_core::native_states::status::CLIFF_ATTACK
            ..=kirby_assist_core::native_states::status::CLIFF_JUMP3)
            .contains(&status)
            && kind != 0
        {
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
                    remaining: kirby_assist_core::gameplay::recovery_frames(
                        frame,
                        reads.motion_end_frame(enemy),
                        cancel,
                        rate,
                    ),
                });
        // None means unknown, not "vulnerable immediately". Divide animation
        // time by its native rate in frames_until before comparing attack startup.
        let vulnerable_in = if !vulnerable
            && motion.is_some()
            && (kirby_assist_core::native_states::status::CLIFF_ATTACK
                ..=kirby_assist_core::native_states::status::CLIFF_JUMP3)
                .contains(&status)
        {
            frames_until(frame, game::normal_hit_frame(enemy, kind, false), rate).or_else(|| {
                // Grounded ledge options keep an instance-specific normal-hit
                // deadline in their shared cliff status work. Never read this
                // status-local slot from jump/fall, which use different work.
                if (kirby_assist_core::native_states::status::CLIFF_ATTACK
                    ..=kirby_assist_core::native_states::status::CLIFF_ESCAPE)
                    .contains(&status)
                {
                    frames_until(frame, game::work_float(enemy, kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME), rate)
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
            air_physics: if reads.situation_kind(enemy) == 2
                && !matches!(
                    status,
                    kirby_assist_core::native_states::status::CLIFF_JUMP1
                        | kirby_assist_core::native_states::status::CLIFF_JUMP2
                ) {
                native_air_physics(enemy)
            } else {
                None
            },
            actionable_in: if status == kirby_assist_core::native_states::status::CLIFF_JUMP3
                && cancel.is_finite()
                && cancel > 0.0
            {
                if cancel <= frame {
                    Some(0.0)
                } else {
                    frames_until(frame, cancel, rate)
                }
            } else if matches!(
                status,
                kirby_assist_core::native_states::status::CLIFF_ATTACK
                    ..=kirby_assist_core::native_states::status::CLIFF_ESCAPE
            ) {
                motion.and_then(|motion| motion.remaining)
            } else if matches!(
                status,
                kirby_assist_core::native_states::status::JUMP..=kirby_assist_core::native_states::status::FALL_AERIAL
                    | kirby_assist_core::native_states::status::ATTACK_AIR
                    | kirby_assist_core::native_states::ESCAPE_AIR
                        ..=kirby_assist_core::native_states::ESCAPE_AIR_SLIDE
            ) {
                // These are ordinary actionable air states, not forced ledge recovery.
                Some(0.0)
            } else {
                None
            },
        };
        if !preview && !in_tracking_range(settings, position, opponent, tracked == Some(id)) {
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

// Entry membership alone does not prove that this actor owns local input.
#[path = "local_actor.rs"]
mod local_actor;
pub(super) use local_actor::human_fighter;
#[path = "local_route.rs"]
mod local_route;
pub(super) use local_route::valid_local_route;
extern "C" {
    #[link_name = "\u{1}_ZN3app7fighter20get_fighter_entry_idEj"]
    fn game_entry_id(object_id: u32) -> i32;
}

/// Bounded ordinary-grab warning pass: existing roster, status filter first.
/// No hitbox scans, allocated lists or retained game pointers. Command grabs
/// are excluded until their timing/geometry has a verified model.
pub(super) unsafe fn grab_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> [Option<kirby_assist_core::spot_dodge::Target>; 8] {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Grabs);
    let mut targets = [None; 8];
    let Some(roster) = reads.roster() else {
        return targets;
    };
    let own_position = [reads.pos_x(own), reads.pos_y(own)];
    let team = reads.team(own);
    for id in roster.into_iter() {
        let Some(id) = id.filter(|id| {
            *id != own_id
                && *id >> 28 == kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
                && game::is_active(*id)
        }) else {
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
        if !matches!(
            status,
            kirby_assist_core::native_states::CATCH
                | kirby_assist_core::native_states::CATCH_DASH
                | kirby_assist_core::native_states::CATCH_TURN
        ) || reads.is_stop(enemy)
            || reads.capture_link(enemy)
        {
            continue;
        }
        let enemy_kind = reads.fighter_kind(enemy);
        let Some(mut grab) = kirby_assist_core::fighter_data::ordinary_grab(enemy_kind, status)
        else {
            continue;
        };
        // Use that attacker's saved reach override, never the controlled
        // fighter's reach. Release catalog locks before any native reads.
        if let Some(reach) = super::SEQUENCES.get().and_then(|saved| {
            saved
                .lock()
                .ok()
                .and_then(|c| c.tuning.get(&enemy_kind).and_then(|t| t.0[1]))
        }) {
            grab.reach = Some(reach);
        }
        let Some(reach) = grab.reach else {
            continue;
        };
        kirby_assist_core::battle_objects::insert_nearest(
            &mut targets,
            kirby_assist_core::spot_dodge::Target {
                id,
                status,
                motion: reads.motion_kind(enemy),
                frame: reads.motion_frame(enemy),
                rate: reads.motion_rate(enemy),
                position: [reads.pos_x(enemy), reads.pos_y(enemy)],
                velocity: [reads.speed_x(enemy, -1), reads.speed_y(enemy, -1)],
                facing: reads.facing(enemy),
                first: grab.first,
                last: grab.last,
                reach,
            },
            |target| {
                kirby_assist_core::battle_objects::distance_squared(own_position, target.position)
            },
        );
    }
    targets
}
