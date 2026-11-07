//! Temporary file-only aggregate probe. No console, networking or game queries.
use std::{
    fmt::{self, Write as _},
    fs::OpenOptions,
    io::{self, BufWriter, Write},
    sync::{
        atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering::Relaxed},
        OnceLock,
    },
    time::{Duration, Instant},
};
const PATH: &str = "sd:/ultimate/kirby_assist/silent_probe_once.log";
const MARKER: &str = "ONE_MATCH_PROBE_V1";
const LIMIT: usize = 32 * 1024;
const LIFETIME: Duration = Duration::from_secs(600);
const INTERVAL: Duration = Duration::from_secs(10);
const DISABLED: u8 = 0;
const ARMED: u8 = 1;
const CAPTURING: u8 = 2;
const FINISHED: u8 = 3;
static PHASE: AtomicU8 = AtomicU8::new(DISABLED);
static ORIGIN: OnceLock<Instant> = OnceLock::new();
static START_MS: AtomicU64 = AtomicU64::new(0);
static END_MS: AtomicU64 = AtomicU64::new(0);
#[derive(Clone, Copy)]
pub enum Event {
    Mapper,
    Evaluation,
    Duplicate,
    Ray,
    MenuBuild,
    FighterChange,
    InputExpiry,
}
const N: usize = 7;
static COUNTS: [AtomicU64; N] = [const { AtomicU64::new(0) }; N];
static SAMPLE_TICK: AtomicU64 = AtomicU64::new(0);
static SAMPLES: AtomicU64 = AtomicU64::new(0);
static NANOS: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
static PENDING: AtomicUsize = AtomicUsize::new(0);
static WORKERS: AtomicUsize = AtomicUsize::new(0);
fn next_phase(phase: u8, ready: bool) -> u8 {
    match (phase, ready) {
        (ARMED, true) => CAPTURING,
        (CAPTURING, false) => FINISHED,
        _ => phase,
    }
}
pub fn battle(ready: Option<bool>) {
    let Some(ready) = ready else {
        return;
    };
    let old = PHASE.load(Relaxed);
    let next = next_phase(old, ready);
    if old != next && PHASE.compare_exchange(old, next, Relaxed, Relaxed).is_ok() {
        let ms = ORIGIN.get().map_or(0, |t| t.elapsed().as_millis() as u64);
        if next == CAPTURING {
            START_MS.store(ms, Relaxed);
        } else {
            END_MS.store(ms, Relaxed);
        }
        kirby_assist_core::probe_counters::set_active(next == CAPTURING);
    }
}
pub fn event(event: Event) {
    if PHASE.load(Relaxed) == CAPTURING {
        COUNTS[event as usize].fetch_add(1, Relaxed);
    }
}
pub struct Observation {
    start: Option<Instant>,
}
pub fn observation() -> Observation {
    let sample = PHASE.load(Relaxed) == CAPTURING && SAMPLE_TICK.fetch_add(1, Relaxed) % 64 == 0;
    Observation {
        start: sample.then(Instant::now),
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        if let Some(start) = self.start.filter(|_| PHASE.load(Relaxed) == CAPTURING) {
            let ns = start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            NANOS.fetch_add(ns, Relaxed);
            PEAK.fetch_max(ns, Relaxed);
            SAMPLES.fetch_add(1, Relaxed);
        }
    }
}
pub fn pending(on: bool) {
    PENDING.store(usize::from(on), Relaxed);
}
pub struct SaveWorker;
pub fn save_worker() -> SaveWorker {
    WORKERS.fetch_add(1, Relaxed);
    SaveWorker
}
impl Drop for SaveWorker {
    fn drop(&mut self) {
        WORKERS.fetch_sub(1, Relaxed);
    }
}
/// Startup only. Fail closed and silently if thread/file creation is unavailable.
pub fn start() {
    let _ = std::thread::Builder::new()
        .name("one-match-probe".into())
        .stack_size(64 * 1024)
        .spawn(|| {
            let _ = run();
            PHASE.store(FINISHED, Relaxed);
            kirby_assist_core::probe_counters::set_active(false);
        });
}
struct Line {
    data: [u8; 768],
    len: usize,
}
impl Line {
    fn new() -> Self {
        Self {
            data: [0; 768],
            len: 0,
        }
    }
}
impl fmt::Write for Line {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self
            .len
            .checked_add(s.len())
            .filter(|n| *n <= self.data.len())
            .ok_or(fmt::Error)?;
        self.data[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}
fn append<W: Write>(w: &mut W, bytes: &mut usize, line: &[u8]) -> io::Result<bool> {
    if bytes.checked_add(line.len()).is_none_or(|n| n > LIMIT) {
        return Ok(false);
    }
    w.write_all(line)?;
    *bytes += line.len();
    Ok(true)
}
fn summary<W: Write>(
    writer: &mut W,
    bytes: &mut usize,
    elapsed: u64,
    reason: &str,
) -> io::Result<bool> {
    let c: [u64; N] = std::array::from_fn(|i| COUNTS[i].swap(0, Relaxed));
    let samples = SAMPLES.swap(0, Relaxed);
    let sum = NANOS.swap(0, Relaxed);
    let peak = PEAK.swap(0, Relaxed);
    let clock = kirby_assist_core::probe_counters::take_clock_aborts();
    let mut line = Line::new();
    writeln!(&mut line,"elapsed_ms={elapsed} end={reason} mapper={} eval={} duplicate={} rays={} menu_rebuild={} fighter_change={} expiry={} clock_cancel={clock} samples={samples} observation_avg_us={:.1} observation_peak_us={:.1} save_pending={} save_workers={}",
        c[0],c[1],c[2],c[3],c[4],c[5],c[6],sum as f64/samples.max(1) as f64/1000.0,peak as f64/1000.0,PENDING.load(Relaxed),WORKERS.load(Relaxed))
        .map_err(|_|io::Error::other("probe row limit"))?;
    append(writer, bytes, &line.data[..line.len])
}
fn run() -> io::Result<()> {
    // The file itself is the persistent one-shot latch. Never truncate/reuse it.
    // Switch std requires create=true even with create_new=true. The latter
    // still rejects an existing file before the creation branch is reached.
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .create_new(true)
        .open(PATH)?;
    let mut writer = BufWriter::with_capacity(4096, file);
    let mut bytes = 0;
    let mut header = Line::new();
    writeln!(&mut header,"{MARKER} build={}\nOne match; 10s delta summaries; 1/64 observation elapsed samples. Includes scheduling/preemption; not system CPU/FPS/temperature/network delay. 32KiB cap; stops at match end or 600s from probe startup. Existing file prevents another run.",env!("CARGO_PKG_VERSION"))
        .map_err(|_|io::Error::other("probe header limit"))?;
    if !append(&mut writer, &mut bytes, &header.data[..header.len])? {
        return Ok(());
    }
    writer.flush()?;
    let origin = *ORIGIN.get_or_init(Instant::now);
    PHASE.store(ARMED, Relaxed);
    loop {
        std::thread::sleep(INTERVAL.min(LIFETIME.saturating_sub(origin.elapsed())));
        let elapsed = origin.elapsed();
        let phase = PHASE.load(Relaxed);
        if phase == FINISHED || elapsed >= LIFETIME {
            PHASE.store(FINISHED, Relaxed);
            kirby_assist_core::probe_counters::set_active(false);
            let reason = if phase == FINISHED {
                "match_end"
            } else {
                "time_limit"
            };
            let ms = if phase == FINISHED {
                END_MS.load(Relaxed)
            } else {
                elapsed.as_millis() as u64
            };
            let _ = summary(
                &mut writer,
                &mut bytes,
                ms.saturating_sub(START_MS.load(Relaxed)),
                reason,
            )?;
            writer.flush()?;
            return Ok(());
        }
        if phase == CAPTURING {
            if !summary(
                &mut writer,
                &mut bytes,
                (elapsed.as_millis() as u64).saturating_sub(START_MS.load(Relaxed)),
                "running",
            )? {
                return Ok(());
            }
            writer.flush()?;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_match_lifecycle_never_rearms() {
        assert_eq!(next_phase(ARMED, false), ARMED);
        assert_eq!(next_phase(ARMED, true), CAPTURING);
        assert_eq!(next_phase(CAPTURING, false), FINISHED);
        assert_eq!(next_phase(FINISHED, true), FINISHED);
        assert_eq!(next_phase(DISABLED, true), DISABLED);
    }
    #[test]
    fn byte_cap_is_exact_and_does_not_partially_write_rows() {
        let mut output = Vec::new();
        let mut bytes = LIMIT - 3;
        assert!(append(&mut output, &mut bytes, b"abc").unwrap());
        assert!(!append(&mut output, &mut bytes, b"d").unwrap());
        assert_eq!(bytes, LIMIT);
        assert_eq!(output, b"abc");
    }
    #[test]
    fn summary_has_only_approved_aggregate_fields() {
        let mut output = Vec::new();
        let mut bytes = 0;
        assert!(summary(&mut output, &mut bytes, 10_000, "running").unwrap());
        let text = String::from_utf8(output).unwrap();
        for key in [
            "mapper=",
            "eval=",
            "duplicate=",
            "rays=",
            "menu_rebuild=",
            "fighter_change=",
            "expiry=",
            "clock_cancel=",
            "samples=",
            "observation_avg_us=",
            "observation_peak_us=",
            "save_pending=",
            "save_workers=",
        ] {
            assert!(text.contains(key));
        }
        assert!(!text.contains("position"));
        assert!(!text.contains("buttons"));
    }
}
