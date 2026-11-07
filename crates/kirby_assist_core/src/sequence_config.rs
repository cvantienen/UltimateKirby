//! Data model for named SD-card ledge choices and input recipes.
//!
//! Parsing may allocate, but live execution copies one fixed-size recipe into
//! `LedgeSettings`; no strings, files, maps or allocations are touched in the
//! frame path. File parsing and validation live in the child `parser` module.
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
pub mod layout;
mod library;
mod parser;
pub mod persist;
pub use library::SequenceLibrary;
pub use parser::validate_facing_blocks;
use parser::{parse_slot, slot_name, valid_id, words_id};

pub const MAX_ACTIONS: usize = 32;
pub const MAX_FILE_BYTES: u64 = 512 * 1024;
pub const MAX_SEQUENCE_FRAME_COUNT: u16 = 3600;
// Admission bound only: parser allocation uses the actual header count, and
// live menus expand only the selected fighter. Leaves room for Pyra/Mythra
// without removing any saved choices; the 512 KiB file limit remains unchanged.
pub const MAX_ENTRIES: usize = 2112;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Opponent,
    AwayFromOpponent,
    Ledge,
    Inward,
    Outward,
    Facing,
    Backward,
    SavedLedge,
    AwayFromLedge,
}

impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Opponent => "Opponent",
            Self::AwayFromOpponent => "Away from opponent",
            Self::Ledge => "Ledge (legacy)",
            Self::Inward => "Inward",
            Self::Outward => "Outward",
            Self::Facing => "Facing",
            Self::Backward => "Backward",
            Self::SavedLedge => "Toward saved ledge",
            Self::AwayFromLedge => "Away from saved ledge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeCheck {
    Ignore,
    Verify(f32),
    FighterGrabRange,
    /// Check once on entry and cancel immediately when outside standing reach.
    FighterGrabRangeInstant,
}

