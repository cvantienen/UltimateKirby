//! Hold-Down reactive ledge assistance. Native observations, not assumed
//! controller-poll frame counts, gate anti-air and shield/grab transitions.
//! Geometry gates are conservative tunable limits, not an exact hitbox model.
//!
//! Read sample() first: it handles release/reset and duplicate input updates.
//! evaluate() tracks the target, observes accepted actions, and dispatches to
//! one of four response methods. The child tests module runs only on the host.
use crate::native_input::{
    MAPPER_A, MAPPER_B, MAPPER_L, MAPPER_R, MAPPER_X, MAPPER_Y, MAPPER_ZL, MAPPER_ZR,
};

pub const DPAD_DOWN: u32 = 1 << 2;
pub const CLIFF_CATCH_MOVE: i32 = 0x75;
pub const CLIFF_WAIT: i32 = 0x77;
pub const CLIFF_ATTACK: i32 = 0x78;
pub const CLIFF_CLIMB: i32 = 0x79;
pub const CLIFF_ESCAPE: i32 = 0x7a;
pub const CLIFF_JUMP1: i32 = 0x7b;
pub const CLIFF_JUMP3: i32 = 0x7d;

// Status IDs are names, not an ordered progression through a ledge visit.
// Idle (0) is numerically below CLIFF_WAIT too! A <= check would mistakenly
// treat a finished normal get-up as hanging and never send its punish.
fn is_hanging(status: i32) -> bool {
    (CLIFF_CATCH_MOVE..=CLIFF_WAIT).contains(&status)
}
// Reserve conflicting physical attacks/jumps/shield while a target is tracked.
// Plus/Minus are never owned. Right-stick directions and stick clicks are
// reserved too, so a physical C-stick attack cannot replace the selected option.
pub const LEDGE_BUTTONS: u32 = MAPPER_A
    | MAPPER_B
    | MAPPER_X
    | MAPPER_Y
    | MAPPER_L
    | MAPPER_R
    | MAPPER_ZL
    | MAPPER_ZR
    | (3 << 16)
    | (0xf << 24);
// Native status IDs used below. Naming them keeps the decision rules readable.
const GUARD_ON: i32 = 0x1b;
const GUARD: i32 = 0x1c;
const GUARD_OFF: i32 = 0x1d;
const GUARD_DAMAGE: i32 = 0x1e;
const UP_TILT: i32 = 0x2b;
const DOWN_TILT: i32 = 0x2c;
const FORWARD_TILT: i32 = 0x2a;
const GRAB_START: i32 = 0x37;
const DASH_GRAB_START: i32 = 0x39;
const TURN_GRAB_START: i32 = 0x3b;
const GRAB_PULL: i32 = 0x38;
const DASH_GRAB_PULL: i32 = 0x3a;
const GRAB_HOLD: i32 = 0x3c;
const GRAB_PUMMEL: i32 = 0x3d;

// Follow timeout advances on non-stopped evaluations. Identical observations
// reuse output without spending another attempt. Retries need a release sample,
// not an arbitrary extra timer; animation acceptance and budgets bound them.
const FOLLOW_UPDATES: u16 = 48;
const MAX_ATTEMPTS: u8 = 6;
// Retain identity/anchor through a short missed observation, but emit no
// movement or attacks until fresh native facts return. Counts mapper updates.
const MISSING_UPDATES: u8 = 2;

/// Grounded normal-getup punish. Forward tilt is angled down for low targets;
/// choosing it does not change the up-tilt jump response or roll/shield grabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GetupAttack {
    #[default]
    DownTilt,
    ForwardTilt,
}

/// Read-only animation timing; remaining values are converted to game frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTiming {
    // kind is an animation Hash40 identifier, not a status ID or frame count.
    // frame can be fractional; rate converts animation progress to game time.
    // remaining=None means unknown, whereas Some(0.0) means already at the end.
    pub kind: u64,
    pub frame: f32,
    pub rate: f32,
    pub remaining: Option<f32>,
}

/// Animation rates can differ by character/motion. Invalid metadata stays
/// unknown instead of becoming a guessed invincibility or action deadline.
pub fn frames_until(frame: f32, deadline: f32, rate: f32) -> Option<f32> {
    if !frame.is_finite()
        || frame < 0.0
        || !deadline.is_finite()
        || !rate.is_finite()
        || rate <= 0.0
    {
        return None;
    }
    // Example: animation frame 10 -> deadline 16 at rate 2 gives 3 game
    // frames. Reject past/invalid deadlines instead of treating them as "now".
    let remaining = (deadline - frame) / rate;
    (deadline > 0.0 && (0.0..=120.0).contains(&remaining)).then_some(remaining)
}

