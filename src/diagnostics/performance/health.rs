//! Diagnostic-only, fixed counters. No heap hooks, game reads or gameplay waits.
//! Technical: Exposes fixed-size timing counters and health snapshots for reports.
//! Plain English: This submodule tracks how long callback work takes.
use std::{
    fmt::Write,
    sync::{
        atomic::{AtomicUsize, Ordering::Relaxed},
        Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
/// Category index for timing measurements in health reports.
pub enum Lane {
    /// Complete accepted mapping callback, including native game work.
    Mapping,
    /// Menu input and cached-view update lane.
    MenuInput,
    /// Menu text construction lane.
    MenuBuild,
    /// Mod draw callback lane, excluding external renderer work.
    Draw,
    /// Explicit save worker lane.
    Save,
    /// Diagnostic log flush lane.
    LogFlush,
    /// Diagnostic report formatting and write-batch lane.
    LogWork,
}
/// Number of lanes represented by [`Lane`].
const LANES: usize = 7;
/// Display label corresponding to each timing lane index.
const LABELS: [&str; LANES] = [
    "Mapping hook (includes game + observations)",
    "Menu input / cache update",
    "Menu text rebuild",
    "Mod draw callback (not whole renderer)",
    "Save worker",
    "Diagnostic log flush",
    "Diagnostic formatting/write batch",
];
/// Lifetime invocation counters indexed by lane.
static CALLS: [AtomicUsize; LANES] = [const { AtomicUsize::new(0) }; LANES];
#[derive(Clone, Copy)]
/// Counted diagnostic event categories used by health windows.
pub enum Event {
    /// A menu snapshot publication was attempted.
    MenuPublish,
    /// Menu strings were rebuilt.
    MenuBuild,
    /// A menu publication had to retry due to lock contention.
    MenuRetry,
    /// Settings were resolved for a sequence or recipe.
    Resolve,
    /// The selected fighter/profile changed.
    FighterChange,
    /// Match-scoped state was reset.
    StateReset,
    /// A save worker began processing a save request.
    SaveStarted,
    /// A save worker completed successfully.
    SaveCompleted,
    /// A save worker failed.
    SaveFailed,
    /// A diagnostic message was dropped from the bounded queue.
    LogDropped,
    /// A local observation pass was entered.
    Observation,
    /// A duplicate observation was skipped.
    ObservationReuse,
    /// A fighter roster scan was performed.
    RosterScan,
    /// A complete local policy evaluation was performed.
    FullObservation,
    /// A collected observation failed publication freshness checks.
    PublicationDiscard,
    /// One or more requests expired due to staleness.
    InputExpired,
    /// A sequence was canceled after a native animation-clock gap.
    TimelineGap,
    /// A sequence was canceled because its motion clock was invalid or timed out.
    TimelineClockAbort,
}
/// Number of event categories represented by [`Event`].
const EVENTS: usize = 18;
/// Human-readable event labels matching the enum discriminants.
const EVENT_NAMES: [&str; EVENTS] = [
    "menu publish attempts",
    "menu text rebuilds",
    "menu lock retries",
    "recipe settings resolutions",
    "fighter/profile switches",
    "match-state resets",
    "save workers started",
    "save workers completed",
    "save failures",
    "logger messages dropped",
    "local observation passes started",
    "duplicate observation passes skipped",
    "opponent roster scans",
    "full local evaluations",
    "observations discarded before publication",
    "input freshness expiry episodes",
    "sequence cancellations: native clock gap",
    "sequence cancellations: invalid clock/timeout",
];
/// Lifetime atomic counters indexed by event discriminant.
static COUNTS: [AtomicUsize; EVENTS] = [const { AtomicUsize::new(0) }; EVENTS];
/// Current approximate logger-queue occupancy.
static DEPTH: AtomicUsize = AtomicUsize::new(0);
/// Maximum queue occupancy observed in the active epoch.
static PEAK_DEPTH: AtomicUsize = AtomicUsize::new(0);
/// Number of active save/report workers.
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
/// Maximum active-worker count observed.
static PEAK_ACTIVE: AtomicUsize = AtomicUsize::new(0);
/// Current retained profile count supplied by menu accounting.
static PROFILES: AtomicUsize = AtomicUsize::new(0);
/// Current retained editor-recipe count.
static RECIPES: AtomicUsize = AtomicUsize::new(0);
/// Current retained draft count.
static DRAFTS: AtomicUsize = AtomicUsize::new(0);
/// Current estimated owned payload bytes.
static RETAINED: AtomicUsize = AtomicUsize::new(0);
/// Peak estimated retained payload bytes.
static PEAK_RETAINED: AtomicUsize = AtomicUsize::new(0);
/// Current cached view payload bytes.
static VIEW: AtomicUsize = AtomicUsize::new(0);
/// Peak cached view payload bytes.
static PEAK_VIEW: AtomicUsize = AtomicUsize::new(0);
/// Number of timing/window updates skipped due to lock contention.
static MISSED: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy, Default)]
/// Accumulated timing statistics for one lane.
struct Timing {
    /// Number of sampled executions.
    samples: u64,
    /// Saturating sum of sampled nanoseconds.
    nanos: u64,
    /// Maximum sampled duration in nanoseconds.
    max: u64,
    /// Count of samples lasting at least one millisecond.
    over_ms: u64,
}
#[derive(Clone, Copy, Default)]
/// Snapshot of retained menu counts and payload estimate.
struct Memory {
    /// Retained profile count.
    profiles: usize,
    /// Retained editor recipe count.
    recipes: usize,
    /// Retained draft count.
    drafts: usize,
    /// Estimated owned payload bytes.
    bytes: usize,
}
impl Memory {
    /// Reads current atomic memory-accounting values.
    fn read() -> Self {
        Self {
            profiles: PROFILES.load(Relaxed),
            recipes: RECIPES.load(Relaxed),
            drafts: DRAFTS.load(Relaxed),
            bytes: RETAINED.load(Relaxed),
        }
    }
    /// Compares counts while intentionally ignoring payload size.
    fn same_items(self, old: Self) -> bool {
        (self.profiles, self.recipes, self.drafts) == (old.profiles, old.recipes, old.drafts)
    }
}
/// Rolling interval data and lifetime-counter baselines for a health report.
struct Window {
    /// Diagnostic generation to which this rolling window belongs.
    epoch: usize,
    /// Start time of the current aggregate interval.
    since: Option<Instant>,
    /// Per-lane sampled duration aggregates.
    timing: [Timing; LANES],
    /// Most recent draw-callback timestamp.
    last_draw: Option<Instant>,
    /// Largest observed interval between draw callbacks in microseconds.
    draw_gap_max: u64,
    /// Counts of draw gaps exceeding 25, 50, and 100 milliseconds.
    gaps: [u64; 3],
    /// Lifetime lane counters captured at the previous window boundary.
    last_calls: [usize; LANES],
    /// Lifetime event counters captured at the previous window boundary.
    last_events: [usize; EVENTS],
    /// Previous memory snapshot for growth comparison.
    memory: Option<Memory>,
    /// Consecutive windows with stable counts and increasing payload.
    growth_streak: u8,
    /// Consecutive windows with active save workers but no completion/failure.
    save_streak: u8,
}
impl Window {
    /// Initializes zeroed timing, counter baselines, and streak state.
    const fn new() -> Self {
        Self {
            epoch: 0,
            since: None,
            timing: [Timing {
                samples: 0,
                nanos: 0,
                max: 0,
                over_ms: 0,
            }; LANES],
            last_draw: None,
            draw_gap_max: 0,
            gaps: [0; 3],
            last_calls: [0; LANES],
            last_events: [0; EVENTS],
            memory: None,
            growth_streak: 0,
            save_streak: 0,
        }
    }
    /// Start an empty window; lifetime counters stay, only their baselines move.
    /// Resets interval aggregates and snapshots cumulative counters for deltas.
    fn restart(&mut self, epoch: usize) {
        *self = Self::new();
        self.epoch = epoch;
        self.last_calls = std::array::from_fn(|i| CALLS[i].load(Relaxed));
        self.last_events = std::array::from_fn(|i| COUNTS[i].load(Relaxed));
    }
}
/// Protects the current bounded health-report aggregation window.
static WINDOW: Mutex<Window> = Mutex::new(Window::new());
/// Restarts a window when its diagnostic generation changes.
fn sync_epoch(w: &mut Window, epoch: usize) {
    if w.epoch != epoch {
        w.restart(epoch);
    }
}

