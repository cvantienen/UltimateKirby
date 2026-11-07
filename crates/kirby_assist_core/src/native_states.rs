//! Shared read-only native state vocabulary (skyline-smash d2ca90d lua_const).
//! Own incapacitation and opponent punishability are deliberately distinct.
pub mod fighter_status;
mod final_states;
pub mod status;
pub mod transition_terms;
mod up_special;
pub mod work_ids;
pub use up_special::up_special_status;

// Common FIGHTER_STATUS_KIND values from the pinned native status table.
pub const GUARD_ON: i32 = status::GUARD_ON;
pub const GUARD: i32 = status::GUARD;
pub const GUARD_OFF: i32 = status::GUARD_OFF;
pub const GUARD_DAMAGE: i32 = status::GUARD_DAMAGE;
pub const ESCAPE: i32 = status::ESCAPE;
pub const ESCAPE_F: i32 = status::ESCAPE_F;
pub const ESCAPE_B: i32 = status::ESCAPE_B;
pub const ESCAPE_AIR: i32 = status::ESCAPE_AIR;
pub const ESCAPE_AIR_SLIDE: i32 = status::ESCAPE_AIR_SLIDE;
pub const CATCH: i32 = status::CATCH;
pub const CATCH_PULL: i32 = status::CATCH_PULL;
pub const CATCH_DASH: i32 = status::CATCH_DASH;
pub const CATCH_DASH_PULL: i32 = status::CATCH_DASH_PULL;
pub const CATCH_TURN: i32 = status::CATCH_TURN;
// Preserve the old public values for compatibility. Use `status::*` internally
// because these historical labels do not match the pinned native table.
pub const CATCH_TURN_PULL: i32 = status::CATCH_WAIT;
pub const CATCH_WAIT: i32 = status::CATCH_ATTACK;
pub const CATCH_ATTACK: i32 = status::CATCH_CUT;
pub const CATCH_CUT: i32 = status::CATCH_JUMP;
pub const THROW: i32 = status::THROW;
pub const CAPTURE_PULLED: i32 = status::CAPTURE_PULLED;
pub const THROWN: i32 = status::THROWN;

/// Special captures/final victims are not ordinary combat targets, even when a
/// transient LinkModule capture link is absent or uses a move-specific link.
pub fn captured_or_scripted(status: i32) -> bool {
    matches!(status,
        status::CAPTURE_PULLED..=status::THROWN
        | status::CLUNG_GANON..=status::SHOULDERED_DONKEY_THROWN
        | status::SWALLOWED..=status::CAPTURE_DAMAGE_YOSHI
        | status::CATCHED_REFLET..=status::CAPTURE_MASTERHAND
        | status::GIMMICK_EATEN
        | status::DEMO..=status::KASEY_WARP
        | status::CLIFF_ROBBED..=status::METAKNIGHT_FINAL_DAMAGE_FALL
        | status::FINAL_VISUAL_ATTACK_OTHER..=status::PSYCHOBREAK
        | status::RYU_FINAL_DAMAGE..=status::RYU_FINAL_DAMAGE_FLY
        | status::CLOUD_FINAL_DAMAGE_FLY..=status::RIDLEY_FINAL_TARGET_END
        | status::CATCHED_RIDLEY..=status::STABBED_DAMAGE
        | status::SWING_GAOGAEN_CATCHED..=status::CAPTURE_WAIT_OCTOPUS
        | status::SIMON_FINAL_TARGET_START..=status::BRAVE_FINAL_TARGET_END
        | status::BUDDY_FINAL_TARGET_START..=status::TANTAN_FINAL_TARGET_END
        | status::CATCHED_PICKEL_TROLLEY..=status::EDGE_FINAL_TARGET_END
        | status::CAPTURE_TRAIL_KEYHOLE..=status::STANDBY
        | status::ENTRY..=status::LOSE
        | status::FINAL)
}

