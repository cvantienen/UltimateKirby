//! Hold-to-run D-pad controls, independent of native pointers and Skyline.
//! Left/right are absolute stick directions, not the fighter's facing direction.
//! The caller captures physical input first and restores native memory after
//! mapping; this module only proposes a trace and left-stick position.
//!
//! Reading order: `DpadReservation` consumes reserved physical triggers,
//! `TurboState` proposes its movement, then `ManualOverlay::apply_controls`
//! combines one chosen stick proposal with button ownership. A button mask
//! names several buttons at once; `owned` includes intentional releases.
use crate::native_input::{ButtonTrace, MapperButtonOverlay};

mod jump_fence;
pub use jump_fence::ManualJumpFence;
mod assist_handoff;
pub use assist_handoff::AssistGuardHandoff;
mod shortcuts;
pub use shortcuts::{ShortcutContext, Shortcuts};
pub const TURBO_TRIGGER: u32 = 1 << 16; // Canonical turbo role; physical source comes from button bindings.
pub const SHORT_HOP_TRIGGER: u32 = crate::native_input::MAPPER_X; // Canonical short-hop role; physical source is remappable.
pub const DPAD_UP: u32 = 1;
pub const DPAD_RIGHT: u32 = 1 << 1;
pub const DPAD_LEFT: u32 = 1 << 3;
pub const DPAD_SIDES: u32 = DPAD_LEFT | DPAD_RIGHT;
pub const LEFT_STICK_DIRECTIONS: u32 = 0xf << 20;
pub const RIGHT_STICK_DIRECTIONS: u32 = 0xf << 24;
pub const STICK_CLICKS: u32 = 3 << 16;

/// Preserve fine physical input and crouching while capping horizontal motion
/// for manual ledge spacing. Attack recipes bypass this cap explicitly.
pub fn ledge_walk_stick(physical: [f32; 2], maximum: f32) -> [f32; 2] {
    let base = if maximum.is_finite() {
        maximum.clamp(0.1, 0.45)
    } else {
        0.35
    };
    // The existing setting remains the baseline for old config files. Use a
    // slightly lower cap normally, with a fixed fast-walk request near full
    // horizontal. The overlay also clears digital stick-flick aliases.
    let full_horizontal = physical[0].is_finite()
        && physical[1].is_finite()
        && physical[0].abs() >= 0.95
        && physical[1].abs() <= 0.15;
    let cap = if full_horizontal {
        0.5 // Full analog deflection can dash even with digital flick aliases cleared.
    } else {
        (base - 0.02).clamp(0.1, 0.45)
    };
    let x = if physical[0].is_finite() {
        physical[0].clamp(-cap, cap)
    } else {
        0.0
    };
    let y = if physical[1].is_finite() {
        physical[1].clamp(-1.0, 1.0)
    } else {
        0.0
    };
    [x, y]
}

/// Release only outward manual walking input at a known ledge gap.
/// This uses copied motion, not geometry searches or native position writes.
pub fn stop_walk_at_ledge(
    mut stick: [f32; 2],
    position: [f32; 2],
    velocity: [f32; 2],
    spot: crate::ledge_position::LedgeSpot,
    stand_distance: f32,
) -> [f32; 2] {
    if !position
        .iter()
        .chain(velocity.iter())
        .chain(spot.edge.iter())
        .all(|v| v.is_finite())
        || !spot.inward.is_finite()
        || spot.inward.abs() != 1.0
        || !stand_distance.is_finite()
        || stand_distance < 1.0
        || (position[1] - spot.edge[1]).abs() > 6.0
        || !stick[0].is_finite()
        || stick[0] * spot.inward >= 0.0
    {
        return stick;
    }
    let clearance = (position[0] - spot.edge[0]) * spot.inward;
    let closing_speed = (-velocity[0] * spot.inward).max(0.0);
    if clearance - closing_speed * 3.0 <= stand_distance + 0.1 {
        stick[0] = 0.0;
    }
    stick
}

