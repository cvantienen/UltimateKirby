#!/usr/bin/env python3
"""Host-compile the production owner selection and cancellation; no Switch build."""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def block(source, marker):
    start = source.index(marker)
    brace = source.index('{', start)
    depth, end = 1, brace + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


def main():
    source = (ROOT / 'src/live.rs').read_text()
    state_source = (ROOT / 'src/live/state.rs').read_text()
    cancel = block(state_source, 'fn cancel_non_owner(')
    handoff = block(source, 'if spot.target().is_none() {')
    preempt = block(source, 'let preempt_grab = |state: &mut State| {')
    phase_change = block(source, 'if shield_fallback_pending\n            && !state.oos.shield_up_b_pending()')
    intent_inputs = block(source, 'struct MapperIntentInputs')
    intent_selection = block(source, 'fn mapper_action_intents(')
    owner_output = block(source, 'fn selected_owner_output(')
    owner_right_stick = block(source, 'fn selected_owner_right_stick(')
    start = source.index('let mut intents = mapper_action_intents(')
    end = source.index('state.cancel_non_owner(winner);', start) + len('state.cancel_non_owner(winner);')
    selection = source[start:end]
    fixture = r'''
#![allow(dead_code, unused_variables, unused_assignments)]
use kirby_assist_core::{
 action_owner::{ActionOwnership,Intent,Owner},
 combat_assist::{TripFollowup,FairFollowup},
 gameplay::{ControllerOutput,GameplayContext},ledge_trap::{LedgeTrap,SEQUENCE_BUTTONS},
 fighter_profiles::presets::PresetSlot,
 manual_controls::{ManualContext,TurboState,DashPivotAssist,TurboOutput},
 native_input::{MAPPER_A,MAPPER_B,MAPPER_R,MAPPER_ZR},
 out_of_shield::OutOfShield,projectile_reflect::Reflector,
 grab_handoff::GrabHandoff,tech_input::TechOutput,
};
#[derive(Default)]struct Menu {builtin_consumed:u32,preview:Option<PresetSlot>,rules:Option<Box<kirby_assist_core::reaction_rules::Resolved>>}
impl Menu {fn rule_config(&self)->Option<&kirby_assist_core::reaction_rules::Resolved>{self.rules.as_deref()}fn preview_slot(&self)->Option<PresetSlot>{self.preview}fn finish_preview(&mut self){self.preview=None;}}
#[derive(Default)]struct State {
 action_owner:ActionOwnership,reflector:Reflector,reflect_output:ControllerOutput,
 turbo:Option<TurboState>,dash_pivot:Option<DashPivotAssist>,

 trip_followup:Option<TripFollowup>,trip_output:ControllerOutput,
 fair_followup:Option<FairFollowup>,fair_output:ControllerOutput,
 rule_engine:Option<Box<kirby_assist_core::reaction_rules::Engine>>,roll_read_output:ControllerOutput,
 ledge:Option<LedgeTrap>,ledge_output:ControllerOutput,ledge_active:bool,
 oos:OutOfShield,oos_output:ControllerOutput,
 autopilot_guarding:bool,guard_hold:Option<kirby_assist_core::autopilot_guard::GuardHold>,
 spot_dodge:Option<kirby_assist_core::spot_dodge::SpotDodge>,spot_output:ControllerOutput,
 trump:Option<kirby_assist_core::ledge_trump::LedgeTrump>,trump_output:ControllerOutput,
 tech_output:TechOutput,tech_urgent:bool,tech_input:Option<kirby_assist_core::tech_input::TechInput>,fighter_kind:Option<i32>,grab_handoff:GrabHandoff,
 settings_menu:Option<Menu>,raw_tick:i64,
}
impl State {__CANCEL__}
__INTENT_INPUTS__
__INTENT_SELECTION__
__OWNER_OUTPUT__
__OWNER_RIGHT_STICK__
fn select(state:&mut State,fresh:bool,manual_scope:bool,mash_stick:Option<[f32;2]>,damage_stick:Option<[f32;2]>)->(Option<Owner>,ControllerOutput,Option<[f32;2]>) {
 select_phases(state,fresh,manual_scope,mash_stick,damage_stick,None)
}
fn select_phases(mut state:&mut State,fresh:bool,manual_scope:bool,mash_stick:Option<[f32;2]>,di_stick:Option<[f32;2]>,sdi_stick:Option<[f32;2]>)->(Option<Owner>,ControllerOutput,Option<[f32;2]>) {
 let damage_stick=di_stick.or(sdi_stick);
 let ledge_priority=state.action_owner.ledge_priority();let native_grab=false;
 let posture=ControllerOutput::default();let turbo_stick=None;let pivot_stick=None;
 let manual_walk_stick=None;let physical_stick=[-1.,1.];let policy_buttons=MAPPER_A;
 let menu_consumed=0;
 let manual_context=Some(ManualContext{status:0x1c,situation:0,motion:1,frame:0.,rate:1.,stopped:false,facing:1.});
 let mut sequence_short_hop=false;
 let mut ledge=ControllerOutput::default();let mut turbo_output=TurboOutput::default();
 let mut mapped_tech_output=None;let mut mapped_grab_handoff=None;
 __SELECT__
 (winner,ledge,turbo_output.stick)
}
fn packet(requested:u32)->ControllerOutput{ControllerOutput{requested,owned:MAPPER_A,..Default::default()}}
fn choose(s:&mut State)->(Option<Owner>,ControllerOutput,Option<[f32;2]>){select(s,true,true,None,None)}
#[test]fn committed_sequences_reject_every_competing_live_proposal_and_cancel_losers() {
 for current in [Owner::RollRead,Owner::OutOfShield,Owner::Ledge,Owner::Trip,Owner::Fair,Owner::Trump,Owner::Reflect] {
  let mut s=State::default();
  s.action_owner.select(true,&[Intent::input_sequence(current,1)]);
  s.roll_read_output=packet(MAPPER_A);s.oos_output=packet(MAPPER_A);
  s.ledge_active=true;s.ledge_output=packet(MAPPER_A);s.trip_output=packet(MAPPER_A);
  s.fair_output=packet(MAPPER_A);
  s.trump_output=packet(MAPPER_A);s.reflect_output=packet(MAPPER_A);
  let (owner,out,stick)=select(&mut s,true,true,None,None);
  assert_eq!(owner,Some(current));assert_eq!(out.requested,MAPPER_A);
  assert_eq!(out.owned,SEQUENCE_BUTTONS);assert_eq!(stick,Some([0.;2]));
  for (o,p) in [(Owner::RollRead,s.roll_read_output),(Owner::OutOfShield,s.oos_output),
   (Owner::Ledge,s.ledge_output),(Owner::Trip,s.trip_output),(Owner::Fair,s.fair_output),
   (Owner::Trump,s.trump_output),(Owner::Reflect,s.reflect_output)] {
   if o!=current {assert_eq!(p.owned,0,"losing {o:?} must not resume behind {current:?}");}
  }
 }
}
#[test]fn native_safety_cancels_attack_packets_and_owns_both_output_axes() {
 let mut s=State::default();s.ledge_active=true;s.ledge_output=packet(MAPPER_A);
 s.action_owner.select(true,&[Intent::input_sequence(Owner::Ledge,1)]);
 s.tech_output=TechOutput{press:true,owns_guard:true};
 let (owner,out,stick)=select(&mut s,true,true,None,Some([-1.,0.]));
 assert_eq!(owner,Some(Owner::Safety));assert_eq!(out.requested,0);
 assert_eq!(stick,Some([-1.,0.]));assert_eq!(s.ledge_output.owned,0);
 s.tech_output=TechOutput::default();let (_,out,_)=choose(&mut s);
 assert_eq!(out.owned,0);assert_eq!(s.action_owner.current_owner(),None);
}
#[test]fn receipt_and_final_release_keep_neutral_input_exclusive_then_release() {
 let mut s=State::default();s.roll_read_output=packet(0);
 for _ in 0..8 {let (o,p,stick)=choose(&mut s);assert_eq!(o,Some(Owner::RollRead));assert_eq!(p.owned,SEQUENCE_BUTTONS);assert_eq!(stick,Some([0.;2]));}
 s.roll_read_output=ControllerOutput::default();assert_eq!(choose(&mut s).0,None);
 assert_eq!(s.action_owner.current_owner(),None);
}
#[test]fn stale_or_invalid_scope_releases_all_cached_output_and_owners() {
 for (fresh,scope) in [(false,true),(true,false),(false,false)] {
  let mut s=State::default();s.roll_read_output=packet(MAPPER_B);s.oos_output=packet(MAPPER_R);
  s.action_owner.select(true,&[Intent::input_sequence(Owner::RollRead,1)]);
  let (o,p,stick)=select(&mut s,fresh,scope,None,None);
  assert_eq!(o,None);assert_eq!(p,ControllerOutput::default());assert_eq!(stick,None);
  assert_eq!(s.action_owner.current_owner(),None);assert_eq!(s.roll_read_output.owned,0);assert_eq!(s.oos_output.owned,0);
 }
}
#[test]fn stale_observations_cancel_hidden_up_b_receipts_instead_of_resuming_the_old_tape() {
 use kirby_assist_core::{reaction_rules::{Candidate,Engine,Resolved},roll_read::Target,
  gameplay::{Opponent,MotionTiming},settings::AssistSettings,sequence_config::Catalog};
 let cat=Catalog::parse("[bowser.roll.receipt]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4,drift:opponent:1:69\n").unwrap();
 let mut rules=Resolved::default();rules.add(Candidate{slot:2,index:0,sequence:cat.entries()[0].sequence}).unwrap();
 let c=GameplayContext{held:true,gameplay:true,can_act:true,status:0x1c,motion_kind:1,motion_rate:1.,facing:1.,..Default::default()};
 let enemy=Opponent{id:2,status:0x20,position:[10.,0.],motion:Some(MotionTiming{kind:2,frame:0.,rate:0.,remaining:None}),..Default::default()};
 let mut targets=[None;8];targets[0]=Some(Target{id:2,status:0x20,motion:2,frame:0.,rate:0.,position:enemy.position,velocity_x:0.,vulnerable_in:None,actionable_in:None});
 let settings=AssistSettings{fighter_kind:15,..Default::default()};let mut engine=Engine::default();
 assert_ne!(engine.sample_simple(true,&rules,settings,c,targets,Some(enemy)).requested & MAPPER_B,0);
 engine.mapped_up_special(0,2);
 assert_eq!(engine.sample_simple(true,&rules,settings,GameplayContext{status:0x1e,stopped:true,can_act:false,..c},targets,Some(enemy)).owned,0);
 assert!(engine.awaiting_up_b());
 let mut s=State::default();s.fighter_kind=Some(15);s.rule_engine=Some(Box::new(engine));
 s.action_owner.select(true,&[Intent::sequence_receipt(Owner::RollRead,0,true)]);
 assert_eq!(select(&mut s,false,true,None,None).0,None);
 assert!(!s.rule_engine.as_ref().unwrap().awaiting_up_b());assert!(!s.rule_engine.as_ref().unwrap().committed());
}
#[test]fn native_dodge_confirmation_transfers_the_same_response_owner_before_competing_ledge_admission() {
 use kirby_assist_core::{spot_dodge::{SpotDodge,Context,Target},sequence_config::Catalog};
 let mut s=State::default();s.action_owner.observe_manual(0,[0.;4]);
 let c=Context{object:1,status:0,situation:0,motion:1,frame:0.,rate:1.,stopped:false,can_spot_dodge:true,position:[0.;2],velocity:[0.;2]};
 let mut targets=[None;8];targets[0]=Some(Target{id:2,status:0x37,motion:2,frame:2.,rate:1.,position:[10.,0.],velocity:[0.;2],facing:-1.,first:6,last:7,reach:12.});
 let mut spot=SpotDodge::default();let out=spot.sample(true,c,targets);assert_eq!(out.requested,MAPPER_ZR);
 s.action_owner.select(true,&[Intent::input_sequence(Owner::GrabDodge,out.owned)]);
 s.action_owner.set_ledge_priority(true);s.spot_dodge=Some(spot);
 {
  let State{spot_dodge,action_owner,..}=&mut s;let spot=spot_dodge.as_mut().unwrap();
  spot.sample(true,Context{status:0x1f,..c},targets);
  __HANDOFF__
 }
 assert_eq!(s.action_owner.current_owner(),Some(Owner::OptionRead));
 let recipe=Catalog::parse("[bowser.roll.follow]\nstart_delay=0\nreaction=auto_grab_dodged:20:12\nactions=input:a:0:0:2\n").unwrap().entries()[0].sequence;
 s.spot_dodge.as_mut().unwrap().punish_automatic(true,Some(&recipe),Default::default(),GameplayContext{gameplay:true,status:0x1f,motion_kind:1,motion_rate:1.,..Default::default()},Some(kirby_assist_core::gameplay::Opponent{id:2,position:[10.,0.],..Default::default()}));
 let (owner,out,stick)=choose(&mut s);
 assert_eq!(owner,Some(Owner::OptionRead));assert_eq!(out.owned,SEQUENCE_BUTTONS);assert_eq!(stick,Some([0.;2]));
 assert!(s.spot_dodge.as_ref().unwrap().punish_pending());
}
fn reconsider_canceled_fallback(mut state:&mut State,c:GameplayContext,targets:[Option<kirby_assist_core::roll_read::Target>;8]) {
 let rules_on=true;let automatic_suspended=false;let clatter=false;let tech=false;
 let file_ledge_timeline=false;let ledge_priority=false;let shield_fallback_pending=true;
 let rule_observation=Some((c,targets,None));
 __PREEMPT__;
 preempt_grab(state);
 __PHASE_CHANGE__
}
#[test]fn completed_fallback_releases_its_lock_before_rechecking_priority_for_a_new_grab_proposal() {
 use kirby_assist_core::{sequence_config::Catalog,reaction_rules::{Resolved,Candidate},roll_read::Target,
  settings::AssistSettings,out_of_shield::{Context,AutomaticConfig}};
 let cat=Catalog::parse("[bowser.roll.priority]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4\n[bowser.roll.backup]\nstart_delay=0\nreaction=auto_shield_backup:25:24\nactions=up_b:4\n").unwrap();
 let mut rules=Resolved::default();rules.add(Candidate{slot:2,index:0,sequence:cat.entries()[0].sequence}).unwrap();
 let backup=cat.entries()[1].sequence;
 let c=GameplayContext{held:true,gameplay:true,can_act:true,status:0x1c,motion_kind:1,motion_rate:1.,facing:1.,..Default::default()};
 let native=Context{object:1,kind:15,status:0x1c,motion:1,frame:0.,rate:1.,stopped:false,grounded:true,position:[0.;2],facing:1.,shield_ready:true,can_grab:true,can_jump:true,can_up_special:true,threatened:false};
 let mut targets=[None;8];targets[0]=Some(Target{id:2,status:0,motion:2,frame:0.,rate:1.,position:[-10.,0.],velocity_x:0.,vulnerable_in:Some(0.),actionable_in:None});
 let mut s=State::default();let settings=AssistSettings{fighter_kind:15,..Default::default()};let config=AutomaticConfig{backup:Some(&backup),..Default::default()};
 let first=s.oos.sample_automatic_configured(true,native,c,targets,None,true,22.,35.,Some(8.),settings,config);
 assert_ne!(first.requested & MAPPER_B,0);s.action_owner.select(true,&[Intent::input_sequence(Owner::OutOfShield,first.owned)]);
 targets[0]=targets[0].map(|t|Target{id:3,status:0x1c,position:[10.,0.],..t});
 targets[1]=targets[0].map(|t|Target{id:4,status:0x20,position:[-15.,0.],..t});
 s.oos_output=s.oos.sample_automatic_configured(true,native,c,targets,None,true,22.,35.,Some(8.),settings,config);
 assert_ne!(s.oos_output.requested & MAPPER_R,0);assert!(!s.oos.shield_up_b_pending());
 s.settings_menu=Some(Menu{rules:Some(Box::new(rules)),..Default::default()});
 reconsider_canceled_fallback(&mut s,c,targets);
 assert!(s.oos.target().is_none());assert_eq!(s.oos_output.owned,0);assert!(s.action_owner.allows(Owner::RollRead));
}
#[test]fn actual_selector_never_mixes_tech_with_di_or_sdi() {
 let mut s=State::default();s.tech_output=TechOutput{press:true,owns_guard:true};
 let (o,p,stick)=select_phases(&mut s,true,true,None,Some([0.,1.]),Some([1.,0.]));
 assert_eq!(o,Some(Owner::Safety));assert_eq!(p.requested,0);assert_eq!(stick,Some([0.,1.]));
 let (_,p,stick)=select_phases(&mut s,true,true,None,None,Some([1.,0.]));
 assert_eq!(p.requested,MAPPER_ZR);assert_eq!(stick,Some([0.;2]));
}
#[test]fn actual_selector_preserves_a_tech_deadline_and_hands_its_release_to_di() {
 let mut s=State::default();s.tech_output=TechOutput{press:true,owns_guard:true};s.tech_urgent=true;
 let (o,p,stick)=select_phases(&mut s,true,true,None,Some([-1.,0.]),Some([1.,0.]));
 assert_eq!(o,Some(Owner::Safety));assert_eq!(p.requested,MAPPER_ZR);assert_eq!(stick,Some([0.;2]));
 s.tech_output=TechOutput{press:false,owns_guard:true};s.tech_urgent=false;
 let (o,p,stick)=select_phases(&mut s,true,true,None,Some([-1.,0.]),Some([1.,0.]));
 assert_eq!(o,Some(Owner::Safety));assert_eq!(p.requested,0);assert_eq!(stick,Some([-1.,0.]));
}
#[test]fn stale_scope_cancels_an_unconsumed_tech_delivery_without_erasing_lockout() {
 use kirby_assist_core::tech_input::{TechInput,TechContext};
 let mut s=State::default();let mut t=TechInput::default();
 let c=TechContext{object:1,status:0x49,motion:1,frame:0.,rate:1.,stopped:false,remaining_stop:0,
  contact_in:Some(1.),input_delay:0.,previous_guard:false,prepare_guard:true};
 s.tech_output=t.sample_with_native_delivery(true,true,28,c,None,12,7,false);t.mapped_press();
 s.tech_input=Some(t);s.tech_urgent=true;
 assert_eq!(select(&mut s,false,true,None,None).0,None);
 assert_eq!(s.tech_output,TechOutput::default());assert!(!s.tech_urgent);
 let t=s.tech_input.as_mut().unwrap();assert_eq!(t.cooldown_remaining(),28.);
 assert!(!t.sample_with_native_delivery(true,true,28,c,None,12,7,false).press);
}
#[test]fn no_active_automation_leaves_gameplay_unowned() {
 let mut s=State::default();assert_eq!(choose(&mut s),(None,ControllerOutput::default(),None));
 assert!(s.action_owner.allows(Owner::RollRead));
}
#[test]fn ledge_shield_keeps_defense_ownership_without_duck_output() {
 let mut s=State::default();s.ledge_active=true;s.autopilot_guarding=true;
 s.ledge_output=ControllerOutput{requested:MAPPER_ZR,owned:MAPPER_ZR,..Default::default()};
 let (owner,out,_)=choose(&mut s);
 assert_eq!(owner,Some(Owner::Defense));assert_eq!(out.requested,MAPPER_ZR);
 s.autopilot_guarding=false;s.ledge_active=false;s.ledge_output=ControllerOutput::default();
 assert_eq!(choose(&mut s).0,None);
}
'''.replace('__CANCEL__', cancel).replace('__INTENT_INPUTS__', intent_inputs).replace('__INTENT_SELECTION__', intent_selection).replace('__OWNER_OUTPUT__', owner_output).replace('__OWNER_RIGHT_STICK__', owner_right_stick).replace('__SELECT__', selection).replace('__HANDOFF__', handoff).replace('__PREEMPT__', preempt).replace('__PHASE_CHANGE__', phase_change)
    deps = ROOT / 'crates/kirby_assist_core/target/debug/deps'
    libs = list(deps.glob('libkirby_assist_core-*.rlib'))
    if not libs:
        raise SystemExit('Run a core integration test first.')
    lib = max(libs, key=lambda p: p.stat().st_mtime)
    with tempfile.TemporaryDirectory(prefix='kirby-owner-') as temporary:
        rust = Path(temporary) / 'live_owner.rs'
        executable = Path(temporary) / 'live_owner_tests'
        rust.write_text(fixture)
        subprocess.run(['rustc', '--edition=2021', '--test', str(rust), '--extern',
                        f'kirby_assist_core={lib}', '-L', f'dependency={deps}', '-o', str(executable)], check=True)
        subprocess.run([str(executable)], check=True)


if __name__ == '__main__':
    main()
