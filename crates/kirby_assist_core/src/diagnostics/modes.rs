//! Diagnostic-only copied evidence. No game reads, formatting or allocation in sample.
use crate::{action_owner::Owner, native_input::*};
use std::fmt::Write;

const OWNER_COUNT: usize = 15;
const REPORT_TEXT_CAPACITY_BYTES: usize = 1200;
const MIN_REPORT_INTERVAL_MS: u64 = 250;
const LABEL_CAPACITY_BYTES: usize = 64;
const CONFIGURATION_ROW_COUNT: usize = 25;
const RESULT_TOP_MARGIN: f32 = 14.0;
const RESULT_BOTTOM_MARGIN: f32 = 8.0;
const MIN_RESULT_HEIGHT: f32 = 40.0;
pub const SAFETY_TECH_GUARD: u8 = 1 << 0;
pub const SAFETY_MASH: u8 = 1 << 1;
pub const SAFETY_SDI: u8 = 1 << 2;
pub const SAFETY_DI: u8 = 1 << 3;

pub const OWNERS: [Owner; OWNER_COUNT] = [
    Owner::Safety,
    Owner::OptionRead,
    Owner::Ledge,
    Owner::RollRead,
    Owner::Trip,
    Owner::JabLock,
    Owner::Fair,
    Owner::Trump,
    Owner::Shortcut,
    Owner::Defense,
    Owner::Turbo,
    Owner::Posture,
    Owner::OutOfShield,
    Owner::GrabDodge,
    Owner::Reflect,
];
pub fn bit(owner: Owner) -> u32 {
    1 << OWNERS.iter().position(|v| *v == owner).unwrap()
}
pub fn name(owner: Owner) -> &'static str {
    match owner {
        Owner::Safety => "Tech / mash / SDI / DI",
        Owner::Reflect => "Projectile reflect / absorb",
        Owner::GrabDodge => "Incoming grab dodge",
        Owner::OptionRead => "Configured dodge follow-up / recovery punish",
        Owner::Ledge => "Ledge trapping",
        Owner::RollRead => "Roll / shield-pressure rules",
        Owner::Trip => "Trip follow-up",
        Owner::JabLock => "Yoshi jab lock",
        Owner::Fair => "Fair-hit follow-up",
        Owner::Trump => "Ledge trump",
        Owner::Shortcut => "Short hop / jab follow-up",
        Owner::Defense => "Defensive spot dodge / shield / duck",
        Owner::Turbo => "Turbo movement",
        Owner::Posture => "Idle crouch / turnaround tilt",
        Owner::OutOfShield => "Out of shield",
    }
}
/// Describe each tool's prerequisites without guessing a particular failed gate.
pub fn purpose(owner: Owner) -> &'static str {
    match owner {
        Owner::OutOfShield => "Confirms shield contact, retains defense through stun/multihits and selects the fighter-specific in-range response; yields to ledge responses.",
        Owner::Reflect => "Uses the configured native reflector for an incoming eligible projectile; yields to ledge, safety, manual attacks and committed punish/grab input. A full bucket is never discharged automatically.",
        Owner::Safety => "Handles native tech windows, capture escape, hitstop SDI and launch DI; physical stick override can cancel DI.",
        Owner::GrabDodge => "Requests a native-ready spot dodge for an incoming ordinary grab; may preempt general punishes, but yields to protected ledge responses and safety.",
        Owner::OptionRead => "Runs configured spot-dodge/grab-dodge follow-ups and retained recovery punishes; yields to incoming grab defense.",
        Owner::RollRead => "Watches approaching/crossing rolls; primes the chosen smash/grab only inside its timing and reach window.",
        Owner::Ledge => "Uses the saved response for neutral/jump/roll/attack/drop; waits for protection, reach and native readiness.",
        Owner::Trip => "Requires an observed trip/hit before buffering its grounded follow-up.",
        Owner::JabLock => "Requires an attributed Yoshi down-tilt hit and a lockable knockdown; confirms native lock before finishing.",
        Owner::Fair => "A fresh Kirby fair-one hit arms the landing smash; shield/stale flags do not arm it.",
        Owner::Trump => "Uses the held trump trigger, a known floor edge and native movement readiness.",
        Owner::Shortcut => "Handles short hop; native startup ends each request.",
        Owner::Defense => "Defensive shield/duck proposals yield to reads and ledge sequences; low hitboxes cannot safely be ducked.",
        Owner::Turbo => "Repeats the configured movement while its trigger and native movement scope permit it.",
        Owner::Posture => "Crouches while grounded and idle near an enemy, or aims configured physical tilts; movement/buttons suspend idle crouch.",
    }
}
pub fn motion_name(status: i32, motion: u64) -> &'static str {
    if status == crate::native_states::status::ATTACK_AIR {
        for (key, name) in [
            ("attack_air_n", "neutral air"),
            ("attack_air_f", "forward air"),
            ("attack_air_b", "back air"),
            ("attack_air_hi", "up air"),
            ("attack_air_lw", "down air"),
        ] {
            if crate::timing::motion_hash(key) == motion {
                return name;
            }
        }
    }
    native_name(status)
}
fn native_action(status: i32) -> u16 {
    match status {
        crate::native_states::status::JUMP_SQUAT..=crate::native_states::status::FALL_SPECIAL => {
            NATIVE_ACTION_AIRBORNE
        }
        crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE => NATIVE_ACTION_SHIELD,
        crate::native_states::ESCAPE => NATIVE_ACTION_SPOT_DODGE,
        crate::native_states::ESCAPE_F..=crate::native_states::ESCAPE_AIR_SLIDE => {
            NATIVE_ACTION_ROLL_OR_AIR_DODGE
        }
        crate::native_states::status::ATTACK..=crate::native_states::status::ATTACK_HI4 => {
            NATIVE_ACTION_GROUND_ATTACK
        }
        crate::native_states::status::ATTACK_AIR => NATIVE_ACTION_AERIAL_ATTACK,
        crate::native_states::status::CATCH..=crate::native_states::status::CATCH_JUMP => {
            NATIVE_ACTION_GRAB
        }
        crate::native_states::status::DAMAGE..=crate::native_states::status::PASSIVE_CEIL => {
            NATIVE_ACTION_DAMAGE_OR_TECH
        }
        crate::native_states::status::CLIFF_CATCH_MOVE
            ..=crate::native_states::status::CLIFF_JUMP2 => NATIVE_ACTION_LEDGE_ACTION,
        _ => NATIVE_ACTION_NONE,
    }
}
const NATIVE_ACTION_NONE: u16 = 0;
const NATIVE_ACTION_AIRBORNE: u16 = 1;
const NATIVE_ACTION_SHIELD: u16 = 2;
const NATIVE_ACTION_SPOT_DODGE: u16 = 4;
const NATIVE_ACTION_ROLL_OR_AIR_DODGE: u16 = 8;
const NATIVE_ACTION_GROUND_ATTACK: u16 = 16;
const NATIVE_ACTION_AERIAL_ATTACK: u16 = 32;
const NATIVE_ACTION_GRAB: u16 = 64;
const NATIVE_ACTION_DAMAGE_OR_TECH: u16 = 128;
const NATIVE_ACTION_LEDGE_ACTION: u16 = 256;
pub fn native_name(status: i32) -> &'static str {
    match status {
        crate::native_states::status::WAIT => "standing",
        crate::native_states::status::WALK..=crate::native_states::status::WALK_BRAKE => "walking",
        crate::native_states::status::DASH..=crate::native_states::status::TURN_RUN_BRAKE => {
            "running / turning"
        }
        crate::native_states::status::JUMP_SQUAT => "jump squat",
        crate::native_states::status::JUMP..=crate::native_states::status::FALL_SPECIAL => {
            "airborne"
        }
        crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV => "crouching",
        crate::native_states::status::LANDING
            ..=crate::native_states::status::LANDING_ATTACK_AIR => "landing",
        crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE => {
            "shield / shield stun"
        }
        crate::native_states::ESCAPE => "spot dodge",
        crate::native_states::ESCAPE_F..=crate::native_states::ESCAPE_B => "roll dodge",
        crate::native_states::ESCAPE_AIR..=crate::native_states::ESCAPE_AIR_SLIDE => "air dodge",
        crate::native_states::status::ATTACK => "jab",
        crate::native_states::status::ATTACK_100..=crate::native_states::status::ATTACK_DASH => {
            "jab follow-up / dash attack"
        }
        crate::native_states::status::ATTACK_S3 => "forward tilt",
        crate::native_states::status::ATTACK_HI3 => "up tilt",
        crate::native_states::status::ATTACK_LW3 => "down tilt",
        crate::native_states::status::ATTACK_S4_START..=crate::native_states::status::ATTACK_S4 => {
            "forward smash"
        }
        crate::native_states::status::ATTACK_LW4_START
            ..=crate::native_states::status::ATTACK_LW4 => "down smash",
        crate::native_states::status::ATTACK_HI4_START
            ..=crate::native_states::status::ATTACK_HI4 => "up smash",
        crate::native_states::status::ATTACK_AIR => "aerial attack",
        crate::native_states::CATCH => "standing grab startup",
        crate::native_states::CATCH_DASH => "dash grab startup",
        crate::native_states::CATCH_TURN => "pivot grab startup",
        crate::native_states::CATCH_PULL
        | crate::native_states::CATCH_DASH_PULL
        | crate::native_states::status::CATCH_WAIT..=crate::native_states::status::CATCH_JUMP => {
            "grab pull / hold / pummel"
        }
        crate::native_states::status::SHIELD_BREAK_FLY
            ..=crate::native_states::status::PASSIVE_CEIL => "knockdown / getup",
        crate::native_states::status::DAMAGE..=crate::native_states::status::DOWN_REFLECT_LR => {
            "damage / hitstun"
        }
        crate::native_states::status::CLIFF_CATCH_MOVE
            ..=crate::native_states::status::CLIFF_JUMP2 => "ledge action",
        _ => "other native action",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub winner: Option<Owner>,
    pub present: u32,
    pub committed: bool,
    pub requested: u32,
    /// Quantized directions only; analog jitter must not create log spam.
    pub stick: Option<[i8; 2]>,
    pub status: i32,
    pub motion: u64,
    pub native_frame: u16,
    pub stopped: bool,
    pub can_act: bool,
    pub hit: Option<bool>,
    pub valid: bool,
    /// Flags are the `SAFETY_*` constants in this module.
    pub safety: u8,
}
#[derive(Clone, Copy, Default)]
struct Evidence {
    proposed: bool,
    selected: u16,
    requests: u32,
    stick: Option<[i8; 2]>,
}
#[derive(Clone, Copy)]
pub struct Report {
    pub first: Snapshot,
    pub last: Snapshot,
    pub at_ms: u64,
    changes: u16,
    modes: [Evidence; OWNERS.len()],
    hit_seen: bool,
    hit_sampled: bool,
    native_actions: u16,
}
impl Report {
    fn new(snapshot: Snapshot, at_ms: u64) -> Self {
        Self {
            first: snapshot,
            last: snapshot,
            at_ms,
            changes: 0,
            modes: [Evidence::default(); OWNERS.len()],
            hit_seen: false,
            hit_sampled: false,
            native_actions: native_action(snapshot.status),
        }
    }
    fn observe(&mut self, s: Snapshot) {
        self.native_actions |= native_action(s.status);
        self.last = s;
        self.changes = self.changes.saturating_add(1);
        self.hit_seen |= s.hit == Some(true);
        self.hit_sampled |= s.hit.is_some();
        for (index, owner) in OWNERS.iter().enumerate() {
            let e = &mut self.modes[index];
            e.proposed |= s.present & (1 << index) != 0;
            if s.winner == Some(*owner) {
                e.selected = e.selected.saturating_add(1);
                e.requests |= s.requested;
                e.stick = s.stick;
            }
        }
    }
    /// Compact menu snapshot, formatted only by the background writer.
    pub fn card_text(&self) -> String {
        let s = self.last;
        let owner = s.winner.map_or("Physical controls", name);
        let state = if !s.valid {
            "Paused: gameplay/input observation unavailable"
        } else if s.stopped {
            "Native stop: retaining input; timers frozen"
        } else if s.requested != 0 {
            "Requesting input; waiting for native startup"
        } else if s.committed {
            "Sequence active: preparing or releasing input"
        } else if s.winner.is_some() {
            "Watching / preparing"
        } else {
            "No assisted input pending"
        };
        let mut text = format!(
            "LATEST SNAPSHOT +{}ms\n{}\n{}\nGame: {} f{}\n",
            self.at_ms,
            owner,
            state,
            motion_name(s.status, s.motion),
            s.native_frame
        );
        let waiting = OWNERS
            .iter()
            .filter(|owner| s.present & bit(**owner) != 0 && Some(**owner) != s.winner)
            .count();
        if waiting > 0 {
            let _ = writeln!(text, "Other proposals yielding priority: {waiting}");
        }
        let _ = writeln!(
            text,
            "Hit: {}. Full details: status.log",
            if self.hit_seen {
                "flag observed; victim unverified"
            } else {
                "not confirmed"
            }
        );
        text
    }
    /// Called by the background writer, never the input hook.
    pub fn format(&self) -> String {
        let mut text = String::with_capacity(REPORT_TEXT_CAPACITY_BYTES);
        let last = self.last;
        let winner = last.winner.map_or("physical controls", name);
        let _ = writeln!(
            text,
            "MODE EXPLANATION +{}ms ({} changes batched)",
            self.at_ms, self.changes
        );
        let why = if !last.valid {
            "assistance paused: observation or gameplay scope is unavailable"
        } else if last.winner == Some(Owner::Safety) {
            "native tech, mash, SDI or launch DI has emergency priority"
        } else if last.committed {
            "a committed sequence retains its input and release frames"
        } else if last.winner == Some(Owner::Ledge) && last.requested == 0 {
            "ledge watch/preparation is reserved ahead of optional helpers"
        } else if last.winner == Some(Owner::RollRead) && last.requested == 0 {
            "roll-read preparation owns the stick before requesting an attack"
        } else if last.winner == Some(Owner::Posture) {
            "no higher-priority mode needs control; this helper only aims the stick, leaving attack buttons physical"
        } else if last.winner.is_some() {
            "this eligible proposal won input priority"
        } else {
            "no mode currently proposes an eligible input"
        };
        let _ = writeln!(text, "Now: {winner}. Why: {why}.");
        if self.first.winner != last.winner {
            let _ = writeln!(text, "Control changed: {} -> {}. Earlier control ended or yielded to priority; this alone does not establish a hit or miss.", self.first.winner.map_or("physical controls", name), winner);
        }

        let _ = writeln!(
            text,
            "Fighter: {} -> {} (status {:#x}); {}.",
            native_name(self.first.status),
            native_name(last.status),
            last.status,
            if last.stopped {
                "native stop is active; timers must not advance"
            } else if last.can_act {
                "ready for a new grounded action"
            } else {
                "new grounded actions are unavailable during this animation"
            }
        );
        let _ = writeln!(
            text,
            "Latest animation: {} f{} (motion {:#x}).",
            motion_name(last.status, last.motion),
            last.native_frame,
            last.motion
        );
        let observed: Vec<_> = [
            (NATIVE_ACTION_AIRBORNE, "jump/fall"),
            (NATIVE_ACTION_SHIELD, "shield"),
            (NATIVE_ACTION_SPOT_DODGE, "spot dodge"),
            (NATIVE_ACTION_ROLL_OR_AIR_DODGE, "roll/air dodge"),
            (NATIVE_ACTION_GROUND_ATTACK, "ground attack"),
            (NATIVE_ACTION_AERIAL_ATTACK, "aerial attack"),
            (NATIVE_ACTION_GRAB, "grab/capture"),
            (NATIVE_ACTION_DAMAGE_OR_TECH, "damage/knockdown"),
            (NATIVE_ACTION_LEDGE_ACTION, "ledge action"),
        ]
        .into_iter()
        .filter_map(|(mask, label)| (self.native_actions & mask != 0).then_some(label))
        .collect();
        let _ = writeln!(text, "Game states observed in batch: {}. These are observations, not proof a particular request caused them.", if observed.is_empty() { "no action startup sampled".to_owned() } else { observed.join(", ") });
        for (index, owner) in OWNERS.iter().enumerate() {
            let e = self.modes[index];
            if !e.proposed && e.selected == 0 {
                continue;
            }
            let _ = writeln!(text, "  {} — {}", name(*owner), purpose(*owner));
            let _ = write!(text, "    Input: ");
            if e.selected == 0 {
                let _ = writeln!(
                    text,
                    "{}",
                    if !last.valid {
                        "proposed, but never applied: gameplay or observation scope was unavailable."
                    } else {
                        "proposed, but never applied in this batch; another mode retained priority."
                    }
                );
            } else {
                let _ = write!(text, "selected; requested button types during batch: ");
                if e.requests == 0 {
                    let _ = write!(text, "no action buttons (preparation, hold or release)");
                } else {
                    let mut separator = "";
                    for (mask, label) in [
                        (MAPPER_X | MAPPER_ZL, "Jump"),
                        (MAPPER_A, "Attack"),
                        (MAPPER_R, "Grab"),
                        (MAPPER_B, "Special"),
                        (MAPPER_ZR, "Shield"),
                    ] {
                        if e.requests & mask != 0 {
                            let _ = write!(text, "{separator}{label}");
                            separator = ", ";
                        }
                    }
                    let _ = write!(text, " (raw {:#x})", e.requests);
                }
                let _ = write!(text, ". Stick: ");
                match e.stick {
                    None => {
                        let _ = write!(text, "physical");
                    }
                    Some([x, y]) => {
                        let horizontal = match x {
                            -1 => "left",
                            1 => "right",
                            _ => "neutral horizontal",
                        };
                        let vertical = match y {
                            -1 => "down",
                            1 => "up",
                            _ => "neutral vertical",
                        };
                        let _ = write!(text, "{horizontal}, {vertical}");
                    }
                }
                let _ = writeln!(text, ".");
            }
        }
        if last.safety != 0 {
            let _ = writeln!(
                text,
                "Safety flags: tech={}, mash={}, SDI={}, DI={}.",
                last.safety & SAFETY_TECH_GUARD != 0,
                last.safety & SAFETY_MASH != 0,
                last.safety & SAFETY_SDI != 0,
                last.safety & SAFETY_DI != 0
            );
        }
        let _ = write!(text, "Evidence: {}. Requests above describe inputs sent before mapping; they do not prove the selected move started or hit. Button names are a union across the batch, not necessarily simultaneous presses. Native state is the latest copied observation. +ms is wall-clock time, not a simulation-frame count.",
            if self.hit_seen { "a hit flag was observed; victim and responsible mode are unverified" }
            else if !self.hit_sampled { "hit evidence was not sampled for these observations" }
            else { "no hit flag was observed in this batch" });
        text
    }
}