/// Abort existing input tapes on forced states, without disabling native
/// mash/tech safety. Do not resume an old attack after a stun/capture ends.
pub fn sequence_interrupted(status: i32) -> bool {
    captured_or_scripted(status)
        || matches!(status,
        status::REBOUND_STOP..=status::REBOUND_JUMP
        | status::DAMAGE..=status::DOWN_WAIT_CONTINUE
        | status::DOWN_DAMAGE..=status::BIND
        | status::SLIP..=status::SLIP_WAIT
        | status::SWIM_DIVE..=status::REBIRTH
        | status::TREAD_DAMAGE..=status::TREAD_FALL
        | status::BURY..=status::HAMMER_LANDING
        | status::KILLER
        | status::GENESIS_SHOOT_START..=status::GENESIS_GET
        | status::LADDER_CATCH..=status::LADDER_END
        | status::SAVING_DAMAGE..=status::SAVING_DAMAGE_FLY
        | status::SAVING_DAMAGE_AIR
        | status::METAMON_OUT..=status::BOSS_DEAD
        | status::DAMAGE_SLEEP_START..=status::SLEEP_END
        | status::GIMMICK_JUMP_BOARD..=status::DAMAGE_FLY_REFLECT_JUMP_BOARD
        | status::TRAIL_REBOUND)
}

/// These vulnerable bodies may be hit by a configured shield backup, but an
/// ordinary grab must not be chosen for their downed/buried/frozen states.
pub fn ordinary_grab_immune(status: i32) -> bool {
    matches!(
        status,
        status::DOWN..=status::LAY_DOWN
            | status::DOWN_DAMAGE..=status::DOWN_REFLECT_LR
            | status::SHIELD_BREAK_DOWN
            | status::SLIP..=status::SLIP_WAIT
            | status::BURY..=status::ICE_JUMP
    )
}

/// Private status values overlap across fighters. Apply these forced-state
/// meanings only to the native kind that owns them.
pub fn fighter_sequence_interrupted(kind: i32, status: i32) -> bool {
    sequence_interrupted(status)
        || fighter_noncombat(kind, status)
        || match kind {
            crate::fighter_kinds::DIDDY_KONG => matches!(
                status,
                fighter_status::DIDDY_SPECIAL_S_DAMAGE_FALL
                    | fighter_status::DIDDY_SPECIAL_HI_CHARGE_DAMAGE
                    | fighter_status::DIDDY_SPECIAL_HI_UPPER_DAMAGE
            ),
            crate::fighter_kinds::GRENINJA => {
                status == fighter_status::GEKKOUGA_SPECIAL_HI_WALL_DAMAGE
            }
            crate::fighter_kinds::BOWSER_JR => {
                status == fighter_status::KOOPAJR_SPECIAL_HI_DAMAGE_END
            }
            crate::fighter_kinds::SQUIRTLE => status == fighter_status::PZENIGAME_SPECIAL_LW_OUT,
            crate::fighter_kinds::IVYSAUR => status == fighter_status::PFUSHIGISOU_SPECIAL_LW_OUT,
            crate::fighter_kinds::CHARIZARD => status == fighter_status::PLIZARDON_SPECIAL_LW_OUT,
            crate::fighter_kinds::PYRA => status == fighter_status::EFLAME_SPECIAL_LW_OUT,
            crate::fighter_kinds::MYTHRA => status == fighter_status::ELIGHT_SPECIAL_LW_OUT,
            _ => false,
        }
}

/// Inactive swap forms and known Final Smash actors are not ordinary punish
/// targets, even through a transient missing capture link.
pub fn fighter_noncombat(kind: i32, status: i32) -> bool {
    final_states::final_status(kind, status)
        || matches!(
            (kind, status),
            (
                crate::fighter_kinds::SQUIRTLE,
                fighter_status::PZENIGAME_SPECIAL_LW_STANDBY
            ) | (
                crate::fighter_kinds::IVYSAUR,
                fighter_status::PFUSHIGISOU_SPECIAL_LW_STANDBY
            ) | (
                crate::fighter_kinds::CHARIZARD,
                fighter_status::PLIZARDON_SPECIAL_LW_STANDBY
            ) | (
                crate::fighter_kinds::PYRA,
                fighter_status::EFLAME_SPECIAL_LW_STANDBY
            ) | (
                crate::fighter_kinds::MYTHRA,
                fighter_status::ELIGHT_SPECIAL_LW_STANDBY
            )
        )
}
