//! Fighter identities and read-only reference facts. Ledge options and input
//! schedules exist only in the startup-loaded sequences.txt catalog.
pub mod bayonetta;
pub mod bowser;
pub mod bowser_jr;
pub mod captain;
pub mod charizard;
pub mod chrom;
pub mod cloud;
pub mod corrin;
pub mod daisy;
pub mod dark_pit;
pub mod dark_samus;
pub mod diddy_kong;
pub mod donkey;
pub mod dr_mario;
pub mod duck_hunt;
pub mod falco;
pub mod fox;
pub mod ganondorf;
pub mod greninja;
pub mod ice_climbers;
pub mod ice_climbers_nana;
pub mod ike;
pub mod incineroar;
pub mod inkling;
pub mod isabelle;
pub mod ivysaur;
pub mod jigglypuff;
pub mod ken;
pub mod king_dedede;
pub mod kirby;
pub mod krool;
pub mod link;
pub mod littlemac;
pub mod lucario;
pub mod lucas;
pub mod lucina;
pub mod luigi;
pub mod mario;
pub mod marth;
pub mod mega_man;
pub mod meta_knight;
pub mod mewtwo;
pub mod mii_brawler;
pub mod mii_gunner;
pub mod mii_swordfighter;
pub mod mr_game_and_watch;
pub mod ness;
pub mod olimar;
pub mod pacman;
pub mod palutena;
pub mod peach;
pub mod pichu;
pub mod pikachu;
pub mod pit;
pub mod pyra;
pub mod mythra;
pub mod presets;
pub mod richter;
pub mod ridley;
pub mod rob;
pub mod robin;
pub mod rosalina_and_luma;
pub mod roy;
pub mod ryu;
pub mod samus;
pub mod sheik;
pub mod shulk;
pub mod simon;
pub mod snake;
pub mod sonic;
pub mod spacing;
pub mod squirtle;
pub mod toon_link;
pub mod villager;
pub mod wario;
pub mod wii_fit_trainer;
pub mod wolf;
pub mod yoshi;
pub mod young_link;
pub mod zelda;
pub mod zero_suit_samus;

use crate::ledge_trap::LedgeSettings;
use std::collections::BTreeMap;

/// Inclusive active animation frames, before jump squat or input delay is added.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveFrames {
    pub first: u8,
    pub last: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabTiming {
    /// Maximum standing-grab reach in stage units; not a guaranteed hit radius.
    pub standing_range: f32,
    pub standing: u8,
    pub dash: u8,
    /// Legacy field name: pivot-grab startup; shield startup is separate below.
    pub turn_or_shield: u8,
    pub shield: u8,
    pub active_frames: u8,
}
impl GrabTiming {
    /// Aim inside reach, rather than running onto the opponent's root. Only a
    /// fraction of maximum extension is used: tether/tongue reach changes during
    /// the animation, and the reference maximum is not an early-frame hitbox.
    pub fn roll_standoff(self) -> f32 {
        (self.standing_range * 0.35).clamp(3.0, 9.0)
    }
    /// Conservative usable area for roll-grab approaches. This is
    /// a tunable fraction of maximum extension, not an animated hitbox model.
    pub fn roll_usable_reach(self) -> f32 {
        (self.standing_range * 0.6).clamp(3.0, 18.0)
    }

    /// These status IDs describe shared game states, not fighter-specific data.
    pub fn startup(self, status: i32) -> u8 {
        match status {
            0x3 | 0x4 => self.dash,
            0x7 | 0x8 => self.turn_or_shield,
            0x1b | 0x1c | 0x1e => self.shield,
            _ => self.standing,
        }
    }
}

/// Fighter-dependent numbers used by the existing ledge input recipes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeFighterData {
    pub neutral_air: MoveFrames,
    pub down_tilt: MoveFrames,
    pub forward_tilt: MoveFrames,
    pub down_smash: MoveFrames,
    /// Separate rear sweep: never count the gap between two hits as active.
    pub down_smash_back: Option<MoveFrames>,
    /// Grounded down-special startup; absent until a profile supplies it.
    pub down_special_startup: Option<u8>,
    /// Grounded up-special startup for registered shield-punish recipes.
    pub up_special_startup: Option<u8>,
    pub grab: GrabTiming,
    pub common_roll_distance: f32,
}
impl LedgeFighterData {
    pub fn down_smash_toward(self, dx: f32, facing: f32) -> MoveFrames {
        if dx.is_finite() && facing.is_finite() && dx * facing < -0.5 {
            self.down_smash_back.unwrap_or(self.down_smash)
        } else {
            self.down_smash
        }
    }
}

