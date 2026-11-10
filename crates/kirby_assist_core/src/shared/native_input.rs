//! Smash's raw controller buttons, before user-configured mapping.
//! Layout/accessor source: HDR a8f98eace1c6a7b45c88cb3f0ac1ef05426624dc,
//! dynamic/src/ext.rs (get_player_idx_from_boma and Controller).
//!
//! This is the final conversion layer: HID-named request bits become Smash's
//! internal button positions, then merge with a fresh physical ButtonTrace.
//! Game mapping remains responsible for turning physical buttons into actions.
//! To support another button, verify its native position, add a conversion, and
//! test held/pressed/released behavior plus preservation of unrelated inputs.
use crate::live_input::{HID_A, HID_B, HID_L, HID_R, HID_X, HID_Y, HID_ZL, HID_ZR};

// MappedInputs::buttons is a different word from ControlModule button IDs.
// Pinned HDR ext.rs a8f98eace1c6a7b45c88cb3f0ac1ef05426624dc, Buttons.
/// Defines the `MAPPED_SPECIAL` constant used by this module.
///
/// Provides the MAPPED SPECIAL constant value.
pub const MAPPED_SPECIAL: u32 = 0x2;
/// Defines the `MAPPED_ATTACK` constant used by this module.
///
/// Provides the MAPPED ATTACK constant value.
pub const MAPPED_ATTACK: u32 = 0x1;
/// Defines the `MAPPED_GUARD` constant used by this module.
///
/// Provides the MAPPED GUARD constant value.
pub const MAPPED_GUARD: u32 = 0x8;
/// Defines the `MAPPED_CATCH` constant used by this module.
///
/// Provides the MAPPED CATCH constant value.
pub const MAPPED_CATCH: u32 = 0x10;
/// Defines the `MAPPED_JUMP` constant used by this module.
///
/// Provides the MAPPED JUMP constant value.
pub const MAPPED_JUMP: u32 = 0x4;
/// Defines the `MAPPED_JUMP_MINI` constant used by this module.
///
/// Provides the MAPPED JUMP MINI constant value.
pub const MAPPED_JUMP_MINI: u32 = 0x40;
/// Defines the `MAPPED_GUARD_HOLD` constant used by this module.
///
/// Provides the MAPPED GUARD HOLD constant value.
pub const MAPPED_GUARD_HOLD: u32 = 0x10000;
/// Defines the `MAPPED_TAUNT_UP` constant used by this module.
///
/// Provides the MAPPED TAUNT UP constant value.
pub const MAPPED_TAUNT_UP: u32 = 0x400;
/// Defines the `MAPPED_TAUNT_DOWN` constant used by this module.
///
/// Provides the MAPPED TAUNT DOWN constant value.
pub const MAPPED_TAUNT_DOWN: u32 = 0x1000;
/// Defines the `MAPPED_TAUNT_SIDES` constant used by this module.
///
/// Provides the MAPPED TAUNT SIDES constant value.
pub const MAPPED_TAUNT_SIDES: u32 = 0x6000;
/// Defines the `MAPPED_ATTACK_RAW` constant used by this module.
///
/// Provides the MAPPED ATTACK RAW constant value.
pub const MAPPED_ATTACK_RAW: u32 = 0x200;
/// Defines the `MAPPED_ATTACK_HOLD_BITS` constant used by this module.
///
/// Provides the MAPPED ATTACK HOLD BITS constant value.
pub const MAPPED_ATTACK_HOLD_BITS: u32 = MAPPED_ATTACK | MAPPED_ATTACK_RAW;
/// Defines the `MAPPED_CSTICK_BUTTON` constant used by this module.
///
/// Provides the MAPPED CSTICK BUTTON constant value.
const MAPPED_CSTICK_BUTTON: u32 = 0x80;
/// `NPAD_ID_HANDHELD` from libnx's `hid.h`.
///
///  NPADIDHANDHELD from libnx's hid.h.
pub const HANDHELD_NPAD_ID: u32 = 0x20;

/// Defines the `MAPPER_SLOT_COUNT` constant used by this module.
///
/// Provides the MAPPER SLOT COUNT constant value.
pub const MAPPER_SLOT_COUNT: i32 = 8;
/// Defines the `MAPPER_DPAD_UP` constant used by this module.
///
/// Provides the MAPPER DPAD UP constant value.
pub const MAPPER_DPAD_UP: u32 = 1;
/// Defines the `MAPPER_DPAD_RIGHT` constant used by this module.
///
/// Provides the MAPPER DPAD RIGHT constant value.
pub const MAPPER_DPAD_RIGHT: u32 = 1 << 1;
/// Defines the `MAPPER_DPAD_DOWN` constant used by this module.
///
/// Provides the MAPPER DPAD DOWN constant value.
pub const MAPPER_DPAD_DOWN: u32 = 1 << 2;
/// Defines the `MAPPER_DPAD_LEFT` constant used by this module.
///
/// Provides the MAPPER DPAD LEFT constant value.
pub const MAPPER_DPAD_LEFT: u32 = 1 << 3;
/// Defines the `MAPPER_DPAD_BUTTONS` constant used by this module.
///
/// Provides the MAPPER DPAD BUTTONS constant value.
pub const MAPPER_DPAD_BUTTONS: u32 =
    MAPPER_DPAD_UP | MAPPER_DPAD_RIGHT | MAPPER_DPAD_DOWN | MAPPER_DPAD_LEFT;
