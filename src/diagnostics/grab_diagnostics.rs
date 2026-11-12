//! A small flight recorder for roll grabs. Hooks copy numbers into fixed storage;
//! only the background logger formats the completed report. No gameplay decisions.
//! Technical: Captures bounded grab-related traces as copied values for deferred
//! diagnostic reporting.
//! Plain English: This module collects grab troubleshooting data without doing
//! file or console work in game hooks.
use kirby_assist_core::{ledge_trap::RollGrabSnapshot, native_input::MAPPED_CATCH};
use std::{fmt::Write, time::Instant};

/// Fixed capacity of the bounded diagnostic checkpoint ring.
const CHECKPOINTS: usize = 32;
/// Minimum interval between emitted progress updates.
const PROGRESS_MS: u64 = 125;
/// Maximum elapsed time before the trace watchdog ends a report.
const WATCHDOG_MS: u64 = 3_000;

/// Represents an event in the roll grab diagnostics, either an observation or a mapper update.
/// Copied checkpoint payload for fighter observations and mapper delivery.
#[derive(Clone, Copy)]
enum Event {
    /// Native grab-related state copied from an observation.
    Observation(RollGrabSnapshot),
    /// Input masks and stick values observed at the mapper boundary.
    Mapper {
        /// Buttons proposed by the overlay.
        proposed: u32,
        /// Buttons applied before native mapping.
        applied: u32,
        /// Buttons pressed on this mapper poll.
        pressed: u32,
        /// Final mapped button mask.
        mapped: u32,
        /// Final mapped left-stick axis values.
        stick: [i8; 2],
        /// Gate or suppression reason associated with delivery.
        gate: &'static str,
    },
}
/// Represents a checkpoint in the roll grab diagnostics, storing the elapsed time and the associated event.
/// Ring-buffer entry pairing an elapsed timestamp with one copied event.
#[derive(Clone, Copy)]
struct Checkpoint {
    /// Milliseconds elapsed since report origin.
    ms: u64,
    /// Copied diagnostic payload at this point.
    event: Event,
}

