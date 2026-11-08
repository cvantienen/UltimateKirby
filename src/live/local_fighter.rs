//! Validate local fighter identity and its callback-discovered controller route.
use super::{fighter_manager_instance, game, game_entry_id};

pub(crate) unsafe fn human_fighter(object: *mut game::Object) -> bool {
    if object.is_null() {
        return false;
    }
    let Some(manager) = fighter_manager_instance() else {
        return false;
    };
    let id = game::battle_object_id(object);
    if id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
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

pub(crate) unsafe fn valid_local_route(id: u32, index: i32) -> bool {
    if !(0..8).contains(&index)
        || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
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