/// A diagonal press can include Up even when the intended trigger is Left/Right.
/// Quarantine all D-pad bits involved in a macro press until their release, so
/// a diagonal or its release cannot turn into a different taunt.
#[derive(Default)]
pub struct DpadReservation {
    quarantined: u32,
}
impl DpadReservation {
    pub fn sample(&mut self, enabled: u32, physical: u32, gameplay: bool) -> u32 {
        if !gameplay || enabled == 0 {
            *self = Self::default();
            return 0;
        }
        if physical & enabled != 0 {
            self.quarantined |= physical & 0xf;
        }
        // Include the old quarantine BEFORE dropping released bits: this poll
        // must swallow the release edge too. Enabled triggers stay reserved even
        // while the fighter is too busy to execute the corresponding macro.
        let consumed = enabled | self.quarantined;
        self.quarantined &= physical;
        consumed
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TurboSettings {
    pub enabled: bool,
    /// Number of observed input updates in each crouch/release phase.
    pub phase_frames: u16,
    /// Normalized horizontal stick magnitude during the release/move phase.
    pub walk_strength: f32,
}
impl Default for TurboSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            phase_frames: 2,
            walk_strength: 0.45,
        }
    }
}

/// Grounded idle/walk/run/turn/crouch states qualify. Do not change game
/// statuses to force the macro through attacks, grabs, hitstop, or aerial states.
pub fn turbo_movement_allowed(status: i32, situation: i32, stopped: bool) -> bool {
    situation == 0 && !stopped && ((0..=9).contains(&status) || (0x11..=0x15).contains(&status))
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TurboOutput {
    pub consume_dpad: bool,
    pub stick: Option<[f32; 2]>,
}

/// Cleans up a deliberate full-stick reversal during a native dash/run. The
/// two-update hold is advanced by the shared game tick, never mapper polls, so
/// online input sampling cannot shorten the pivot window.
#[derive(Default)]
pub struct DashPivotAssist {
    last_direction: i8,
    active_direction: i8,
    dash_walk_direction: i8,
    updates_left: u8,
    last_tick: Option<i64>,
    last_output: TurboOutput,
}
impl DashPivotAssist {
    /// A higher-priority tool must not leave movement queued for handback.
    pub fn cancel_pending(&mut self) {
        *self = Self::default();
    }
    pub fn sample(
        &mut self,
        tick: i64,
        status: i32,
        situation: i32,
        stopped: bool,
        physical_stick: [f32; 2],
    ) -> TurboOutput {
        // Revalidate before the duplicate cache: landing/stop/damage and fresh
        // physical vertical input can change between mapper polls online.
        if situation != 0
            || stopped
            || !(0..=8).contains(&status)
            || !physical_stick.iter().all(|v| v.is_finite())
            || physical_stick[1].abs() > 0.3
            || physical_stick[0].abs() < 0.1
        {
            *self = Self::default();
            return TurboOutput::default();
        }
        if self
            .last_tick
            .is_some_and(|last| tick < last || tick.saturating_sub(last) > 1)
        {
            *self = Self::default();
        }
        if self.last_tick == Some(tick) {
            // A duplicate native tick can still contain newer physical input
            // online. Never replay an old reversal or walk cap against it.
            if self.last_output.stick.is_some_and(|stick| {
                stick[0] * physical_stick[0] < 0.
                    || (stick[0].abs() >= 0.8) != (physical_stick[0].abs() >= 0.8)
                    || stick[0].abs() >= 0.8 && !(3..=8).contains(&status)
            }) {
                self.cancel_pending();
                return TurboOutput::default();
            }
            return self.last_output;
        }
        let x = physical_stick[0];
        let direction = if x >= 0.8 {
            1
        } else if x <= -0.8 {
            -1
        } else {
            0
        };
        // Native common statuses 3..=8 cover dash/turn-dash/run/brake/turn-run.
        // Do not reshape ordinary walking, crouching, attacks, or aerial input.
        let movement_status = (3..=8).contains(&status);
        if !movement_status || direction == 0 {
            self.updates_left = 0;
            self.active_direction = 0;
        }
        if movement_status
            && direction != 0
            && self.last_direction != 0
            && direction == -self.last_direction
        {
            self.active_direction = direction;
            self.updates_left = 2;
        }
        // A full dash followed by a deliberate same-direction stick ease is a
        // dash-walk gesture. Keep the eased input at the fastest value below
        // the digital dash threshold; a continuously full stick still runs.
        if status == 3 && direction != 0 {
            self.dash_walk_direction = direction;
        }
        let gentle_direction = if (0.1..0.8).contains(&x) {
            1
        } else if (-0.8..-0.1).contains(&x) {
            -1
        } else {
            0
        };
        let dash_walking = self.dash_walk_direction != 0
            && gentle_direction == self.dash_walk_direction
            && (0..=8).contains(&status);
        if x.abs() < 0.1
            || direction != 0 && direction != self.dash_walk_direction
            || gentle_direction != 0 && gentle_direction != self.dash_walk_direction
        {
            self.dash_walk_direction = 0;
        }
        if direction != 0 {
            self.last_direction = direction;
        }
        let output = if self.updates_left != 0 {
            self.updates_left -= 1;
            TurboOutput {
                consume_dpad: false,
                stick: Some([f32::from(self.active_direction), 0.0]),
            }
        } else if dash_walking {
            TurboOutput {
                consume_dpad: false,
                stick: Some([f32::from(self.dash_walk_direction) * 0.49, 0.0]),
            }
        } else {
            TurboOutput::default()
        };
        self.last_tick = Some(tick);
        self.last_output = output;
        output
    }
}

#[derive(Default)]
pub struct TurboState {
    direction: i8,
    next_phase: u32,
    last_tick: Option<i64>,
    last_output: TurboOutput,
}
impl TurboState {
    /// Held L3 restores the original crouch/walk cancel with one native update
    /// per phase. The gentle facing-direction walk is intentional: neutral-only
    /// release lets the ordinary crouch recovery play instead of this turbo.
    /// The legacy phase setting does not slow this dedicated turbo binding.
    /// `tick` identifies a selected local INPUT UPDATE, not a fighter animation
    /// or a crouch acknowledgment. Online delay postpones observed poses; using
    /// them here stretches every phase by that delay. Repeated IDs reuse output.
    pub fn sample_teabag(
        &mut self,
        settings: TurboSettings,
        tick: i64,
        held: bool,
        facing: f32,
        gameplay: bool,
        can_move: bool,
    ) -> TurboOutput {
        self.sample_facing(
            TurboSettings {
                phase_frames: 1,
                ..settings
            },
            tick,
            held,
            facing,
            gameplay,
            can_move,
        )
    }

    /// Facing-direction crouch/walk oscillator, also reused by L3 turbo.
    pub fn sample_facing(
        &mut self,
        settings: TurboSettings,
        tick: i64,
        held: bool,
        facing: f32,
        gameplay: bool,
        can_move: bool,
    ) -> TurboOutput {
        let direction = if held && facing.is_finite() && facing.abs() >= 0.5 {
            if facing < 0.0 {
                DPAD_LEFT
            } else {
                DPAD_RIGHT
            }
        } else {
            0
        };
        let mut out = self.sample(settings, tick, direction, gameplay, can_move);
        // Start creeping on the first crouch phase too, rather than spending
        // the initial updates planted. Native crouch/walk rules still apply.
        if let Some(stick) = out.stick.as_mut() {
            let strength = if settings.walk_strength.is_finite() {
                settings.walk_strength.clamp(0.05, 0.45)
            } else {
                0.25
            };
            stick[0] = if direction == DPAD_LEFT {
                -strength
            } else {
                strength
            };
        }
        out.consume_dpad = false;
        out
    }

    /// A held trigger repeats; an edge-triggered "just pressed" check would only
    /// run once. Both directions held cancel movement and still suppress taunts.
    pub fn sample(
        &mut self,
        settings: TurboSettings,
        tick: i64,
        physical: u32,
        gameplay: bool,
        can_move: bool,
    ) -> TurboOutput {
        if !settings.enabled || !gameplay {
            *self = Self::default();
            return TurboOutput::default();
        }
        // Match the held state, not pressed edges: releasing the D-pad stops
        // the macro immediately. Both-side and neither-side masks choose zero.
        let direction = match physical & DPAD_SIDES {
            DPAD_LEFT => -1,
            DPAD_RIGHT => 1,
            _ => 0,
        };
        let mut output = TurboOutput {
            consume_dpad: true,
            stick: None,
        };
        if direction == 0 || !can_move {
            *self = Self::default();
            return output;
        }
        if self.direction != direction || self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
            self.direction = direction;
        }
        if self.last_tick == Some(tick) {
            return self.last_output;
        }
        // next_phase starts at zero, so the FIRST eligible poll crouches.
        // With phase_frames=2: crouch, crouch, move, move, repeat. A direction
        // switch resets to zero above, while a repeated tick reuses its output.
        let crouching =
            (self.next_phase / u32::from(settings.phase_frames.max(1))).is_multiple_of(2);
        // Never propagate NaN into native controller memory if a direct caller
        // bypasses the validated text parser.
        let strength = if settings.walk_strength.is_finite() {
            settings.walk_strength.clamp(0.05, 0.6)
        } else {
            0.25
        };
        output.stick = Some(if crouching {
            [0.0, -1.0]
        } else {
            [f32::from(direction) * strength, 0.0]
        });
        let period = u32::from(settings.phase_frames.max(1)) * 2;
        self.next_phase = (self.next_phase + 1) % period;
        self.last_tick = Some(tick);
        self.last_output = output;
        output
    }
}

