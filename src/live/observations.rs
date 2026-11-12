//! Read-only native observations: physics, floor support, roster and opponents.
//!
//! Resolve an object ID to a fresh game accessor before reading it; never store
//! accessor pointers in policy state. Native calls remain outside STATE locks.
//! Opponent get-up timings come from that opponent's native animation metadata;
//! these are separate from our fighter's attack startup and input recipes.
//! Technical: Implements bounded read-only queries for combat threats, floor
//! support, roster identity, and ledge/opponent observations.
//! Plain English: This module gathers nearby game facts that assistance policies use.
use super::{MANAGER_SLOT, STATE};
use crate::game_ffi::{self as game};
/// Maximum number of attack-geometry entries scanned in one frame.
const ATTACK_GEOMETRY_SCAN_BUDGET: usize = 16;
use kirby_assist_core::gameplay::{frames_until, MotionTiming, Opponent};
use kirby_assist_core::kirby::TripTarget;
use kirby_assist_core::ledge_position::LedgePosition;
use kirby_assist_core::ledge_trap::{in_tracking_range, LedgeTrap, CLIFF_CATCH_MOVE, CLIFF_JUMP3};
use kirby_assist_core::settings::AssistSettings;
use std::sync::atomic::Ordering;

/// Tests axis-aligned absolute X/Y deltas against the supplied bounds.
pub(super) fn within_observation_range(
    origin: [f32; 2],
    target: [f32; 2],
    limits: [f32; 2],
) -> bool {
    (target[0] - origin[0]).abs() <= limits[0] && (target[1] - origin[1]).abs() <= limits[1]
}

/// returns `true` if it’s yours or its category is not an allowed one.
/// blocks fighter, weapon, and item categories that are not allowed.
/// # Safety
/// The native game API must be available when this function is called.
unsafe fn non_valid_id(id: u32, own_id: u32) -> bool {
    if id == own_id
        || !matches!(
            id >> 28,
            kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
                | kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
                | kirby_assist_core::battle_objects::OBJECT_CATEGORY_ITEM
        )
    {
        return true;
    }

    // SAFETY: The caller guarantees that the native game API is available.
    unsafe { !game::is_active(id) }
}

/// Tests whether a target is within the attack scan range based on position, velocity, and envelope.
fn is_within_attack_scan_range(
    position: [f32; 2],
    velocity: [f32; 2],
    target: [f32; 2],
    target_velocity: [f32; 2],
    envelope: kirby_assist_core::autopilot_guard::Envelope,
) -> bool {
    // Early exit if the position or velocity data is not finite.
    if !target
        .iter()
        .chain(target_velocity.iter())
        .all(|value| value.is_finite())
    {
        return false;
    }

    let x_limit = envelope.scan_x.max(envelope.warning_x)
        + (target_velocity[0] - velocity[0]).abs() * envelope.forecast;
    let y_limit = envelope.scan_y.max(envelope.warning_y)
        + (target_velocity[1] - velocity[1]).abs() * envelope.forecast;

    (target[0] - position[0]).abs() <= x_limit && (target[1] - position[1]).abs() <= y_limit
}

/// Returns true when this object should be skipped.
/// # Safety
/// If `enemy` is non-null, it must point to a live game object for these reads.
unsafe fn non_valid_enemy(
    reads: &super::frame::FrameReads,
    enemy: *mut game::Object,
    id: u32,
    team: u64,
) -> bool {
    if enemy.is_null() {
        return true;
    }
    // SAFETY: the caller guarantees a non-null pointer refers to a live object.
    // determine if the enemy should be skipped based on its ID and team
    unsafe { game::battle_object_id(enemy) != id || reads.team(enemy) == team }
}

/// Converts joint-relative attack endpoints to world coordinates.
/// # Safety
/// `object` must point to a live game object with the native joint API available.
unsafe fn world_hitbox_endpoints(
    object: *mut game::Object,
    data: &game::AttackGeometry,
) -> (game::Vector3, game::Vector3) {
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
    unsafe { game::joint_position(object, data.node, &local, &mut from, true) };
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
        unsafe { game::joint_position(object, data.node, &local, &mut to, true) };
    }
    (from, to)
}

/// Checks one object's active hitboxes, sharing the scan budget and own-depth cache.
/// An exhausted geometry budget conservatively reports a threat.
struct HitboxScan<'a> {
    own: *mut game::Object,
    position: [f32; 2],
    velocity: [f32; 2],
    envelope: kirby_assist_core::autopilot_guard::Envelope,
    geometry_budget: &'a mut usize,
    own_z: &'a mut Option<f32>,
}

