//! Startup and explicit-save parser for named SD-card ledge choices and recipes.
//!
//! Parsing may allocate, but live execution copies one fixed-size recipe into
//! `LedgeSettings`; no strings, files, maps or allocations are touched in the
//! frame path.
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
pub mod persist;
pub mod layout;
mod library;
pub use library::SequenceLibrary;

pub const MAX_ACTIONS: usize = 32;
pub const MAX_FILE_BYTES: u64 = 512 * 1024;
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
            Self::Opponent => "Opponent", Self::AwayFromOpponent => "Away from opponent",
            Self::Ledge => "Ledge (legacy)", Self::Inward => "Inward", Self::Outward => "Outward",
            Self::Facing => "Facing", Self::Backward => "Backward",
            Self::SavedLedge => "Toward saved ledge", Self::AwayFromLedge => "Away from saved ledge",
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
pub enum InputReference { #[default] Screen, Facing, Opponent }
impl InputReference {
    pub const ALL:[Self;3]=[Self::Screen,Self::Facing,Self::Opponent];
    pub fn token(self)->&'static str {match self {Self::Screen=>"screen",Self::Facing=>"facing",Self::Opponent=>"opponent"}}
    fn parse(s:&str)->Result<Self,String>{Self::ALL.into_iter().find(|r|r.token()==s).ok_or_else(||"input reference must be screen, facing or opponent".into())}
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
    OpponentInput { buttons: u32, stick: [f32; 2], frames: u8 },
    /// Simultaneous gameplay buttons and both sticks; no readiness/move writes.
    Controller { buttons:u32, stick:[f32;2], cstick:[f32;2], frames:u8,
        reference:InputReference, cstick_reference:InputReference },
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
    Chase { gap:f32, strength:f32, frames:u8 },
    Wait(u8),
    Face(Direction),
    IfFacingLedge { away: bool },
    /// Instant current opponent-side branch; no turn or input is performed.
    IfFacingOpponent { away: bool },
    ElseFacing,
    EndFacing,
    /// Input-only positioning at a locked ledge's inward offset.
    LedgePosition { offset:f32, strength:f32, tolerance:f32, brake:f32, frames:u16 },
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
    pub fn needs_ledge(self)->bool {
        matches!(self,Self::IfFacingLedge{..}|Self::LedgePosition{..}
            |Self::Face(Direction::Ledge|Direction::Inward|Direction::Outward|Direction::SavedLedge|Direction::AwayFromLedge)
            |Self::Walk{direction:Direction::Ledge|Direction::Inward|Direction::Outward|Direction::SavedLedge|Direction::AwayFromLedge,..}
            |Self::Run{direction:Direction::Ledge|Direction::Inward|Direction::Outward|Direction::SavedLedge|Direction::AwayFromLedge,..}
            |Self::Drift{direction:Direction::Ledge|Direction::Inward|Direction::Outward|Direction::SavedLedge|Direction::AwayFromLedge,..})
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
            timeout: 3600,
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
impl Catalog {
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() as u64 > MAX_FILE_BYTES {
            return Err("sequence file exceeds 512 KiB".into());
        }
        // Entry embeds all 32 actions. Geometric Vec growth temporarily kept
        // both the old and new roster buffers alive during saves. Count headers
        // first, then reserve once and report allocation failure instead of
        // invoking the aborting OOM handler for the largest save allocation.
        let count = source
            .lines()
            .filter(|line| {
                let line = line.split('#').next().unwrap_or("").trim();
                line.starts_with('[') && line.ends_with(']') && !retired_button_header(line) && !settings_header(line)
            })
            .take(MAX_ENTRIES + 1)
            .count();
        if count > MAX_ENTRIES {
            return Err(format!(
                "at most {MAX_ENTRIES} sequence sections are allowed"
            ));
        }
        let mut entries: Vec<Entry> = Vec::new();
        entries.try_reserve_exact(count).map_err(|_| {
            "Not enough memory to validate sequences; retry outside battle".to_owned()
        })?;
        let identities = FighterProfiles::default();
        let mut current: Option<(Entry, usize)> = None;
        let mut numeric_keys = Vec::new();
        let mut retired_buttons = false;
        let mut universal = None::<String>;
        let mut universal_section = false;
        let mut universal_keys = Vec::new();
        let mut tuning_kind = None;
        let mut tuning = std::collections::BTreeMap::new();
        for (index, raw) in source.lines().enumerate() {
            let line_number = index + 1;
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                finish_entry(&mut current, &mut entries)?;
                tuning_kind = None;
                retired_buttons = false;
                universal_section = line == "[universal.settings]";
                if universal_section {
                    if universal.is_some() { return Err(format!("line {line_number}: duplicate universal settings")); }
                    universal = Some(String::new());
                    continue;
                }
                if let Some(id)=line.strip_prefix("[template.").and_then(|v|v.strip_suffix(']')) {
                    if !valid_id(id) || entries.iter().any(|e|e.fighter_kind==-1 && e.choice==id) {
                        return Err(format!("line {line_number}: invalid or duplicate template id {id}"));
                    }
                    numeric_keys.clear();
                    current=Some((Entry {fighter_kind:-1,fighter:"template".into(),slot:PresetSlot::X,
                        choice:id.into(),name:None,default:false,explicit_start:false,sequence:Sequence::default()},line_number));
                    continue;
                }
                retired_buttons = retired_button_header(line);
                if retired_buttons {
                    continue;
                }
                let header = &line[1..line.len() - 1];
                let mut parts = header.split('.');
                let fighter = words_id(parts.next().unwrap_or("").trim());
                let slot = words_id(parts.next().unwrap_or("").trim());
                let choice = words_id(parts.next().unwrap_or("").trim());
                if fighter.is_empty()
                    || slot.is_empty()
                    || (choice.is_empty() && slot != "settings")
                    || parts.next().is_some()
                {
                    return Err(format!(
                        "line {line_number}: section must be [fighter.situation.choice_id]"
                    ));
                }
                let fighter_kind = crate::opponent_data::all()
                    .iter()
                    .find(|entry| entry.name.eq_ignore_ascii_case(&fighter))
                    .map(|entry| entry.kind)
                    .or_else(|| {
                        (0..0x5e).find(|kind| {
                            identities
                                .profile(*kind)
                                .is_some_and(|p| p.fighter_kind == *kind && p.name == fighter)
                        })
                    })
                    .ok_or_else(|| format!("line {line_number}: unknown fighter {fighter}"))?;
                if slot == "settings" && choice.is_empty() {
                    if tuning.insert(fighter_kind, crate::fighter_tuning::Tuning::default()).is_some() {
                        return Err(format!("line {line_number}: duplicate fighter settings"));
                    }
                    tuning_kind = Some(fighter_kind);
                    continue;
                }
                let slot = parse_slot(&slot)
                    .ok_or_else(|| format!("line {line_number}: unknown situation {slot}"))?;
                if !valid_id(&choice) {
                    return Err(format!("line {line_number}: invalid choice id {choice}"));
                }
                if entries.iter().any(|entry| {
                    entry.fighter_kind == fighter_kind
                        && entry.slot == slot
                        && entry.choice == choice
                }) {
                    return Err(format!("line {line_number}: duplicate section {header}"));
                }
                numeric_keys.clear();
                current = Some((
                    Entry {
                        fighter_kind,
                        fighter,
                        slot,
                        choice,
                        name: None,
                        default: false,
                        explicit_start: false,
                        sequence: Sequence::default(),
                    },
                    line_number,
                ));
                continue;
            }
            if retired_buttons {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {line_number}: expected key = value"));
            };
            if universal_section {
                let key = key.trim().to_ascii_lowercase();
                if universal_keys.contains(&key) { return Err(format!("line {line_number}: duplicate universal key {key}")); }
                universal.as_mut().unwrap().push_str(&format!("{key} = {}\n", value.trim()));
                universal_keys.push(key);
                continue;
            }
            if let Some(kind) = tuning_kind {
                tuning.get_mut(&kind).unwrap().set(key.trim(), value.trim())
                    .map_err(|e|format!("line {line_number}: {e}"))?;
                continue;
            }
            let Some((entry, _)) = current.as_mut() else {
                return Err(format!(
                    "line {line_number}: name/actions must follow a section"
                ));
            };
            match key.trim().to_ascii_lowercase().as_str() {
                "name" | "label" => {
                    if entry.name.is_some() {
                        return Err(format!("line {line_number}: duplicate name"));
                    }
                    let value = value.trim();
                    let value = if value.starts_with('"') || value.ends_with('"') {
                        value
                            .strip_prefix('"')
                            .and_then(|v| v.strip_suffix('"'))
                            .ok_or_else(|| {
                                format!("line {line_number}: name needs matching quotes")
                            })?
                    } else {
                        value
                    };
                    if value.trim().is_empty()
                        || value.len() > 64
                        || value.chars().any(char::is_control)
                    {
                        return Err(format!("line {line_number}: name must be 1–64 bytes without control characters"));
                    }
                    entry.name = Some(value.trim().to_owned());
                }
                "builtin" => {
                    if entry.sequence.builtin.is_some() { return Err(format!("line {line_number}: duplicate builtin")); }
                    entry.sequence.builtin = Some(crate::button_bindings::Builtin::parse(value.trim())
                        .ok_or_else(||format!("line {line_number}: unknown built-in button action"))?);
                }
                "mode" => {
                    if entry.sequence.mode.is_some() {return Err(format!("line {line_number}: duplicate mode"));}
                    let mode=crate::button_bindings::Mode::parse(&words_id(value)).ok_or_else(||format!("line {line_number}: unknown/unimplemented mode {}",value.trim()))?;
                    entry.sequence.mode=Some(crate::button_bindings::ModeBinding {mode,activation:crate::button_bindings::Activation::Hold});
                }
                "activation" => {
                    if numeric_keys.iter().any(|k|k=="activation") {return Err(format!("line {line_number}: duplicate activation"));}
                    let Some(binding)=entry.sequence.mode.as_mut() else {return Err(format!("line {line_number}: activation must follow mode"));};
                    binding.activation=match value.trim() {"hold"=>crate::button_bindings::Activation::Hold,"toggle"=>crate::button_bindings::Activation::Toggle,_=>return Err(format!("line {line_number}: activation must be hold or toggle"))};
                    numeric_keys.push("activation".into());
                }
                "ground_only" => {
                    if numeric_keys.iter().any(|k|k=="ground_only") { return Err(format!("line {line_number}: duplicate ground_only")); }
                    entry.sequence.ground_only=match value.trim() {"true"=>true,"false"=>false,_=>return Err(format!("line {line_number}: ground_only must be true or false"))};
                    numeric_keys.push("ground_only".into());
                }
                "while_held" => {
                    if numeric_keys.iter().any(|k| k=="while_held") { return Err(format!("line {line_number}: duplicate while_held")); }
                    entry.sequence.while_held = match value.trim() { "true" => true, "false" => false, _ => return Err(format!("line {line_number}: while_held must be true or false")) };
                    numeric_keys.push("while_held".into());
                }
                "reaction" => {
                    if numeric_keys.iter().any(|k| k=="reaction") {return Err(format!("line {line_number}: duplicate reaction"));}
                    entry.sequence.reaction=crate::reaction_rules::Rule::parse(value.trim()).map_err(|e|format!("line {line_number}: {e}"))?;
                    numeric_keys.push("reaction".into());
                }
                "actions" => {
                    if entry.sequence.len != 0 {
                        return Err(format!("line {line_number}: duplicate actions"));
                    }
                    let parsed = parse_actions(value.trim(), line_number)?;
                    entry.sequence.actions = parsed.actions;
                    entry.sequence.len = parsed.len;
                }
                "default" => {
                    if numeric_keys.iter().any(|key| key == "default") {
                        return Err(format!("line {line_number}: duplicate default"));
                    }
                    entry.default = match value.trim() { "true" => true, "false" => false, _ => return Err(format!("line {line_number}: default must be true or false")) };
                    numeric_keys.push("default".to_owned());
                }
                "start_delay" | "timeout" | "spatial_timeout" => {
                    let key = key.trim().to_ascii_lowercase();
                    if numeric_keys.contains(&key) {
                        return Err(format!("line {line_number}: duplicate {key}"));
                    }
                    let frames: u16 = value.trim().parse()
                        .map_err(|_| format!("line {line_number}: {key} must be an integer frame count"))?;
                    if frames > 3600 || key != "start_delay" && frames == 0 {
                        return Err(format!("line {line_number}: {key} must be {}..3600", if key == "start_delay" { 0 } else { 1 }));
                    }
                    match key.as_str() {
                        "start_delay" => { entry.sequence.start_delay = frames; entry.explicit_start = true; },
                        "timeout" => entry.sequence.timeout = frames,
                        _ => entry.sequence.spatial_timeout = frames,
                    }
                    numeric_keys.push(key);
                }
                _ => {
                    return Err(format!(
                        "line {line_number}: supported keys are name, actions, builtin, default, start_delay, timeout, spatial_timeout, while_held, ground_only and reaction"
                    ))
                }
            }
        }
        finish_entry(&mut current, &mut entries)?;
        if let Some(settings) = universal.as_deref() {
            validate_universal_settings(settings)?;
        }
        let mut templates=Vec::new();
        entries.retain(|e| {
            if e.fighter_kind!=-1 {return true;}
            templates.push(Template {id:e.choice.clone(),name:e.name.clone().unwrap_or_else(||e.choice.replace('_'," ")),sequence:e.sequence});
            false
        });
        if templates.len()>MAX_TEMPLATES {return Err(format!("at most {MAX_TEMPLATES} generic templates are allowed"));}
        Ok(Self { entries, tuning, universal, templates })
    }

