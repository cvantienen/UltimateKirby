//! Bounded, input-only roll reaction rules. No native queries or file access.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::InputTimeline,
    roll_read::{Target, ESCAPE_B, ESCAPE_F},
    sequence_config::{Action, Sequence},
    settings::AssistSettings,
};
/// Determines whether the `starts_direct_up_b` condition holds.
/// Checks whether starts direct up b.
pub fn starts_direct_up_b(sequence: &Sequence) -> bool {
    automatic_watch::starts_up_b(sequence)
}
mod automatic_watch;
mod scope;
pub use automatic_watch::{sample_optional_simple, simple_readiness, SimpleStart};
pub use scope::{
    automatic_fighter_target_available, automatic_follow_target_available, automatic_start_allowed,
    automatic_start_status, automatic_target_available, automatic_up_b_receipt_status,
    automatic_up_b_status, AutomaticObservationScope,
};
/// An automatic reaction tape: delivered holds survive observation gaps and a
/// pending Grab/Up-B is rejected after manual takeoff.
pub(crate) fn automatic_plan(sequence: Sequence, enemy: Opponent) -> InputTimeline {
    let mut plan = InputTimeline::new(sequence, enemy);
    plan.allow_continuous_progress();
    plan.guard_automatic_ground_actions();
    plan
}
/// Per-actor memory of a native event that may already have been answered.
pub(crate) trait Tracked: Copy {
    /// Returns the native identifier of the observed object.
    fn id(&self) -> u32;
    /// Returns an optional `status` value.
    fn status(&self) -> i32;
}
/// Positive status evidence that an actor left its remembered event.
pub(crate) fn forget_ended<T: Tracked, const N: usize>(
    tracks: &mut [Option<T>],
    known: &[Option<(u32, i32)>; N],
) {
    for track in tracks {
        if track.is_some_and(|t| {
            known
                .iter()
                .flatten()
                .any(|(id, status)| *id == t.id() && *status != t.status())
        }) {
            *track = None;
        }
    }
}
/// Missing samples are not evidence of a new event: keep selected memories of
/// actors absent from this poll in spare slots until a different status is seen.
pub(crate) fn retain_missing<T: Tracked>(
    tracks: &mut [Option<T>],
    old: impl IntoIterator<Item = T>,
    observed: impl Iterator<Item = (u32, i32)> + Clone,
    keep: impl Fn(&T) -> bool,
) {
    for previous in old.into_iter().filter(|p| keep(p)) {
        if tracks.iter().flatten().any(|t| t.id() == previous.id())
            || observed
                .clone()
                .any(|(id, status)| id == previous.id() && status != previous.status())
        {
            continue;
        }
        if let Some(empty) = tracks.iter_mut().find(|t| t.is_none()) {
            *empty = Some(previous);
        }
    }
}
/// Sets the upper bound for rules per mode.
/// Holds the MAX RULES PER MODE value.
pub const MAX_RULES_PER_MODE: usize = 8;
/// Sets the upper bound for rules.
/// Holds the MAX RULES value.
pub const MAX_RULES: usize = MAX_RULES_PER_MODE * 7;

/// Automatic reaction slots, kept in one place so menu, cache, and runtime
/// code cannot disagree about what a slot means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutomaticEvent {
    /// Opponent ground roll.
    Roll,
    /// Opponent spot dodge.
    Spot,
    /// Opponent air dodge.
    Air,
    /// Shield grab response.
    ShieldGrab,
    /// Grab-dodge follow-up.
    GrabDodged,
    /// Shield backup response.
    ShieldBackup,
}
impl AutomaticEvent {
    /// All events in their stable serialized/cache order.
    pub const ALL: [Self; 6] = [
        Self::Roll,
        Self::Spot,
        Self::Air,
        Self::ShieldGrab,
        Self::GrabDodged,
        Self::ShieldBackup,
    ];
    /// Returns the stable slot index.
    pub const fn index(self) -> usize {
        match self {
            Self::Roll => 0,
            Self::Spot => 1,
            Self::Air => 2,
            Self::ShieldGrab => 3,
            Self::GrabDodged => 4,
            Self::ShieldBackup => 5,
        }
    }
    /// Observation requirements for independently enabled automatic families.
    pub const fn enabled_mask(reactions: bool, shield: bool, grab_dodge: bool) -> u8 {
        let reactions = if reactions {
            (1 << Self::Roll.index()) | (1 << Self::Spot.index()) | (1 << Self::Air.index())
        } else { 0 };
        let shield = if shield {
            (1 << Self::ShieldGrab.index()) | (1 << Self::ShieldBackup.index())
        } else { 0 };
        let dodge = if grab_dodge { 1 << Self::GrabDodged.index() } else { 0 };
        reactions | shield | dodge
    }
    /// Converts a serialized trigger into its automatic event identity.
    pub const fn from_trigger(trigger: Trigger) -> Option<Self> {
        match trigger {
            Trigger::AutoRoll => Some(Self::Roll),
            Trigger::AutoSpot => Some(Self::Spot),
            Trigger::AutoAir => Some(Self::Air),
            Trigger::AutoShieldGrab => Some(Self::ShieldGrab),
            Trigger::AutoGrabDodged => Some(Self::GrabDodged),
            Trigger::AutoShieldBackup => Some(Self::ShieldBackup),
            _ => None,
        }
    }
    /// Converts an automatic event back to its configured trigger.
    pub const fn trigger(self) -> Trigger {
        match self {
            Self::Roll => Trigger::AutoRoll,
            Self::Spot => Trigger::AutoSpot,
            Self::Air => Trigger::AutoAir,
            Self::ShieldGrab => Trigger::AutoShieldGrab,
            Self::GrabDodged => Trigger::AutoGrabDodged,
            Self::ShieldBackup => Trigger::AutoShieldBackup,
        }
    }
}

