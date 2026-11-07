//! Existing regression cases for `tech_input`. Kept outside the runtime code.
use super::*;
fn ctx(frame: f32) -> TechContext {
    TechContext {
        object: 1,
        status: 0x49,
        motion: 1,
        frame,
        rate: 1.0,
        stopped: false,
        remaining_stop: 0,
        contact_in: Some(3.0),
        input_delay: 0.0,
        previous_guard: false,
        prepare_guard: false,
    }
}
#[test]
fn unmapped_requests_do_not_spend_cooldown_or_disappear_on_duplicate_polls() {
    let mut input = TechInput::default();
    for _ in 0..30 {
        assert!(input.sample(true, true, 40, ctx(1.0)).press);
        assert_eq!(input.cooldown_remaining(), 0.0);
    }
    assert!(input.sample(true, true, 40, ctx(2.0)).press);
    input.mapped_press();
    assert_eq!(input.cooldown_remaining(), 40.0);
    assert!(!input.sample(true, true, 40, ctx(2.0)).press);
    assert_eq!(input.cooldown_remaining(), 40.0);
}
#[test]
fn held_guard_gets_release_then_press_and_hitstop_does_not_consume_lockout() {
    let mut input = TechInput::default();
    let release = input.sample(
        true,
        true,
        40,
        TechContext {
            previous_guard: true,
            ..ctx(1.0)
        },
    );
    assert!(!release.press && release.owns_guard);
    assert!(input.sample(true, true, 40, ctx(1.0)).press);
    input.mapped_press();
    for remaining in (1..=10).rev() {
        input.sample(
            true,
            true,
            40,
            TechContext {
                stopped: true,
                remaining_stop: remaining,
                ..ctx(1.0)
            },
        );
    }
    assert_eq!(input.cooldown_remaining(), 40.0);
}
#[test]
fn success_clears_synthetic_cooldown_and_changed_prediction_cancels_unmapped_press() {
    let mut input = TechInput::default();
    input.sample(true, true, 40, ctx(1.0));
    input.mapped_press();
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    status: 0x69,
                    motion: 2,
                    frame: 1.0,
                    ..ctx(1.0)
                }
            )
            .press
    );
    assert!(input.sample(true, true, 40, ctx(2.0)).press);
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    contact_in: None,
                    ..ctx(2.0)
                }
            )
            .press
    );
    assert_eq!(input.cooldown_remaining(), 0.0);
}
#[test]
fn delay_schedules_earlier_but_unknown_contact_and_disable_never_press() {
    let mut input = TechInput::default();
    let early = TechContext {
        contact_in: Some(14.0),
        ..ctx(1.0)
    };
    assert!(!input.sample(true, true, 40, early).press);
    assert!(
        input
            .sample(
                true,
                true,
                40,
                TechContext {
                    input_delay: 6.0,
                    ..early
                }
            )
            .press
    );
    assert!(!input.sample(false, true, 40, early).press);
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    contact_in: None,
                    ..early
                }
            )
            .press
    );
}
#[test]
fn cooldown_uses_rate_normalized_motion_progress_not_poll_count() {
    let mut input = TechInput::default();
    let first = TechContext {
        rate: 0.5,
        ..ctx(1.0)
    };
    input.sample(true, true, 40, first);
    input.mapped_press();
    for _ in 0..20 {
        input.sample(true, true, 40, first);
    }
    input.sample(
        true,
        true,
        40,
        TechContext {
            frame: 3.0,
            ..first
        },
    );
    assert_eq!(input.cooldown_remaining(), 36.0);
}
#[test]
fn native_hitstop_allows_a_buffered_request_and_additive_mode_preserves_held_guard() {
    let mut input = TechInput::default();
    let stop = TechContext {
        stopped: true,
        motion: 0,
        rate: 0.0,
        remaining_stop: 2,
        ..ctx(1.0)
    };
    assert!(input.sample(true, true, 40, stop).press);
    input.mapped_press();
    assert_eq!(input.cooldown_remaining(), 40.0);
    let mut additive = TechInput::default();
    assert_eq!(
        additive.sample(
            true,
            false,
            40,
            TechContext {
                previous_guard: true,
                ..ctx(1.0)
            }
        ),
        TechOutput::default()
    );
}
#[test]
fn shared_overlay_releases_both_triggers_before_a_fresh_tech_edge() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::{ButtonTrace, MAPPER_L, MAPPER_R, MAPPER_ZL, MAPPER_ZR};
    let shoulders = MAPPER_ZL | MAPPER_ZR | MAPPER_L | MAPPER_R;
    let human = ButtonTrace {
        held: MAPPER_ZL | MAPPER_ZR,
        previous: MAPPER_ZL | MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let mut overlay = ManualOverlay::default();
    let mut input = TechInput::default();
    let release = input.sample(
        true,
        true,
        40,
        TechContext {
            previous_guard: true,
            ..ctx(1.0)
        },
    );
    assert!(release.owns_guard && !release.press);
    let mapped = overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
    assert_eq!(mapped.buttons.held & shoulders, 0);
    let press = input.sample(true, true, 40, ctx(1.0));
    assert!(press.press);
    let mapped = overlay.apply_controls(
        human,
        [0.0; 2],
        TurboOutput::default(),
        0,
        MAPPER_ZR,
        shoulders,
    );
    assert_eq!(mapped.buttons.held & shoulders, MAPPER_ZR);
    assert_eq!(mapped.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn unavailable_animation_data_and_human_guard_do_not_erase_lockout_tracking() {
    let mut input = TechInput::default();
    input.sample(true, true, 40, ctx(1.0));
    input.mapped_press();
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    motion: 0,
                    rate: 0.0,
                    ..ctx(1.0)
                }
            )
            .press
    );
    assert_eq!(input.cooldown_remaining(), 40.0);
    input.sample(true, true, 40, ctx(2.0));
    input.mapped_human_guard();
    assert_eq!(input.cooldown_remaining(), 40.0);
    assert!(!input.sample(true, true, 40, ctx(3.0)).press);
}