    pub fn validate(&self, profiles: &FighterProfiles) -> Result<(), String> {
        // Count each row once. The old nested full-catalog scan allocated a
        // temporary row Vec for every section, repeatedly during a single save.
        let mut rows = std::collections::BTreeMap::<(i32, usize), (usize, usize)>::new();
        let mut simple_events=std::collections::BTreeSet::new();
        let mut rule_counts = std::collections::BTreeMap::<(i32,usize),usize>::new();
        for entry in &self.entries {
            validate_facing_blocks(entry.sequence.actions())?;
            crate::reaction_rules::validate_response(entry.sequence)?;
            if let Some(event)=entry.sequence.reaction.trigger.simple(){if !simple_events.insert((entry.fighter_kind,event)){return Err("Only one auto-react sequence per event per fighter is allowed".into());}}

            if entry.sequence.reaction.active() {
                let count=rule_counts.entry((entry.fighter_kind,entry.sequence.reaction.trigger.family())).or_default();*count+=1;
                if *count>crate::reaction_rules::MAX_RULES_PER_MODE{return Err(format!("{} exceeds 8 rules in one assist mode",entry.fighter));}
            }
            let row = rows
                .entry((entry.fighter_kind, entry.slot as usize))
                .or_default();
            if row.0 == 0 {
                profiles
                    .profile(entry.fighter_kind)
                    .filter(|profile| profile.fighter_kind == entry.fighter_kind)
                    .ok_or_else(|| format!("{} has no compiled ledge profile", entry.fighter))?;
            }
            row.0 += usize::from(!entry.sequence.reaction.active());
            row.1 += usize::from(entry.default && !entry.sequence.reaction.active());
            if row.0 > entry.slot.max_choices() {
                return Err(format!(
                    "{}.{} exceeds {} menu options",
                    entry.fighter,
                    slot_name(entry.slot),
                    entry.slot.max_choices()
                ));
            }
            if row.1 > 1 {
                return Err(format!(
                    "{}.{} has multiple defaults",
                    entry.fighter,
                    slot_name(entry.slot)
                ));
            }
        }
        Ok(())
    }

