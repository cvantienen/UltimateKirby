//! Opt-in diagnostics only. Sample one in sixteen accepted local observations;
//! fixed counters on the game thread, report formatting on the existing writer.
//! Technical: Samples diagnostic timing/counter data with bounded atomics and
//! optional per-lane traces.
//! Plain English: This module measures plugin work only when diagnostic logging
//! is enabled.
pub mod health {
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
    /// Technical: Category index for timing measurements in health reports.
    /// Plain English: Names the kind of work whose time is counted.
    pub enum Lane {
        /// Technical: Complete accepted mapping callback, including native game work.
        /// Plain English: Time for the whole controller-mapping callback.
        Mapping,
        /// Technical: Menu input and cached-view update lane.
        /// Plain English: Time spent handling menu input.
        MenuInput,
        /// Technical: Menu text construction lane.
        /// Plain English: Time spent rebuilding menu labels.
        MenuBuild,
        /// Technical: Mod draw callback lane, excluding external renderer work.
        /// Plain English: Time spent in the plugin's drawing callback.
        Draw,
        /// Technical: Explicit save worker lane.
        /// Plain English: Time spent saving settings.
        Save,
        /// Technical: Diagnostic log flush lane.
        /// Plain English: Time spent flushing log files.
        LogFlush,
        /// Technical: Diagnostic report formatting and write-batch lane.
        /// Plain English: Time spent preparing and writing diagnostic reports.
        LogWork,
    }
    /// Technical: Number of lanes represented by [`Lane`].
    /// Plain English: How many types of work the health recorder tracks.
    const LANES: usize = 7;
    /// Technical: Display label corresponding to each timing lane index.
    /// Plain English: Names shown beside timing measurements.
    const LABELS: [&str; LANES] = [
        "Mapping hook (includes game + observations)",
        "Menu input / cache update",
        "Menu text rebuild",
        "Mod draw callback (not whole renderer)",
        "Save worker",
        "Diagnostic log flush",
        "Diagnostic formatting/write batch",
    ];
    /// Technical: Lifetime invocation counters indexed by lane.
    /// Plain English: Counts how often each kind of work runs.
    static CALLS: [AtomicUsize; LANES] = [const { AtomicUsize::new(0) }; LANES];
    #[derive(Clone, Copy)]
    /// Technical: Counted diagnostic event categories used by health windows.
    /// Plain English: Names notable events the report keeps count of.
    pub enum Event {
        /// Technical: A menu snapshot publication was attempted.
        /// Plain English: The menu view was updated.
        MenuPublish,
        /// Technical: Menu strings were rebuilt.
        /// Plain English: The plugin regenerated menu text.
        MenuBuild,
        /// Technical: A menu publication had to retry due to lock contention.
        /// Plain English: A menu update could not proceed right away.
        MenuRetry,
        /// Technical: Settings were resolved for a sequence or recipe.
        /// Plain English: The plugin looked up settings for a selected move.
        Resolve,
        /// Technical: The selected fighter/profile changed.
        /// Plain English: The player switched to a different character.
        FighterChange,
        /// Technical: Match-scoped state was reset.
        /// Plain English: The plugin cleared state for a match transition.
        StateReset,
        /// Technical: A save worker began processing a save request.
        /// Plain English: A settings save started.
        SaveStarted,
        /// Technical: A save worker completed successfully.
        /// Plain English: A settings save finished.
        SaveCompleted,
        /// Technical: A save worker failed.
        /// Plain English: A settings save did not succeed.
        SaveFailed,
        /// Technical: A diagnostic message was dropped from the bounded queue.
        /// Plain English: A log message could not fit in the waiting queue.
        LogDropped,
        /// Technical: A local observation pass was entered.
        /// Plain English: The plugin started checking fighter state.
        Observation,
        /// Technical: A duplicate observation was skipped.
        /// Plain English: A repeated state check was avoided.
        ObservationReuse,
        /// Technical: A fighter roster scan was performed.
        /// Plain English: The plugin listed the current fighters.
        RosterScan,
        /// Technical: A complete local policy evaluation was performed.
        /// Plain English: All active helper checks were run.
        FullObservation,
        /// Technical: A collected observation failed publication freshness checks.
        /// Plain English: Old fighter data was discarded instead of being used.
        PublicationDiscard,
        /// Technical: One or more requests expired due to staleness.
        /// Plain English: A helper request was too old to use.
        InputExpired,
        /// Technical: A sequence was canceled after a native animation-clock gap.
        /// Plain English: A sequence stopped because the fighter's timing jumped.
        TimelineGap,
        /// Technical: A sequence was canceled because its motion clock was invalid or timed out.
        /// Plain English: A sequence stopped because its animation timing was unusable.
        TimelineClockAbort,
    }
    /// Technical: Number of event categories represented by [`Event`].
    /// Plain English: How many event types the report counts.
    const EVENTS: usize = 18;
    /// Technical: Human-readable event labels matching the enum discriminants.
    /// Plain English: Names shown beside the event totals.
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
    /// Technical: Lifetime atomic counters indexed by event discriminant.
    /// Plain English: Running totals for each diagnostic event.
    static COUNTS: [AtomicUsize; EVENTS] = [const { AtomicUsize::new(0) }; EVENTS];
    /// Technical: Current approximate logger-queue occupancy.
    /// Plain English: How many log messages are currently waiting.
    static DEPTH: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Maximum queue occupancy observed in the active epoch.
    /// Plain English: The highest number of waiting log messages seen.
    static PEAK_DEPTH: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Number of active save/report workers.
    /// Plain English: How many background workers are running now.
    static ACTIVE: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Maximum active-worker count observed.
    /// Plain English: The most background workers that ran at once.
    static PEAK_ACTIVE: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Current retained profile count supplied by menu accounting.
    /// Plain English: How many fighter profiles are retained.
    static PROFILES: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Current retained editor-recipe count.
    /// Plain English: How many saved editor recipes are retained.
    static RECIPES: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Current retained draft count.
    /// Plain English: How many unsaved drafts are retained.
    static DRAFTS: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Current estimated owned payload bytes.
    /// Plain English: Approximate memory used by saved menu data.
    static RETAINED: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Peak estimated retained payload bytes.
    /// Plain English: Largest saved-menu memory estimate seen.
    static PEAK_RETAINED: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Current cached view payload bytes.
    /// Plain English: Approximate memory used by the current menu display.
    static VIEW: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Peak cached view payload bytes.
    /// Plain English: Largest menu-display memory estimate seen.
    static PEAK_VIEW: AtomicUsize = AtomicUsize::new(0);
    /// Technical: Number of timing/window updates skipped due to lock contention.
    /// Plain English: Counts diagnostic samples missed because state was busy.
    static MISSED: AtomicUsize = AtomicUsize::new(0);
    #[derive(Clone, Copy, Default)]
    /// Technical: Accumulated timing statistics for one lane.
    /// Plain English: Stores sample count and elapsed time for one kind of work.
    struct Timing {
        /// Technical: Number of sampled executions.
        /// Plain English: How many timing samples were recorded.
        samples: u64,
        /// Technical: Saturating sum of sampled nanoseconds.
        /// Plain English: Total time across recorded samples.
        nanos: u64,
        /// Technical: Maximum sampled duration in nanoseconds.
        /// Plain English: Longest sample recorded.
        max: u64,
        /// Technical: Count of samples lasting at least one millisecond.
        /// Plain English: How many samples took one millisecond or longer.
        over_ms: u64,
    }
    #[derive(Clone, Copy, Default)]
    /// Technical: Snapshot of retained menu counts and payload estimate.
    /// Plain English: Current counts and estimated size of saved menu data.
    struct Memory {
        /// Technical: Retained profile count.
        /// Plain English: Number of profiles kept in memory.
        profiles: usize,
        /// Technical: Retained editor recipe count.
        /// Plain English: Number of recipes kept in memory.
        recipes: usize,
        /// Technical: Retained draft count.
        /// Plain English: Number of drafts kept in memory.
        drafts: usize,
        /// Technical: Estimated owned payload bytes.
        /// Plain English: Approximate memory used by those items.
        bytes: usize,
    }
    impl Memory {
        /// Technical: Reads current atomic memory-accounting values.
        /// Plain English: Takes a snapshot of menu memory counts.
        fn read() -> Self {
            Self {
                profiles: PROFILES.load(Relaxed),
                recipes: RECIPES.load(Relaxed),
                drafts: DRAFTS.load(Relaxed),
                bytes: RETAINED.load(Relaxed),
            }
        }
        /// Technical: Compares counts while intentionally ignoring payload size.
        /// Plain English: Checks whether the number of items stayed the same.
        fn same_items(self, old: Self) -> bool {
            (self.profiles, self.recipes, self.drafts) == (old.profiles, old.recipes, old.drafts)
        }
    }
    /// Technical: Rolling interval data and lifetime-counter baselines for a health report.
    /// Plain English: Holds the measurements and previous totals used to make each report.
    struct Window {
        /// Technical: Diagnostic generation to which this rolling window belongs.
        /// Plain English: Which enabled measurement period these numbers describe.
        epoch: usize,
        /// Technical: Start time of the current aggregate interval.
        /// Plain English: When this report window began.
        since: Option<Instant>,
        /// Technical: Per-lane sampled duration aggregates.
        /// Plain English: Timing totals for each kind of work.
        timing: [Timing; LANES],
        /// Technical: Most recent draw-callback timestamp.
        /// Plain English: When the last plugin draw callback ran.
        last_draw: Option<Instant>,
        /// Technical: Largest observed interval between draw callbacks in microseconds.
        /// Plain English: Longest pause between two plugin draw updates.
        draw_gap_max: u64,
        /// Technical: Counts of draw gaps exceeding 25, 50, and 100 milliseconds.
        /// Plain English: How often drawing updates were widely spaced.
        gaps: [u64; 3],
        /// Technical: Lifetime lane counters captured at the previous window boundary.
        /// Plain English: Earlier call totals used to calculate new activity.
        last_calls: [usize; LANES],
        /// Technical: Lifetime event counters captured at the previous window boundary.
        /// Plain English: Earlier event totals used to calculate new activity.
        last_events: [usize; EVENTS],
        /// Technical: Previous memory snapshot for growth comparison.
        /// Plain English: Earlier menu memory counts used to check for unexplained growth.
        memory: Option<Memory>,
        /// Technical: Consecutive windows with stable counts and increasing payload.
        /// Plain English: How many report periods showed unexplained memory growth.
        growth_streak: u8,
        /// Technical: Consecutive windows with active save workers but no completion/failure.
        /// Plain English: How long a save worker has appeared stuck.
        save_streak: u8,
    }
    impl Window {
        /// Technical: Initializes zeroed timing, counter baselines, and streak state.
        /// Plain English: Starts an empty performance report window.
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
        /// Technical: Resets interval aggregates and snapshots cumulative counters for deltas.
        /// Plain English: Begins a fresh report period without erasing lifetime counts.
        fn restart(&mut self, epoch: usize) {
            *self = Self::new();
            self.epoch = epoch;
            self.last_calls = std::array::from_fn(|i| CALLS[i].load(Relaxed));
            self.last_events = std::array::from_fn(|i| COUNTS[i].load(Relaxed));
        }
    }
    /// Technical: Protects the current bounded health-report aggregation window.
    /// Plain English: Holds the timing totals being collected for the next report.
    static WINDOW: Mutex<Window> = Mutex::new(Window::new());
    /// Technical: Restarts a window when its diagnostic generation changes.
    /// Plain English: Clears old period totals when performance settings are reset.
    fn sync_epoch(w: &mut Window, epoch: usize) {
        if w.epoch != epoch {
            w.restart(epoch);
        }
    }

