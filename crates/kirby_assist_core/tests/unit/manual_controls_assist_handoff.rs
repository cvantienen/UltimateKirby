mod tests {
    use super::*;
    use crate::native_input::{MAPPER_ZL, MAPPER_ZR};
    /// Checks whether the required input is currently held.
    fn held(mask: u32) -> ButtonTrace {
        ButtonTrace {
            held: mask,
            previous: mask,
            ..Default::default()
        }
    }
    /// Regression test for `old_shield_hold_does_not_return_as_a_roll_or_air_dodge`.
    #[test]
    fn old_shield_hold_does_not_return_as_a_roll_or_air_dodge() {
        let mut h = AssistGuardHandoff::new();
        let g = MAPPER_ZR | MAPPER_ZL;
        assert_eq!(h.sample(true, held(g), g, g, 0), g);
        for _ in 0..8 {
            assert_eq!(h.sample(false, held(g), g, 0, 0), g);
        }
        assert_eq!(h.sample(false, held(MAPPER_ZR), g, 0, 0), MAPPER_ZR);
        assert_eq!(h.sample(false, ButtonTrace::default(), g, 0, 0), 0);
        assert_eq!(
            h.sample(
                false,
                ButtonTrace {
                    held: g,
                    pressed: g,
                    ..Default::default()
                },
                g,
                0,
                0
            ),
            0
        );
    }
    /// Regression test for `requested_defense_and_fresh_manual_shield_are_not_suppressed`.
    #[test]
    fn requested_defense_and_fresh_manual_shield_are_not_suppressed() {
        let mut h = AssistGuardHandoff::new();
        let g = MAPPER_ZR;
        assert_eq!(h.sample(true, held(g), g, g, g), 0);
        h.sample(false, ButtonTrace::default(), g, 0, 0);
        assert_eq!(
            h.sample(
                true,
                ButtonTrace {
                    held: g,
                    pressed: g,
                    ..Default::default()
                },
                g,
                g,
                0
            ),
            0
        );
        assert_eq!(h.sample(false, held(g), g, 0, 0), 0);
    }
}
