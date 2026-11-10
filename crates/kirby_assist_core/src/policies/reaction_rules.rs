//! Bounded, input-only roll reaction rules. No native queries or file access.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::InputTimeline,
    roll_read::{Target, ESCAPE_B, ESCAPE_F},
    sequence_config::{Action, Sequence},
    settings::AssistSettings,
};
/// Determines whether the `starts_direct_up_b` condition holds.
///
///
///  Checks whether starts direct up b.
pub fn starts_direct_up_b(sequence: &Sequence) -> bool {
    simple::starts_up_b(sequence)
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
///
///  An automatic reaction tape: delivered holds survive observation gaps and a pending Grab/Up-B is rejected after manual takeoff.
pub(crate) fn automatic_plan(sequence: Sequence, enemy: Opponent) -> InputTimeline {
    let mut plan = InputTimeline::new(sequence, enemy);
    plan.allow_continuous_progress();
    plan.guard_automatic_ground_actions();
    plan
}
/// Per-actor memory of a native event that may already have been answered.
///
///  Per-actor memory of a native event that may already have been answered.
pub(crate) trait Tracked: Copy {
    /// Returns the native identifier of the observed object.
    ///
    ///
///  Gets the object’s identifier.
    fn id(&self) -> u32;
    /// Returns an optional `status` value.
    ///
    ///
///  Gets status if it is available.
    fn status(&self) -> i32;
}
/// Positive status evidence that an actor left its remembered event.
///
///  Positive status evidence that an actor left its remembered event.
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
///
///  Missing samples are not evidence of a new event: keep selected memories of actors absent from this poll in spare slots until a different status is seen.
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
///
///
///  Holds the MAX RULES PER MODE value.
pub const MAX_RULES_PER_MODE: usize = 8;
/// Sets the upper bound for rules.
///
///
///  Holds the MAX RULES value.
pub const MAX_RULES: usize = MAX_RULES_PER_MODE * 7;
/// Defines the available `Trigger` cases.
///
///
///  Lists the available Trigger choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trigger {
    #[default]
    /// Represents the `Off` case of `Trigger`.
    ///
    ///
///  Represents the Off option in Trigger.
    Off,
    /// Represents the `Roll` case of `Trigger`.
    ///
    ///
///  Represents the Roll option in Trigger.
    Roll,
    /// Represents the `Crossed` case of `Trigger`.
    ///
    ///
///  Represents the Crossed option in Trigger.
    Crossed,
    /// Represents the `Vulnerable` case of `Trigger`.
    ///
    ///
///  Represents the Vulnerable option in Trigger.
    Vulnerable,
    /// Represents the `CrossVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the CrossVulnerable option in Trigger.
    CrossVulnerable,
    /// Represents the `ShieldHit` case of `Trigger`.
    ///
    ///
///  Represents the ShieldHit option in Trigger.
    ShieldHit,
    /// Represents the `GrabDodged` case of `Trigger`.
    ///
    ///
///  Represents the GrabDodged option in Trigger.
    GrabDodged,
    /// Represents the `LedgeRegrab` case of `Trigger`.
    ///
    ///
///  Represents the LedgeRegrab option in Trigger.
    LedgeRegrab,
    /// Represents the `SpotDodge` case of `Trigger`.
    ///
    ///
///  Represents the SpotDodge option in Trigger.
    SpotDodge,
    /// Represents the `EnemyShield` case of `Trigger`.
    ///
    ///
///  Represents the EnemyShield option in Trigger.
    EnemyShield,
    /// Represents the `ShieldRoll` case of `Trigger`.
    ///
    ///
///  Represents the ShieldRoll option in Trigger.
    ShieldRoll,
    /// Represents the `ShieldCross` case of `Trigger`.
    ///
    ///
///  Represents the ShieldCross option in Trigger.
    ShieldCross,
    /// Represents the `ShieldVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the ShieldVulnerable option in Trigger.
    ShieldVulnerable,
    /// Represents the `ShieldCrossVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the ShieldCrossVulnerable option in Trigger.
    ShieldCrossVulnerable,
    /// Represents the `ShieldAway` case of `Trigger`.
    ///
    ///
///  Represents the ShieldAway option in Trigger.
    ShieldAway,
    /// Represents the `ShieldAwayVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the ShieldAwayVulnerable option in Trigger.
    ShieldAwayVulnerable,
    /// Represents the `ShieldPressureVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the ShieldPressureVulnerable option in Trigger.
    ShieldPressureVulnerable,
    /// Represents the `ShieldHitVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the ShieldHitVulnerable option in Trigger.
    ShieldHitVulnerable,
    /// Represents the `GrabDodgedVulnerable` case of `Trigger`.
    ///
    ///
///  Represents the GrabDodgedVulnerable option in Trigger.
    GrabDodgedVulnerable,
    /// Represents the `SpotDodgeAny` case of `Trigger`.
    ///
    ///
///  Represents the SpotDodgeAny option in Trigger.
    SpotDodgeAny,
    /// Represents the `AutoRoll` case of `Trigger`.
    ///
    ///
///  Represents the AutoRoll option in Trigger.
    AutoRoll,
    /// Represents the `AutoSpot` case of `Trigger`.
    ///
    ///
///  Represents the AutoSpot option in Trigger.
    AutoSpot,
    /// Represents the `AutoAir` case of `Trigger`.
    ///
    ///
///  Represents the AutoAir option in Trigger.
    AutoAir,
    /// Represents the `AutoGrab` case of `Trigger`.
    ///
    ///
///  Represents the AutoGrab option in Trigger.
    AutoGrab,
    /// Represents the `AutoShieldGrab` case of `Trigger`.
    ///
    ///
///  Represents the AutoShieldGrab option in Trigger.
    AutoShieldGrab,
    /// Represents the `AutoGrabDodged` case of `Trigger`.
    ///
    ///
///  Represents the AutoGrabDodged option in Trigger.
    AutoGrabDodged,
    /// Represents the `AutoShieldBackup` case of `Trigger`.
    ///
    ///
///  Represents the AutoShieldBackup option in Trigger.
    AutoShieldBackup,
}
impl Trigger {
    /// Defines the `AUTO_EVENTS` constant used by this module.
    ///
    ///
///  Provides the AUTO EVENTS constant value.
    pub const AUTO_EVENTS: [Self; 7] = [
        Self::AutoRoll,
        Self::AutoSpot,
        Self::AutoAir,
        Self::AutoGrab,
        Self::AutoShieldGrab,
        Self::AutoGrabDodged,
        Self::AutoShieldBackup,
    ];
    /// Returns the `auto_name` result.
    ///
    ///
///  Gets the auto name result.
    pub fn auto_name(self) -> &'static str {
        match self {
            Self::AutoRoll => "Roll",
            Self::AutoSpot => "Enemy spot dodge",
            Self::AutoAir => "Enemy air dodge",
            Self::AutoGrab => "Automatic grab (unshielded)",
            Self::AutoShieldGrab => "Automatic grab (shielded)",
            Self::AutoGrabDodged => "After dodging enemy grab",
            Self::AutoShieldBackup => "Shield grab backup",
            _ => "Reaction",
        }
    }

    /// Defines the `ALL` constant used by this module.
    ///
    ///
