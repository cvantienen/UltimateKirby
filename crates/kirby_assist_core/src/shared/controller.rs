//! Controller-facing contracts: logical input, action ownership and Up-B receipts.

pub mod action_owner {
    //! One committed action owner, independent of controller polling frequency.
    //! Priority admits new work; only native safety interrupts a committed tape.
    //! Explicit cancellation and completion release both input and decision locks.
    /// Native grab startup, pull, hold, pummel, release and throw belong to the
    /// player's grab interaction. No automated punish may inject a throw here.
    ///
    ///  Native grab startup, pull, hold, pummel, release and throw belong to the player's grab interaction. No automated punish may inject a throw here.
    /// ### Arguments
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn native_grab_active(status: i32) -> bool {
        matches!(
            status,
            crate::native_states::CATCH..=crate::native_states::THROW
        )
    }

    /// The owner of a committed action,
    /// determining which gameplay source controls the input.
    ///
///  The owner of a committed action, determining which gameplay source controls the input.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Owner {
        /// Represents the `Safety` case of `Owner`.
        ///
        ///
///  Represents the Safety option in Owner.
        Safety,
        /// Represents the `GrabDodge` case of `Owner`.
        ///
        ///
///  Represents the GrabDodge option in Owner.
        GrabDodge,
        /// Represents the `OptionRead` case of `Owner`.
        ///
        ///
///  Represents the OptionRead option in Owner.
        OptionRead,
        /// Represents the `Trip` case of `Owner`.
        ///
        ///
///  Represents the Trip option in Owner.
        Trip,
        /// Represents the `Fair` case of `Owner`.
        ///
        ///
///  Represents the Fair option in Owner.
        Fair,
        /// Represents the `Trump` case of `Owner`.
        ///
        ///
///  Represents the Trump option in Owner.
        Trump,
        /// Represents the `Ledge` case of `Owner`.
        ///
        ///
///  Represents the Ledge option in Owner.
        Ledge,
        /// Represents the `RollRead` case of `Owner`.
        ///
        ///
///  Represents the RollRead option in Owner.
        RollRead,
        /// Represents the `Defense` case of `Owner`.
        ///
        ///
///  Represents the Defense option in Owner.
        Defense,
        /// Represents the `Turbo` case of `Owner`.
        ///
        ///
///  Represents the Turbo option in Owner.
        Turbo,
        /// Represents the `Posture` case of `Owner`.
        ///
        ///
///  Represents the Posture option in Owner.
        Posture,
        /// Represents the `OutOfShield` case of `Owner`.
        ///
        ///
///  Represents the OutOfShield option in Owner.
        OutOfShield,
        /// Represents the `Reflect` case of `Owner`.
        ///
        ///
///  Represents the Reflect option in Owner.
        Reflect,
    }

    /// Select the base physical escape request for automatic special owners.
    /// Shield grabs and authored ledge/button tapes retain their own controls.
    ///
    ///  Select the base physical escape request for automatic special owners. Shield grabs and authored ledge/button tapes retain their own controls.
    /// ### Arguments
    /// - `owner` (`Option<Owner>`): Represents the owner associated with the value; the function uses it as input to the operation described above.
    /// - `roll` (`u32`): Represents the roll; the function uses it as input to the operation described above.
    /// - `option` (`u32`): Represents the option; the function uses it as input to the operation described above.
    /// - `shield_up_b` (`Option<u32>`): Represents the shield up b; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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
        ///
        ///
        ///  Gets the priority result.
        /// ### Arguments
        /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns the resulting `u8` value.
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
        ///
        ///
        ///  Checks whether exclusive.
        /// ### Arguments
        /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn exclusive(self) -> bool {
            !matches!(self, Self::Defense | Self::Turbo | Self::Posture)
        }
    }

    /// Complete the selected packet at the shared output boundary. Even a neutral
    /// wait/release owns all gameplay sources; absent axes mean neutral, not human
    /// passthrough. Passive movement/posture helpers deliberately remain partial.
    ///
    ///  Complete the selected packet at the shared output boundary. Even a neutral wait/release owns all gameplay sources; absent axes mean neutral, not human passthrough. Passive movement/posture helpers deliberately remain partial.
    /// ### Arguments
    /// - `owner` (`Option<Owner>`): Represents the owner associated with the value; the function uses it as input to the operation described above.
    /// - `committed` (`bool`): Represents the committed; the function uses it as input to the operation described above.
    /// - `output` (`crate::gameplay::ControllerOutput`): Represents the output data being produced; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `crate::gameplay::ControllerOutput` value.
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
    ///
    ///  Exactly one native safety phase owns a packet. Capture mash, final launch DI, imminent tech, then early SDI is the fixed precedence. A losing tech proposal receives no mapper acknowledgment and cannot spend its lockout.
    /// ### Arguments
    /// - `tech` (`crate::tech_input::TechOutput`): Represents the tech; the function uses it as input to the operation described above.
    /// - `mash` (`Option<[f32; 2]>`): Represents the mash; the function uses it as input to the operation described above.
    /// - `di` (`Option<[f32; 2]>`): Represents the di; the function uses it as input to the operation described above.
    /// - `sdi` (`Option<[f32; 2]>`): Represents the sdi; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `( crate::gameplay::ControllerOutput, Option<crate::tech_input::TechOutput>, )` value.
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
    ///
    ///  Only an independently validated imminent tech press can displace launch DI. No sticks/buttons are combined; DI resumes on the Guard release packet.
    /// ### Arguments
    /// - `tech` (`crate::tech_input::TechOutput`): Represents the tech; the function uses it as input to the operation described above.
    /// - `mash` (`Option<[f32; 2]>`): Represents the mash; the function uses it as input to the operation described above.
    /// - `di` (`Option<[f32; 2]>`): Represents the di; the function uses it as input to the operation described above.
    /// - `sdi` (`Option<[f32; 2]>`): Represents the sdi; the function uses it as input to the operation described above.
    /// - `urgent` (`bool`): Represents the urgent; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `( crate::gameplay::ControllerOutput, Option<crate::tech_input::TechOutput>, )` value.
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
    ///