    pub fn find(&self, fighter_kind: i32, slot: PresetSlot, choice: &str) -> Option<Sequence> {
        self.entries
            .iter()
            .find(|entry| {
                entry.fighter_kind == fighter_kind && entry.slot == slot && entry.choice == choice
            })
            .map(|entry| entry.sequence)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// Startup-only filtering, before menus are frozen and recipes can execute.
    pub fn for_build(mut self, diagnostic: bool) -> Self {
        if !diagnostic {
            self.entries.retain(|entry| {
                !entry.choice.starts_with("test")
                    && !entry
                        .name
                        .as_deref()
                        .is_some_and(|name| name.to_ascii_lowercase().contains("[test]"))
            });
        }
        self
    }
}

// Return only the validation result here. Expanded button settings must not
// reserve config-sized Result/map temporaries in every catalog parser frame,
// including small files that do not contain universal settings at all.
#[inline(never)]
fn validate_universal_settings(settings: &str) -> Result<(), String> {
    crate::live_input::LiveConfig::parse(settings)
        .map(|_| ())
        .map_err(|e| format!("universal.settings: {e}"))
}

// Policy sections contain no 32-action Entry; do not reserve recipe storage for them.
fn settings_header(line: &str) -> bool {
    let Some(header)=line.strip_prefix('[').and_then(|s|s.strip_suffix(']')) else {return false;};
    let mut parts=header.split('.');
    parts.next().is_some() && parts.next().is_some_and(|s|s.trim().eq_ignore_ascii_case("settings")) && parts.next().is_none()
}

/// Old universal bindings remain inert; never migrate them into fighter lists.
fn retired_button_header(line: &str) -> bool {
    let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return false;
    };
    let mut parts = header.split('.');
    let fighter=parts.next().unwrap_or("").trim();
    let slot=parts.next().unwrap_or("").trim();
    // Compatibility only: retired trigger sections never enter the catalog,
    // menu, ownership masks or playback. Keep installed saved files loadable.
    if ["l","r","zl","zr","lz","rz"].iter().any(|s|slot.eq_ignore_ascii_case(s)) {
        return true;
    }
    fighter.eq_ignore_ascii_case("universal") &&
        ["x","y","button_x","button_y","button-x","button-y","button x","button y"]
            .iter().any(|s|slot.eq_ignore_ascii_case(s))
}

fn finish_entry(
    current: &mut Option<(Entry, usize)>,
    entries: &mut Vec<Entry>,
) -> Result<(), String> {
    if let Some((entry, line)) = current.take() {
        if entry.fighter_kind==-1 && (entry.default || entry.sequence.mode.is_some()
            || entry.sequence.builtin.is_some() || entry.sequence.reaction.active() || entry.sequence.while_held) {
            return Err(format!("line {line}: templates contain actions/timings only, without defaults, modes, builtins, reactions or held activation"));
        }
        if let Some(binding) = entry.sequence.mode {
            if !entry.slot.is_button() || !binding.mode.supports(entry.fighter_kind) {
                return Err(format!(
                    "line {line}: mode is unavailable for this fighter/button row"
                ));
            }
            if entry.sequence.len != 0 || entry.sequence.start_delay != 0 {
                return Err(format!(
                    "line {line}: mode bindings cannot contain actions or start delays"
                ));
            }
        }
        if entry.sequence.builtin.is_some() && (!entry.slot.is_button() || entry.sequence.mode.is_some() || entry.sequence.len != 0 || entry.sequence.start_delay != 0 || entry.sequence.reaction.active() || entry.sequence.while_held || entry.sequence.ground_only) {
            return Err(format!("line {line}: builtin requires a button row and cannot contain actions, conditions, delays, loops or modes"));
        }
        if !entry.explicit_start && entry.sequence.mode.is_none() && entry.sequence.builtin.is_none() {
            return Err(format!(
                "line {line}: section requires start_delay = FRAMES (use 0 for immediate input)"
            ));
        }
        if entry.sequence.len == 0 && entry.sequence.mode.is_none() && entry.sequence.builtin.is_none() {
            return Err(format!("line {line}: section is missing actions"));
        }
        if entries.len() >= MAX_ENTRIES {
            return Err(format!(
                "at most {MAX_ENTRIES} sequence sections are allowed"
            ));
        }
        entries.push(entry);
    }
    Ok(())
}

fn words_id(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .replace('-', "_")
}

fn push(sequence: &mut Sequence, action: Action) -> Result<(), String> {
    if usize::from(sequence.len) == MAX_ACTIONS {
        return Err(format!(
            "at most {MAX_ACTIONS} expanded actions are allowed"
        ));
    }
    sequence.actions[usize::from(sequence.len)] = action;
    sequence.len += 1;
    Ok(())
}

fn parse_actions(value: &str, line: usize) -> Result<Sequence, String> {
    let mut result = Sequence::default();
    for token in value.split(',').map(str::trim) {
        if token.is_empty() {
            return Err(format!("line {line}: empty action"));
        }
        parse_action(token, &mut result).map_err(|e| format!("line {line}: {e}"))?;
    }
    validate_facing_blocks(result.actions()).map_err(|e| format!("line {line}: {e}"))?;
    Ok(result)
}

/// Flat facing blocks are deliberately bounded and cannot nest.
pub fn validate_facing_blocks(actions: &[Action]) -> Result<(), String> {
    let mut open=false;let mut otherwise=false;
    for action in actions {
        match action {
            Action::IfFacingLedge{..}|Action::IfFacingOpponent{..} => {
                if open {return Err("Facing blocks cannot nest; add end_facing first".into());}
                open=true;otherwise=false;
            }
            Action::ElseFacing => {
                if !open || otherwise {return Err("else_facing needs an open facing block and may appear only once".into());}
                otherwise=true;
            }
            Action::EndFacing => {
                if !open {return Err("end_facing has no matching facing condition".into());}
                open=false;
            }
            _=>{}
        }
    }
    if open {return Err("Facing block needs end_facing".into());}
    Ok(())
}

fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
    Action::Input {
        buttons,
        stick,
        frames,
        relative,
    }
}

