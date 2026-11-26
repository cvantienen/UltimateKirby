mod tests {
    use super::*;
    use crate::native_states::{situation, status};

    /// Regression test for `receipt_preserves_startup_wait_and_cancellation_gates`.
    #[test]
    fn receipt_preserves_startup_wait_and_cancellation_gates() {
        let context = GameplayContext {
            status: status::WAIT,
            situation: situation::GROUND,
            can_act: true,
            ..Default::default()
        };
        let mut receipt = UpBReceipt::new();
        let mut delivery = DeliveryBudget::new();
        assert_eq!(
            evaluate(false, false, false, false, context, &receipt, &delivery),
            ReceiptAction::Continue
        );
        receipt.mapped(0);
        assert_eq!(
            evaluate(false, false, false, false, context, &receipt, &delivery),
            ReceiptAction::Continue
        );
        assert_eq!(
            evaluate(false, false, true, false, context, &receipt, &delivery),
            ReceiptAction::Wait
        );
        assert_eq!(
            evaluate(false, true, true, false, context, &receipt, &delivery),
            ReceiptAction::Confirm
        );
        assert_eq!(
            evaluate(false, true, false, false, context, &receipt, &delivery),
            ReceiptAction::Confirm
        );
        assert_eq!(
            evaluate(true, false, true, false, context, &receipt, &delivery),
            ReceiptAction::Continue
        );
        let stopped = GameplayContext {
            stopped: true,
            ..context
        };
        assert_eq!(
            evaluate(false, false, false, false, stopped, &receipt, &delivery),
            ReceiptAction::Wait
        );
        let airborne = GameplayContext {
            situation: situation::AIR,
            ..context
        };
        assert_eq!(
            evaluate(false, false, false, false, airborne, &receipt, &delivery),
            ReceiptAction::Cancel
        );
        for tick in 0..24 {
            delivery.mapped(tick);
        }
        assert_eq!(
            evaluate(false, false, false, false, context, &receipt, &delivery),
            ReceiptAction::Wait
        );
        for tick in 1..24 {
            receipt.tick(tick);
        }
        assert_eq!(
            evaluate(false, false, false, false, context, &receipt, &delivery),
            ReceiptAction::Cancel
        );
    }
}
