//! Match-owned ledge memory, independent of temporary input owners and L releases.
//! Only already-observed opponents are inspected; no roster/geometry queries.
use super::{
    finite2, finite_opponent, is_hanging, is_ledge_option, AssistSettings, ControllerOutput,
    GameplayContext, LedgeTrap, Opponent,
};
use crate::ledge_position::LedgeSpot;

/// Defines the `Visit` data structure.
///
///
///  Holds the data for Visit.
#[derive(Clone, Copy)]
struct Visit {
    /// Stores the `id` value in `Visit` as `u32`.
    ///
    ///
///  Holds the id value used by Visit.
    id: u32,
    /// Stores the `last_status` value in `Visit` as `i32`.
    ///
    ///
///  Holds the last status value used by Visit.
    last_status: i32,
    /// Stores the `armed` value in `Visit` as `bool`.
    ///
    ///
///  Holds the armed value used by Visit.
    armed: bool,
    /// Stores the `handled` value in `Visit` as `bool`.
    ///
    ///
///  Holds the handled value used by Visit.
    handled: bool,
}
/// Defines the `LedgeVisits` data structure.
///
///
///  Holds the data for LedgeVisits.
#[derive(Default)]
pub struct LedgeVisits {
    /// Stores the `owner` value in `LedgeVisits` as `Option<(u32`.
    ///
    ///
///  Optionally stores the owner for LedgeVisits; it may be absent.
    owner: Option<(u32, i32)>,
    /// Stores the `visits` value in `LedgeVisits` as `[Option<Visit>; 8]`.
    ///
    ///
///  Stores the visits collection for LedgeVisits.
    visits: [Option<Visit>; 8],
    /// Stores the `replace` value in `LedgeVisits` as `usize`.
    ///
    ///
///  Holds the replace value used by LedgeVisits.
    replace: usize,
    /// Stores the `ledge` value in `LedgeVisits` as `Option<LedgeSpot>`.
    ///
    ///
///  Optionally stores the ledge for LedgeVisits; it may be absent.
    ledge: Option<LedgeSpot>,
    /// Stores the `visit_epoch` value in `LedgeVisits` as `u64`.
    ///
    ///
///  Holds the visit epoch value used by LedgeVisits.
    visit_epoch: u64,
}
/// Defines the `Observation` data structure.
///
///
///  Holds the data for Observation.
#[derive(Clone, Copy)]
struct Observation {
    /// Stores the `new_visit` value in `Observation` as `bool`.
    ///
    ///
///  Holds the new visit value used by Observation.
    new_visit: bool,
    /// Stores the `allowed` value in `Observation` as `bool`.
    ///
    ///
///  Holds the allowed value used by Observation.
    allowed: bool,
}
impl LedgeVisits {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
///  Creates a new value.
    pub const fn new() -> Self {
        Self {
            owner: None,
            visits: [None; 8],
            replace: 0,
            ledge: None,
            visit_epoch: 0,
        }
    }
    /// Sets the owner.
    ///
    ///
///  Sets the owner.
    pub fn set_owner(&mut self, id: u32, kind: i32) {
        if self.owner != Some((id, kind)) {
            *self = Self::default();
            self.owner = Some((id, kind));
        }
    }
    /// Visits the epoch.
    ///
    ///
///  Visits the epoch.
    pub fn visit_epoch(&self) -> u64 {
        self.visit_epoch
    }
    /// Returns an optional `walking_ledge` value.
    ///
    ///
///  Gets walking ledge if it is available.
    pub fn walking_ledge(&self) -> Option<LedgeSpot> {
        self.ledge
    }
    /// Records the time of the latest fighter observation.
    ///
    ///
///  Remembers when a fighter was last seen.
    fn observe(&mut self, e: Opponent, own: [f32; 2]) -> Observation {
        let index = self
            .visits
            .iter()
            .position(|v| v.is_some_and(|v| v.id == e.id))
            .or_else(|| self.visits.iter().position(Option::is_none))
            .unwrap_or_else(|| {
                let i = self.replace;
                self.replace = (i + 1) % 8;
                i
            });
        let old = self.visits[index].filter(|v| v.id == e.id);
        // A real new cliff state can also reveal a visit whose hang was missed.
        // A standing fighter's sticky previous_status is never a new visit.
        let new_visit = is_hanging(e.status) && old.is_none_or(|v| !is_hanging(v.last_status))
            || is_ledge_option(e.status)
                && old
                    .is_none_or(|v| !is_hanging(v.last_status) && !is_ledge_option(v.last_status));
        let mut visit = old.unwrap_or(Visit {
            id: e.id,
            last_status: e.status,
            armed: false,
            handled: false,
        });
        if new_visit {
            visit.armed = true;
            visit.handled = false;
            self.visit_epoch = self.visit_epoch.wrapping_add(1);
            // Keep the last real grabbed edge until a fresh cliff anchor is
            // available; a clock-only regrab observation supplies no new edge.
        } else if old.is_none()
            && is_hanging(e.previous_status)
            && e.situation == crate::native_states::situation::AIR
        {
            visit.armed = true; // First observation of an actual ledge release/drop.
        }
        if is_hanging(e.status) {
            if let Some(edge) = e.cliff_position.filter(|p| finite2(*p)) {
                let previous = self
                    .ledge
                    .filter(|s| {
                        (s.edge[0] - edge[0]).abs() <= 2.0 && (s.edge[1] - edge[1]).abs() <= 2.0
                    })
                    .map(|s| s.inward);
                let dx = own[0] - edge[0];
                let inward = previous
                    .or_else(|| (dx.is_finite() && dx.abs() > 0.001).then_some(dx.signum()));
                if let Some(inward) = inward {
                    self.ledge = Some(LedgeSpot { edge, inward });
                }
            }
        }
        visit.last_status = e.status;
        let allowed = visit.armed && !visit.handled;
        self.visits[index] = Some(visit);
        Observation { new_visit, allowed }
    }
    /// Updates the `claim` state using the supplied inputs.
    ///
    ///
///  Updates claim.
    fn claim(&mut self, id: u32) {
        if let Some(v) = self.visits.iter_mut().flatten().find(|v| v.id == id) {
            v.handled = true;
        }
    }
}

