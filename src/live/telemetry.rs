//! Diagnostic sampling on native callbacks only; transport receives copied scalars.
use super::{game, native_params};
use crate::diagnostics::telemetry as transport;
use kirby_assist_core::{battle_objects, native_states as native, telemetry::Sample};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};

static SAMPLING: AtomicBool = AtomicBool::new(false);
static LAST: Mutex<Option<Instant>> = Mutex::new(None);
static PROJECTILES: Mutex<[Option<u32>; 32]> = Mutex::new([None; 32]);
struct SamplingGuard;
impl Drop for SamplingGuard {
    fn drop(&mut self) {
        SAMPLING.store(false, Ordering::Release);
    }
}
fn float(values: &mut [Option<u64>; 64], index: usize, value: f32) {
    values[index] = value.is_finite().then_some(u64::from(value.to_bits()));
}
fn integer(values: &mut [Option<u64>; 64], index: usize, value: i32) {
    values[index] = Some(value as i64 as u64);
}
fn boolean(values: &mut [Option<u64>; 64], index: usize, value: bool) {
    values[index] = Some(u64::from(value));
}
/// Called after the existing native ATTACK trampoline; never installs a second hook.
pub(super) unsafe fn remember_attack(object: *mut game::Object) {
    if !transport::ready() || object.is_null() {
        return;
    }
    let id = game::battle_object_id(object);
    if !matches!(
        id >> 28,
        battle_objects::OBJECT_CATEGORY_WEAPON | battle_objects::OBJECT_CATEGORY_ITEM
    ) {
        return;
    }
    if let Ok(mut ids) = PROJECTILES.try_lock() {
        if !ids.contains(&Some(id)) {
            ids.rotate_right(1);
            ids[0] = Some(id);
        }
    }
}
/// Samples the bounded fighter roster and callback-discovered attacking objects.
/// Native reads execute synchronously; each object is revalidated before publication.
pub(super) unsafe fn observe(object: *mut game::Object) {
    if !transport::enabled() {
        return;
    }
    let training = game::is_training_mode();
    transport::observe(training);
    if !training {
        if let Ok(mut ids) = PROJECTILES.try_lock() {
            *ids = [None; 32];
        }
        return;
    }
    if !transport::ready() || object.is_null() || !super::human_fighter(object) {
        return;
    }
    if SAMPLING.swap(true, Ordering::Acquire) {
        return;
    }
    let _guard = SamplingGuard;
    if let Ok(mut last) = LAST.try_lock() {
        if last.is_some_and(|t| t.elapsed() < Duration::from_millis(16)) {
            return;
        }
        *last = Some(Instant::now());
    } else {
        return;
    }
    let Some(ids) = super::fighter_ids() else {
        transport::revoke();
        return;
    };
    for id in ids.into_iter().flatten() {
        sample_object(id);
    }
    let projectiles = PROJECTILES.try_lock().map(|ids| *ids).unwrap_or([None; 32]);
    for id in projectiles.into_iter().flatten() {
        if game::is_active(id) {
            sample_object(id);
        } else if let Ok(mut ids) = PROJECTILES.try_lock() {
            for entry in ids.iter_mut().filter(|entry| **entry == Some(id)) {
                *entry = None;
            }
        }
    }
}
unsafe fn sample_object(id: u32) {
    if !transport::ready() || !game::is_active(id) {
        return;
    }
    let object = game::module_accessor(id);
    if object.is_null() || game::battle_object_id(object) != id {
        return;
    }
    let category = id >> 28;
    let fighter = category == battle_objects::OBJECT_CATEGORY_FIGHTER;
    if !fighter
        && !matches!(
            category,
            battle_objects::OBJECT_CATEGORY_WEAPON | battle_objects::OBJECT_CATEGORY_ITEM
        )
    {
        return;
    }
    let mut sample = Sample {
        session: 0,
        sequence: 0,
        sample_us: 0,
        object_id: id,
        category,
        kind: game::fighter_kind(id),
        group: 0,
        dropped: 0,
        values: [None; 64],
    };
    let v = &mut sample.values;
    let status = game::status_kind(object);
    let motion = game::motion_kind(object);
    integer(v, 0, status);
    integer(
        v,
        1,
        game::previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
    );
    integer(v, 2, game::situation_kind(object));
    v[3] = Some(motion);
    float(v, 4, game::motion_frame(object));
    float(v, 5, game::motion_rate(object));
    float(v, 6, game::motion_end_frame(object));
    float(v, 7, game::pos_x(object));
    float(v, 8, game::pos_y(object));
    float(v, 9, game::pos_z(object));
    float(v, 10, game::facing(object));
    let scale = game::scale(object);
    float(v, 11, scale);
    let velocity = [
        game::speed_x(object, game::ALL_KINETIC_ENERGY_RESERVES),
        game::speed_y(object, game::ALL_KINETIC_ENERGY_RESERVES),
    ];
    float(v, 12, velocity[0]);
    float(v, 13, velocity[1]);
    v[14] = Some(game::team(object));
    v[15] = Some(game::team_owner_id(object));
    let stopped = game::is_stop(object);
    boolean(v, 16, stopped);
    if fighter {
        let damage_stopped = stopped && game::is_damage_stop(object);
        boolean(v, 17, damage_stopped);
        v[18] = Some(if damage_stopped {
            game::hitstop_remaining(object)
        } else {
            0
        });
        float(v, 19, game::damage_percent(object, 0));
        v[20] = Some(game::hit_status(object, 0));
        boolean(v, 21, game::cancel_enabled(object));
        float(v, 22, game::cancel_frame(object, motion, false));
        float(v, 23, game::normal_hit_frame(object, motion, false));
        boolean(v, 24, game::passable_ground(object));
        boolean(v, 25, game::is_link(object, 0));
        boolean(v, 26, game::is_link(object, 2));
        float(v, 27, game::clatter_time(object, 0));
        float(
            v,
            31,
            game::param_float(object, native_params::PARAM_AIR_ACCEL_Y, 0),
        );
        float(
            v,
            32,
            game::param_float(object, native_params::PARAM_AIR_SPEED_Y_STABLE, 0),
        );
        float(
            v,
            33,
            game::param_float(object, native_params::PARAM_GROUND_BRAKE, 0),
        );
        if damage_stopped {
            float(
                v,
                34,
                game::work_float(
                    object,
                    native::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X,
                ),
            );
            float(
                v,
                35,
                game::work_float(
                    object,
                    native::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y,
                ),
            );
        }
        if status == native::GUARD_DAMAGE {
            integer(
                v,
                37,
                game::work_int(
                    object,
                    native::work_ids::FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME,
                ),
            );
        }
        if status == native::status::CLIFF_CATCH || status == native::status::CLIFF_WAIT {
            float(
                v,
                38,
                game::work_float(
                    object,
                    native::work_ids::FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME,
                ),
            );
        }
        if kirby_assist_core::tech_prediction::is_tumble_status(status) {
            float(
                v,
                39,
                game::work_float(
                    object,
                    native::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME,
                ),
            );
            boolean(
                v,
                40,
                game::is_flag(
                    object,
                    native::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE,
                ),
            );
        }
        for (index, term) in [
            (41, native::transition_terms::PASSIVE),
            (42, native::transition_terms::PASSIVE_FB),
            (43, native::transition_terms::PASSIVE_WALL),
            (45, native::transition_terms::PASSIVE_CEIL),
            (46, native::transition_terms::CONT_CATCH),
            (47, native::transition_terms::CONT_JUMP_SQUAT_BUTTON),
            (48, native::transition_terms::CONT_GUARD_ON),
            (49, native::transition_terms::CONT_ESCAPE_AIR),
        ] {
            boolean(v, index, game::transition_enabled(object, term));
        }
        boolean(v, 50, game::check_button_on(object, 0x3));
        boolean(v, 51, game::check_button_on(object, 0x10));
        float(
            v,
            52,
            game::param_float(object, native_params::PARAM_DAMAGE_FLY_TOP_GRAVITY, 0),
        );
        float(
            v,
            53,
            game::param_float(
                object,
                native_params::PARAM_DAMAGE_FLY_TOP_TERMINAL_SPEED,
                0,
            ),
        );
        let common = kirby_assist_core::timing::motion_hash("common");
        float(
            v,
            54,
            game::param_float(
                object,
                common,
                kirby_assist_core::timing::motion_hash("walk_stick_x"),
            ),
        );
        float(
            v,
            55,
            game::param_float(
                object,
                common,
                kirby_assist_core::timing::motion_hash("dash_stick_x"),
            ),
        );
        if sample.kind == kirby_assist_core::fighter_kinds::MR_GAME_AND_WATCH {
            float(
                v,
                56,
                game::work_float(
                    object,
                    native::work_ids::FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE,
                ),
            );
        }
        integer(
            v,
            57,
            game::work_int(
                object,
                native::work_ids::FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID,
            ),
        );
        if v[25] == Some(1) {
            v[58] = Some(game::link_node_object_id(object, 0));
        }
        boolean(
            v,
            59,
            game::transition_enabled(object, native::transition_terms::CONT_JUMP_SQUAT),
        );
        boolean(
            v,
            60,
            game::transition_enabled(object, native::transition_terms::CONT_ATTACK),
        );
        boolean(
            v,
            61,
            game::transition_enabled(object, native::transition_terms::CONT_SPECIAL_LW),
        );
    }
    v[62] = Some(16);
    v[63] = Some(32);
    boolean(
        v,
        28,
        game::attack_infliction_now(object, game::COLLISION_KIND_MASK_HIT),
    );
    let mut active = 0;
    for slot in 0..8 {
        if game::attack_active(object, slot, false) {
            active |= 1 << slot;
        }
    }
    v[30] = Some(active);
    if !game::is_active(id) || game::module_accessor(id) != object || !game::is_training_mode() {
        return;
    }
    transport::publish(sample);
    for slot in 0..8 {
        let mut attack = sample;
        attack.group = slot + 1;
        attack.values = [None; 64];
        let v = &mut attack.values;
        boolean(v, 0, active & (1 << slot) != 0);
        if active & (1 << slot) != 0 {
            let ptr = game::attack_geometry(object, i32::from(slot), false);
            if !ptr.is_null() {
                // Copy only the verified geometry prefix, not opaque capability bytes.
                let data = game::AttackGeometry {
                    x: (*ptr).x,
                    y: (*ptr).y,
                    z: (*ptr).z,
                    padding: 0,
                    x2: (*ptr).x2,
                    y2: (*ptr).y2,
                    z2: (*ptr).z2,
                    padding2: 0,
                    power: (*ptr).power,
                    size: (*ptr).size,
                    angle: (*ptr).angle,
                    kbg: (*ptr).kbg,
                    fkb: (*ptr).fkb,
                    bkb: (*ptr).bkb,
                    slip: (*ptr).slip,
                    hitlag: (*ptr).hitlag,
                    sdi: (*ptr).sdi,
                    node: (*ptr).node,
                    check_type: (*ptr).check_type,
                    situation: (*ptr).situation,
                    capability_padding: [0; 17],
                    reflectable: 0,
                    absorbable: 0,
                    direct: 0,
                };
                v[1] = Some(data.node);
                for (i, value) in [
                    (2, data.x),
                    (3, data.y),
                    (4, data.z),
                    (5, data.x2),
                    (6, data.y2),
                    (7, data.z2),
                    (8, data.size),
                    (9, data.power),
                    (14, data.hitlag),
                    (15, data.sdi),
                    (17, data.slip),
                ] {
                    float(v, i, value);
                }
                for (i, value) in [
                    (10, data.angle),
                    (11, data.kbg),
                    (12, data.fkb),
                    (13, data.bkb),
                ] {
                    integer(v, i, value);
                }
                v[23] = Some(u64::from(data.check_type));
                v[24] = Some(u64::from(data.situation));
                if let Some(world) = super::hitboxes::world_geometry(object, data, scale, velocity)
                {
                    for (i, value) in world
                        .start
                        .into_iter()
                        .chain(world.end)
                        .chain([world.radius])
                        .enumerate()
                    {
                        float(v, 25 + i, value);
                    }
                }
            }
        }
        if !game::is_active(id) || game::module_accessor(id) != object || !game::is_training_mode()
        {
            return;
        }
        transport::publish(attack);
    }
}
