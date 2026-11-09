//! Declarative rules: what situation activates a tool, when it fires, and what
//! logical input it requests. These types neither detect game state nor install
//! hooks. Shared text settings are parsed in live_input/config.rs; fighter
//! response functions are compiled Rust registrations.
//!
//! Extending the Scenario enum also requires updating its priority and this
//! fixed rule capacity; adding a new field alone does not wire a new live tool.
use crate::{
    input::InputRecipe,
    state::{FighterKind, RunContext, Scenario},
};

/// Maximum number of scenarios that can be configured without a heap allocator.
pub const MAX_SCENARIO_RULES: usize = 10;

/// Top-level configuration. Rules are disabled by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssistConfig {
    pub enabled: bool,
    pub scope: AssistScope,
    // Fixed-size storage avoids allocating a vector whenever policy is evaluated.
    pub rules: [ScenarioRule; MAX_SCENARIO_RULES],
}

impl Default for AssistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scope: AssistScope::default(),
            rules: [ScenarioRule::DISABLED; MAX_SCENARIO_RULES],
        }
    }
}

impl AssistConfig {
    /// Replaces the matching rule, or uses the first empty slot.
    ///
    /// Returns `false` for `Scenario::None` or when no empty slot remains.
    /// Disabled rules retain their settings until explicitly removed.
    pub fn set_rule(&mut self, rule: ScenarioRule) -> bool {
        if rule.scenario == Scenario::None {
            return false;
        }

        // iter_mut() visits mutable references. find() returns the first match
        // as Option<&mut ScenarioRule>; `*slot = rule` replaces that array entry,
        // not the entire config. Search existing entries first to avoid duplicates.
        if let Some(slot) = self
            .rules
            .iter_mut()
            .find(|current| current.scenario == rule.scenario)
        {
            *slot = rule;
            return true;
        }

        if let Some(slot) = self
            .rules
            .iter_mut()
            .find(|current| current.scenario == Scenario::None)
        {
            *slot = rule;
            return true;
        }

        false
    }

    /// Toggle a tool without losing its recipe or timing.
    pub fn set_rule_enabled(&mut self, scenario: Scenario, enabled: bool) -> bool {
        if scenario == Scenario::None {
            return false;
        }
        if let Some(rule) = self.rules.iter_mut().find(|rule| rule.scenario == scenario) {
            rule.enabled = enabled;
            return true;
        }
        false
    }

    /// Remove a tool and free its configuration slot.
    pub fn remove_rule(&mut self, scenario: Scenario) -> bool {
        if scenario == Scenario::None {
            return false;
        }
        if let Some(rule) = self.rules.iter_mut().find(|rule| rule.scenario == scenario) {
            *rule = ScenarioRule::DISABLED;
            return true;
        }
        false
    }

    // Disabled entries are intentionally ignored, but retain their settings so
    // toggling a tool back on does not discard its timing/custom recipe.
    pub fn rule_for(&self, scenario: Scenario) -> Option<ScenarioRule> {
        self.rules
            .iter()
            .copied()
            .find(|rule| rule.enabled && rule.scenario == scenario)
    }
}

/// Select which fighter/controller may receive assistance, in any game mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssistScope {
    pub require_locally_human_controlled: bool,
    pub require_kirby: bool,
}

impl AssistScope {
    // `!flag || condition` means "only require condition when flag is true".
    // This scope is independent of game mode; the live adapter separately
    // validates its supported native controller/fighter before using the engine.
    pub fn allows(self, context: RunContext) -> bool {
        (!self.require_locally_human_controlled || context.is_locally_human_controlled)
            && (!self.require_kirby || context.fighter == FighterKind::Kirby)
    }
}

impl Default for AssistScope {
    fn default() -> Self {
        Self {
            require_locally_human_controlled: true,
            require_kirby: false,
        }
    }
}

/// The action to take after a rule's timing condition has been met.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioAction {
    /// One logical controller frame, merged with the human player's frame.
    ControllerRecipe(InputRecipe),
    /// A future feature for replaying a saved multi-frame input sequence.
    /// It is deliberately non-functional until a playback backend is written.
    PlaybackSlot(RecordingSlot),
}

