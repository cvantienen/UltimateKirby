//! Diagnostic-only frozen ledge results. Observers collect copied facts in fixed
//! storage; the logger worker formats/publishes once when an attempt ends.
//! Technical: Records bounded ledge-response observations and derives diagnostic
//! calibration summaries from copied samples.
//! Plain English: This module saves troubleshooting evidence about ledge behavior.
use kirby_assist_core::{
    ledge_trap::{CalibrationFrame, CalibrationKind, CalibrationMove},
    manual_controls::RIGHT_STICK_DIRECTIONS,
    native_input::{self, MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_X, MAPPER_ZL, MAPPER_ZR},
};
#[cfg(feature = "imgui-menu")]
use std::{
    ffi::{CStr, CString},
    sync::Mutex,
};
use std::{
    fmt::Write,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

/// Technical: Generation counter invalidating stale rendered calibration cards.
/// Plain English: Identifies which calibration result the menu currently shows.
static EPOCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "imgui-menu")]
/// Technical: Mutex-protected cached calibration-card text and its generation.
/// Plain English: Stores the troubleshooting card displayed in the menu.
static CARD: Mutex<Option<(usize, CString)>> = Mutex::new(None);
/// Technical: Time limit after which an unfinished trace is considered stalled.
/// Plain English: Ends a calibration attempt after four seconds without completion.
const WATCHDOG_MS: u64 = 4_000;
/// Technical: Raw right-stick direction mask used for calibration classification.
/// Plain English: Identifies the right-stick directions tracked by the recorder.
const RAW_CSTICK: u32 = RIGHT_STICK_DIRECTIONS;
/// Technical: Stable names corresponding to the five recorded input channels.
/// Plain English: Labels the jump, attack, grab, special, and shield inputs in reports.
const CHANNELS: [&str; 5] = ["Jump", "Attack", "Grab", "Special", "Shield"];

#[derive(Clone, Copy, Debug)]
/// Technical: Compact timestamped copy of own/target animation facts.
/// Plain English: Stores a moment in the attempt using copied animation details.
struct Stamp {
    /// Technical: Elapsed wall-clock milliseconds from the report origin.
    /// Plain English: How long after the attempt began this sample was taken.
    ms: u64,
    /// Technical: Target's native status ID, if present.
    /// Plain English: The opponent's action at this sample.
    target_status: Option<i32>,
    /// Technical: Target motion hash, if present.
    /// Plain English: The opponent's animation at this sample.
    target_motion: Option<u64>,
    /// Technical: Target's fractional motion frame, if present.
    /// Plain English: How far the opponent's animation had progressed.
    target_frame: Option<f32>,
    /// Technical: Target's motion playback rate, if present.
    /// Plain English: How quickly the opponent's animation was playing.
    target_rate: Option<f32>,
    /// Technical: Own fighter's native status ID.
    /// Plain English: The player's action at this sample.
    own_status: i32,
    /// Technical: Own fighter's current motion frame.
    /// Plain English: How far the player's animation had progressed.
    own_frame: f32,
}
impl Stamp {
    /// Technical: Copies timing/status facts from a calibration frame.
    /// Plain English: Creates a compact record of what both fighters were doing.
    fn new(frame: CalibrationFrame, ms: u64) -> Self {
        Self {
            ms,
            target_status: frame.enemy.map(|e| e.status),
            target_motion: frame.enemy.and_then(|e| e.motion.map(|m| m.kind)),
            target_frame: frame.enemy.and_then(|e| e.motion.map(|m| m.frame)),
            target_rate: frame.enemy.and_then(|e| e.motion.map(|m| m.rate)),
            own_status: frame.context.status,
            own_frame: frame.context.motion_frame,
        }
    }
    /// Technical: Formats the target's copied status and motion clock.
    /// Plain English: Produces a short readable timestamp for the report.
    fn describe(self) -> String {
        let frame = self
            .target_frame
            .map_or_else(|| "?".into(), |f| format!("{f:.1}"));
        let status = self
            .target_status
            .map_or_else(|| "?".into(), |s| format!("{s:#x}"));
        let rate = self
            .target_rate
            .map_or_else(|| "?".into(), |v| format!("{v:.2}"));
        format!("target {status} f{frame} r{rate} (+{}ms)", self.ms)
    }
}

#[derive(Clone, Copy, Debug)]
/// Technical: Captures neutral-attack activity and contemporaneous opponent spacing/protection.
/// Plain English: Records what the player and opponent were doing when a neutral move became active.
struct NeutralActivity {
    /// Technical: Timestamp and animation snapshot for this activity sample.
    /// Plain English: When the activity happened and which animation was playing.
    stamp: Stamp,
    /// Technical: Own fighter's motion hash at the sample.
    /// Plain English: The player's animation during the activity.
    own_motion: u64,
    /// Technical: Opponent-relative X/Y position delta, if a target was observed.
    /// Plain English: How far away and above/below the opponent was.
    gap: Option<[f32; 2]>,
    /// Technical: Whether the target was vulnerable at the sample.
    /// Plain English: Whether the opponent could be hit then.
    vulnerable: Option<bool>,
    /// Technical: Remaining protection duration, if available.
    /// Plain English: How long the opponent's protection was expected to last.
    protection: Option<f32>,
    /// Technical: Remaining recovery duration for a cliff-climb target, if applicable.
    /// Plain English: How long the opponent had before they could act.
    recovery: Option<f32>,
}
impl NeutralActivity {
    /// Technical: Copies neutral activity and target spacing/protection from a frame.
    /// Plain English: Captures the fighter and opponent state for this moment.
    fn new(frame: CalibrationFrame, stamp: Stamp) -> Self {
        Self {
            stamp,
            own_motion: frame.context.motion_kind,
            gap: frame.enemy.map(|e| {
                [
                    e.position[0] - frame.context.position[0],
                    e.position[1] - frame.context.position[1],
                ]
            }),
            vulnerable: frame.enemy.map(|e| e.vulnerable),
            recovery: frame.enemy.and_then(|e| {
                if e.status == kirby_assist_core::ledge_trap::CLIFF_CLIMB {
                    e.actionable_in
                } else {
                    None
                }
            }),
            protection: frame.enemy.and_then(|e| {
                if e.vulnerable {
                    Some(0.0)
                } else {
                    e.vulnerable_in
                }
            }),
        }
    }
    /// Technical: Formats copied animation, vulnerability, timing, and spacing values.
    /// Plain English: Turns the activity snapshot into one readable report line.
    fn describe(self) -> String {
        format!(
            "own {} f{:.1}; {}; vulnerable={:?}; protection_left={}; recovery_left={}; gap_x/height={:?}",
            kirby_assist_core::diagnostics::modes::motion_name(
                self.stamp.own_status,
                self.own_motion
            ),
            self.stamp.own_frame,
            self.stamp.describe(),
            self.vulnerable,
            number(self.protection),
            number(self.recovery),
            self.gap
        )
    }
}

