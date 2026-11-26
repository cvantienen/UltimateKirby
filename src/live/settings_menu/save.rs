//! Explicit menu saves only. No SD work on the input/render/observation thread.
//! Technical: Handles explicit save/load file operations, recovery, and serialized
//! settings data away from input and render callbacks.
//! Plain English: This module reads or saves settings only when the player asks.
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

/// Canonical saved sequence and tuning file path.
const PATH: &str = "sd:/ultimate/kirby_assist/sequences.txt";
/// Exclusive temporary path used while writing a replacement.
const TEMP: &str = "sd:/ultimate/kirby_assist/sequences.txt.menu.tmp";
/// Verified recovery copy retained while replacing the canonical file.
const BACKUP: &str = "sd:/ultimate/kirby_assist/sequences.txt.menu.bak";
/// Stack allocation reserved for the save worker thread.
const WORKER_STACK_BYTES: usize = 2 * 1024 * 1024;
/// Switch filesystem error code for an already-existing file.
const RAW_OS_ERROR_ALREADY_EXISTS: i32 = 0x402;
/// Immutable request data captured for one explicit save operation.
pub(super) struct Job {
    /// Native fighter kind associated with this request.
    pub kind: i32,
    /// Preset category containing the selected choice.
    pub slot: PresetSlot,
    /// Index of the selected preset in the category.
    pub index: usize,
    /// Recipe revision captured when the request was created.
    pub revision: u64,
    /// Stable catalog identifier used to revalidate the selected recipe.
    pub choice: String,
    /// Optional persisted display-name override.
    pub name: Option<String>,
    /// Whether this request removes rather than updates the recipe.
    pub delete: bool,
    /// Whether a successful update should set the startup default.
    pub make_default: bool,
    /// Whether the selected catalog entry may be created by persistence.
    pub allow_new: bool,
    /// Value snapshot of the recipe to persist.
    pub sequence: Sequence,
    /// Optional fighter-tuning payload; present for tuning-only jobs.
    pub tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
/// What a result must report when its worker cannot deliver one itself.
#[derive(Clone, Copy)]
struct Identity {
    /// Fighter kind retained for a fallback failure message.
    kind: i32,
    /// Preset category retained for a fallback failure message.
    slot: PresetSlot,
    /// Selected catalog index retained for result correlation.
    index: usize,
    /// Source revision retained to reject stale completion application.
    revision: u64,
    /// Whether the request represented deletion.
    delete: bool,
    /// Optional tuning payload identity.
    tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
impl Job {
    /// Copies the fields needed to correlate an eventual completion.
    fn identity(&self) -> Identity {
        Identity {
            kind: self.kind,
            slot: self.slot,
            index: self.index,
            revision: self.revision,
            delete: self.delete,
            tuning: self.tuning,
        }
    }
}
/// Completion payload returned from the worker to the menu owner.
pub(super) struct ResultMessage {
    /// Stable catalog identifier of the completed recipe, if applicable.
    pub choice: Option<String>,
    /// Native fighter kind associated with the completed request.
    pub kind: i32,
    /// Preset category associated with the completed request.
    pub slot: PresetSlot,
    /// Catalog index associated with the completed request.
    pub index: usize,
    /// Source recipe revision associated with the completed request.
    pub revision: u64,
    /// User-facing completion or failure status.
    pub message: String,
    /// Whether the file operation completed successfully.
    pub success: bool,
    /// Whether the operation deleted the selected recipe.
    pub deleted: bool,
    /// Successful recipe snapshot used to reconcile current edits.
    pub snapshot: Option<(Sequence, Option<String>)>,
    /// Optional tuning payload associated with the operation.
    pub tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
impl ResultMessage {
    /// Constructs a failure completion preserving request correlation fields.
    fn failure(identity: Identity, message: String) -> Self {
        Self {
            choice: None,
            kind: identity.kind,
            slot: identity.slot,
            index: identity.index,
            revision: identity.revision,
            message,
            success: false,
            deleted: identity.delete,
            snapshot: None,
            tuning: identity.tuning,
        }
    }
}
// No pending snapshots/queue: explicit Training saves start one temporary
// worker immediately. Retain its handle until exit; never detach a live
// Switch worker. A second save waits until completion is consumed.
/// Serializes explicit save requests across worker creation and result consumption.
static BUSY: AtomicBool = AtomicBool::new(false);
/// Join handle and request identity retained until worker teardown.
struct Worker {
    /// Join handle used to confirm worker termination without blocking on active work.
    handle: std::thread::JoinHandle<()>,
    /// Request identity used if the worker exits without publishing a result.
    identity: Identity,
}
/// Owns the sole current save worker until its join completes.
static WORKER: Mutex<Option<Worker>> = Mutex::new(None);
/// Keeps replaced libraries alive until the menu/input thread can drop them.
/// Large catalog destruction on the save worker caused post-save pauses/crashes.
static RETIRED_LIBRARIES: Mutex<Vec<kirby_assist_core::sequence_config::SequenceLibrary>> =
    Mutex::new(Vec::new());
/// Drops replaced catalogs outside the save worker.
pub(super) fn reap_retired_libraries() {
    let retired = std::mem::take(
        &mut *RETIRED_LIBRARIES
            .lock()
            .unwrap_or_else(|error| error.into_inner()),
    );
    drop(retired);
}
// Only explicit saves and cached-text replacement use this gate; never inputs,
// detection or drawing. Check BUSY under the gate to close the start/rebuild race.
/// Serializes explicit saves with cached-view replacement.
static UI_SAVE_GATE: Mutex<()> = Mutex::new(());
/// Acquires the short cached-UI edit gate only while no save is active.
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
/// Reads the save-in-progress flag with acquire ordering.
pub(super) fn busy() -> bool {
    BUSY.load(Ordering::Acquire)
}
/// Signals that a result is available in [`RESULT`].
static RESULT_READY: AtomicBool = AtomicBool::new(false);
/// Holds the single published completion message for the UI thread.
static RESULT: Mutex<Option<ResultMessage>> = Mutex::new(None);
#[cfg(test)]
/// Serializes tests that access the process-wide worker state.
pub(super) static WORKER_TEST: Mutex<()> = Mutex::new(());
/// Checks for a published result or an exited worker requiring synthesis.
pub(super) fn has_result() -> bool {
    RESULT_READY.load(Ordering::Acquire)
        || busy()
            && WORKER.try_lock().is_ok_and(|worker| {
                worker
                    .as_ref()
                    .is_some_and(|worker| worker.handle.is_finished())
            })
}
/// Joins only a finished worker, consumes its result, and releases the save gate.
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
            result = Some(ResultMessage::failure(
                identity,
                "Save worker stopped unexpectedly; save not confirmed. Retry Save".into(),
            ));
        }
    }
    if result.is_some() {
        RESULT_READY.store(false, Ordering::Release);
        BUSY.store(false, Ordering::Release);
    }
    result
}
/// Claims the exclusive save state only for an allowed Training operation.
fn claim_worker(training: bool) -> Result<(), &'static str> {
    if !training {
        return Err("Saving is available only in Training Mode");
    }
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map(|_| ())
        .map_err(|_| "Save in progress; retry when it finishes")
}
/// Validates and starts one explicit save, returning immediate status text.
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
/// Allows tests to substitute filesystem operations without touching the SD path.
pub(super) static TEST_OPERATION: Mutex<Option<fn(&Job) -> Result<bool, String>>> =
    Mutex::new(None);
