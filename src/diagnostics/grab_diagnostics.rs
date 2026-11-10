//! A small flight recorder for roll grabs. Hooks copy numbers into fixed storage;
//! only the background logger formats the completed report. No gameplay decisions.
//! Technical: Captures bounded grab-related traces as copied values for deferred
//! diagnostic reporting.
//! Plain English: This module collects grab troubleshooting data without doing
//! file or console work in game hooks.
use kirby_assist_core::{ledge_trap::RollGrabSnapshot, native_input::MAPPED_CATCH};
use std::{fmt::Write, time::Instant};

/// Technical: Fixed capacity of the bounded diagnostic checkpoint ring.
/// Plain English: The report keeps only this many recent trace points.
const CHECKPOINTS: usize = 32;
/// Technical: Minimum interval between emitted progress updates.
/// Plain English: How often the report can add a progress note.
const PROGRESS_MS: u64 = 125;
/// Technical: Maximum elapsed time before the trace watchdog ends a report.
/// Plain English: How long a grab trace can run before it is stopped.
const WATCHDOG_MS: u64 = 3_000;

/// Represents an event in the roll grab diagnostics, either an observation or a mapper update.
/// Technical: Copied checkpoint payload for fighter observations and mapper delivery.
/// Plain English: One saved observation or input-delivery detail in a grab trace.
#[derive(Clone, Copy)]
enum Event {
    /// Technical: Native grab-related state copied from an observation.
    /// Plain English: A snapshot of what the fighter and target were doing.
    Observation(RollGrabSnapshot),
    /// Technical: Input masks and stick values observed at the mapper boundary.
    /// Plain English: The buttons and stick direction before and after mapping.
    Mapper {
        /// Technical: Buttons proposed by the overlay.
        /// Plain English: Buttons the helper wanted to send.
        proposed: u32,
        /// Technical: Buttons applied before native mapping.
        /// Plain English: Buttons added by the helper before the game processed them.
        applied: u32,
        /// Technical: Buttons pressed on this mapper poll.
        /// Plain English: Buttons that became newly pressed this update.
        pressed: u32,
        /// Technical: Final mapped button mask.
        /// Plain English: Buttons the game received after mapping.
        mapped: u32,
        /// Technical: Final mapped left-stick axis values.
        /// Plain English: The left-stick direction the game received.
        stick: [i8; 2],
        /// Technical: Gate or suppression reason associated with delivery.
        /// Plain English: Why the requested input may have been blocked.
        gate: &'static str,
    },
}
/// Represents a checkpoint in the roll grab diagnostics, storing the elapsed time and the associated event.
/// Technical: Ring-buffer entry pairing an elapsed timestamp with one copied event.
/// Plain English: A timestamped detail in the grab troubleshooting trace.
#[derive(Clone, Copy)]
struct Checkpoint {
    /// Technical: Milliseconds elapsed since report origin.
    /// Plain English: How long after the trace started this point occurred.
    ms: u64,
    /// Technical: Copied diagnostic payload at this point.
    /// Plain English: What happened at this point in the trace.
    event: Event,
}

