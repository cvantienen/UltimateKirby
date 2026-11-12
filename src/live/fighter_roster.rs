//! Eight entries plus independently validated Ice Climber partners. No scans
//! of battle-object memory, native writes, allocations or retained pointers.
//! Technical: Builds a bounded roster from validated fighter-manager entries
//! and checks that local fighters match callback-discovered controller routes.
//! Plain English: This module identifies the fighters in the match and confirms
//! which one belongs to a human-controlled player.
use super::{active_fighter_manager, fighter_manager_instance, game, game_entry_id};
use kirby_assist_core::battle_objects::BITSHIFT_CATEGORY;
use kirby_assist_core::battle_objects::{append_fighter, FightersInGame, FIGHTER_CAPACITY};

/// Copies the active fighter IDs from the manager and adds valid
/// Ice Climbers partners to the fixed-capacity roster.
pub(crate) unsafe fn fighter_ids() -> Option<FightersInGame> {
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
        if id >> BITSHIFT_CATEGORY != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
            || !game::is_active(id)
        {
            continue;
        }
        let object = game::module_accessor(id);
        if object.is_null() || game::battle_object_id(object) != id {
            continue;
        }
        append_fighter(&mut ids, id);
        append_partner(&mut ids, id, object, entry_id);
    }
    Some(ids)
}

/// appends fighters and their Ice Climber partners to the list of fighters in the game.
/// Validates the partner's category, active object identity, entry,
/// and complementary Popo/Nana kind before appending it.
/// to the same pair.
/// # Safety
/// The supplied object pointer must remain valid for the native reads in this call.
unsafe fn append_partner(
    ids: &mut FightersInGame,
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

/// Validate local fighter identity and its callback-discovered controller route.
/// Rejects invalid objects, non-current Ice Climbers actors, and CPU entries.
/// # Safety
/// The object pointer must be valid for the native reads in this call.
pub(crate) unsafe fn human_fighter(object: *mut game::Object) -> bool {
    if object.is_null() {
        return false;
    }
    let Some(manager) = fighter_manager_instance() else {
        return false;
    };
    let id = game::battle_object_id(object);
    if id >> BITSHIFT_CATEGORY != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(id)
    {
        return false;
    }
    let entry_id = game_entry_id(id);
    if !(0..8).contains(&entry_id) {
        return false;
    }
    let entry = game::fighter_entry(manager, entry_id) as *mut game::Object;
    // An Ice Climber partner shares the leader's human entry and can share a
    // mapper index. Only the entry's current actor may drive local assistance.
    if entry.is_null() || game::current_fighter_id(entry) != u64::from(id) {
        return false;
    }
    let information = game::fighter_information(manager, entry_id);
    !information.is_null() && !game::is_cpu(information)
}

/// Confirms a fighter ID and controller index against the active native
/// object and human-entry checks.
/// # Safety
/// Native game state referenced by the ID must remain valid during this call.
pub(crate) unsafe fn valid_local_route(id: u32, index: i32) -> bool {
    if !(0..8).contains(&index)
        || id >> BITSHIFT_CATEGORY != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(id)
    {
        return false;
    }
    let object = game::module_accessor(id);
    !object.is_null()
        && game::battle_object_id(object) == id
        && kirby_assist_core::native_input::controller_index(object.cast()) == Some(index)
        && human_fighter(object)
}