/// Result consumed by the native mapper; tech/mash button ownership is composed
/// first. The shared stick overlay accepts analog mash as well as D-pad tools,
/// owning stick/attack/shield fields only while the active policy requests them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManualFrame {
    pub buttons: ButtonTrace,
    pub stick: [f32; 2],
    pub clear_autorepeat: u32,
}
#[derive(Default)]
pub struct ManualOverlay {
    buttons: MapperButtonOverlay,
    had_stick_override: bool,
    reaction_owned: bool,
}
impl ManualOverlay {
    /// Native Guard can have been removed after raw mapping by an earlier mode.
    /// A scheduler-confirmed new tech edge must not inherit that raw held bit.
    pub fn rearm_tech_guard(&mut self, mask: u32, human: ButtonTrace) {
        self.buttons.rearm_requested(mask, human.previous);
    }

    pub fn apply(
        &mut self,
        human: ButtonTrace,
        physical_stick: [f32; 2],
        output: TurboOutput,
    ) -> ManualFrame {
        self.apply_controls(human, physical_stick, output, 0, 0, 0)
    }

    /// Compose an additional hold control through the SAME button history.
    /// Separate chained histories could fabricate edges when ownership changes.
    pub fn apply_controls(
        &mut self,
        human: ButtonTrace,
        physical_stick: [f32; 2],
        output: TurboOutput,
        extra_consumed: u32,
        requested_buttons: u32,
        owned_buttons: u32,
    ) -> ManualFrame {
        self.apply_reaction_controls(
            human,
            physical_stick,
            output,
            extra_consumed,
            requested_buttons,
            owned_buttons,
            false,
        )
    }
    /// Only Auto react opts into a fresh first horizontal flick. Automatic
    /// grabs, ledge tapes, native safety and other helpers keep their history.
    pub fn apply_reaction_controls(
        &mut self,
        human: ButtonTrace,
        physical_stick: [f32; 2],
        output: TurboOutput,
        extra_consumed: u32,
        requested_buttons: u32,
        owned_buttons: u32,
        reaction_owned: bool,
    ) -> ManualFrame {
        let entering_reaction = reaction_owned && !self.reaction_owned;
        self.reaction_owned = reaction_owned;
        if !output.consume_dpad
            && output.stick.is_none()
            && extra_consumed == 0
            && owned_buttons == 0
            && requested_buttons == 0
        {
            // One final history update releases any synthetic attack/shield
            // before returning ownership to fresh physical input.
            let buttons = self.buttons.apply_raw(human, 0, 0);
            let clear_autorepeat = if self.had_stick_override {
                LEFT_STICK_DIRECTIONS
            } else {
                0
            };
            self.had_stick_override = false;
            return ManualFrame {
                buttons,
                stick: physical_stick,
                clear_autorepeat,
            };
        }
        // The physical trigger was read before this call. Remove the D-pad
        // from all four words, including its history, so no taunt edge leaks.
        let consumed = crate::native_input::physical_button_aliases(
            extra_consumed | if output.consume_dpad { DPAD_SIDES } else { 0 },
        );
        let owned_buttons = crate::native_input::physical_button_aliases(owned_buttons);
        let human = ButtonTrace {
            held: human.held & !consumed,
            previous: human.previous & !consumed,
            pressed: human.pressed & !consumed,
            released: human.released & !consumed,
        };
        // Raw direction bits accompany the normalized stick floats. Without
        // owning those bits, a physically held direction could disagree with
        // the automated stick value during the original mapper's button queries.
        let mut owned = consumed | owned_buttons;
        let mut requested = requested_buttons;
        if let Some([x, y]) = output.stick {
            owned |= LEFT_STICK_DIRECTIONS;
            // Analog values remain authoritative. These coarse direction bits
            // keep full crouch input consistent without marking gentle walking
            // as a fully deflected digital stick direction.
            if x >= 0.5 && consumed & (1 << 21) == 0 {
                requested |= 1 << 21;
            }
            if x <= -0.5 && consumed & (1 << 23) == 0 {
                requested |= 1 << 23;
            }
            if y >= 0.5 {
                requested |= 1 << 20;
            }
            if y <= -0.5 {
                requested |= 1 << 22;
            }
        }
        let clear_autorepeat = owned
            | if self.had_stick_override {
                LEFT_STICK_DIRECTIONS
            } else {
                0
            };
        self.had_stick_override = output.stick.is_some();
        if entering_reaction
            && output
                .stick
                .is_some_and(|s| s[0].abs() >= 0.95 && s[1].abs() <= 0.3)
        {
            self.buttons
                .rearm_requested(requested & ((1 << 21) | (1 << 23)), human.previous);
        }
        ManualFrame {
            // Menu controls can share buttons with a configured recipe. Physical
            // menu presses were filtered above; retain explicitly owned macro
            // press/release history rather than erasing the generated input.
            buttons: remove_consumed(
                self.buttons.apply_raw(human, requested, owned),
                consumed & !owned_buttons,
            ),
            stick: output.stick.unwrap_or(physical_stick),
            clear_autorepeat,
        }
    }
}

