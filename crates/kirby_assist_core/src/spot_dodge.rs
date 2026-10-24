//! Constant, input-only ordinary-grab defense. Root-space geometry is an
//! approximation, not proof of collision or a guaranteed dodge online.
use crate::{ledge_trap::LedgeOutput, native_input::MAPPER_ZR, timing::motion_elapsed};
pub const LOOKAHEAD: f32 = 4.0;
pub const ESCAPE: i32 = 0x1f;
#[derive(Clone, Copy)]
pub struct Context {
    pub object: u32,
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub can_spot_dodge: bool,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}
pub fn ground_status(status: i32) -> bool {
    matches!(status, 0..=9 | 0x11..=0x15 | 0x1b..=0x1d)
}
#[derive(Clone, Copy)]
pub struct Target {
    pub id: u32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub facing: f32,
    pub first: u8,
    pub last: u8,
    pub reach: f32,
}
#[derive(Clone, Copy)]
struct Track {
    id: u32,
    status: i32,
    motion: u64,
    frame: f32,
    rate: f32,
    used: bool,
}

pub fn contact(c: Context, t: Target) -> Option<f32> {
    if !matches!(t.status, 0x37 | 0x39 | 0x3b)
        || t.motion == 0
        || !t.frame.is_finite()
        || t.frame < 0.0
        || !t.rate.is_finite()
        || t.rate <= 0.0
        || !t.facing.is_finite()
        || t.facing.abs() < 0.5
        || t.first == 0
        || t.last < t.first
        || t.last > 120
        || !t.reach.is_finite()
        || !(1.0..=60.0).contains(&t.reach)
        || !c
            .position
            .iter()
            .chain(c.velocity.iter())
            .chain(t.position.iter())
            .chain(t.velocity.iter())
            .all(|v| v.is_finite())
    {
        return None;
    }
    let first = ((f32::from(t.first) - t.frame) / t.rate).max(0.0);
    let last = (f32::from(t.last) - t.frame) / t.rate;
    if first > LOOKAHEAD || last < 0.0 {
        return None;
    }
    // Check the same future time for active grab, facing and root separation.
    for step in 0..=8 {
        let at = first + step as f32 * 0.5;
        if at > last || at > LOOKAHEAD {
            break;
        }
        let dx = c.position[0] - t.position[0] + (c.velocity[0] - t.velocity[0]) * at;
        let dy = c.position[1] - t.position[1] + (c.velocity[1] - t.velocity[1]) * at;
        if (-2.0..=t.reach + 2.0).contains(&(dx * t.facing)) && dy.abs() <= 8.0 {
            return Some(at);
        }
    }
    None
}

#[derive(Default)]
pub struct SpotDodge {
    object: Option<u32>,
    previous: Option<(i32, u64, f32, f32)>,
    tracks: [Option<Track>; 8],
    target: Option<u32>,
    age: f32,
    output: LedgeOutput,
}
impl SpotDodge {
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn cancel_pending(&mut self) {
        self.target = None;
        self.output = LedgeOutput::default();
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        if self.object != Some(c.object) {
            *self = Self::default();
            self.object = Some(c.object);
        }
        if enabled
            && c.stopped
            && c.situation == 0
            && ground_status(c.status)
            && c.motion != 0
            && c.frame.is_finite()
            && c.frame >= 0.0
            && c.rate.is_finite()
            && c.rate > 0.0
        {
            return self.output;
        }
        let old = self.tracks;
        let mut threat: Option<(usize, f32)> = None;
        for (slot, entry) in targets.iter().enumerate() {
            let Some(t) = entry else {
                self.tracks[slot] = None;
                continue;
            };
            if !matches!(t.status, 0x37 | 0x39 | 0x3b)
                || t.motion == 0
                || !t.frame.is_finite()
                || t.frame < 0.0
            {
                self.tracks[slot] = None;
                continue;
            }
            let used = old.iter().flatten().any(|p| {
                p.id == t.id
                    && p.status == t.status
                    && p.motion == t.motion
                    && t.frame >= p.frame
                    && p.used
            });
            self.tracks[slot] = Some(Track {
                id: t.id,
                status: t.status,
                motion: t.motion,
                frame: t.frame,
                rate: t.rate,
                used,
            });
            if !used {
                if let Some(at) = contact(c, *t) {
                    if threat.is_none_or(|(_, best)| at < best) {
                        threat = Some((slot, at));
                    }
                }
            }
        }
        if !enabled
            || c.situation != 0
            || c.motion == 0
            || !c.frame.is_finite()
            || c.frame < 0.0
            || !c.rate.is_finite()
            || c.rate <= 0.0
            || !ground_status(c.status)
        {
            self.cancel_pending();
            self.previous = None;
            return self.output;
        }
        let stamp = (c.status, c.motion, c.frame, c.rate);
        let mut elapsed = motion_elapsed(self.previous, stamp, 0.0);
        if elapsed == 0.0 {
            if let Some(t) = targets.iter().flatten().find(|t| Some(t.id) == self.target) {
                if let Some(p) = old
                    .iter()
                    .flatten()
                    .find(|p| p.id == t.id && p.status == t.status && p.motion == t.motion)
                {
                    elapsed = motion_elapsed(
                        Some((p.status, p.motion, p.frame, p.rate)),
                        (t.status, t.motion, t.frame, t.rate),
                        0.0,
                    );
                }
            }
        }
        self.previous = Some(stamp);
        if !elapsed.is_finite() || !(0.0..=8.0).contains(&elapsed) {
            self.cancel_pending();
            return self.output;
        }
        if let Some(id) = self.target {
            self.age += elapsed;
            if self.age >= crate::move_input::ACCEPTANCE_TIMEOUT
                || !targets
                    .iter()
                    .flatten()
                    .any(|t| t.id == id && contact(c, *t).is_some())
            {
                self.cancel_pending();
            } else {
                self.output = if crate::move_input::delivery_pressed(self.age) {
                    crate::move_input::output([0.0, -1.0], MAPPER_ZR)
                } else {
                    crate::move_input::output([0.0; 2], 0)
                };
            }
            return self.output;
        }
        self.output = LedgeOutput::default();
        if !c.can_spot_dodge {
            return self.output;
        }
        if let Some((slot, _)) = threat {
            let track = self.tracks[slot].as_mut().unwrap();
            track.used = true;
            self.target = Some(track.id);
            self.age = 0.0;
            // Down+Guard requests spot dodge, with no horizontal roll input.
            self.output = crate::move_input::output([0.0, -1.0], MAPPER_ZR);
        }
        self.output
    }
}