/// Defines the `MAPPER_STICK_CLICKS` constant used by this module.
///
/// Provides the MAPPER STICK CLICKS constant value.
pub const MAPPER_STICK_CLICKS: u32 = 3 << 16;
// Controller::ButtonBitfield l_*/r_* digital stick aliases in the pinned HDR
// layout. A stick flick includes these as well as its analog axis.
/// Defines the `MAPPER_LEFT_STICK_UP` constant used by this module.
///
/// Provides the MAPPER LEFT STICK UP constant value.
pub const MAPPER_LEFT_STICK_UP: u32 = 1 << 20;
/// Defines the `MAPPER_LEFT_STICK_RIGHT` constant used by this module.
///
/// Provides the MAPPER LEFT STICK RIGHT constant value.
pub const MAPPER_LEFT_STICK_RIGHT: u32 = 1 << 21;
/// Defines the `MAPPER_LEFT_STICK_DOWN` constant used by this module.
///
/// Provides the MAPPER LEFT STICK DOWN constant value.
pub const MAPPER_LEFT_STICK_DOWN: u32 = 1 << 22;
/// Defines the `MAPPER_LEFT_STICK_LEFT` constant used by this module.
///
/// Provides the MAPPER LEFT STICK LEFT constant value.
pub const MAPPER_LEFT_STICK_LEFT: u32 = 1 << 23;
/// Defines the `MAPPER_LEFT_STICK_HORIZONTAL` constant used by this module.
///
/// Provides the MAPPER LEFT STICK HORIZONTAL constant value.
pub const MAPPER_LEFT_STICK_HORIZONTAL: u32 = MAPPER_LEFT_STICK_RIGHT | MAPPER_LEFT_STICK_LEFT;
/// Defines the `MAPPER_LEFT_STICK_DIRECTIONS` constant used by this module.
///
/// Provides the MAPPER LEFT STICK DIRECTIONS constant value.
pub const MAPPER_LEFT_STICK_DIRECTIONS: u32 = 0xf << 20;
/// Defines the `MAPPER_RIGHT_STICK_UP` constant used by this module.
///
/// Provides the MAPPER RIGHT STICK UP constant value.
pub const MAPPER_RIGHT_STICK_UP: u32 = 1 << 24;
/// Defines the `MAPPER_RIGHT_STICK_RIGHT` constant used by this module.
///
/// Provides the MAPPER RIGHT STICK RIGHT constant value.
pub const MAPPER_RIGHT_STICK_RIGHT: u32 = 1 << 25;
/// Defines the `MAPPER_RIGHT_STICK_DOWN` constant used by this module.
///
/// Provides the MAPPER RIGHT STICK DOWN constant value.
pub const MAPPER_RIGHT_STICK_DOWN: u32 = 1 << 26;
/// Defines the `MAPPER_RIGHT_STICK_LEFT` constant used by this module.
///
/// Provides the MAPPER RIGHT STICK LEFT constant value.
pub const MAPPER_RIGHT_STICK_LEFT: u32 = 1 << 27;
/// Defines the `MAPPER_RIGHT_STICK_DIRECTIONS` constant used by this module.
///
/// Provides the MAPPER RIGHT STICK DIRECTIONS constant value.
pub const MAPPER_RIGHT_STICK_DIRECTIONS: u32 = 0xf << 24;
/// Defines the `MAPPER_EXTRA_BUTTON_CHANNELS` constant used by this module.
///
/// Provides the MAPPER EXTRA BUTTON CHANNELS constant value.
const MAPPER_EXTRA_BUTTON_CHANNELS: u32 = 0xf << 12;
/// Defines the `MODULE_ACCESSOR_CONTROL_MODULE_POINTER_OFFSET_BYTES` constant used by this module.
///
/// Provides the MODULE ACCESSOR CONTROL MODULE POINTER OFFSET BYTES constant value.
const MODULE_ACCESSOR_CONTROL_MODULE_POINTER_OFFSET_BYTES: usize = 0x48;
/// Defines the `CONTROL_MODULE_INPUT_CONTEXT_POINTER_OFFSET_BYTES` constant used by this module.
///
/// Provides the CONTROL MODULE INPUT CONTEXT POINTER OFFSET BYTES constant value.
const CONTROL_MODULE_INPUT_CONTEXT_POINTER_OFFSET_BYTES: usize = 0x118;
/// Defines the `INPUT_CONTEXT_CONTROLLER_RECORD_POINTER_OFFSET_BYTES` constant used by this module.
///
/// Provides the INPUT CONTEXT CONTROLLER RECORD POINTER OFFSET BYTES constant value.
const INPUT_CONTEXT_CONTROLLER_RECORD_POINTER_OFFSET_BYTES: usize = 0x58;
/// Defines the `CONTROLLER_RECORD_INDEX_POINTER_OFFSET_BYTES` constant used by this module.
///
/// Provides the CONTROLLER RECORD INDEX POINTER OFFSET BYTES constant value.
const CONTROLLER_RECORD_INDEX_POINTER_OFFSET_BYTES: usize = 8;
/// Defines the `INPUT_INDEX_FIELD_OFFSET_BYTES` constant used by this module.
///
/// Provides the INPUT INDEX FIELD OFFSET BYTES constant value.
const INPUT_INDEX_FIELD_OFFSET_BYTES: usize = 8;
/// Defines the `MINIMUM_NATIVE_POINTER_ADDRESS` constant used by this module.
///
/// Provides the MINIMUM NATIVE POINTER ADDRESS constant value.
const MINIMUM_NATIVE_POINTER_ADDRESS: usize = 0x1000;
/// Defines the `MAPPER_A` constant used by this module.
///
/// Provides the MAPPER A constant value.
pub const MAPPER_A: u32 = 1 << 5;
/// Defines the `MAPPER_B` constant used by this module.
///
/// Provides the MAPPER B constant value.
pub const MAPPER_B: u32 = 1 << 6;
/// Defines the `MAPPER_X` constant used by this module.
///
/// Provides the MAPPER X constant value.
pub const MAPPER_X: u32 = 1 << 4;
/// Defines the `MAPPER_Y` constant used by this module.
///
/// Provides the MAPPER Y constant value.
pub const MAPPER_Y: u32 = 1 << 7;
/// Defines the `MAPPER_L` constant used by this module.
///
/// Provides the MAPPER L constant value.
pub const MAPPER_L: u32 = 1 << 8;
/// Defines the `MAPPER_R` constant used by this module.
///
/// Provides the MAPPER R constant value.
pub const MAPPER_R: u32 = 1 << 9;
/// Defines the `MAPPER_ZL` constant used by this module.
///
/// Provides the MAPPER ZL constant value.
pub const MAPPER_ZL: u32 = 1 << 10;
/// Defines the `MAPPER_ZR` constant used by this module.
///
/// Provides the MAPPER ZR constant value.
pub const MAPPER_ZR: u32 = 1 << 11;
/// Defines the `MAPPER_PLUS` constant used by this module.
///
/// Provides the MAPPER PLUS constant value.
pub const MAPPER_PLUS: u32 = 1 << 18;
/// Defines the `MAPPER_MINUS` constant used by this module.
///
/// Provides the MAPPER MINUS constant value.
pub const MAPPER_MINUS: u32 = 1 << 19;
/// Defines the `MAPPER_MENU` constant used by this module.
///
/// Provides the MAPPER MENU constant value.
pub const MAPPER_MENU: u32 = MAPPER_PLUS | MAPPER_MINUS;
// Established recipe layout, BEFORE physical custom-button bindings. Brother
// translates these canonical inputs only at the existing physical overlay.
/// Defines the `SEQUENCE_JUMP_BUTTONS` constant used by this module.
///
/// Provides the SEQUENCE JUMP BUTTONS constant value.
pub const SEQUENCE_JUMP_BUTTONS: u32 = MAPPER_X | MAPPER_Y | MAPPER_ZL;
/// Defines the `SEQUENCE_GUARD_BUTTONS` constant used by this module.
///
/// Provides the SEQUENCE GUARD BUTTONS constant value.
pub const SEQUENCE_GUARD_BUTTONS: u32 = MAPPER_ZR;
/// Native Grab also expands to Guard/Attack; it is a guard-producing alias,
/// while canonical X/Y/ZL remain jump inputs.
///
///  Native Grab also expands to Guard/Attack; it is a guard-producing alias, while canonical X/Y/ZL remain jump inputs.
pub const SEQUENCE_GUARD_INPUTS: u32 = SEQUENCE_GUARD_BUTTONS | MAPPER_R;
// Native Controller channels outside the ordinary face/shoulder words.
/// Defines the `MAPPER_DIGITAL_L` constant used by this module.
///
/// Provides the MAPPER DIGITAL L constant value.
pub const MAPPER_DIGITAL_L: u32 = 1 << 28;
/// Defines the `MAPPER_DIGITAL_R` constant used by this module.
///
/// Provides the MAPPER DIGITAL R constant value.
pub const MAPPER_DIGITAL_R: u32 = 1 << 29;
/// Defines the `MAPPER_EXTRA_BUTTONS` constant used by this module.
///
/// Provides the MAPPER EXTRA BUTTONS constant value.
pub const MAPPER_EXTRA_BUTTONS: u32 =
    MAPPER_EXTRA_BUTTON_CHANNELS | MAPPER_DIGITAL_L | MAPPER_DIGITAL_R;