///  Provides the ALL constant value.
    pub const ALL: [Self; 27] = [
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
        Self::AutoGrab,
        Self::AutoShieldGrab,
        Self::AutoGrabDodged,
        Self::AutoShieldBackup,
    ];
    /// Returns the `family` result.
    ///
    ///
///  Gets the family result.
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
    ///
    ///
///  Gets simple if it is available.
    pub fn simple(self) -> Option<usize> {
        match self {
            Self::AutoRoll => Some(0),
            Self::AutoSpot => Some(1),
            Self::AutoAir => Some(2),
            Self::AutoGrab => Some(3),
            Self::AutoShieldGrab => Some(4),
            Self::AutoGrabDodged => Some(5),
            Self::AutoShieldBackup => Some(6),
            _ => None,
        }
    }
    /// Determines whether the `shield_pressure` condition holds.
    ///
    ///
///  Checks whether shield pressure.
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
    /// Defines the `SHIELD_EVENTS` constant used by this module.
    ///
    ///
///  Provides the SHIELD EVENTS constant value.
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
    ///
    ///
///  Checks whether defensive.
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
    ///
    ///
///  Checks whether event matches.
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
    ///
    ///
///  Checks whether vulnerable.
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
    ///
    ///
///  Gets the events result.
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
    ///
    ///
///  Gets the text form used when saving this value.
    pub fn token(self) -> &'static str {
        match self {
            Self::AutoGrab => "auto_grab",
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
/// Defines the available `SelfState` cases.
///
///
///  Lists the available SelfState choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelfState {
    #[default]
    /// Represents the `Any` case of `SelfState`.
    ///
    ///
///  Represents the Any option in SelfState.
    Any,
    /// Represents the `Grounded` case of `SelfState`.
    ///
    ///
///  Represents the Grounded option in SelfState.
    Grounded,
    /// Represents the `Airborne` case of `SelfState`.
    ///
    ///
///  Represents the Airborne option in SelfState.
    Airborne,
    /// Represents the `Standing` case of `SelfState`.
    ///
    ///
///  Represents the Standing option in SelfState.
    Standing,
    /// Represents the `Walking` case of `SelfState`.
    ///
    ///
///  Represents the Walking option in SelfState.
    Walking,
    /// Represents the `Running` case of `SelfState`.
    ///
    ///
///  Represents the Running option in SelfState.
    Running,
    /// Represents the `Shielding` case of `SelfState`.
    ///
    ///
///  Represents the Shielding option in SelfState.
    Shielding,
}
impl SelfState {
    /// Defines the `ALL` constant used by this module.
    ///
    ///
///  Provides the ALL constant value.
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
    ///
    ///
///  Gets the text form used when saving this value.
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
    ///
    ///
///  Checks whether matches.
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
/// Defines the available `Side` cases.
///
///
///  Lists the available Side choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    /// Represents the `Any` case of `Side`.
    ///
    ///
///  Represents the Any option in Side.
    Any,
    /// Represents the `Front` case of `Side`.
    ///
    ///
///  Represents the Front option in Side.
    Front,
    /// Represents the `Behind` case of `Side`.
    ///
    ///
///  Represents the Behind option in Side.
    Behind,
}
impl Side {
    /// Defines the `ALL` constant used by this module.
    ///
    ///
///  Provides the ALL constant value.
    pub const ALL: [Self; 3] = [Self::Any, Self::Front, Self::Behind];
    /// Returns the serialized token for the value.
    ///
    ///
///  Gets the text form used when saving this value.
    pub fn token(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Front => "front",
            Self::Behind => "behind",
        }
    }
}
/// Activation group, independent of trigger conditions and response actions.
///
///  Activation group, independent of trigger conditions and response actions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Control {
    #[default]
    /// Represents the `Menu` case of `Control`.
    ///
    ///
///  Represents the Menu option in Control.
    Menu,
    /// Represents the `ToggleX` case of `Control`.
    ///
    ///
