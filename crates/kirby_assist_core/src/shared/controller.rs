//! Controller-facing contracts: logical input, action ownership and Up-B receipts.

pub mod action_owner {
    //! One committed action owner, independent of controller polling frequency.
    //! Priority admits new work; only native safety interrupts a committed tape.
    //! Explicit cancellation and completion release both input and decision locks.
    /// Native grab startup, pull, hold, pummel, release and throw belong to the
    /// player's grab interaction. No automated punish may inject a throw here.
    /// Plain English: Native grab startup, pull, hold, pummel, release and throw belong to the player's grab interaction. No automated punish may inject a throw here.
    pub fn native_grab_active(status: i32) -> bool {
        matches!(
            status,
            crate::native_states::CATCH..=crate::native_states::THROW
        )
    }

    /// The owner of a committed action,
    /// determining which gameplay source controls the input.
    /// Plain English: The owner of a committed action, determining which gameplay source controls the input.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Owner {
        /// Technical: Represents the `Safety` case of `Owner`.
        ///
        /// Plain English: Represents the Safety option in Owner.
        Safety,
        /// Technical: Represents the `GrabDodge` case of `Owner`.
        ///
        /// Plain English: Represents the GrabDodge option in Owner.
        GrabDodge,
        /// Technical: Represents the `OptionRead` case of `Owner`.
        ///
        /// Plain English: Represents the OptionRead option in Owner.
        OptionRead,
        /// Technical: Represents the `Trip` case of `Owner`.
        ///
        /// Plain English: Represents the Trip option in Owner.
        Trip,
        /// Technical: Represents the `Fair` case of `Owner`.
        ///
        /// Plain English: Represents the Fair option in Owner.
        Fair,
        /// Technical: Represents the `Trump` case of `Owner`.
        ///
        /// Plain English: Represents the Trump option in Owner.
        Trump,
        /// Technical: Represents the `Ledge` case of `Owner`.
        ///
        /// Plain English: Represents the Ledge option in Owner.
        Ledge,
        /// Technical: Represents the `RollRead` case of `Owner`.
        ///
        /// Plain English: Represents the RollRead option in Owner.
        RollRead,
        /// Technical: Represents the `Defense` case of `Owner`.
        ///
        /// Plain English: Represents the Defense option in Owner.
        Defense,
        /// Technical: Represents the `Turbo` case of `Owner`.
        ///
        /// Plain English: Represents the Turbo option in Owner.
        Turbo,
        /// Technical: Represents the `Posture` case of `Owner`.
        ///
        /// Plain English: Represents the Posture option in Owner.
        Posture,
        /// Technical: Represents the `OutOfShield` case of `Owner`.
        ///
        /// Plain English: Represents the OutOfShield option in Owner.
        OutOfShield,
        /// Technical: Represents the `Reflect` case of `Owner`.
        ///
        /// Plain English: Represents the Reflect option in Owner.
        Reflect,
    }

    /// Select the base physical escape request for automatic special owners.
    /// Shield grabs and authored ledge/button tapes retain their own controls.
    /// Plain English: Select the base physical escape request for automatic special owners. Shield grabs and authored ledge/button tapes retain their own controls.
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
        /// Technical: Returns the `priority` result.
        ///
        /// Plain English: Gets the priority result.
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
        /// Technical: Determines whether the `exclusive` condition holds.
        ///
        /// Plain English: Checks whether exclusive.
        pub fn exclusive(self) -> bool {
            !matches!(self, Self::Defense | Self::Turbo | Self::Posture)
        }
    }

    /// Complete the selected packet at the shared output boundary. Even a neutral
    /// wait/release owns all gameplay sources; absent axes mean neutral, not human
    /// passthrough. Passive movement/posture helpers deliberately remain partial.
    /// Plain English: Complete the selected packet at the shared output boundary. Even a neutral wait/release owns all gameplay sources; absent axes mean neutral, not human passthrough. Passive movement/posture helpers deliberately remain partial.
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
    /// Plain English: Exactly one native safety phase owns a packet. Capture mash, final launch DI, imminent tech, then early SDI is the fixed precedence. A losing tech proposal receives no mapper acknowledgment and cannot spend its lockout.
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
    /// Only an independently validated imminent tech press can displace launch DI.
    /// No sticks/buttons are combined; DI resumes on the Guard release packet.
    /// Plain English: Only an independently validated imminent tech press can displace launch DI. No sticks/buttons are combined; DI resumes on the Guard release packet.
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
            && (di.is_none() || urgent && tech.press)
            && (tech.press || tech.owns_guard))
            .then_some(tech);
        let output = crate::gameplay::ControllerOutput {
            requested: if tech.is_some_and(|t| t.press) {
                crate::native_input::MAPPER_ZR
            } else {
                0
            },
            stick: mash.or_else(|| if tech.is_none() { di.or(sdi) } else { None }),
            ..Default::default()
        };
        (output, tech)
    }

    /// Represents the intent of an action owner, including whether it is present and committed.
    /// Plain English: Represents the intent of an action owner, including whether it is present and committed.
    #[derive(Clone, Copy)]
    pub struct Intent {
        /// Technical: Stores the `owner` value in `Intent` as `Owner`.
        ///
        /// Plain English: Holds the owner value used by Intent.
        pub owner: Owner,
        /// Technical: Stores the `present` value in `Intent` as `bool`.
        ///
        /// Plain English: Holds the present value used by Intent.
        pub present: bool,
        /// Technical: Stores the `committed` value in `Intent` as `bool`.
        ///
        /// Plain English: Holds the committed value used by Intent.
        pub committed: bool,
    }
    impl Intent {
        /// The final neutral release still belongs to the sequence, even after its
        /// interpreter has finished. Repeated mapper polls must deliver that release
        /// before a competing owner can take over on the next observation.
        /// A bounded native receipt keeps the reservation and emits an exclusive
        /// neutral packet at the shared output boundary until success or expiry.
        /// Plain English: The final neutral release still belongs to the sequence, even after its interpreter has finished. Repeated mapper polls must deliver that release before a competing owner can take over on the next observation. A bounded native.
        pub fn sequence_receipt(owner: Owner, owned_buttons: u32, receipt: bool) -> Self {
            Self {
                owner,
                present: owned_buttons != 0 || receipt,
                committed: owned_buttons != 0 || receipt,
            }
        }
        /// Technical: Returns the `input_sequence` result.
        ///
        /// Plain English: Gets the input sequence result.
        pub fn input_sequence(owner: Owner, owned_buttons: u32) -> Self {
            Self::sequence_receipt(owner, owned_buttons, false)
        }
    }

    /// Tracks the ownership status of various action sources, including ledge priority and manual takeover.
    /// Plain English: Tracks the ownership status of various action sources, including ledge priority and manual takeover.
    #[derive(Default)]
    pub struct ActionOwnership {
        /// Technical: Stores the `locked` value in `ActionOwnership` as `Option<Owner>`.
        ///
        /// Plain English: Optionally stores the locked for ActionOwnership; it may be absent.
        locked: Option<Owner>,
        /// Technical: Stores the `ledge_priority` value in `ActionOwnership` as `bool`.
        ///
        /// Plain English: Holds the ledge priority value used by ActionOwnership.
        ledge_priority: bool,
        /// Technical: Stores the `automatic_suspended` value in `ActionOwnership` as `bool`.
        ///
        /// Plain English: Holds the automatic suspended value used by ActionOwnership.
        automatic_suspended: bool,
        /// Technical: Stores the `manual` value in `ActionOwnership` as `crate::manual_controls::ManualTakeover`.
        ///
        /// Plain English: Holds the manual value used by ActionOwnership.
        manual: crate::manual_controls::ManualTakeover,
        /// Technical: Stores the `locked_manual` value in `ActionOwnership` as `Option<crate::manual_controls::ManualTakeover>`.
        ///
        /// Plain English: Optionally stores the locked manual for ActionOwnership; it may be absent.
        locked_manual: Option<crate::manual_controls::ManualTakeover>,
    }
    impl ActionOwnership {
        /// Technical: Constructs a new value with the type’s initial state.
        ///
        /// Plain English: Creates a new value.
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
        /// Plain English: An observed ledge visit reserves admission ahead of new general reads. An existing committed sequence finishes first; native safety still wins. Configured actions all share this gate; future action owners are blocked by.
        pub fn set_ledge_priority(&mut self, active: bool) {
            self.ledge_priority = active;
        }

        /// Technical: Determines whether the `ledge_priority` condition holds.
        ///
        /// Plain English: Checks whether ledge priority.
        pub fn ledge_priority(&self) -> bool {
            self.ledge_priority
        }
        /// Technical: Determines whether the `ledge_compatible` condition holds.
        ///
        /// Plain English: Checks whether ledge compatible.
        fn ledge_compatible(owner: Owner) -> bool {
            matches!(
                owner,
                Owner::Ledge | Owner::Safety | Owner::Defense | Owner::Turbo | Owner::Posture
            )
        }
        /// Technical: Updates the `unlock` state using the supplied inputs.
        ///
        /// Plain English: Updates unlock.
        fn unlock(&mut self) {
            self.locked = None;
            self.locked_manual = None;
        }
        /// Release only this owner when native startup/cancellation has ended it.
        /// Plain English: Release only this owner when native startup/cancellation has ended it.
        pub fn release(&mut self, owner: Owner) {
            if self.locked == Some(owner) {
                self.unlock();
            }
        }
        /// Transfer an explicitly chained response after native confirmation. This
        /// retains the input snapshot; unrelated proposals must use select().
        /// Plain English: Transfer an explicitly chained response after native confirmation. This retains the input snapshot; unrelated proposals must use select().
        pub fn handoff(&mut self, from: Owner, to: Owner) -> bool {
            if self.locked != Some(from) {
                return false;
            }
            self.locked = Some(to);
            true
        }
        /// Current committed owner; preparation with no committed input is None.
        /// Plain English: Current committed owner; preparation with no committed input is None.
        pub fn current_owner(&self) -> Option<Owner> {
            self.locked
        }
        /// Technical: Checks whether the supplied owner is managed automatically.
        ///
        /// Plain English: Reports whether this owner uses automatic handling.
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
        /// Technical: Determines whether the `automatic_active` condition holds.
        ///
        /// Plain English: Checks whether automatic active.
        pub fn automatic_active(&self) -> bool {
            self.locked.is_some_and(Self::automatic)
        }
        /// Observe fresh physical intent continuously, but freeze the automatic
        /// admission/movement view for the lifetime of the committed owner. Explicit
        /// until-manual recipe steps still read physical_inputs().epoch().
        /// Plain English: Observe fresh physical intent continuously, but freeze the automatic admission/movement view for the lifetime of the committed owner. Explicit until-manual recipe steps still read physicalinputs().epoch().
        pub fn observe_manual(&mut self, buttons: u32, sticks: [f32; 4]) {
            self.manual.sample(buttons, sticks);
        }
        /// Technical: Returns the `physical_inputs` result.
        ///
        /// Plain English: Gets the physical inputs result.
        pub fn physical_inputs(&self) -> &crate::manual_controls::ManualTakeover {
            &self.manual
        }
        /// Technical: Returns the `policy_inputs` result.
        ///
        /// Plain English: Gets the policy inputs result.
        pub fn policy_inputs(&self) -> &crate::manual_controls::ManualTakeover {
            self.locked_manual.as_ref().unwrap_or(&self.manual)
        }
        /// Suspension prevents new automatic work. Cancellation of an active tape
        /// is explicit (disable, scene loss, native interruption or recipe handback).
        /// Plain English: Suspension prevents new automatic work. Cancellation of an active tape is explicit (disable, scene loss, native interruption or recipe handback).
        pub fn set_automatic_suspended(&mut self, active: bool) {
            self.automatic_suspended = active;
        }
        /// Technical: Determines whether the `automatic_suspended` condition holds.
        ///
        /// Plain English: Checks whether automatic suspended.
        pub fn automatic_suspended(&self) -> bool {
            self.automatic_suspended
        }
        /// Scope-independent admission gates shared by every owner.
        /// Plain English: Scope-independent admission gates shared by every owner.
        fn admissible(&self, owner: Owner) -> bool {
            (!self.automatic_suspended || !Self::automatic(owner))
                && (!self.ledge_priority || Self::ledge_compatible(owner))
        }
        /// Priority admits new work; it never replaces an in-flight tape. Native
        /// safety cancels a tape whose gameplay prerequisites have been lost.
        /// Plain English: Priority admits new work; it never replaces an in-flight tape. Native safety cancels a tape whose gameplay prerequisites have been lost.
        pub fn allows(&self, owner: Owner) -> bool {
            self.locked == Some(owner)
                || (self.admissible(owner) && self.locked.is_none_or(|_| owner == Owner::Safety))
        }

        /// Check admission after releasing a particular owner without changing state.
        /// Plain English: Check admission after releasing a particular owner without changing state.
        pub fn allows_after_release(&self, owner: Owner, releasing: Owner) -> bool {
            if self.locked != Some(releasing) {
                return self.allows(owner);
            }
            self.admissible(owner)
        }

        /// Technical: Returns an optional `select` value.
        ///
        /// Plain English: Gets select if it is available.
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
    /// Plain English: A set of logical buttons the assist is allowed to add.
    pub struct ButtonSet(
        /// Technical: Bit mask containing logical button permissions.
        ///
        /// Plain English: Tracks which assist buttons are allowed.
        u16,
    );

    impl ButtonSet {
        /// Technical: Defines the `EMPTY` constant used by this module.
        ///
        /// Plain English: Provides the EMPTY constant value.
        pub const EMPTY: Self = Self(0);
        /// Technical: Defines the `ATTACK` constant used by this module.
        ///
        /// Plain English: Provides the ATTACK constant value.
        pub const ATTACK: Self = Self(1 << 0);
        /// Technical: Defines the `SPECIAL` constant used by this module.
        ///
        /// Plain English: Provides the SPECIAL constant value.
        pub const SPECIAL: Self = Self(1 << 1);
        /// Technical: Defines the `JUMP` constant used by this module.
        ///
        /// Plain English: Provides the JUMP constant value.
        pub const JUMP: Self = Self(1 << 2);
        /// Technical: Defines the `SHIELD` constant used by this module.
        ///
        /// Plain English: Provides the SHIELD constant value.
        pub const SHIELD: Self = Self(1 << 3);
        /// Technical: Defines the `GRAB` constant used by this module.
        ///
        /// Plain English: Provides the GRAB constant value.
        pub const GRAB: Self = Self(1 << 4);
        /// Technical: Defines the `DPAD_UP` constant used by this module.
        ///
        /// Plain English: Provides the DPAD UP constant value.
        pub const DPAD_UP: Self = Self(1 << 5);
        /// Technical: Defines the `DPAD_DOWN` constant used by this module.
        ///
        /// Plain English: Provides the DPAD DOWN constant value.
        pub const DPAD_DOWN: Self = Self(1 << 6);
        /// Technical: Defines the `DPAD_LEFT` constant used by this module.
        ///
        /// Plain English: Provides the DPAD LEFT constant value.
        pub const DPAD_LEFT: Self = Self(1 << 7);
        /// Technical: Defines the `DPAD_RIGHT` constant used by this module.
        ///
        /// Plain English: Provides the DPAD RIGHT constant value.
        pub const DPAD_RIGHT: Self = Self(1 << 8);

        // & keeps bits shared by both masks. Comparing against `other` asks whether
        // ALL its bits are present, so contains(ATTACK.union(JUMP)) needs both.
        /// Technical: Determines whether the `contains` condition holds.
        ///
        /// Plain English: Checks whether contains.
        pub const fn contains(self, other: Self) -> bool {
            (self.0 & other.0) == other.0
        }

        // Bitwise OR combines sets: ATTACK | JUMP means both are held.
        /// Technical: Returns the `union` result.
        ///
        /// Plain English: Gets the union result.
        pub const fn union(self, other: Self) -> Self {
            Self(self.0 | other.0)
        }

        /// Technical: Returns the `difference` result.
        ///
        /// Plain English: Gets the difference result.
        pub const fn difference(self, other: Self) -> Self {
            Self(self.0 & !other.0)
        }

        /// Technical: Checks whether empty.
        ///
        /// Plain English: Checks whether empty.
        pub const fn is_empty(self) -> bool {
            self.0 == 0
        }
    }

    /// A normalized logical stick position. Its numeric range is adapter-defined.
    /// Plain English: A normalized logical stick position. Its numeric range is adapter-defined.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct StickPosition {
        /// Technical: Stores the `x` value in `StickPosition` as `i16`.
        ///
        /// Plain English: Holds the x value used by StickPosition.
        pub x: i16,
        /// Technical: Stores the `y` value in `StickPosition` as `i16`.
        ///
        /// Plain English: Holds the y value used by StickPosition.
        pub y: i16,
    }

    impl StickPosition {
        /// Technical: Defines the `NEUTRAL` constant used by this module.
        ///
        /// Plain English: Provides the NEUTRAL constant value.
        pub const NEUTRAL: Self = Self { x: 0, y: 0 };
        /// Technical: Defines the `LEFT` constant used by this module.
        ///
        /// Plain English: Provides the LEFT constant value.
        pub const LEFT: Self = Self { x: -1, y: 0 };
        /// Technical: Defines the `RIGHT` constant used by this module.
        ///
        /// Plain English: Provides the RIGHT constant value.
        pub const RIGHT: Self = Self { x: 1, y: 0 };
        /// Technical: Defines the `UP` constant used by this module.
        ///
        /// Plain English: Provides the UP constant value.
        pub const UP: Self = Self { x: 0, y: 1 };
        /// Technical: Defines the `DOWN` constant used by this module.
        ///
        /// Plain English: Provides the DOWN constant value.
        pub const DOWN: Self = Self { x: 0, y: -1 };
    }

    /// A logical input snapshot used by the generic adapter and examples.
    /// It is NOT layout-compatible with the game's native Controller allocation.
    /// Plain English: A logical input snapshot used by the generic adapter and examples. It is NOT layout-compatible with the game's native Controller allocation.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ControllerFrame {
        /// Technical: Stores the `buttons` value in `ControllerFrame` as `ButtonSet`.
        ///
        /// Plain English: Holds the buttons value used by ControllerFrame.
        pub buttons: ButtonSet,
        /// Technical: Stores the `left_stick` value in `ControllerFrame` as `StickPosition`.
        ///
        /// Plain English: Holds the left stick value used by ControllerFrame.
        pub left_stick: StickPosition,
        /// Technical: Stores the `right_stick` value in `ControllerFrame` as `StickPosition`.
        ///
        /// Plain English: Holds the right stick value used by ControllerFrame.
        pub right_stick: StickPosition,
        /// Technical: Stores the `left_trigger` value in `ControllerFrame` as `u8`.
        ///
        /// Plain English: Holds the left trigger value used by ControllerFrame.
        pub left_trigger: u8,
        /// Technical: Stores the `right_trigger` value in `ControllerFrame` as `u8`.
        ///
        /// Plain English: Holds the right trigger value used by ControllerFrame.
        pub right_trigger: u8,
    }

    /// Whether an action leaves movement alone or requests a left-stick position.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    // This enum distinguishes "do not touch the player's stick" from "actively
    // request neutral". A neutral Request is still an override requiring permission.
    /// Plain English: Whether an action leaves movement alone or requests a left-stick position.
    pub enum LeftStickIntent {
        #[default]
        /// Technical: Represents the `PreservePlayer` case of `LeftStickIntent`.
        ///
        /// Plain English: Represents the PreservePlayer option in LeftStickIntent.
        PreservePlayer,
        /// Technical: Represents the `Request` case of `LeftStickIntent`.
        ///
        /// Plain English: Represents the Request option in LeftStickIntent.
        Request(
            /// Technical: Carries the requested stick position in `LeftStickIntent`.
            ///
            /// Plain English: Holds the requested stick position for this `LeftStickIntent` choice.
            StickPosition,
        ),
    }

    /// A named, inspectable recipe for one logical controller frame.
    /// Plain English: A named, inspectable recipe for one logical controller frame.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct InputRecipe {
        /// Technical: Stores the `buttons_to_add` value in `InputRecipe` as `ButtonSet`.
        ///
        /// Plain English: Holds the buttons to add value used by InputRecipe.
        pub buttons_to_add: ButtonSet,
        /// Technical: Stores the `left_stick` value in `InputRecipe` as `LeftStickIntent`.
        ///
        /// Plain English: Holds the left stick value used by InputRecipe.
        pub left_stick: LeftStickIntent,
    }

    impl InputRecipe {
        /// Technical: Defines the `NONE` constant used by this module.
        ///
        /// Plain English: Provides the NONE constant value.
        pub const NONE: Self = Self {
            buttons_to_add: ButtonSet::EMPTY,
            left_stick: LeftStickIntent::PreservePlayer,
        };

        /// Technical: Returns the `buttons` result.
        ///
        /// Plain English: Gets the buttons result.
        pub const fn buttons(buttons_to_add: ButtonSet) -> Self {
            Self {
                buttons_to_add,
                left_stick: LeftStickIntent::PreservePlayer,
            }
        }

        /// Technical: Returns the `buttons_with_left_stick` result.
        ///
        /// Plain English: Gets the buttons with left stick result.
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
    /// Plain English: Explicit permissions for arbitration. Keep the default button-only policy until a directional input path is independently tested.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct InputPermissions {
        /// Technical: Stores the `allow_left_stick_override` value in `InputPermissions` as `bool`.
        ///
        /// Plain English: Holds the allow left stick override value used by InputPermissions.
        pub allow_left_stick_override: bool,
    }

    /// The non-destructive result of applying a recipe to player input.
    /// Plain English: The non-destructive result of applying a recipe to player input.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ArbitrationResult {
        /// Technical: Stores the `frame` value in `ArbitrationResult` as `ControllerFrame`.
        ///
        /// Plain English: Holds the frame value used by ArbitrationResult.
        pub frame: ControllerFrame,
        /// Technical: Stores the `added_buttons` value in `ArbitrationResult` as `ButtonSet`.
        ///
        /// Plain English: Holds the added buttons value used by ArbitrationResult.
        pub added_buttons: ButtonSet,
        /// Technical: Stores the `blocked_left_stick_request` value in `ArbitrationResult` as `bool`.
        ///
        /// Plain English: Holds the blocked left stick request value used by ArbitrationResult.
        pub blocked_left_stick_request: bool,
    }

    /// Applies one assist recipe to the player's input for this frame.
    ///
    /// Requested buttons are added to held player buttons; an empty request adds
    /// nothing. The left stick stays with the player unless an override is allowed.
    /// The right stick and both triggers always stay with the player.
    ///
    /// This generic helper always adds requested buttons, even with default
    /// permissions. It does not implement exclusive ownership or saved-sequence
    /// playback.
    /// Plain English: Applies one assist recipe to the player's input for this frame. Requested buttons are added to held player buttons; an empty request adds nothing. The left stick stays with the player unless an override is allowed. The right.
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

        /// Technical: Defines the `PLAYER` constant used by this module.
        ///
        /// Plain English: Provides the PLAYER constant value.
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

        /// Technical: Regression test for `buttons_are_added_without_taking_movement`.
        ///
        /// Plain English: Checks that buttons are added without taking movement.
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

        /// Technical: Regression test for `stick_request_requires_explicit_permission`.
        ///
        /// Plain English: Checks that stick request requires explicit permission.
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

    /// Technical: Defines the available `ReceiptAction` cases.
    ///
    /// Plain English: Lists the available ReceiptAction choices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum ReceiptAction {
        /// Technical: Represents the `Continue` case of `ReceiptAction`.
        ///
        /// Plain English: Represents the Continue option in ReceiptAction.
        Continue,
        /// Technical: Represents the `Confirm` case of `ReceiptAction`.
        ///
        /// Plain English: Represents the Confirm option in ReceiptAction.
        Confirm,
        /// Technical: Represents the `Wait` case of `ReceiptAction`.
        ///
        /// Plain English: Represents the Wait option in ReceiptAction.
        Wait,
        /// Technical: Represents the `Cancel` case of `ReceiptAction`.
        ///
        /// Plain English: Represents the Cancel option in ReceiptAction.
        Cancel,
    }

    /// Technical: Checks whether a pending operation is stored.
    ///
    /// Plain English: Checks whether something is waiting to be processed.
    pub(crate) fn pending(seen: bool, sent: bool) -> bool {
        !seen && sent
    }

    /// Technical: Determines whether the `manual_handback` condition holds.
    ///
    /// Plain English: Checks whether manual handback.
    pub(crate) fn manual_handback(
        automatic: bool,
        seen: bool,
        context: GameplayContext,
        epoch: u64,
    ) -> bool {
        automatic && seen && context.manual_input_epoch != epoch
    }

    /// Technical: Returns the `evaluate` result.
    ///
    /// Plain English: Gets the evaluate result.
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

        /// Technical: Regression test for `receipt_preserves_startup_wait_and_cancellation_gates`.
        ///
        /// Plain English: Checks that receipt preserves startup wait and cancellation gates.
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
