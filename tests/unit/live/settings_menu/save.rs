use super::*;
use kirby_assist_core::sequence_config::Catalog;
/// Unique directory suffix counter for isolated test fixtures.
static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Minimal multi-fighter catalog fixture used by persistence tests.
const SOURCE: &str = "# custom catalog\n[yoshi.roll.grab]\nname = Tongue grab\nstart_delay=0\ndefault=true\nactions=grab\n\n[kirby.roll.other]\nstart_delay=0\nactions=a\n";
/// RAII-owned temporary directory for one persistence regression.
struct Directory(std::path::PathBuf);
impl Directory {
    /// Creates a unique test directory and returns its owner.
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
    /// Removes the fixture directory and all of its test files.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Builds a representative recipe-save request fixture.
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
/// Verifies two fighter-specific edits preserve each other's catalog sections.
fn pyra_and_mythra_edits_save_once_without_overwriting_the_other_form() {
    let source = "[pyra.neutral.custom]\nname=Pyra original\nstart_delay=0\nactions=wait:2\n\n[mythra.neutral.custom]\nname=Mythra original\nstart_delay=0\nactions=wait:4\n".to_owned();
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
/// Verifies initial save proceeds while a concurrent menu refresh remains nonblocking.
fn first_save_survives_a_concurrent_menu_refresh_without_blocking_input() {
    /// Counts injected writes in the concurrent refresh regression.
    static WRITES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    /// Test-only save operation that increments [`WRITES`].
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
/// Verifies completed workers are joined before UI edits and subsequent saves proceed.
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
/// Verifies save admission requires Training and rejects overlapping work.
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
/// Verifies default updates publish the latest catalog without changing other categories.
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
                |e| e.fighter_kind == kirby_assist_core::fighter_kinds::YOSHI && e.choice == "grab"
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
/// Verifies success and failure workers exit, publish outcomes, and release admission.
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
        assert!(result.message.contains("Saved") || result.message.contains("test I/O failure"));
        assert!(!BUSY.load(Ordering::Acquire));
    }
}
#[test]
/// Verifies oversized input is rejected before replacement starts.
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
/// Verifies Switch rename fallback preserves the original and validates publication.
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
/// Verifies failed publication rolls back to the original live file.
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
/// Verifies startup recovery handles a missing live file and a valid published file.
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
fn orphan_temporary_recovery_preserves_valid_live_file_and_allows_next_save() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    let temp = directory.0.join("temp");
    let backup = directory.0.join("backup");
    fs::write(&path, SOURCE).unwrap();
    fs::write(&temp, "partial draft").unwrap();
    recover_paths(&path, &temp).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
    assert!(!temp.exists());
    assert!(write_recipe_paths(&job(), &path, &temp, &backup).unwrap().1);
    assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
}

#[test]
fn orphan_temporary_is_retained_when_live_file_is_missing_or_invalid() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    let temp = directory.0.join("temp");
    fs::write(&temp, "partial draft").unwrap();
    assert!(recover_paths(&path, &temp).is_err());
    assert_eq!(fs::read_to_string(&temp).unwrap(), "partial draft");
    fs::write(&path, "invalid external edit").unwrap();
    assert!(recover_paths(&path, &temp).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "invalid external edit");
    assert_eq!(fs::read_to_string(&temp).unwrap(), "partial draft");
}

#[test]
/// Verifies invalid live content prevents recovery from overwriting retained originals.
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
/// Verifies diagnostic rename and delete operations validate and back up the catalog.
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
/// Verifies a recipe save produces a valid catalog and verified backup.
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
/// Verifies adding a first non-default Y recipe remains valid after reload.
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
/// Invalid catalogs stay untouched; verified orphan recovery permits a fresh save.
fn invalid_file_is_preserved_and_orphan_temp_allows_a_verified_save() {
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
    write_paths(&job(), &path, &temp, &backup).unwrap();
    validate_file(&path).unwrap();
    assert!(!temp.exists());
    assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
}
#[test]
/// Verifies backup creation failure leaves the live file and cleans owned temporary data.
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
/// Verifies tuning updates use the verified-save path without dropping recipes.
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
    tuning.set("auto_di_default", "false").unwrap();
    tuning.set("auto_sdi_default", "true").unwrap();
    let (saved, changed) = write_tuning_paths(job.kind, tuning, &path, &temp, &backup).unwrap();
    assert!(changed);
    assert_eq!(saved.tuning[&5], tuning);
    assert_eq!(saved.entries(), Catalog::parse(SOURCE).unwrap().entries());
    assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
    assert!(fs::read_to_string(&path).unwrap().starts_with(SOURCE));
    assert!(!temp.exists());
}

