//! Read-only damage environment observations. Cache copied bounds per hit;
//! refresh at launch-input handoff after SDI, and never retain game pointers.
//! Technical: Converts short-lived native damage-state reads into copied blast
//! zone geometry and optional trajectory estimates.
//! Plain English: This module estimates where a launched fighter may travel without
//! storing game-owned pointers.
use crate::game_ffi as game;
use kirby_assist_core::auto_di::{BlastZone, Trajectory};
/// Cache key containing the object id, damage-stop countdown, and copied blast-zone geometry.
pub(super) type GeometryCache = (u32, u64, Option<BlastZone>);
/// Observes the damage environment for a given object, caching geometry and computing trajectory if applicable.
/// Reuses matching cached blast-zone bounds and computes trajectory data
/// only during the bounded damage-stop observation window.
/// # Safety
/// The object pointer must be valid for the duration of the native reads.
pub(super) unsafe fn observe(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    object_id: u32,
    damage_stop: bool,
    remaining: u64,
    cached: Option<GeometryCache>,
) -> (Option<GeometryCache>, Option<Trajectory>) {
    if !damage_stop || remaining > 120 {
        return (None, None);
    }
    let position = [reads.pos_x(object), reads.pos_y(object)];
    let previous = cached.filter(|(id, last, _)| {
        *id == object_id && remaining <= *last && !(remaining <= 3 && *last > 3)
    });
    let blast = if let Some((_, _, blast)) = previous {
        blast
    } else {
        let z = game::pos_z(object);
        if z.is_finite() {
            BlastZone::observe(position, |p| {
                game::check_dead_area(&game::Vector3 {
                    x: p[0],
                    y: p[1],
                    z,
                })
            })
        } else {
            None
        }
    };
    let geometry = Some((object_id, remaining, blast));
    let trajectory = blast.and_then(|blast| {
        let launch = reads.damage_launch(object);
        let angle = launch[1].atan2(launch[0]).to_degrees();
        // Native damage-fly-top uses its own gravity/fall-speed parameters.
        // The original upward launch sector selects this mode, before DI.
        let top = launch.iter().all(|v| v.is_finite()) && (70.0..=110.0).contains(&angle);
        let physics = kirby_assist_core::gameplay::AirPhysics::new(
            game::param_float(
                object,
                if top {
                    super::native_params::PARAM_DAMAGE_FLY_TOP_GRAVITY
                } else {
                    super::native_params::PARAM_AIR_ACCEL_Y
                },
                0,
            ),
            game::param_float(
                object,
                if top {
                    super::native_params::PARAM_DAMAGE_FLY_TOP_TERMINAL_SPEED
                } else {
                    super::native_params::PARAM_AIR_SPEED_Y_STABLE
                },
                0,
            ),
        )?;
        Some(Trajectory {
            position,
            blast,
            hitstun: game::work_float(
                object,
                kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME,
            ),
            gravity: physics.gravity,
            fall_speed: physics.terminal_speed,
        })
    });
    (geometry, trajectory)
}
