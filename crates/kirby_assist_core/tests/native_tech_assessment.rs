use kirby_assist_core::{
    action_owner::safety_output_with_deadline,
    native_states::{situation, status},
    tech_input::{native_tech_state, FrameTechInput, NativeTechState, TechOutput},
};

#[test]
fn native_flags_cover_ground_wall_ceiling_and_success() {
    assert_eq!(
        native_tech_state(status::DAMAGE_FLY, situation::AIR, false, true, false, false),
        NativeTechState::Opportunity
    );
    assert_eq!(
        native_tech_state(status::STOP_WALL, situation::AIR, false, false, true, false),
        NativeTechState::Opportunity
    );
    assert_eq!(
        native_tech_state(status::STOP_CEIL, situation::AIR, false, false, false, true),
        NativeTechState::Opportunity
    );
    for successful in [
        status::PASSIVE,
        status::PASSIVE_FB,
        status::PASSIVE_WALL,
        status::PASSIVE_WALL_JUMP,
        status::PASSIVE_CEIL,
    ] {
        assert_eq!(
            native_tech_state(successful, situation::AIR, false, false, false, false),
            NativeTechState::Succeeded
        );
    }
}

#[test]
fn passive_disable_flag_and_unrelated_statuses_never_pulse() {
    assert_eq!(
        native_tech_state(status::DAMAGE_FLY, situation::AIR, true, true, true, true),
        NativeTechState::Inactive
    );
    assert_eq!(
        native_tech_state(status::FALL, situation::AIR, false, true, true, true),
        NativeTechState::Inactive
    );
}

#[test]
fn guard_alternates_once_per_frame_and_stops_immediately() {
    let mut input = FrameTechInput::default();
    let on = TechOutput {
        press: true,
        owns_guard: true,
    };
    let off = TechOutput {
        press: false,
        owns_guard: true,
    };
    assert_eq!(input.sample(true, 1, 20, NativeTechState::Opportunity, false), on);
    assert_eq!(input.sample(true, 1, 20, NativeTechState::Opportunity, false), on);
    assert_eq!(input.sample(true, 1, 21, NativeTechState::Opportunity, false), off);
    assert_eq!(input.sample(true, 1, 22, NativeTechState::Opportunity, false), on);
    assert_eq!(input.sample(true, 1, 23, NativeTechState::Inactive, false), TechOutput::default());
    assert_eq!(input.sample(true, 1, 24, NativeTechState::Succeeded, false), TechOutput::default());
}

#[test]
fn held_guard_is_released_before_the_first_automatic_press() {
    let mut input = FrameTechInput::default();
    assert_eq!(
        input.sample(true, 1, 1, NativeTechState::Opportunity, true),
        TechOutput {
            press: false,
            owns_guard: true,
        }
    );
    assert_eq!(
        input.sample(true, 1, 2, NativeTechState::Opportunity, true),
        TechOutput {
            press: true,
            owns_guard: true,
        }
    );
}

#[test]
fn release_phase_keeps_guard_ownership_without_blocking_di() {
    let release = TechOutput {
        press: false,
        owns_guard: true,
    };
    let (packet, applied) =
        safety_output_with_deadline(release, None, Some([0.5, 1.0]), None, false);
    assert_eq!(applied, Some(release));
    assert_eq!(packet.stick, Some([0.5, 1.0]));
    assert_eq!(packet.requested, 0);
}
