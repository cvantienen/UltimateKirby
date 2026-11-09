#!/usr/bin/env python3
"""Host-test the actual live input freshness/publication guards; no Switch build."""
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def main():
    source = (ROOT / 'src/live.rs').read_text()
    start = source.index('                        let fresh = ')
    fresh = source[start:source.index('                        let ledge_fresh', start)]
    start = source.index('        // Queries belong to the controller sample')
    guard = source[start:source.index('        state.tech_floor_motion =', start)]
    start = source.index('    let manual_automatic_blocked =')
    manual_guard = source[start:source.index('    let damage_stop =', start)]
    manual_guard = manual_guard.replace('STATE.lock().ok().map_or(true, |mut state| {',
                                        'Some(state).map_or(true, |state| {')
    start = source.index('        let admission_suspended =', source.index('        // Keep watches fresh'))
    receipt_guard = source[start:source.index('        let ledge_priority =', start)]
    start = source.index('                        state.grab_handoff.restrict(&mut intents);')
    arbitration = source[start:source.index('                        sequence_short_hop =', start)]
    start = source.index('                            if ledge_inputs_reserved\n                                && state.action_owner.current_owner().is_none()')
    brace = source.index('{', start)
    depth, end = 1, brace + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    ledge_reservation = source[start:end]
    start = source.index('                        mapped_attack_hold = fresh')
    attack_delivery = source[start:source.index('                        // Forward all exclusive sequence axes', start)]
    start = source.index('                        if fresh && manual_scope && exclusive_sequence')
    brace = source.index('{', start)
    depth, end = 1, brace + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    literal_delivery = source[start:end]
    start = source.index('                            let committed_inputs =')
    binding_guard = source[start:source.index('                            let changed =', start)]
    start = source.index('                        if winner == Some(Owner::Safety) {\n                            (selected, mapped_tech_output)')
    selected_packet = source[start:source.index('                        mapped_grab_handoff =', start)]
    fixture = r'''
#![allow(dead_code,unused_mut,unused_variables)]
use kirby_assist_core::session::{MatchSession,SessionTicket};
use std::{time::{Instant,Duration},sync::atomic::{AtomicBool,Ordering}};
static RUNTIME_ENABLED:AtomicBool=AtomicBool::new(true);
const REQUEST_LIFETIME:Duration=Duration::from_millis(35);
struct State {
 session:MatchSession,mapping_index:Option<i32>,associated_object:Option<u32>,
 observation_sequence:u64,raw_tick:i64,evaluated_tick:Option<i64>,
 observed_object:Option<u32>,request_seen:Option<Instant>,
}
fn seed()->(State,SessionTicket) {
 let mut session=MatchSession::new();session.update(true);
 let ticket=session.ticket().unwrap();
 (State{session,mapping_index:Some(0),associated_object:Some(1),
 observation_sequence:2,raw_tick:10,evaluated_tick:None,
 observed_object:Some(1),request_seen:Some(Instant::now())},ticket)
}
fn publish(state:&mut State,session_ticket:SessionTicket,index:i32,object_id:u32,
 observation_sequence:u64,tick:i64) {
 GUARD
 state.evaluated_tick=Some(tick);
 state.request_seen=Some(Instant::now());
}
fn fresh(state:&State)->bool {
 let now=Instant::now();
 FRESH
 fresh
}
#[test]fn publication_never_relabels_a_late_observation_as_the_current_update() {
 for tick in [9,11,i64::MIN,i64::MAX] {
  let (mut s,ticket)=seed();
  publish(&mut s,ticket,0,1,2,tick);
  assert_eq!(s.evaluated_tick,None);
  assert!(!fresh(&s));
 }
 let (mut s,ticket)=seed();
 publish(&mut s,ticket,0,1,2,10);
 assert!(fresh(&s));
 s.raw_tick=11;
 assert!(!fresh(&s),"recent wall-clock timestamp cannot authorize replay");
 publish(&mut s,ticket,0,1,2,10);
 assert_eq!(s.evaluated_tick,Some(10));
 assert!(!fresh(&s));
 publish(&mut s,ticket,0,1,2,11);
 assert!(fresh(&s),"fresh synchronous observation delivers immediately");
}
#[test]fn session_route_and_cancellation_fences_remain_required() {
 for (index,id,sequence) in [(1,1,2),(0,2,2),(0,1,1)] {
  let (mut s,ticket)=seed();
  publish(&mut s,ticket,index,id,sequence,10);
  assert_eq!(s.evaluated_tick,None);
 }
 let (mut s,ticket)=seed();s.session.update(false);s.session.update(true);
 publish(&mut s,ticket,0,1,2,10);
 assert_eq!(s.evaluated_tick,None);
 let (mut s,ticket)=seed();publish(&mut s,ticket,0,1,2,10);
 s.request_seen=Some(Instant::now()-Duration::from_millis(36));
 assert!(!fresh(&s),"same tick still obeys wall-clock timeout");
}
mod manual_native_wiring {
 use kirby_assist_core::{action_owner::{ActionOwnership,Owner,Intent},
  gameplay::{GameplayContext,ControllerOutput},grab_handoff::GrabHandoff,manual_controls::ManualJumpFence,
  out_of_shield::OutOfShield,spot_dodge::SpotDodge,
  native_input::{MAPPER_X,MAPPER_A}};
 struct Menu {actions:u32,reserved:bool}
 impl Menu {
  fn reserves_automatic_modes(&self)->bool{self.reserved}
  fn for_fighter(&self,_kind:i32)->Option<&Self>{Some(self)}
  fn manual_native_actions(&self)->u32{self.actions}
 }
 struct NativeState {
  settings_menu:Option<Menu>,manual_jump_fence:ManualJumpFence,
  ledge:Option<kirby_assist_core::ledge_trap::LedgeTrap>,oos:OutOfShield,spot_dodge:Option<SpotDodge>,
  tech_output:kirby_assist_core::tech_input::TechOutput,mash_stick:Option<[f32;2]>,
  grab_handoff:GrabHandoff,action_owner:ActionOwnership,ledge_active:bool,
 }
 impl NativeState {
  fn new()->Self {Self {settings_menu:Some(Menu{actions:0,reserved:false}),manual_jump_fence:ManualJumpFence::new(),
   ledge:None,oos:OutOfShield::new(),spot_dodge:None,tech_output:Default::default(),mash_stick:None,
   grab_handoff:GrabHandoff::new(),action_owner:ActionOwnership::new(),ledge_active:false}}
 }
 fn blocked(state:&mut NativeState,context:GameplayContext)->bool {
  let object_id=1;let fighter_kind=15;
  let native_status=context.status;let native_auto_ground=context.situation;
  let native_auto_stopped=context.stopped;let own_motion_kind=context.motion_kind;
  let ledge_motion_frame=context.motion_frame;let timing_rate=context.motion_rate;
  MANUAL_GUARD
  manual_automatic_blocked
 }
 fn c()->GameplayContext{GameplayContext{gameplay:true,motion_kind:1,motion_rate:1.,..Default::default()}}
 #[test]fn actual_live_guard_respects_manual_jump_fence() {
  let mut s=NativeState::new();assert!(!blocked(&mut s,c()));
  s.settings_menu.as_mut().unwrap().actions=MAPPER_X;
  assert!(blocked(&mut s,c()));
  s.settings_menu.as_mut().unwrap().actions=0;
  for _ in 0..100 {assert!(blocked(&mut s,c()));}
  assert!(!blocked(&mut s,GameplayContext{status:0xa,..c()}));
  assert!(!blocked(&mut s,c()));
 }
 #[test]fn active_sequences_ignore_manual_binding_edges_without_priming_a_late_jump() {
  for owner in [Owner::RollRead,Owner::OutOfShield,Owner::OptionRead,Owner::Reflect,Owner::Ledge] {
   let mut s=NativeState::new();s.action_owner.select(true,&[Intent::input_sequence(owner,1)]);
   s.settings_menu.as_mut().unwrap().actions=MAPPER_X|MAPPER_A;
   assert!(!blocked(&mut s,c()));
   s.settings_menu.as_mut().unwrap().actions=0;s.action_owner.release(owner);
   assert!(!blocked(&mut s,c()),"ignored jump must not become a deferred manual fence");
  }
 }
 fn binding_guard(state:&NativeState,damage_recovery:bool)->(bool,bool) {
  BINDING_GUARD
  (bindings_blocked,safety_jump)
 }
 #[test]fn inactive_recovery_keeps_manual_jump_but_no_committed_owner_accepts_a_jump_override() {
  let mut s=NativeState::new();assert_eq!(binding_guard(&s,false),(false,false));
  assert_eq!(binding_guard(&s,true),(true,true));
  for owner in [Owner::Safety,Owner::RollRead,Owner::OutOfShield,Owner::Ledge,Owner::OptionRead,Owner::Reflect] {
   s.action_owner.select(true,&[Intent::input_sequence(owner,1)]);
   assert_eq!(binding_guard(&s,true),(true,false));s.action_owner.release(owner);
  }
  s.tech_output.press=true;assert_eq!(binding_guard(&s,true),(true,false));
 }
 fn reserve_ledge(state:&mut NativeState,ledge_inputs_reserved:bool) {LEDGE_RESERVATION}
 fn choose(state:&mut NativeState)->Option<Owner> {
  let fresh=true;let manual_scope=true;let native_grab=false;
  let mut intents=[Intent::input_sequence(Owner::Ledge,1),Intent::input_sequence(Owner::OutOfShield,1)];
  ARBITRATION
  winner
 }
 #[test]fn actual_mapper_arbitration_waits_for_prior_grab_receipt_before_ledge_playback() {
  let mut s=NativeState::new();
  let context=kirby_assist_core::out_of_shield::Context {object:1,kind:15,status:0x1c,motion:1,frame:0.,rate:1.,
   stopped:false,grounded:true,position:[0.;2],facing:1.,shield_ready:true,can_grab:true,
   can_jump:false,can_up_special:false,threatened:false};
  let mut targets=[None;8];targets[0]=Some(kirby_assist_core::roll_read::Target {
   id:2,status:0x1c,motion:2,frame:0.,rate:1.,position:[10.,0.],velocity_x:0.,
   vulnerable_in:Some(0.),actionable_in:None});
  s.oos.sample_automatic(true,context,GameplayContext{status:0x1c,held:true,..c()},targets,None,true,22.,35.,Some(8.));
  s.oos.mapped_grab(0,2);assert!(s.oos.grab_delivery_pending());
  s.grab_handoff.mapped(0,Some(Owner::OutOfShield),true,true,true);
  s.action_owner.set_automatic_suspended(true);reserve_ledge(&mut s,true);
  assert!(s.oos.grab_delivery_pending());assert!(s.grab_handoff.pending());
  assert_eq!(choose(&mut s),None);
  for frame in 0..=24 {s.grab_handoff.observe_native(true,Some(GameplayContext{motion_frame:frame as f32,..c()}));}
  assert!(!s.grab_handoff.pending());
  assert_eq!(choose(&mut s),Some(Owner::Ledge));
  assert!(s.grab_handoff.permits(Some(Owner::Ledge),ControllerOutput{requested:kirby_assist_core::native_input::MAPPER_ZR,
   stick:Some([0.;2]),..Default::default()},None));
 }
 #[test]fn actual_ledge_reservation_preserves_its_own_grab_receipt() {
  let mut s=NativeState::new();s.grab_handoff.mapped(0,Some(Owner::Ledge),true,true,true);
  s.action_owner.set_automatic_suspended(true);reserve_ledge(&mut s,true);
  assert!(s.grab_handoff.pending());assert_eq!(choose(&mut s),Some(Owner::Ledge));
 }
 fn proposals_blocked(state:&mut NativeState,automatic_suspended:bool)->bool {
  let observe_ledge=true;let ledge_opponent:Option<kirby_assist_core::gameplay::Opponent>=None;
  RECEIPT_GUARD
  automatic_suspended
 }
 #[test]fn fresh_native_shield_readiness_does_not_keep_a_stale_raw_cancel_frame() {
  let mut s=NativeState::new();s.action_owner.set_automatic_suspended(true);
  assert!(!blocked(&mut s,GameplayContext{status:0x1c,..c()}));
  assert!(!proposals_blocked(&mut s,false));assert!(!s.action_owner.automatic_suspended());
  assert!(proposals_blocked(&mut s,true));assert!(s.action_owner.automatic_suspended());
 }
 #[test]fn held_ledge_admission_does_not_interrupt_current_work_or_admit_a_new_auto_on_completion() {
  let mut s=NativeState::new();s.ledge_active=true;
  s.action_owner.select(true,&[Intent::input_sequence(Owner::RollRead,1)]);
  assert!(!proposals_blocked(&mut s,false));assert!(s.action_owner.allows(Owner::RollRead));
  assert!(!s.action_owner.allows(Owner::OutOfShield));
  s.action_owner.release(Owner::RollRead);
  assert!(!s.action_owner.allows(Owner::OutOfShield));assert!(s.action_owner.allows(Owner::Ledge));
 }
 #[test]fn actual_live_guard_keeps_watches_live_but_blocks_proposals_behind_a_queued_grab() {
  let mut s=NativeState::new();blocked(&mut s,c());
  s.grab_handoff.mapped(0,Some(Owner::OutOfShield),true,true,true);
  for _ in 0..100 {assert!(!blocked(&mut s,c()));assert!(proposals_blocked(&mut s,false));}
  assert!(!blocked(&mut s,GameplayContext{status:0x37,..c()}));
  assert!(!proposals_blocked(&mut s,false));
  let mut s=NativeState::new();blocked(&mut s,c());
  s.grab_handoff.mapped(0,Some(Owner::OutOfShield),true,true,true);
  s.action_owner.select(true,&[Intent::input_sequence(Owner::OutOfShield,1)]);
  assert!(!blocked(&mut s,c()),"existing owner can finish its bounded initial R pulse");
 }
}
mod exclusive_packet_wiring {
 use kirby_assist_core::{action_owner::{ActionOwnership,Owner,Intent},gameplay::ControllerOutput,ledge_trap::SEQUENCE_BUTTONS,
  tech_input::TechOutput,native_input::{MAPPER_ZR,MAPPER_B}};
 struct State {action_owner:ActionOwnership,tech_output:TechOutput,tech_urgent:bool}
 fn packet(state:&mut State,winner:Option<Owner>,mut selected:ControllerOutput,mash_stick:Option<[f32;2]>,damage_stick:Option<[f32;2]>)->(ControllerOutput,Option<TechOutput>) {
  let di_stick=damage_stick;let sdi_stick=None;
  let mut mapped_tech_output=None;
  SELECTED_PACKET
  (selected,mapped_tech_output)
 }
 #[test]fn actual_live_packet_has_one_exclusive_winner_including_wait_and_safety_phases() {
  for owner in [Owner::Ledge,Owner::RollRead,Owner::OutOfShield,Owner::OptionRead,Owner::Trump,Owner::Trip,Owner::Fair,Owner::Reflect] {
   let mut s=State{action_owner:ActionOwnership::new(),tech_output:TechOutput{press:true,owns_guard:true},tech_urgent:false};
   s.action_owner.select(true,&[Intent::input_sequence(owner,1)]);
   let (p,t)=packet(&mut s,Some(owner),ControllerOutput{requested:MAPPER_B,..Default::default()},Some([1.,0.]),Some([0.,1.]));
   assert_eq!(p.owned,SEQUENCE_BUTTONS);assert_eq!(p.requested,MAPPER_B);assert_eq!(p.stick,Some([0.;2]));assert!(t.is_none());
   let (p,_)=packet(&mut s,Some(owner),ControllerOutput::default(),None,None);
   assert_eq!(p.owned,SEQUENCE_BUTTONS);assert_eq!(p.requested,0);assert_eq!(p.stick,Some([0.;2]));
  }
  let mut s=State{action_owner:ActionOwnership::new(),tech_output:TechOutput{press:true,owns_guard:true},tech_urgent:false};
  s.action_owner.select(true,&[Intent::input_sequence(Owner::Safety,1)]);
  let (p,t)=packet(&mut s,Some(Owner::Safety),ControllerOutput::default(),None,Some([0.,1.]));
  assert_eq!(p.requested,0);assert_eq!(p.stick,Some([0.,1.]));assert!(t.is_none());
  s.tech_urgent=true;
  let (p,t)=packet(&mut s,Some(Owner::Safety),ControllerOutput::default(),None,Some([0.,1.]));
  assert_eq!(p.requested,MAPPER_ZR);assert_eq!(p.stick,Some([0.;2]));assert!(t.is_some());
  let (p,t)=packet(&mut s,Some(Owner::Safety),ControllerOutput::default(),Some([1.,0.]),Some([0.,1.]));
  assert_eq!(p.requested,0);assert_eq!(p.stick,Some([1.,0.]));assert!(t.is_none());
 }
}
mod literal_delivery_wiring {
 use kirby_assist_core::{action_owner::Owner,gameplay::ControllerOutput,ledge_trap::SEQUENCE_BUTTONS,native_input::*};
 use kirby_assist_core::out_of_shield::OutOfShield;
 struct Config{control_layout:ControlLayout}struct Raw{buttons:ButtonTrace}
 struct State{oos:OutOfShield}
 fn request(winner:Option<Owner>,requested:u32,held:u32,owned:u32,
  layout:ControlLayout,fresh:bool,manual_scope:bool,exclusive_sequence:bool)->(bool,Option<u32>) {
  let selected=ControllerOutput{requested,owned,..Default::default()};
  let canonical_requested=requested;let config=Config{control_layout:layout};
  let manual=Raw{buttons:ButtonTrace{held,..Default::default()}};
  let state=State{oos:OutOfShield::new()};
  let mut mapped_attack_hold=false;let mut mapped_literal_request=None;let mut mapped_unshielded_auto_grab=false;
  ATTACK_DELIVERY
  LITERAL_DELIVERY
  (mapped_attack_hold,mapped_literal_request)
 }
 #[test]fn actual_selected_ledge_smash_keeps_attack_for_both_control_layouts() {
  for layout in [ControlLayout::Default,ControlLayout::Brother] {
   for owner in [Owner::Ledge,Owner::RollRead,Owner::OutOfShield,Owner::OptionRead,Owner::Safety,Owner::Trump,Owner::Fair,Owner::Trip,Owner::Reflect] {
    let (attack,packet)=request(Some(owner),MAPPER_A,layout.physical_buttons(MAPPER_A),SEQUENCE_BUTTONS,layout,true,true,true);
    assert_eq!(attack,matches!(owner,Owner::Ledge|Owner::RollRead|Owner::OutOfShield|Owner::OptionRead));assert_eq!(packet,Some(MAPPER_A));
    assert_eq!(mapped_literal_buttons(0,packet.unwrap()),0x201);
   }
  }
 }
 #[test]fn actual_delivery_requires_fresh_selected_owned_and_surviving_input() {
  for (fresh,scope,owned,held,owner) in [(false,true,SEQUENCE_BUTTONS,MAPPER_A,Some(Owner::Ledge)),
   (true,false,SEQUENCE_BUTTONS,MAPPER_A,Some(Owner::Ledge)),(true,true,0,MAPPER_A,Some(Owner::Ledge)),
   (true,true,SEQUENCE_BUTTONS,0,Some(Owner::Ledge)),
   (true,true,SEQUENCE_BUTTONS,MAPPER_A,None)] {
   let (attack,packet)=request(owner,MAPPER_A,held,owned,ControlLayout::Default,fresh,scope,true);
   assert!(!attack);assert_eq!(packet.unwrap_or(0),0);
  }
 }
 #[test]fn actual_oos_request_keeps_catch_and_special_distinct_and_releases_old_shield() {
  for layout in [ControlLayout::Default,ControlLayout::Brother] {
   for (requested,expected) in [(MAPPER_R|MAPPER_ZR,MAPPED_CATCH|MAPPED_GUARD),
    (MAPPER_B|MAPPER_ZR,MAPPED_SPECIAL|MAPPED_GUARD),(0,0)] {
    let (_,packet)=request(Some(Owner::OutOfShield),requested,layout.physical_buttons(requested),SEQUENCE_BUTTONS,layout,true,true,true);
    let stale=if requested==0 {MAPPED_GUARD|MAPPED_GUARD_HOLD|MAPPED_CATCH}else{0};
    assert_eq!(mapped_literal_buttons(stale,packet.unwrap()),expected);
   }
  }
 }
}
'''.replace('BINDING_GUARD', binding_guard).replace('SELECTED_PACKET', selected_packet).replace('ATTACK_DELIVERY', attack_delivery).replace('LITERAL_DELIVERY', literal_delivery).replace('MANUAL_GUARD', manual_guard).replace('RECEIPT_GUARD', receipt_guard).replace('ARBITRATION', arbitration).replace('LEDGE_RESERVATION', ledge_reservation).replace('\n GUARD\n', '\n'+guard+'\n').replace('\n FRESH\n', '\n'+fresh+'\n')
    deps = ROOT / 'crates/kirby_assist_core/target/debug/deps'
    libs = list(deps.glob('libkirby_assist_core-*.rlib'))
    if not libs:
        raise SystemExit('Run a core host test first.')
    library = max(libs, key=lambda p: p.stat().st_mtime)
    with tempfile.TemporaryDirectory(prefix='kirby-input-publication-') as directory:
        rust = Path(directory) / 'wiring.rs'
        binary = Path(directory) / 'wiring-tests'
        rust.write_text(fixture)
        subprocess.run(['rustc', '--edition=2021', '--test', str(rust), '--extern',
                        f'kirby_assist_core={library}', '-L', f'dependency={deps}',
                        '-o', str(binary)], check=True)
        subprocess.run([str(binary)], check=True)

if __name__ == '__main__':
    main()
