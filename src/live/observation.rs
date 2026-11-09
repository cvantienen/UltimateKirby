use super::*;

static OBSERVING_FIGHTER: AtomicBool = AtomicBool::new(false);

// Native queries can re-enter command hooks. Never recursively sample another
// full observation while the mapper's synchronous refresh is in progress.
struct FighterObservationGuard;
impl FighterObservationGuard {
    fn acquire() -> Option<Self> {
        OBSERVING_FIGHTER
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Self)
    }
}
impl Drop for FighterObservationGuard {
    fn drop(&mut self) {
        OBSERVING_FIGHTER.store(false, Ordering::Release);
    }
}
// Diagnostic snapshots are absent from silent builds, including formatting.
#[cfg(feature = "diagnostic-logging")]
#[derive(Clone, Copy, PartialEq, Eq)]
/// Represents a diagnostic snapshot of the ledge state for the fighter.
pub(super) struct LedgeDiagnostic {
    target: Option<u32>,
    enemy_status: i32,
    vulnerable: bool,
    kirby_status: i32,
    can_act: bool,
    can_grab: bool,
    can_dash_attack: bool,
    can_jump: bool,
    can_air_attack: bool,
    hit_confirmed: bool,
    own_motion: u64,
    requested: u32,
    action: &'static str,
}
/// Represents the observation needs for the current state and configuration.
struct ObservationNeeds {
    find_trump_edge: bool,
    find_trap_edge: bool,
    observe_ledge: bool,
    preview_start: bool,
    continue_ledge_response: bool,
    literal_ledge_timeline: bool,
    file_ledge_timeline: bool,
    ground_ledge_timeline: bool,
    observe_trump: bool,
}
/// Captures a snapshot of the observation policy for the current state.
#[derive(Default)]
struct ObservationPolicySnapshot {
    automatic_suspended: bool,
    trip_queued: bool,
    grab_dodge_enabled: bool,
    posture_modes: (bool, bool),
    posture_down: bool,
    roster_missing: bool,
    oos_enabled: bool,
    oos_target: Option<u32>,
    rules_on: bool,
    rule_target: Option<u32>,
    dodge_follow: Option<u32>,
    automatic_sequence_committed: bool,
    up_b_receipt: bool,
    rules_ground: Option<(f32, f32, bool)>,
    simple_starts: [Option<kirby_assist_core::reaction_rules::SimpleStart>; 3],
    dodge_lookahead: f32,
    manual_input_epoch: u64,
    analysis_settings: kirby_assist_core::fighter_tuning::Tuning,
    automatic_limits: [Option<(f32, f32)>; 3],
    dodge_direct_up_b: bool,
    want_up_special: bool,
    ordinary_backup: bool,
    stationary_intent: bool,
    grab_intent: bool,
    self_shielded: bool,
    grab_dash_pending: bool,
}

fn rule_needs_ground_observation(
    ground_watch: Option<(f32, f32, bool)>,
    target_is_locked: bool,
    targets: &[Option<kirby_assist_core::roll_read::Target>; 8],
    position: [f32; 2],
    follow_requested: bool,
    follow: Option<&kirby_assist_core::gameplay::Opponent>,
) -> bool {
    let Some((distance, height, active)) = ground_watch else {
        return false;
    };
    if target_is_locked {
        return active;
    }

    let limits = [distance, height];
    targets.iter().flatten().any(|target| {
        (matches!(
            target.status,
            kirby_assist_core::native_states::ESCAPE
                ..=kirby_assist_core::native_states::ESCAPE_AIR_SLIDE
        ) || target.vulnerable_in == Some(0.))
            && observations::within_observation_range(position, target.position, limits)
    }) || follow_requested
        && follow.is_some_and(|enemy| {
            observations::within_observation_range(position, enemy.position, limits)
        })
}

/// Determines the observation needs based on the current state and configuration.
fn observation_needs(state: &State, config: &Config) -> ObservationNeeds {
    let trump_held = state
        .settings_menu
        .as_ref()
        .is_some_and(|menu| menu.ledge_trump_held());
    let preview_selected = state
        .settings_menu
        .as_ref()
        .is_some_and(|m| m.preview_slot().is_some());
    let file_ledge_timeline = state
        .ledge
        .as_ref()
        .is_some_and(LedgeTrap::file_timeline_active);

    ObservationNeeds {
        find_trump_edge: state
            .trump
            .as_ref()
            .is_none_or(|trump| trump.needs_edge(trump_held))
            && state
                .trump_seek_tick
                .is_none_or(|last| state.raw_tick.wrapping_sub(last) >= 30)
            && trump_held,
        find_trap_edge: config.assist.enabled
            && config.assist.auto_position
            && !config.assist.manual_spacing
            && !config.assist.distance_agnostic
            && state.ledge_active
            && !preview_selected
            && state
                .ledge_seek_tick
                .is_none_or(|last| state.raw_tick.wrapping_sub(last) >= 30)
            && state
                .ledge_position
                .as_ref()
                .is_none_or(|p| p.spot().is_none()),
        observe_ledge: config.assist.enabled && state.ledge_active,
        preview_start: preview_selected && !file_ledge_timeline,
        continue_ledge_response: state
            .ledge
            .as_ref()
            .is_some_and(LedgeTrap::needs_response_observation),
        literal_ledge_timeline: state
            .ledge
            .as_ref()
            .is_some_and(LedgeTrap::file_inputs_only),
        file_ledge_timeline,
        ground_ledge_timeline: state
            .ledge
            .as_ref()
            .is_some_and(LedgeTrap::file_needs_ground),
        observe_trump: trump_held || state.trump.as_ref().is_some_and(LedgeTrump::active),
    }
}

/// Resolves the observation setup for the given fighter and session,
///  returning the configuration, setup sequence, and observation needs if available.
fn resolve_observation_setup(
    index: i32,
    session_ticket: SessionTicket,
    fighter_kind: i32,
    config: Config,
) -> Option<(Config, u64, ObservationNeeds)> {
    let (assist, setup_sequence) = {
        let state = STATE.lock().ok()?;
        if !state.session.accepts(session_ticket) || state.mapping_index != Some(index) {
            return None;
        }
        let assist = state
            .settings_menu
            .as_ref()
            .map(|menu| menu.settings(Some(fighter_kind), &config.assist))
            .unwrap_or_else(|| settings_for_fighter(Some(fighter_kind), config.assist));
        (assist, state.observation_sequence)
    };
    let config = Config { assist, ..config };
    let needs = {
        let state = STATE.lock().ok()?;
        observation_needs(&state, &config)
    };
    Some((config, setup_sequence, needs))
}
/// Determines whether manual automatic input is currently blocked for the given fighter and context.
fn manual_automatic_input_blocked(
    object_id: u32,
    fighter_kind: i32,
    context: GameplayContext,
) -> bool {
    STATE.lock().ok().map_or(true, |mut state| {
        let actions = state
            .settings_menu
            .as_ref()
            .and_then(|menu| menu.for_fighter(fighter_kind))
            .map_or(0, |menu| menu.manual_native_actions());
        state.grab_handoff.observe_native(true, Some(context));
        let manual_allowed = state.action_owner.current_owner().is_none();
        let pending = state.manual_jump_fence.sample(
            object_id,
            fighter_kind,
            manual_allowed && actions & kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS != 0,
            context,
        );
        manual_allowed && (pending || actions != 0)
    })
}
/// Captures a snapshot of the observation policy for the given fighter, configuration, and context.
fn snapshot_observation_policy(
    fighter_kind: i32,
    manual_automatic_blocked: bool,
    native_status: i32,
) -> ObservationPolicySnapshot {
    STATE
        .lock()
        .ok()
        .map_or_else(ObservationPolicySnapshot::default, |state| {
            let menu = state
                .settings_menu
                .as_ref()
                .and_then(|menu| menu.for_fighter(fighter_kind));
            let committed_automatic = state.action_owner.automatic_active();
            let automatic_suspended = state.fighter_kind != Some(fighter_kind)
                || !committed_automatic
                    && (manual_automatic_blocked
                        || state.ledge_active
                        || menu.is_some_and(|menu| menu.reserves_automatic_modes()));
            let rules = (!automatic_suspended)
                .then(|| menu.and_then(|menu| menu.rule_config()))
                .flatten();
            let dodge_enabled =
                !automatic_suspended && menu.is_some_and(|menu| menu.grab_dodge_enabled());
            let grabs_enabled =
                !automatic_suspended && menu.is_some_and(|menu| menu.out_of_shield_enabled());
            let event_mask = if rules.is_some() { 0b111 } else { 0 }
                | if grabs_enabled {
                    (1 << 3) | (1 << 4) | (1 << 6)
                } else {
                    0
                }
                | if dodge_enabled { 1 << 5 } else { 0 };
            let cached_rules = (!automatic_suspended)
                .then(|| menu.and_then(|menu| menu.automatic_rules()))
                .flatten();
            let ground_watch =
                cached_rules.and_then(|rules| rules.automatic_ground_watch(event_mask));
            let want_up_special =
                cached_rules.is_some_and(|rules| rules.automatic_needs_up_special(event_mask));
            let ordinary_backup = grabs_enabled
                && cached_rules.is_some_and(|rules| {
                    rules.simple_sequence(6).is_some() && !rules.automatic_needs_up_special(1 << 6)
                });
            let (rule_target, dodge_follow, automatic_sequence_committed, up_b_receipt) =
                if automatic_suspended {
                    (None, None, false, false)
                } else {
                    (
                        state
                            .rule_engine
                            .as_ref()
                            .and_then(|engine| engine.target()),
                        state
                            .spot_dodge
                            .as_ref()
                            .and_then(|dodge| dodge.follow_target()),
                        state
                            .rule_engine
                            .as_ref()
                            .is_some_and(|engine| engine.committed())
                            || state.oos.configured_committed()
                                && !state.oos.grab_delivery_pending()
                            || state.spot_dodge.as_ref().is_some_and(|dodge| {
                                dodge.punish_pending() && !dodge.grab_delivery_pending()
                            }),
                        state.oos.up_b_receipt_pending()
                            || state
                                .rule_engine
                                .as_ref()
                                .is_some_and(|engine| engine.up_b_receipt_pending())
                            || state
                                .spot_dodge
                                .as_ref()
                                .is_some_and(|dodge| dodge.up_b_receipt_pending()),
                    )
                };
            let rules_ground = ground_watch.map(|(distance, height)| {
                (
                    distance,
                    height,
                    state
                        .rule_engine
                        .as_ref()
                        .is_some_and(|engine| engine.needs_ground())
                        || state.oos.needs_ground()
                        || state
                            .spot_dodge
                            .as_ref()
                            .is_some_and(|dodge| dodge.needs_ground()),
                )
            });

            ObservationPolicySnapshot {
                automatic_suspended,
                trip_queued: !automatic_suspended
                    && kirby_followups::watching(&state.trip_followup),
                grab_dodge_enabled: dodge_enabled,
                posture_modes: if !automatic_suspended {
                    menu.map_or((false, false), |menu| menu.posture_modes())
                } else {
                    (false, false)
                },
                posture_down: state.posture_down,
                roster_missing: state.ledge_roster.is_none(),
                oos_enabled: grabs_enabled,
                oos_target: if automatic_suspended {
                    None
                } else {
                    state.oos.target()
                },
                rules_on: rules.is_some(),
                rule_target,
                dodge_follow,
                automatic_sequence_committed,
                up_b_receipt,
                rules_ground,
                simple_starts: rules.map_or([None; 3], |rules| rules.simple_starts()),
                dodge_lookahead: menu.map_or(kirby_assist_core::spot_dodge::LOOKAHEAD, |menu| {
                    menu.dodge_lookahead()
                }),
                manual_input_epoch: state.action_owner.physical_inputs().epoch(),
                analysis_settings: menu
                    .map_or(Default::default(), |menu| menu.fighter_tuning(fighter_kind)),
                automatic_limits: menu.map_or([None; 3], |menu| menu.automatic_limits()),
                dodge_direct_up_b: menu
                    .and_then(|menu| menu.grab_dodge_sequence())
                    .is_none_or(kirby_assist_core::reaction_rules::starts_direct_up_b),
                want_up_special,
                ordinary_backup,
                stationary_intent: state
                    .action_owner
                    .policy_inputs()
                    .stationary_assist(native_status),
                grab_intent: state
                    .action_owner
                    .policy_inputs()
                    .grounded_grab_assist(native_status),
                self_shielded: state.action_owner.policy_inputs().shield_held()
                    || matches!(
                        native_status,
                        kirby_assist_core::native_states::GUARD_ON
                            ..=kirby_assist_core::native_states::GUARD_DAMAGE
                    ),
                grab_dash_pending: state.oos.approach_pending(),
            }
        })
}
/// Refreshes the master state for the specified controller, returning the readiness status if available.
/// Represents an observation of a fighter, including its object ID, index, session ticket, fighter kind, and tick timestamp.
struct FighterObservation {
    object_id: u32,
    index: i32,
    session_ticket: SessionTicket,
    fighter_kind: i32,
    tick: i64,
}