/// Response switches. Move selection for normal get-up uses GetupAttack.
/// Ledge-drop recipes are selected separately through LedgeSettings.
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
    pub ledge: LedgeFighterData,
    pub menu_options: presets::MenuPresets,
}
impl FighterProfile {
    /// Same compiled facts used for opponent recognition, without file loading.
    pub fn reference(&self) -> Option<&'static crate::opponent_data::Reference> {
        crate::opponent_data::profile(self.fighter_kind)
    }
    /// Startup/parser only: authoritative simple grab facts come from this
    /// fighter's reference row before saved overrides are applied. Missing or
    /// compound data keeps its authored same-fighter fact, never Kirby's.
    fn with_reference_grab_facts(mut self) -> Self {
        if let Some(g) = crate::opponent_data::ordinary_grab(self.fighter_kind, 0x37) {
            self.ledge.grab.standing_range = g.reach;
            self.ledge.grab.standing = g.first;
            self.ledge.grab.active_frames = g.last - g.first + 1;
        }
        if let Some(g) = crate::opponent_data::ordinary_grab(self.fighter_kind, 0x39) {
            self.ledge.grab.dash = g.first;
        }
        if let Some(g) = crate::opponent_data::ordinary_grab(self.fighter_kind, 0x3b) {
            self.ledge.grab.turn_or_shield = g.first;
        }
        if let Some(frame) = self.reference()
            .and_then(|r| r.stat("outofshield.grabpostshieldstun"))
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|f| (1..=120).contains(f))
        {
            self.ledge.grab.shield = frame;
        }
        self
    }
    pub fn apply(&self, mut settings: LedgeSettings) -> LedgeSettings {
        settings.implementation =
            crate::ledge_trap::ResponseImplementation::for_kind(self.fighter_kind);
        settings.fighter_kind = self.fighter_kind;
        // Identity/reference facts do not supply a ledge option fallback.
        // A startup-empty list may acquire a live menu recipe later. Keep the
        // tool available for known fighters; missing selected timelines still
        // produce no response and never fall back to compiled moves.
        settings.enabled &= settings.implementation.is_some();
        settings.fighter = self.ledge;
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
        const BUILDERS: &[fn() -> FighterProfile] = &[
            kirby::profile,
            luigi::profile,
            captain::profile,
            mario::profile,
            donkey::profile,
            link::profile,
            samus::profile,
            yoshi::profile,
            fox::profile,
            pikachu::profile,
            sonic::profile,
            krool::profile,
            isabelle::profile,
            pacman::profile,
            littlemac::profile,
            dark_samus::profile,
            ness::profile,
            jigglypuff::profile,
            peach::profile,
            daisy::profile,
            bowser::profile,
            sheik::profile,
            zelda::profile,
            dr_mario::profile,
            pichu::profile,
            falco::profile,
            marth::profile,
            lucina::profile,
            young_link::profile,
            ganondorf::profile,
            mewtwo::profile,
            roy::profile,
            chrom::profile,
            mr_game_and_watch::profile,
            meta_knight::profile,
            pit::profile,
            dark_pit::profile,
            zero_suit_samus::profile,
            wario::profile,
            snake::profile,
            ike::profile,
            squirtle::profile,
            ivysaur::profile,
            charizard::profile,
            diddy_kong::profile,
            lucas::profile,
            king_dedede::profile,
            olimar::profile,
            lucario::profile,
            rob::profile,
            toon_link::profile,
            wolf::profile,
            villager::profile,
            mega_man::profile,
            wii_fit_trainer::profile,
            rosalina_and_luma::profile,
            greninja::profile,
            palutena::profile,
            robin::profile,
            shulk::profile,
            bowser_jr::profile,
            duck_hunt::profile,
            ryu::profile,
            ken::profile,
            cloud::profile,
            corrin::profile,
            bayonetta::profile,
            inkling::profile,
            ridley::profile,
            simon::profile,
            richter::profile,
            incineroar::profile,
            mii_brawler::profile,
            mii_swordfighter::profile,
            mii_gunner::profile,
            ice_climbers::profile,
            ice_climbers_nana::profile,
            pyra::profile,
            mythra::profile,
        ];
        let mut profiles = BTreeMap::new();
        for build in BUILDERS {
            let profile = build().with_reference_grab_facts();
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
        if templates.is_empty() {return;}
        let templates: &'static [crate::sequence_config::Template]=Box::leak(templates.to_vec().into_boxed_slice());
        for profile in self.profiles.values_mut() {profile.menu_options.templates=templates;}
    }
    pub fn add_sequence_choices(
        &mut self,
        catalog: &crate::sequence_config::Catalog,
    ) -> Result<(), String> {
        catalog.validate(self)?;
        // One shared startup allocation, never duplicated per fighter or frame.
        let templates: &'static [crate::sequence_config::Template]=Box::leak(catalog.templates.clone().into_boxed_slice());
        for profile in self.profiles.values_mut() {
            profile.menu_options.templates=templates;
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
                        if entry.default { selected = Some(options.len()); }
                    }
                    let sequence = Box::leak(Box::new(entry.sequence));
                    options.push(presets::SavedChoice::file(id, name, sequence));
                }
                list.selected =
                    selected.unwrap_or_else(|| if slot.is_button() { slot.default_index(options.len()) } else { first_regular.unwrap_or(options.len()) });
                list.options = Box::leak(options.into_boxed_slice());
            }
        }
        self.validate()
    }
    /// Check programmer-supplied catalog edits once before installing hooks.
    pub fn validate(&self) -> Result<(), String> {
        if !self.profiles.contains_key(&6) {
            return Err("Kirby identity/reference profile is missing".into());
        }
        for (&kind, profile) in &self.profiles {
            if profile.fighter_kind != kind || profile.reference().is_none_or(|r| r.kind != kind) {
                return Err(format!("missing/mismatched fighter reference {}", profile.name));
            }
            if crate::ledge_trap::ResponseImplementation::for_kind(profile.fighter_kind).is_none() {
                return Err(format!("unregistered compiled fighter {}", profile.name));
            }
            let data = profile.ledge;
            if data
                .up_special_startup
                .is_some_and(|n| !(1..=120).contains(&n))
            {
                return Err(format!("invalid up-special timing {}", profile.name));
            }
            if !data.grab.standing_range.is_finite()
                || !(1.0..=60.0).contains(&data.grab.standing_range)
                || data
                    .down_smash_back
                    .is_some_and(|f| f.first == 0 || f.first > f.last || f.last > 120)
                || [
                    data.grab.standing,
                    data.grab.dash,
                    data.grab.turn_or_shield,
                    data.grab.shield,
                    data.grab.active_frames,
                ]
                .iter()
                .any(|f| !(1..=120).contains(f))
            {
                return Err(format!("invalid rear-smash/grab timing {}", profile.name));
            }
            if [
                data.neutral_air,
                data.down_tilt,
                data.forward_tilt,
                data.down_smash,
            ]
            .iter()
            .any(|frames| frames.first == 0 || frames.first > frames.last || frames.last > 120)
                || data.neutral_air.last > 117
                || data
                    .down_special_startup
                    .is_some_and(|startup| !(1..=120).contains(&startup))
                || !data.common_roll_distance.is_finite()
                || !(2.0..=100.0).contains(&data.common_roll_distance)
            {
                return Err(format!("invalid compiled move timing {}", profile.name));
            }
            for slot in presets::PresetSlot::ALL {
                let list = profile.menu_options.list(slot);
                // Button lists append a permanent Normal button choice after
                // their file entries. No explicit default selects that choice.
                let selected_is_valid = list.selected < list.options.len()
                    || slot.is_button() && list.selected < list.options.len() + slot.templates()
                    || list.selected == list.options.len()
                        && (slot.is_button() || list.options.iter().all(|c| c.sequence.reaction.active()));
                if list.options.iter().filter(|c| !c.sequence.reaction.active()).count() > slot.max_choices() || !selected_is_valid {
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
        if !(0..0x5e).contains(&fighter_kind) {
            return None;
        }
        self.profiles.get(&fighter_kind)
    }

    /// Use only the requested fighter identity and its loaded file catalog.
    /// None still covers startup before a valid fighter is observed.
    pub fn ledge_settings(&self, fighter_kind: Option<i32>, mut base: LedgeSettings) -> LedgeSettings {
        // A fallback may receive a previously resolved live snapshot. Selected
        // indices and tapes belong to that identity, never the incoming one.
        if fighter_kind != Some(base.fighter_kind) {
            base.response_choices.fill(None);
            base.sequence_overrides.fill(None);
        }
        match fighter_kind.and_then(|kind| self.profile(kind)) {
            Some(profile) => profile.apply(base),
            None => LedgeSettings {
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
mod tests;