///  Represents the intent of an action owner, including whether it is present and committed.
    #[derive(Clone, Copy)]
    pub struct Intent {
        /// Stores the `owner` value in `Intent` as `Owner`.
        ///
        ///
///  Holds the owner value used by Intent.
        pub owner: Owner,
        /// Stores the `present` value in `Intent` as `bool`.
        ///
        ///
///  Holds the present value used by Intent.
        pub present: bool,
        /// Stores the `committed` value in `Intent` as `bool`.
        ///
        ///
///  Holds the committed value used by Intent.
        pub committed: bool,
    }
    impl Intent {
        /// The final neutral release still belongs to the sequence, even after its
        /// interpreter has finished. Repeated mapper polls must deliver that release
        /// before a competing owner can take over on the next observation.
        /// A bounded native receipt keeps the reservation and emits an exclusive
        /// neutral packet at the shared output boundary until success or expiry.
        ///
        ///  The final neutral release still belongs to the sequence, even after its interpreter has finished. Repeated mapper polls must deliver that release before a competing owner can take over on the next observation. A bounded native.
        /// ### Arguments
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// - `owned_buttons` (`u32`): Represents the owned buttons; the function uses it as input to the operation described above.
        /// - `receipt` (`bool`): Represents the receipt; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub fn sequence_receipt(owner: Owner, owned_buttons: u32, receipt: bool) -> Self {
            Self {
                owner,
                present: owned_buttons != 0 || receipt,
                committed: owned_buttons != 0 || receipt,
            }
        }
        /// Returns the `input_sequence` result.
        ///
        ///
        ///  Gets the input sequence result.
        /// ### Arguments
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// - `owned_buttons` (`u32`): Represents the owned buttons; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub fn input_sequence(owner: Owner, owned_buttons: u32) -> Self {
            Self::sequence_receipt(owner, owned_buttons, false)
        }
    }

    /// Tracks the ownership status of various action sources, including ledge priority and manual takeover.
    ///
