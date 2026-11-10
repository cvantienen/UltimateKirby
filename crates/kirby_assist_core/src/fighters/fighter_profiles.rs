//! Fighter identities and read-only reference facts. Ledge options and input
//! schedules exist only in the startup-loaded sequences.txt catalog.
pub mod presets {
    //! Startup-frozen file menu: labels and references to validated input timelines.
    //! No compiled fighter choices, response functions or move timing defaults.

    /// Defines the available `PresetSlot` cases.
    ///
    ///
///  Lists the available PresetSlot choices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum PresetSlot {
        /// Represents the `Neutral` case of `PresetSlot`.
        ///
        ///
///  Represents the Neutral option in PresetSlot.
        Neutral,
        /// Represents the `Jump` case of `PresetSlot`.
        ///
        ///
///  Represents the Jump option in PresetSlot.
        Jump,
        /// Represents the `Roll` case of `PresetSlot`.
        ///
        ///
///  Represents the Roll option in PresetSlot.
        Roll,
        /// Represents the `Attack` case of `PresetSlot`.
        ///
        ///
///  Represents the Attack option in PresetSlot.
        Attack,
        /// Represents the `Drop` case of `PresetSlot`.
        ///
        ///
///  Represents the Drop option in PresetSlot.
        Drop,
        /// Represents the `X` case of `PresetSlot`.
        ///
        ///
///  Represents the X option in PresetSlot.
        X,
        /// Represents the `Y` case of `PresetSlot`.
        ///
        ///
///  Represents the Y option in PresetSlot.
        Y,
        /// Represents the `DpadUp` case of `PresetSlot`.
        ///
        ///
///  Represents the DpadUp option in PresetSlot.
        DpadUp,
        /// Represents the `DpadDown` case of `PresetSlot`.
        ///
        ///
///  Represents the DpadDown option in PresetSlot.
        DpadDown,
        /// Represents the `DpadLeft` case of `PresetSlot`.
        ///
        ///
///  Represents the DpadLeft option in PresetSlot.
        DpadLeft,
        /// Represents the `DpadRight` case of `PresetSlot`.
        ///
        ///
///  Represents the DpadRight option in PresetSlot.
        DpadRight,
        /// Represents the `L3` case of `PresetSlot`.
        ///
        ///
///  Represents the L3 option in PresetSlot.
        L3,
        /// Represents the `R3` case of `PresetSlot`.
        ///
        ///
///  Represents the R3 option in PresetSlot.
        R3,
    }
    impl PresetSlot {
        /// Defines the `COUNT` constant used by this module.
        ///
        ///
///  Provides the COUNT constant value.
        pub const COUNT: usize = 13;
        /// Defines the `ALL` constant used by this module.
        ///
        ///
///  Provides the ALL constant value.
        pub const ALL: [Self; Self::COUNT] = [
            Self::Neutral,
            Self::Jump,
            Self::Roll,
            Self::Attack,
            Self::Drop,
            Self::X,
            Self::Y,
            Self::DpadUp,
            Self::DpadDown,
            Self::DpadLeft,
            Self::DpadRight,
            Self::L3,
            Self::R3,
        ];
        /// Defines the `BUTTONS` constant used by this module.
        ///
        ///
///  Provides the BUTTONS constant value.
        pub const BUTTONS: [Self; 8] = [
            Self::X,
            Self::Y,
            Self::DpadUp,
            Self::DpadDown,
            Self::DpadLeft,
            Self::DpadRight,
            Self::L3,
            Self::R3,
        ];
        /// Returns the button associated with the binding.
        ///
        ///
///  Gets the assigned button.
        pub fn button(self) -> u32 {
            use crate::{manual_controls::*, native_input::*};
            match self {
                Self::X => MAPPER_X,
                Self::Y => MAPPER_Y,
                Self::DpadUp => DPAD_UP,
                Self::DpadDown => crate::ledge_trap::DPAD_DOWN,
                Self::DpadLeft => DPAD_LEFT,
                Self::DpadRight => DPAD_RIGHT,
                Self::L3 => 1 << 16,
                Self::R3 => 1 << 17,
                _ => 0,
            }
        }
        /// Checks whether button.
        ///
        ///
///  Checks whether button.
        pub fn is_button(self) -> bool {
            self as usize >= Self::X as usize
        }
        /// Builds the default the index.
        ///
        ///
///  Holds the default index value.
        pub fn default_index(self, saved: usize) -> usize {
            saved
                + if self.is_button() {
                    crate::button_bindings::Builtin::default_for(self).template_offset()
                } else {
                    0
                }
        }
        /// Returns the maximum number of choices supported by the slot.
        ///
        ///
///  Gets how many choices can fit in the slot.
        pub fn max_choices(self) -> usize {
            32
        }
        /// Returns the stored recipe templates.
        ///
        ///
///  Gets the available recipe templates.
        pub fn templates(self) -> usize {
            if self.is_button() {
                5 // Normal + custom recipe + three native helpers.
            } else {
                1
            }
        }
    }
    /// A file-authored menu choice. There are no executable fighter handlers here.
    ///
