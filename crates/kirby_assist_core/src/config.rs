//! Declarative rules: what situation activates a tool, when it fires, and what
//! logical input it requests. These types neither detect game state nor install
//! hooks. Shared text settings are parsed in live_input.rs; fighter
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

/// Experimental DI/SDI support; independent runtime preferences default OFF.
pub const AUTO_DAMAGE_STICK_ENABLED: bool = true;

/// Top-level configuration. Rules are disabled by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssistConfig {
    /// Stores whether `AssistConfig` is enabled.
    pub enabled: bool,
    /// Runtime scope value used by AssistConfig..
    pub scope: AssistScope,
    // Fixed-size storage avoids allocating a vector whenever policy is evaluated.
    /// Runtime rules value used by AssistConfig..
    pub rules: [ScenarioRule; MAX_SCENARIO_RULES],
}

impl Default for AssistConfig {
    /// Returns the default value for this type.
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
    /// Returns `false` for `Scenario::None` or when no empty slot remains.
    /// Disabled rules retain their settings until explicitly removed.
    pub fn set_rule(&mut self, rule: ScenarioRule) -> bool {
        if rule.scenario == Scenario::None {
            return false;
        }

        // Search existing entries first to avoid duplicates; `Scenario::None`
        // marks an empty slot.
        let position = |scenario| self.rules.iter().position(|r| r.scenario == scenario);
        let Some(slot) = position(rule.scenario).or_else(|| position(Scenario::None)) else {
            return false;
        };
        self.rules[slot] = rule;
        true
    }

    /// Returns mutable access to the configured rule for the requested scenario.
    fn rule_mut(&mut self, scenario: Scenario) -> Option<&mut ScenarioRule> {
        if scenario == Scenario::None {
            return None;
        }
        self.rules.iter_mut().find(|rule| rule.scenario == scenario)
    }

    /// Toggle a tool without losing its recipe or timing.
    pub fn set_rule_enabled(&mut self, scenario: Scenario, enabled: bool) -> bool {
        self.rule_mut(scenario)
            .map(|rule| rule.enabled = enabled)
            .is_some()
    }

    /// Remove a tool and free its configuration slot.
    pub fn remove_rule(&mut self, scenario: Scenario) -> bool {
        self.rule_mut(scenario)
            .map(|rule| *rule = ScenarioRule::DISABLED)
            .is_some()
    }

    // Disabled entries are intentionally ignored, but retain their settings so
    // toggling a tool back on does not discard its timing/custom recipe.
    /// Returns the enabled rule configured for the requested scenario.
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
    /// Controls whether `AssistScope` requires local human control.
    pub require_locally_human_controlled: bool,
    /// Controls whether `AssistScope` requires Kirby.
    pub require_kirby: bool,
}

impl AssistScope {
    // `!flag || condition` means "only require condition when flag is true".
    // This scope is independent of game mode; the live adapter separately
    // validates its supported native controller/fighter before using the engine.
    /// Returns whether the supplied context is permitted by this scope.
    pub fn allows(self, context: RunContext) -> bool {
        (!self.require_locally_human_controlled || context.is_locally_human_controlled)
            && (!self.require_kirby || context.fighter == FighterKind::Kirby)
    }
}

impl Default for AssistScope {
    /// Returns the default value for this type.
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
    ControllerRecipe(
        /// Carries the logical controller recipe in `ScenarioAction`.
        InputRecipe,
    ),
    /// A future feature for replaying a saved multi-frame input sequence.
    /// It is deliberately non-functional until a playback backend is written.
    PlaybackSlot(
        /// Carries the reserved recording slot in `ScenarioAction`.
        RecordingSlot,
    ),
}

