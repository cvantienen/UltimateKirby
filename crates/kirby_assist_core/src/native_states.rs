//! Shared read-only native state vocabulary (skyline-smash d2ca90d lua_const).
//! Own incapacitation and opponent punishability are deliberately distinct.
mod final_states;
mod up_special;
pub use up_special::up_special_status;

/// Special captures/final victims are not ordinary combat targets, even when a
/// transient LinkModule capture link is absent or uses a move-specific link.
pub fn captured_or_scripted(status: i32) -> bool {
    matches!(status,
        0x41..=0x46 | 0xbd..=0xc7 | 0xe6..=0xf7 | 0xfe..=0xff
        | 0x102 | 0x107..=0x110 | 0x116..=0x119 | 0x11f..=0x147
        | 0x14a..=0x14b | 0x14d..=0x16e | 0x171..=0x174
        | 0x176..=0x18c | 0x18f..=0x1a2 | 0x1ab..=0x1ba
        | 0x1be..=0x1d0 | 0x1d2..=0x1d6 | 0x1d9..=0x1db | 0x1e0)
}

/// Abort existing input tapes on forced states, without disabling native
/// mash/tech safety. Do not resume an old attack after a stun/capture ends.
pub fn sequence_interrupted(status: i32) -> bool {
    captured_or_scripted(status)
        || matches!(status,
        0x24..=0x26 | 0x47..=0x54 | 0x5a..=0x66 | 0x86..=0x88
        | 0xab..=0xb6 | 0xb9..=0xbc | 0xc8..=0xdf
        | 0x100 | 0x103..=0x106 | 0x111..=0x115 | 0x148..=0x149
        | 0x14c | 0x196..=0x198 | 0x1a3..=0x1aa | 0x1bb..=0x1bd | 0x1d1)
}

/// These vulnerable bodies may be hit by a configured shield backup, but an
/// ordinary grab must not be chosen for their downed/buried/frozen states.
pub fn ordinary_grab_immune(status: i32) -> bool {
    matches!(status, 0x50..=0x56 | 0x5a..=0x5b | 0x5e | 0x86..=0x88 | 0xc8..=0xcc)
}

/// Private status values overlap across fighters. Apply these forced-state
/// meanings only to the native kind that owns them.
pub fn fighter_sequence_interrupted(kind: i32, status: i32) -> bool {
    sequence_interrupted(status)
        || fighter_noncombat(kind, status)
        || match kind {
            39 => matches!(status, 0x1ef | 0x1f1 | 0x1f3), // Diddy damage phases.
            53 => status == 0x1e8,                         // Greninja recovery wall damage.
            58 => status == 0x1eb,                         // Clown-car recovery damage end.
            36 | 37 | 38 | 90 | 91 => status == 0x1e2,     // Own swap-out recovery.
            _ => false,
        }
}

/// Inactive swap forms and known Final Smash actors are not ordinary punish
/// targets, even through a transient missing capture link.
pub fn fighter_noncombat(kind: i32, status: i32) -> bool {
    final_states::final_status(kind, status)
        || matches!(kind, 36 | 37 | 38 | 90 | 91) && status == 0x1e1
}