///  A file-authored menu choice. There are no executable fighter handlers here.
    #[derive(Clone, Copy, Debug)]
    pub struct SavedChoice {
        /// Stores the `id` value in `SavedChoice` as `&'static str`.
        ///
        ///
///  Holds the id value used by SavedChoice.
        pub id: &'static str,
        /// Stores the `name` value in `SavedChoice` as `&'static str`.
        ///
        ///
///  Holds the name value used by SavedChoice.
        pub name: &'static str,
        /// Stores the `sequence` value in `SavedChoice` as `&'static crate::sequence_config::Sequence`.
        ///
        ///
///  Holds the sequence value used by SavedChoice.
        pub sequence: &'static crate::sequence_config::Sequence,
    }
    impl SavedChoice {
        /// Returns the file-authored value for the requested entry.
        ///
        ///
///  Gets the value read from the configuration file.
        pub(crate) fn file(
            id: &'static str,
            name: &'static str,
            sequence: &'static crate::sequence_config::Sequence,
        ) -> Self {
            Self { id, name, sequence }
        }
    }
    /// Defines the `SavedList` data structure.
    ///
    ///
///  Holds the data for SavedList.
    #[derive(Clone, Copy, Debug)]
    pub struct SavedList {
        /// Stores the `selected` value in `SavedList` as `usize`.
        ///
        ///
///  Holds the selected value used by SavedList.
        pub selected: usize,
        /// Stores the `options` value in `SavedList` as `&'static [SavedChoice]`.
        ///
        ///
///  Holds the options value used by SavedList.
        pub options: &'static [SavedChoice],
    }
    impl SavedList {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
///  Creates a new value.
        pub const fn new(options: &'static [SavedChoice]) -> Self {
            Self {
                selected: 0,
                options,
            }
        }
    }
    /// Defines the `MenuPresets` data structure.
    ///
    ///
///  Holds the data for MenuPresets.
    #[derive(Clone, Copy, Debug)]
    pub struct MenuPresets {
        /// Stores the `lists` value in `MenuPresets` as `[SavedList; PresetSlot::COUNT]`.
        ///
        ///
///  Stores the lists collection for MenuPresets.
        pub lists: [SavedList; PresetSlot::COUNT],
        /// Stores the `templates` value in `MenuPresets` as `&'static [crate::sequence_config::Template]`.
        ///
        ///
///  Holds the templates value used by MenuPresets.
        pub templates: &'static [crate::sequence_config::Template],
    }
    // Empty until startup loads the sole text catalog.
    impl Default for MenuPresets {
        /// Returns the default value for this type.
        ///
        ///
///  Returns the default value for this type.
        fn default() -> Self {
            Self::EMPTY
        }
    }
    impl MenuPresets {
        /// Defines the `EMPTY` constant used by this module.
        ///
        ///
///  Provides the EMPTY constant value.
        pub const EMPTY: Self = Self {
            lists: [SavedList::new(&[]); PresetSlot::COUNT],
            templates: &[],
        };
        /// Returns the available entries in the requested list.
        ///
        ///
///  Gets the items in the list.
        pub fn list(&self, slot: PresetSlot) -> &SavedList {
            &self.lists[slot as usize]
        }
    }
}
pub mod spacing {
    //! Display-only, qualitative spacing estimates for the compiled response list.
    //!
    //! These describe the move's useful coverage, not the response's detection
    //! radius or a guaranteed collision distance. Animation, opponent hurtboxes,
    //! height and aerial drift prevent turning a hitbox radius into a universal
    //! maximum. Keep numerical distances out until measured in world coordinates.
    //! Nothing here gates an attack or reads live fighter/opponent state.
    use super::presets::PresetSlot;
    use crate::fighter_kinds as kind;

