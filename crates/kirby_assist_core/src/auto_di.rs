//! Input-only, bounded DI heuristic. No guaranteed KO/combo escape prediction.
//! Uses launch direction/speed, not guessed percentages or hardcoded stage edges.
//! DI/LSI reference: rubendal/SSBU-Calculator, js/formulas.js (DI and LSI).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Adaptive,
    Survival,
    Escape,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub mode: Mode,
    pub survival_speed: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: Mode::Adaptive,
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
    SideSurvival,
    VerticalSurvival,
    ComboEscape,
}
impl Reason {
    pub fn label(self) -> &'static str {
        match self {
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
    let mut angle = v[1].atan2(v[0]);
    if v[1].abs() >= speed * 0.17 {
        angle += 0.17 * (s[1] * v[0] - s[0] * v[1]) / speed;
    }
    let degrees = angle.to_degrees().rem_euclid(360.0);
    let vertical = (65.0..115.0).contains(&degrees) || (245.0..295.0).contains(&degrees);
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
    let escape = settings.mode == Mode::Escape
        || settings.mode == Mode::Adaptive && speed < settings.survival_speed;
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
    let mut best = [0.0; 2];
    let mut best_score = f32::NEG_INFINITY;
    for stick in STICKS {
        let result = predicted_launch(c.launch, stick);
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
    entry: [f32; 2],
    manual: bool,
}
impl AutoDi {
    pub fn manual_override(&self) -> bool {
        self.manual
    }
    pub fn decision(&self) -> Option<Decision> {
        self.decision
    }
    pub fn sample(
        &mut self,
        settings: Settings,
        c: Context,
        physical: [f32; 2],
    ) -> Option<[f32; 2]> {
        if !settings.enabled
            || !c.damage_stop
            || !(1..=120).contains(&c.remaining)
            || !physical.iter().all(|v| v.is_finite())
        {
            *self = Self::default();
            return None;
        }
        let new_hit = self
            .last
            .is_none_or(|(id, left)| id != c.object || c.remaining > left);
        if new_hit {
            self.entry = physical;
            self.manual = false;
            self.key = None;
            self.decision = None;
        } else if physical[0].hypot(physical[1]) < 0.2 {
            self.entry = [0.0; 2];
        } else if physical[0].hypot(physical[1]) >= 0.4
            && (physical[0] - self.entry[0]).hypot(physical[1] - self.entry[1]) >= 0.4
        {
            self.manual = true;
        }
        self.last = Some((c.object, c.remaining));
        if self.manual {
            return None;
        }
        let key = (c.launch, c.outward, settings);
        if self.key != Some(key) {
            self.decision = choose(settings, c);
            self.key = Some(key);
        }
        // Earlier hitstop belongs to SDI. Hold stable DI during the final three
        // observed hitstop frames so SDI's neutral flick cannot erase launch DI.
        if c.remaining <= 3 {
            self.decision.map(|d| d.stick)
        } else {
            None
        }
    }
}