/// Slot names reserve an understandable API for a future input recorder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingSlot {
    One,
    Two,
    Three,
    Four,
    Five,
}

/// How a scenario's active-frame counter selects controller frames.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleTiming {
    /// Wait this many scenario frames before the first action frame.
    pub start_after_frames: u16,
    /// Number of action frames to emit each time the rule activates.
    pub active_for_frames: u16,
    /// Zero means "run once". Any other value repeats from the first frame.
    pub repeat_every_frames: u16,
}

impl RuleTiming {
    pub const IMMEDIATELY_ONCE: Self = Self {
        start_after_frames: 0,
        active_for_frames: 1,
        repeat_every_frames: 0,
    };

    pub const fn after(start_after_frames: u16) -> Self {
        Self {
            start_after_frames,
            ..Self::IMMEDIATELY_ONCE
        }
    }

    pub const fn pulse_every(repeat_every_frames: u16) -> Self {
        Self {
            repeat_every_frames,
            ..Self::IMMEDIATELY_ONCE
        }
    }

    // Frame zero belongs to the scenario. After the initial delay, modulo (%)
    // repeats a pulse pattern: interval 3 / duration 1 means on, off, off, repeat.
    // The live caller supplies observed input updates, not a universal game clock.
    pub fn is_active_on(self, frames_active: u32) -> bool {
        if frames_active < self.start_after_frames as u32 {
            return false;
        }

        // Example: delay 2, duration 1, repeat 3 fires at ages 2, 5, 8, ...
        // .max(1) prevents a zero-duration action or a modulo-by-zero panic.
        // These generic recipe timings do not tune the live analog mash cycle.
        let duration = self.active_for_frames.max(1) as u32;
        let elapsed = frames_active - self.start_after_frames as u32;
        if self.repeat_every_frames == 0 {
            elapsed < duration
        } else {
            elapsed % (self.repeat_every_frames.max(1) as u32) < duration
        }
    }
}

/// A readable declaration: "in this scenario, propose this action at this time."
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioRule {
    pub enabled: bool,
    pub scenario: Scenario,
    pub timing: RuleTiming,
    pub action: ScenarioAction,
}

impl ScenarioRule {
    pub const DISABLED: Self = Self {
        enabled: false,
        scenario: Scenario::None,
        timing: RuleTiming::IMMEDIATELY_ONCE,
        action: ScenarioAction::ControllerRecipe(InputRecipe::NONE),
    };

    pub const fn controller(scenario: Scenario, timing: RuleTiming, recipe: InputRecipe) -> Self {
        Self {
            enabled: true,
            scenario,
            timing,
            action: ScenarioAction::ControllerRecipe(recipe),
        }
    }