    /// Technical: Increments one lifetime event counter.
    /// Plain English: Records that a diagnostic event happened once.
    pub fn event(e: Event) {
        add_event(e, 1);
    }
    /// Technical: Adds `n` occurrences to an event's lifetime counter.
    /// Plain English: Adds several occurrences to the event total at once.
    pub fn add_event(e: Event, n: usize) {
        COUNTS[e as usize].fetch_add(n, Relaxed);
    }
    /// Reservations precede try_send so a fast receiver cannot underflow the gauge.
    /// Occupancy is approximate during handoff and capped at the actual channel limit.
    /// Technical: Reserves one approximate queue slot and updates the capped peak.
    /// Plain English: Counts a message before it is handed to the logger queue.
    pub fn queue_enter() {
        let n = DEPTH.fetch_add(1, Relaxed) + 1;
        PEAK_DEPTH.fetch_max(n.min(128), Relaxed);
    }
    /// Technical: Releases one previously reserved queue slot.
    /// Plain English: Removes a message from the waiting-count estimate.
    pub fn queue_leave() {
        DEPTH.fetch_sub(1, Relaxed);
    }
    /// Technical: Publishes current retained-menu counts and updates the payload peak.
    /// Plain English: Updates memory estimates for profiles, recipes, and drafts.
    pub fn retained(profiles: usize, recipes: usize, drafts: usize, bytes: usize) {
        PROFILES.store(profiles, Relaxed);
        RECIPES.store(recipes, Relaxed);
        DRAFTS.store(drafts, Relaxed);
        RETAINED.store(bytes, Relaxed);
        PEAK_RETAINED.fetch_max(bytes, Relaxed);
    }
    /// Technical: Publishes current cached-view bytes and updates the view peak.
    /// Plain English: Updates the estimated size of the displayed menu text.
    pub fn view_bytes(bytes: usize) {
        VIEW.store(bytes, Relaxed);
        PEAK_VIEW.fetch_max(bytes, Relaxed);
    }
    /// Technical: RAII marker for a currently active background worker.
    /// Plain English: Counts a save worker while it is running.
    pub struct Worker;
    impl Worker {
        /// Technical: Increments active-worker counters and records a save-start event.
        /// Plain English: Marks a save worker as started.
        pub fn start() -> Self {
            event(Event::SaveStarted);
            let n = ACTIVE.fetch_add(1, Relaxed) + 1;
            PEAK_ACTIVE.fetch_max(n, Relaxed);
            Self
        }
    }
    impl Drop for Worker {
        /// Technical: Decrements the active-worker count when the marker is dropped.
        /// Plain English: Marks the save worker as finished.
        fn drop(&mut self) {
            ACTIVE.fetch_sub(1, Relaxed);
        }
    }

