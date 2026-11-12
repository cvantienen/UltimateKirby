//! Controller-facing contracts: logical input, action ownership and Up-B receipts.

pub mod action_owner {
    //! One committed action owner, independent of controller polling frequency.
    //! Priority admits new work; only native safety interrupts a committed tape.
    //! Explicit cancellation and completion release both input and decision locks.
    /// Native grab startup, pull, hold, pummel, release and throw belong to the
    /// player's grab interaction. No automated punish may inject a throw here.
    pub fn native_grab_active(status: i32) -> bool {
        matches!(
            status,
            crate::native_states::CATCH..=crate::native_states::THROW
        )
    }

    /// The owner of a committed action,
    /// determining which gameplay source controls the input.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Owner {
/// `Safety` option in the `Owner` Smash policy state.
        Safety,
/// `GrabDodge` option in the `Owner` Smash policy state.
        GrabDodge,
/// `OptionRead` option in the `Owner` Smash policy state.
        OptionRead,
/// `Trip` option in the `Owner` Smash policy state.
        Trip,
/// `Fair` option in the `Owner` Smash policy state.
        Fair,
/// `Trump` option in the `Owner` Smash policy state.
        Trump,
/// `Ledge` option in the `Owner` Smash policy state.
        Ledge,
/// `RollRead` option in the `Owner` Smash policy state.
        RollRead,
/// `Defense` option in the `Owner` Smash policy state.
        Defense,
/// `Turbo` option in the `Owner` Smash policy state.
        Turbo,
/// `Posture` option in the `Owner` Smash policy state.
        Posture,
/// `OutOfShield` option in the `Owner` Smash policy state.
        OutOfShield,
/// `Reflect` option in the `Owner` Smash policy state.
        Reflect,
    }

    /// Select the base physical escape request for automatic special owners.
    /// Shield grabs and authored ledge/button tapes retain their own controls.
    pub fn automatic_escape_request(
        owner: Option<Owner>,
        roll: u32,
        option: u32,
        shield_up_b: Option<u32>,
    ) -> Option<u32> {
        match owner {
            Some(Owner::RollRead) => Some(roll),
            Some(Owner::OptionRead) => Some(option),
            Some(Owner::OutOfShield) => shield_up_b,
            _ => None,
        }
    }
    impl Owner {
        /// Returns the `priority` result.
        fn priority(self) -> u8 {
            match self {
                Self::Safety => 100,
                Self::GrabDodge => 99,
                Self::OptionRead => 98,
                Self::Trip => 90,
                Self::Fair => 80,
                Self::Trump => 70,
                Self::Ledge => 95,
                Self::RollRead => 97,
                Self::Defense => 30,
                Self::Turbo => 10,
                Self::Posture => 5,
                Self::OutOfShield => 92,
                Self::Reflect => 35,
            }
        }
        /// Determines whether the `exclusive` condition holds.
        pub fn exclusive(self) -> bool {
            !matches!(self, Self::Defense | Self::Turbo | Self::Posture)
        }
    }

    /// Complete the selected packet at the shared output boundary. Even a neutral
    /// wait/release owns all gameplay sources; absent axes mean neutral, not human
    /// passthrough. Passive movement/posture helpers deliberately remain partial.
    pub fn exclusive_output(
        owner: Option<Owner>,
        committed: bool,
        mut output: crate::gameplay::ControllerOutput,
    ) -> crate::gameplay::ControllerOutput {
        if committed && owner.is_some_and(Owner::exclusive) {
            output.owned |= crate::ledge_trap::SEQUENCE_BUTTONS;
            output.stick = Some(output.stick.unwrap_or([0.; 2]));
        }
        output
    }

    /// Exactly one native safety phase owns a packet. Capture mash, final launch
    /// DI, imminent tech, then early SDI is the fixed precedence. A losing tech
    /// proposal receives no mapper acknowledgment and cannot spend its lockout.
    pub fn safety_output(
        tech: crate::tech_input::TechOutput,
        mash: Option<[f32; 2]>,
        di: Option<[f32; 2]>,
        sdi: Option<[f32; 2]>,
    ) -> (
        crate::gameplay::ControllerOutput,
        Option<crate::tech_input::TechOutput>,
    ) {
        safety_output_with_deadline(tech, mash, di, sdi, false)
    }
    /// A native tech opportunity may displace launch DI for a Guard press.
    /// The alternating release packet keeps Guard ownership while DI resumes.
    pub fn safety_output_with_deadline(
        tech: crate::tech_input::TechOutput,
        mash: Option<[f32; 2]>,
        di: Option<[f32; 2]>,
        sdi: Option<[f32; 2]>,
        urgent: bool,
    ) -> (
        crate::gameplay::ControllerOutput,
        Option<crate::tech_input::TechOutput>,
    ) {
        let tech = (mash.is_none()
            && (di.is_none() || urgent && tech.press || !tech.press)
            && (tech.press || tech.owns_guard))
            .then_some(tech);
        let output = crate::gameplay::ControllerOutput {
            requested: if tech.is_some_and(|t| t.press) {
                crate::native_input::MAPPER_ZR
            } else {
                0
            },
            stick: mash.or_else(|| {
                if tech.is_none_or(|output| !output.press) {
                    di.or(sdi)
                } else {
                    None
                }
            }),
            ..Default::default()
        };
        (output, tech)
    }

    /// Represents the intent of an action owner, including whether it is present and committed.
    #[derive(Clone, Copy)]
    pub struct Intent {
/// Runtime owner value used by Intent..
        pub owner: Owner,
/// Runtime present value used by Intent..
        pub present: bool,
/// Runtime committed value used by Intent..
        pub committed: bool,
    }
    impl Intent {
        /// The final neutral release still belongs to the sequence, even after its
        /// interpreter has finished. Repeated mapper polls must deliver that release
        /// before a competing owner can take over on the next observation.
        /// A bounded native receipt keeps the reservation and emits an exclusive
        /// neutral packet at the shared output boundary until success or expiry.
        pub fn sequence_receipt(owner: Owner, owned_buttons: u32, receipt: bool) -> Self {
            Self {
                owner,
                present: owned_buttons != 0 || receipt,
                committed: owned_buttons != 0 || receipt,
            }
        }
        /// Returns the `input_sequence` result.
        pub fn input_sequence(owner: Owner, owned_buttons: u32) -> Self {
            Self::sequence_receipt(owner, owned_buttons, false)
        }
    }

    /// Tracks the ownership status of various action sources, including ledge priority and manual takeover.
    #[derive(Default)]
    pub struct ActionOwnership {
/// Runtime locked value used by ActionOwnership..
        locked: Option<Owner>,
/// Runtime ledge priority value used by ActionOwnership..
        ledge_priority: bool,
/// Runtime automatic suspended value used by ActionOwnership..
        automatic_suspended: bool,
/// Runtime manual value used by ActionOwnership..
        manual: crate::manual_controls::ManualTakeover,
/// Runtime locked manual value used by ActionOwnership..
        locked_manual: Option<crate::manual_controls::ManualTakeover>,
    }
    impl ActionOwnership {
        /// Constructs a new value with the type’s initial state.
        pub const fn new() -> Self {
            Self {
                locked: None,
                ledge_priority: false,
                automatic_suspended: false,
                manual: crate::manual_controls::ManualTakeover::new(),
                locked_manual: None,
            }
        }
        /// An observed ledge visit reserves admission ahead of new general reads.
        /// An existing committed sequence finishes first; native safety still wins.
        /// Configured actions all share this gate; future action owners are blocked
        /// by default unless explicitly classified as passive/safety here.
        pub fn set_ledge_priority(&mut self, active: bool) {
            self.ledge_priority = active;
        }

        /// Determines whether the `ledge_priority` condition holds.
        pub fn ledge_priority(&self) -> bool {
            self.ledge_priority
        }
        /// Determines whether the `ledge_compatible` condition holds.
        fn ledge_compatible(owner: Owner) -> bool {
            matches!(
                owner,
                Owner::Ledge | Owner::Safety | Owner::Defense | Owner::Turbo | Owner::Posture
            )
        }
        /// Updates the `unlock` state using the supplied inputs.
        fn unlock(&mut self) {
            self.locked = None;
            self.locked_manual = None;
        }
        /// Release only this owner when native startup/cancellation has ended it.
        pub fn release(&mut self, owner: Owner) {
            if self.locked == Some(owner) {
                self.unlock();
            }
        }
        /// Transfer an explicitly chained response after native confirmation. This
        /// retains the input snapshot; unrelated proposals must use select().
        pub fn handoff(&mut self, from: Owner, to: Owner) -> bool {
            if self.locked != Some(from) {
                return false;
            }
            self.locked = Some(to);
            true
        }
        /// Current committed owner; preparation with no committed input is None.
        pub fn current_owner(&self) -> Option<Owner> {
            self.locked
        }
        /// Checks whether the supplied owner is managed automatically.
        fn automatic(owner: Owner) -> bool {
            matches!(
                owner,
                Owner::RollRead
                    | Owner::OutOfShield
                    | Owner::GrabDodge
                    | Owner::OptionRead
                    | Owner::Trip
                    | Owner::Fair
                    | Owner::Reflect
            )
        }
        /// Determines whether the `automatic_active` condition holds.
        pub fn automatic_active(&self) -> bool {
            self.locked.is_some_and(Self::automatic)
        }
        /// Observe fresh physical intent continuously, but freeze the automatic
        /// admission/movement view for the lifetime of the committed owner. Explicit
        /// until-manual recipe steps still read physical_inputs().epoch().
        pub fn observe_manual(&mut self, buttons: u32, sticks: [f32; 4]) {
            self.manual.sample(buttons, sticks);
        }
        /// Returns the `physical_inputs` result.
        pub fn physical_inputs(&self) -> &crate::manual_controls::ManualTakeover {
            &self.manual
        }
        /// Returns the `policy_inputs` result.
        pub fn policy_inputs(&self) -> &crate::manual_controls::ManualTakeover {
            self.locked_manual.as_ref().unwrap_or(&self.manual)
        }
        /// Suspension prevents new automatic work. Cancellation of an active tape
        /// is explicit (disable, scene loss, native interruption or recipe handback).
        pub fn set_automatic_suspended(&mut self, active: bool) {
            self.automatic_suspended = active;
        }
        /// Determines whether the `automatic_suspended` condition holds.
        pub fn automatic_suspended(&self) -> bool {
            self.automatic_suspended
        }
        /// Scope-independent admission gates shared by every owner.
        fn admissible(&self, owner: Owner) -> bool {
            (!self.automatic_suspended || !Self::automatic(owner))
                && (!self.ledge_priority || Self::ledge_compatible(owner))
        }
        /// Priority admits new work; it never replaces an in-flight tape. Native
        /// safety cancels a tape whose gameplay prerequisites have been lost.
        pub fn allows(&self, owner: Owner) -> bool {
            self.locked == Some(owner)
                || (self.admissible(owner) && self.locked.is_none_or(|_| owner == Owner::Safety))
        }

        /// Check admission after releasing a particular owner without changing state.
        pub fn allows_after_release(&self, owner: Owner, releasing: Owner) -> bool {
            if self.locked != Some(releasing) {
                return self.allows(owner);
            }
            self.admissible(owner)
        }

        /// Returns an optional `select` value.
        pub fn select(&mut self, valid_scope: bool, intents: &[Intent]) -> Option<Owner> {
            if !valid_scope {
                self.unlock();
                self.ledge_priority = false;
                self.automatic_suspended = false;
                return None;
            }
            if self.locked.is_some_and(|owner| {
                !intents
                    .iter()
                    .any(|i| i.owner == owner && i.present && i.committed)
            }) {
                self.unlock();
            }
            let winner = intents
                .iter()
                .filter(|i| i.present && self.allows(i.owner))
                .max_by_key(|i| i.owner.priority());
            let next = winner.filter(|i| i.committed).map(|i| i.owner);
            if next != self.locked {
                self.locked_manual = next.map(|_| self.manual);
            }
            self.locked = next;
            winner.map(|i| i.owner)
        }
    }
}
pub mod input {
    //! Logical controller input and conservative arbitration.
    //!
    //! These are application-level names, not Horizon HID values or Smash memory
    //! layouts. The live policy translates its chosen logical recipe to physical
    //! button requests, and native_input translates those to Smash's raw bitfield.
    //! Generic arbitration here is additive; takeover exists separately in the raw
    //! live overlay and does not change these teaching/example types.

