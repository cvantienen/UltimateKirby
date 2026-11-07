//! One committed action owner, independent of controller polling frequency.
//! Priority admits new work; only native safety interrupts a committed tape.
//! Explicit cancellation and completion release both input and decision locks.
/// Native grab startup, pull, hold, pummel, release and throw belong to the
/// player's grab interaction. No automated punish may inject a throw here.
pub fn native_grab_active(status:i32)->bool {matches!(status,0x37..=0x40)}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Safety,
    GrabDodge,
    OptionRead,
    Trip,
    JabLock,
    Finisher,
    Fair,
    Trump,
    Slide,
    Shortcut,
    Ledge,
    RollRead,
    Defense,
    Turbo,
    Posture,
    OutOfShield,
    Reflect,
}
/// Select the base physical escape request for automatic special owners.
/// Committed Bowser Up-B bypasses this policy via bowser_up_b_owner; shield
/// grabs and authored ledge/button tapes retain their own controls.
pub fn automatic_escape_request(owner:Option<Owner>,roll:u32,option:u32,shield_up_b:Option<u32>)->Option<u32> {
    match owner {
        Some(Owner::RollRead)=>Some(roll),
        Some(Owner::OptionRead)=>Some(option),
        Some(Owner::OutOfShield)=>shield_up_b,
        _=>None,
    }
}
/// Identify an already selected Bowser Up-B for native tracking observations.
/// Fail closed for mixed owners; generic input exclusivity applies to all fighters.
pub fn bowser_up_b_owner(kind: Option<i32>, owner: Option<Owner>, roll: bool, option: bool, shield: bool) -> Option<Owner> {
    let active = match (kind, roll, option, shield) {
        (Some(15), true, false, false) => Owner::RollRead,
        (Some(15), false, true, false) => Owner::OptionRead,
        (Some(15), false, false, true) => Owner::OutOfShield,
        _ => return None,
    };
    (owner.is_none() || owner == Some(active)).then_some(active)
}
impl Owner {
    fn priority(self) -> u8 {
        match self {
            Self::Safety => 100,
            Self::GrabDodge => 99,
            Self::OptionRead => 98,
            Self::Trip => 90,
            Self::JabLock => 75,
            Self::Finisher => 85,
            Self::Fair => 80,
            Self::Trump => 70,
            Self::Slide => 65,
            Self::Shortcut => 60,
            Self::Ledge => 95,
            Self::RollRead => 97,
            Self::Defense => 30,
            Self::Turbo => 10,
            Self::Posture => 5,
            Self::OutOfShield => 92,
            Self::Reflect => 35,
        }
    }
    fn interrupts(self, _current: Self) -> bool {
        // Priority admits new work; it never replaces an in-flight tape. Native
        // safety cancels a tape whose gameplay prerequisites have been lost.
        self == Self::Safety
    }
    pub fn exclusive(self) -> bool {
        !matches!(self, Self::Defense | Self::Turbo | Self::Posture)
    }
}

/// Complete the selected packet at the shared output boundary. Even a neutral
/// wait/release owns all gameplay sources; absent axes mean neutral, not human
/// passthrough. Passive movement/posture helpers deliberately remain partial.
pub fn exclusive_output(owner: Option<Owner>, committed: bool, mut output: crate::ledge_trap::LedgeOutput) -> crate::ledge_trap::LedgeOutput {
    if committed && owner.is_some_and(Owner::exclusive) {
        output.owned |= crate::ledge_trap::SEQUENCE_BUTTONS;
        output.stick = Some(output.stick.unwrap_or([0.; 2]));
    }
    output
}