/// Represents a report for roll grab diagnostics, including metadata, checkpoints, and various counters.
/// Technical: Bounded recorder snapshot describing one roll-grab attempt and its delivery history.
/// Plain English: A troubleshooting record showing why a grab did or did not happen.
pub struct Report {
    /// Technical: Monotonic identifier assigned to this trace.
    /// Plain English: The trace's unique number.
    id: u64,
    /// Technical: Fighter kind associated with the trace, if known.
    /// Plain English: Which fighter the trace was about.
    fighter: Option<i32>,
    /// Technical: Target object identifier associated with the trace, if known.
    /// Plain English: Which opponent the trace was about.
    target: Option<u32>,
    /// Technical: Time origin used to calculate checkpoint durations.
    /// Plain English: When this trace started.
    origin: Instant,
    /// Technical: First captured native grab observation.
    /// Plain English: The fighter state at the beginning of the trace.
    first: RollGrabSnapshot,
    /// Technical: Most recent captured native grab observation.
    /// Plain English: The latest fighter state in the trace.
    last: RollGrabSnapshot,
    /// Technical: Fixed-size circular checkpoint storage.
    /// Plain English: The limited list of recent events kept by the trace.
    checkpoints: [Option<Checkpoint>; CHECKPOINTS],
    /// Technical: Number of checkpoint writes, including overwritten ring entries.
    /// Plain English: How many events have been recorded so far.
    written: usize,
    /// Technical: Count of observation updates.
    /// Plain English: How many fighter snapshots were captured.
    observations: u32,
    /// Technical: Count of grab requests.
    /// Plain English: How many grab requests were made.
    requests: u32,
    /// Technical: Count of mapper polls during the trace.
    /// Plain English: How many controller updates were checked.
    mapper_polls: u32,
    /// Technical: Count of polls where a grab was proposed.
    /// Plain English: How many updates tried to send a grab.
    proposed_polls: u32,
    /// Technical: Count of polls where the grab overlay was applied.
    /// Plain English: How many updates added the grab button.
    applied_polls: u32,
    /// Technical: Count of polls where native mapped CATCH was present.
    /// Plain English: How many updates delivered the mapped grab button.
    catch_polls: u32,
    /// Technical: Count of rising edges on native mapped CATCH.
    /// Plain English: How many times the grab button became newly pressed.
    catch_edges: u32,
    /// Technical: Count of exits classified as whiff or interruption.
    /// Plain English: How many attempts ended without a confirmed grab.
    whiff_exits: u32,
    /// Technical: Whether native grab startup was observed.
    /// Plain English: Whether the game began a grab animation.
    native_started: bool,
    /// Technical: Elapsed timestamp of the most recent progress update.
    /// Plain English: When the trace last recorded progress.
    last_progress_ms: u64,
    /// Technical: Latest elapsed time observed by the recorder.
    /// Plain English: How long the trace has been running.
    latest_ms: u64,
    /// Technical: First checkpoint at which a request was observed.
    /// Plain English: When the first grab request appeared.
    first_request: Option<Checkpoint>,
    /// Technical: First checkpoint at which native grab startup was observed.
    /// Plain English: When the game first started the grab.
    first_native_start: Option<Checkpoint>,
    /// Technical: Most recent checkpoint waiting for native progress.
    /// Plain English: The latest point where the grab was still waiting.
    last_wait: Option<Checkpoint>,
    /// Technical: Most recent mapping inputs and delivery gate summary.
    /// Plain English: The last buttons and reason seen at the mapper.
    last_mapping: Option<(u32, u32, bool, &'static str)>,
    /// Technical: Stable trace termination reason.
    /// Plain English: Why this record stopped.
    end: &'static str,
}
impl Report {
    /// Returns the elapsed time in milliseconds since the report's origin.
    /// Plain English: Tells how many milliseconds have passed since this trace began.
    fn elapsed(&self, now: Instant) -> u64 {
        now.saturating_duration_since(self.origin)
            .as_millis()
            .min(u128::from(u64::MAX)) as u64
    }
    /// Pushes a new checkpoint into the report, recording the elapsed time and event.
    /// Plain English: Adds a recent event to the bounded trace history.
    fn push(&mut self, ms: u64, event: Event) {
        // Preserve the initial checkpoint separately; the ring retains recent
        // evidence and reports overwrite counts instead of allocating/growing.
        self.checkpoints[self.written % CHECKPOINTS] = Some(Checkpoint { ms, event });
        self.written = self.written.saturating_add(1);
    }
    /// Called exclusively on the logger thread, never under the gameplay mutex.
    /// Formats the report into a human-readable string for logging or display.
    /// Plain English: Turns the saved trace into readable troubleshooting text.
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
    /// Technical: Appends the copied native snapshot fields to the report buffer.
    /// Plain English: Adds one timestamped fighter/grab state to the troubleshooting log.
    fn format_observation(text: &mut String, label: &str, ms: u64, s: RollGrabSnapshot) {
        let _ = writeln!(text, "  +{ms}ms {label} gate={} status={:#x}@{:.2} pos={:?} vx={:.3} facing={} endpoint={:?} stand={:?} endpoint_gap={:?} target_gap_facing={:?} usable_reach={:.2} grab_range={:.2} ready(control/act/grab)={}/{}/{} stopped={} safe_ground={:?} enemy_status={:?}@{:?} enemy_pos={:?} vulnerable={:?} opens_in={:?} adjusted_opens_in={:?} actionable_in={:?} active={:?} request={:#x} grab_button={:#x} attempts={} animations={} native_grabbing={} captured={}", s.phase, s.status, s.motion_frame, s.position, s.velocity_x, s.facing, s.endpoint, s.stand_x, s.endpoint.map(|x| x-s.position[0]), s.enemy_position.map(|p| (p[0]-s.position[0])*s.facing), s.usable_reach, s.grab_range, s.can_control, s.can_act, s.can_grab, s.stopped, s.safe_ground, s.enemy_status, s.enemy_frame, s.enemy_position, s.vulnerable, s.vulnerable_in, s.adjusted_opens_in, s.actionable_in, s.active, s.requested, s.grab_button, s.attempts, s.animations, s.native_grabbing, s.captured);
    }
}
/// Represents the trace of a roll grab, including the active report and its completion status.
/// Technical: Owns one active recorder plus completion and identifier state.
/// Plain English: Tracks a single roll-grab attempt and its report.
pub struct GrabTrace {
    /// Technical: Active bounded report, absent when no trace is being collected.
    /// Plain English: The current trace, if one is running.
    active: Option<Report>,
    /// Technical: Whether a completed report awaits delivery.
    /// Plain English: Whether a finished trace is ready to be read.
    completed: bool,
    /// Technical: Monotonic id assigned to the next report.
    /// Plain English: The number used for the next trace.
    next_id: u64,
}
impl GrabTrace {
    /// Creates a new instance of `GrabTrace` with no active report and initial state.
    /// Plain English: Starts a grab trace recorder with nothing recorded.
    pub const fn new() -> Self {
        Self {
            active: None,
            completed: false,
            next_id: 0,
        }
    }
    /// Returns `true` if there is an active report being tracked.
    /// Plain English: Checks whether a grab trace is currently running.
    pub fn active(&self) -> bool {
        self.active.is_some()
    }
    /// Observes a new roll grab snapshot, updating the active report or finishing it if necessary.
    /// Plain English: Adds the latest fighter snapshot and ends the trace if tracking stops.
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
    /// Plain English: Records what the helper tried to send and what the game received.
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
    /// Plain English: Ends the current trace and sends its report to the logger.
    pub fn finish(&mut self, reason: &'static str) {
        if let Some(mut report) = self.active.take() {
            report.end = reason;
            crate::logging::write_grab(report);
            self.completed = true;
        }
    }
    /// Resets the grab trace by finishing the current report with the given reason and clearing the completion flag.
    /// Plain English: Ends the current trace and clears its finished status.
    pub fn reset(&mut self, reason: &'static str) {
        self.finish(reason);
        self.completed = false;
    }
}

#[cfg(test)]
/// Technical: Host-side regressions for bounded grab-trace sampling and evidence.
/// Plain English: Checks that grab logs stay accurate without growing every poll.
mod tests {
    use super::*;
    use kirby_assist_core::native_input::MAPPER_R;
    use std::time::Duration;

    /// Technical: Creates a minimal target-bearing fixture for trace tests.
    /// Plain English: Makes a sample grab state for the tests.
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
    /// Technical: Verifies duplicate polls do not add repeated request or catch edges.
    /// Plain English: Ensures repeated updates do not invent extra grab events.
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
    /// Technical: Verifies reports distinguish undelivered overlays from wrong button mapping.
    /// Plain English: Checks logs explain whether the grab input was lost before mapping.
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
    /// Technical: Verifies capture evidence remains independent from bounded ring history.
    /// Plain English: Ensures capture is still reported even if earlier trace entries roll off.
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
    /// Technical: Verifies progress sampling and held-input completion behavior.
    /// Plain English: Ensures idle polls are sampled slowly and a finished trace stays finished.
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