    /// A set of logical buttons the assist is allowed to add.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    // A newtype wraps a number so logical buttons cannot accidentally be passed
    // where a raw hardware mask is expected. `1 << n` reserves one bit per button.
    ///
    pub struct ButtonSet(
        /// Bit mask containing logical button permissions.
        u16,
    );

    impl ButtonSet {
/// `EMPTY` is a configuration value used by the Smash runtime.
        pub const EMPTY: Self = Self(0);
/// `ATTACK` is a configuration value used by the Smash runtime.
        pub const ATTACK: Self = Self(1 << 0);
/// `SPECIAL` is a configuration value used by the Smash runtime.
        pub const SPECIAL: Self = Self(1 << 1);
/// `JUMP` is a configuration value used by the Smash runtime.
        pub const JUMP: Self = Self(1 << 2);
/// `SHIELD` is a configuration value used by the Smash runtime.
        pub const SHIELD: Self = Self(1 << 3);
/// `GRAB` is a configuration value used by the Smash runtime.
        pub const GRAB: Self = Self(1 << 4);
/// `DPAD_UP` is a configuration value used by the Smash runtime.
        pub const DPAD_UP: Self = Self(1 << 5);
/// `DPAD_DOWN` is a configuration value used by the Smash runtime.
        pub const DPAD_DOWN: Self = Self(1 << 6);
/// `DPAD_LEFT` is a configuration value used by the Smash runtime.
        pub const DPAD_LEFT: Self = Self(1 << 7);
/// `DPAD_RIGHT` is a configuration value used by the Smash runtime.
        pub const DPAD_RIGHT: Self = Self(1 << 8);

