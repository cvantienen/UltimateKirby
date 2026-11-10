use super::*;
use kirby_assist_core::manual_controls::ManualFrame;
use kirby_assist_core::native_input::ButtonTrace;

/// Left-stick side flick bits of the native button word.
const LEFT_STICK_SIDE_FLICKS: u32 = (1 << 21) | (1 << 23);

fn mask_if(condition: bool, mask: u32) -> u32 {
    if condition {
        mask
    } else {
        0
    }
}

/// Whether `button` (canonical) is held in `trace` once translated to the physical layout.
fn physical_held(config: &Config, trace: ButtonTrace, button: u32) -> bool {
    trace.held & config.control_layout.physical_buttons(button) != 0
}

/// Outcome of owner arbitration for one poll.
struct Arbitration {
    winner: Option<Owner>,
    selected: ControllerOutput,
    situation: i32,
    tech_press: bool,
    tech_takeover: bool,
    mash_takeover: bool,
    #[cfg(feature = "diagnostic-logging")]
    intents: [Intent; 13],
}
/// Result of admitting a mapper callback: the immutable startup config, the
/// selected physical controller, and the match session this callback belongs to.
#[derive(Clone, Copy)]
pub(super) struct MapperAdmission {
    pub(super) config: &'static Config,
    controller: *mut controller::Controller,
    master_ready_go: Option<bool>,
    pub(super) session_ticket: SessionTicket,
}
/// Adjustments applied to the native mapper's result: stick input, automatic requests, and other control flags.
#[derive(Clone, Copy, Default)]
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

/// Feedback recorded after native mapping: which manual and automatic actions actually reached the final buttons.
#[derive(Clone, Copy, Default)]
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
    // vanilla passthrough, before any mapper state lock. Only the selected
    // controller refreshes the master match lock.
    let selected = selected_controller(wrapper, index, config.player_index).map(|controller| {
        let ready = observations::battle_menu_ready_go();
        refresh_master_match_lock(ready);
        (controller, ready)
    });
    settings_menu::poll_maintenance();
    if !RUNTIME_ENABLED.load(Ordering::Acquire) {
        return None;
    }

    // Only the configured physical controller may establish participation.
    let (controller, master_ready_go) = selected?;
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
        controller,
        master_ready_go,
        session_ticket,
    })
}
/// Enters the training passthrough mode for the admitted controller, returning `true` when the callback should stay a passthrough.
pub(super) unsafe fn enter_training_passthrough(admission: &MapperAdmission, index: i32) -> bool {
    training::enter_passthrough(
        trace(admission.controller),
        index,
        admission.config.control_layout,
    )
}

