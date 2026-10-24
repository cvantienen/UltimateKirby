//! One committed action owner, independent of controller polling frequency.
//! Native safety responses and confirmed follow-ups have explicit interruption
//! rules. Lower-priority preparation never steals a committed sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Safety,
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
}
impl Owner {
    fn priority(self) -> u8 {
        match self {
            Self::Safety => 100,
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
        }
    }
    fn interrupts(self, current: Self) -> bool {
        if self == Self::Safety {
            return true;
        }
        if matches!(self, Self::OptionRead | Self::RollRead) {
            return !matches!(current, Self::Safety | Self::OptionRead | Self::RollRead);
        }
        // Neither timed punish may be interrupted by lower-priority tools.
        // If both have committed, finish the current sequence before switching.
        if matches!(
            current,
            Self::Safety | Self::OptionRead | Self::Ledge | Self::RollRead
        ) {
            return false;
        }
        if matches!(self, Self::Ledge | Self::RollRead) {
            return true;
        }
        self == Self::Trip
            || self == Self::Fair && matches!(current, Self::JabLock | Self::Defense | Self::Turbo)
            || matches!(
                self,
                Self::JabLock | Self::Finisher | Self::Trump | Self::Slide | Self::Shortcut
            ) && matches!(current, Self::JabLock | Self::Defense | Self::Turbo)
    }
}
#[derive(Clone, Copy)]
pub struct Intent {
    pub owner: Owner,
    pub present: bool,
    pub committed: bool,
}
#[derive(Default)]
pub struct ActionOwnership {
    locked: Option<Owner>,
}
impl ActionOwnership {
    pub const fn new() -> Self {
        Self { locked: None }
    }
    pub fn allows(&self, owner: Owner) -> bool {
        self.locked
            .is_none_or(|current| current == owner || owner.interrupts(current))
    }
    pub fn select(&mut self, valid_scope: bool, intents: &[Intent]) -> Option<Owner> {
        if !valid_scope {
            self.locked = None;
            return None;
        }
        if self.locked.is_some_and(|owner| {
            !intents
                .iter()
                .any(|i| i.owner == owner && i.present && i.committed)
        }) {
            self.locked = None;
        }
        let winner = intents
            .iter()
            .filter(|i| i.present && self.allows(i.owner))
            .max_by_key(|i| i.owner.priority());
        self.locked = winner.filter(|i| i.committed).map(|i| i.owner);
        winner.map(|i| i.owner)
    }
}