/// Applies the save validation and worker path with an injectable operation.
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
    let identity = job.identity();
    *worker = spawn_worker(job, operation).map(|handle| Worker { handle, identity });
    "Saving now; keep Smash open until complete".into()
}
/// Replaces the result slot before publishing readiness with release ordering.
fn publish(result: ResultMessage) {
    let mut slot = RESULT.lock().unwrap_or_else(|error| error.into_inner());
    *slot = Some(result);
    RESULT_READY.store(true, Ordering::Release);
}
/// Maps the operation outcome and request flags to stable user-facing text.
fn completion_message(job: &Job, outcome: &Result<bool, String>) -> String {
    match outcome {
        Ok(false) => "Already saved; file unchanged".into(),
        Ok(true) if job.delete => "Deleted from sequences.txt (backup created)".into(),
        Ok(true) if job.make_default => "Saved as startup default (backup created)".into(),
        Ok(true) => "Saved to sequences.txt (backup created)".into(),
        Err(error) => format!("Save failed: {error}"),
    }
}
/// Starts a named, stack-sized worker and publishes one correlated result.
fn spawn_worker(
    job: Job,
    operation: fn(&Job) -> Result<bool, String>,
) -> Option<std::thread::JoinHandle<()>> {
    let identity = job.identity();
    match std::thread::Builder::new()
        .name("recipe-save".into())
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            // Accept the first explicit save even if a cached view is being
            // replaced. Only this background worker waits for that short edit;
            // the input/render callbacks never wait or discard the snapshot.
            let ui_gate = UI_SAVE_GATE.lock().unwrap_or_else(|e| e.into_inner());
            drop(ui_gate);
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
            publish(ResultMessage {
                message: completion_message(&job, &outcome),
                success,
                snapshot: (success && job.tuning.is_none()).then_some((job.sequence, job.name)),
                choice: Some(job.choice),
                kind: job.kind,
                slot: job.slot,
                index: job.index,
                revision: job.revision,
                deleted: job.delete,
                tuning: job.tuning,
            });
        }) {
        Ok(handle) => Some(handle),
        Err(error) => {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::SaveFailed);
            publish(ResultMessage::failure(
                identity,
                format!("Cannot start save worker: {error}"),
            ));
            None
        }
    }
}

