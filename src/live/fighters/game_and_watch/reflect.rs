//! Scoped read-only adapter. Reuses the existing bounded ATTACK-source ring;
//! no roster enumeration, heap allocation, native writes or cross-frame pointers.
use crate::game_ffi as game;
use kirby_assist_core::{
    autopilot_guard::Hitbox,
    projectile_reflect::{self as policy, Config, Context, Threat},
    timing::AnimationSample,
};
use kirby_assist_core::battle_objects::ObjectCategory;
pub(super) unsafe fn observe(
    reads: &super::frame::FrameReads,
    own: *mut game::Object,
    own_id: u32,
    kind: i32,
    config: Option<Config>,
    tracked: Option<u32>,
    allowed: bool,
) -> Option<(Config, Context, Option<Threat>)> {
    let config = config.filter(|_| allowed && policy::supports(kind))?;
    let sources = super::attack_sources::projectiles();
    if tracked.is_none() && sources.iter().all(Option::is_none) {
        return None;
    }
    let active = tracked.is_some();
    let status = reads.status_kind(own);
    let situation = reads.situation_kind(own);
    if !policy::movable(status, situation) && !(active && policy::bucket(status)) {
        return None;
    }
    if reads.capture_link(own) {
        return None;
    }
    // Verified FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE.
    // Full bucket would be Oil Panic's attack, never automatic projectile defense.
    let gauge = game::work_float(
        own,
        kirby_assist_core::native_states::work_ids::FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE,
    );
    if !gauge.is_finite() || !(0.0..3.0).contains(&gauge) {
        return None;
    }
    let stopped = reads.is_stop(own);
    // CONT_SPECIAL_LW from pinned lua_const.rs. Shield is excluded by movable;
    // this mode never drops guard to queue an unavailable down special.
    let can_special = !stopped
        && policy::movable(status, situation)
        && game::transition_enabled(
            own,
            kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_LW,
        );
    if !active && !can_special {
        return None;
    }
    let facing = reads.facing(own);
    let mut context = Context {
        kind,
        situation,
        facing,
        can_special,
        gauge,
        stopped,
        animation: AnimationSample {
            object: own_id,
            status,
            kind: reads.motion_kind(own),
            frame: reads.motion_frame(own),
            auxiliary: 0.,
        },
    };
    if stopped {
        return Some((config, context, None));
    }
    if sources.iter().all(Option::is_none) {
        return Some((config, context, None));
    }
    if !(0.99..=1.01).contains(&game::scale(own)) {
        return None;
    }
    let origin = [reads.pos_x(own), reads.pos_y(own), game::pos_z(own)];
    let velocity = [reads.speed_x(own), reads.speed_y(own)];
    if !origin
        .iter()
        .chain(velocity.iter())
        .chain([&facing])
        .all(|v| v.is_finite())
    {
        return None;
    }
    let team = reads.team(own);
    let mut best: Option<Threat> = None;
    let mut geometry = 0;
    'sources: for source in sources.into_iter().flatten() {
        let id = source.id;
        if id == own_id
            || !matches!(
                id >> 28,
                kirby_assist_core::battle_objects::OBJECT_CATEGORY_WEAPON
                    | kirby_assist_core::battle_objects::OBJECT_CATEGORY_ITEM
            )
        {
            continue;
        }
        if !game::is_active(id) {
            super::attack_sources::forget(id);
            continue;
        }
        let object = game::module_accessor(id);
        if object.is_null()
            || game::battle_object_id(object) != id
            || reads.team(object) == team
            || reads.is_stop(object)
        {
            continue;
        }
        let root = [reads.pos_x(object), reads.pos_y(object)];
        let speed = [reads.speed_x(object), reads.speed_y(object)];
        if !root.iter().chain(speed.iter()).all(|v| v.is_finite())
            || (root[0] - origin[0]).abs() > 96. + (speed[0] - velocity[0]).abs() * config.horizon()
            || (root[1] - origin[1]).abs() > 96. + (speed[1] - velocity[1]).abs() * config.horizon()
        {
            continue;
        }
        let Some(category) = ObjectCategory::from_id(id) else {
            continue;
        };
        if category == ObjectCategory::Weapon {
            // Only consider weapons that are projectiles.
            let source_kind = reads.fighter_kind(object);
            // Determine the status of the source based on its kind.
            // Pikmin have a special status that needs to be checked to determine if they are projectiles.
            let source_status =
                if source_kind == kirby_assist_core::battle_objects::WEAPON_KIND_PIKMIN_PIKMIN {
                    reads.status_kind(object)    
                } else {
                    /// For non-Pikmin weapons, the source status is set to 0.
                    /// Non-Pikmin weapons do not have a special status for projectile determination.
                    /// so using 0 makes sense for non-Pikmin weapons, since they do not have a special status for projectile determination.
                    /// the special status is only relevant for Pikmin weapons due to their unique projectile behavior, 
                    0
                };
            if !ObjectCategory::is_projectile(id, source_kind, source_status)
            {
                continue;
            }
        }
        let scale = game::scale(object);
        if !scale.is_finite() || !(0.01..=10.).contains(&scale) {
            continue;
        }
        for slot in 0..8 {
            if !game::attack_active(object, slot, false) {
                continue;
            }
            // Bound expensive native geometry even with many overlapping articles.
            if geometry >= 16 {
                break 'sources;
            }
            geometry += 1;
            let data = game::attack_geometry(object, slot, false);
            if data.is_null() {
                continue;
            }
            let data = data.read();
            if data.direct != 0
                || data.reflectable != 1 && data.absorbable != 1
                || data.situation
                    & if situation == kirby_assist_core::native_states::situation::GROUND {
                        kirby_assist_core::native_states::situation::ATTACK_GROUND_MASK
                    } else {
                        kirby_assist_core::native_states::situation::ATTACK_AIR_MASK
                    }
                    == 0
                || ![data.x, data.y, data.z, data.x2, data.y2, data.z2, data.size]
                    .iter()
                    .all(|v| v.is_finite())
                || data.size <= 0.
            {
                continue;
            }
            let local = game::Vector3 {
                x: data.x,
                y: data.y,
                z: data.z,
            };
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
            if data.x2 != 0. || data.y2 != 0. || data.z2 != 0. {
                let local = game::Vector3 {
                    x: data.x2,
                    y: data.y2,
                    z: data.z2,
                };
                game::joint_position(object, data.node, &local, &mut end, true);
            }
            let candidate = policy::predict(
                config,
                origin,
                velocity,
                facing,
                id,
                Hitbox {
                    start: [start.x, start.y, start.z],
                    end: [end.x, end.y, end.z],
                    radius: data.size * scale,
                    velocity: speed,
                },
                data.reflectable == 1,
                data.absorbable == 1,
            );
            if let Some(mut threat) = candidate {
                threat.generation = source.generation;
                if tracked == Some(threat.id) {
                    best = Some(threat);
                    break 'sources;
                }
                if best.is_none_or(|old| threat.in_frames < old.in_frames) {
                    best = Some(threat);
                }
            }
        }
    }
    // Projectile progress also advances bounded delivery/hold accounting when
    // the player's idle pose has a frozen native animation frame.
    context.animation.auxiliary = best.map_or(0., |threat| threat.in_frames);
    Some((config, context, best))
}
