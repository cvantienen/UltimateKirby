//! Bounded, conservative capsule-envelope tests. Body bounds are calibration
//! estimates, not native Kirby hurtboxes. No allocation or game memory here.
#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub radius: f32,
    pub velocity: [f32; 2],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Contact {
    None,
    Duck(f32),
    Shield,
}

// World-unit estimates for unscaled Kirby; extra clearance makes ducking
// conservative. These must be calibrated offline against native hurtboxes.
pub const CROUCH_TOP: f32 = 4.5;
pub const STAND_TOP: f32 = 12.0;
pub const BODY_HALF_WIDTH: f32 = 4.0;
pub const LOOKAHEAD: usize = 6;

pub fn contact(position: [f32; 3], velocity: [f32; 2], hit: Hitbox) -> Contact {
    if !position
        .iter()
        .chain(velocity.iter())
        .chain(hit.start.iter())
        .chain(hit.end.iter())
        .chain(hit.velocity.iter())
        .chain([&hit.radius])
        .all(|v| v.is_finite())
        || !(0.0..=100.0).contains(&hit.radius)
        || hit.radius == 0.0
    {
        return Contact::None;
    }
    let lo = std::array::from_fn::<_, 3, _>(|i| hit.start[i].min(hit.end[i]) - hit.radius);
    let hi = std::array::from_fn::<_, 3, _>(|i| hit.start[i].max(hit.end[i]) + hit.radius);
    if hi[2] < position[2] - BODY_HALF_WIDTH || lo[2] > position[2] + BODY_HALF_WIDTH {
        return Contact::None;
    }
    // Swept per-frame slabs avoid skipping a fast projectile between samples.
    let mut duck_at: Option<f32> = None;
    for frame in 0..=LOOKAHEAD {
        let t = frame as f32;
        let next = (frame + 1).min(LOOKAHEAD) as f32;
        let dx = hit.velocity[0] - velocity[0];
        let dy = hit.velocity[1] - velocity[1];
        let min_x = lo[0] + (dx * t).min(dx * next);
        let max_x = hi[0] + (dx * t).max(dx * next);
        let min_y = lo[1] + (dy * t).min(dy * next);
        let max_y = hi[1] + (dy * t).max(dy * next);
        if max_x < position[0] - BODY_HALF_WIDTH
            || min_x > position[0] + BODY_HALF_WIDTH
            || max_y < position[1]
            || min_y > position[1] + STAND_TOP
        {
            continue;
        }
        // Every predicted overlap must clear crouching height, including the
        // lower endpoint of an extended/diagonal hitbox.
        if min_y <= position[1] + CROUCH_TOP {
            return Contact::Shield;
        }
        duck_at.get_or_insert(t);
    }
    duck_at.map_or(Contact::None, Contact::Duck)
}