/// Native buttons that an assist policy may replace, excluding the D-pad.
///
///  Native buttons that an assist policy may replace, excluding the D-pad.
pub const ASSIST_BUTTONS: u32 = MAPPER_A
    | MAPPER_B
    | MAPPER_X
    | MAPPER_Y
    | MAPPER_L
    | MAPPER_R
    | MAPPER_ZL
    | MAPPER_ZR
    | MAPPER_STICK_CLICKS
    | MAPPER_RIGHT_STICK_DIRECTIONS
    | MAPPER_EXTRA_BUTTONS;
/// Physical gameplay controls, excluding pause/menu and left-stick DI directions.
///
///  Physical gameplay controls, excluding pause/menu and left-stick DI directions.
pub const GAMEPLAY_BUTTONS: u32 = ASSIST_BUTTONS | MAPPER_DPAD_BUTTONS;

/// Every gameplay control belongs to the caller: a fully owned input tape.
///
///  Every gameplay control belongs to the caller: a fully owned input tape.
pub const fn fully_owned(owned: u32) -> bool {
    owned & GAMEPLAY_BUTTONS == GAMEPLAY_BUTTONS
}

/// `bits` when `condition` holds, otherwise no bits.
///
///  bits when condition holds, otherwise no bits.
pub(crate) const fn bits_if(condition: bool, bits: u32) -> u32 {
    if condition {
        bits
    } else {
        0
    }
}

