//! Run with `cargo run --example auto_controls` from the core crate.
//! This desktop example prints logical decisions; it neither loads Skyline nor
//! sends controller input. Change the presets/timing here to learn the engine
//! before wiring a new tool into the real observer and native output layer.
use kirby_assist_core::{
    AssistConfig, AssistEngine, AutoMash, AutoTech, ControllerFrame, FighterKind,
    FighterObservation, InputPermissions, RunContext, Scenario,
};

fn main() {
    // Struct update syntax fills unspecified fields using their defaults.
    let mut config = AssistConfig {
        enabled: true,
        ..AssistConfig::default()
    };
    // rule() converts a convenient preset to a ScenarioRule. Registering a
    // rule isn't detection: observations below explicitly say when it is active.
    assert!(config.set_rule(AutoTech::default().rule()));
    assert!(config.set_rule(AutoMash::default().rule()));
    let context = RunContext {
        is_locally_human_controlled: true,
        fighter: FighterKind::Kirby,
    };
    // Fabricate seven updates of each scenario; the real adapter supplies facts
    // and maintains counters instead of iterating over made-up situations.
    for scenario in [Scenario::Tech, Scenario::Clatter] {
        for frame in 0..7 {
            let decision = AssistEngine.evaluate(
                config,
                context,
                FighterObservation::one(scenario, frame),
                ControllerFrame::default(),
                InputPermissions::default(),
            );
            println!(
                "{scenario:?} frame {frame}: {:?} {:?}",
                decision.status, decision.proposed.buttons
            );
        }
    }
}