///  Represents the ToggleX option in Control.
    ToggleX,
    /// Represents the `ToggleY` case of `Control`.
    ///
    ///
///  Represents the ToggleY option in Control.
    ToggleY,
    /// Represents the `ToggleUp` case of `Control`.
    ///
    ///
///  Represents the ToggleUp option in Control.
    ToggleUp,
    /// Represents the `ToggleDown` case of `Control`.
    ///
    ///
///  Represents the ToggleDown option in Control.
    ToggleDown,
    /// Represents the `ToggleLeft` case of `Control`.
    ///
    ///
///  Represents the ToggleLeft option in Control.
    ToggleLeft,
    /// Represents the `ToggleRight` case of `Control`.
    ///
    ///
///  Represents the ToggleRight option in Control.
    ToggleRight,
}
impl Control {
    /// Defines the `ALL` constant used by this module.
    ///
    ///
///  Provides the ALL constant value.
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
    ///
    ///
///  Checks whether this bit is on.
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
    /// Returns the serialized token for the value.
    ///
    ///
///  Gets the text form used when saving this value.
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
    ///
    ///
///  Gets the text shown for this value.
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
    ///
    ///
///  Gets the assigned button.
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
/// Defines the `Rule` data structure.
///
///
///  Holds the data for Rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    /// Stores the `trigger` value in `Rule` as `Trigger`.
    ///
    ///
///  Holds the trigger value used by Rule.
    pub trigger: Trigger,
    /// Stores the `state` value in `Rule` as `SelfState`.
    ///
    ///
///  Holds the state value used by Rule.
    pub state: SelfState,
    /// Stores the `min` value in `Rule` as `f32`.
    ///
    ///
///  Holds the min value used by Rule.
    pub min: f32,
    /// Stores the `max` value in `Rule` as `f32`.
    ///
    ///
///  Holds the max value used by Rule.
    pub max: f32,
    /// Stores the `height` value in `Rule` as `f32`.
    ///
    ///
///  Holds the height value used by Rule.
    pub height: f32,
    /// Stores the `side` value in `Rule` as `Side`.
    ///
    ///
///  Holds the side value used by Rule.
    pub side: Side,
    /// Stores the `priority` value in `Rule` as `u8`.
    ///
    ///
///  Holds the priority value used by Rule.
    pub priority: u8,
    /// Stores the `control` value in `Rule` as `Control`.
    ///
    ///
///  Holds the control value used by Rule.
    pub control: Control,
}
impl Default for Rule {
    /// Returns the default value for this type.
    ///
    ///
///  Returns the default value for this type.
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
    ///
    ///
///  Checks whether the feature is on.
    pub fn active(self) -> bool {
        self.trigger != Trigger::Off
    }
    /// The first failing self-state/range/height/side gate for a target at
    /// `target`, in the order shown as feedback. Non-finite geometry never fits.
    ///
///  The first failing self-state/range/height/side gate for a target at target, in the order shown as feedback. Non-finite geometry never fits.
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
    ///
    ///
///  Gets the field result.
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
    ///
    ///
///  Updates adjust.
    pub fn adjust(&mut self, n: u8, up: bool) {
        /// Advances the selected choice to the next value.
        ///
        ///
///  Moves to the next choice.
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
    ///
    ///
///  Gets the validate result.
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
    ///
    ///
///  Reads the source and checks that it is valid.
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
    ///
    ///
///  Gets the text form used when saving this value.
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
///
///
///  Validates the response.
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
/// Defines the `Candidate` data structure.
///
///
///  Holds the data for Candidate.
#[derive(Clone, Copy)]
pub struct Candidate {
    /// Stores the `slot` value in `Candidate` as `usize`.
    ///
    ///
///  Holds the slot value used by Candidate.
    pub slot: usize,
    /// Stores the `index` value in `Candidate` as `usize`.
    ///
    ///
///  Holds the index value used by Candidate.
    pub index: usize,
    /// Stores the `sequence` value in `Candidate` as `Sequence`.
    ///
    ///
///  Holds the sequence value used by Candidate.
    pub sequence: Sequence,
}
/// Defines the `Resolved` data structure.
///
///
///  Holds the data for Resolved.
pub struct Resolved {
    /// Stores the `rules` value in `Resolved` as `Vec<Option<Candidate>>`.
    ///
    ///
///  Stores the configured scenario rules in fixed-size slots.
    pub rules: Vec<Option<Candidate>>,
    /// Stores the `len` value in `Resolved` as `usize`.
    ///
    ///
///  Holds the len value used by Resolved.
    pub len: usize,
    /// Stores the `controls` value in `Resolved` as `u8`.
    ///
    ///
///  Holds the controls value used by Resolved.
    pub controls: u8,
    /// Stores the `enabled_controls` value in `Resolved` as `u8`.
    ///
    ///
///  Holds the enabled controls value used by Resolved.
    pub enabled_controls: u8,
    /// Stores the `defense_mask` value in `Resolved` as `u32`.
    ///
    ///
///  Holds the defense mask value used by Resolved.
    defense_mask: u32,
    /// Stores the `simple` value in `Resolved` as `Option<Box<[Option<Sequence>; 7]>>`.
    ///
    ///
///  Optionally stores the simple for Resolved; it may be absent.
    simple: Option<Box<[Option<Sequence>; 7]>>,
    /// Stores the `simple_starts` value in `Resolved` as `[Option<SimpleStart>; 3]`.
    ///
    ///
///  Stores the simple starts collection for Resolved.
    simple_starts: [Option<SimpleStart>; 3],
    /// Stores the `automatic_ground` value in `Resolved` as `[Option<(f32`.
    ///
    ///
///  Stores the automatic ground collection for Resolved.
    automatic_ground: [Option<(f32, f32)>; 7],
    /// Stores the `automatic_up_special` value in `Resolved` as `u8`.
    ///
    ///
///  Holds the automatic up special value used by Resolved.
    automatic_up_special: u8,
}
impl Default for Resolved {
    /// Returns the default value for this type.
    ///
    ///
///  Returns the default value for this type.
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
    /// Returns an optional `simple_sequence` value.
    ///
    ///
///  Gets simple sequence if it is available.
    pub fn simple_sequence(&self, index: usize) -> Option<&Sequence> {
        self.simple
            .as_ref()
            .and_then(|s| s.get(index))
            .and_then(Option::as_ref)
    }
    /// Checks whether simple events.
    ///
    ///
///  Checks whether simple events.
    pub fn has_simple_events(&self) -> bool {
        (0..3).any(|i| self.simple_sequence(i).is_some())
    }
    /// Checks whether defense.
    ///
    ///
///  Checks whether defense.
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
    ///
    ///
///  Sets the enabled controls.
    pub fn set_enabled_controls(&mut self, controls: u8) {
        self.enabled_controls = controls;
    }
    /// Returns the `add` result.
    ///
    ///
///  Gets the add result.
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
        if let Some(index) = c.sequence.reaction.trigger.simple() {
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
/// Defines the available `Feedback` cases.
///
///
///  Lists the available Feedback choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Feedback {
    #[default]
    /// Represents the `Watching` case of `Feedback`.
    ///
    ///
///  Represents the Watching option in Feedback.
    Watching,
    /// Represents the `State` case of `Feedback`.
    ///
    ///
///  Represents the State option in Feedback.
    State,
    /// Represents the `Range` case of `Feedback`.
    ///
    ///
///  Represents the Range option in Feedback.
    Range,
    /// Represents the `Height` case of `Feedback`.
    ///
    ///
///  Represents the Height option in Feedback.
    Height,
    /// Represents the `Side` case of `Feedback`.
    ///
    ///
///  Represents the Side option in Feedback.
    Side,
    /// Represents the `Crossing` case of `Feedback`.
    ///
    ///
///  Represents the Crossing option in Feedback.
    Crossing,
    /// Represents the `Invulnerable` case of `Feedback`.
    ///
    ///
///  Represents the Invulnerable option in Feedback.
    Invulnerable,
    /// Represents the `Matched` case of `Feedback`.
    ///
    ///
///  Represents the Matched option in Feedback.
    Matched(
        /// Carries the index of the matched rule in `Feedback`.
        ///
        ///
///  Holds the index of the matched rule for this `Feedback` choice.
        u8,
    ),
    /// Represents the `Cancelled` case of `Feedback`.
    ///
    ///
///  Represents the Cancelled option in Feedback.
    Cancelled,
    /// Represents the `MissingTarget` case of `Feedback`.
    ///
    ///
///  Represents the MissingTarget option in Feedback.
    MissingTarget,
    /// Represents the `Finished` case of `Feedback`.
    ///
    ///
///  Represents the Finished option in Feedback.
    Finished,
}
impl Feedback {
    /// Returns the display label for the value.
    ///
    ///
///  Gets the text shown for this value.
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
/// Defines the `Track` data structure.
///
///
///  Holds the data for Track.
#[derive(Clone, Copy)]
struct Track {
    /// Stores the `id` value in `Track` as `u32`.
    ///
    ///
///  Holds the id value used by Track.
    id: u32,
    /// Stores the `status` value in `Track` as `i32`.
    ///
    ///
///  Holds the status value used by Track.
    status: i32,
    /// Stores the `motion` value in `Track` as `u64`.
    ///
    ///
///  Holds the motion value used by Track.
    motion: u64,
    /// Stores the `frame` value in `Track` as `f32`.
    ///
    ///
///  Holds the frame value used by Track.
    frame: f32,
    /// Stores the `dx` value in `Track` as `f32`.
    ///
    ///
///  Holds the dx value used by Track.
    dx: f32,
    /// Stores the `crossed` value in `Track` as `bool`.
    ///
    ///
///  Holds the crossed value used by Track.
    crossed: bool,
    /// Stores the `used` value in `Track` as `bool`.
    ///
    ///
///  Holds the used value used by Track.
    used: bool,
}
impl Track {
    /// Determines whether the `continues` condition holds.
    ///
    ///
///  Checks whether continues.
    fn continues(&self, target: &Target) -> bool {
        crate::timing::same_motion_event(
            (self.id, self.status, self.motion, self.frame),
            (target.id, target.status, target.motion, target.frame),
        )
    }
}
impl Tracked for Track {
    /// Returns the native identifier of the observed object.
    ///
    ///
///  Gets the object’s identifier.
    fn id(&self) -> u32 {
        self.id
    }
    /// Returns the `status` result.
    ///
    ///
///  Gets the status result.
    fn status(&self) -> i32 {
        self.status
    }
}
/// Defines the `Engine` data structure.
///
///
///  Holds the data for Engine.
#[derive(Default)]
pub struct Engine {
    /// Stores the `tracks` value in `Engine` as `[Option<Track>; crate::battle_objects::FIGHTER_CAPACITY]`.
    ///
    ///
///  Stores the tracks collection for Engine.
    tracks: [Option<Track>; crate::battle_objects::FIGHTER_CAPACITY],
    /// Stores the `plan` value in `Engine` as `Option<InputTimeline>`.
    ///
    ///
///  Optionally stores the plan for Engine; it may be absent.
    plan: Option<InputTimeline>,
    /// Stores the `target` value in `Engine` as `Option<u32>`.
    ///
    ///
///  Optionally stores the target for Engine; it may be absent.
    target: Option<u32>,
    /// Stores the `feedback` value in `Engine` as `Feedback`.
    ///
    ///
///  Holds the feedback value used by Engine.
    pub feedback: Feedback,
    /// Stores the `last_priority` value in `Engine` as `Option<u8>`.
    ///
    ///
///  Optionally stores the last priority for Engine; it may be absent.
    pub last_priority: Option<u8>,
    /// Stores the `active_control` value in `Engine` as `Option<Control>`.
    ///
    ///
///  Optionally stores the active control for Engine; it may be absent.
    active_control: Option<Control>,
    /// Stores the `up_b` value in `Engine` as `crate::move_input::UpBFlow`.
    ///
    ///
///  Holds the up b value used by Engine.
    up_b: crate::move_input::UpBFlow,
}
impl Engine {
    /// Observes the statuses.
    ///
    ///
///  Observes the statuses.
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        forget_ended(&mut self.tracks, known);
    }
    /// Returns the selected target observation.
    ///
    ///
///  Gets the chosen target.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Checks whether an input owner has committed the action.
    ///
    ///
///  Checks whether the action has been committed.
    pub fn committed(&self) -> bool {
        self.plan.is_some()
    }
    /// Determines whether the `direct_up_b` condition holds.
    ///
    ///
///  Checks whether direct up b.
    pub fn direct_up_b(&self) -> bool {
        self.up_b.armed() && self.plan.is_some()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    ///
    ///
///  Checks whether up b delivery started.
    pub fn up_b_delivery_started(&self) -> bool {
        self.direct_up_b() && self.up_b.started()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    ///
    ///
///  Updates attempted up special.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.direct_up_b() && self.target == Some(target) {
            self.up_b.attempted(tick);
        }
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    ///
    ///
///  Updates mapped up special.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.direct_up_b() && self.target == Some(target) {
            self.up_b.mapped(tick);
        }
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    ///
    ///
///  Checks whether up b receipt pending.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.direct_up_b() && self.up_b.receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    ///
    ///
///  Checks whether awaiting up b.
    pub fn awaiting_up_b(&self) -> bool {
        self.direct_up_b() && self.up_b.awaiting()
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    ///
    ///
///  Updates delivery tick.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.direct_up_b() {
            self.up_b.tick(tick);
        }
    }
    /// Determines whether ground.
    ///
    ///
///  Checks whether ground.
    pub fn needs_ground(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::needs_ground)
    }
    /// Returns an optional `right_stick` value.
    ///
    ///
///  Gets right stick if it is available.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.plan.as_ref().map(InputTimeline::right_stick)
    }
    /// Determines whether the `walking` condition holds.
    ///
    ///
///  Checks whether walking.
    pub fn walking(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::walking)
    }
    /// Determines whether the `attack_hold` condition holds.
    ///
    ///
