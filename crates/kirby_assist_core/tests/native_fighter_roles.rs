//! Execute production roster/ownership adapters with native objects replaced by
//! stable host allocations. No hooks, game calls, networking or Switch build.
#![allow(dead_code)]
use std::{cell::RefCell, ptr};
#[repr(C)]
struct Object {
    tag: u64,
    id: u32,
    pad: u32,
}
struct Actor {
    object: Box<Object>,
    kind: i32,
    entry: i32,
    partner: u32,
    active: bool,
}
struct World {
    actors: Vec<Actor>,
    entries: Vec<Box<Object>>,
    current: Vec<u32>,
    cpu: Vec<bool>,
}
impl World {
    fn new() -> Self {
        Self {
            actors: Vec::new(),
            entries: Vec::new(),
            current: Vec::new(),
            cpu: Vec::new(),
        }
    }
    fn entry(&mut self, current: u32, cpu: bool) -> i32 {
        let id = self.entries.len();
        self.entries.push(Box::new(Object {
            tag: id as u64,
            id: 0,
            pad: 0,
        }));
        self.current.push(current);
        self.cpu.push(cpu);
        id as i32
    }
    fn actor(&mut self, id: u32, kind: i32, entry: i32, partner: u32) {
        self.actors.push(Actor {
            object: Box::new(Object { tag: 0, id, pad: 0 }),
            kind,
            entry,
            partner,
            active: true,
        });
    }
}
thread_local! {static WORLD:RefCell<World>=RefCell::new(World::new());}
mod game {
    pub(super) use super::Object;
    use super::*;
    pub unsafe fn battle_object_id(object: *mut Object) -> u32 {
        (*object).id
    }
    pub unsafe fn is_active(id: u32) -> bool {
        WORLD.with(|w| {
            w.borrow()
                .actors
                .iter()
                .any(|a| a.object.id == id && a.active)
        })
    }
    pub unsafe fn entry_count(_: *mut Object) -> i32 {
        WORLD.with(|w| w.borrow().entries.len() as i32)
    }
    pub unsafe fn entry_id(_: *mut Object, ordinal: i32) -> i32 {
        ordinal
    }
    pub unsafe fn fighter_entry(_: *mut Object, entry: i32) -> u64 {
        WORLD.with(|w| {
            w.borrow_mut()
                .entries
                .get_mut(entry as usize)
                .map_or(0, |e| (&mut **e as *mut Object) as u64)
        })
    }
    pub unsafe fn current_fighter_id(e: *mut Object) -> u64 {
        WORLD.with(|w| w.borrow().current[(*e).tag as usize] as u64)
    }
    pub unsafe fn module_accessor(id: u32) -> *mut Object {
        WORLD.with(|w| {
            w.borrow_mut()
                .actors
                .iter_mut()
                .find(|a| a.object.id == id)
                .map_or(ptr::null_mut(), |a| &mut *a.object)
        })
    }
    pub unsafe fn fighter_kind(id: u32) -> i32 {
        WORLD.with(|w| {
            w.borrow()
                .actors
                .iter()
                .find(|a| a.object.id == id)
                .unwrap()
                .kind
        })
    }
    pub unsafe fn work_int(o: *mut Object, slot: i32) -> i32 {
        assert_eq!(slot, kirby_assist_core::native_states::work_ids::FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID);
        WORLD.with(|w| {
            let w = w.borrow();
            let a = w.actors.iter().find(|a| a.object.id == (*o).id).unwrap();
            assert!(matches!(a.kind, 75 | 76));
            a.partner as i32
        })
    }
    pub unsafe fn fighter_information(m: *mut Object, entry: i32) -> *mut Object {
        fighter_entry(m, entry) as *mut Object
    }
    pub unsafe fn is_cpu(e: *mut Object) -> bool {
        WORLD.with(|w| w.borrow().cpu[(*e).tag as usize])
    }
}
static MANAGER: Object = Object {
    tag: 99,
    id: 0,
    pad: 0,
};
unsafe fn fighter_manager_instance() -> Option<*mut Object> {
    Some((&MANAGER as *const Object).cast_mut())
}
unsafe fn active_fighter_manager() -> Option<*mut Object> {
    fighter_manager_instance()
}
unsafe fn game_entry_id(id: u32) -> i32 {
    WORLD.with(|w| {
        w.borrow()
            .actors
            .iter()
            .find(|a| a.object.id == id)
            .map_or(-1, |a| a.entry)
    })
}
#[path = "../../../src/live/fighter_roster.rs"]
mod roster;
use roster as local_fighter;
fn reset() {
    WORLD.with(|w| *w.borrow_mut() = World::new());
}
#[test]
fn a_human_entrys_partner_or_standby_form_cannot_take_over_the_local_controller() {
    reset();
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        let e = w.entry(1, false);
        w.actor(1, 75, e, 2);
        w.actor(2, 76, e, 1);
        w.actor(3, 90, e, u32::MAX);
    });
    unsafe {
        assert!(local_fighter::human_fighter(game::module_accessor(1)));
        assert!(!local_fighter::human_fighter(game::module_accessor(2)));
        assert!(!local_fighter::human_fighter(game::module_accessor(3)));
        WORLD.with(|w| w.borrow_mut().current[0] = 3);
        assert!(local_fighter::human_fighter(game::module_accessor(3)));
        assert!(!local_fighter::human_fighter(game::module_accessor(1)));
        WORLD.with(|w| w.borrow_mut().cpu[0] = true);
        assert!(!local_fighter::human_fighter(game::module_accessor(3)));
        assert!(!local_fighter::human_fighter(ptr::null_mut()));
    }
}
#[test]
fn eight_ice_climber_entries_include_sixteen_distinct_fighters_without_changing_control_ownership()
{
    reset();
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        for e in 0..8 {
            let main = 1 + e * 2;
            let partner = main + 1;
            let entry = w.entry(main, e != 0);
            w.actor(main, 75, entry, partner);
            w.actor(partner, 76, entry, main);
        }
    });
    let ids = unsafe { roster::fighter_ids().unwrap() };
    assert_eq!(ids, std::array::from_fn(|i| Some(i as u32 + 1)));
    for id in ids.into_iter().flatten() {
        assert_eq!(
            unsafe { local_fighter::human_fighter(game::module_accessor(id)) },
            id == 1
        );
    }
}
#[test]
fn missing_recycled_wrong_entry_or_article_partners_are_rejected() {
    for case in 0..7 {
        reset();
        WORLD.with(|w| {
            let mut w = w.borrow_mut();
            let e = w.entry(1, false);
            let partner = if case == 3 { 0x10000002 } else { 2 };
            w.actor(1, 75, e, partner);
            w.actor(
                partner,
                if case == 2 {
                    43
                } else if case == 6 {
                    75
                } else {
                    76
                },
                if case == 1 { 1 } else { e },
                1,
            );
            match case {
                0 => w.actors[1].active = false,
                4 => w.actors[1].object.id = 9,
                5 => w.actors[0].partner = 1,
                _ => {}
            }
        });
        let ids = unsafe { roster::fighter_ids().unwrap() };
        assert_eq!(
            ids.into_iter().flatten().collect::<Vec<_>>(),
            vec![1],
            "case {case}"
        );
    }
}
#[test]
fn olimars_pikmin_and_rosalinas_luma_are_not_fighter_roster_or_control_candidates() {
    reset();
    WORLD.with(|w| {
        let mut w = w.borrow_mut();
        let e = w.entry(1, false);
        w.actor(1, 43, e, u32::MAX);
        w.actor(0x10000002, 43, e, u32::MAX);
        let e = w.entry(3, true);
        w.actor(3, 51, e, u32::MAX);
        w.actor(0x10000004, 51, e, u32::MAX);
    });
    let ids = unsafe { roster::fighter_ids().unwrap() };
    assert_eq!(ids.into_iter().flatten().collect::<Vec<_>>(), vec![1, 3]);
    for id in [0x10000002, 0x10000004] {
        assert!(!unsafe { local_fighter::human_fighter(game::module_accessor(id)) });
    }
}
