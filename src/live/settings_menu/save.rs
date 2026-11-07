//! Explicit menu saves only. No SD work on the input/render/observation thread.
use kirby_assist_core::{
    fighter_profiles::presets::PresetSlot,
    sequence_config::{persist, Sequence, MAX_FILE_BYTES},
};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

const PATH: &str = "sd:/ultimate/kirby_assist/sequences.txt";
const TEMP: &str = "sd:/ultimate/kirby_assist/sequences.txt.menu.tmp";
const BACKUP: &str = "sd:/ultimate/kirby_assist/sequences.txt.menu.bak";
const WORKER_STACK_BYTES: usize = 2 * 1024 * 1024;
const RAW_OS_ERROR_ALREADY_EXISTS: i32 = 0x402;
pub(super) struct Job {
    pub kind: i32,
    pub slot: PresetSlot,
    pub index: usize,
    pub revision: u64,
    pub choice: String,
    pub name: Option<String>,
    pub delete: bool,
    pub make_default: bool,
    pub allow_new: bool,
    pub sequence: Sequence,
    pub tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
pub(super) struct ResultMessage {
    pub choice: Option<String>,
    pub kind: i32,
    pub slot: PresetSlot,
    pub index: usize,
    pub revision: u64,
    pub message: String,
    pub success: bool,
    pub deleted: bool,
    pub snapshot: Option<(Sequence, Option<String>)>,
    pub tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
// No pending snapshots/queue: explicit Training saves start one temporary
// worker immediately. Retain its handle until exit; never detach a live
// Switch worker. A second save waits until completion is consumed.
static BUSY: AtomicBool = AtomicBool::new(false);
struct Worker {
    handle: std::thread::JoinHandle<()>,
    identity: (i32, PresetSlot, usize, u64),
}
static WORKER: Mutex<Option<Worker>> = Mutex::new(None);
// Only explicit saves and cached-text replacement use this gate; never inputs,
// detection or drawing. Check BUSY under the gate to close the start/rebuild race.
static UI_SAVE_GATE: Mutex<()> = Mutex::new(());
pub(super) fn try_ui_edit() -> Option<std::sync::MutexGuard<'static, ()>> {
    if busy() {
        return None;
    }
    let guard = UI_SAVE_GATE.try_lock().ok()?;
    if busy() {
        None
    } else {
        Some(guard)
    }
}
pub(super) fn busy() -> bool {
    BUSY.load(Ordering::Acquire)
}
static RESULT_READY: AtomicBool = AtomicBool::new(false);
static RESULT: Mutex<Option<ResultMessage>> = Mutex::new(None);
#[cfg(test)]
pub(super) static WORKER_TEST: Mutex<()> = Mutex::new(());
pub(super) fn has_result() -> bool {
    RESULT_READY.load(Ordering::Acquire)
        || busy()
            && WORKER.try_lock().is_ok_and(|worker| {
                worker
                    .as_ref()
                    .is_some_and(|worker| worker.handle.is_finished())
            })
}
pub(super) fn poll() -> Option<ResultMessage> {
    if !has_result() {
        return None;
    }
    // A posted result precedes thread-local/closure teardown. Keep the worker
    // and cached UI alive until the thread has actually exited.
    let handle = {
        let mut worker = WORKER.try_lock().ok()?;
        if worker
            .as_ref()
            .is_some_and(|worker| !worker.handle.is_finished())
        {
            return None;
        }
        worker.take()
    };
    let exited = handle.map(|worker| {
        let failed = worker.handle.join().is_err(); // Already finished: no I/O wait.
        (worker.identity, failed)
    });
    let mut result = RESULT
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take();
    if let Some((identity, failed)) = exited {
        if failed || result.is_none() {
            result = Some(ResultMessage {
                choice: None,
                kind: identity.0,
                slot: identity.1,
                index: identity.2,
                revision: identity.3,
                message: "Save worker stopped unexpectedly; save not confirmed. Retry Save".into(),
                success: false,
                deleted: false,
                snapshot: None,
                tuning: None,
            });
        }
    }
    if result.is_some() {
        RESULT_READY.store(false, Ordering::Release);
        BUSY.store(false, Ordering::Release);
    }
    result
}
fn claim_worker(training: bool) -> Result<(), &'static str> {
    if !training {
        return Err("Saving is available only in Training Mode");
    }
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map(|_| ())
        .map_err(|_| "Save in progress; retry when it finishes")
}
pub(super) fn request(job: Job, training: bool) -> String {
    #[cfg(test)]
    if let Some(operation) = *TEST_OPERATION.lock().unwrap_or_else(|e| e.into_inner()) {
        return request_with(job, training, operation);
    }
    request_with(job, training, write)
}
// Host adapter regressions exercise real Y input/worker/completion routing
// without accessing an SD path. Absent from production builds.
#[cfg(test)]
pub(super) static TEST_OPERATION: Mutex<Option<fn(&Job) -> Result<bool, String>>> =
    Mutex::new(None);