///  Tracks the ownership status of various action sources, including ledge priority and manual takeover.
    #[derive(Default)]
    pub struct ActionOwnership {
        /// Stores the `locked` value in `ActionOwnership` as `Option<Owner>`.
        ///
        ///
///  Optionally stores the locked for ActionOwnership; it may be absent.
        locked: Option<Owner>,
        /// Stores the `ledge_priority` value in `ActionOwnership` as `bool`.
        ///
        ///
///  Holds the ledge priority value used by ActionOwnership.
        ledge_priority: bool,
        /// Stores the `automatic_suspended` value in `ActionOwnership` as `bool`.
        ///
        ///
///  Holds the automatic suspended value used by ActionOwnership.
        automatic_suspended: bool,
        /// Stores the `manual` value in `ActionOwnership` as `crate::manual_controls::ManualTakeover`.
        ///
        ///
///  Holds the manual value used by ActionOwnership.
        manual: crate::manual_controls::ManualTakeover,
        /// Stores the `locked_manual` value in `ActionOwnership` as `Option<crate::manual_controls::ManualTakeover>`.
        ///
        ///
///  Optionally stores the locked manual for ActionOwnership; it may be absent.
        locked_manual: Option<crate::manual_controls::ManualTakeover>,
    }
    impl ActionOwnership {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
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
        ///
        ///  An observed ledge visit reserves admission ahead of new general reads. An existing committed sequence finishes first; native safety still wins. Configured actions all share this gate; future action owners are blocked by.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `active` (`bool`): Represents the active; the function uses it as input to the operation described above.
        pub fn set_ledge_priority(&mut self, active: bool) {
            self.ledge_priority = active;
        }

        /// Determines whether the `ledge_priority` condition holds.
        ///
        ///
        ///  Checks whether ledge priority.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn ledge_priority(&self) -> bool {
            self.ledge_priority
        }
        /// Determines whether the `ledge_compatible` condition holds.
        ///
        ///
        ///  Checks whether ledge compatible.
        /// ### Arguments
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        fn ledge_compatible(owner: Owner) -> bool {
            matches!(
                owner,
                Owner::Ledge | Owner::Safety | Owner::Defense | Owner::Turbo | Owner::Posture
            )
        }
        /// Updates the `unlock` state using the supplied inputs.
        ///
        ///
        ///  Updates unlock.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        fn unlock(&mut self) {
            self.locked = None;
            self.locked_manual = None;
        }
        /// Release only this owner when native startup/cancellation has ended it.
        ///
        ///  Release only this owner when native startup/cancellation has ended it.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        pub fn release(&mut self, owner: Owner) {
            if self.locked == Some(owner) {
                self.unlock();
            }
        }
        /// Transfer an explicitly chained response after native confirmation. This
        /// retains the input snapshot; unrelated proposals must use select().
        ///
        ///  Transfer an explicitly chained response after native confirmation. This retains the input snapshot; unrelated proposals must use select().
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `from` (`Owner`): Represents the from; the function uses it as input to the operation described above.
        /// - `to` (`Owner`): Represents the to; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn handoff(&mut self, from: Owner, to: Owner) -> bool {
            if self.locked != Some(from) {
                return false;
            }
            self.locked = Some(to);
            true
        }
        /// Current committed owner; preparation with no committed input is None.
        ///
        ///  Current committed owner; preparation with no committed input is None.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
        pub fn current_owner(&self) -> Option<Owner> {
            self.locked
        }
        /// Checks whether the supplied owner is managed automatically.
        ///
        ///
        ///  Reports whether this owner uses automatic handling.
        /// ### Arguments
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
        ///
        ///
        ///  Checks whether automatic active.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn automatic_active(&self) -> bool {
            self.locked.is_some_and(Self::automatic)
        }
        /// Observe fresh physical intent continuously, but freeze the automatic
        /// admission/movement view for the lifetime of the committed owner. Explicit
        /// until-manual recipe steps still read physical_inputs().epoch().
        ///
        ///  Observe fresh physical intent continuously, but freeze the automatic admission/movement view for the lifetime of the committed owner. Explicit until-manual recipe steps still read physicalinputs().epoch().
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
        /// - `sticks` (`[f32; 4]`): Represents the sticks; the function uses it as input to the operation described above.
        pub fn observe_manual(&mut self, buttons: u32, sticks: [f32; 4]) {
            self.manual.sample(buttons, sticks);
        }
        /// Returns the `physical_inputs` result.
        ///
        ///
        ///  Gets the physical inputs result.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns the resulting `&crate::manual_controls::ManualTakeover` value.
        pub fn physical_inputs(&self) -> &crate::manual_controls::ManualTakeover {
            &self.manual
        }
        /// Returns the `policy_inputs` result.
        ///
        ///
        ///  Gets the policy inputs result.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns the resulting `&crate::manual_controls::ManualTakeover` value.
        pub fn policy_inputs(&self) -> &crate::manual_controls::ManualTakeover {
            self.locked_manual.as_ref().unwrap_or(&self.manual)
        }
        /// Suspension prevents new automatic work. Cancellation of an active tape
        /// is explicit (disable, scene loss, native interruption or recipe handback).
        ///
        ///  Suspension prevents new automatic work. Cancellation of an active tape is explicit (disable, scene loss, native interruption or recipe handback).
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `active` (`bool`): Represents the active; the function uses it as input to the operation described above.
        pub fn set_automatic_suspended(&mut self, active: bool) {
            self.automatic_suspended = active;
        }
        /// Determines whether the `automatic_suspended` condition holds.
        ///
        ///
        ///  Checks whether automatic suspended.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn automatic_suspended(&self) -> bool {
            self.automatic_suspended
        }
        /// Scope-independent admission gates shared by every owner.
        ///
        ///  Scope-independent admission gates shared by every owner.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        fn admissible(&self, owner: Owner) -> bool {
            (!self.automatic_suspended || !Self::automatic(owner))
                && (!self.ledge_priority || Self::ledge_compatible(owner))
        }
        /// Priority admits new work; it never replaces an in-flight tape. Native
        /// safety cancels a tape whose gameplay prerequisites have been lost.
        ///
        ///  Priority admits new work; it never replaces an in-flight tape. Native safety cancels a tape whose gameplay prerequisites have been lost.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn allows(&self, owner: Owner) -> bool {
            self.locked == Some(owner)
                || (self.admissible(owner) && self.locked.is_none_or(|_| owner == Owner::Safety))
        }

        /// Check admission after releasing a particular owner without changing state.
        ///
        ///  Check admission after releasing a particular owner without changing state.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `owner` (`Owner`): Represents the owner associated with the value; the function uses it as input to the operation described above.
        /// - `releasing` (`Owner`): Represents the releasing; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn allows_after_release(&self, owner: Owner, releasing: Owner) -> bool {
            if self.locked != Some(releasing) {
                return self.allows(owner);
            }
            self.admissible(owner)
        }

        /// Returns an optional `select` value.
        ///
        ///
        ///  Gets select if it is available.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `valid_scope` (`bool`): Represents the valid scope; the function uses it as input to the operation described above.
        /// - `intents` (`&[Intent]`): Represents the intents; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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
