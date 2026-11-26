mod takeover_tests {
    use super::*;
    use crate::{gameplay::ControllerOutput, native_input::GAMEPLAY_BUTTONS};

    #[test]
    fn held_counter_preempts_all_assists_without_restoring_old_owner() {
        let mut ownership = ActionOwnership::new();
        let intent = |owner| Intent {
            owner,
            present: true,
            committed: true,
        };
        assert_eq!(
            ownership.select(true, &[intent(Owner::Ledge)]),
            Some(Owner::Ledge)
        );
        assert_eq!(ownership.set_counter_hold(true), Some(Owner::Ledge));
        assert!(ownership.allows(Owner::Counter));
        assert!(!ownership.allows(Owner::Spacing));
        assert_eq!(
            ownership.select(true, &[intent(Owner::Counter)]),
            Some(Owner::Counter)
        );
        let neutral = exclusive_output(Some(Owner::Counter), true, ControllerOutput::NONE);
        assert_eq!(neutral.owned & GAMEPLAY_BUTTONS, GAMEPLAY_BUTTONS);
        assert_eq!(neutral.stick, Some([0.; 2]));
        let shield = ControllerOutput {
            requested: crate::native_input::MAPPER_ZR,
            owned: crate::ledge_trap::SEQUENCE_BUTTONS,
            stick: Some([0.; 2]),
            ..ControllerOutput::NONE
        };
        assert_eq!(
            exclusive_output(Some(Owner::Counter), true, shield),
            shield
        );
        assert_eq!(
            ownership.select(true, &[intent(Owner::Safety), intent(Owner::Counter)]),
            Some(Owner::Counter)
        );
        ownership.release(Owner::Safety);
        ownership.set_counter_hold(true);
        assert_eq!(
            ownership.select(true, &[intent(Owner::Counter)]),
            Some(Owner::Counter)
        );
        assert_eq!(
            ownership.set_counter_hold(false),
            Some(Owner::Counter)
        );
        assert_eq!(ownership.select(true, &[]), None);
        assert_eq!(
            ownership.select(true, &[intent(Owner::Spacing)]),
            Some(Owner::Spacing)
        );
    }

    #[test]
    fn detected_ledge_wait_and_queued_reaction_own_neutral_controller() {
        for owner in [Owner::Ledge, Owner::RollRead] {
            let mut ownership = ActionOwnership::new();
            let intent = Intent::sequence_receipt(owner, 0, true);
            assert_eq!(ownership.select(true, &[intent]), Some(owner));
            assert_eq!(ownership.current_owner(), Some(owner));
            let output =
                exclusive_output(ownership.current_owner(), true, ControllerOutput::default());
            assert_eq!(output.owned & GAMEPLAY_BUTTONS, GAMEPLAY_BUTTONS);
            assert_eq!(output.stick, Some([0.; 2]));
            assert_eq!(ownership.select(true, &[intent]), Some(owner));
            assert_eq!(ownership.select(true, &[]), None);
            assert_eq!(ownership.current_owner(), None);
        }
    }
}