fn buttons(value: &str) -> Result<u32, String> {
    use crate::native_input::*;
    let mut mask = 0;
    for button in value.split('+') {
        mask |= match button {
            "a" | "attack" => MAPPER_A,
            "b" | "special" => MAPPER_B,
            "x" | "jump" => MAPPER_X,
            "y" => MAPPER_Y,
            "l" => MAPPER_L,
            "r" | "grab" => MAPPER_R,
            "zl" => MAPPER_ZL,
            "zr" | "shield" => MAPPER_ZR,
            "none" | "0" => 0,
            _ => return Err(format!("unknown button {button}")),
        };
    }
    Ok(mask)
}

fn parse_action(token: &str, sequence: &mut Sequence) -> Result<(), String> {
    use crate::native_input::{
        MAPPER_A as A, MAPPER_B as B, MAPPER_R as R, MAPPER_X as X, MAPPER_ZR as ZR,
    };
    let normalized = token
        .split(':')
        .map(|v| {
            if v.trim().starts_with('-') {
                v.trim().to_ascii_lowercase()
            } else {
                words_id(v)
            }
        })
        .collect::<Vec<_>>()
        .join(":");
    let normalized = if let Some(wait) = normalized.strip_prefix("wait_") {
        format!(
            "wait:{}",
            wait.strip_suffix("_frames")
                .or_else(|| wait.strip_suffix("_frame"))
                .unwrap_or(wait)
        )
    } else if let Some(direction) = normalized.strip_prefix("face_") {
        format!("face:{direction}")
    } else if normalized == "grab_no_range_check" {
        "grab:ignore".into()
    } else if normalized == "grab_range_verified" {
        "grab:fighter_range".into()
    } else {
        normalized
    };
    let parts: Vec<&str> = normalized.split(':').collect();
    let name = parts[0];
    let exact = |count| {
        if parts.len() == count {
            Ok(())
        } else {
            Err(format!("invalid action {token}"))
        }
    };
    let duration = |index: usize, default: u8| -> Result<u8, String> {
        let frames = match parts.get(index) {
            Some(value) => parse_u8(value, token)?,
            None => default,
        };
        if frames == 0 {
            Err(format!("{token}: input duration must be 1..255"))
        } else {
            Ok(frames)
        }
    };
    match name {
        "wait" | "release" => {
            if parts.len() > 2 {
                return Err(format!("invalid action {token}"));
            }
            push(
                sequence,
                Action::Wait(if parts.len() == 2 {
                    parse_u8(parts[1], token)?
                } else {
                    1
                }),
            )
        }
        "check" | "require" => {
            let condition = parts
                .get(1)
                .and_then(|v| Condition::from_token(v))
                .ok_or_else(|| format!("{token}: unknown check condition"))?;
            let wait_index = if condition.has_distance() { 3 } else { 2 };
            if parts.len() < wait_index || parts.len() > wait_index + 1 {
                return Err(format!("invalid check {token}"));
            }
            let value = if condition.has_distance() {
                parse_float(parts[2], 0.0, 100.0, token)?
            } else {
                0.0
            };
            let wait = parts
                .get(wait_index)
                .map_or(Ok(0), |v| parse_u8(v, token))?;
            push(
                sequence,
                Action::Check {
                    condition,
                    value,
                    wait,
                },
            )
        }
        "tap" => {
            exact(7)?;
            let frames = parts[6]
                .parse::<u16>()
                .ok()
                .filter(|v| (1..=3600).contains(v))
                .ok_or_else(|| format!("{token}: total frames must be 1..3600"))?;
            let tap_buttons = buttons(parts[1])?;
            if tap_buttons == 0 {
                return Err("tap requires a button".into());
            }
            push(
                sequence,
                Action::Tap {
                    buttons: tap_buttons,
                    stick: [
                        parse_float(parts[2], -1., 1., token)?,
                        parse_float(parts[3], -1., 1., token)?,
                    ],
                    hold: duration(4, 1)?,
                    gap: duration(5, 1)?,
                    frames,
                },
            )
        }
        "press" | "hold" => {
            if parts.len() != 2 && parts.len() != 3 {
                return Err(format!("use press:BUTTONS[:FRAMES] or hold:BUTTONS:FRAMES"));
            }
            push(
                sequence,
                input(buttons(parts[1])?, [0.0; 2], duration(2, 1)?, false),
            )
        }
        "controller_input" => {
            if !(7..=9).contains(&parts.len()) {return Err("use controller_input:BUTTONS:LX:LY:RX:RY:FRAMES[:LEFT_REF[:RIGHT_REF]]".into());}
            push(sequence,Action::Controller {buttons:buttons(parts[1])?,
                stick:[parse_float(parts[2],-1.,1.,token)?,parse_float(parts[3],-1.,1.,token)?],
                cstick:[parse_float(parts[4],-1.,1.,token)?,parse_float(parts[5],-1.,1.,token)?],
                frames:duration(6,4)?,reference:InputReference::parse(parts.get(7).copied().unwrap_or("screen"))?,
                cstick_reference:InputReference::parse(parts.get(8).copied().unwrap_or("screen"))?})
        }
        "opponent_input" => {
            exact(5)?;
            push(sequence, Action::OpponentInput {
                buttons: buttons(parts[1])?,
                stick: [parse_float(parts[2], -1.0, 1.0, token)?, parse_float(parts[3], -1.0, 1.0, token)?],
                frames: duration(4, 1)?,
            })
        }
        "input" | "facing_input" => {
            exact(5)?;
            push(
                sequence,
                input(
                    buttons(parts[1])?,
                    [
                        parse_float(parts[2], -1.0, 1.0, token)?,
                        parse_float(parts[3], -1.0, 1.0, token)?,
                    ],
                    duration(4, 1)?,
                    name == "facing_input",
                ),
            )
        }
        "cstick" => {
            exact(4)?;
            push(
                sequence,
                Action::CStick {
                    stick: [
                        parse_float(parts[1], -1.0, 1.0, token)?,
                        parse_float(parts[2], -1.0, 1.0, token)?,
                    ],
                    frames: duration(3, 1)?,
                },
            )
        }
        "stick" => {
            exact(4)?;
            push(
                sequence,
                input(
                    0,
                    [
                        parse_float(parts[1], -1.0, 1.0, token)?,
                        parse_float(parts[2], -1.0, 1.0, token)?,
                    ],
                    duration(3, 1)?,
                    false,
                ),
            )
        }
        "hold_until_manual" | "facing_hold_until_manual" => {
            exact(4)?;
            let held = buttons(parts[1])?;
            if held == 0 { return Err("hold_until_manual requires a button".into()); }
            push(sequence, input(held, [parse_float(parts[2], -1., 1., token)?, parse_float(parts[3], -1., 1., token)?], 0, name == "facing_hold_until_manual"))
        }
        "chase_if_far" => { exact(4)?; push(sequence,Action::Chase{gap:parse_float(parts[1],0.,100.,token)?,strength:parse_float(parts[2],0.,1.,token)?,frames:duration(3,14)?}) }
        "shield_until_manual" => { exact(1)?; push(sequence, Action::ShieldUntilManual) }
        "shield_until_block" => {
            exact(2)?;
            push(sequence, Action::ShieldUntilBlock(duration(1, 90)?))
        }
        "if_facing_ledge" | "if_back_to_ledge" => {
            exact(1)?;
            push(sequence, Action::IfFacingLedge { away: name == "if_back_to_ledge" })
        }
        "if_facing_opponent" | "if_back_to_opponent" => {
            exact(1)?;push(sequence,Action::IfFacingOpponent{away:name=="if_back_to_opponent"})
        }
        "else_facing" => {exact(1)?;push(sequence,Action::ElseFacing)}
        "end_facing" => { exact(1)?; push(sequence, Action::EndFacing) }
        "face" => {
            exact(2)?;
            push(sequence, Action::Face(parse_direction(parts[1], token)?))
        }
        "walk" => {
            if parts.len() != 4 && parts.len() != 5 {
                return Err(format!("use walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]"));
            }
            let direction = parse_direction(parts[1], token)?;
            let min_gap = if parts.len() == 5 {
                if direction != Direction::Opponent {
                    return Err("MIN_GAP requires walk:opponent".into());
                }
                Some(parse_float(parts[4], 0.0, 60.0, token)?)
            } else {
                None
            };
            push(
                sequence,
                Action::Walk {
                    direction,
                    units: parse_float(parts[2], 0.0, 100.0, token)?,
                    strength: parse_float(parts[3], 0.1, 0.95, token)?,
                    min_gap,
                },
            )
        }
        "run_to_ledge" => {
            if !(3..=6).contains(&parts.len()) {return Err("use run_to_ledge:OFFSET:STRENGTH[:TOLERANCE[:BRAKE_DISTANCE[:MAX_FRAMES]]]".into());}
            push(sequence,Action::LedgePosition {
                offset:parse_float(parts[1],0.,200.,token)?,
                strength:parse_float(parts[2],0.3,1.,token)?,
                tolerance:parts.get(3).map_or(Ok(1.),|v|parse_float(v,0.,20.,token))?,
                brake:parts.get(4).map_or(Ok(6.),|v|parse_float(v,0.,100.,token))?,
                frames:parts.get(5).map_or(Ok(180),|v|v.parse::<u16>().ok().filter(|n|(1..=3600).contains(n)).ok_or_else(||format!("invalid movement timeout in {token}")))?,
            })
        }
        "run" => {
            exact(3)?;
            push(
                sequence,
                Action::Run {
                    direction: parse_direction(parts[1], token)?,
                    units: parse_float(parts[2], 0.0, 100.0, token)?,
                },
            )
        }
        "walk_to_opponent" | "drift_to_opponent" => {
            exact(4)?;
            push(
                sequence,
                Action::Approach {
                    airborne: name == "drift_to_opponent",
                    strength: parse_float(
                        parts[1],
                        0.1,
                        if name == "drift_to_opponent" {
                            1.0
                        } else {
                            0.95
                        },
                        token,
                    )?,
                    frames: duration(2, 1)?,
                    gap: parse_float(parts[3], 0.0, 60.0, token)?,
                },
            )
        }
        "drift" | "drift_for" => {
            exact(4)?;
            push(
                sequence,
                Action::Drift {
                    direction: parse_direction(parts[1], token)?,
                    strength: parse_float(parts[2], 0.0, 1.0, token)?,
                    frames: duration(3, 1)?,
                },
            )
        }
        "range" => {
            exact(2)?;
            let range = match parts[1] {
                "ignore" => RangeCheck::Ignore,
                "fighter_range" => RangeCheck::FighterGrabRange,
                "fighter_range_instant" => RangeCheck::FighterGrabRangeInstant,
                value => RangeCheck::Verify(parse_float(value, 0.0, 60.0, token)?),
            };
            push(sequence, Action::Range(range))
        }
        "grab" => {
            if parts.len() > 2 {
                return Err(format!("invalid action {token}"));
            }
            let range = match parts.get(1).copied().unwrap_or("ignore") {
                "ignore" | "no_range_check" | "unverified" => RangeCheck::Ignore,
                "fighter_range" | "range_verified" | "verified" => RangeCheck::FighterGrabRange,
                "fighter_range_instant" => RangeCheck::FighterGrabRangeInstant,
                v => RangeCheck::Verify(parse_float(v, 0.0, 60.0, token)?),
            };
            if range != RangeCheck::Ignore {
                push(sequence, Action::Range(range))?;
            }
            push(sequence, input(R, [0.0; 2], 1, false))
        }
        _ => {
            if parts.len() > 3 {
                return Err(format!("invalid action {token}"));
            }
            let name = if let Some(name) = name
                .strip_prefix("turnaround_")
                .or_else(|| name.strip_prefix("turn_around_"))
            {
                push(sequence, Action::Face(Direction::Backward))?;
                name
            } else {
                name
            };
            let (paired, name) = name
                .strip_prefix("short_hop_")
                .or_else(|| name.strip_prefix("shorthop_"))
                .map_or((false, name), |v| (true, v));
            if name == "dash_attack" || name == "dashattack" {
                push(sequence, input(0, [1.0, 0.0], duration(1, 1)?, true))?;
                return push(sequence, input(A, [1.0, 0.0], duration(2, 1)?, true));
            }
            if matches!(name, "a" | "b" | "x" | "y" | "l" | "r" | "zl" | "zr") {
                return push(
                    sequence,
                    input(buttons(name)?, [0.0; 2], duration(1, 1)?, false),
                );
            }
            let (mask, stick, default) = match name {
                "short_hop" | "shorthop" | "jump" => (X, [0.0; 2], 1),
                "full_hop" | "fullhop" => {
                    if parts.len() < 2 {
                        return Err(
                            "full_hop requires an explicit hold duration, e.g. full_hop:4".into(),
                        );
                    }
                    (X, [0.0; 2], 1)
                }
                "shield" => (ZR, [0.0; 2], 1),
                "jab" | "attack" => (A, [0.0; 2], 1),
                "forward_tilt" | "f_tilt" | "ftilt" => (A, [0.6, 0.0], 1),
                "up_tilt" | "u_tilt" | "utilt" => (A, [0.0, 0.6], 1),
                "down_tilt" | "d_tilt" | "dtilt" => (A, [0.0, -0.6], 1),
                "forward_smash" | "f_smash" | "fsmash" => (A, [1.0, 0.0], 1),
                "up_smash" | "u_smash" | "usmash" => (A, [0.0, 1.0], 1),
                "down_smash" | "d_smash" | "dsmash" => (A, [0.0, -1.0], 1),
                "nair" | "neutral_air" | "neutral_aerial" => (A, [0.0; 2], 1),
                "fair" | "forward_air" | "forward_aerial" => (A, [1.0, 0.0], 1),
                "bair" | "back_air" | "back_aerial" => (A, [-1.0, 0.0], 1),
                "uair" | "up_air" | "up_aerial" => (A, [0.0, 1.0], 1),
                "dair" | "down_air" | "down_aerial" => (A, [0.0, -1.0], 1),
                "neutral_special" | "neutral_b" | "special" => (B, [0.0; 2], 1),
                "side_special" | "side_b" => (B, [1.0, 0.0], 1),
                "up_special" | "up_b" | "final_cutter" => (B, [0.0, 1.0], 1),
                "down_special" | "down_b" | "stone" => (B, [0.0, -1.0], 1),
                "dash" => (0, [1.0, 0.0], 1),
                "crouch" | "fast_fall" => (0, [0.0, -1.0], 1),
                "roll_forward" => (ZR, [1.0, 0.0], 1),
                "roll_backward" => (ZR, [-1.0, 0.0], 1),
                "spot_dodge" => (ZR, [0.0, -1.0], 1),
                "air_dodge" => (ZR, [0.0; 2], 1),
                _ => return Err(format!("unknown action {token}")),
            };
            if paired
                && !matches!(
                    name,
                    "nair"
                        | "fair"
                        | "bair"
                        | "uair"
                        | "dair"
                        | "neutral_air"
                        | "forward_air"
                        | "back_air"
                        | "up_air"
                        | "down_air"
                )
            {
                return Err(format!("{token}: short_hop_ requires an aerial name"));
            }
            let strength = if parts.len() == 3 {
                parse_float(parts[2], 0.0, 1.0, token)?
            } else {
                1.0
            };
            let stick = [stick[0] * strength, stick[1] * strength];
            push(
                sequence,
                input(
                    mask | if paired { X } else { 0 },
                    stick,
                    duration(1, default)?,
                    true,
                ),
            )
        }
    }
}