/// Apply consumption after history reconstruction too: old native trigger
/// history must never reappear as a synthetic taunt press/release.
fn remove_consumed(trace: ButtonTrace, mask: u32) -> ButtonTrace {
    ButtonTrace {
        held: trace.held & !mask,
        previous: trace.previous & !mask,
        pressed: trace.pressed & !mask,
        released: trace.released & !mask,
    }
}

/// Filter the mapper RESULT as well as raw inputs. Native mapping can retain
/// repeat/history state beyond the four raw words. Only reserved taunts are
/// removed; this neither adds attack commands nor changes unrelated buttons.
// These masks belong to the mapped ACTION format, not raw D-pad bit positions.
// A new trigger must handle both raw consumption and this final mapped filter
// if native taunt repeat/history can survive the raw overlay.
/// Physical bindings reserve taunts, but an explicit owned sequence request
/// must retain its ordinary D-pad meaning. Side-taunt filtering is a pair.
pub fn sequence_taunt_reservation(consumed: u32, requested: u32, owned: u32) -> u32 {
    let intentional = requested & owned & 0xf;
    let sides = if intentional & DPAD_SIDES != 0 {
        DPAD_SIDES
    } else {
        0
    };
    consumed & !(intentional | sides)
}
pub fn suppress_reserved_taunts(mapped_buttons: u32, consumed: u32) -> u32 {
    let sides = if consumed & DPAD_SIDES != 0 {
        0x6000
    } else {
        0
    };
    let down = if consumed & crate::ledge_trap::DPAD_DOWN != 0 {
        0x1000
    } else {
        0
    };
    let up = if consumed & DPAD_UP != 0 { 0x400 } else { 0 };
    mapped_buttons & !(sides | down | up)
}

