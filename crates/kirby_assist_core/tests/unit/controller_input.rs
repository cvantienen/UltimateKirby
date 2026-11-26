mod tests {
    use super::*;

    /// `PLAYER` is a configuration value used by the Smash runtime.
    const PLAYER: ControllerFrame = ControllerFrame {
        buttons: ButtonSet::JUMP,
        left_stick: StickPosition {
            x: 12_000,
            y: -4_000,
        },
        right_stick: StickPosition {
            x: -8_000,
            y: 10_000,
        },
        left_trigger: 33,
        right_trigger: 91,
    };

    /// Regression test for `buttons_are_added_without_taking_movement`.
    #[test]
    fn buttons_are_added_without_taking_movement() {
        let result = arbitrate(
            PLAYER,
            InputRecipe::buttons(ButtonSet::ATTACK),
            InputPermissions::default(),
        );

        assert!(result.frame.buttons.contains(ButtonSet::JUMP));
        assert!(result.frame.buttons.contains(ButtonSet::ATTACK));
        assert_eq!(result.frame.left_stick, PLAYER.left_stick);
        assert_eq!(result.frame.right_stick, PLAYER.right_stick);
        assert_eq!(result.frame.left_trigger, PLAYER.left_trigger);
        assert_eq!(result.frame.right_trigger, PLAYER.right_trigger);
    }

    /// Regression test for `stick_request_requires_explicit_permission`.
    #[test]
    fn stick_request_requires_explicit_permission() {
        let recipe = InputRecipe::buttons_with_left_stick(ButtonSet::SHIELD, StickPosition::LEFT);

        let blocked = arbitrate(PLAYER, recipe, InputPermissions::default());
        assert!(blocked.blocked_left_stick_request);
        assert_eq!(blocked.frame.left_stick, PLAYER.left_stick);

        let allowed = arbitrate(
            PLAYER,
            recipe,
            InputPermissions {
                allow_left_stick_override: true,
            },
        );
        assert!(!allowed.blocked_left_stick_request);
        assert_eq!(allowed.frame.left_stick, StickPosition::LEFT);
    }
}
