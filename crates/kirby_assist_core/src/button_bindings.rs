//! Mode identities for On/Off settings and legacy file compatibility.
//! These legacy automatic-helper identities stay menu-only. Reaction rules use
//! their own explicit Control whitelist; Builtin below exposes only native button helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    LedgeTrap,
    LedgeTrump,
    AutoTech,
    AutoMash,
    IdleCrouch,
    TiltTurn,
    SpotPunish,
    JabLock,
    OutOfShield,
}
impl Mode {
    pub const ALL: [Self; 9] = [
        Self::LedgeTrap,
        Self::LedgeTrump,
        Self::AutoTech,
        Self::AutoMash,
        Self::IdleCrouch,
        Self::TiltTurn,
        Self::SpotPunish,
        Self::JabLock,
        Self::OutOfShield,
    ];
    pub fn bit(self) -> u16 {
        1 << self as usize
    }
    pub fn id(self) -> &'static str {
        match self {
            Self::LedgeTrap => "ledge_trap",
            Self::LedgeTrump => "ledge_trump",
            Self::AutoTech => "auto_tech",
            Self::AutoMash => "auto_mash",
            Self::IdleCrouch => "idle_crouch",
            Self::TiltTurn => "tilt_turn",
            Self::SpotPunish => "spot_punish",
            Self::JabLock => "jab_lock",
            Self::OutOfShield => "out_of_shield",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::LedgeTrap => "Ledge trap",
            Self::LedgeTrump => "Ledge trump",
            Self::AutoTech => "Auto-tech",
            Self::AutoMash => "Auto-mash",
            Self::IdleCrouch => "Idle crouch",
            Self::TiltTurn => "Dtilt face",
            Self::SpotPunish => "Spot punish",
            Self::JabLock => "Jab lock",
            Self::OutOfShield => "Out of shield",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.id() == s)
    }
    pub fn supports(self, kind: i32) -> bool {
        match self {
            Self::IdleCrouch => kind == 6,
            Self::TiltTurn => matches!(kind, 5 | 6),
            Self::JabLock => kind == 5,
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
            "Hold: Ledge trump",
            "Toggle: Ledge trump",
            "Hold: Auto-tech",
            "Toggle: Auto-tech",
            "Hold: Auto-mash",
            "Toggle: Auto-mash",
            "Hold: Idle crouch",
            "Toggle: Idle crouch",
            "Hold: Dtilt face",
            "Toggle: Dtilt face",
            "Hold: Spot punish",
            "Toggle: Spot punish",
            "Hold: Jab lock",
            "Toggle: Jab lock",
            "Hold: Out of shield",
            "Toggle: Out of shield",
        ];
        LABELS[self.index()]
    }
    pub fn choice_id(self) -> &'static str {
        const IDS: [&str; ModeBinding::COUNT] = [
            "__mode_ledge_trap_hold",
            "__mode_ledge_trap_toggle",
            "__mode_ledge_trump_hold",
            "__mode_ledge_trump_toggle",
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
            "__mode_jab_lock_hold",
            "__mode_jab_lock_toggle",
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
    ShortHop,
    LedgeAssist,
    TurboTeabag,
}
impl Builtin {
    pub const ALL: [Self; 4] = [
        Self::Normal,
        Self::ShortHop,
        Self::LedgeAssist,
        Self::TurboTeabag,
    ];
    pub fn token(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::ShortHop => "short_hop",
            Self::LedgeAssist => "ledge_assist",
            Self::TurboTeabag => "turbo_teabag",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal button",
            Self::ShortHop => "Built-in: Short hop",
            Self::LedgeAssist => "Built-in: Ledge assist / walk",
            Self::TurboTeabag => "Built-in: Turbo t-bag",
        }
    }
    pub fn choice_id(self) -> &'static str {
        match self {
            Self::Normal => "__normal_button",
            Self::ShortHop => "__builtin_short_hop",
            Self::LedgeAssist => "__builtin_ledge_assist",
            Self::TurboTeabag => "__builtin_turbo_teabag",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.token() == value)
    }
    pub fn default_for(slot: crate::fighter_profiles::presets::PresetSlot) -> Self {
        use crate::fighter_profiles::presets::PresetSlot;
        match slot {
            PresetSlot::X => Self::ShortHop,
            PresetSlot::L3 => Self::TurboTeabag,
            _ => Self::Normal,
        }
    }
    pub fn template_offset(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::ShortHop => 2,
            Self::LedgeAssist => 3,
            Self::TurboTeabag => 4,
        }
    }
}
