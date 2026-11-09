//! Continuous swept capsule bounds against an observed body envelope.
use crate::autopilot_guard::Hitbox;

/// Computes the earliest time of intersection between a swept capsule and an axis-aligned bounding box.
pub(crate) fn swept_capsule_aabb(
    hit: Hitbox,
    body_lo: [f32; 3],
    body_hi: [f32; 3],
    velocity: [f32; 2],
    horizon: f32,
) -> Option<f32> {
    let relative = [
        hit.velocity[0] - velocity[0],
        hit.velocity[1] - velocity[1],
        0.,
    ];
    let mut enter = 0.0f32;
    let mut leave = horizon;
    for axis in 0..3 {
        let lo = hit.start[axis].min(hit.end[axis]) - hit.radius;
        let hi = hit.start[axis].max(hit.end[axis]) + hit.radius;
        let speed = relative[axis];
        if speed.abs() < 0.0001 {
            if hi < body_lo[axis] || lo > body_hi[axis] {
                return None;
            }
        } else {
            let a = (body_lo[axis] - hi) / speed;
            let b = (body_hi[axis] - lo) / speed;
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return None;
            }
        }
    }
    if leave < 0. || enter > horizon {
        return None;
    }
    Some(enter)
}