pub struct Recorder {
    previous: Option<Snapshot>,
    pending: Option<Report>,
    last_ms: Option<u64>,
}
impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}
impl Recorder {
    pub const fn new() -> Self {
        Self {
            previous: None,
            pending: None,
            last_ms: None,
        }
    }
    /// At most four fixed-size batches per second. Unchanged polls do no work.
    pub fn sample(&mut self, snapshot: Snapshot, ms: u64) -> Option<Report> {
        if self.last_ms.is_some_and(|last| ms < last) {
            *self = Self::new();
        }
        let relevant = snapshot.present != 0
            || snapshot.winner.is_some()
            || self
                .previous
                .is_some_and(|p| p.present != 0 || p.winner.is_some());
        // Animation frame numbers enrich reports but must not make a stable
        // crouch/watch mode produce four log messages every second forever.
        let changed = self.previous.map(|mut previous| {
            previous.native_frame = snapshot.native_frame;
            previous
        }) != Some(snapshot);
        if relevant && changed {
            self.pending
                .get_or_insert_with(|| Report::new(self.previous.unwrap_or(snapshot), ms))
                .observe(snapshot);
        }
        self.previous = Some(snapshot);
        if self
            .last_ms
            .is_none_or(|last| ms.saturating_sub(last) >= MIN_REPORT_INTERVAL_MS)
        {
            if let Some(report) = self.pending.take() {
                self.last_ms = Some(ms);
                return Some(Report {
                    at_ms: ms,
                    ..report
                });
            }
        }
        None
    }
}