/// Consume the alternate native click channel of an owned physical shoulder.
/// This expands suppression only; generated buttons keep their normal meaning.
///
///  Consume the alternate native click channel of an owned physical shoulder. This expands suppression only; generated buttons keep their normal meaning.
pub fn physical_button_aliases(mask: u32) -> u32 {
    mask | bits_if(mask & (MAPPER_L | MAPPER_ZL) != 0, MAPPER_DIGITAL_L)
        | bits_if(mask & (MAPPER_R | MAPPER_ZR) != 0, MAPPER_DIGITAL_R)
}

/// Native analog trigger axes are independent of the button-history words.
/// Filter only an owned/consumed side, restoring both original axes after mapping.
///
///  Native analog trigger axes are independent of the button-history words. Filter only an owned/consumed side, restoring both original axes after mapping.
pub fn owned_trigger_axes(physical: [f32; 2], suppressed: u32) -> [f32; 2] {
    [
        if suppressed & (MAPPER_L | MAPPER_ZL | MAPPER_DIGITAL_L) != 0 {
            0.
        } else {
            physical[0]
        },
        if suppressed & (MAPPER_R | MAPPER_ZR | MAPPER_DIGITAL_R) != 0 {
            0.
        } else {
            physical[1]
        },
    ]
}

/// Every exclusive tape forwards the same analog packet, including neutral
/// release frames. Partial helpers and native safety keep their existing path.
///
///  Every exclusive tape forwards the same analog packet, including neutral release frames. Partial helpers and native safety keep their existing path.
pub fn exclusive_sequence_stick(owned: u32, stick: Option<[f32; 2]>) -> Option<[f32; 2]> {
    stick
        .filter(|_| fully_owned(owned))
        .filter(|s| s.iter().all(|v| v.is_finite()))
}

/// Walking suppresses a dash flick; explicit full-strength Drift/Run must not.
///
///  Walking suppresses a dash flick; explicit full-strength Drift/Run must not.
pub fn sequence_flick_consumption(owned: u32, walking: bool) -> u32 {
    bits_if(fully_owned(owned) && walking, MAPPER_LEFT_STICK_HORIZONTAL)
}

/// Raw source buttons/axes were already masked before native mapping. Preserve
/// its action result: e.g. Grab can contain Guard/Attack flags and ZL is Jump in
/// the canonical recipe layout. Only suppress an unrequested physical C-stick.
///
///  Raw source buttons/axes were already masked before native mapping. Preserve its action result: e.g. Grab can contain Guard/Attack flags and ZL is Jump in the canonical recipe layout. Only suppress an unrequested physical C-stick.
pub fn mapped_sequence_cstick(buttons: u32, right: [f32; 2]) -> u32 {
    if right == [0.; 2] {
        buttons & !MAPPED_CSTICK_BUTTON
    } else {
        buttons
    }
}

/// Fixed automatic grab and direct Up-B tapes own all raw sources. Remove
/// unrequested jump/shield output that can turn a dash/Up-B into a jump/roll. Keep
/// native Attack/Special/Catch semantics and every explicitly requested jump.
///
///  Fixed automatic grab and direct Up-B tapes own all raw sources. Remove unrequested jump/shield output that can turn a dash/Up-B into a jump/roll. Keep native Attack/Special/Catch semantics and every explicitly requested jump.
pub fn mapped_automatic_actions(buttons: u32, requested: u32) -> u32 {
    let mut result = buttons;
    if requested & SEQUENCE_JUMP_BUTTONS == 0 {
        result &= !(MAPPED_JUMP | MAPPED_JUMP_MINI);
    }
    if requested & SEQUENCE_GUARD_INPUTS == 0 {
        result &= !(MAPPED_GUARD | MAPPED_GUARD_HOLD);
    }
    result
}