#[test]
fn pending_tech_cannot_replay_into_ordinary_fall_or_air_dodge() {
    for status in [0xe, 0xf, 0x22, 0x23] {
        let mut input = TechInput::default();
        assert!(input.sample(true, true, 40, ctx(1.0)).press);
        let output = input.sample(
            true,
            true,
            40,
            TechContext {
                status,
                contact_in: Some(1.0),
                ..ctx(2.0)
            },
        );
        assert!(!output.press && !output.owns_guard);
        input.mapped_press();
        assert_eq!(input.cooldown_remaining(), 0.0);
    }
}

#[test]
fn safe_preparation_releases_held_guard_before_first_one_frame_contact_without_early_press() {
    let mut input = TechInput::default();
    let approach = TechContext {
        contact_in: Some(10.0),
        previous_guard: true,
        prepare_guard: true,
        ..ctx(1.0)
    };
    let release = input.sample(true, true, 40, approach);
    assert!(release.owns_guard && !release.press);
    assert_eq!(input.cooldown_remaining(), 0.0);
    let collision = input.sample(
        true,
        true,
        40,
        TechContext {
            frame: 2.0,
            contact_in: Some(1.0),
            previous_guard: false,
            ..approach
        },
    );
    assert!(collision.press && collision.owns_guard);
}

#[test]
fn prediction_gap_releases_ownership_without_rearming_lockout_and_recovery_hands_back() {
    let mut input = TechInput::default();
    let approach = TechContext {
        prepare_guard: true,
        ..ctx(1.0)
    };
    assert!(input.sample(true, true, 40, approach).press);
    input.mapped_press();
    let gap = input.sample(
        true,
        true,
        40,
        TechContext {
            frame: 2.0,
            contact_in: None,
            ..approach
        },
    );
    assert!(!gap.owns_guard && !gap.press);
    assert_eq!(input.cooldown_remaining(), 39.0);
    let recovered = input.sample(
        true,
        true,
        40,
        TechContext {
            frame: 3.0,
            contact_in: None,
            prepare_guard: false,
            ..approach
        },
    );
    assert_eq!(recovered, TechOutput::default());
    let mut additive = TechInput::default();
    assert_eq!(
        additive.sample(
            true,
            false,
            40,
            TechContext {
                contact_in: None,
                ..approach
            }
        ),
        TechOutput::default()
    );
    for status in [0xe, 0xf, 0x22, 0x67] {
        let mut input = TechInput::default();
        assert_eq!(
            input.sample(
                true,
                true,
                40,
                TechContext {
                    status,
                    contact_in: None,
                    ..approach
                }
            ),
            TechOutput::default()
        );
    }
}