    /// Compact display aliases only. IDs, handlers and timing stay unchanged.
    /// The UI resolves these when its cached snapshot changes, never per game frame.
    ///
///  Compact display aliases only. IDs, handlers and timing stay unchanged. The UI resolves these when its cached snapshot changes, never per game frame.
    pub fn menu_name(name: &str) -> &str {
        match name {
            "Down tilt" => "Dtilt",
            "Forward tilt" => "Ftilt",
            "Forward tilt 1" => "Ftilt 1",
            "Up tilt" => "Utilt",
            "Grounded up tilt" => "Ground Utilt",
            "Forward air" => "Fair",
            "Forward air 1" => "Fair 1",
            "Up air" => "Uair",
            "Down smash" => "Dsmash",
            "Forward smash" => "Fsmash",
            "Down tilt > shield" => "Dtilt > Shield",
            "Shield > nair" => "Shield Nair",
            "Shield > fair" => "Shield Fair",
            "Shield > fair 1" => "Shield Fair 1",
            "Shield > up air" => "Shield Uair",
            "Shield > zair" => "Shield Zair",
            "Shield > Up-B" => "Shield Up-B",
            "Shield > slingshot" => "Shield Sling",
            "Shield > trampoline" => "Shield Tramp",
            "Shield > fair (Pikmin)" => "Shield Pikmin Fair",
            "Shield > up air (Pikmin)" => "Shield Pikmin Uair",
            "Grab / rear nair" => "Grab/rear Nair",
            "Grab / rear shield" => "Grab/rear Guard",
            "Short-hop zair" => "SH Zair",
            "Short-hop dair" => "SH Dair",
            "Short-hop bite" => "SH Bite",
            "Hop nair (near)" => "Near Nair",
            "Hop fair (near)" => "Near Fair",
            "Hop fair 1 (near)" => "Near Fair 1",
            "Hop dair (near)" => "Near Dair",
            "Hop fair (Pikmin)" => "Pikmin Fair",
            "Hop dair (Pikmin)" => "Pikmin Dair",
            "Neutral-B / Inhale" => "Inhale",
            "Down-B / Stone" => "Stone",
            "Immediate down-B" => "Bomb (now)",
            "Homing missile" => "Homing Msl",
            "Super missile" => "Super Msl",
            "Homing (stay back)" => "Homing (stay)",
            "Super (stay back)" => "Super (stay)",
            "Flame fair (stay back)" => "Flame Fair (stay)",
            "Headbutt f-smash" => "Head Fsmash",
            "Belly flop (risky)" => "Flop (risky)",
            "Dair shred (risky)" => "Dair (risky)",
            "Meteor chase (risky)" => "Meteor (risky)",
            "Turnip spike (risky)" => "Turnip (risky)",
            "Tiny up-air (joke)" => "Tiny Uair (joke)",
            _ => name,
        }
    }

    /// An empty hint deliberately leaves grabs, shield and automatic roll approaches
    /// alone: their starting distance is not the move's final striking distance.
    /// Fighter kind is the resolved profile kind, so fallback profiles stay coherent.
    ///
///  An empty hint deliberately leaves grabs, shield and automatic roll approaches alone: their starting distance is not the move's final striking distance. Fighter kind is the resolved profile kind, so fallback profiles stay coherent.
    pub fn hint(fighter_kind: i32, slot: PresetSlot, choice: &str) -> &'static str {
        // Missile roll choices deliberately stay back instead of using the roll
        // approach, so retain their ranged hint even in that row.
        if matches!(
            choice,
            "homing_missile" | "super_missile" | "area_smash" | "projectile_fair"
        ) {
            return "~ranged / travel";
        }
        if slot == PresetSlot::Roll
            || matches!(choice, "shield" | "grab" | "shield_grab" | "shield_rear")
        {
            return "";
        }
        match choice {
            "nair" | "shield_nair" | "drop_nair" => nair_hint(fighter_kind),
            "down_tilt" | "down_tilt_shield" => down_tilt_hint(fighter_kind),
            "forward_tilt" | "poke" => forward_tilt_hint(fighter_kind),
            "stone" => "~close",
            "inhale" => "~close / front",
            "zair" | "shield_zair" => "~long / front",
            "bomb" => "~near ledge",
            "up_tilt" => "~close / above",
            "up_air" | "shield_up_air" => {
                if fighter_kind == kind::LITTLE_MAC {
                    "~touching / above"
                } else {
                    "~above"
                }
            }
            "forward_air" if fighter_kind == kind::ISABELLE => "~ranged / front",
            "forward_air" | "shield_fair" => "~mid / front",
            "bair" if fighter_kind == kind::YOSHI => "~close / rear",
            "slingshot" => "~ranged / front",
            "short_hop_dair" => short_hop_dair_hint(fighter_kind),
            "drop_fair" => "~front / offstage",
            "belly_flop" if slot == PresetSlot::Drop => "~below / offstage",
            "belly_flop" => "~mid",
            "party_popper" => "~close / front",
            "ghost" | "ghosts" => "~long",
            "down_smash" => "~mid / low",
            "forward_smash" | "low_forward_smash" => "~mid / front",
            "sweep" => "~mid / low",
            "shield_up_b" => "~close / above",
            // A new action without reviewed metadata should never inherit another
            // move's estimate just because it occupies the same menu row.
            _ => "~spacing?",
        }
    }

    // Per-move reach classes; a fighter outside a list takes that move's shorter default.
    /// Returns the neutral-air reference hint.
    ///
    ///
