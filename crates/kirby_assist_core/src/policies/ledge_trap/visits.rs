//! Match-owned ledge memory, independent of temporary input owners and L releases.
//! Only already-observed opponents are inspected; no roster/geometry queries.
use super::{
    finite2, finite_opponent, is_hanging, is_ledge_option, AssistSettings, ControllerOutput,
    GameplayContext, LedgeTrap, Opponent,
};
use crate::ledge_position::LedgeSpot;

/// Runtime Smash data carried by `Visit`.
#[derive(Clone, Copy)]
struct Visit {
    /// battle-object ID.
    id: u32,
    /// Runtime last status value used by Visit..
    last_status: i32,
    /// Runtime armed value used by Visit..
    armed: bool,
    /// Runtime handled value used by Visit..
    handled: bool,
}
/// Runtime Smash data carried by `LedgeVisits`.
#[derive(Default)]
pub struct LedgeVisits {
    /// Runtime owner value used by LedgeVisits..
    owner: Option<(u32, i32)>,
    /// Runtime visits value used by LedgeVisits..
    visits: [Option<Visit>; 8],
    /// Runtime replace value used by LedgeVisits..
    replace: usize,
    /// Runtime ledge value used by LedgeVisits..
    ledge: Option<LedgeSpot>,
    /// Runtime visit epoch value used by LedgeVisits..
    visit_epoch: u64,
}
/// Runtime Smash data carried by `Observation`.
#[derive(Clone, Copy)]
struct Observation {
    /// Runtime new visit value used by Observation..
    new_visit: bool,
    /// Runtime allowed value used by Observation..
    allowed: bool,
}
impl LedgeVisits {
    /// Constructs a new value with the type’s initial state.
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
    pub fn set_owner(&mut self, id: u32, kind: i32) {
        if self.owner != Some((id, kind)) {
            *self = Self::default();
            self.owner = Some((id, kind));
        }
    }
    /// Visits the epoch.
    pub fn visit_epoch(&self) -> u64 {
        self.visit_epoch
    }
    /// Returns an optional `walking_ledge` value.
    pub fn walking_ledge(&self) -> Option<LedgeSpot> {
        self.ledge
    }
    /// Records the time of the latest fighter observation.
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
                // A fresh grab or a landing ends a drop response, including a
                // long shield hold. Other committed responses keep their inputs.
                let drop_ended = self.file_timeline.is_some()
                    && self.slot == Some(crate::fighter_profiles::presets::PresetSlot::Drop)
                    && self.target == Some(e.id)
                    && (visit.new_visit
                        || (e.situation == crate::native_states::situation::GROUND
                            && !is_ledge_option(e.status)));
                if drop_ended
                    || (self.file_timeline.is_none()
                        && (visit.new_visit || self.complete && visit.allowed))
                {
                    *self = Self {
                        diagnostics_enabled: self.diagnostics_enabled,
                        ..Self::default()
                    };
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
#[path = "../../../tests/unit/policy_ledge_trap_visits.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/unit/policy_ledge_trap_previews.rs"]
mod preview_tests;