/// Applies the specified output adjustments to the mapper output, modifying stick input, automatic requests, and other control flags.
unsafe fn apply_mapper_output_adjustments(
    output: *mut c_void,
    adjustments: &MapperOutputAdjustments,
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
        if let Some((tick, target)) = auto_dodge {
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
    if guard && !state.last_mapped_guard {
        if let Some(tech) = state.tech_input.as_mut() {
            if tech_applied {
                tech.mapped_press();
            } else {
                tech.mapped_human_guard();
            }
        }
        #[cfg(feature = "diagnostic-logging")]
        if tech_applied
            && CONFIG
                .get()
                .is_some_and(|config| config.controls.tech_debug)
        {
            messages
                .push("tech mapped: fresh GUARD edge confirmed; request cooldown committed".into());
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
/// The selected controller's physical input, read before any assist is applied,
/// plus the static settings every overlay stage consults.
#[derive(Clone, Copy)]
struct OverlayFrame {
    controller: *mut controller::Controller,
    index: i32,
    config: &'static Config,
    physical: ButtonTrace,
    /// `physical.held` as seen by the Training compatibility layer.
    tested_held: u32,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
    physical_autorepeat: u32,
    physical_triggers: [f32; 2],
}

/// Whether this mapper slot's human fighter was observed recently and is the associated object.
fn local_route_current(state: &State, index: i32) -> bool {
    state.human_routes[index as usize].is_some_and(|(id, seen)| {
        Some(id) == state.associated_object && seen.elapsed() < SESSION_OBSERVATION_GAP
    })
}

/// Starts or ends the menu's battle session as the local fighter enters or leaves a live match.
fn track_menu_battle(state: &mut State, index: i32, connected: bool, menu_ready_go: Option<bool>) {
    let intro_current = state.menu_intro_object.is_some_and(|intro| {
        state.human_routes[index as usize]
            .is_some_and(|(id, seen)| id == intro && seen.elapsed() < SESSION_OBSERVATION_GAP)
    });
    let local_fighter_current = connected && local_route_current(state, index);
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
}

/// Binds mapper state to this slot, discarding anything learned from a previously selected slot.
unsafe fn bind_mapper_index(
    state: &mut State,
    frame: &OverlayFrame,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    if state.mapping_index != Some(frame.index) {
        state.cancel();
        state.associated_object = None;
        state.mapping_index = Some(frame.index);
        #[cfg(feature = "diagnostic-logging")]
        {
            state.mapper_confirmed = false;
            messages.push(format!(
                "selected local controller {} bound to mapper {} (npad={}, style={})",
                frame.config.player_index,
                frame.index,
                (*frame.controller).npad,
                (*frame.controller).style
            ));
        }
    }
    #[cfg(feature = "diagnostic-logging")]
    {
        state.mapper_confirmed = true;
    }
}

/// Updates the mapper menu and input state for the selected controller, handling menu readiness, battle activity, and diagnostic logging.
unsafe fn update_mapper_menu_and_input(
    frame: &OverlayFrame,
    menu_ready_go: Option<bool>,
    menu_training_mode: bool,
    #[cfg(feature = "diagnostic-logging")] diagnostics: &mut MapperDiagnostics,
) {
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    let connected = (*frame.controller).connected != 0;
    track_menu_battle(&mut state, frame.index, connected, menu_ready_go);
    if !connected {
        state.cancel();
        state.associated_object = None;
        state.mapping_index = None;
        return;
    }
    bind_mapper_index(
        &mut state,
        frame,
        #[cfg(feature = "diagnostic-logging")]
        &mut diagnostics.messages,
    );
    let policy_buttons = observe_raw_input(&mut state, frame);
    let kind = state.menu_fighter_kind;
    run_settings_menu(&mut state, frame, policy_buttons, menu_training_mode, kind);
    apply_menu_modes(&mut state, frame, kind);
    #[cfg(feature = "diagnostic-logging")]
    {
        diagnostics.trump_pressed = trump_input_pressed(&state, frame.physical);
    }
}

/// Records this poll's physical input, advances the local scheduling tick and
/// returns the canonical policy buttons.
fn observe_raw_input(state: &mut State, frame: &OverlayFrame) -> u32 {
    let OverlayFrame {
        config,
        physical,
        tested_held,
        physical_stick,
        physical_right_stick,
        ..
    } = *frame;
    // A new mapper call advances our local scheduling tick.
    // Other hooks can run several times within this tick;
    // evaluated_tick prevents them consuming extra pulses.
    let now = Instant::now();
    state.raw_tick = state.raw_tick.wrapping_add(1);
    state.raw_seen = Some(now);
    state.raw_connected = true;
    let takeover_buttons = config.control_layout.takeover_buttons(tested_held);
    let policy_buttons = config.control_layout.policy_buttons(tested_held);
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
    policy_buttons
}

/// Feeds this poll to the settings menu and publishes its consumed/native buttons.
fn run_settings_menu(
    state: &mut State,
    frame: &OverlayFrame,
    policy_buttons: u32,
    menu_training_mode: bool,
    kind: Option<i32>,
) {
    let OverlayFrame {
        config,
        physical,
        tested_held,
        ..
    } = *frame;
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
            tested_held,
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
    // L remains an assist trigger; Up remains available to bindings
    // diagonals. Menu-consumed sides/Down never reach tools.
    // Preserve Y for the first native-readiness check;
    // the later overlay still consumes its mapped action.
    let (policy_buttons, helper_consumed) = match state.settings_menu.as_ref() {
        Some(menu) => {
            menu.display_battle(state.menu_battle_active && kind.is_some());
            (menu.native_buttons, menu.consumed & !menu.builtin_consumed)
        }
        None => (policy_buttons, 0),
    };
    state.raw_buttons =
        kirby_assist_core::settings_menu::observation_buttons(policy_buttons, helper_consumed);
}

/// Applies the menu's mode switches: attack-source tracking, automatic-mode
/// suspension and the ledge-assist lifecycle.
fn apply_menu_modes(state: &mut State, frame: &OverlayFrame, kind: Option<i32>) {
    let OverlayFrame {
        config, physical, ..
    } = *frame;
    let menu = state.settings_menu.as_ref();
    let ledge_held =
        menu.is_some_and(|p| p.sequence_enabled(config.assist.enabled) && p.ledge_held);
    attack_sources::tracking(
        config.autopilot_shield && ledge_held
            || !ledge_held && menu.is_some_and(|m| m.reflect_config(kind.unwrap_or(-1)).is_some()),
    );
    let manual_actions = menu.map_or(0, |m| m.manual_native_actions());
    let ledge_inputs_reserved = ledge_held || menu.is_some_and(|m| m.reserves_automatic_modes());
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
        if let Some(slot) = state.settings_menu.as_ref().and_then(|m| m.preview_slot()) {
            let mut preview = LedgeTrap::default();
            preview.request_preview(slot);
            state.ledge = Some(preview);
        }
    }
}

/// Diagnostic only: a D-pad/X/Y press while the trump hold is configured.
#[cfg(feature = "diagnostic-logging")]
fn trump_input_pressed(state: &State, physical: ButtonTrace) -> bool {
    physical.pressed
        & (kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS
            | kirby_assist_core::native_input::MAPPER_X
            | MAPPER_Y)
        != 0
        && state
            .settings_menu
            .as_ref()
            .is_some_and(|m| m.ledge_trump_held())
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
/// Diagnostic-build reports collected while preparing the overlay and written after mapping.
#[cfg(feature = "diagnostic-logging")]
#[derive(Default)]
struct MapperDiagnostics {
    trump_pressed: bool,
    ledge_edge: Option<u32>,
    grab_mapping_comparison: Option<GrabMappingComparison>,
    messages: Vec<String>,
    mode_report: Option<kirby_assist_core::diagnostics::modes::Report>,
}

/// Overlay result handed from the pre-map stage to the post-map stage: the
/// saved physical controller, the final-button adjustments and feedback, and
/// the D-pad bits whose taunts must be filtered from the mapped output.
#[derive(Default)]
pub(super) struct MapperPreparation {
    pub(super) restore: Option<ControllerRestore>,
    adjustments: MapperOutputAdjustments,
    feedback: MapperDeliveryFeedback,
    reserved_dpad: u32,
    #[cfg(feature = "diagnostic-logging")]
    diagnostics: MapperDiagnostics,
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
    let ledge_acquired = ledge_priority || ledge_committed;
    // An idle always-on watch must not swallow Turbo movement. Acquired ledge
    // visits/tapes keep priority.
    let ledge_present = state.ledge_active
        && (state.action_owner.current_owner() == Some(Owner::Ledge)
            || !state.autopilot_guarding
                && (!inputs.turbo_active || ledge_acquired)
                && (state.oos_output.owned == 0 || ledge_acquired))
        && (state.ledge_output.owned != 0
            || state.ledge_output.stick.is_some()
            || state
                .ledge
                .as_ref()
                .is_none_or(|trap| !trap.manual_after_response()));
    [
        committed_intent(
            Owner::Reflect,
            state.reflect_output.owned != 0 || state.reflector.active(),
        ),
        // Preserve the automation reservation while R may be queued; a
        // neutral output still leaves movement manual.
        committed_intent(
            Owner::OutOfShield,
            state.oos_output.owned != 0
                || state.oos.grab_delivery_pending()
                || state.oos.awaiting_up_b(),
        ),
        committed_intent(
            Owner::Safety,
            state.tech_output.press
                || state.tech_output.owns_guard
                || inputs.mash_active
                || inputs.damage_active,
        ),
        committed_intent(Owner::Trip, state.trip_output.owned != 0),
        committed_intent(Owner::Fair, state.fair_output.owned != 0),
        committed_intent(Owner::Trump, state.trump_output.owned != 0),
        Intent {
            owner: Owner::Ledge,
            present: ledge_present,
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
        committed_intent(
            Owner::GrabDodge,
            state.spot_output.owned != 0
                && state
                    .spot_dodge
                    .as_ref()
                    .is_some_and(|s| s.target().is_some()),
        ),
        committed_intent(
            Owner::OptionRead,
            (state.spot_output.owned != 0
                || state
                    .spot_dodge
                    .as_ref()
                    .is_some_and(|s| s.punish_pending()))
                && state
                    .spot_dodge
                    .as_ref()
                    .is_none_or(|s| s.target().is_none()),
        ),
        passive_intent(
            Owner::Defense,
            state.autopilot_guarding && state.ledge_output.owned != 0,
        ),
        passive_intent(Owner::Turbo, inputs.turbo_active || inputs.pivot_active),
        passive_intent(Owner::Posture, inputs.posture_active),
    ]
}

/// An intent whose owner keeps its reservation until its own script finishes.
fn committed_intent(owner: Owner, present: bool) -> Intent {
    Intent {
        owner,
        present,
        committed: true,
    }
}
/// An intent that yields to every committed owner.
fn passive_intent(owner: Owner, present: bool) -> Intent {
    Intent {
        owner,
        present,
        committed: false,
    }
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
        Some(Owner::Ledge | Owner::Defense) => state.ledge_output,
        Some(Owner::RollRead) => state.roll_read_output,
        Some(Owner::GrabDodge | Owner::OptionRead) => state.spot_output,
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

/// A grounded Grab packet neutralises the C-stick so the native mapper cannot read it as a smash.
fn grab_release_right_stick(selected: ControllerOutput, situation: i32) -> Option<[f32; 2]> {
    (situation == kirby_assist_core::native_states::situation::GROUND
        && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0)
        .then_some([0.; 2])
}

/// The C-stick packet the winning script requests, for the owners that supply one.
fn owner_right_stick(state: &State, winner: Option<Owner>) -> Option<[f32; 2]> {
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

/// Determines the appropriate right stick input based on the winning owner, selected controller output, and current situation.
fn selected_owner_right_stick(
    state: &State,
    winner: Option<Owner>,
    selected: ControllerOutput,
    situation: i32,
) -> Option<[f32; 2]> {
    grab_release_right_stick(selected, situation).or_else(|| owner_right_stick(state, winner))
}

/// Represents a sampled posture, including configured C-stick inputs, controller output, and tilt aiming status.
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
struct DamageStickSample {
    di_stick: Option<[f32; 2]>,
    sdi_stick: Option<[f32; 2]>,
    damage_stick: Option<[f32; 2]>,
}
/// Handles manual ledge walking, returning the appropriate stick input if conditions are met.
fn manual_ledge_walk(
    state: &State,
    frame: &OverlayFrame,
    gate: &PollGate,
    turbo: TurboOutput,
) -> Option<[f32; 2]> {
    let OverlayFrame {
        config,
        physical_stick,
        physical_right_stick,
        ..
    } = *frame;
    let PollGate {
        policy_buttons,
        fresh,
        manual_scope,
        ..
    } = *gate;
    if !manual_scope
        || policy_buttons & MAPPER_L == 0
        || !state.ledge_active
        || state
            .settings_menu
            .as_ref()
            .is_some_and(|menu| menu.preview_slot().is_some())
    {
        return None;
    }
    let settings = state.settings_menu.as_ref().map_or_else(
        || settings_for_fighter(state.fighter_kind, config.assist),
        |p| p.settings(state.fighter_kind, &config.assist),
    );
    if !settings.enabled {
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
    if state.ledge_output.stick.is_none() && turbo.stick.is_none() {
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

/// The classic monitor chord passes through untouched before D-pad Down can
/// start a trump. This applies to standalone builds as well as Training; OS
/// overlays remain responsible for their own focus.
fn pass_monitor_chord(config: &Config, physical: ButtonTrace) -> bool {
    if !kirby_assist_core::settings_menu::monitor_chord(
        config.control_layout.policy_buttons(physical.held),
    ) {
        return false;
    }
    if let Ok(mut state) = STATE.lock() {
        state.cancel();
        state
            .settings_menu
            .get_or_insert_with(Default::default)
            .observe_passthrough(physical.held, config.control_layout);
    }
    true
}

/// Diagnostic only: records the first poll of every mapper index.
#[cfg(feature = "diagnostic-logging")]
unsafe fn note_observed_mapper(
    index: i32,
    controller: *mut controller::Controller,
    config: &Config,
    messages: &mut Vec<String>,
) {
    if SEEN_MAPPERS.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0 {
        messages.push(format!("mapper observed index {index}: npad={}, style={}, connected={}; configured local controller={}", (*controller).npad, (*controller).style, (*controller).connected, config.player_index));
    }
}

/// Pre-map stage: saves the physical controller, advances menu/route
/// bookkeeping, lets the winning tool overlay the controller, and returns what
/// the post-map stage must apply, record and restore. `None` leaves the call a
/// plain passthrough.
///
/// # Safety
/// `admission.controller` must be the live native controller of this callback and `output` the mapper's result buffer.
pub(super) unsafe fn prepare_mapper_overlay(
    admission: MapperAdmission,
    index: i32,
    output: *mut c_void,
    training_mode: Option<bool>,
    training_test_mode: bool,
) -> Option<MapperPreparation> {
    let MapperAdmission {
        config,
        controller,
        master_ready_go,
        session_ticket,
    } = admission;
    let mut prep = MapperPreparation::default();
    sync_diagnostic_options();
    if output.is_null() {
        return Some(prep);
    }
    #[cfg(feature = "diagnostic-logging")]
    note_observed_mapper(index, controller, config, &mut prep.diagnostics.messages);
    // Read physical input before applying any assist. Reusing last
    // output as "human input" would keep automated buttons latched.
    let physical = trace(controller);
    if pass_monitor_chord(config, physical) {
        return None;
    }
    let frame = OverlayFrame {
        controller,
        index,
        config,
        physical,
        tested_held: if training_test_mode {
            training_test::policy_buttons(physical.held)
        } else {
            physical.held
        },
        physical_stick: [(*controller).left_stick_x, (*controller).left_stick_y],
        physical_right_stick: [(*controller).right_stick_x, (*controller).right_stick_y],
        physical_autorepeat: (*controller).autorepeat[0],
        physical_triggers: [(*controller).left_trigger, (*controller).right_trigger],
    };
    // Without Training Modpack, query only the selected controller;
    // with it, reuse the earlier shortcut-policy scene read.
    let menu_training_mode = training_mode.unwrap_or_else(|| game::is_training_mode());
    update_mapper_menu_and_input(
        &frame,
        master_ready_go,
        menu_training_mode,
        #[cfg(feature = "diagnostic-logging")]
        &mut prep.diagnostics,
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
            compose_overlay(&mut state, &frame, &mut prep);
        }
    }
    Some(prep)
}

/// Runs every overlay stage for one admitted poll while holding the state lock.
unsafe fn compose_overlay(state: &mut State, frame: &OverlayFrame, prep: &mut MapperPreparation) {
    let gate = poll_gate(state, frame);
    #[cfg(feature = "diagnostic-logging")]
    track_input_expiry(state, &gate);
    let mut samples = sample_tools(state, frame, &gate, prep);
    let arbitration = arbitrate_owner(state, &gate, &samples, prep);
    samples.turbo_output.stick = winner_left_stick(&arbitration, &samples);
    state.cancel_non_owner(arbitration.winner);
    #[cfg(feature = "diagnostic-logging")]
    report_mode_snapshot(state, frame, &gate, &samples, &arbitration, prep);
    let request = compose_native_request(state, &gate, &samples, &arbitration, prep);
    let (manual, approach_pulse) =
        apply_manual_overlay(state, frame, &samples, &arbitration, &request);
    plan_stick_overlay(
        state,
        frame,
        &gate,
        &samples,
        &arbitration,
        &request,
        &manual,
        prep,
    );
    let modified = plan_delivery_feedback(
        state,
        frame,
        &arbitration,
        &samples,
        &manual,
        approach_pulse,
        prep,
    );
    #[cfg(feature = "diagnostic-logging")]
    report_overlay_diagnostics(state, frame, &gate, &arbitration, &manual, modified, prep);
    commit_overlay(
        state,
        frame,
        &gate,
        &samples,
        &arbitration,
        &request,
        &manual,
        modified,
        prep,
    );
}

/// Freshness and scope gates, computed once per poll.
#[derive(Clone, Copy)]
struct PollGate {
    policy_buttons: u32,
    fresh: bool,
    manual_scope: bool,
    native_grab: bool,
    mash_stick: Option<[f32; 2]>,
    #[cfg(feature = "diagnostic-logging")]
    now: Instant,
}

fn poll_gate(state: &State, frame: &OverlayFrame) -> PollGate {
    let OverlayFrame {
        index,
        config,
        physical,
        tested_held,
        ..
    } = *frame;
    // Recreate the canonical policy view in this lock scope.
    // The earlier copy configures observation/menu state, but
    // is intentionally scoped to that mapper-state update.
    let policy_buttons = config.control_layout.policy_buttons(tested_held);
    let policy_buttons = state
        .settings_menu
        .as_ref()
        .map_or(policy_buttons, |m| m.native_buttons);
    let now = Instant::now();
    let fresh =
        kirby_assist_core::observation::input_update_current(state.evaluated_tick, state.raw_tick)
            && state
                .request_seen
                .is_some_and(|seen| now.duration_since(seen) < REQUEST_LIFETIME);
    let manual_scope = local_route_current(state, index)
        && state.associated_object == state.observed_object
        && physical.held & MAPPER_PLUS == 0;
    // Expired requests become neutral. The overlay still
    // knows the previous assisted output, allowing a release.
    let native_grab = state
        .manual_context
        .is_some_and(|c| kirby_assist_core::action_owner::native_grab_active(c.status));
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
    PollGate {
        policy_buttons,
        fresh,
        manual_scope,
        native_grab,
        mash_stick,
        #[cfg(feature = "diagnostic-logging")]
        now,
    }
}

/// Diagnostic only: reports the transition into an expired-input state once.
#[cfg(feature = "diagnostic-logging")]
fn track_input_expiry(state: &mut State, gate: &PollGate) {
    let expired = gate.manual_scope
        && state.menu_battle_active
        && !gate.fresh
        && state.request_seen.is_some();
    if expired && !state.last_input_expired {
        crate::performance::health::event(crate::performance::health::Event::InputExpired);
    }
    state.last_input_expired = expired;
}

/// Each manual tool's proposal for this poll, before owner arbitration.
#[derive(Clone, Copy)]
struct ToolSamples {
    turbo_output: TurboOutput,
    turbo_stick: Option<[f32; 2]>,
    pivot_stick: Option<[f32; 2]>,
    manual_walk_stick: Option<[f32; 2]>,
    manual_context: Option<ManualContext>,
    posture: PostureSample,
    menu_consumed: u32,
    damage: DamageStickSample,
}

/// Samples the Turbo t-bag and dash-pivot helpers from the physical input.
fn sample_turbo_and_pivot(
    state: &mut State,
    frame: &OverlayFrame,
    gate: &PollGate,
) -> (TurboOutput, Option<[f32; 2]>) {
    let OverlayFrame {
        config,
        physical,
        physical_stick,
        physical_right_stick,
        ..
    } = *frame;
    let PollGate {
        policy_buttons,
        fresh,
        manual_scope,
        ..
    } = *gate;
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
    let turbo_output = state
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
            && physical.held & !kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS == 0
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
    (turbo_output, pivot_stick)
}

/// Samples every manual tool for this poll: turbo, dash pivot, ledge walk, posture and damage sticks.
fn sample_tools(
    state: &mut State,
    frame: &OverlayFrame,
    gate: &PollGate,
    prep: &mut MapperPreparation,
) -> ToolSamples {
    let OverlayFrame {
        config,
        physical,
        physical_stick,
        physical_right_stick,
        ..
    } = *frame;
    let PollGate {
        policy_buttons,
        fresh,
        manual_scope,
        mash_stick,
        ..
    } = *gate;
    let (turbo_output, pivot_stick) = sample_turbo_and_pivot(state, frame, gate);
    let turbo_stick = turbo_output.stick;
    let manual_walk_stick = manual_ledge_walk(state, frame, gate, turbo_output);
    let manual_context = state.manual_context;
    let posture = sample_mapper_posture(
        state,
        manual_context,
        manual_scope,
        fresh,
        policy_buttons,
        physical,
        physical_stick,
        physical_right_stick,
    );
    // Menu input is reserved during entry/respawn too.
    let menu_consumed = if state.menu_battle_active && state.menu_fighter_kind.is_some() {
        state.settings_menu.as_ref().map_or(0, |menu| menu.consumed)
    } else {
        0
    };
    prep.reserved_dpad |= menu_consumed & kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS;
    #[cfg(feature = "diagnostic-logging")]
    report_trump_input(state, gate, prep);
    let damage = sample_damage_sticks(
        state,
        config,
        manual_scope,
        fresh,
        mash_stick,
        physical_stick,
        #[cfg(feature = "diagnostic-logging")]
        &mut prep.diagnostics.messages,
    );
    ToolSamples {
        turbo_output,
        turbo_stick,
        pivot_stick,
        manual_walk_stick,
        manual_context,
        posture,
        menu_consumed,
        damage,
    }
}

/// Diagnostic only: describes the trump input of a poll that pressed its binding.
#[cfg(feature = "diagnostic-logging")]
fn report_trump_input(state: &State, gate: &PollGate, prep: &mut MapperPreparation) {
    if prep.diagnostics.trump_pressed {
        prep.diagnostics.messages.push(format!(
            "trump input: scope={}, fresh={}, action={}, stick={:?}, reserved={:#x}",
            gate.manual_scope,
            gate.fresh,
            state
                .trump
                .as_ref()
                .map_or("unobserved", LedgeTrump::action_name),
            state.trump_output.stick,
            prep.reserved_dpad,
        ));
    }
}

/// Resolves the owner of this poll's left stick and buttons, then records what the mapper hook must deliver for it.
fn arbitrate_owner(
    state: &mut State,
    gate: &PollGate,
    samples: &ToolSamples,
    prep: &mut MapperPreparation,
) -> Arbitration {
    let PollGate {
        fresh,
        manual_scope,
        native_grab,
        mash_stick,
        ..
    } = *gate;
    let ToolSamples {
        turbo_stick,
        pivot_stick,
        manual_context,
        posture: PostureSample {
            output: posture, ..
        },
        damage:
            DamageStickSample {
                di_stick,
                sdi_stick,
                damage_stick,
            },
        ..
    } = *samples;
    // Single final authority for sticks/buttons. Preparation
    // keeps ledge/roll timing ahead of every optional tool.
    // A detected ledge visit reserves its response against
    // general reads. Before acquisition they run normally.
    // Single final authority for sticks/buttons. Preparation
    // keeps ledge/roll timing ahead of every optional tool.
    // A detected ledge visit reserves its response against
    // general reads. Before acquisition they run normally.
    let mut intents = mapper_action_intents(
        state,
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
    let handoff_context = manual_context.filter(|_| fresh).map(|c| GameplayContext {
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
    prep.adjustments.short_hop = owner_native_short_hop(state, winner);
    let mut selected = selected_owner_output(state, winner, posture, handoff_situation);
    let selected_right = selected_owner_right_stick(state, winner, selected, handoff_situation);
    if !state.grab_handoff.permits(winner, selected, selected_right) {
        // Stop the old script, not just this packet; otherwise its
        // delayed B/Up step could resume after receipt expiry.
        if let Some(owner) = winner {
            state.action_owner.release(owner);
        }
        winner = None;
        selected = ControllerOutput::default();
    }
    let mut mapped_tech_output = None;
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
    let tech_takeover = winner == Some(Owner::Safety) && mapped_tech_output.is_some();
    let mash_takeover = winner == Some(Owner::Safety) && mash_stick.is_some();
    prep.feedback.grab_handoff = Some((
        state.raw_tick,
        winner,
        selected.requested & kirby_assist_core::native_input::MAPPER_R != 0,
        fresh
            && manual_scope
            && !native_grab
            && handoff_situation == kirby_assist_core::native_states::situation::GROUND
            && !kirby_assist_core::ledge_trap::sequence_interrupted(handoff_status),
    ));
    prep.adjustments.tech_output = mapped_tech_output;
    let tech_press = mapped_tech_output.is_some_and(|t| t.press);
    prep.feedback.tech_applied = tech_press;
    Arbitration {
        winner,
        selected,
        situation: handoff_situation,
        tech_press,
        tech_takeover,
        mash_takeover,
        #[cfg(feature = "diagnostic-logging")]
        intents,
    }
}

/// Whether the winning script asked the mapped jump to be shortened.
fn owner_native_short_hop(state: &State, winner: Option<Owner>) -> bool {
    match winner {
        Some(Owner::Ledge) => state.ledge_output.native_short_hop,
        Some(Owner::OutOfShield) => state.oos_output.native_short_hop,
        Some(Owner::OptionRead) => state.spot_output.native_short_hop,
        _ => false,
    }
}

/// The left-stick packet the winning owner supplies.
fn winner_left_stick(arbitration: &Arbitration, samples: &ToolSamples) -> Option<[f32; 2]> {
    let Arbitration {
        winner, selected, ..
    } = *arbitration;
    let ToolSamples {
        turbo_stick,
        pivot_stick,
        manual_walk_stick,
        ..
    } = *samples;
    match winner {
        Some(Owner::Safety) => selected.stick,
        Some(Owner::Turbo) => turbo_stick.or(pivot_stick),
        // An armed ledge wait reserves priority without an
        // attack stick. Keep its shared physical walk cap;
        // owned releases and active recipes must never fall through.
        Some(Owner::Ledge) if selected.owned == 0 && selected.requested == 0 => {
            selected.stick.or(manual_walk_stick)
        }
        None => manual_walk_stick,
        _ => selected.stick,
    }
}

/// Diagnostic only: reports a movement override and feeds the mode recorder.
#[cfg(feature = "diagnostic-logging")]
fn report_mode_snapshot(
    state: &mut State,
    frame: &OverlayFrame,
    gate: &PollGate,
    samples: &ToolSamples,
    arbitration: &Arbitration,
    prep: &mut MapperPreparation,
) {
    let OverlayFrame { physical_stick, .. } = *frame;
    let PollGate {
        now,
        fresh,
        manual_scope,
        mash_stick,
        ..
    } = *gate;
    let ToolSamples {
        turbo_output,
        manual_context,
        damage:
            DamageStickSample {
                di_stick,
                sdi_stick,
                ..
            },
        ..
    } = *samples;
    let Arbitration {
        winner,
        selected: ledge,
        ref intents,
        ..
    } = *arbitration;
    let messages = &mut prep.diagnostics.messages;
    use kirby_assist_core::diagnostics::modes::{bit, Snapshot};
    let status = manual_context.map_or(-1, |c| c.status);
    // Log only a change into a real assist override, not
    // every poll or native animation stop. No game reads.
    let blocker = if manual_scope
        && fresh
        && physical_stick[0].abs() >= 0.8
        && physical_stick[1].abs() < 0.3
        && turbo_output
            .stick
            .is_some_and(|s| s[0].abs() <= 0.2 || s[0] * physical_stick[0] < 0.0)
    {
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
        native_frame: manual_context.map_or(0, |c| c.frame.clamp(0.0, 65535.0) as u16),
        stopped: manual_context.is_some_and(|c| c.stopped),
        can_act: state.turbo_ready,
        hit: state.mode_hit,
        valid: fresh && manual_scope,
        safety: (if state.tech_output.press || state.tech_output.owns_guard {
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
        .min(u128::from(u64::MAX)) as u64;
    prep.diagnostics.mode_report = state.mode_diagnostics.sample(snapshot, ms);
}

/// The button masks and stick packets the winning owner asks the native mapper to treat as its input.
#[derive(Clone, Copy)]
struct NativeRequest {
    consumed: u32,
    requested: u32,
    owned: u32,
    cancel_stick: Option<[f32; 2]>,
    sequence_walking: bool,
    exclusive_sequence: bool,
}

/// The C-stick packet to force this poll: a grounded Grab release, or the
/// winning script's own stick while its input is current.
fn cancel_right_stick(
    state: &State,
    winner: Option<Owner>,
    selected: ControllerOutput,
    situation: i32,
    fresh: bool,
) -> Option<[f32; 2]> {
    if let Some(release) = grab_release_right_stick(selected, situation) {
        return Some(release);
    }
    let current = fresh
        && match winner {
            Some(Owner::RollRead | Owner::OptionRead) => selected.owned != 0,
            Some(Owner::OutOfShield) => state.oos_output.owned != 0,
            Some(Owner::Ledge) => true,
            _ => false,
        };
    current.then(|| owner_right_stick(state, winner)).flatten()
}

fn compose_native_request(
    state: &State,
    gate: &PollGate,
    samples: &ToolSamples,
    arbitration: &Arbitration,
    prep: &mut MapperPreparation,
) -> NativeRequest {
    let PollGate { fresh, .. } = *gate;
    let ToolSamples {
        turbo_output,
        manual_walk_stick,
        menu_consumed,
        posture: PostureSample { tilt_aiming, .. },
        ..
    } = *samples;
    let Arbitration {
        winner,
        selected,
        situation: handoff_situation,
        tech_press,
        tech_takeover,
        mash_takeover,
        ..
    } = *arbitration;
    // A held C-stick can otherwise inject an attack while
    // left-stick-only mashing. Block its digital aliases too.
    let blocked_stick_buttons = if mash_takeover {
        kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
            | kirby_assist_core::manual_controls::STICK_CLICKS
    } else {
        0
    };
    let cancel_stick = cancel_right_stick(state, winner, selected, handoff_situation, fresh);
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
        Some(Owner::RollRead) => state.rule_engine.as_ref().is_some_and(|r| r.walking()),
        // A moving grab retains raw horizontal input; it is not a file Walk.
        Some(Owner::OutOfShield) => false,
        Some(Owner::OptionRead) => state.spot_dodge.as_ref().is_some_and(|s| s.walking()),
        _ => false,
    };
    let exclusive_sequence = selected.owned & kirby_assist_core::native_input::GAMEPLAY_BUTTONS
        == kirby_assist_core::native_input::GAMEPLAY_BUTTONS;
    if exclusive_sequence {
        prep.reserved_dpad |= kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS;
    }
    let owned_direction_flick = !exclusive_sequence
        && selected.owned & kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS != 0
        && !mash_takeover
        && !matches!(winner, Some(Owner::Trip | Owner::Fair))
        && !(winner == Some(Owner::Posture) && tilt_aiming);
    let walk_cap_flick = manual_walk_stick.is_some()
        && turbo_output.stick == manual_walk_stick
        && selected.owned == 0
        && !mash_takeover;
    let consumed = kirby_assist_core::native_input::sequence_flick_consumption(
        selected.owned,
        sequence_walking,
    ) | prep.reserved_dpad
        | menu_consumed
        | mask_if(
            owned_direction_flick || walk_cap_flick,
            LEFT_STICK_SIDE_FLICKS,
        );
    let requested =
        kirby_assist_core::native_input::selected_requests(selected.owned, selected.requested, 0);
    // Deliver the native jump pulse through the same raw
    // history as every other simulated input. The final
    // mapped flag supplements this path; it does not replace it.
    if exclusive_sequence {
        prep.adjustments.automatic_request = Some(requested);
    }
    prep.feedback.manual_jump = winner == Some(Owner::Ledge)
        && requested & kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS != 0;
    let owned = selected.owned
        | mask_if(tech_takeover, state.tech_output.owned_buttons())
        | mask_if(tech_press, MAPPER_ZR)
        | mask_if(
            cancel_stick.is_some(),
            kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS,
        )
        | blocked_stick_buttons;
    // Keep source consumption above; only the final taunt
    // filter yields to an explicitly owned sequence D-pad.
    prep.reserved_dpad = kirby_assist_core::manual_controls::sequence_taunt_reservation(
        prep.reserved_dpad,
        selected.requested,
        selected.owned,
    );
    NativeRequest {
        consumed,
        requested,
        owned,
        cancel_stick,
        sequence_walking,
        exclusive_sequence,
    }
}

/// Runs the shared manual overlay over the physical input and returns its
/// frame plus whether Out of Shield is mid approach pulse.
fn apply_manual_overlay(
    state: &mut State,
    frame: &OverlayFrame,
    samples: &ToolSamples,
    arbitration: &Arbitration,
    request: &NativeRequest,
) -> (ManualFrame, bool) {
    let OverlayFrame {
        config,
        physical,
        physical_stick,
        ..
    } = *frame;
    let turbo_output = samples.turbo_output;
    let Arbitration {
        winner, tech_press, ..
    } = *arbitration;
    let NativeRequest {
        consumed: canonical_consumed,
        requested: canonical_requested,
        owned: canonical_owned,
        ..
    } = *request;
    let approach_pulse = winner == Some(Owner::OutOfShield) && state.oos.approach_pending();
    if tech_press && !state.last_mapped_guard {
        state
            .manual_overlay
            .get_or_insert_with(ManualOverlay::default)
            .rearm_tech_guard(config.control_layout.physical_buttons(MAPPER_ZR), physical);
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
    (manual, approach_pulse)
}

/// Chooses the analog sticks and final-button adjustments the mapper hook applies after native mapping.
fn plan_stick_overlay(
    state: &State,
    frame: &OverlayFrame,
    gate: &PollGate,
    samples: &ToolSamples,
    arbitration: &Arbitration,
    request: &NativeRequest,
    manual: &ManualFrame,
    prep: &mut MapperPreparation,
) {
    let OverlayFrame { config, .. } = *frame;
    let PollGate {
        fresh,
        manual_scope,
        ..
    } = *gate;
    let ToolSamples {
        turbo_output,
        turbo_stick,
        manual_walk_stick,
        ..
    } = *samples;
    let Arbitration {
        winner, selected, ..
    } = *arbitration;
    let NativeRequest {
        requested: canonical_requested,
        cancel_stick,
        sequence_walking,
        exclusive_sequence,
        ..
    } = *request;
    let adjustments = &mut prep.adjustments;
    if fresh && manual_scope && sequence_walking {
        adjustments.left_stick = selected.stick;
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
        adjustments.left_stick = turbo_output.stick;
    }
    // Preserve the selected literal A packet, including directional
    // smashes and one-frame presses. Plan metadata used to retain
    // only neutral multi-frame holds, losing A while the stick survived.
    adjustments.attack_hold = fresh
        && manual_scope
        && matches!(
            winner,
            Some(Owner::Ledge | Owner::RollRead | Owner::OutOfShield | Owner::OptionRead)
        )
        && selected.owned & kirby_assist_core::native_input::MAPPER_A != 0
        && selected.requested & kirby_assist_core::native_input::MAPPER_A != 0
        && physical_held(
            config,
            manual.buttons,
            kirby_assist_core::native_input::MAPPER_A,
        );
    // Forward all exclusive sequence axes identically. Drift
    // and Run must not depend on the native mapper retaining
    // an analog value after physical flick history was masked.
    if let Some(stick) =
        kirby_assist_core::native_input::exclusive_sequence_stick(selected.owned, selected.stick)
    {
        adjustments.left_stick = Some(stick);
    }
    if exclusive_sequence {
        adjustments.right_stick = Some(cancel_stick.unwrap_or([0.; 2]));
    }
    if fresh && manual_scope && exclusive_sequence && winner.is_some() {
        adjustments.literal_request = Some(
            [
                kirby_assist_core::native_input::MAPPER_A,
                kirby_assist_core::native_input::MAPPER_B,
                kirby_assist_core::native_input::MAPPER_R,
                kirby_assist_core::native_input::MAPPER_ZR,
            ]
            .into_iter()
            .fold(0, |mask, button| {
                mask | mask_if(
                    canonical_requested & selected.owned & button != 0
                        && physical_held(config, manual.buttons, button),
                    button,
                )
            }),
        );
        adjustments.unshielded_auto_grab =
            winner == Some(Owner::OutOfShield) && state.oos.unshielded_grab_started();
    }
}

/// Records which automatic actions the final buttons must confirm, and
/// returns the button history after posture side-tilt aliasing.
fn plan_delivery_feedback(
    state: &mut State,
    frame: &OverlayFrame,
    arbitration: &Arbitration,
    samples: &ToolSamples,
    manual: &ManualFrame,
    approach_pulse: bool,
    prep: &mut MapperPreparation,
) -> ButtonTrace {
    let OverlayFrame { config, .. } = *frame;
    let Arbitration {
        winner, selected, ..
    } = *arbitration;
    let configured_c_stick = samples.posture.configured_c_stick;
    let feedback = &mut prep.feedback;
    let posture_side = if winner == Some(Owner::Posture) {
        configured_c_stick.and_then(|(_, right)| (right[1] == 0.0).then_some(right[0]))
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
    let r_delivered = selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
        && physical_held(config, modified, kirby_assist_core::native_input::MAPPER_R);
    let b_delivered = selected.requested & kirby_assist_core::native_input::MAPPER_B != 0
        && physical_held(config, modified, kirby_assist_core::native_input::MAPPER_B);
    if winner == Some(Owner::Reflect) {
        feedback.reflector_tick = Some(state.raw_tick);
    }
    feedback.reflect = winner == Some(Owner::Reflect) && b_delivered;
    if winner == Some(Owner::OutOfShield) && r_delivered {
        feedback.auto_grab = state
            .oos
            .target()
            .map(|target| (state.raw_tick, target, true));
    } else if approach_pulse && selected.stick.is_some_and(|s| s[0].abs() >= 0.95) {
        feedback.auto_grab = state
            .oos
            .target()
            .map(|target| (state.raw_tick, target, false));
    }
    if winner == Some(Owner::OptionRead) && r_delivered {
        feedback.dodge_punish_grab = state
            .spot_dodge
            .as_ref()
            .and_then(|s| s.follow_target())
            .map(|id| (state.raw_tick, id));
    }
    if winner == Some(Owner::GrabDodge)
        && selected.requested & MAPPER_ZR != 0
        && selected.stick.is_some_and(|s| s[1] < -0.5)
    {
        feedback.auto_dodge = state
            .spot_dodge
            .as_ref()
            .and_then(|s| s.target())
            .map(|target| (state.raw_tick, target));
    }
    if winner == Some(Owner::OutOfShield) && state.oos.shield_up_b_pending() && b_delivered {
        feedback.auto_up_special = state
            .oos
            .target()
            .map(|target| (state.raw_tick, target, Owner::OutOfShield));
    } else if b_delivered {
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
        feedback.auto_up_special = target
            .zip(winner)
            .map(|(target, owner)| (state.raw_tick, target, owner));
    }
    modified
}

/// Diagnostic only: grab-delivery traces, ledge edge, queued logs and the grab mapping comparison.
#[cfg(feature = "diagnostic-logging")]
fn report_overlay_diagnostics(
    state: &mut State,
    frame: &OverlayFrame,
    gate: &PollGate,
    arbitration: &Arbitration,
    manual: &ManualFrame,
    modified: ButtonTrace,
    prep: &mut MapperPreparation,
) {
    let OverlayFrame {
        config,
        physical,
        physical_stick,
        physical_triggers,
        ..
    } = *frame;
    let PollGate {
        fresh,
        manual_scope,
        ..
    } = *gate;
    let Arbitration {
        winner,
        selected,
        tech_press,
        mash_takeover,
        ..
    } = *arbitration;
    let diagnostics = &mut prep.diagnostics;
    let messages = &mut diagnostics.messages;
    if state.grab_trace.active() || state.calibration.active() {
        prep.feedback.grab_delivery = Some((
            state.ledge_output.requested,
            modified.held,
            modified.pressed,
            if !fresh {
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
    if selected.requested & modified.pressed != 0 {
        diagnostics.ledge_edge = Some(selected.requested);
    }
    let grab_button = config
        .control_layout
        .physical_buttons(kirby_assist_core::native_input::MAPPER_R);
    let automatic = winner == Some(Owner::OutOfShield)
        && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
        && modified.held & grab_button != 0;
    let manual_grab = physical.held & grab_button != 0
        && selected.requested & kirby_assist_core::native_input::MAPPER_R == 0;
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
            diagnostics.grab_mapping_comparison = Some((
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
    messages.extend(state.pending_combat_log.take());
    messages.extend(state.pending_tech_log.take());
    messages.extend(state.pending_ledge_log.take());
    if config.controls.tech_debug
        && state.last_logged_scenario.1
        && modified.pressed & MAPPER_ZR != 0
        && tech_press
    {
        messages.push(format!(
            "tech ZR applied: input_update={}, held={:#x}, previous={:#x}, pressed={:#x}",
            state.raw_tick, modified.held, modified.previous, modified.pressed
        ));
        messages.push(format!(
            "tech request timing: {:?}",
            state
                .tech_input
                .as_ref()
                .and_then(TechInput::last_observation)
        ));
    }
}

/// Publishes the final buttons and sticks into the native controller and records what to restore.
unsafe fn commit_overlay(
    state: &mut State,
    frame: &OverlayFrame,
    gate: &PollGate,
    samples: &ToolSamples,
    arbitration: &Arbitration,
    request: &NativeRequest,
    manual: &ManualFrame,
    modified: ButtonTrace,
    prep: &mut MapperPreparation,
) {
    let OverlayFrame {
        controller,
        physical,
        physical_stick,
        physical_right_stick,
        physical_autorepeat,
        physical_triggers,
        ..
    } = *frame;
    let policy_buttons = gate.policy_buttons;
    let PostureSample {
        configured_c_stick,
        output: posture,
        ..
    } = samples.posture;
    let Arbitration {
        winner,
        selected,
        mash_takeover,
        ..
    } = *arbitration;
    let cancel_stick = request.cancel_stick;
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
    let blocks_c_stick =
        kirby_assist_core::move_input::blocks_physical_c_stick(selected.owned) || mash_takeover;
    let posture_c_stick = if winner == Some(Owner::Posture) {
        configured_c_stick.map(|(_, right)| right).or_else(|| {
            kirby_assist_core::posture::Posture::aimed_c_stick(
                policy_buttons,
                physical_right_stick,
                posture.stick,
            )
        })
    } else {
        None
    };
    if let Some(stick) = cancel_stick
        .or(posture_c_stick)
        .or_else(|| blocks_c_stick.then_some([0.; 2]))
    {
        (*controller).right_stick_x = stick[0];
        (*controller).right_stick_y = stick[1];
    }
    // Repeat flags can otherwise leak a held side-taunt
    // even after held/pressed/released words were consumed.
    (*controller).autorepeat[0] &= !manual.clear_autorepeat;
    prep.restore = Some(ControllerRestore {
        controller,
        physical,
        stick: physical_stick,
        right_stick: physical_right_stick,
        autorepeat: physical_autorepeat,
        triggers: physical_triggers,
    });
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
        adjustments,
        feedback,
        reserved_dpad,
        #[cfg(feature = "diagnostic-logging")]
        diagnostics,
    } = preparation;
    #[cfg(feature = "diagnostic-logging")]
    let grab_delivery = feedback.grab_delivery;
    #[cfg(feature = "diagnostic-logging")]
    let MapperDiagnostics {
        trump_pressed,
        ledge_edge,
        grab_mapping_comparison,
        mut messages,
        mode_report,
    } = diagnostics;
    #[cfg(feature = "diagnostic-logging")]
    let native_grab_mapping =
        if grab_mapping_comparison.is_some() && restore.is_some() && !output.is_null() {
            Some((*output.cast::<MappedInputs>()).buttons)
        } else {
            None
        };
    if restore.is_some() && !output.is_null() {
        apply_mapper_output_adjustments(output, &adjustments);
        record_mapped_delivery(
            output,
            index,
            session_ticket,
            feedback,
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
