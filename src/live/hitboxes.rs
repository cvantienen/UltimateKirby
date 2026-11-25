//! Shared current-attack conversion. No native pointers survive this call.
//! Endpoint presence and radius scaling retain the existing ABI assumptions;
//! see docs/OBSERVATIONS_AND_TIMING.md for their verification limits.
use crate::game_ffi as game;
use kirby_assist_core::{autopilot_guard::Hitbox, native_states::situation};

pub(super) fn situation_mask(value: i32) -> Option<u16> {
    match value {
        situation::GROUND => Some(situation::ATTACK_GROUND_MASK),
        situation::AIR => Some(situation::ATTACK_AIR_MASK),
        _ => None,
    }
}

/// `data` must already be copied before any further native call.
/// # Safety
/// `object` must be a live accessor matching the copied attack's identity.
pub(super) unsafe fn world_geometry(
    object: *mut game::Object,
    data: game::AttackGeometry,
    scale: f32,
    velocity: [f32; 2],
) -> Option<Hitbox> {
    if !scale.is_finite()
        || !(0.01..=10.).contains(&scale)
        || ![data.x, data.y, data.z, data.x2, data.y2, data.z2, data.size]
            .iter()
            .chain(velocity.iter())
            .all(|v| v.is_finite())
        || data.size <= 0.
    {
        return None;
    }
    let transform = |local: game::Vector3| {
        let mut world = game::Vector3 {
            x: f32::NAN,
            y: f32::NAN,
            z: f32::NAN,
        };
        game::joint_position(object, data.node, &local, &mut world, true);
        [world.x, world.y, world.z]
    };
    let start = transform(game::Vector3 {
        x: data.x,
        y: data.y,
        z: data.z,
    });
    // The pinned prefix has no verified endpoint-presence flag. Keep the
    // existing zero sentinel; do not read opaque padding as a shape flag.
    let end = if data.x2 != 0. || data.y2 != 0. || data.z2 != 0. {
        transform(game::Vector3 {
            x: data.x2,
            y: data.y2,
            z: data.z2,
        })
    } else {
        start
    };
    let radius = data.size * scale;
    (start.iter().chain(end.iter()).all(|v| v.is_finite()) && radius.is_finite() && radius > 0.)
        .then_some(Hitbox {
            start,
            end,
            radius,
            velocity,
        })
}