/// Preserve one response per visit even when this engine is recreated.
/// Explicit editor previews remain independently repeatable.
impl LedgeTrap {
    /// Samples the with visits.
    ///
    ///
///  Samples the with visits.
    pub fn sample_with_visits(
        &mut self,
        visits: &mut LedgeVisits,
        settings: AssistSettings,
        tick: i64,
        ctx: GameplayContext,
        enemy: Option<Opponent>,
    ) -> ControllerOutput {
        if self.preview.is_some() {
            return self.sample(settings, tick, ctx, enemy);
        }
        let observed = enemy
            .filter(finite_opponent)
            .map(|e| (e, visits.observe(e, ctx.position)));
        if settings.enabled && ctx.held && ctx.gameplay {
            if let Some((e, visit)) = observed {
                if visit.new_visit && (self.target == Some(e.id) || self.file_timeline.is_none())
                    || self.complete && visit.allowed
                {
                    *self = Self::default();
                }
                let continuing = self.file_timeline.is_some() && self.target == Some(e.id);
                if !continuing && !visit.allowed {
                    self.visit_consumed = true;
                    self.last_output = ControllerOutput::default();
                    return self.last_output;
                }
            }
        }
        let out = self.sample(settings, tick, ctx, enemy);
        if let Some((e, _)) = observed {
            if self.target == Some(e.id) && self.slot.is_some() {
                visits.claim(e.id);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledge_trap::{
        tests::{context, enemy},
        CLIFF_ATTACK, CLIFF_CLIMB, CLIFF_WAIT,
    };
    use crate::{
        fighter_profiles::presets::PresetSlot, native_input::MAPPER_A, sequence_config::Catalog,
    };
    /// Returns the current settings view.
    ///
    ///
///  Gets the settings being edited.
    fn settings() -> AssistSettings {
        let c = Catalog::parse("[kirby.neutral.test]\nstart_delay=0\nactions=a:1").unwrap();
        let mut s = AssistSettings {
            enabled: true,
            ..Default::default()
        };
        s.sequence_overrides[PresetSlot::Neutral as usize] = Some(c.entries()[0].sequence);
        s
    }
    /// Returns the `ctx` result.
    ///
    ///
///  Gets the ctx result.
    fn ctx(frame: f32) -> GameplayContext {
        GameplayContext {
            motion_kind: 123,
            motion_frame: frame,
            ..context()
        }
    }
    /// Regression test for `completed_visit_stays_consumed_after_release_reset_and_sticky_previous_status`.
    ///
    ///
///  Checks that completed visit stays consumed after release reset and sticky previous status.
    #[test]
    fn completed_visit_stays_consumed_after_release_reset_and_sticky_previous_status() {
        let mut v = LedgeVisits::default();
        v.set_owner(1, 6);
        let mut trap = LedgeTrap::default();
        let getup = Opponent {
            previous_status: CLIFF_WAIT,
            ..enemy(CLIFF_CLIMB, true)
        };
        assert_eq!(
            trap.sample_with_visits(&mut v, settings(), 0, ctx(0.0), Some(getup))
                .requested,
            MAPPER_A
        );
        trap.sample_with_visits(&mut v, settings(), 1, ctx(1.0), Some(getup));
        assert!(trap.complete);
        let standing = Opponent {
            previous_status: CLIFF_CLIMB,
            ..enemy(0, true)
        };
        trap.sample_with_visits(
            &mut v,
            settings(),
            2,
            GameplayContext {
                held: false,
                ..ctx(2.0)
            },
            Some(standing),
        );
        trap = LedgeTrap::default();
        for frame in 3..100 {
            assert_eq!(
                trap.sample_with_visits(
                    &mut v,
                    settings(),
                    frame,
                    ctx(frame as f32),
                    Some(standing)
                )
                .requested,
                0
            );
            trap = LedgeTrap::default(); // Also model another assist taking ownership.
        }
    }
    /// Regression test for `interrupting_an_active_response_does_not_restart_it_and_new_grab_rearms`.
    ///
    ///
///  Checks that interrupting an active response does not restart it and new grab rearms.
    #[test]
    fn interrupting_an_active_response_does_not_restart_it_and_new_grab_rearms() {
        let mut v = LedgeVisits::default();
        let mut t = LedgeTrap::default();
        let getup = Opponent {
            previous_status: CLIFF_WAIT,
            ..enemy(CLIFF_CLIMB, true)
        };
        assert_eq!(
            t.sample_with_visits(&mut v, settings(), 0, ctx(0.0), Some(getup))
                .requested,
            MAPPER_A
        );
        t.sample_with_visits(
            &mut v,
            settings(),
            1,
            GameplayContext {
                held: false,
                ..ctx(1.0)
            },
            Some(getup),
        );
        assert_eq!(
            t.sample_with_visits(&mut v, settings(), 2, ctx(2.0), Some(getup))
                .requested,
            0
        );
        assert!(!t.reserves_response(Some(getup)));
        assert_eq!(t.action_name(), "await_new_ledge_visit");
        let hang = Opponent {
            cliff_position: Some([-10.0, 0.0]),
            ..enemy(CLIFF_WAIT, false)
        };
        t.sample_with_visits(&mut v, settings(), 3, ctx(3.0), Some(hang));
        assert_eq!(
            t.sample_with_visits(&mut v, settings(), 4, ctx(4.0), Some(getup))
                .requested,
            MAPPER_A
        );
    }
    /// Regression test for `standing_previous_getup_alone_cannot_create_a_visit`.
    ///
    ///
///  Checks that standing previous getup alone cannot create a visit.
    #[test]
    fn standing_previous_getup_alone_cannot_create_a_visit() {
        let mut v = LedgeVisits::default();
        let mut t = LedgeTrap::default();
        let standing = Opponent {
            previous_status: CLIFF_CLIMB,
            ..enemy(0, true)
        };
        assert_eq!(
            t.sample_with_visits(&mut v, settings(), 0, ctx(0.0), Some(standing))
                .requested,
            0
        );
        assert!(t.file_timeline.is_none());
    }
    /// Regression test for `ledge_attack_seen_without_its_hang_is_still_a_response_visit`.
    ///
    ///
///  Checks that ledge attack seen without its hang is still a response visit.
    #[test]
    fn ledge_attack_seen_without_its_hang_is_still_a_response_visit() {
        let c = Catalog::parse("[kirby.attack.test]\nstart_delay=0\nactions=y:1").unwrap();
        let mut s = settings();
        s.sequence_overrides[PresetSlot::Attack as usize] = Some(c.entries()[0].sequence);
        let attack = Opponent {
            previous_status: CLIFF_WAIT,
            ..enemy(CLIFF_ATTACK, true)
        };
        let mut v = LedgeVisits::default();
        let mut t = LedgeTrap::default();
        assert_eq!(
            t.sample_with_visits(&mut v, s, 0, ctx(0.0), Some(attack))
                .requested,
            crate::native_input::MAPPER_Y
        );
        // The same attack continuing, or following its own hang, is one visit.
        let epoch = v.visit_epoch();
        assert!(!v.observe(attack, [0.0; 2]).new_visit);
        assert_eq!(v.visit_epoch(), epoch);
        let mut seen_hang = LedgeVisits::default();
        seen_hang.observe(enemy(CLIFF_WAIT, false), [0.0; 2]);
        let epoch = seen_hang.visit_epoch();
        assert!(!seen_hang.observe(attack, [0.0; 2]).new_visit);
        assert_eq!(seen_hang.visit_epoch(), epoch);
    }
    /// Regression test for `walking_anchor_only_comes_from_a_real_observed_hang_and_survives_input_release`.
    ///
    ///
///  Checks that walking anchor only comes from a real observed hang and survives input release.
    #[test]
    fn walking_anchor_only_comes_from_a_real_observed_hang_and_survives_input_release() {
        let mut v = LedgeVisits::default();
        v.set_owner(1, 6);
        let stand = Opponent {
            cliff_position: Some([99.0, 20.0]),
            ..enemy(0, true)
        };
        v.observe(stand, [0.0; 2]);
        assert!(v.walking_ledge().is_none());
        let hang = Opponent {
            cliff_position: Some([0.0; 2]),
            ..enemy(CLIFF_WAIT, false)
        };
        v.observe(hang, [10.0, 0.0]);
        assert_eq!(
            v.walking_ledge(),
            Some(LedgeSpot {
                edge: [0.0; 2],
                inward: 1.0
            })
        );
        v.observe(hang, [-2.0, 0.0]); // Never flip the ledge's inward direction at the boundary.
        assert_eq!(v.walking_ledge().unwrap().inward, 1.0);
        v.observe(enemy(0, true), [10.0, 0.0]);
        assert_eq!(v.walking_ledge().unwrap().edge, [0.0; 2]);
        v.set_owner(1, 6);
        assert!(v.walking_ledge().is_some());
        v.set_owner(4, 5);
        assert!(v.walking_ledge().is_none());
    }
    /// Regression test for `completed_engine_rearms_for_next_observed_hang_without_releasing_l`.
    ///
    ///
///  Checks that completed engine rearms for next observed hang without releasing l.
    #[test]
    fn completed_engine_rearms_for_next_observed_hang_without_releasing_l() {
        let mut v = LedgeVisits::default();
        let mut t = LedgeTrap::default();
        for base in [0, 10] {
            let hang = enemy(CLIFF_WAIT, false);
            t.sample_with_visits(&mut v, settings(), base, ctx(base as f32), Some(hang));
            let getup = Opponent {
                previous_status: CLIFF_WAIT,
                ..enemy(CLIFF_CLIMB, true)
            };
            assert_eq!(
                t.sample_with_visits(
                    &mut v,
                    settings(),
                    base + 1,
                    ctx((base + 1) as f32),
                    Some(getup)
                )
                .requested,
                MAPPER_A
            );
            t.sample_with_visits(
                &mut v,
                settings(),
                base + 2,
                ctx((base + 2) as f32),
                Some(getup),
            );
            assert!(t.complete);
        }
    }
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use crate::ledge_trap::{
        tests::{context, enemy},
        CLIFF_CLIMB, CLIFF_WAIT,
    };
    use crate::{
        fighter_profiles::presets::PresetSlot, native_input::MAPPER_A, sequence_config::Catalog,
    };
    /// Regression test for `editor_previews_repeat_without_consuming_the_real_visit`.
    ///
    ///
///  Checks that editor previews repeat without consuming the real visit.
    #[test]
    fn editor_previews_repeat_without_consuming_the_real_visit() {
        let c = Catalog::parse("[kirby.neutral.test]\nstart_delay=0\nactions=a:1").unwrap();
        let mut s = AssistSettings {
            enabled: true,
            ..Default::default()
        };
        s.sequence_overrides[0] = Some(c.entries()[0].sequence);
        let mut v = LedgeVisits::default();
        let e = Opponent {
            cliff_position: Some([-10.0, 0.0]),
            ..enemy(CLIFF_WAIT, false)
        };
        v.observe(e, [0.0; 2]);
        for tick in [0, 1] {
            let mut t = LedgeTrap::default();
            t.request_preview(PresetSlot::Neutral);
            let ctx = GameplayContext {
                motion_kind: 123,
                motion_frame: tick as f32,
                ..context()
            };
            assert_eq!(
                t.sample_with_visits(&mut v, s, tick, ctx, Some(e))
                    .requested,
                MAPPER_A
            );
        }
        let mut t = LedgeTrap::default();
        let e = Opponent {
            previous_status: CLIFF_WAIT,
            ..enemy(CLIFF_CLIMB, true)
        };
        assert_eq!(
            t.sample_with_visits(
                &mut v,
                s,
                2,
                GameplayContext {
                    motion_kind: 123,
                    motion_frame: 2.0,
                    ..context()
                },
                Some(e)
            )
            .requested,
            MAPPER_A
        );
    }
    /// Regression test for `opposite_ledge_replaces_inward_side_but_same_edge_motion_and_own_crossing_do_not`.
    ///
    ///
///  Checks that opposite ledge replaces inward side but same edge motion and own crossing do not.
    #[test]
    fn opposite_ledge_replaces_inward_side_but_same_edge_motion_and_own_crossing_do_not() {
        let mut v = LedgeVisits::new();
        v.set_owner(1, 15);
        for (edge, inward) in [(80., -1.), (-80., 1.)] {
            let e = Opponent {
                cliff_position: Some([edge, 0.]),
                ..enemy(CLIFF_WAIT, false)
            };
            v.observe(e, [0., 0.]);
            let spot = v.walking_ledge().unwrap();
            assert_eq!(spot.inward, inward);
            for clearance in [-2., 0., 4., 8., 20.] {
                let position = [edge + inward * clearance, 0.];
                let away = [inward * 0.5, 0.];
                assert_eq!(
                    crate::manual_controls::stop_walk_at_ledge(away, position, [0.; 2], spot, 8.),
                    away
                );
                if clearance <= 8. {
                    assert_eq!(
                        crate::manual_controls::stop_walk_at_ledge(
                            [-away[0], 0.],
                            position,
                            [0.; 2],
                            spot,
                            8.
                        ),
                        [0.; 2]
                    );
                }
            }
            let moved = Opponent {
                id: 3,
                cliff_position: Some([edge + 0.5, 0.]),
                ..e
            };
            v.observe(moved, [edge - inward * 2., 0.]);
            assert_eq!(
                v.walking_ledge().unwrap().inward,
                inward,
                "same physical edge keeps direction even across opponents"
            );
        }
    }
}
