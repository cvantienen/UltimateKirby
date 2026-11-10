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

/// Technical: Canonical saved sequence and tuning file path.
/// Plain English: The file where user settings and recipes are stored.
const PATH: &str = "sd:/ultimate/kirby_assist/sequences.txt";
/// Technical: Exclusive temporary path used while writing a replacement.
/// Plain English: The temporary file used to prepare a save safely.
const TEMP: &str = "sd:/ultimate/kirby_assist/sequences.txt.menu.tmp";
/// Technical: Verified recovery copy retained while replacing the canonical file.
/// Plain English: A backup kept in case the save is interrupted.
const BACKUP: &str = "sd:/ultimate/kirby_assist/sequences.txt.menu.bak";
/// Technical: Stack allocation reserved for the save worker thread.
/// Plain English: Extra stack space provided to the background save worker.
const WORKER_STACK_BYTES: usize = 2 * 1024 * 1024;
/// Technical: Switch filesystem error code for an already-existing file.
/// Plain English: The platform code used when the temporary file already exists.
const RAW_OS_ERROR_ALREADY_EXISTS: i32 = 0x402;
/// Technical: Immutable request data captured for one explicit save operation.
/// Plain English: Everything the background worker needs to save one menu edit.
pub(super) struct Job {
    /// Technical: Native fighter kind associated with this request.
    /// Plain English: The fighter whose settings are being saved.
    pub kind: i32,
    /// Technical: Preset category containing the selected choice.
    /// Plain English: Which recipe slot is being saved.
    pub slot: PresetSlot,
    /// Technical: Index of the selected preset in the category.
    /// Plain English: Which choice in that slot is being changed.
    pub index: usize,
    /// Technical: Recipe revision captured when the request was created.
    /// Plain English: The menu version that requested the save.
    pub revision: u64,
    /// Technical: Stable catalog identifier used to revalidate the selected recipe.
    /// Plain English: The recipe's identity, so a later save cannot affect the wrong choice.
    pub choice: String,
    /// Technical: Optional persisted display-name override.
    /// Plain English: A custom name for this recipe, if it has one.
    pub name: Option<String>,
    /// Technical: Whether this request removes rather than updates the recipe.
    /// Plain English: Whether this save deletes the selected recipe.
    pub delete: bool,
    /// Technical: Whether a successful update should set the startup default.
    /// Plain English: Whether the saved recipe should become the default choice.
    pub make_default: bool,
    /// Technical: Whether the selected catalog entry may be created by persistence.
    /// Plain English: Whether saving may add a new recipe entry.
    pub allow_new: bool,
    /// Technical: Value snapshot of the recipe to persist.
    /// Plain English: The recipe contents captured for this save.
    pub sequence: Sequence,
    /// Technical: Optional fighter-tuning payload; present for tuning-only jobs.
    /// Plain English: Updated fighter settings, when this save is for tuning.
    pub tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
/// What a result must report when its worker cannot deliver one itself.
/// Plain English: Information used to describe a save that stopped unexpectedly.
#[derive(Clone, Copy)]
struct Identity {
    /// Technical: Fighter kind retained for a fallback failure message.
    /// Plain English: Which fighter the failed save belonged to.
    kind: i32,
    /// Technical: Preset category retained for a fallback failure message.
    /// Plain English: Which recipe slot the failed save belonged to.
    slot: PresetSlot,
    /// Technical: Selected catalog index retained for result correlation.
    /// Plain English: Which menu choice the failed save referred to.
    index: usize,
    /// Technical: Source revision retained to reject stale completion application.
    /// Plain English: Which menu version requested the failed save.
    revision: u64,
    /// Technical: Whether the request represented deletion.
    /// Plain English: Whether the failed operation was a delete.
    delete: bool,
    /// Technical: Optional tuning payload identity.
    /// Plain English: The fighter tuning associated with this operation, if any.
    tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
impl Job {
    /// Technical: Copies the fields needed to correlate an eventual completion.
    /// Plain English: Keeps a small record of the save request for its result.
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
/// Technical: Completion payload returned from the worker to the menu owner.
/// Plain English: The outcome message and saved data the menu applies after a save.
pub(super) struct ResultMessage {
    /// Technical: Stable catalog identifier of the completed recipe, if applicable.
    /// Plain English: The recipe name-key the result applies to.
    pub choice: Option<String>,
    /// Technical: Native fighter kind associated with the completed request.
    /// Plain English: Which fighter was being saved.
    pub kind: i32,
    /// Technical: Preset category associated with the completed request.
    /// Plain English: Which recipe slot was saved.
    pub slot: PresetSlot,
    /// Technical: Catalog index associated with the completed request.
    /// Plain English: Which choice in that slot was saved.
    pub index: usize,
    /// Technical: Source recipe revision associated with the completed request.
    /// Plain English: Which menu version started the save.
    pub revision: u64,
    /// Technical: User-facing completion or failure status.
    /// Plain English: The message shown after the save finishes.
    pub message: String,
    /// Technical: Whether the file operation completed successfully.
    /// Plain English: Whether the save worked.
    pub success: bool,
    /// Technical: Whether the operation deleted the selected recipe.
    /// Plain English: Whether the save removed a recipe.
    pub deleted: bool,
    /// Technical: Successful recipe snapshot used to reconcile current edits.
    /// Plain English: The recipe contents that were actually saved.
    pub snapshot: Option<(Sequence, Option<String>)>,
    /// Technical: Optional tuning payload associated with the operation.
    /// Plain English: The fighter tuning that was saved, if this was a tuning save.
    pub tuning: Option<kirby_assist_core::fighter_tuning::Tuning>,
}
impl ResultMessage {
    /// Technical: Constructs a failure completion preserving request correlation fields.
    /// Plain English: Builds an error result that still points to the original request.
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
/// Technical: Serializes explicit save requests across worker creation and result consumption.
/// Plain English: Prevents overlapping saves.
static BUSY: AtomicBool = AtomicBool::new(false);
/// Technical: Join handle and request identity retained until worker teardown.
/// Plain English: Keeps the save thread tracked until it has fully stopped.
struct Worker {
    /// Technical: Join handle used to confirm worker termination without blocking on active work.
    /// Plain English: Lets the menu check when the save thread has ended.
    handle: std::thread::JoinHandle<()>,
    /// Technical: Request identity used if the worker exits without publishing a result.
    /// Plain English: Details needed to report an unexpected save-thread exit.
    identity: Identity,
}
/// Technical: Owns the sole current save worker until its join completes.
/// Plain English: Stores the save thread that is currently running.
static WORKER: Mutex<Option<Worker>> = Mutex::new(None);
// Only explicit saves and cached-text replacement use this gate; never inputs,
// detection or drawing. Check BUSY under the gate to close the start/rebuild race.
/// Technical: Serializes explicit saves with cached-view replacement.
/// Plain English: Coordinates saves with menu text changes.
static UI_SAVE_GATE: Mutex<()> = Mutex::new(());
/// Technical: Acquires the short cached-UI edit gate only while no save is active.
/// Plain English: Lets the display be changed only when a save is not using it.
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
/// Technical: Reads the save-in-progress flag with acquire ordering.
/// Plain English: Checks whether a save is still running.
pub(super) fn busy() -> bool {
    BUSY.load(Ordering::Acquire)
}
/// Technical: Signals that a result is available in [`RESULT`].
/// Plain English: Marks that a completed save message is waiting.
static RESULT_READY: AtomicBool = AtomicBool::new(false);
/// Technical: Holds the single published completion message for the UI thread.
/// Plain English: Stores the result that the menu will read next.
static RESULT: Mutex<Option<ResultMessage>> = Mutex::new(None);
#[cfg(test)]
/// Technical: Serializes tests that access the process-wide worker state.
/// Plain English: Prevents concurrent save tests from sharing state.
pub(super) static WORKER_TEST: Mutex<()> = Mutex::new(());
/// Technical: Checks for a published result or an exited worker requiring synthesis.
/// Plain English: Checks whether the menu has a save result to process.
pub(super) fn has_result() -> bool {
    RESULT_READY.load(Ordering::Acquire)
        || busy()
            && WORKER.try_lock().is_ok_and(|worker| {
                worker
                    .as_ref()
                    .is_some_and(|worker| worker.handle.is_finished())
            })
}
/// Technical: Joins only a finished worker, consumes its result, and releases the save gate.
/// Plain English: Collects a completed save without waiting for active disk work.
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
/// Technical: Claims the exclusive save state only for an allowed Training operation.
/// Plain English: Starts a save only in Training mode and only when none is already active.
fn claim_worker(training: bool) -> Result<(), &'static str> {
    if !training {
        return Err("Saving is available only in Training Mode");
    }
    BUSY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .map(|_| ())
        .map_err(|_| "Save in progress; retry when it finishes")
}
/// Technical: Validates and starts one explicit save, returning immediate status text.
/// Plain English: Requests a save and tells the menu whether it started.
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
/// Technical: Allows tests to substitute filesystem operations without touching the SD path.
/// Plain English: Gives tests a safe fake save operation.
pub(super) static TEST_OPERATION: Mutex<Option<fn(&Job) -> Result<bool, String>>> =
    Mutex::new(None);
/// Technical: Applies the save validation and worker path with an injectable operation.
/// Plain English: Runs a save request using either real storage or a test substitute.
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
/// Technical: Replaces the result slot before publishing readiness with release ordering.
/// Plain English: Delivers the finished save message to the menu.
fn publish(result: ResultMessage) {
    let mut slot = RESULT.lock().unwrap_or_else(|error| error.into_inner());
    *slot = Some(result);
    RESULT_READY.store(true, Ordering::Release);
}
/// Technical: Maps the operation outcome and request flags to stable user-facing text.
/// Plain English: Chooses the message shown for a completed save.
fn completion_message(job: &Job, outcome: &Result<bool, String>) -> String {
    match outcome {
        Ok(false) => "Already saved; file unchanged".into(),
        Ok(true) if job.delete => "Deleted from sequences.txt (backup created)".into(),
        Ok(true) if job.make_default => "Saved as startup default (backup created)".into(),
        Ok(true) => "Saved to sequences.txt (backup created)".into(),
        Err(error) => format!("Save failed: {error}"),
    }
}
/// Technical: Starts a named, stack-sized worker and publishes one correlated result.
/// Plain English: Runs the file save on its own tracked background thread.
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

/// Technical: Publishes committed tuning into the shared in-memory sequence library.
/// Plain English: Updates the active saved-settings cache after writing tuning.
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
/// Technical: Persists a tuning or recipe job and publishes changed recipe snapshots.
/// Plain English: Saves the requested settings and updates the in-memory library.
fn write(job: &Job) -> Result<bool, String> {
    if let Some(tuning) = job.tuning {
        let changed = write_tuning_paths(
            job.kind,
            tuning,
            Path::new(PATH),
            Path::new(TEMP),
            Path::new(BACKUP),
        )?;
        if let Some(saved) = super::super::SEQUENCES.get() {
            publish_tuning(saved, job.kind, tuning);
        }
        return Ok(changed);
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
/// Recover an interrupted save, then read the live file and apply `update`.
/// Technical: Runs recovery before applying a text transformation to the current file.
/// Plain English: Repairs an interrupted save before changing the file.
fn read_updated(
    path: &Path,
    temp: &Path,
    update: impl FnOnce(&str) -> Result<String, String>,
) -> Result<(String, String), String> {
    recover_paths(path, temp)?;
    let original = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    let updated = update(&original)?;
    Ok((original, updated))
}
/// Technical: Applies tuning persistence and commits only after validating the full file.
/// Plain English: Saves fighter settings while preserving existing recipes.
fn write_tuning_paths(
    kind: i32,
    tuning: kirby_assist_core::fighter_tuning::Tuning,
    path: &Path,
    temp: &Path,
    backup: &Path,
) -> Result<bool, String> {
    // Full-file validation still runs, but its temporary catalogs are dropped
    // before disk replacement. No recipe snapshot survives/publishes this save.
    let (original, updated) = read_updated(path, temp, |original| {
        persist::update_tuning(original, kind, tuning)
    })?;
    commit_paths(path, temp, backup, &original, &updated)
}
/// Technical: Reads at most the file limit plus one detection byte.
/// Plain English: Reads the save file safely without allowing an oversized file.
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
/// Technical: Test adapter returning only the parsed catalog from a recipe write.
/// Plain English: Helper used by tests to check the saved recipe catalog.
fn write_paths(
    job: &Job,
    path: &Path,
    temp: &Path,
    backup_path: &Path,
) -> Result<kirby_assist_core::sequence_config::Catalog, String> {
    write_recipe_paths(job, path, temp, backup_path).map(|(catalog, _)| catalog)
}
/// Technical: Validates and serializes recipe edits, then commits the verified replacement.
/// Plain English: Saves recipe changes while keeping the previous file recoverable.
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
    let saved_catalog = kirby_assist_core::sequence_config::Catalog::parse(&updated)?
        .for_build(cfg!(feature = "diagnostic-logging"));
    let changed = commit_paths(path, temp, backup_path, &original, &updated)?;
    Ok((saved_catalog, changed))
}
/// Technical: Rejects a commit when the live file changed since its initial read.
/// Plain English: Prevents overwriting changes made by another tool during the save.
fn ensure_unchanged(path: &Path, original: &str) -> Result<(), String> {
    if read_bounded(path)? == original.as_bytes() {
        Ok(())
    } else {
        Err("sequences.txt changed during save; retry".into())
    }
}
/// Write through an already opened file, close the native handle, then read the
/// bytes back before any rename or later step relies on them.
/// Technical: Flushes, closes, and verifies a complete file before it can be renamed.
/// Plain English: Confirms the saved bytes are really on disk before continuing.
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
/// Technical: Coordinates backup, temporary-file verification, and rollback-safe replacement.
/// Plain English: Replaces the settings file while keeping the previous version recoverable.
fn commit_paths(
    path: &Path,
    temp: &Path,
    backup_path: &Path,
    original: &str,
    updated: &str,
) -> Result<bool, String> {
    if updated == original {
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

/// Technical: Derives the rollback-marker path associated with a canonical save file.
/// Plain English: Finds the backup name used to recover an interrupted replacement.
fn recovery_path(path: &Path) -> std::path::PathBuf {
    path.with_file_name(format!(
        "{}.menu.rollback",
        path.file_name().unwrap_or_default().to_string_lossy()
    ))
}
/// Technical: Checks path existence while distinguishing absence from filesystem errors.
/// Plain English: Checks whether a file exists and reports other inspection failures.
fn exists(path: &Path) -> Result<bool, String> {
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Cannot inspect {}: {e}", path.display())),
    }
}
/// Technical: Bounded-reads and validates persisted sequence syntax.
/// Plain English: Checks that a save file is small enough and has valid contents.
fn validate_file(path: &Path) -> Result<(), String> {
    let source = String::from_utf8(read_bounded(path)?).map_err(|e| e.to_string())?;
    persist::validate_output(&source)
}
/// Startup/next-save recovery is inactive unless an interrupted replacement
/// left our rollback marker. A valid published file always wins over the old one.
/// Technical: Restores or cleans an interrupted replacement before normal use.
/// Plain English: Finishes recovery after a save was interrupted.
pub(super) fn recover() -> Result<(), String> {
    recover_paths(Path::new(PATH), Path::new(TEMP))
}
/// Technical: Reconciles a rollback marker with the live file and removes stale temporary data.
/// Plain English: Keeps a valid saved file and restores the original if needed.
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
/// Technical: Replaces the live file, falling back to a recoverable rollback rename when needed.
/// Plain English: Publishes the new file without losing the previous good copy.
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
/// Technical: Filesystem-independent host regressions for save and recovery invariants.
/// Plain English: Tests that saves, backups, and recovery behave correctly.
mod tests {
    use super::*;
    use kirby_assist_core::sequence_config::Catalog;
    /// Technical: Unique directory suffix counter for isolated test fixtures.
    /// Plain English: Gives each test folder a different name.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    /// Technical: Minimal multi-fighter catalog fixture used by persistence tests.
    /// Plain English: Sample saved recipes used to test file changes.
    const SOURCE: &str = "# custom catalog\n[yoshi.roll.grab]\nname = Tongue grab\nstart_delay=0\ndefault=true\nactions=grab\n\n[kirby.roll.other]\nstart_delay=0\nactions=a\n";
    /// Technical: RAII-owned temporary directory for one persistence regression.
    /// Plain English: Test files are removed when this fixture goes away.
    struct Directory(std::path::PathBuf);
    impl Directory {
        /// Technical: Creates a unique test directory and returns its owner.
        /// Plain English: Makes a fresh folder for a save test.
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
        /// Technical: Removes the fixture directory and all of its test files.
        /// Plain English: Cleans up the test folder after each test.
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    /// Technical: Builds a representative recipe-save request fixture.
    /// Plain English: Makes a sample save request for tests.
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
    /// Technical: Verifies two fighter-specific edits preserve each other's catalog sections.
    /// Plain English: Checks that saving one form does not overwrite the other.
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
    /// Technical: Verifies initial save proceeds while a concurrent menu refresh remains nonblocking.
    /// Plain English: Checks that the first save succeeds without freezing menu updates.
    fn first_save_survives_a_concurrent_menu_refresh_without_blocking_input() {
        /// Technical: Counts injected writes in the concurrent refresh regression.
        /// Plain English: Tracks how many times the test save operation ran.
        static WRITES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        /// Technical: Test-only save operation that increments [`WRITES`].
        /// Plain English: Fake save action used to check request counts.
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
    /// Technical: Verifies completed workers are joined before UI edits and subsequent saves proceed.
    /// Plain English: Confirms the menu waits for a save thread to fully finish.
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
            identity: Job {
                kind: 5,
                slot: PresetSlot::Roll,
                index: 0,
                revision: 0,
                ..job()
            }
            .identity(),
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
    /// Technical: Verifies save admission requires Training and rejects overlapping work.
    /// Plain English: Checks saves only run in Training and do not pile up.
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
    /// Technical: Verifies default updates publish the latest catalog without changing other categories.
    /// Plain English: Checks that choosing a default keeps unrelated recipes.
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
    /// Technical: Verifies success and failure workers exit, publish outcomes, and release admission.
    /// Plain English: Checks another save can start after either a successful or failed save.
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
    /// Technical: Verifies oversized input is rejected before replacement starts.
    /// Plain English: Checks a too-large file is left untouched.
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
    /// Technical: Verifies Switch rename fallback preserves the original and validates publication.
    /// Plain English: Checks the backup remains safe if replacing a file needs a fallback.
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
    /// Technical: Verifies failed publication rolls back to the original live file.
    /// Plain English: Checks a failed save does not lose the old settings.
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
    /// Technical: Verifies startup recovery handles a missing live file and a valid published file.
    /// Plain English: Checks recovery finishes interrupted saves without discarding valid data.
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
    /// Technical: Verifies invalid live content prevents recovery from overwriting retained originals.
    /// Plain English: Checks recovery preserves the backup when the current file is broken.
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
    /// Technical: Verifies diagnostic rename and delete operations validate and back up the catalog.
    /// Plain English: Checks rename and delete changes are saved safely.
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
    /// Technical: Verifies a recipe save produces a valid catalog and verified backup.
    /// Plain English: Checks a saved recipe works and the previous file is backed up.
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
    /// Technical: Verifies adding a first non-default Y recipe remains valid after reload.
    /// Plain English: Checks a new Y recipe can be reopened later.
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
    /// Technical: Verifies invalid catalogs and pre-existing temporary files block replacement.
    /// Plain English: Checks broken data or an occupied temp file cannot replace the save file.
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
    /// Technical: Verifies backup creation failure leaves the live file and cleans owned temporary data.
    /// Plain English: Checks a failed backup does not damage the current settings.
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
    /// Technical: Verifies tuning updates use the verified-save path without dropping recipes.
    /// Plain English: Checks fighter settings can be saved while recipes remain intact.
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
    /// Technical: Verifies publishing tuning preserves compact recipes and other fighter entries.
    /// Plain English: Checks updating one fighter's settings keeps everyone else's data.
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
    /// Technical: Verifies identical serialized output skips replacement for each save type.
    /// Plain English: Checks unchanged settings do not rewrite files or backups.
    fn commit_skips_identical_content_for_every_save_kind_and_keeps_the_backup() {
        let directory = Directory::new();
        let path = directory.0.join("sequences.txt");
        let temp = directory.0.join("temp");
        let backup = directory.0.join("backup");
        fs::write(&path, SOURCE).unwrap();
        fs::write(&backup, "previous backup must stay").unwrap();
        assert!(!commit_paths(&path, &temp, &backup, SOURCE, SOURCE).unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            "previous backup must stay"
        );
        assert!(!temp.exists());
        let updated = format!("{SOURCE}\n# edited\n");
        assert!(commit_paths(&path, &temp, &backup, SOURCE, &updated).unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), updated);
        assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
    }
    #[test]
    /// Technical: Verifies failed tuning backup creation preserves the active file.
    /// Plain English: Checks fighter settings remain safe if their backup cannot be made.
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
    /// Technical: Verifies an unchanged recipe does not replace the file or backup.
    /// Plain English: Checks saving the same recipe does not rewrite anything.
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
    /// Technical: Verifies long holds and until-takeover actions persist without expansion.
    /// Plain English: Checks advanced button timing stays compact when saved.
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
    /// Technical: Verifies a worker exit without publication synthesizes failure and releases the gate.
    /// Plain English: Checks an unexpectedly stopped save does not leave saving permanently locked.
    fn exited_worker_without_a_result_releases_save_gate_and_reports_failure() {
        let _guard = WORKER_TEST.lock().unwrap();
        let mut settings = kirby_assist_core::fighter_tuning::Tuning::default();
        settings.set("grab_range", "14").unwrap();
        // A recipe job and a fighter-settings job must each report as themselves,
        // so completion routing does not mistake a settings failure for a recipe.
        for (tuning, delete) in [(None, false), (None, true), (Some(settings), false)] {
            claim_worker(true).unwrap();
            let handle = std::thread::spawn(|| {});
            while !handle.is_finished() {
                std::thread::yield_now();
            }
            *WORKER.lock().unwrap() = Some(Worker {
                handle,
                identity: Job {
                    kind: 5,
                    slot: PresetSlot::Roll,
                    index: 9,
                    revision: 123,
                    delete,
                    tuning,
                    ..job()
                }
                .identity(),
            });
            assert!(has_result());
            let result = poll().unwrap();
            assert!(!result.success);
            assert_eq!((result.kind, result.index, result.revision), (5, 9, 123));
            assert_eq!(result.deleted, delete);
            assert_eq!(result.tuning, tuning);
            assert!(result.message.contains("not confirmed"));
            assert!(!busy());
            assert!(try_ui_edit().is_some());
        }
    }
    #[test]
    /// Technical: Verifies malformed snapshots are rejected before worker creation.
    /// Plain English: Checks invalid recipe data never starts a save.
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
    /// Technical: Verifies boundary values and invalid axes preserve the existing catalog.
    /// Plain English: Checks edge-case values cannot damage saved recipes.
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
    /// Technical: Verifies a full-roster boundary save fits the configured worker stack.
    /// Plain English: Checks the largest supported catalog can be saved by the worker.
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
    /// Technical: Verifies full-roster edit, default, copy, delete, and settings round trips.
    /// Plain English: Checks all major menu save actions survive reload.
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
    /// Technical: Verifies a default recipe save captures the latest actions, timing, name, and loop.
    /// Plain English: Checks the default save includes the player's newest edits.
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