use kirby_assist_core::diagnostics::modes::Label as ChoiceName;
/// Technical: Frozen diagnostic record for one selected fighter option attempt.
/// Plain English: All copied details used to explain how one test went.
pub struct Report {
    /// Technical: Recorder generation captured when this attempt began.
    /// Plain English: Identifies the calibration run that produced this report.
    epoch: usize,
    /// Technical: Stable fighter profile key for the observed character.
    /// Plain English: The character name used in the report.
    fighter: &'static str,
    /// Technical: Native fighter-kind ID corresponding to `fighter`.
    /// Plain English: The game's numeric ID for the character.
    fighter_kind: i32,
    /// Technical: Human-readable selected option label.
    /// Plain English: The menu choice the player tested.
    choice: ChoiceName,
    /// Technical: Wall-clock start instant used to compute elapsed time.
    /// Plain English: When this attempt began.
    origin: Instant,
    /// Technical: First copied calibration frame for the attempt.
    /// Plain English: The fighter and target state when recording started.
    first: CalibrationFrame,
    /// Technical: Most recent copied calibration frame.
    /// Plain English: The latest fighter and target state recorded.
    last: CalibrationFrame,
    /// Technical: Timestamp derived from the first frame and elapsed time.
    /// Plain English: The starting time and animation details in compact form.
    first_stamp: Stamp,
    /// Technical: Timestamp derived from the latest frame and elapsed time.
    /// Plain English: The most recent time and animation details.
    last_stamp: Stamp,
    /// Technical: Resolved move timing/configuration associated with the attempt.
    /// Plain English: The move recipe being tested.
    model: CalibrationMove,
    /// Technical: First timestamped request observed for each input channel.
    /// Plain English: When each button request was first made.
    requests: [Option<Stamp>; 5],
    /// Technical: Whether each requested channel was acknowledged by the mapper.
    /// Plain English: Which requested inputs were actually mapped.
    mapped: [bool; 5],
    /// Technical: First mapper-acknowledgment timestamp for each input channel.
    /// Plain English: When each input first appeared in mapped output.
    mapped_first: [Option<Stamp>; 5],
    /// Technical: Optional input-to-mapper latency sample in milliseconds.
    /// Plain English: Measured delay for the diagnostic run, if available.
    test_latency: Option<u8>,
    /// Technical: Performance-recorder epoch used for this latency sample.
    /// Plain English: Identifies which performance sample the delay came from.
    test_epoch: usize,
    /// Technical: Calibration frame at the first accepted raw request.
    /// Plain English: The fighter state when the first input was requested.
    first_request: Option<CalibrationFrame>,
    /// Technical: Timestamp when native jump/squat startup was observed.
    /// Plain English: When the game began the jump action.
    jump_start: Option<Stamp>,
    /// Technical: Timestamp when the selected native move started.
    /// Plain English: When the game began the chosen move.
    move_start: Option<Stamp>,
    /// Technical: Optional own animation frame when the configured move became active.
    /// Plain English: The move's animation frame when activity was observed.
    activity_frame: Option<f32>,
    /// Technical: First copied snapshot of a neutral attack's active state.
    /// Plain English: The first time the neutral attack appeared active.
    neutral_first_active: Option<NeutralActivity>,
    /// Technical: First active neutral-attack snapshot while the target was vulnerable.
    /// Plain English: When the move was active while the opponent could be hit.
    neutral_vulnerable_active: Option<NeutralActivity>,
    /// Technical: First timestamp at which the target was observed vulnerable.
    /// Plain English: When the opponent's protection was first seen to end.
    vulnerable: Option<Stamp>,
    /// Technical: First observed hit-infliction timestamp.
    /// Plain English: When the game first reported a hit.
    hit: Option<Stamp>,
    /// Technical: Whether native capture was observed.
    /// Plain English: Whether the game confirmed a grab.
    captured: bool,
    /// Technical: Whether native shield block was observed.
    /// Plain English: Whether the attack was seen to be blocked.
    blocked: bool,
    /// Technical: Last native or policy gate reason recorded.
    /// Plain English: The latest reason the move was waiting or stopped.
    last_wait: &'static str,
    /// Technical: Last mapper admission gate label.
    /// Plain English: The latest reason input did or did not pass mapping.
    mapper_gate: &'static str,
    /// Technical: Raw requested canonical button mask.
    /// Plain English: Buttons the helper asked to press.
    raw_requested: u32,
    /// Technical: Button mask admitted by the overlay before native mapping.
    /// Plain English: Buttons that made it into the temporary controller input.
    raw_applied: u32,
    /// Technical: Final mapped button mask captured by the recorder.
    /// Plain English: Buttons the game mapper produced.
    mapped_buttons: u32,
    /// Technical: Stable completion reason for this attempt.
    /// Plain English: Why the recording ended.
    end: &'static str,
    /// Technical: Whether the selected action was observed as native execution.
    /// Plain English: Whether the game actually started the requested move.
    was_native: bool,
    /// Technical: Optional finer-grained end reason for the selected timeline.
    /// Plain English: An extra detail about why the sequence ended.
    ending: Option<&'static str>,
}
/// Worker-only explanation; retain unknown gate names with readable spacing.
/// Technical: Maps known gate identifiers to readable phrases and normalizes unknown labels.
/// Plain English: Rewrites internal wait reasons into clearer report text.
fn readable_gate(gate: &str) -> std::borrow::Cow<'_, str> {
    let known = match gate {
        "watch_ledge" => "Watching for an opponent ledge option",
        "native stop" => "Hitstop/native stop: waiting without spending timing frames",
        "native control unavailable" => "Current game state does not permit control",
        "normal_getup_nair" => "Timing neutral air against neutral getup",
        "prepare_neutral_air" => "Preparing jump and neutral-air selection",
        "short_hop_neutral_air" => "Delivering the short-hop neutral-air route",
        "roll_wait_grounded" => "Waiting to be grounded before punishing the roll",
        "roll_punish_window_missed" => "Recovery window expired; avoiding a late punish",
        "roll_close_attack_range" => "Walking into the predicted roll-punish range",
        "roll_brake_before_destination" | "roll_release_approach" => {
            "Releasing movement before the punish input"
        }
        "roll_face_grab" => "Turning toward the projected grab contact point",
        "roll_wait_smash_ready" => "Waiting for native down-smash readiness",
        "roll_wait_tilt_ready" => "Waiting for native down-tilt readiness",
        "roll_wait_attack_release" => "Releasing attack so the next press has a fresh edge",
        "shield_wait_for_block" => "Holding shield while waiting for a block",
        "shield_grab_after_stun" => "Waiting for shield stun/recovery before grab",
        "finished" => "Attempt finished; returning control",
        "not observed" => "No mapper delivery sample was recorded",
        _ => return gate.replace('_', " ").into(),
    };
    known.into()
}
impl Report {
    /// Technical: Classifies the strongest observed result from capture, hit, block,
    /// native start, and input evidence.
    /// Plain English: Summarizes what the game appeared to do during the attempt.
    fn result(&self) -> &'static str {
        if self.captured {
            "CAPTURE confirmed"
        } else if self.hit.is_some() {
            "HIT observed; victim unverified"
        } else if self.blocked && self.model.kind == CalibrationKind::Shield {
            "BLOCK observed"
        } else if self.move_start.is_some() && self.model.kind == CalibrationKind::Shield {
            "SHIELD observed; no block confirmed"
        } else if self.neutral_first_active.is_some() {
            "ACTIVE attack observed; hit not confirmed"
        } else if self.move_start.is_some() {
            "START observed; hit not confirmed"
        } else if self.action_requested() {
            "REQUESTED; no native move observed"
        } else if self.raw_requested != 0 && self.model.kind == CalibrationKind::Unknown {
            "INPUT requested; move model unavailable"
        } else {
            "NO MOVE REQUEST; see last gate"
        }
    }

    /// Technical: Returns a caveated narrative consistent with the available native evidence.
    /// Plain English: Explains what the result can and cannot confirm.
    fn explanation(&self) -> &'static str {
        if self.captured {
            "The game confirmed a capture."
        } else if self.hit.is_some() {
            "A hit flag was observed; the victim is not verified."
        } else if self.blocked {
            "A block was observed; this is not an attack hit."
        } else if self.neutral_first_active.is_some() {
            "A native active attack was observed after the neutral recipe input. Compare protection and spacing below; hitbox/hurtbox overlap is not measured."
        } else if self.move_start.is_some() {
            "The selected move started, but no hit was confirmed. Timing or spacing may explain the miss; the log cannot prove which."
        } else if self.model.kind == CalibrationKind::Unknown && self.raw_requested != 0 {
            "This raw-input recipe has no selected-move model. Check raw/applied/mapped evidence below; missing readiness or optional opponent samples do not prove an input was skipped."
        } else if self.action_requested() && self.action_mapped() {
            "Input reached mapping, but the selected move was not observed before this attempt ended. Check the last native state and interruption reason."
        } else if self.action_requested() {
            "A move was requested, but mapped delivery was not observed. Check the mapper gate below."
        } else if self.jump_start.is_some() {
            "A jump was observed without a confirmed selected move. Check the attack channel and last waiting gate."
        } else if self.last.context.stopped {
            "Native stop froze progression; the controller waited instead of spending timing frames."
        } else if !self.last.context.can_control {
            "Native control was unavailable, so the controller did not start a move."
        } else if self.last.enemy.is_none() {
            "No opponent was available in the last observation; no selected move was requested."
        } else if !self.last.context.can_act {
            "A new grounded action was unavailable in the last native state."
        } else {
            "No selected move was requested. Check the last gate, protection countdown and spacing below for the unsatisfied conditions."
        }
    }

    /// Compact frozen summary; all formatting happens on the logger worker.
    /// Plain English: Prepares a short copy of the calibration result for the menu card.
    #[cfg(any(feature = "imgui-menu", test))]
    fn card_text(&self) -> String {
        let mut text = String::with_capacity(768);
        let _ = writeln!(
            text,
            "{} | {:?} > {}",
            self.fighter, self.first.slot, self.choice
        );
        let _ = writeln!(text, "{}", self.result());
        let _ = writeln!(text, "Why: {}", self.explanation());
        let at =
            |stamp: Option<Stamp>| stamp.map_or_else(|| "?".into(), |s| format!("+{}ms", s.ms));
        let _ = writeln!(
            text,
            "Jump: {} | move: {}",
            at(self.jump_start),
            at(self.move_start)
        );
        let _ = writeln!(
            text,
            "Protection off: {} | hit: {}",
            at(self.vulnerable),
            at(self.hit)
        );
        let _ = writeln!(
            text,
            "Last: {} f{:.1}",
            kirby_assist_core::diagnostics::modes::native_name(self.last.context.status),
            self.last.context.motion_frame
        );
        let _ = writeln!(text, "Wait: {}", readable_gate(self.last_wait));
        let _ = writeln!(text, "End: {}", self.end.replace('_', " "));
        text.push_str("? = unobserved. Full evidence: status.log");
        text
    }

    /// Technical: Converts an instant into saturated milliseconds from this report's origin.
    /// Plain English: Measures how many milliseconds have passed since recording began.
    fn elapsed(&self, now: Instant) -> u64 {
        now.saturating_duration_since(self.origin)
            .as_millis()
            .min(u128::from(u64::MAX)) as u64
    }
    /// Technical: Checks whether the input channel matching the configured move was requested.
    /// Plain English: Tells whether the relevant button was ever requested.
    fn action_requested(&self) -> bool {
        match self.model.kind {
            CalibrationKind::Attack => self.requests[1].is_some(),
            CalibrationKind::Grab | CalibrationKind::Zair => self.requests[2].is_some(),
            CalibrationKind::Special => self.requests[3].is_some(),
            CalibrationKind::Shield => self.requests[4].is_some(),
            CalibrationKind::Unknown => false,
        }
    }
    /// Technical: Checks whether the input channel matching the configured move was mapped.
    /// Plain English: Tells whether the relevant button reached the mapped output.
    fn action_mapped(&self) -> bool {
        match self.model.kind {
            CalibrationKind::Attack => self.mapped[1],
            CalibrationKind::Grab | CalibrationKind::Zair => self.mapped[2],
            CalibrationKind::Special => self.mapped[3],
            CalibrationKind::Shield => self.mapped[4],
            CalibrationKind::Unknown => false,
        }
    }
    /// Worker thread only. Native frames and wall time are kept separate.
    /// Plain English: Turns the saved calibration into a full readable report.
    pub fn format(&self) -> String {
        let mut text = String::with_capacity(3_072);
        let label = match self.first.slot {
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral => "NEUTRAL",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Jump => "JUMP",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Roll => "ROLL",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Attack => "ATTACK",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Drop => "DROP",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::X => "X BUTTON",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Y => "Y BUTTON",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadUp => "D-PAD UP",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadDown => "D-PAD DOWN",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadLeft => "D-PAD LEFT",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::DpadRight => "D-PAD RIGHT",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::L3 => "L3",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::R3 => "R3",
        };
        let result = self.result();
        let _ = writeln!(text, "What happened: {result}\nWhy: {}", self.explanation());
        let _ = writeln!(
            text,
            "LEDGE RESULT | {} ({:#x}) | {label} > {}",
            self.fighter.replace('_', " ").to_uppercase(),
            self.fighter_kind,
            self.choice
        );
        let _ = writeln!(
            text,
            "{result}\nEnd: {}; +{}ms",
            self.end, self.last_stamp.ms
        );
        let _ = writeln!(
            text,
            "TEST CONTEXT: perf_epoch={}; {}",
            self.test_epoch,
            crate::performance::latency_text(self.test_latency)
        );
        let _ = writeln!(text,"TIMING LIMIT: native deltas use one unchanged target motion/rate; wall ms include stalls. Neither is network latency.");
        text.push_str("MAPPED ACK LIMIT: our hook boundary only; another hook/external delay queue may run afterward. Ack native fields use the last copied observation.\n");
        let _ = writeln!(text, "Recognized: {}", self.first_stamp.describe());
        let _ = writeln!(
            text,
            "Protection first observed off: {}",
            point(self.vulnerable)
        );
        let first = self
            .model
            .first
            .map_or_else(|| "?".into(), |v| v.to_string());
        let last = self
            .model
            .last
            .map_or_else(|| "?".into(), |v| v.to_string());
        let hop =
            self.model.full_hop.map_or(
                "not committed/n.a.",
                |full| if full { "full" } else { "short" },
            );
        let _ = writeln!(text, "Configured move window: {first}-{last}; hop: {hop}");
        if let Some(frame) = self.model.fast_fall_from {
            let _ = writeln!(
                text,
                "Configured fast fall: accepted nair f{frame}+; downward velocity required"
            );
        }
        if let Some(lead) = self.last.lead.or(self.first.lead) {
            let _ = writeln!(text, "Configured early lead: {lead:.1}f");
        }
        if let Some(drift) = self.model.drift {
            let _ = writeln!(
                text,
                "Configured stick drift: {drift:.2}; end frame: {:?}",
                self.model.drift_until
            );
        }
        for (i, name) in CHANNELS.iter().enumerate() {
            if self.requests[i].is_some() {
                let _ = writeln!(
                    text,
                    "{name}: {} | mapped {}",
                    point(self.requests[i]),
                    if self.mapped[i] {
                        "YES"
                    } else {
                        "not observed"
                    }
                );
            }
        }
        for (i, name) in CHANNELS.iter().enumerate() {
            if let Some(request) = self.requests[i] {
                let ack = self.mapped_first[i];
                let _=writeln!(text,"INPUT PATH [{name}]: request {}; mapped ack {}; request_to_ack_wall_ms={:?}; request_to_ack_target_frames={:?}",
                    request.describe(),point(ack),ack.map(|s|s.ms.saturating_sub(request.ms)),ack.and_then(|s| target_delta(request,s)));
            }
        }
        if let Some(request) = self.requests[1] {
            let observed = self
                .neutral_first_active
                .map(|p| p.stamp)
                .or(self.move_start);
            let _=writeln!(text,"INPUT PATH [Attack observation]: request_to_observed_wall_ms={:?}; request_to_observed_target_frames={:?}; observed={}",
                observed.map(|s|s.ms.saturating_sub(request.ms)),observed.and_then(|s|target_delta(request,s)),point(observed));
        }
        let _ = writeln!(
            text,
            "SEQUENCE END: {}",
            self.last
                .timeline_end
                .unwrap_or("not observed / still running")
        );
        let _ = writeln!(text, "Native jump/squat: {}", point(self.jump_start));
        let _ = writeln!(text, "Native selected move: {}", point(self.move_start));
        let _ = writeln!(text, "Hit evidence: {}", point(self.hit));
        if self.first.slot == kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral {
            let _ = writeln!(
                text,
                "Neutral first native active: {}",
                self.neutral_first_active
                    .map_or_else(|| "not observed".into(), NeutralActivity::describe)
            );
            let _ = writeln!(
                text,
                "Neutral active while target vulnerable: {}",
                self.neutral_vulnerable_active
                    .map_or_else(|| "not observed".into(), NeutralActivity::describe)
            );
            text.push_str("Active flags confirm attack activity, not contact/reach or which victim was hit.\n");
        }
        if let Some(start) = self.move_start {
            let _ = writeln!(
                text,
                "Own start: {} (status {:#x}), animation frame {:.1}",
                kirby_assist_core::diagnostics::modes::native_name(start.own_status),
                start.own_status,
                start.own_frame
            );
        }
        let _ = writeln!(
            text,
            "Last own: {} (status {:#x}), animation {} at frame {:.1}",
            kirby_assist_core::diagnostics::modes::native_name(self.last.context.status),
            self.last.context.status,
            kirby_assist_core::diagnostics::modes::motion_name(
                self.last.context.status,
                self.last.context.motion_kind
            ),
            self.last.context.motion_frame
        );
        let _ = writeln!(
            text,
            "Configured-window pose observed: {}",
            number(self.activity_frame)
        );
        let spacing = self.first_request.unwrap_or(self.last);
        let gap = spacing
            .enemy
            .map(|e| e.position[0] - spacing.context.position[0]);
        let front_gap = gap.map(|x| x * spacing.context.facing);
        let height = spacing
            .enemy
            .map(|e| e.position[1] - spacing.context.position[1]);
        let where_at = if self.first_request.is_some() {
            "At move request"
        } else {
            "At last observation"
        };
        let _ = writeln!(
            text,
            "{where_at}: front gap {}; height {}",
            number(front_gap),
            number(height)
        );
        let _ = writeln!(
            text,
            "Own x/facing/vx: {:.2}/{:.1}/{:.2}; target x: {}",
            spacing.context.position[0],
            spacing.context.facing,
            spacing.context.velocity[0],
            number(spacing.enemy.map(|e| e.position[0]))
        );
        let _ = writeln!(
            text,
            "Config range/ref: {}; manual air spacing: {}",
            number(spacing.configured_range),
            spacing.distance_agnostic
        );
        let _ = writeln!(
            text,
            "Protection/recovery left: {} / {}",
            number(spacing.enemy.and_then(|e| if e.vulnerable {
                Some(0.0)
            } else {
                e.vulnerable_in
            })),
            number(spacing.enemy.and_then(|e| e.actionable_in))
        );
        if let Some(spot) = self.last.endpoint {
            let _ = writeln!(text, "Roll aim/stand: {:.2} / {:.2}", spot[0], spot[1]);
        }
        let _ = writeln!(
            text,
            "Last waiting condition: {} [gate: {}]\nInput delivery: {}",
            readable_gate(self.last_wait),
            self.last_wait,
            readable_gate(self.mapper_gate)
        );
        let _ = writeln!(
            text,
            "Raw req/applied: {:#x}/{:#x}; mapped: {:#x}",
            self.raw_requested, self.raw_applied, self.mapped_buttons
        );
        let _ = writeln!(
            text,
            "Ready act/jump/grab: {}/{}/{}; blocked: {}",
            self.last.context.can_act,
            self.last.context.can_jump,
            self.last.context.can_grab,
            self.blocked
        );
        let motion_changed = self
            .requests
            .iter()
            .flatten()
            .any(|stamp| stamp.target_motion != self.first_stamp.target_motion)
            || self.last_stamp.target_motion != self.first_stamp.target_motion;
        let _ = writeln!(text, "Target motion changed: {motion_changed}; +ms is wall time.\nNative frames are motion-local, not global.\nConfigured window/pose is not observed hitbox activity.");
        text
    }
}
/// Technical: Stores or classifies the sequence end marker used by calibration reports.
/// Plain English: Remembers how the configured sequence finished.
fn note_timeline_end(reason: Option<&'static str>) {
    match reason {
        Some("native_clock_gap") => {
            crate::performance::health::event(crate::performance::health::Event::TimelineGap)
        }
        Some("clock_invalid_or_timeout") => {
            crate::performance::health::event(crate::performance::health::Event::TimelineClockAbort)
        }
        _ => {}
    }
}

