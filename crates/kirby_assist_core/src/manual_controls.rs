//! Hold-to-run D-pad controls, independent of native pointers and Skyline.
//! Left/right are absolute stick directions, not the fighter's facing direction.
//! The caller captures physical input first and restores native memory after
//! mapping; this module only proposes a trace and left-stick position.
use crate::native_input::{ButtonTrace, MapperButtonOverlay};

pub const DPAD_UP: u32 = 1;
pub const DPAD_RIGHT: u32 = 1 << 1;
pub const DPAD_LEFT: u32 = 1 << 3;
pub const DPAD_SIDES: u32 = DPAD_LEFT | DPAD_RIGHT;
pub const LEFT_STICK_DIRECTIONS: u32 = 0xf << 20;
pub const RIGHT_STICK_DIRECTIONS: u32 = 0xf << 24;
pub const STICK_CLICKS: u32 = 3 << 16;

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

#[derive(Default)]
pub struct TurboState {
    direction: i8,
    next_phase: u32,
    last_tick: Option<i64>,
    last_output: TurboOutput,
}
impl TurboState {
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
}
impl ManualOverlay {
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
        if !output.consume_dpad
            && output.stick.is_none()
            && extra_consumed == 0
            && owned_buttons == 0
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
        let consumed = extra_consumed | if output.consume_dpad { DPAD_SIDES } else { 0 };
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
            if x >= 0.5 {
                requested |= 1 << 21;
            }
            if x <= -0.5 {
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
        ManualFrame {
            buttons: remove_consumed(self.buttons.apply_raw(human, requested, owned), consumed),
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
mod tests {
    use super::*;

    #[test]
    fn stick_only_override_works_without_dpad_or_button_ownership_and_restores_input() {
        use crate::native_input::MAPPER_A;
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace {
            held: MAPPER_A | (1 << 22),
            previous: MAPPER_A | (1 << 22),
            ..ButtonTrace::default()
        };
        let physical = [0.1, -0.8];
        let mash = TurboOutput {
            consume_dpad: false,
            stick: Some([1.0, 0.0]),
        };
        let frame = overlay.apply_controls(human, physical, mash, 0, 0, 0);
        assert_eq!(frame.stick, [1.0, 0.0]);
        assert_eq!(frame.buttons.held & LEFT_STICK_DIRECTIONS, 1 << 21);
        assert_eq!(frame.buttons.held & MAPPER_A, MAPPER_A);
        assert_ne!(frame.clear_autorepeat & LEFT_STICK_DIRECTIONS, 0);
        let restored = overlay.apply_controls(human, physical, TurboOutput::default(), 0, 0, 0);
        assert_eq!(restored.stick, physical);
        assert_eq!(restored.buttons.held, human.held);
    }
    #[test]
    fn left_right_repeat_and_direction_switch_starts_with_crouch() {
        let settings = TurboSettings {
            enabled: true,
            // This test exercises one-update phases at this explicit strength,
            // independently of the player's editable default tuning.
            phase_frames: 1,
            walk_strength: 0.25,
        };
        let mut state = TurboState::default();
        for tick in 0..6 {
            let output = state.sample(settings, tick, DPAD_LEFT, true, true);
            assert_eq!(
                output.stick,
                Some(if tick % 2 == 0 {
                    [0.0, -1.0]
                } else {
                    [-0.25, 0.0]
                })
            );
            assert_eq!(state.sample(settings, tick, DPAD_LEFT, true, true), output);
        }
        assert_eq!(
            state.sample(settings, 6, DPAD_RIGHT, true, true).stick,
            Some([0.0, -1.0])
        );
        assert_eq!(
            state.sample(settings, 7, DPAD_RIGHT, true, true).stick,
            Some([0.25, 0.0])
        );
    }
    #[test]
    fn release_conflict_pause_and_unsupported_movement_cancel_immediately() {
        let settings = TurboSettings {
            enabled: true,
            ..TurboSettings::default()
        };
        let mut state = TurboState::default();
        for (buttons, gameplay, allowed) in [
            (0, true, true),
            (DPAD_SIDES, true, true),
            (DPAD_LEFT, true, false),
            (DPAD_LEFT, false, true),
        ] {
            state.sample(settings, 1, DPAD_LEFT, true, true);
            let output = state.sample(settings, 1, buttons, gameplay, allowed);
            assert!(output.stick.is_none());
            assert_eq!(output.consume_dpad, gameplay);
        }
        assert!(!turbo_movement_allowed(0, 2, false));
        assert!(!turbo_movement_allowed(0x49, 0, false));
        assert!(!turbo_movement_allowed(0, 0, true));
        for status in [0, 1, 2, 6, 0x11, 0x12, 0x13, 0x14, 0x15] {
            assert!(turbo_movement_allowed(status, 0, false));
        }
    }
    #[test]
    fn suppresses_all_taunt_edges_preserves_attack_and_restores_physical_stick() {
        let attack = 1 << 5;
        let human = ButtonTrace {
            held: DPAD_LEFT | attack,
            previous: DPAD_LEFT,
            pressed: DPAD_LEFT | attack,
            released: DPAD_RIGHT,
        };
        let mut overlay = ManualOverlay::default();
        let crouch = overlay.apply(
            human,
            [0.4, 0.7],
            TurboOutput {
                consume_dpad: true,
                stick: Some([0.0, -1.0]),
            },
        );
        assert_eq!(crouch.buttons.held & DPAD_SIDES, 0);
        assert_eq!(crouch.buttons.previous & DPAD_SIDES, 0);
        assert_eq!(crouch.buttons.pressed & DPAD_SIDES, 0);
        assert_eq!(crouch.buttons.released & DPAD_SIDES, 0);
        assert_ne!(crouch.buttons.held & attack, 0);
        assert_ne!(crouch.buttons.pressed & (1 << 22), 0);
        let release = overlay.apply(
            ButtonTrace {
                held: attack,
                ..ButtonTrace::default()
            },
            [0.4, 0.7],
            TurboOutput {
                consume_dpad: true,
                stick: None,
            },
        );
        assert_eq!(release.stick, [0.4, 0.7]);
        assert_ne!(release.buttons.released & (1 << 22), 0);
        // Ownership hand-back may reconstruct one final edge; the following
        // sample must then be exactly the fresh physical trace.
        assert_eq!(
            overlay
                .apply(human, [0.4, 0.7], TurboOutput::default())
                .buttons
                .held,
            human.held
        );
        assert_eq!(
            overlay
                .apply(human, [0.4, 0.7], TurboOutput::default())
                .buttons,
            human
        );
    }
    #[test]
    fn left_stick_trigger_and_attack_release_share_one_consistent_history() {
        use crate::ledge_trap::{DPAD_DOWN, LEDGE_TRIGGER};
        use crate::native_input::MAPPER_A;
        let human = ButtonTrace {
            held: LEDGE_TRIGGER | DPAD_DOWN,
            previous: LEDGE_TRIGGER | DPAD_DOWN,
            pressed: LEDGE_TRIGGER,
            released: LEDGE_TRIGGER,
        };
        let mut overlay = ManualOverlay::default();
        let press = overlay.apply_controls(
            human,
            [0.1, 0.2],
            TurboOutput::default(),
            LEDGE_TRIGGER,
            MAPPER_A,
            MAPPER_A,
        );
        assert_eq!(press.buttons.held, MAPPER_A | DPAD_DOWN);
        assert_eq!(press.buttons.pressed, MAPPER_A);
        assert_eq!(press.buttons.previous & LEDGE_TRIGGER, 0);
        assert_eq!(press.buttons.released & LEDGE_TRIGGER, 0);
        assert_ne!(press.buttons.held & DPAD_DOWN, 0);
        let release = overlay.apply(ButtonTrace::default(), [0.1, 0.2], TurboOutput::default());
        assert_eq!(release.buttons.released, MAPPER_A);
        assert_eq!(release.stick, [0.1, 0.2]);
    }
    #[test]
    fn reserved_taunts_are_removed_from_mapped_result_without_affecting_other_actions() {
        let attack_and_up = 0x1 | 0x400;
        let all = attack_and_up | 0x7000;
        assert_eq!(
            suppress_reserved_taunts(all, DPAD_SIDES),
            attack_and_up | 0x1000
        );
        assert_eq!(
            suppress_reserved_taunts(all, crate::ledge_trap::DPAD_DOWN),
            attack_and_up | 0x6000
        );
        assert_eq!(
            suppress_reserved_taunts(all, DPAD_SIDES | crate::ledge_trap::DPAD_DOWN),
            attack_and_up
        );
        assert_eq!(suppress_reserved_taunts(all, 0), all);
    }
    #[test]
    fn first_reservation_clears_stale_dpad_history_and_takeover_releases_human_attack() {
        use crate::ledge_trap::{DPAD_DOWN, LEDGE_BUTTONS};
        use crate::native_input::MAPPER_A;
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace {
            held: DPAD_DOWN | DPAD_LEFT | MAPPER_A,
            previous: DPAD_DOWN | DPAD_LEFT | MAPPER_A,
            pressed: DPAD_DOWN,
            released: DPAD_LEFT,
        };
        overlay.apply(human, [0.0, 0.0], TurboOutput::default());
        let setup = overlay.apply_controls(
            human,
            [0.0, 0.0],
            TurboOutput::default(),
            DPAD_SIDES | DPAD_DOWN,
            0,
            LEDGE_BUTTONS,
        );
        for word in [
            setup.buttons.held,
            setup.buttons.previous,
            setup.buttons.pressed,
            setup.buttons.released,
        ] {
            assert_eq!(word & (DPAD_SIDES | DPAD_DOWN), 0);
        }
        assert_eq!(setup.buttons.held & MAPPER_A, 0);
        let press = overlay.apply_controls(
            human,
            [0.0, 0.0],
            TurboOutput::default(),
            DPAD_SIDES | DPAD_DOWN,
            MAPPER_A,
            LEDGE_BUTTONS,
        );
        assert_eq!(press.buttons.pressed & MAPPER_A, MAPPER_A);
    }
    #[test]
    fn diagonal_up_is_quarantined_until_its_release_without_disabling_normal_up_taunt() {
        let mut reservation = DpadReservation::default();
        let diagonal = reservation.sample(DPAD_SIDES, DPAD_LEFT | DPAD_UP, true);
        assert_ne!(diagonal & DPAD_UP, 0);
        assert_eq!(suppress_reserved_taunts(0x7400 | 1, diagonal), 0x1000 | 1);
        assert_ne!(reservation.sample(DPAD_SIDES, DPAD_UP, true) & DPAD_UP, 0);
        assert_ne!(reservation.sample(DPAD_SIDES, 0, true) & DPAD_UP, 0);
        assert_eq!(reservation.sample(DPAD_SIDES, 0, true) & DPAD_UP, 0);
        assert_eq!(reservation.sample(DPAD_SIDES, DPAD_UP, true) & DPAD_UP, 0);
        assert_eq!(reservation.sample(DPAD_SIDES, DPAD_LEFT, false), 0);
    }
    #[test]
    fn turbo_starts_down_on_first_poll_and_in_grounded_running_states() {
        let settings = TurboSettings {
            enabled: true,
            ..TurboSettings::default()
        };
        for status in 0..=9 {
            assert!(turbo_movement_allowed(status, 0, false));
            let mut state = TurboState::default();
            assert_eq!(
                state.sample(settings, 12345, DPAD_RIGHT, true, true).stick,
                Some([0.0, -1.0])
            );
        }
    }
    #[test]
    fn phase_length_and_reentry_do_not_keep_old_direction_or_clock() {
        let settings = TurboSettings {
            enabled: true,
            phase_frames: 2,
            walk_strength: 0.3,
        };
        let mut state = TurboState::default();
        for tick in 0..4 {
            assert_eq!(
                state.sample(settings, tick, DPAD_RIGHT, true, true).stick,
                Some(if tick < 2 { [0.0, -1.0] } else { [0.3, 0.0] })
            );
        }
        state.sample(settings, 4, 0, true, true);
        assert_eq!(
            state.sample(settings, 5, DPAD_RIGHT, true, true).stick,
            Some([0.0, -1.0])
        );
        assert_eq!(
            state.sample(settings, 0, DPAD_RIGHT, true, true).stick,
            Some([0.0, -1.0])
        );
    }
}