fn parse_direction(value: &str, token: &str) -> Result<Direction, String> {
    match value {
        "opponent" | "enemy" | "toward" | "towards" | "toward_opponent" | "towards_opponent"
        | "toward_enemy" | "towards_enemy" => Ok(Direction::Opponent),
        "away" | "away_from_opponent" | "away_from_enemy" => Ok(Direction::AwayFromOpponent),
        "ledge" => Ok(Direction::Ledge),
        "saved_ledge" | "toward_saved_ledge" => Ok(Direction::SavedLedge),
        "away_from_ledge" | "away_from_saved_ledge" => Ok(Direction::AwayFromLedge),
        "inward" => Ok(Direction::Inward),
        "outward" => Ok(Direction::Outward),
        "facing" | "forward" => Ok(Direction::Facing),
        "backward" | "back" => Ok(Direction::Backward),
        _ => Err(format!("invalid direction in {token}")),
    }
}

fn parse_float(value: &str, min: f32, max: f32, token: &str) -> Result<f32, String> {
    let value = value
        .strip_suffix("_units")
        .or_else(|| value.strip_suffix("_unit"))
        .unwrap_or(value);
    let (raw, divisor) = value
        .strip_suffix('%')
        .or_else(|| value.strip_suffix("_percent"))
        .map_or((value, 1.0), |v| (v, 100.0));
    let value: f32 = raw
        .parse()
        .map_err(|_| format!("invalid number in {token}"))?;
    let value = value / divisor;
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(format!("number out of range in {token}"))
    }
}