/// A fully owned input tape supplies its own buttons exclusively. Native safety
/// must win owner selection, rather than being mixed into another owner's tape.
///
///  A fully owned input tape supplies its own buttons exclusively. Native safety must win owner selection, rather than being mixed into another owner's tape.
pub fn selected_requests(owned: u32, selected: u32, other: u32) -> u32 {
    if fully_owned(owned) {
        selected
    } else {
        selected | other
    }
}

/// Fixed native jump triggers are never custom-binding sources. Manual escape
/// suspends automatic modes rather than mixing human input into their tapes.
///
///  Fixed native jump triggers are never custom-binding sources. Manual escape suspends automatic modes rather than mixing human input into their tapes.
pub fn automatic_escape(
    layout: ControlLayout,
    held: u32,
    pressed: u32,
    reacting: bool,
    grounded: bool,
    requested: u32,
) -> bool {
    let jumps = match layout {
        ControlLayout::Default => MAPPER_ZL,
        ControlLayout::Brother => MAPPER_R | MAPPER_ZR,
    };
    let starting_grounded_special = grounded && requested & MAPPER_B != 0;
    held & (jumps | layout.physical_buttons(MAPPER_R)) != 0
        || reacting
            && !starting_grounded_special
            && pressed & layout.physical_buttons(SEQUENCE_GUARD_BUTTONS) != 0
}

/// Arm a passive Up-B receipt only after final native Special + Up delivery.
/// Failed mapper attempts still spend their separate bounded pulse budget.
///
///  Arm a passive Up-B receipt only after final native Special + Up delivery. Failed mapper attempts still spend their separate bounded pulse budget.
pub fn mapped_up_special_delivered(buttons: u32, left_y: i8) -> bool {
    buttons & MAPPED_SPECIAL != 0 && left_y > 63
}

/// Serialize a normalized simulated stick axis into the native mapped-input packet.
/// Matches the controller packet's signed-byte scale; native deadzones still apply.
///
///  Serialize a normalized simulated stick axis into the native mapped-input packet. Matches the controller packet's signed-byte scale; native deadzones still apply.
pub fn mapped_stick_axis(axis: f32) -> i8 {
    if axis.is_finite() {
        (axis.clamp(-1.0, 1.0) * i8::MAX as f32) as i8
    } else {
        0
    }
}

/// Drop every `sources` bit, then add each pair's destination when any of its
/// source bits is held. Bits outside `sources` pass through unchanged.
///
///  Drop every sources bit, then add each pair's destination when any of its source bits is held. Bits outside sources pass through unchanged.
fn remap_buttons(buttons: u32, sources: u32, pairs: &[(u32, u32)]) -> u32 {
    pairs.iter().fold(buttons & !sources, |mapped, &(from, to)| {
        mapped | bits_if(buttons & from != 0, to)
    })
}

/// Defines the available `ControlLayout` cases.
///
/// Lists the available ControlLayout choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlLayout {
    #[default]
    /// Represents the `Default` case of `ControlLayout`.
    ///
    ///
///  Represents the Default option in ControlLayout.
    Default,
    /// A=Attack, L=Special, R/ZR=Jump, ZL=Shield, X=Grab.
    /// Turbo t-bag remains an independent, configurable button role.
    ///
///  A=Attack, L=Special, R/ZR=Jump, ZL=Shield, X=Grab. Turbo t-bag remains an independent, configurable button role.
    Brother,
}
impl ControlLayout {
    /// Convert physical assist triggers into the canonical controls consumed by
    /// policies. Native R/ZR jumps are deliberately hidden from the old X
    /// shortcut while still passing untouched to the game's mapper.
    ///
///  Convert physical assist triggers into the canonical controls consumed by policies. Native R/ZR jumps are deliberately hidden from the old X shortcut while still passing untouched to the game's mapper.
    pub fn policy_buttons(self, physical: u32) -> u32 {
        if self == Self::Default {
            return physical;
        }
        remap_buttons(
            physical,
            MAPPER_B | MAPPER_X | MAPPER_L | MAPPER_R | MAPPER_ZL | MAPPER_ZR,
            &[
                (MAPPER_B | MAPPER_L, MAPPER_B),
                (MAPPER_X, MAPPER_R),
                (MAPPER_ZL, MAPPER_ZR),
            ],
        )
    }