/// Validated config values. Defaults and response timing are shared by both builds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeSettings {
    pub enabled: bool,
    pub getup_attack: GetupAttack,
    // Ranges/heights/stance distances are WORLD units, not stick percentages.
    // Increasing reach only makes the policy willing to attack from farther
    // away; it does not enlarge Kirby's real hitbox and can create more whiffs.
    pub anti_air_range: f32,
    pub anti_air_height: f32,
    /// Broad target acquisition limits in world units.
    pub horizontal_range: f32,
    pub vertical_range: f32,
    /// Separate attack limits prevent treating acquisition range as hitbox reach.
    pub tilt_range: f32,
    pub grab_range: f32,
    /// Desired pursuit separations, independent of the attack eligibility gates.
    /// These must be smaller than the matching range in the text config.
    pub tilt_approach_distance: f32,
    pub grab_approach_distance: f32,
    // Follow limits keep the acquired opponent through its get-up travel;
    // acquisition limits above decide whether to acquire it in the first place.
    pub follow_range: f32,
    pub follow_vertical_range: f32,
    pub auto_position: bool,
    /// Heuristic stage-side stance offset, not a universal optimal location.
    pub stand_distance: f32,
    // Unlike distances, move_strength is normalized stick magnitude (0..1).
    // grab_buffer_frames permits one early input near native shieldstun end;
    // it does not shorten shieldstun or the native grab animation.
    pub move_strength: f32,
    pub grab_buffer_frames: u16,
    /// After a confirmed capture, use the remembered ledge side to throw outward.
    /// Only applies in native catch-wait; it never walks Kirby toward the edge.
    pub throw_outward: bool,
    /// Native mask of the button mapped to Grab; R for this user's controls.
    pub grab_button: u32,
}
impl Default for LedgeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            getup_attack: GetupAttack::DownTilt,
            anti_air_range: 8.0,
            anti_air_height: 18.0,
            horizontal_range: 12.0,
            vertical_range: 10.0,
            tilt_range: 10.0,
            grab_range: 8.0,
            tilt_approach_distance: 6.0,
            grab_approach_distance: 5.0,
            follow_range: 40.0,
            follow_vertical_range: 48.0,
            auto_position: true,
            stand_distance: 9.0,
            move_strength: 1.0,
            grab_buffer_frames: 3,
            throw_outward: true,
            grab_button: MAPPER_R,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
// All fields are copied observations. Missing optional metadata is represented
// explicitly so tests and native callers cannot silently invent a deadline.
pub struct Opponent {
    pub id: u32,
    pub status: i32,
    pub previous_status: i32,
    pub situation: i32,
    pub position: [f32; 2],
    /// Aggregate native hit status, not an individual hurtbox model.
    pub vulnerable: bool,
    pub velocity: [f32; 2],
    pub cliff_position: Option<[f32; 2]>,
    /// Sanitized native motion metadata; None means prediction is unavailable.
    pub vulnerable_in: Option<f32>,
    pub motion: Option<MotionTiming>,
    /// Present for get-up attacks; None preserves conservative block-only behavior.
    pub attack_active: Option<bool>,
}
/// Acquisition can use the actual cliff anchor instead of a hanging fighter's
/// much lower root. Tracked opponents get a separate, bounded follow-up box.
pub fn in_tracking_range(
    settings: LedgeSettings,
    position: [f32; 2],
    enemy: Opponent,
    tracked: bool,
) -> bool {
    let reference = if !tracked && is_hanging(enemy.status) {
        enemy.cliff_position.unwrap_or(enemy.position)
    } else {
        enemy.position
    };
    let dx = reference[0] - position[0];
    let dy = reference[1] - position[1];
    let limits = if tracked {
        [settings.follow_range, settings.follow_vertical_range]
    } else {
        [settings.horizontal_range, settings.vertical_range]
    };
    dx.is_finite() && dy.is_finite() && dx.abs() <= limits[0] && dy.abs() <= limits[1]
}

/// Kirby's current native state, copied by the Switch adapter before evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeContext {
    pub held: bool,
    pub gameplay: bool,
    /// Kirby can act from normal grounded movement right now.
    pub can_act: bool,
    /// The assist may own inputs, including during its guard/grab/tilt animation.
    pub can_control: bool,
    /// Native hitstop: preserve an existing shield, but start no new actions.
    pub stopped: bool,
    pub status: i32,
    pub situation: i32,
    /// Kirby's native motion clock distinguishes real progress when a target
    /// holds one animation pose; it is also part of duplicate-observation reuse.
    pub motion_frame: f32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    // Facing uses its sign (+right/-left); position/velocity remain absolute
    // world axes. Multiplying dx by facing asks if the target is in front.
    pub facing: f32,
    pub previous_status: i32,
    pub can_grab: bool,
    pub shield_stun_frames: Option<i32>,
    pub shield_attacker: Option<u32>,
    /// Native hit infliction during Kirby's current tilt, not just animation start.
    pub hit_confirmed: bool,
    /// Ground support along a short swept lookahead, left then right.
    pub safe_ground: [bool; 2],
}
/// Proposed inputs, applied by ManualOverlay rather than written into game state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LedgeOutput {
    pub consume_down: bool,
    pub stick: Option<[f32; 2]>,
    /// Buttons to hold for this update (a mask of native controller bits).
    pub requested: u32,
    /// Buttons to replace, including buttons that should currently be released.
    pub owned: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// Plan = which response we are following; Pending below = which action input
