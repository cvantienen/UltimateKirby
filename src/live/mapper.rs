use super::*;

unsafe fn refresh_master_for_controller(
    wrapper: *mut c_void,
    index: i32,
    config: &Config,
) -> Option<Option<bool>> {
    selected_controller(wrapper, index, config.player_index)?;
    let ready = observations::battle_menu_ready_go();
    refresh_master_match_lock(ready);
    Some(ready)
}

/// Represents the admission state for the mapper callback, including configuration, readiness, and session ticket.
pub(super) struct MapperAdmission {
    pub(super) config: &'static Config,
    pub(super) master_ready_go: Option<Option<bool>>,
    pub(super) session_ticket: SessionTicket,
}
/// Represents the output adjustments for the mapper callback, including stick input, automatic requests, and other control flags.
struct MapperOutputAdjustments {
    left_stick: Option<[f32; 2]>,
    right_stick: Option<[f32; 2]>,
    automatic_request: Option<u32>,
    short_hop: bool,
    attack_hold: bool,
    literal_request: Option<u32>,
    unshielded_auto_grab: bool,
    tech_output: Option<TechOutput>,
}

/// Represents the feedback delivered to the mapper callback, including manual and automatic actions, grab delivery, and other relevant events.
#[derive(Clone, Copy)]
struct MapperDeliveryFeedback {
    manual_jump: bool,
    reflect: bool,
    reflector_tick: Option<i64>,
    auto_grab: Option<(i64, u32, bool)>,
    grab_handoff: Option<(i64, Option<Owner>, bool, bool)>,
    dodge_punish_grab: Option<(i64, u32)>,
    auto_dodge: Option<(i64, u32)>,
    auto_up_special: Option<(i64, u32, Owner)>,
    tech_applied: bool,
    #[cfg(feature = "diagnostic-logging")]
    grab_delivery: Option<(u32, u32, u32, &'static str)>,
}
/// Admits the mapper callback for the specified controller, returning the admission state if successful.
pub(super) unsafe fn admit_mapper_callback(
    wrapper: *mut c_void,
    index: i32,
) -> Option<MapperAdmission> {
    let config = CONFIG.get().map(Box::as_ref)?;
    if !config.controls.enabled {
        return None;
    }

    // Keep the read-only battle gate and bounded menu maintenance active in
    // vanilla passthrough, before any mapper state lock.
    let master_ready_go = refresh_master_for_controller(wrapper, index, config);
    settings_menu::poll_maintenance();
    if !RUNTIME_ENABLED.load(Ordering::Acquire) {
        return None;
    }

    // Only the configured physical controller may establish participation.
    let controller = selected_controller(wrapper, index, config.player_index)?;
    if (*controller).connected == 0 {
        runtime::suspend_participation();
        return None;
    }
    let session_ticket = live_match_ticket()?;
    let candidate = runtime::mapper_candidate(index);
    if !candidate.is_some_and(|id| observations::valid_local_route(id, index)) {
        runtime::suspend_participation();
        return None;
    }

    LOCAL_PARTICIPANT.store(true, Ordering::Release);
    Some(MapperAdmission {
        config,
        master_ready_go,
        session_ticket,
    })
}
/// Enters the training passthrough mode for the specified controller and configuration, returning `true` if successful.
pub(super) unsafe fn enter_training_passthrough(
    wrapper: *mut c_void,
    index: i32,
    config: &Config,
) -> bool {
    let selected_physical = CONFIG
        .get()
        .and_then(|config| selected_controller(wrapper, index, config.player_index))
        .map(|controller| trace(controller));
    let Some(physical) = selected_physical else {
        return true;
    };
    training::enter_passthrough(physical, index, config.control_layout)
}

/// Applies the specified output adjustments to the mapper output, modifying stick input, automatic requests, and other control flags.
unsafe fn apply_mapper_output_adjustments(
    output: *mut c_void,
    adjustments: MapperOutputAdjustments,
) {
    let mapped = output.cast::<MappedInputs>();
    // Preserve selected analog input after the native mapper's flick filtering.
    if let Some(stick) = adjustments.left_stick {
        (*mapped).left_x = kirby_assist_core::native_input::mapped_stick_axis(stick[0]);
        (*mapped).left_y = kirby_assist_core::native_input::mapped_stick_axis(stick[1]);
    }
    if let Some(right) = adjustments.right_stick {
        (*mapped).buttons =
            kirby_assist_core::native_input::mapped_sequence_cstick((*mapped).buttons, right);
        (*mapped).right_x = kirby_assist_core::native_input::mapped_stick_axis(right[0]);
        (*mapped).right_y = kirby_assist_core::native_input::mapped_stick_axis(right[1]);
    }
    if let Some(requested) = adjustments.automatic_request {
        (*mapped).buttons =
            kirby_assist_core::native_input::mapped_automatic_actions((*mapped).buttons, requested);
    }
    (*mapped).buttons =
        kirby_assist_core::native_input::mapped_short_hop((*mapped).buttons, adjustments.short_hop);
    (*mapped).buttons =
        kirby_assist_core::native_input::mapped_short_hop((*mapped).buttons, adjustments.short_hop);
    // Raw trigger history can lose A even while retaining the stick.
    (*mapped).buttons = kirby_assist_core::native_input::mapped_attack_hold(
        (*mapped).buttons,
        adjustments.attack_hold,
    );
    if let Some(requested) = adjustments.literal_request {
        (*mapped).buttons =
            kirby_assist_core::native_input::mapped_literal_buttons((*mapped).buttons, requested);
        (*mapped).buttons = kirby_assist_core::native_input::mapped_unshielded_auto_grab(
            (*mapped).buttons,
            requested,
            adjustments.unshielded_auto_grab,
        );
    }
    if let Some(tech) = adjustments.tech_output {
        (*mapped).buttons = kirby_assist_core::tech_input::mapped_guard((*mapped).buttons, tech);
    }
}
/// Records a manual jump input for the given state and mapped inputs, updating the manual jump fence if applicable.
fn record_mapped_manual_jump(state: &mut State, mapped: &MappedInputs, manual_jump: bool) {
    if manual_jump
        && mapped.buttons
            & (kirby_assist_core::native_input::MAPPED_JUMP
                | kirby_assist_core::native_input::MAPPED_JUMP_MINI)
            != 0
    {
        if let (Some(object), Some(kind), Some(context)) = (
            state.associated_object,
            state.fighter_kind,
            state.manual_context,
        ) {
            state.manual_jump_fence.sample(
                object,
                kind,
                true,
                GameplayContext {
                    gameplay: true,
                    status: context.status,
                    situation: context.situation,
                    motion_kind: context.motion,
                    motion_frame: context.frame,
                    motion_rate: context.rate,
                    stopped: context.stopped,
                    ..GameplayContext::default()
                },
            );
        }
    }
}
/// Records a mapped reflector input for the given state and mapped inputs, handling reflector delivery and receipt polling.
fn record_mapped_reflector(
    state: &mut State,
    mapped: &MappedInputs,
    reflect: bool,
    reflector_tick: Option<i64>,
) {
    if reflect
        && mapped.buttons & kirby_assist_core::native_input::MAPPED_SPECIAL != 0
        && mapped.left_y < -63
    {
        let tick = state.raw_tick;
        state.reflector.mapped(tick);
    }
    if let Some(tick) = reflector_tick {
        state.reflector.poll_receipt(tick);
        if !state.reflector.active() {
            state.reflect_output = ControllerOutput::default();
            state.action_owner.release(Owner::Reflect);
        }
    }
}
/// Records mapped grab inputs for the given state and mapped inputs, handling automatic grabs and grab handoff events.
fn record_mapped_grabs(
    state: &mut State,
    mapped: &MappedInputs,
    auto_grab: Option<(i64, u32, bool)>,
    grab_handoff: Option<(i64, Option<Owner>, bool, bool)>,
) {
    if let Some((tick, target, grab)) = auto_grab {
        if grab {
            state.oos.mapped_grab(tick, target);
        } else {
            state.oos.mapped_approach(tick, target);
        }
    }
    if let Some((tick, owner, grab, eligible)) = grab_handoff {
        state.grab_handoff.mapped(
            tick,
            owner,
            grab,
            mapped.buttons & kirby_assist_core::native_input::MAPPED_CATCH != 0,
            eligible,
        );
    }
}
/// Records mapped dodge inputs for the given state and mapped inputs, handling guard, punish grabs, and automatic dodges.
fn record_mapped_dodges(
    state: &mut State,
    mapped: &MappedInputs,
    guard: bool,
    punish_grab: Option<(i64, u32)>,
    auto_dodge: Option<(i64, u32)>,
) {
    let delivery_tick = state.raw_tick;
    if let Some(spot) = state.spot_dodge.as_mut() {
        if let Some((tick, target)) = punish_grab {
            spot.mapped_grab(tick, target);
        }
        spot.delivery_tick(delivery_tick);
    }
    if let Some((tick, target)) = auto_dodge {
        if let Some(spot) = state.spot_dodge.as_mut() {
            spot.attempted_dodge(tick, target);
            // Stop pressing only after actual Guard + Down delivery; failed
            // packets retain their bounded retry budget.
            if guard && mapped.left_y < 0 {
                spot.mapped_dodge(tick, target);
            }
        }
    }
}

/// Records mapped up special inputs for the given state and mapped inputs, handling out-of-shield, roll read, and spot dodge scenarios.
fn record_mapped_up_special(
    state: &mut State,
    mapped: &MappedInputs,
    request: Option<(i64, u32, Owner)>,
) {
    let Some((tick, target, owner)) = request else {
        return;
    };
    // Rejected packets spend delivery attempts but cannot arm a delayed
    // receipt. Confirmation requires final Special + Up.
    let delivered =
        kirby_assist_core::native_input::mapped_up_special_delivered(mapped.buttons, mapped.left_y);
    match owner {
        Owner::OutOfShield => {
            state.oos.attempted_up_special(tick, target);
            if delivered {
                state.oos.mapped_up_special(tick, target);
            }
        }
        Owner::RollRead => {
            if let Some(engine) = state.rule_engine.as_mut() {
                engine.attempted_up_special(tick, target);
                if delivered {
                    engine.mapped_up_special(tick, target);
                }
            }
        }
        Owner::OptionRead => {
            if let Some(spot) = state.spot_dodge.as_mut() {
                spot.attempted_up_special(tick, target);
                if delivered {
                    spot.mapped_up_special(tick, target);
                }
            }
        }
        _ => {}
    }
}

/// Records mapped tech inputs for the given state, handling guard and tech application scenarios.
fn record_mapped_tech(
    state: &mut State,
    guard: bool,
    tech_applied: bool,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    if tech_applied && guard && !state.last_mapped_guard {
        if let Some(tech) = state.tech_input.as_mut() {
            tech.mapped_press();
        }
        #[cfg(feature = "diagnostic-logging")]
        if CONFIG
            .get()
            .is_some_and(|config| config.controls.tech_debug)
        {
            messages
                .push("tech mapped: fresh GUARD edge confirmed; request cooldown committed".into());
        }
    }
    if !tech_applied && guard && !state.last_mapped_guard {
        if let Some(tech) = state.tech_input.as_mut() {
            tech.mapped_human_guard();
        }
    }
}

#[cfg(feature = "diagnostic-logging")]
fn record_mapped_grab_diagnostics(
    state: &mut State,
    mapped: &MappedInputs,
    grab_delivery: Option<(u32, u32, u32, &'static str)>,
) {
    if let Some((proposed, applied, pressed, gate)) = grab_delivery {
        state.grab_trace.mapped(
            proposed,
            applied,
            pressed,
            mapped.buttons,
            [mapped.left_x, mapped.left_y],
            gate,
            Instant::now(),
        );
        state
            .calibration
            .mapped(proposed, applied, mapped.buttons, gate);
    }
}

/// Records mapped grab diagnostics for the given state and mapped inputs, handling grab delivery events.
unsafe fn record_mapped_delivery(
    output: *mut c_void,
    index: i32,
    session_ticket: SessionTicket,
    feedback: MapperDeliveryFeedback,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    let mapped = &*output.cast::<MappedInputs>();
    let guard = mapped.buttons
        & (kirby_assist_core::native_input::MAPPED_GUARD
            | kirby_assist_core::native_input::MAPPED_GUARD_HOLD)
        != 0;
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    if state.mapping_index != Some(index) || !state.session.accepts(session_ticket) {
        return;
    }

    record_mapped_manual_jump(&mut state, mapped, feedback.manual_jump);
    record_mapped_reflector(
        &mut state,
        mapped,
        feedback.reflect,
        feedback.reflector_tick,
    );
    record_mapped_grabs(
        &mut state,
        mapped,
        feedback.auto_grab,
        feedback.grab_handoff,
    );
    let delivery_tick = state.raw_tick;
    state.oos.delivery_tick(delivery_tick);
    if let Some(engine) = state.rule_engine.as_mut() {
        engine.delivery_tick(delivery_tick);
    }
    record_mapped_dodges(
        &mut state,
        mapped,
        guard,
        feedback.dodge_punish_grab,
        feedback.auto_dodge,
    );
    record_mapped_up_special(&mut state, mapped, feedback.auto_up_special);
    record_mapped_tech(
        &mut state,
        guard,
        feedback.tech_applied,
        #[cfg(feature = "diagnostic-logging")]
        messages,
    );
    state.last_mapped_guard = guard;
    state.last_mapped_stick = Some([mapped.left_x as f32 / 127., mapped.left_y as f32 / 127.]);
    #[cfg(feature = "diagnostic-logging")]
    record_mapped_grab_diagnostics(&mut state, mapped, feedback.grab_delivery);
}
/// Updates the mapper menu and input state for the specified controller, handling menu readiness, battle activity, and diagnostic logging.
unsafe fn update_mapper_menu_and_input(
    controller: *mut controller::Controller,
    index: i32,
    config: &Config,
    physical: kirby_assist_core::native_input::ButtonTrace,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
    menu_ready_go: Option<bool>,
    menu_training_mode: bool,
    training_test_mode: bool,
    #[cfg(feature = "diagnostic-logging")] trump_pressed: &mut bool,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    if let Ok(mut state) = STATE.lock() {
        let intro_current = state.menu_intro_object.is_some_and(|intro| {
            state.human_routes[index as usize]
                .is_some_and(|(id, seen)| id == intro && seen.elapsed() < SESSION_OBSERVATION_GAP)
        });
        let local_fighter_current = (*controller).connected != 0
            && state.human_routes[index as usize].is_some_and(|(id, seen)| {
                Some(id) == state.associated_object && seen.elapsed() < SESSION_OBSERVATION_GAP
            });
        let menu_battle_active =
            local_fighter_current && menu_ready_go.is_some_and(|go| go || intro_current);
        if menu_ready_go != Some(false) {
            state.menu_intro_object = None;
        }
        if state.menu_battle_active != menu_battle_active {
            state.menu_battle_active = menu_battle_active;
            if menu_battle_active {
                #[cfg(feature = "imgui-menu")]
                state
                    .settings_menu
                    .get_or_insert_with(settings_menu::SettingsMenu::default)
                    .begin_match();
            } else {
                attack_sources::clear();
                state.menu_fighter_kind = None;
                if let Some(settings_menu) = state.settings_menu.as_mut() {
                    settings_menu.end_match();
                }
            }
        }
        if (*controller).connected == 0 {
            state.cancel();
            state.associated_object = None;
            state.mapping_index = None;
        } else {
            if state.mapping_index != Some(index) {
                state.cancel();
                state.associated_object = None;
                state.mapping_index = Some(index);
                #[cfg(feature = "diagnostic-logging")]
                {
                    state.mapper_confirmed = false;
                    messages.push(format!(
                        "selected local controller {} bound to mapper {index} (npad={}, style={})",
                        config.player_index,
                        (*controller).npad,
                        (*controller).style
                    ));
                }
            }
            #[cfg(feature = "diagnostic-logging")]
            if !state.mapper_confirmed {
                state.mapper_confirmed = true;
            }
            // A new mapper call advances our local scheduling tick.
            // Other hooks can run several times within this tick;
            // evaluated_tick prevents them consuming extra pulses.
            let now = Instant::now();
            state.raw_tick = state.raw_tick.wrapping_add(1);
            state.raw_seen = Some(now);
            state.raw_connected = true;
            let policy_buttons = if training_test_mode {
                training_test::policy_buttons(physical.held)
            } else {
                physical.held
            };
            let takeover_buttons = config.control_layout.takeover_buttons(policy_buttons);
            let policy_buttons = config.control_layout.policy_buttons(policy_buttons);
            state.raw_buttons = policy_buttons;
            state.action_owner.observe_manual(
                takeover_buttons,
                [
                    physical_stick[0],
                    physical_stick[1],
                    physical_right_stick[0],
                    physical_right_stick[1],
                ],
            );
            state.posture_down = kirby_assist_core::posture::Posture::any_tilt_intent(
                policy_buttons,
                physical_stick,
                physical_right_stick,
            );
            if physical.held & MAPPER_PLUS != 0 {
                state.cancel();
            }
            {
                let kind = state.menu_fighter_kind;
                let menu_available = state.menu_battle_active && physical.held & MAPPER_PLUS == 0;
                let gameplay = menu_available && state.fighter_kind.is_some();
                // Raw button/menu observations still run, but a committed
                // input owner cannot be replaced by a queued button tape.
                let damage_recovery = state
                    .manual_context
                    .is_some_and(|c| kirby_assist_core::ledge_trap::sequence_interrupted(c.status));
                let committed_inputs = state.action_owner.current_owner().is_some()
                    || state.tech_output.press
                    || state.tech_output.owns_guard
                    || state.mash_stick.is_some();
                let bindings_blocked = committed_inputs || damage_recovery;
                let safety_jump = !committed_inputs && damage_recovery;
                let changed = state
                    .settings_menu
                    .get_or_insert_with(settings_menu::SettingsMenu::default)
                    .input(
                        menu_available,
                        gameplay,
                        bindings_blocked,
                        safety_jump,
                        kind,
                        policy_buttons,
                        if training_test_mode {
                            training_test::policy_buttons(physical.held)
                        } else {
                            physical.held
                        },
                        config.control_layout,
                        &config.assist,
                        config.controls.auto_mash,
                        config.auto_spot_dodge,
                        config.controls.auto_tech,
                        menu_training_mode,
                    );
                if changed {
                    state.cancel_manual_tools();
                }
                let consumed = state.settings_menu.as_ref().map_or(0, |p| p.consumed);
                if let Some(settings_menu) = state.settings_menu.as_ref() {
                    settings_menu.display_battle(state.menu_battle_active && kind.is_some());
                }
                // L remains an assist trigger; Up remains available to bindings
                // diagonals. Menu-consumed sides/Down never reach tools.
                // Preserve Y for the first native-readiness check;
                // the later overlay still consumes its mapped action.
                let helper_consumed = consumed
                    & !state
                        .settings_menu
                        .as_ref()
                        .map_or(0, |m| m.builtin_consumed);
                let policy_buttons = state
                    .settings_menu
                    .as_ref()
                    .map_or(policy_buttons, |m| m.native_buttons);
                state.raw_buttons = kirby_assist_core::settings_menu::observation_buttons(
                    policy_buttons,
                    helper_consumed,
                );
                let ledge_held = config.controls.enabled
                    && state
                        .settings_menu
                        .as_ref()
                        .is_some_and(|p| p.sequence_enabled(config.assist.enabled))
                    && state.settings_menu.as_ref().is_some_and(|p| p.ledge_held);
                attack_sources::tracking(
                    config.autopilot_shield && ledge_held
                        || !ledge_held
                            && state
                                .settings_menu
                                .as_ref()
                                .is_some_and(|m| m.reflect_config(kind.unwrap_or(-1)).is_some()),
                );
                let manual_actions = state
                    .settings_menu
                    .as_ref()
                    .map_or(0, |m| m.manual_native_actions());
                let ledge_inputs_reserved = ledge_held
                    || state
                        .settings_menu
                        .as_ref()
                        .is_some_and(|m| m.reserves_automatic_modes());
                let suspend_auto = state.action_owner.current_owner().is_none()
                    && (ledge_inputs_reserved
                        || manual_actions != 0
                        || kirby_assist_core::native_input::automatic_escape(
                            config.control_layout,
                            physical.held,
                            physical.pressed,
                            false,
                            state.manual_context.is_some_and(|c| {
                                c.situation == kirby_assist_core::native_states::situation::GROUND
                            }),
                            0,
                        ));
                state.action_owner.set_automatic_suspended(suspend_auto);
                if suspend_auto {
                    state.suspend_automatic_punishes();
                }
                if ledge_inputs_reserved
                    && state.action_owner.current_owner().is_none()
                    && state.grab_handoff.reserve_ledge_inputs()
                {
                    // A queued Grab must settle before unrelated playback;
                    // only native confirmation/expiry can release its receipt.
                    state.oos.cancel_pending();
                    if let Some(spot) = state.spot_dodge.as_mut() {
                        spot.cancel_response();
                    }
                }
                #[cfg(feature = "diagnostic-logging")]
                {
                    *trump_pressed = physical.pressed
                        & (kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS
                            | kirby_assist_core::native_input::MAPPER_X
                            | MAPPER_Y)
                        != 0
                        && state
                            .settings_menu
                            .as_ref()
                            .is_some_and(|m| m.ledge_trump_held());
                }
                if consumed & DPAD_SIDES != 0
                    || physical.held & (DPAD_UP | MAPPER_DPAD_DOWN | MAPPER_MINUS) != 0
                {
                    // Menu takeover cancels before a native refresh
                    // could replay a cached jump/dodge request.
                }
                if state.ledge_active != ledge_held {
                    if !ledge_held {
                        state.action_owner.release(Owner::Ledge);
                    }
                    #[cfg(feature = "diagnostic-logging")]
                    state.grab_trace.reset("autopilot_released_or_mode_changed");
                    #[cfg(feature = "diagnostic-logging")]
                    state.calibration.reset("autopilot released/mode changed");
                    state.ledge_active = ledge_held;
                    state.ledge_walk_motion = None;
                    state.autopilot_guarding = false;
                    state.guard_hold = None;
                    state.ledge = None;
                    state.ledge_position = None;
                    state.ledge_seek_tick = None;
                    state.ledge_output = ControllerOutput::default();
                }
                if state.ledge_active && state.ledge.is_none() {
                    if let Some(slot) = state.settings_menu.as_ref().and_then(|m| m.preview_slot())
                    {
                        let mut preview = LedgeTrap::default();
                        preview.request_preview(slot);
                        state.ledge = Some(preview);
                    }
                }
            }
        }
    }
}
/// Refreshes the selected fighter for the specified controller index, observing the fighter if valid or clearing the state if not.
unsafe fn refresh_selected_fighter(index: i32) {
    let candidate = STATE.lock().ok().and_then(|state| {
        (state.mapping_index == Some(index))
            .then_some(state.human_routes[index as usize])
            .flatten()
            .filter(|(_, seen)| seen.elapsed() < SESSION_OBSERVATION_GAP)
            .map(|(id, _)| id)
    });
    let Some(id) = candidate else {
        return;
    };
    let object = if game::is_active(id) {
        game::module_accessor(id)
    } else {
        std::ptr::null_mut()
    };
    let valid_route = !object.is_null()
        && game::battle_object_id(object) == id
        && controller_index(object.cast()) == Some(index)
        && human_fighter(object);
    if valid_route {
        observation::observe_fighter(object, false);
    } else if let Ok(mut state) = STATE.lock() {
        if state.mapping_index == Some(index) {
            LOCAL_PARTICIPANT.store(false, Ordering::Release);
            attack_sources::clear();
            state.cancel();
            state.associated_object = None;
            state.human_routes[index as usize] = None;
        }
    }
}

#[cfg(feature = "diagnostic-logging")]
type GrabMappingComparison = (
    bool,
    Option<Owner>,
    u32,
    kirby_assist_core::native_input::ButtonTrace,
    kirby_assist_core::native_input::ButtonTrace,
    [f32; 2],
    [f32; 2],
    [f32; 2],
    [f32; 2],
);
/// Represents the preparation state for the mapper,
/// including restored controller state, mapped inputs, and diagnostic information.
pub(super) struct MapperPreparation {
    pub(super) restore: Option<ControllerRestore>,
    mapped_left_stick: Option<[f32; 2]>,
    mapped_automatic_request: Option<u32>,
    mapped_auto_grab: Option<(i64, u32, bool)>,
    mapped_reflect: bool,
    mapped_reflector: Option<i64>,
    mapped_auto_up_special: Option<(i64, u32, Owner)>,
    mapped_auto_dodge: Option<(i64, u32)>,
    mapped_dodge_punish_grab: Option<(i64, u32)>,
    mapped_attack_hold: bool,
    mapped_literal_request: Option<u32>,
    mapped_unshielded_auto_grab: bool,
    mapped_sequence_right: Option<[f32; 2]>,
    reserved_dpad: u32,
    tech_applied: bool,
    mapped_tech_output: Option<TechOutput>,
    mapped_grab_handoff: Option<(i64, Option<Owner>, bool, bool)>,
    sequence_short_hop: bool,
    mapped_manual_jump: bool,
    #[cfg(feature = "diagnostic-logging")]
    trump_pressed: bool,
    #[cfg(feature = "diagnostic-logging")]
    ledge_edge: Option<u32>,
    #[cfg(feature = "diagnostic-logging")]
    grab_delivery: Option<(u32, u32, u32, &'static str)>,
    #[cfg(feature = "diagnostic-logging")]
    grab_mapping_comparison: Option<GrabMappingComparison>,
    #[cfg(feature = "diagnostic-logging")]
    messages: Vec<String>,
    #[cfg(feature = "diagnostic-logging")]
    mode_report: Option<kirby_assist_core::diagnostics::modes::Report>,
}

/// Represents the input flags for the mapper, indicating the current conditions of the controller and game state.
/// # Fields
/// * `fresh` - Indicates if the input is fresh.
/// * `manual_scope` - Indicates if manual scope is active.
/// * `mash_active` - Indicates if mash input is active.
/// * `damage_active` - Indicates if damage input is active.
/// * `turbo_active` - Indicates if turbo input is active.
/// * `pivot_active` - Indicates if pivot input is active.
/// * `posture_active` - Indicates if posture input is active.
struct MapperIntentInputs {
    fresh: bool,
    manual_scope: bool,
    mash_active: bool,
    damage_active: bool,
    turbo_active: bool,
    pivot_active: bool,
    posture_active: bool,
}
/// Computes the action intents for the mapper based on the current state and input flags. Returns an array of `Intent` for each possible action.
/// # Arguments
/// * `state` - The current state of the mapper.
/// * `inputs` - The input flags indicating the current conditions.
///
/// # Returns
/// An array of `Intent` for each possible action.
fn mapper_action_intents(state: &mut State, inputs: MapperIntentInputs) -> [Intent; 13] {
    let ledge_priority = state.action_owner.ledge_priority()
        && state.ledge_active
        && inputs.fresh
        && inputs.manual_scope
        && state
            .ledge
            .as_ref()
            .is_some_and(|trap| trap.needs_response_observation() || state.ledge_output.owned != 0);
    state.action_owner.set_ledge_priority(ledge_priority);
    let ledge_committed = state
        .ledge
        .as_ref()
        .is_some_and(LedgeTrap::committed_input_pending)
        || state.ledge_output.owned != 0 && !state.autopilot_guarding;
    let intents = [
        Intent {
            owner: Owner::Reflect,
            present: state.reflect_output.owned != 0 || state.reflector.active(),
            committed: true,
        },
        Intent {
            owner: Owner::OutOfShield,
            // Preserve the automation reservation while R may be
            // queued; a neutral output still leaves movement manual.
            present: state.oos_output.owned != 0
                || state.oos.grab_delivery_pending()
                || state.oos.awaiting_up_b(),
            committed: true,
        },
        Intent {
            owner: Owner::Safety,
            present: state.tech_output.press
                || state.tech_output.owns_guard
                || inputs.mash_active
                || inputs.damage_active,
            committed: true,
        },
        Intent {
            owner: Owner::Trip,
            present: state.trip_output.owned != 0,
            committed: true,
        },
        Intent {
            owner: Owner::Fair,
            present: state.fair_output.owned != 0,
            committed: true,
        },
        Intent {
            owner: Owner::Trump,
            present: state.trump_output.owned != 0,
            committed: true,
        },
        Intent {
            owner: Owner::Ledge,
            present: state.ledge_active
                && (state.action_owner.current_owner() == Some(Owner::Ledge)
                    || !state.autopilot_guarding
                // An idle always-on watch must not swallow
                // Turbo movement. Acquired ledge visits/tapes keep priority.
                && (!inputs.turbo_active
                    || ledge_priority || ledge_committed)
                && (state.oos_output.owned == 0 || ledge_priority || ledge_committed))
                && (state.ledge_output.owned != 0
                    || state.ledge_output.stick.is_some()
                    || state
                        .ledge
                        .as_ref()
                        .is_none_or(|trap| !trap.manual_after_response())),
            committed: ledge_committed,
        },
        Intent::sequence_receipt(
            Owner::RollRead,
            state.roll_read_output.owned,
            state
                .rule_engine
                .as_ref()
                .is_some_and(|e| e.awaiting_up_b()),
        ),
        Intent {
            owner: Owner::GrabDodge,
            present: state.spot_output.owned != 0
                && state
                    .spot_dodge
                    .as_ref()
                    .is_some_and(|s| s.target().is_some()),
            committed: true,
        },
        Intent {
            owner: Owner::OptionRead,
            present: (state.spot_output.owned != 0
                || state
                    .spot_dodge
                    .as_ref()
                    .is_some_and(|s| s.punish_pending()))
                && state
                    .spot_dodge
                    .as_ref()
                    .is_none_or(|s| s.target().is_none()),
            committed: true,
        },
        Intent {
            owner: Owner::Defense,
            present: state.autopilot_guarding && state.ledge_output.owned != 0,
            committed: false,
        },
        Intent {
            owner: Owner::Turbo,
            present: inputs.turbo_active || inputs.pivot_active,
            committed: false,
        },
        Intent {
            owner: Owner::Posture,
            present: inputs.posture_active,
            committed: false,
        },
    ];
    intents
}
/// Selects the appropriate controller output based on the winning owner, posture, and current situation, applying grab handoff and out-of-shield adjustments as needed.
fn selected_owner_output(
    state: &mut State,
    winner: Option<Owner>,
    posture: ControllerOutput,
    situation: i32,
) -> ControllerOutput {
    let mut selected = match winner {
        Some(Owner::Reflect) => state.reflect_output,
        Some(Owner::Trip) => state.trip_output,
        Some(Owner::Fair) => state.fair_output,
        Some(Owner::Trump) => state.trump_output,
        Some(Owner::OutOfShield) => state.oos_output,
        Some(Owner::Posture) => posture,
        Some(Owner::Ledge) => state.ledge_output,
        Some(Owner::RollRead) => state.roll_read_output,
        Some(Owner::GrabDodge | Owner::OptionRead) => state.spot_output,
        Some(Owner::Defense) => state.ledge_output,
        _ => ControllerOutput::default(),
    };
    if winner == Some(Owner::OutOfShield) {
        let movement = state.action_owner.policy_inputs().stick();
        selected = state.oos.with_grab_movement(selected, movement);
    }
    if situation == kirby_assist_core::native_states::situation::GROUND {
        selected = kirby_assist_core::grab_handoff::GrabHandoff::grab_packet(selected);
    }
    selected
}

/// Determines the appropriate right stick input based on the winning owner, selected controller output, and current situation.
fn selected_owner_right_stick(
    state: &State,
    winner: Option<Owner>,
    selected: ControllerOutput,
    situation: i32,
) -> Option<[f32; 2]> {
    if situation == kirby_assist_core::native_states::situation::GROUND
        && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
    {
        return Some([0.; 2]);
    }
    match winner {
        Some(Owner::Ledge) => state
            .ledge
            .as_ref()
            .and_then(LedgeTrap::requested_right_stick),
        Some(Owner::RollRead) => state
            .rule_engine
            .as_ref()
            .and_then(|rule| rule.right_stick()),
        Some(Owner::OutOfShield) => state.oos.right_stick(),
        Some(Owner::OptionRead) => state
            .spot_dodge
            .as_ref()
            .and_then(|dodge| dodge.right_stick()),
        _ => None,
    }
}

/// Represents a sampled posture, including configured C-stick inputs, controller output, and tilt aiming status.
struct PostureSample {
    configured_c_stick: Option<([f32; 2], [f32; 2])>,
    output: ControllerOutput,
    tilt_aiming: bool,
}

fn sample_mapper_posture(
    state: &mut State,
    manual_context: Option<ManualContext>,
    manual_scope: bool,
    fresh: bool,
    policy_buttons: u32,
    physical: kirby_assist_core::native_input::ButtonTrace,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
) -> PostureSample {
    let (idle_crouch, tilt_turn) = state
        .settings_menu
        .as_ref()
        .map_or((false, false), |m| m.posture_modes());
    let tilt_settings = state.fighter_kind.map_or(Default::default(), |kind| {
        state
            .settings_menu
            .as_ref()
            .map_or(Default::default(), |menu| {
                menu.fighter_tuning(kind).tilt_turn(kind)
            })
    });
    let posture_target = state.posture_target;
    let posture_floor_safe = state.posture_floor_safe;
    let configured_c_stick = manual_context.and_then(|context| {
        (manual_scope && fresh && tilt_turn)
            .then(|| {
                kirby_assist_core::posture::Posture::configured_c_stick(
                    tilt_settings,
                    policy_buttons,
                    physical_stick,
                    physical_right_stick,
                    posture_target,
                    context,
                )
            })
            .flatten()
    });
    let posture = manual_context.map_or_else(ControllerOutput::default, |context| {
        state
            .posture
            .get_or_insert_with(Default::default)
            .sample_configured(
                manual_scope && fresh,
                idle_crouch
                    && posture_floor_safe
                    // Yield immediately on action edges and
                    // the release poll; do not add Down to
                    // an input returning from a grab/shield.
                    && kirby_assist_core::posture::Posture::idle_buttons_clear(physical),
                tilt_turn,
                tilt_settings,
                policy_buttons,
                physical_stick,
                physical_right_stick,
                posture_target,
                context,
            )
    });
    // Turnaround tilt and idle crouch stay passive at the
    // lowest priority, including physical tilt buffer gaps.
    let tilt_aiming = tilt_turn
        && posture.stick.is_some()
        && (configured_c_stick.is_some()
            || kirby_assist_core::posture::Posture::down_tilt_intent(
                policy_buttons,
                physical_stick,
                physical_right_stick,
            ));
    PostureSample {
        configured_c_stick,
        output: posture,
        tilt_aiming,
    }
}
/// Represents a sampled damage stick state, including DI, SDI, and damage stick inputs.
struct DamageStickSample {
    di_stick: Option<[f32; 2]>,
    sdi_stick: Option<[f32; 2]>,
    damage_stick: Option<[f32; 2]>,
}
/// Handles manual ledge walking, returning the appropriate stick input if conditions are met.
fn manual_ledge_walk(
    state: &State,
    settings: &AssistSettings,
    manual_scope: bool,
    fresh: bool,
    policy_buttons: u32,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
    ledge: ControllerOutput,
    turbo: TurboOutput,
) -> Option<[f32; 2]> {
    if !settings.enabled
        || !manual_scope
        || policy_buttons & MAPPER_L == 0
        || !state.ledge_active
        || state
            .settings_menu
            .as_ref()
            .is_some_and(|menu| menu.preview_slot().is_some())
    {
        return None;
    }

    let walking = if fresh && settings.manual_spacing && state.ledge_walk_ready {
        let stick = ledge_walk_stick(physical_stick, settings.walk_strength);
        let spot = state.ledge_visits.walking_ledge();
        let manual_attack = policy_buttons
            & (kirby_assist_core::native_input::MAPPER_A
                | kirby_assist_core::native_input::MAPPER_B
                | settings.grab_button)
            != 0
            || physical_right_stick.iter().any(|value| value.abs() > 0.2);
        Some(match (state.ledge_walk_motion, spot) {
            (Some((position, velocity)), Some(spot)) if !manual_attack => {
                kirby_assist_core::manual_controls::stop_walk_at_ledge(
                    stick,
                    position,
                    velocity,
                    spot,
                    settings.stand_distance,
                )
            }
            _ => stick,
        })
    } else {
        None
    };
    if ledge.stick.is_none() && turbo.stick.is_none() {
        walking
    } else {
        None
    }
}
/// Samples the damage stick inputs for the given state and configuration, handling DI, SDI, and damage trajectory considerations.
fn sample_damage_sticks(
    state: &mut State,
    config: &Config,
    manual_scope: bool,
    fresh: bool,
    mash_stick: Option<[f32; 2]>,
    physical_stick: [f32; 2],
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) -> DamageStickSample {
    let sdi_context = state.sdi_context;
    let sdi_enabled = config.auto_sdi && manual_scope && fresh;
    let di_context = state.di_context;
    let damage_trajectory = state.damage_trajectory;
    let di_stick = di_context.and_then(|context| {
        state
            .di
            .get_or_insert_with(Default::default)
            .sample_exclusive_with_trajectory(
                kirby_assist_core::auto_di::Settings {
                    enabled: config.auto_di.enabled && manual_scope && fresh,
                    ..config.auto_di
                },
                context,
                damage_trajectory,
            )
    });
    if !config.auto_di.enabled || di_context.is_none() || !manual_scope || !fresh {
        state.di = None;
    }
    #[cfg(feature = "diagnostic-logging")]
    {
        let decision = di_stick.and_then(|_| state.di.as_ref().and_then(|di| di.decision()));
        if state.last_di_diagnostic != decision {
            if let Some(d) = decision {
                messages.push(format!("AUTO DI: {}; stick=({:.2},{:.2}); reserved launch={:?}. Final victim-hitstop input only; a heuristic, not guaranteed survival or combo escape.", d.reason.label(), d.stick[0], d.stick[1], di_context.map(|c| c.launch)));
            } else if state.last_di_diagnostic.is_some() {
                messages.push("AUTO DI released: hitstop ended or DI became unavailable.".into());
            }
            state.last_di_diagnostic = decision;
        }
    }
    // A tech deadline can precede DI; ordinary tech/release
    // yields to launch DI. Losing modes cannot emit inputs
    // or consume a flick as though it had been delivered.
    let tech_owns = state.tech_output.press || state.tech_output.owns_guard;
    let sdi_stick = if di_stick.is_none() && !tech_owns && mash_stick.is_none() {
        let previous = state.last_mapped_stick.unwrap_or(physical_stick);
        sdi_context.and_then(|context| {
            state
                .sdi
                .get_or_insert_with(SdiState::default)
                .sample_exclusive_with_previous(
                    sdi_enabled,
                    context,
                    damage_trajectory.map(|t| (t.position, t.blast)),
                    previous,
                )
        })
    } else {
        state.sdi = None;
        None
    };
    let damage_stick = di_stick.or(sdi_stick);
    if !sdi_enabled || sdi_context.is_none() {
        state.sdi = None;
    }
    DamageStickSample {
        di_stick,
        sdi_stick,
        damage_stick,
    }
}

/// Prepares the mapper overlay for the given controller index and session ticket. Returns a `MapperPreparation` if successful.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers for the output and wrapper. The caller must ensure that these pointers are valid and correctly aligned.
pub(super) unsafe fn prepare_mapper_overlay(
    index: i32,
    output: *mut c_void,
    wrapper: *mut c_void,
    master_ready_go: Option<Option<bool>>,
    training_mode: Option<bool>,
    training_test_mode: bool,
    session_ticket: SessionTicket,
) -> Option<MapperPreparation> {
    let mut restore = None;
    let mut mapped_left_stick = None;
    let mut mapped_automatic_request = None;
    let mut mapped_auto_grab = None;
    let mut mapped_reflect = false;
    let mut mapped_reflector = None;
    let mut mapped_auto_up_special = None;
    let mut mapped_auto_dodge = None;
    let mut mapped_dodge_punish_grab = None;
    let mut mapped_attack_hold = false;
    let mut mapped_literal_request = None;
    let mut mapped_unshielded_auto_grab = false;
    let mut mapped_sequence_right = None;
    let mut reserved_dpad = 0;
    let mut tech_applied = false;
    let mut mapped_tech_output = None;
    let mut mapped_grab_handoff = None;
    let mut sequence_short_hop = false;
    let mut mapped_manual_jump = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut trump_pressed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut ledge_edge = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut grab_delivery = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut grab_mapping_comparison = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut messages = Vec::new();
    #[cfg(feature = "diagnostic-logging")]
    let mut mode_report = None;
    if let Some(config) = CONFIG.get().map(|config| **config) {
        let config = diagnostic_config(config);
        // A startup-disabled mod is a true passthrough: do not bind routes,
        // update menus, refresh fighters, run policies or touch controller data.
        if !config.controls.enabled {
            return None;
        }
        if !wrapper.is_null()
            && !output.is_null()
            && (0..kirby_assist_core::native_input::MAPPER_SLOT_COUNT).contains(&index)
        {
            // The wrapper stores a Controller pointer at this byte offset. read() loads
            // that pointer; cast() alone only changes its type. These native
            // offsets belong to the verified layout, not user-tunable settings.
            let controller = controller_from_wrapper(wrapper);
            #[cfg(feature = "diagnostic-logging")]
            if !controller.is_null()
                && SEEN_MAPPERS.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0
            {
                messages.push(format!("mapper observed index {index}: npad={}, style={}, connected={}; configured local controller={}", (*controller).npad, (*controller).style, (*controller).connected, config.player_index));
            }
            if !controller.is_null()
                && selected_controller_mapper(config.player_index, (*controller).npad, index)
                    == Some(index)
            {
                // Read physical input before applying any assist. Reusing last
                // output as "human input" would keep automated buttons latched.
                let physical = trace(controller);
                // Let the classic monitor chord pass before D-pad Down can
                // start a trump. This applies to standalone builds as well as
                // Training; OS overlays remain responsible for their own focus.
                if kirby_assist_core::settings_menu::monitor_chord(
                    config.control_layout.policy_buttons(physical.held),
                ) {
                    if let Ok(mut state) = STATE.lock() {
                        state.cancel();
                        state
                            .settings_menu
                            .get_or_insert_with(Default::default)
                            .observe_passthrough(physical.held, config.control_layout);
                    }
                    return None;
                }
                let physical_stick = [(*controller).left_stick_x, (*controller).left_stick_y];
                let physical_right_stick =
                    [(*controller).right_stick_x, (*controller).right_stick_y];
                let physical_autorepeat = (*controller).autorepeat[0];
                let physical_triggers = [(*controller).left_trigger, (*controller).right_trigger];
                // Only the selected controller checks the existing battle
                // manager guard. Menu lifetime is independent of request TTL.
                let menu_ready_go =
                    master_ready_go.unwrap_or_else(|| observations::battle_menu_ready_go());
                // Without Training Modpack, query only the selected controller;
                // with it, reuse the earlier shortcut-policy scene read.
                let menu_training_mode = training_mode.unwrap_or_else(|| game::is_training_mode());
                update_mapper_menu_and_input(
                    controller,
                    index,
                    &config,
                    physical,
                    physical_stick,
                    physical_right_stick,
                    menu_ready_go,
                    menu_training_mode,
                    training_test_mode,
                    #[cfg(feature = "diagnostic-logging")]
                    &mut trump_pressed,
                    #[cfg(feature = "diagnostic-logging")]
                    &mut messages,
                );
                // Restore 1.0.4's immediate pre-map refresh without its roster
                // scan: only resolve a recently observed human fighter ID.
                // Revalidate identity on every refresh; never retain a pointer.
                refresh_selected_fighter(index);
                if let Ok(mut state) = STATE.lock() {
                    if state.session.accepts(session_ticket)
                        && state.mapping_index == Some(index)
                        && (*controller).connected != 0
                    {
                        // Recreate the canonical policy view in this lock scope.
                        // The earlier copy configures observation/menu state, but
                        // is intentionally scoped to that mapper-state update.
                        let policy_buttons = if training_test_mode {
                            training_test::policy_buttons(physical.held)
                        } else {
                            physical.held
                        };
                        let policy_buttons = config.control_layout.policy_buttons(policy_buttons);
                        let policy_buttons = state
                            .settings_menu
                            .as_ref()
                            .map_or(policy_buttons, |m| m.native_buttons);
                        let now = Instant::now();
                        let fresh = kirby_assist_core::observation::input_update_current(
                            state.evaluated_tick,
                            state.raw_tick,
                        ) && state
                            .request_seen
                            .is_some_and(|seen| now.duration_since(seen) < REQUEST_LIFETIME);
                        let ledge_fresh = fresh;
                        let route_current =
                            state.human_routes[index as usize].is_some_and(|(id, seen)| {
                                Some(id) == state.associated_object
                                    && seen.elapsed() < SESSION_OBSERVATION_GAP
                            });
                        let ledge_config = state.settings_menu.as_ref().map_or_else(
                            || settings_for_fighter(state.fighter_kind, config.assist),
                            |p| p.settings(state.fighter_kind, &config.assist),
                        );
                        let manual_scope = config.controls.enabled
                            && route_current
                            && state.associated_object.is_some()
                            && state.associated_object == state.observed_object
                            && physical.held & MAPPER_PLUS == 0;
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            let expired = manual_scope
                                && state.menu_battle_active
                                && !fresh
                                && state.request_seen.is_some();
                            if expired && !state.last_input_expired {
                                crate::performance::health::event(
                                    crate::performance::health::Event::InputExpired,
                                );
                            }
                            state.last_input_expired = expired;
                        }
                        // Expired requests become neutral. The overlay still
                        // knows the previous assisted output, allowing a release.
                        let native_grab = state.manual_context.is_some_and(|c| {
                            kirby_assist_core::action_owner::native_grab_active(c.status)
                        });
                        let auto_mash = state
                            .settings_menu
                            .as_ref()
                            .map_or(config.controls.auto_mash, |menu| {
                                menu.auto_mash(config.controls.auto_mash)
                            });
                        let mash_stick = if fresh && auto_mash {
                            state.mash_stick
                        } else {
                            None
                        };

                        // Compose all native requests through ONE button history.
                        // Independent overlays can disagree about a release when
                        // tech, mash and ledge ownership switch on the same frame.
                        // Read the trigger from PHYSICAL buttons, then consume
                        // its taunt bits. Releasing returns the fresh physical
                        // stick immediately, rather than forcing neutral.
                        let tick = state.policy_tick.unwrap_or(state.raw_tick);
                        let can_move = fresh && state.policy_tick.is_some() && state.turbo_ready;
                        // The selected button binding alone activates Turbo t-bag.
                        // L3 is not remapped to the L ledge-assist role.
                        let turbo_held = policy_buttons & TURBO_TRIGGER != 0;
                        let turbo_facing = state.manual_context.map_or(1.0, |ctx| ctx.facing);
                        let turbo_input_tick = state.raw_tick;
                        let mut turbo_output = state
                            .turbo
                            .get_or_insert_with(TurboState::default)
                            .sample_teabag(
                                config.turbo,
                                // Generate the next local input packet now;
                                // never wait for delayed crouch acknowledgment.
                                turbo_input_tick,
                                turbo_held,
                                turbo_facing,
                                manual_scope,
                                can_move,
                            );
                        // Bound directions are consumed by the menu adapter. Normal
                        // directions pass through; no hard-coded B-reverse/trump.
                        // L holds ledge trap; held Y runs its saved mode.
                        // Changing tools releases the previous policy.
                        // Stale observations still release automation ownership.
                        let turbo_stick = turbo_output.stick;
                        let pivot_context = state.manual_context.filter(|_| {
                            fresh
                                && manual_scope
                                && state.policy_tick.is_some()
                                && turbo_stick.is_none()
                                && physical.held
                                    & !kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS
                                    == 0
                                && physical_right_stick
                                    .iter()
                                    .all(|v| v.is_finite() && v.abs() <= 0.1)
                        });
                        if pivot_context.is_none() {
                            state.dash_pivot = None;
                        }
                        let pivot_stick = pivot_context.map_or(None, |context| {
                            state
                                .dash_pivot
                                .get_or_insert_with(Default::default)
                                .sample(
                                    tick,
                                    context.status,
                                    context.situation,
                                    context.stopped,
                                    physical_stick,
                                )
                                .stick
                        });
                        let mut ledge = if ledge_fresh && state.ledge_active {
                            state.ledge_output
                        } else {
                            ControllerOutput::default()
                        };
                        let manual_walk_stick = manual_ledge_walk(
                            &state,
                            &ledge_config,
                            manual_scope,
                            fresh,
                            policy_buttons,
                            physical_stick,
                            physical_right_stick,
                            ledge,
                            turbo_output,
                        );
                        #[cfg(feature = "diagnostic-logging")]
                        if trump_pressed {
                            messages.push(format!(
                                "trump input: scope={}, fresh={}, action={}, stick={:?}, reserved={reserved_dpad:#x}",
                                manual_scope, fresh,
                                state.trump.as_ref().map_or("unobserved", LedgeTrump::action_name),
                                state.trump_output.stick,
                            ));
                        }
                        let manual_context = state.manual_context;
                        let PostureSample {
                            configured_c_stick,
                            output: posture,
                            tilt_aiming,
                        } = sample_mapper_posture(
                            &mut state,
                            manual_context,
                            manual_scope,
                            fresh,
                            policy_buttons,
                            physical,
                            physical_stick,
                            physical_right_stick,
                        );
                        // Menu input is reserved during entry/respawn too.
                        let menu_consumed =
                            if state.menu_battle_active && state.menu_fighter_kind.is_some() {
                                state.settings_menu.as_ref().map_or(0, |menu| menu.consumed)
                            } else {
                                0
                            };
                        reserved_dpad |=
                            menu_consumed & kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS;
                        let DamageStickSample {
                            di_stick,
                            sdi_stick,
                            damage_stick,
                        } = sample_damage_sticks(
                            &mut state,
                            &config,
                            manual_scope,
                            fresh,
                            mash_stick,
                            physical_stick,
                            #[cfg(feature = "diagnostic-logging")]
                            &mut messages,
                        );
                        // Single final authority for sticks/buttons. Preparation
                        // keeps ledge/roll timing ahead of every optional tool.
                        // A detected ledge visit reserves its response against
                        // general reads. Before acquisition they run normally.
                        let mut intents = mapper_action_intents(
                            &mut state,
                            MapperIntentInputs {
                                fresh,
                                manual_scope,
                                mash_active: mash_stick.is_some(),
                                damage_active: damage_stick.is_some(),
                                turbo_active: turbo_stick.is_some(),
                                pivot_active: pivot_stick.is_some(),
                                posture_active: posture.stick.is_some(),
                            },
                        );
                        let handoff_status = manual_context.map_or(-1, |c| c.status);
                        let handoff_situation = manual_context.map_or(
                            kirby_assist_core::native_states::situation::UNAVAILABLE,
                            |c| c.situation,
                        );
                        let handoff_context =
                            manual_context.filter(|_| fresh).map(|c| GameplayContext {
                                gameplay: manual_scope,
                                status: c.status,
                                situation: c.situation,
                                motion_kind: c.motion,
                                motion_frame: c.frame,
                                motion_rate: c.rate,
                                stopped: c.stopped,
                                ..GameplayContext::default()
                            });
                        state
                            .grab_handoff
                            .observe_native(manual_scope, handoff_context);
                        state.grab_handoff.restrict(&mut intents);
                        let mut winner = state
                            .action_owner
                            .select(fresh && manual_scope && !native_grab, &intents);
                        sequence_short_hop = winner == Some(Owner::Ledge)
                            && state.ledge_output.native_short_hop
                            || winner == Some(Owner::OutOfShield)
                                && state.oos_output.native_short_hop
                            || winner == Some(Owner::OptionRead)
                                && state.spot_output.native_short_hop;
                        let mut selected =
                            selected_owner_output(&mut state, winner, posture, handoff_situation);
                        let selected_right =
                            selected_owner_right_stick(&state, winner, selected, handoff_situation);
                        if !state.grab_handoff.permits(winner, selected, selected_right) {
                            // Stop the old script, not just this packet; otherwise its
                            // delayed B/Up step could resume after receipt expiry.
                            if let Some(owner) = winner {
                                state.action_owner.release(owner);
                            }
                            winner = None;
                            selected = ControllerOutput::default();
                        }
                        if winner == Some(Owner::Safety) {
                            (selected, mapped_tech_output) =
                                kirby_assist_core::action_owner::safety_output_with_deadline(
                                    state.tech_output,
                                    mash_stick,
                                    di_stick,
                                    sdi_stick,
                                    state.tech_urgent,
                                );
                        }
                        selected = kirby_assist_core::action_owner::exclusive_output(
                            winner,
                            state.action_owner.current_owner() == winner && winner.is_some(),
                            selected,
                        );
                        let tech_takeover =
                            winner == Some(Owner::Safety) && mapped_tech_output.is_some();
                        let mash_takeover = winner == Some(Owner::Safety) && mash_stick.is_some();
                        mapped_grab_handoff = Some((
                            state.raw_tick,
                            winner,
                            selected.requested & kirby_assist_core::native_input::MAPPER_R != 0,
                            fresh
                                && manual_scope
                                && !native_grab
                                && handoff_situation
                                    == kirby_assist_core::native_states::situation::GROUND
                                && !kirby_assist_core::ledge_trap::sequence_interrupted(
                                    handoff_status,
                                ),
                        ));
                        ledge = selected;
                        turbo_output.stick = match winner {
                            Some(Owner::Safety) => selected.stick,
                            Some(Owner::Turbo) => turbo_stick.or(pivot_stick),
                            // An armed ledge wait reserves priority without an
                            // attack stick. Keep its shared physical walk cap;
                            // owned releases and active recipes must never fall through.
                            Some(Owner::Ledge)
                                if selected.owned == 0 && selected.requested == 0 =>
                            {
                                selected.stick.or(manual_walk_stick)
                            }
                            None => manual_walk_stick,
                            _ => selected.stick,
                        };
                        state.cancel_non_owner(winner);
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            use kirby_assist_core::diagnostics::modes::{bit, Snapshot};
                            let status = manual_context.map_or(-1, |c| c.status);
                            // Log only a change into a real assist override, not
                            // every poll or native animation stop. No game reads.
                            let blocker = if manual_scope
                                && fresh
                                && physical_stick[0].abs() >= 0.8
                                && physical_stick[1].abs() < 0.3
                                && turbo_output.stick.is_some_and(|s| {
                                    s[0].abs() <= 0.2 || s[0] * physical_stick[0] < 0.0
                                }) {
                                winner
                            } else {
                                None
                            };
                            if blocker != state.last_movement_blocker {
                                state.last_movement_blocker = blocker;
                                if let Some(owner) = blocker {
                                    messages.push(format!("MOVEMENT OVERRIDE: owner={}, native_status={status:#x}, physical_stick={physical_stick:?}, proposed_stick={:?}, requested={:#x}, ledge_active={}",
                                        kirby_assist_core::diagnostics::modes::name(owner),turbo_output.stick,ledge.requested,state.ledge_active));
                                }
                            }
                            let snapshot = Snapshot {
                                winner,
                                present: intents
                                    .iter()
                                    .filter(|i| i.present)
                                    .fold(0, |mask, i| mask | bit(i.owner)),
                                committed: intents
                                    .iter()
                                    .any(|i| Some(i.owner) == winner && i.committed),
                                requested: ledge.requested,
                                stick: turbo_output.stick.map(|stick| {
                                    stick.map(|v| {
                                        if v > 0.08 {
                                            1
                                        } else if v < -0.08 {
                                            -1
                                        } else {
                                            0
                                        }
                                    })
                                }),
                                status,
                                motion: manual_context.map_or(0, |c| c.motion),
                                native_frame: manual_context
                                    .map_or(0, |c| c.frame.clamp(0.0, 65535.0) as u16),
                                stopped: manual_context.is_some_and(|c| c.stopped),
                                can_act: state.turbo_ready,
                                hit: state.mode_hit,
                                valid: fresh && manual_scope,
                                safety: (if state.tech_output.press || state.tech_output.owns_guard
                                {
                                    kirby_assist_core::diagnostics::modes::SAFETY_TECH_GUARD
                                } else {
                                    0
                                }) | (if mash_stick.is_some() {
                                    kirby_assist_core::diagnostics::modes::SAFETY_MASH
                                } else {
                                    0
                                }) | (if sdi_stick.is_some() && di_stick.is_none() {
                                    kirby_assist_core::diagnostics::modes::SAFETY_SDI
                                } else {
                                    0
                                }) | (if di_stick.is_some() {
                                    kirby_assist_core::diagnostics::modes::SAFETY_DI
                                } else {
                                    0
                                }),
                            };
                            // Reuse the mapper's timestamp; do not read another clock per poll.
                            let origin = *state.mode_log_origin.get_or_insert(now);
                            let ms = now
                                .saturating_duration_since(origin)
                                .as_millis()
                                .min(u128::from(u64::MAX))
                                as u64;
                            mode_report = state.mode_diagnostics.sample(snapshot, ms);
                        }

                        // A held C-stick can otherwise inject an attack while
                        // left-stick-only mashing. Block its digital aliases too.
                        let blocked_stick_buttons = if mash_takeover {
                            kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                                | kirby_assist_core::manual_controls::STICK_CLICKS
                        } else {
                            0
                        };
                        let cancel_stick = if handoff_situation
                            == kirby_assist_core::native_states::situation::GROUND
                            && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
                        {
                            Some([0.; 2])
                        } else if winner == Some(Owner::RollRead) && fresh && selected.owned != 0 {
                            state.rule_engine.as_ref().and_then(|r| r.right_stick())
                        } else if winner == Some(Owner::OutOfShield)
                            && fresh
                            && state.oos_output.owned != 0
                        {
                            state.oos.right_stick()
                        } else if winner == Some(Owner::OptionRead) && fresh && selected.owned != 0
                        {
                            state.spot_dodge.as_ref().and_then(|s| s.right_stick())
                        } else if winner == Some(Owner::Ledge) && fresh {
                            state
                                .ledge
                                .as_ref()
                                .and_then(LedgeTrap::requested_right_stick)
                        } else {
                            None
                        };
                        // All manual tools share ONE final edge history. When
                        // adding a tool, compose its requested/owned masks here
                        // and decide its stick priority explicitly; do not write
                        // another independent A/R/ZR history after this point.
                        let sequence_walking = match winner {
                            Some(Owner::Ledge) => state
                                .ledge
                                .as_ref()
                                .and_then(LedgeTrap::requested_walk_stick)
                                .is_some(),
                            Some(Owner::RollRead) => {
                                state.rule_engine.as_ref().is_some_and(|r| r.walking())
                            }
                            // A moving grab retains raw horizontal input; it is not a file Walk.
                            Some(Owner::OutOfShield) => false,
                            Some(Owner::OptionRead) => {
                                state.spot_dodge.as_ref().is_some_and(|s| s.walking())
                            }
                            _ => false,
                        };
                        let exclusive_sequence = ledge.owned
                            & kirby_assist_core::native_input::GAMEPLAY_BUTTONS
                            == kirby_assist_core::native_input::GAMEPLAY_BUTTONS;
                        if exclusive_sequence {
                            reserved_dpad |= kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS;
                        }
                        let canonical_consumed =
                            kirby_assist_core::native_input::sequence_flick_consumption(
                                ledge.owned,
                                sequence_walking,
                            ) | reserved_dpad
                                | menu_consumed
                                | if !exclusive_sequence
                                    && ledge.owned
                                        & kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS
                                        != 0
                                    && !mash_takeover
                                    && !matches!(winner, Some(Owner::Trip | Owner::Fair))
                                    && !(matches!(winner, Some(Owner::Posture)) && tilt_aiming)
                                {
                                    (1 << 21) | (1 << 23)
                                } else {
                                    0
                                }
                                | if manual_walk_stick.is_some()
                                    && turbo_output.stick == manual_walk_stick
                                    && ledge.owned == 0
                                    && !mash_takeover
                                {
                                    (1 << 21) | (1 << 23)
                                } else {
                                    0
                                };
                        let canonical_requested =
                            kirby_assist_core::native_input::selected_requests(
                                ledge.owned,
                                ledge.requested,
                                0,
                            );
                        // Deliver the native jump pulse through the same raw
                        // history as every other simulated input. The final
                        // mapped flag supplements this path; it does not replace it.
                        if exclusive_sequence {
                            mapped_automatic_request = Some(canonical_requested);
                        }
                        mapped_manual_jump = winner == Some(Owner::Ledge)
                            && canonical_requested
                                & kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS
                                != 0;
                        let canonical_owned = ledge.owned
                            | if tech_takeover {
                                state.tech_output.owned_buttons()
                            } else {
                                0
                            }
                            | if mapped_tech_output.is_some_and(|t| t.press) {
                                MAPPER_ZR
                            } else {
                                0
                            }
                            | if cancel_stick.is_some() {
                                kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                            } else {
                                0
                            }
                            | blocked_stick_buttons;
                        // Keep source consumption above; only the final taunt
                        // filter yields to an explicitly owned sequence D-pad.
                        reserved_dpad =
                            kirby_assist_core::manual_controls::sequence_taunt_reservation(
                                reserved_dpad,
                                ledge.requested,
                                ledge.owned,
                            );
                        let approach_pulse =
                            winner == Some(Owner::OutOfShield) && state.oos.approach_pending();
                        if mapped_tech_output.is_some_and(|t| t.press) && !state.last_mapped_guard {
                            state
                                .manual_overlay
                                .get_or_insert_with(ManualOverlay::default)
                                .rearm_tech_guard(
                                    config.control_layout.physical_buttons(MAPPER_ZR),
                                    physical,
                                );
                        }
                        let manual = state
                            .manual_overlay
                            .get_or_insert_with(ManualOverlay::default)
                            .apply_reaction_controls(
                                physical,
                                physical_stick,
                                turbo_output,
                                config.control_layout.physical_buttons(canonical_consumed),
                                config.control_layout.physical_buttons(canonical_requested),
                                config
                                    .control_layout
                                    .physical_owned_buttons(canonical_owned),
                                winner == Some(Owner::RollRead) || approach_pulse,
                            );
                        if fresh && manual_scope && sequence_walking {
                            mapped_left_stick = ledge.stick;
                        }
                        // Built-in analog outputs must survive the native
                        // mapper's flick/history filtering just like file Walk.
                        if fresh
                            && manual_scope
                            && (winner == Some(Owner::Turbo) && turbo_stick.is_some()
                                || (winner == Some(Owner::Ledge) || winner.is_none())
                                    && selected.owned == 0
                                    && manual_walk_stick.is_some()
                                    && turbo_output.stick == manual_walk_stick)
                        {
                            mapped_left_stick = turbo_output.stick;
                        }
                        // Preserve the selected literal A packet, including directional
                        // smashes and one-frame presses. Plan metadata used to retain
                        // only neutral multi-frame holds, losing A while the stick survived.
                        mapped_attack_hold = fresh
                            && manual_scope
                            && matches!(
                                winner,
                                Some(
                                    Owner::Ledge
                                        | Owner::RollRead
                                        | Owner::OutOfShield
                                        | Owner::OptionRead
                                )
                            )
                            && selected.owned & kirby_assist_core::native_input::MAPPER_A != 0
                            && selected.requested & kirby_assist_core::native_input::MAPPER_A != 0
                            && manual.buttons.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_A)
                                != 0;
                        // Forward all exclusive sequence axes identically. Drift
                        // and Run must not depend on the native mapper retaining
                        // an analog value after physical flick history was masked.
                        if let Some(stick) =
                            kirby_assist_core::native_input::exclusive_sequence_stick(
                                ledge.owned,
                                ledge.stick,
                            )
                        {
                            mapped_left_stick = Some(stick);
                        }
                        if exclusive_sequence {
                            mapped_sequence_right = Some(cancel_stick.unwrap_or([0.; 2]));
                        }
                        if fresh && manual_scope && exclusive_sequence && winner.is_some() {
                            mapped_literal_request = Some(
                                [
                                    kirby_assist_core::native_input::MAPPER_A,
                                    kirby_assist_core::native_input::MAPPER_B,
                                    kirby_assist_core::native_input::MAPPER_R,
                                    kirby_assist_core::native_input::MAPPER_ZR,
                                ]
                                .into_iter()
                                .fold(0, |mask, button| {
                                    mask | if canonical_requested & selected.owned & button != 0
                                        && manual.buttons.held
                                            & config.control_layout.physical_buttons(button)
                                            != 0
                                    {
                                        button
                                    } else {
                                        0
                                    }
                                }),
                            );
                            mapped_unshielded_auto_grab = winner == Some(Owner::OutOfShield)
                                && state.oos.unshielded_grab_started();
                        }
                        let posture_side = if winner == Some(Owner::Posture) {
                            configured_c_stick
                                .and_then(|(_, right)| (right[1] == 0.0).then_some(right[0]))
                        } else {
                            None
                        };
                        let modified = if winner.is_none() || winner == Some(Owner::Posture) {
                            state
                                .posture_side_aliases
                                .apply(manual.buttons, posture_side)
                        } else {
                            state.posture_side_aliases = Default::default();
                            manual.buttons
                        };
                        if winner == Some(Owner::Reflect) {
                            mapped_reflector = Some(state.raw_tick);
                        }
                        mapped_reflect = winner == Some(Owner::Reflect)
                            && selected.requested & kirby_assist_core::native_input::MAPPER_B != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_B)
                                != 0;
                        if winner == Some(Owner::OutOfShield)
                            && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_R)
                                != 0
                        {
                            mapped_auto_grab = state
                                .oos
                                .target()
                                .map(|target| (state.raw_tick, target, true));
                        } else if approach_pulse
                            && selected.stick.is_some_and(|s| s[0].abs() >= 0.95)
                        {
                            mapped_auto_grab = state
                                .oos
                                .target()
                                .map(|target| (state.raw_tick, target, false));
                        }
                        if winner == Some(Owner::OptionRead)
                            && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_R)
                                != 0
                        {
                            mapped_dodge_punish_grab = state
                                .spot_dodge
                                .as_ref()
                                .and_then(|s| s.follow_target())
                                .map(|id| (state.raw_tick, id));
                        }
                        if winner == Some(Owner::GrabDodge)
                            && selected.requested & MAPPER_ZR != 0
                            && selected.stick.is_some_and(|s| s[1] < -0.5)
                        {
                            mapped_auto_dodge = state
                                .spot_dodge
                                .as_ref()
                                .and_then(|s| s.target())
                                .map(|target| (state.raw_tick, target));
                        }
                        if winner == Some(Owner::OutOfShield)
                            && state.oos.shield_up_b_pending()
                            && selected.requested & kirby_assist_core::native_input::MAPPER_B != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_B)
                                != 0
                        {
                            mapped_auto_up_special = state
                                .oos
                                .target()
                                .map(|target| (state.raw_tick, target, Owner::OutOfShield));
                        } else if selected.requested & kirby_assist_core::native_input::MAPPER_B
                            != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_B)
                                != 0
                        {
                            let target = match winner {
                                Some(Owner::RollRead) => state
                                    .rule_engine
                                    .as_ref()
                                    .filter(|e| e.direct_up_b())
                                    .and_then(|e| e.target()),
                                Some(Owner::OptionRead) => state
                                    .spot_dodge
                                    .as_ref()
                                    .filter(|s| s.direct_up_b())
                                    .and_then(|s| s.follow_target()),
                                _ => None,
                            };
                            mapped_auto_up_special = target
                                .zip(winner)
                                .map(|(target, owner)| (state.raw_tick, target, owner));
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if state.grab_trace.active() || state.calibration.active() {
                            grab_delivery = Some((
                                state.ledge_output.requested,
                                modified.held,
                                modified.pressed,
                                if !ledge_fresh {
                                    "expired_observation"
                                } else if !manual_scope {
                                    "outside_manual_scope"
                                } else if mash_takeover {
                                    "mash_priority"
                                } else if state.tech_output.owns_guard {
                                    "tech_priority"
                                } else if winner == Some(Owner::Trip) {
                                    "trip_priority"
                                } else if winner == Some(Owner::Fair) {
                                    "fair_priority"
                                } else if state.trump_output.owned != 0 {
                                    "trump_priority"
                                } else {
                                    "ledge"
                                },
                            ));
                        }
                        tech_applied = mapped_tech_output.is_some_and(|t| t.press);
                        #[cfg(feature = "diagnostic-logging")]
                        if ledge.requested & modified.pressed != 0 {
                            ledge_edge = Some(ledge.requested);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            let grab_button = config
                                .control_layout
                                .physical_buttons(kirby_assist_core::native_input::MAPPER_R);
                            let automatic = winner == Some(Owner::OutOfShield)
                                && selected.requested & kirby_assist_core::native_input::MAPPER_R
                                    != 0
                                && modified.held & grab_button != 0;
                            let manual_grab = physical.held & grab_button != 0
                                && selected.requested & kirby_assist_core::native_input::MAPPER_R
                                    == 0;
                            if automatic || manual_grab {
                                let (counter, pressed) = if automatic {
                                    (&AUTOMATIC_GRAB_COMPARISON_POLLS, modified.pressed)
                                } else {
                                    (&MANUAL_GRAB_COMPARISON_POLLS, physical.pressed)
                                };
                                if pressed & grab_button != 0 {
                                    counter.store(0, Ordering::Relaxed);
                                }
                                if counter
                                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                                        (count < 8).then_some(count + 1)
                                    })
                                    .is_ok()
                                {
                                    grab_mapping_comparison = Some((
                                        automatic,
                                        winner,
                                        selected.requested,
                                        physical,
                                        modified,
                                        physical_stick,
                                        manual.stick,
                                        physical_triggers,
                                        kirby_assist_core::native_input::owned_trigger_axes(
                                            physical_triggers,
                                            manual.clear_autorepeat,
                                        ),
                                    ));
                                }
                            }
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if let Some(message) = state.pending_combat_log.take() {
                            messages.push(message);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if let Some(message) = state.pending_tech_log.take() {
                            messages.push(message);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if let Some(message) = state.pending_ledge_log.take() {
                            messages.push(message);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if config.controls.tech_debug
                            && state.last_logged_scenario.1
                            && modified.pressed & MAPPER_ZR != 0
                            && mapped_tech_output.is_some_and(|t| t.press)
                        {
                            messages.push(format!("tech ZR applied: input_update={}, held={:#x}, previous={:#x}, pressed={:#x}", state.raw_tick, modified.held, modified.previous, modified.pressed));
                            messages.push(format!(
                                "tech request timing: {:?}",
                                state
                                    .tech_input
                                    .as_ref()
                                    .and_then(TechInput::last_observation)
                            ));
                        }
                        state.last_output_buttons = modified.held;
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            state.buttons_modified |= modified != physical;
                        }
                        // Mutation is confined to the verified controller
                        // fields, after policy evaluation. This is software
                        // input before user button mapping, not a physical HID
                        // report transmitted by a Pro Controller.
                        write_trace(controller, modified);
                        (*controller).left_stick_x = manual.stick[0];
                        (*controller).left_stick_y = manual.stick[1];
                        let triggers = kirby_assist_core::native_input::owned_trigger_axes(
                            physical_triggers,
                            manual.clear_autorepeat,
                        );
                        (*controller).left_trigger = triggers[0];
                        (*controller).right_trigger = triggers[1];
                        // Action-owning sequences suppress competing C-stick
                        // attacks. The facing helper only angles an existing
                        // physical down C-stick; restore both sticks afterward.
                        if kirby_assist_core::move_input::blocks_physical_c_stick(ledge.owned)
                            || mash_takeover
                        {
                            (*controller).right_stick_x = 0.0;
                            (*controller).right_stick_y = 0.0;
                        }
                        if winner == Some(Owner::Posture) {
                            if let Some(stick) =
                                configured_c_stick.map(|(_, right)| right).or_else(|| {
                                    kirby_assist_core::posture::Posture::aimed_c_stick(
                                        policy_buttons,
                                        physical_right_stick,
                                        posture.stick,
                                    )
                                })
                            {
                                (*controller).right_stick_x = stick[0];
                                (*controller).right_stick_y = stick[1];
                            }
                        }
                        if let Some(stick) = cancel_stick {
                            (*controller).right_stick_x = stick[0];
                            (*controller).right_stick_y = stick[1];
                        }
                        // Repeat flags can otherwise leak a held side-taunt
                        // even after held/pressed/released words were consumed.
                        (*controller).autorepeat[0] &= !manual.clear_autorepeat;
                        restore = Some(ControllerRestore {
                            controller,
                            physical,
                            stick: physical_stick,
                            right_stick: physical_right_stick,
                            autorepeat: physical_autorepeat,
                            triggers: physical_triggers,
                        });
                    }
                }
            }
        }
    }
    Some(MapperPreparation {
        restore,
        mapped_left_stick,
        mapped_automatic_request,
        mapped_auto_grab,
        mapped_reflect,
        mapped_reflector,
        mapped_auto_up_special,
        mapped_auto_dodge,
        mapped_dodge_punish_grab,
        mapped_attack_hold,
        mapped_literal_request,
        mapped_unshielded_auto_grab,
        mapped_sequence_right,
        reserved_dpad,
        tech_applied,
        mapped_tech_output,
        mapped_grab_handoff,
        sequence_short_hop,
        mapped_manual_jump,
        #[cfg(feature = "diagnostic-logging")]
        trump_pressed,
        #[cfg(feature = "diagnostic-logging")]
        ledge_edge,
        #[cfg(feature = "diagnostic-logging")]
        grab_delivery,
        #[cfg(feature = "diagnostic-logging")]
        grab_mapping_comparison,
        #[cfg(feature = "diagnostic-logging")]
        messages,
        #[cfg(feature = "diagnostic-logging")]
        mode_report,
    })
}
/// Finalizes the mapper overlay for the given output and preparation state, applying adjustments and recording delivery feedback.
pub(super) unsafe fn finish_mapper_overlay(
    output: *mut c_void,
    index: i32,
    session_ticket: SessionTicket,
    preparation: MapperPreparation,
) {
    let MapperPreparation {
        restore,
        mapped_left_stick,
        mapped_automatic_request,
        mapped_auto_grab,
        mapped_reflect,
        mapped_reflector,
        mapped_auto_up_special,
        mapped_auto_dodge,
        mapped_dodge_punish_grab,
        mapped_attack_hold,
        mapped_literal_request,
        mapped_unshielded_auto_grab,
        mapped_sequence_right,
        reserved_dpad,
        tech_applied,
        mapped_tech_output,
        mapped_grab_handoff,
        sequence_short_hop,
        mapped_manual_jump,
        #[cfg(feature = "diagnostic-logging")]
        trump_pressed,
        #[cfg(feature = "diagnostic-logging")]
        ledge_edge,
        #[cfg(feature = "diagnostic-logging")]
        grab_delivery,
        #[cfg(feature = "diagnostic-logging")]
        grab_mapping_comparison,
        #[cfg(feature = "diagnostic-logging")]
        mut messages,
        #[cfg(feature = "diagnostic-logging")]
        mode_report,
    } = preparation;
    #[cfg(feature = "diagnostic-logging")]
    let native_grab_mapping =
        if grab_mapping_comparison.is_some() && restore.is_some() && !output.is_null() {
            Some((*output.cast::<MappedInputs>()).buttons)
        } else {
            None
        };
    if restore.is_some() && !output.is_null() {
        apply_mapper_output_adjustments(
            output,
            MapperOutputAdjustments {
                left_stick: mapped_left_stick,
                right_stick: mapped_sequence_right,
                automatic_request: mapped_automatic_request,
                short_hop: sequence_short_hop,
                attack_hold: mapped_attack_hold,
                literal_request: mapped_literal_request,
                unshielded_auto_grab: mapped_unshielded_auto_grab,
                tech_output: mapped_tech_output,
            },
        );
        record_mapped_delivery(
            output,
            index,
            session_ticket,
            MapperDeliveryFeedback {
                manual_jump: mapped_manual_jump,
                reflect: mapped_reflect,
                reflector_tick: mapped_reflector,
                auto_grab: mapped_auto_grab,
                grab_handoff: mapped_grab_handoff,
                dodge_punish_grab: mapped_dodge_punish_grab,
                auto_dodge: mapped_auto_dodge,
                auto_up_special: mapped_auto_up_special,
                tech_applied,
                #[cfg(feature = "diagnostic-logging")]
                grab_delivery,
            },
            #[cfg(feature = "diagnostic-logging")]
            &mut messages,
        );
    }
    if reserved_dpad != 0 {
        let mapped = output.cast::<MappedInputs>();
        (*mapped).buttons = suppress_reserved_taunts((*mapped).buttons, reserved_dpad);
    }
    #[cfg(feature = "diagnostic-logging")]
    if let (
        Some((
            automatic,
            owner,
            requested,
            physical,
            applied,
            physical_stick,
            applied_stick,
            physical_triggers,
            applied_triggers,
        )),
        Some(native),
    ) = (grab_mapping_comparison, native_grab_mapping)
    {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "grab mapping comparison: source={}, owner={owner:?}, requested={requested:#x}, physical={physical:?}, applied={applied:?}, physical_stick={physical_stick:?}, applied_stick={applied_stick:?}, physical_triggers={physical_triggers:?}, applied_triggers={applied_triggers:?}, native_buttons={native:#x}, final_buttons={:#x}, final_left=({},{})",
            if automatic { "automatic" } else { "manual" },
            mapped.buttons,
            mapped.left_x,
            mapped.left_y,
        ));
    }
    #[cfg(feature = "diagnostic-logging")]
    if trump_pressed {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "trump mapped: buttons={:#x}, left=({},{})",
            mapped.buttons, mapped.left_x, mapped.left_y
        ));
    }
    #[cfg(feature = "diagnostic-logging")]
    if let Some(requested) = ledge_edge.filter(|_| grab_delivery.is_none()) {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "ledge mapper: raw_request={requested:#x}, mapped_buttons={:#x}, left=({},{})",
            mapped.buttons, mapped.left_x, mapped.left_y
        ));
        if requested & (MAPPER_X | MAPPER_A) == (MAPPER_X | MAPPER_A) {
            messages.push(format!(
                "short-hop mapped pair: jump={}, jump_mini={}, attack={}, neutral_stick={}; native ATTACK_AIR/attack_air_n still confirms acceptance",
                mapped.buttons & kirby_assist_core::native_input::MAPPED_JUMP != 0,
                mapped.buttons & kirby_assist_core::native_input::MAPPED_JUMP_MINI != 0,
                mapped.buttons & kirby_assist_core::native_input::MAPPED_ATTACK != 0,
                mapped.left_x == 0 && mapped.left_y == 0,
            ));
        }
    }
    // Restore every field we changed, including C-stick and repeat flags.
    // A new field added to the overlay must also be captured and restored here.
    // Clearing reserved taunts affects `output`, so restoring raw input does not
    // undo that filter or any other action the original mapper already produced.
    if let Some(saved) = restore {
        saved.restore();
    }
    #[cfg(feature = "diagnostic-logging")]
    if let Some(report) = mode_report {
        crate::logging::write_modes(report);
    }
    #[cfg(feature = "diagnostic-logging")]
    for message in messages {
        crate::logging::write_owned(message);
    }
}
