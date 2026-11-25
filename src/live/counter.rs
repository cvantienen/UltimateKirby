//! Read-only bounded live-hit scan for configured down-special counters.
//! Melee uses estimated contact geometry, not fighter-root distance. Projectile
//! contact can veto, but cannot admit, an owner-dependent counter.
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
    melee: bool,
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
        if data.direct > 1 || !data.power.is_finite() {
            *exhausted = true;
            continue;
        }
        if data.direct != u8::from(melee) || data.situation & mask == 0 || data.power <= 0.0 {
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
    if !can_special || !(0.99..=1.01).contains(&game::scale(own)) {
        // The configured body is a nominal-size estimate, not scaled hurtbox data.
        return Some((config, context, None));
    }
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
    let mut soonest = f32::INFINITY;
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
        let admissible =
            !stopped && enemy_motion != 0 && enemy_frame.is_finite() && enemy_frame >= 0.;
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
            config,
            if stopped {
                0.
            } else {
                remaining.unwrap_or(config.horizon())
            },
            &mut geometry_budget,
            &mut exhausted,
            true,
            air,
        );
        let attack = attack.map(|in_frames| Threat {
            id,
            motion: enemy_motion,
            status: enemy_status,
            frame: enemy_frame,
            in_frames,
        });
        if let Some(threat) = attack {
            if threat.in_frames < soonest || !admissible && threat.in_frames == soonest {
                soonest = threat.in_frames;
                best = admissible.then_some(threat);
            }
        }
    }
    // Projectile counters require owner proximity. The cached team_owner_id
    // declaration does not establish current ownership across reflection or
    // transfer, so projectile contact may veto but never admit a counter.
    for source in super::attack_sources::projectiles().into_iter().flatten() {
        if exhausted {
            break;
        }
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
            config,
            if stopped { 0. } else { config.horizon() },
            &mut geometry_budget,
            &mut exhausted,
            false,
            air,
        ) {
            if contact <= soonest {
                soonest = contact;
                best = None;
            }
        }
    }
    // A truncated scan cannot establish which attack will arrive first.
    Some((config, context, best.filter(|_| !exhausted)))
}
