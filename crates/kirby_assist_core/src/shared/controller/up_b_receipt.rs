//! Bounded input-free wait for a direct automatic Up-B startup.
use crate::gameplay::GameplayContext;
use crate::move_input::{DeliveryBudget, UpBReceipt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ReceiptAction {
    Continue,
    Confirm,
    Wait,
    Cancel,
}

pub(crate) fn pending(seen: bool, sent: bool) -> bool {
    !seen && sent
}

pub(crate) fn manual_handback(
    automatic: bool,
    seen: bool,
    context: GameplayContext,
    epoch: u64,
) -> bool {
    automatic && seen && context.manual_input_epoch != epoch
}

pub(crate) fn evaluate(
    seen: bool,
    native: bool,
    waiting: bool,
    from_dodge: bool,
    context: GameplayContext,
    receipt: &UpBReceipt,
    delivery: &DeliveryBudget,
) -> ReceiptAction {
    if native && !seen {
        return ReceiptAction::Confirm;
    }
    if !pending(seen, receipt.sent()) {
        return ReceiptAction::Continue;
    }

    if !crate::reaction_rules::automatic_up_b_receipt_status(
        context.status,
        context.situation,
        from_dodge,
    ) || receipt.expired()
    {
        return ReceiptAction::Cancel;
    }
    if waiting || context.stopped || !context.can_act || delivery.exhausted() {
        ReceiptAction::Wait
    } else {
        ReceiptAction::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_states::{situation, status};

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