#[test]
fn first_unknown_clock_still_records_the_configured_human_guard_cooldown() {
    let mut input=TechInput::default();
    let unknown=TechContext{motion:0,frame:f32::NAN,contact_in:None,..ctx(0.)};
    assert_eq!(input.sample(true,true,40,unknown),TechOutput::default());
    input.mapped_human_guard();
    assert_eq!(input.cooldown_remaining(),40.);
    assert!(!input.sample(true,true,40,ctx(1.)).press);
}

#[test]
fn observed_tech_clears_lockout_even_before_its_animation_clock_is_available() {
    for status in 0x67..=0x6b {
        let mut input=TechInput::default();
        input.sample(true,true,40,ctx(1.0));
        input.mapped_press();
        assert_eq!(input.cooldown_remaining(),40.0);
        let accepted=TechContext {status,motion:0,frame:f32::NAN,rate:0.0,
            contact_in:None,..ctx(2.0)};
        assert_eq!(input.sample(true,true,40,accepted),TechOutput::default());
        assert_eq!(input.cooldown_remaining(),0.0);
        // A later eligible hit must not inherit the completed attempt's lockout.
        assert!(input.sample(true,true,40,ctx(3.0)).press);
    }
}

#[test]
fn unacknowledged_press_rearms_if_guard_became_held_before_delivery() {
    let mut input=TechInput::default();
    assert!(input.sample(true,true,40,ctx(1.0)).press);
    let held=TechContext {previous_guard:true,..ctx(2.0)};
    let release=input.sample(true,true,40,held);
    assert!(!release.press && release.owns_guard);
    assert_eq!(input.cooldown_remaining(),0.0);
    assert!(input.sample(true,true,40,ctx(3.0)).press);
    input.mapped_press();
    assert_eq!(input.cooldown_remaining(),40.0);
}

#[test]
fn native_safe_contact_can_deliver_during_a_frozen_or_unavailable_animation_clock() {
    for (motion,frame,rate) in [(1,3.0,0.0),(0,f32::NAN,0.0)] {
        let mut input=TechInput::default();
        let c=TechContext {motion,frame,rate,prepare_guard:true,contact_in:Some(1.0),..ctx(3.0)};
        let out=input.sample(true,true,40,c);
        assert!(out.press && out.owns_guard);
        input.mapped_press();
        for _ in 0..100 {
            let out=input.sample(true,true,40,c);
            assert!(!out.press);
            assert_eq!(input.cooldown_remaining(),40.0,"unknown clock must not spend lockout by poll count");
        }
        let unsafe_c=TechContext {status:0xe,..c};
        assert_eq!(input.sample(true,true,40,unsafe_c),TechOutput::default());
    }
}

#[test]
fn tech_preparation_and_delivery_mask_guard_aliases_without_erasing_jump_attack_or_di() {
    use crate::{action_owner::{ActionOwnership,Owner,Intent},manual_controls::{ManualOverlay,TurboOutput},native_input::*};
    for layout in [ControlLayout::Default,ControlLayout::Brother] {
        let mut t=TechInput::default();let mut overlay=ManualOverlay::default();let mut owner=ActionOwnership::default();
        let physical=ButtonTrace{held:GAMEPLAY_BUTTONS,previous:GAMEPLAY_BUTTONS,..Default::default()};
        // Even a pre-existing committed sequence cannot block tech preparation.
        owner.select(true,&[Intent{owner:Owner::Ledge,present:true,committed:true}]);
        owner.set_ledge_priority(true);
        let released=t.sample(true,true,40,TechContext{prepare_guard:true,previous_guard:true,..ctx(0.)});
        assert_eq!(owner.select(true,&[Intent{owner:Owner::Safety,present:released.owns_guard,committed:false}]),Some(Owner::Safety));
        let out=overlay.apply_controls(physical,[-0.7,0.2],TurboOutput::default(),0,0,layout.physical_owned_buttons(released.owned_buttons()));
        let suppressed=physical_button_aliases(layout.physical_owned_buttons(released.owned_buttons()));
        assert_eq!(out.buttons.held & GAMEPLAY_BUTTONS,physical.held & !suppressed);
        assert_eq!(out.buttons.held & (MAPPER_A|MAPPER_B|MAPPER_Y),MAPPER_A|MAPPER_B|MAPPER_Y);
        assert_eq!(out.stick,[-0.7,0.2]);
        let pressed=t.sample(true,true,40,TechContext{prepare_guard:true,previous_guard:false,..ctx(0.)});
        assert!(pressed.press);
        let guard=layout.physical_buttons(MAPPER_ZR);
        let out=overlay.apply_controls(physical,[-0.7,0.2],TurboOutput::default(),0,guard,layout.physical_owned_buttons(pressed.owned_buttons()));
        assert_eq!(out.buttons.held & GAMEPLAY_BUTTONS,(physical.held & !suppressed)|guard);assert_eq!(out.buttons.pressed & guard,guard);
        t.mapped_press();assert_eq!(t.cooldown_remaining(),40.);
    }
}