fn parse_u8(value: &str, token: &str) -> Result<u8, String> {
    let value = value.strip_prefix("until_motion_frame_").unwrap_or(value);
    let value = value
        .strip_suffix("_frames")
        .or_else(|| value.strip_suffix("_frame"))
        .unwrap_or(value);
    value
        .parse()
        .map_err(|_| format!("invalid frame count in {token}"))
}

fn parse_slot(value: &str) -> Option<PresetSlot> {
    Some(match value.to_ascii_lowercase().as_str() {
        "neutral" | "neutral_getup" => PresetSlot::Neutral,
        "jump" | "jump_getup" => PresetSlot::Jump,
        "roll" | "roll_getup" => PresetSlot::Roll,
        "attack" | "getup_attack" => PresetSlot::Attack,
        "drop" | "ledge_drop" => PresetSlot::Drop,
        "x" | "button_x" | "button-x" => PresetSlot::X,
        "y" | "button_y" | "button-y" => PresetSlot::Y,
        "dpad_up" => PresetSlot::DpadUp,
        "dpad_down" => PresetSlot::DpadDown,
        "dpad_left" => PresetSlot::DpadLeft,
        "dpad_right" => PresetSlot::DpadRight,
        "l3" => PresetSlot::L3, "r3" => PresetSlot::R3,
        _ => return None,
    })
}

