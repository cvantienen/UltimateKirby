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
    /// Category index for timing measurements in health reports.
    ///
    ///  Names the kind of work whose time is counted.
    pub enum Lane {
        /// Complete accepted mapping callback, including native game work.
        ///
        ///  Time for the whole controller-mapping callback.
        Mapping,
        /// Menu input and cached-view update lane.
        ///
        ///  Time spent handling menu input.
        MenuInput,
        /// Menu text construction lane.
        ///
        ///  Time spent rebuilding menu labels.
        MenuBuild,
        /// Mod draw callback lane, excluding external renderer work.
        ///
        ///  Time spent in the plugin's drawing callback.
        Draw,
        /// Explicit save worker lane.
        ///
        ///  Time spent saving settings.
        Save,
        /// Diagnostic log flush lane.
        ///
        ///  Time spent flushing log files.
        LogFlush,
        /// Diagnostic report formatting and write-batch lane.
        ///
        ///  Time spent preparing and writing diagnostic reports.
        LogWork,
    }
    /// Number of lanes represented by [`Lane`].
    ///
    ///  How many types of work the health recorder tracks.
    const LANES: usize = 7;
    /// Display label corresponding to each timing lane index.
    ///
    ///  Names shown beside timing measurements.
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
    ///
    ///  Counts how often each kind of work runs.
    static CALLS: [AtomicUsize; LANES] = [const { AtomicUsize::new(0) }; LANES];
    #[derive(Clone, Copy)]
    /// Counted diagnostic event categories used by health windows.
    ///
    ///  Names notable events the report keeps count of.
    pub enum Event {
        /// A menu snapshot publication was attempted.
        ///
        ///  The menu view was updated.
        MenuPublish,
        /// Menu strings were rebuilt.
        ///
        ///  The plugin regenerated menu text.
        MenuBuild,
        /// A menu publication had to retry due to lock contention.
        ///
        ///  A menu update could not proceed right away.
        MenuRetry,
        /// Settings were resolved for a sequence or recipe.
        ///
        ///  The plugin looked up settings for a selected move.
        Resolve,
        /// The selected fighter/profile changed.
        ///
        ///  The player switched to a different character.
        FighterChange,
        /// Match-scoped state was reset.
        ///
        ///  The plugin cleared state for a match transition.
        StateReset,
        /// A save worker began processing a save request.
        ///
        ///  A settings save started.
        SaveStarted,
        /// A save worker completed successfully.
        ///
        ///  A settings save finished.
        SaveCompleted,
        /// A save worker failed.
        ///
        ///  A settings save did not succeed.
        SaveFailed,
        /// A diagnostic message was dropped from the bounded queue.
        ///
        ///  A log message could not fit in the waiting queue.
        LogDropped,
        /// A local observation pass was entered.
        ///
        ///  The plugin started checking fighter state.
        Observation,
        /// A duplicate observation was skipped.
        ///
        ///  A repeated state check was avoided.
        ObservationReuse,
        /// A fighter roster scan was performed.
        ///
        ///  The plugin listed the current fighters.
        RosterScan,
        /// A complete local policy evaluation was performed.
        ///
        ///  All active helper checks were run.
        FullObservation,
        /// A collected observation failed publication freshness checks.
        ///
        ///  Old fighter data was discarded instead of being used.
        PublicationDiscard,
        /// One or more requests expired due to staleness.
        ///
        ///  A helper request was too old to use.
        InputExpired,
        /// A sequence was canceled after a native animation-clock gap.
        ///
        ///  A sequence stopped because the fighter's timing jumped.
        TimelineGap,
        /// A sequence was canceled because its motion clock was invalid or timed out.
        ///
        ///  A sequence stopped because its animation timing was unusable.
        TimelineClockAbort,
    }
    /// Number of event categories represented by [`Event`].
    ///
    ///  How many event types the report counts.
    const EVENTS: usize = 18;
    /// Human-readable event labels matching the enum discriminants.
    ///
    ///  Names shown beside the event totals.
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
    ///
    ///  Running totals for each diagnostic event.
    static COUNTS: [AtomicUsize; EVENTS] = [const { AtomicUsize::new(0) }; EVENTS];
    /// Current approximate logger-queue occupancy.
    ///
    ///  How many log messages are currently waiting.
    static DEPTH: AtomicUsize = AtomicUsize::new(0);
    /// Maximum queue occupancy observed in the active epoch.
    ///
    ///  The highest number of waiting log messages seen.
    static PEAK_DEPTH: AtomicUsize = AtomicUsize::new(0);
    /// Number of active save/report workers.
    ///
    ///  How many background workers are running now.
    static ACTIVE: AtomicUsize = AtomicUsize::new(0);
    /// Maximum active-worker count observed.
    ///
    ///  The most background workers that ran at once.
    static PEAK_ACTIVE: AtomicUsize = AtomicUsize::new(0);
    /// Current retained profile count supplied by menu accounting.
    ///
    ///  How many fighter profiles are retained.
    static PROFILES: AtomicUsize = AtomicUsize::new(0);
    /// Current retained editor-recipe count.
    ///
    ///  How many saved editor recipes are retained.
    static RECIPES: AtomicUsize = AtomicUsize::new(0);
    /// Current retained draft count.
    ///
    ///  How many unsaved drafts are retained.
    static DRAFTS: AtomicUsize = AtomicUsize::new(0);
    /// Current estimated owned payload bytes.
    ///
    ///  Approximate memory used by saved menu data.
    static RETAINED: AtomicUsize = AtomicUsize::new(0);
    /// Peak estimated retained payload bytes.
    ///
    ///  Largest saved-menu memory estimate seen.
    static PEAK_RETAINED: AtomicUsize = AtomicUsize::new(0);
    /// Current cached view payload bytes.
    ///
    ///  Approximate memory used by the current menu display.
    static VIEW: AtomicUsize = AtomicUsize::new(0);
    /// Peak cached view payload bytes.
    ///
    ///  Largest menu-display memory estimate seen.
    static PEAK_VIEW: AtomicUsize = AtomicUsize::new(0);
    /// Number of timing/window updates skipped due to lock contention.
    ///
    ///  Counts diagnostic samples missed because state was busy.
    static MISSED: AtomicUsize = AtomicUsize::new(0);
    #[derive(Clone, Copy, Default)]
    /// Accumulated timing statistics for one lane.
    ///
    ///  Stores sample count and elapsed time for one kind of work.
    struct Timing {
        /// Number of sampled executions.
        ///
        ///  How many timing samples were recorded.
        samples: u64,
        /// Saturating sum of sampled nanoseconds.
        ///
        ///  Total time across recorded samples.
        nanos: u64,
        /// Maximum sampled duration in nanoseconds.
        ///
        ///  Longest sample recorded.
        max: u64,
        /// Count of samples lasting at least one millisecond.
        ///
        ///  How many samples took one millisecond or longer.
        over_ms: u64,
    }
    #[derive(Clone, Copy, Default)]
    /// Snapshot of retained menu counts and payload estimate.
    ///
    ///  Current counts and estimated size of saved menu data.
    struct Memory {
        /// Retained profile count.
        ///
        ///  Number of profiles kept in memory.
        profiles: usize,
        /// Retained editor recipe count.
        ///
        ///  Number of recipes kept in memory.
        recipes: usize,
        /// Retained draft count.
        ///
        ///  Number of drafts kept in memory.
        drafts: usize,
        /// Estimated owned payload bytes.
        ///
        ///  Approximate memory used by those items.
        bytes: usize,
    }
    impl Memory {
        /// Reads current atomic memory-accounting values.
        ///
        ///  Takes a snapshot of menu memory counts.
        fn read() -> Self {
            Self {
                profiles: PROFILES.load(Relaxed),
                recipes: RECIPES.load(Relaxed),
                drafts: DRAFTS.load(Relaxed),
                bytes: RETAINED.load(Relaxed),
            }
        }
        /// Compares counts while intentionally ignoring payload size.
        ///
        ///  Checks whether the number of items stayed the same.
        fn same_items(self, old: Self) -> bool {
            (self.profiles, self.recipes, self.drafts) == (old.profiles, old.recipes, old.drafts)
        }
    }
    /// Rolling interval data and lifetime-counter baselines for a health report.
    ///
    ///  Holds the measurements and previous totals used to make each report.
    struct Window {
        /// Diagnostic generation to which this rolling window belongs.
        ///
        ///  Which enabled measurement period these numbers describe.
        epoch: usize,
        /// Start time of the current aggregate interval.
        ///
        ///  When this report window began.
        since: Option<Instant>,
        /// Per-lane sampled duration aggregates.
        ///
        ///  Timing totals for each kind of work.
        timing: [Timing; LANES],
        /// Most recent draw-callback timestamp.
        ///
        ///  When the last plugin draw callback ran.
        last_draw: Option<Instant>,
        /// Largest observed interval between draw callbacks in microseconds.
        ///
        ///  Longest pause between two plugin draw updates.
        draw_gap_max: u64,
        /// Counts of draw gaps exceeding 25, 50, and 100 milliseconds.
        ///
        ///  How often drawing updates were widely spaced.
        gaps: [u64; 3],
        /// Lifetime lane counters captured at the previous window boundary.
        ///
        ///  Earlier call totals used to calculate new activity.
        last_calls: [usize; LANES],
        /// Lifetime event counters captured at the previous window boundary.
        ///
        ///  Earlier event totals used to calculate new activity.
        last_events: [usize; EVENTS],
        /// Previous memory snapshot for growth comparison.
        ///
        ///  Earlier menu memory counts used to check for unexplained growth.
        memory: Option<Memory>,
        /// Consecutive windows with stable counts and increasing payload.
        ///
        ///  How many report periods showed unexplained memory growth.
        growth_streak: u8,
        /// Consecutive windows with active save workers but no completion/failure.
        ///
        ///  How long a save worker has appeared stuck.
        save_streak: u8,
    }
    impl Window {
        /// Initializes zeroed timing, counter baselines, and streak state.
        ///
        ///  Starts an empty performance report window.
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
        ///
        ///  Begins a fresh report period without erasing lifetime counts.
        fn restart(&mut self, epoch: usize) {
            *self = Self::new();
            self.epoch = epoch;
            self.last_calls = std::array::from_fn(|i| CALLS[i].load(Relaxed));
            self.last_events = std::array::from_fn(|i| COUNTS[i].load(Relaxed));
        }
    }
    /// Protects the current bounded health-report aggregation window.
    ///
    ///  Holds the timing totals being collected for the next report.
    static WINDOW: Mutex<Window> = Mutex::new(Window::new());
    /// Restarts a window when its diagnostic generation changes.
    ///
    ///  Clears old period totals when performance settings are reset.
    fn sync_epoch(w: &mut Window, epoch: usize) {
        if w.epoch != epoch {
            w.restart(epoch);
        }
    }

    /// Increments one lifetime event counter.
    ///
    ///  Records that a diagnostic event happened once.
    pub fn event(e: Event) {
        add_event(e, 1);
    }
    /// Adds `n` occurrences to an event's lifetime counter.
    ///
    ///  Adds several occurrences to the event total at once.
    pub fn add_event(e: Event, n: usize) {
        COUNTS[e as usize].fetch_add(n, Relaxed);
    }
    /// Reservations precede try_send so a fast receiver cannot underflow the gauge.
    /// Occupancy is approximate during handoff and capped at the actual channel limit.
    /// Reserves one approximate queue slot and updates the capped peak.
    ///
    ///  Counts a message before it is handed to the logger queue.
    pub fn queue_enter() {
        let n = DEPTH.fetch_add(1, Relaxed) + 1;
        PEAK_DEPTH.fetch_max(n.min(128), Relaxed);
    }
    /// Releases one previously reserved queue slot.
    ///
    ///  Removes a message from the waiting-count estimate.
    pub fn queue_leave() {
        DEPTH.fetch_sub(1, Relaxed);
    }
    /// Publishes current retained-menu counts and updates the payload peak.
    ///
    ///  Updates memory estimates for profiles, recipes, and drafts.
    pub fn retained(profiles: usize, recipes: usize, drafts: usize, bytes: usize) {
        PROFILES.store(profiles, Relaxed);
        RECIPES.store(recipes, Relaxed);
        DRAFTS.store(drafts, Relaxed);
        RETAINED.store(bytes, Relaxed);
        PEAK_RETAINED.fetch_max(bytes, Relaxed);
    }
    /// Publishes current cached-view bytes and updates the view peak.
    ///
    ///  Updates the estimated size of the displayed menu text.
    pub fn view_bytes(bytes: usize) {
        VIEW.store(bytes, Relaxed);
        PEAK_VIEW.fetch_max(bytes, Relaxed);
    }
    /// RAII marker for a currently active background worker.
    ///
    ///  Counts a save worker while it is running.
    pub struct Worker;
    impl Worker {
        /// Increments active-worker counters and records a save-start event.
        ///
        ///  Marks a save worker as started.
        pub fn start() -> Self {
            event(Event::SaveStarted);
            let n = ACTIVE.fetch_add(1, Relaxed) + 1;
            PEAK_ACTIVE.fetch_max(n, Relaxed);
            Self
        }
    }
    impl Drop for Worker {
        /// Decrements the active-worker count when the marker is dropped.
        ///
        ///  Marks the save worker as finished.
        fn drop(&mut self) {
            ACTIVE.fetch_sub(1, Relaxed);
        }
    }

    /// RAII timer for a sampled health-report lane.
    ///
    ///  Measures one piece of plugin work when sampling is enabled.
    pub struct Trace {
        /// Timing-lane index recorded by this guard.
        ///
        ///  Which kind of work this timer measures.
        lane: usize,
        /// Diagnostic generation captured at timer creation.
        ///
        ///  Prevents an old timer from entering a newer report period.
        epoch: usize,
        /// Start instant for sampled timing, absent when not selected.
        ///
        ///  When this measured operation began, if it was sampled.
        start: Option<Instant>,
    }
    impl Trace {
        /// Starts a sampled lane timer according to the active sampling cadence.
        ///
        ///  Begins timing this operation when it is selected for measurement.
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
        ///
        ///  Adds the measured time to the current report if it is still current.
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
    ///
    ///  Records how much time passed between plugin draw updates.
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
    /// Restarts the aggregate window while preserving lifetime atomics.
    ///
    ///  Clears the current report period but keeps overall event totals.
    pub fn clear_window() {
        if let Ok(mut w) = WINDOW.try_lock() {
            w.restart(super::EPOCH.load(Relaxed));
        }
    }
    /// Produces a report after five seconds of data, using try-lock snapshots.
    ///
    ///  Builds a periodic health report when enough data is available.
    pub fn report() -> Option<String> {
        if !super::enabled() {
            return None;
        }
        let now = Instant::now();
        let epoch = super::EPOCH.load(Relaxed);
        let memory = Memory::read();
        let workers = ACTIVE.load(Relaxed);
        let (
            timing,
            calls,
            delta,
            events,
            event_delta,
            gap,
            gaps,
            elapsed,
            growth_streak,
            save_streak,
        ) = {
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
            let delta =
                std::array::from_fn::<_, LANES, _>(|i| calls[i].wrapping_sub(w.last_calls[i]));
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
    ///
    ///  These warnings point to things to investigate, not proven causes.
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
        if t[Lane::LogFlush as usize].max >= 5_000_000 || t[Lane::LogWork as usize].max >= 5_000_000
        {
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
    /// Exercises health-window collection and reset behavior for unit tests.
    ///
    ///  Sets up sample data and checks that the report is produced and cleared.
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
    ///
    ///  Checks warnings are useful without claiming more than the data proves.
    mod triage_tests {
        use super::*;
        #[test]
        /// Verifies distinct event and timing thresholds produce their labeled flags.
        ///
        ///  Checks several separate problems are reported with the right hints.
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
        /// Verifies a quiet interval does not imply health and thresholds are contextual.
        ///
        ///  Checks the report avoids declaring the whole system healthy.
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
/// Index for nested observation-timing regions and their counters.
///
///  Names the parts of fighter observation that can be timed.
pub enum Part {
    /// Outer aggregate covering all local-observation work.
    ///
    ///  Total time for the complete local observation.
    Total,
    /// Tech prediction sub-region.
    ///
    ///  Time used to predict tech timing.
    Tech,
    /// Ledge tracking and setup sub-region.
    ///
    ///  Time spent checking ledge state.
    Ledge,
    /// Incoming attack defense sub-region.
    ///
    ///  Time spent checking how to defend from attacks.
    Shield,
    /// Floor-support observation sub-region.
    ///
    ///  Time spent checking the fighter's floor support.
    Ground,
    /// Roll detection sub-region.
    ///
    ///  Time spent checking for rolls.
    Roll,
    /// Grab detection sub-region.
    ///
    ///  Time spent checking for grabs.
    Grabs,
    /// Opponent-facing helper sub-region.
    ///
    ///  Time spent checking the opponent-facing helper.
    Posture,
    /// Trip and attacker-target sub-region.
    ///
    ///  Time spent checking trip and attacker targets.
    Trip,
    /// Combo decision sub-region.
    ///
    ///  Time spent checking combo decisions.
    Combo,
}
/// Number of distinct observation regions through [`Part::Combo`].
///
///  How many types of observation work are tracked.
const N: usize = Part::Combo as usize + 1;
/// Display label matching each observation region index.
///
///  Names used in observation timing reports.
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
/// Manually supplied external-latency label, stored as zero-or-one-based frames.
///
///  A player-entered label for external input delay.
static TEST_LATENCY: AtomicUsize = AtomicUsize::new(0);
/// Enables optional diagnostic instrumentation.
///
///  Turns performance measurements on or off.
static ENABLED: AtomicBool = AtomicBool::new(false);
/// Indicates that the current root observation was selected for sampling.
///
///  Marks whether this observation should be measured.
static SAMPLE: AtomicBool = AtomicBool::new(false);
/// Monotonic invocation counter used for one-in-sixteen sampling.
///
///  Picks occasional observations to measure.
static TICK: AtomicUsize = AtomicUsize::new(0);
/// Current nested observation region index for ray attribution.
///
///  Tracks which observation section is currently running.
static CURRENT: AtomicUsize = AtomicUsize::new(0);
/// Generation counter invalidating measurements and cards across resets.
///
///  Distinguishes current diagnostics from earlier sessions.
static EPOCH: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy)]
/// Fixed-size sampled observation timing and ray counters for one epoch.
///
///  Stores performance totals for the current measurement period.
struct Counters {
    /// Diagnostic generation to which these counters belong.
    ///
    ///  Which measurement session these totals describe.
    epoch: usize,
    /// Sampled invocation count for each observation part.
    ///
    ///  How many measured calls each part had.
    calls: [u64; N],
    /// Saturating-compatible accumulated nanoseconds per part.
    ///
    ///  Total measured time for each part.
    nanos: [u64; N],
    /// Maximum sampled duration in nanoseconds per part.
    ///
    ///  Longest measured call for each part.
    max: [u64; N],
    /// Raycast/geometry observation count attributed to each part.
    ///
    ///  How many ray checks happened in each part.
    rays: [u64; N],
    /// Most recent sampled observation completion time.
    ///
    ///  When the latest measured observation ended.
    seen: Option<Instant>,
    /// Start time of the current report interval.
    ///
    ///  When this timing report window began.
    since: Option<Instant>,
}
impl Counters {
    /// Constructs an empty counter set at epoch zero.
    ///
    ///  Starts a new empty observation timing record.
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
    /// Resets sampled totals when their generation no longer matches.
    ///
    ///  Clears old measurements when diagnostics have been reset.
    fn sync(&mut self, epoch: usize) {
        if self.epoch != epoch {
            *self = Self::new();
            self.epoch = epoch;
        }
    }
}
/// Protects the fixed-size observation timing snapshot.
///
///  Holds the current timing totals.
static STATS: Mutex<Counters> = Mutex::new(Counters::new());
/// Shared epoch-tagged diagnostic card shown by the renderer.
///
///  Stores the latest diagnostic report for display.
static CARD: Card = Card::new();

/// Text card tagged with the epoch that produced it. The logger worker publishes;
/// drawing only try-locks, so it never waits and never shows an older epoch.
///
///  The background logger can update this report card without making drawing wait.
pub struct Card(
    #[doc = "Technical: Optional report text paired with the generation that produced it."]
    #[doc = "Plain English: The report text and its session number, when one is ready."]
    pub(crate) Mutex<Option<(usize, CString)>>,
);
impl Card {
    /// Initializes an empty card slot.
    ///
    ///  Creates a diagnostic card with no report yet.
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }
    /// Keep `text` only while `current` still names `epoch`, re-checked under the lock.
    /// Publishes owned report text only when its epoch is current both before and under lock.
    ///
    ///  Saves the report only if it still belongs to the current session.
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
    /// Clears the card with a nonblocking lock attempt.
    ///
    ///  Removes the displayed report if the card is not busy.
    pub fn clear(&self) {
        if let Ok(mut card) = self.0.try_lock() {
            *card = None;
        }
    }
    /// Draw the card if it is free and belongs to the current epoch.
    /// Calls `draw` only for an available card from the current generation.
    ///
    ///  Displays the report only when it is ready and up to date.
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
/// Clamps a nanosecond `u128` value to the `u64` counter range.
///
///  Prevents very large durations from overflowing the report counter.
pub fn clamp_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
/// Manual label for Training Modpack trials; never changes input delay.
/// Stores a capped label and clears measurements when it changes.
///
///  Records the player's latency note without changing the game.
pub fn set_test_latency(index: u8) {
    let index = usize::from(index.min(31));
    if TEST_LATENCY.swap(index, Ordering::Relaxed) != index {
        clear();
    }
}
/// Reads the active diagnostic generation.
///
///  Gets the current measurement session number.
pub fn context_epoch() -> usize {
    EPOCH.load(Ordering::Relaxed)
}
/// Decodes the stored zero-or-one-based latency label into an optional frame count.
///
///  Gets the manually entered latency label, if set.
pub fn test_latency() -> Option<u8> {
    TEST_LATENCY
        .load(Ordering::Relaxed)
        .checked_sub(1)
        .map(|n| n as u8)
}
/// Formats an optional manual latency label without implying measurement.
///
///  Turns the chosen latency note into report text.
pub fn latency_text(label: Option<u8>) -> String {
    label.map_or_else(
        || "UNLABELLED".into(),
        |n| format!("external +{n}f (manual label, not measured)"),
    )
}
/// Reads the diagnostics-enabled flag.
///
///  Checks whether performance diagnostics are turned on.
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
/// Changes diagnostics enablement and invalidates prior data on transitions.
///
///  Turns measurements on or off and starts a fresh report period.
pub fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) != on {
        clear();
    }
}
/// Advances the epoch and clears sample, counter, and card state nonblockingly.
///
///  Resets performance measurements and removes stale report text.
pub fn clear() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
    SAMPLE.store(false, Ordering::Relaxed);
    health::clear_window();
    if let Ok(mut stats) = STATS.try_lock() {
        *stats = Counters::new();
    }
    CARD.clear();
}
/// RAII region timer that attributes sampled observation work to a part.
///
///  Measures a selected section of fighter observation when sampling is active.
pub struct Scope {
    /// Diagnostic generation captured when this scope started.
    ///
    ///  Prevents old timing work appearing in a newer report.
    epoch: usize,
    /// Region index whose duration and rays this scope accumulates.
    ///
    ///  Which observation section this timer represents.
    part: usize,
    /// Previously active nested region restored when this scope drops.
    ///
    ///  Remembers the section that was running before this one.
    previous: usize,
    /// Start instant when selected for sampling, otherwise absent.
    ///
    ///  When the measured section began, if it was sampled.
    start: Option<Instant>,
    /// Marks the outer scope responsible for ending the sampling window.
    ///
    ///  Shows whether this timer started the overall measurement.
    root: bool,
}
impl Scope {
    /// Begins one sampled root interval using the fixed 1-in-16 cadence.
    ///
    ///  Starts measuring an occasional complete observation.
    pub fn root() -> Self {
        let sample = enabled() && TICK.fetch_add(1, Ordering::Relaxed) % 16 == 0;
        SAMPLE.store(sample, Ordering::Relaxed);
        let mut scope = Self::new(Part::Total);
        scope.root = true;
        scope
    }
    /// Begins a nested region and swaps its index into ray attribution state.
    ///
    ///  Starts timing one named part of the current observation.
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
    /// Restores the parent region, accumulates sampled time, and closes root sampling.
    ///
    ///  Saves the timing totals and finishes the measurement when this scope ends.
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
/// Attributes one sampled ray observation to the currently nested region.
///
///  Counts a ray check for whichever observation section is running.
pub fn ray() {
    if enabled() && SAMPLE.load(Ordering::Relaxed) {
        if let Ok(mut stats) = STATS.try_lock() {
            stats.sync(EPOCH.load(Ordering::Relaxed));
            stats.rays[CURRENT.load(Ordering::Relaxed).min(N - 1)] += 1;
        }
    }
}

#[cfg(test)]
/// End-to-end regressions for opt-in timing, report generation, and stale-card clearing.
///
///  Tests that diagnostics appear when enabled and disappear when reset.
mod tests {
    use super::*;
    #[test]
    /// Exercises sampling enablement, report thresholds, epoch reset, and card freshness.
    ///
    ///  Checks measured reports work and old reports are not shown after a reset.
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
/// Draws the diagnostic card only while instrumentation is enabled.
///
///  Shows the latest performance report when diagnostics are on.
pub fn with_card(draw: impl FnOnce(&CStr)) {
    if enabled() {
        CARD.show(&EPOCH, draw);
    }
}
/// Mean and peak helpers for nanosecond counters, reported in microseconds.
/// Converts accumulated nanoseconds and sample count into a mean in microseconds.
///
///  Calculates the average measured time.
fn avg_us(nanos: u64, calls: u64) -> f64 {
    nanos as f64 / calls.max(1) as f64 / 1000.0
}
/// Converts a nanosecond duration to microseconds.
///
///  Changes a measured time into report-friendly units.
fn micros(nanos: u64) -> f64 {
    nanos as f64 / 1000.0
}
/// Called only by the existing background writer. No extra thread or queue.
///
///  Builds an observation timing report on the current logging worker.
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
/// Combines currently available observation and health report sections.
///
///  Returns the diagnostic reports that are ready to display.
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