/// Technical: Computes target-motion frame delta only when target clock evidence matches.
/// Plain English: Measures animation progress without treating wall time as game time.
fn target_delta(a: Stamp, b: Stamp) -> Option<f32> {
    let rate = a.target_rate?;
    let other_rate = b.target_rate?;
    if a.target_status != b.target_status
        || a.target_motion.is_none()
        || a.target_motion != b.target_motion
        || !rate.is_finite()
        || rate <= 0.0
        || !other_rate.is_finite()
        || other_rate <= 0.0
        || (rate - other_rate).abs() > 0.001
    {
        return None;
    }
    let delta = (b.target_frame? - a.target_frame?) / rate;
    (delta.is_finite() && (0.0..=600.0).contains(&delta)).then_some(delta)
}

/// Technical: Formats an optional timestamp or returns a stable missing-value label.
/// Plain English: Displays a time point when known, otherwise says it was not observed.
fn point(stamp: Option<Stamp>) -> String {
    stamp.map_or_else(|| "not observed".into(), Stamp::describe)
}
/// Technical: Formats an optional finite numeric value for a report.
/// Plain English: Shows a number when available and a placeholder when it is not.
fn number(value: Option<f32>) -> String {
    value
        .filter(|v| v.is_finite())
        .map_or_else(|| "unknown".into(), |v| format!("{v:.1}"))
}
/// Technical: Resolves the canonical button mask for each calibration input channel.
/// Plain English: Maps a tested move to its jump, attack, grab, special, and shield buttons.
fn channel_masks(frame: CalibrationFrame, kind: CalibrationKind) -> [u32; 5] {
    let grab = if kind == CalibrationKind::Grab {
        frame.grab_button
    } else {
        MAPPER_R
    };
    let reserved = if kind == CalibrationKind::Grab {
        grab
    } else {
        0
    };
    [
        (MAPPER_X | MAPPER_ZL) & !reserved,
        (MAPPER_A | RAW_CSTICK) & !reserved,
        grab,
        MAPPER_B & !reserved,
        MAPPER_ZR & !reserved,
    ]
}