/// Publishes the verified disk catalog and returns the old library for deferred cleanup.
fn publish_catalog(
    saved: &Mutex<kirby_assist_core::sequence_config::SequenceLibrary>,
    catalog: kirby_assist_core::sequence_config::Catalog,
) -> kirby_assist_core::sequence_config::SequenceLibrary {
    let library = kirby_assist_core::sequence_config::SequenceLibrary::from_catalog(catalog);
    let mut saved = saved.lock().unwrap_or_else(|e| e.into_inner());
    std::mem::replace(&mut *saved, library)
}
/// Persists a tuning or recipe job and publishes the catalog reread from disk.
fn write(job: &Job) -> Result<bool, String> {
    let (catalog, changed) = if let Some(tuning) = job.tuning {
        write_tuning_paths(
            job.kind,
            tuning,
            Path::new(PATH),
            Path::new(TEMP),
            Path::new(BACKUP),
        )?
    } else {
        write_recipe_paths(job, Path::new(PATH), Path::new(TEMP), Path::new(BACKUP))?
    };
    // Even a no-op save refreshes stale runtime data from the canonical file.
    if let Some(saved) = super::super::SEQUENCES.get() {
        let old = publish_catalog(saved, catalog);
        super::super::SEQUENCES_UPDATED.store(true, Ordering::Release);
        RETIRED_LIBRARIES
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(old);
    }
    Ok(changed)
}
/// Recover an interrupted save, then read the live file and apply `update`.
/// Runs recovery before applying a text transformation to the current file.
fn read_updated(
    path: &Path,
    temp: &Path,
    update: impl FnOnce(&str) -> Result<String, String>,
) -> Result<(String, String), String> {
    recover_paths(path, temp)?;
    let original = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    // A migration conflict must not prevent editing existing assignments.
    // Recovery and the update still validate the original source independently.
    let migrated = persist::migrate_mode_swap(&original).unwrap_or_else(|_| original.clone());
    let updated = update(&migrated)?;
    Ok((original, updated))
}
/// Applies tuning persistence and commits only after validating the full file.
fn write_tuning_paths(
    kind: i32,
    tuning: kirby_assist_core::fighter_tuning::Tuning,
    path: &Path,
    temp: &Path,
    backup: &Path,
) -> Result<(kirby_assist_core::sequence_config::Catalog, bool), String> {
    let (original, updated) = read_updated(path, temp, |original| {
        persist::update_tuning(original, kind, tuning)
    })?;
    let changed = commit_paths(path, temp, backup, &original, &updated)?;
    Ok((reload_saved_catalog(path, &updated)?, changed))
}