///  Gets the note about neutral air.
    fn nair_hint(fighter_kind: i32) -> &'static str {
        match fighter_kind {
            kind::LITTLE_MAC => "~touching", // Little Mac's intentionally tiny aerial.
            kind::DONKEY_KONG
            | kind::LINK
            | kind::SAMUS
            | kind::DARK_SAMUS
            | kind::BOWSER
            | kind::MARTH
            | kind::LUCINA
            | kind::GANONDORF
            | kind::ROY
            | kind::CHROM
            | kind::IKE
            | kind::CHARIZARD
            | kind::SONIC
            | kind::KING_DEDEDE
            | kind::ROB
            | kind::ROSALINA_AND_LUMA
            | kind::SHULK
            | kind::CLOUD
            | kind::CORRIN
            | kind::RIDLEY
            | kind::KING_K_ROOL => "~mid",
            _ => "~close",
        }
    }
    /// Returns the down-tilt reference hint.
    ///
    ///
///  Gets the note about down tilt.
    fn down_tilt_hint(fighter_kind: i32) -> &'static str {
        match fighter_kind {
            kind::DONKEY_KONG
            | kind::LINK
            | kind::SAMUS
            | kind::DARK_SAMUS
            | kind::YOSHI
            | kind::KIRBY
            | kind::CAPTAIN_FALCON
            | kind::MARTH
            | kind::LUCINA
            | kind::GANONDORF
            | kind::MEWTWO
            | kind::ROY
            | kind::CHROM
            | kind::IKE
            | kind::IVYSAUR
            | kind::CHARIZARD
            | kind::KING_DEDEDE
            | kind::TOON_LINK
            | kind::GRENINJA
            | kind::PALUTENA
            | kind::ROBIN
            | kind::SHULK
            | kind::CLOUD
            | kind::CORRIN
            | kind::RIDLEY
            | kind::KING_K_ROOL
            | kind::ISABELLE
            | kind::MII_SWORDFIGHTER => "~mid / low",
            _ => "~close / low",
        }
    }
    /// Returns the forward-tilt reference hint.
    ///
    ///
///  Gets the note about forward tilt.
    fn forward_tilt_hint(fighter_kind: i32) -> &'static str {
        match fighter_kind {
            kind::DONKEY_KONG
            | kind::LINK
            | kind::SAMUS
            | kind::DARK_SAMUS
            | kind::MARTH
            | kind::LUCINA
            | kind::GANONDORF
            | kind::MEWTWO
            | kind::IKE
            | kind::CHARIZARD
            | kind::SHULK
            | kind::CLOUD
            | kind::CORRIN
            | kind::RIDLEY
            | kind::SIMON
            | kind::RICHTER
            | kind::KING_K_ROOL
            | kind::MII_SWORDFIGHTER => "~long",
            _ => "~mid",
        }
    }
    /// Returns the short-hop down-air reference hint.
    ///
    ///
///  Gets the note about short-hop down air.
    fn short_hop_dair_hint(fighter_kind: i32) -> &'static str {
        match fighter_kind {
            kind::DARK_SAMUS
            | kind::NESS
            | kind::JIGGLYPUFF
            | kind::PEACH
            | kind::DAISY
            | kind::MARTH
            | kind::LUCINA
            | kind::YOUNG_LINK
            | kind::GANONDORF
            | kind::MEWTWO
            | kind::ROY
            | kind::CHROM
            | kind::MR_GAME_AND_WATCH
            | kind::META_KNIGHT
            | kind::PIT
            | kind::DARK_PIT
            | kind::ZERO_SUIT_SAMUS
            | kind::WARIO
            | kind::SNAKE
            | kind::IKE
            | kind::SQUIRTLE
            | kind::IVYSAUR
            | kind::CHARIZARD
            | kind::DIDDY_KONG
            | kind::LUCAS
            | kind::SONIC
            | kind::KING_DEDEDE
            | kind::OLIMAR
            | kind::LUCARIO
            | kind::ROB
            | kind::TOON_LINK
            | kind::WOLF
            | kind::VILLAGER
            | kind::MEGA_MAN
            | kind::WII_FIT_TRAINER
            | kind::ROSALINA_AND_LUMA
            | kind::LITTLE_MAC
            | kind::GRENINJA
            | kind::PALUTENA
            | kind::PAC_MAN
            | kind::ROBIN
            | kind::SHULK
            | kind::BOWSER_JR
            | kind::DUCK_HUNT
            | kind::RYU
            | kind::KEN
            | kind::CLOUD
            | kind::CORRIN
            | kind::BAYONETTA
            | kind::INKLING
            | kind::RIDLEY
            | kind::SIMON
            | kind::RICHTER
            | kind::KING_K_ROOL
            | kind::ISABELLE
            | kind::INCINEROAR
            | kind::SQUIRTLE_FIGHTER
            | kind::IVYSAUR_FIGHTER
            | kind::CHARIZARD_FIGHTER
            | kind::POKEMON_TRAINER
            | kind::PYRA_FIGHTER
            | kind::MYTHRA_FIGHTER
            | kind::ICE_CLIMBERS_POPO
            | kind::ICE_CLIMBERS_NANA => "~below / near edge",
            _ => "~below / offstage",
        }
    }
}