/// Technical: Per-session calibration lifecycle and current bounded report.
/// Plain English: Tracks a single attempt and decides when its report is complete.
pub struct Recorder {
    /// Technical: Current report, absent when there is no active attempt.
    /// Plain English: The calibration report being collected now.
    active: Option<Report>,
    /// Technical: Suppresses resuming an attempt after it has completed.
    /// Plain English: Keeps a finished option from starting another report until a new visit.
    closed: bool,
}
impl Recorder {
    /// Technical: Creates an idle recorder with no report and an open lifecycle.
    /// Plain English: Starts calibration recording in a clean state.
    pub const fn new() -> Self {
        Self {
            active: None,
            closed: false,
        }
    }
    /// Technical: Reports whether a calibration report is currently being collected.
    /// Plain English: Checks whether calibration tracking is active.
    pub fn active(&self) -> bool {
        self.active.is_some()
    }
    /// Technical: Starts, updates, or closes a report from copied calibration frames,
    /// distinguishing option changes and animation rewinds.
    /// Plain English: Records what happened during the selected option attempt.
    pub fn observe(
        &mut self,
        frame: Option<CalibrationFrame>,
        fighter: (&'static str, i32),
        choice: &str,
        now: Instant,
    ) {
        let Some(frame) = frame else {
            self.stop("response/tracking ended");
            self.closed = false;
            return;
        };
        if let Some(report) = self.active.as_ref() {
            let rewound = match (report.last.enemy, frame.enemy) {
                (Some(old), Some(new)) => match (old.motion, new.motion) {
                    (Some(a), Some(b)) => {
                        old.status == new.status && a.kind == b.kind && b.frame + 0.5 < a.frame
                    }
                    _ => false,
                },
                _ => false,
            };
            if report.first.target != frame.target
                || report.first.slot != frame.slot
                || report.choice.as_str() != choice
                || rewound
            {
                self.stop("next option/visit");
                self.closed = false;
            }
        }
        if self.closed {
            return;
        }
        if self.active.is_none() {
            note_timeline_end(frame.timeline_end);
            let stamp = Stamp::new(frame, 0);
            self.active = Some(Report {
                epoch: EPOCH.load(Ordering::Acquire),
                fighter: fighter.0,
                fighter_kind: fighter.1,
                choice: ChoiceName::new(choice),
                origin: now,
                first: frame,
                last: frame,
                first_stamp: stamp,
                last_stamp: stamp,
                model: frame.expected,
                requests: [None; 5],
                mapped: [false; 5],
                mapped_first: [None; 5],
                test_latency: crate::performance::test_latency(),
                test_epoch: crate::performance::context_epoch(),
                first_request: None,
                jump_start: None,
                move_start: None,
                activity_frame: None,
                neutral_first_active: None,
                neutral_vulnerable_active: None,
                vulnerable: None,
                hit: None,
                captured: false,
                blocked: false,
                last_wait: frame.phase,
                mapper_gate: "not observed",
                raw_requested: 0,
                raw_applied: 0,
                mapped_buttons: 0,
                end: "pending",
                was_native: false,
                ending: None,
            });
        }
        let report = self.active.as_mut().unwrap();
        let ms = report.elapsed(now);
        let stamp = Stamp::new(frame, ms);
        if report.move_start.is_none() && frame.expected.kind != CalibrationKind::Unknown {
            report.model = frame.expected;
        }
        let masks = channel_masks(frame, report.model.kind);
        for (i, mask) in masks.into_iter().enumerate() {
            if frame.output.requested & mask != 0 && report.requests[i].is_none() {
                report.requests[i] = Some(stamp);
            }
        }
        if report.action_requested() {
            report.first_request.get_or_insert(frame);
        }
        if frame.timeline_end != report.last.timeline_end {
            note_timeline_end(frame.timeline_end);
        }
        report.raw_requested |= frame.output.requested;
        let neutral_raw_attack = report.first.slot
            == kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral
            && report.model.kind == CalibrationKind::Unknown
            && report.raw_requested & (MAPPER_A | RAW_CSTICK) != 0;
        if neutral_raw_attack && frame.native_attack_active == Some(true) {
            let activity = NeutralActivity::new(frame, stamp);
            report.neutral_first_active.get_or_insert(activity);
            if frame.enemy.is_some_and(|e| e.vulnerable) {
                report.neutral_vulnerable_active.get_or_insert(activity);
            }
            if frame.context.hit_confirmed {
                report.hit.get_or_insert(stamp);
            }
        }
        if report.requests[0].is_some()
            && matches!(
                frame.context.status,
                kirby_assist_core::native_states::status::JUMP_SQUAT
                    | kirby_assist_core::native_states::status::JUMP
            )
            && report.jump_start.is_none()
        {
            report.jump_start = Some(stamp);
        }
        let native = report.action_requested() && report.model.native.matches(frame.context);
        if native && report.move_start.is_none() {
            report.move_start = Some(stamp);
        }
        let activity_native = match report.model.native {
            kirby_assist_core::ledge_trap::NativeRule::Ground { attack, .. } => {
                native && frame.context.status == attack
            }
            _ => native,
        };
        let window_pose = activity_native
            && report
                .model
                .first
                .is_some_and(|first| frame.context.motion_frame >= f32::from(first))
            && report
                .model
                .last
                .is_none_or(|last| frame.context.motion_frame <= f32::from(last));
        if window_pose {
            report
                .activity_frame
                .get_or_insert(frame.context.motion_frame);
        }
        if frame.enemy.is_some_and(|e| e.vulnerable) && report.vulnerable.is_none() {
            report.vulnerable = Some(stamp);
        }
        if window_pose
            && matches!(
                report.model.kind,
                CalibrationKind::Attack | CalibrationKind::Zair | CalibrationKind::Special
            )
            && frame.context.hit_confirmed
            && report.hit.is_none()
        {
            report.hit = Some(stamp);
        }
        report.blocked |= frame.blocked;
        report.captured |= frame.captured;
        if frame.output.requested & (MAPPER_A | RAW_CSTICK | MAPPER_B | MAPPER_R) == 0 && !native {
            report.last_wait = if frame.context.stopped {
                "native stop"
            } else if !frame.context.can_control {
                "native control unavailable"
            } else {
                frame.phase
            };
        }
        let ended_native = report.was_native && !native && !frame.context.stopped;
        report.was_native = native;
        report.last = frame;
        report.last_stamp = stamp;
        let end = if report.captured {
            Some("capture")
        } else if report.hit.is_some() {
            Some("hit evidence")
        } else if frame.complete && !neutral_raw_attack {
            Some("response handed back control")
        } else if neutral_raw_attack
            && report.neutral_first_active.is_some()
            && !matches!(
                frame.context.status,
                kirby_assist_core::native_states::status::ATTACK
                    ..=kirby_assist_core::native_states::status::ATTACK_AIR
            )
            && !frame.context.stopped
        {
            Some("observed neutral attack ended/interrupted")
        } else if ended_native {
            report.ending = Some("selected move ended/interrupted");
            None
        } else if frame.blocked && report.model.kind == CalibrationKind::Shield {
            Some("shield block")
        } else if frame.enemy.is_some_and(|e| {
            e.situation == kirby_assist_core::native_states::situation::GROUND
                && e.status < kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE
        }) && !native
            && !frame.context.stopped
        {
            Some("target option ended")
        } else if ms >= WATCHDOG_MS {
            Some("diagnostic timeout; gameplay unchanged")
        } else {
            None
        };
        if let Some(end) = end {
            self.stop(end);
        }
    }
    /// Technical: Records mapper proposal, applied overlay, and final mapped-button evidence.
    /// Plain English: Tracks whether requested inputs reached the game's mapper output.
    pub fn mapped(&mut self, proposed: u32, applied: u32, mapped: u32, gate: &'static str) {
        let Some(report) = self.active.as_mut() else {
            return;
        };
        let masks = channel_masks(report.last, report.model.kind);
        let logical = [
            native_input::MAPPED_JUMP | native_input::MAPPED_JUMP_MINI,
            native_input::MAPPED_ATTACK,
            native_input::MAPPED_CATCH,
            native_input::MAPPED_SPECIAL,
            native_input::MAPPED_GUARD,
        ];
        for i in 0..5 {
            if proposed & applied & masks[i] != 0 && mapped & logical[i] != 0 {
                report.mapped[i] = true;
                if report.mapped_first[i].is_none() {
                    // Native fields are from the most recent copied observation;
                    // only wall time is refreshed at mapper acknowledgment.
                    let mut stamp = report.last_stamp;
                    stamp.ms = report.elapsed(Instant::now());
                    report.mapped_first[i] = Some(stamp);
                }
            }
        }
        report.raw_applied |= proposed & applied;
        report.mapped_buttons |= mapped;
        if proposed != 0 {
            report.mapper_gate = gate;
        }
        if let Some(reason) = report.ending {
            self.stop(reason);
        }
    }
    /// Technical: Completes the current report with a reason and closes the attempt.
    /// Plain English: Ends recording and sends the report to the logger.
    pub fn stop(&mut self, reason: &'static str) {
        if let Some(mut report) = self.active.take() {
            report.end = reason;
            crate::logging::write_calibration(report);
            self.closed = true;
        }
    }
    /// Technical: Stops any current report and reopens the recorder lifecycle.
    /// Plain English: Clears the completed state so a later option visit can be tracked.
    pub fn reset(&mut self, reason: &'static str) {
        self.stop(reason);
        self.closed = false;
    }
}

/// Called only by the background writer. An old queued result cannot leak into
/// the next battle/fighter after its epoch has been invalidated.
/// Plain English: Updates the display only if the report still belongs to this session.
pub fn publish(report: &Report) {
    if report.epoch != EPOCH.load(Ordering::Acquire) {
        return;
    }
    #[cfg(feature = "imgui-menu")]
    {
        let text = CString::new(report.card_text()).ok();
        if let Ok(mut card) = CARD.lock() {
            if report.epoch == EPOCH.load(Ordering::Acquire) {
                *card = text.map(|text| (report.epoch, text));
            }
        }
    }
    #[cfg(not(feature = "imgui-menu"))]
    let _ = report;
}
/// Technical: Advances the recorder epoch and clears the cached menu card.
/// Plain English: Removes stale calibration text from the diagnostic display.
pub fn clear() {
    EPOCH.fetch_add(1, Ordering::AcqRel);
    #[cfg(feature = "imgui-menu")]
    if let Ok(mut card) = CARD.try_lock() {
        *card = None;
    }
}
#[cfg(feature = "imgui-menu")]
/// Technical: Invokes the renderer with the cached card only when its epoch is current.
/// Plain English: Shows the latest valid calibration card in the menu.
pub fn with_card(draw: impl FnOnce(&CStr)) {
    if let Ok(card) = CARD.try_lock() {
        if let Some((epoch, card)) = card.as_ref() {
            if *epoch == EPOCH.load(Ordering::Acquire) {
                draw(card);
            }
        }
    }
}

#[cfg(test)]
/// Technical: Host-side calibration recorder behavior tests.
/// Plain English: Checks how calibration samples and reports behave.
mod tests {
    use super::*;
    use kirby_assist_core::{
        fighter_profiles::presets::PresetSlot,
        gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
        ledge_trap::NativeRule,
        settings::AssistSettings,
        timing::motion_hash,
    };
    use std::time::Duration;

