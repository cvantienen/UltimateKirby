//! Read-only bounded live-hit scans for held defense and configured counters.
//! Melee uses estimated contact geometry, not fighter-root distance. Projectile
//! contact admits universal defense; configured counters also use validated
//! incoming projectile contact. Counter fighter admission remains separate from
//! the shared contact solver: universal defense is not Lucina-only.
use crate::game_ffi as game;
use kirby_assist_core::{
    auto_counter::{self as policy, Config, Context, Threat},
    timing::AnimationSample,
};

/// # Safety
/// `enemy` must be a live fighter with valid native attack geometry.
unsafe fn active_threat(
    enemy: *mut game::Object,
    situation: i32,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    enemy_velocity: [f32; 2],
    config: Config,
    remaining: f32,
    budget: &mut usize,
    exhausted: &mut bool,
    require_direct: bool,
    air: Option<kirby_assist_core::gameplay::AirPhysics>,
) -> Option<f32> {
    let mut scale = None;
    let mut earliest: Option<f32> = None;
    for slot in 0..8 {
        if !game::attack_active(enemy, slot, false) {
            continue;
        }
        if *budget == 0 {
            *exhausted = true;
            break;
        }
        *budget -= 1;
        let data = game::attack_geometry(enemy, slot, false);
        if data.is_null() {
            *exhausted = true;
            continue;
        }
        let data = data.read();
        let mask = super::hitboxes::situation_mask(situation)?;
        if !data.power.is_finite() {
            *exhausted = true;
            continue;
        }
        if data.situation & mask == 0 || data.power <= 0.0 {
            continue;
        }
        if require_direct && data.direct != 1 {
            continue;
        }
        let scale = *scale.get_or_insert_with(|| game::scale(enemy));
        let Some(hitbox) = super::hitboxes::world_geometry(enemy, data, scale, enemy_velocity)
        else {
            *exhausted = true;
            continue;
        };
        if let Some(contact) =
            policy::incoming_contact(config, origin, own_velocity, hitbox, remaining, air)
        {
            earliest = Some(earliest.map_or(contact, |old| old.min(contact)));
        }
    }
    earliest
}