/// Increments one lifetime event counter.
pub fn event(e: Event) {
    add_event(e, 1);
}
/// Adds `n` occurrences to an event's lifetime counter.
pub fn add_event(e: Event, n: usize) {
    if !super::enabled() {
        return;
    }
    COUNTS[e as usize].fetch_add(n, Relaxed);
}
/// Reservations precede try_send so a fast receiver cannot underflow the gauge.
/// Occupancy is approximate during handoff and capped at the actual channel limit.
/// Reserves one approximate queue slot and updates the capped peak.
pub fn queue_enter() {
    if !super::enabled() {
        return;
    }
    let n = DEPTH.fetch_add(1, Relaxed) + 1;
    PEAK_DEPTH.fetch_max(n.min(128), Relaxed);
}
/// Releases one previously reserved queue slot.
pub fn queue_leave() {
    if super::enabled() {
        decrement(&DEPTH);
    }
}

fn decrement(counter: &AtomicUsize) {
    let value = counter.load(Relaxed);
    let _ = counter.compare_exchange(value, value.saturating_sub(1), Relaxed, Relaxed);
}
/// Publishes current retained-menu counts and updates the payload peak.
pub fn retained(profiles: usize, recipes: usize, drafts: usize, bytes: usize) {
    if !super::enabled() {
        return;
    }
    PROFILES.store(profiles, Relaxed);
    RECIPES.store(recipes, Relaxed);
    DRAFTS.store(drafts, Relaxed);
    RETAINED.store(bytes, Relaxed);
    PEAK_RETAINED.fetch_max(bytes, Relaxed);
}
/// Publishes current cached-view bytes and updates the view peak.
pub fn view_bytes(bytes: usize) {
    if !super::enabled() {
        return;
    }
    VIEW.store(bytes, Relaxed);
    PEAK_VIEW.fetch_max(bytes, Relaxed);
}
/// RAII marker for a currently active background worker.
pub struct Worker(Option<usize>);
static WORKER_SESSION: AtomicUsize = AtomicUsize::new(0);
impl Worker {
    /// Increments active-worker counters and records a save-start event.
    pub fn start() -> Self {
        if !super::enabled() {
            return Self(None);
        }
        let epoch = WORKER_SESSION.load(Relaxed);
        event(Event::SaveStarted);
        let n = ACTIVE.fetch_add(1, Relaxed) + 1;
        PEAK_ACTIVE.fetch_max(n, Relaxed);
        Self(Some(epoch))
    }
}
impl Drop for Worker {
    /// Decrements the active-worker count when the marker is dropped.
    fn drop(&mut self) {
        if self.0 == Some(WORKER_SESSION.load(Relaxed)) {
            decrement(&ACTIVE);
        }
    }
}

