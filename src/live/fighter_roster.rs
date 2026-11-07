//! Eight entries plus independently validated Ice Climber partners. No scans
//! of battle-object memory, native writes, allocations or retained pointers.
use super::{active_fighter_manager, game, game_entry_id};
use kirby_assist_core::battle_objects::{append_fighter, FighterRoster, FIGHTER_CAPACITY};
pub(crate) unsafe fn fighter_ids() -> Option<FighterRoster> {
    let manager = active_fighter_manager()?;
    let count = game::entry_count(manager);
    if !(0..=8).contains(&count) {
        return None;
    }
    let mut ids = [None; FIGHTER_CAPACITY];
    for ordinal in 0..count {
        let entry_id = game::entry_id(manager, ordinal);
        if !(0..8).contains(&entry_id) {
            continue;
        }
        let entry = game::fighter_entry(manager, entry_id) as *mut game::Object;
        if entry.is_null() {
            continue;
        }
        let id = game::current_fighter_id(entry) as u32;
        if id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
            || !game::is_active(id)
        {
            continue;
        }
        let object = game::module_accessor(id);
        if object.is_null() || game::battle_object_id(object) != id {
            continue;
        }
        append_fighter(&mut ids, id);
        let kind = game::fighter_kind(id);
        if !matches!(
            kind,
            kirby_assist_core::fighter_kinds::ICE_CLIMBERS_POPO
                | kirby_assist_core::fighter_kinds::ICE_CLIMBERS_NANA
        ) {
            continue;
        }
        // FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID, pinned SDK.
        let partner = game::work_int(object, kirby_assist_core::native_states::work_ids::FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID) as u32;
        if partner == id
            || partner >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
            || !game::is_active(partner)
        {
            continue;
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
            continue;
        }
        append_fighter(&mut ids, partner);
    }
    Some(ids)
}
