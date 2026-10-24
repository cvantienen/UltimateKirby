//! Diagnostic-only frozen ledge results. Observers collect copied facts in fixed
//! storage; the logger worker formats/publishes once when an attempt ends.
use kirby_assist_core::{
    ledge_trap::{CalibrationFrame, CalibrationKind, CalibrationMove},
    native_input::{MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_X, MAPPER_ZL, MAPPER_ZR},
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

static EPOCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "imgui-menu")]
static CARD: Mutex<Option<(usize, CString)>> = Mutex::new(None);
const WATCHDOG_MS: u64 = 4_000;
const RAW_CSTICK: u32 = 0xf << 24;
const CHANNELS: [&str; 5] = ["Jump", "Attack", "Grab", "Special", "Shield"];

#[derive(Clone, Copy, Debug)]
struct Stamp {
    ms: u64,
    target_status: Option<i32>,
    target_motion: Option<u64>,
    target_frame: Option<f32>,
    target_rate: Option<f32>,
    own_status: i32,
    own_frame: f32,
}
impl Stamp {
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

pub struct Report {
    epoch: usize,
    fighter: &'static str,
    fighter_kind: i32,
    choice: &'static str,
    origin: Instant,
    first: CalibrationFrame,
    last: CalibrationFrame,
    first_stamp: Stamp,
    last_stamp: Stamp,
    model: CalibrationMove,
    requests: [Option<Stamp>; 5],
    mapped: [bool; 5],
    first_request: Option<CalibrationFrame>,
    jump_start: Option<Stamp>,
    move_start: Option<Stamp>,
    activity_frame: Option<f32>,
    vulnerable: Option<Stamp>,
    hit: Option<Stamp>,
    captured: bool,
    blocked: bool,
    last_wait: &'static str,
    mapper_gate: &'static str,
    raw_requested: u32,
    raw_applied: u32,
    mapped_buttons: u32,
    end: &'static str,
    was_native: bool,
    ending: Option<&'static str>,
}
/// Worker-only explanation; retain unknown gate names with readable spacing.
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
        "two_frame_watch_recovery" => "Watching the opponent's recovery toward the ledge",
        "two_frame_prepare_input" => "Releasing input before the two-frame punish window",
        "two_frame_attack_request" => "Requesting the precisely timed two-frame attack",
        "finished" | "two_frame_handoff" => "Attempt finished; returning control",
        "not observed" => "No mapper delivery sample was recorded",
        _ => return gate.replace('_', " ").into(),
    };
    known.into()
}
impl Report {
    fn result(&self) -> &'static str {
        if self.captured {
            "CAPTURE confirmed"
        } else if self.hit.is_some() {
            "HIT observed; victim unverified"
        } else if self.blocked && self.model.kind == CalibrationKind::Shield {
            "BLOCK observed"
        } else if self.move_start.is_some() && self.model.kind == CalibrationKind::Shield {
            "SHIELD observed; no block confirmed"
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

    fn explanation(&self) -> &'static str {
        if self.captured {
            "The game confirmed a capture."
        } else if self.hit.is_some() {
            "A hit flag was observed; the victim is not verified."
        } else if self.blocked {
            "A block was observed; this is not an attack hit."
        } else if self.move_start.is_some() {
            "The selected move started, but no hit was confirmed. Timing or spacing may explain the miss; the log cannot prove which."
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

    fn elapsed(&self, now: Instant) -> u64 {
        now.saturating_duration_since(self.origin)
            .as_millis()
            .min(u128::from(u64::MAX)) as u64
    }
    fn action_requested(&self) -> bool {
        match self.model.kind {
            CalibrationKind::Attack => self.requests[1].is_some(),
            CalibrationKind::Grab | CalibrationKind::Zair => self.requests[2].is_some(),
            CalibrationKind::Special => self.requests[3].is_some(),
            CalibrationKind::Shield => self.requests[4].is_some(),
            CalibrationKind::Unknown => false,
        }
    }
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
    pub fn format(&self) -> String {
        let mut text = String::with_capacity(3_072);
        let label = match self.first.slot {
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral => "NEUTRAL",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Jump => "JUMP",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Roll => "ROLL",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Attack => "ATTACK",
            kirby_assist_core::fighter_profiles::presets::PresetSlot::Drop => "DROP",
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
        let _ = writeln!(text, "Native jump/squat: {}", point(self.jump_start));
        let _ = writeln!(text, "Native selected move: {}", point(self.move_start));
        let _ = writeln!(text, "Hit evidence: {}", point(self.hit));
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
fn point(stamp: Option<Stamp>) -> String {
    stamp.map_or_else(|| "not observed".into(), Stamp::describe)
}
fn number(value: Option<f32>) -> String {
    value
        .filter(|v| v.is_finite())
        .map_or_else(|| "unknown".into(), |v| format!("{v:.1}"))
}
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

pub struct Recorder {
    active: Option<Report>,
    closed: bool,
}
impl Recorder {
    pub const fn new() -> Self {
        Self {
            active: None,
            closed: false,
        }
    }
    pub fn active(&self) -> bool {
        self.active.is_some()
    }
    pub fn observe(
        &mut self,
        frame: Option<CalibrationFrame>,
        fighter: (&'static str, i32),
        choice: &'static str,
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
                || report.choice != choice
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
            let stamp = Stamp::new(frame, 0);
            self.active = Some(Report {
                epoch: EPOCH.load(Ordering::Acquire),
                fighter: fighter.0,
                fighter_kind: fighter.1,
                choice,
                origin: now,
                first: frame,
                last: frame,
                first_stamp: stamp,
                last_stamp: stamp,
                model: frame.expected,
                requests: [None; 5],
                mapped: [false; 5],
                first_request: None,
                jump_start: None,
                move_start: None,
                activity_frame: None,
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
        report.raw_requested |= frame.output.requested;
        if report.requests[0].is_some()
            && matches!(frame.context.status, 0xa | 0xb)
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
        } else if frame.complete {
            Some("response handed back control")
        } else if ended_native {
            report.ending = Some("selected move ended/interrupted");
            None
        } else if frame.blocked && report.model.kind == CalibrationKind::Shield {
            Some("shield block")
        } else if frame
            .enemy
            .is_some_and(|e| e.situation == 0 && e.status < 0x75)
            && !native
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
    pub fn mapped(&mut self, proposed: u32, applied: u32, mapped: u32, gate: &'static str) {
        let Some(report) = self.active.as_mut() else {
            return;
        };
        let masks = channel_masks(report.last, report.model.kind);
        let logical = [0x4 | 0x40, 0x1, 0x10, 0x2, 0x8];
        for i in 0..5 {
            if proposed & applied & masks[i] != 0 && mapped & logical[i] != 0 {
                report.mapped[i] = true;
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
    pub fn stop(&mut self, reason: &'static str) {
        if let Some(mut report) = self.active.take() {
            report.end = reason;
            crate::logging::write_calibration(report);
            self.closed = true;
        }
    }
    pub fn reset(&mut self, reason: &'static str) {
        self.stop(reason);
        self.closed = false;
    }
}

/// Called only by the background writer. An old queued result cannot leak into
/// the next battle/fighter after its epoch has been invalidated.
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
pub fn clear() {
    EPOCH.fetch_add(1, Ordering::AcqRel);
    #[cfg(feature = "imgui-menu")]
    if let Ok(mut card) = CARD.try_lock() {
        *card = None;
    }
}
#[cfg(feature = "imgui-menu")]
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
mod tests {
    use super::*;
    use kirby_assist_core::{
        fighter_profiles::presets::PresetSlot,
        ledge_trap::{
            LedgeContext, LedgeOutput, LedgeSettings, MotionTiming, NativeRule, Opponent,
        },
        timing::motion_hash,
    };
    use std::time::Duration;

    fn frame(slot: PresetSlot) -> CalibrationFrame {
        let context = LedgeContext {
            held: true,
            gameplay: true,
            can_act: true,
            can_control: true,
            stopped: false,
            status: 0,
            situation: 0,
            motion_frame: 1.0,
            motion_kind: motion_hash("wait"),
            motion_rate: 1.0,
            position: [0.0, 0.0],
            velocity: [0.0; 2],
            ground_brake: Some(0.1),
            facing: 1.0,
            previous_status: 0,
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
            status: 0x7a,
            previous_status: 0x77,
            situation: 0,
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
            output: LedgeOutput {
                requested: MAPPER_X | MAPPER_A,
                ..LedgeOutput::default()
            },
            phase: "prepare",
            complete: false,
            captured: false,
            blocked: false,
            endpoint: None,
            configured_range: Some(12.0),
            lead: None,
            distance_agnostic: true,
            grab_button: LedgeSettings::default().grab_button,
        }
    }

    #[test]
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
    fn jump_mapping_does_not_confirm_attack_or_wrong_aerial() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Neutral);
        recorder.observe(Some(f), ("kirby", 6), "Nair", now);
        recorder.mapped(MAPPER_X | MAPPER_A, MAPPER_X, 0x4, "ledge");
        assert!(recorder.active.as_ref().unwrap().mapped[0]);
        assert!(!recorder.active.as_ref().unwrap().mapped[1]);
        f.output = LedgeOutput::default();
        f.context.situation = 2;
        f.context.status = 0x36;
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
    fn grounded_startup_or_charge_is_not_an_active_attack_pose() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Roll);
        f.expected = CalibrationMove {
            kind: CalibrationKind::Attack,
            native: NativeRule::Ground {
                startup: 0x30,
                attack: 0x32,
            },
            first: Some(7),
            last: Some(19),
            ..CalibrationMove::unknown()
        };
        recorder.observe(Some(f), ("kirby", 6), "Dsmash", now);
        f.context.status = 0x31;
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
    fn shield_preparation_does_not_replace_grab_request_spacing() {
        let now = Instant::now();
        let mut recorder = Recorder::new();
        let mut f = frame(PresetSlot::Attack);
        f.expected = CalibrationMove {
            kind: CalibrationKind::Grab,
            native: NativeRule::Statuses(&[0x37]),
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
        recorder.mapped(f.output.requested, f.output.requested, 0x18, "ledge");
        assert!(recorder.active.as_ref().unwrap().mapped[2]);
    }

    #[test]
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
}