/// # Safety
/// `own` and `enemy` must point to live game objects with native APIs available.
unsafe fn object_has_active_hitbox_threat(
    enemy: *mut game::Object,
    target_velocity: [f32; 2],
    near: bool,
    scan: &mut HitboxScan<'_>,
) -> bool {
    let mut scale = None;
    for slot in 0..8 {
        // Skip inactive attack slots early to save computation.
        if !game::attack_active(enemy, slot, false) {
            continue;
        }
        // Early exit if the enemy is near, as it poses an immediate threat.
        if near {
            return true;
        }
        // Early exit if the geometry budget is exhausted, as we cannot safely evaluate further.
        if *scan.geometry_budget == 0 {
            return true;
        }
        // Consume one unit of the geometry budget for this attack slot.
        *scan.geometry_budget -= 1;
        let data = game::attack_geometry(enemy, slot, false);
        // Skip if the attack geometry data is not available.
        if data.is_null() {
            continue;
        }
        let scale = *scale.get_or_insert_with(|| game::scale(enemy));
        if !scale.is_finite() || !(0.01..=10.0).contains(&scale) {
            continue;
        }

        let data = data.read();
        // Invalid coordinates and non-positive hitbox sizes will still be rejected
        if ![
            data.x,
            data.y,
            data.z,
            data.x2,
            data.y2,
            data.z2,
            data.size,
        ]
        .iter()
        .all(|x| x.is_finite())
            || data.size <= 0.0
        {
            continue;
}
        // Compute the world-space hitbox endpoints for the attack.
        let (from, to) = world_hitbox_endpoints(enemy, &data);
        let z = *scan.own_z.get_or_insert_with(|| game::pos_z(scan.own));
        // Early exit if the active hitbox poses a threat.
        if kirby_assist_core::autopilot_guard::active_hitbox_threat_with(
            [scan.position[0], scan.position[1], z],
            scan.velocity,
            kirby_assist_core::autopilot_guard::Hitbox {
                start: [from.x, from.y, from.z],
                end: [to.x, to.y, to.z],
                radius: data.size * scale,
                velocity: target_velocity,
            },
            scan.envelope,
        ) {
            return true;
        }
    }
    false
}

/// L shield defense uses the cached roster and bounded remembered weapon/item sources.
pub(super) unsafe fn incoming_fighter_attack(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    position: [f32; 2],
    velocity: [f32; 2],
    envelope: kirby_assist_core::autopilot_guard::Envelope,
) -> bool {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Shield);

    let roster = reads.roster();
    let roster = roster.unwrap_or([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    let team = reads.team(own);
    let mut geometry_budget = ATTACK_GEOMETRY_SCAN_BUDGET;
    let mut own_z = None;

    // Iterate over all potential attack sources.
    for id in roster
        .into_iter()
        .chain(super::attack_sources::sources())
        .flatten()
    {
        // skip self and invalid IDs
        if unsafe { non_valid_id(id, own_id) } {
            continue;
        }
        // Skip enemies that should be ignored based on team and validity checks.
        let enemy = game::module_accessor(id);
        if unsafe { non_valid_enemy(reads, enemy, id, team) } {
            continue;
        }
        // Skip targets that are outside the attack scan range.
        let target = [reads.pos_x(enemy), reads.pos_y(enemy)];
        let target_velocity = [reads.speed_x(enemy), reads.speed_y(enemy)];
        // Is this object close enough to examine?
        if !is_within_attack_scan_range(position, velocity, target, target_velocity, envelope) {
            continue;
        }
        // Will the opponent’s root position enter the warning rectangle soon?
        let near = kirby_assist_core::autopilot_guard::approaching_active_attack_with(
            position,
            velocity,
            target,
            target_velocity,
            envelope,
        );
        // check for incoming up-special recovery attacks
        if id >> 28 == kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER {
            let status = reads.status_kind(enemy);
            if status >= kirby_assist_core::native_states::status::SPECIAL_HI {
                let previous = if status >= kirby_assist_core::native_states::status::FINAL {
                    reads.previous_status(enemy, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX)
                } else {
                    0
                };
                // Is a recognized Up-B recovery potentially approaching?
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
                    return true;
                }
            }
        }
        // Does an active hitbox currently overlap—or soon sweep into—the defensive area?
        if object_has_active_hitbox_threat(
            enemy,
            target_velocity,
            near,
            &mut HitboxScan {
                own,
                position,
                velocity,
                envelope,
                geometry_budget: &mut geometry_budget,
                own_z: &mut own_z,
            },
        ) {
            return true;
        }
        // If the opponent is not near, skip further imminent attack checks.
        if !near {
            continue;
        }
        // Check for imminent attacks from nearby opponents.
        // determine if the opponent is about to perform an imminent
        // attack based on their motion and envelope.
        let kind = reads.fighter_kind(enemy);
        let motion = reads.motion_kind(enemy);
        let frame = reads.motion_frame(enemy);
        let rate = reads.motion_rate(enemy);
        // Will a known move become active soon? Predicts a hitbox before it exists.
        let imminent = kirby_assist_core::autopilot_guard::referenced_imminent_attack_with(
            kind,
            motion,
            frame,
            rate,
            envelope.forecast,
        );
        if imminent {
            return true;
        }
    }
    false
}