    /// Technical: Builds a baseline calibration observation for a selected option.
    /// Plain English: Creates a reusable fighter snapshot for test cases.
    fn frame(slot: PresetSlot) -> CalibrationFrame {
        let context = GameplayContext {
            held: true,
            gameplay: true,
            manual_input_epoch: 0,
            can_act: true,
            can_control: true,
            stopped: false,
            status: kirby_assist_core::native_states::status::WAIT,
            situation: kirby_assist_core::native_states::situation::GROUND,
            motion_frame: 1.0,
            motion_kind: motion_hash("wait"),
            motion_rate: 1.0,
            position: [0.0, 0.0],
            velocity: [0.0; 2],
            ground_brake: Some(0.1),
            facing: 1.0,
            previous_status: kirby_assist_core::native_states::status::WAIT,
            can_grab: true,
            can_tilt: true,
            can_down_tilt: true,
            can_down_special: true,
            can_dash_attack: true,
            can_down_smash: true,
            input_delay: 0.0,
            can_jump: true,
            can_air_attack: true,
            air_physics: None,
            short_hop_speed: None,
            full_hop_speed: None,
            shield_stun_frames: None,
            shield_attacker: None,
            hit_confirmed: false,
            safe_ground: [true; 2],
        };
        let enemy = Opponent {
            id: 42,
            status: kirby_assist_core::native_states::status::CLIFF_ESCAPE,
            previous_status: kirby_assist_core::native_states::status::CLIFF_WAIT,
            situation: kirby_assist_core::native_states::situation::GROUND,
            position: [5.0, 0.0],
            vulnerable: false,
            velocity: [0.0; 2],
            cliff_position: Some([0.0; 2]),
            vulnerable_in: Some(10.0),
            motion: Some(MotionTiming {
                kind: motion_hash("cliff_escape"),
                frame: 12.0,
                rate: 1.0,
                remaining: Some(30.0),
            }),
            attack_active: None,
            air_physics: None,
            actionable_in: Some(30.0),
        };
        CalibrationFrame {
            target: 42,
            slot,
            native_attack_active: None,
            timeline_end: None,
            expected: CalibrationMove {
                kind: CalibrationKind::Attack,
                native: NativeRule::Aerial {
                    motion: motion_hash("attack_air_n"),
                    custom_status: None,
                },
                first: Some(8),
                last: Some(32),
                ..CalibrationMove::unknown()
            },
            context,
            enemy: Some(enemy),
            output: ControllerOutput {
                requested: MAPPER_X | MAPPER_A,
                ..ControllerOutput::default()
            },
            phase: "prepare",
            complete: false,
            captured: false,
            blocked: false,
            endpoint: None,
            configured_range: Some(12.0),
            lead: None,
            distance_agnostic: true,
            grab_button: AssistSettings::default().grab_button,
        }
    }