fn reload_saved_catalog(
    path: &Path,
    expected: &str,
) -> Result<kirby_assist_core::sequence_config::Catalog, String> {
    let bytes = read_bounded(path)
        .map_err(|error| format!("Save reload failed; runtime not refreshed: {error}"))?;
    if bytes != expected.as_bytes() {
        return Err("Saved file changed before reload; runtime not refreshed. Retry Save".into());
    }
    let source = String::from_utf8(bytes).map_err(|error| error.to_string())?;
    persist::validate_output(&source)?;
    Ok(kirby_assist_core::sequence_config::Catalog::parse(&source)?
        .for_build(cfg!(feature = "diagnostic-logging")))
}
#[cfg(feature = "diagnostic-logging")]
pub(super) fn lan_read() -> Result<(Vec<u8>, String), String> {
    let _gate = try_ui_edit().ok_or("Save or menu edit busy")?;
    recover()?;
    let bytes = read_bounded(Path::new(PATH))?;
    let revision = kirby_assist_core::diagnostic_http::revision(&bytes);
    Ok((bytes, revision))
}

#[cfg(feature = "diagnostic-logging")]
pub(super) fn lan_write(
    bytes: &[u8],
    revision: &str,
    allowed: impl Fn() -> bool,
) -> Result<(), String> {
    struct Release;
    impl Drop for Release {
        fn drop(&mut self) {
            BUSY.store(false, Ordering::Release);
        }
    }
    if !allowed() {
        return Err("Training editor permission expired".into());
    }
    claim_worker(true)?;
    let _release = Release;
    let _ui_gate = UI_SAVE_GATE.lock().unwrap_or_else(|e| e.into_inner());
    let catalog = lan_commit_paths(
        bytes,
        revision,
        Path::new(PATH),
        Path::new(TEMP),
        Path::new(BACKUP),
        allowed,
    )?;
    if let Some(saved) = super::super::SEQUENCES.get() {
        let old = publish_catalog(saved, catalog);
        RETIRED_LIBRARIES
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(old);
        super::super::SEQUENCES_UPDATED.store(true, Ordering::Release);
    }
    #[cfg(not(test))]
    crate::diagnostics::lan::request_reload();
    Ok(())
}

#[cfg(feature = "diagnostic-logging")]
fn lan_commit_paths(
    bytes: &[u8],
    revision: &str,
    path: &Path,
    temp: &Path,
    backup: &Path,
    allowed: impl Fn() -> bool,
) -> Result<kirby_assist_core::sequence_config::Catalog, String> {
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("File exceeds 512 KiB".into());
    }
    let updated = std::str::from_utf8(bytes).map_err(|_| "Sequence file must be UTF-8")?;
    persist::validate_output(updated)?;
    if !allowed() {
        return Err("Training editor permission expired".into());
    }
    recover_paths(path, temp)?;
    let original_bytes = read_bounded(path)?;
    if kirby_assist_core::diagnostic_http::revision(&original_bytes) != revision {
        return Err("Stale file revision; reload before editing".into());
    }
    let original = std::str::from_utf8(&original_bytes).map_err(|_| "Saved file must be UTF-8")?;
    let before = kirby_assist_core::sequence_config::Catalog::parse(original)?;
    let after = kirby_assist_core::sequence_config::Catalog::parse(updated)?;
    if before.universal != after.universal || before.templates != after.templates {
        return Err("Live edits support fighter settings/recipes only; universal settings and templates require restart".into());
    }
    drop((before, after));
    if !allowed() {
        return Err("Training editor permission expired".into());
    }
    // Once admitted, finish the recoverable filesystem transaction even if the
    // player exits Training; never interrupt between rollback/publish renames.
    commit_paths(path, temp, backup, original, updated)?;
    reload_saved_catalog(path, updated)
}