///  A set of logical buttons the assist is allowed to add.
    pub struct ButtonSet(
        /// Bit mask containing logical button permissions.
        ///
        ///
///  Tracks which assist buttons are allowed.
        u16,
    );

    impl ButtonSet {
        /// Defines the `EMPTY` constant used by this module.
        ///
        ///
///  Provides the EMPTY constant value.
        pub const EMPTY: Self = Self(0);
        /// Defines the `ATTACK` constant used by this module.
        ///
        ///
///  Provides the ATTACK constant value.
        pub const ATTACK: Self = Self(1 << 0);
        /// Defines the `SPECIAL` constant used by this module.
        ///
        ///
///  Provides the SPECIAL constant value.
        pub const SPECIAL: Self = Self(1 << 1);
        /// Defines the `JUMP` constant used by this module.
        ///
        ///
///  Provides the JUMP constant value.
        pub const JUMP: Self = Self(1 << 2);
        /// Defines the `SHIELD` constant used by this module.
        ///
        ///
///  Provides the SHIELD constant value.
        pub const SHIELD: Self = Self(1 << 3);
        /// Defines the `GRAB` constant used by this module.
        ///
        ///
///  Provides the GRAB constant value.
        pub const GRAB: Self = Self(1 << 4);
        /// Defines the `DPAD_UP` constant used by this module.
        ///
        ///
///  Provides the DPAD UP constant value.
        pub const DPAD_UP: Self = Self(1 << 5);
        /// Defines the `DPAD_DOWN` constant used by this module.
        ///
        ///
///  Provides the DPAD DOWN constant value.
        pub const DPAD_DOWN: Self = Self(1 << 6);
        /// Defines the `DPAD_LEFT` constant used by this module.
        ///
        ///
///  Provides the DPAD LEFT constant value.
        pub const DPAD_LEFT: Self = Self(1 << 7);
        /// Defines the `DPAD_RIGHT` constant used by this module.
        ///
        ///
///  Provides the DPAD RIGHT constant value.
        pub const DPAD_RIGHT: Self = Self(1 << 8);

        // & keeps bits shared by both masks. Comparing against `other` asks whether
        // ALL its bits are present, so contains(ATTACK.union(JUMP)) needs both.
        /// Determines whether the `contains` condition holds.
        ///
        ///
        ///  Checks whether contains.
        /// ### Arguments
        /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `other` (`Self`): Represents the other; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub const fn contains(self, other: Self) -> bool {
            (self.0 & other.0) == other.0
        }

        // Bitwise OR combines sets: ATTACK | JUMP means both are held.
        /// Returns the `union` result.
        ///
        ///
        ///  Gets the union result.
        /// ### Arguments
        /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `other` (`Self`): Represents the other; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub const fn union(self, other: Self) -> Self {
            Self(self.0 | other.0)
        }

        /// Returns the `difference` result.
        ///
        ///
        ///  Gets the difference result.
        /// ### Arguments
        /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `other` (`Self`): Represents the other; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub const fn difference(self, other: Self) -> Self {
            Self(self.0 & !other.0)
        }

        /// Checks whether empty.
        ///
        ///
        ///  Checks whether empty.
        /// ### Arguments
        /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub const fn is_empty(self) -> bool {
            self.0 == 0
        }
    }

    /// A normalized logical stick position. Its numeric range is adapter-defined.
    ///
