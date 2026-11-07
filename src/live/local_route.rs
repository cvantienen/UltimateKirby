//! Revalidate the existing callback-discovered controller route. No roster scan,
//! network query, scene hook, native writes or additional memory offsets.
use super::{game, human_fighter};

pub(crate) unsafe fn valid_local_route(id: u32, index: i32) -> bool {
    if !(0..8).contains(&index) || id >> 28 != 0 || !game::is_active(id) {
        return false;
    }
    let object = game::module_accessor(id);
    !object.is_null()
        && object.cast::<u8>().add(8).cast::<u32>().read() == id
        && kirby_assist_core::native_input::controller_index(object.cast()) == Some(index)
        && human_fighter(object)
}