/// Represents a report for roll grab diagnostics, including metadata, checkpoints, and various counters.
/// Bounded recorder snapshot describing one roll-grab attempt and its delivery history.
pub struct Report {
    /// Monotonic identifier assigned to this trace.
    id: u64,
    /// Fighter kind associated with the trace, if known.
    fighter: Option<i32>,
    /// Target object identifier associated with the trace, if known.
    target: Option<u32>,
    /// Time origin used to calculate checkpoint durations.
    origin: Instant,
    /// First captured native grab observation.
    first: RollGrabSnapshot,
    /// Most recent captured native grab observation.
    last: RollGrabSnapshot,
    /// Fixed-size circular checkpoint storage.
    checkpoints: [Option<Checkpoint>; CHECKPOINTS],
    /// Number of checkpoint writes, including overwritten ring entries.
    written: usize,
    /// Count of observation updates.
    observations: u32,
    /// Count of grab requests.
    requests: u32,
    /// Count of mapper polls during the trace.
    mapper_polls: u32,
    /// Count of polls where a grab was proposed.
    proposed_polls: u32,
    /// Count of polls where the grab overlay was applied.
    applied_polls: u32,
    /// Count of polls where native mapped CATCH was present.
    catch_polls: u32,
    /// Count of rising edges on native mapped CATCH.
    catch_edges: u32,
    /// Count of exits classified as whiff or interruption.
    whiff_exits: u32,
    /// Whether native grab startup was observed.
    native_started: bool,
    /// Elapsed timestamp of the most recent progress update.
    last_progress_ms: u64,
    /// Latest elapsed time observed by the recorder.
    latest_ms: u64,
    /// First checkpoint at which a request was observed.
    first_request: Option<Checkpoint>,
    /// First checkpoint at which native grab startup was observed.
    first_native_start: Option<Checkpoint>,
    /// Most recent checkpoint waiting for native progress.
    last_wait: Option<Checkpoint>,
    /// Most recent mapping inputs and delivery gate summary.
    last_mapping: Option<(u32, u32, bool, &'static str)>,
    /// Stable trace termination reason.
    end: &'static str,
}
impl Report {
    /// Returns the elapsed time in milliseconds since the report's origin.
    fn elapsed(&self, now: Instant) -> u64 {
        now.saturating_duration_since(self.origin)
            .as_millis()
            .min(u128::from(u64::MAX)) as u64
    }
    /// Pushes a new checkpoint into the report, recording the elapsed time and event.
    fn push(&mut self, ms: u64, event: Event) {
        // Preserve the initial checkpoint separately; the ring retains recent
        // evidence and reports overwrite counts instead of allocating/growing.
        self.checkpoints[self.written % CHECKPOINTS] = Some(Checkpoint { ms, event });
        self.written = self.written.saturating_add(1);
    }
    /// Called exclusively on the logger thread, never under the gameplay mutex.
    /// Formats the report into a human-readable string for logging or display.
    pub fn format(&self) -> String {
        let result = if self.last.captured {
            "capture_confirmed"
        } else if self.native_started {
            "native_grab_started_no_capture_observed"
        } else if self.requests == 0 {
            "no_grab_requested_check_last_gate"
        } else if self.proposed_polls == 0 {
            "request_not_seen_by_mapper"
        } else if self.applied_polls == 0 {
            "overlay_not_delivered_check_priority_or_expiry"
        } else if self.catch_polls == 0 {
            "raw_button_not_mapped_to_CATCH_check_controller_mapping"
        } else {
            "mapped_CATCH_no_native_start_observed"
        };
        let mut text = String::with_capacity(8_192);
        let _ = writeln!(text, "roll grab trace: id={}, fighter={:?}, target={:?}, end={}, result={}, observations={}, requests={}, native_started={}, whiff_or_interrupt_exits={}, mapper_polls={}, proposed={}, applied={}, CATCH_polls={}, CATCH_edges={}, overwritten={}", self.id, self.fighter, self.target, self.end, result, self.observations, self.requests, self.native_started, self.whiff_exits, self.mapper_polls, self.proposed_polls, self.applied_polls, self.catch_polls, self.catch_edges, self.written.saturating_sub(CHECKPOINTS));
        Self::format_observation(&mut text, "first", 0, self.first);
        for (label, point) in [
            ("first_request", self.first_request),
            ("first_native_start", self.first_native_start),
            ("last_wait", self.last_wait),
        ] {
            if let Some(Checkpoint {
                ms,
                event: Event::Observation(snapshot),
            }) = point
            {
                Self::format_observation(&mut text, label, ms, snapshot);
            }
        }
        let start = self.written.saturating_sub(CHECKPOINTS);
        for index in start..self.written {
            if let Some(point) = self.checkpoints[index % CHECKPOINTS] {
                match point.event {
                    Event::Observation(snapshot) => {
                        Self::format_observation(&mut text, "observe", point.ms, snapshot)
                    }
                    Event::Mapper {
                        proposed,
                        applied,
                        pressed,
                        mapped,
                        stick,
                        gate,
                    } => {
                        let _ = writeln!(text, "  +{}ms mapper gate={} proposed={:#x} applied={:#x} pressed={:#x} mapped={:#x} CATCH={} stick={:?}", point.ms, gate, proposed, applied, pressed, mapped, mapped & MAPPED_CATCH != 0, stick);
                    }
                }
            }
        }
        Self::format_observation(&mut text, "last", self.latest_ms, self.last);
        text
    }
    /// Appends the copied native snapshot fields to the report buffer.
    fn format_observation(text: &mut String, label: &str, ms: u64, s: RollGrabSnapshot) {
        let _ = writeln!(text, "  +{ms}ms {label} gate={} status={:#x}@{:.2} pos={:?} vx={:.3} facing={} endpoint={:?} stand={:?} endpoint_gap={:?} target_gap_facing={:?} usable_reach={:.2} grab_range={:.2} ready(control/act/grab)={}/{}/{} stopped={} safe_ground={:?} enemy_status={:?}@{:?} enemy_pos={:?} vulnerable={:?} opens_in={:?} adjusted_opens_in={:?} actionable_in={:?} active={:?} request={:#x} grab_button={:#x} attempts={} animations={} native_grabbing={} captured={}", s.phase, s.status, s.motion_frame, s.position, s.velocity_x, s.facing, s.endpoint, s.stand_x, s.endpoint.map(|x| x-s.position[0]), s.enemy_position.map(|p| (p[0]-s.position[0])*s.facing), s.usable_reach, s.grab_range, s.can_control, s.can_act, s.can_grab, s.stopped, s.safe_ground, s.enemy_status, s.enemy_frame, s.enemy_position, s.vulnerable, s.vulnerable_in, s.adjusted_opens_in, s.actionable_in, s.active, s.requested, s.grab_button, s.attempts, s.animations, s.native_grabbing, s.captured);
    }
}
/// Represents the trace of a roll grab, including the active report and its completion status.
/// Owns one active recorder plus completion and identifier state.
pub struct GrabTrace {
    /// Active bounded report, absent when no trace is being collected.
    active: Option<Report>,
    /// Whether a completed report awaits delivery.
    completed: bool,
    /// Monotonic id assigned to the next report.
    next_id: u64,
}
impl GrabTrace {
    /// Creates a new instance of `GrabTrace` with no active report and initial state.
    pub const fn new() -> Self {
        Self {
            active: None,
            completed: false,
            next_id: 0,
        }
    }
    /// Returns `true` if there is an active report being tracked.
    pub fn active(&self) -> bool {
        self.active.is_some()
    }
    /// Observes a new roll grab snapshot, updating the active report or finishing it if necessary.
    pub fn observe(
        &mut self,
        snapshot: Option<RollGrabSnapshot>,
        fighter: Option<i32>,
        now: Instant,
    ) {
        let Some(snapshot) = snapshot else {
            self.finish("tracking_or_response_ended");
            self.completed = false;
            return;
        };
        if self
            .active
            .as_ref()
            .is_some_and(|report| report.target != snapshot.target)
        {
            self.finish("target_changed");
            self.completed = false;
        }
        if self.completed {
            return;
        }
        if self.active.is_none() {
            self.next_id = self.next_id.wrapping_add(1);
            self.active = Some(Report {
                id: self.next_id,
                fighter,
                target: snapshot.target,
                origin: now,
                first: snapshot,
                last: snapshot,
                checkpoints: [None; CHECKPOINTS],
                written: 0,
                observations: 0,
                requests: 0,
                mapper_polls: 0,
                proposed_polls: 0,
                applied_polls: 0,
                catch_polls: 0,
                catch_edges: 0,
                whiff_exits: 0,
                native_started: false,
                last_progress_ms: 0,
                latest_ms: 0,
                first_request: None,
                first_native_start: None,
                last_wait: None,
                last_mapping: None,
                end: "pending",
            });
        }
        let report = self.active.as_mut().unwrap();
        let ms = report.elapsed(now);
        report.latest_ms = ms;
        let previous = report.last;
        let initial = report.observations == 0;
        report.observations = report.observations.saturating_add(1);
        let request = snapshot.phase == "roll_request_grab"
            && (initial
                || previous.phase != snapshot.phase
                || previous.attempts != snapshot.attempts);
        report.requests += u32::from(request);
        if request && report.first_request.is_none() {
            report.first_request = Some(Checkpoint {
                ms,
                event: Event::Observation(snapshot),
            });
        }
        if snapshot.native_grabbing && report.first_native_start.is_none() {
            report.first_native_start = Some(Checkpoint {
                ms,
                event: Event::Observation(snapshot),
            });
        }
        report.native_started |= snapshot.native_grabbing;
        if !snapshot.native_grabbing
            && !snapshot.captured
            && snapshot.requested & snapshot.grab_button == 0
            && snapshot.phase != "roll_punish_window_missed"
        {
            report.last_wait = Some(Checkpoint {
                ms,
                event: Event::Observation(snapshot),
            });
        }
        let exited = previous.native_grabbing && !snapshot.native_grabbing && !snapshot.captured;
        report.whiff_exits += u32::from(exited);
        // Capture request/acceptance/outcome edges immediately. Movement and
        // readiness chatter get at most eight numeric checkpoints per second.
        let important = initial
            || request
            || snapshot.captured
            || exited
            || snapshot.native_grabbing != previous.native_grabbing;
        if important || ms.saturating_sub(report.last_progress_ms) >= PROGRESS_MS {
            report.push(ms, Event::Observation(snapshot));
            report.last_progress_ms = ms;
        }
        report.last = snapshot;
        let end = if snapshot.captured {
            Some("capture")
        } else if snapshot.phase == "roll_punish_window_missed" && !snapshot.native_grabbing {
            Some("roll_window_ended")
        } else if ms >= WATCHDOG_MS {
            Some("diagnostic_watchdog_gameplay_unchanged")
        } else {
            None
        };
        if let Some(end) = end {
            self.finish(end);
        }
    }
    /// Records a mapper update for the roll grab, including proposed and applied button states, stick input, and gate.
    pub fn mapped(
        &mut self,
        proposed: u32,
        applied: u32,
        pressed: u32,
        mapped: u32,
        stick: [i8; 2],
        gate: &'static str,
        now: Instant,
    ) {
        let Some(report) = self.active.as_mut() else {
            return;
        };
        let button = report.last.grab_button;
        let catch = mapped & MAPPED_CATCH != 0;
        report.mapper_polls = report.mapper_polls.saturating_add(1);
        report.proposed_polls += u32::from(proposed & button != 0);
        report.applied_polls += u32::from(proposed & applied & button != 0);
        report.catch_polls += u32::from(proposed & button != 0 && catch);
        let key = (proposed & button, applied & button, catch, gate);
        if proposed & button != 0 && catch && report.last_mapping.is_none_or(|last| !last.2) {
            report.catch_edges += 1;
        }
        if report.last_mapping != Some(key) {
            report.push(
                report.elapsed(now),
                Event::Mapper {
                    proposed,
                    applied,
                    pressed,
                    mapped,
                    stick,
                    gate,
                },
            );
        }
        report.last_mapping = Some(key);
    }
    /// Finishes the active report with the given reason, writing it to the log and marking it as completed.
    pub fn finish(&mut self, reason: &'static str) {
        if let Some(mut report) = self.active.take() {
            report.end = reason;
            crate::logging::write_grab(report);
            self.completed = true;
        }
    }
    /// Resets the grab trace by finishing the current report with the given reason and clearing the completion flag.
    pub fn reset(&mut self, reason: &'static str) {
        self.finish(reason);
        self.completed = false;
    }
}

#[cfg(test)]
/// Host-side regressions for bounded grab-trace sampling and evidence.
mod tests {
    use super::*;
    use kirby_assist_core::native_input::MAPPER_R;
    use std::time::Duration;