///  A normalized logical stick position. Its numeric range is adapter-defined.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct StickPosition {
        /// Stores the `x` value in `StickPosition` as `i16`.
        ///
        ///
///  Holds the x value used by StickPosition.
        pub x: i16,
        /// Stores the `y` value in `StickPosition` as `i16`.
        ///
        ///
///  Holds the y value used by StickPosition.
        pub y: i16,
    }

    impl StickPosition {
        /// Defines the `NEUTRAL` constant used by this module.
        ///
        ///
///  Provides the NEUTRAL constant value.
        pub const NEUTRAL: Self = Self { x: 0, y: 0 };
        /// Defines the `LEFT` constant used by this module.
        ///
        ///
///  Provides the LEFT constant value.
        pub const LEFT: Self = Self { x: -1, y: 0 };
        /// Defines the `RIGHT` constant used by this module.
        ///
        ///
///  Provides the RIGHT constant value.
        pub const RIGHT: Self = Self { x: 1, y: 0 };
        /// Defines the `UP` constant used by this module.
        ///
        ///
///  Provides the UP constant value.
        pub const UP: Self = Self { x: 0, y: 1 };
        /// Defines the `DOWN` constant used by this module.
        ///
        ///
///  Provides the DOWN constant value.
        pub const DOWN: Self = Self { x: 0, y: -1 };
    }

    /// A logical input snapshot used by the generic adapter and examples.
    /// It is NOT layout-compatible with the game's native Controller allocation.
    ///