// we attempted. Keeping them separate lets us retry a rejected input while
// preserving the opponent option that caused it. These are state, not commands.
enum OptionPlan {
    #[default]
    Watch,
    UpTilt,
    Roll,
    Shield {
        blocked: bool,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Pending {
    #[default]
    None,
    DownTilt,
    ForwardTilt,
    UpTilt,
    Grab,
}
/// Identical native animation observations are one decision, even if the
/// mapper is polled repeatedly before the game advances that animation.
#[derive(Clone, Copy, PartialEq)]
struct Observation {
    settings: LedgeSettings,
    context: LedgeContext,
    opponent: Opponent,
}

/// Memory for one held-Down sequence. No native game pointers are stored here.
#[derive(Default)]
pub struct LedgeTrap {
    // Visit identity and stance: anchor is the stage ledge, inward points from
    // it onto the stage. finished ends attempts after hit/capture evidence.
    target: Option<u32>,
    anchor: Option<[f32; 2]>,
    inward: f32,
    finished: bool,
    // A directional sample precedes A so the game sees a tilt setup. Preparation
    // is cleared by movement or a plan change; it isn't proof an attack started.
    grounded_tilt_prepared: bool,
    climb_seen: bool,
    plan: OptionPlan,
    pending: Pending,
    last_status: i32,
    outside_updates: u16,
    missing_updates: u8,
    stance_moving: bool,
    // Requested pulses and observed animations have separate budgets: a request
    // can be ignored, and an accepted animation can still whiff. Both are bounded.
    attempts: u8,
    grab_buffer_sent: bool,
    grab_ready_seen: bool,
    grab_in_progress: bool,
    grab_animations: u8,
    tilt_in_progress: bool,
    tilt_animations: u8,
    // Copy snapshots only. Same-animation root travel supplements kinetic speed
    // because ledge animations can move a fighter without matching kinetic speed.
    previous_enemy: Option<Opponent>,
    observed_velocity: [f32; 2],
    /// Once shield is dropped to pursue pushback, never raise it again while
    /// waiting for guard-off/repositioning. Reset on a new option/ledge visit.
    shield_pursuing: bool,
    // Empty slots during startup are not an ended attack. Require a witnessed
    // active -> empty transition before offering a whiff punish without a block.
    getup_hitboxes_seen: bool,
    getup_hitboxes_ended: bool,
    last_tick: Option<i64>,
    // Previous proposal is needed for release gaps and stick preparation. The
    // cached Observation also includes both fighters' clocks to distinguish a
    // repeated poll from meaningful progress before spending another attempt.
    last_output: LedgeOutput,
    last_observation: Option<Observation>,
}
impl LedgeTrap {
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Read-only diagnostic snapshot of the velocity used for target prediction.
    pub fn observed_target_velocity(&self) -> [f32; 2] {
        self.observed_velocity
    }
    pub fn action_name(&self) -> &'static str {
        if self.finished {
            if let Some([x, _]) = self.last_output.stick {
                if self.pending == Pending::Grab && x.abs() == 1.0 {
                    return "throw_outward";
                }
                if x != 0.0 {
                    return "return_to_ledge";
                }
            }
            return "finished";
        }
        match self.plan {
            OptionPlan::Watch => "watch_then_grounded_tilt",
            OptionPlan::UpTilt => "grounded_up_tilt",
            OptionPlan::Roll => "wait_roll_then_grab",
            OptionPlan::Shield { blocked: false } => "shield_wait_for_block",
            OptionPlan::Shield { blocked: true } if self.shield_pursuing => {
                "close_distance_then_grab"
            }
            OptionPlan::Shield { blocked: true } => "shield_grab_after_stun",
        }
    }
    // Only reset attempt/preparation history when the plan CHANGES. Recreating
    // it every poll would forget release gaps and shield pursuit progress.
    fn set_plan(&mut self, plan: OptionPlan) {
        if self.plan != plan {
            self.plan = plan;
            self.grounded_tilt_prepared = false;
            self.attempts = 0;
            self.pending = Pending::None;
            self.grab_buffer_sent = false;
            self.grab_ready_seen = false;
            self.shield_pursuing = false;
            self.getup_hitboxes_seen = false;
            self.getup_hitboxes_ended = false;
        }
    }
    /// Fresh pulses have a release gap and bounded retries. A pulse is not an
    /// accepted action: confirmation comes from a later native state snapshot.
    fn try_tilt(&mut self, pending: Pending) -> bool {
        if self.last_output.requested & MAPPER_A != 0
            || self.attempts >= MAX_ATTEMPTS
            || self.tilt_animations >= MAX_ATTEMPTS
        {
            return false;
        }
        self.attempts += 1;
        self.pending = pending;
        true
    }
    /// Grab buffering gets one early pulse. Native readiness opens a fresh
    /// request budget, so early pulses cannot use up all actual punish attempts.
    /// A held previous R must be released; there is no arbitrary 3-poll delay.
    fn try_grab(&mut self, settings: LedgeSettings, buffering: bool) -> bool {
        if self.grab_animations >= MAX_ATTEMPTS {
            return false;
        }
        if !buffering && !self.grab_ready_seen {
            self.grab_ready_seen = true;
            self.attempts = 0;
        }
        if buffering && self.grab_buffer_sent {
            return false;
        }
        if self.last_output.requested & settings.grab_button != 0 || self.attempts >= MAX_ATTEMPTS {
            return false;
        }
        if buffering {
            self.grab_buffer_sent = true;
        }
        self.attempts += 1;
        self.pending = Pending::Grab;
        true
    }
    // Public entry point: pass current facts and receive a value to overlay.
    // `&mut self` allows updating this held-trigger session; contexts/settings
    // are Copy snapshots, so the policy cannot mutate the game's real fighter.
    pub fn sample(
        &mut self,
        settings: LedgeSettings,
        tick: i64,
        context: LedgeContext,
        opponent: Option<Opponent>,
    ) -> LedgeOutput {
        if !settings.enabled || !context.held || !context.gameplay {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last_tick == Some(tick) {
            return self.last_output;
        }
        if self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
        }
        // First deduplicate the caller's tick, then (with valid motion metadata)
        // identical native facts. If motion is unknown we cannot infer native
        // progress here, so fall back to the caller's input-update cadence.
        let observation = opponent
            .filter(|enemy| enemy.motion.is_some())
            .map(|opponent| Observation {
                settings,
                context,
                opponent,
            });
        if observation.is_some() && observation == self.last_observation {
            self.last_tick = Some(tick);
            return self.last_output;
        }
        let out = self.evaluate(settings, context, opponent);
        self.last_tick = Some(tick);
        self.last_output = out;
        self.last_observation = observation;
        out
    }
    fn evaluate(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        opponent: Option<Opponent>,
    ) -> LedgeOutput {
        // Reserve Down while the tool is armed even before a target is found.
        // Actual button/stick takeover is added only after tracking and native
        // eligibility checks below. None stick means leave physical movement.
        let mut out = LedgeOutput {
            consume_down: true,
            ..LedgeOutput::default()
        };
        // Confirm before target lookup: a hit can launch the opponent outside
        // the tracking box on this very update. Losing proximity must not lose
        // the confirmation or prevent Kirby returning to his ledge stance.
        self.observe_action(ctx);
        let Some(mut enemy) = opponent else {
            if self.finished {
                self.complete_punish(settings, ctx, &mut out);
                return out;
            }
            self.missing_updates = self.missing_updates.saturating_add(1);
            if self.target.is_some() && self.missing_updates <= MISSING_UPDATES {
                return out;
            }
            self.target = None;
            self.plan = OptionPlan::Watch;
            return out;
        };
        self.missing_updates = 0;
        // 1. Acquire/follow one opponent; reset the visit on a new target/regrab.
        let dx = enemy.position[0] - ctx.position[0];
        let dy = enemy.position[1] - ctx.position[1];
        let nearby =
            in_tracking_range(settings, ctx.position, enemy, self.target == Some(enemy.id))
                && dx.is_finite()
                && dy.is_finite()
                && ctx.facing.is_finite();
        let on_ledge = (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&enemy.status);
        let recent_option = (CLIFF_ATTACK..=CLIFF_JUMP3).contains(&enemy.previous_status);
        if nearby
            && (on_ledge || recent_option)
            && (self.target != Some(enemy.id)
                || (is_hanging(enemy.status) && !is_hanging(self.last_status))
                || (on_ledge && !(CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&self.last_status)))
        {
            // Reset the entire visit, preserving no pending attack from a
            // different opponent or an earlier regrab.
            *self = Self {
                target: Some(enemy.id),
                ..Self::default()
            };
        }
        if !ctx.stopped {
            self.outside_updates = if on_ledge {
                0
            } else {
                self.outside_updates.saturating_add(1)
            };
        }
        let tracked = nearby
            && self.target == Some(enemy.id)
            && (on_ledge || self.outside_updates <= FOLLOW_UPDATES);
        self.last_status = enemy.status;
        if !tracked {
            if self.finished {
                self.complete_punish(settings, ctx, &mut out);
                return out;
            }
            self.target = None;
            self.plan = OptionPlan::Watch;
            return out;
        }

        // Refresh the ledge anchor while hanging, then retain it when the
        // opponent moves onstage. Following the opponent root as a stance anchor
        // would move the desired standing spot throughout its get-up animation.
        if is_hanging(enemy.status) && ctx.situation == 0 {
            if let Some(anchor) = enemy
                .cliff_position
                .filter(|p| p.iter().all(|v| v.is_finite()))
            {
                self.anchor = Some(anchor);
                if (ctx.position[0] - anchor[0]).abs() > 0.5 {
                    self.inward = (ctx.position[0] - anchor[0]).signum();
                }
            }
        }
        self.climb_seen |= enemy.status == CLIFF_CLIMB || enemy.previous_status == CLIFF_CLIMB;
        enemy.velocity = self.target_velocity(enemy, ctx.stopped);

        // 2. Confirm previous requests before selecting this update's response.
        self.choose_response(ctx, enemy);
        if self.finished {
            self.complete_punish(settings, ctx, &mut out);
            return out;
        }
        // Hitstop freezes decisions, not the existing shield hold. Returning an
        // unowned neutral output here used to release ZR on block impact. Only
        // preserve shield in native guard states: damage/air actions stay free.
        if ctx.stopped {
            if matches!(self.plan, OptionPlan::Shield { .. })
                && !self.shield_pursuing
                && !self.finished
                && ctx.situation == 0
                && matches!(ctx.status, GUARD_ON | GUARD | GUARD_DAMAGE)
            {
                out.owned = LEDGE_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                out.requested = MAPPER_ZR;
            }
            return out;
        }
        if !ctx.can_control {
            self.grounded_tilt_prepared = false;
            return out;
        }
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        if self.grab_in_progress || self.tilt_in_progress {
            return out;
        }
        // 3. Each response changes only the proposed stick/button output.
        match self.plan {
            OptionPlan::UpTilt => self.respond_to_jump(settings, ctx, enemy, &mut out),
            OptionPlan::Shield { .. } => {
                self.respond_to_getup_attack(settings, ctx, enemy, &mut out)
            }
            OptionPlan::Roll => self.respond_to_roll(settings, ctx, enemy, &mut out),
            OptionPlan::Watch => self.watch_ledge(settings, ctx, enemy, &mut out),
        }
        out
    }

    /// Finish a capture using ordinary directional throw input, then recover
    /// the original stance. A successful punish ends attacks for this visit,
    /// not positioning for the next one. A new ledge visit resets the session.
    fn complete_punish(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        out: &mut LedgeOutput,
    ) {
        if !ctx.can_control || ctx.stopped || ctx.situation != 0 {
            return;
        }
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        if settings.throw_outward
            && self.pending == Pending::Grab
            && ctx.status == GRAB_HOLD
            && self.anchor.is_some()
            && self.inward != 0.0
        {
            // Left/right are world directions. Depending on Kirby's facing,
            // the game chooses forward or back throw. No A means no pummel;
            // no R means no repeated grab while already holding an opponent.
            out.stick = Some([-self.inward, 0.0]);
        } else if ctx.can_act {
            self.position_at_ledge(settings, ctx, out);
        }
    }
    /// Observe acceptance, captures, and whiff recovery before choosing inputs.
    fn observe_action(&mut self, ctx: LedgeContext) {
        // Three separate milestones: requested button -> observed action
        // animation -> actual hit/capture. Only the last marks this visit done.
        // Recovery after a whiff reopens attempts within the animation limit.
        let status = ctx.status;
        let tilting = match self.pending {
            Pending::DownTilt => status == DOWN_TILT,
            Pending::ForwardTilt => status == FORWARD_TILT,
            Pending::UpTilt => status == UP_TILT,
            _ => false,
        };
        // A requested input is only an attempt. Native status confirms which
        // animation started; a grab additionally needs a captured opponent.
        self.finished |= match self.pending {
            Pending::DownTilt | Pending::ForwardTilt | Pending::UpTilt => {
                tilting && ctx.hit_confirmed
            }
            Pending::Grab => matches!(status, GRAB_PULL | DASH_GRAB_PULL | GRAB_HOLD | GRAB_PUMMEL),
            Pending::None => false,
        };
        if tilting && !self.tilt_in_progress {
            self.tilt_animations += 1;
        }
        if self.tilt_in_progress && !tilting && !self.finished {
            self.attempts = 0;
            self.pending = Pending::None;
        }
        self.tilt_in_progress = tilting;
        let grabbing = self.pending == Pending::Grab
            && matches!(status, GRAB_START | DASH_GRAB_START | TURN_GRAB_START);
        if grabbing && !self.grab_in_progress {
            self.grab_animations += 1;
        }
        if self.grab_in_progress && !grabbing && !self.finished {
            self.attempts = 0;
            self.pending = Pending::None;
            if matches!(self.plan, OptionPlan::Shield { .. }) {
                // A recovered whiff is actionable movement again: retry R
                // directly instead of paying for another shield startup.
                self.shield_pursuing = true;
            }
        }
        self.grab_in_progress = grabbing;
    }

    /// An observed ledge option selects a response; shield needs block evidence.
    fn choose_response(&mut self, ctx: LedgeContext, enemy: Opponent) {
        // Recover a transition missed between input samples for ALL options,
        // not only jump. Current ledge state always wins over older history.
        let option = if (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&enemy.status) {
            enemy.status
        } else {
            enemy.previous_status
        };
        if !self.finished {
            if option == CLIFF_ATTACK {
                if !matches!(self.plan, OptionPlan::Shield { .. }) {
                    self.set_plan(OptionPlan::Shield { blocked: false });
                }
            } else if option == CLIFF_ESCAPE {
                self.set_plan(OptionPlan::Roll);
            } else if option == CLIFF_CLIMB {
                self.set_plan(OptionPlan::Watch);
            } else if (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&enemy.status)
                || enemy.situation == 2 && (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&option)
            {
                // Remember the option even if Kirby is still recovering. The
                // native previous status also catches a short missed transition.
                self.set_plan(OptionPlan::UpTilt);
            }
        }
        if matches!(self.plan, OptionPlan::Shield { .. })
            && !ctx.stopped
            && enemy.status == CLIFF_ATTACK
        {
            match enemy.attack_active {
                Some(true) => {
                    self.getup_hitboxes_seen = true;
                    self.getup_hitboxes_ended = false;
                }
                Some(false) if self.getup_hitboxes_seen => self.getup_hitboxes_ended = true,
                _ => {}
            }
        }
        let blocked_now = ctx.status == GUARD_DAMAGE
            && enemy.status == CLIFF_ATTACK
            && ctx.shield_attacker.is_none_or(|id| id == enemy.id);
        let missed_stun_transition = ctx.previous_status == GUARD_DAMAGE
            && matches!(ctx.status, GUARD_ON | GUARD)
            && self.last_output.requested & MAPPER_ZR != 0;
        if matches!(self.plan, OptionPlan::Shield { .. }) && (blocked_now || missed_stun_transition)
        {
            self.plan = OptionPlan::Shield { blocked: true };
        }
    }

    /// Native kinetic speed can omit animation-driven ledge travel. Estimate
    /// world-space speed from root displacement only when the SAME animation
    /// clock advances. Repeated mapper polls reuse the last estimate.
    fn target_velocity(&mut self, enemy: Opponent, stopped: bool) -> [f32; 2] {
        let mut velocity = enemy.velocity.map(finite);
        if !stopped {
            if let (Some(previous), Some(current)) = (self.previous_enemy, enemy.motion) {
                if let Some(old) = previous.motion {
                    if previous.id == enemy.id
                        && previous.status == enemy.status
                        && old.kind == current.kind
                    {
                        let elapsed = (current.frame - old.frame) / current.rate;
                        if current.frame == old.frame {
                            return self.observed_velocity;
                        }
                        if elapsed.is_finite() && (0.25..=8.0).contains(&elapsed) {
                            let observed = [
                                (enemy.position[0] - previous.position[0]) / elapsed,
                                (enemy.position[1] - previous.position[1]) / elapsed,
                            ];
                            if observed
                                .iter()
                                .all(|speed| speed.is_finite() && speed.abs() <= 30.0)
                            {
                                velocity = observed;
                            }
                        }
                    }
                }
            }
            self.previous_enemy = Some(enemy);
            self.observed_velocity = velocity;
        }
        velocity
    }

    /// Prepare up tilt and intersect its active window with the predicted
    /// first hittable position, rather than attacking the current root alone.
    fn respond_to_jump(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        if ctx.situation != 0 || !ctx.can_act {
            return;
        }
        let window = AttackWindow {
            first: 4,
            last: 10,
            range: settings.anti_air_range,
            height: [-6.0, settings.anti_air_height],
            front_only: false,
        };
        // Moderate Up chooses up tilt with A rather than an up-smash flick.
        // Tap Jump must be off in the player's controls. This stays grounded;
        // no jump/forward-air sequence is generated by this response.
        let prepared =
            self.last_output.stick == Some([0.0, 0.6]) && self.last_output.owned & MAPPER_A != 0;
        let predicted = predicted_offset(ctx, enemy, 4.0);
        if !window.may_connect(ctx, enemy) && predicted[0].abs() > settings.anti_air_range {
            self.approach(
                settings,
                ctx,
                predicted[0],
                settings.anti_air_range - 1.0,
                out,
            );
            return;
        }
        // Holding Up while a low target rises avoids another preparation poll.
        out.stick = Some([0.0, 0.6]);
        if prepared && window.may_connect(ctx, enemy) && self.try_tilt(Pending::UpTilt) {
            out.requested = MAPPER_A;
        }
    }

    /// Hold shield for a confirmed block, or finish committed pushback pursuit.
    fn respond_to_getup_attack(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        let dx = enemy.position[0] - ctx.position[0];
        let dy = enemy.position[1] - ctx.position[1];
        let blocked = matches!(self.plan, OptionPlan::Shield { blocked: true });
        let may_punish = blocked || self.getup_hitboxes_ended;
        let attack_ended = enemy.status != CLIFF_ATTACK || self.getup_hitboxes_ended;
        if ctx.situation != 0 {
            return;
        }
        let in_reach =
            dx.abs() <= settings.grab_range && dy.abs() <= 5.0 && dx * ctx.facing >= -0.5;
        // The opponent may be protected right now but hittable when Grab
        // becomes active. Conversely, current proximity alone is not enough
        // if their observed motion carries them out of the grab window.
        let startup = grab_startup(ctx)
            + if ctx.status == GUARD_DAMAGE {
                ctx.shield_stun_frames.unwrap_or(0).clamp(0, 6) as u8
            } else {
                0
            };
        let window = AttackWindow {
            first: startup,
            last: startup + 1,
            range: settings.grab_range,
            height: [-5.0, 5.0],
            front_only: true,
        };
        // Latch the departure from shield. A transient in-range sample
        // during GUARD_OFF must not re-raise ZR and restart guarding.
        if may_punish
            && (ctx.status == GUARD_OFF
                || settings.auto_position && attack_ended && enemy.situation == 0 && !in_reach)
        {
            self.shield_pursuing = true;
        }
        if self.shield_pursuing {
            if ctx.can_act && enemy.situation == 0 && attack_ended {
                if in_reach && window.may_connect(ctx, enemy) {
                    if self.try_grab(settings, false) {
                        out.requested = settings.grab_button;
                    }
                } else if !in_reach {
                    if settings.auto_position && dx * ctx.facing < -0.5 {
                        self.ground_move(settings, ctx, dx, out);
                    } else {
                        self.approach(
                            settings,
                            ctx,
                            dx,
                            settings
                                .grab_approach_distance
                                .min((settings.grab_range - 0.5).max(0.0)),
                            out,
                        );
                    }
                }
            }
            return;
        }
        // Keep ZR down through shieldstun, its buffer, and all retry
        // release gaps. Only R pulses; an in-range punish never needs
        // a shield-drop/shield-raise cycle.
        out.requested = MAPPER_ZR;
        // For an in-range shield grab, keep ZR held and pulse the configured
        // Grab button; dropping shield first adds native guard-off delay. Only
        // pursuit after pushback deliberately commits to dropping shield.
        let buffer_ready = ctx.status == GUARD_DAMAGE
            && ctx.shield_stun_frames.is_some_and(|frames| {
                frames >= 0 && frames <= i32::from(settings.grab_buffer_frames)
            });
        let legal_guard =
            matches!(ctx.status, GUARD_ON | GUARD) && (ctx.can_grab || ctx.status == GUARD);
        if may_punish
            && enemy.attack_active != Some(true)
            && in_reach
            && window.may_connect(ctx, enemy)
            && enemy.situation == 0
            && (buffer_ready || legal_guard)
            && self.try_grab(settings, buffer_ready)
        {
            out.requested |= settings.grab_button;
        }
    }

    /// Follow roll travel and time Grab's active frames against vulnerability.
    fn respond_to_roll(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        // Pursuit and grab eligibility must aim at the SAME startup horizon.
        // Different horizons made Kirby chase one point while rejecting grabs
        // against another. predicted_offset still caps travel at animation end.
        let startup = grab_startup(ctx);
        let projected = predicted_offset(ctx, enemy, f32::from(startup));
        if settings.auto_position && ctx.can_act && enemy.situation == 0 {
            self.approach(
                settings,
                ctx,
                projected[0],
                settings
                    .grab_approach_distance
                    .min((settings.grab_range - 0.5).max(0.0)),
                out,
            );
        }
        if enemy.situation != 0 {
            return;
        }
        // Dash/run grabs start later than standing grabs. Prebuffer only when
        // native invincibility timing and predicted reach support that startup.
        // Startup estimates differ by Kirby's current movement. This checks
        // where the target may be when Grab becomes active, not just where its
        // invincible roll root is right now. Unknown expiry waits for vulnerability.
        let window = AttackWindow {
            first: startup,
            last: startup + 1,
            range: settings.grab_range,
            height: [-5.0, 5.0],
            front_only: true,
        };
        let timed_roll =
            enemy.status == CLIFF_ESCAPE && (enemy.vulnerable || enemy.vulnerable_in.is_some());
        if enemy.status == CLIFF_ESCAPE && !timed_roll {
            return;
        }
        if projected[0] * ctx.facing < -0.5 && ctx.can_act {
            self.ground_move(settings, ctx, projected[0], out);
            return;
        }
        if !window.may_connect(ctx, enemy) {
            return;
        }
        out.stick = Some([0.0, 0.0]);
        if ctx.can_act {
            if self.try_grab(settings, false) {
                out.requested = settings.grab_button;
            }
        } else if matches!(ctx.status, GUARD_ON | GUARD) {
            out.requested = MAPPER_ZR;
            if (ctx.can_grab || ctx.status == GUARD) && self.try_grab(settings, false) {
                out.requested |= settings.grab_button;
            }
        }
    }

    /// Position while hanging; prepare the selected grounded normal-getup tilt.
    fn watch_ledge(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        let dx = enemy.position[0] - ctx.position[0];
        if is_hanging(enemy.status) && self.position_at_ledge(settings, ctx, out) {
            self.grounded_tilt_prepared = false;
            return;
        }
        if is_hanging(enemy.status) && ctx.can_act {
            out.stick = Some([0.0, 0.6]);
            self.grounded_tilt_prepared = false;
            return;
        }
        if !ctx.can_act || enemy.situation != 0 {
            self.grounded_tilt_prepared = false;
            return;
        }
        // To change normal-getup move selection, start with this match and
        // verify its real stick recipe, active frames, and confirmation status.
        // Config selects between existing moves; range changes cannot add a move.
        let (stick, pending, first, last) = match settings.getup_attack {
            GetupAttack::DownTilt => ([0.0, -0.6], Pending::DownTilt, 4, 6),
            GetupAttack::ForwardTilt => (
                [ctx.facing.signum() * 0.6, -0.3],
                Pending::ForwardTilt,
                5,
                8,
            ),
        };
        let window = AttackWindow {
            first,
            last,
            range: settings.tilt_range,
            height: [-5.0, 3.0],
            front_only: true,
        };
        let projected = predicted_offset(ctx, enemy, f32::from(first));
        // A normal get-up outside poke range used to wait forever. Close on
        // supported ground, then prepare the chosen direction before A.
        if self.climb_seen && projected[0].abs() > settings.tilt_range {
            self.approach(
                settings,
                ctx,
                projected[0],
                settings
                    .tilt_approach_distance
                    .min((settings.tilt_range - 0.5).max(0.0)),
                out,
            );
            self.grounded_tilt_prepared = false;
            return;
        }
        if dx * ctx.facing < -0.5 {
            self.ground_move(settings, ctx, dx, out);
            self.grounded_tilt_prepared = false;
            return;
        }
        out.stick = Some(stick);
        let prepared = self.grounded_tilt_prepared && self.last_output.stick == Some(stick);
        if prepared && self.climb_seen && window.may_connect(ctx, enemy) && self.try_tilt(pending) {
            out.requested = MAPPER_A;
        }
        self.grounded_tilt_prepared = true;
    }

    /// Shared by hanging coverage and post-punish recovery. Returning true
    /// reserves this sample for movement/turning; never add an attack to it.
    fn position_at_ledge(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        out: &mut LedgeOutput,
    ) -> bool {
        if !settings.auto_position || !ctx.can_act {
            return false;
        }
        let Some(anchor) = self.anchor else {
            return false;
        };
        // The anchor is the stage edge, not the launched opponent's position.
        let actual_error = anchor[0] + self.inward * settings.stand_distance - ctx.position[0];
        let speed = finite(ctx.velocity[0]);
        let error = actual_error - speed * 2.0;
        // Hysteresis: begin outside 1.25 units, settle inside 0.5. Retain the
        // phase between those thresholds instead of restarting movement on
        // every small root-position change. Neutral stick lets momentum decay
        // when the lookahead says that continuing would overshoot.
        if self.inward != 0.0 {
            self.stance_moving |= actual_error.abs() > 1.25 || error.abs() > 1.25;
            if self.stance_moving {
                if actual_error.abs() <= 0.5 && speed.abs() <= 0.2 {
                    self.stance_moving = false;
                } else {
                    out.stick = Some([0.0, 0.0]);
                    if error.abs() >= 0.5 && error * actual_error > 0.0 {
                        self.ground_move(settings, ctx, error, out);
                    }
                    return true;
                }
            }
        }
        let toward = anchor[0] - ctx.position[0];
        if toward * ctx.facing < -0.5 {
            self.ground_move(settings, ctx, toward, out);
            return true;
        }
        false
    }

    fn ground_move(
        &self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        error: f32,
        out: &mut LedgeOutput,
    ) {
        if !error.is_finite() || error.abs() < 0.5 {
            return;
        }
        // Bool -> index: false=0 chooses left ground probe; true=1 chooses right.
        // A blocked probe leaves the default neutral stick. Movement never forces
        // position/velocity; the game still applies Kirby's ordinary acceleration.
        let index = usize::from(error > 0.0);
        if ctx.safe_ground[index] {
            // Reduce deflection continuously near the target; cap turns so
            // facing correction does not become a full-speed run away from it.
            let maximum = if settings.move_strength.is_finite() {
                settings.move_strength.clamp(0.1, 1.0)
            } else {
                0.65
            };
            let strength = (error.abs() * 0.12).clamp(maximum.min(0.15), maximum);
            let strength = if error * ctx.facing < 0.0 {
                strength.min(0.3)
            } else {
                strength
            };
            out.stick = Some([error.signum() * strength, 0.0]);
        }
    }
    fn approach(
        &self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        dx: f32,
        distance: f32,
        out: &mut LedgeOutput,
    ) {
        if settings.auto_position && dx.abs() > distance.max(0.0) {
            self.ground_move(settings, ctx, dx - dx.signum() * distance.max(0.0), out);
        }
    }
}

/// Keep roll and shield-punish predictions consistent. These are startup
/// estimates, not a way to bypass the game's transitions or shield-grab penalty.
fn grab_startup(ctx: LedgeContext) -> u8 {
    match ctx.status {
        0x3 | 0x4 => 9, // Dash/run grab
        0x2 | 0x7 | 0x8 | GUARD_ON | GUARD | GUARD_DAMAGE => 10,
        _ => 6,
    }
}

/// Approximate root-position reach during an attack's active frames. Values
/// are from Kirby's measured frame data: https://ultimateframedata.com/kirby .
/// Unknown invincibility expiry requires actual vulnerability; never guess it.
#[derive(Clone, Copy)]
struct AttackWindow {
    first: u8,
    last: u8,
    range: f32,
    height: [f32; 2],
    front_only: bool,
}
impl AttackWindow {
    fn may_connect(self, ctx: LedgeContext, enemy: Opponent) -> bool {
        // Inclusive range visits each estimated active frame. any() succeeds if
        // at least ONE future sample is in reach AND vulnerable. This is a root
        // position approximation, not exact animated hitbox/hurtbox collision.
        // It does not predict future decisions, gravity, or acceleration.
        (self.first..=self.last).any(|frame| {
            let lead = f32::from(frame);
            let hittable = enemy.vulnerable
                || enemy.vulnerable_in.is_some_and(|remaining| {
                    remaining.is_finite() && remaining >= 0.0 && remaining <= lead
                });
            let [x, y] = predicted_offset(ctx, enemy, lead);
            hittable
                && x.abs() <= self.range
                && (self.height[0]..=self.height[1]).contains(&y)
                && (!self.front_only || x * ctx.facing >= -0.5)
        })
    }
}

fn predicted_offset(ctx: LedgeContext, enemy: Opponent, lead: f32) -> [f32; 2] {
    // Grounded ledge animation travel ends with that animation. Do not extend a
    // roll's last measured velocity indefinitely beyond its native end frame.
    let travel = if matches!(enemy.status, CLIFF_ESCAPE | CLIFF_CLIMB) {
        enemy
            .motion
            .and_then(|motion| motion.remaining)
            .filter(|left| left.is_finite() && *left >= 0.0)
            .map_or(lead, |left| left.min(lead))
    } else {
        lead
    };
    // Relative position + estimated enemy travel - short own-momentum estimate.
    // Values here are world distances, not controller stick output. Limiting
    // ledge travel above avoids projecting a completed roll forever onward.
    [
        enemy.position[0] - ctx.position[0] + finite(enemy.velocity[0]) * travel
            - finite(ctx.velocity[0]) * lead.min(2.0),
        enemy.position[1] - ctx.position[1] + finite(enemy.velocity[1]) * travel
            - finite(ctx.velocity[1]) * lead.min(2.0),
    ]
}

// Native kinetic readings may be unavailable in transitions; never turn NaN
// into a controller input or a successful geometry comparison.
fn finite(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests;
