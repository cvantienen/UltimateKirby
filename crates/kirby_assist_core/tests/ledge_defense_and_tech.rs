//! Ledge watch, defensive-only shield and acknowledged tech clocks share one owner.
use kirby_assist_core::{action_owner::{ActionOwnership,Intent,Owner},autopilot_guard::{GuardHold,GuardClock,Envelope,recovery_warning},
 fighter_profiles::FighterProfiles,ledge_menu::MenuSelection,tech_input::{TechInput,TechContext,mapped_guard},tech_prediction::{tech_gate_with_floor,TechGate},native_input::*};
fn clock(frame:f32)->GuardClock {GuardClock{status:0x1c,motion:1,frame,rate:1.,stopped:false}}
fn tc(frame:f32)->TechContext {TechContext{object:1,status:0x49,motion:1,frame,rate:1.,stopped:false,remaining_stop:0,contact_in:Some(1.),input_delay:0.,previous_guard:false,prepare_guard:true}}
#[test]
fn ledge_toggle_reserves_auto_modes_through_manual_pauses_but_safety_and_shield_still_win() {
 let profiles=FighterProfiles::default();let p=&profiles.profile(15).unwrap().menu_options;
 let mut menu=MenuSelection::new(p);menu.auto_ledge=true;
 assert!(menu.reserves_automatic_modes());assert!(!menu.autopilot_held(4));
 let mut owners=ActionOwnership::default();owners.set_automatic_suspended(menu.reserves_automatic_modes());
 for owner in [Owner::OutOfShield,Owner::GrabDodge,Owner::OptionRead,Owner::RollRead,Owner::Trip,Owner::JabLock,Owner::Fair,Owner::Finisher] {
  assert!(!owners.allows(owner));
  assert_eq!(owners.select(true,&[Intent{owner,present:true,committed:true}]),None);
 }
 assert_eq!(owners.select(true,&[Intent{owner:Owner::Defense,present:true,committed:false}]),Some(Owner::Defense));
 assert_eq!(owners.select(true,&[Intent{owner:Owner::Ledge,present:true,committed:true},Intent{owner:Owner::Safety,present:true,committed:false}]),Some(Owner::Safety));
 menu.auto_ledge=false;owners.set_automatic_suspended(menu.reserves_automatic_modes());assert!(owners.allows(Owner::GrabDodge));
}
#[test]
fn rising_up_b_warning_covers_startup_and_first_character_continuation_without_reacting_to_idle() {
 for (kind,status,previous) in [(15,0x1de,0),(15,0x1e1,0),(6,0x1fc,0x1de)] {
  assert!(recovery_warning(kind,status,previous,[0.;2],[0.;2],[0.,-40.],[0.,8.],Envelope::default()));
  assert!(recovery_warning(kind,status,previous,[0.;2],[0.;2],[0.,-40.],[0.,0.],Envelope::default()),"prepare before startup accelerates upward");
  assert!(!recovery_warning(kind,status,previous,[0.;2],[0.;2],[160.,-40.],[0.,8.],Envelope::default()));
 }
 assert!(!recovery_warning(6,0x1e6,0x1de,[0.;2],[0.;2],[0.,-40.],[0.,8.],Envelope::default()),"Kirby inhale is not a Final Cutter continuation");
 assert!(!recovery_warning(15,0,0,[0.;2],[0.;2],[0.,-10.],[0.,2.],Envelope::default()));
 assert!(!recovery_warning(6,0x1de,0,[0.;2],[0.;2],[0.,-80.],[0.,-8.],Envelope::default()));
}
#[test]
fn rising_below_stage_and_crossing_weapon_geometry_requests_defense() {
 use kirby_assist_core::autopilot_guard::{active_hitbox_threat,duck_geometry::Hitbox};
 assert!(active_hitbox_threat([0.;3],[0.;2],Hitbox{start:[0.,-40.,0.],end:[0.,-34.,0.],radius:5.,velocity:[0.,10.]}));
 assert!(active_hitbox_threat([0.;3],[0.;2],Hitbox{start:[-55.,8.,0.],end:[-50.,8.,0.],radius:3.,velocity:[15.,0.]}));
}
#[test]
fn shield_stun_multihit_gaps_and_invalid_hitstop_clock_keep_guard_then_release_without_a_punish() {
 let mut shield=GuardHold::default();assert!(shield.sample(true,true,clock(10.)));
 let frozen=GuardClock{status:0x1e,motion:0,frame:f32::NAN,rate:0.,stopped:true};
 for _ in 0..50 {assert!(shield.sample(true,false,frozen));}
 assert!(shield.sample(true,false,GuardClock{status:0x1e,..clock(0.)}));
 assert!(shield.sample(true,false,clock(0.)));
 // An unexplained non-stopped rewind must not leave shield latched.
 assert!(shield.sample(true,false,clock(0.5)));
 assert!(!shield.sample(true,false,clock(0.)));
 assert!(!shield.sample(true,false,clock(10.)));
 assert!(!shield.sample(false,true,clock(11.)));
}
#[test]
fn floor_tech_requires_native_transition_and_never_accepts_normal_ground_states_or_untechable_hits() {
 assert_eq!(tech_gate_with_floor(0x49,0,false,false,true),TechGate::Predict);
 assert_eq!(tech_gate_with_floor(0x4b,0,false,true,true),TechGate::Hitstop);
 assert_eq!(tech_gate_with_floor(0x49,0,false,false,false),TechGate::NotAirborne);
 assert_eq!(tech_gate_with_floor(0x49,0,true,false,true),TechGate::PassiveDisabled);
 for status in [0,0x1c,0x1f,0x47,0x50,0x53,0x5a] {assert_eq!(tech_gate_with_floor(status,0,false,false,true),TechGate::OutsideTumble);}
}
#[test]
fn reaction_clock_expires_tech_cooldown_during_frozen_damage_without_counting_polls_or_hitstop() {
 let mut input=TechInput::default();let c=TechContext{rate:0.,..tc(1.)};
 assert!(input.sample_with_reaction_clock(true,true,40,c,Some(100.)).press);input.mapped_press();
 for _ in 0..100 {assert!(!input.sample_with_reaction_clock(true,true,40,c,Some(100.)).press);}
 assert_eq!(input.cooldown_remaining(),40.);
 for remaining in (61..100).rev() {assert!(!input.sample_with_reaction_clock(true,true,40,c,Some(remaining as f32)).press);}
 assert_eq!(input.cooldown_remaining(),1.);
 assert!(input.sample_with_reaction_clock(true,true,40,c,Some(60.)).press);
 input.mapped_press();let stopped=TechContext{stopped:true,remaining_stop:3,..c};
 input.sample_with_reaction_clock(true,true,40,stopped,Some(59.));assert_eq!(input.cooldown_remaining(),40.);
 let next=TechContext{previous_guard:true,..c};let out=input.sample_with_reaction_clock(true,true,40,next,Some(58.));
 assert!(!out.press);assert_eq!(input.cooldown_remaining(),40.);
 assert_eq!(mapped_guard(MAPPED_GUARD|MAPPED_GUARD_HOLD|MAPPED_CATCH,out)&(MAPPED_GUARD|MAPPED_GUARD_HOLD|MAPPED_CATCH),0);
}
#[test]
fn native_timer_and_animation_do_not_double_spend_tech_lockout_and_bad_timers_cannot_clear_it() {
 let mut input=TechInput::default();input.sample_with_reaction_clock(true,true,40,tc(0.),Some(80.));input.mapped_press();
 input.sample_with_reaction_clock(true,true,40,tc(1.),Some(79.));assert_eq!(input.cooldown_remaining(),39.);
 input.sample_with_reaction_clock(true,true,40,tc(1.),Some(100.));assert_eq!(input.cooldown_remaining(),39.);
 for bad in [f32::NAN,f32::INFINITY,-1.,2000.] {input.sample_with_reaction_clock(true,true,40,tc(1.),Some(bad));assert_eq!(input.cooldown_remaining(),39.);}
}