        // & keeps bits shared by both masks. Comparing against `other` asks whether
        // ALL its bits are present, so contains(ATTACK.union(JUMP)) needs both.
        /// Determines whether the `contains` condition holds.
        pub const fn contains(self, other: Self) -> bool {
            (self.0 & other.0) == other.0
        }

        // Bitwise OR combines sets: ATTACK | JUMP means both are held.
        /// Returns the `union` result.
        pub const fn union(self, other: Self) -> Self {
            Self(self.0 | other.0)
        }

        /// Returns the `difference` result.
        pub const fn difference(self, other: Self) -> Self {
            Self(self.0 & !other.0)
        }

        /// Checks whether empty.
        pub const fn is_empty(self) -> bool {
            self.0 == 0
        }
    }

    /// A normalized logical stick position. Its numeric range is adapter-defined.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct StickPosition {
/// Runtime x value used by StickPosition..
        pub x: i16,
/// Runtime y value used by StickPosition..
        pub y: i16,
    }

    impl StickPosition {
/// `NEUTRAL` is a configuration value used by the Smash runtime.
        pub const NEUTRAL: Self = Self { x: 0, y: 0 };
/// `LEFT` is a configuration value used by the Smash runtime.
        pub const LEFT: Self = Self { x: -1, y: 0 };
/// `RIGHT` is a configuration value used by the Smash runtime.
        pub const RIGHT: Self = Self { x: 1, y: 0 };
/// `UP` is a configuration value used by the Smash runtime.
        pub const UP: Self = Self { x: 0, y: 1 };
/// `DOWN` is a configuration value used by the Smash runtime.
        pub const DOWN: Self = Self { x: 0, y: -1 };
    }

    /// A logical input snapshot used by the generic adapter and examples.
    /// It is NOT layout-compatible with the game's native Controller allocation.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ControllerFrame {