    /// Technical: RAII timer for a sampled health-report lane.
    /// Plain English: Measures one piece of plugin work when sampling is enabled.
    pub struct Trace {
        /// Technical: Timing-lane index recorded by this guard.
        /// Plain English: Which kind of work this timer measures.
        lane: usize,
        /// Technical: Diagnostic generation captured at timer creation.
        /// Plain English: Prevents an old timer from entering a newer report period.
        epoch: usize,
        /// Technical: Start instant for sampled timing, absent when not selected.
        /// Plain English: When this measured operation began, if it was sampled.
        start: Option<Instant>,
    }
    impl Trace {
        /// Technical: Starts a sampled lane timer according to the active sampling cadence.
        /// Plain English: Begins timing this operation when it is selected for measurement.
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
        /// Technical: Accumulates duration only if the timer still matches the active epoch.
        /// Plain English: Adds the measured time to the current report if it is still current.
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
    /// Technical: Updates callback-gap timing and threshold buckets without blocking.
    /// Plain English: Records how much time passed between plugin draw updates.
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
    /// Technical: Restarts the aggregate window while preserving lifetime atomics.
    /// Plain English: Clears the current report period but keeps overall event totals.
    pub fn clear_window() {
        if let Ok(mut w) = WINDOW.try_lock() {
            w.restart(super::EPOCH.load(Relaxed));
        }
    }
    /// Technical: Produces a report after five seconds of data, using try-lock snapshots.
    /// Plain English: Builds a periodic health report when enough data is available.
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
    /// Plain English: These warnings point to things to investigate, not proven causes.
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
            flag("LOGGER_COST","A diagnostic write/format batch exceeded 5ms on its worker. Logging may affect the test; silent excludes it.");
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
    /// Technical: Exercises health-window collection and reset behavior for unit tests.
    /// Plain English: Sets up sample data and checks that the report is produced and cleared.
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
    /// Technical: Regressions for health-triage criteria and their diagnostic limitations.
    /// Plain English: Checks warnings are useful without claiming more than the data proves.
    mod triage_tests {
        use super::*;
        #[test]
        /// Technical: Verifies distinct event and timing thresholds produce their labeled flags.
        /// Plain English: Checks several separate problems are reported with the right hints.
        fn flags_separate_expiry_reprocessing_logger_and_growth_from_proof_of_heat() {
            let mut t = [Timing::default(); LANES];
            let mut calls = [0; LANES];
            calls[Lane::Draw as usize] = 300;
            let mut e = [0; EVENTS];
            e[Event::FullObservation as usize] = 600;
            e[Event::InputExpired as usize] = 1;
            e[Event::TimelineGap as usize] = 1;
            t[Lane::LogFlush as usize].max = 6_000_000;
            let text = review(&t, &calls, &e, 120_000, 1, 100, 3, 2);
            for code in [
                "INPUT_EXPIRY",
                "SEQUENCE_GAP",
                "EVALUATION_RATE",
                "LOGGER_COST",
                "LOGGER_PRESSURE",
                "RETAINED_GROWTH",
                "SAVE_WORKER",
                "CALLBACK_GAP",
            ] {
                assert!(text.contains(code), "{code}");
            }
            assert!(text.contains("not proof"));
        }
        #[test]
        /// Technical: Verifies a quiet interval does not imply health and thresholds are contextual.
        /// Plain English: Checks the report avoids declaring the whole system healthy.
        fn quiet_window_does_not_claim_system_health_and_expected_edits_need_context() {
            let text = review(
                &[Timing::default(); LANES],
                &[0; LANES],
                &[0; EVENTS],
                0,
                0,
                0,
                0,
                0,
            );
            assert!(text.contains("no configured threshold"));
            assert!(text.contains("not ruled out"));
            assert!(!text.contains("REVIEW ["));
        }
    }
}
use std::{
    ffi::{CStr, CString},
    fmt::Write,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
/// Technical: Index for nested observation-timing regions and their counters.
/// Plain English: Names the parts of fighter observation that can be timed.
pub enum Part {
    /// Technical: Outer aggregate covering all local-observation work.
    /// Plain English: Total time for the complete local observation.
    Total,
    /// Technical: Tech prediction sub-region.
    /// Plain English: Time used to predict tech timing.
    Tech,
    /// Technical: Ledge tracking and setup sub-region.
    /// Plain English: Time spent checking ledge state.
    Ledge,
    /// Technical: Incoming attack defense sub-region.
    /// Plain English: Time spent checking how to defend from attacks.
    Shield,
    /// Technical: Floor-support observation sub-region.
    /// Plain English: Time spent checking the fighter's floor support.
    Ground,
    /// Technical: Roll detection sub-region.
    /// Plain English: Time spent checking for rolls.
    Roll,
    /// Technical: Grab detection sub-region.
    /// Plain English: Time spent checking for grabs.
    Grabs,
    /// Technical: Opponent-facing helper sub-region.
    /// Plain English: Time spent checking the opponent-facing helper.
    Posture,
    /// Technical: Trip and attacker-target sub-region.
    /// Plain English: Time spent checking trip and attacker targets.
    Trip,
    /// Technical: Combo decision sub-region.
    /// Plain English: Time spent checking combo decisions.
    Combo,
}
/// Technical: Number of distinct observation regions through [`Part::Combo`].
/// Plain English: How many types of observation work are tracked.
const N: usize = Part::Combo as usize + 1;
/// Technical: Display label matching each observation region index.
/// Plain English: Names used in observation timing reports.
const LABELS: [&str; N] = [
    "All local observation work",
    "Tech prediction",
    "Ledge tracking / setup",
    "Incoming attack defense",
    "Floor support",
    "Roll detection",
    "Grab detection",
    "Opponent-facing helper",
    "Trip / attacker targets",
    "Combo decisions",
];
/// Technical: Manually supplied external-latency label, stored as zero-or-one-based frames.
/// Plain English: A player-entered label for external input delay.
static TEST_LATENCY: AtomicUsize = AtomicUsize::new(0);
/// Technical: Enables optional diagnostic instrumentation.
/// Plain English: Turns performance measurements on or off.
static ENABLED: AtomicBool = AtomicBool::new(false);
/// Technical: Indicates that the current root observation was selected for sampling.
/// Plain English: Marks whether this observation should be measured.
static SAMPLE: AtomicBool = AtomicBool::new(false);
/// Technical: Monotonic invocation counter used for one-in-sixteen sampling.
/// Plain English: Picks occasional observations to measure.
static TICK: AtomicUsize = AtomicUsize::new(0);
/// Technical: Current nested observation region index for ray attribution.
/// Plain English: Tracks which observation section is currently running.
static CURRENT: AtomicUsize = AtomicUsize::new(0);
/// Technical: Generation counter invalidating measurements and cards across resets.
/// Plain English: Distinguishes current diagnostics from earlier sessions.
static EPOCH: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy)]
/// Technical: Fixed-size sampled observation timing and ray counters for one epoch.
/// Plain English: Stores performance totals for the current measurement period.
struct Counters {
    /// Technical: Diagnostic generation to which these counters belong.
    /// Plain English: Which measurement session these totals describe.
    epoch: usize,
    /// Technical: Sampled invocation count for each observation part.
    /// Plain English: How many measured calls each part had.
    calls: [u64; N],
    /// Technical: Saturating-compatible accumulated nanoseconds per part.
    /// Plain English: Total measured time for each part.
    nanos: [u64; N],
    /// Technical: Maximum sampled duration in nanoseconds per part.
    /// Plain English: Longest measured call for each part.
    max: [u64; N],
    /// Technical: Raycast/geometry observation count attributed to each part.
    /// Plain English: How many ray checks happened in each part.
    rays: [u64; N],
    /// Technical: Most recent sampled observation completion time.
    /// Plain English: When the latest measured observation ended.
    seen: Option<Instant>,
    /// Technical: Start time of the current report interval.
    /// Plain English: When this timing report window began.
    since: Option<Instant>,
}
impl Counters {
    /// Technical: Constructs an empty counter set at epoch zero.
    /// Plain English: Starts a new empty observation timing record.
    const fn new() -> Self {
        Self {
            epoch: 0,
            calls: [0; N],
            nanos: [0; N],
            max: [0; N],
            rays: [0; N],
            seen: None,
            since: None,
        }
    }
    /// Start a fresh window if the counters belong to an older epoch.
    /// Technical: Resets sampled totals when their generation no longer matches.
    /// Plain English: Clears old measurements when diagnostics have been reset.
    fn sync(&mut self, epoch: usize) {
        if self.epoch != epoch {
            *self = Self::new();
            self.epoch = epoch;
        }
    }
}
/// Technical: Protects the fixed-size observation timing snapshot.
/// Plain English: Holds the current timing totals.
static STATS: Mutex<Counters> = Mutex::new(Counters::new());
/// Technical: Shared epoch-tagged diagnostic card shown by the renderer.
/// Plain English: Stores the latest diagnostic report for display.
static CARD: Card = Card::new();

