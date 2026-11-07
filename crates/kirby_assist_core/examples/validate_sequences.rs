//! Host-only startup validation for the exact bytes about to be uploaded.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    sequence_config::{Catalog, MAX_FILE_BYTES},
};
use std::io::{self, Read};

fn validate() -> Result<(usize, usize), String> {
    let mut bytes = Vec::new();
    io::stdin()
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("sequences.txt exceeds the startup size limit".into());
    }
    let source = std::str::from_utf8(&bytes).map_err(|e| format!("invalid UTF-8: {e}"))?;
    let catalog = Catalog::parse(source)?;
    // Match startup validation, including menu capacity after test filtering.
    for diagnostic in [false, true] {
        let mut profiles = FighterProfiles::default();
        profiles.validate()?;
        profiles.add_sequence_choices(&catalog.clone().for_build(diagnostic))?;
    }
    Ok((catalog.entries().len(), catalog.tuning.len()))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "--list-options") {
        let Some(path) = args.get(1) else {
            eprintln!("usage: --list-options PATH [FIGHTER]");
            std::process::exit(1);
        };
        let result = std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|source| Catalog::parse(&source));
        let catalog = match result {
            Ok(catalog) => catalog,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        };
        for entry in catalog.entries() {
            if args.get(2).is_some_and(|fighter| {
                entry.fighter != "universal" && !entry.fighter.eq_ignore_ascii_case(fighter)
            }) {
                continue;
            }
            println!(
                "{} {:?}.{}: start_delay={} {}",
                entry.fighter,
                entry.slot,
                entry.choice,
                entry.sequence.start_delay,
                entry.name.as_deref().unwrap_or(&entry.choice)
            );
        }
        return;
    }
    match validate() {
        Ok((count, settings)) => {
            println!("Validated {count} sequence sections and {settings} fighter settings sections for silent and diagnostic builds.")
        }
        Err(error) => {
            eprintln!("Invalid sequences.txt: {error}");
            std::process::exit(1);
        }
    }
}