///  Checks whether attack hold.
    pub fn attack_hold(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::attack_hold)
    }
    /// Cancels the pending operation and clears its state.
    ///
    ///
///  Stops the operation that was waiting.
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
    ///
///  Rebuild the per-slot event memory from this poll. A consumed event whose actor is missing stays consumed until a different status is observed. originx additionally follows each target's side and roll crossing.
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
    ///
    ///
///  Checks the current buttons and updates whether the toggle should activate.
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
            matches!(t.status, ESCAPE_F | ESCAPE_B) && t.rate > 0.
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
    ///
///  Commit the first priority-ordered rule that matches an unconsumed roll.
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
            if r.trigger.defensive() || rules.enabled_controls & r.control.bit() == 0 {
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
                follow.get_or_insert(enemy);
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

mod simple;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fighter_profiles::{presets::PresetSlot, FighterProfiles},
        native_input::*,
        sequence_config::{persist, Catalog},
    };
    /// Returns the `sequence` result.
    ///
    ///
///  Gets the sequence result.
    fn sequence(trigger: &str, actions: &str) -> Sequence {
        Catalog::parse(&format!(
            "[yoshi.roll.rule]\nstart_delay=0\nreaction={trigger}\nactions={actions}\n"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    /// Returns the `context` result.
    ///
    ///
///  Gets the context result.
    fn context(frame: f32) -> GameplayContext {
        GameplayContext {
            held: true,
            gameplay: true,
            status: crate::native_states::status::WAIT,
            situation: crate::native_states::situation::GROUND,
            motion_kind: 1,
            motion_frame: frame,
            motion_rate: 1.,
            facing: 1.,
            position: [0., 0.],
            safe_ground: [true; 2],
            ..GameplayContext::default()
        }
    }
    /// Returns the selected target observation.
    ///
    ///
///  Gets the chosen target.
    fn target(frame: f32, x: f32) -> Target {
        Target {
            id: 7,
            status: ESCAPE_F,
            motion: 456,
            frame,
            rate: 1.,
            position: [x, 0.],
            velocity_x: 0.,
            vulnerable_in: Some(0.),
            actionable_in: Some(5.),
        }
    }
    /// Regression test for `event_identity_requires_matching_motion_status_and_nonrewound_frame`.
    ///
    ///
///  Checks that event identity requires matching motion status and nonrewound frame.
    #[test]
    fn event_identity_requires_matching_motion_status_and_nonrewound_frame() {
        let target = target(5., 1.);
        let track = Track {
            id: target.id,
            status: target.status,
            motion: target.motion,
            frame: target.frame,
            dx: 1.,
            crossed: false,
            used: true,
        };
        assert!(track.continues(&target));
        assert!(track.continues(&Target {
            frame: 6.,
            ..target
        }));
        assert!(!track.continues(&Target {
            frame: 4.,
            ..target
        }));
        assert!(!track.continues(&Target {
            frame: f32::NAN,
            ..target
        }));
        assert!(!track.continues(&Target { id: 8, ..target }));
        assert!(!track.continues(&Target {
            status: ESCAPE_B,
            ..target
        }));
        assert!(!track.continues(&Target {
            motion: 457,
            ..target
        }));
    }

    /// Regression test for `simple_tracking_retains_consumed_missing_actor_until_status_changes`.
    ///
    ///
///  Checks that simple tracking retains consumed missing actor until status changes.
    #[test]
    fn simple_tracking_retains_consumed_missing_actor_until_status_changes() {
        let mut engine = Engine::default();
        let first = target(5., 1.);
        let mut targets = [None; 8];
        targets[0] = Some(first);
        engine.observe_simple_events(&targets);
        engine.tracks[0].as_mut().unwrap().used = true;

        targets[0] = Some(Target { id: 8, ..first });
        engine.observe_simple_events(&targets);
        assert!(engine
            .tracks
            .iter()
            .flatten()
            .any(|t| t.id == first.id && t.used));

        targets[1] = Some(Target { frame: 6., ..first });
        engine.observe_simple_events(&targets);
        assert!(engine.tracks[1].unwrap().used);

        targets[1] = Some(Target { frame: 1., ..first });
        engine.observe_simple_events(&targets);
        assert!(!engine.tracks[1].unwrap().used);

        engine.tracks[1].as_mut().unwrap().used = true;
        targets[1] = Some(Target { status: 0, ..first });
        engine.observe_simple_events(&targets);
        assert!(!engine.tracks.iter().flatten().any(|t| t.id == first.id));
    }
    /// Returns the `resolved` result.
    ///
    ///
///  Gets the resolved result.
    fn resolved(s: Sequence) -> Resolved {
        let mut r = Resolved::default();
        r.add(Candidate {
            slot: 0,
            index: 0,
            sequence: s,
        })
        .unwrap();
        r
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
///  Checks the current buttons and updates whether the toggle should activate.
    fn sample(e: &mut Engine, r: &Resolved, c: GameplayContext, t: Target) -> ControllerOutput {
        e.sample(
            true,
            r,
            AssistSettings::default(),
            c,
            [Some(t), None, None, None, None, None, None, None],
            None,
        )
    }
    /// Regression test for `roll_sequence_in_static_shield_uses_the_real_opponent_clock_from_its_first_sample`.
    ///
    ///
///  Checks that roll sequence in static shield uses the real opponent clock from its first sample.
    #[test]
    fn roll_sequence_in_static_shield_uses_the_real_opponent_clock_from_its_first_sample() {
        for repeats in [0, 20] {
            let rules = resolved(sequence("roll:shielding:0:20:12:any:1", "wait:2,up_b:1"));
            let mut engine = Engine::default();
            let c = GameplayContext {
                status: crate::native_states::GUARD,
                motion_frame: 0.0,
                motion_rate: 1.0,
                ..context(0.0)
            };
            // Direct Up-B retains guard through its native shield cancel.
            for (frame, expected) in [0, 0, MAPPER_B | MAPPER_ZR, 0].into_iter().enumerate() {
                let t = target(frame as f32, 5.0);
                let following = Opponent {
                    id: t.id,
                    status: t.status,
                    position: t.position,
                    motion: Some(crate::gameplay::MotionTiming {
                        kind: t.motion,
                        frame: t.frame,
                        rate: t.rate,
                        remaining: t.actionable_in,
                    }),
                    ..Opponent::default()
                };
                let targets = [Some(t), None, None, None, None, None, None, None];
                // Discovery does not yet have a locked follow ID. The target
                // snapshot itself must seed this first frame's native clock.
                let follow = (frame > 0).then_some(following);
                let out =
                    engine.sample(true, &rules, AssistSettings::default(), c, targets, follow);
                assert_eq!(out.requested, expected, "native frame {frame}");
                for _ in 0..repeats {
                    let duplicate = engine.sample(
                        true,
                        &rules,
                        AssistSettings::default(),
                        c,
                        targets,
                        Some(following),
                    );
                    assert_eq!(duplicate.requested, out.requested);
                }
            }
        }
    }
    /// Regression test for `different_native_roll_motion_does_not_inherit_the_previous_rolls_used_flag`.
    ///
    ///
///  Checks that different native roll motion does not inherit the previous rolls used flag.
    #[test]
    fn different_native_roll_motion_does_not_inherit_the_previous_rolls_used_flag() {
        let rules = resolved(sequence("roll:any:0:20:12:any:1", "up_b:1"));
        let mut engine = Engine::default();
        assert_eq!(
            sample(&mut engine, &rules, context(0.0), target(0.0, 5.0)).requested,
            MAPPER_B
        );
        sample(&mut engine, &rules, context(1.0), target(1.0, 5.0));
        let t = Target {
            motion: 789,
            ..target(2.0, 5.0)
        };
        assert_eq!(
            sample(&mut engine, &rules, context(2.0), t).requested,
            MAPPER_B
        );
    }
    /// Regression test for `reaction_persists_roundtrip_and_disabling_replaces_old_metadata`.
    ///
    ///
///  Checks that reaction persists roundtrip and disabling replaces old metadata.
    #[test]
    fn reaction_persists_roundtrip_and_disabling_replaces_old_metadata() {
        let src="[yoshi.roll.rule]\nstart_delay=0\nactions=jab:4\n[kirby.roll.keep]\nstart_delay=0\nactions=grab\n";
        let seq = sequence("roll:grounded:2:20:12:front:2", "face:opponent,grab");
        let saved = persist::update(src, 5, PresetSlot::Roll, "rule", seq, false).unwrap();
        assert_eq!(Catalog::parse(&saved).unwrap().entries()[0].sequence, seq);
        let mut disabled = seq;
        disabled.reaction = Rule::default();
        let off = persist::update(&saved, 5, PresetSlot::Roll, "rule", disabled, false).unwrap();
        assert!(!off.contains("reaction ="));
        assert!(off.ends_with("[kirby.roll.keep]\nstart_delay=0\nactions=grab\n"));
    }
    /// Regression test for `invalid_rules_and_more_than_eight_are_rejected`.
    ///
    ///
///  Checks that invalid rules and more than eight are rejected.
    #[test]
    fn invalid_rules_and_more_than_eight_are_rejected() {
        for s in [
            "roll:any:20:10:12:any:1",
            "roll:any:0:NaN:12:any:1",
            "roll:any:0:20:12:any:0",
            "roll:wat:0:20:12:any:1",
        ] {
            assert!(Rule::parse(s).is_err());
        }
        let mut src = String::new();
        for i in 0..9 {
            src.push_str(&format!("[yoshi.roll.r{i}]\nstart_delay=0\nreaction=roll:any:0:20:12:any:1\nactions=jab:4\n"));
        }
        assert!(Catalog::parse(&src)
            .unwrap()
            .validate(&FighterProfiles::default())
            .is_err());
        assert!(validate_response(sequence("roll:any:0:20:12:any:1", "walk:ledge:8:50%")).is_err());
    }
    /// Regression test for `one_attempt_per_roll_and_new_roll_rearms`.
    ///
    ///
///  Checks that one attempt per roll and new roll rearms.
    #[test]
    fn one_attempt_per_roll_and_new_roll_rearms() {
        let r = resolved(sequence("roll:any:0:20:12:any:1", "jab:1"));
        let mut e = Engine::default();
        assert_eq!(
            sample(&mut e, &r, context(1.), target(1., 5.)).requested,
            MAPPER_A
        );
        sample(&mut e, &r, context(2.), target(2., 5.));
        assert_eq!(sample(&mut e, &r, context(3.), target(3., 5.)).requested, 0);
        assert_eq!(
            sample(&mut e, &r, context(4.), target(0., 5.)).requested,
            MAPPER_A
        );
    }
    /// Regression test for `crossing_survives_exact_zero_and_priority_chooses_the_first_match`.
    ///
    ///
///  Checks that crossing survives exact zero and priority chooses the first match.
    #[test]
    fn crossing_survives_exact_zero_and_priority_chooses_the_first_match() {
        let r = resolved(sequence("roll_cross:any:0:20:12:any:1", "down_smash:4"));
        let mut e = Engine::default();
        assert_eq!(sample(&mut e, &r, context(1.), target(1., 4.)).requested, 0);
        sample(&mut e, &r, context(2.), target(2., 0.));
        assert_eq!(
            sample(&mut e, &r, context(3.), target(3., -3.)).requested,
            MAPPER_A
        );
        let mut r = resolved(sequence("roll:any:0:20:12:any:5", "jab:4"));
        r.add(Candidate {
            slot: 0,
            index: 1,
            sequence: sequence("roll:any:0:20:12:any:1", "up_b:4"),
        })
        .unwrap();
        assert_eq!(
            sample(&mut Engine::default(), &r, context(1.), target(1., 5.)).requested,
            MAPPER_B
        );
    }
    /// Regression test for `filters_wait_for_native_vulnerability_and_cancel_never_replays`.
    ///
    ///
///  Checks that filters wait for native vulnerability and cancel never replays.
    #[test]
    fn filters_wait_for_native_vulnerability_and_cancel_never_replays() {
        let r = resolved(sequence("roll_vulnerable:walking:3:10:5:front:1", "jab:4"));
        let mut e = Engine::default();
        let mut c = context(1.);
        assert_eq!(sample(&mut e, &r, c, target(1., 5.)).requested, 0);
        assert_eq!(e.feedback, Feedback::State);
        c.status = crate::native_states::status::WALK;
        c.motion_frame = 2.;
        let mut t = target(2., 5.);
        t.vulnerable_in = Some(2.);
        assert_eq!(sample(&mut e, &r, c, t).requested, 0);
        assert_eq!(e.feedback, Feedback::Invulnerable);
        c.motion_frame = 3.;
        assert_eq!(sample(&mut e, &r, c, target(3., 5.)).requested, MAPPER_A);
        e.cancel();
        c.motion_frame = 4.;
        assert_eq!(sample(&mut e, &r, c, target(4., 5.)).requested, 0);
    }
    /// Regression test for `spatial_response_requires_fresh_target_and_no_new_plan_starts_in_stop`.
    ///
    ///
///  Checks that spatial response requires fresh target and no new plan starts in stop.
    #[test]
    fn spatial_response_requires_fresh_target_and_no_new_plan_starts_in_stop() {
        let r = resolved(sequence(
            "roll:any:0:20:12:any:1",
            "face:opponent,wait:5,face:opponent",
        ));
        let mut e = Engine::default();
        let mut c = context(1.);
        c.stopped = true;
        assert_eq!(sample(&mut e, &r, c, target(1., 5.)).requested, 0);
        assert!(!e.committed());
        c.stopped = false;
        sample(&mut e, &r, c, target(1., 5.));
        assert!(e.committed());
        c.motion_frame = 2.;
        sample(&mut e, &r, c, target(2., 5.));
        assert!(!e.committed());
        assert_eq!(e.feedback, Feedback::MissingTarget);
    }
}

#[cfg(test)]
mod combined_trigger_test {
    use super::*;
    /// Regression test for `combined_trigger_roundtrips_and_requires_both_crossing_and_vulnerability`.
    ///
    ///
///  Checks that combined trigger roundtrips and requires both crossing and vulnerability.
    #[test]
    fn combined_trigger_roundtrips_and_requires_both_crossing_and_vulnerability() {
        let rule = Rule::parse("roll_cross_vulnerable:grounded:0:20:12:any:1").unwrap();
        assert_eq!(Rule::parse(&rule.token()).unwrap(), rule);
        let mut sequence = Sequence::default();
        sequence.reaction = rule;
        sequence.len = 1;
        sequence.actions[0] = Action::Input {
            buttons: crate::native_input::MAPPER_A,
            stick: [0., -1.],
            frames: 4,
            relative: true,
        };
        let mut rules = Resolved::default();
        rules
            .add(Candidate {
                slot: 0,
                index: 0,
                sequence,
            })
            .unwrap();
        let mut engine = Engine::default();
        let mut ctx = GameplayContext {
            held: true,
            gameplay: true,
            status: crate::native_states::status::WAIT,
            situation: crate::native_states::situation::GROUND,
            motion_kind: 1,
            motion_frame: 1.,
            motion_rate: 1.,
            facing: 1.,
            ..GameplayContext::default()
        };
        for (frame, x, vuln, expected) in [
            (1., 5., 0., false),
            (2., -4., 3., false),
            (3., -4., 0., true),
        ] {
            ctx.motion_frame = frame;
            let target = Target {
                id: 3,
                status: ESCAPE_F,
                motion: 456,
                frame,
                rate: 1.,
                position: [x, 0.],
                velocity_x: 0.,
                vulnerable_in: Some(vuln),
                actionable_in: Some(6.),
            };
            let out = engine.sample(
                true,
                &rules,
                AssistSettings::default(),
                ctx,
                [Some(target), None, None, None, None, None, None, None],
                None,
            );
            assert_eq!(out.requested != 0, expected);
        }
    }
}

#[cfg(test)]
mod activation_tests {
    use super::*;
    use crate::{
        fighter_profiles::presets::PresetSlot,
        sequence_config::{persist, Catalog},
    };
    /// Regression test for `old_seven_fields_stay_menu_and_all_explicit_controls_roundtrip`.
    ///
    ///
///  Checks that old seven fields stay menu and all explicit controls roundtrip.
    #[test]
    fn old_seven_fields_stay_menu_and_all_explicit_controls_roundtrip() {
        let old = "roll:grounded:0:20:12:any:1";
        assert_eq!(Rule::parse(old).unwrap().control, Control::Menu);
        assert_eq!(Rule::parse(old).unwrap().token(), old);
        for c in Control::ALL {
            let mut r = Rule::parse(old).unwrap();
            r.control = c;
            assert_eq!(Rule::parse(&r.token()).unwrap(), r);
            let source = "[yoshi.roll.rule]\nstart_delay=0\nactions=jab:4\n";
            let mut seq = Catalog::parse(source).unwrap().entries()[0].sequence;
            seq.reaction = r;
            let file = persist::update(source, 5, PresetSlot::Roll, "rule", seq, false).unwrap();
            assert_eq!(Catalog::parse(&file).unwrap().entries()[0].sequence, seq);
        }
        assert!(Rule::parse(&format!("{old}:toggle_zr")).is_err());
    }
    /// Regression test for `only_enabled_control_groups_can_start_and_disabling_cancels_playback`.
    ///
    ///
///  Checks that only enabled control groups can start and disabling cancels playback.
    #[test]
    fn only_enabled_control_groups_can_start_and_disabling_cancels_playback() {
        let mut seq = Sequence::default();
        seq.len = 1;
        seq.actions[0] = Action::Input {
            buttons: crate::native_input::MAPPER_A,
            stick: [0.; 2],
            frames: 10,
            relative: true,
        };
        seq.reaction = Rule::parse("roll:any:0:20:12:any:1:toggle_x").unwrap();
        let mut rules = Resolved::default();
        rules
            .add(Candidate {
                slot: 0,
                index: 0,
                sequence: seq,
            })
            .unwrap();
        let mut e = Engine::default();
        let mut c = GameplayContext {
            held: true,
            gameplay: true,
            situation: crate::native_states::situation::GROUND,
            status: crate::native_states::status::WAIT,
            motion_kind: 1,
            motion_frame: 1.,
            motion_rate: 1.,
            facing: 1.,
            ..GameplayContext::default()
        };
        let t = Target {
            id: 1,
            status: ESCAPE_F,
            motion: 456,
            frame: 1.,
            rate: 1.,
            position: [5., 0.],
            velocity_x: 0.,
            vulnerable_in: Some(0.),
            actionable_in: Some(5.),
        };
        let targets = [Some(t), None, None, None, None, None, None, None];
        assert_eq!(
            e.sample(true, &rules, AssistSettings::default(), c, targets, None)
                .requested,
            0
        );
        rules.enabled_controls = Control::ToggleX.bit();
        c.motion_frame = 2.;
        assert_ne!(
            e.sample(true, &rules, AssistSettings::default(), c, targets, None)
                .requested,
            0
        );
        rules.enabled_controls = 0;
        c.motion_frame = 3.;
        assert_eq!(
            e.sample(true, &rules, AssistSettings::default(), c, targets, None)
                .requested,
            0
        );
        assert!(!e.committed());
    }
}
