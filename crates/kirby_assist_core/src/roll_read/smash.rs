//! Ordinary-roll punish: approaching/crossing rolls only, with reach and
//! vulnerability checked on the SAME future active frame. One attempt per roll.
use super::*;
use crate::timing::{ActiveWindow, RecoveryWindow};
impl RollRead {
    pub(super) fn sample_smash(
        &mut self,
        cfg: FighterConfig,
        c: Context,
        targets: [Option<Target>; 8],
    ) -> LedgeOutput {
        if matches!(c.status, 0x30..=0x32) {
            return self.end_smash();
        }
        let old_tracks = self.tracks;
        for (slot, target) in targets.iter().enumerate() {
            let Some(t) = target.filter(|t| basic_roll(*t, cfg, c)) else {
                self.tracks[slot] = None;
                continue;
            };
            let dx = t.position[0] - c.position[0];
            // Roster slots can reorder; match identity and native roll age.
            let old = old_tracks
                .iter()
                .flatten()
                .find(|p| p.id == t.id && p.status == t.status && t.frame >= p.frame);
            self.tracks[slot] = Some(Track {
                id: t.id,
                status: t.status,
                frame: t.frame,
                rate: t.rate,
                dx,
                used: old.is_some_and(|p| p.used),
                crossing: dx * t.velocity_x < -0.05 || old.is_some_and(|p| p.crossing),
            });
        }
        // Live target changes win over duplicate-own-clock caching.
        if self.target.is_some_and(|id| {
            !targets
                .iter()
                .flatten()
                .any(|t| t.id == id && valid(*t, cfg, c))
        }) {
            return self.end_smash();
        }
        if self.phase == 2
            && !targets
                .iter()
                .flatten()
                .any(|t| Some(t.id) == self.target && valid(*t, cfg, c) && contact(*t, cfg, c, 0.0))
        {
            return self.end_smash();
        }
        if c.stopped {
            return self.output;
        }
        let clock = (c.status, c.motion, c.frame);
        let mut elapsed = crate::timing::motion_elapsed(
            self.previous
                .map(|p| (p.0, p.1, p.2, self.previous_rate.max(0.01))),
            (c.status, c.motion, c.frame, c.rate),
            1.0,
        );
        // A native idle pose can stay frozen while the opponent rolls. Use the
        // selected roll's measured clock as fallback, never controller polls.
        if elapsed == 0.0 && self.phase != 0 {
            elapsed = targets
                .iter()
                .flatten()
                .find_map(|t| {
                    if Some(t.id) != self.target {
                        return None;
                    }
                    let old = old_tracks
                        .iter()
                        .flatten()
                        .find(|p| p.id == t.id && p.status == t.status)?;
                    Some(crate::timing::motion_elapsed(
                        Some((old.status, 1, old.frame, old.rate)),
                        (t.status, 1, t.frame, t.rate),
                        0.0,
                    ))
                })
                .unwrap_or(0.0);
            if elapsed == 0.0 {
                return self.output;
            }
        }
        if !elapsed.is_finite() {
            return self.end_smash();
        }
        self.previous = Some(clock);
        self.previous_rate = c.rate;
        if self.phase == 2 {
            self.request_age += elapsed;
            let hittable = targets.iter().flatten().any(|t| {
                Some(t.id) == self.target && valid(*t, cfg, c) && contact(*t, cfg, c, 0.0)
            });
            if self.request_age >= crate::move_input::ACCEPTANCE_TIMEOUT
                || !c.can_smash
                || !hittable
            {
                return self.end_smash();
            }
            self.output = if crate::move_input::delivery_pressed(self.request_age) {
                controls([0.0, -1.0], MAPPER_A)
            } else {
                controls([0.0; 2], 0)
            };
            return self.output;
        }
        let releasing_shield = matches!(c.status, 0x1b..=0x1d);
        if !c.can_act && !c.can_smash && !releasing_shield {
            return self.end_smash();
        }
        let primer = if matches!(self.phase, 1 | 4) {
            0.0
        } else {
            1.0
        };
        let delivery_lead = primer + shield_release_frames(c);
        let chosen = targets
            .iter()
            .enumerate()
            .filter_map(|(slot, t)| {
                let t = t.filter(|t| valid(*t, cfg, c))?;
                let track = self.tracks[slot]?;
                if track.used || !track.crossing || self.target.is_some_and(|id| id != t.id) {
                    return None;
                }
                let dx = projected(t, c);
                if !contact(t, cfg, c, delivery_lead) && !can_approach(t, cfg, c, dx) {
                    return None;
                }
                Some((slot, t, dx))
            })
            .min_by(|(_, _, a), (_, _, b)| a.abs().total_cmp(&b.abs()));
        let Some((slot, t, dx)) = chosen else {
            return self.end_smash();
        };
        self.target = Some(t.id);
        if releasing_shield {
            // Releasing ALL owned shield buttons also overrides a physical
            // held shield. Do not send Down (spot dodge) or A (shield grab)
            // until native shield release has completed and Smash is legal.
            self.phase = 4;
            self.output = controls([0.0; 2], 0);
            return self.output;
        }
        if contact(t, cfg, c, primer) {
            if matches!(self.phase, 1 | 4) && c.can_smash {
                self.phase = 2;
                self.request_age = 0.0;
                self.tracks[slot].as_mut().unwrap().used = true;
                self.output = controls(
                    crate::move_input::smash_stick(crate::move_input::Smash::Down, c.facing),
                    MAPPER_A,
                );
            } else {
                self.phase = 1;
                self.output = controls([0.0; 2], 0);
            }
            return self.output;
        }
        self.phase = 3;
        self.output = controls([0.0; 2], 0);
        if let Some(m) = movement(c) {
            let coast = (m.velocity_x * m.velocity_x / (2.0 * m.brake)).min(8.0);
            if c.can_act && dx.abs() > coast + 1.0 && m.safe_ground[usize::from(dx > 0.0)] {
                self.output = controls([dx.signum() * SMASH_WALK_STICK, 0.0], 0);
            }
        }
        self.output
    }
    fn end_smash(&mut self) -> LedgeOutput {
        self.phase = 0;
        self.target = None;
        self.output = LedgeOutput::default();
        self.output
    }
}
fn basic_roll(t: Target, cfg: FighterConfig, c: Context) -> bool {
    matches!(t.status, ESCAPE_F | ESCAPE_B)
        && t.frame.is_finite()
        && t.frame >= 0.0
        && t.rate.is_finite()
        && t.rate > 0.0
        && t.position.iter().all(|v| v.is_finite())
        && t.velocity_x.is_finite()
        && (t.position[0] - c.position[0]).abs() <= cfg.watch_distance
        && (t.position[1] - c.position[1]).abs() <= cfg.height
}
fn valid(t: Target, cfg: FighterConfig, c: Context) -> bool {
    basic_roll(t, cfg, c)
        && t.vulnerable_in
            .is_some_and(|v| v.is_finite() && (0.0..=120.0).contains(&v))
        && t.actionable_in
            .is_some_and(|v| v.is_finite() && v > f32::from(cfg.first) && v <= 120.0)
        && t.actionable_in.unwrap() > t.vulnerable_in.unwrap()
}
fn movement(c: Context) -> Option<Movement> {
    c.movement.filter(|m| {
        m.walk_speed.is_finite()
            && (0.01..=5.0).contains(&m.walk_speed)
            && m.velocity_x.is_finite()
            && m.brake.is_finite()
            && (0.001..=5.0).contains(&m.brake)
    })
}
fn relative_at(t: Target, c: Context, frames: f32) -> f32 {
    // Bounded forecast, not an exact animation endpoint; recheck each observation.
    let own = movement(c).map_or(0.0, |m| m.velocity_x * frames.min(2.0));
    t.position[0] + t.velocity_x * frames.min(6.0) - c.position[0] - own
}
fn projected(t: Target, c: Context) -> f32 {
    relative_at(t, c, t.vulnerable_in.unwrap())
}
fn contact(t: Target, cfg: FighterConfig, c: Context, primer: f32) -> bool {
    [(Some((cfg.first, cfg.last)), false), (cfg.back, true)]
        .into_iter()
        .any(|(window, rear)| {
            let Some((first, last)) = window else {
                return false;
            };
            (ActiveWindow {
                first: u16::from(first),
                last: u16::from(last),
            })
            .first_contact(
                primer,
                RecoveryWindow {
                    vulnerable_in: t.vulnerable_in,
                    actionable_in: t.actionable_in,
                },
                |frames| {
                    let dx = relative_at(t, c, frames);
                    dx.abs() <= cfg.reach
                        && (cfg.back.is_none()
                            || if rear {
                                dx * c.facing <= 1.0
                            } else {
                                dx * c.facing >= -1.0
                            })
                },
            )
            .is_some()
        })
}
fn can_approach(t: Target, cfg: FighterConfig, c: Context, dx: f32) -> bool {
    let release = shield_release_frames(c);
    if dx.abs() <= cfg.reach {
        return t.actionable_in.unwrap()
            > t.vulnerable_in
                .unwrap()
                .max(release + f32::from(cfg.first) + 1.0);
    }
    let Some(m) = movement(c) else {
        return false;
    };
    if (!c.can_act && !matches!(c.status, 0x1b..=0x1d)) || !m.safe_ground[usize::from(dx > 0.0)] {
        return false;
    }
    let travel = (dx.abs() - cfg.reach + 1.0) / (m.walk_speed * SMASH_WALK_STICK);
    let brake = m.velocity_x.abs() / m.brake;
    release + travel + brake + 1.0 + f32::from(cfg.first) < t.actionable_in.unwrap()
}

/// Universal ordinary shield release is 11 frames. Account for remaining
/// entry commitment conservatively, and remaining native exit animation time.
/// This is eligibility only: actual A delivery still waits for native readiness.
fn shield_release_frames(c: Context) -> f32 {
    match c.status {
        0x1b => 11.0 + ((3.0 - c.frame).max(0.0) / c.rate),
        0x1c => 11.0,
        0x1d => (11.0 - c.frame).max(0.0) / c.rate,
        _ => 0.0,
    }
}