/// Represents a publication of an observation, including the session ticket, index, tick timestamp, and setup sequence.
struct ObservationPublication {
    session_ticket: SessionTicket,
    index: i32,
    tick: i64,
    setup_sequence: u64,
}
/// Provides methods for working with observation publications, including checking if they are current.
impl ObservationPublication {
    fn is_current(&self, state: &State) -> bool {
        state.session.accepts(self.session_ticket)
            && state.mapping_index == Some(self.index)
            && kirby_assist_core::observation::observation_inputs_current(
                self.setup_sequence,
                state.observation_sequence,
                self.tick,
                state.raw_tick,
            )
            && RUNTIME_ENABLED.load(Ordering::Acquire)
    }
}
/// Refreshes the observation roster if necessary, based on the requested state, active consumer status, and the current publication.
unsafe fn refresh_observation_roster(
    requested: bool,
    active_consumer: bool,
    publication: &ObservationPublication,
) {
    if !requested || !active_consumer {
        return;
    }
    let needs_roster = STATE.lock().ok().is_some_and(|state| {
        state
            .ledge_roster
            .is_none_or(|(_, observed_tick)| observed_tick != publication.tick)
    });
    if !needs_roster {
        return;
    }
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::RosterScan);
    let roster = fighter_ids();
    if let Ok(mut state) = STATE.lock() {
        if publication.is_current(&state) {
            state.ledge_roster = roster.map(|ids| (ids, publication.tick));
        }
    }
}
/// Begins a full observation of a fighter, updating the state with the associated object, fighter kind, and observation sequence. Returns the new observation sequence if successful.
fn begin_full_observation(
    publication: &ObservationPublication,
    object_id: u32,
    fighter_kind: i32,
    native_status: i32,
    discover_roster: bool,
) -> Option<u64> {
    let mut state = STATE.lock().ok()?;
    if !publication.is_current(&state) {
        return None;
    }
    if state
        .associated_object
        .is_some_and(|previous| previous != object_id)
        || state
            .fighter_kind
            .is_some_and(|previous| previous != fighter_kind)
    {
        state.cancel();
    }
    state.ledge_visits.set_owner(object_id, fighter_kind);
    state.associated_object = Some(object_id);
    state.fighter_kind = Some(fighter_kind);
    state.menu_fighter_kind = Some(fighter_kind);
    if native_status == kirby_assist_core::native_states::status::ENTRY && !state.menu_battle_active
    {
        state.menu_intro_object = Some(object_id);
    }
    state
        .observation_setup
        .complete(publication.tick, discover_roster);
    if state.evaluated_tick == Some(publication.tick) && state.observed_object == Some(object_id) {
        return None;
    }
    state.observation_sequence = state.observation_sequence.wrapping_add(1);
    Some(state.observation_sequence)
}
/// Admits a fighter observation, validating the object and controller index, and updating the state with the observation if successful. Returns the fighter observation if applicable.
unsafe fn admit_fighter_observation(
    object: *mut game::Object,
    discover_roster: bool,
) -> Option<FighterObservation> {
    if object.is_null() {
        return None;
    }
    // The callback owns a live accessor. Reject inactive/recycled identities
    // before dereferencing any ControlModule or other fighter module pointers.
    let object_id = game::battle_object_id(object);
    if object_id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(object_id)
        || game::module_accessor(object_id) != object
    {
        return None;
    }
    // ControlModule +0x118 -> +0x58 -> +8 -> index at +8.
    // A direct ControlModule+8 read does not resolve the controller index.
    let Some(index) = controller_index(object.cast()) else {
        #[cfg(feature = "diagnostic-logging")]
        if feature_activity_allowed() && !ROUTE_UNAVAILABLE_LOGGED.swap(true, Ordering::Relaxed) {
            log!("fighter controller ownership chain unavailable; waiting for a valid local fighter input index");
        }
        return None;
    };
    // Nonselected/remote callbacks may share human metadata. They cannot
    // initialize policies, cache opponents or renew the local menu presence.
    if !STATE
        .lock()
        .ok()
        .is_some_and(|state| state.mapping_index == Some(index))
    {
        return None;
    }
    // CPU actors cannot establish participation even if they share a default
    // input index. Arena roster slots are not physical controller identities.
    if !human_fighter(object) {
        return None;
    }
    let session_ticket = live_match_ticket()?;
    let fighter_kind = game::fighter_kind(object_id);
    let (tick, target) = {
        let mut state = STATE.lock().ok()?;
        if !state.session.accepts(session_ticket) {
            return None;
        }
        let now = Instant::now();
        state.human_routes[index as usize] = Some((object_id, now));
        if state.mapping_index == Some(index) {
            settings_menu::observe_local_fighter(now);
        }
        let tick = state.raw_tick;
        // Keep controller discovery fresh, but skip setup already rejected by
        // the publication guard below. A callback must still refresh the roster
        // after a mapper-only observation, so do not skip that discovery pass.
        if state.mapping_index == Some(index)
            && state.evaluated_tick == Some(tick)
            && state.observed_object == Some(object_id)
            && state.associated_object == Some(object_id)
            && state.fighter_kind == Some(fighter_kind)
            && !state.observation_setup.needed(tick, true, discover_roster)
        {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::ObservationReuse);
            return None;
        }
        (tick, state.mapping_index)
    };
    if target != Some(index) {
        #[cfg(feature = "diagnostic-logging")]
        if ROUTE_MISMATCHES.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0 {
            log!(&format!("human fighter input index {index}; selected physical controller's mapper is {target:?}; route cached for mapping"));
        }
        return None;
    }
    // Menus/recording may still issue fighter callbacks. Do not regenerate
    // proposals or advance recipes while Training owns the selected input.
    if TRAINING_PASSTHROUGH.load(Ordering::Relaxed) && game::is_training_mode() {
        return None;
    }
    Some(FighterObservation {
        object_id,
        index,
        session_ticket,
        fighter_kind,
        tick,
    })
}
/// Represents an observation of a tech situation, including whether the fighter has stopped, clatter status, input delay, auto-tech settings, floor motion history, tech assessment, and timing information.
struct TechObservation {
    stopped: bool,
    clatter_remaining: f32,
    clatter: bool,
    input_delay: f32,
    auto_tech_enabled: bool,
    tech_floor_motion: kirby_assist_core::tech_prediction::FloorMotionHistory,
    assessment: tech::TechAssessment,
    tech: bool,
    native_tech_guard: bool,
    tech_timing: (u16, u16, u16),
}