// Keep the historical per-fighter module paths while sharing their construction.
macro_rules! fighter_profiles {
    ($( $module:ident: $kind:ident, $name:literal $(, $constant:ident)?; )*) => {
        $(
            pub mod $module {
                $(pub const $constant: i32 = crate::fighter_kinds::$kind;)?

                /// Returns the profile associated with the fighter identity.
                ///
                ///
///  Gets the fighter’s profile.
                pub fn profile() -> super::FighterProfile {
                    let kind = crate::fighter_kinds::$kind;
                    super::FighterProfile {
                        name: $name,
                        fighter_kind: kind,
                        grab: super::FighterGrabData::from_kind(kind),
                        menu_options: super::presets::MenuPresets::EMPTY,
                    }
                }
            }
        )*
        /// Defines the `BUILDERS` constant used by this module.
        ///
        ///
///  Provides the BUILDERS constant value.
        const BUILDERS: &[fn() -> FighterProfile] = &[$($module::profile),*];
    };
}

fighter_profiles! {
    kirby: KIRBY, "kirby", FIGHTER_KIND;
    luigi: LUIGI, "luigi";
    captain: CAPTAIN_FALCON, "captain_falcon";
    mario: MARIO, "mario";
    donkey: DONKEY_KONG, "donkey_kong";
    link: LINK, "link";
    samus: SAMUS, "samus";
    yoshi: YOSHI, "yoshi";
    fox: FOX, "fox";
    pikachu: PIKACHU, "pikachu";
    sonic: SONIC, "sonic", FIGHTER_KIND;
    krool: KING_K_ROOL, "krool", FIGHTER_KIND;
    isabelle: ISABELLE, "isabelle", FIGHTER_KIND;
    pacman: PAC_MAN, "pacman", FIGHTER_KIND;
    littlemac: LITTLE_MAC, "littlemac", FIGHTER_KIND;
    dark_samus: DARK_SAMUS, "dark_samus";
    ness: NESS, "ness";
    jigglypuff: JIGGLYPUFF, "jigglypuff";
    peach: PEACH, "peach";
    daisy: DAISY, "daisy";
    bowser: BOWSER, "bowser";
    sheik: SHEIK, "sheik";
    zelda: ZELDA, "zelda";
    dr_mario: DR_MARIO, "dr_mario";
    pichu: PICHU, "pichu";
    falco: FALCO, "falco";
    marth: MARTH, "marth";
    lucina: LUCINA, "lucina";
    young_link: YOUNG_LINK, "young_link";
    ganondorf: GANONDORF, "ganondorf";
    mewtwo: MEWTWO, "mewtwo";
    roy: ROY, "roy";
    chrom: CHROM, "chrom";
    mr_game_and_watch: MR_GAME_AND_WATCH, "mr_game_and_watch";
    meta_knight: META_KNIGHT, "meta_knight";
    pit: PIT, "pit";
    dark_pit: DARK_PIT, "dark_pit";
    zero_suit_samus: ZERO_SUIT_SAMUS, "zero_suit_samus";
    wario: WARIO, "wario";
    snake: SNAKE, "snake";
    ike: IKE, "ike";
    squirtle: SQUIRTLE, "squirtle";
    ivysaur: IVYSAUR, "ivysaur";
    charizard: CHARIZARD, "charizard";
    diddy_kong: DIDDY_KONG, "diddy_kong";
    lucas: LUCAS, "lucas";
    king_dedede: KING_DEDEDE, "king_dedede";
    olimar: OLIMAR, "olimar";
    lucario: LUCARIO, "lucario";
    rob: ROB, "rob";
    toon_link: TOON_LINK, "toon_link";
    wolf: WOLF, "wolf";
    villager: VILLAGER, "villager";
    mega_man: MEGA_MAN, "mega_man";
    wii_fit_trainer: WII_FIT_TRAINER, "wii_fit_trainer";
    rosalina_and_luma: ROSALINA_AND_LUMA, "rosalina_and_luma";
    greninja: GRENINJA, "greninja";
    palutena: PALUTENA, "palutena";
    robin: ROBIN, "robin";
    shulk: SHULK, "shulk";
    bowser_jr: BOWSER_JR, "bowser_jr";
    duck_hunt: DUCK_HUNT, "duck_hunt";
    ryu: RYU, "ryu";
    ken: KEN, "ken";
    cloud: CLOUD, "cloud";
    corrin: CORRIN, "corrin";
    bayonetta: BAYONETTA, "bayonetta";
    inkling: INKLING, "inkling";
    ridley: RIDLEY, "ridley";
    simon: SIMON, "simon";
    richter: RICHTER, "richter";
    incineroar: INCINEROAR, "incineroar";
    mii_brawler: MII_BRAWLER, "mii_brawler";
    mii_swordfighter: MII_SWORDFIGHTER, "mii_swordfighter";
    mii_gunner: MII_GUNNER, "mii_gunner";
    ice_climbers: ICE_CLIMBERS_POPO, "ice_climbers";
    ice_climbers_nana: ICE_CLIMBERS_NANA, "ice_climbers_nana";
    pyra: PYRA, "pyra", FIGHTER_KIND;
    mythra: MYTHRA, "mythra", FIGHTER_KIND;
}

