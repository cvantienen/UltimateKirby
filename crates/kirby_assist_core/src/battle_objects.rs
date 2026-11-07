//! Bounded copied identities and candidate selection; no native pointers.
pub const FIGHTER_CAPACITY: usize = 16; // Eight entries, each with an IC partner.
pub const OBJECT_CATEGORY_FIGHTER: u32 = 0;
pub const OBJECT_CATEGORY_WEAPON: u32 = 1;
pub const OBJECT_CATEGORY_ITEM: u32 = 4;
pub const WEAPON_KIND_PIKMIN_PIKMIN: i32 = 0x9d;
pub const WEAPON_KIND_ROSETTA_TICO: i32 = 0xe6;
pub type FighterRoster = [Option<u32>; FIGHTER_CAPACITY];

pub fn append_fighter(roster: &mut FighterRoster, id: u32) {
    if id >> 28 != OBJECT_CATEGORY_FIGHTER || roster.contains(&Some(id)) {
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
    match id >> 28 {
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