/// RAII timer for a sampled health-report lane.
pub struct Trace {
    /// Timing-lane index recorded by this guard.
    lane: usize,
    /// Diagnostic generation captured at timer creation.
    epoch: usize,
    /// Start instant for sampled timing, absent when not selected.
    start: Option<Instant>,
}
impl Trace {
    /// Starts a sampled lane timer according to the active sampling cadence.
    pub fn new(lane: Lane) -> Self {
        let epoch = super::EPOCH.load(Relaxed);
        let start = if super::enabled() {
            let call = CALLS[lane as usize].fetch_add(1, Relaxed);
            (matches!(lane, Lane::MenuBuild | Lane::Save | Lane::LogFlush) || call % 16 == 0)
                .then(Instant::now)
        } else {
            None
        };
        Self {
            lane: lane as usize,
            epoch,
            start,
        }
    }
}
impl Drop for Trace {
    /// Accumulates duration only if the timer still matches the active epoch.
    fn drop(&mut self) {
        let Some(start) = self.start else {
            return;
        };
        let ns = super::clamp_u64(start.elapsed().as_nanos());
        if !super::enabled() || self.epoch != super::EPOCH.load(Relaxed) {
            return;
        }
        if let Ok(mut w) = WINDOW.try_lock() {
            if self.epoch != super::EPOCH.load(Relaxed) {
                return;
            }
            sync_epoch(&mut w, self.epoch);
            w.since.get_or_insert(start);
            let t = &mut w.timing[self.lane];
            t.samples += 1;
            t.nanos = t.nanos.saturating_add(ns);
            t.max = t.max.max(ns);
            t.over_ms += u64::from(ns >= 1_000_000);
        } else {
            MISSED.fetch_add(1, Relaxed);
        }
    }
}
/// Updates callback-gap timing and threshold buckets without blocking.
pub fn draw_tick(now: Instant) {
    if !super::enabled() {
        return;
    }
    if let Ok(mut w) = WINDOW.try_lock() {
        sync_epoch(&mut w, super::EPOCH.load(Relaxed));
        w.since.get_or_insert(now);
        if let Some(last) = w.last_draw.replace(now) {
            let us = super::clamp_u64(now.saturating_duration_since(last).as_micros());
            w.draw_gap_max = w.draw_gap_max.max(us);
            for (i, limit) in [25_000, 50_000, 100_000].into_iter().enumerate() {
                w.gaps[i] += u64::from(us >= limit);
            }
        }
    } else {
        MISSED.fetch_add(1, Relaxed);
    }
}
pub(super) fn reset_session() {
    WORKER_SESSION.fetch_add(1, Relaxed);
    for value in CALLS.iter().chain(COUNTS.iter()).chain([
        &DEPTH,
        &PEAK_DEPTH,
        &ACTIVE,
        &PEAK_ACTIVE,
        &PROFILES,
        &RECIPES,
        &DRAFTS,
        &RETAINED,
        &PEAK_RETAINED,
        &VIEW,
        &PEAK_VIEW,
        &MISSED,
    ]) {
        value.store(0, Relaxed);
    }
    clear_window();
}