// Fighter instance parameters are read only while ledge assist is observing.
// Missing/invalid keys stay None; no character-wide gravity constants are invented.
/// Reads native vertical air parameters and validates them through `AirPhysics`.
/// # Safety
/// `object` must be a live native fighter object.
pub(super) unsafe fn native_air_physics(
    object: *mut game::Object,
) -> Option<kirby_assist_core::gameplay::AirPhysics> {
    kirby_assist_core::gameplay::AirPhysics::new(
        game::param_float(object, super::native_params::PARAM_AIR_ACCEL_Y, 0),
        game::param_float(object, super::native_params::PARAM_AIR_SPEED_Y_STABLE, 0),
    )
}

// Sample support along each possible ground move. Probe a short swept
// lookahead, including current momentum, to stop walking across a visible edge.
// This is intentionally conservative on sloped/moving/passable platforms.
/// Sweeps the fighter's candidate ground movement using native floor probes.
/// # Safety
/// `object` must be live and valid for the duration of the native probes.
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
/// finds the nearest valid enemy fighter and returns that enemy’s position relative to you.
/// # Safety
/// Native manager and object state must remain valid during this synchronous query.
pub(super) unsafe fn posture_opponent(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> Option<[f32; 2]> {
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Posture);
    // roster consists of all potential fighter IDs in the current match.
    let roster = reads.roster()?;
    // x,y, and team of the controlled(you) fighter.
    let own_x = reads.pos_x(own);
    let own_y = reads.pos_y(own);
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
        if enemy.is_null() || game::battle_object_id(enemy) != id || reads.team(enemy) == team {
            continue;
        }
        let dx = reads.pos_x(enemy) - own_x;
        let dy = reads.pos_y(enemy) - own_y;
        let distance = dx * dx + dy * dy;
        if distance.is_finite() && nearest.is_none_or(|old| distance < old.0) {
            nearest = Some((distance, [dx, dy]));
        }
    }
    nearest.map(|(_, offset)| offset)
}

