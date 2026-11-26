//! Shared, bounded recovery punish for spot dodges, avoided grabs and far rolls.
//! Reach values are calibration estimates; native recovery and readiness gate inputs.
use crate::{
    gameplay::ControllerOutput,
    native_input::MAPPER_A,
    timing::{motion_elapsed, ActiveWindow, RecoveryWindow},
};
/// Available `Kind` choices for the Smash assistance menu.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `Spot` option in the `Kind` Smash policy state.
    Spot,
    /// `Grab` option in the `Kind` Smash policy state.
    Grab,
    /// `Roll` option in the `Kind` Smash policy state.
    Roll,
}
/// Runtime Smash data carried by `Target`.
#[derive(Clone, Copy)]
pub struct Target {
    /// battle-object ID.
    pub id: u32,
    /// native fighter-kind ID.
    pub kind: Kind,
    /// native fighter status.
    pub status: i32,
    /// native animation/motion ID.
    pub motion: u64,
    /// current native animation frame.
    pub frame: f32,
    /// native animation playback rate.
    pub rate: f32,
    /// world-space position.
    pub position: [f32; 2],
    /// world-space velocity.
    pub velocity: [f32; 2],
    /// Runtime opens value used by Target..
    pub opens: f32,
    /// Runtime closes value used by Target..
    pub closes: f32,
}
/// Runtime Smash data carried by `Context`.
#[derive(Clone, Copy)]
pub struct Context {
    /// Runtime object value used by Context..
    pub object: u32,
    /// native fighter status.
    pub status: i32,
    /// native animation/motion ID.
    pub motion: u64,
    /// current native animation frame.
    pub frame: f32,
    /// native animation playback rate.
    pub rate: f32,
    /// Runtime stopped value used by Context..
    pub stopped: bool,
    /// grounded-state observation.
    pub grounded: bool,
    /// Runtime can act value used by Context..
    pub can_act: bool,
    /// Runtime down ready value used by Context..
    pub down_ready: bool,
    /// Runtime forward ready value used by Context..
    pub forward_ready: bool,
    /// left/right facing sign.
    pub facing: f32,
    /// world-space position.
    pub position: [f32; 2],
    /// world-space velocity.
    pub velocity: [f32; 2],
}
impl Context {
    /// Grounded with a usable motion clock and a known facing.
    fn ready_to_read(&self) -> bool {
        self.grounded
            && crate::timing::clock_known(self.motion, self.frame, self.rate)
            && self.facing.is_finite()
            && self.facing.abs() >= 0.5
    }
}
/// Runtime Smash data carried by `Config`.
#[derive(Clone, Copy)]
pub struct Config {
    /// Runtime down value used by Config..
    pub down: [u8; 2],
    /// Runtime down back value used by Config..
    pub down_back: Option<[u8; 2]>,
    /// Runtime forward value used by Config..
    pub forward: [u8; 2],
    /// Runtime down reach value used by Config..
    pub down_reach: f32,
    /// Runtime forward reach value used by Config..
    pub forward_reach: f32,
}
/// Runtime Smash data carried by `Track`.
#[derive(Clone, Copy)]
struct Track {
    /// battle-object ID.
    id: u32,
    /// native fighter status.
    status: i32,
    /// native animation/motion ID.
    motion: u64,
    /// current native animation frame.
    frame: f32,
    /// native animation playback rate.
    rate: f32,
    /// Runtime used value used by Track..
    used: bool,
}
const SEEKING_TARGET: u8 = 0;
const PRIMING_ATTACK: u8 = 1;
const AWAITING_ACCEPTANCE: u8 = 2;

