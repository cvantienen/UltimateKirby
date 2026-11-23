//! Read-only bounded fighter melee scan for configured down-special counters.
use crate::game_ffi as game;
use kirby_assist_core::{
    auto_counter::{self as policy, Config, Context, Threat},
    autopilot_guard::Hitbox,
    timing::AnimationSample,
};

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
        && game::transition_enabled(
            own,
            kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_LW,
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
    if stopped || status == kirby_assist_core::native_states::status::SPECIAL_LW {
        return Some((config, context, None));
    }
    let origin = [reads.pos_x(own), reads.pos_y(own), game::pos_z(own)];
    let velocity = [reads.speed_x(own), reads.speed_y(own)];
    if !origin.iter().chain(velocity.iter()).all(|v| v.is_finite()) {
        return Some((config, context, None));
    }
    let team = reads.team(own);
    let mut best: Option<Threat> = None;
    let mut geometry = 0;
    let roster = reads
        .roster()
        .unwrap_or([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    'fighters: for id in roster.into_iter().flatten() {
        if id == own_id || !game::is_active(id) {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null()
            || game::battle_object_id(enemy) != id
            || reads.team(enemy) == team
            || reads.is_stop(enemy)
        {
            continue;
        }
        let position = [reads.pos_x(enemy), reads.pos_y(enemy)];
        let speed = [reads.speed_x(enemy), reads.speed_y(enemy)];
        if !position.iter().chain(speed.iter()).all(|v| v.is_finite())
            || (position[0] - origin[0]).abs()
                > 96.0 + (speed[0] - velocity[0]).abs() * config.horizon()
            || (position[1] - origin[1]).abs()
                > 96.0 + (speed[1] - velocity[1]).abs() * config.horizon()
        {
            continue;
        }
        let scale = game::scale(enemy);
        if !scale.is_finite() || !(0.01..=10.0).contains(&scale) {
            continue;
        }
        for slot in 0..8 {
            if !game::attack_active(enemy, slot, false) {
                continue;
            }
            if geometry >= 16 {
                break 'fighters;
            }
            geometry += 1;
            let data = game::attack_geometry(enemy, slot, false);
            if data.is_null() {
                continue;
            }
            let data = data.read();
            if data.direct != 1
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
                || data.size <= 0.0
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
            game::joint_position(enemy, data.node, &local, &mut start, true);
            let mut end = game::Vector3 {
                x: start.x,
                y: start.y,
                z: start.z,
            };
            if data.x2 != 0.0 || data.y2 != 0.0 || data.z2 != 0.0 {
                let local = game::Vector3 {
                    x: data.x2,
                    y: data.y2,
                    z: data.z2,
                };
                game::joint_position(enemy, data.node, &local, &mut end, true);
            }
            if let Some(threat) = policy::predict(
                config,
                origin,
                velocity,
                id,
                reads.motion_kind(enemy),
                reads.status_kind(enemy),
                reads.motion_frame(enemy),
                Hitbox {
                    start: [start.x, start.y, start.z],
                    end: [end.x, end.y, end.z],
                    radius: data.size * scale,
                    velocity: speed,
                },
            ) {
                if best.is_none_or(|old| threat.in_frames < old.in_frames) {
                    best = Some(threat);
                }
            }
        }
    }
    Some((config, context, best))
}