/// Builds the bounded target snapshot used by combat policies.
/// # Safety
/// Native fighter IDs must be revalidated by this function before each accessor read.
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
        if enemy.is_null() || game::battle_object_id(enemy) != id || reads.team(enemy) == own_team {
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
                    previous_status: reads
                        .previous_status(enemy, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
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
pub(super) struct RuleTargetQuery {
    pub(super) follow: Option<u32>,
    pub(super) grab_range: Option<[f32; 2]>,
    pub(super) shield_fallback: bool,
    pub(super) watch_events: bool,
    pub(super) sequence_follow: bool,
}

pub(super) unsafe fn rule_targets(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    query: RuleTargetQuery,
) -> (
    [Option<kirby_assist_core::roll_read::Target>; 8],
    Option<Opponent>,
) {
    let RuleTargetQuery {
        follow,
        grab_range,
        shield_fallback,
        watch_events,
        sequence_follow,
    } = query;
    #[cfg(feature = "diagnostic-logging")]
    let _scope = crate::performance::Scope::new(crate::performance::Part::Roll);
    let mut targets = [None; 8];
    let mut following = None;
    // A committed response follows only its locked actor during recovery.
    // It never searches other fighters for a fresh punish in that state.
    if !watch_events && grab_range.is_none() {
        return (
            targets,
            follow.and_then(|id| rule_follow(reads, own, own_id, id, sequence_follow)),
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
        if e.is_null() || game::battle_object_id(e) != id || reads.team(e) == team {
            continue;
        }
        let status = reads.status_kind(e);
        if status >= kirby_assist_core::native_states::status::FINAL
            && kirby_assist_core::native_states::fighter_noncombat(reads.fighter_kind(e), status)
        {
            continue;
        }
        let available = kirby_assist_core::reaction_rules::automatic_target_available(status);
        let retained_follow = sequence_follow
            && follow == Some(id)
            && kirby_assist_core::reaction_rules::automatic_follow_target_available(status);
        if !(available || retained_follow) {
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
                .contains(&reads.previous_status(e, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX))
                || matches!(
                    status,
                    kirby_assist_core::native_states::ESCAPE
                        ..=kirby_assist_core::native_states::ESCAPE_B
                ) && situation != kirby_assist_core::native_states::situation::GROUND
                || matches!(
                    status,
                    kirby_assist_core::native_states::ESCAPE_AIR
                        | kirby_assist_core::native_states::ESCAPE_AIR_SLIDE
                ) && situation != kirby_assist_core::native_states::situation::AIR);
        let position = [reads.pos_x(e), reads.pos_y(e)];
        let eligible_grab =
            grab && grab_from.is_some_and(|(from, facing, range)| {
                facing.is_finite()
                    && facing.abs() >= 0.5
                    && (shield_fallback || (position[0] - from[0]) * facing > 0.)
                    && within_observation_range(from, position, range)
            }) && game::hit_status(e, 0) == 0
                && !(event
                    && (kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE
                        ..=kirby_assist_core::native_states::status::CLIFF_JUMP3)
                        .contains(
                            &reads.previous_status(e, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
                        ));
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
        let velocity_x = if eligible_grab { reads.speed_x(e) } else { 0. };
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

/// Updates observation state for a configured reaction-rule follow-up.
/// # Safety
/// Any native objects read by the function must remain valid for this observation.
unsafe fn rule_follow(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    id: u32,
    sequence_follow: bool,
) -> Option<Opponent> {
    if id == own_id
        || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(id)
    {
        return None;
    }
    let e = game::module_accessor(id);
    if e.is_null() || game::battle_object_id(e) != id || reads.team(e) == reads.team(own) {
        return None;
    }
    let status = reads.status_kind(e);
    if status >= kirby_assist_core::native_states::status::FINAL
        && kirby_assist_core::native_states::fighter_noncombat(reads.fighter_kind(e), status)
    {
        return None;
    }
    if !(kirby_assist_core::reaction_rules::automatic_target_available(status)
        || sequence_follow
            && kirby_assist_core::reaction_rules::automatic_follow_target_available(status))
        || reads.capture_link(e)
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
// `Option<FightersInGame>` has two levels: None means no usable roster;
// Some(array) can still contain empty slots. Callers use flatten() to visit
// only present fighter IDs. IDs must be resolved to fresh accessors each time.
/// Resolves the cached fighter-manager slot into a current native instance.
/// # Safety
/// The cached slot must originate from the validated manager resolver.
unsafe fn fighter_manager_instance() -> Option<*mut game::Object> {
    let slot = MANAGER_SLOT.load(Ordering::Relaxed);
    if slot == 0 {
        return None;
    }
    let manager = (slot as *const *mut game::Object).read();
    (!manager.is_null()).then_some(manager)
}

/// Returns a validated active fighter manager outside the results state.
/// # Safety
/// Native manager bindings must match the pinned game ABI.
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
/// Supplies bounded fighter-ID and local-controller route validation.
mod fighter_roster;
pub(super) use fighter_roster::{fighter_ids, human_fighter, valid_local_route};

// Prefer the tracked opponent through its get-up; selection uses the existing
// bounded follow box. All stage preparation queries are ground-only.
/// Clock-only observation for a committed literal tape whose own pose has zero
/// motion rate or a held edge-balancing pose. Read the already locked actor; no roster scan, hitbox/physics
/// lookup, ray query or action-readiness query. Never retain its native pointer.
/// Reads animation timing for an already selected ledge target without rescanning the roster.
pub(super) unsafe fn tracked_ledge_clock(reads: &super::frame::FrameReads) -> Option<Opponent> {
    let id = STATE.lock().ok()?.ledge.as_ref()?.observation_target()?;
    if !game::is_active(id) {
        return None;
    }
    let object = game::module_accessor(id);
    if object.is_null() || game::battle_object_id(object) != id {
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
        previous_status: reads.previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
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

/// Selects the nearest valid opponent relevant to the current ledge observation.
/// # Safety
/// Native object accessors are only valid during the current observation.
pub(super) unsafe fn nearest_ledge_opponent(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    own_id: u32,
    settings: AssistSettings,
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
                .settings_menu
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
        if enemy.is_null() || game::battle_object_id(enemy) != id || reads.team(enemy) == team {
            continue;
        }
        let status = reads.status_kind(enemy);
        let previous_status =
            reads.previous_status(enemy, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX);
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
            velocity: [reads.speed_x(enemy), reads.speed_y(enemy)],
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

extern "C" {
    #[link_name = "\u{1}_ZN3app7fighter20get_fighter_entry_idEj"]
    /// Resolves a battle-object ID to its owning native entry ID.
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
        // Skip if the enemy is null, the ID doesn't match, or the enemy is on the same team.
        if enemy.is_null() || game::battle_object_id(enemy) != id || reads.team(enemy) == team {
            continue;
        }
        // Skip if the enemy is not in a grab state or is otherwise unavailable.
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
                velocity: [reads.speed_x(enemy), reads.speed_y(enemy)],
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
