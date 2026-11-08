//! Private status interpretations must be scoped to their owning fighter.
use super::{fighter_status, final_states};

pub(super) fn interrupted(kind: i32, status: i32) -> bool {
    noncombat(kind, status)
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

pub(super) fn noncombat(kind: i32, status: i32) -> bool {
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