/// Serialized event identities used to select a saved response.
///
/// An event says *what was observed*, not which move to perform. The sequence's
/// actions supply controller inputs; own-state/range/side conditions admit it.
/// `Auto*` events use the simplified distance/height configuration. Other events
/// retain detailed conditions. A token remaining in this enum is not proof of
/// live support: `LedgeRegrab`, for example, is retained but inert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trigger {
    #[default]
    /// `Off` option in the `Trigger` Smash policy state.
    Off,
    /// `Roll` option in the `Trigger` Smash policy state.
    Roll,
    /// `Crossed` option in the `Trigger` Smash policy state.
    Crossed,
    /// `Vulnerable` option in the `Trigger` Smash policy state.
    Vulnerable,
    /// `CrossVulnerable` option in the `Trigger` Smash policy state.
    CrossVulnerable,
    /// `ShieldHit` option in the `Trigger` Smash policy state.
    ShieldHit,
    /// `GrabDodged` option in the `Trigger` Smash policy state.
    GrabDodged,
    /// `LedgeRegrab` option in the `Trigger` Smash policy state.
    LedgeRegrab,
    /// `SpotDodge` option in the `Trigger` Smash policy state.
    SpotDodge,
    /// `EnemyShield` option in the `Trigger` Smash policy state.
    EnemyShield,
    /// `ShieldRoll` option in the `Trigger` Smash policy state.
    ShieldRoll,
    /// `ShieldCross` option in the `Trigger` Smash policy state.
    ShieldCross,
    /// `ShieldVulnerable` option in the `Trigger` Smash policy state.
    ShieldVulnerable,
    /// `ShieldCrossVulnerable` option in the `Trigger` Smash policy state.
    ShieldCrossVulnerable,
    /// `ShieldAway` option in the `Trigger` Smash policy state.
    ShieldAway,
    /// `ShieldAwayVulnerable` option in the `Trigger` Smash policy state.
    ShieldAwayVulnerable,
    /// `ShieldPressureVulnerable` option in the `Trigger` Smash policy state.
    ShieldPressureVulnerable,
    /// `ShieldHitVulnerable` option in the `Trigger` Smash policy state.
    ShieldHitVulnerable,
    /// `GrabDodgedVulnerable` option in the `Trigger` Smash policy state.
    GrabDodgedVulnerable,
    /// `SpotDodgeAny` option in the `Trigger` Smash policy state.
    SpotDodgeAny,
    /// `AutoRoll` option in the `Trigger` Smash policy state.
    AutoRoll,
    /// `AutoSpot` option in the `Trigger` Smash policy state.
    AutoSpot,
    /// `AutoAir` option in the `Trigger` Smash policy state.
    AutoAir,
    /// `AutoShieldGrab` option in the `Trigger` Smash policy state.
    AutoShieldGrab,
    /// `AutoGrabDodged` option in the `Trigger` Smash policy state.
    AutoGrabDodged,
    /// `AutoShieldBackup` option in the `Trigger` Smash policy state.
    AutoShieldBackup,
}
impl Trigger {
    /// Automatic events exposed in the simplified rules menu.
    pub const AUTO_EVENTS: [Self; 6] = [
        AutomaticEvent::Roll.trigger(),
        AutomaticEvent::Spot.trigger(),
        AutomaticEvent::Air.trigger(),
        AutomaticEvent::ShieldGrab.trigger(),
        AutomaticEvent::GrabDodged.trigger(),
        AutomaticEvent::ShieldBackup.trigger(),
    ];
    /// Returns the `auto_name` result.
    pub fn auto_name(self) -> &'static str {
        match self {
            Self::AutoRoll => "Roll",
            Self::AutoSpot => "Enemy spot dodge",
            Self::AutoAir => "Enemy air dodge",
            Self::AutoShieldGrab => "Automatic grab (shielded)",
            Self::AutoGrabDodged => "After dodging enemy grab",
            Self::AutoShieldBackup => "Shield grab backup",
            _ => "Reaction",
        }
    }

    /// `ALL` is a configuration value used by the Smash runtime.
    pub const ALL: [Self; 26] = [
        Self::Off,
        Self::Roll,
        Self::Crossed,
        Self::Vulnerable,
        Self::CrossVulnerable,
        Self::ShieldHit,
        Self::GrabDodged,
        Self::LedgeRegrab,
        Self::SpotDodge,
        Self::EnemyShield,
        Self::ShieldRoll,
        Self::ShieldCross,
        Self::ShieldVulnerable,
        Self::ShieldCrossVulnerable,
        Self::ShieldAway,
        Self::ShieldAwayVulnerable,
        Self::ShieldPressureVulnerable,
        Self::ShieldHitVulnerable,
        Self::GrabDodgedVulnerable,
        Self::SpotDodgeAny,
        Self::AutoRoll,
        Self::AutoSpot,
        Self::AutoAir,
        Self::AutoShieldGrab,
        Self::AutoGrabDodged,
        Self::AutoShieldBackup,
    ];
    /// Returns the `family` result.
    pub fn family(self) -> usize {
        match self {
            t if t.simple().is_some() => 6,
            Self::ShieldHit | Self::ShieldHitVulnerable => 1,
            Self::GrabDodged | Self::GrabDodgedVulnerable => 2,
            Self::LedgeRegrab => 3,
            Self::SpotDodge | Self::SpotDodgeAny => 4,
            t if t.shield_pressure() => 5,
            _ => 0,
        }
    }
    /// Returns an optional `simple` value.
    pub fn simple(self) -> Option<usize> {
        AutomaticEvent::from_trigger(self).map(AutomaticEvent::index)
    }
    /// Determines whether the `shield_pressure` condition holds.
    pub fn shield_pressure(self) -> bool {
        matches!(
            self,
            Self::EnemyShield
                | Self::ShieldRoll
                | Self::ShieldCross
                | Self::ShieldVulnerable
                | Self::ShieldCrossVulnerable
                | Self::ShieldAway
                | Self::ShieldAwayVulnerable
                | Self::ShieldPressureVulnerable
        )
    }
    /// `SHIELD_EVENTS` is a configuration value used by the Smash runtime.
    pub const SHIELD_EVENTS: [Self; 8] = [
        Self::EnemyShield,
        Self::ShieldRoll,
        Self::ShieldCross,
        Self::ShieldVulnerable,
        Self::ShieldCrossVulnerable,
        Self::ShieldAway,
        Self::ShieldAwayVulnerable,
        Self::ShieldPressureVulnerable,
    ];
    /// Determines whether the `defensive` condition holds.
    pub fn defensive(self) -> bool {
        self.shield_pressure()
            || matches!(
                self,
                Self::ShieldHit
                    | Self::ShieldHitVulnerable
                    | Self::GrabDodged
                    | Self::GrabDodgedVulnerable
                    | Self::LedgeRegrab
                    | Self::SpotDodge
                    | Self::SpotDodgeAny
            )
    }
    /// Determines whether the `event_matches` condition holds.
    pub fn event_matches(self, event: Self) -> bool {
        self == event
            || matches!(
                (self, event),
                (Self::ShieldHitVulnerable, Self::ShieldHit)
                    | (Self::GrabDodgedVulnerable, Self::GrabDodged)
                    | (Self::SpotDodgeAny, Self::SpotDodge)
            )
    }
    /// Determines whether vulnerable.
    pub fn needs_vulnerable(self) -> bool {
        matches!(
            self,
            Self::ShieldHitVulnerable
                | Self::GrabDodgedVulnerable
                | Self::SpotDodge
                | Self::ShieldPressureVulnerable
        )
    }
    /// Returns the `events` result.
    pub fn events(self) -> &'static [Self] {
        match self.family() {
            6 => &Self::AUTO_EVENTS,
            0 => &[
                Self::Roll,
                Self::Crossed,
                Self::Vulnerable,
                Self::CrossVulnerable,
            ],
            1 => &[Self::ShieldHit, Self::ShieldHitVulnerable],
            2 => &[Self::GrabDodged, Self::GrabDodgedVulnerable],
            3 => &[Self::LedgeRegrab],
            4 => &[Self::SpotDodge, Self::SpotDodgeAny],
            _ => &Self::SHIELD_EVENTS,
        }
    }
    /// Returns the serialized token for the value.
    pub fn token(self) -> &'static str {
        match self {
            Self::AutoShieldGrab => "auto_shield_grab",
            Self::AutoGrabDodged => "auto_grab_dodged",
            Self::AutoShieldBackup => "auto_shield_backup",
            Self::AutoRoll => "auto_roll",
            Self::AutoSpot => "auto_spot_dodge",
            Self::AutoAir => "auto_air_dodge",
            Self::Off => "off",
            Self::Roll => "roll",
            Self::Crossed => "roll_cross",
            Self::Vulnerable => "roll_vulnerable",
            Self::CrossVulnerable => "roll_cross_vulnerable",
            Self::ShieldHit => "shield_hit",
            Self::GrabDodged => "grab_dodged",
            Self::LedgeRegrab => "ledge_regrab",
            Self::SpotDodge => "spot_dodge",
            Self::EnemyShield => "enemy_shield",
            Self::ShieldRoll => "shield_roll",
            Self::ShieldCross => "shield_roll_cross",
            Self::ShieldVulnerable => "shield_roll_vulnerable",
            Self::ShieldCrossVulnerable => "shield_roll_cross_vulnerable",
            Self::ShieldAway => "shield_roll_away",
            Self::ShieldAwayVulnerable => "shield_roll_away_vulnerable",
            Self::ShieldPressureVulnerable => "shield_pressure_vulnerable",
            Self::ShieldHitVulnerable => "shield_hit_vulnerable",
            Self::GrabDodgedVulnerable => "grab_dodged_vulnerable",
            Self::SpotDodgeAny => "spot_dodge_any",
        }
    }
}
/// Available `SelfState` choices for the Smash assistance menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelfState {
    #[default]
    /// `Any` option in the `SelfState` Smash policy state.
    Any,
    /// `Grounded` option in the `SelfState` Smash policy state.
    Grounded,
    /// `Airborne` option in the `SelfState` Smash policy state.
    Airborne,
    /// `Standing` option in the `SelfState` Smash policy state.
    Standing,
    /// `Walking` option in the `SelfState` Smash policy state.
    Walking,
    /// `Running` option in the `SelfState` Smash policy state.
    Running,
    /// `Shielding` option in the `SelfState` Smash policy state.
    Shielding,
}
impl SelfState {
    /// `ALL` is a configuration value used by the Smash runtime.
    pub const ALL: [Self; 7] = [
        Self::Any,
        Self::Grounded,
        Self::Airborne,
        Self::Standing,
        Self::Walking,
        Self::Running,
        Self::Shielding,
    ];
    /// Returns the serialized token for the value.
    pub fn token(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Grounded => "grounded",
            Self::Airborne => "airborne",
            Self::Standing => "standing",
            Self::Walking => "walking",
            Self::Running => "running",
            Self::Shielding => "shielding",
        }
    }
    /// Determines whether the `matches` condition holds.
    pub(crate) fn matches(self, c: GameplayContext) -> bool {
        match self {
            Self::Any => true,
            Self::Grounded => c.situation == crate::native_states::situation::GROUND,
            Self::Airborne => c.situation == crate::native_states::situation::AIR,
            Self::Standing => {
                c.situation == crate::native_states::situation::GROUND
                    && c.status == crate::native_states::status::WAIT
            }
            Self::Walking => {
                c.situation == crate::native_states::situation::GROUND
                    && c.status == crate::native_states::status::WALK
            }
            Self::Running => {
                c.situation == crate::native_states::situation::GROUND
                    && matches!(
                        c.status,
                        crate::native_states::status::DASH
                            | crate::native_states::status::RUN
                            | crate::native_states::status::TURN_DASH
                            | crate::native_states::status::TURN_RUN
                    )
            }
            Self::Shielding => {
                c.situation == crate::native_states::situation::GROUND
                    && matches!(
                        c.status,
                        crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
                    )
            }
        }
    }
}
/// Available `Side` choices for the Smash assistance menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    /// `Any` option in the `Side` Smash policy state.
    Any,
    /// `Front` option in the `Side` Smash policy state.
    Front,
    /// `Behind` option in the `Side` Smash policy state.
    Behind,
}
impl Side {
    /// `ALL` is a configuration value used by the Smash runtime.
    pub const ALL: [Self; 3] = [Self::Any, Self::Front, Self::Behind];
    /// Returns the serialized token for the value.
    pub fn token(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Front => "front",
            Self::Behind => "behind",
        }
    }
}
/// Activation group, independent of trigger conditions and response actions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Control {
    #[default]
    /// `Menu` option in the `Control` Smash policy state.
    Menu,
    /// `ToggleX` option in the `Control` Smash policy state.
    ToggleX,
    /// `ToggleY` option in the `Control` Smash policy state.
    ToggleY,
    /// `ToggleUp` option in the `Control` Smash policy state.
    ToggleUp,
    /// `ToggleDown` option in the `Control` Smash policy state.
    ToggleDown,
    /// `ToggleLeft` option in the `Control` Smash policy state.
    ToggleLeft,
    /// `ToggleRight` option in the `Control` Smash policy state.
    ToggleRight,
}
impl Control {
    /// `ALL` is a configuration value used by the Smash runtime.
    pub const ALL: [Self; 7] = [
        Self::Menu,
        Self::ToggleX,
        Self::ToggleY,
        Self::ToggleUp,
        Self::ToggleDown,
        Self::ToggleLeft,
        Self::ToggleRight,
    ];
    /// Tests whether the requested bit is set in the value.
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
    /// Returns the serialized token for the value.
    pub fn token(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::ToggleX => "toggle_x",
            Self::ToggleY => "toggle_y",
            Self::ToggleUp => "toggle_up",
            Self::ToggleDown => "toggle_down",
            Self::ToggleLeft => "toggle_left",
            Self::ToggleRight => "toggle_right",
        }
    }
    /// Returns the display label for the value.
    pub fn label(self) -> &'static str {
        match self {
            Self::Menu => "Menu",
            Self::ToggleX => "Toggle X",
            Self::ToggleY => "Toggle Y",
            Self::ToggleUp => "Toggle D-pad Up",
            Self::ToggleDown => "Toggle D-pad Down",
            Self::ToggleLeft => "Toggle D-pad Left",
            Self::ToggleRight => "Toggle D-pad Right",
        }
    }
    /// Returns the button associated with the binding.
    pub fn button(self) -> u32 {
        use crate::{manual_controls::*, native_input::*};
        match self {
            Self::Menu => 0,
            Self::ToggleX => MAPPER_X,
            Self::ToggleY => MAPPER_Y,
            Self::ToggleUp => DPAD_UP,
            Self::ToggleDown => crate::ledge_trap::DPAD_DOWN,
            Self::ToggleLeft => DPAD_LEFT,
            Self::ToggleRight => DPAD_RIGHT,
        }
    }
}
/// Runtime Smash data carried by `Rule`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    /// Runtime trigger value used by Rule..
    pub trigger: Trigger,
    /// Runtime state value used by Rule..
    pub state: SelfState,
    /// Runtime min value used by Rule..
    pub min: f32,
    /// Runtime max value used by Rule..
    pub max: f32,
    /// Runtime height value used by Rule..
    pub height: f32,
    /// Runtime side value used by Rule..
    pub side: Side,
    /// Runtime priority value used by Rule..
    pub priority: u8,
    /// Runtime control value used by Rule..
    pub control: Control,
}
impl Default for Rule {
    /// Returns the default value for this type.
    fn default() -> Self {
        Self {
            trigger: Trigger::Off,
            state: SelfState::Any,
            min: 0.,
            max: 25.,
            height: 12.,
            side: Side::Any,
            priority: 1,
            control: Control::Menu,
        }
    }
}
impl Rule {
    /// Checks whether the feature is active.
    pub fn active(self) -> bool {
        self.trigger != Trigger::Off
    }
    /// The first failing self-state/range/height/side gate for a target at
    /// `target`, in the order shown as feedback. Non-finite geometry never fits.
    pub(crate) fn rejects(self, c: GameplayContext, target: [f32; 2]) -> Option<Feedback> {
        if !self.state.matches(c) {
            return Some(Feedback::State);
        }
        let dx = target[0] - c.position[0];
        let dy = target[1] - c.position[1];
        if !dx.is_finite() || dx.abs() < self.min || dx.abs() > self.max {
            return Some(Feedback::Range);
        }
        if !dy.is_finite() || dy.abs() > self.height {
            return Some(Feedback::Height);
        }
        let in_front = dx * c.facing;
        if matches!(self.side, Side::Front) && in_front <= 0.001
            || matches!(self.side, Side::Behind) && in_front >= -0.001
        {
            return Some(Feedback::Side);
        }
        None
    }
    /// Returns the `field` result.
    pub fn field(self, n: u8) -> String {
        match n {
            0 => format!(
                "Auto reaction: {}",
                match self.trigger {
                    Trigger::AutoRoll => "Opponent roll (any)",
                    Trigger::AutoSpot => "Opponent spot dodge (any)",
                    Trigger::AutoAir => "Opponent air dodge (any)",
                    Trigger::ShieldPressureVulnerable => "Shield / shield roll vulnerable",
                    Trigger::ShieldHitVulnerable => "After shield hit: enemy vulnerable",
                    Trigger::GrabDodgedVulnerable => "After dodging grab: enemy vulnerable",
                    Trigger::SpotDodgeAny => "Any enemy spot dodge",
                    Trigger::EnemyShield => "Opponent holding shield",
                    Trigger::ShieldRoll => "Roll out of shield",
                    Trigger::ShieldCross => "Shield roll crossed me",
                    Trigger::ShieldVulnerable => "Shield roll vulnerable",
                    Trigger::ShieldCrossVulnerable => "Shield roll crossed + vulnerable",
                    Trigger::ShieldAway => "Roll away out of shield",
                    Trigger::ShieldAwayVulnerable => "Shield roll away + vulnerable",
                    Trigger::SpotDodge => "Opponent spot dodge vulnerable",
                    Trigger::ShieldHit => "After shield hit",
                    Trigger::GrabDodged => "After dodging grab",
                    Trigger::LedgeRegrab => "Vulnerable ledge hang / regrab",
                    _ => self.trigger.token(),
                }
            ),
            1 => format!("    My state: {}", self.state.token()),
            2 => format!("    Min X distance: {:.0}", self.min),
            3 => format!("    Max X distance: {:.0}", self.max),
            4 => format!("    Max height difference: {:.0}", self.height),
            5 => format!("    Enemy side: {}", self.side.token()),
            6 => format!("    Priority: {} (1 first)", self.priority),
            _ if self.trigger.shield_pressure() => {
                "    Mode control: Shield pressure ON/OFF".into()
            }
            _ if self.trigger.family() == Trigger::SpotDodge.family() => {
                "    Mode control: Spot punish ON/OFF".into()
            }
            _ if self.trigger.family() == Trigger::ShieldHit.family() => {
                "    Mode control: Out of shield ON/OFF".into()
            }
            _ if self.trigger == Trigger::LedgeRegrab => {
                "    Mode control: Ledge regrab punish (while ledge assist active)".into()
            }
            _ if self.trigger.family() == Trigger::GrabDodged.family() => {
                "    Mode control: Grab dodge + punish ON/OFF".into()
            }
            _ => format!(
                "    Mode control: {}{}",
                self.control.label(),
                if self.control == Control::Menu {
                    ""
                } else {
                    " (replaces button input)"
                }
            ),
        }
    }
    /// Updates the `adjust` state using the supplied inputs.
    pub fn adjust(&mut self, n: u8, up: bool) {
        /// Advances the selected choice to the next value.
        fn cycle<T: Copy + PartialEq>(v: &mut T, all: &[T], up: bool) {
            let i = all.iter().position(|x| x == v).unwrap_or(0);
            *v = all[(i + if up { 1 } else { all.len() - 1 }) % all.len()]
        }
        let d = if up { 1. } else { -1. };
        match n {
            0 => {
                cycle(&mut self.trigger, &Trigger::ALL, up);
                if self.trigger == Trigger::LedgeRegrab {
                    cycle(&mut self.trigger, &Trigger::ALL, up);
                }
                if self.trigger.defensive() {
                    self.control = Control::Menu;
                }
            }
            1 => cycle(&mut self.state, &SelfState::ALL, up),
            2 => self.min = (self.min + d).clamp(0., self.max),
            3 => self.max = (self.max + d).clamp(self.min, 100.),
            4 => self.height = (self.height + d).clamp(0., 100.),
            5 => cycle(&mut self.side, &Side::ALL, up),
            7 if self.trigger.defensive() => self.control = Control::Menu,
            7 => cycle(&mut self.control, &Control::ALL, up),
            _ => {
                self.priority = if up {
                    self.priority.saturating_add(1).min(8)
                } else {
                    self.priority.saturating_sub(1).max(1)
                }
            }
        }
    }
    /// Returns the `validate` result.
    pub fn validate(self) -> Result<(), String> {
        if (self.trigger.defensive() || self.trigger.simple().is_some())
            && self.control != Control::Menu
        {
            return Err("defensive reactions use their ON/OFF mode, not a button control".into());
        }
        if ![self.min, self.max, self.height]
            .iter()
            .all(|x| x.is_finite())
            || self.min < 0.
            || self.max < self.min
            || self.max > 100.
            || !(0. ..=100.).contains(&self.height)
            || !(1..=8).contains(&self.priority)
        {
            Err("reaction needs 0 <= min <= max <= 100, height 0..100 and priority 1..8".into())
        } else {
            Ok(())
        }
    }
    /// Parses the supplied source into the module’s validated representation.
    pub fn parse(s: &str) -> Result<Self, String> {
        let p: Vec<_> = s.trim().split(':').collect();
        if p.len() == 3 {
            let trigger = Trigger::ALL
                .into_iter()
                .find(|t| t.token() == p[0] && t.simple().is_some())
                .ok_or("short reaction syntax requires an auto-react event")?;
            let rule = Self {
                trigger,
                state: SelfState::Grounded,
                min: 0.,
                max: p[1].parse().map_err(|_| "invalid max distance")?,
                height: p[2].parse().map_err(|_| "invalid max height")?,
                side: Side::Any,
                priority: 1,
                control: Control::Menu,
            };
            rule.validate()?;
            return Ok(rule);
        }
        if p.len() != 7 && p.len() != 8 {
            return Err("reaction = TRIGGER:STATE:MIN:MAX:HEIGHT:SIDE:PRIORITY[:CONTROL]".into());
        }
        let r = Self {
            trigger: Trigger::ALL
                .into_iter()
                .find(|x| x.token() == p[0])
                .ok_or("unknown reaction trigger")?,
            state: SelfState::ALL
                .into_iter()
                .find(|x| x.token() == p[1])
                .ok_or("unknown reaction state")?,
            min: p[2].parse().map_err(|_| "invalid reaction min")?,
            max: p[3].parse().map_err(|_| "invalid reaction max")?,
            height: p[4].parse().map_err(|_| "invalid reaction height")?,
            side: Side::ALL
                .into_iter()
                .find(|x| x.token() == p[5])
                .ok_or("unknown reaction side")?,
            priority: p[6].parse().map_err(|_| "invalid reaction priority")?,
            control: match p.get(7) {
                None => Control::Menu,
                Some(value) => Control::ALL
                    .into_iter()
                    .find(|c| c.token() == *value)
                    .ok_or("unknown reaction control")?,
            },
        };
        r.validate()?;
        Ok(if r.trigger.simple().is_some() {
            Self {
                state: SelfState::Grounded,
                min: 0.,
                side: Side::Any,
                priority: 1,
                control: Control::Menu,
                ..r
            }
        } else {
            r
        })
    }
    /// Returns the serialized token for the value.
    pub fn token(self) -> String {
        if self.trigger.simple().is_some() {
            return format!("{}:{}:{}", self.trigger.token(), self.max, self.height);
        }
        let token = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.trigger.token(),
            self.state.token(),
            self.min,
            self.max,
            self.height,
            self.side.token(),
            self.priority
        );
        if self.control == Control::Menu {
            token
        } else {
            format!("{token}:{}", self.control.token())
        }
    }
}
/// Validates the response.
pub fn validate_response(s: Sequence) -> Result<(), String> {
    s.reaction.validate()?;
    crate::sequence_config::validate_facing_blocks(s.actions())?;
    if !s.reaction.active() {
        return Ok(());
    }
    if s.len == 0 || s.mode.is_some() {
        return Err("reaction requires an input recipe".into());
    }
    if s.actions().iter().any(|a| a.needs_ledge()) {
        return Err("onstage reaction has no ledge anchor; use opponent/away_from_opponent/facing/backward directions".into());
    }
    Ok(())
}
/// Runtime Smash data carried by `Candidate`.
#[derive(Clone, Copy)]
pub struct Candidate {
    /// Runtime slot value used by Candidate..
    pub slot: usize,
    /// Runtime index value used by Candidate..
    pub index: usize,
    /// Runtime sequence value used by Candidate..
    pub sequence: Sequence,
}
/// Runtime Smash data carried by `Resolved`.
pub struct Resolved {
    /// Runtime rules value used by Resolved..
    pub rules: Vec<Option<Candidate>>,
    /// Runtime len value used by Resolved..
    pub len: usize,
    /// Runtime controls value used by Resolved..
    pub controls: u8,
    /// Runtime enabled controls value used by Resolved..
    pub enabled_controls: u8,
    /// Runtime defense mask value used by Resolved..
    defense_mask: u32,
    /// Runtime simple value used by Resolved..
    simple: Option<Box<[Option<Sequence>; 7]>>,
    /// Runtime simple starts value used by Resolved..
    simple_starts: [Option<SimpleStart>; 3],
    /// Runtime automatic ground value used by Resolved..
    automatic_ground: [Option<(f32, f32)>; 7],
    /// Runtime automatic up special value used by Resolved..
    automatic_up_special: u8,
}
impl Default for Resolved {
    /// Returns the default value for this type.
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            len: 0,
            controls: 0,
            enabled_controls: Control::Menu.bit(),
            defense_mask: 0,
            simple: None,
            simple_starts: [None; 3],
            automatic_ground: [None; 7],
            automatic_up_special: 0,
        }
    }
}
impl Resolved {
    /// Returns a configured automatic sequence without exposing its array
    /// layout to callers.
    pub fn automatic_sequence(&self, event: AutomaticEvent) -> Option<&Sequence> {
        self.simple_sequence(event.index())
    }
    /// Returns an optional `simple_sequence` value.
    pub fn simple_sequence(&self, index: usize) -> Option<&Sequence> {
        self.simple
            .as_ref()
            .and_then(|s| s.get(index))
            .and_then(Option::as_ref)
    }
    /// Checks whether simple events.
    pub fn has_simple_events(&self) -> bool {
        (0..3).any(|i| self.simple_sequence(i).is_some())
    }
    /// Checks whether defense.
    pub fn has_defense(&self, trigger: Trigger) -> bool {
        let alias = match trigger {
            Trigger::ShieldHit => 1 << Trigger::ShieldHitVulnerable as u8,
            Trigger::GrabDodged => 1 << Trigger::GrabDodgedVulnerable as u8,
            Trigger::SpotDodge => 1 << Trigger::SpotDodgeAny as u8,
            _ => 0,
        };
        self.defense_mask & ((1 << trigger as u8) | alias) != 0
    }
    /// Sets the enabled controls.
    pub fn set_enabled_controls(&mut self, controls: u8) {
        self.enabled_controls = controls;
    }
    /// Returns the `add` result.
    pub fn add(&mut self, c: Candidate) -> Result<(), String> {
        if !c.sequence.reaction.active() {
            return Ok(());
        }
        validate_response(c.sequence)?;
        if self.len == MAX_RULES
            || self
                .rules
                .iter()
                .flatten()
                .filter(|r| {
                    r.sequence.reaction.trigger.family() == c.sequence.reaction.trigger.family()
                })
                .count()
                >= MAX_RULES_PER_MODE
        {
            return Err("At most 8 rules per assist mode per fighter".into());
        }
        if let Some(event) = AutomaticEvent::from_trigger(c.sequence.reaction.trigger) {
            let index = event.index();
            let simple = self.simple.get_or_insert_with(|| Box::new([None; 7]));
            if simple[index].is_some() {
                return Err(
                    "Auto-react supports one sequence per event; edit the existing event sequence"
                        .into(),
                );
            }
            simple[index] = Some(c.sequence);
            let up_special = starts_direct_up_b(&c.sequence);
            if up_special {
                self.automatic_up_special |= 1 << index;
            }
            if index < 3 {
                self.simple_starts[index] = Some(SimpleStart {
                    distance: c.sequence.reaction.max,
                    height: c.sequence.reaction.height,
                    up_special,
                });
            }
            if automatic_watch::needs_ground(&c.sequence) {
                self.automatic_ground[index] =
                    Some((c.sequence.reaction.max, c.sequence.reaction.height));
            }
        }
        self.rules
            .try_reserve(1)
            .map_err(|_| "Not enough memory for assist rules")?;
        if c.sequence.reaction.trigger.defensive() {
            self.defense_mask |= 1 << c.sequence.reaction.trigger as u8;
        } else {
            self.controls |= c.sequence.reaction.control.bit();
        }
        self.rules.push(Some(c));
        self.len += 1;
        self.rules[..self.len].sort_by_key(|x| x.unwrap().sequence.reaction.priority);
        Ok(())
    }
}
/// Available `Feedback` choices for the Smash assistance menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Feedback {
    #[default]
    /// `Watching` option in the `Feedback` Smash policy state.
    Watching,
    /// `State` option in the `Feedback` Smash policy state.
    State,
    /// `Range` option in the `Feedback` Smash policy state.
    Range,
    /// `Height` option in the `Feedback` Smash policy state.
    Height,
    /// `Side` option in the `Feedback` Smash policy state.
    Side,
    /// `Crossing` option in the `Feedback` Smash policy state.
    Crossing,
    /// `Invulnerable` option in the `Feedback` Smash policy state.
    Invulnerable,
    /// `Matched` option in the `Feedback` Smash policy state.
    Matched(
        /// Carries the index of the matched rule in `Feedback`.
        u8,
    ),
    /// `Cancelled` option in the `Feedback` Smash policy state.
    Cancelled,
    /// `MissingTarget` option in the `Feedback` Smash policy state.
    MissingTarget,
    /// `Finished` option in the `Feedback` Smash policy state.
    Finished,
}
impl Feedback {
    /// Returns the display label for the value.
    pub fn label(self) -> &'static str {
        match self {
            Self::Watching => "watching rolls",
            Self::State => "self state",
            Self::Range => "outside range",
            Self::Height => "height",
            Self::Side => "enemy side",
            Self::Crossing => "waiting for crossing",
            Self::Invulnerable => "waiting for vulnerability",
            Self::Matched(_) => "matched",
            Self::Cancelled => "preempted",
            Self::MissingTarget => "target lost",
            Self::Finished => "finished",
        }
    }
}
/// Runtime Smash data carried by `Track`.
#[derive(Clone, Copy)]
struct Track {
    /// battle-object ID.
    id: u32,
    /// native fighter status.
    status: i32,
    /// native animation/motion ID.
    motion: u64,
    /// current native animation frame.
    frame: f32,
    /// Runtime dx value used by Track..
    dx: f32,
    /// Runtime crossed value used by Track..
    crossed: bool,
    /// Runtime used value used by Track..
    used: bool,
}
impl Track {
    /// Determines whether the `continues` condition holds.
    fn continues(&self, target: &Target) -> bool {
        crate::timing::same_motion_event(
            (self.id, self.status, self.motion, self.frame),
            (target.id, target.status, target.motion, target.frame),
        )
    }
}
impl Tracked for Track {
    /// Returns the native identifier of the observed object.
    fn id(&self) -> u32 {
        self.id
    }
    /// Returns the `status` result.
    fn status(&self) -> i32 {
        self.status
    }
}
/// Runtime Smash data carried by `Engine`.
#[derive(Default)]
pub struct Engine {
    /// Runtime tracks value used by Engine..
    tracks: [Option<Track>; crate::battle_objects::FIGHTER_CAPACITY],
    /// Runtime plan value used by Engine..
    plan: Option<InputTimeline>,
    /// selected opponent/projectile target.
    target: Option<u32>,
    /// Runtime feedback value used by Engine..
    pub feedback: Feedback,
    /// Runtime last priority value used by Engine..
    pub last_priority: Option<u8>,
    /// Runtime active control value used by Engine..
    active_control: Option<Control>,
    /// Runtime up b value used by Engine..
    up_b: crate::move_input::UpBFlow,
}
impl Engine {
    /// Observes the statuses.
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        forget_ended(&mut self.tracks, known);
    }
    /// Returns the selected target observation.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Checks whether an input owner has committed the action.
    pub fn committed(&self) -> bool {
        self.plan.is_some()
    }
    /// Determines whether the `direct_up_b` condition holds.
    pub fn direct_up_b(&self) -> bool {
        self.up_b.armed() && self.plan.is_some()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    pub fn up_b_delivery_started(&self) -> bool {
        self.direct_up_b() && self.up_b.started()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.direct_up_b() && self.target == Some(target) {
            self.up_b.attempted(tick);
        }
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.direct_up_b() && self.target == Some(target) {
            self.up_b.mapped(tick);
        }
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.direct_up_b() && self.up_b.receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    pub fn awaiting_up_b(&self) -> bool {
        self.direct_up_b() && self.up_b.awaiting()
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.direct_up_b() {
            self.up_b.tick(tick);
        }
    }
    /// Determines whether ground.
    pub fn needs_ground(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::needs_ground)
    }
    /// Returns an optional `right_stick` value.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.plan.as_ref().map(InputTimeline::right_stick)
    }
    /// Determines whether the `walking` condition holds.
    pub fn walking(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::walking)
    }
    /// Determines whether the `attack_hold` condition holds.
    pub fn attack_hold(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::attack_hold)
    }
    /// Cancels the pending operation and clears its state.
    pub fn cancel(&mut self) {
        self.plan = None;
        self.target = None;
        self.active_control = None;
        self.up_b.disarm();
        self.feedback = Feedback::Cancelled;
    }
    /// Rebuild the per-slot event memory from this poll. A consumed event whose
    /// actor is missing stays consumed until a different status is observed.
    /// `origin_x` additionally follows each target's side and roll crossing.
    fn observe_events(
        &mut self,
        targets: &[Option<Target>; 8],
        origin_x: Option<f32>,
        tracked: impl Fn(&Target) -> bool,
    ) {
        let old = self.tracks;
        self.tracks = std::array::from_fn(|i| {
            let t = targets.get(i).copied().flatten().filter(|t| {
                tracked(t)
                    && t.frame.is_finite()
                    && t.frame >= 0.
                    && t.rate.is_finite()
                    && t.rate >= 0.
                    && t.position.iter().all(|v| v.is_finite())
            })?;
            let prev = old.iter().flatten().find(|p| p.continues(&t));
            let dx = origin_x.map_or(0., |x| t.position[0] - x);
            Some(Track {
                id: t.id,
                status: t.status,
                motion: t.motion,
                frame: t.frame,
                dx: if dx.abs() > 0.001 {
                    dx
                } else {
                    prev.map_or(dx, |p| p.dx)
                },
                crossed: origin_x.is_some() && prev.is_some_and(|p| p.crossed || p.dx * dx < 0.),
                used: prev.is_some_and(|p| p.used),
            })
        });
        retain_missing(
            &mut self.tracks,
            old.into_iter().flatten(),
            targets.iter().flatten().map(|t| (t.id, t.status)),
            |p| p.used,
        );
    }
    /// Updates the toggle state from the current button sample and match state.
    pub fn sample(
        &mut self,
        allowed: bool,
        rules: &Resolved,
        settings: AssistSettings,
        mut c: GameplayContext,
        targets: [Option<Target>; 8],
        mut follow: Option<Opponent>,
    ) -> ControllerOutput {
        self.observe_events(&targets, Some(c.position[0]), |t| {
            matches!(
                t.status,
                ESCAPE_F
                    | ESCAPE_B
                    | crate::native_states::status::PASSIVE_FB
                    | crate::native_states::status::DOWN_STAND_FB
            ) && t.rate > 0.
        });
        if !allowed
            || self
                .active_control
                .is_some_and(|control| rules.enabled_controls & control.bit() == 0)
            || !c.gameplay
            || !c.position.iter().all(|v| v.is_finite())
            || !c.facing.is_finite()
            || c.facing.abs() < 0.5
        {
            self.cancel();
            return ControllerOutput::default();
        }
        if self.plan.is_none() && !c.stopped {
            self.start_matching_roll(rules, c, &targets, &mut follow);
        }
        c.held = true;
        let Some(plan) = self.plan.as_mut() else {
            return ControllerOutput::default();
        };
        if !plan.inputs_only() && follow.filter(|e| Some(e.id) == self.target).is_none() {
            self.cancel();
            self.feedback = Feedback::MissingTarget;
            return ControllerOutput::default();
        }
        let out = plan.sample(settings, c, follow, None, 0.);
        if plan.finished() {
            self.plan = None;
            self.target = None;
            self.active_control = None;
            self.feedback = Feedback::Finished
        }
        out
    }
    /// Commit the first priority-ordered rule that matches an unconsumed roll.
    fn start_matching_roll(
        &mut self,
        rules: &Resolved,
        c: GameplayContext,
        targets: &[Option<Target>; 8],
        follow: &mut Option<Opponent>,
    ) {
        self.feedback = Feedback::Watching;
        for candidate in rules.rules.iter().flatten() {
            let r = candidate.sequence.reaction;
            if !matches!(
                r.trigger,
                Trigger::Roll | Trigger::Crossed | Trigger::Vulnerable | Trigger::CrossVulnerable
            ) || rules.enabled_controls & r.control.bit() == 0
            {
                continue;
            }
            for (i, t) in targets.iter().enumerate() {
                let (Some(t), Some(track)) = (t, self.tracks[i]) else {
                    continue;
                };
                if track.used {
                    continue;
                }
                if let Some(rejected) = r.rejects(c, t.position) {
                    self.feedback = rejected;
                    continue;
                }
                if matches!(r.trigger, Trigger::Crossed | Trigger::CrossVulnerable)
                    && !track.crossed
                {
                    self.feedback = Feedback::Crossing;
                    continue;
                }
                if matches!(r.trigger, Trigger::Vulnerable | Trigger::CrossVulnerable)
                    && !t.vulnerable_now()
                {
                    self.feedback = Feedback::Invulnerable;
                    continue;
                }
                let enemy = t.opponent(crate::native_states::situation::GROUND, t.actionable_in);
                let mut seq = candidate.sequence;
                seq.while_held = false;
                *follow = Some(follow.filter(|e| e.id == enemy.id).unwrap_or(enemy));
                self.plan = Some(InputTimeline::new(seq, enemy));
                self.target = Some(t.id);
                self.tracks[i].as_mut().unwrap().used = true;
                self.feedback = Feedback::Matched(r.priority);
                self.last_priority = Some(r.priority);
                self.active_control = Some(r.control);
                return;
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/policy_reaction_rules.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/policy_reaction_combined_trigger.rs"]
mod combined_trigger_test;

#[cfg(test)]
#[path = "../../tests/unit/policy_reaction_activation.rs"]
mod activation_tests;
