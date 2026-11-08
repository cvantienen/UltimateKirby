//! Fighter identities and read-only reference facts. Ledge options and input
//! schedules exist only in the startup-loaded sequences.txt catalog.
pub mod presets;
pub mod spacing;

// Keep the historical per-fighter module paths while sharing their construction.
macro_rules! fighter_profiles {
    ($( $module:ident: $kind:ident, $name:literal $(, $constant:ident)?; )*) => {
        $(
            pub mod $module {
                $(pub const $constant: i32 = crate::fighter_kinds::$kind;)?

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabTiming {
    /// Zero means the canonical source has no simple numeric reach.
    pub standing_range: f32,
    pub standing_startup: u8,
    pub dash_startup: u8,
    pub pivot_startup: u8,
    pub shield_startup: u8,
    pub standing_active_frames: u8,
}

/// Grab timing and reach values derived from a fighter's canonical data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FighterGrabData {
    pub grab: GrabTiming,
}
impl FighterGrabData {
    /// Resolve simple grab facts from this fighter's canonical data row.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LedgeResponses {
    pub neutral_getup: bool,
    pub jump_getup: bool,
    pub roll_getup: bool,
    pub attack_getup: bool,
    pub rear_block_nair: bool,
}
impl Default for LedgeResponses {
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

#[derive(Clone, Debug)]
pub struct FighterProfile {
    pub name: &'static str,
    pub fighter_kind: i32,
    pub grab: FighterGrabData,
    pub menu_options: presets::MenuPresets,
}
impl FighterProfile {
    /// Full-roster facts are available regardless of this fighter's match role.
    pub fn fighter_data(&self) -> Option<&'static crate::fighter_data::FighterData> {
        crate::fighter_data::fighter(self.fighter_kind)
    }
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

#[derive(Clone, Debug)]
pub struct FighterProfiles {
    profiles: BTreeMap<i32, FighterProfile>,
}
impl Default for FighterProfiles {
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
    /// Startup only. Freeze the file-authored menus into process-lifetime
    /// storage so all existing borrowed catalog consumers stay allocation-free.
    /// The file/section/list caps bound this retained memory. Never call per frame.
    /// Share inert Training templates without expanding fighter-specific menus.
    /// Runtime uses owned selected-fighter records instead of leaking all tapes.
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
#[path = "../tests/unit/fighter_profiles/tests.rs"]
mod tests;