/// Explicit, bounded sequence predicates using already-copied fighter facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    SelfGrounded,
    SelfAirborne,
    EnemyGrounded,
    EnemyAirborne,
    EnemyShielding,
    EnemyNotShielding,
    EnemyInFront,
    EnemyBehind,
    EnemyWithin,
    EnemyOutside,
}
impl Condition {
    pub fn token(self) -> &'static str {
        match self {
            Self::SelfGrounded => "self_grounded",
            Self::SelfAirborne => "self_airborne",
            Self::EnemyGrounded => "enemy_grounded",
            Self::EnemyAirborne => "enemy_airborne",
            Self::EnemyShielding => "enemy_shielding",
            Self::EnemyNotShielding => "enemy_not_shielding",
            Self::EnemyInFront => "enemy_in_front",
            Self::EnemyBehind => "enemy_behind",
            Self::EnemyWithin => "enemy_within",
            Self::EnemyOutside => "enemy_outside",
        }
    }
    pub fn from_token(token: &str) -> Option<Self> {
        [
            Self::SelfGrounded,
            Self::SelfAirborne,
            Self::EnemyGrounded,
            Self::EnemyAirborne,
            Self::EnemyShielding,
            Self::EnemyNotShielding,
            Self::EnemyInFront,
            Self::EnemyBehind,
            Self::EnemyWithin,
            Self::EnemyOutside,
        ]
        .into_iter()
        .find(|c| c.token() == token)
    }
    pub fn needs_enemy(self) -> bool {
        !matches!(self, Self::SelfGrounded | Self::SelfAirborne)
    }
    pub fn has_distance(self) -> bool {
        matches!(self, Self::EnemyWithin | Self::EnemyOutside)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputReference {
    #[default]
    Screen,
    Facing,
    Opponent,
}
impl InputReference {
    pub const ALL: [Self; 3] = [Self::Screen, Self::Facing, Self::Opponent];
    pub fn token(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Facing => "facing",
            Self::Opponent => "opponent",
        }
    }
    fn parse(s: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|r| r.token() == s)
            .ok_or_else(|| "input reference must be screen, facing or opponent".into())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Action {
    #[default]
    Empty,
    /// Literal canonical controller input, with no action-readiness gate.
    Input {
        buttons: u32,
        stick: [f32; 2],
        frames: u8,
        relative: bool,
    },
    /// Literal input with horizontal axis toward (+) / away from (-) the target.
    /// Direction is captured at step entry; requires a fresh observed target.
    OpponentInput {
        buttons: u32,
        stick: [f32; 2],
        frames: u8,
    },
    /// Simultaneous gameplay buttons and both sticks; no readiness/move writes.
    Controller {
        buttons: u32,
        stick: [f32; 2],
        cstick: [f32; 2],
        frames: u8,
        reference: InputReference,
        cstick_reference: InputReference,
    },
    /// Repeated literal pulses; release stick/buttons between presses.
    Tap {
        buttons: u32,
        stick: [f32; 2],
        hold: u8,
        gap: u8,
        frames: u16,
    },
    CStick {
        stick: [f32; 2],
        frames: u8,
    },
    /// Skip when already near; otherwise chase for exactly frames before continuing.
    Chase {
        gap: f32,
        strength: f32,
        frames: u8,
    },
    Wait(u8),
    Face(Direction),
    IfFacingLedge {
        away: bool,
    },
    /// Instant current opponent-side branch; no turn or input is performed.
    IfFacingOpponent {
        away: bool,
    },
    ElseFacing,
    EndFacing,
    /// Input-only positioning at a locked ledge's inward offset.
    LedgePosition {
        offset: f32,
        strength: f32,
        tolerance: f32,
        brake: f32,
        frames: u16,
    },
    Approach {
        airborne: bool,
        strength: f32,
        frames: u8,
        gap: f32,
    },
    Walk {
        direction: Direction,
        units: f32,
        strength: f32,
        min_gap: Option<f32>,
    },
    Run {
        direction: Direction,
        units: f32,
    },
    /// Timed horizontal stick; resolves its spatial direction each update.
    Drift {
        direction: Direction,
        strength: f32,
        frames: u8,
    },
    Range(RangeCheck),
    ShieldUntilBlock(u8),
    /// Grounded shield until a fresh physical input takes control.
    ShieldUntilManual,
    /// Continue if true, else wait up to `wait` native frames, then end. Zero is immediate.
    Check {
        condition: Condition,
        value: f32,
        wait: u8,
    },
}

impl Action {
    /// Onstage reaction recipes have no saved ledge geometry.
    pub fn needs_ledge(self) -> bool {
        matches!(
            self,
            Self::IfFacingLedge { .. }
                | Self::LedgePosition { .. }
                | Self::Face(
                    Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge
                )
                | Self::Walk {
                    direction: Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge,
                    ..
                }
                | Self::Run {
                    direction: Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge,
                    ..
                }
                | Self::Drift {
                    direction: Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge,
                    ..
                }
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sequence {
    pub actions: [Action; MAX_ACTIONS],
    pub len: u8,
    pub start_delay: u16,
    pub timeout: u16,
    pub spatial_timeout: u16,
    pub mode: Option<crate::button_bindings::ModeBinding>,
    pub builtin: Option<crate::button_bindings::Builtin>,
    pub while_held: bool,
    /// Opt-in: stop this tape if the fighter becomes airborne.
    pub ground_only: bool,
    pub reaction: crate::reaction_rules::Rule,
}
impl Default for Sequence {
    fn default() -> Self {
        Self {
            actions: [Action::Empty; MAX_ACTIONS],
            len: 0,
            start_delay: 0,
            timeout: MAX_SEQUENCE_FRAME_COUNT,
            spatial_timeout: 60,
            mode: None,
            builtin: None,
            while_held: false,
            ground_only: false,
            reaction: crate::reaction_rules::Rule::default(),
        }
    }
}
impl Sequence {
    pub fn actions(&self) -> &[Action] {
        &self.actions[..usize::from(self.len)]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub fighter_kind: i32,
    pub fighter: String,
    pub slot: PresetSlot,
    pub choice: String,
    pub name: Option<String>,
    pub default: bool,
    explicit_start: bool,
    pub sequence: Sequence,
}

/// Inert, fighter-independent Training editor starter; never a live response.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub sequence: Sequence,
}
pub const MAX_TEMPLATES: usize = 64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    entries: Vec<Entry>,
    pub templates: Vec<Template>,
    pub universal: Option<String>,
    pub tuning: std::collections::BTreeMap<i32, crate::fighter_tuning::Tuning>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_input::*;
    fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
        Action::Input {
            buttons,
            stick,
            frames,
            relative,
        }
    }

    fn parse(actions: &str) -> Sequence {
        Catalog::parse(&format!(
            "[kirby.drop.custom]\nstart_delay=0\nactions = {actions}"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    #[test]
    fn only_universal_bindings_are_ignored_and_fighter_bindings_stay_separate() {
        let source="[universal.x.old]\nold invalid text ignored\n[kirby.x.custom]\nstart_delay=0\nactions=short_hop\n[yoshi.x.custom]\nstart_delay=2\nactions=grab\n[kirby.y.custom]\nstart_delay=1\nactions=shield\n";
        let c = Catalog::parse(source).unwrap();
        assert_eq!(c.len(), 3);
        assert_ne!(c.entries()[0].fighter_kind, c.entries()[1].fighter_kind);
        assert_ne!(c.entries()[0].sequence, c.entries()[1].sequence);
        assert!(Catalog::parse("[kirby.x.bad]\nstart_delay=oops\nactions=grab").is_err());
    }
    #[test]
    fn file_facing_inputs_and_special_strength_are_explicit_and_bounded() {
        let s = parse("facing_input:A:1:-0.5:1, side_special:2:0.5");
        assert_eq!(s.actions()[0], input(MAPPER_A, [1.0, -0.5], 1, true));
        assert_eq!(s.actions()[1], input(MAPPER_B, [0.5, 0.0], 2, true));
        assert!(Catalog::parse("[kirby.neutral.bad]\nstart_delay=0\nactions=full_hop").is_err());
        assert!(
            Catalog::parse("[kirby.neutral.bad]\nstart_delay=0\nactions=up_special:1:2").is_err()
        );
    }

    #[test]
    fn timeline_accepts_ordered_multi_move_air_specials_and_explicit_durations() {
        let sequence = parse("Short Hop, wait:2, Up B:3, wait:6, Down Special, release, grab");
        assert_eq!(sequence.len, 7);
        assert_eq!(sequence.actions()[0], input(MAPPER_X, [0.0; 2], 1, true));
        assert_eq!(sequence.actions()[1], Action::Wait(2));
        assert_eq!(sequence.actions()[2], input(MAPPER_B, [0.0, 1.0], 3, true));
        assert_eq!(sequence.actions()[4], input(MAPPER_B, [0.0, -1.0], 1, true));
    }
    #[test]
    fn timeline_raw_inputs_preserve_signed_axes_and_button_chords() {
        let sequence =
            parse("input:A+X:-1:0.5:3, cstick:0:-1:1, hold:shield:8, press:B, stick:1:0:4");
        assert_eq!(
            sequence.actions()[0],
            input(MAPPER_A | MAPPER_X, [-1.0, 0.5], 3, false)
        );
        assert_eq!(
            sequence.actions()[1],
            Action::CStick {
                stick: [0.0, -1.0],
                frames: 1
            }
        );
        assert_eq!(sequence.actions()[2], input(MAPPER_ZR, [0.0; 2], 8, false));
    }
    #[test]
    fn timeline_universal_actions_validate_for_every_configured_fighter_without_hitbox_data() {
        let profiles = FighterProfiles::default();
        let mut checked = 0;
        for kind in 0..crate::fighter_kinds::PLAYABLE_KIND_COUNT {
            if !profiles
                .profile(kind)
                .is_some_and(|p| p.fighter_kind == kind)
            {
                continue;
            }
            let mut catalog = Catalog::parse("[kirby.neutral.everything]\nstart_delay=0\nactions = nair, fair, bair, uair, dair, jab, ftilt, utilt, dtilt, fsmash, usmash, dsmash, neutral_special, side_special, up_special, down_special, dash_attack, shield, roll_forward, roll_backward, spot_dodge, air_dodge, grab").unwrap();
            catalog.entries[0].fighter_kind = kind;
            catalog.validate(&profiles).unwrap();
            checked += 1;
        }
        assert!(checked > 70);
    }
    #[test]
    fn timeline_numeric_keys_are_order_independent_and_duplicates_invalid() {
        let catalog = Catalog::parse("[kirby.roll.custom]\nstart_delay = 7\ntimeout = 200\nspatial_timeout = 15\nactions = dash_attack:3:2\nname = My attack").unwrap();
        let sequence = catalog.entries()[0].sequence;
        assert_eq!(
            (
                sequence.start_delay,
                sequence.timeout,
                sequence.spatial_timeout
            ),
            (7, 200, 15)
        );
        assert_eq!(sequence.actions()[0], input(0, [1.0, 0.0], 3, true));
        assert_eq!(sequence.actions()[1], input(MAPPER_A, [1.0, 0.0], 2, true));
        assert!(
            Catalog::parse("[kirby.roll.custom]\nstart_delay=1\nstart_delay=2\nactions=jab")
                .is_err()
        );
    }
    #[test]
    fn timeline_file_menus_require_explicit_delays_and_have_no_code_fallback() {
        assert!(Catalog::parse("[kirby.neutral.custom]\nactions=jab")
            .unwrap_err()
            .contains("requires start_delay"));
        assert!(
            Catalog::parse("[kirby.neutral.custom]\nstart_delay=0\nactions=code:nair").is_err()
        );
        let catalog = Catalog::parse("[kirby.neutral.first]\nstart_delay=0\ndefault=true\nactions=jab\n[kirby.neutral.second]\nstart_delay=2\ndefault=true\nactions=up_b").unwrap();
        assert!(catalog
            .validate(&FighterProfiles::default())
            .unwrap_err()
            .contains("multiple defaults"));
    }

    #[test]
    fn timeline_bounds_unknown_names_and_nonfinite_axes_are_rejected() {
        for actions in [
            "up_special:0",
            "input:A:NaN:0:1",
            "input:A:2:0:1",
            "hold:bad:2",
            "jab,,jab",
            "cstick:0:1:256",
            "unknown",
            "walk:away:5:0.5:2",
        ] {
            assert!(
                Catalog::parse(&format!(
                    "[kirby.neutral.custom]\nstart_delay=0\nactions={actions}"
                ))
                .is_err(),
                "{actions}"
            );
        }
        let long = vec!["jab"; MAX_ACTIONS + 1].join(",");
        assert!(Catalog::parse(&format!(
            "[kirby.neutral.custom]\nstart_delay=0\nactions={long}"
        ))
        .is_err());
        assert!(Catalog::parse(
            "[kirby.neutral.custom]\nstart_delay=0\nactions=jab\n[kirby.neutral.custom]\nstart_delay=0\nactions=up_special"
        )
        .is_err());
    }
    #[test]
    fn timeline_deployment_validates_and_filters_only_diagnostic_tests() {
        let catalog = Catalog::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
        )))
        .unwrap();
        for diagnostic in [false, true] {
            let mut profiles = FighterProfiles::default();
            profiles
                .add_sequence_choices(&catalog.clone().for_build(diagnostic))
                .unwrap();
        }
        assert!(catalog.clone().for_build(false).len() < catalog.for_build(true).len());
    }
}

#[cfg(test)]
mod facing_block_tests {
    use super::*;
    #[test]
    fn markers_validate_and_save_roundtrip() {
        let source="[yoshi.neutral.test]\nstart_delay=0\nactions=if_facing_ledge, drift:saved_ledge:0.25:4, end_facing, if_back_to_ledge, walk:away_from_ledge:4:0.3, end_facing";
        let c = Catalog::parse(source).unwrap();
        let s = c.entries()[0].sequence;
        let saved = persist::update(source, 5, PresetSlot::Neutral, "test", s, false).unwrap();
        assert_eq!(Catalog::parse(&saved).unwrap().entries()[0].sequence, s);
        for actions in [
            "end_facing",
            "if_facing_ledge, a",
            "if_facing_ledge, if_back_to_ledge, a, end_facing, end_facing",
        ] {
            assert!(Catalog::parse(&format!(
                "[yoshi.neutral.test]\nstart_delay=0\nactions={actions}"
            ))
            .is_err());
        }
    }
}

#[cfg(test)]
mod generic_template_tests {
    use super::*;
    #[test]
    fn named_templates_are_inert_bounded_and_survive_organizing_and_saving() {
        let source="[universal.settings]\nenabled=true\n[template.shield]\nname=Quick shield\nstart_delay=0\nactions=input:zr:0:0:4\n[kirby.neutral.test]\nstart_delay=0\nactions=jab:2\n";
        let c = Catalog::parse(source).unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!(c.templates.len(), 1);
        assert_eq!(c.templates[0].id, "shield");
        assert_eq!(c.templates[0].name, "Quick shield");
        let organized = layout::organize(source).unwrap();
        let again = Catalog::parse(&organized).unwrap();
        assert_eq!(again.templates, c.templates);
        let updated = persist::update(
            source,
            6,
            PresetSlot::Neutral,
            "test",
            c.entries()[0].sequence,
            false,
        )
        .unwrap();
        assert_eq!(Catalog::parse(&updated).unwrap().templates, c.templates);
        for diagnostic in [false, true] {
            let c = c.clone().for_build(diagnostic);
            assert_eq!(c.templates.len(), 1);
            c.validate(&FighterProfiles::default()).unwrap();
        }
    }
    #[test]
    fn templates_cannot_activate_modes_reactions_or_defaults_and_reject_duplicate_ids() {
        for value in [
            "default=true",
            "while_held=true",
            "reaction=auto_roll:30:12",
            "mode=auto_tech",
            "builtin=normal",
        ] {
            assert!(
                Catalog::parse(&format!(
                    "[template.bad]\nstart_delay=0\nactions=jab\n{value}\n"
                ))
                .is_err(),
                "{value}"
            );
        }
        assert!(Catalog::parse(
            "[template.a]\nstart_delay=0\nactions=jab\n[template.a]\nstart_delay=0\nactions=grab\n"
        )
        .is_err());
        assert!(Catalog::parse("[template.a.b]\nstart_delay=0\nactions=jab\n").is_err());
        let text = (0..=MAX_TEMPLATES)
            .map(|i| format!("[template.test_{i}]\nstart_delay=0\nactions=jab\n"))
            .collect::<String>();
        assert!(Catalog::parse(&text)
            .unwrap_err()
            .contains("generic templates"));
    }
}