#[cfg(test)]
mod tests;

/// Constant-size physical-input history for explicit sequence handback.
#[derive(Clone, Copy, Default)]
pub struct ManualTakeover {
    previous: Option<(u32, [f32; 4])>,
    epoch: u64,
    action_epoch: u64,
}
impl ManualTakeover {
    pub const fn new() -> Self {
        Self {
            previous: None,
            epoch: 0,
            action_epoch: 0,
        }
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn stick(&self) -> [f32; 2] {
        self.previous
            .map_or([0.; 2], |(_, axes)| [axes[0], axes[1]])
    }
    /// Physical gameplay-button/C-stick edges, separate from movement intent.
    /// Committed automation reads the owner's frozen policy snapshot.
    pub fn action_epoch(&self) -> u64 {
        self.action_epoch
    }
    pub fn shield_held(&self) -> bool {
        self.previous
            .is_some_and(|(buttons, _)| buttons & crate::native_input::MAPPER_ZR != 0)
    }
    /// Native WAIT can lag a fresh run input. Stationary assists must inspect
    /// the physical stick too; an existing shield still permits OOS actions.
    pub fn stationary_assist(&self, status: i32) -> bool {
        use crate::native_input::*;
        self.previous.is_some_and(|(buttons, axes)| {
            if !axes.iter().all(|v| v.is_finite())
                || buttons & (MAPPER_A | MAPPER_B | MAPPER_R) != 0
                || axes[2].abs() > 0.2
                || axes[3].abs() > 0.2
            {
                return false;
            }
            let cap = if matches!(status, 0x1b | 0x1c) && buttons & MAPPER_ZR != 0 {
                0.7
            } else {
                0.2
            };
            axes[0].abs() <= cap && axes[1].abs() <= cap
        })
    }
    /// Grounded offense may use a held horizontal walk/run input. Native or
    /// freshly pressed shield keeps the stricter tilt/roll gate; manual attacks,
    /// C-stick, and strong vertical jump/drop intent keep player priority.
    pub fn grounded_grab_assist(&self, status: i32) -> bool {
        use crate::native_input::*;
        self.previous.is_some_and(|(buttons, axes)| {
            if buttons & MAPPER_ZR != 0 || matches!(status, 0x1b..=0x1e) {
                return self.stationary_assist(status);
            }
            axes.iter().all(|v| v.is_finite())
                && axes[1].abs() <= 0.7
                && axes[2].abs() <= 0.2
                && axes[3].abs() <= 0.2
                && buttons & (MAPPER_A | MAPPER_B | MAPPER_R) == 0
        })
    }
    pub fn sample(&mut self, buttons: u32, sticks: [f32; 4]) {
        use crate::native_input::*;
        let mask = MAPPER_A | MAPPER_B | MAPPER_X | MAPPER_Y | MAPPER_R | MAPPER_ZR | MAPPER_ZL;
        if let Some((old_buttons, old)) = self.previous {
            let pressed = buttons & !old_buttons & mask != 0;
            let moved = sticks.iter().zip(old).any(|(new, old)| {
                new.is_finite()
                    && old.is_finite()
                    && ((new.abs() > 0.25) != (old.abs() > 0.25) || (*new - old).abs() > 0.2)
            });
            if pressed || moved {
                self.epoch = self.epoch.wrapping_add(1);
            }
            let cstick_moved = sticks[2..].iter().zip(&old[2..]).any(|(new, old)| {
                new.is_finite()
                    && old.is_finite()
                    && ((new.abs() > 0.25) != (old.abs() > 0.25) || (*new - *old).abs() > 0.2)
            });
            if pressed || cstick_moved {
                self.action_epoch = self.action_epoch.wrapping_add(1);
            }
        }
        self.previous = Some((buttons, sticks));
    }
}
