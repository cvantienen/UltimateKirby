//! Startup-only SD configuration. sequences.txt is the sole ledge option catalog;
//! a missing file leaves empty menus with no compiled response fallback.
//! Technical: Loads and validates startup configuration and the SD-card sequence
//! catalog before the live hooks are installed.
//! Plain English: This module reads the user's settings and sequence choices at startup.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    live_input::LiveConfig,
    sequence_config::{Catalog, SequenceLibrary, MAX_FILE_BYTES},
};

/// SD-card path for the legacy/shared live settings file.
///
///  Where the plugin looks for the user's main configuration.
const CONFIG_PATH: &str = "sd:/ultimate/kirby_assist/config.txt";
/// SD-card path for the optional sequence catalog.
///
///  Where the plugin looks for user-defined sequences.
const SEQUENCES_PATH: &str = "sd:/ultimate/kirby_assist/sequences.txt";
/// Type alias for the loaded configuration, including the live config, fighter profiles, and sequence library.
/// Groups the validated settings, fighter profile catalog, and sequence library.
///
///  This is the complete set of configuration data the plugin needs.
type LoadedConfig = (Box<LiveConfig>, FighterProfiles, SequenceLibrary);

// Full saved catalogs can exceed the native loader's 64 KiB stack when nested
// config/parser/allocator frames overlap. Join this startup-only worker before
// installing any hooks. Its stack and thread are released after loading.
/// Loads the configuration from the SD card, including the live config, fighter profiles, and sequence library.
/// Returns an error if the configuration cannot be loaded.
/// Runs the loader on a dedicated, enlarged stack and joins it before returning.
///
///  It safely loads startup files using extra stack space, then returns the result.
#[inline(never)]
pub(super) fn load_config() -> Result<LoadedConfig, String> {
    std::thread::Builder::new()
        .name("kirby-config-load".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(load_on_worker)
        .map_err(|error| format!("Cannot start config loader: {error}"))?
        .join()
        .map_err(|_| "Config loader failed; no hooks installed".to_owned())?
}

/// Returning the large Copy config inline would retain config-sized slots on
/// the caller's small loader stack. This box is freed after publication.
/// Performs file recovery, catalog parsing, configuration parsing,
/// and runtime-profile preparation on the loader worker.
///
///  The worker reads the files and packages their validated settings.
#[inline(never)]
fn load_on_worker() -> Result<LoadedConfig, String> {
    super::settings_menu::recover_saved_file()?;
    let sequences = match std::fs::metadata(SEQUENCES_PATH) {
        Ok(metadata) if metadata.len() > MAX_FILE_BYTES => {
            return Err(format!(
                "{SEQUENCES_PATH}: file exceeds {MAX_FILE_BYTES} bytes"
            ))
        }
        Ok(_) => {
            let source = std::fs::read_to_string(SEQUENCES_PATH)
                .map_err(|error| format!("{SEQUENCES_PATH}: {error}"))?;
            Catalog::parse(&source).map_err(|error| format!("{SEQUENCES_PATH}: {error}"))?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Catalog::default(),
        Err(error) => return Err(format!("{SEQUENCES_PATH}: {error}")),
    };
    let config = if let Some(settings) = sequences.universal.as_deref() {
        LiveConfig::parse(settings).map_err(|error| format!("{SEQUENCES_PATH}: {error}"))?
    } else {
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(contents) => {
                LiveConfig::parse(&contents).map_err(|error| format!("{CONFIG_PATH}: {error}"))?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => LiveConfig::default(),
            Err(error) => return Err(format!("{CONFIG_PATH}: {error}")),
        }
    };
    prepare_runtime(config, sequences)
}

/// Prepares the runtime configuration, including fighter profiles and sequence library.
/// Returns the loaded configuration or an error if preparation fails.
/// Validates the profiles and build-filtered catalog, attaches shared
/// templates, and boxes the live configuration.
///
///  It checks that the loaded choices are usable and prepares them for play.
/// ### Arguments
/// - `config` (`LiveConfig`): Represents the configuration being read or changed; the function uses it as input to the operation described above.
/// - `sequences` (`Catalog`): Represents the sequences; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
fn prepare_runtime(config: LiveConfig, sequences: Catalog) -> Result<LoadedConfig, String> {
    let mut profiles = FighterProfiles::default();
    profiles.validate()?;
    let sequences = sequences.for_build(cfg!(feature = "diagnostic-logging"));
    sequences.validate(&profiles)?;
    profiles.add_sequence_templates(&sequences.templates);
    // Keep dormant definitions compact. Only the selected fighter is expanded
    // by its session menu; no whole-roster static menu/timeline publication.
    Ok((
        Box::new(config),
        profiles,
        SequenceLibrary::from_catalog(sequences),
    ))
}

#[cfg(test)]
/// Host-side startup configuration regression tests.
///
///  Checks that configuration preparation preserves compact templates.
mod tests {
    use super::*;
    /// Verifies shared templates remain shared and unrelated menus stay unexpanded.
    ///
    ///  Guards against startup building full menus for every character.
    #[test]
    fn startup_keeps_shared_templates_but_never_expands_other_fighter_menus() {
        let catalog=Catalog::parse("[template.defend]\nstart_delay=0\nactions=shield:4\n[bowser.roll.spin]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[lucina.neutral.poke]\nstart_delay=23\nactions=forward_tilt:1\n").unwrap();
        let (_, profiles, library) = prepare_runtime(LiveConfig::default(), catalog).unwrap();
        let mut count = 0;
        for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
            let Some(profile) = profiles.profile(kind) else {
                continue;
            };
            count += 1;
            assert!(profile
                .menu_options
                .lists
                .iter()
                .all(|list| list.options.is_empty()));
            assert_eq!(profile.menu_options.templates.len(), 1);
        }
        assert_eq!(count, 79);
        assert_eq!(library.fighter_entries(15), 1);
        assert_eq!(library.fighter_entries(22), 1);
        assert_eq!(library.for_fighter(5).len(), 0);
        assert!(std::ptr::eq(
            profiles
                .profile(15)
                .unwrap()
                .menu_options
                .templates
                .as_ptr(),
            profiles
                .profile(22)
                .unwrap()
                .menu_options
                .templates
                .as_ptr()
        ));
    }
}