    /// One-to-one ownership conversion for physical custom binding sources.
    /// Unlike policy_buttons this retains jump buttons hidden from old helpers.
    ///
///  One-to-one ownership conversion for physical custom binding sources. Unlike policybuttons this retains jump buttons hidden from old helpers.
    pub fn binding_ownership(self, physical: u32) -> u32 {
        if self == Self::Default {
            return physical;
        }
        remap_buttons(
            physical,
            MAPPER_X | MAPPER_R | MAPPER_L | MAPPER_ZL | MAPPER_ZR,
            &[
                (MAPPER_X, MAPPER_R),
                (MAPPER_R, MAPPER_X),
                (MAPPER_L, MAPPER_B),
                (MAPPER_ZL, MAPPER_ZR),
                (MAPPER_ZR, MAPPER_ZL),
            ],
        )
    }

    /// Manual intent includes jump buttons hidden from shortcut policy.
    ///
///  Manual intent includes jump buttons hidden from shortcut policy.
    pub fn takeover_buttons(self, physical: u32) -> u32 {
        let buttons = self.policy_buttons(physical);
        if self == Self::Default {
            return buttons;
        }
        buttons
            | bits_if(physical & MAPPER_R != 0, MAPPER_X)
            | bits_if(physical & MAPPER_ZR != 0, MAPPER_Y)
    }

    /// Full takeovers suppress every raw gameplay button, including buttons
    /// unused by the simulated layout that may still map to shield/jump in-game.
    ///
///  Full takeovers suppress every raw gameplay button, including buttons unused by the simulated layout that may still map to shield/jump in-game.
    pub fn physical_owned_buttons(self, canonical: u32) -> u32 {
        self.physical_buttons(canonical) | bits_if(fully_owned(canonical), GAMEPLAY_BUTTONS)
    }

    /// Convert canonical assist buttons back to this layout's physical buttons.
    ///
///  Convert canonical assist buttons back to this layout's physical buttons.
    pub fn physical_buttons(self, canonical: u32) -> u32 {
        if self == Self::Default {
            return canonical;
        }
        remap_buttons(
            canonical,
            MAPPER_B | MAPPER_X | MAPPER_R | MAPPER_ZL | MAPPER_ZR,
            &[
                (MAPPER_B, MAPPER_L),
                (MAPPER_X, MAPPER_R),
                (MAPPER_ZL, MAPPER_ZR),
                (MAPPER_R, MAPPER_X),
                (MAPPER_ZR, MAPPER_ZL),
            ],
        )
    }
}

/// Mapped action bits, distinct from the physical MAPPER_* button positions.
/// Pinned HDR dynamic/src/ext.rs Buttons: Jump=0x4, JumpMini=0x40.
/// Annotate an existing mapped jump only when its owner requested a short hop;
/// never manufacture a jump, attack, or shield input.
///
///  Mapped action bits, distinct from the physical MAPPER button positions. Pinned HDR dynamic/src/ext.rs Buttons: Jump=0x4, JumpMini=0x40. Annotate an existing mapped jump only when its owner requested a short hop; never.
pub fn mapped_short_hop(buttons: u32, requested: bool) -> u32 {
    if requested && buttons & MAPPED_JUMP != 0 {
        buttons | MAPPED_JUMP_MINI
    } else {
        buttons
    }
}

/// Bind the configured physical controller to its current game mapper slot.
/// Controller identity and mapper slot are separate namespaces: controller 0
/// can occupy another mapper slot after entering a different battle lobby.
///
///  Bind the configured physical controller to its current game mapper slot. Controller identity and mapper slot are separate namespaces: controller 0 can occupy another mapper slot after entering a different battle lobby.
pub fn selected_controller_mapper(
    selected_player: u32,
    controller_npad: u32,
    mapper_index: i32,
) -> Option<i32> {
    // libnx hid.h: No1..No8 = 0..7; Handheld has its own ID. It is the
    // console's local Player 1 input, rather than an additional battle slot.
    let local_player = if controller_npad == HANDHELD_NPAD_ID {
        0
    } else {
        controller_npad
    };
    (selected_player < 8 && local_player == selected_player && (0..8).contains(&mapper_index))
        .then_some(mapper_index)
}

// Conversion is per button, not a cast: the two bitfields use different orders.
// Unsupported request bits naturally produce no native bits.
/// Returns the `mapper_buttons` result.
///
/// Gets the mapper buttons result.
pub fn mapper_buttons(hid: u64) -> u32 {
    (u32::from(hid & HID_A != 0) * MAPPER_A)
        | (u32::from(hid & HID_B != 0) * MAPPER_B)
        | (u32::from(hid & HID_X != 0) * MAPPER_X)
        | (u32::from(hid & HID_Y != 0) * MAPPER_Y)
        | (u32::from(hid & HID_L != 0) * MAPPER_L)
        | (u32::from(hid & HID_R != 0) * MAPPER_R)
        | (u32::from(hid & HID_ZL != 0) * MAPPER_ZL)
        | (u32::from(hid & HID_ZR != 0) * MAPPER_ZR)
}