fn slot_name(slot: PresetSlot) -> &'static str {
    match slot {
        PresetSlot::Neutral => "neutral",
        PresetSlot::Jump => "jump",
        PresetSlot::Roll => "roll",
        PresetSlot::Attack => "attack",
        PresetSlot::Drop => "drop",
        PresetSlot::X => "x",
        PresetSlot::Y => "y",
        PresetSlot::DpadUp => "dpad_up",
        PresetSlot::DpadDown => "dpad_down",
        PresetSlot::DpadLeft => "dpad_left",
        PresetSlot::DpadRight => "dpad_right",
        PresetSlot::L3 => "l3", PresetSlot::R3 => "r3",
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_input::*;
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
        for kind in 0..0x5e {
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
        let c=Catalog::parse(source).unwrap();
        let s=c.entries()[0].sequence;
        let saved=persist::update(source,5,PresetSlot::Neutral,"test",s,false).unwrap();
        assert_eq!(Catalog::parse(&saved).unwrap().entries()[0].sequence,s);
        for actions in ["end_facing","if_facing_ledge, a","if_facing_ledge, if_back_to_ledge, a, end_facing, end_facing"] {
            assert!(Catalog::parse(&format!("[yoshi.neutral.test]\nstart_delay=0\nactions={actions}")).is_err());
        }
    }
}