/// Keep completed results below the menu; hide rather than overlap if it fills the screen.
pub fn left_result_space(display_height: f32, scale: f32, menu_height: f32) -> Option<(f32, f32)> {
    if !display_height.is_finite()
        || !scale.is_finite()
        || !menu_height.is_finite()
        || display_height <= 0.0
        || scale <= 0.0
        || menu_height < 0.0
    {
        return None;
    }
    let y = RESULT_TOP_MARGIN * scale + menu_height;
    let remaining = display_height - y - RESULT_BOTTOM_MARGIN * scale;
    (remaining >= MIN_RESULT_HEIGHT * scale).then_some((y, remaining))
}

/// A copied menu name survives selected-fighter menu teardown without leaking
/// strings. Configuration/report updates allocate no label storage on frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Label {
    bytes: [u8; LABEL_CAPACITY_BYTES],
    len: usize,
}
impl Label {
    pub fn new(text: &str) -> Self {
        let mut len = text.len().min(LABEL_CAPACITY_BYTES);
        while !text.is_char_boundary(len) {
            len -= 1;
        }
        let mut label = Self {
            bytes: [0; LABEL_CAPACITY_BYTES],
            len,
        };
        label.bytes[..len].copy_from_slice(&text.as_bytes()[..len]);
        label
    }
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Configuration {
    pub fighter: &'static str,
    pub rows: [Label; CONFIGURATION_ROW_COUNT],
    pub auto_reflect: bool,
}
impl Configuration {
    pub fn new(fighter: &'static str, rows: [&str; CONFIGURATION_ROW_COUNT]) -> Self {
        Self {
            fighter,
            rows: rows.map(Label::new),
            auto_reflect: false,
        }
    }
    pub fn with_reflect(mut self, enabled: bool) -> Self {
        self.auto_reflect = enabled;
        self
    }
    pub fn format(&self) -> String {
        let mut text = format!("SAVED MODES for {} (changed configuration)\n", self.fighter);
        if self.fighter == "mr_game_and_watch" {
            let _=writeln!(text,"  Auto-reflect: {}. Native-ready projectile defense only; full bucket and ledge mode suspend it.",if self.auto_reflect {"ON"}else{"OFF"});
        }
        for (label, value) in [
            "Neutral getup",
            "Jump getup",
            "Ledge roll",
            "Getup attack",
            "Ledge drop",
            "Onstage roll read",
            "Y binding",
            "Auto mash",
            "Idle crouch",
            "Turnaround tilt",
            "Yoshi jab lock",
            "Spot-dodge punish",
            "Grab dodge",
            "Auto tech",
            "Ledge trump",
            "Performance gauge",
            "X button recipe",
            "Y button recipe",
            "D-pad Up binding",
            "D-pad Down binding",
            "D-pad Left binding",
            "D-pad Right binding",
            "Ledge trap mode",
            "Out of shield",
            "Shield pressure",
        ]
        .iter()
        .zip(self.rows)
        {
            let help = match *label {
                "Neutral getup" | "Jump getup" | "Ledge roll" | "Getup attack" | "Ledge drop" => "Hold the ledge tool; waits for the matching ledge option, safe spacing and protection to end.",
                "Shield pressure" => "Watches enemy shields and shield-origin rolls; selects a configured pressure response without interrupting protected ledge actions.",
                "Onstage roll read" => "Watches approaching/crossing ground rolls; chooses the configured punish inside recovery and reach.",
                "Y binding" => "Existing Y mode, used while Y sequence is set to Native.",
                "X sequence" | "Y sequence" => "An assigned recipe runs once per fresh button press while the menu is closed; Native preserves the existing control.",
                "Auto mash" => "Acts only during supported capture/mash states, then returns physical control.",
                "Idle crouch" => "Grounded idle within 45 horizontal / 20 vertical units of an enemy; movement, actions or held L suspend crouching.",
                "Turnaround tilt" => "Aims a physical down tilt toward the enemy; does not generate an attack press.",
                "Yoshi jab lock" => "Requires an attributed down-tilt hit, native knockdown and legal lock/recovery window.",
                "Spot-dodge punish" => "Reads enemy spot-dodge recovery; selects near down smash or farther forward smash.",
                _ => "Detects ordinary grab threats, requests spot dodge, then watches recovery for a smash punish.",
            };
            let _ = writeln!(text, "  {label}: {value}\n    {help}");
        }
        text.push_str("Priority: emergency safety > spot/grab/roll reads > ledge sequences > other punishes/tools > defensive helpers > turbo > crouch/tilt. Saved choices are available options; an eligible trigger and native state are still required.");
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_RECORDER_SIZE_BYTES: usize = 1024;
    const MAX_REPORT_SIZE_BYTES: usize = 768;

    #[test]
    fn copied_labels_survive_menu_teardown_and_truncate_on_utf8_boundaries() {
        let name = "é".repeat(40);
        let label = Label::new(&name);
        drop(name);
        assert_eq!(label.as_str(), "é".repeat(32));
        assert_eq!(label.to_string(), label.as_str());
        assert!(Label::new("").is_empty());
        let menu_name = String::from("Saved custom option");
        let configuration =
            Configuration::new("Bowser", [menu_name.as_str(); CONFIGURATION_ROW_COUNT]);
        drop(menu_name);
        assert_eq!(configuration.rows[0].as_str(), "Saved custom option");
    }
    fn snapshot() -> Snapshot {
        Snapshot {
            winner: Some(Owner::Ledge),
            present: bit(Owner::Ledge) | bit(Owner::Posture),
            committed: false,
            requested: 0,
            stick: None,
            status: crate::native_states::status::WAIT,
            motion: 1,
            native_frame: 0,
            stopped: false,
            can_act: true,
            hit: Some(false),
            valid: true,
            safety: 0,
        }
    }
    #[test]
    fn snapshots_explain_every_owner_and_preserve_intermediate_native_startup() {
        for owner in OWNERS {
            let mut r = Recorder::new();
            let s = Snapshot {
                winner: Some(owner),
                present: bit(owner),
                ..snapshot()
            };
            let report = r.sample(s, 0).unwrap();
            assert!(report.format().contains(purpose(owner)));
            assert!(report.card_text().contains(name(owner)));
        }
        let mut r = Recorder::new();
        r.sample(snapshot(), 0);
        r.sample(
            Snapshot {
                status: crate::native_states::status::ATTACK_AIR,
                motion: crate::timing::motion_hash("attack_air_f"),
                native_frame: 10,
                ..snapshot()
            },
            10,
        );
        let report = r.sample(snapshot(), 250).unwrap();
        assert!(report.format().contains("aerial attack"));
        assert!(report.format().contains("not proof"));
        assert!(report.card_text().contains("Hit: not confirmed"));
        assert_eq!(report.at_ms, 250);
        assert_eq!(
            motion_name(
                crate::native_states::status::ATTACK_AIR,
                crate::timing::motion_hash("attack_air_lw"),
            ),
            "down air"
        );
    }
    #[test]
    fn animation_frame_only_changes_do_not_spam_stable_modes() {
        let mut r = Recorder::new();
        r.sample(snapshot(), 0);
        for frame in 1..1000 {
            assert!(r
                .sample(
                    Snapshot {
                        native_frame: frame,
                        ..snapshot()
                    },
                    u64::from(frame)
                )
                .is_none());
        }
    }
    #[test]
    fn duplicate_polls_are_free_and_batches_have_a_hard_rate_limit() {
        let mut recorder = Recorder::new();
        assert!(recorder.sample(snapshot(), 0).is_some());
        for ms in 1..250 {
            assert!(recorder.sample(snapshot(), ms).is_none());
        }
        let mut count = 1;
        for ms in 250..=1250 {
            let s = Snapshot {
                requested: if ms % 2 == 0 { MAPPER_A } else { 0 },
                ..snapshot()
            };
            count += usize::from(recorder.sample(s, ms).is_some());
        }
        assert_eq!(count, 6);
        assert!(recorder.sample(snapshot(), 0).is_some());
    }
    #[test]
    fn buffered_press_survives_release_in_the_same_batch_without_claiming_a_hit() {
        let mut r = Recorder::new();
        r.sample(snapshot(), 0);
        r.sample(
            Snapshot {
                requested: MAPPER_X | MAPPER_A,
                committed: true,
                ..snapshot()
            },
            1,
        );
        r.sample(
            Snapshot {
                status: crate::native_states::status::JUMP_SQUAT,
                can_act: false,
                ..snapshot()
            },
            16,
        );
        let report = r.sample(snapshot(), 250).unwrap().format();
        assert!(report.contains("Jump, Attack"));
        assert!(report.contains("never applied"));
        assert!(report.contains("do not prove"));
        assert!(report.contains("no hit flag"));
    }
    #[test]
    fn every_owner_and_configured_row_has_a_readable_label() {
        assert_eq!(
            OWNERS.iter().fold(0, |mask, owner| mask | bit(*owner)),
            (1u32 << OWNERS.len()) - 1
        );
        for owner in OWNERS {
            assert!(!name(owner).is_empty());
        }
        let text = Configuration::new("yoshi", ["OFF"; CONFIGURATION_ROW_COUNT]).format();
        assert!(text.contains("Neutral getup: OFF"));
        assert!(text.contains("Yoshi jab lock: OFF"));
    }
    #[test]
    fn configuration_rows_remain_aligned_after_removing_jab_grab() {
        let rows = [
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
            "16", "17", "18", "19", "20", "21", "22", "23", "24",
        ];
        let text = Configuration::new("yoshi", rows).format();
        for expected in [
            "Turnaround tilt: 9",
            "Yoshi jab lock: 10",
            "Spot-dodge punish: 11",
            "X button recipe: 16",
            "D-pad Right binding: 21",
            "Ledge trap mode: 22",
            "Out of shield: 23",
            "Shield pressure: 24",
        ] {
            assert!(text.contains(expected), "missing {expected}");
        }
    }
    #[test]
    fn left_result_stacks_below_menu_or_hides_instead_of_overlapping() {
        for height in [480.0_f32, 720.0, 1080.0, 2160.0] {
            let scale = (height / 720.0).clamp(0.65, 3.0);
            let menu = 330.0 * scale;
            let (y, available) = left_result_space(height, scale, menu).unwrap();
            assert!(y > 8.0 * scale + menu);
            assert!(y + available <= height);
            assert!(left_result_space(height, scale, height).is_none());
        }
        assert!(left_result_space(f32::NAN, 1.0, 100.0).is_none());
    }
    #[test]
    fn disabled_modes_do_not_log_manual_play_and_evidence_storage_is_bounded() {
        let mut r = Recorder::new();
        for ms in 0..1000 {
            let idle = Snapshot {
                winner: None,
                present: 0,
                status: ms as i32,
                ..snapshot()
            };
            assert!(r.sample(idle, ms).is_none());
        }
        assert!(std::mem::size_of::<Recorder>() <= MAX_RECORDER_SIZE_BYTES);
        assert!(std::mem::size_of::<Report>() <= MAX_REPORT_SIZE_BYTES);
    }
    #[test]
    fn unknown_hit_evidence_and_invalid_scope_are_explained_without_inventing_a_blocker() {
        let mut r = Recorder::new();
        let report = r
            .sample(
                Snapshot {
                    winner: None,
                    valid: false,
                    hit: None,
                    ..snapshot()
                },
                0,
            )
            .unwrap()
            .format();
        assert!(report.contains("scope was unavailable"));
        assert!(report.contains("hit evidence was not sampled"));
        assert!(!report.contains("another mode retained priority"));
    }
}