/// Restarts the aggregate window while preserving lifetime atomics.
pub fn clear_window() {
    if let Ok(mut w) = WINDOW.try_lock() {
        w.restart(super::EPOCH.load(Relaxed));
    }
}
/// Produces a report after five seconds of data, using try-lock snapshots.
pub fn report() -> Option<String> {
    if !super::enabled() {
        return None;
    }
    let now = Instant::now();
    let epoch = super::EPOCH.load(Relaxed);
    let memory = Memory::read();
    let workers = ACTIVE.load(Relaxed);
    let (timing, calls, delta, events, event_delta, gap, gaps, elapsed, growth_streak, save_streak) = {
        let mut w = WINDOW.try_lock().ok()?;
        if epoch != super::EPOCH.load(Relaxed) {
            return None;
        }
        sync_epoch(&mut w, epoch);
        let elapsed = now.saturating_duration_since(w.since?);
        if elapsed < Duration::from_secs(5) {
            return None;
        }
        let calls = std::array::from_fn::<_, LANES, _>(|i| CALLS[i].load(Relaxed));
        let events = std::array::from_fn::<_, EVENTS, _>(|i| COUNTS[i].load(Relaxed));
        let delta = std::array::from_fn::<_, LANES, _>(|i| calls[i].wrapping_sub(w.last_calls[i]));
        let event_delta =
            std::array::from_fn::<_, EVENTS, _>(|i| events[i].wrapping_sub(w.last_events[i]));
        let unexplained_growth = w
            .memory
            .is_some_and(|old| memory.same_items(old) && memory.bytes > old.bytes)
            && event_delta[Event::MenuBuild as usize] == 0
            && event_delta[Event::FighterChange as usize] == 0;
        w.growth_streak = if unexplained_growth {
            w.growth_streak.saturating_add(1)
        } else {
            0
        };
        w.save_streak = if workers > 0
            && event_delta[Event::SaveCompleted as usize] == 0
            && event_delta[Event::SaveFailed as usize] == 0
        {
            w.save_streak.saturating_add(1)
        } else {
            0
        };
        w.memory = Some(memory);
        let result = (
            w.timing,
            calls,
            delta,
            events,
            event_delta,
            w.draw_gap_max,
            w.gaps,
            elapsed,
            w.growth_streak,
            w.save_streak,
        );
        w.timing = [Timing::default(); LANES];
        w.draw_gap_max = 0;
        w.gaps = [0; 3];
        w.since = Some(now);
        w.last_calls = calls;
        w.last_events = events;
        result
    };
    let mut out = format!(
        "RUNTIME HEALTH: {:.2}s window; cumulative events since plugin start; timing calls while Perf ON\n",
        elapsed.as_secs_f64()
    );
    let _ = writeln!(
        out,
        "TEST CONTEXT: perf_epoch={epoch}; {}",
        super::latency_text(super::test_latency())
    );
    out.push_str(&review(
        &timing,
        &delta,
        &event_delta,
        gap,
        workers,
        DEPTH.load(Relaxed),
        growth_streak,
        save_streak,
    ));
    let seconds = elapsed.as_secs_f64().max(0.001);
    let full = event_delta[Event::FullObservation as usize];
    let _ = writeln!(out,"RATES: mapping {:.1}/s; full local evaluations {:.1}/s; roster {:.1}/s; draw callbacks {:.1}/s; full/draw {:.2} (callbacks are not simulation frames)",
        delta[Lane::Mapping as usize] as f64/seconds,full as f64/seconds,event_delta[Event::RosterScan as usize] as f64/seconds,
        delta[Lane::Draw as usize] as f64/seconds,full as f64/delta[Lane::Draw as usize].max(1) as f64);
    for i in 0..LANES {
        let t = timing[i];
        let _ = writeln!(
            out,
            "{}: calls {} (+{}), samples {}, avg {:.1}us max {:.1}us, samples >=1ms {}",
            LABELS[i],
            calls[i],
            delta[i],
            t.samples,
            t.nanos as f64 / t.samples.max(1) as f64 / 1000.0,
            t.max as f64 / 1000.0,
            t.over_ms
        );
    }
    let _=writeln!(out,"Hot paths sampled 1/16; rebuild/save/flush calls timed individually. Nested times overlap; mapping includes vanilla game work.\nDraw callback gaps: max {:.2}ms; >=25/50/100ms {:?} (not an FPS meter; transitions/pauses may cause gaps)",gap as f64/1000.0,gaps);
    for i in 0..EVENTS {
        let _ = writeln!(
            out,
            "{}: {} (+{})",
            EVENT_NAMES[i], events[i], event_delta[i]
        );
    }
    let _ = writeln!(out,
        "Retained menus: profiles {}, editor recipes {}, drafts {}, estimated owned payload {} bytes (peak {}); view {} bytes (peak {})\n\
         Save: active workers {} (peak {}); reserved stack {} bytes per active worker\n\
         Logger: approximately queued {} (peak {}, capacity 128), timing/gap updates skipped on contention {}\n\
         Memory estimates exclude allocator/BTree node overhead, immutable startup catalogs, ImGui/game/SDK allocations and total system memory; growth is evidence to investigate, not proof of a leak.",
        PROFILES.load(Relaxed), RECIPES.load(Relaxed), DRAFTS.load(Relaxed),
        RETAINED.load(Relaxed), PEAK_RETAINED.load(Relaxed), VIEW.load(Relaxed), PEAK_VIEW.load(Relaxed),
        ACTIVE.load(Relaxed), PEAK_ACTIVE.load(Relaxed), 2 * 1024 * 1024,
        DEPTH.load(Relaxed).min(128), PEAK_DEPTH.load(Relaxed), MISSED.load(Relaxed));
    if !super::enabled() || epoch != super::EPOCH.load(Relaxed) {
        return None;
    }
    Some(out)
}