/// Preserve an explicitly requested A input after native mapper/history
/// filtering. Pinned HDR dynamic/src/ext.rs Buttons: Attack=1, AttackRaw=0x200.
/// This is a held input word, not command flags or repeated trigger edges.
/// No ownership/input request means the original mapper output is untouched.
///
///  Preserve an explicitly requested A input after native mapper/history filtering. Pinned HDR dynamic/src/ext.rs Buttons: Attack=1, AttackRaw=0x200. This is a held input word, not command flags or repeated trigger edges. No.
pub fn mapped_attack_hold(buttons: u32, requested: bool) -> u32 {
    if requested {
        buttons | MAPPED_ATTACK_HOLD_BITS
    } else {
        buttons
    }
}

/// Final literal controller actions, after raw-source/history filtering. The
/// caller supplies only selected, owned buttons still present in its raw packet.
/// These are held input bits; no command flags, move/state writes or new edges.
///
///  Final literal controller actions, after raw-source/history filtering. The caller supplies only selected, owned buttons still present in its raw packet. These are held input bits; no command flags, move/state writes or new edges.
pub fn mapped_literal_buttons(buttons: u32, requested: u32) -> u32 {
    let mut mapped = buttons;
    // Fully owned neutral/attack frames release the preceding shield too.
    // Otherwise mapper hold history can outlive the raw ZR release and prevent
    // the next authored move from leaving shield.
    if requested & SEQUENCE_GUARD_INPUTS == 0 {
        mapped &= !(MAPPED_GUARD | MAPPED_GUARD_HOLD);
    }
    // Catch alias only for an explicit Grab or the A+ZR Guard-Attack pair.
    let catch =
        requested & MAPPER_R != 0 || requested & (MAPPER_A | MAPPER_ZR) == (MAPPER_A | MAPPER_ZR);
    if !catch {
        mapped &= !MAPPED_CATCH;
    }
    mapped_attack_hold(mapped, requested & MAPPER_A != 0)
        | bits_if(requested & MAPPER_B != 0, MAPPED_SPECIAL)
        | bits_if(requested & MAPPER_R != 0, MAPPED_CATCH)
        | bits_if(requested & MAPPER_ZR != 0, MAPPED_GUARD)
}