fn request_with(job: Job, training: bool, operation: fn(&Job) -> Result<bool, String>) -> String {
    if job.tuning.is_none() && !job.delete {
        if usize::from(job.sequence.len) > kirby_assist_core::sequence_config::MAX_ACTIONS {
            return "Save failed: recipe exceeds the action limit".into();
        }
        if job.sequence.len == 0 && job.sequence.mode.is_none() && job.sequence.builtin.is_none() {
            return "Save failed: add an action or delay before saving".into();
        }
    }
    if let Err(message) = claim_worker(training) {
        return message.into();
    }
    // Keep native thread ownership through completion. Detaching immediately
    // after creation overlapped the cached-menu destruction in crash reports.
    let mut worker = WORKER.lock().unwrap_or_else(|e| e.into_inner());
    let identity = (job.kind, job.slot, job.index, job.revision);
    *worker = spawn_worker(job, operation).map(|handle| Worker { handle, identity });
    "Saving now; keep Smash open until complete".into()
}
fn publish(result: ResultMessage) {
    let mut slot = RESULT.lock().unwrap_or_else(|error| error.into_inner());
    *slot = Some(result);
    RESULT_READY.store(true, Ordering::Release);
}
fn spawn_worker(
    job: Job,
    operation: fn(&Job) -> Result<bool, String>,
) -> Option<std::thread::JoinHandle<()>> {
    let identity = (
        job.kind,
        job.slot,
        job.index,
        job.revision,
        job.delete,
        job.tuning,
    );
    match std::thread::Builder::new()
        .name("recipe-save".into())
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            // Accept the first explicit save even if a cached view is being
            // replaced. Only this background worker waits for that short edit;
            // the input/render callbacks never wait or discard the snapshot.
            let ui_gate = UI_SAVE_GATE.lock().unwrap_or_else(|e| e.into_inner());
            drop(ui_gate);
            #[cfg(feature = "silent-probe")]
            let _probe_save_worker = crate::one_match_probe::save_worker();
            #[cfg(feature = "diagnostic-logging")]
            let _worker = crate::performance::health::Worker::start();
            #[cfg(feature = "diagnostic-logging")]
            let _save_time =
                crate::performance::health::Trace::new(crate::performance::health::Lane::Save);
            let outcome = operation(&job);
            let success = outcome.is_ok();
            #[cfg(feature = "diagnostic-logging")]
            {
                crate::performance::health::event(crate::performance::health::Event::SaveCompleted);
                if !success {
                    crate::performance::health::event(
                        crate::performance::health::Event::SaveFailed,
                    );
                }
            }
            let message = match outcome {
                Ok(false) => "Already saved; file unchanged".into(),
                Ok(true) if job.delete => "Deleted from sequences.txt (backup created)".into(),
                Ok(true) if job.make_default => "Saved as startup default (backup created)".into(),
                Ok(true) => "Saved to sequences.txt (backup created)".into(),
                Err(error) => format!("Save failed: {error}"),
            };
            publish(ResultMessage {
                choice: Some(job.choice),
                kind: job.kind,
                slot: job.slot,
                index: job.index,
                revision: job.revision,
                message,
                success,
                deleted: job.delete,
                snapshot: (success && job.tuning.is_none()).then_some((job.sequence, job.name)),
                tuning: job.tuning,
            });
        }) {
        Ok(handle) => Some(handle),
        Err(error) => {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::SaveFailed);
            publish(ResultMessage {
                choice: None,
                kind: identity.0,
                slot: identity.1,
                index: identity.2,
                revision: identity.3,
                deleted: identity.4,
                success: false,
                snapshot: None,
                tuning: identity.5,
                message: format!("Cannot start save worker: {error}"),
            });
            None
        }
    }
}

