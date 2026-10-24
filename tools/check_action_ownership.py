#!/usr/bin/env python3
"""Host-test the actual live ownership/cancellation blocks, without Skyline.

Run core integration tests first to produce a host rlib. No Switch build or
network access is performed. The temporary fixture copies production statements,
not a second implementation of selection/cancellation.
"""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def block(source, marker):
    start = source.index(marker)
    brace = source.index("{", start)
    depth, end = 1, brace + 1
    while depth:
        depth += (source[end] == "{") - (source[end] == "}")
        end += 1
    return source[start:end]


def main():
    source = (ROOT / "src/live.rs").read_text()
    cancel = block(source, "fn cancel_non_owner(")
    jab_lock_initial = block(source, "jab_lock_output: LedgeOutput {").split(": ", 1)[1]
    jab_grab_initial = block(source, "jab_grab_ready: kirby_assist_core::jab_grab::Readiness {").split(": ", 1)[1]
    start = source.index("let ledge_committed =")
    end = source.index("state.cancel_non_owner(winner);", start) + len("state.cancel_non_owner(winner);")
    selection = source[start:end]
    walk_start = source.index("                        let mut manual_walk_stick = None;")
    walk_end = source.index("                        // A tapped side", walk_start)
    walk_preparation = source[walk_start:walk_end]
    roll_start = source.index("        state.roll_read_output = if let Some((tuning, context, targets))")
    gate_start = source.index("            let allowed =", roll_start)
    gate_end = source.index("            let reader =", gate_start)
    roll_gate = source[gate_start:gate_end]
    slide_start = source.index("            let slide_context = SlideContext {")
    slide_end = source.index("            let trump_held", slide_start)
    slide_delivery = source[slide_start:slide_end]
    spot_start = source.index("        state.spot_output = if let Some((context, targets))")
    spot_end = source.index('        #[cfg(feature = "diagnostic-logging")]\n        let previous_option_output', spot_start)
    spot_delivery = source[spot_start:spot_end]
    fixture = r'''
#![allow(dead_code, unused_variables, unused_assignments)]
use kirby_assist_core::{
 action_owner::{ActionOwnership, Intent, Owner},
 combat_assist::{TripFollowup, TripContext, TripTarget, FairFollowup, FairContext},
 finisher::{Finisher, Context as FinisherContext}, button_modes::YMode,
 ledge_trap::{LedgeTrap, LedgeOutput}, two_frame::TwoFrame,
 roll_read::{RollRead, Context as RollContext, Target, ESCAPE_F},
 ledge_trump::LedgeTrump, slide::{SlideState, SlideContext},
 manual_controls::{Shortcuts, TurboOutput}, tech_input::TechOutput,
 native_input::{MAPPER_A, MAPPER_ZR},
};
#[derive(Default)]
struct State {
 jab_lock:Option<kirby_assist_core::jab_lock::JabLock>,jab_lock_output:LedgeOutput,
 action_owner:ActionOwnership, trip_followup:Option<TripFollowup>,trip_output:LedgeOutput,
 fair_followup:Option<FairFollowup>,fair_output:LedgeOutput,
 finisher:Option<Finisher>,finisher_output:LedgeOutput,
 roll_read:Option<RollRead>,roll_read_output:LedgeOutput,
 ledge:Option<LedgeTrap>,two_frame:Option<TwoFrame>,ledge_output:LedgeOutput,ledge_active:bool,ledge_walk_ready:bool,
 autopilot_guarding:bool,guard_hold:Option<kirby_assist_core::autopilot_guard::GuardHold>,
 duck_defense:Option<kirby_assist_core::autopilot_guard::duck::DuckDefense>,duck_output:LedgeOutput,
 spot_dodge:Option<kirby_assist_core::spot_dodge::SpotDodge>,spot_output:LedgeOutput,
 option_read:Option<kirby_assist_core::option_read::Punisher>,option_output:LedgeOutput,grab_punish_target:Option<u32>,
 trump:Option<LedgeTrump>,trump_output:LedgeOutput,slide:Option<SlideState>,slide_output:LedgeOutput,
 shortcuts:Option<Shortcuts>,tech_output:TechOutput,jab_grab:Option<kirby_assist_core::jab_grab::JabGrab>,
}
const JAB_LOCK_INITIAL: LedgeOutput = __LIVE_JAB_INIT__;
// This production field lives inside State::new, which must remain const.
const JAB_GRAB_INITIAL: kirby_assist_core::jab_grab::Readiness = __LIVE_JAB_GRAB_INIT__;
#[test] fn live_jab_grab_initializer_is_const_and_starts_without_native_permissions() {
 assert!(!JAB_GRAB_INITIAL.can_grab);
 assert!(!JAB_GRAB_INITIAL.can_walk);
 assert!(!JAB_GRAB_INITIAL.cancel_jab);
}

impl State { CANCEL }
fn select(state:&mut State)->(Option<Owner>,LedgeOutput,Option<[f32;2]>) {
 select_with_posture(state,LedgeOutput::default())
}
fn select_with_posture(state:&mut State,posture:LedgeOutput)->(Option<Owner>,LedgeOutput,Option<[f32;2]>) {
 select_with_walk(state,posture,None)
}
fn select_with_walk(state:&mut State,posture:LedgeOutput,manual_walk_stick:Option<[f32;2]>)->(Option<Owner>,LedgeOutput,Option<[f32;2]>) {
 select_with_damage(state,posture,manual_walk_stick,None)
}
fn select_with_damage(state:&mut State,posture:LedgeOutput,manual_walk_stick:Option<[f32;2]>,damage_stick:Option<[f32;2]>)->(Option<Owner>,LedgeOutput,Option<[f32;2]>) {
 let fresh=true; let manual_scope=true; let mash_stick=None;
 let shortcut=LedgeOutput::default(); let turbo_stick=None;
 let mut ledge=LedgeOutput::default(); let mut turbo_output=TurboOutput::default();
 SELECT
 (winner,ledge,turbo_output.stick)
}
#[test]
fn roll_read_rejects_down_tilt_chase_and_safety_cancels_it() {
 let mut state=State::default();
 state.action_owner.select(true,&[Intent{owner:Owner::RollRead,present:true,committed:true}]);
 state.roll_read_output=kirby_assist_core::move_input::output([0.0,-1.0],MAPPER_A);
 state.jab_lock_output=kirby_assist_core::move_input::output([0.65,0.0],0);
 assert_eq!(select(&mut state).0,Some(Owner::RollRead));
 assert_eq!(state.jab_lock_output.owned,0);
 state.roll_read_output=kirby_assist_core::move_input::output([0.0,-1.0],MAPPER_A);
 assert_eq!(select(&mut state).0,Some(Owner::RollRead));
 state.tech_output.press=true;assert_eq!(select(&mut state).0,Some(Owner::Safety));
 assert_eq!(state.jab_lock_output.owned,0);
}
fn fair(frame:f32,hit:bool)->FairContext {
 FairContext{object:1,kirby:true,status:0x36,motion:0xc3495ada5,frame,rate:1.0,
 grounded:false,stopped:false,facing:1.0,hit_now:hit}
}
#[test] fn uncommitted_hit_watchers_are_not_spent_by_empty_selection() {
 let mut state=State::default(); let mut trip=TripFollowup::default();
 let mut c=TripContext{object:1,kirby:true,status:0x2c,motion:1,frame:5.0,grounded:true,facing:1.0,stopped:false,targets:[None;8]};
 state.trip_output=trip.sample(true,c); state.trip_followup=Some(trip);
 let mut follow=FairFollowup::default();state.fair_output=follow.sample(true,fair(9.0,false));state.fair_followup=Some(follow);
 assert_eq!(select(&mut state).0,None);
 c.frame=6.0;c.targets[0]=Some(TripTarget{id:2,status:0x88,previous_status:0x86});
 assert_ne!(state.trip_followup.as_mut().unwrap().sample(true,c).owned,0);
 assert_ne!(state.fair_followup.as_mut().unwrap().sample(true,fair(10.0,true)).owned,0);
}
#[test] fn live_roll_commit_rejects_passive_shield_and_safety_cancels_queued_attack() {
 let mut state=State::default();let mut read=RollRead::default();
 let c=RollContext{status:0,situation:0,motion:1,frame:0.0,rate:1.0,stopped:false,
 can_act:true,can_smash:true,can_grab:true,facing:1.0,position:[0.0;2],movement:None};
 let mut targets=[None;8];targets[0]=Some(Target{id:2,status:ESCAPE_F,frame:2.0,rate:1.0,
 position:[-8.0,0.0],velocity_x:1.0,vulnerable_in:Some(14.0),actionable_in:Some(25.0)});
 state.roll_read_output=read.sample(true,kirby_assist_core::roll_read::fighter_config(6).unwrap(),c,targets);
 state.roll_read=Some(read);assert_eq!(select(&mut state).0,Some(Owner::RollRead));
 state.autopilot_guarding=true;state.ledge_active=true;
 state.ledge_output=LedgeOutput{owned:MAPPER_ZR,requested:MAPPER_ZR,stick:Some([0.0;2]),consume_trigger:false};
 let (owner,out,_)=select(&mut state);assert_eq!(owner,Some(Owner::RollRead));assert_eq!(out.requested,0);
 assert!(!state.autopilot_guarding);
 state.tech_output.press=true;assert_eq!(select(&mut state).0,Some(Owner::Safety));
 assert_eq!(state.roll_read_output.owned,0);assert!(!state.roll_read.as_ref().unwrap().committed());
}
#[test] fn roll_wins_over_trip_and_cancelled_y_does_not_restart_while_held() {
 let mut state=State::default();let mut tool=Finisher::default();
 let mut fighter=fair(0.0,false);fighter.status=0;fighter.motion=1;fighter.grounded=true;
 let c=FinisherContext{fighter_kind:6,target_direction:None,fighter,velocity_y:0.0,can_jump:true,can_start:true};
 state.finisher_output=tool.sample(true,true,YMode::FairSmash,c);state.finisher=Some(tool);
 state.trip_output=LedgeOutput{owned:MAPPER_A,requested:MAPPER_A,stick:Some([1.0,0.0]),consume_trigger:false};
 state.roll_read_output=LedgeOutput{owned:MAPPER_A,requested:MAPPER_A,stick:Some([0.0,-1.0]),consume_trigger:false};
 let (owner,out,_)=select(&mut state);assert_eq!(owner,Some(Owner::RollRead));assert_eq!(out.stick,Some([0.0,-1.0]));
 assert_eq!(state.finisher_output.owned,0);assert_eq!(state.trip_output.owned,0);
 assert_eq!(state.finisher.as_mut().unwrap().sample(true,true,YMode::FairSmash,c).owned,0);
}
#[test] fn stick_only_ledge_preparation_is_not_discarded() {
 let mut state=State::default();state.ledge_active=true;state.ledge_output.stick=Some([0.3,0.0]);
 assert_eq!(select(&mut state),(Some(Owner::Ledge),state.ledge_output,Some([0.3,0.0])));
}

#[test] fn armed_ledge_wait_blocks_crouch_and_lower_punishes_without_injecting_inputs() {
 let mut state=State::default();state.ledge_active=true;
 state.trip_output=kirby_assist_core::move_input::output([1.0,0.0],MAPPER_A);
 let posture=LedgeOutput{stick:Some([0.0,-1.0]),..LedgeOutput::default()};
 let (owner,out,stick)=select_with_posture(&mut state,posture);
 assert_eq!(owner,Some(Owner::Ledge));assert_eq!(out.requested,0);assert_eq!(stick,None);
 assert_eq!(state.trip_output.owned,0);
 state.tech_output.press=true;assert_eq!(select_with_posture(&mut state,posture).0,Some(Owner::Safety));
}
#[test] fn roll_read_neutral_primer_overrides_crouch_and_turnaround_tilt() {
 for stick in [[0.0,-1.0],[-0.8,-1.0],[0.8,-1.0]] {
  let mut state=State::default();
  state.roll_read_output=kirby_assist_core::move_input::output([0.0,0.0],0);
  let posture=LedgeOutput{stick:Some(stick),..LedgeOutput::default()};
  let (owner,out,applied)=select_with_posture(&mut state,posture);
  assert_eq!(owner,Some(Owner::RollRead));assert_eq!(out.requested,0);assert_eq!(applied,Some([0.0,0.0]));
 }
}

fn roll_allowed(state:&State,clatter:bool,tech:bool)->bool {
 ROLL_GATE
 allowed
}
#[test] fn live_roll_observation_is_never_vetoed_by_lower_mode_proposals() {
 for lower in [Owner::Trip,Owner::JabLock,Owner::Finisher,Owner::Fair,Owner::Trump,
               Owner::Slide,Owner::Shortcut,Owner::Defense,Owner::Turbo,Owner::Posture] {
  let mut state=State::default();
  state.action_owner.select(true,&[Intent{owner:lower,present:true,committed:true}]);
  let proposal=kirby_assist_core::move_input::output([0.8,-1.0],MAPPER_A);
  state.trip_output=proposal;state.finisher_output=proposal;state.fair_output=proposal;
  state.slide_output=proposal;state.trump_output=proposal;state.duck_output=proposal;
  state.autopilot_guarding=true;
  assert!(roll_allowed(&state,false,false),"{lower:?} blocked roll timing");
  assert!(!roll_allowed(&state,true,false));assert!(!roll_allowed(&state,false,true));
 }
 let mut state=State::default();
 state.action_owner.select(true,&[Intent{owner:Owner::Ledge,present:true,committed:true}]);
 assert!(roll_allowed(&state,false,false));
}

fn prepare_walk(state:&State,ledge_config:kirby_assist_core::ledge_trap::LedgeSettings,physical_stick:[f32;2])->Option<[f32;2]> {
 use kirby_assist_core::manual_controls::ledge_walk_stick;
 let manual_scope=true;let fresh=true;
 let ledge=state.ledge_output;let mut turbo_output=TurboOutput::default();
 WALK_PREPARATION
 manual_walk_stick
}
#[test] fn armed_autopilot_preserves_manual_walk_for_every_compiled_fighter_and_both_sides() {
 let profiles=kirby_assist_core::fighter_profiles::FighterProfiles::default();
 let mut count=0;
 for kind in 0..100 {
  let Some(profile)=profiles.profile(kind) else {continue};count+=1;
  for direction in [-1.0,1.0] {
   let settings=profile.apply(kirby_assist_core::ledge_trap::LedgeSettings{enabled:true,manual_spacing:true,..Default::default()});
   let mut state=State::default();state.ledge_active=true;state.ledge_walk_ready=true;
   let walking=prepare_walk(&state,settings,[direction,0.0]);
   assert!(walking.is_some(),"{} did not propose walking",profile.name);
   let (owner,out,stick)=select_with_walk(&mut state,LedgeOutput::default(),walking);
   assert_eq!(owner,Some(Owner::Ledge));assert_eq!(out.requested,0);
   assert_eq!(stick,walking,"{} lost walking during ledge reservation",profile.name);
   assert!(stick.unwrap()[0].abs()<=0.5);
  }
 }
 assert!(count>=70,"all compiled fighters must be checked");
}
#[test] fn walking_never_leaks_into_action_release_drift_or_a_roll_read() {
 for (owner,output,expected) in [
  (Owner::Ledge,kirby_assist_core::move_input::output([0.0,0.0],MAPPER_A),Some([0.0,0.0])),
  (Owner::Ledge,LedgeOutput{owned:MAPPER_A,..LedgeOutput::default()},None),
  (Owner::Ledge,kirby_assist_core::move_input::output([0.25,0.0],0),Some([0.25,0.0])),
  (Owner::RollRead,LedgeOutput{owned:MAPPER_A,..LedgeOutput::default()},None),
 ] {
  let mut state=State::default();state.ledge_active=true;
  if owner==Owner::Ledge {state.ledge_output=output;} else {state.roll_read_output=output;}
  let (winner,_,stick)=select_with_walk(&mut state,LedgeOutput::default(),Some([0.5,0.0]));
  assert_eq!(winner,Some(owner));assert_eq!(stick,expected);
 }
}

#[test] fn launch_di_is_safety_only_and_cannot_mix_with_a_ledge_attack() {
 let mut state=State::default();state.ledge_active=true;
 state.ledge_output=kirby_assist_core::move_input::output([0.0,0.0],MAPPER_A);
 let (owner,out,stick)=select_with_damage(&mut state,LedgeOutput::default(),None,Some([-1.0,0.0]));
 assert_eq!(owner,Some(Owner::Safety));assert_eq!(out.requested,0);assert_eq!(stick,Some([-1.0,0.0]));
 assert_eq!(state.ledge_output.owned,0);
 let (owner,_,stick)=select_with_damage(&mut state,LedgeOutput::default(),None,None);
 assert_eq!(owner,Some(Owner::Ledge));assert_eq!(stick,None);
}

fn live_slide(state:&mut State,context:SlideContext,policy_tick:Option<i64>,slide_buttons:u32) {
 struct Config {dpad_slide:bool}
 let config=Config{dpad_slide:true};let slide_scope=context.gameplay;let stopped=context.stopped;
 let can_slide_jump=context.can_jump;let can_slide_air_dodge=context.can_air_dodge;
 SLIDE_DELIVERY
}
#[test] fn live_slide_keeps_pending_guard_through_missing_clock_and_cancels_on_damage() {
 let mut state=State::default();
 let ground=SlideContext{gameplay:true,stopped:false,status:0,situation:0,motion_frame:0.0,
  can_jump:true,can_air_dodge:false,safe_ground:[true;2],input_delay:0.0};
 live_slide(&mut state,ground,Some(0),kirby_assist_core::manual_controls::DPAD_RIGHT);
 assert_eq!(select(&mut state).0,Some(Owner::Slide));
 live_slide(&mut state,SlideContext{stopped:true,..ground},None,0);
 assert_eq!(select(&mut state).0,Some(Owner::Slide));
 assert_eq!(state.slide_output.requested,0);
 let late=SlideContext{status:0xa,motion_frame:2.0,can_jump:false,..ground};
 live_slide(&mut state,late,Some(1),0);
 assert_eq!(state.slide_output.requested,MAPPER_ZR);
 let requested=state.slide_output;
 live_slide(&mut state,SlideContext{stopped:true,..late},None,0);
 let (owner,out,stick)=select(&mut state);
 assert_eq!(owner,Some(Owner::Slide));assert_eq!(out,requested);assert_eq!(stick,requested.stick);
 live_slide(&mut state,SlideContext{status:0x47,stopped:true,..late},None,0);
 assert_eq!(state.slide_output,LedgeOutput::default());
 assert_eq!(select(&mut state).0,None);assert!(state.slide.is_none());
}

#[test] fn constant_spot_dodge_beats_crouch_but_never_ledge_or_roll_and_never_mixes_sticks() {
 for priority in [None,Some(Owner::Ledge),Some(Owner::RollRead)] {
  let mut state=State::default();
  state.spot_output=kirby_assist_core::move_input::output([0.0,-1.0],MAPPER_ZR);
  if priority==Some(Owner::Ledge) {state.ledge_active=true;}
  if priority==Some(Owner::RollRead) {state.roll_read_output=kirby_assist_core::move_input::output([0.0,0.0],0);}
  let posture=LedgeOutput{stick:Some([0.8,-1.0]),..LedgeOutput::default()};
  let (owner,out,stick)=select_with_posture(&mut state,posture);
  assert_eq!(owner,Some(Owner::OptionRead));
  assert_eq!(out.requested,MAPPER_ZR);assert_eq!(stick,Some([0.0,-1.0]));
  assert_eq!(state.ledge_output.owned,0);assert_eq!(state.roll_read_output.owned,0);
 }
}

#[test] fn spot_punish_preempts_held_committed_ledge_without_mixing_inputs() {
 use kirby_assist_core::option_read::{Punisher,Context,Config,Target,Kind};
 let mut state=State::default();state.ledge_active=true;
 state.ledge_output=kirby_assist_core::move_input::output([0.0,0.0],MAPPER_A);
 state.action_owner.select(true,&[Intent{owner:Owner::Ledge,present:true,committed:true}]);
 let c=Context{object:1,status:0,motion:1,frame:0.0,rate:1.0,stopped:false,grounded:true,can_act:true,down_ready:true,forward_ready:true,facing:1.0,position:[0.0;2],velocity:[0.0;2]};
 let cfg=Config{down:[7,19],forward:[13,15],down_reach:14.0,forward_reach:22.0};
 let t=Target{id:2,kind:Kind::Spot,status:0x1f,motion:2,frame:10.0,rate:1.0,position:[10.0,0.0],velocity:[0.0;2],opens:10.0,closes:30.0};
 let mut targets=[None;8];targets[0]=Some(t);let mut read=Punisher::default();
 read.sample(true,c,cfg,targets);state.option_output=read.sample(true,Context{frame:1.0,..c},cfg,targets);state.option_read=Some(read);
 let (owner,out,stick)=select(&mut state);assert_eq!(owner,Some(Owner::OptionRead));assert_eq!(out.requested,MAPPER_A);assert_eq!(stick,Some([0.0,-1.0]));assert_eq!(state.ledge_output.owned,0);
}

fn live_spot_delivery(state:&mut std::sync::MutexGuard<'_,State>,context:kirby_assist_core::spot_dodge::Context,targets:[Option<kirby_assist_core::spot_dodge::Target>;8]) {
 let spot_observation=Some((context,targets));let clatter=false;let tech=false;
 SPOT_DELIVERY
}
#[test] fn production_grab_handoff_compiles_through_mutex_guard_and_preserves_target() {
 use kirby_assist_core::spot_dodge::{Context,Target,ESCAPE};
 let state=std::sync::Mutex::new(State::default());let mut state=state.lock().unwrap();
 let c=Context{object:1,status:0,situation:0,motion:1,frame:0.0,rate:1.0,stopped:false,can_spot_dodge:true,position:[0.0;2],velocity:[0.0;2]};
 let t=Target{id:2,status:0x37,motion:2,frame:2.0,rate:1.0,position:[10.0,0.0],velocity:[0.0;2],facing:-1.0,first:6,last:7,reach:12.0};
 let mut targets=[None;8];targets[0]=Some(t);
 live_spot_delivery(&mut state,c,targets);assert_eq!(state.spot_output.requested,MAPPER_ZR);
 live_spot_delivery(&mut state,Context{status:ESCAPE,motion:3,can_spot_dodge:false,..c},targets);
 assert_eq!(state.grab_punish_target,Some(2));assert_eq!(state.spot_output.requested,0);
}
'''.replace("CANCEL", cancel).replace("SELECT", selection).replace("ROLL_GATE", roll_gate).replace("SLIDE_DELIVERY", slide_delivery).replace("SPOT_DELIVERY", spot_delivery).replace("WALK_PREPARATION", walk_preparation).replace("__LIVE_JAB_INIT__", jab_lock_initial).replace("__LIVE_JAB_GRAB_INIT__", jab_grab_initial)
    deps = ROOT / "crates/kirby_assist_core/target/debug/deps"
    libs = list(deps.glob("libkirby_assist_core-*.rlib"))
    if not libs:
        raise SystemExit("Run the core action_ownership integration test first.")
    lib = max(libs, key=lambda p: p.stat().st_mtime)
    with tempfile.TemporaryDirectory(prefix="kirby-owner-") as temporary:
        directory = Path(temporary)
        rust = directory / "live_owner.rs"
        executable = directory / "live_owner_tests"
        rust.write_text(fixture)
        subprocess.run(["rustc", "--edition=2021", "--test", str(rust), "--extern",
                        f"kirby_assist_core={lib}", "-L", f"dependency={deps}", "-o", str(executable)], check=True)
        subprocess.run([str(executable)], check=True)


if __name__ == "__main__":
    main()