/// Match the native Catch+Guard result observed for an unshielded manual Grab.
///
///  Match the native Catch+Guard result observed for an unshielded manual Grab.
pub fn mapped_unshielded_auto_grab(buttons: u32, requested: u32, active: bool) -> u32 {
    if active && requested & MAPPER_R != 0 && requested & MAPPER_ZR == 0 {
        buttons | MAPPED_CATCH | MAPPED_GUARD
    } else {
        buttons
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
// All four words describe one view of the same input. `pressed` is an edge
// (off -> on), whereas `held` stays on for as long as a button remains pressed.
/// Defines the `ButtonTrace` data structure.
///
/// Holds the data for ButtonTrace.
pub struct ButtonTrace {
    /// Stores the `held` value in `ButtonTrace` as `u32`.
    ///
    ///
///  Holds the held value used by ButtonTrace.
    pub held: u32,
    /// Stores the `previous` value in `ButtonTrace` as `u32`.
    ///
    ///
///  Holds the previous value used by ButtonTrace.
    pub previous: u32,
    /// Stores the `pressed` value in `ButtonTrace` as `u32`.
    ///
    ///
///  Holds the pressed value used by ButtonTrace.
    pub pressed: u32,
    /// Stores the `released` value in `ButtonTrace` as `u32`.
    ///
    ///
///  Holds the released value used by ButtonTrace.
    pub released: u32,
}

/// Changes only assist-owned bits and their edge history. The caller supplies
/// a fresh physical trace each mapping call and restores it after mapping.
///
///  Changes only assist-owned bits and their edge history. The caller supplies a fresh physical trace each mapping call and restores it after mapping.
#[derive(Default)]
pub struct MapperButtonOverlay {
    /// Stores the `last_output` value in `MapperButtonOverlay` as `Option<u32>`.
    ///
    ///
///  Optionally stores the last output for MapperButtonOverlay; it may be absent.
    last_output: Option<u32>,
    /// Stores the `last_affected` value in `MapperButtonOverlay` as `u32`.
    ///
    ///
///  Holds the last affected value used by MapperButtonOverlay.
    last_affected: u32,
}
impl MapperButtonOverlay {
    /// A newly started reaction owns its first horizontal flick independently
    /// of the human's earlier held direction. Re-arm only those requested bits;
    /// preserve the remaining history, including guard release and safety.
    ///
///  A newly started reaction owns its first horizontal flick independently of the human's earlier held direction. Re-arm only those requested bits; preserve the remaining history, including guard release and safety.
    pub fn rearm_requested(&mut self, mask: u32, previous: u32) {
        if mask != 0 {
            self.last_output = Some(self.last_output.unwrap_or(previous) & !mask);
        }
    }
    /// Restores the state to its initial configuration.
    ///
    ///
///  Returns the state to its starting point.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    // `human` must come from the freshly read physical structure, not our last
    // returned trace. The native hook saves/restores that structure around the
    // original mapping call; this helper only returns an ordinary Rust value.
    /// Applies the supplied configuration or action.
    ///
    ///
///  Makes the supplied change.
    pub fn apply(&mut self, human: ButtonTrace, requested: u64, owned: u64) -> ButtonTrace {
        let requested = mapper_buttons(requested);
        let owned = mapper_buttons(owned);
        // Include the previous update's affected bits for one final release or
        // hand-back to human input when the scenario ends. Otherwise a synthetic
        // press could disappear without the corresponding release edge.
        self.apply_raw(human, requested, owned)
    }

    /// Already-converted native masks for manual D-pad/stick ownership. Callers
    /// must limit masks to their own controls; use apply() for HID-named requests.
    ///
///  Already-converted native masks for manual D-pad/stick ownership. Callers must limit masks to their own controls; use apply() for HID-named requests.
    pub fn apply_raw(&mut self, human: ButtonTrace, requested: u32, owned: u32) -> ButtonTrace {
        // requested = bits to turn ON; owned = bits automation may replace,
        // including turning them OFF. Request=0 with owned=ZR releases ZR even
        // when the human holds it; request=0/owned=0 hands it back to the human.
        let affected = requested | owned | self.last_affected;
        // & !mask clears only owned bits, then | adds requested presses.
        // Buttons outside the mask are copied unchanged from the human trace.
        let held = (human.held & !owned) | requested;
        let previous =
            (human.previous & !affected) | (self.last_output.unwrap_or(human.previous) & affected);
        // Rebuild edges only for affected buttons; unrelated native edge words
        // remain untouched. last_output supplies the previous GAME-visible state.
        // For an affected bit: pressed = held now AND not held previously;
        // released = held previously AND not held now. A continuous held bit
        // has neither edge, which is why repeatedly writing "held" isn't turbo.
        let result = ButtonTrace {
            held,
            previous,
            pressed: (human.pressed & !affected) | (held & !previous & affected),
            released: (human.released & !affected) | (previous & !held & affected),
        };
        self.last_output = Some(held);
        self.last_affected = requested | owned;
        result
    }
}

/// Resolve the input-mapper index through the game's controller ownership chain.
/// A ControlModule's +8 field is NOT its controller index.
///
/// # Safety
/// `accessor` and every non-null pointer in this game-version-specific chain
/// must point to live, sufficiently large, aligned native allocations.
///
///  Resolve the input-mapper index through the game's controller ownership chain. A ControlModule's +8 field is NOT its controller index. # Safety accessor and every non-null pointer in this game-version-specific chain must point.
pub unsafe fn controller_index(accessor: *const u8) -> Option<i32> {
    if !native_pointer_address(accessor) {
        return None;
    }
    // Each step loads the pointer stored at an offset in the previous object.
    // add(offset) alone moves an address; read() follows the stored pointer.
    // A non-null pointer is still not proof of validity: native lifetime/version
    // checks remain the caller's responsibility, as documented in Safety above.
    let control = read_native_pointer(accessor, MODULE_ACCESSOR_CONTROL_MODULE_POINTER_OFFSET_BYTES)?;
    let first = read_native_pointer(control, CONTROL_MODULE_INPUT_CONTEXT_POINTER_OFFSET_BYTES)?;
    let second = read_native_pointer(first, INPUT_CONTEXT_CONTROLLER_RECORD_POINTER_OFFSET_BYTES)?;
    let input = read_native_pointer(second, CONTROLLER_RECORD_INDEX_POINTER_OFFSET_BYTES)?;
    let index = input
        .add(INPUT_INDEX_FIELD_OFFSET_BYTES)
        .cast::<i32>()
        .read();
    (0..MAPPER_SLOT_COUNT).contains(&index).then_some(index)
}

/// Follow the pointer stored `offset` bytes into `object`; same safety contract
/// as `controller_index`.
///
///  Follow the pointer stored offset bytes into object; same safety contract as controllerindex.
unsafe fn read_native_pointer(object: *const u8, offset: usize) -> Option<*const u8> {
    let pointer = object.add(offset).cast::<*const u8>().read();
    native_pointer_address(pointer).then_some(pointer)
}

// These checks reject null, small sentinel and misaligned addresses. They do
// NOT prove mapped memory or object lifetime; callers must validate the live
// native callback identity/readiness before entering this chain.
/// Determines whether the `native_pointer_address` condition holds.
///
/// Checks whether native pointer address.
fn native_pointer_address(pointer: *const u8) -> bool {
    let address = pointer as usize;
    address >= MINIMUM_NATIVE_POINTER_ADDRESS
        && address.is_multiple_of(std::mem::align_of::<usize>())
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
#[path = "../../tests/unit/native_input.rs"]
mod tests;
