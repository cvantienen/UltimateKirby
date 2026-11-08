//! Read-only partner resolution for Ice Climbers roster entries.
use super::{game, game_entry_id};
use kirby_assist_core::battle_objects::{append_fighter, FighterRoster};

pub(super) unsafe fn append_partner(
    ids: &mut FighterRoster,
    id: u32,
    object: *mut game::Object,
    entry_id: i32,
) {
    let kind = game::fighter_kind(id);
    if !matches!(
        kind,
        kirby_assist_core::fighter_kinds::ICE_CLIMBERS_POPO
            | kirby_assist_core::fighter_kinds::ICE_CLIMBERS_NANA
    ) {
        return;
    }
    let partner = game::work_int(
        object,
        kirby_assist_core::native_states::work_ids::FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID,
    ) as u32;
    if partner == id
        || partner >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(partner)
    {
        return;
    }
    let other = game::module_accessor(partner);
    if other.is_null()
        || game::battle_object_id(other) != partner
        || game_entry_id(partner) != entry_id
        || game::fighter_kind(partner)
            != if kind == kirby_assist_core::fighter_kinds::ICE_CLIMBERS_POPO {
                kirby_assist_core::fighter_kinds::ICE_CLIMBERS_NANA
            } else {
                kirby_assist_core::fighter_kinds::ICE_CLIMBERS_POPO
            }
    {
        return;
    }
    append_fighter(ids, partner);
}