///  A logical input snapshot used by the generic adapter and examples. It is NOT layout-compatible with the game's native Controller allocation.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ControllerFrame {
        /// Stores the `buttons` value in `ControllerFrame` as `ButtonSet`.
        ///
        ///
///  Holds the buttons value used by ControllerFrame.
        pub buttons: ButtonSet,
        /// Stores the `left_stick` value in `ControllerFrame` as `StickPosition`.
        ///
        ///
///  Holds the left stick value used by ControllerFrame.
        pub left_stick: StickPosition,
        /// Stores the `right_stick` value in `ControllerFrame` as `StickPosition`.
        ///
        ///
///  Holds the right stick value used by ControllerFrame.
        pub right_stick: StickPosition,
        /// Stores the `left_trigger` value in `ControllerFrame` as `u8`.
        ///
        ///
///  Holds the left trigger value used by ControllerFrame.
        pub left_trigger: u8,
        /// Stores the `right_trigger` value in `ControllerFrame` as `u8`.
        ///
        ///
///  Holds the right trigger value used by ControllerFrame.
        pub right_trigger: u8,
    }

    /// Whether an action leaves movement alone or requests a left-stick position.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    // This enum distinguishes "do not touch the player's stick" from "actively
    // request neutral". A neutral Request is still an override requiring permission.
    ///
///  Whether an action leaves movement alone or requests a left-stick position.
    pub enum LeftStickIntent {
        #[default]
        /// Represents the `PreservePlayer` case of `LeftStickIntent`.
        ///
        ///
///  Represents the PreservePlayer option in LeftStickIntent.
        PreservePlayer,
        /// Represents the `Request` case of `LeftStickIntent`.
        ///
        ///
///  Represents the Request option in LeftStickIntent.
        Request(
            /// Carries the requested stick position in `LeftStickIntent`.
            ///
            ///
///  Holds the requested stick position for this `LeftStickIntent` choice.
            StickPosition,
        ),
    }

    /// A named, inspectable recipe for one logical controller frame.
    ///
///  A named, inspectable recipe for one logical controller frame.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct InputRecipe {
        /// Stores the `buttons_to_add` value in `InputRecipe` as `ButtonSet`.
        ///
        ///
///  Holds the buttons to add value used by InputRecipe.
        pub buttons_to_add: ButtonSet,
        /// Stores the `left_stick` value in `InputRecipe` as `LeftStickIntent`.
        ///
        ///
///  Holds the left stick value used by InputRecipe.
        pub left_stick: LeftStickIntent,
    }

    impl InputRecipe {
        /// Defines the `NONE` constant used by this module.
        ///
        ///
///  Provides the NONE constant value.
        pub const NONE: Self = Self {
            buttons_to_add: ButtonSet::EMPTY,
            left_stick: LeftStickIntent::PreservePlayer,
        };

        /// Returns the `buttons` result.
        ///
        ///
        ///  Gets the buttons result.
        /// ### Arguments
        /// - `buttons_to_add` (`ButtonSet`): Represents the buttons to add; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub const fn buttons(buttons_to_add: ButtonSet) -> Self {
            Self {
                buttons_to_add,
                left_stick: LeftStickIntent::PreservePlayer,
            }
        }

        /// Returns the `buttons_with_left_stick` result.
        ///
        ///
        ///  Gets the buttons with left stick result.
        /// ### Arguments
        /// - `buttons_to_add` (`ButtonSet`): Represents the buttons to add; the function uses it as input to the operation described above.
        /// - `left_stick` (`StickPosition`): Represents the left stick; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
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
    ///
///  Explicit permissions for arbitration. Keep the default button-only policy until a directional input path is independently tested.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct InputPermissions {
        /// Stores the `allow_left_stick_override` value in `InputPermissions` as `bool`.
        ///
        ///
///  Holds the allow left stick override value used by InputPermissions.
        pub allow_left_stick_override: bool,
    }

    /// The non-destructive result of applying a recipe to player input.
    ///