    pub const fn playback(scenario: Scenario, timing: RuleTiming, slot: RecordingSlot) -> Self {
        Self {
            enabled: true,
            scenario,
            timing,
            action: ScenarioAction::PlaybackSlot(slot),
        }
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ButtonSet;

    #[test]
    fn all_scenarios_fit_and_disabled_settings_survive_other_tools() {
        let mut config = AssistConfig::default();
        let scenarios = [
            Scenario::Tech,
            Scenario::Clatter,
            Scenario::Tumble,
            Scenario::Hitstun,
            Scenario::Parry,
            Scenario::Footstool,
            Scenario::Landing,
            Scenario::Ledge,
            Scenario::Grounded,
            Scenario::Airborne,
        ];
        let tech = ScenarioRule::controller(
            Scenario::Tech,
            RuleTiming::after(2),
            InputRecipe::buttons(ButtonSet::SHIELD),
        );
        assert!(config.set_rule(tech));
        assert!(config.set_rule_enabled(Scenario::Tech, false));
        for scenario in &scenarios[1..] {
            assert!(config.set_rule(ScenarioRule::controller(
                *scenario,
                RuleTiming::IMMEDIATELY_ONCE,
                InputRecipe::NONE,
            )));
        }
        assert!(config.set_rule_enabled(Scenario::Tech, true));
        assert_eq!(config.rule_for(Scenario::Tech), Some(tech));
        assert!(config.remove_rule(Scenario::Tech));
        assert!(!config.remove_rule(Scenario::Tech));
        assert_eq!(config.rule_for(Scenario::Tech), None);
        assert!(!config.set_rule(ScenarioRule::DISABLED));
        assert!(!config.set_rule_enabled(Scenario::None, true));
        assert!(!config.remove_rule(Scenario::None));
        assert!(config.set_rule(tech));
    }

    #[test]
    fn new_rule_replaces_old_rule_for_the_same_scenario() {
        let mut config = AssistConfig::default();
        assert!(config.set_rule(ScenarioRule::controller(
            Scenario::Clatter,
            RuleTiming::IMMEDIATELY_ONCE,
            InputRecipe::buttons(ButtonSet::ATTACK),
        )));
        assert!(config.set_rule(ScenarioRule::controller(
            Scenario::Clatter,
            RuleTiming::pulse_every(3),
            InputRecipe::buttons(ButtonSet::JUMP),
        )));

        assert_eq!(
            config.rule_for(Scenario::Clatter).unwrap().timing,
            RuleTiming::pulse_every(3)
        );
    }

    #[test]
    fn timing_supports_delays_and_pulses() {
        let timing = RuleTiming {
            start_after_frames: 2,
            active_for_frames: 1,
            repeat_every_frames: 3,
        };

        assert!(!timing.is_active_on(0));
        assert!(!timing.is_active_on(1));
        assert!(timing.is_active_on(2));
        assert!(!timing.is_active_on(3));
        assert!(timing.is_active_on(5));
    }
}
pub mod button_bindings {
    //! Mode identities for On/Off settings and legacy file compatibility.
    //! Reaction rules use their own explicit Control whitelist; Builtin below
    //! exposes native button helpers.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        LedgeTrap,
        AutoTech,
        AutoMash,
        IdleCrouch,
        TiltTurn,
        SpotPunish,
        OutOfShield,
    }
    impl Mode {
        pub const ALL: [Self; 7] = [
            Self::LedgeTrap,
            Self::AutoTech,
            Self::AutoMash,
            Self::IdleCrouch,
            Self::TiltTurn,
            Self::SpotPunish,
            Self::OutOfShield,
        ];
        pub fn bit(self) -> u16 {
            1 << self as usize
        }
        pub fn id(self) -> &'static str {
            match self {
                Self::LedgeTrap => "ledge_trap",
                Self::AutoTech => "auto_tech",
                Self::AutoMash => "auto_mash",
                Self::IdleCrouch => "idle_crouch",
                Self::TiltTurn => "tilt_turn",
                Self::SpotPunish => "spot_punish",
                Self::OutOfShield => "out_of_shield",
            }
        }
        pub fn label(self) -> &'static str {
            match self {
                Self::LedgeTrap => "Ledge trap",
                Self::AutoTech => "Auto-tech",
                Self::AutoMash => "Auto-mash",
                Self::IdleCrouch => "Idle crouch",
                Self::TiltTurn => "Tilt face",
                Self::SpotPunish => "Spot punish",
                Self::OutOfShield => "Out of shield",
            }
        }
        pub fn parse(s: &str) -> Option<Self> {
            Self::ALL.into_iter().find(|m| m.id() == s)
        }
        pub fn supports(self, kind: i32) -> bool {
            match self {
                Self::IdleCrouch => kind == crate::fighter_kinds::KIRBY,
                Self::TiltTurn => true,
                Self::OutOfShield => true,
                _ => true,
            }
        }
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum Activation {
        #[default]
        Hold,
        Toggle,
    }
    impl Activation {
        pub fn id(self) -> &'static str {
            match self {
                Self::Hold => "hold",
                Self::Toggle => "toggle",
            }
        }
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ModeBinding {
        pub mode: Mode,
        pub activation: Activation,
    }
    impl ModeBinding {
        pub const COUNT: usize = Mode::ALL.len() * 2;
        pub fn at(index: usize) -> Option<Self> {
            Some(Self {
                mode: *Mode::ALL.get(index / 2)?,
                activation: if index % 2 == 0 {
                    Activation::Hold
                } else {
                    Activation::Toggle
                },
            })
        }
        pub fn index(self) -> usize {
            self.mode as usize * 2 + usize::from(self.activation == Activation::Toggle)
        }
        pub fn label(self) -> &'static str {
            const LABELS: [&str; ModeBinding::COUNT] = [
                "Hold: Ledge trap",
                "Toggle: Ledge trap",
                "Hold: Auto-tech",
                "Toggle: Auto-tech",
                "Hold: Auto-mash",
                "Toggle: Auto-mash",
                "Hold: Idle crouch",
                "Toggle: Idle crouch",
                "Hold: Tilt face",
                "Toggle: Tilt face",
                "Hold: Spot punish",
                "Toggle: Spot punish",
                "Hold: Out of shield",
                "Toggle: Out of shield",
            ];
            LABELS[self.index()]
        }
        pub fn choice_id(self) -> &'static str {
            const IDS: [&str; ModeBinding::COUNT] = [
                "__mode_ledge_trap_hold",
                "__mode_ledge_trap_toggle",
                "__mode_auto_tech_hold",
                "__mode_auto_tech_toggle",
                "__mode_auto_mash_hold",
                "__mode_auto_mash_toggle",
                "__mode_idle_crouch_hold",
                "__mode_idle_crouch_toggle",
                "__mode_tilt_turn_hold",
                "__mode_tilt_turn_toggle",
                "__mode_spot_punish_hold",
                "__mode_spot_punish_toggle",
                "__mode_out_of_shield_hold",
                "__mode_out_of_shield_toggle",
            ];
            IDS[self.index()]
        }
    }

    /// Existing native input helpers, assignable without executing a recipe tape.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Builtin {
        Normal,
        LedgeAssist,
        TurboTeabag,
        LedgeTrump,
    }
    impl Builtin {
        pub const ALL: [Self; 4] = [
            Self::Normal,
            Self::LedgeAssist,
            Self::TurboTeabag,
            Self::LedgeTrump,
        ];
        pub fn token(self) -> &'static str {
            match self {
                Self::Normal => "normal",
                Self::LedgeAssist => "ledge_assist",
                Self::TurboTeabag => "turbo_teabag",
                Self::LedgeTrump => "ledge_trump",
            }
        }
        pub fn label(self) -> &'static str {
            match self {
                Self::Normal => "Normal button",
                Self::LedgeAssist => "Built-in: Ledge assist / walk",
                Self::TurboTeabag => "Built-in: Turbo t-bag",
                Self::LedgeTrump => "Built-in: Ledge trump",
            }
        }
        pub fn choice_id(self) -> &'static str {
            match self {
                Self::Normal => "__normal_button",
                Self::LedgeAssist => "__builtin_ledge_assist",
                Self::TurboTeabag => "__builtin_turbo_teabag",
                Self::LedgeTrump => "__builtin_ledge_trump",
            }
        }
        pub fn parse(value: &str) -> Option<Self> {
            match value {
                "short_hop" => Some(Self::Normal),
                _ => Self::ALL.into_iter().find(|b| b.token() == value),
            }
        }
        pub fn default_for(slot: crate::fighter_profiles::presets::PresetSlot) -> Self {
            use crate::fighter_profiles::presets::PresetSlot;
            match slot {
                PresetSlot::L3 => Self::TurboTeabag,
                _ => Self::Normal,
            }
        }
        pub fn template_offset(self) -> usize {
            match self {
                Self::Normal => 0,
                Self::LedgeAssist => 2,
                Self::TurboTeabag => 3,
                Self::LedgeTrump => 4,
            }
        }
    }
}
pub mod option_read {
    //! Shared, bounded recovery punish for spot dodges, avoided grabs and far rolls.
    //! Reach values are calibration estimates; native recovery and readiness gate inputs.
    use crate::{
        gameplay::ControllerOutput,
        native_input::MAPPER_A,
        timing::{motion_elapsed, ActiveWindow, RecoveryWindow},
    };
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Spot,
        Grab,
        Roll,
    }
    #[derive(Clone, Copy)]
    pub struct Target {
        pub id: u32,
        pub kind: Kind,
        pub status: i32,
        pub motion: u64,
        pub frame: f32,
        pub rate: f32,
        pub position: [f32; 2],
        pub velocity: [f32; 2],
        pub opens: f32,
        pub closes: f32,
    }
    #[derive(Clone, Copy)]
    pub struct Context {
        pub object: u32,
        pub status: i32,
        pub motion: u64,
        pub frame: f32,
        pub rate: f32,
        pub stopped: bool,
        pub grounded: bool,
        pub can_act: bool,
        pub down_ready: bool,
        pub forward_ready: bool,
        pub facing: f32,
        pub position: [f32; 2],
        pub velocity: [f32; 2],
    }
    #[derive(Clone, Copy)]
    pub struct Config {
        pub down: [u8; 2],
        pub down_back: Option<[u8; 2]>,
        pub forward: [u8; 2],
        pub down_reach: f32,
        pub forward_reach: f32,
    }
    #[derive(Clone, Copy)]
    struct Track {
        id: u32,
        status: i32,
        motion: u64,
        frame: f32,
        rate: f32,
        used: bool,
    }
    #[derive(Default)]
    pub struct Punisher {
        object: Option<u32>,
        tracks: [Option<Track>; 8],
        previous: Option<(i32, u64, f32, f32)>,
        target: Option<u32>,
        phase: u8,
        forward: bool,
        age: f32,
        output: ControllerOutput,
    }
    impl Punisher {
        pub fn committed(&self) -> bool {
            self.target.is_some() && self.output.owned != 0
        }
        pub fn cancel_pending(&mut self) {
            self.target = None;
            self.phase = 0;
            self.output = ControllerOutput::default();
        }
        pub fn sample(
            &mut self,
            enabled: bool,
            c: Context,
            cfg: Config,
            targets: [Option<Target>; 8],
        ) -> ControllerOutput {
            self.sample_with_height(enabled, c, cfg, targets, 8.)
        }
        pub fn sample_with_height(
            &mut self,
            enabled: bool,
            c: Context,
            cfg: Config,
            targets: [Option<Target>; 8],
            height: f32,
        ) -> ControllerOutput {
            if self.object != Some(c.object) {
                *self = Self::default();
                self.object = Some(c.object);
            }
            let old = self.tracks;
            for (i, t) in targets.iter().enumerate() {
                self.tracks[i] = t.map(|t| Track {
                    id: t.id,
                    status: t.status,
                    motion: t.motion,
                    frame: t.frame,
                    rate: t.rate,
                    used: old.iter().flatten().any(|p| {
                        p.id == t.id
                            && p.status == t.status
                            && p.motion == t.motion
                            && t.frame >= p.frame
                            && p.used
                    }),
                });
            }
            if !enabled
                || !c.grounded
                || !c.frame.is_finite()
                || c.frame < 0.0
                || c.motion == crate::timing::NO_MOTION_KIND
                || !c.rate.is_finite()
                || c.rate <= 0.0
                || !c.facing.is_finite()
                || c.facing.abs() < 0.5
            {
                self.cancel_pending();
                self.previous = None;
                return self.output;
            }
            if matches!(
                c.status,
                crate::native_states::status::ATTACK_S4_START
                    ..=crate::native_states::status::ATTACK_LW4
            ) {
                self.cancel_pending();
                return self.output;
            }
            if c.stopped {
                return self.output;
            }
            let clock = (c.status, c.motion, c.frame, c.rate);
            let mut delta = motion_elapsed(self.previous, clock, 0.0);
            self.previous = Some(clock);
            if delta == 0.0 {
                if let Some(t) = targets.iter().flatten().find(|t| Some(t.id) == self.target) {
                    if let Some(p) = old
                        .iter()
                        .flatten()
                        .find(|p| p.id == t.id && p.motion == t.motion)
                    {
                        delta = motion_elapsed(
                            Some((p.status, p.motion, p.frame, p.rate)),
                            (t.status, t.motion, t.frame, t.rate),
                            0.0,
                        );
                    }
                }
            }
            if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
                self.cancel_pending();
                return self.output;
            }
            if self.target.is_some_and(|id| {
                !targets
                    .iter()
                    .flatten()
                    .any(|t| t.id == id && contact(c, cfg, *t, self.forward, 0.0, height))
            }) {
                self.cancel_pending();
                return self.output;
            }
            if self.phase != 0 && delta == 0.0 {
                return self.output;
            }
            if self.phase == 2 {
                self.age += delta;
                if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT
                    || !targets.iter().flatten().any(|t| {
                        Some(t.id) == self.target && contact(c, cfg, *t, self.forward, 0.0, height)
                    })
                {
                    self.cancel_pending();
                } else {
                    self.output = if crate::move_input::smash_retry_pressed(
                        self.age,
                        self.target.unwrap_or_default(),
                    ) {
                        crate::move_input::output(
                            if self.forward {
                                [c.facing.signum(), 0.0]
                            } else {
                                [0.0, -1.0]
                            },
                            MAPPER_A,
                        )
                    } else {
                        crate::move_input::output([0.0; 2], 0)
                    };
                }
                return self.output;
            }
            let shield = match c.status {
                crate::native_states::GUARD_ON..=crate::native_states::GUARD => 11.0,
                crate::native_states::GUARD_OFF => (11.0 - c.frame).max(0.0),
                _ => 0.0,
            };
            if !c.can_act
                && shield == 0.0
                && !(matches!(
                    c.status,
                    crate::native_states::GUARD_ON..=crate::native_states::GUARD_OFF
                ) && (c.down_ready || c.forward_ready))
            {
                self.cancel_pending();
                return self.output;
            }
            let setup = shield + if self.phase == 1 { 0.0 } else { 1.0 };
            let mut selected = None;
            for (i, t) in targets.iter().enumerate() {
                let Some(t) = t else { continue };
                if self.tracks[i].unwrap().used || self.target.is_some_and(|id| id != t.id) {
                    continue;
                }
                let at = t.opens.clamp(0.0, 120.0);
                let x = t.position[0] - c.position[0] + (t.velocity[0] - c.velocity[0]) * at;
                let forward = x.abs() > cfg.down_reach;
                if t.kind == Kind::Roll && !forward {
                    continue;
                } // close rolls retain the existing down-smash reader
                if contact(c, cfg, *t, forward, setup, height) {
                    selected = Some((i, *t, forward, x));
                    break;
                }
            }
            let Some((i, t, forward, x)) = selected else {
                self.cancel_pending();
                return self.output;
            };
            self.target = Some(t.id);
            self.forward = forward;
            self.output = crate::move_input::output([0.0; 2], 0);
            if forward && x * c.facing < 0.5 {
                if shield == 0.0 && c.can_act {
                    self.output.stick = Some([x.signum() * 0.3, 0.0]);
                }
                self.phase = 0;
                return self.output;
            }
            if shield > 0.0 {
                self.phase = 1;
                return self.output;
            }
            if self.phase != 1 {
                self.phase = 1;
                return self.output;
            }
            if !(if forward {
                c.forward_ready
            } else {
                c.down_ready
            }) {
                return self.output;
            }
            self.phase = 2;
            self.age = 0.0;
            self.tracks[i].as_mut().unwrap().used = true;
            self.output = crate::move_input::output(
                if forward {
                    [c.facing.signum(), 0.0]
                } else {
                    [0.0, -1.0]
                },
                MAPPER_A,
            );
            self.output
        }
    }
    fn contact(c: Context, cfg: Config, t: Target, forward: bool, setup: f32, height: f32) -> bool {
        if !t.frame.is_finite()
            || !t.rate.is_finite()
            || t.rate <= 0.0
            || !t.opens.is_finite()
            || t.opens < 0.0
            || !t.closes.is_finite()
            || t.closes <= t.opens
            || t.motion == crate::timing::NO_MOTION_KIND
            || !c
                .position
                .iter()
                .chain(c.velocity.iter())
                .chain(t.position.iter())
                .chain(t.velocity.iter())
                .all(|v| v.is_finite())
        {
            return false;
        }
        let reach = if forward {
            cfg.forward_reach
        } else {
            cfg.down_reach
        };
        let windows = if forward {
            [Some((cfg.forward, false)), None]
        } else {
            [Some((cfg.down, false)), cfg.down_back.map(|w| (w, true))]
        };
        windows.into_iter().flatten().any(|(frames, back)| {
            ActiveWindow {
                first: frames[0] as u16,
                last: frames[1] as u16,
            }
            .first_contact(
                setup,
                RecoveryWindow {
                    vulnerable_in: Some(t.opens),
                    actionable_in: Some(t.closes),
                },
                |at| {
                    let x = t.position[0] - c.position[0] + (t.velocity[0] - c.velocity[0]) * at;
                    let y = t.position[1] - c.position[1] + (t.velocity[1] - c.velocity[1]) * at;
                    let side_ok = forward
                        || cfg.down_back.is_none()
                        || x.abs() <= 0.001
                        || (x * c.facing < 0.) == back;
                    side_ok && x.abs() <= reach && y.abs() <= height
                },
            )
            .is_some()
        })
    }
}
pub mod settings {
    //! Settings shared by ledge and defense-response policies.
    use crate::fighter_profiles::{FighterGrabData, LedgeResponses};
    use crate::native_input::MAPPER_R;

