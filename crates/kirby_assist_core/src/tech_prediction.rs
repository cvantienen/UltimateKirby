//! Pure collision-prediction helpers, without game FFI or memory writes.
//! The native caller supplies status/motion and performs the actual ray checks.
//! Host tests use simple planes to test coverage; they do not simulate Smash's
//! complete collision system or prove that an in-game hit is legally techable.
//!
//! Tune body offsets here, keep native queries in src/live.rs, and add geometry
//! regression tests for each new case instead of embedding stage-specific fixes.
pub const CATCHED_GANON: i32 = 0xbe;

/// Grounded Flame Choke uses a capture status, not damage-fly or a falling ray.
/// Vanilla game_specials applies its throw at native motion frame 31:
/// https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/main/smashline/lua2cpp_ganon/ganon/SpecialS.txt
/// The caller must resolve the actual capture link, not a nearby Ganondorf.
pub fn grounded_ganon_contact_in(
    victim_status: i32,
    captor_kind: i32,
    captor_status: i32,
    frame: f32,
    rate: f32,
) -> Option<f32> {
    if victim_status != CATCHED_GANON
        || captor_kind != 0x18
        || captor_status != 0x1e2
        || !frame.is_finite()
        || !(0.0..=31.0).contains(&frame)
        || !rate.is_finite()
        || rate <= 0.0
    {
        return None;
    }
    let remaining = (31.0 - frame) / rate;
    (remaining.is_finite() && remaining <= 120.0).then_some(remaining)
}

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
    if ![x, y, vx, vy, horizon].iter().all(|v| v.is_finite()) || !(1.0..=20.0).contains(&horizon) {
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

/// Estimate first contact using bounded ray refinement. The native query only
/// reports a hit, so this brackets time rather than assuming the full lookahead
/// is the collision time. At most five refinement queries per hit ray.
pub fn first_contact_in(
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    horizon: f32,
    mut collides: impl FnMut(CollisionRay) -> bool,
) -> Option<f32> {
    let full = tech_probes(x, y, vx, vy, horizon);
    let mut earliest: Option<f32> = None;
    for (index, ray) in full.into_iter().enumerate() {
        let Some(ray) = ray else {
            continue;
        };
        if !collides(ray) {
            continue;
        }
        let (mut low, mut high) = (1.0, horizon);
        if let Some(near) = tech_probes(x, y, vx, vy, 1.0)[index] {
            if collides(near) {
                high = 1.0;
            }
        }
        for _ in 0..4 {
            if high - low <= 0.25 {
                break;
            }
            let mid = (low + high) * 0.5;
            let Some(ray) = tech_probes(x, y, vx, vy, mid)[index] else {
                break;
            };
            if collides(ray) {
                high = mid;
            } else {
                low = mid;
            }
        }
        earliest = Some(earliest.map_or(high, |old| old.min(high)));
    }
    earliest
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests;

#[cfg(test)]
mod grounded_ganon_tests;