///  The non-destructive result of applying a recipe to player input.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct ArbitrationResult {
        /// Stores the `frame` value in `ArbitrationResult` as `ControllerFrame`.
        ///
        ///
///  Holds the frame value used by ArbitrationResult.
        pub frame: ControllerFrame,
        /// Stores the `added_buttons` value in `ArbitrationResult` as `ButtonSet`.
        ///
        ///
///  Holds the added buttons value used by ArbitrationResult.
        pub added_buttons: ButtonSet,
        /// Stores the `blocked_left_stick_request` value in `ArbitrationResult` as `bool`.
        ///
        ///
///  Holds the blocked left stick request value used by ArbitrationResult.
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
    ///
    ///  Applies one assist recipe to the player's input for this frame. Requested buttons are added to held player buttons; an empty request adds nothing. The left stick stays with the player unless an override is allowed. The right.
    /// ### Arguments
    /// - `user` (`ControllerFrame`): Represents the user; the function uses it as input to the operation described above.
    /// - `recipe` (`InputRecipe`): Represents the input recipe to use; the function uses it as input to the operation described above.
    /// - `permissions` (`InputPermissions`): Represents the permissions; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ArbitrationResult` value.
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

        /// Defines the `PLAYER` constant used by this module.
        ///
        ///
///  Provides the PLAYER constant value.
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
        ///
        ///
///  Checks that buttons are added without taking movement.
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
        ///
        ///
///  Checks that stick request requires explicit permission.
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

    /// Defines the available `ReceiptAction` cases.
    ///
    ///
///  Lists the available ReceiptAction choices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum ReceiptAction {
        /// Represents the `Continue` case of `ReceiptAction`.
        ///
        ///
///  Represents the Continue option in ReceiptAction.
        Continue,
        /// Represents the `Confirm` case of `ReceiptAction`.
        ///
        ///
///  Represents the Confirm option in ReceiptAction.
        Confirm,
        /// Represents the `Wait` case of `ReceiptAction`.
        ///
        ///
///  Represents the Wait option in ReceiptAction.
        Wait,
        /// Represents the `Cancel` case of `ReceiptAction`.
        ///
        ///
///  Represents the Cancel option in ReceiptAction.
        Cancel,
    }

    /// Checks whether a pending operation is stored.
    ///
    ///
    ///  Checks whether something is waiting to be processed.
    /// ### Arguments
    /// - `seen` (`bool`): Represents the seen; the function uses it as input to the operation described above.
    /// - `sent` (`bool`): Represents the sent; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub(crate) fn pending(seen: bool, sent: bool) -> bool {
        !seen && sent
    }

    /// Determines whether the `manual_handback` condition holds.
    ///
    ///
    ///  Checks whether manual handback.
    /// ### Arguments
    /// - `automatic` (`bool`): Represents the automatic; the function uses it as input to the operation described above.
    /// - `seen` (`bool`): Represents the seen; the function uses it as input to the operation described above.
    /// - `context` (`GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `epoch` (`u64`): Represents the epoch; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub(crate) fn manual_handback(
        automatic: bool,
        seen: bool,
        context: GameplayContext,
        epoch: u64,
    ) -> bool {
        automatic && seen && context.manual_input_epoch != epoch
    }

    /// Returns the `evaluate` result.
    ///
    ///
    ///  Gets the evaluate result.
    /// ### Arguments
    /// - `seen` (`bool`): Represents the seen; the function uses it as input to the operation described above.
    /// - `native` (`bool`): Represents the native; the function uses it as input to the operation described above.
    /// - `waiting` (`bool`): Represents the waiting; the function uses it as input to the operation described above.
    /// - `from_dodge` (`bool`): Represents the from dodge; the function uses it as input to the operation described above.
    /// - `context` (`GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `receipt` (`&UpBReceipt`): Represents the receipt; the function uses it as input to the operation described above.
    /// - `delivery` (`&DeliveryBudget`): Represents the delivery; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ReceiptAction` value.
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
        ///
        ///
///  Checks that receipt preserves startup wait and cancellation gates.
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