/// Worker-only investigation hints. Thresholds are not a diagnosis of heat,
/// network delay, whole-process CPU use, GPU load, or a verified memory leak.
fn review(
    t: &[Timing; LANES],
    calls: &[usize; LANES],
    e: &[usize; EVENTS],
    gap: u64,
    workers: usize,
    queued: usize,
    growth: u8,
    save_age: u8,
) -> String {
    let mut out = String::from("TRIAGE: investigation hints, not proof of cause\n");
    let mut flags = 0;
    let mut flag = |code: &str, message: &str| {
        flags += 1;
        let _ = writeln!(out, "REVIEW [{code}]: {message}");
    };
    if e[Event::InputExpired as usize] > 0 {
        flag("INPUT_EXPIRY","35ms freshness protection suppressed stale input. Compare stalls and cancellations; this is not measured network delay.");
    }
    if e[Event::TimelineGap as usize] > 0 {
        flag("SEQUENCE_GAP","A recipe cancelled after an observed native-clock jump. It did not backfill skipped taps.");
    }
    if e[Event::TimelineClockAbort as usize] > 0 {
        flag("SEQUENCE_CLOCK","A recipe clock was invalid or exceeded its timeout. Inspect the LEDGE RESULT termination reason.");
    }
    if e[Event::PublicationDiscard as usize] > 0 {
        flag("PUBLICATION","An observation was superseded or its session changed before publication. Transitions may explain this.");
    }
    let draw = calls[Lane::Draw as usize];
    if draw > 30 && e[Event::FullObservation as usize] > draw + draw / 2 {
        flag("EVALUATION_RATE","Full local evaluations exceed 1.5 per draw callback. Compare the same scene at 0f; draw callbacks are not simulation frames.");
    }
    if e[Event::MenuBuild as usize] > 30 && e[Event::MenuBuild as usize] > draw / 2 {
        flag("MENU_REBUILDS","Frequent text rebuilds. Expected while editing; investigate if controls and selections were untouched.");
    }
    if t[Lane::Mapping as usize].max >= 2_000_000 {
        flag("MAPPING_TIME","A sampled inclusive mapping call exceeded 2ms. Includes vanilla mapping/nested work; do not sum nested rows.");
    }
    if t[Lane::Draw as usize].max >= 2_000_000 {
        flag(
            "DRAW_TIME",
            "The mod draw callback exceeded 2ms. This excludes the rest of the renderer.",
        );
    }
    if t[Lane::LogFlush as usize].max >= 5_000_000 || t[Lane::LogWork as usize].max >= 5_000_000 {
        flag("LOGGER_COST","A diagnostic write/format batch exceeded 5ms on its worker. Logging may affect the test; disable performance sampling to compare.");
    }
    if workers > 1 || save_age >= 2 {
        flag("SAVE_WORKER","Multiple save workers or a worker remains active across windows without completion. Inspect save counts and SD activity.");
    }
    if growth >= 3 {
        flag("RETAINED_GROWTH","Estimated payload rose across three windows with unchanged item counts and no text rebuild/profile switch. Evidence to investigate, not proof of a leak.");
    }
    if queued >= 96 || e[Event::LogDropped as usize] > 0 {
        flag("LOGGER_PRESSURE","The bounded logger queue is busy or dropped messages. Missing evidence may be due to diagnostic load.");
    }
    if gap >= 100_000 {
        flag("CALLBACK_GAP","A draw-callback gap exceeded 100ms. Pauses/loading/network stalls can cause this; it is not an FPS reading.");
    }
    if flags == 0 {
        out.push_str("TRIAGE: no configured threshold crossed; unmeasured heat/GPU/system work is not ruled out.\n");
    }
    out
}

