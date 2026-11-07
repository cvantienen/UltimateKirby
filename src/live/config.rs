//! Startup-only SD configuration. sequences.txt is the sole ledge option catalog;
//! a missing file leaves empty menus with no compiled response fallback.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    live_input::LiveConfig,
    sequence_config::{Catalog, SequenceLibrary, MAX_FILE_BYTES},
};

const CONFIG_PATH: &str = "sd:/ultimate/kirby_assist/config.txt";
const SEQUENCES_PATH: &str = "sd:/ultimate/kirby_assist/sequences.txt";

type LoadedConfig = (Box<LiveConfig>, FighterProfiles, SequenceLibrary);

// Full saved catalogs can exceed the native loader's 64 KiB stack when nested
// config/parser/allocator frames overlap. Join this startup-only worker before
// installing any hooks. Its stack and thread are released after loading.
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

// Returning the large Copy config inline would retain config-sized slots on
// the caller's small loader stack. This box is freed after publication.
#[inline(never)]
fn load_on_worker() -> Result<LoadedConfig, String> {
    super::ledge_menu::recover_saved_file()?;
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

fn prepare_runtime(mut config: LiveConfig, sequences: Catalog) -> Result<LoadedConfig, String> {
    config.dpad_b_reverse = false;
    config.auto_ledge_trump = false;
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
mod tests {
    use super::*;
    #[test]
    fn startup_keeps_shared_templates_but_never_expands_other_fighter_menus() {
        let catalog=Catalog::parse("[template.defend]\nstart_delay=0\nactions=shield:4\n[bowser.roll.spin]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[lucina.neutral.poke]\nstart_delay=23\nactions=forward_tilt:1\n").unwrap();
        let (_, profiles, library) = prepare_runtime(LiveConfig::default(), catalog).unwrap();
        let mut count = 0;
        for kind in 0..0x5e {
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
