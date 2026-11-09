//! Bounded copied identities and candidate selection; no native pointers.
/// Maximum number of fighter current slots in the roster.
pub const FIGHTER_CAPACITY: usize = 16; 
/// Object category for fighters.
pub const OBJECT_CATEGORY_FIGHTER: u32 = 0;
/// Object category for weapons.
pub const OBJECT_CATEGORY_WEAPON: u32 = 1;
/// Object category for items.
pub const OBJECT_CATEGORY_ITEM: u32 = 4;
/// Weapon kind for Pikmin Pikmin.
pub const WEAPON_KIND_PIKMIN_PIKMIN: i32 = 0x9d;
/// Weapon kind for Rosetta Tico.
pub const WEAPON_KIND_ROSETTA_TICO: i32 = 0xe6;
/// Bitshift amount (28) for extracting the category from an object ID.
/// That label is a 32-bit number. 
/// Smash stores the object’s category in the first four bits and other identifying information in the remaining 28
pub const BITSHIFT_CATEGORY: u32 = 28;
/// Array holding current fighters in game.
/// Each slot can hold an optional fighter ID; empty slots are None.
pub type FightersInGame = [Option<u32>; FIGHTER_CAPACITY];


/// Recieves a mutable reference to a fighter roster and a fighter ID to append.
pub fn append_fighter(roster: &mut FightersInGame, id: u32) {
    // Ignore invalid IDs or duplicates.
    if id >> BITSHIFT_CATEGORY != OBJECT_CATEGORY_FIGHTER || roster.contains(&Some(id)) {
        return;
    }
    if let Some(slot) = roster.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(id);
    }
}

/// Extra actors must not turn roster ordinals into indices of eight-slot
/// policy arrays. Keep the nearest relevant candidates when those arrays fill.
pub fn insert_nearest<T: Copy, const N: usize>(
    targets: &mut [Option<T>; N],
    candidate: T,
    distance: impl Fn(T) -> f32,
) {
    let candidate_distance = distance(candidate);
    if !candidate_distance.is_finite() || candidate_distance < 0. {
        return;
    }
    if let Some(slot) = targets.iter_mut().find(|slot| slot.is_none()) {
        *slot = Some(candidate);
        return;
    }
    let Some((farthest, far_distance)) = targets
        .iter()
        .enumerate()
        .filter_map(|(i, t)| t.map(|t| (i, distance(t))))
        .max_by(|a, b| a.1.total_cmp(&b.1))
    else {
        return;
    };
    if candidate_distance < far_distance {
        targets[farthest] = Some(candidate);
    }
}

pub fn distance_squared(from: [f32; 2], to: [f32; 2]) -> f32 {
    (to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2)
}

/// Articles are not all incoming projectiles. Luma and already attached Pikmin
/// cannot be bucketed; native hitbox capability checks still decide all others.
pub fn projectile_actor(id: u32, kind: i32, status: i32) -> bool {
    match id >> BITSHIFT_CATEGORY {
        OBJECT_CATEGORY_WEAPON => {
            kind != WEAPON_KIND_ROSETTA_TICO
                && !(kind == WEAPON_KIND_PIKMIN_PIKMIN
                    && matches!(
                        status,
                        crate::native_states::status::LANDING_LIGHT
                            ..=crate::native_states::status::LANDING_FALL_SPECIAL
                    ))
        }
        OBJECT_CATEGORY_ITEM => true,
        _ => false,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ObjectCategory {
    Fighter = OBJECT_CATEGORY_FIGHTER,
    Weapon  = OBJECT_CATEGORY_WEAPON,
    Item    = OBJECT_CATEGORY_ITEM,
    // ...
}

impl ObjectCategory {
    pub const fn from_id(id: u32) -> Option<Self> {
        match id >> BITSHIFT_CATEGORY {
            OBJECT_CATEGORY_FIGHTER => Some(Self::Fighter),
            OBJECT_CATEGORY_WEAPON  => Some(Self::Weapon),
            OBJECT_CATEGORY_ITEM    => Some(Self::Item),
            _ => None,
        }
    }
    /// check to ensure the weapon is not a non-projectile type like Rosetta & Tico, 
    /// and that Pikmin are not already attached or in a landing state
    pub fn is_projectile(id: u32, kind: i32, status: i32) -> bool {
        match Self::from_id(id) {
            Some(Self::Weapon) => {
                kind != WEAPON_KIND_ROSETTA_TICO
                    && !(kind == WEAPON_KIND_PIKMIN_PIKMIN
                        && matches!(
                            status,
                            crate::native_states::status::LANDING_LIGHT
                                ..=crate::native_states::status::LANDING_FALL_SPECIAL
                        ))
            }

            Some(Self::Item) => true,

            _ => false,
        }
    }
}

/// Entry-based damage attribution cannot distinguish two Ice Climbers on
/// opposite sides. Preserve agreement; return no invented side on conflict.
#[derive(Default)]
pub struct EntryDirection {
    side: Option<f32>,
    conflict: bool,
}
impl EntryDirection {
    pub fn observe(&mut self, direction: f32) {
        if !direction.is_finite() || self.side.is_some_and(|side| side != direction) {
            self.conflict = true;
        }
        self.side = Some(direction);
    }
    pub fn direction(&self) -> Option<f32> {
        if self.conflict {
            None
        } else {
            self.side
        }
    }
}