/// Text card tagged with the epoch that produced it. The logger worker publishes;
/// drawing only try-locks, so it never waits and never shows an older epoch.
/// Plain English: The background logger can update this report card without making drawing wait.
pub struct Card(#[doc = "Technical: Optional report text paired with the generation that produced it."] #[doc = "Plain English: The report text and its session number, when one is ready."] pub(crate) Mutex<Option<(usize, CString)>>);
impl Card {
    /// Technical: Initializes an empty card slot.
    /// Plain English: Creates a diagnostic card with no report yet.
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }
    /// Keep `text` only while `current` still names `epoch`, re-checked under the lock.
    /// Technical: Publishes owned report text only when its epoch is current both before and under lock.
    /// Plain English: Saves the report only if it still belongs to the current session.
    pub fn publish(&self, epoch: usize, current: &AtomicUsize, text: CString) {
        if epoch != current.load(Ordering::Acquire) {
            return;
        }
        if let Ok(mut card) = self.0.lock() {
            if epoch == current.load(Ordering::Acquire) {
                *card = Some((epoch, text));
            }
        }
    }
    /// Technical: Clears the card with a nonblocking lock attempt.
    /// Plain English: Removes the displayed report if the card is not busy.
    pub fn clear(&self) {
        if let Ok(mut card) = self.0.try_lock() {
            *card = None;
        }
    }
    /// Draw the card if it is free and belongs to the current epoch.
    /// Technical: Calls `draw` only for an available card from the current generation.
    /// Plain English: Displays the report only when it is ready and up to date.
    pub fn show(&self, current: &AtomicUsize, draw: impl FnOnce(&CStr)) -> bool {
        let Ok(card) = self.0.try_lock() else {
            return false;
        };
        match card.as_ref() {
            Some((epoch, text)) if *epoch == current.load(Ordering::Acquire) => {
                draw(text);
                true
            }
            _ => false,
        }
    }
}
/// Saturating conversion for elapsed-time counters.
/// Technical: Clamps a nanosecond `u128` value to the `u64` counter range.
/// Plain English: Prevents very large durations from overflowing the report counter.
pub fn clamp_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
/// Manual label for Training Modpack trials; never changes input delay.
/// Technical: Stores a capped label and clears measurements when it changes.
/// Plain English: Records the player's latency note without changing the game.
pub fn set_test_latency(index: u8) {
    let index = usize::from(index.min(31));
    if TEST_LATENCY.swap(index, Ordering::Relaxed) != index {
        clear();
    }
}
/// Technical: Reads the active diagnostic generation.
/// Plain English: Gets the current measurement session number.
pub fn context_epoch() -> usize {
    EPOCH.load(Ordering::Relaxed)
}
/// Technical: Decodes the stored zero-or-one-based latency label into an optional frame count.
/// Plain English: Gets the manually entered latency label, if set.
pub fn test_latency() -> Option<u8> {
    TEST_LATENCY
        .load(Ordering::Relaxed)
        .checked_sub(1)
        .map(|n| n as u8)
}
/// Technical: Formats an optional manual latency label without implying measurement.
/// Plain English: Turns the chosen latency note into report text.
pub fn latency_text(label: Option<u8>) -> String {
    label.map_or_else(
        || "UNLABELLED".into(),
        |n| format!("external +{n}f (manual label, not measured)"),
    )
}
/// Technical: Reads the diagnostics-enabled flag.
/// Plain English: Checks whether performance diagnostics are turned on.
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
/// Technical: Changes diagnostics enablement and invalidates prior data on transitions.
/// Plain English: Turns measurements on or off and starts a fresh report period.
pub fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) != on {
        clear();
    }
}
/// Technical: Advances the epoch and clears sample, counter, and card state nonblockingly.
/// Plain English: Resets performance measurements and removes stale report text.
pub fn clear() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
    SAMPLE.store(false, Ordering::Relaxed);
    health::clear_window();
    if let Ok(mut stats) = STATS.try_lock() {
        *stats = Counters::new();
    }
    CARD.clear();
}
/// Technical: RAII region timer that attributes sampled observation work to a part.
/// Plain English: Measures a selected section of fighter observation when sampling is active.
pub struct Scope {
    /// Technical: Diagnostic generation captured when this scope started.
    /// Plain English: Prevents old timing work appearing in a newer report.
    epoch: usize,
    /// Technical: Region index whose duration and rays this scope accumulates.
    /// Plain English: Which observation section this timer represents.
    part: usize,
    /// Technical: Previously active nested region restored when this scope drops.
    /// Plain English: Remembers the section that was running before this one.
    previous: usize,
    /// Technical: Start instant when selected for sampling, otherwise absent.
    /// Plain English: When the measured section began, if it was sampled.
    start: Option<Instant>,
    /// Technical: Marks the outer scope responsible for ending the sampling window.
    /// Plain English: Shows whether this timer started the overall measurement.
    root: bool,
}
impl Scope {
    /// Technical: Begins one sampled root interval using the fixed 1-in-16 cadence.
    /// Plain English: Starts measuring an occasional complete observation.
    pub fn root() -> Self {
        let sample = enabled() && TICK.fetch_add(1, Ordering::Relaxed) % 16 == 0;
        SAMPLE.store(sample, Ordering::Relaxed);
        let mut scope = Self::new(Part::Total);
        scope.root = true;
        scope
    }
    /// Technical: Begins a nested region and swaps its index into ray attribution state.
    /// Plain English: Starts timing one named part of the current observation.
    pub fn new(part: Part) -> Self {
        let start = (enabled() && SAMPLE.load(Ordering::Relaxed)).then(Instant::now);
        let previous = if start.is_some() {
            CURRENT.swap(part as usize, Ordering::Relaxed)
        } else {
            0
        };
        Self {
            epoch: EPOCH.load(Ordering::Relaxed),
            part: part as usize,
            previous,
            start,
            root: false,
        }
    }
}
impl Drop for Scope {
    /// Technical: Restores the parent region, accumulates sampled time, and closes root sampling.
    /// Plain English: Saves the timing totals and finishes the measurement when this scope ends.
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let ns = clamp_u64(start.elapsed().as_nanos());
            CURRENT.store(self.previous, Ordering::Relaxed);
            if enabled() && self.epoch == EPOCH.load(Ordering::Relaxed) {
                if let Ok(mut stats) = STATS.try_lock() {
                    stats.sync(self.epoch);
                    stats.calls[self.part] += 1;
                    stats.nanos[self.part] += ns;
                    stats.max[self.part] = stats.max[self.part].max(ns);
                    let now = Instant::now();
                    stats.seen = Some(now);
                    stats.since.get_or_insert(now);
                }
            }
        }
        if self.root {
            SAMPLE.store(false, Ordering::Relaxed);
        }
    }
}
/// Technical: Attributes one sampled ray observation to the currently nested region.
/// Plain English: Counts a ray check for whichever observation section is running.
pub fn ray() {
    if enabled() && SAMPLE.load(Ordering::Relaxed) {
        if let Ok(mut stats) = STATS.try_lock() {
            stats.sync(EPOCH.load(Ordering::Relaxed));
            stats.rays[CURRENT.load(Ordering::Relaxed).min(N - 1)] += 1;
        }
    }
}

