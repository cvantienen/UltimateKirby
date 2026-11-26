mod tests {
    use super::*;

    /// Regression test for `specific_scenario_beats_broad_scenario`.
    #[test]
    fn specific_scenario_beats_broad_scenario() {
        let observation = FighterObservation {
            active: [
                ScenarioSample::new(Scenario::Grounded, 12),
                ScenarioSample::new(Scenario::Tumble, 3),
                ScenarioSample::EMPTY,
            ],
        };

        assert_eq!(
            observation.highest_priority(),
            Some(ScenarioSample::new(Scenario::Tumble, 3))
        );
    }
}
