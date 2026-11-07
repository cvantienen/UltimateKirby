//! Opt-in diagnostics only. Sample one in sixteen accepted local observations;
//! fixed counters on the game thread, report formatting on the existing writer.
#[path = "performance/health.rs"]
pub mod health;
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
pub enum Part {
    Total,
    Tech,
    Ledge,
    Shield,
    Ground,
    Roll,
    Grabs,
    Posture,
    Jab,
    Trip,
    Combo,
}
const N: usize = Part::Combo as usize + 1;
const LABELS: [&str; N] = [
    "All local observation work",
    "Tech prediction",
    "Ledge tracking / setup",
    "Incoming attack defense",
    "Floor support",
    "Roll detection",
    "Grab detection",
    "Opponent-facing helper",
    "Jab-lock targets",
    "Trip / attacker targets",
    "Combo decisions",
];
static TEST_LATENCY: AtomicUsize = AtomicUsize::new(0);
static ENABLED: AtomicBool = AtomicBool::new(false);
static SAMPLE: AtomicBool = AtomicBool::new(false);
static TICK: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static EPOCH: AtomicUsize = AtomicUsize::new(0);
#[derive(Clone, Copy)]
struct Counters {
    epoch: usize,
    calls: [u64; N],
    nanos: [u64; N],
    max: [u64; N],
    rays: [u64; N],
    seen: Option<Instant>,
    since: Option<Instant>,
}
impl Counters {
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
}
static STATS: Mutex<Counters> = Mutex::new(Counters::new());
static CARD: Mutex<Option<(usize, CString)>> = Mutex::new(None);
/// Manual label for Training Modpack trials; never changes input delay.
pub fn set_test_latency(index: u8) {
    if TEST_LATENCY.swap(usize::from(index.min(31)), Ordering::Relaxed)
        != usize::from(index.min(31))
    {
        clear();
    }
}
pub fn context_epoch() -> usize {
    EPOCH.load(Ordering::Relaxed)
}
pub fn test_latency() -> Option<u8> {
    TEST_LATENCY
        .load(Ordering::Relaxed)
        .checked_sub(1)
        .map(|n| n as u8)
}
pub fn latency_text(label: Option<u8>) -> String {
    label.map_or_else(
        || "UNLABELLED".into(),
        |n| format!("external +{n}f (manual label, not measured)"),
    )
}
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}
pub fn set_enabled(on: bool) {
    if ENABLED.swap(on, Ordering::Relaxed) != on {
        clear();
    }
}
pub fn clear() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
    SAMPLE.store(false, Ordering::Relaxed);
    health::clear_window();
    if let Ok(mut stats) = STATS.try_lock() {
        *stats = Counters::new();
    }
    if let Ok(mut card) = CARD.try_lock() {
        *card = None;
    }
}
pub struct Scope {
    epoch: usize,
    part: usize,
    previous: usize,
    start: Option<Instant>,
    root: bool,
}
impl Scope {
    pub fn root() -> Self {
        let sample = enabled() && TICK.fetch_add(1, Ordering::Relaxed) % 16 == 0;
        SAMPLE.store(sample, Ordering::Relaxed);
        let mut scope = Self::new(Part::Total);
        scope.root = true;
        scope
    }
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
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let ns = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            CURRENT.store(self.previous, Ordering::Relaxed);
            if enabled() && self.epoch == EPOCH.load(Ordering::Relaxed) {
                if let Ok(mut stats) = STATS.try_lock() {
                    if stats.epoch != self.epoch {
                        *stats = Counters::new();
                        stats.epoch = self.epoch;
                    }
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
pub fn ray() {
    if enabled() && SAMPLE.load(Ordering::Relaxed) {
        if let Ok(mut stats) = STATS.try_lock() {
            let epoch = EPOCH.load(Ordering::Relaxed);
            if stats.epoch != epoch {
                *stats = Counters::new();
                stats.epoch = epoch;
            }
            stats.rays[CURRENT.load(Ordering::Relaxed).min(N - 1)] += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
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
pub fn with_card(draw: impl FnOnce(&CStr)) {
    if !enabled() {
        return;
    }
    if let Ok(card) = CARD.try_lock() {
        if let Some((epoch, card)) = card.as_ref() {
            if *epoch == EPOCH.load(Ordering::Relaxed) {
                draw(card);
            }
        }
    }
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
            if let Ok(mut card) = CARD.try_lock() {
                *card = None;
            }
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
        snapshot.nanos[0] as f64 / snapshot.calls[0].max(1) as f64 / 1000.0,
        snapshot.max[0] as f64 / 1000.0
    );
    for &i in order.iter().filter(|&&i| snapshot.calls[i] != 0).take(3) {
        let _ = writeln!(
            compact,
            "{}: {:.1}us avg, {} rays",
            LABELS[i],
            snapshot.nanos[i] as f64 / snapshot.calls[i] as f64 / 1000.0,
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
            snapshot.nanos[i] as f64 / calls.max(1) as f64 / 1000.0,
            snapshot.max[i] as f64 / 1000.0,
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
    if let Ok(mut card) = CARD.try_lock() {
        *card = CString::new(compact).ok().map(|card| (epoch, card));
    }
    Some(text)
}

/// Existing logger worker is the only report formatter/writer.
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