    /// Creates a minimal target-bearing fixture for trace tests.
    fn snapshot(phase: &'static str) -> RollGrabSnapshot {
        RollGrabSnapshot {
            target: Some(42),
            phase,
            grab_button: MAPPER_R,
            can_grab: true,
            ..RollGrabSnapshot::default()
        }
    }
    #[test]
    /// Verifies duplicate polls do not add repeated request or catch edges.
    fn repeated_polls_do_not_fill_trace_or_invent_request_edges() {
        let now = Instant::now();
        let mut trace = GrabTrace::new();
        let s = snapshot("roll_request_grab");
        for _ in 0..1000 {
            trace.observe(Some(s), Some(6), now);
            trace.mapped(MAPPER_R, MAPPER_R, 0, MAPPED_CATCH, [0, 0], "ledge", now);
        }
        let report = trace.active.as_ref().unwrap();
        assert_eq!(report.requests, 1);
        assert_eq!(report.catch_edges, 1);
        assert_eq!(report.written, 2);
        assert!(report
            .format()
            .contains("mapped_CATCH_no_native_start_observed"));
    }
    #[test]
    /// Verifies reports distinguish undelivered overlays from wrong button mapping.
    fn mapper_evidence_distinguishes_expired_overlay_from_wrong_mapping() {
        let now = Instant::now();
        let mut trace = GrabTrace::new();
        trace.observe(Some(snapshot("roll_request_grab")), Some(6), now);
        trace.mapped(MAPPER_R, 0, 0, 0, [0, 0], "expired_observation", now);
        assert!(trace
            .active
            .as_ref()
            .unwrap()
            .format()
            .contains("overlay_not_delivered"));
        // GUARD is not CATCH; a shield mapping must not confirm grab delivery.
        trace.mapped(
            MAPPER_R,
            MAPPER_R,
            MAPPER_R,
            kirby_assist_core::native_input::MAPPED_GUARD,
            [0, 0],
            "ledge",
            now,
        );
        assert!(trace
            .active
            .as_ref()
            .unwrap()
            .format()
            .contains("raw_button_not_mapped_to_CATCH"));
        trace.mapped(
            MAPPER_R,
            MAPPER_R,
            MAPPER_R,
            MAPPED_CATCH,
            [0, 0],
            "ledge",
            now,
        );
        assert_eq!(trace.active.as_ref().unwrap().catch_edges, 1);
    }
    #[test]
    /// Verifies capture evidence remains independent from bounded ring history.
    fn acceptance_evidence_survives_a_full_ring_and_capture_is_separate() {
        let now = Instant::now();
        let mut trace = GrabTrace::new();
        trace.observe(Some(snapshot("roll_request_grab")), Some(5), now);
        let mut s = snapshot("roll_wait_native_catch");
        s.native_grabbing = true;
        s.animations = 1;
        trace.observe(Some(s), Some(5), now + Duration::from_millis(16));
        let report = trace.active.as_mut().unwrap();
        for ms in 0..100 {
            report.push(ms, Event::Observation(s));
        }
        let text = report.format();
        assert!(text.contains("first_request"));
        assert!(text.contains("first_native_start"));
        assert!(text.contains("native_grab_started_no_capture_observed"));
        assert_eq!(report.checkpoints.len(), CHECKPOINTS);
        s.captured = true;
        report.last = s;
        assert!(report.format().contains("result=capture_confirmed"));
    }
    #[test]
    /// Verifies progress sampling and held-input completion behavior.
    fn idle_progress_is_sampled_and_completed_trace_does_not_restart_while_held() {
        let now = Instant::now();
        let mut trace = GrabTrace::new();
        let s = snapshot("roll_fast_walk_to_grab");
        for ms in 0..1000 {
            trace.observe(Some(s), Some(6), now + Duration::from_millis(ms));
        }
        assert_eq!(trace.active.as_ref().unwrap().written, 8);
        trace.finish("test_release");
        trace.observe(Some(s), Some(6), now);
        assert!(!trace.active());
        trace.observe(None, Some(6), now);
        trace.observe(Some(s), Some(6), now);
        assert!(trace.active());
    }
}
