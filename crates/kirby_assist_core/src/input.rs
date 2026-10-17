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
pub struct ButtonSet(u16);

impl ButtonSet {
    pub const EMPTY: Self = Self(0);
    pub const ATTACK: Self = Self(1 << 0);
    pub const SPECIAL: Self = Self(1 << 1);
    pub const JUMP: Self = Self(1 << 2);
    pub const SHIELD: Self = Self(1 << 3);
    pub const GRAB: Self = Self(1 << 4);
    pub const DPAD_UP: Self = Self(1 << 5);
    pub const DPAD_DOWN: Self = Self(1 << 6);
    pub const DPAD_LEFT: Self = Self(1 << 7);
    pub const DPAD_RIGHT: Self = Self(1 << 8);

    // & keeps bits shared by both masks. Comparing against `other` asks whether
    // ALL its bits are present, so contains(ATTACK.union(JUMP)) needs both.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    // Bitwise OR combines sets: ATTACK | JUMP means both are held.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// A normalized logical stick position. Its numeric range is adapter-defined.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StickPosition {
    pub x: i16,
    pub y: i16,
}

impl StickPosition {
    pub const NEUTRAL: Self = Self { x: 0, y: 0 };
    pub const LEFT: Self = Self { x: -1, y: 0 };
    pub const RIGHT: Self = Self { x: 1, y: 0 };
    pub const UP: Self = Self { x: 0, y: 1 };
    pub const DOWN: Self = Self { x: 0, y: -1 };
}

/// A logical input snapshot used by the generic adapter and examples.
/// It is NOT layout-compatible with the game's native Controller allocation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ControllerFrame {
    pub buttons: ButtonSet,
    pub left_stick: StickPosition,
    pub right_stick: StickPosition,
    pub left_trigger: u8,
    pub right_trigger: u8,
}

/// Whether an action leaves movement alone or requests a left-stick position.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
// This enum distinguishes "do not touch the player's stick" from "actively
// request neutral". A neutral Request is still an override requiring permission.
pub enum LeftStickIntent {
    #[default]
    PreservePlayer,
    Request(StickPosition),
}

/// A named, inspectable recipe for one logical controller frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputRecipe {
    pub buttons_to_add: ButtonSet,
    pub left_stick: LeftStickIntent,
}

impl InputRecipe {
    pub const NONE: Self = Self {
        buttons_to_add: ButtonSet::EMPTY,
        left_stick: LeftStickIntent::PreservePlayer,
    };

    pub const fn buttons(buttons_to_add: ButtonSet) -> Self {
        Self {
            buttons_to_add,
            left_stick: LeftStickIntent::PreservePlayer,
        }
    }

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
    pub allow_left_stick_override: bool,
}

/// The non-destructive result of applying a recipe to player input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArbitrationResult {
    pub frame: ControllerFrame,
    pub added_buttons: ButtonSet,
    pub blocked_left_stick_request: bool,
}

/// Merges a single assist recipe into player input.
///
/// Buttons are always additive. Stick requests are ignored unless explicitly
/// allowed in `InputPermissions`; triggers and the right stick are always the
/// player's values.
// Adding a new recipe field requires deciding how it merges here; never assume
// it should override the human input merely because a tool requested it.
pub const fn arbitrate(
    user: ControllerFrame,
    recipe: InputRecipe,
    permissions: InputPermissions,
) -> ArbitrationResult {
    // The tuple gives two names to one match result: the chosen stick and a
    // diagnostic flag. Pattern guards (`if ...`) select the permitted Request.
    let (left_stick, blocked_left_stick_request) = match recipe.left_stick {
        LeftStickIntent::PreservePlayer => (user.left_stick, false),
        LeftStickIntent::Request(position) if permissions.allow_left_stick_override => {
            (position, false)
        }
        LeftStickIntent::Request(_) => (user.left_stick, true),
    };

    ArbitrationResult {
        frame: ControllerFrame {
            buttons: user.buttons.union(recipe.buttons_to_add),
            left_stick,
            right_stick: user.right_stick,
            left_trigger: user.left_trigger,
            right_trigger: user.right_trigger,
        },
        added_buttons: recipe.buttons_to_add,
        blocked_left_stick_request,
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;

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