/// Represents the basic native observation of a fighter, including status, situation, motion details, and input blocking information.
struct NativeObservationBasics {
    status: i32,
    situation: i32,
    stopped: bool,
    motion_frame: f32,
    motion_kind: u64,
    motion_rate: f32,
    manual_automatic_blocked: bool,
    damage_stop: bool,
}
/// Observes the basic native state of a fighter, including status, situation, motion details, and input blocking information. Returns a `NativeObservationBasics` struct with the observed values.
unsafe fn observe_native_basics(
    reads: &frame::FrameReads,
    object: *mut game::Object,
    object_id: u32,
    fighter_kind: i32,
    config: &Config,
) -> NativeObservationBasics {
    let status = reads.status_kind(object);
    let situation = reads.situation_kind(object);
    let stopped = reads.is_stop(object);
    let motion_frame = reads.motion_frame(object);
    let motion_kind = reads.motion_kind(object);
    let motion_rate = reads.motion_rate(object);
    let manual_context = GameplayContext {
        gameplay: true,
        status,
        situation,
        stopped,
        motion_kind,
        motion_frame,
        motion_rate,
        ..GameplayContext::default()
    };
    let manual_automatic_blocked =
        manual_automatic_input_blocked(object_id, fighter_kind, manual_context);
    let damage_stop = (config.auto_sdi || config.auto_di.enabled)
        && (kirby_assist_core::native_states::status::DAMAGE
            ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
            .contains(&status)
        && reads.is_stop(object)
        && game::is_damage_stop(object);
    NativeObservationBasics {
        status,
        situation,
        stopped,
        motion_frame,
        motion_kind,
        motion_rate,
        manual_automatic_blocked,
        damage_stop,
    }
}
/// Observes the tech state of a fighter, including hitstop, clatter, input delay, auto-tech settings, and floor motion history. Returns a `TechObservation` struct with the observed values.
unsafe fn observe_tech_state(
    reads: &frame::FrameReads,
    object: *mut game::Object,
    config: &Config,
    analysis_settings: kirby_assist_core::fighter_tuning::Tuning,
) -> TechObservation {
    // Hitstop freezes native action progress. It differs from cancellation:
    // keep remembered cooldowns/plans while withholding new mash/tech requests.
    let stopped = reads.is_stop(object);
    let clatter_remaining = game::clatter_time(object, 0);
    let clatter = !stopped && clatter_remaining.is_finite() && clatter_remaining > 0.0;
    // Prediction is evaluated separately from the scheduler: finding contact
    // does not mean the cooldown is ready or that the game will accept a tech.
    // Learned input-delay compensation has been removed. Native move timing
    // and explicit response configuration are the only timing inputs.
    let input_delay = 0.0;
    let (auto_tech_enabled, mut tech_floor_motion) =
        STATE.lock().ok().map_or(Default::default(), |state| {
            (
                state
                    .settings_menu
                    .as_ref()
                    .map_or(config.controls.auto_tech, |menu| {
                        menu.auto_tech(config.controls.auto_tech)
                    }),
                state.tech_floor_motion,
            )
        });
    let assessment = assess_tech(
        reads,
        object,
        config.lookahead,
        stopped,
        input_delay,
        auto_tech_enabled,
        &mut tech_floor_motion,
        analysis_settings.tech_body(),
    );
    let tech = !clatter && assessment.prepare_guard && assessment.contact_in.is_some();
    // Read the fighter's consumed button state outside STATE. GuardHold is a
    // second shield alias; a mapped neutral/press can still be in transit.
    let native_tech_guard = auto_tech_enabled
        && (game::button_on(object, game::CONTROL_PAD_BUTTON_GUARD)
            || game::button_on(object, game::CONTROL_PAD_BUTTON_GUARD_HOLD));
    // Current native common.prc values, rather than the legacy configured
    // synthetic forty-frame lockout. Invalid/missing keys retain compatibility.
    // Parameter names and integer types are pinned in skyline-smash params/common.rs.
    let tech_timing = if auto_tech_enabled && (tech || assessment.prepare_guard) {
        self::tech::native_timing(object, config.controls.tech_cooldown)
    } else {
        (
            config.controls.tech_cooldown,
            kirby_assist_core::tech_input::DEFAULT_PASSIVE_TRIGGER_FRAME,
            kirby_assist_core::tech_input::DEFAULT_GANON_PASSIVE_TRIGGER_FRAME,
        )
    };
    TechObservation {
        stopped,
        clatter_remaining,
        clatter,
        input_delay,
        auto_tech_enabled,
        tech_floor_motion,
        assessment,
        tech,
        native_tech_guard,
        tech_timing,
    }
}

// This observer reads game facts and publishes input requests; it never changes
// fighter status. Native reads happen outside STATE and publication revalidates
/// Observes the fighter's state, including native reads and session validation. This function never changes fighter status and ensures that the session is revalidated because native reads can re-enter hooks.
pub(super) unsafe fn observe_fighter(object: *mut game::Object, discover_roster: bool) {
    let Some(config) = CONFIG.get().map(Box::as_ref) else {
        return;
    };
    if !config.controls.enabled || !RUNTIME_ENABLED.load(Ordering::Acquire) {
        return;
    }
    let Some(FighterObservation {
        object_id,
        index,
        session_ticket,
        fighter_kind,
        tick,
    }) = admit_fighter_observation(object, discover_roster)
    else {
        return;
    };
    let Some(_observation_guard) = FighterObservationGuard::acquire() else {
        return;
    };
    // Only an accepted local observation needs profile/menu resolution and
    // policy setup. Duplicate command hooks still refresh human routing and
    // presence above, including the existing roster-discovery exception.
    // Training settings are identical to gameplay settings; recording/menu
    // passthrough was handled above. No extra scene query or config identity pass.
    let config = diagnostic_config(*config);
    let Some((config, setup_sequence, needs)) =
        resolve_observation_setup(index, session_ticket, fighter_kind, config)
    else {
        return;
    };
    let ObservationNeeds {
        find_trump_edge,
        find_trap_edge,
        observe_ledge,
        preview_start,
        continue_ledge_response,
        literal_ledge_timeline,
        file_ledge_timeline,
        ground_ledge_timeline,
        observe_trump,
    } = needs;

    let reads = frame::FrameReads::default();
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::root();
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::Observation);
    let NativeObservationBasics {
        status: native_status,
        situation: native_auto_ground,
        stopped: native_auto_stopped,
        motion_frame: ledge_motion_frame,
        motion_kind: own_motion_kind,
        motion_rate: timing_rate,
        manual_automatic_blocked,
        damage_stop,
    } = observe_native_basics(&reads, object, object_id, fighter_kind, &config);
    // Copy saved-mode flags once; no native queries happen while holding this
    // lock. All readers in this observation use the same configuration snapshot.
    let ObservationPolicySnapshot {
        automatic_suspended,
        trip_queued,
        grab_dodge_enabled,
        posture_modes,
        posture_down,
        roster_missing,
        oos_enabled,
        oos_target,
        rules_on,
        rule_target,
        dodge_follow,
        automatic_sequence_committed,
        up_b_receipt,
        rules_ground,
        simple_starts,
        dodge_lookahead,
        manual_input_epoch,
        analysis_settings,
        automatic_limits,
        dodge_direct_up_b,
        want_up_special,
        ordinary_backup,
        stationary_intent,
        grab_intent,
        self_shielded,
        grab_dash_pending,
    } = snapshot_observation_policy(fighter_kind, manual_automatic_blocked, native_status);
    let own_capture_link = if !automatic_suspended
        && (rules_on
            || oos_enabled
            || grab_dodge_enabled
            || rule_target.is_some()
            || oos_target.is_some()
            || dodge_follow.is_some())
        && (kirby_assist_core::reaction_rules::automatic_start_status(native_status)
            || automatic_sequence_committed
            || native_status == kirby_assist_core::native_states::ESCAPE && dodge_follow.is_some())
    {
        reads.capture_link(object)
    } else {
        false
    };
    let automatic_scope = kirby_assist_core::reaction_rules::AutomaticObservationScope::new(
        automatic_suspended,
        native_status,
        native_auto_ground,
        native_auto_stopped,
        own_capture_link,
        dodge_follow.is_some(),
        automatic_sequence_committed,
    )
    .with_up_b_receipt(
        automatic_suspended,
        own_capture_link,
        up_b_receipt,
        native_status,
        native_auto_ground,
        dodge_follow.is_some(),
    );
    let trip_kirby = kirby_followups::trip_enabled(
        automatic_suspended,
        config.down_tilt_trip_smash,
        fighter_kind,
    );
    let pressure_on = oos_enabled;
    let (legacy_grab_close, legacy_grab_approach) =
        if oos_enabled && (automatic_limits[0].is_none() || automatic_limits[1].is_none()) {
            let standing = if config.assist.fighter_kind == fighter_kind {
                config.assist.fighter.grab.standing_range
            } else {
                0.
            };
            analysis_settings.auto_grab_ranges(fighter_kind, standing)
        } else {
            (0., 0.)
        };
    let auto_grab_range = automatic_limits[1].map_or(legacy_grab_close, |v| v.0);
    let auto_grab_approach_range = automatic_limits[0].map_or(legacy_grab_approach, |v| v.0);
    let auto_grab_height = automatic_limits[0].map_or(24., |v| v.1);
    let auto_shield_height = automatic_limits[1].map_or(24., |v| v.1);
    let auto_backup_range = automatic_limits[2].map_or(auto_grab_range, |v| v.0);
    let auto_backup_height = automatic_limits[2].map_or(24., |v| v.1);
    let trip_watch = kirby_followups::trip_watch(trip_kirby, native_status, trip_queued);
    let observe_roll_read =
        automatic_scope.acquire && (rules_on || pressure_on) || automatic_scope.follow;
    let defense_observation_enabled = grab_dodge_enabled || oos_enabled;
    let roster_scan_requested = discover_roster
        || preview_start
        || (rules_on || pressure_on) && roster_missing
        || defense_observation_enabled && roster_missing
        || posture_modes.1 && posture_down && roster_missing;
    let roster_has_active_consumer = observe_ledge
        || trip_watch
        || damage_stop
        || observe_roll_read
        || posture_modes.1
            && (posture_down
                || matches!(
                    native_status,
                    kirby_assist_core::native_states::status::ATTACK_LW3
                        | kirby_assist_core::native_states::status::ATTACK_S3
                        | kirby_assist_core::native_states::status::ATTACK_HI3
                ))
        || posture_modes.0
        || defense_observation_enabled && automatic_scope.acquire;
    // A first Down input can need facing before any other assist discovered
    // opponents. Seed once here; ordinary refreshes stay on the existing path.
    let publication = ObservationPublication {
        session_ticket,
        index,
        tick,
        setup_sequence,
    };
    refresh_observation_roster(
        roster_scan_requested,
        roster_has_active_consumer,
        &publication,
    );
    let Some(observation_sequence) = begin_full_observation(
        &publication,
        object_id,
        fighter_kind,
        native_status,
        discover_roster,
    ) else {
        return;
    };
    if !config.controls.enabled || !game::is_active(object_id) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::FullObservation);
    let TechObservation {
        stopped,
        clatter_remaining,
        clatter,
        input_delay,
        auto_tech_enabled,
        tech_floor_motion,
        assessment,
        tech,
        native_tech_guard,
        tech_timing,
    } = observe_tech_state(&reads, object, &config, analysis_settings);
    if matches!(
        assessment.status,
        kirby_assist_core::native_states::status::DEAD
            | kirby_assist_core::native_states::status::REBIRTH
            | kirby_assist_core::native_states::status::STANDBY
            | kirby_assist_core::native_states::status::ENTRY
            ..=kirby_assist_core::native_states::status::LOSE
    ) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }

    // Read native data outside STATE's mutex to avoid re-entrant deadlocks.
    // One bounded roster pass, only while custom rules are enabled. Follow the
    // locked target through recovery without a second scan or stale position.
    let rule_observation = if automatic_scope.acquire && (rules_on || pressure_on)
        || rule_target.is_some()
        || oos_target.is_some()
        || dodge_follow.is_some()
    {
        let (targets, follow) = if !clatter && !tech && automatic_scope.observe_targets() {
            observations::rule_targets(
                &reads,
                object,
                object_id,
                rule_target.or(oos_target).or(dodge_follow),
                (oos_enabled && automatic_scope.acquire).then_some(if self_shielded {
                    [
                        auto_grab_range.max(auto_backup_range),
                        auto_shield_height.max(auto_backup_height),
                    ]
                } else {
                    [auto_grab_approach_range, auto_grab_height]
                }),
                self_shielded,
                rules_on && automatic_scope.acquire,
                automatic_sequence_committed,
            )
        } else {
            ([None; 8], None)
        };
        let position = [reads.pos_x(object), reads.pos_y(object)];
        let velocity = [reads.speed_x(object), reads.speed_y(object)];
        let rule_needs_ground = rule_needs_ground_observation(
            rules_ground,
            rule_target.is_some() || oos_target.is_some(),
            &targets,
            position,
            dodge_follow.is_some(),
            follow.as_ref(),
        );
        let grab_ground = oos_enabled
            && !self_shielded
            && grab_intent
            && !stopped
            && (grab_dash_pending
                || kirby_assist_core::out_of_shield::ground_grab_state(assessment.status)
                    && targets.iter().flatten().any(|t| {
                        kirby_assist_core::out_of_shield::automatic_grab_target_status(t.status)
                            && t.vulnerable_in == Some(0.)
                            && assessment.status == kirby_assist_core::native_states::status::WAIT
                            && (t.position[0] - position[0]).abs() > auto_grab_range
                    }));
        let needs_ground = assessment.situation
            == kirby_assist_core::native_states::situation::GROUND
            && (rule_needs_ground || grab_ground);
        Some((
            GameplayContext {
                held: true,
                gameplay: true,
                manual_input_epoch,
                stopped,
                status: assessment.status,
                situation: assessment.situation,
                motion_frame: ledge_motion_frame,
                motion_kind: own_motion_kind,
                motion_rate: timing_rate,
                position,
                velocity,
                facing: reads.facing(object),
                previous_status: reads
                    .previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
                // Only admit a new grounded Up-B reaction when the native game
                // can accept it now. Locomotion keeps its momentum until then.
                can_act: want_up_special
                    && !own_capture_link
                    && !clatter
                    && !tech
                    && !stopped
                    && assessment.situation == kirby_assist_core::native_states::situation::GROUND
                    && (automatic_scope.acquire
                        || automatic_scope.follow
                            && assessment.status == kirby_assist_core::native_states::ESCAPE
                            && game::cancel_enabled(object))
                    && game::transition_enabled(
                        object,
                        kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_HI,
                    ), // CONT_SPECIAL_HI
                ground_brake: if assessment.situation
                    == kirby_assist_core::native_states::situation::GROUND
                    && rule_needs_ground
                {
                    reads.ground_brake(object)
                } else {
                    None
                },
                safe_ground: if needs_ground {
                    reads.ground_support(object, position, velocity[0])
                } else {
                    [false; 2]
                },
                ..GameplayContext::default()
            },
            targets,
            follow,
        ))
    } else {
        None
    };
    // Per-event readiness is computed outside STATE, only for an actual nearby
    // candidate. Ordinary fighter recipes never depend on Up-B availability.
    let simple_readiness = rule_observation
        .as_ref()
        .map_or([false; 3], |(context, targets, _)| {
            kirby_assist_core::reaction_rules::simple_readiness(
                simple_starts,
                *context,
                targets,
                || {
                    game::transition_enabled(object, kirby_assist_core::native_states::transition_terms::CONT_CATCH) // CONT_CATCH
                || game::transition_enabled(object, kirby_assist_core::native_states::transition_terms::CONT_JUMP_SQUAT_BUTTON) // CONT_JUMP_SQUAT_BUTTON
                },
            )
        });
    // Query grab/OOS readiness only with a current vulnerable target or active attempt.
    let oos_readiness = if oos_enabled
        && automatic_scope.acquire
        && (grab_intent || oos_target.is_some())
        && assessment.situation == kirby_assist_core::native_states::situation::GROUND
        && kirby_assist_core::out_of_shield::ground_grab_state(assessment.status)
        && (oos_target.is_some()
            || rule_observation.as_ref().is_some_and(|(_, targets, _)| {
                targets
                    .iter()
                    .flatten()
                    .any(|t| t.vulnerable_in == Some(0.))
            })) {
        let ready = !stopped
            && (assessment.status != kirby_assist_core::native_states::GUARD_DAMAGE
                || game::work_int(
                    object,
                    kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME,
                ) <= 0);
        (
            ready,
            ready
                && kirby_assist_core::out_of_shield::automatic_grab_ready_with_shield(
                    assessment.status,
                    self_shielded,
                    |term| game::transition_enabled(object, term),
                ),
            ready
                && ordinary_backup
                && self_shielded
                && (game::transition_enabled(
                    object,
                    kirby_assist_core::native_states::transition_terms::CONT_JUMP_SQUAT_BUTTON,
                ) || game::transition_enabled(
                    object,
                    kirby_assist_core::native_states::transition_terms::CONT_JUMP_SQUAT,
                )),
            ready && self_shielded && rule_observation.as_ref().is_some_and(|(c, _, _)| c.can_act),
        )
    } else {
        (false, false, false, false)
    };
    // Observation stage: build ordinary Rust values from read-only game calls.
    // Startup/cancel/hit-status queries describe the real game; they do not
    // make an input legal. The core uses these values to decide what to request.
    // Full movement/ledge queries (including ground rays) are demand-driven.
    // Stick-click/Y shortcuts use the existing motion sample, without ground rays.
    let mut guard_threat = false;
    let mut guard_ready = false;
    // Status-gated opponent acquisition comes first. Unrelated enemies stop at
    // their status checks; a ledge grab/getup gets full facts in this same pass.
    let ledge_opponent = if observe_ledge
        && literal_ledge_timeline
        && kirby_assist_core::ledge_trap::needs_opponent_clock(native_status, timing_rate)
    {
        observations::tracked_ledge_clock(&reads)
    } else {
        (observe_ledge && !literal_ledge_timeline)
            .then(|| nearest_ledge_opponent(&reads, object, object_id, config.assist))
            .flatten()
    };
    let response_observation = observe_ledge
        && !file_ledge_timeline
        && (ledge_opponent.is_some() || continue_ledge_response);
    let spatial_timeline = observe_ledge && ground_ledge_timeline;
    let position_observation = observe_ledge
        && !file_ledge_timeline
        && config.assist.auto_position
        && !config.assist.manual_spacing
        && !config.assist.distance_agnostic;
    let ledge_observation = if observe_ledge || observe_trump {
        #[cfg(feature = "diagnostic-logging")]
        let _performance = crate::performance::Scope::new(crate::performance::Part::Ledge);
        let position = [reads.pos_x(object), reads.pos_y(object)];
        let velocity = [reads.speed_x(object), reads.speed_y(object)];
        let guard_armed = observe_ledge
            && !file_ledge_timeline
            && config.autopilot_shield
            && assessment.situation == kirby_assist_core::native_states::situation::GROUND
            && !kirby_assist_core::native_states::fighter_sequence_interrupted(
                fighter_kind,
                assessment.status,
            )
            && !clatter
            && !tech
            && STATE.lock().ok().is_some_and(|s| s.ledge_active);
        if guard_armed {
            // CONT_GUARD_ON, verified in pinned skyline-smash lua_const.rs.
            // Existing guard/shieldstun can retain ZR even during hitstop.
            guard_ready = matches!(
                assessment.status,
                kirby_assist_core::native_states::GUARD_ON
                    | kirby_assist_core::native_states::GUARD
                    | kirby_assist_core::native_states::GUARD_DAMAGE
            ) || (!stopped
                && game::transition_enabled(
                    object,
                    kirby_assist_core::native_states::transition_terms::CONT_GUARD_ON,
                ));
            guard_ready &= !kirby_assist_core::action_owner::native_grab_active(assessment.status)
                && !reads.capture_link(object);
            if guard_ready {
                guard_threat = observations::incoming_fighter_attack(
                    &reads,
                    object,
                    object_id,
                    position,
                    velocity,
                    analysis_settings.shield(),
                );
            }
        }
        // File actions are literal inputs; do not query move startup/readiness.
        let shield_stun_frames = None;
        let shield_attacker = None;
        let context = GameplayContext {
            held: false,
            gameplay: true,
            manual_input_epoch,
            can_act: false,
            can_control: false,
            stopped,
            status: assessment.status,
            situation: assessment.situation,
            motion_frame: ledge_motion_frame,
            motion_kind: own_motion_kind,
            motion_rate: timing_rate,
            position,
            velocity,
            ground_brake: if response_observation || spatial_timeline || position_observation {
                reads.ground_brake(object)
            } else {
                None
            },
            facing: reads.facing(object),
            previous_status: reads
                .previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
            can_grab: false,
            can_tilt: false,
            can_down_tilt: false,
            can_down_special: false,
            can_dash_attack: false,
            can_down_smash: false,
            can_jump: false,
            can_air_attack: false,
            input_delay: 0.0,
            air_physics: None,
            short_hop_speed: None,
            full_hop_speed: None,
            shield_stun_frames,
            shield_attacker,
            hit_confirmed: false,
            safe_ground: if assessment.situation
                == kirby_assist_core::native_states::situation::GROUND
                && (response_observation
                    || spatial_timeline
                    || observe_trump
                    || position_observation)
            {
                reads.ground_support(object, position, velocity[0])
            } else {
                [false, false]
            },
        };
        // The trump uses the current fighter's floor geometry, independently of the
        // opponent roster or Y acquisition range. Query native rays outside STATE.
        let edge = if find_trump_edge
            && turbo_movement_allowed(context.status, context.situation, stopped)
            && !clatter
            && !tech
        {
            nearest_floor_edge(position, context.facing, |probe| {
                let origin = game::Vector2 {
                    x: probe[0],
                    y: probe[1] + 2.0,
                };
                let delta = game::Vector2 { x: 0.0, y: -6.0 };
                game::ray_check(object, &origin, &delta, true) != 0
            })
        } else {
            None
        };
        let spot = if find_trap_edge
            && turbo_movement_allowed(context.status, context.situation, stopped)
            && !clatter
            && !tech
        {
            nearest_ground_ledge(
                position,
                context.facing,
                |probe| {
                    let origin = game::Vector2 {
                        x: probe[0],
                        y: probe[1] + 2.0,
                    };
                    let delta = game::Vector2 { x: 0.0, y: -6.0 };
                    game::ray_check(object, &origin, &delta, true) != 0
                },
                config.assist.stand_distance,
            )
        } else {
            None
        };
        Some((context, ledge_opponent, edge, spot))
    } else {
        None
    };
    let (trip_targets, outward) = if trip_watch || damage_stop {
        combat_targets(&reads, object, object_id, trip_watch, damage_stop)
    } else {
        ([None; 8], 0.0)
    };
    let followup_observation = kirby_followups::Observation {
        object: object_id,
        kind: fighter_kind,
        status: assessment.status,
        situation: assessment.situation,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        rate: timing_rate,
        stopped,
        facing: reads.facing(object),
    };
    let trip_context =
        kirby_followups::trip_context(&followup_observation, trip_kirby, trip_targets);
    let fair_pending = !automatic_suspended
        && STATE
            .lock()
            .ok()
            .is_some_and(|s| kirby_followups::fair_active(&s.fair_followup));
    // Only the still-supported Kirby automatic fair-one follow-up needs hit
    // evidence. Retired manually bound finishers do not widen this window.
    let fair_hit_window = kirby_followups::fair_hit_window(
        automatic_suspended,
        kirby_followups::is_kirby(fighter_kind),
        config.kirby_fair1_smash,
        assessment.status,
        own_motion_kind,
    );
    let fair_context =
        kirby_followups::fair_context(&followup_observation, fair_hit_window, || {
            game::attack_infliction_now(object, game::COLLISION_KIND_MASK_HIT)
        });
    let observe_posture_target = (posture_modes.0
        || posture_modes.1
            && (posture_down
                || matches!(
                    assessment.status,
                    kirby_assist_core::native_states::status::ATTACK_LW3
                        | kirby_assist_core::native_states::status::ATTACK_S3
                        | kirby_assist_core::native_states::status::ATTACK_HI3
                )))
        && kirby_assist_core::posture::Posture::needs_configured_target(
            assessment.status,
            assessment.situation,
            stopped || clatter || tech,
        );
    let posture_opponent = if observe_posture_target {
        observations::posture_opponent(&reads, object, object_id)
    } else {
        None
    };
    let posture_target = posture_opponent
        .map(|offset| offset[0])
        .filter(|x| x.abs() > 0.5)
        .map(f32::signum);
    // A Down input on a passable platform may drop through. Leave those floors
    // free of automatic Down; walking remains capped on these floors.
    let posture_floor_safe = posture_modes.0
        && assessment.situation == kirby_assist_core::native_states::situation::GROUND
        && kirby_assist_core::posture::Posture::opponent_near_with(
            posture_opponent,
            analysis_settings.value(kirby_assist_core::fighter_tuning::CROUCH_RANGE, 45.),
            analysis_settings.value(kirby_assist_core::fighter_tuning::CROUCH_HEIGHT, 20.),
        )
        && !game::passable_ground(object);
    let manual_context = ManualContext {
        status: assessment.status,
        situation: assessment.situation,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        rate: timing_rate,
        stopped: stopped || clatter || tech,
        facing: trip_context.facing,
    };
    let combo_velocity_y = if fair_hit_window || fair_pending {
        reads.speed_y(object)
    } else {
        0.0
    };
    let dodge_response_ready = if grab_dodge_enabled && !dodge_direct_up_b {
        !own_capture_link
            && !stopped
            && assessment.situation == kirby_assist_core::native_states::situation::GROUND
            && (automatic_scope.acquire
                || automatic_scope.follow
                    && assessment.status == kirby_assist_core::native_states::ESCAPE
                    && game::cancel_enabled(object))
    } else {
        false
    };
    let spot_observation = if grab_dodge_enabled
        && !own_capture_link
        && assessment.situation == kirby_assist_core::native_states::situation::GROUND
        && (automatic_scope.acquire
            || automatic_scope.follow && assessment.status == kirby_assist_core::spot_dodge::ESCAPE)
        && (stationary_intent && kirby_assist_core::spot_dodge::ground_status(assessment.status)
            || assessment.status == kirby_assist_core::spot_dodge::ESCAPE && dodge_follow.is_some())
    {
        // Detect a real grab before querying our dodge transition. Keep the
        // ordinary sample even with no target so pending requests can release.
        let targets = observations::grab_targets(&reads, object, object_id);
        Some((
            kirby_assist_core::spot_dodge::Context {
                object: object_id,
                status: assessment.status,
                situation: assessment.situation,
                motion: own_motion_kind,
                frame: ledge_motion_frame,
                rate: timing_rate,
                stopped,
                can_spot_dodge: targets.iter().any(Option::is_some)
                    && !clatter
                    && !tech
                    && !stopped
                    && (game::transition_enabled(
                        object,
                        kirby_assist_core::native_states::transition_terms::CONT_ESCAPE,
                    ) || kirby_assist_core::spot_dodge::ground_status(assessment.status)
                        && game::transition_enabled(
                            object,
                            kirby_assist_core::native_states::transition_terms::CONT_GUARD_ON,
                        )),
                position: [reads.pos_x(object), reads.pos_y(object)],
                velocity: [reads.speed_x(object), reads.speed_y(object)],
            },
            targets,
        ))
    } else {
        None
    };
    let mut sdi_context = SdiContext {
        object: object_id,
        damage_stop,
        input_delay: 0.0,
        remaining: if damage_stop {
            game::hitstop_remaining(object)
        } else {
            0
        },
        outward,
    };
    // Read reserved launch velocity only for the damage victim during hitstop.
    // Pinned FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X/Y.
    let di_context = (config.auto_di.enabled
        && damage_stop
        && (kirby_assist_core::native_states::status::DAMAGE_FLY
            ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
            .contains(&native_status))
    .then(|| kirby_assist_core::auto_di::Context {
        object: object_id,
        damage_stop,
        remaining: sdi_context.remaining,
        launch: reads.damage_launch(object),
        outward,
    });
    let cached_geometry = STATE.lock().ok().and_then(|s| s.damage_geometry);
    let (damage_geometry, damage_trajectory) = damage::observe(
        &reads,
        object,
        object_id,
        damage_stop,
        sdi_context.remaining,
        cached_geometry,
    );
    #[cfg(feature = "diagnostic-logging")]
    let mut scenario_changed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut acquired = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut first_button_write = false;
    let oos_follow = rule_observation.and_then(|(_, _, follow)| follow);
    let oos_context =
        ((oos_enabled && automatic_scope.acquire) || oos_target.is_some()).then(|| {
            kirby_assist_core::out_of_shield::Context {
                object: object_id,
                kind: fighter_kind,
                status: assessment.status,
                motion: own_motion_kind,
                frame: ledge_motion_frame,
                rate: timing_rate,
                stopped,
                grounded: assessment.situation
                    == kirby_assist_core::native_states::situation::GROUND,
                position: [reads.pos_x(object), reads.pos_y(object)],
                facing: reads.facing(object),
                shield_ready: oos_readiness.0,
                can_grab: oos_readiness.1,
                can_jump: oos_readiness.2,
                can_up_special: oos_readiness.3,
                threatened: guard_threat,
            }
        });
    #[cfg(feature = "diagnostic-logging")]
    let neutral_native_activity = if observe_ledge
        && (ledge_opponent.is_some_and(|e| e.status == kirby_assist_core::ledge_trap::CLIFF_CLIMB)
            || STATE.lock().ok().is_some_and(|s| {
                s.calibration.active()
                    && s.ledge.as_ref().and_then(LedgeTrap::calibration_slot)
                        == Some(kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral)
            })) {
        Some(
            matches!(
                assessment.status,
                kirby_assist_core::native_states::status::ATTACK
                    ..=kirby_assist_core::native_states::status::ATTACK_AIR
            ) && (0..ATTACK_SLOT_COUNT).any(|slot| game::attack_active(object, slot, false)),
        )
    } else {
        None
    };
    let reflect_observation = {
        let (configured, target, allowed) = STATE.lock().ok().map_or((None, None, false), |s| {
            let manual = s.raw_buttons
                & (kirby_assist_core::native_input::MAPPER_A
                    | kirby_assist_core::native_input::MAPPER_B
                    | kirby_assist_core::native_input::MAPPER_R
                    | kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS
                    | kirby_assist_core::native_input::SEQUENCE_GUARD_INPUTS)
                != 0;
            (
                s.settings_menu
                    .as_ref()
                    .and_then(|m| m.reflect_config(fighter_kind)),
                s.reflector.target(),
                !automatic_suspended
                    && !clatter
                    && !tech
                    && (!manual || s.action_owner.current_owner() == Some(Owner::Reflect))
                    && s.action_owner.allows(Owner::Reflect),
            )
        });
        reflect::observe(
            &reads,
            object,
            object_id,
            fighter_kind,
            configured,
            target,
            allowed,
        )
    };
    // Policy/publication stage: the native reads above are finished. Check the
    // session again before updating history, because those reads can re-enter
    // hooks. No file loading or JSON parsing belongs in this per-update path.
    if let Ok(mut state) = STATE.lock() {
        // Queries belong to the controller sample captured before observation.
        // Never relabel a late result as a newer sample: buttons/ledge priority
        // and native eligibility may have changed while those reads ran.
        // The mapper also refreshes synchronously before selecting its packet.
        if !state.session.accepts(session_ticket)
            || state.mapping_index != Some(index)
            || state.associated_object != Some(object_id)
            || state.observation_sequence != observation_sequence
            || !kirby_assist_core::observation::input_update_current(Some(tick), state.raw_tick)
            || !RUNTIME_ENABLED.load(Ordering::Acquire)
        {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(
                crate::performance::health::Event::PublicationDiscard,
            );
            return;
        }
        if state.evaluated_tick == Some(tick) && state.observed_object == Some(object_id) {
            return;
        }
        state.tech_floor_motion = tech_floor_motion;
        if state.grab_handoff.pending() {
            let clock_context = rule_observation.map_or(
                GameplayContext {
                    gameplay: true,
                    status: native_status,
                    situation: native_auto_ground,
                    stopped: native_auto_stopped,
                    motion_kind: own_motion_kind,
                    motion_frame: ledge_motion_frame,
                    motion_rate: timing_rate,
                    ..Default::default()
                },
                |(c, _, _)| c,
            );
            let clock_enemy =
                rule_observation.and_then(|(_, targets, follow)| {
                    follow.or_else(|| {
                        targets.iter().flatten().next().map(|t| {
                            kirby_assist_core::gameplay::Opponent {
                                id: t.id,
                                status: t.status,
                                motion: Some(kirby_assist_core::gameplay::MotionTiming {
                                    kind: t.motion,
                                    frame: t.frame,
                                    rate: t.rate,
                                    remaining: None,
                                }),
                                ..Default::default()
                            }
                        })
                    })
                });
            state
                .grab_handoff
                .observe_enemy_progress(clock_context, clock_enemy);
        }
        // Reuse facts already read by active analyses. Missing targets are not
        // evidence of a new event; a known native status change is. This adds
        // no roster pass or native queries and prevents duplicate/stale reads.
        if state.spot_dodge.is_some() || state.rule_engine.is_some() || oos_enabled {
            let known_statuses = reads.known_statuses();
            state.oos.observe_statuses(&known_statuses);
            if let Some(spot) = state.spot_dodge.as_mut() {
                spot.observe_statuses(&known_statuses);
            }
            if let Some(engine) = state.rule_engine.as_mut() {
                engine.observe_statuses(&known_statuses);
            }
        }
        let now = Instant::now();
        if !state.raw_connected
            || state
                .raw_seen
                .is_none_or(|seen| now.duration_since(seen) >= CONTROLLER_SAMPLE_LIFETIME)
            || state.raw_buttons & MAPPER_PLUS != 0
        {
            state.cancel();
            return;
        }
        // The preceding mapper delivered the final release. Drop completed
        // locks before proposal generation, not after this frame's candidates
        // have already been rejected by allows(). This adds no release frame.
        if !state.rule_engine.as_ref().is_some_and(|e| e.committed()) {
            state.action_owner.release(Owner::RollRead);
        }
        if state.oos.target().is_none() {
            state.action_owner.release(Owner::OutOfShield);
        }
        // Establish priority before sampling policies: a previously committed
        // general read must not prevent same-observation ledge acquisition.
        // Keep watches fresh so static guard can acknowledge elapsed native
        // frames, but do not acquire/consume another reaction behind a queued R.
        // Reconcile raw cancellation with current native/manual eligibility.
        // A cancelled unshielded response must not add a frame to a legal OOS grab.
        // Continuation uses the committed snapshot. Once a completed owner is
        // released above, new admission must see the current physical intent.
        let stationary_intent = state
            .action_owner
            .policy_inputs()
            .stationary_assist(assessment.status);
        let grab_intent = state
            .action_owner
            .policy_inputs()
            .grounded_grab_assist(assessment.status);
        let self_shielded = state.action_owner.policy_inputs().shield_held()
            || matches!(
                assessment.status,
                kirby_assist_core::native_states::GUARD_ON
                    ..=kirby_assist_core::native_states::GUARD_DAMAGE
            );
        let admission_suspended = automatic_suspended
            || state.ledge_active
            || state
                .settings_menu
                .as_ref()
                .is_some_and(|m| m.reserves_automatic_modes());
        state
            .action_owner
            .set_automatic_suspended(admission_suspended);
        let receipt_blocked =
            state.grab_handoff.pending() && state.action_owner.current_owner().is_none();
        let automatic_suspended = automatic_suspended || receipt_blocked;
        let ledge_priority = state.ledge_active
            && observe_ledge
            && state
                .ledge
                .as_ref()
                .map_or(ledge_opponent.is_some(), |trap| {
                    trap.reserves_response(ledge_opponent)
                });
        state.action_owner.set_ledge_priority(ledge_priority);
        #[cfg(feature = "diagnostic-logging")]
        {
            first_button_write = state.buttons_modified && !state.button_mod_logged;
            state.button_mod_logged |= first_button_write;
            acquired = state.observed_object != Some(object_id);
        }
        #[cfg(feature = "diagnostic-logging")]
        let combo_performance = crate::performance::Scope::new(crate::performance::Part::Combo);
        let fair_allowed = state.action_owner.allows(Owner::Fair);
        state.fair_output = kirby_followups::fair(
            &mut state.fair_followup,
            kirby_followups::is_kirby(fighter_kind) && config.kirby_fair1_smash,
            !automatic_suspended && !clatter && !tech && fair_allowed,
            fair_context,
            combo_velocity_y,
        );
        // An incoming-grab defense is sampled once before new offensive
        // choices. Existing committed owners remain protected by allows().
        state.spot_output = if let Some((context, targets)) = spot_observation {
            let allowed = !automatic_suspended
                && !clatter
                && !tech
                && !own_capture_link
                && stationary_intent
                && state.action_owner.allows(Owner::GrabDodge);
            let (_previous_target, output, _current_target) = {
                let State {
                    spot_dodge,
                    action_owner,
                    ..
                } = &mut *state;
                let spot = spot_dodge.get_or_insert_with(Default::default);
                let _previous_target = spot.target();
                let dodge = spot.sample_with_shape(
                    allowed,
                    context,
                    targets,
                    dodge_lookahead,
                    analysis_settings
                        .value(kirby_assist_core::fighter_tuning::GRAB_DODGE_PADDING, 2.),
                    analysis_settings
                        .value(kirby_assist_core::fighter_tuning::GRAB_DODGE_HEIGHT, 8.),
                );

                // Native dodge startup or a cancelled warning releases its
                // defense-only lock before the ordinary follow-up is sampled.
                if spot.target().is_none() {
                    if spot.punish_pending() {
                        action_owner.handoff(Owner::GrabDodge, Owner::OptionRead);
                    } else {
                        action_owner.release(Owner::GrabDodge);
                    }
                }
                let output = dodge;
                let target = spot.target();
                (_previous_target, output, target)
            };
            #[cfg(feature = "diagnostic-logging")]
            if output.requested != 0 && _previous_target.is_none() {
                log!(&format!("AUTO SPOT DODGE proposed: ordinary grab {:?} is within the configured timing/reach forecast. Down+Guard requested; protected ledge/safety priority and native readiness still apply. Native avoidance is unverified.", _current_target));
            } else if _previous_target.is_some()
                && context.status == kirby_assist_core::spot_dodge::ESCAPE
            {
                log!("AUTO SPOT DODGE: native spot-dodge startup observed; released input. Grab avoidance remains unverified.");
            } else if _previous_target.is_some() && output.owned == 0 {
                log!("AUTO SPOT DODGE released without native startup evidence: threat/readiness scope changed, delivery expired, or higher priority prevented it. Avoidance is unverified.");
            }
            output
        } else {
            if let Some(spot) = state.spot_dodge.as_mut() {
                spot.cancel_pending();
                if !grab_dodge_enabled || automatic_suspended || clatter || tech {
                    spot.suspend_response();
                }
            }
            ControllerOutput::default()
        };
        // A confirmed own dodge releases Down/Guard before a single Up-B plan.
        // Native-confirmed dodge recovery retains the chained response owner;
        // its neutral wait cannot expose manual inputs or a competing punish.
        let punish_allowed = grab_dodge_enabled
            && !automatic_suspended
            && !own_capture_link
            && !clatter
            && !tech
            && !state.oos.grab_delivery_pending()
            && state.action_owner.allows(Owner::OptionRead);
        if let Some((context, _, follow)) = rule_observation {
            let State {
                spot_dodge,
                settings_menu,
                action_owner,
                spot_output,
                ..
            } = &mut *state;
            if let Some(spot) = spot_dodge.as_mut() {
                let sequence = settings_menu.as_ref().and_then(|m| m.grab_dodge_sequence());
                let context = if dodge_direct_up_b {
                    context
                } else {
                    GameplayContext {
                        can_act: dodge_response_ready,
                        ..context
                    }
                };
                let punish =
                    spot.punish_automatic(punish_allowed, sequence, config.assist, context, follow);
                if punish.owned != 0 {
                    *spot_output = punish;
                }
                if !spot.punish_pending() && punish.owned == 0 {
                    action_owner.release(Owner::OptionRead);
                }
            }
        } else if let Some(spot) = state.spot_dodge.as_mut() {
            spot.suspend_response();
        }
        // An immediately usable Up-B may replace an unaccepted automatic grab.
        // Admission and this handoff share the core selector; cached Grab input
        // is cleared before the winning reaction can reach the mapper.
        let preempt_grab = |state: &mut State| {
            let prioritize_up_b = (state.action_owner.allows(Owner::RollRead)
                || state.oos.approach_active()
                    && state
                        .action_owner
                        .allows_after_release(Owner::RollRead, Owner::OutOfShield))
                && rules_on
                && !automatic_suspended
                && !clatter
                && !tech
                && !file_ledge_timeline
                && !ledge_priority
                && state.spot_output.owned == 0;
            if prioritize_up_b
                && state.oos.automatic_grab_pending()
                && !state.oos.grab_delivery_pending()
            {
                if let Some((context, targets, follow)) = rule_observation {
                    let yielded = {
                        let State {
                            rule_engine,
                            settings_menu,
                            oos,
                            action_owner,
                            ..
                        } = state;
                        if let Some(rules) = settings_menu.as_ref().and_then(|m| m.rule_config()) {
                            let engine = rule_engine.get_or_insert_with(Box::default);
                            engine.preempt_automatic_grab(
                                true,
                                rules,
                                context,
                                &targets,
                                follow,
                                oos,
                                action_owner,
                            )
                        } else {
                            false
                        }
                    };
                    if yielded {
                        state.oos_output = ControllerOutput::default();
                    }
                }
            }
        };
        preempt_grab(&mut state);
        // Sample remaining pending grabs once so accepted/failed attempts
        // release before fallback. New reactions precede NEW grabs.
        let shield_fallback_pending = state.oos.shield_up_b_pending();
        let grab_pending = state.oos.target().is_some();
        let sample_grab = |state: &mut State| {
            let oos_reserved = automatic_suspended
                || state.action_owner.current_owner() != Some(Owner::OutOfShield)
                    && (ledge_priority || file_ledge_timeline);
            let protected_up_b = state.action_owner.current_owner() == Some(Owner::OutOfShield);
            let oos_allowed = oos_enabled
                && !own_capture_link
                && (automatic_scope.acquire
                    || automatic_scope.follow
                    || state.oos.configured_committed())
                && (grab_intent || state.oos.configured_committed() || protected_up_b)
                && !oos_reserved
                && state.spot_output.owned == 0
                && state.roll_read_output.owned == 0
                && !clatter
                && !tech
                && state.action_owner.allows(Owner::OutOfShield)
                && !state.rule_engine.as_ref().is_some_and(|r| r.committed());
            state.oos_output = if let Some(context) = oos_context {
                let (lc, targets) = rule_observation
                    .map(|(c, t, _)| (c, t))
                    .unwrap_or((GameplayContext::default(), [None; 8]));
                let State {
                    oos, settings_menu, ..
                } = state;
                let sequences = settings_menu
                    .as_ref()
                    .map_or(Default::default(), |m| m.automatic_sequences());
                oos.sample_automatic_configured(
                    oos_allowed,
                    context,
                    lc,
                    targets,
                    oos_follow,
                    self_shielded,
                    legacy_grab_close,
                    legacy_grab_approach,
                    Some(f32::from(if self_shielded {
                        config.assist.fighter.grab.shield_startup
                    } else {
                        config.assist.fighter.grab.dash_startup
                    })),
                    config.assist,
                    sequences,
                )
            } else {
                state.oos = Default::default();
                ControllerOutput::default()
            };
            if state.oos.target().is_none() && state.oos_output.owned == 0 {
                state.action_owner.release(Owner::OutOfShield);
            }
        };
        if grab_pending {
            sample_grab(&mut state);
        }
        // A cancelled shield fallback may have selected a fresh front grab.
        // Apply the same roll priority before either new packet is emitted.
        if shield_fallback_pending
            && !state.oos.shield_up_b_pending()
            && state.oos.automatic_grab_pending()
        {
            // The old fallback ended; the replacement is a new proposal and
            // must compete under current admission priority, not inherit a lock.
            state.action_owner.release(Owner::OutOfShield);
            preempt_grab(&mut state);
        }
        if !rules_on {
            if automatic_suspended {
                if let Some(e) = state.rule_engine.as_mut() {
                    e.cancel();
                }
            } else {
                state.rule_engine = None;
            }
        }
        state.roll_read_output = if let Some((context, targets, follow)) = rule_observation {
            let allowed = rules_on
                && !automatic_suspended
                && !own_capture_link
                && !clatter
                && !tech
                && (state.action_owner.current_owner() == Some(Owner::RollRead)
                    || !file_ledge_timeline && !ledge_priority)
                && state.spot_output.owned == 0
                && !state.oos.grab_delivery_pending()
                && state.action_owner.allows(Owner::RollRead);
            let state = &mut *state;
            let rules = rules_on
                .then(|| state.settings_menu.as_ref().and_then(|m| m.rule_config()))
                .flatten();
            kirby_assist_core::reaction_rules::sample_optional_simple(
                &mut state.rule_engine,
                allowed,
                rules,
                config.assist,
                context,
                targets,
                follow,
                simple_readiness,
            )
        } else {
            if let Some(e) = state.rule_engine.as_mut() {
                e.cancel();
            }
            ControllerOutput::default()
        };
        if !grab_pending {
            sample_grab(&mut state);
        }
        #[cfg(feature = "diagnostic-logging")]
        drop(combo_performance);
        let trip_allowed = state.action_owner.allows(Owner::Trip);
        state.trip_output = kirby_followups::trip(
            &mut state.trip_followup,
            trip_kirby,
            !clatter && !tech && trip_allowed,
            trip_context,
        );
        #[cfg(feature = "diagnostic-logging")]
        {
            let key = (
                kirby_followups::watching(&state.trip_followup),
                sdi_context.damage_stop && sdi_context.remaining <= 120,
                state.trip_output.requested,
            );
            if key != state.last_combat_diagnostic {
                state.last_combat_diagnostic = key;
                state.pending_combat_log = Some(format!(
                    "combat assist: trip_queued={}, SDI_damage_stop={}, trip_request={:#x}, native_hitstop_remaining={}",
                    key.0, key.1, key.2, sdi_context.remaining));
            }
        }
        sdi_context.input_delay = input_delay;
        state.sdi_context = Some(sdi_context);
        state.di_context = di_context;
        state.damage_geometry = damage_geometry;
        state.damage_trajectory = damage_trajectory;
        state.observed_object = Some(object_id);
        state.evaluated_tick = Some(tick);
        // Tech needs the last GAME-visible ZR to decide whether a release is
        // required. The restored physical trace may differ from that last output.
        let previous_buttons =
            if state.last_output_buttons & config.control_layout.physical_buttons(MAPPER_ZR) != 0 {
                HID_ZR
            } else {
                0
            };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let relevant = (kirby_assist_core::native_states::status::DAMAGE
                ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
                .contains(&assessment.status)
                || (kirby_assist_core::native_states::status::PASSIVE
                    ..=kirby_assist_core::native_states::status::PASSIVE_CEIL)
                    .contains(&assessment.status)
                || (kirby_assist_core::native_states::status::CATCHED_GANON
                    ..=kirby_assist_core::native_states::status::CATCHED_AIR_END_GANON)
                    .contains(&assessment.status);
            let key = (assessment.status, assessment.situation, assessment.reason);
            if relevant && state.last_tech_diagnostic != Some(key) {
                state.last_tech_diagnostic = Some(key);
                state.pending_tech_log = Some(format!("tech observation: input_update={tick}, status={:#x}, situation={}, reason={}, velocity=({:.2},{:.2}), lookahead={:.1}", assessment.status, assessment.situation, assessment.reason, assessment.velocity[0], assessment.velocity[1], config.lookahead));
                if let Some(message) = state.pending_tech_log.as_mut() {
                    message.push_str(&format!(
                        ", contact_in={:?}, hitstop_remaining={}, input_delay={input_delay:.1}, capture_clock={:?}",
                        assessment.contact_in, assessment.remaining_stop, assessment.capture_clock
                    ));
                }
            } else if !relevant {
                state.last_tech_diagnostic = None;
            }
        }
        // During hitstop, publish neutral assist without resetting the policy's
        // cooldown/history. Mapping polls are not advancing the damage clock.
        let policy_tick = state
            .animation_clock
            .get_or_insert_with(AnimationClock::default)
            .observe(
                AnimationSample {
                    object: object_id,
                    status: assessment.status,
                    kind: own_motion_kind,
                    frame: ledge_motion_frame,
                    auxiliary: if clatter { clatter_remaining } else { 0.0 },
                },
                stopped,
            );
        state.policy_tick = policy_tick;
        if policy_tick.is_none() {
            if let Some(policy) = state.policy.as_mut() {
                policy.pause_inputs();
            }
        }
        let auto_mash = state
            .settings_menu
            .as_ref()
            .map_or(config.controls.auto_mash, |menu| {
                menu.auto_mash(config.controls.auto_mash)
            });
        let mash_requested = if let Some(policy_tick) = policy_tick {
            state
                .policy
                .get_or_insert_with(RawAutoControls::default)
                .update_with_buttons(
                    kirby_assist_core::live_input::LiveSettings {
                        auto_tech: false,
                        auto_mash,
                        ..config.controls
                    },
                    policy_tick,
                    object_id,
                    clatter,
                    false,
                    previous_buttons,
                )
        } else {
            0
        };
        let previous_guard = state.last_mapped_guard;
        state.tech_output = state
            .tech_input
            .get_or_insert_with(TechInput::default)
            .sample_with_native_delivery(
                config.controls.enabled && auto_tech_enabled,
                true,
                tech_timing.0,
                TechContext {
                    object: object_id,
                    status: assessment.status,
                    motion: assessment
                        .capture_clock
                        .map_or(own_motion_kind, |clock| clock.0),
                    frame: assessment
                        .capture_clock
                        .map_or(ledge_motion_frame, |clock| clock.1),
                    rate: assessment
                        .capture_clock
                        .map_or(timing_rate, |clock| clock.2),
                    stopped,
                    remaining_stop: assessment.remaining_stop,
                    contact_in: if !clatter && tech {
                        assessment.contact_in
                    } else {
                        None
                    },
                    input_delay: input_delay,
                    previous_guard,
                    prepare_guard: !clatter && assessment.prepare_guard,
                },
                assessment.reaction_remaining,
                tech_timing.1,
                tech_timing.2,
                native_tech_guard,
            );
        state.tech_urgent = state
            .tech_input
            .as_ref()
            .is_some_and(TechInput::urgent_press);
        let _requested = mash_requested | if state.tech_output.press { HID_ZR } else { 0 };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let cooldown = state
                .tech_input
                .as_ref()
                .map_or(0.0, TechInput::cooldown_remaining);
            if let Some(message) = state.pending_tech_log.as_mut() {
                message.push_str(&format!(", requested={_requested:#x}, cooldown={cooldown}"));
            }
        }
        // Cap grounded physical inputs during recovery too, preventing a held
        // full stick from buffering a dash as the attack finishes. Damage/air
        // movement and higher-priority mash/tech keep their physical inputs.
        state.ledge_walk_ready = !clatter
            && !tech
            && assessment.situation == kirby_assist_core::native_states::situation::GROUND
            && (0..=kirby_assist_core::native_states::THROW).contains(&assessment.status);
        #[cfg(feature = "diagnostic-logging")]
        {
            state.mode_hit = ledge_observation
                .as_ref()
                .map(|(context, _, _, _)| context.hit_confirmed);
        }
        state.manual_context = Some(manual_context);
        state.posture_target = posture_target;
        state.posture_floor_safe = posture_floor_safe;
        state.turbo_ready = !clatter
            && !tech
            && turbo_movement_allowed(assessment.status, assessment.situation, stopped);

        if let Some((mut context, opponent, edge, spot)) = ledge_observation {
            state.ledge_walk_motion = state
                .ledge_walk_ready
                .then_some((context.position, context.velocity));
            let previewing = state
                .settings_menu
                .as_ref()
                .is_some_and(|m| m.preview_slot().is_some());
            context.input_delay = 0.0;
            context.can_act = state.turbo_ready;
            let trump_held = state
                .settings_menu
                .as_ref()
                .is_some_and(|menu| menu.ledge_trump_held());
            if !trump_held {
                state.trump_seek_tick = None;
            } else if find_trump_edge
                && turbo_movement_allowed(context.status, context.situation, stopped)
                && !clatter
                && !tech
            {
                // A failed geometry search must not repeat hundreds of rays
                // on every held-button poll. A new press still searches now.
                state.trump_seek_tick = Some(tick);
            }
            let trump_allowed = state.action_owner.allows(Owner::Trump);
            state.trump_output = state.trump.get_or_insert_with(LedgeTrump::default).sample(
                trump_allowed,
                tick,
                trump_held,
                GameplayContext {
                    gameplay: !clatter && !tech,
                    ..context
                },
                edge,
            );
            let held = state.ledge_active && state.action_owner.allows(Owner::Ledge);
            // Preparation stays grounded. Continue only a policy-started hop and
            // keep ownership through attack/shield/grab animations; mash/tech keep their existing priority.
            // The policy can preserve ZR during native guard hitstop.
            let grounded = assessment.situation
                == kirby_assist_core::native_states::situation::GROUND
                && (state.turbo_ready
                    || matches!(assessment.status, kirby_assist_core::native_states::status::GUARD_ON..=kirby_assist_core::native_states::status::GUARD_DAMAGE | kirby_assist_core::native_states::status::ATTACK_DASH..=kirby_assist_core::native_states::status::ATTACK_LW4 | kirby_assist_core::native_states::status::CATCH..=kirby_assist_core::native_states::status::CATCH_ATTACK));
            let owned_jump = state
                .ledge
                .as_ref()
                .is_some_and(|trap| trap.owns_jump_state(assessment.status));
            let can_control = !stopped && !clatter && !tech && (grounded || owned_jump);
            // can_act permits a new grounded action. can_control also covers
            // our ongoing shield/tilt/dash attack/grab, allowing ownership without repeatedly
            // launching actions during recovery. Do not collapse these flags.
            context.held = held;
            context.can_act = state.turbo_ready;
            context.can_control = can_control;
            if context.situation != kirby_assist_core::native_states::situation::GROUND {
                state.ledge_position = None;
                state.ledge_seek_tick = None;
            }
            if held
                && !previewing
                && context.situation == kirby_assist_core::native_states::situation::GROUND
                && config.assist.auto_position
                && !config.assist.manual_spacing
                && !config.assist.distance_agnostic
            {
                if find_trap_edge && state.turbo_ready && !stopped && !clatter && !tech {
                    state.ledge_seek_tick = Some(tick);
                }
                state
                    .ledge_position
                    .get_or_insert_with(LedgePosition::default)
                    .lock(spot);
                if let Some(spot) = state
                    .ledge_position
                    .as_ref()
                    .and_then(LedgePosition::spot)
                    .filter(|_| !config.assist.manual_spacing && !config.assist.distance_agnostic)
                {
                    state
                        .ledge
                        .get_or_insert_with(LedgeTrap::default)
                        .prepare_stance(spot.edge, spot.inward);
                }
            }
            state.ledge_output = {
                let state = &mut *state;
                let before = state.ledge_visits.visit_epoch();
                let output = state
                    .ledge
                    .get_or_insert_with(LedgeTrap::default)
                    .sample_with_visits(
                        &mut state.ledge_visits,
                        config.assist,
                        tick,
                        context,
                        opponent,
                    );
                if state.ledge_visits.visit_epoch() != before {
                    // A new ledge visit invalidates the previous approach shield
                    // tail, even when our stationary guard clock cannot advance.
                    state.guard_hold = None;
                    state.autopilot_guarding = false;
                }
                output
            };
            // Completion hands ownership back before the general reaction pass.
            // Keep the final emitted input protected until its frame is delivered.
            let reserve = state.ledge_active
                && state.ledge.as_ref().is_some_and(|trap| {
                    trap.needs_response_observation() || state.ledge_output.owned != 0
                });
            state.action_owner.set_ledge_priority(reserve);
            if !reserve {
                state.action_owner.release(Owner::Ledge);
            }
            if previewing
                && state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::manual_after_response)
            {
                if let Some(menu) = state.settings_menu.as_mut() {
                    menu.finish_preview();
                }
            }
            // Keep observing options during approach. An actionable opponent
            // response takes priority immediately; hanging/no-target uses the
            // locked, damped preparation controller instead of root chasing.
            #[cfg(feature = "diagnostic-logging")]
            let mut positioning_action = None;
            if held
                && !previewing
                && !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::file_timeline_active)
                && !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::manual_after_response)
                && !config.assist.manual_spacing
                && !config.assist.distance_agnostic
                && context.situation == kirby_assist_core::native_states::situation::GROUND
                && !clatter
                && !tech
            {
                let preparing = !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::owns_roll_navigation)
                    && (state.ledge_output.owned == 0
                        || state.ledge.as_ref().is_some_and(|trap| {
                            matches!(trap.action_name(), "finished" | "return_to_ledge")
                        })
                        || opponent.is_some_and(|e| {
                            (CLIFF_CATCH_MOVE
                                ..=kirby_assist_core::native_states::status::CLIFF_WAIT)
                                .contains(&e.status)
                        }));
                if preparing {
                    let movement = state
                        .ledge_position
                        .get_or_insert_with(LedgePosition::default)
                        .sample(tick, context, config.assist);
                    #[cfg(feature = "diagnostic-logging")]
                    {
                        positioning_action = state
                            .ledge_position
                            .as_ref()
                            .map(LedgePosition::action_name);
                    }
                    if movement.owned != 0 {
                        state.ledge_output = movement;
                    }
                }
            }
            // Guard all incoming options while watching/after a response.
            // Committed authored timelines keep their literal controls.
            // Never replace a requested punish input, committed move, capture
            // handback, trump, or a higher-priority mash/tech response.
            let passive_plan = state
                .ledge
                .as_ref()
                .is_none_or(|trap| !trap.committed_input_pending());
            let defense_eligible = state.action_owner.allows(Owner::Defense)
                && held
                && config.autopilot_shield
                && passive_plan
                && context.situation == kirby_assist_core::native_states::situation::GROUND
                && !clatter
                && !tech
                && state.ledge_output.requested == 0
                && state.roll_read_output.owned == 0
                && state.trump_output.owned == 0;
            let guard_eligible = defense_eligible && guard_ready;
            let defensive_guard = state
                .guard_hold
                .get_or_insert_with(kirby_assist_core::autopilot_guard::GuardHold::default)
                .sample_with_progress(
                    guard_eligible,
                    guard_threat,
                    kirby_assist_core::autopilot_guard::GuardClock {
                        status: context.status,
                        motion: context.motion_kind,
                        frame: context.motion_frame,
                        rate: context.motion_rate,
                        stopped,
                    },
                    analysis_settings.value(kirby_assist_core::fighter_tuning::SHIELD_HOLD, 4.),
                    if guard_eligible
                        && matches!(
                            context.status,
                            kirby_assist_core::native_states::GUARD_ON
                                | kirby_assist_core::native_states::GUARD
                        )
                    {
                        reads.known_guard_clock(object_id)
                    } else {
                        None
                    },
                );
            state.autopilot_guarding = defensive_guard;
            if defensive_guard {
                state.ledge_output = ControllerOutput {
                    consume_trigger: true,
                    requested: MAPPER_ZR,
                    owned: kirby_assist_core::native_input::ASSIST_BUTTONS,
                    stick: Some([0.0, 0.0]),
                    native_short_hop: false,
                };
            }
            #[cfg(feature = "diagnostic-logging")]
            {
                let state = &mut *state;
                let metadata = state.ledge.as_ref().and_then(|trap| {
                    let slot = trap.calibration_slot()?;
                    let profile = PROFILES.get()?.profile(state.fighter_kind?)?;
                    let (id, name) = state
                        .settings_menu
                        .as_ref()?
                        .selected_choice_labels(fighter_kind, slot)?;
                    let mut frame = trap.calibration_frame(config.assist, context, opponent, id)?;
                    frame.output = state.ledge_output;
                    frame.native_attack_active = neutral_native_activity;
                    Some((
                        frame,
                        profile.name,
                        kirby_assist_core::diagnostics::modes::Label::new(
                            kirby_assist_core::fighter_profiles::spacing::menu_name(name),
                        ),
                    ))
                });
                if held {
                    if let Some((frame, fighter, choice)) = metadata {
                        state.calibration.observe(
                            Some(frame),
                            (fighter, fighter_kind),
                            choice.as_str(),
                            now,
                        );
                    } else {
                        state.calibration.observe(None, ("", fighter_kind), "", now);
                    }
                } else {
                    state.calibration.reset("autopilot released/other mode");
                }
            }
            #[cfg(feature = "diagnostic-logging")]
            let grab_snapshot = if held {
                state
                    .ledge
                    .as_ref()
                    .and_then(LedgeTrap::roll_grab_diagnostic)
            } else {
                None
            };
            #[cfg(feature = "diagnostic-logging")]
            {
                let fighter = state.fighter_kind;
                if !held {
                    state.grab_trace.reset("autopilot_released");
                } else {
                    state.grab_trace.observe(grab_snapshot, fighter, now);
                }
            }
            #[cfg(feature = "diagnostic-logging")]
            if held && grab_snapshot.is_none() {
                let key = LedgeDiagnostic {
                    target: opponent.map(|enemy| enemy.id),
                    enemy_status: opponent.map_or(-1, |enemy| enemy.status),
                    vulnerable: opponent.is_some_and(|enemy| enemy.vulnerable),
                    kirby_status: assessment.status,
                    can_act: context.can_act,
                    can_grab: context.can_grab,
                    can_dash_attack: context.can_dash_attack,
                    can_jump: context.can_jump,
                    can_air_attack: context.can_air_attack,
                    hit_confirmed: context.hit_confirmed,
                    own_motion: own_motion_kind,
                    requested: state.ledge_output.requested,
                    action: if state.autopilot_guarding {
                        "autopilot_defensive_shield"
                    } else {
                        positioning_action.unwrap_or_else(|| {
                            state.ledge.as_ref().map_or("none", LedgeTrap::action_name)
                        })
                    },
                };
                let origin = *state.ledge_log_origin.get_or_insert(now);
                let elapsed_ms = now
                    .saturating_duration_since(origin)
                    .as_millis()
                    .min(u128::from(u64::MAX)) as u64;
                if let Some(emission) = state.ledge_diagnostics.sample_throttled(
                    key,
                    elapsed_ms,
                    config.controls.tech_debug,
                    125,
                ) {
                    state.pending_ledge_log = Some(format!(
                        concat!(
                            "ledge observation: input_update={tick}, policy_tick={policy_tick:?}, ",
                            "periodic={}, coalesced_updates={}, ",
                            "target={:?}, enemy_status={:#x}, vulnerable={}, ",
                            "kirby_status={:#x}, own_motion={:#x}, motion_frame={:.2}, ",
                            "can_act={}, requested={:#x}, action={}, situation={}, ",
                            "previous_status={:#x}, can_grab={}, can_dash_attack={}, hit_confirmed={}, ",
                            "can_jump={}, can_air_attack={}, air_physics={:?}, short_hop_speed={:?}, ",
                            "shield_stun={:?}, safe_ground={:?}, position={:?}, ",
                            "target_position={:?}, target_vulnerable_in={:?}, ",
                            "target_previous_status={:?}, target_motion={:?}, target_actionable_in={:?}, ",
                            "target_observed_velocity={:?}, file_options={:?}, ",
                            "locked_ledge={:?}, stance_distance={}"
                        ),
                        emission.periodic,
                        emission.coalesced_updates,
                        key.target,
                        key.enemy_status,
                        key.vulnerable,
                        key.kirby_status,
                        own_motion_kind,
                        ledge_motion_frame,
                        key.can_act,
                        key.requested,
                        key.action,
                        assessment.situation,
                        context.previous_status,
                        context.can_grab,
                        context.can_dash_attack,
                        context.hit_confirmed,
                        context.can_jump,
                        context.can_air_attack,
                        context.air_physics,
                        context.short_hop_speed,
                        context.shield_stun_frames,
                        context.safe_ground,
                        context.position,
                        opponent.map(|enemy| enemy.position),
                        opponent.and_then(|enemy| enemy.vulnerable_in),
                        opponent.map(|enemy| enemy.previous_status),
                        opponent.and_then(|enemy| enemy.motion),
                        opponent.and_then(|enemy| enemy.actionable_in),
                        state
                            .ledge
                            .as_ref()
                            .map(LedgeTrap::observed_target_velocity),
                        config.assist.sequence_overrides.iter().any(Option::is_some),
                        state.ledge_position.as_ref().and_then(LedgePosition::spot),
                        config.assist.stand_distance,
                        tick = tick,
                        policy_tick = policy_tick,
                    ));
                }
            } else {
                state.ledge_diagnostics = DiagnosticGate::new();
                state.ledge_log_origin = None;
                state.pending_ledge_log = None;
            }
        } else {
            #[cfg(feature = "diagnostic-logging")]
            state.grab_trace.reset("ledge_observation_unavailable");
            #[cfg(feature = "diagnostic-logging")]
            state.calibration.reset("ledge observation unavailable");
            state.autopilot_guarding = false;
            state.guard_hold = None;
            state.ledge_output = ControllerOutput::default();
            state.trump_output = ControllerOutput::default();
            state.trump = None; // Clear released Down history without native reads.
            state.trump_seek_tick = None;
        }
        state.reflect_output = if let Some((configured, context, threat)) = reflect_observation {
            // Preparation happens after existing modes. Never admit two new
            // punish inputs on the same observation, nor steal their receipt.
            let allowed = (!state.ledge_active
                || state.action_owner.current_owner() == Some(Owner::Reflect))
                && state.action_owner.allows(Owner::Reflect)
                && !clatter
                && !tech
                && state.oos_output.owned == 0
                && !state.oos.grab_delivery_pending()
                && state.roll_read_output.owned == 0
                && state.spot_output.owned == 0
                && state.trip_output.owned == 0
                && state.fair_output.owned == 0
                && state.trump_output.owned == 0;
            state
                .reflector
                .sample(allowed, Some(configured), context, threat)
        } else {
            state.reflector.cancel();
            ControllerOutput::default()
        };
        state.mash_stick = if policy_tick.is_none() {
            None
        } else {
            state.policy.as_ref().and_then(RawAutoControls::mash_stick)
        };
        state.request_seen = Some(now);
        #[cfg(feature = "diagnostic-logging")]
        if state.last_logged_scenario != (clatter, tech) {
            state.last_logged_scenario = (clatter, tech);
            scenario_changed = true;
        }
    }
    #[cfg(feature = "diagnostic-logging")]
    if first_button_write {
        log!("raw controller mapper button overlay confirmed; game acceptance still depends on its input rules");
    }
    #[cfg(feature = "diagnostic-logging")]
    if acquired {
        log!(&format!(
            "observing human fighter object {object_id:#x}, input index {index}"
        ));
    }
    #[cfg(feature = "diagnostic-logging")]
    if scenario_changed {
        log!(&format!(
            "scenario: mashable={clatter}, predicted_tech_collision={tech}"
        ));
    }
}
