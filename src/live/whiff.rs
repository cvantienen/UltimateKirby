//! Parked prototype: not registered by live.rs and never runs in the live build.
//! Bounded read-only recovery observations. No new hook, roster enumeration,
//! geometry scan or file access. All object pointers are reacquired per sample.
use super::{game, ground_support, STATE};
use kirby_assist_core::{
    finisher,
    opponent_data::OpponentProfiles,
    whiff_punish::{Context, Target},
};
pub(super) struct Observation {
    pub context: Context,
    pub targets: [Option<Target>; 8],
    pub profiles: OpponentProfiles,
}
pub(super) unsafe fn observe(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    id: u32,
    starter: finisher::Context,
) -> Observation {
    let (roster, mut profiles) =
        STATE
            .lock()
            .ok()
            .map_or(([None; 8], OpponentProfiles::default()), |s| {
                (
                    s.ledge_roster.map_or([None; 8], |r| r.0),
                    s.opponent_profiles,
                )
            });
    let position = [reads.pos_x(object), reads.pos_y(object)];
    let vx = reads.speed_x(object, -1);
    let team = reads.team(object);
    let mut targets = [None; 8];
    for (slot, enemy_id) in roster.into_iter().enumerate() {
        let Some(enemy_id) = enemy_id.filter(|x| *x != id && *x >> 28 == 0 && game::is_active(*x))
        else {
            profiles.clear(slot);
            continue;
        };
        let enemy = game::module_accessor(enemy_id);
        if enemy.is_null()
            || enemy.cast::<u8>().add(8).cast::<u32>().read() != enemy_id
            || reads.team(enemy) == team
        {
            profiles.clear(slot);
            continue;
        }
        let Some(reference) = profiles.bind(slot, enemy_id, game::fighter_kind(enemy_id)) else {
            continue;
        };
        let status = reads.status_kind(enemy);
        if !matches!(status, 0x29..=0x2c | 0x2f | 0x32 | 0x35) || reads.situation_kind(enemy) != 0 {
            continue;
        }
        let at = [reads.pos_x(enemy), reads.pos_y(enemy)];
        if (at[0] - position[0]).abs() > 40.0 || (at[1] - position[1]).abs() > 5.0 {
            continue;
        }
        let motion = reads.motion_kind(enemy);
        let Some(data) = reference
            .move_for_motion(motion)
            .filter(|m| m.whiff_eligible)
        else {
            continue;
        };
        let (Some(last), Some(total)) = (data.last_active(), data.total) else {
            continue;
        };
        let frame = reads.motion_frame(enemy);
        if !frame.is_finite() || frame<=f32::from(last) || game::cancel_enabled(enemy)
            || game::hit_status(enemy,0)!=0
            // Pinned COLLISION_KIND_MASK_HIT=2, SHIELD=4. Persistent per-attack
            // contact rejects a blocked/connected move even after hitstop ends.
            || game::attack_infliction(enemy,game::COLLISION_KIND_MASK_HIT|0x4)
            || (0..8).any(|n|game::attack_active(enemy,n,false))
        {
            continue;
        }
        let end = reads.motion_end_frame(enemy);
        // A changed/unknown animation length is not a matching reference move.
        if !end.is_finite() || (end - f32::from(total)).abs() > 2.0 {
            continue;
        }
        let Some(remaining) =
            data.recovery(kirby_assist_core::opponent_data::RecoveryObservation {
                motion,
                status,
                grounded: true,
                frame,
                rate: reads.motion_rate(enemy),
                end,
                cancel: game::cancel_frame(enemy, motion, false),
                can_cancel: false,
                active_attack: false,
                registered_contact: false,
                vulnerable: true,
            })
        else {
            continue;
        };
        targets[slot] = Some(Target {
            id: enemy_id,
            kind: reference.kind,
            motion,
            frame,
            position: at,
            velocity_x: reads.speed_x(enemy, -1),
            remaining,
        });
    }
    let near = targets.iter().any(Option::is_some);
    Observation {
        context: Context {
            starter,
            position,
            velocity_x: vx,
            walk_speed: near.then(|| {
                game::param_float(
                    object,
                    kirby_assist_core::timing::motion_hash("walk_speed_max"),
                    0,
                )
            }),
            can_tilt: near && game::transition_enabled(object, 0x1e000057),
            can_smash: near && game::transition_enabled(object, 0x1e000059), // CONT_ATTACK_S4_START
            safe_ground: if near && starter.fighter.grounded {
                ground_support(object, position, vx)
            } else {
                [false; 2]
            },
        },
        targets,
        profiles,
    }
}
