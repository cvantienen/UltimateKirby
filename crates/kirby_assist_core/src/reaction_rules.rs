//! Bounded, input-only roll reaction rules. No native queries or file access.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings},
    roll_read::{Target, ESCAPE_B, ESCAPE_F},
    sequence_config::{Action, Sequence},
};
pub fn starts_direct_up_b(sequence: &Sequence) -> bool {
    simple::starts_up_b(sequence)
}
mod automatic_watch;
mod scope;
pub use automatic_watch::{sample_optional_simple, simple_readiness, SimpleStart};
pub use scope::{
    automatic_fighter_target_available, automatic_start_allowed, automatic_start_status,
    automatic_target_available, automatic_tracking_target_available, automatic_up_b_receipt_status,
    automatic_up_b_status, AutomaticObservationScope,
};
pub(crate) fn native_up_b_started(kind: i32, c: GameplayContext) -> bool {
    simple::up_b_status(kind, c)
}
pub const MAX_RULES_PER_MODE: usize = 8;
pub const MAX_RULES: usize = MAX_RULES_PER_MODE * 7;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trigger {
    #[default]
    Off,
    Roll,
    Crossed,
    Vulnerable,
    CrossVulnerable,
    ShieldHit,
    GrabDodged,
    LedgeRegrab,
    SpotDodge,
    EnemyShield,
    ShieldRoll,
    ShieldCross,
    ShieldVulnerable,
    ShieldCrossVulnerable,
    ShieldAway,
    ShieldAwayVulnerable,
    ShieldPressureVulnerable,
    ShieldHitVulnerable,
    GrabDodgedVulnerable,
    SpotDodgeAny,
    AutoRoll,
    AutoSpot,
    AutoAir,
    AutoGrab,
    AutoShieldGrab,
    AutoGrabDodged,
    AutoShieldBackup,
}
impl Trigger {
    pub const AUTO_EVENTS: [Self; 7] = [
        Self::AutoRoll,
        Self::AutoSpot,
        Self::AutoAir,
        Self::AutoGrab,
        Self::AutoShieldGrab,
        Self::AutoGrabDodged,
        Self::AutoShieldBackup,
    ];
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
    pub fn event_matches(self, event: Self) -> bool {
        self == event
            || matches!(
                (self, event),
                (Self::ShieldHitVulnerable, Self::ShieldHit)
                    | (Self::GrabDodgedVulnerable, Self::GrabDodged)
                    | (Self::SpotDodgeAny, Self::SpotDodge)
            )
    }
    pub fn needs_vulnerable(self) -> bool {
        matches!(
            self,
            Self::ShieldHitVulnerable
                | Self::GrabDodgedVulnerable
                | Self::SpotDodge
                | Self::ShieldPressureVulnerable
        )
    }
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelfState {
    #[default]
    Any,
    Grounded,
    Airborne,
    Standing,
    Walking,
    Running,
    Shielding,
}
impl SelfState {
    pub const ALL: [Self; 7] = [
        Self::Any,
        Self::Grounded,
        Self::Airborne,
        Self::Standing,
        Self::Walking,
        Self::Running,
        Self::Shielding,
    ];
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
    pub(crate) fn matches(self, c: GameplayContext) -> bool {
        match self {
            Self::Any => true,
            Self::Grounded => c.situation == 0,
            Self::Airborne => c.situation == 2,
            Self::Standing => c.situation == 0 && c.status == crate::native_states::status::WAIT,
            Self::Walking => c.situation == 0 && c.status == crate::native_states::status::WALK,
            Self::Running => {
                c.situation == 0
                    && matches!(
                        c.status,
                        crate::native_states::status::DASH
                            | crate::native_states::status::RUN
                            | crate::native_states::status::TURN_DASH
                            | crate::native_states::status::TURN_RUN
                    )
            }
            Self::Shielding => {
                c.situation == 0
                    && matches!(
                        c.status,
                        crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
                    )
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Any,
    Front,
    Behind,
}
impl Side {
    pub const ALL: [Self; 3] = [Self::Any, Self::Front, Self::Behind];
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
    Menu,
    ToggleX,
    ToggleY,
    ToggleUp,
    ToggleDown,
    ToggleLeft,
    ToggleRight,
}
impl Control {
    pub const ALL: [Self; 7] = [
        Self::Menu,
        Self::ToggleX,
        Self::ToggleY,
        Self::ToggleUp,
        Self::ToggleDown,
        Self::ToggleLeft,
        Self::ToggleRight,
    ];
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    pub trigger: Trigger,
    pub state: SelfState,
    pub min: f32,
    pub max: f32,
    pub height: f32,
    pub side: Side,
    pub priority: u8,
    pub control: Control,
}
impl Default for Rule {
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
    pub fn active(self) -> bool {
        self.trigger != Trigger::Off
    }
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
    pub fn adjust(&mut self, n: u8, up: bool) {
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
#[derive(Clone, Copy)]
pub struct Candidate {
    pub slot: usize,
    pub index: usize,
    pub sequence: Sequence,
}
pub struct Resolved {
    pub rules: Vec<Option<Candidate>>,
    pub len: usize,
    pub needs_ground: bool,
    pub ground_distance: f32,
    pub ground_height: f32,
    pub controls: u8,
    pub enabled_controls: u8,
    defense_mask: u32,
    defense_needs_ground: bool,
    defense_watch: [Option<(f32, f32)>; 7],
    simple: Option<Box<[Option<Sequence>; 7]>>,
    simple_starts: [Option<SimpleStart>; 3],
    automatic_ground: [Option<(f32, f32)>; 7],
    automatic_up_special: u8,
}
impl Default for Resolved {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            len: 0,
            needs_ground: false,
            ground_distance: 0.,
            ground_height: 0.,
            controls: 0,
            enabled_controls: Control::Menu.bit(),
            defense_mask: 0,
            defense_needs_ground: false,
            defense_watch: [None; 7],
            simple: None,
            simple_starts: [None; 3],
            automatic_ground: [None; 7],
            automatic_up_special: 0,
        }
    }
}
impl Resolved {
    pub fn simple_sequence(&self, index: usize) -> Option<&Sequence> {
        self.simple
            .as_ref()
            .and_then(|s| s.get(index))
            .and_then(Option::as_ref)
    }
    pub fn has_simple(&self) -> bool {
        self.simple.is_some()
    }
    pub fn has_simple_events(&self) -> bool {
        (0..3).any(|i| self.simple_sequence(i).is_some())
    }
    pub fn has_defense(&self, trigger: Trigger) -> bool {
        let alias = match trigger {
            Trigger::ShieldHit => 1 << Trigger::ShieldHitVulnerable as u8,
            Trigger::GrabDodged => 1 << Trigger::GrabDodgedVulnerable as u8,
            Trigger::SpotDodge => 1 << Trigger::SpotDodgeAny as u8,
            _ => 0,
        };
        self.defense_mask & ((1 << trigger as u8) | alias) != 0
    }
    pub fn defense_ground(&self) -> bool {
        self.defense_needs_ground
    }
    pub fn defense_watch(&self, trigger: Trigger) -> Option<(f32, f32)> {
        self.defense_watch[trigger.family()]
    }
    pub fn set_enabled_controls(&mut self, controls: u8) {
        if self.enabled_controls != controls {
            self.enabled_controls = controls;
            self.rebuild_ground_watch();
        }
    }
    fn rebuild_ground_watch(&mut self) {
        self.needs_ground = false;
        self.ground_distance = 0.;
        self.ground_height = 0.;
        for c in self.rules.iter().flatten() {
            if c.sequence.reaction.trigger.defensive()
                || self.enabled_controls & c.sequence.reaction.control.bit() == 0
            {
                continue;
            }
            if c.sequence.actions().iter().any(|a| {
                matches!(
                    a,
                    Action::Chase { .. }
                        | Action::Walk { .. }
                        | Action::Run { .. }
                        | Action::Approach {
                            airborne: false,
                            ..
                        }
                )
            }) {
                self.needs_ground = true;
                self.ground_distance = self.ground_distance.max(c.sequence.reaction.max);
                self.ground_height = self.ground_height.max(c.sequence.reaction.height);
            }
        }
    }
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
            if starts_direct_up_b(&c.sequence) {
                self.automatic_up_special |= 1 << index;
            }
            if index < 3 {
                self.simple_starts[index] = Some(SimpleStart {
                    distance: c.sequence.reaction.max,
                    height: c.sequence.reaction.height,
                    up_special: starts_direct_up_b(&c.sequence),
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
            let walking = c.sequence.actions().iter().any(|a| {
                matches!(
                    a,
                    Action::Walk { .. }
                        | Action::Run { .. }
                        | Action::Approach {
                            airborne: false,
                            ..
                        }
                )
            });
            self.defense_needs_ground |= walking;
            if walking {
                let r = c.sequence.reaction;
                let old = self.defense_watch[r.trigger.family()].unwrap_or((0., 0.));
                self.defense_watch[r.trigger.family()] =
                    Some((old.0.max(r.max), old.1.max(r.height)));
            }
        } else {
            self.controls |= c.sequence.reaction.control.bit();
        }
        self.rules.push(Some(c));
        self.len += 1;
        self.rebuild_ground_watch();
        self.rules[..self.len].sort_by_key(|x| x.unwrap().sequence.reaction.priority);
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Feedback {
    #[default]
    Watching,
    State,
    Range,
    Height,
    Side,
    Crossing,
    Invulnerable,
    Matched(u8),
    Cancelled,
    MissingTarget,
    Finished,
}
impl Feedback {
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
#[derive(Clone, Copy)]
struct Track {
    id: u32,
    status: i32,
    motion: u64,
    frame: f32,
    dx: f32,
    crossed: bool,
    used: bool,
}
#[derive(Default)]
pub struct Engine {
    tracks: [Option<Track>; crate::battle_objects::FIGHTER_CAPACITY],
    plan: Option<InputTimeline>,
    target: Option<u32>,
    pub feedback: Feedback,
    pub last_priority: Option<u8>,
    active_control: Option<Control>,
    simple_up_b: bool,
    simple_up_b_seen: bool,
    simple_manual_epoch: u64,
    simple_delivery: crate::move_input::DeliveryBudget,
    simple_receipt: crate::move_input::UpBReceipt,
    simple_waiting: bool,
}
impl Engine {
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        for track in &mut self.tracks {
            if track.is_some_and(|t| {
                known
                    .iter()
                    .flatten()
                    .any(|(id, status)| *id == t.id && *status != t.status)
            }) {
                *track = None;
            }
        }
    }
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn committed(&self) -> bool {
        self.plan.is_some()
    }
    pub fn direct_up_b(&self) -> bool {
        self.simple_up_b && self.plan.is_some()
    }
    pub fn up_b_delivery_started(&self) -> bool {
        self.direct_up_b() && (self.simple_up_b_seen || self.simple_receipt.sent())
    }
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.direct_up_b() && !self.simple_up_b_seen && self.target == Some(target) {
            self.simple_delivery.mapped(tick);
        }
    }
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.direct_up_b() && !self.simple_up_b_seen && self.target == Some(target) {
            self.simple_delivery.mapped(tick);
            self.simple_receipt.mapped(tick);
        }
    }

    pub fn up_b_receipt_pending(&self) -> bool {
        self.direct_up_b() && !self.simple_up_b_seen && self.simple_receipt.sent()
    }
    pub fn awaiting_up_b(&self) -> bool {
        self.up_b_receipt_pending() && self.simple_waiting
    }
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.up_b_receipt_pending() {
            self.simple_receipt.tick(tick);
        }
    }
    pub fn needs_ground(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::needs_ground)
    }
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.plan.as_ref().map(InputTimeline::right_stick)
    }
    pub fn walking(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::walking)
    }
    pub fn attack_hold(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::attack_hold)
    }
    pub fn cancel(&mut self) {
        self.plan = None;
        self.target = None;
        self.active_control = None;
        self.simple_up_b = false;
        self.simple_up_b_seen = false;
        self.simple_manual_epoch = 0;
        self.simple_delivery = Default::default();
        self.simple_receipt = Default::default();
        self.simple_waiting = false;
        self.feedback = Feedback::Cancelled;
    }
    pub fn sample(
        &mut self,
        allowed: bool,
        rules: &Resolved,
        settings: LedgeSettings,
        mut c: GameplayContext,
        targets: [Option<Target>; 8],
        mut follow: Option<Opponent>,
    ) -> ControllerOutput {
        let old = self.tracks;
        for (i, t) in targets.iter().enumerate() {
            self.tracks[i] = t
                .filter(|t| {
                    matches!(t.status, ESCAPE_F | ESCAPE_B)
                        && t.frame.is_finite()
                        && t.frame >= 0.
                        && t.rate.is_finite()
                        && t.rate > 0.
                        && t.position.iter().all(|v| v.is_finite())
                })
                .map(|t| {
                    let dx = t.position[0] - c.position[0];
                    let prev = old.iter().flatten().find(|p| {
                        p.id == t.id
                            && p.status == t.status
                            && p.motion == t.motion
                            && t.frame >= p.frame
                    });
                    Track {
                        id: t.id,
                        status: t.status,
                        motion: t.motion,
                        frame: t.frame,
                        dx: if dx.abs() > 0.001 {
                            dx
                        } else {
                            prev.map_or(dx, |p| p.dx)
                        },
                        crossed: prev.is_some_and(|p| p.crossed || p.dx * dx < 0.),
                        used: prev.is_some_and(|p| p.used),
                    }
                });
        }
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
            self.feedback = Feedback::Watching;
            'rules: for candidate in rules.rules.iter().flatten() {
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
                    if !r.state.matches(c) {
                        self.feedback = Feedback::State;
                        continue;
                    }
                    let d = (t.position[0] - c.position[0]).abs();
                    if d < r.min || d > r.max {
                        self.feedback = Feedback::Range;
                        continue;
                    }
                    if (t.position[1] - c.position[1]).abs() > r.height {
                        self.feedback = Feedback::Height;
                        continue;
                    }
                    let side = (t.position[0] - c.position[0]) * c.facing;
                    if matches!(r.side, Side::Front) && side <= 0.001
                        || matches!(r.side, Side::Behind) && side >= -0.001
                    {
                        self.feedback = Feedback::Side;
                        continue;
                    }
                    if matches!(r.trigger, Trigger::Crossed | Trigger::CrossVulnerable)
                        && !track.crossed
                    {
                        self.feedback = Feedback::Crossing;
                        continue;
                    }
                    if matches!(r.trigger, Trigger::Vulnerable | Trigger::CrossVulnerable)
                        && !t.vulnerable_in.is_some_and(|v| v.is_finite() && v <= 0.)
                    {
                        self.feedback = Feedback::Invulnerable;
                        continue;
                    }
                    let enemy = Opponent {
                        id: t.id,
                        status: t.status,
                        situation: 0,
                        position: t.position,
                        velocity: [t.velocity_x, 0.],
                        motion: Some(crate::gameplay::MotionTiming {
                            kind: t.motion,
                            frame: t.frame,
                            rate: t.rate,
                            remaining: t.actionable_in,
                        }),
                        vulnerable_in: t.vulnerable_in,
                        ..Opponent::default()
                    };
                    let mut seq = candidate.sequence;
                    seq.while_held = false;
                    if follow.is_none() {
                        follow = Some(enemy);
                    }
                    self.plan = Some(InputTimeline::new(seq, enemy));
                    self.target = Some(t.id);
                    self.tracks[i].as_mut().unwrap().used = true;
                    self.feedback = Feedback::Matched(r.priority);
                    self.last_priority = Some(r.priority);
                    self.active_control = Some(r.control);
                    break 'rules;
                }
            }
        }
        c.held = true;
        if let Some(plan) = self.plan.as_mut() {
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
        } else {
            ControllerOutput::default()
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
    fn sequence(trigger: &str, actions: &str) -> Sequence {
        Catalog::parse(&format!(
            "[yoshi.roll.rule]\nstart_delay=0\nreaction={trigger}\nactions={actions}\n"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    fn context(frame: f32) -> GameplayContext {
        GameplayContext {
            held: true,
            gameplay: true,
            status: crate::native_states::status::WAIT,
            situation: 0,
            motion_kind: 1,
            motion_frame: frame,
            motion_rate: 1.,
            facing: 1.,
            position: [0., 0.],
            safe_ground: [true; 2],
            ..GameplayContext::default()
        }
    }
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
    fn sample(e: &mut Engine, r: &Resolved, c: GameplayContext, t: Target) -> ControllerOutput {
        e.sample(
            true,
            r,
            LedgeSettings::default(),
            c,
            [Some(t), None, None, None, None, None, None, None],
            None,
        )
    }
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
                let out = engine.sample(true, &rules, LedgeSettings::default(), c, targets, follow);
                assert_eq!(out.requested, expected, "native frame {frame}");
                for _ in 0..repeats {
                    let duplicate = engine.sample(
                        true,
                        &rules,
                        LedgeSettings::default(),
                        c,
                        targets,
                        Some(following),
                    );
                    assert_eq!(duplicate.requested, out.requested);
                }
            }
        }
    }
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
            situation: 0,
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
                LedgeSettings::default(),
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
            situation: 0,
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
            e.sample(true, &rules, LedgeSettings::default(), c, targets, None)
                .requested,
            0
        );
        rules.enabled_controls = Control::ToggleX.bit();
        c.motion_frame = 2.;
        assert_ne!(
            e.sample(true, &rules, LedgeSettings::default(), c, targets, None)
                .requested,
            0
        );
        rules.enabled_controls = 0;
        c.motion_frame = 3.;
        assert_eq!(
            e.sample(true, &rules, LedgeSettings::default(), c, targets, None)
                .requested,
            0
        );
        assert!(!e.committed());
    }
}

#[cfg(test)]
mod activation_ground_watch_test {
    use super::*;
    #[test]
    fn disabled_movement_groups_do_not_enable_ground_watch() {
        let parsed=crate::sequence_config::Catalog::parse("[yoshi.roll.attack]\nstart_delay=0\nreaction=roll:any:0:12:10:any:1\nactions=jab:4\n[yoshi.roll.move]\nstart_delay=0\nreaction=roll:any:0:40:5:any:2:toggle_x\nactions=walk:opponent:4:50%\n").unwrap();
        let mut resolved = Resolved::default();
        for (index, e) in parsed.entries().iter().enumerate() {
            resolved
                .add(Candidate {
                    slot: 2,
                    index,
                    sequence: e.sequence,
                })
                .unwrap();
        }
        assert!(!resolved.needs_ground);
        resolved.set_enabled_controls(Control::ToggleX.bit());
        assert!(resolved.needs_ground);
        assert_eq!(resolved.ground_distance, 40.);
        resolved.set_enabled_controls(Control::Menu.bit());
        assert!(!resolved.needs_ground);
        resolved.set_enabled_controls(0);
        assert!(!resolved.needs_ground);
    }
}
