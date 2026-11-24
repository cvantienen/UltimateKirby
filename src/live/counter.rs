//! Read-only bounded fighter melee scan for configured down-special counters.
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
    budget: &mut usize,
) -> bool {
    let scale = game::scale(enemy);
    if !scale.is_finite() || !(0.01..=10.0).contains(&scale) {
        return false;
    }
    for slot in 0..8 {
        if !game::attack_active(enemy, slot, false) {
            continue;
        }
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let data = game::attack_geometry(enemy, slot, false);
        if data.is_null() {
            continue;
        }
        let data = data.read();
        let mask = if situation == kirby_assist_core::native_states::situation::GROUND {
            kirby_assist_core::native_states::situation::ATTACK_GROUND_MASK
        } else {
            kirby_assist_core::native_states::situation::ATTACK_AIR_MASK
        };
        if data.direct != 1
            || data.situation & mask == 0
            || ![
                data.x, data.y, data.z, data.x2, data.y2, data.z2, data.size, data.power,
            ]
            .iter()
            .all(|v| v.is_finite())
            || data.size <= 0.0
            || data.power <= 0.0
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
        if policy::active_melee_threat(
            config,
            origin,
            own_velocity,
            kirby_assist_core::autopilot_guard::Hitbox {
                start: [from.x, from.y, from.z],
                end: [to.x, to.y, to.z],
                radius: data.size * scale,
                velocity: enemy_velocity,
            },
        ) {
            return true;
        }
    }
    false
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
        && (game::transition_enabled(
            own,
            kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_LW,
        ) || kirby_assist_core::native_states::moving_down_special_input(status, situation)
            || situation == kirby_assist_core::native_states::situation::AIR
                && kirby_assist_core::native_states::jump_or_fall(status));
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
    let own_velocity = [reads.speed_x(own), reads.speed_y(own)];
    if !origin
        .iter()
        .chain(own_velocity.iter())
        .all(|v| v.is_finite())
    {
        return Some((config, context, None));
    }
    let team = reads.team(own);
    let mut best: Option<Threat> = None;
    let mut soonest = f32::INFINITY;
    let mut geometry_budget = 16;
    let roster = reads
        .roster()
        .unwrap_or([None; kirby_assist_core::battle_objects::FIGHTER_CAPACITY]);
    for id in roster.into_iter().flatten() {
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
        let position = [reads.pos_x(enemy), reads.pos_y(enemy), game::pos_z(enemy)];
        if !position.iter().all(|v| v.is_finite()) {
            continue;
        }
        let enemy_motion = reads.motion_kind(enemy);
        let enemy_status = reads.status_kind(enemy);
        let enemy_frame = reads.motion_frame(enemy);
        let velocity = [reads.speed_x(enemy), reads.speed_y(enemy)];
        if !policy::nearby_attacker(config, origin, own_velocity, position, velocity) {
            continue;
        }
        // A known move may be answered during startup; once any live hitbox
        // exists, its native geometry is authoritative instead of frame data.
        let has_active_hitbox = (0..8).any(|slot| game::attack_active(enemy, slot, false));
        let facing = reads.facing(enemy);
        let imminent = (!has_active_hitbox
            && facing.is_finite()
            && facing.abs() >= 0.5
            && facing * (origin[0] - position[0]) >= 0.0
            && (position[0] - origin[0]).abs() <= config.half_width + 8.0)
            .then(|| {
                policy::predicted_attack(
                    config,
                    origin,
                    own_velocity,
                    position,
                    velocity,
                    reads.fighter_kind(enemy),
                    id,
                    enemy_motion,
                    enemy_status,
                    enemy_frame,
                    reads.motion_rate(enemy),
                )
                .filter(|t| t.in_frames > 0.0)
            })
            .flatten();
        let attack = active_threat(
            enemy,
            situation,
            origin,
            own_velocity,
            velocity,
            config,
            &mut geometry_budget,
        )
        .then_some(Threat {
            id,
            motion: enemy_motion,
            status: enemy_status,
            frame: enemy_frame,
            in_frames: 0.0,
        })
        .or(imminent);
        if let Some(threat) = attack {
            if threat.in_frames < soonest {
                soonest = threat.in_frames;
                best = Some(threat);
            }
        }
    }
    Some((config, context, best))
}
