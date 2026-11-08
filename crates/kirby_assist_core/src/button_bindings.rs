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
    JabLock,
    OutOfShield,
}
impl Mode {
    pub const ALL: [Self; 8] = [
        Self::LedgeTrap,
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
            Self::AutoTech => "Auto-tech",
            Self::AutoMash => "Auto-mash",
            Self::IdleCrouch => "Idle crouch",
            Self::TiltTurn => "Tilt face",
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
            Self::IdleCrouch => kind == crate::fighter_kinds::KIRBY,
            Self::TiltTurn => true,
            Self::JabLock => kind == crate::fighter_kinds::YOSHI,
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