/// Reads at most the file limit plus one detection byte.
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
/// Test adapter returning only the parsed catalog from a recipe write.
fn write_paths(
    job: &Job,
    path: &Path,
    temp: &Path,
    backup_path: &Path,
) -> Result<kirby_assist_core::sequence_config::Catalog, String> {
    write_recipe_paths(job, path, temp, backup_path).map(|(catalog, _)| catalog)
}
/// Validates and serializes recipe edits, then commits the verified replacement.
fn write_recipe_paths(
    job: &Job,
    path: &Path,
    temp: &Path,
    backup_path: &Path,
) -> Result<(kirby_assist_core::sequence_config::Catalog, bool), String> {
    let (original, updated) = read_updated(path, temp, |original| {
        let updated = if job.delete {
            persist::delete(original, job.kind, job.slot, &job.choice, job.allow_new)?
        } else {
            persist::update_named(
                original,
                job.kind,
                job.slot,
                &job.choice,
                job.sequence,
                job.allow_new,
                job.name.as_deref(),
            )?
        };
        if job.make_default && !job.delete {
            persist::set_default(&updated, job.kind, job.slot, &job.choice)
        } else {
            Ok(updated)
        }
    })?;
    let changed = commit_paths(path, temp, backup_path, &original, &updated)?;
    Ok((reload_saved_catalog(path, &updated)?, changed))
}
/// Rejects a commit when the live file changed since its initial read.
fn ensure_unchanged(path: &Path, original: &str) -> Result<(), String> {
    if read_bounded(path)? == original.as_bytes() {
        Ok(())
    } else {
        Err("sequences.txt changed during save; retry".into())
    }
}
/// Write through an already opened file, close the native handle, then read the
/// bytes back before any rename or later step relies on them.
/// Flushes, closes, and verifies a complete file before it can be renamed.
fn write_verified(
    mut file: fs::File,
    path: &Path,
    bytes: &[u8],
    verify_error: &str,
) -> Result<(), String> {
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    if read_bounded(path)? == bytes {
        Ok(())
    } else {
        Err(verify_error.into())
    }
}
/// Replace the live file with `updated`, keeping a verified backup of `original`.
/// Returns false, touching no file, when the update changes nothing.
/// Coordinates backup, temporary-file verification, and rollback-safe replacement.
fn commit_paths(
    path: &Path,
    temp: &Path,
    backup_path: &Path,
    original: &str,
    updated: &str,
) -> Result<bool, String> {
    if updated == original {
        ensure_unchanged(path, original)?;
        return Ok(false);
    }
    // Refuse to reuse an unclaimed temporary file. Replacement preserves the
    // original under a recovery name when Switch cannot overwrite by rename.
    let temporary = OpenOptions::new()
        .write(true)
        // Skyline's Switch std backend tests create and create_new separately:
        // create_new alone returns raw FS result 514 for a missing file.
        .create(true)
        .create_new(true)
        .open(temp)
        .map_err(|e| format!("Cannot create save temporary file: {e}"))?;
    let result = (|| {
        write_verified(
            temporary,
            temp,
            updated.as_bytes(),
            "Temporary file verification failed",
        )?;
        ensure_unchanged(path, original)?;
        let backup = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(backup_path)
            .map_err(|e| format!("Cannot create backup: {e}"))?;
        write_verified(
            backup,
            backup_path,
            original.as_bytes(),
            "Backup verification failed",
        )?;
        // Detect ordinary concurrent edits again after the backup I/O.
        ensure_unchanged(path, original)?;
        replace_file(path, temp, original, |from, to| fs::rename(from, to))
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result.map(|()| true)
}

/// Derives the rollback-marker path associated with a canonical save file.
fn recovery_path(path: &Path) -> std::path::PathBuf {
    path.with_file_name(format!(
        "{}.menu.rollback",
        path.file_name().unwrap_or_default().to_string_lossy()
    ))
}
/// Checks path existence while distinguishing absence from filesystem errors.
fn exists(path: &Path) -> Result<bool, String> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Cannot inspect {}: {e}", path.display())),
    }
}
/// Bounded-reads and validates persisted sequence syntax.
fn validate_file(path: &Path) -> Result<(), String> {
    let source = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    persist::validate_output(&source)
}
/// Startup/next-save recovery is inactive unless an interrupted replacement
/// left our rollback marker. A valid published file always wins over the old one.
/// Restores or cleans an interrupted replacement before normal use.
pub(super) fn recover() -> Result<(), String> {
    recover_paths(Path::new(PATH), Path::new(TEMP))
}
/// Reconciles a rollback marker with the live file and removes stale temporary data.
fn recover_paths(path: &Path, temp: &Path) -> Result<(), String> {
    let rollback = recovery_path(path);
    if !exists(&rollback)? {
        if exists(temp)? {
            // A crash before the rollback rename leaves only our temporary.
            // Discard it only after proving the canonical file remains usable.
            validate_file(path)?;
            fs::remove_file(temp)
                .map_err(|e| format!("Cannot remove interrupted save temporary file: {e}"))?;
        }
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
/// Replaces the live file, falling back to a recoverable rollback rename when needed.
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
    ensure_unchanged(path, original)?;
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
/// Filesystem-independent host regressions for save and recovery invariants.
#[path = "../../../tests/unit/live/settings_menu/save.rs"]
mod tests;