use crate::settings::AssistSettings;
use std::collections::BTreeMap;

/// Defines the `GrabTiming` data structure.
///
///
///  Holds the data for GrabTiming.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabTiming {
    /// Zero means the canonical source has no simple numeric reach.
    ///
///  Zero means the canonical source has no simple numeric reach.
    pub standing_range: f32,
    /// Stores the `standing_startup` value in `GrabTiming` as `u8`.
    ///
    ///
///  Holds the standing startup value used by GrabTiming.
    pub standing_startup: u8,
    /// Stores the `dash_startup` value in `GrabTiming` as `u8`.
    ///
    ///
///  Holds the dash startup value used by GrabTiming.
    pub dash_startup: u8,
    /// Stores the `pivot_startup` value in `GrabTiming` as `u8`.
    ///
    ///
///  Holds the pivot startup value used by GrabTiming.
    pub pivot_startup: u8,
    /// Stores the `shield_startup` value in `GrabTiming` as `u8`.
    ///
    ///
///  Holds the shield startup value used by GrabTiming.
    pub shield_startup: u8,
    /// Stores the `standing_active_frames` value in `GrabTiming` as `u8`.
    ///
    ///
///  Sets the standing active frames timing for GrabTiming.
    pub standing_active_frames: u8,
}

/// Grab timing and reach values derived from a fighter's canonical data.
///
///  Grab timing and reach values derived from a fighter's canonical data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FighterGrabData {
    /// Stores the `grab` value in `FighterGrabData` as `GrabTiming`.
    ///
    ///
///  Holds the grab value used by FighterGrabData.
    pub grab: GrabTiming,
}
impl FighterGrabData {
    /// Resolve simple grab facts from this fighter's canonical data row.
    ///
///  Resolve simple grab facts from this fighter's canonical data row.
    pub fn from_kind(kind: i32) -> Self {
        let grab = |status| crate::fighter_data::ordinary_grab(kind, status);
        let standing = grab(crate::native_states::CATCH);
        let shield = crate::fighter_data::fighter(kind)
            .and_then(|fighter| fighter.stat("outofshield.grabpostshieldstun"))
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|frame| (1..=120).contains(frame));
        Self {
            grab: GrabTiming {
                standing_range: standing.and_then(|facts| facts.reach).unwrap_or(0.0),
                standing_startup: standing.map_or(0, |facts| facts.first),
                dash_startup: grab(crate::native_states::CATCH_DASH).map_or(0, |facts| facts.first),
                pivot_startup: grab(crate::native_states::CATCH_TURN)
                    .map_or(0, |facts| facts.first),
                shield_startup: shield.unwrap_or(0),
                standing_active_frames: standing.map_or(0, |facts| facts.last - facts.first + 1),
            },
        }
    }
}
/// Response switches. Move selection for normal get-up uses GetupAttack.
/// Ledge-drop recipes are selected separately through AssistSettings.
///
///  Response switches. Move selection for normal get-up uses GetupAttack. Ledge-drop recipes are selected separately through AssistSettings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LedgeResponses {
    /// Stores the `neutral_getup` value in `LedgeResponses` as `bool`.
    ///
    ///
///  Holds the neutral getup value used by LedgeResponses.
    pub neutral_getup: bool,
    /// Stores the `jump_getup` value in `LedgeResponses` as `bool`.
    ///
    ///
///  Holds the jump getup value used by LedgeResponses.
    pub jump_getup: bool,
    /// Stores the `roll_getup` value in `LedgeResponses` as `bool`.
    ///
    ///
///  Holds the roll getup value used by LedgeResponses.
    pub roll_getup: bool,
    /// Stores the `attack_getup` value in `LedgeResponses` as `bool`.
    ///
    ///
///  Holds the attack getup value used by LedgeResponses.
    pub attack_getup: bool,
    /// Stores the `rear_block_nair` value in `LedgeResponses` as `bool`.
    ///
    ///
///  Holds the rear block nair value used by LedgeResponses.
    pub rear_block_nair: bool,
}
impl Default for LedgeResponses {
    /// Returns the default value for this type.
    ///
    ///
///  Returns the default value for this type.
    fn default() -> Self {
        Self {
            neutral_getup: true,
            jump_getup: true,
            roll_getup: true,
            attack_getup: true,
            rear_block_nair: true,
        }
    }
}