/// Exactly one native safety phase owns a packet. Capture mash, final launch
/// DI, imminent tech, then early SDI is the fixed precedence. A losing tech
/// proposal receives no mapper acknowledgment and cannot spend its lockout.
pub fn safety_output(tech: crate::tech_input::TechOutput, mash: Option<[f32; 2]>, di: Option<[f32; 2]>, sdi: Option<[f32; 2]>) -> (crate::ledge_trap::LedgeOutput, Option<crate::tech_input::TechOutput>) {
    safety_output_with_deadline(tech,mash,di,sdi,false)
}
/// Only an independently validated imminent tech press can displace launch DI.
/// No sticks/buttons are combined; DI resumes on the Guard release packet.
pub fn safety_output_with_deadline(tech: crate::tech_input::TechOutput, mash: Option<[f32; 2]>, di: Option<[f32; 2]>, sdi: Option<[f32; 2]>, urgent:bool) -> (crate::ledge_trap::LedgeOutput, Option<crate::tech_input::TechOutput>) {
    let tech = (mash.is_none() && (di.is_none() || urgent && tech.press) && (tech.press || tech.owns_guard)).then_some(tech);
    let output = crate::ledge_trap::LedgeOutput {
        requested: if tech.is_some_and(|t| t.press) { crate::native_input::MAPPER_ZR } else { 0 },
        stick: mash.or_else(|| if tech.is_none() {di.or(sdi)} else {None}),
        ..Default::default()
    };
    (output, tech)
}
#[derive(Clone, Copy)]
pub struct Intent {
    pub owner: Owner,
    pub present: bool,
    pub committed: bool,
}
impl Intent {
    /// The final neutral release still belongs to the sequence, even after its
    /// interpreter has finished. Repeated mapper polls must deliver that release
    /// before a competing owner can take over on the next observation.
    /// A bounded native receipt keeps the reservation and emits an exclusive
    /// neutral packet at the shared output boundary until success or expiry.
    pub fn sequence_receipt(owner:Owner,owned_buttons:u32,receipt:bool)->Self {
        Self{owner,present:owned_buttons!=0 || receipt,committed:owned_buttons!=0 || receipt}
    }
    pub fn input_sequence(owner: Owner, owned_buttons: u32) -> Self {
        Self {owner, present: owned_buttons != 0, committed: owned_buttons != 0}
    }
}
#[derive(Default)]
pub struct ActionOwnership {
    locked: Option<Owner>,
    ledge_priority: bool,
    automatic_suspended: bool,
    manual: crate::manual_controls::ManualTakeover,
    locked_manual: Option<crate::manual_controls::ManualTakeover>,
}
impl ActionOwnership {
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

    pub fn ledge_priority(&self) -> bool {
        self.ledge_priority
    }
    fn ledge_compatible(owner: Owner) -> bool {
        matches!(owner, Owner::Ledge | Owner::Safety | Owner::Defense | Owner::Turbo | Owner::Posture)
    }
    /// Release only this owner when native startup/cancellation has ended it.
    pub fn release(&mut self, owner: Owner) {
        if self.locked == Some(owner) { self.locked = None; self.locked_manual = None; }
    }
    /// Transfer an explicitly chained response after native confirmation. This
    /// retains the input snapshot; unrelated proposals must use select().
    pub fn handoff(&mut self, from: Owner, to: Owner) -> bool {
        if self.locked != Some(from) { return false; }
        self.locked = Some(to);
        true
    }
    /// Current committed owner; preparation with no committed input is None.
    pub fn current_owner(&self) -> Option<Owner> { self.locked }
    fn automatic(owner:Owner)->bool {
        matches!(owner,Owner::RollRead|Owner::OutOfShield|Owner::GrabDodge|Owner::OptionRead
            |Owner::Trip|Owner::JabLock|Owner::Fair|Owner::Finisher|Owner::Reflect)
    }
    pub fn automatic_active(&self) -> bool { self.locked.is_some_and(Self::automatic) }
    /// Observe fresh physical intent continuously, but freeze the automatic
    /// admission/movement view for the lifetime of the committed owner. Explicit
    /// until-manual recipe steps still read physical_inputs().epoch().
    pub fn observe_manual(&mut self, buttons: u32, sticks: [f32; 4]) {
        self.manual.sample(buttons, sticks);
    }
    pub fn physical_inputs(&self) -> &crate::manual_controls::ManualTakeover { &self.manual }
    pub fn policy_inputs(&self) -> &crate::manual_controls::ManualTakeover {
        self.locked_manual.as_ref().unwrap_or(&self.manual)
    }
    pub fn manual_jump_preempts(&self, _confirmed_spin: bool) -> bool { false }
    /// Suspension prevents new automatic work. Cancellation of an active tape
    /// is explicit (disable, scene loss, native interruption or recipe handback).
    pub fn set_automatic_suspended(&mut self, active: bool) {
        self.automatic_suspended = active;
    }
    pub fn automatic_suspended(&self)->bool{self.automatic_suspended}
    pub fn allows(&self, owner: Owner) -> bool {
        self.locked == Some(owner) ||
            ((!self.automatic_suspended || !Self::automatic(owner))
                && (!self.ledge_priority || Self::ledge_compatible(owner))
                && self.locked.is_none_or(|current| owner.interrupts(current)))
    }

    pub fn select(&mut self, valid_scope: bool, intents: &[Intent]) -> Option<Owner> {
        if !valid_scope {
            self.locked = None;
            self.locked_manual = None;
            self.ledge_priority = false;
            self.automatic_suspended = false;
            return None;
        }
        if self.locked.is_some_and(|owner| {
            !intents
                .iter()
                .any(|i| i.owner == owner && i.present && i.committed)
        }) {
            self.locked = None;
            self.locked_manual = None;
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

/// Built-in manual jump handback is available only without an active input
/// owner. A passive watch/posture/turbo may yield; safety and punish tapes do not.
pub fn permits_builtin_jump(owner:Option<Owner>, ledge_reserved:bool)->bool {
    match owner {
        None|Some(Owner::Posture|Owner::Turbo)=>true,
        Some(Owner::Ledge)=>!ledge_reserved,
        _=>false,
    }
}