fn publish_tuning(
    saved: &Mutex<kirby_assist_core::sequence_config::SequenceLibrary>,
    kind: i32,
    tuning: kirby_assist_core::fighter_tuning::Tuning,
) {
    saved
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .tuning
        .insert(kind, tuning);
}
fn write(job: &Job) -> Result<bool, String> {
    if let Some(tuning) = job.tuning {
        write_tuning_paths(
            job.kind,
            tuning,
            Path::new(PATH),
            Path::new(TEMP),
            Path::new(BACKUP),
        )?;
        if let Some(saved) = super::super::SEQUENCES.get() {
            publish_tuning(saved, job.kind, tuning);
        }
        return Ok(true);
    }
    let (catalog, changed) =
        write_recipe_paths(job, Path::new(PATH), Path::new(TEMP), Path::new(BACKUP))?;
    if !changed {
        return Ok(false); // No file replacement or live recipe publication for an unchanged save.
    }
    // Recipe saves replace the owned snapshot on the temporary worker.
    if let Some(saved) = super::super::SEQUENCES.get() {
        let library = kirby_assist_core::sequence_config::SequenceLibrary::from_catalog(catalog);
        let old = {
            let mut saved = saved.lock().unwrap_or_else(|e| e.into_inner());
            std::mem::replace(&mut *saved, library)
        };
        super::super::SEQUENCES_UPDATED.store(true, Ordering::Release);
        drop(old);
    }
    Ok(true)
}
fn write_tuning_paths(
    kind: i32,
    tuning: kirby_assist_core::fighter_tuning::Tuning,
    path: &Path,
    temp: &Path,
    backup: &Path,
) -> Result<(), String> {
    recover_paths(path, temp)?;
    let original = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    // Full-file validation still runs, but its temporary catalogs are dropped
    // before disk replacement. No recipe snapshot survives/publishes this save.
    let updated = persist::update_tuning(&original, kind, tuning)?;
    commit_paths(path, temp, backup, &original, &updated)
}
fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    // Bound the actual read, not only a racy metadata check: FTP/other tools
    // can replace or grow the file between stat and read.
    let file = fs::File::open(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let mut data = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    if data.len() as u64 > MAX_FILE_BYTES {
        return Err(format!("{} exceeds the 512 KiB save limit", path.display()));
    }
    Ok(data)
}
#[cfg(test)]
fn write_paths(
    job: &Job,
    path: &Path,
    temp: &Path,
    backup_path: &Path,
) -> Result<kirby_assist_core::sequence_config::Catalog, String> {
    write_recipe_paths(job, path, temp, backup_path).map(|(catalog, _)| catalog)
}
fn write_recipe_paths(
    job: &Job,
    path: &Path,
    temp: &Path,
    backup_path: &Path,
) -> Result<(kirby_assist_core::sequence_config::Catalog, bool), String> {
    if job.tuning.is_some() {
        return Err("Fighter settings must use the dedicated settings save path".into());
    }
    recover_paths(path, temp)?;
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err("Existing sequences.txt is too large".into());
    }
    let original = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    let updated = if job.delete {
        persist::delete(&original, job.kind, job.slot, &job.choice, job.allow_new)?
    } else {
        persist::update_named(
            &original,
            job.kind,
            job.slot,
            &job.choice,
            job.sequence,
            job.allow_new,
            job.name.as_deref(),
        )?
    };

    let updated = if job.make_default && !job.delete {
        persist::set_default(&updated, job.kind, job.slot, &job.choice)?
    } else {
        updated
    };
    let saved_catalog = kirby_assist_core::sequence_config::Catalog::parse(&updated)?
        .for_build(cfg!(feature = "diagnostic-logging"));

    if updated == original {
        return Ok((saved_catalog, false));
    }
    commit_paths(path, temp, backup_path, &original, &updated)?;
    Ok((saved_catalog, true))
}
fn commit_paths(
    path: &Path,
    temp: &Path,
    backup_path: &Path,
    original: &str,
    updated: &str,
) -> Result<(), String> {
    // Refuse to reuse an unclaimed temporary file. Replacement preserves the
    // original under a recovery name when Switch cannot overwrite by rename.
    let mut temporary = OpenOptions::new()
        .write(true)
        // Skyline's Switch std backend tests create and create_new separately:
        // create_new alone returns raw FS result 514 for a missing file.
        .create(true)
        .create_new(true)
        .open(temp)
        .map_err(|e| format!("Cannot create save temporary file: {e}"))?;
    let result = (|| {
        temporary
            .write_all(updated.as_bytes())
            .map_err(|e| e.to_string())?;
        temporary.sync_all().map_err(|e| e.to_string())?;
        // Close the native write handle before read-back and later renames.
        drop(temporary);
        if read_bounded(temp)? != updated.as_bytes() {
            return Err("Temporary file verification failed".into());
        }
        if read_bounded(path)? != original.as_bytes() {
            return Err("sequences.txt changed during save; retry".into());
        }
        let mut backup = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(backup_path)
            .map_err(|e| format!("Cannot create backup: {e}"))?;
        backup
            .write_all(original.as_bytes())
            .map_err(|e| e.to_string())?;
        backup.sync_all().map_err(|e| e.to_string())?;
        drop(backup);
        if read_bounded(backup_path)? != original.as_bytes() {
            return Err("Backup verification failed".into());
        }
        // Detect ordinary concurrent edits again after the backup I/O.
        if read_bounded(path)? != original.as_bytes() {
            return Err("sequences.txt changed during save; retry".into());
        }
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(temp);
        return Err(error);
    }
    if let Err(error) = replace_file(path, temp, original, |from, to| fs::rename(from, to)) {
        let _ = fs::remove_file(temp);
        return Err(error);
    }
    Ok(())
}

fn recovery_path(path: &Path) -> std::path::PathBuf {
    path.with_file_name(format!(
        "{}.menu.rollback",
        path.file_name().unwrap_or_default().to_string_lossy()
    ))
}
fn exists(path: &Path) -> Result<bool, String> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Cannot inspect {}: {e}", path.display())),
    }
}
fn validate_file(path: &Path) -> Result<(), String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_FILE_BYTES {
        return Err("Recovery file is too large".into());
    }
    let source = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    persist::validate_output(&source)
}
/// Startup/next-save recovery is inactive unless an interrupted replacement
/// left our rollback marker. A valid published file always wins over the old one.
pub(super) fn recover() -> Result<(), String> {
    recover_paths(Path::new(PATH), Path::new(TEMP))
}
fn recover_paths(path: &Path, temp: &Path) -> Result<(), String> {
    let rollback = recovery_path(path);
    if !exists(&rollback)? {
        return Ok(());
    }
    if exists(path)? {
        validate_file(path).map_err(|e| {
            format!(
                "Interrupted menu save: live file invalid; original retained at {}: {e}",
                rollback.display()
            )
        })?;
        // A valid published file must still load if optional cleanup fails.
        // A later save retries cleanup before moving the live file again.
        let _ = fs::remove_file(&rollback);
    } else {
        validate_file(&rollback)?;
        fs::rename(&rollback, path)
            .map_err(|e| format!("Cannot restore interrupted menu save: {e}"))?;
    }
    // The recovered/published file is valid. Cleanup must not disable the mod.
    let _ = fs::remove_file(temp);
    Ok(())
}
fn replace_file<F>(path: &Path, temp: &Path, original: &str, mut rename: F) -> Result<(), String>
where
    F: FnMut(&Path, &Path) -> io::Result<()>,
{
    match rename(temp, path) {
        Ok(()) => return Ok(()),
        Err(e)
            if e.kind() == io::ErrorKind::AlreadyExists
                || e.raw_os_error() == Some(RAW_OS_ERROR_ALREADY_EXISTS) => {}
        Err(e) => {
            return Err(format!(
                "Could not replace sequences.txt; original retained: {e}"
            ))
        }
    }
    let rollback = recovery_path(path);
    if exists(&rollback)? {
        return Err("Previous save recovery file exists; retry after recovery".into());
    }
    if read_bounded(path)? != original.as_bytes() {
        return Err("sequences.txt changed during save; retry".into());
    }
    rename(path, &rollback)
        .map_err(|e| format!("Cannot preserve original before replacement: {e}"))?;
    if let Err(publish) = rename(temp, path) {
        return match rename(&rollback,path) {
            Ok(())=>Err(format!("Save failed; original restored: {publish}")),
            Err(restore)=>Err(format!("Save interrupted; original retained in {}. Restart to recover. Write: {publish}; restore: {restore}",rollback.display())),
        };
    }
    // The live file is already complete. A retained rollback is safe and is
    // cleaned up by startup/next-save recovery if this cleanup fails.
    let _ = fs::remove_file(&rollback);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kirby_assist_core::sequence_config::Catalog;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    const SOURCE: &str = "# custom catalog\n[yoshi.roll.grab]\nname = Tongue grab\nstart_delay=0\ndefault=true\nactions=grab\n\n[kirby.roll.other]\nstart_delay=0\nactions=a\n";
    struct Directory(std::path::PathBuf);
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "kirby-menu-save-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn job() -> Job {
        let catalog=Catalog::parse("[yoshi.roll.grab]\nstart_delay=4\nactions=walk:inward:4:0.5,wait:2,face:opponent,grab:fighter_range").unwrap();
        Job {
            kind: catalog.entries()[0].fighter_kind,
            slot: PresetSlot::Roll,
            index: 0,
            revision: 1,
            choice: "grab".into(),
            name: None,
            delete: false,
            make_default: false,
            allow_new: false,
            sequence: catalog.entries()[0].sequence,
            tuning: None,
        }
    }

    #[test]
    fn pyra_and_mythra_edits_save_once_without_overwriting_the_other_form() {
        let source = format!(
            "{}\n{}",
            include_str!("../../../deployment/ultimate/kirby_assist/fighters/pyra.txt"),
            include_str!("../../../deployment/ultimate/kirby_assist/fighters/mythra.txt")
        );
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, &source).unwrap();
        for (kind, name, delay) in [(90, "pyra", 3), (91, "mythra", 7)] {
            let mut j = job();
            j.kind = kind;
            j.slot = PresetSlot::Neutral;
            j.choice = "custom".into();
            let recipe = Catalog::parse(&format!(
                "[{name}.neutral.custom]\nstart_delay={delay}\nactions=input:a:0:-0.6:1\n"
            ))
            .unwrap();
            j.sequence = recipe.entries()[0].sequence;
            j.name = Some(format!("{name} saved once"));
            j.make_default = true;
            let before = Catalog::parse(&fs::read_to_string(&path).unwrap()).unwrap();
            let other = if kind == kirby_assist_core::fighter_kinds::PYRA {
                kirby_assist_core::fighter_kinds::MYTHRA
            } else {
                kirby_assist_core::fighter_kinds::PYRA
            };
            let unchanged = before.find(other, PresetSlot::Neutral, "custom");
            drop(before);
            let (saved, changed) = write_recipe_paths(&j, &path, &temp, &backup).unwrap();
            assert!(changed);
            assert_eq!(
                saved.find(kind, PresetSlot::Neutral, "custom"),
                Some(j.sequence)
            );
            assert_eq!(saved.find(other, PresetSlot::Neutral, "custom"), unchanged);
            drop(saved);
            assert!(
                !write_recipe_paths(&j, &path, &temp, &backup).unwrap().1,
                "first save must persist without repeating the edit"
            );
        }
    }

    #[test]
    fn first_save_survives_a_concurrent_menu_refresh_without_blocking_input() {
        static WRITES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        fn write_once(job: &Job) -> Result<bool, String> {
            assert_eq!(job.sequence.start_delay, 4);
            WRITES.fetch_add(1, Ordering::SeqCst);
            Ok(true)
        }
        let _guard = WORKER_TEST.lock().unwrap();
        WRITES.store(0, Ordering::SeqCst);
        let ui = UI_SAVE_GATE.lock().unwrap();
        assert!(request_with(job(), true, write_once).starts_with("Saving now"));
        assert!(busy());
        assert_eq!(WRITES.load(Ordering::SeqCst), 0);
        assert!(poll().is_none());
        assert!(request_with(job(), true, write_once).contains("retry"));
        drop(ui);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(result) = poll() {
                assert!(result.success);
                assert_eq!(result.snapshot.unwrap().0.start_delay, 4);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(WRITES.load(Ordering::SeqCst), 1);
        assert!(!busy());
        assert!(WORKER.lock().unwrap().is_none());
        assert!(RESULT.lock().unwrap().is_none());
    }

    #[test]
    fn result_cannot_release_ui_or_next_save_until_worker_teardown_finishes() {
        let _guard = WORKER_TEST.lock().unwrap();
        claim_worker(true).unwrap();
        assert!(try_ui_edit().is_none());
        let posted = std::sync::Arc::new(std::sync::Barrier::new(2));
        let release = std::sync::Arc::new(std::sync::Barrier::new(2));
        let (a, b) = (posted.clone(), release.clone());
        let handle = std::thread::spawn(move || {
            publish(ResultMessage {
                choice: Some("grab".into()),
                kind: kirby_assist_core::fighter_kinds::YOSHI,
                slot: PresetSlot::Roll,
                index: 0,
                revision: 0,
                message: "Saved".into(),
                success: true,
                deleted: false,
                snapshot: None,
                tuning: None,
            });
            a.wait();
            b.wait(); // Simulate post-publication closure/thread-local teardown.
        });
        *WORKER.lock().unwrap() = Some(Worker {
            handle,
            identity: (5, PresetSlot::Roll, 0, 0),
        });
        posted.wait();
        assert!(has_result());
        assert!(poll().is_none());
        assert!(busy());
        assert!(try_ui_edit().is_none());
        assert!(request(job(), true).contains("retry"));
        release.wait();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(result) = poll() {
                assert!(result.success);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert!(!busy());
        assert!(WORKER.lock().unwrap().is_none());
        assert!(try_ui_edit().is_some());
    }

    #[test]
    fn save_is_training_only_and_busy_requests_do_not_accumulate() {
        let _guard = WORKER_TEST.lock().unwrap();
        assert!(!BUSY.load(Ordering::Acquire));
        assert!(request(job(), false).contains("only in Training"));
        assert!(!BUSY.load(Ordering::Acquire));
        claim_worker(true).unwrap();
        assert!(request(job(), true).contains("retry"));
        spawn_worker(job(), |_| Ok(true)).unwrap().join().unwrap();
        assert!(has_result());
        assert!(
            request(job(), true).contains("retry"),
            "completion must be consumed first"
        );
        assert!(poll().unwrap().success);
        assert!(!BUSY.load(Ordering::Acquire));
    }

    #[test]
    fn default_save_publishes_latest_catalog_and_preserves_other_categories() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        let source =
            format!("{SOURCE}\n[yoshi.roll.other]\nstart_delay=0\ndefault=false\nactions=jab\n");
        fs::write(&path, &source).unwrap();
        let mut selected = job();
        selected.choice = "other".into();
        selected.make_default = true;
        let saved = write_paths(&selected, &path, &temp, &backup).unwrap();
        assert!(
            !saved
                .entries()
                .iter()
                .find(
                    |e| e.fighter_kind == kirby_assist_core::fighter_kinds::YOSHI
                        && e.choice == "grab"
                )
                .unwrap()
                .default
        );
        assert!(
            saved
                .entries()
                .iter()
                .find(
                    |e| e.fighter_kind == kirby_assist_core::fighter_kinds::YOSHI
                        && e.choice == "other"
                )
                .unwrap()
                .default
        );
        assert_eq!(fs::read_to_string(&backup).unwrap(), source);
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("[kirby.roll.other]\nstart_delay=0\nactions=a"));
    }

    #[test]
    fn worker_exits_after_one_job_on_success_and_error_and_allows_the_next_save() {
        let _guard = WORKER_TEST.lock().unwrap();
        for operation in [
            (|_: &Job| Ok(true)) as fn(&Job) -> Result<bool, String>,
            (|_: &Job| Err("test I/O failure".into())) as fn(&Job) -> Result<bool, String>,
        ] {
            claim_worker(true).unwrap();
            let handle = spawn_worker(job(), operation).unwrap();
            handle.join().unwrap(); // A persistent recv-loop would never finish.
            let result = poll().unwrap();
            assert!(
                result.message.contains("Saved") || result.message.contains("test I/O failure")
            );
            assert!(!BUSY.load(Ordering::Acquire));
        }
    }
    #[test]
    fn oversized_reads_are_bounded_and_do_not_start_replacement() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::File::create(&path)
            .unwrap()
            .set_len(MAX_FILE_BYTES + 1)
            .unwrap();
        assert!(read_bounded(&path).unwrap_err().contains("512 KiB"));
        assert!(write_paths(&job(), &path, &temp, &backup).is_err());
        assert_eq!(fs::metadata(&path).unwrap().len(), MAX_FILE_BYTES + 1);
        assert!(!temp.exists() && !backup.exists());
    }

    #[test]
    fn switch_style_rename_preserves_original_and_publishes_valid_temp() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        fs::write(&path, SOURCE).unwrap();
        let updated = persist::update(
            SOURCE,
            job().kind,
            job().slot,
            "grab",
            job().sequence,
            false,
        )
        .unwrap();
        fs::write(&temp, &updated).unwrap();
        replace_file(&path, &temp, SOURCE, |from, to| {
            if to.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Switch rename target exists",
                ));
            }
            fs::rename(from, to)
        })
        .unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), updated);
        assert!(!recovery_path(&path).exists());
        assert!(!temp.exists());
    }
    #[test]
    fn failed_switch_publish_rolls_back_without_losing_original() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        fs::write(&path, SOURCE).unwrap();
        fs::write(&temp, "new draft").unwrap();
        let result = replace_file(&path, &temp, SOURCE, |from, to| {
            if to.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "target exists",
                ));
            }
            if from == temp {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "simulated publish failure",
                ));
            }
            fs::rename(from, to)
        });
        assert!(result.unwrap_err().contains("original restored"));
        assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
        assert!(!recovery_path(&path).exists());
    }
    #[test]
    fn interrupted_switch_save_recovers_missing_live_file_or_keeps_valid_published_file() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let rollback = recovery_path(&path);
        fs::write(&rollback, SOURCE).unwrap();
        fs::write(&temp, "interrupted draft").unwrap();
        recover_paths(&path, &temp).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
        assert!(!temp.exists());
        assert!(!rollback.exists());
        let updated = persist::update(
            SOURCE,
            job().kind,
            job().slot,
            "grab",
            job().sequence,
            false,
        )
        .unwrap();
        fs::write(&rollback, SOURCE).unwrap();
        fs::write(&path, &updated).unwrap();
        recover_paths(&path, &temp).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), updated);
        assert!(!rollback.exists());
    }
    #[test]
    fn recovery_does_not_overwrite_an_invalid_live_file_or_discard_its_original() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let rollback = recovery_path(&path);
        fs::write(&rollback, SOURCE).unwrap();
        fs::write(&path, "external invalid edit").unwrap();
        assert!(recover_paths(&path, &temp).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "external invalid edit");
        assert_eq!(fs::read_to_string(&rollback).unwrap(), SOURCE);
    }
    #[test]
    fn diagnostic_rename_and_delete_are_validated_and_backed_up() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        let mut edited = job();
        edited.name = Some("Perfect grab".into());
        write_paths(&edited, &path, &temp, &backup).unwrap();
        let renamed = fs::read_to_string(&path).unwrap();
        assert_eq!(
            Catalog::parse(&renamed).unwrap().entries()[0]
                .name
                .as_deref(),
            Some("Perfect grab")
        );
        edited.delete = true;
        write_paths(&edited, &path, &temp, &backup).unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), renamed);
        let deleted = fs::read_to_string(&path).unwrap();
        assert_eq!(Catalog::parse(&deleted).unwrap().entries().len(), 1);
        assert!(deleted.contains("[kirby.roll.other]"));
    }
    #[test]
    fn save_writes_valid_recipe_and_verified_backup_preserving_other_sections() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        write_paths(&job(), &path, &temp, &backup).unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.ends_with("[kirby.roll.other]\nstart_delay=0\nactions=a\n"));
        assert_eq!(
            Catalog::parse(&saved).unwrap().entries()[0]
                .sequence
                .actions(),
            job().sequence.actions()
        );
        assert!(!temp.exists());
    }
    #[test]
    fn saving_first_nondefault_y_recipe_remains_valid_on_restart() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        let mut custom = job();
        custom.kind = 5;
        custom.slot = PresetSlot::Y;
        custom.index = 1;
        custom.choice = "menu_custom".into();
        custom.allow_new = true;
        write_paths(&custom, &path, &temp, &backup).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        let catalog = Catalog::parse(&saved).unwrap();
        let y = catalog
            .entries()
            .iter()
            .find(|e| {
                e.fighter_kind == kirby_assist_core::fighter_kinds::YOSHI && e.slot == PresetSlot::Y
            })
            .unwrap();
        assert!(!y.default);
        for diagnostic in [false, true] {
            let mut profiles = kirby_assist_core::fighter_profiles::FighterProfiles::default();
            profiles
                .add_sequence_choices(&catalog.clone().for_build(diagnostic))
                .unwrap();
            profiles.validate().unwrap();
        }
        assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
    }
    #[test]
    fn saving_a_dpad_mode_keeps_binding_type_activation_and_startup_validity() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        let mut mode = job();
        mode.kind = 5;
        mode.slot = PresetSlot::DpadUp;
        mode.choice = "__mode_ledge_trump_toggle".into();
        mode.allow_new = true;
        mode.name = Some("Toggle: Ledge trump".into());
        mode.sequence = Sequence {
            mode: Some(kirby_assist_core::button_bindings::ModeBinding {
                mode: kirby_assist_core::button_bindings::Mode::LedgeTrump,
                activation: kirby_assist_core::button_bindings::Activation::Toggle,
            }),
            ..Default::default()
        };
        write_paths(&mode, &path, &temp, &backup).unwrap();
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("[yoshi.dpad_up.__mode_ledge_trump_toggle]"));
        assert!(saved.contains("mode = ledge_trump\nactivation = toggle"));
        let catalog = Catalog::parse(&saved).unwrap();
        assert_eq!(
            catalog
                .entries()
                .iter()
                .find(|e| e.slot == PresetSlot::DpadUp)
                .unwrap()
                .sequence,
            mode.sequence
        );
        let mut profiles = kirby_assist_core::fighter_profiles::FighterProfiles::default();
        profiles.add_sequence_choices(&catalog).unwrap();
        profiles.validate().unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
    }
    #[test]
    fn invalid_file_or_existing_temp_never_overwrites_live_file() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, "invalid").unwrap();
        assert!(write_paths(&job(), &path, &temp, &backup).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "invalid");
        assert!(!backup.exists());
        assert!(!temp.exists());
        fs::write(&path, SOURCE).unwrap();
        fs::write(&temp, "leftover").unwrap();
        assert!(write_paths(&job(), &path, &temp, &backup).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
        assert_eq!(fs::read_to_string(&temp).unwrap(), "leftover");
        assert!(!backup.exists());
    }
    #[test]
    fn backup_failure_keeps_live_file_and_cleans_own_temp() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("missing/backup");
        fs::write(&path, SOURCE).unwrap();
        assert!(write_paths(&job(), &path, &temp, &backup).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
        assert!(!temp.exists());
    }
    #[test]
    fn fighter_settings_share_verified_save_and_preserve_all_recipes() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        let job = job();
        let mut tuning = kirby_assist_core::fighter_tuning::Tuning::default();
        tuning.set("walk_strength", "0.28").unwrap();
        tuning.set("grab_range", "14").unwrap();
        write_tuning_paths(job.kind, tuning, &path, &temp, &backup).unwrap();
        let saved = Catalog::parse(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(saved.tuning[&5], tuning);
        assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
        assert!(fs::read_to_string(&path).unwrap().starts_with(SOURCE));
        assert!(!temp.exists());
    }

    #[test]
    fn publishing_settings_preserves_compact_recipes_and_other_fighters() {
        let mut original = Catalog::parse(SOURCE).unwrap();
        let mut other = kirby_assist_core::fighter_tuning::Tuning::default();
        other.set("grab_range", "8").unwrap();
        original.tuning.insert(6, other);
        let original_entries = original.entries().to_vec();
        let saved =
            Mutex::new(kirby_assist_core::sequence_config::SequenceLibrary::from_catalog(original));
        let mut tuning = kirby_assist_core::fighter_tuning::Tuning::default();
        tuning.set("stand_distance", "20").unwrap();
        publish_tuning(&saved, 5, tuning);
        let after = saved.lock().unwrap();
        let mut entries = after.for_fighter(5).entries().to_vec();
        entries.extend_from_slice(after.for_fighter(6).entries());
        assert_eq!(entries, original_entries);
        assert_eq!(after.tuning[&5], tuning);
        assert_eq!(after.tuning[&6], other);
    }
    #[test]
    fn settings_backup_failure_preserves_active_file() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("missing/backup");
        fs::write(&path, SOURCE).unwrap();
        assert!(write_tuning_paths(5, Default::default(), &path, &temp, &backup).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
        assert!(!temp.exists());
    }

    #[test]
    fn unchanged_recipe_save_does_not_replace_file_or_backup() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        let mut j = job();
        let source="[yoshi.roll.grab]\nname=Grab\nstart_delay=4\nactions=walk:inward:4:0.5,wait:2,face:opponent,grab:fighter_range\n";
        j.name = Some("Grab".into());
        fs::write(&path, source).unwrap();
        fs::write(&backup, "previous backup must stay").unwrap();
        for _ in 0..4 {
            let (_, changed) = write_recipe_paths(&j, &path, &temp, &backup).unwrap();
            assert!(!changed);
            assert_eq!(fs::read_to_string(&path).unwrap(), source);
            assert_eq!(
                fs::read_to_string(&backup).unwrap(),
                "previous backup must stay"
            );
            assert!(!temp.exists());
        }
    }

    #[test]
    fn long_button_holds_and_until_takeover_save_without_expanding_actions() {
        for input in ["hold:b:160", "hold:b:255", "hold_until_manual:b:0:0"] {
            let directory = Directory::new();
            let path = directory.0.join("sequences.txt");
            let temp = directory.0.join("temp");
            let backup = directory.0.join("backup");
            fs::write(&path, SOURCE).unwrap();
            let mut j = job();
            j.sequence = Catalog::parse(&format!(
                "[yoshi.roll.grab]\nstart_delay=0\nactions={input}\n"
            ))
            .unwrap()
            .entries()[0]
                .sequence;
            write_paths(&j, &path, &temp, &backup).unwrap();
            let saved = Catalog::parse(&fs::read_to_string(&path).unwrap()).unwrap();
            assert_eq!(
                saved.find(5, PresetSlot::Roll, "grab").unwrap().actions(),
                j.sequence.actions()
            );
            assert_eq!(j.sequence.len, 1);
            assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
        }
    }

    #[test]
    fn exited_worker_without_a_result_releases_save_gate_and_reports_failure() {
        let _guard = WORKER_TEST.lock().unwrap();
        claim_worker(true).unwrap();
        let handle = std::thread::spawn(|| {});
        while !handle.is_finished() {
            std::thread::yield_now();
        }
        *WORKER.lock().unwrap() = Some(Worker {
            handle,
            identity: (5, PresetSlot::Roll, 9, 123),
        });
        assert!(has_result());
        let result = poll().unwrap();
        assert!(!result.success);
        assert_eq!((result.kind, result.index, result.revision), (5, 9, 123));
        assert!(result.message.contains("not confirmed"));
        assert!(!busy());
        assert!(try_ui_edit().is_some());
    }
    #[test]
    fn malformed_save_snapshot_is_rejected_before_starting_a_worker() {
        let _guard = WORKER_TEST.lock().unwrap();
        let mut j = job();
        j.sequence.len = 255;
        assert!(request(j, true).contains("action limit"));
        assert!(!busy());
        assert!(WORKER.lock().unwrap().is_none());
        let mut j = job();
        j.sequence.len = 0;
        assert!(request(j, true).contains("add an action"));
        assert!(!busy());
    }
    #[test]
    fn repeated_boundary_value_saves_and_invalid_axes_preserve_the_file() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        for i in 0..24 {
            let mut j = job();
            let x = if i % 2 == 0 { -1. } else { 1. };
            j.sequence=Catalog::parse(&format!("[yoshi.roll.grab]\nstart_delay=0\nactions=facing_input:b:{x}:1:255,wait:255,down_smash:160\n")).unwrap().entries()[0].sequence;
            let before = fs::read(&path).unwrap();
            write_paths(&j, &path, &temp, &backup).unwrap();
            assert_eq!(
                Catalog::parse(&fs::read_to_string(&path).unwrap())
                    .unwrap()
                    .find(5, PresetSlot::Roll, "grab")
                    .unwrap(),
                j.sequence
            );
            assert_eq!(fs::read(&backup).unwrap(), before);
            assert!(!temp.exists());
        }
        let before = fs::read(&path).unwrap();
        let mut j = job();
        j.sequence.len = 255;
        assert!(write_paths(&j, &path, &temp, &backup).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        let mut j = job();
        j.sequence.actions[0] = kirby_assist_core::sequence_config::Action::Input {
            buttons: 64,
            stick: [f32::NAN, 1.],
            frames: 255,
            relative: false,
        };
        assert!(write_paths(&j, &path, &temp, &backup).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(!temp.exists());
    }

    #[test]
    fn full_catalog_boundary_save_fits_the_actual_worker_stack() {
        std::thread::Builder::new().stack_size(WORKER_STACK_BYTES).spawn(|| {
            let source=include_str!("../../../deployment/ultimate/kirby_assist/assembled/sequences.txt");
            let catalog=Catalog::parse(source).unwrap();let entry=&catalog.entries()[0];
            let mut j=job();j.kind=entry.fighter_kind;j.slot=entry.slot;j.choice=entry.choice.clone();
            j.sequence=Catalog::parse("[kirby.neutral.test]\nstart_delay=0\nactions=facing_input:b:1:-1:255,wait:255,down_smash:160\n").unwrap().entries()[0].sequence;
            drop(catalog);
            let directory=Directory::new();let path=directory.0.join("sequences.txt");
            let temp=directory.0.join("temp");let backup=directory.0.join("backup");fs::write(&path,source).unwrap();
            let saved=write_paths(&j,&path,&temp,&backup).unwrap();
            assert_eq!(saved.find(j.kind,j.slot,&j.choice),Some(j.sequence));
            assert_eq!(fs::read_to_string(backup).unwrap(),source);assert!(!temp.exists());
        }).unwrap().join().unwrap();
    }

    #[test]
    fn current_full_roster_edits_defaults_copies_deletes_and_settings_round_trip() {
        std::thread::Builder::new()
            .stack_size(WORKER_STACK_BYTES)
            .spawn(|| {
                // Read maintained split sources, not a potentially stale generated
                // bundle. All writes/backup/recovery checks use a temporary folder.
                let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../deployment/ultimate/kirby_assist");
                let mut source = fs::read_to_string(root.join("sequences.txt")).unwrap();
                source.push_str("\n\n");
                let mut files: Vec<_> = fs::read_dir(root.join("fighters"))
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .filter(|p| p.extension().is_some_and(|e| e == "txt"))
                    .collect();
                files.sort();
                for file in files {
                    source.push_str(&fs::read_to_string(file).unwrap());
                    source.push_str("\n\n");
                }
                let original = Catalog::parse(&source).unwrap();
                let entry = original
                    .entries()
                    .iter()
                    .find(|e| {
                        e.fighter == "lucina"
                            && e.slot == PresetSlot::Neutral
                            && !e.sequence.reaction.active()
                    })
                    .unwrap();
                let untouched = original.find(15, PresetSlot::Roll, "auto_roll").unwrap();
                let mut j = job();
                j.kind = entry.fighter_kind;
                j.slot = entry.slot;
                j.choice = entry.choice.clone();
                j.sequence = entry.sequence;
                j.sequence.start_delay = 31;
                j.name = Some("Training saved once".into());
                j.make_default = true;
                drop(original);
                let directory = Directory::new();
                let path = directory.0.join("sequences.txt");
                let temp = directory.0.join("temp");
                let backup = directory.0.join("backup");
                fs::write(&path, &source).unwrap();
                let saved = write_paths(&j, &path, &temp, &backup).unwrap();
                assert_eq!(saved.find(j.kind, j.slot, &j.choice), Some(j.sequence));
                assert_eq!(
                    saved.find(15, PresetSlot::Roll, "auto_roll"),
                    Some(untouched)
                );
                assert_eq!(
                    saved
                        .entries()
                        .iter()
                        .find(|e| e.fighter_kind == j.kind
                            && e.slot == j.slot
                            && e.choice == j.choice)
                        .unwrap()
                        .name
                        .as_deref(),
                    Some("Training saved once")
                );
                assert_eq!(fs::read_to_string(&backup).unwrap(), source);
                drop(saved);
                let once = fs::read_to_string(&path).unwrap();
                assert!(
                    !write_recipe_paths(&j, &path, &temp, &backup).unwrap().1,
                    "unchanged retry should not require another write"
                );
                assert_eq!(fs::read_to_string(&path).unwrap(), once);
                // A copied draft uses the one free catalog section, then deletion
                // restores that capacity. This exercises the actual near-limit file.
                let mut copied = job();
                copied.kind = j.kind;
                copied.slot = PresetSlot::X;
                copied.choice = "training_copy".into();
                copied.sequence = j.sequence;
                copied.allow_new = true;
                copied.name = Some("Copied training recipe".into());
                copied.make_default = true;
                let saved = write_paths(&copied, &path, &temp, &backup).unwrap();
                assert_eq!(
                    saved.find(copied.kind, copied.slot, &copied.choice),
                    Some(copied.sequence)
                );
                drop(saved);
                let before_delete = fs::read_to_string(&path).unwrap();
                copied.delete = true;
                let saved = write_paths(&copied, &path, &temp, &backup).unwrap();
                assert!(saved
                    .find(copied.kind, copied.slot, &copied.choice)
                    .is_none());
                drop(saved);
                assert_eq!(fs::read_to_string(&backup).unwrap(), before_delete);
                let mut tuning = kirby_assist_core::fighter_tuning::Tuning::default();
                tuning.set("walk_strength", "0.4").unwrap();
                let before_settings = fs::read_to_string(&path).unwrap();
                write_tuning_paths(j.kind, tuning, &path, &temp, &backup).unwrap();
                let final_source = fs::read_to_string(&path).unwrap();
                persist::validate_output(&final_source).unwrap();
                let saved = Catalog::parse(&final_source).unwrap();
                assert_eq!(saved.tuning.get(&j.kind), Some(&tuning));
                assert_eq!(saved.find(j.kind, j.slot, &j.choice), Some(j.sequence));
                assert_eq!(fs::read_to_string(&backup).unwrap(), before_settings);
                assert!(!temp.exists());
                assert!(final_source.len() as u64 <= MAX_FILE_BYTES);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn button_default_save_keeps_latest_edited_actions_timing_name_and_loop() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        let source =
            format!("{SOURCE}\n[yoshi.y.old]\nstart_delay=0\ndefault=true\nactions=jab:4\n");
        fs::write(&path, &source).unwrap();
        let mut j = job();
        j.slot = PresetSlot::Y;
        j.choice = "new".into();
        j.name = Some("Edited button default".into());
        j.allow_new = true;
        j.make_default = true;
        j.sequence=Catalog::parse("[yoshi.y.new]\nstart_delay=5\nwhile_held=true\nground_only=true\nactions=facing_input:b:1:1:160,wait:7,drift:opponent:1:40\n").unwrap().entries()[0].sequence;
        let saved = write_paths(&j, &path, &temp, &backup).unwrap();
        let selected = saved
            .entries()
            .iter()
            .find(|e| {
                e.fighter_kind == kirby_assist_core::fighter_kinds::YOSHI
                    && e.slot == PresetSlot::Y
                    && e.default
            })
            .unwrap();
        assert_eq!(selected.choice, "new");
        assert_eq!(selected.sequence, j.sequence);
        assert_eq!(selected.name.as_deref(), j.name.as_deref());
        assert_eq!(
            saved
                .entries()
                .iter()
                .filter(
                    |e| e.fighter_kind == kirby_assist_core::fighter_kinds::YOSHI
                        && e.slot == PresetSlot::Y
                        && e.default
                )
                .count(),
            1
        );
        assert_eq!(fs::read_to_string(backup).unwrap(), source);
        assert!(!temp.exists());
    }
}