/// Runtime buttons value used by ControllerFrame..
        pub buttons: ButtonSet,
/// Runtime left stick value used by ControllerFrame..
        pub left_stick: StickPosition,
/// Runtime right stick value used by ControllerFrame..
        pub right_stick: StickPosition,
/// Runtime left trigger value used by ControllerFrame..
        pub left_trigger: u8,
/// Runtime right trigger value used by ControllerFrame..
        pub right_trigger: u8,
    }

    /// Whether an action leaves movement alone or requests a left-stick position.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    // This enum distinguishes "do not touch the player's stick" from "actively
    // request neutral". A neutral Request is still an override requiring permission.
    ///  Whether an action leaves movement alone or requests a left-stick position.
    pub enum LeftStickIntent {
        #[default]
        /// Represents the `PreservePlayer` case of `LeftStickIntent`.
        PreservePlayer,
        /// Represents the `Request` case of `LeftStickIntent`.
        Request(
            /// Carries the requested stick position in `LeftStickIntent`.
            StickPosition,
        ),
    }

    /// A named, inspectable recipe for one logical controller frame.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct InputRecipe {
/// Runtime buttons to add value used by InputRecipe..
        pub buttons_to_add: ButtonSet,
/// Runtime left stick value used by InputRecipe..
        pub left_stick: LeftStickIntent,
    }

    impl InputRecipe {
/// `NONE` is a configuration value used by the Smash runtime.
        pub const NONE: Self = Self {
            buttons_to_add: ButtonSet::EMPTY,
            left_stick: LeftStickIntent::PreservePlayer,
        };

        /// Returns the `buttons` result.
        pub const fn buttons(buttons_to_add: ButtonSet) -> Self {
            Self {
                buttons_to_add,
                left_stick: LeftStickIntent::PreservePlayer,
            }
        }

        /// Returns the `buttons_with_left_stick` result.
        pub const fn buttons_with_left_stick(
            buttons_to_add: ButtonSet,
            left_stick: StickPosition,
        ) -> Self {
            Self {
                buttons_to_add,
                left_stick: LeftStickIntent::Request(left_stick),
            }
        }
    }

    /// Explicit permissions for arbitration. Keep the default button-only policy
    /// until a directional input path is independently tested.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct InputPermissions {
