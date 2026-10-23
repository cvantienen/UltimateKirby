//! Only shared config.txt settings are read from SD at startup. Fighter action
//! functions, menus, move data are compiled Rust data.
use kirby_assist_core::{fighter_profiles::FighterProfiles, live_input::LiveConfig};

const CONFIG_PATH: &str = "sd:/ultimate/kirby_assist/config.txt";

pub(super) fn load_config() -> Result<(LiveConfig, FighterProfiles), String> {
    let mut config = match std::fs::read_to_string(CONFIG_PATH) {
        Ok(contents) => {
            LiveConfig::parse(&contents).map_err(|error| format!("{CONFIG_PATH}: {error}"))?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => LiveConfig::default(),
        Err(error) => return Err(format!("{CONFIG_PATH}: {error}")),
    };
    // Enable the slide engine; live routing reserves D-pad sides for the visible menu.
    config.dpad_slide = true;
    let profiles = FighterProfiles::default();
    profiles.validate()?;
    Ok((config, profiles))
}