/// Defines the `FighterProfile` data structure.
///
///
///  Holds the data for FighterProfile.
#[derive(Clone, Debug)]
pub struct FighterProfile {
    /// Stores the `name` value in `FighterProfile` as `&'static str`.
    ///
    ///
///  Holds the name value used by FighterProfile.
    pub name: &'static str,
    /// Stores the `fighter_kind` value in `FighterProfile` as `i32`.
    ///
    ///
///  Holds the fighter kind value used by FighterProfile.
    pub fighter_kind: i32,
    /// Stores the `grab` value in `FighterProfile` as `FighterGrabData`.
    ///
    ///
///  Holds the grab value used by FighterProfile.
    pub grab: FighterGrabData,
    /// Stores the `menu_options` value in `FighterProfile` as `presets::MenuPresets`.
    ///
    ///
///  Holds the menu options value used by FighterProfile.
    pub menu_options: presets::MenuPresets,
}
impl FighterProfile {
    /// Full-roster facts are available regardless of this fighter's match role.
    ///
///  Full-roster facts are available regardless of this fighter's match role.
    pub fn fighter_data(&self) -> Option<&'static crate::fighter_data::FighterData> {
        crate::fighter_data::fighter(self.fighter_kind)
    }
    /// Applies the supplied configuration or action.
    ///
    ///
///  Makes the supplied change.
    pub fn apply(&self, mut settings: AssistSettings) -> AssistSettings {
        settings.implementation =
            crate::ledge_trap::ResponseImplementation::for_kind(self.fighter_kind);
        settings.fighter_kind = self.fighter_kind;
        // Identity/reference facts do not supply a ledge option fallback.
        // A startup-empty list may acquire a live menu recipe later. Keep the
        // tool available for known fighters; missing selected timelines still
        // produce no response and never fall back to compiled moves.
        settings.enabled &= settings.implementation.is_some();
        settings.fighter = self.grab;
        // A fighter profile never enables a globally disabled ledge tool.
        settings
    }
}

/// Defines the `FighterProfiles` data structure.
///
///
///  Holds the data for FighterProfiles.
#[derive(Clone, Debug)]
pub struct FighterProfiles {
    /// Stores the `profiles` value in `FighterProfiles` as `BTreeMap<i32`.
    ///
    ///
///  Holds the profiles value used by FighterProfiles.
    profiles: BTreeMap<i32, FighterProfile>,
}
impl Default for FighterProfiles {
    /// Returns the default value for this type.
    ///
    ///
///  Returns the default value for this type.
    fn default() -> Self {
        // Keep the constructor table in read-only storage and build one profile
        // at a time. A [FighterProfile; 77] temporary exceeded the native 64 KiB
        // loader stack once per-fighter menu lists grew to include D-pad slots.
        let mut profiles = BTreeMap::new();
        for build in BUILDERS {
            let profile = build();
            profiles.insert(profile.fighter_kind, profile);
        }
        Self { profiles }
    }
}

impl FighterProfiles {
    /// Constructs a value from the test file.
    ///
    ///
///  Constructs a value from the test file.
    #[cfg(test)]
    pub(crate) fn from_test_file() -> Self {
        let catalog = crate::sequence_config::Catalog::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
        )))
        .unwrap()
        .for_build(cfg!(feature = "diagnostic-logging"));
        let mut profiles = Self::default();
        profiles.add_sequence_choices(&catalog).unwrap();
        profiles
    }
    /// Share inert Training templates without expanding fighter-specific menus.
    /// Runtime uses owned selected-fighter records instead of leaking all tapes.
    ///
///  Share inert Training templates without expanding fighter-specific menus. Runtime uses owned selected-fighter records instead of leaking all tapes.
    pub fn add_sequence_templates(&mut self, templates: &[crate::sequence_config::Template]) {
        if templates.is_empty() {
            return;
        }
        let templates: &'static [crate::sequence_config::Template] =
            Box::leak(templates.to_vec().into_boxed_slice());
        for profile in self.profiles.values_mut() {
            profile.menu_options.templates = templates;
        }
    }
    /// Startup only. Freeze the file-authored menus into process-lifetime
    /// storage so all existing borrowed catalog consumers stay allocation-free.
    /// The file/section/list caps bound this retained memory. Never call per frame.
    ///