    #[test]
    /// Technical: Verifies raw recipes do not fabricate readiness or opponent failures.
    /// Plain English: Ensures the report only records evidence the game actually provided.
    fn raw_recipe_diagnostics_do_not_invent_readiness_or_opponent_failures() {
        let mut f = frame(PresetSlot::Drop);
        f.expected = CalibrationMove::unknown();
        f.enemy = None;
        f.context.can_act = false;
        f.context.can_control = false;
        f.output.requested = MAPPER_ZR;
        let mut recorder = Recorder::new();
        recorder.observe(Some(f), ("yoshi", 5), "Shield", Instant::now());
        recorder.mapped(MAPPER_ZR, MAPPER_ZR, native_input::MAPPED_GUARD, "ledge");
        let report = recorder.active.as_ref().unwrap();
        assert_eq!(report.result(), "INPUT requested; move model unavailable");
        assert!(report.explanation().contains("raw-input recipe"));
        assert!(!report.explanation().contains("No opponent was available"));
        assert!(!report.explanation().contains("controller did not start"));
        assert_ne!(report.raw_applied, 0);
    }
    #[test]
    /// Technical: Verifies each option channel creates one report without live publication.
    /// Plain English: Checks all five button categories are recorded once.
    fn all_five_options_record_once_without_live_report_updates() {
        let now = Instant::now();
        for slot in PresetSlot::ALL {
            let mut recorder = Recorder::new();
            for _ in 0..1000 {
                recorder.observe(Some(frame(slot)), ("kirby", 6), "Nair", now);
            }
            let report = recorder.active.as_ref().unwrap();
            assert_eq!(report.first.slot, slot);
            assert_eq!(report.requests.iter().flatten().count(), 2);
            assert!(report.move_start.is_none());
            assert!(!report.format().contains("CAPTURE confirmed"));
        }
    }