    /// Validated assistance settings. Defaults and response timing are shared by both builds.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AssistSettings {
        /// Legacy identity tag; no executable fighter handler is dispatched.
        pub implementation: Option<crate::ledge_trap::ResponseImplementation>,
        /// Native fighter identity for startup-loaded file menus.
        pub fighter_kind: i32,
        /// Indices into the fighter's startup-resolved menu lists. None preserves direct
        /// core/legacy callers that supply settings without using the battle menu.
        pub response_choices: [Option<usize>; crate::fighter_profiles::presets::PresetSlot::COUNT],
        /// Startup-parsed recipes for the five selected compiled or file-added IDs.
        /// Fixed-size copies keep text/file work out of live callbacks.
        pub sequence_overrides: [Option<crate::sequence_config::Sequence>;
            crate::fighter_profiles::presets::PresetSlot::COUNT],
        /// Numeric-only startup adjustments; these cannot register new actions.
        /// Resolved move timings and response switches for our selected fighter.
        pub fighter: FighterGrabData,
        pub responses: LedgeResponses,
        pub enabled: bool,
        /// Enable the selected response to a witnessed generic ledge release.
        pub drop_defense: bool,
        /// Native-time budget to request/confirm the drop punish before shielding.
        // Ranges/heights/stance distances are WORLD units, not stick percentages.
        // Increasing reach only makes the policy willing to attack from farther
        // away; it does not enlarge Kirby's real hitbox and can create more whiffs.
        /// Broad target acquisition limits in world units.
        pub horizontal_range: f32,
        pub vertical_range: f32,
        // Follow limits keep the acquired opponent through its get-up travel;
        // acquisition limits above decide whether to acquire it in the first place.
        pub follow_range: f32,
        pub follow_vertical_range: f32,
        /// Preserve player movement between response inputs.
        pub manual_spacing: bool,
        /// Player owns spacing; normal/jump/grab timing does not test reach.
        pub distance_agnostic: bool,
        /// Baseline horizontal walk strength: normal cap is -0.02, near-full
        /// horizontal cap is +0.03, both bounded to 0.1..0.45.
        pub walk_strength: f32,
        pub auto_position: bool,
        /// Heuristic stage-side stance offset, not a universal optimal location.
        pub stand_distance: f32,
        // Unlike distances, move_strength is normalized stick magnitude (0..1).
        pub move_strength: f32,
        /// Legacy parsed setting. Captures always return control to the player.
        pub throw_outward: bool,
        /// Native mask of the button mapped to Grab; R for this user's controls.
        pub grab_button: u32,
    }

    impl Default for AssistSettings {
        fn default() -> Self {
            Self {
                implementation: Some(crate::ledge_trap::ResponseImplementation::Kirby),
                fighter_kind: crate::fighter_kinds::KIRBY,
                response_choices: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
                sequence_overrides: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
                fighter: FighterGrabData::from_kind(6),
                responses: LedgeResponses::default(),
                enabled: false,
                drop_defense: true,
                horizontal_range: 12.0,
                vertical_range: 10.0,
                follow_range: 40.0,
                follow_vertical_range: 48.0,
                manual_spacing: true,
                distance_agnostic: true,
                walk_strength: 0.35,
                auto_position: true,
                stand_distance: 8.0,
                move_strength: 1.0,
                throw_outward: false,
                grab_button: MAPPER_R,
            }
        }
    }
}