/// Runtime Smash data carried by `Punisher`.
#[derive(Default)]
pub struct Punisher {
    /// Runtime object value used by Punisher..
    object: Option<u32>,
    /// Runtime tracks value used by Punisher..
    tracks: [Option<Track>; 8],
    /// Runtime previous value used by Punisher..
    previous: Option<(i32, u64, f32, f32)>,
    /// selected opponent/projectile target.
    target: Option<u32>,
    /// Runtime phase value used by Punisher..
    phase: u8,
    /// Runtime forward value used by Punisher..
    forward: bool,
    attack_age_frames: f32,
    /// controller input output.
    output: ControllerOutput,
}
impl Punisher {
    /// Checks whether an input owner has committed the action.
    pub fn committed(&self) -> bool {
        self.target.is_some() && self.output.owned != 0
    }
    /// Cancels the pending.
    pub fn cancel_pending(&mut self) {
        self.target = None;
        self.phase = SEEKING_TARGET;
        self.output = ControllerOutput::default();
    }
    /// Updates the toggle state from the current button sample and match state.
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        cfg: Config,
        targets: [Option<Target>; 8],
    ) -> ControllerOutput {
        self.sample_with_height(enabled, c, cfg, targets, 8.)
    }
    /// Samples the with height.
    pub fn sample_with_height(
        &mut self,
        enabled: bool,
        c: Context,
        cfg: Config,
        targets: [Option<Target>; 8],
        height: f32,
    ) -> ControllerOutput {
        if self.object != Some(c.object) {
            *self = Self::default();
            self.object = Some(c.object);
        }
        let old = self.refresh_tracks(&targets);
        if !enabled || !c.ready_to_read() {
            self.cancel_pending();
            self.previous = None;
            return self.output;
        }
        if matches!(
            c.status,
            crate::native_states::status::ATTACK_S4_START
                ..=crate::native_states::status::ATTACK_LW4
        ) {
            self.cancel_pending();
            return self.output;
        }
        if c.stopped {
            return self.output;
        }
        let delta = self.clock_delta(&c, &targets, old);
        if !delta.is_finite()
            || !(0.0..=8.0).contains(&delta)
            || !self.target_in_contact(c, cfg, &targets, height)
        {
            self.cancel_pending();
            return self.output;
        }
        if self.phase != SEEKING_TARGET && delta == 0.0 {
            return self.output;
        }
        if self.phase == AWAITING_ACCEPTANCE {
            return self.retry_attack(c.facing, delta);
        }
        let shield = shield_frames(&c);
        let in_guard = matches!(
            c.status,
            crate::native_states::GUARD_ON..=crate::native_states::GUARD_OFF
        );
        if !c.can_act && shield == 0.0 && !(in_guard && (c.down_ready || c.forward_ready)) {
            self.cancel_pending();
            return self.output;
        }
        let setup = shield
            + if self.phase == PRIMING_ATTACK {
                0.0
            } else {
                1.0
            };
        let Some((target_slot, target, forward, horizontal_offset)) =
            self.select_target(c, cfg, &targets, setup, height)
        else {
            self.cancel_pending();
            return self.output;
        };
        self.target = Some(target.id);
        self.forward = forward;
        self.output = crate::move_input::output([0.0; 2], 0);
        if forward && horizontal_offset * c.facing < 0.5 {
            if shield == 0.0 && c.can_act {
                self.output.stick = Some([horizontal_offset.signum() * 0.3, 0.0]);
            }
            self.phase = SEEKING_TARGET;
            return self.output;
        }
        // Wait out the shield, then one observed frame, before attacking.
        if shield > 0.0 || self.phase != PRIMING_ATTACK {
            self.phase = PRIMING_ATTACK;
            return self.output;
        }
        if !(if forward {
            c.forward_ready
        } else {
            c.down_ready
        }) {
            return self.output;
        }
        self.phase = AWAITING_ACCEPTANCE;
        self.attack_age_frames = 0.0;
        if let Some(track) = self.tracks[target_slot].as_mut() {
            track.used = true;
        }
        self.output = attack_output(forward, c.facing);
        self.output
    }

    /// Remember each target's motion clock; `used` survives while the same
    /// motion event continues. Returns the previous tracks.
    fn refresh_tracks(&mut self, targets: &[Option<Target>; 8]) -> [Option<Track>; 8] {
        let old = self.tracks;
        for (track, target) in self.tracks.iter_mut().zip(targets) {
            *track = target.map(|t| Track {
                id: t.id,
                status: t.status,
                motion: t.motion,
                frame: t.frame,
                rate: t.rate,
                used: old.iter().flatten().any(|p| {
                    crate::timing::same_motion_event(
                        (p.id, p.status, p.motion, p.frame),
                        (t.id, t.status, t.motion, t.frame),
                    ) && p.used
                }),
            });
        }
        old
    }

    /// Elapsed frames of our own clock, else of the committed target's, else NaN
    /// when either clock is invalid or rewound.
    fn clock_delta(
        &mut self,
        c: &Context,
        targets: &[Option<Target>; 8],
        old: [Option<Track>; 8],
    ) -> f32 {
        let clock = (c.status, c.motion, c.frame, c.rate);
        let delta = motion_elapsed(self.previous, clock, 0.0);
        self.previous = Some(clock);
        if delta != 0.0 {
            return delta;
        }
        let previous_clock = targets
            .iter()
            .flatten()
            .find(|t| Some(t.id) == self.target)
            .and_then(|t| {
                let p = old
                    .iter()
                    .flatten()
                    .find(|p| p.id == t.id && p.motion == t.motion)?;
                Some((p, t))
            });
        previous_clock.map_or(delta, |(p, t)| {
            motion_elapsed(
                Some((p.status, p.motion, p.frame, p.rate)),
                (t.status, t.motion, t.frame, t.rate),
                0.0,
            )
        })
    }

    /// A committed target must stay in reach of the committed attack.
    fn target_in_contact(
        &self,
        c: Context,
        cfg: Config,
        targets: &[Option<Target>; 8],
        height: f32,
    ) -> bool {
        self.target.is_none_or(|id| {
            targets
                .iter()
                .flatten()
                .any(|t| t.id == id && contact(c, cfg, *t, self.forward, 0.0, height))
        })
    }

    /// First untouched target (or the committed one) already in reach once
    /// `setup` frames have elapsed. Returns its slot, side and horizontal offset.
    fn select_target(
        &self,
        c: Context,
        cfg: Config,
        targets: &[Option<Target>; 8],
        setup: f32,
        height: f32,
    ) -> Option<(usize, Target, bool, f32)> {
        for (i, t) in targets.iter().enumerate() {
            let Some(t) = t else { continue };
            if self.tracks[i].is_some_and(|track| track.used)
                || self.target.is_some_and(|id| id != t.id)
            {
                continue;
            }
            let at = t.opens.clamp(0.0, 120.0);
            let x = t.position[0] - c.position[0] + (t.velocity[0] - c.velocity[0]) * at;
            let forward = x.abs() > cfg.down_reach;
            if t.kind == Kind::Roll && !forward {
                continue;
            } // close rolls retain the existing down-smash reader
            if contact(c, cfg, *t, forward, setup, height) {
                return Some((i, *t, forward, x));
            }
        }
        None
    }

    /// Hold the committed smash while native acceptance is pending, repeating the
    /// press on the game's retry schedule until it times out.
    fn retry_attack(&mut self, facing: f32, delta: f32) -> ControllerOutput {
        self.attack_age_frames += delta;
        if self.attack_age_frames >= crate::move_input::ACCEPTANCE_TIMEOUT {
            self.cancel_pending();
        } else if crate::move_input::smash_retry_pressed(
            self.attack_age_frames,
            self.target.unwrap_or_default(),
        ) {
            self.output = attack_output(self.forward, facing);
        } else {
            self.output = crate::move_input::output([0.0; 2], 0);
        }
        self.output
    }
}
/// Returns the measured number of shield frames.
fn shield_frames(c: &Context) -> f32 {
    match c.status {
        crate::native_states::GUARD_ON..=crate::native_states::GUARD => 11.0,
        crate::native_states::GUARD_OFF => (11.0 - c.frame).max(0.0),
        _ => 0.0,
    }
}
/// Returns the attack output requested by the current response.
fn attack_output(forward: bool, facing: f32) -> ControllerOutput {
    crate::move_input::output(
        if forward {
            [facing.signum(), 0.0]
        } else {
            [0.0, -1.0]
        },
        MAPPER_A,
    )
}
/// Checks whether the observed interaction includes contact.
fn contact(c: Context, cfg: Config, t: Target, forward: bool, setup: f32, height: f32) -> bool {
    if !t.frame.is_finite()
        || !t.rate.is_finite()
        || t.rate <= 0.0
        || !t.opens.is_finite()
        || t.opens < 0.0
        || !t.closes.is_finite()
        || t.closes <= t.opens
        || t.motion == crate::timing::NO_MOTION_KIND
        || !c
            .position
            .iter()
            .chain(c.velocity.iter())
            .chain(t.position.iter())
            .chain(t.velocity.iter())
            .all(|v| v.is_finite())
    {
        return false;
    }
    let reach = if forward {
        cfg.forward_reach
    } else {
        cfg.down_reach
    };
    let windows = if forward {
        [Some((cfg.forward, false)), None]
    } else {
        [Some((cfg.down, false)), cfg.down_back.map(|w| (w, true))]
    };
    windows.into_iter().flatten().any(|(frames, back)| {
        ActiveWindow {
            first: frames[0] as u16,
            last: frames[1] as u16,
        }
        .first_contact(
            setup,
            RecoveryWindow {
                vulnerable_in: Some(t.opens),
                actionable_in: Some(t.closes),
            },
            |at| {
                let x = t.position[0] - c.position[0] + (t.velocity[0] - c.velocity[0]) * at;
                let y = t.position[1] - c.position[1] + (t.velocity[1] - c.velocity[1]) * at;
                let side_ok = forward
                    || cfg.down_back.is_none()
                    || x.abs() <= 0.001
                    || (x * c.facing < 0.) == back;
                side_ok && x.abs() <= reach && y.abs() <= height
            },
        )
        .is_some()
    })
}
