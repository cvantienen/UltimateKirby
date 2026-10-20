//! Pure collision-prediction helpers, without game FFI or memory writes.
//! The native caller supplies status/motion and performs the actual ray checks.
//! Host tests use simple planes to test coverage; they do not simulate Smash's
//! complete collision system or prove that an in-game hit is legally techable.
//!
//! Tune body offsets here, keep native queries in src/live.rs, and add geometry
//! regression tests for each new case instead of embedding stage-specific fixes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// These are reasons OUR detector proceeds or stops. Passing Predict is not the
// same as the game's final decision to accept a shield input as a legal tech.
pub enum TechGate {
    OutsideTumble,
    NotAirborne,
    PassiveDisabled,
    Hitstop,
    Predict,
}

impl TechGate {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OutsideTumble => "outside_supported_tumble",
            Self::NotAirborne => "not_airborne",
            Self::PassiveDisabled => "native_passive_disabled",
            Self::Hitstop => "hitstop_wait",
            Self::Predict => "predict",
        }
    }
}

/// Classify native gates without claiming that passing them guarantees a tech.
pub fn tech_gate(status: i32, situation: i32, passive_disabled: bool, stopped: bool) -> TechGate {
    // This pinned status range covers damage-fly/reflect/fall, not ordinary
    // airborne movement. Broadening it carelessly could cause unrelated dodges.
    if !(0x49..=0x4f).contains(&status) {
        TechGate::OutsideTumble
    } else if situation != 2 {
        TechGate::NotAirborne
    } else if passive_disabled {
        TechGate::PassiveDisabled
    } else if stopped {
        TechGate::Hitstop
    } else {
        TechGate::Predict
    }
}

#[derive(Clone, Copy, Debug)]
// Coordinates and displacement use game world units. [f32; 2] is a fixed
// two-element float array: index 0 is x (right), index 1 is y (up).
pub struct CollisionRay {
    pub origin: [f32; 2],
    pub delta: [f32; 2],
}

/// Three probes per approaching surface; unused directions stay empty.
pub fn tech_probes(x: f32, y: f32, vx: f32, vy: f32, horizon: f32) -> [Option<CollisionRay>; 9] {
    // Reserve three floor, three wall, and three ceiling slots. Option::None
    // means that direction is not approaching; callers use flatten() to skip it.
    let mut probes = [None; 9];
    if ![x, y, vx, vy, horizon].iter().all(|v| v.is_finite()) || !(1.0..=5.0).contains(&horizon) {
        return probes;
    }
    if ![vx * horizon, vy * horizon].iter().all(|v| v.is_finite()) {
        return probes;
    }
    // Side offsets and wall heights approximate Kirby's body. These constants
    // are not queried collision bounds; slopes/platform motion remain heuristic.
    // velocity * horizon sweeps ahead rather than checking only the current point.
    // enumerate() pairs each body offset with a slot index. Positive y is up:
    // negative vy approaches a floor, positive vy approaches a ceiling. Faster
    // motion makes a longer sweep, rather than waiting until Kirby touches it.
    for (i, offset) in [-2.5, 0.0, 2.5].into_iter().enumerate() {
        if vy < -0.1 {
            probes[i] = Some(CollisionRay {
                origin: [x + offset, y + 1.0],
                delta: [vx * horizon, vy * horizon - 1.5],
            });
        }
        if vx.abs() > 0.1 {
            probes[i + 3] = Some(CollisionRay {
                origin: [x, y + [1.0, 3.5, 6.0][i]],
                delta: [vx * horizon + vx.signum() * 2.5, vy * horizon],
            });
        }
        if vy > 0.1 {
            probes[i + 6] = Some(CollisionRay {
                origin: [x + offset, y + 5.0],
                delta: [vx * horizon, vy * horizon + 2.0],
            });
        }
    }
    probes
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;

    fn crosses_plane(ray: &CollisionRay, axis: usize, position: f32) -> bool {
        let t = (position - ray.origin[axis]) / ray.delta[axis];
        (0.0..=1.0).contains(&t)
    }

    #[test]
    fn native_gate_reasons_distinguish_disabled_hitstop_and_non_tumble_hits() {
        assert_eq!(tech_gate(0x4b, 2, true, false), TechGate::PassiveDisabled);
        assert_eq!(tech_gate(0x4b, 0, false, false), TechGate::NotAirborne);
        assert_eq!(tech_gate(0x4b, 2, false, true), TechGate::Hitstop);
        assert_eq!(tech_gate(0x47, 0, false, false), TechGate::OutsideTumble);
        assert_eq!(tech_gate(0x48, 2, false, false), TechGate::OutsideTumble);
        assert_eq!(tech_gate(0x4b, 2, false, false), TechGate::Predict);
    }

    #[test]
    fn sweep_catches_fast_floor_wall_and_ceiling_before_contact() {
        for (x, y, vx, vy, axis, plane) in [
            (0.0, 40.0, 0.0, -15.0, 1, 0.0),
            (0.0, 0.0, 20.0, 0.0, 0, 40.0),
            (0.0, 0.0, 0.0, 20.0, 1, 45.0),
        ] {
            assert!(tech_probes(x, y, vx, vy, 5.0)
                .iter()
                .flatten()
                .any(|r| crosses_plane(r, axis, plane)));
        }
        assert!(!tech_probes(0.0, 40.0, 0.0, -0.2, 5.0)
            .iter()
            .flatten()
            .any(|r| crosses_plane(r, 1, 0.0)));
    }

    #[test]
    fn body_edge_probe_catches_a_ledge_that_center_ray_misses() {
        let probes = tech_probes(0.0, 10.0, 0.0, -5.0, 5.0);
        let hits_ledge =
            |r: &CollisionRay| crosses_plane(r, 1, 0.0) && (2.0..=3.0).contains(&r.origin[0]);
        assert!(!hits_ledge(probes[1].as_ref().unwrap()));
        assert!(hits_ledge(probes[2].as_ref().unwrap()));
    }

    #[test]
    fn stationary_or_invalid_motion_does_not_request_collision_probes() {
        for (vx, horizon) in [
            (0.0, 5.0),
            (f32::NAN, 5.0),
            (2.0, 0.0),
            (2.0, f32::INFINITY),
            (f32::MAX, 5.0),
        ] {
            assert!(tech_probes(0.0, 0.0, vx, 0.0, horizon)
                .iter()
                .all(Option::is_none));
        }
    }
}