/// Slot names reserve an understandable API for a future input recorder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingSlot {
    /// `One` option in the `RecordingSlot` Smash policy state.
    One,
    /// `Two` option in the `RecordingSlot` Smash policy state.
    Two,
    /// `Three` option in the `RecordingSlot` Smash policy state.
    Three,
    /// `Four` option in the `RecordingSlot` Smash policy state.
    Four,
    /// `Five` option in the `RecordingSlot` Smash policy state.
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
    /// `IMMEDIATELY_ONCE` is a configuration value used by the Smash runtime.
    pub const IMMEDIATELY_ONCE: Self = Self {
        start_after_frames: 0,
        active_for_frames: 1,
        repeat_every_frames: 0,
    };

    /// Returns the `after` result.
    pub const fn after(start_after_frames: u16) -> Self {
        Self {
            start_after_frames,
            ..Self::IMMEDIATELY_ONCE
        }
    }

    /// Returns the `pulse_every` result.
    pub const fn pulse_every(repeat_every_frames: u16) -> Self {
        Self {
            repeat_every_frames,
            ..Self::IMMEDIATELY_ONCE
        }
    }

    // Frame zero belongs to the scenario. After the initial delay, modulo (%)
    // repeats a pulse pattern: interval 3 / duration 1 means on, off, off, repeat.
    // The live caller supplies observed input updates, not a universal game clock.
    /// Checks whether the timing rule is active at the supplied scenario age.
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
    /// Stores whether `ScenarioRule` is enabled.
    pub enabled: bool,
    /// Runtime scenario value used by ScenarioRule..
    pub scenario: Scenario,
    /// Runtime timing value used by ScenarioRule..
    pub timing: RuleTiming,
    /// Runtime action value used by ScenarioRule..
    pub action: ScenarioAction,
}

impl ScenarioRule {
    /// `DISABLED` is a configuration value used by the Smash runtime.
    pub const DISABLED: Self = Self {
        enabled: false,
        scenario: Scenario::None,
        timing: RuleTiming::IMMEDIATELY_ONCE,
        action: ScenarioAction::ControllerRecipe(InputRecipe::NONE),
    };

    /// Returns the `controller` result.
    pub const fn controller(scenario: Scenario, timing: RuleTiming, recipe: InputRecipe) -> Self {
        Self {
            enabled: true,
            scenario,
            timing,
            action: ScenarioAction::ControllerRecipe(recipe),
        }
    }