#[cfg(test)]
mod generic_template_tests {
 use super::*;
 #[test]
 fn named_templates_are_inert_bounded_and_survive_organizing_and_saving() {
  let source="[universal.settings]\nenabled=true\n[template.shield]\nname=Quick shield\nstart_delay=0\nactions=input:zr:0:0:4\n[kirby.neutral.test]\nstart_delay=0\nactions=jab:2\n";
  let c=Catalog::parse(source).unwrap();assert_eq!(c.len(),1);assert_eq!(c.templates.len(),1);
  assert_eq!(c.templates[0].id,"shield");assert_eq!(c.templates[0].name,"Quick shield");
  let organized=layout::organize(source).unwrap();let again=Catalog::parse(&organized).unwrap();assert_eq!(again.templates,c.templates);
  let updated=persist::update(source,6,PresetSlot::Neutral,"test",c.entries()[0].sequence,false).unwrap();
  assert_eq!(Catalog::parse(&updated).unwrap().templates,c.templates);
  for diagnostic in [false,true] {let c=c.clone().for_build(diagnostic);assert_eq!(c.templates.len(),1);c.validate(&FighterProfiles::default()).unwrap();}
 }
 #[test]
 fn templates_cannot_activate_modes_reactions_or_defaults_and_reject_duplicate_ids() {
  for value in ["default=true","while_held=true","reaction=auto_roll:30:12","mode=auto_tech","builtin=normal"] {
   assert!(Catalog::parse(&format!("[template.bad]\nstart_delay=0\nactions=jab\n{value}\n")).is_err(),"{value}");
  }
  assert!(Catalog::parse("[template.a]\nstart_delay=0\nactions=jab\n[template.a]\nstart_delay=0\nactions=grab\n").is_err());
  assert!(Catalog::parse("[template.a.b]\nstart_delay=0\nactions=jab\n").is_err());
  let text=(0..=MAX_TEMPLATES).map(|i|format!("[template.test_{i}]\nstart_delay=0\nactions=jab\n")).collect::<String>();
  assert!(Catalog::parse(&text).unwrap_err().contains("generic templates"));
 }
}
