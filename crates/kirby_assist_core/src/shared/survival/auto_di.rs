//! Input-only, bounded DI heuristic. No guaranteed KO/combo escape prediction.
//! Uses launch direction/speed, not guessed percentages or hardcoded stage edges.
//! DI/LSI reference: rubendal/SSBU-Calculator, js/formulas.js (DI and LSI).
mod trajectory {
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
}
pub use trajectory::{choose_trajectory, BlastZone, Trajectory};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    Adaptive,
    #[default]
    Survival,
    Escape,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub mode: Mode,
    /// Legacy adaptive threshold, retained for existing config compatibility.
    pub survival_speed: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: Mode::Survival,
            survival_speed: 3.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Context {
    pub object: u32,
    pub damage_stop: bool,
    pub remaining: u64,
    /// Native reserved damage velocity, before applying launch DI.
    pub launch: [f32; 2],
    /// Away from attributed attacker/sole opponent; zero means unknown.
    pub outward: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    TrajectorySurvival,
    SideSurvival,
    VerticalSurvival,
    ComboEscape,
}
impl Reason {
    pub fn label(self) -> &'static str {
        match self {
            Self::TrajectorySurvival => "maximize predicted blast-zone clearance",
            Self::SideSurvival => "reduce sideways launch (survival heuristic)",
            Self::VerticalSurvival => "flatten vertical launch (survival heuristic)",
            Self::ComboEscape => "increase separation from the attributed attacker",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    pub stick: [f32; 2],
    pub reason: Reason,
}

// A small analytic approximation, not a trajectory/collision simulator. Do not
// infer blast-zone safety, recovery ability, throws or guaranteed true combos.
pub fn predicted_launch(v: [f32; 2], s: [f32; 2]) -> [f32; 2] {
    let speed = v[0].hypot(v[1]);
    if !v.iter().chain(s.iter()).all(|v| v.is_finite()) || speed < 0.00001 {
        return v;
    }
    let s = s.map(|v| {
        // Match the live signed-byte packet and Ultimate's DI sensitivity.
        let raw = crate::native_input::mapped_stick_axis(v) as f32;
        if raw.abs() < 24. {
            0.
        } else {
            (raw / 120.).clamp(-1., 1.)
        }
    });
    let mut angle = v[1].atan2(v[0]);
    if v[1].abs().atan2(v[0].abs()) >= 0.17 {
        angle += 0.17 * ((s[1] * v[0] - s[0] * v[1]) / speed).clamp(-1., 1.);
    }
    let degrees = angle.to_degrees().rem_euclid(360.0);
    let vertical = degrees > 65.0 && degrees < 115.0 || degrees > 245.0 && degrees < 295.0;
    let lsi = if vertical {
        1.0
    } else if s[1] >= 0.0 {
        1.0 + 0.095 * s[1]
    } else {
        1.0 + 0.08 * s[1]
    };
    let (sin, cos) = angle.sin_cos();
    [speed * cos * lsi, speed * sin * lsi]
}
pub fn choose(settings: Settings, c: Context) -> Option<Decision> {
    let speed = c.launch[0].hypot(c.launch[1]);
    if !settings.enabled
        || !c.launch.iter().all(|v| v.is_finite())
        || !(0.05..=30.0).contains(&speed)
        || !c.outward.is_finite()
        || !settings.survival_speed.is_finite()
        || !(0.1..=30.0).contains(&settings.survival_speed)
    {
        return None;
    }
    let away = if c.outward.abs() > 0.5 {
        c.outward.signum()
    } else {
        0.0
    };
    // Launch speed alone cannot tell whether separating from an attacker is
    // safe near a blast zone. Legacy Adaptive now uses survival behavior too;
    // aggressive separation requires the explicit Escape setting.
    let escape = settings.mode == Mode::Escape;
    if escape && away == 0.0 {
        return None;
    }
    // For nearly vertical launches either horizontal direction may be equally
    // good. Without an attacker direction, abstain rather than guess a side.
    if !escape && c.launch[0].abs() < 0.01 && away == 0.0 {
        return None;
    }
    const D: f32 = std::f32::consts::FRAC_1_SQRT_2;
    const STICKS: [[f32; 2]; 9] = [
        [0.0, 0.0],
        [-1.0, 0.0],
        [1.0, 0.0],
        [0.0, -1.0],
        [0.0, 1.0],
        [-D, -D],
        [D, -D],
        [-D, D],
        [D, D],
    ];
    let side = c.launch[0].abs() >= c.launch[1].abs();
    let axis = usize::from(!side);
    let mut best = [0.0; 2];
    let mut best_score = f32::NEG_INFINITY;
    for stick in STICKS {
        let result = predicted_launch(c.launch, stick);
        // Survival must not trade angle for a higher launch speed from LSI.
        if !escape
            && (result[0].hypot(result[1]) > speed + 0.00001
                || result[axis].abs() > c.launch[axis].abs() + 0.00001)
        {
            continue;
        }
        // Escape DI favors horizontal separation, not increasing upward speed.
        let score = if escape {
            result[0] * away - (result[1] - c.launch[1]).max(0.0) * 0.1
        } else if side {
            -result[0].abs()
        } else {
            -result[1].abs()
        };
        let score = score + stick[0] * away * 0.0001;
        if score > best_score {
            best_score = score;
            best = stick;
        }
    }
    Some(Decision {
        stick: best,
        reason: if escape {
            Reason::ComboEscape
        } else if side {
            Reason::SideSurvival
        } else {
            Reason::VerticalSurvival
        },
    })
}

#[derive(Default)]
pub struct AutoDi {
    last: Option<(u32, u64)>,
    key: Option<([f32; 2], f32, Settings)>,
    decision: Option<Decision>,
    manual: bool,
    stage_key: Option<([f32; 2], Settings)>,
    stage_decision: Option<Decision>,
}
impl AutoDi {
    pub fn manual_override(&self) -> bool {
        self.manual
    }
    pub fn decision(&self) -> Option<Decision> {
        self.decision
    }
    /// An automated damage response owns its stick through the launch phase.
    /// Physical intent is handled by the shared owner, never by a second veto.
    pub fn sample_exclusive(&mut self, settings: Settings, c: Context) -> Option<[f32; 2]> {
        self.manual = false;
        self.sample(settings, c, [0.; 2])
    }
    /// Re-evaluate at the start of the final launch-input phase, then hold the
    /// same result through release. Early SDI may have changed the position.
    pub fn sample_exclusive_with_trajectory(
        &mut self,
        settings: Settings,
        c: Context,
        trajectory: Option<Trajectory>,
    ) -> Option<[f32; 2]> {
        let fallback = self.sample_exclusive(settings, c);
        if !settings.enabled || !c.damage_stop || c.remaining > 3 {
            return fallback;
        }
        if let Some(trajectory) = trajectory.filter(|t| t.valid()) {
            let key = (c.launch, settings);
            if self.stage_key != Some(key) {
                self.stage_decision = choose_trajectory(settings, c, trajectory);
                self.stage_key = Some(key);
            }
            self.decision = self.stage_decision;
            self.decision.map(|d| d.stick)
        } else {
            fallback
        }
    }
    pub fn sample(
        &mut self,
        settings: Settings,
        c: Context,
        physical: [f32; 2],
    ) -> Option<[f32; 2]> {
        if !settings.enabled
            || !c.damage_stop
            || c.remaining > 120
            || !physical.iter().all(|v| v.is_finite())
        {
            *self = Self::default();
            return None;
        }
        let new_hit = self
            .last
            .is_none_or(|(id, left)| id != c.object || c.remaining > left);
        if new_hit {
            self.manual = false;
            self.key = None;
            self.decision = None;
            self.stage_key = None;
            self.stage_decision = None;
        }
        // A direction already held when the hit connects is still the player's
        // input. Never require a release/re-flick before handing DI back.
        self.manual |= physical[0].hypot(physical[1]) >= 0.2;
        self.last = Some((c.object, c.remaining));
        if self.manual {
            return None;
        }
        let key = (c.launch, c.outward, settings);
        if self.key != Some(key) {
            self.decision = choose(settings, c);
            self.key = Some(key);
        }
        // Earlier hitstop belongs to SDI. Hold through the zero-count native
        // damage-stop release observation too: dropping the stick there can
        // erase DI immediately before native damage-vector correction.
        if c.remaining <= 3 {
            self.decision.map(|d| d.stick)
        } else {
            None
        }
    }
}
