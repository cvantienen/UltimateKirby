//! Opt-in diagnostics only. Sample one in sixteen accepted local observations;
//! fixed counters on the game thread, report formatting on the existing writer.
//! Technical: Samples diagnostic timing/counter data with bounded atomics and
//! optional per-lane traces.
//! Plain English: This module measures plugin work only when diagnostic logging
//! is enabled.
#[path = "performance/health.rs"]
pub mod health;
#[cfg(test)]
use std::ffi::CStr;
use std::{
    ffi::CString,
    fmt::Write,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
/// Index for nested observation-timing regions and their counters.
pub enum Part {
    /// Outer aggregate covering all local-observation work.
    Total,
    /// Tech prediction sub-region.
    Tech,
    /// Ledge tracking and setup sub-region.
    Ledge,
    /// Incoming attack defense sub-region.
    Shield,
    /// Floor-support observation sub-region.
    Ground,
    /// Roll detection sub-region.
    Roll,
    /// Grab detection sub-region.
    Grabs,
    /// Opponent-facing helper sub-region.
    Posture,
    /// Trip and attacker-target sub-region.
    Trip,
    /// Combo decision sub-region.
    Combo,
}
/// Number of distinct observation regions through [`Part::Combo`].
const N: usize = Part::Combo as usize + 1;
/// Display label matching each observation region index.
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
static TEST_LATENCY: AtomicUsize = AtomicUsize::new(0);
/// Enables optional diagnostic instrumentation.
static ENABLED: AtomicBool = AtomicBool::new(false);
/// Indicates that the current root observation was selected for sampling.
static SAMPLE: AtomicBool = AtomicBool::new(false);
/// Monotonic invocation counter used for one-in-sixteen sampling.
static TICK: AtomicUsize = AtomicUsize::new(0);
/// Current nested observation region index for ray attribution.
static CURRENT: AtomicUsize = AtomicUsize::new(0);
/// Generation counter invalidating measurements and cards across resets.
static EPOCH: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy)]
/// Fixed-size sampled observation timing and ray counters for one epoch.
struct Counters {
    /// Diagnostic generation to which these counters belong.
    epoch: usize,
    /// Sampled invocation count for each observation part.
    calls: [u64; N],
    /// Saturating-compatible accumulated nanoseconds per part.
    nanos: [u64; N],
    /// Maximum sampled duration in nanoseconds per part.
    max: [u64; N],
    /// Raycast/geometry observation count attributed to each part.
    rays: [u64; N],
    /// Most recent sampled observation completion time.
    seen: Option<Instant>,
    /// Start time of the current report interval.
    since: Option<Instant>,
}
impl Counters {
    /// Constructs an empty counter set at epoch zero.
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
    fn sync(&mut self, epoch: usize) {
        if self.epoch != epoch {
            *self = Self::new();
            self.epoch = epoch;
        }
    }
}
/// Protects the fixed-size observation timing snapshot.
static STATS: Mutex<Counters> = Mutex::new(Counters::new());
/// Shared epoch-tagged diagnostic card shown by the renderer.
static CARD: Card = Card::new();

/// Text card tagged with the epoch that produced it. The logger worker publishes;
/// drawing only try-locks, so it never waits and never shows an older epoch.
pub struct Card(
    #[doc = "Technical: Optional report text paired with the generation that produced it."]
    #[doc = "Plain English: The report text and its session number, when one is ready."]
    pub(crate) Mutex<Option<(usize, CString)>>,
);
impl Card {
    /// Initializes an empty card slot.
    pub const fn new() -> Self {
        Self(Mutex::new(None))
    }
    /// Keep `text` only while `current` still names `epoch`, re-checked under the lock.
    /// Publishes owned report text only when its epoch is current both before and under lock.
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
    pub fn clear(&self) {
        if let Ok(mut card) = self.0.try_lock() {
            *card = None;
        }
    }
    /// Draw the card if it is free and belongs to the current epoch.
    /// Calls `draw` only for an available card from the current generation.
    #[cfg(test)]
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
pub fn clamp_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}
/// Manual label for Training Modpack trials; never changes input delay.
/// Stores a capped label and clears measurements when it changes.
pub fn set_test_latency(index: u8) {
    let index = usize::from(index.min(31));
    if TEST_LATENCY.swap(index, Ordering::Relaxed) != index {
        clear();
    }
}
/// Reads the active diagnostic generation.
pub fn context_epoch() -> usize {
    EPOCH.load(Ordering::Relaxed)
}
/// Decodes the stored zero-or-one-based latency label into an optional frame count.
pub fn test_latency() -> Option<u8> {
    TEST_LATENCY
        .load(Ordering::Relaxed)
        .checked_sub(1)
        .map(|n| n as u8)
}
/// Formats an optional manual latency label without implying measurement.
pub fn latency_text(label: Option<u8>) -> String {
    label.map_or_else(
        || "UNLABELLED".into(),
        |n| format!("external +{n}f (manual label, not measured)"),
    )
}
/// Reads the diagnostics-enabled flag.
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
/// Changes diagnostics enablement and invalidates prior data on transitions.
pub fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) != on {
        clear();
    }
}
/// Advances the epoch and clears sample, counter, and card state nonblockingly.
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
pub struct Scope {
    /// Diagnostic generation captured when this scope started.
    epoch: usize,
    /// Region index whose duration and rays this scope accumulates.
    part: usize,
    /// Previously active nested region restored when this scope drops.
    previous: usize,
    /// Start instant when selected for sampling, otherwise absent.
    start: Option<Instant>,
    /// Marks the outer scope responsible for ending the sampling window.
    root: bool,
}
impl Scope {
    /// Begins one sampled root interval using the fixed 1-in-16 cadence.
    pub fn root() -> Self {
        let sample = enabled() && TICK.fetch_add(1, Ordering::Relaxed) % 16 == 0;
        SAMPLE.store(sample, Ordering::Relaxed);
        let mut scope = Self::new(Part::Total);
        scope.root = true;
        scope
    }
    /// Begins a nested region and swaps its index into ray attribution state.
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
#[path = "../../tests/unit/diagnostics/performance/tests.rs"]
mod tests;
#[cfg(test)]
/// Draws the diagnostic card only while instrumentation is enabled.
pub fn with_card(draw: impl FnOnce(&CStr)) {
    if enabled() {
        CARD.show(&EPOCH, draw);
    }
}
/// Mean and peak helpers for nanosecond counters, reported in microseconds.
/// Converts accumulated nanoseconds and sample count into a mean in microseconds.
fn avg_us(nanos: u64, calls: u64) -> f64 {
    nanos as f64 / calls.max(1) as f64 / 1000.0
}
/// Converts a nanosecond duration to microseconds.
fn micros(nanos: u64) -> f64 {
    nanos as f64 / 1000.0
}
/// Called only by the existing background writer. No extra thread or queue.
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