#[cfg(test)]
/// Technical: End-to-end regressions for opt-in timing, report generation, and stale-card clearing.
/// Plain English: Tests that diagnostics appear when enabled and disappear when reset.
mod tests {
    use super::*;
    #[test]
    /// Technical: Exercises sampling enablement, report thresholds, epoch reset, and card freshness.
    /// Plain English: Checks measured reports work and old reports are not shown after a reset.
    fn opt_in_sampling_reports_and_clears_without_stale_cards() {
        set_enabled(false);
        {
            let _root = Scope::root();
            ray();
        }
        assert!(report().is_none());
        set_enabled(true);
        TICK.store(0, Ordering::Relaxed);
        {
            let _root = Scope::root();
            let _tech = Scope::new(Part::Tech);
            ray();
        }
        {
            let mut stats = STATS.lock().unwrap();
            assert_eq!(stats.calls[Part::Tech as usize], 1);
            assert_eq!(stats.rays[Part::Tech as usize], 1);
            stats.since = Some(Instant::now() - Duration::from_secs(6));
        }
        let text = report().unwrap();
        health::exercise_report();
        assert!(text.contains("Tech prediction"));
        let mut shown = false;
        with_card(|_| shown = true);
        assert!(shown);
        clear();
        with_card(|_| panic!("old session card leaked"));
        assert!(report().is_none());
        set_enabled(false);
    }
}
/// Technical: Draws the diagnostic card only while instrumentation is enabled.
/// Plain English: Shows the latest performance report when diagnostics are on.
pub fn with_card(draw: impl FnOnce(&CStr)) {
    if enabled() {
        CARD.show(&EPOCH, draw);
    }
}
/// Mean and peak helpers for nanosecond counters, reported in microseconds.
/// Technical: Converts accumulated nanoseconds and sample count into a mean in microseconds.
/// Plain English: Calculates the average measured time.
fn avg_us(nanos: u64, calls: u64) -> f64 {
    nanos as f64 / calls.max(1) as f64 / 1000.0
}
/// Technical: Converts a nanosecond duration to microseconds.
/// Plain English: Changes a measured time into report-friendly units.
fn micros(nanos: u64) -> f64 {
    nanos as f64 / 1000.0
}
/// Called only by the existing background writer. No extra thread or queue.
/// Plain English: Builds an observation timing report on the current logging worker.
fn observation_report() -> Option<String> {
    if !enabled() {
        return None;
    }
    let now = Instant::now();
    let epoch = EPOCH.load(Ordering::Relaxed);
    let snapshot = {
        let mut stats = STATS.try_lock().ok()?;
        if stats.epoch != epoch {
            return None;
        }
        if stats
            .seen
            .is_some_and(|seen| now.duration_since(seen) > Duration::from_secs(2))
        {
            *stats = Counters::new();
            CARD.clear();
            return None;
        }
        if stats
            .since
            .is_none_or(|since| now.duration_since(since) < Duration::from_secs(5))
        {
            return None;
        }
        std::mem::replace(&mut *stats, Counters::new())
    };
    let mut order: Vec<usize> = (1..N).collect();
    order.sort_unstable_by_key(|&i| std::cmp::Reverse(snapshot.nanos[i]));
    let mut compact = format!(
        "PERF: 1/16 observations\nLocal avg {:.1}us / peak {:.1}us\n",
        avg_us(snapshot.nanos[0], snapshot.calls[0]),
        micros(snapshot.max[0])
    );
    for &i in order.iter().filter(|&&i| snapshot.calls[i] != 0).take(3) {
        let _ = writeln!(
            compact,
            "{}: {:.1}us avg, {} rays",
            LABELS[i],
            avg_us(snapshot.nanos[i], snapshot.calls[i]),
            snapshot.rays[i]
        );
    }
    compact.push_str("Sampled times, not FPS. Full gauges: performance.log");
    let mut text = String::from("PERFORMANCE: sampled 1/16 local observations, ~5s window\nInclusive elapsed CPU-side time; nested rows overlap. Not GPU load/FPS.\n");
    let _ = writeln!(
        text,
        "TEST CONTEXT: perf_epoch={epoch}; {}",
        latency_text(test_latency())
    );
    let total = snapshot.nanos[0].max(1) as f64;
    for i in std::iter::once(0).chain(order) {
        let calls = snapshot.calls[i];
        let share = (snapshot.nanos[i] as f64 / total * 100.0).min(100.0);
        let gauge = "##########"
            .get(..(share / 10.0).ceil().min(10.0) as usize)
            .unwrap_or("");
        let _ = writeln!(
            text,
            "{:<28} [{:<10}] {:4.1}% | avg {:.1}us max {:.1}us | sampled calls {} rays {}{}",
            LABELS[i],
            gauge,
            share,
            avg_us(snapshot.nanos[i], calls),
            micros(snapshot.max[i]),
            calls,
            snapshot.rays[i],
            if calls == 0 {
                " (inactive / not sampled)"
            } else {
                ""
            }
        );
    }
    if !enabled() || epoch != EPOCH.load(Ordering::Relaxed) {
        return None;
    }
    if let Ok(card) = CString::new(compact) {
        CARD.publish(epoch, &EPOCH, card);
    }
    Some(text)
}

/// Existing logger worker is the only report formatter/writer.
/// Technical: Combines currently available observation and health report sections.
/// Plain English: Returns the diagnostic reports that are ready to display.
pub fn report() -> Option<String> {
    match (observation_report(), health::report()) {
        (Some(mut a), Some(b)) => {
            a.push_str(&b);
            Some(a)
        }
        (Some(a), None) | (None, Some(a)) => Some(a),
        (None, None) => None,
    }
}