#[test]
fn unchanged_settings_save_reloads_disk_catalog_instead_of_stale_runtime_data() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    let temp = directory.0.join("temp");
    let backup = directory.0.join("backup");
    let mut tuning = kirby_assist_core::fighter_tuning::Tuning::default();
    tuning.set("stand_distance", "20").unwrap();
    let source = persist::update_tuning(SOURCE, 5, tuning).unwrap();
    fs::write(&path, &source).unwrap();
    // Apply any legacy mode migration before exercising an unchanged save.
    write_tuning_paths(5, tuning, &path, &temp, &backup).unwrap();
    fs::write(&backup, "previous backup").unwrap();
    let saved = Mutex::new(
        kirby_assist_core::sequence_config::SequenceLibrary::from_catalog(Catalog::default()),
    );
    let (catalog, changed) = write_tuning_paths(5, tuning, &path, &temp, &backup).unwrap();
    assert!(!changed);
    drop(publish_catalog(&saved, catalog));
    let after = saved.lock().unwrap();
    assert_eq!(after.tuning[&5], tuning);
    let mut entries = after.for_fighter(5).entries().to_vec();
    entries.extend_from_slice(after.for_fighter(6).entries());
    assert_eq!(entries, Catalog::parse(SOURCE).unwrap().entries());
    assert_eq!(fs::read_to_string(&backup).unwrap(), "previous backup");
    assert!(!temp.exists());
}

#[cfg(feature = "diagnostic-logging")]
#[test]
fn lan_save_checks_revision_permission_and_preserves_universal_settings() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    let temp = directory.0.join("temp");
    let backup = directory.0.join("backup");
    fs::write(&path, SOURCE).unwrap();
    let revision = kirby_assist_core::diagnostic_http::revision(SOURCE.as_bytes());
    let updated = SOURCE.replace("Tongue grab", "LAN saved grab");
    assert!(lan_commit_paths(updated.as_bytes(), "stale", &path, &temp, &backup, || true).is_err());
    assert!(
        lan_commit_paths(updated.as_bytes(), &revision, &path, &temp, &backup, || {
            false
        })
        .is_err()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), SOURCE);
    let global = format!("{SOURCE}\n[universal.settings]\nauto_sdi=false\n");
    assert!(
        lan_commit_paths(global.as_bytes(), &revision, &path, &temp, &backup, || true).is_err()
    );
    let saved = lan_commit_paths(updated.as_bytes(), &revision, &path, &temp, &backup, || {
        true
    })
    .unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), updated);
    assert_eq!(fs::read_to_string(&backup).unwrap(), SOURCE);
    assert_eq!(saved.entries(), Catalog::parse(&updated).unwrap().entries());
    assert!(
        lan_commit_paths(SOURCE.as_bytes(), &revision, &path, &temp, &backup, || true).is_err()
    );
}

#[test]
fn reload_rejects_changed_or_missing_canonical_file_without_publishing() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    fs::write(&path, format!("{SOURCE}\n# external change\n")).unwrap();
    assert!(reload_saved_catalog(&path, SOURCE)
        .unwrap_err()
        .contains("changed before reload"));
    fs::remove_file(&path).unwrap();
    assert!(reload_saved_catalog(&path, SOURCE)
        .unwrap_err()
        .contains("Save reload failed"));
}
#[test]
/// Verifies identical serialized output skips replacement for each save type.
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
fn unchanged_save_rejects_concurrent_edits_without_touching_files() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    let temp = directory.0.join("temp");
    let backup = directory.0.join("backup");
    let external = format!("{SOURCE}\n# external edit\n");
    fs::write(&path, &external).unwrap();
    fs::write(&backup, "previous backup").unwrap();
    assert!(commit_paths(&path, &temp, &backup, SOURCE, SOURCE)
        .unwrap_err()
        .contains("changed during save"));
    assert_eq!(fs::read_to_string(&path).unwrap(), external);
    assert_eq!(fs::read_to_string(&backup).unwrap(), "previous backup");
    assert!(!temp.exists());
}

#[test]
/// Verifies failed tuning backup creation preserves the active file.
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
/// Verifies an unchanged recipe does not replace the file or backup.
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
/// Verifies long holds and until-takeover actions persist without expansion.
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
/// Verifies a worker exit without publication synthesizes failure and releases the gate.
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
/// Verifies malformed snapshots are rejected before worker creation.
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
/// Verifies boundary values and invalid axes preserve the existing catalog.
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
/// Verifies a full-roster boundary save fits the configured worker stack.
fn full_catalog_boundary_save_fits_the_actual_worker_stack() {
    std::thread::Builder::new().stack_size(WORKER_STACK_BYTES).spawn(|| {
        let source=include_str!("../../../../deployment/ultimate/kirby_assist/assembled/sequences.txt");
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
/// Verifies full-roster edit, default, copy, delete, and settings round trips.
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
                    .find(|e| e.fighter_kind == j.kind && e.slot == j.slot && e.choice == j.choice)
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
/// Verifies a default recipe save captures the latest actions, timing, name, and loop.
fn button_default_save_keeps_latest_edited_actions_timing_name_and_loop() {
    let directory = Directory::new();
    let path = directory.0.join("sequences.txt");
    let temp = directory.0.join("temp");
    let backup = directory.0.join("backup");
    let source = format!("{SOURCE}\n[yoshi.y.old]\nstart_delay=0\ndefault=true\nactions=jab:4\n");
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