    /// Returns the `playback` result.
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
#[path = "../tests/unit/config.rs"]
mod tests;
pub mod button_bindings {
    //! Stable identities for mode switches, compatibility bindings and built-ins.
    //!
    //! `Mode` names a tool-level switch; `ModeBinding` records historical
    //! hold/toggle assignments. Parsing an identity is not proof the current
    //! menu offers it or the native adapter admits it. Recipe validation and
    //! reaction-control whitelists impose separate restrictions.
    //!
    //! `Builtin` selects an input helper without a user-authored action tape.
    //! Its serialized token, visible menu offset and native activation path
    //! must stay aligned when adding a helper. Do not rename saved tokens to
    //! improve display wording, or expose a mode by changing this enum alone.
    //! See `docs/CONTROLS.md` and `docs/SETTINGS_REFERENCE.md` for user workflows.
    /// Available `Mode` choices for the Smash assistance menu.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        /// `LedgeTrap` option in the `Mode` Smash policy state.
        LedgeTrap,
        /// `AutoTech` option in the `Mode` Smash policy state.
        AutoTech,
        /// `AutoMash` option in the `Mode` Smash policy state.
        AutoMash,
        /// `IdleCrouch` option in the `Mode` Smash policy state.
        IdleCrouch,
        /// `TiltTurn` option in the `Mode` Smash policy state.
        TiltTurn,
        /// `SpotPunish` option in the `Mode` Smash policy state.
        SpotPunish,
        /// `OutOfShield` option in the `Mode` Smash policy state.
        OutOfShield,
    }
    impl Mode {
        /// `ALL` is a configuration value used by the Smash runtime.
        pub const ALL: [Self; 7] = [
            Self::LedgeTrap,
            Self::AutoTech,
            Self::AutoMash,
            Self::IdleCrouch,
            Self::TiltTurn,
            Self::SpotPunish,
            Self::OutOfShield,
        ];
        /// Tests whether the requested bit is set in the value.
        pub fn bit(self) -> u16 {
            1 << self as usize
        }
        /// Returns the native identifier of the observed object.
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
        /// Returns the display label for the value.
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
        /// Parses the supplied source into the module’s validated representation.
        pub fn parse(s: &str) -> Option<Self> {
            Self::ALL.into_iter().find(|m| m.id() == s)
        }
        /// Checks whether the current value supports the requested operation.
        pub fn supports(self, kind: i32) -> bool {
            self != Self::IdleCrouch || kind == crate::fighter_kinds::KIRBY
        }
    }
    /// Available `Activation` choices for the Smash assistance menu.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum Activation {
        #[default]
        /// `Hold` option in the `Activation` Smash policy state.
        Hold,
        /// `Toggle` option in the `Activation` Smash policy state.
        Toggle,
    }
    impl Activation {
        /// Returns the native identifier of the observed object.
        pub fn id(self) -> &'static str {
            match self {
                Self::Hold => "hold",
                Self::Toggle => "toggle",
            }
        }
    }
    /// Runtime Smash data carried by `ModeBinding`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ModeBinding {
        /// Runtime mode value used by ModeBinding..
        pub mode: Mode,
        /// Runtime activation value used by ModeBinding..
        pub activation: Activation,
    }
    impl ModeBinding {
        /// `COUNT` is a configuration value used by the Smash runtime.
        pub const COUNT: usize = Mode::ALL.len() * 2;
        /// Converts a millisecond offset into a `Duration`.
        pub fn at(index: usize) -> Option<Self> {
            Some(Self {
                mode: *Mode::ALL.get(index / 2)?,
                activation: if index.is_multiple_of(2) {
                    Activation::Hold
                } else {
                    Activation::Toggle
                },
            })
        }
        /// Returns the index of the selected entry.
        pub fn index(self) -> usize {
            self.mode as usize * 2 + usize::from(self.activation == Activation::Toggle)
        }
        /// Returns the display label for the value.
        pub fn label(self) -> &'static str {
            /// `LABELS` is a configuration value used by the Smash runtime.
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
        /// Returns the stable identifier of the selected choice.
        pub fn choice_id(self) -> &'static str {
            /// `IDS` is a configuration value used by the Smash runtime.
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
        /// `Normal` option in the `Builtin` Smash policy state.
        Normal,
        /// `LedgeAssist` option in the `Builtin` Smash policy state.
        LedgeAssist,
        /// `TurboTeabag` option in the `Builtin` Smash policy state.
        TurboTeabag,
        /// `LedgeTrump` option in the `Builtin` Smash policy state.
        LedgeTrump,
        /// Hold-only universal spacing and configured attack assistance.
        AutoSpacing,
        Counter,
        AutoSpacingToggle,
        CounterToggle,
    }
    impl Builtin {
        /// Native helpers retained for loading existing saved configurations.
        pub const ALL: [Self; 8] = [
            Self::Normal,
            Self::LedgeAssist,
            Self::TurboTeabag,
            Self::LedgeTrump,
            Self::AutoSpacing,
            Self::Counter,
            Self::AutoSpacingToggle,
            Self::CounterToggle,
        ];
        /// Selectable native helpers; legacy ledge assistance remains loadable only.
        pub const MENU: [Self; 7] = [
            Self::Normal,
            Self::TurboTeabag,
            Self::LedgeTrump,
            Self::AutoSpacing,
            Self::Counter,
            Self::AutoSpacingToggle,
            Self::CounterToggle,
        ];
        /// Returns the serialized token for the value.
        pub fn token(self) -> &'static str {
            match self {
                Self::Normal => "normal",
                Self::LedgeAssist => "ledge_assist",
                Self::TurboTeabag => "turbo_teabag",
                Self::LedgeTrump => "ledge_trump",
                Self::AutoSpacing => "auto_spacing",
                Self::Counter => "counter_hold",
                Self::AutoSpacingToggle => "auto_spacing_toggle",
                Self::CounterToggle => "counter_toggle",
            }
        }
        /// Returns the display label for the value.
        pub fn label(self) -> &'static str {
            match self {
                Self::Normal => "Short hop",
                Self::LedgeAssist => "Built-in: Ledge assist / walk",
                Self::TurboTeabag => "Built-in: Turbo t-bag",
                Self::LedgeTrump => "Built-in: Ledge trump",
                Self::AutoSpacing => "Built-in: Auto-Spacing & Attack (hold)",
                Self::Counter => "Built-in: Counter (hold)",
                Self::AutoSpacingToggle => "Built-in: Auto-Spacing & Attack (toggle)",
                Self::CounterToggle => "Built-in: Counter (toggle)",
            }
        }
        /// Returns the stable identifier of the selected choice.
        pub fn choice_id(self) -> &'static str {
            match self {
                Self::Normal => "__normal_button",
                Self::LedgeAssist => "__builtin_ledge_assist",
                Self::TurboTeabag => "__builtin_turbo_teabag",
                Self::LedgeTrump => "__builtin_ledge_trump",
                Self::AutoSpacing => "__builtin_auto_spacing",
                Self::Counter => "__builtin_counter_hold",
                Self::AutoSpacingToggle => "__builtin_auto_spacing_toggle",
                Self::CounterToggle => "__builtin_counter_toggle",
            }
        }
        /// Parses the supplied source into the module’s validated representation.
        pub fn parse(value: &str) -> Option<Self> {
            match value {
                // Retired saved helpers must not reactivate or invalidate the catalog.
                "short_hop" | "auto_defense_hold" | "auto_defense_toggle" => Some(Self::Normal),
                _ => Self::ALL.into_iter().find(|b| b.token() == value),
            }
        }
        /// Builds the default the for.
        pub fn default_for(slot: crate::fighter_profiles::presets::PresetSlot) -> Self {
            match slot {
                crate::fighter_profiles::presets::PresetSlot::L3 => Self::AutoSpacing,
                crate::fighter_profiles::presets::PresetSlot::DpadDown => Self::TurboTeabag,
                _ => Self::Normal,
            }
        }
        /// Returns the legacy template offset used by saved startup defaults.
        pub fn template_offset(self) -> usize {
            match self {
                Self::Normal => 0,
                Self::LedgeAssist => 2,
                Self::TurboTeabag => 3,
                Self::LedgeTrump => 4,
                Self::AutoSpacing => 5,
                Self::Counter => 6,
                Self::AutoSpacingToggle => 7,
                Self::CounterToggle => 8,
            }
        }
        /// Returns the selected menu index for a visible button built-in.
        pub fn visible_template_offset(self) -> Option<usize> {
            match self {
                Self::Normal => Some(0),
                Self::TurboTeabag => {
                    Some(crate::fighter_profiles::presets::PresetSlot::B_REVERSE_OFFSET + 1)
                }
                Self::LedgeTrump => {
                    Some(crate::fighter_profiles::presets::PresetSlot::B_REVERSE_OFFSET + 2)
                }
                Self::AutoSpacing => {
                    Some(crate::fighter_profiles::presets::PresetSlot::B_REVERSE_OFFSET + 3)
                }
                Self::Counter => {
                    Some(crate::fighter_profiles::presets::PresetSlot::B_REVERSE_OFFSET + 4)
                }
                Self::AutoSpacingToggle => {
                    Some(crate::fighter_profiles::presets::PresetSlot::B_REVERSE_OFFSET + 5)
                }
                Self::CounterToggle => {
                    Some(crate::fighter_profiles::presets::PresetSlot::B_REVERSE_OFFSET + 6)
                }
                Self::LedgeAssist => None,
            }
        }
    }
}
#[path = "policies/option_read.rs"]
pub mod option_read;
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
        /// Runtime responses value used by AssistSettings..
        pub responses: LedgeResponses,
        /// Stores whether `AssistSettings` is enabled.
        pub enabled: bool,
        /// Enable the selected response to a witnessed generic ledge release.
        pub drop_defense: bool,
        /// Native-time budget to request/confirm the drop punish before shielding.
        // Ranges/heights/stance distances are WORLD units, not stick percentages.
        // Increasing reach only makes the policy willing to attack from farther
        // away; it does not enlarge Kirby's real hitbox and can create more whiffs.
        /// Broad target acquisition limits in world units.
        pub horizontal_range: f32,
        /// Runtime vertical range value used by AssistSettings..
        pub vertical_range: f32,
        // Follow limits keep the acquired opponent through its get-up travel;
        // acquisition limits above decide whether to acquire it in the first place.
        /// Runtime follow range value used by AssistSettings..
        pub follow_range: f32,
        /// Runtime follow vertical range value used by AssistSettings..
        pub follow_vertical_range: f32,
        /// Preserve player movement between response inputs.
        pub manual_spacing: bool,
        /// Player owns spacing; normal/jump/grab timing does not test reach.
        pub distance_agnostic: bool,
        /// Baseline horizontal walk strength: normal cap is -0.02, near-full
        /// horizontal cap is +0.03, both bounded to 0.1..0.45.
        pub walk_strength: f32,
        /// Runtime auto position value used by AssistSettings..
        pub auto_position: bool,
        /// Heuristic stage-side stance offset, not a universal optimal location.
        pub stand_distance: f32,
        // Unlike distances, move_strength is normalized stick magnitude (0..1).
        /// Runtime move strength value used by AssistSettings..
        pub move_strength: f32,
        /// Legacy parsed setting. Captures always return control to the player.
        pub throw_outward: bool,
        /// Native mask of the button mapped to Grab; R for this user's controls.
        pub grab_button: u32,
    }

    impl Default for AssistSettings {
        /// Returns the default value for this type.
        fn default() -> Self {
            Self {
                implementation: Some(crate::ledge_trap::ResponseImplementation::Kirby),
                fighter_kind: crate::fighter_kinds::KIRBY,
                response_choices: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
                sequence_overrides: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
                fighter: FighterGrabData::from_kind(crate::fighter_kinds::KIRBY),
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
