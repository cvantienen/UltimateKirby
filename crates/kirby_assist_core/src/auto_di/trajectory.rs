//! Bounded, collision-free survival forecast. This scores controller inputs;
//! stage collision, recovery options and the game's KO decision remain native.
use super::{predicted_launch, Context, Decision, Mode, Reason, Settings};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlastZone {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}
impl BlastZone {
    pub fn contains(self, p: [f32; 2]) -> bool {
        [self.left, self.right, self.bottom, self.top, p[0], p[1]]
            .iter()
            .all(|v| v.is_finite())
            && self.left < p[0]
            && p[0] < self.right
            && self.bottom < p[1]
            && p[1] < self.top
    }
    /// Read the actual death rectangle through GroundUtility::check_dead_area.
    /// Pinned native result enum: NONE=-1, UP=0, DOWN=1, LEFT=2, RIGHT=3.
    /// Each axis is bracketed independently from a verified interior point.
    /// Unknown/nonrectangular results fail closed instead of inventing a stage.
    pub fn observe(p: [f32; 2], mut query: impl FnMut([f32; 2]) -> i32) -> Option<Self> {
        if !p.iter().all(|v| v.is_finite() && v.abs() < 8192.) || query(p) != -1 {
            return None;
        }
        let mut edges = [0.; 4];
        for (slot, (axis, sign, outside)) in [(0, -1., 2), (0, 1., 3), (1, -1., 1), (1, 1., 0)]
            .into_iter()
            .enumerate()
        {
            let at = |distance: f32| {
                let mut point = p;
                point[axis] += sign * distance;
                point
            };
            let (mut low, mut high) = (0., 64.);
            loop {
                match query(at(high)) {
                    -1 if high < 8192. => {
                        low = high;
                        high *= 2.;
                    }
                    result if result == outside => break,
                    _ => return None,
                }
            }
            while high - low > 0.25 {
                let mid = (low + high) * 0.5;
                match query(at(mid)) {
                    -1 => low = mid,
                    result if result == outside => high = mid,
                    _ => return None,
                }
            }
            // Use the interior side of the bracket (at most 0.25 units inset).
            edges[slot] = at(low)[axis];
        }
        let bounds = Self {
            left: edges[0],
            right: edges[1],
            bottom: edges[2],
            top: edges[3],
        };
        bounds.contains(p).then_some(bounds)
    }
    pub fn margin(self, p: [f32; 2]) -> f32 {
        (p[0] - self.left)
            .min(self.right - p[0])
            .min(p[1] - self.bottom)
            .min(self.top - p[1])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trajectory {
    pub position: [f32; 2],
    pub blast: BlastZone,
    pub hitstun: f32,
    pub gravity: f32,
    pub fall_speed: f32,
}
impl Trajectory {
    pub fn valid(self) -> bool {
        self.blast.contains(self.position)
            && self.hitstun.is_finite()
            && (1.0..=240.0).contains(&self.hitstun)
            && self.gravity.is_finite()
            && (0.001..=1.0).contains(&self.gravity)
            && self.fall_speed.is_finite()
            && (0.1..=10.0).contains(&self.fall_speed)
    }
    /// Larger is safer. Surviving the forecast outranks delaying a predicted KO;
    /// among survivors prefer clearance throughout flight, including corners.
    /// Uses Ultimate's 0.051 launch-speed decay and native fighter gravity.
    pub fn score(self, launch: [f32; 2]) -> f32 {
        let initial_speed = launch[0].hypot(launch[1]);
        if !self.valid() || !launch.iter().all(|v| v.is_finite()) || initial_speed < 0.00001 {
            return f32::NEG_INFINITY;
        }
        let direction = [launch[0] / initial_speed, launch[1] / initial_speed];
        let mut p = self.position;
        let mut fall = 0.0_f32;
        let mut margin = f32::INFINITY;
        let frames = self.hitstun.ceil() as usize;
        for frame in 0..frames {
            let speed = (initial_speed - 0.051 * frame as f32).max(0.);
            let velocity = [direction[0] * speed, direction[1] * speed];
            let previous = p;
            p[0] += velocity[0];
            p[1] += velocity[1] - fall;
            fall = (fall + self.gravity).min(self.fall_speed);
            // Upper blast crossings need sufficient remaining vertical launch
            // speed; do not treat merely drifting above it as an immediate KO.
            let clearance = (p[0] - self.blast.left)
                .min(self.blast.right - p[0])
                .min(p[1] - self.blast.bottom)
                .min(if velocity[1] >= 2.4 {
                    self.blast.top - p[1]
                } else {
                    f32::INFINITY
                });
            margin = margin.min(clearance);
            if clearance <= 0. {
                let mut fraction = 1.0_f32;
                for (before, after) in [
                    (previous[0] - self.blast.left, p[0] - self.blast.left),
                    (self.blast.right - previous[0], self.blast.right - p[0]),
                    (previous[1] - self.blast.bottom, p[1] - self.blast.bottom),
                    (
                        if velocity[1] >= 2.4 {
                            self.blast.top - previous[1]
                        } else {
                            f32::INFINITY
                        },
                        if velocity[1] >= 2.4 {
                            self.blast.top - p[1]
                        } else {
                            f32::INFINITY
                        },
                    ),
                ] {
                    if after <= 0. && before.is_finite() {
                        fraction = fraction.min((before / (before - after)).clamp(0., 1.));
                    }
                }
                return frame as f32 + fraction;
            }
        }
        1000. + margin.clamp(0., 1000.)
    }
}

pub fn choose_trajectory(
    settings: Settings,
    c: Context,
    trajectory: Trajectory,
) -> Option<Decision> {
    // Reuse all launch/config validation and preserve explicit Escape behavior.
    let fallback = super::choose(settings, c);
    if settings.mode == Mode::Escape || !trajectory.valid() {
        return fallback;
    }
    if !settings.enabled
        || !c.launch.iter().all(|v| v.is_finite())
        || !(0.05..=30.0).contains(&c.launch[0].hypot(c.launch[1]))
        || !settings.survival_speed.is_finite()
        || !(0.1..=30.).contains(&settings.survival_speed)
    {
        return None;
    }
    let mut best = [0.; 2];
    let mut score = trajectory.score(c.launch);
    let toward_center = |v: [f32; 2]| {
        let b = trajectory.blast;
        v[0] * ((b.left + b.right) * 0.5 - trajectory.position[0]) / (b.right - b.left)
            + v[1] * ((b.bottom + b.top) * 0.5 - trajectory.position[1]) / (b.top - b.bottom)
    };
    let mut tie = toward_center(c.launch);
    // Include neutral, full circle, and exact perpendicular directions rather
    // than quantizing survival choices to eight controller directions.
    let perpendicular = c.launch[0].atan2(-c.launch[1]);
    for i in 0..66 {
        let angle = if i < 64 {
            i as f32 * std::f32::consts::TAU / 64.
        } else {
            perpendicular + (i - 64) as f32 * std::f32::consts::PI
        };
        let (y, x) = angle.sin_cos();
        let stick = [x, y];
        let result = predicted_launch(c.launch, stick);
        let candidate = trajectory.score(result);
        let candidate_tie = toward_center(result);
        if candidate > score + 0.0001
            || (candidate - score).abs() <= 0.0001 && candidate_tie > tie + 0.0001
        {
            score = candidate;
            best = stick;
            tie = candidate_tie;
        }
    }
    Some(Decision {
        stick: best,
        reason: Reason::TrajectorySurvival,
    })
}