/// Runtime allow left stick override value used by InputPermissions..
        pub allow_left_stick_override: bool,
    }

    /// The non-destructive result of applying a recipe to player input.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ArbitrationResult {
/// current native animation frame.
        pub frame: ControllerFrame,
/// Runtime added buttons value used by ArbitrationResult..
        pub added_buttons: ButtonSet,
/// Runtime blocked left stick request value used by ArbitrationResult..
        pub blocked_left_stick_request: bool,
    }

    /// Applies one assist recipe to the player's input for this frame.
    /// Requested buttons are added to held player buttons; an empty request adds
    /// nothing. The left stick stays with the player unless an override is allowed.
    /// The right stick and both triggers always stay with the player.
    /// This generic helper always adds requested buttons, even with default
    /// permissions. It does not implement exclusive ownership or saved-sequence
    /// playback.
    pub const fn arbitrate(
        user: ControllerFrame,
        recipe: InputRecipe,
        permissions: InputPermissions,
    ) -> ArbitrationResult {
        // Keep the player's stick, or use the requested position only when permitted.
        // Report a blocked request so callers can distinguish it from no request.
        let (left_stick, blocked_left_stick_request) = match recipe.left_stick {
            LeftStickIntent::PreservePlayer => (user.left_stick, false),
            LeftStickIntent::Request(position) if permissions.allow_left_stick_override => {
                (position, false)
            }
            LeftStickIntent::Request(_) => (user.left_stick, true),
        };

        // Union preserves every held button and adds the recipe's requested buttons.
        ArbitrationResult {
            frame: ControllerFrame {
                buttons: user.buttons.union(recipe.buttons_to_add),
                left_stick,
                right_stick: user.right_stick,
                left_trigger: user.left_trigger,
                right_trigger: user.right_trigger,
            },
            // Distinguish newly added buttons from ones the player already held.
            added_buttons: recipe.buttons_to_add.difference(user.buttons),
            blocked_left_stick_request,
        }
    }

    // Compiled only by cargo test on the host; these assertions are not live
    // Switch logging or gameplay callbacks. Add regressions here for new behavior.
    #[cfg(test)]
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
}
pub(crate) mod up_b_receipt {
    //! Bounded input-free wait for a direct automatic Up-B startup.
    use crate::gameplay::GameplayContext;
    use crate::move_input::{DeliveryBudget, UpBReceipt};

/// Available `ReceiptAction` choices for the Smash assistance menu.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum ReceiptAction {
/// `Continue` option in the `ReceiptAction` Smash policy state.
        Continue,
/// `Confirm` option in the `ReceiptAction` Smash policy state.
        Confirm,
/// `Wait` option in the `ReceiptAction` Smash policy state.
        Wait,
/// `Cancel` option in the `ReceiptAction` Smash policy state.
        Cancel,
    }

    /// Checks whether a pending operation is stored.
    pub(crate) fn pending(seen: bool, sent: bool) -> bool {
        !seen && sent
    }

    /// Determines whether the `manual_handback` condition holds.
    pub(crate) fn manual_handback(
        automatic: bool,
        seen: bool,
        context: GameplayContext,
        epoch: u64,
    ) -> bool {
        automatic && seen && context.manual_input_epoch != epoch
    }

    /// Returns the `evaluate` result.
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
}