/// Universal held defense uses the same bounded geometry reader, but projectile
/// ownership is irrelevant to shield/dodge admission (unlike counter retaliation).
/// `allowed` must include the selected owner's native-safety arbitration.
/// # Safety
/// Accessors must remain valid for these synchronous reads.
pub(super) unsafe fn observe_defense(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    kind: i32,
    values: kirby_assist_core::fighter_tuning::Tuning,
    lead_frames: f32,
    input_delay: f32,
    allowed: bool,
) -> (kirby_assist_core::defense_response::Context, Option<kirby_assist_core::defense_response::Contact>) {
    use kirby_assist_core::{defense_response as defense, fighter_tuning as tuning, native_states as native};
    let status = reads.status_kind(own);
    let situation = reads.situation_kind(own);
    let stopped = reads.is_stop(own);
    let allowed = defense::native_allowed(kind, status, situation, reads.capture_link(own), allowed);
    let guarding = matches!(status, native::GUARD_ON | native::GUARD | native::GUARD_DAMAGE);
    let context = defense::Context {
        object: own_id,
        situation,
        allowed,
        stopped,
        guard_ready: allowed && situation == native::situation::GROUND
            && (guarding || !stopped && game::transition_enabled(own, native::transition_terms::CONT_GUARD_ON)),
        air_dodge_ready: allowed && situation == native::situation::AIR
            && game::transition_enabled(own, native::transition_terms::CONT_ESCAPE_AIR),
        air_dodging: matches!(status, native::ESCAPE_AIR | native::ESCAPE_AIR_SLIDE),
    };
    let Some(horizon) = defense::defense_horizon(lead_frames, input_delay) else {
        return (context, None);
    };
    if !allowed || !matches!(situation, native::situation::GROUND | native::situation::AIR) {
        return (context, None);
    }
    // Reuse the contact solver, not its Lucina-only counter admission policy.
    let body = Config {
        startup: 1.0,
        input_delay: (horizon - 1.0).max(0.0),
        margin: 0.0,
        half_width: values.get(tuning::SHIELD_BODY_HALF_WIDTH),
        height: values.get(tuning::SHIELD_BODY_HEIGHT),
        half_depth: values.get(tuning::SHIELD_BODY_HALF_DEPTH),
    };
    let Some(body) = defense::scaled_body(body, game::scale(own)) else {
        return (context, None);
    };
    let origin = [reads.pos_x(own), reads.pos_y(own), game::pos_z(own)];
    let velocity = if stopped { [0.0; 2] } else { [reads.speed_x(own), reads.speed_y(own)] };
    let air = (situation == native::situation::AIR && !stopped)
        .then(|| super::observations::native_air_physics(own)).flatten();
    let team = reads.team(own);
    let roster = reads.roster().unwrap_or([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    let projectiles = super::attack_sources::projectiles();
    let mut budget = (kirby_assist_core::battle_objects::FIGHTER_CAPACITY
        + kirby_assist_core::projectile_reflect::SOURCE_CAPACITY) * 8;
    let mut exhausted = false;
    let mut best: Option<defense::Contact> = None;
    for (id, projectile) in roster.into_iter().flatten().map(|id| (id, false))
        .chain(projectiles.into_iter().flatten().map(|source| (source.id, true)))
    {
        use kirby_assist_core::battle_objects::{OBJECT_CATEGORY_FIGHTER, OBJECT_CATEGORY_ITEM, OBJECT_CATEGORY_WEAPON};
        let category = id >> 28;
        if id == own_id || if projectile {
            !matches!(category, OBJECT_CATEGORY_WEAPON | OBJECT_CATEGORY_ITEM)
        } else { category != OBJECT_CATEGORY_FIGHTER } {
            continue;
        }
        if !game::is_active(id) {
            if projectile { super::attack_sources::forget(id); }
            continue;
        }
        let source = game::module_accessor(id);
        if source.is_null() || game::battle_object_id(source) != id || reads.team(source) == team {
            continue;
        }
        let source_stopped = reads.is_stop(source);
        let source_velocity = if source_stopped { [0.0; 2] } else {
            [reads.speed_x(source), reads.speed_y(source)]
        };
        // Defender hitstop freezes our path, not independently moving threats.
        let remaining = if source_stopped || projectile { horizon } else {
            // Known active windows cap a forecast; unknown moves use only their
            // already-active geometry, never a catalog's future startup hitbox.
            policy::active_remaining(reads.fighter_kind(source), reads.motion_kind(source),
                reads.motion_frame(source), reads.motion_rate(source)).unwrap_or(horizon).min(horizon)
        };
        if let Some(in_frames) = active_threat(source, situation, origin, velocity,
            source_velocity, body, remaining, &mut budget, &mut exhausted, false, air)
        {
            if best.is_none_or(|old| in_frames < old.in_frames) {
                best = Some(defense::Contact { source: id, in_frames, projectile });
            }
        }
    }
    // Unknown geometry is not fabricated contact evidence. Known contact remains
    // usable for defense even if another source could not be classified.
    (context, best)
}

/// Copy opponent motion identities independently of hitboxes and our readiness.
/// # Safety
/// Native accessors must remain valid during this observation.
pub(super) unsafe fn attacker_poses(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
) -> [Option<(u32, i32, u64, f32)>; 16] {
    let mut poses = [None; 16];
    let team = reads.team(own);
    for (slot, id) in reads.roster().into_iter().flatten().flatten().enumerate() {
        if slot >= poses.len() {
            break;
        }
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
        let pose = (id, reads.status_kind(enemy), reads.motion_kind(enemy), reads.motion_frame(enemy));
        if pose.2 != 0 && pose.3.is_finite() && pose.3 >= 0. {
            poses[slot] = Some(pose);
        }
    }
    poses
}

/// # Safety
/// Native accessors must be valid for the duration of these synchronous reads.
pub(super) unsafe fn observe(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    kind: i32,
    config: Option<Config>,
    allowed: bool,
) -> Option<(Config, Context, Option<Threat>)> {
    let config = config.filter(|_| allowed && policy::supports(kind))?;
    let status = reads.status_kind(own);
    let situation = reads.situation_kind(own);
    if !(policy::ready(status, situation)
        || status == kirby_assist_core::native_states::status::SPECIAL_LW)
        || kirby_assist_core::native_states::fighter_sequence_interrupted(kind, status)
        || reads.capture_link(own)
    {
        return None;
    }
    let stopped = reads.is_stop(own);
    let can_special = !stopped
        && status != kirby_assist_core::native_states::status::SPECIAL_LW
        && policy::can_request(
            status,
            situation,
            game::transition_enabled(
                own,
                kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_LW,
            ),
        );
    let context = Context {
        kind,
        situation,
        can_special,
        stopped,
        animation: AnimationSample {
            object: own_id,
            status,
            kind: reads.motion_kind(own),
            frame: reads.motion_frame(own),
            auxiliary: 0.0,
        },
    };
    if !can_special {
        return Some((config, context, None));
    }
    let Some(geometry_config) = kirby_assist_core::defense_response::scaled_body(config, game::scale(own)) else {
        return Some((config, context, None));
    };
    let origin = [reads.pos_x(own), reads.pos_y(own), game::pos_z(own)];
    let own_velocity = [reads.speed_x(own), reads.speed_y(own)];
    if !origin
        .iter()
        .chain(own_velocity.iter())
        .all(|v| v.is_finite())
    {
        return Some((config, context, None));
    }
    let air = (situation == kirby_assist_core::native_states::situation::AIR)
        .then(|| super::observations::native_air_physics(own))
        .flatten();
    let team = reads.team(own);
    let mut best: Option<Threat> = None;
    // Cover every slot in the bounded fighter and projectile snapshots. A
    // third multi-hitbox fighter must not veto an already observed close hit.
    let mut geometry_budget =
        (kirby_assist_core::battle_objects::FIGHTER_CAPACITY
            + kirby_assist_core::projectile_reflect::SOURCE_CAPACITY) * 8;
    let mut exhausted = false;
    let roster = reads
        .roster()
        .unwrap_or([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    for id in roster.into_iter().flatten() {
        if id == own_id
            || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
            || !game::is_active(id)
        {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null() || game::battle_object_id(enemy) != id {
            exhausted = true;
            continue;
        }
        if reads.team(enemy) == team {
            continue;
        }
        let enemy_motion = reads.motion_kind(enemy);
        let enemy_status = reads.status_kind(enemy);
        let enemy_frame = reads.motion_frame(enemy);
        let stopped = reads.is_stop(enemy);
        let velocity = if stopped {
            [0.; 2]
        } else {
            [reads.speed_x(enemy), reads.speed_y(enemy)]
        };
        let remaining = policy::active_remaining(
            reads.fighter_kind(enemy),
            enemy_motion,
            enemy_frame,
            reads.motion_rate(enemy),
        );
        let attack = active_threat(
            enemy,
            situation,
            origin,
            own_velocity,
            velocity,
            geometry_config,
            if stopped { config.horizon() } else { policy::forecast_horizon(config, remaining) },
            &mut geometry_budget,
            &mut exhausted,
            !policy::melee_status(enemy_status),
            air,
        );
        let attack = attack.map(|in_frames| Threat {
            id,
            motion: enemy_motion,
            status: enemy_status,
            frame: enemy_frame,
            in_frames,
        }).or_else(|| {
            // Frozen animation cannot advance toward a catalog startup. Active
            // geometry remains observable, including our motion into a frozen hit.
            if stopped || (0..8).any(|slot| game::attack_active(enemy, slot, false)) {
                return None;
            }
            policy::predicted_attack(geometry_config, origin, own_velocity,
                [reads.pos_x(enemy), reads.pos_y(enemy), game::pos_z(enemy)], velocity,
                reads.fighter_kind(enemy), id, enemy_motion, enemy_status, enemy_frame,
                reads.motion_rate(enemy))
        });
        if let Some(threat) = attack {
            best = policy::prefer_incoming(config, best, threat);
        }
    }
    // Counter admission depends on incoming hostile geometry, not owner distance.
    for source in super::attack_sources::projectiles().into_iter().flatten() {
        if !matches!(
            source.id >> 28,
            kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
                | kirby_assist_core::battle_objects::OBJECT_CATEGORY_ITEM
        ) {
            continue;
        }
        if !game::is_active(source.id) {
            super::attack_sources::forget(source.id);
            continue;
        }
        let projectile = game::module_accessor(source.id);
        if projectile.is_null() || game::battle_object_id(projectile) != source.id {
            exhausted = true;
            continue;
        }
        if reads.team(projectile) == team {
            continue;
        }
        if !kirby_assist_core::battle_objects::is_projectile(
            source.id,
            reads.fighter_kind(projectile),
            reads.status_kind(projectile),
        ) {
            continue;
        }
        let stopped = reads.is_stop(projectile);
        let velocity = if stopped {
            [0.; 2]
        } else {
            [reads.speed_x(projectile), reads.speed_y(projectile)]
        };
        if let Some(contact) = active_threat(
            projectile,
            situation,
            origin,
            own_velocity,
            velocity,
            geometry_config,
            config.horizon(),
            &mut geometry_budget,
            &mut exhausted,
            false,
            air,
        ) {
            best = policy::prefer_incoming(config, best, policy::projectile_threat(source, contact));
        }
    }
    // An unreadable unrelated source must not veto a known incoming contact.
    Some((config, context, best))
}