#[cfg(test)]
/// Exercises health-window collection and reset behavior for unit tests.
pub(super) fn exercise_report() {
    retained(2, 3, 4, 500);
    view_bytes(100);
    let builds = COUNTS[Event::MenuBuild as usize].load(Relaxed);
    event(Event::MenuBuild);
    {
        let _trace = Trace::new(Lane::Mapping);
    }
    let now = Instant::now();
    draw_tick(now);
    draw_tick(now + Duration::from_millis(30));
    WINDOW.lock().unwrap().since = Some(now - Duration::from_secs(6));
    let text = report().unwrap();
    assert!(text.contains("profiles 2, editor recipes 3, drafts 4"));
    assert!(text.contains("payload 500 bytes"));
    assert!(text.contains("Mapping hook (includes game + observations)"));
    assert!(text.contains("[1, 0, 0]"));
    clear_window();
    assert!(report().is_none());
    WINDOW.lock().unwrap().epoch = super::EPOCH.load(Relaxed).wrapping_sub(1);
    draw_tick(Instant::now());
    assert_eq!(WINDOW.lock().unwrap().epoch, super::EPOCH.load(Relaxed));
    assert_eq!(
        COUNTS[Event::MenuBuild as usize].load(Relaxed),
        builds + 1,
        "window resets must retain lifetime rebuild counts"
    );
}

#[cfg(test)]
/// Regressions for health-triage criteria and their diagnostic limitations.
#[path = "../../../tests/unit/diagnostics/performance/triage_tests.rs"]
mod triage_tests;