#[test]
fn early_hitstop_without_a_collision_keeps_inputs_available_for_sdi() {
    let mut t=TechInput::default();
    for remaining in (6..=15).rev() {
        let out=t.sample(true,true,40,TechContext{stopped:true,remaining_stop:remaining,
            prepare_guard:true,previous_guard:true,contact_in:None,..ctx(0.)});
        assert_eq!(out,TechOutput::default());assert_eq!(t.cooldown_remaining(),0.);
    }
    let out=t.sample(true,true,40,TechContext{stopped:true,remaining_stop:5,
        prepare_guard:true,previous_guard:false,contact_in:Some(6.),..ctx(0.)});
    assert!(out.press && out.owns_guard); // Already released before imminent contact.
}

#[test]
fn no_or_distant_collision_forecast_does_not_reserve_controls() {
 for contact in [None,Some(12.),Some(60.),Some(f32::NAN),Some(-1.)] {
  let mut input=TechInput::default();
  assert_eq!(input.sample(true,true,40,TechContext{prepare_guard:true,previous_guard:true,contact_in:contact,..ctx(0.)}),TechOutput::default());
 }
 let mut input=TechInput::default();
 let preparation=input.sample(true,true,40,TechContext{prepare_guard:true,previous_guard:true,contact_in:Some(10.),..ctx(0.)});
 assert!(preparation.owns_guard&&!preparation.press);
 assert_eq!(preparation.owned_buttons(),crate::native_input::SEQUENCE_GUARD_INPUTS);
}

#[test]
fn guard_handback_after_tech_never_manufactures_a_held_shield_or_grab_edge() {
 use crate::{manual_controls::{AssistGuardHandoff,ManualOverlay,TurboOutput},native_input::*};
 for layout in [ControlLayout::Default,ControlLayout::Brother] {
  let mut handback=AssistGuardHandoff::new();let mut overlay=ManualOverlay::default();
  let guards=layout.physical_buttons(SEQUENCE_GUARD_INPUTS);
  let jumps=layout.physical_buttons(SEQUENCE_JUMP_BUTTONS);
  let held=guards|jumps|MAPPER_A|MAPPER_B;
  let trace=ButtonTrace{held,previous:held,..Default::default()};
  handback.sample(false,trace,guards,0,0);
  let blocked=handback.sample(true,trace,guards,guards,0);
  let out=overlay.apply_controls(trace,[0.;2],TurboOutput::default(),0,0,guards|blocked);
  assert_eq!(out.buttons.held&jumps,jumps);assert_eq!(out.buttons.held&guards,0);
  let blocked=handback.sample(false,trace,guards,0,0);
  let out=overlay.apply_controls(trace,[0.;2],TurboOutput::default(),0,0,blocked);
  assert_eq!(out.buttons.held&guards,0,"no old shield resurrected on handback");
  handback.sample(false,ButtonTrace{held:0,..Default::default()},guards,0,0);
  let blocked=handback.sample(false,trace,guards,0,0);
  assert_eq!(blocked&guards,0,"a fresh manual press rearms immediately");
 }
}