    #[test]
    /// Technical: Verifies jump mapping is not misclassified as attack/aerial confirmation.
    /// Plain English: Ensures a jump alone does not count as the wrong move succeeding.
    fn jump_mapping_does_not_confirm_attack_or_wrong_aerial() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Neutral);
        recorder.observe(Some(f), ("kirby", 6), "Nair", now);
        recorder.mapped(
            MAPPER_X | MAPPER_A,
            MAPPER_X,
            native_input::MAPPED_JUMP,
            "ledge",
        );
        assert!(recorder.active.as_ref().unwrap().mapped[0]);
        assert!(!recorder.active.as_ref().unwrap().mapped[1]);
        f.output = ControllerOutput::default();
        f.context.situation = kirby_assist_core::native_states::situation::AIR;
        f.context.status = kirby_assist_core::native_states::status::ATTACK_AIR;
        f.context.motion_kind = motion_hash("attack_air_f");
        recorder.observe(
            Some(f),
            ("kirby", 6),
            "Nair",
            now + Duration::from_millis(16),
        );
        assert!(recorder.active.as_ref().unwrap().move_start.is_none());
        f.context.motion_kind = motion_hash("attack_air_n");
        f.context.motion_frame = 0.0;
        recorder.observe(
            Some(f),
            ("kirby", 6),
            "Nair",
            now + Duration::from_millis(32),
        );
        let report = recorder.active.as_ref().unwrap();
        assert!(report.move_start.is_some());
        assert!(report.hit.is_none());
        assert!(report.activity_frame.is_none());
        assert!(report
            .format()
            .contains("START observed; hit not confirmed"));
    }

    #[test]
    /// Technical: Verifies grounded startup/charge does not count as active attack pose.
    /// Plain English: Ensures a move winding up is not reported as already attacking.
    fn grounded_startup_or_charge_is_not_an_active_attack_pose() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Roll);
        f.expected = CalibrationMove {
            kind: CalibrationKind::Attack,
            native: NativeRule::Ground {
                startup: kirby_assist_core::native_states::status::ATTACK_LW4_START,
                attack: kirby_assist_core::native_states::status::ATTACK_LW4,
            },
            first: Some(7),
            last: Some(19),
            ..CalibrationMove::unknown()
        };
        recorder.observe(Some(f), ("kirby", 6), "Dsmash", now);
        f.context.status = kirby_assist_core::native_states::status::ATTACK_LW4_HOLD;
        f.context.motion_frame = 9.0;
        f.context.hit_confirmed = true;
        recorder.observe(
            Some(f),
            ("kirby", 6),
            "Dsmash",
            now + Duration::from_millis(16),
        );
        let report = recorder.active.as_ref().unwrap();
        assert!(report.move_start.is_some());
        assert!(report.activity_frame.is_none());
        assert!(report.hit.is_none());
    }

    #[test]
    /// Technical: Verifies shield preparation retains independent grab-request spacing.
    /// Plain English: Ensures shield input does not distort grab timing evidence.
    fn shield_preparation_does_not_replace_grab_request_spacing() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Attack);
        f.expected = CalibrationMove {
            kind: CalibrationKind::Grab,
            native: NativeRule::Statuses(&[kirby_assist_core::native_states::CATCH]),
            ..CalibrationMove::unknown()
        };
        f.output.requested = MAPPER_ZR;
        recorder.observe(Some(f), ("kirby", 6), "Grab", now);
        assert!(recorder.active.as_ref().unwrap().first_request.is_none());
        f.context.position[0] = 2.0;
        f.output.requested = MAPPER_R | MAPPER_ZR;
        recorder.observe(
            Some(f),
            ("kirby", 6),
            "Grab",
            now + Duration::from_millis(16),
        );
        assert_eq!(
            recorder
                .active
                .as_ref()
                .unwrap()
                .first_request
                .unwrap()
                .context
                .position[0],
            2.0
        );
        recorder.mapped(
            f.output.requested,
            f.output.requested,
            native_input::MAPPED_GUARD | native_input::MAPPED_CATCH,
            "ledge",
        );
        assert!(recorder.active.as_ref().unwrap().mapped[2]);
    }

    #[test]
    /// Technical: Verifies completed reports stay closed until the option is revisited.
    /// Plain English: Prevents a finished attempt from restarting during repeated polls.
    fn completed_result_stays_closed_until_next_option_visit() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let f = frame(PresetSlot::Jump);
        recorder.observe(Some(f), ("yoshi", 5), "Nair", now);
        recorder.stop("test completion");
        recorder.observe(Some(f), ("yoshi", 5), "Nair", now);
        assert!(!recorder.active());
        recorder.observe(None, ("yoshi", 5), "Nair", now);
        recorder.observe(Some(f), ("yoshi", 5), "Nair", now);
        assert!(recorder.active());
    }
    #[test]
    /// Technical: Verifies neutral raw reports include actual active/protection/spacing facts.
    /// Plain English: Checks that neutral move reports use observed game evidence.
    fn raw_neutral_report_survives_recipe_end_and_captures_real_active_protection_and_spacing() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Neutral);
        f.expected = CalibrationMove::unknown();
        f.complete = true;
        f.context.status = kirby_assist_core::native_states::status::JUMP_SQUAT;
        f.native_attack_active = Some(false);
        let mut enemy = f.enemy.unwrap();
        enemy.status = kirby_assist_core::ledge_trap::CLIFF_CLIMB;
        f.enemy = Some(enemy);
        recorder.observe(Some(f), ("yoshi", 5), "Raw Nair", now);
        assert!(
            recorder.active(),
            "recipe completion is not attack completion"
        );
        f.context.status = kirby_assist_core::native_states::status::ATTACK_AIR;
        f.context.motion_kind = motion_hash("attack_air_n");
        f.context.motion_frame = 3.0;
        f.context.position = [0.0, 8.0];
        f.output = ControllerOutput::default();
        f.native_attack_active = Some(true);
        recorder.observe(
            Some(f),
            ("yoshi", 5),
            "Raw Nair",
            now + Duration::from_millis(16),
        );
        let report = recorder.active.as_ref().unwrap();
        assert_eq!(report.neutral_first_active.unwrap().vulnerable, Some(false));
        assert_eq!(report.neutral_first_active.unwrap().gap, Some([5.0, -8.0]));
        assert!(report.neutral_vulnerable_active.is_none());
        enemy.vulnerable = true;
        enemy.actionable_in = Some(1.0);
        f.enemy = Some(enemy);
        f.context.motion_frame = 4.0;
        recorder.observe(
            Some(f),
            ("yoshi", 5),
            "Raw Nair",
            now + Duration::from_millis(32),
        );
        let report = recorder.active.as_ref().unwrap();
        assert_eq!(
            report.neutral_vulnerable_active.unwrap().recovery,
            Some(1.0)
        );
        let text = report.format();
        assert!(text.contains("Neutral first native active:"));
        assert!(text.contains("recovery_left=1.0"));
        assert!(text.contains("ACTIVE attack observed"));
        assert!(report.hit.is_none());
        f.context.status = kirby_assist_core::native_states::status::LANDING_ATTACK_AIR;
        f.native_attack_active = Some(false);
        recorder.observe(
            Some(f),
            ("yoshi", 5),
            "Raw Nair",
            now + Duration::from_millis(48),
        );
        assert!(!recorder.active());
    }
    #[test]
    /// Technical: Verifies only raw neutral attack traces continue after recipe completion.
    /// Plain English: Keeps other finished recipes from extending their reports.
    fn only_raw_neutral_attack_reports_extend_past_recipe_completion() {
        for slot in [PresetSlot::Neutral, PresetSlot::Jump] {
            let mut recorder = Recorder::new();
            let mut f = frame(slot);
            f.complete = true;
            if slot == PresetSlot::Neutral {
                f.expected = CalibrationMove::unknown();
                f.output.requested = MAPPER_X; // No attack to watch.
            }
            recorder.observe(Some(f), ("kirby", 6), "Input", Instant::now());
            assert!(!recorder.active());
        }
    }
    #[test]
    /// Technical: Requires matching target-clock evidence before calculating native frame delta.
    /// Plain English: Avoids comparing animation times from different targets or motions.
    fn input_path_only_computes_native_delta_with_matching_clock_evidence() {
        let a = Stamp::new(frame(PresetSlot::Neutral), 0);
        let mut b = a;
        b.target_frame = Some(a.target_frame.unwrap() + 4.0);
        b.ms = 90;
        assert_eq!(target_delta(a, b), Some(4.0));
        b.target_motion = Some(99);
        assert_eq!(target_delta(a, b), None);
        b = a;
        b.target_rate = Some(f32::NAN);
        assert_eq!(target_delta(a, b), None);
        b = a;
        b.target_frame = Some(0.0);
        assert_eq!(target_delta(a, b), None);
    }
    #[test]
    /// Technical: Verifies mapper acknowledgment is timestamped separately and retains trial label.
    /// Plain English: Keeps the input request, mapper confirmation, and option name distinct.
    fn mapper_ack_has_its_own_point_and_report_uses_stored_trial_label() {
        let mut r = Recorder::new();
        let f = frame(PresetSlot::Jump);
        r.observe(Some(f), ("kirby", 6), "Nair", Instant::now());
        r.mapped(
            f.output.requested,
            f.output.requested,
            native_input::MAPPED_ATTACK | native_input::MAPPED_JUMP,
            "test",
        );
        r.active.as_mut().unwrap().test_latency = Some(4);
        let report = r.active.as_ref().unwrap();
        assert_eq!(report.test_latency, Some(4));
        assert!(report.mapped_first[1].is_some());
        assert!(report.format().contains("INPUT PATH [Attack]"));
        assert!(report.format().contains("external +4f"));
        assert!(report.format().contains("MAPPED ACK LIMIT"));
    }
}