///  Startup only. Freeze the file-authored menus into process-lifetime storage so all existing borrowed catalog consumers stay allocation-free. The file/section/list caps bound this retained memory. Never call per frame.
    pub fn add_sequence_choices(
        &mut self,
        catalog: &crate::sequence_config::Catalog,
    ) -> Result<(), String> {
        catalog.validate(self)?;
        // One shared startup allocation, never duplicated per fighter or frame.
        let templates: &'static [crate::sequence_config::Template] =
            Box::leak(catalog.templates.clone().into_boxed_slice());
        for profile in self.profiles.values_mut() {
            profile.menu_options.templates = templates;
            for slot in presets::PresetSlot::ALL {
                let entries = catalog.entries().iter().filter(|e| {
                    e.fighter_kind == profile.fighter_kind
                        && e.slot == slot
                        && e.sequence.mode.is_none()
                });
                let list = &mut profile.menu_options.lists[slot as usize];
                let mut options = Vec::new();
                let mut selected = None;
                let mut first_regular = None;
                for entry in entries {
                    let id = Box::leak(entry.choice.clone().into_boxed_str());
                    let name = Box::leak(
                        entry
                            .name
                            .clone()
                            .unwrap_or_else(|| entry.choice.replace('_', " "))
                            .into_boxed_str(),
                    );
                    if !entry.sequence.reaction.active() {
                        first_regular.get_or_insert(options.len());
                        if entry.default {
                            selected = Some(options.len());
                        }
                    }
                    let sequence = Box::leak(Box::new(entry.sequence));
                    options.push(presets::SavedChoice::file(id, name, sequence));
                }
                list.selected = selected.unwrap_or_else(|| {
                    if slot.is_button() {
                        slot.default_index(options.len())
                    } else {
                        first_regular.unwrap_or(options.len())
                    }
                });
                list.options = Box::leak(options.into_boxed_slice());
            }
        }
        self.validate()
    }
    /// Check programmer-supplied catalog edits once before installing hooks.
    ///
///  Check programmer-supplied catalog edits once before installing hooks.
    pub fn validate(&self) -> Result<(), String> {
        if !self.profiles.contains_key(&6) {
            return Err("Kirby identity/fighter-data profile is missing".into());
        }
        for (&kind, profile) in &self.profiles {
            if profile.fighter_kind != kind
                || profile.fighter_data().is_none_or(|data| data.kind != kind)
            {
                return Err(format!(
                    "missing/mismatched fighter reference {}",
                    profile.name
                ));
            }
            if crate::ledge_trap::ResponseImplementation::for_kind(profile.fighter_kind).is_none() {
                return Err(format!("unregistered compiled fighter {}", profile.name));
            }
            let data = profile.grab;
            if data.grab.standing_range != 0.0
                && (!data.grab.standing_range.is_finite()
                    || !(1.0..=60.0).contains(&data.grab.standing_range))
                || [
                    data.grab.standing_startup,
                    data.grab.dash_startup,
                    data.grab.pivot_startup,
                    data.grab.shield_startup,
                    data.grab.standing_active_frames,
                ]
                .iter()
                .any(|frame| *frame != 0 && !(1..=120).contains(frame))
            {
                return Err(format!("invalid fighter grab data {}", profile.name));
            }
            for slot in presets::PresetSlot::ALL {
                let list = profile.menu_options.list(slot);
                // Button lists append a permanent Normal button choice after
                // their file entries. No explicit default selects that choice.
                let selected_is_valid = list.selected < list.options.len()
                    || slot.is_button() && list.selected < list.options.len() + slot.templates()
                    || list.selected == list.options.len()
                        && (slot.is_button()
                            || list.options.iter().all(|c| c.sequence.reaction.active()));
                if list
                    .options
                    .iter()
                    .filter(|c| !c.sequence.reaction.active())
                    .count()
                    > slot.max_choices()
                    || !selected_is_valid
                {
                    return Err(format!(
                        "invalid file response list {} / {slot:?}",
                        profile.name
                    ));
                }
                let mut ids = std::collections::BTreeSet::new();
                for choice in list.options {
                    if choice.id.is_empty()
                        || choice.name.is_empty()
                        || choice.name.contains('\0')
                        || !ids.insert(choice.id)
                    {
                        return Err(format!(
                            "invalid file response entry {} / {}",
                            profile.name, choice.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Returns the profile associated with the fighter identity.
    ///
    ///
///  Gets the fighter’s profile.
    pub fn profile(&self, fighter_kind: i32) -> Option<&FighterProfile> {
        // Native playable kinds are 0..FIGHTER_KIND_TERM (0x5e), exclusive.
        // Missing/invalid identities are inert; never substitute another fighter's
        // option catalog.
        if !(0..crate::fighter_kinds::PLAYABLE_KIND_COUNT).contains(&fighter_kind) {
            return None;
        }
        self.profiles.get(&fighter_kind)
    }

    /// Use only the requested fighter identity and its loaded file catalog.
    /// None still covers startup before a valid fighter is observed.
    ///
///  Use only the requested fighter identity and its loaded file catalog. None still covers startup before a valid fighter is observed.
    pub fn settings_for_fighter(
        &self,
        fighter_kind: Option<i32>,
        mut base: AssistSettings,
    ) -> AssistSettings {
        // A fallback may receive a previously resolved live snapshot. Selected
        // indices and tapes belong to that identity, never the incoming one.
        if fighter_kind != Some(base.fighter_kind) {
            base.response_choices.fill(None);
            base.sequence_overrides.fill(None);
        }
        match fighter_kind.and_then(|kind| self.profile(kind)) {
            Some(profile) => profile.apply(base),
            None => AssistSettings {
                enabled: false,
                implementation: None,
                response_choices: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
                sequence_overrides: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
                ..base
            },
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/fighter_profiles.rs"]
mod tests;
