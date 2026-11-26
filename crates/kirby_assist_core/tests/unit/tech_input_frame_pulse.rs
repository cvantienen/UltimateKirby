use super::*;
use crate::native_states::{situation, status};

#[test]
fn native_ground_wall_and_ceiling_states_are_classified() {
    assert_eq!(
        native_tech_state(
            status::DAMAGE_FLY,
            situation::AIR,
            false,
            true,
            false,
            false
        ),
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
    assert_eq!(
        native_tech_state(
            status::PASSIVE_CEIL,
            situation::AIR,
            false,
            true,
            true,
            true
        ),
        NativeTechState::Succeeded
    );
    assert_eq!(
        native_tech_state(status::DAMAGE_FLY, situation::AIR, true, true, true, true),
        NativeTechState::Inactive
    );
}

#[test]
fn native_surface_contact_can_trigger_without_a_ray_forecast() {
    use crate::native_states::{situation, status};
    assert!(native_contact_due(
        status::DAMAGE_FLY,
        situation::GROUND,
        false,
        true,
        false,
        false
    ));
    assert!(native_contact_due(
        status::STOP_WALL,
        situation::AIR,
        false,
        false,
        true,
        false
    ));
    assert!(native_contact_due(
        status::STOP_CEIL,
        situation::AIR,
        false,
        false,
        false,
        true
    ));
    assert!(!native_contact_due(
        status::DAMAGE_FLY,
        situation::AIR,
        false,
        true,
        false,
        false
    ));
    assert!(!native_contact_due(
        status::DAMAGE_FLY,
        situation::GROUND,
        true,
        true,
        false,
        false
    ));
    assert!(!native_contact_due(
        status::STOP_WALL,
        situation::AIR,
        false,
        false,
        false,
        false
    ));
}
#[test]
fn pulses_once_per_frame_and_releases_when_the_opportunity_ends() {
    let mut input = FrameTechInput::default();
    let on = input.sample(true, 7, 10, NativeTechState::Opportunity, false);
    assert_eq!(
        on,
        TechOutput {
            press: true,
            owns_guard: true
        }
    );
    assert_eq!(
        input.sample(true, 7, 10, NativeTechState::Opportunity, false),
        on
    );
    assert_eq!(
        input.sample(true, 7, 11, NativeTechState::Opportunity, false),
        TechOutput {
            press: false,
            owns_guard: true
        }
    );
    assert_eq!(
        input.sample(true, 7, 12, NativeTechState::Opportunity, false),
        on
    );
    assert_eq!(
        input.sample(true, 7, 13, NativeTechState::Succeeded, false),
        TechOutput::default()
    );
    assert_eq!(
        input.sample(true, 7, 14, NativeTechState::Inactive, false),
        TechOutput::default()
    );
}

#[test]
fn frame_rewind_restarts_guard_preparation_without_replaying_old_phase() {
    let mut input = FrameTechInput::default();
    assert!(
        input
            .sample(true, 1, 10, NativeTechState::Opportunity, false)
            .press
    );
    let release = TechOutput {
        press: false,
        owns_guard: true,
    };
    assert_eq!(
        input.sample(true, 1, 11, NativeTechState::Opportunity, false),
        release
    );
    assert_eq!(
        input.sample(true, 1, 0, NativeTechState::Opportunity, true),
        release
    );
    assert_eq!(
        input.sample(true, 1, 0, NativeTechState::Opportunity, true),
        release
    );
    assert!(
        input
            .sample(true, 1, 1, NativeTechState::Opportunity, false)
            .press
    );
    assert_eq!(
        input.sample(true, 2, 1, NativeTechState::Opportunity, true),
        release
    );
}

#[test]
fn held_guard_is_released_before_the_first_automatic_press() {
    let mut input = FrameTechInput::default();
    assert_eq!(
        input.sample(true, 1, 1, NativeTechState::Opportunity, true),
        TechOutput {
            press: false,
            owns_guard: true
        }
    );
    assert_eq!(
        input.sample(true, 1, 2, NativeTechState::Opportunity, true),
        TechOutput {
            press: true,
            owns_guard: true
        }
    );
}
