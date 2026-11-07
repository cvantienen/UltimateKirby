#!/usr/bin/env python3
"""Host-compile actual reflection publication/cancellation/delivery statements.

Run a core test first to produce its host rlib. Never builds a Switch plugin.
"""
from pathlib import Path
import subprocess
import tempfile

ROOT=Path(__file__).resolve().parents[1]

def block(source,marker):
    start=source.index(marker)
    brace=source.index('{',start)
    depth,end=1,brace+1
    while depth:
        depth+=(source[end]=='{')-(source[end]=='}')
        end+=1
    return source[start:end]

def main():
    source=(ROOT/'src/live.rs').read_text()
    start=source.index('        state.reflect_output = if let Some')
    end=source.index('        state.mash_stick =',start)
    publication=source[start:end]
    cancel=block(source,'if keep!=Some(Owner::Reflect)')
    receipt=block(source,'if mapped_reflect &&')
    poll=block(source,'if let Some(tick)=mapped_reflector')
    fixture=r'''
#![allow(dead_code)]
use kirby_assist_core::{action_owner::{ActionOwnership,Owner},
    gameplay::ControllerOutput,native_input::*,out_of_shield::OutOfShield,
    projectile_reflect::{Config,Context,Threat,Reflector},timing::AnimationSample,
    fighter_tuning::Tuning};
#[derive(Default)]
struct State {
    reflector:Reflector,reflect_output:ControllerOutput,action_owner:ActionOwnership,
    ledge_active:bool,oos:OutOfShield,oos_output:ControllerOutput,
    roll_read_output:ControllerOutput,spot_output:ControllerOutput,trip_output:ControllerOutput,
    fair_output:ControllerOutput,jab_lock_output:ControllerOutput,trump_output:ControllerOutput,
    raw_tick:i64,
}
fn publish(state:&mut State,reflect_observation:Option<(Config,Context,Option<Threat>)>,clatter:bool,tech:bool) {
    PUBLICATION
}
fn cancel(state:&mut State,keep:Option<Owner>) {
    let State{reflector,reflect_output,..}=state;
    struct Ref<'a>{reflector:&'a mut Reflector,reflect_output:&'a mut ControllerOutput}
    impl Ref<'_> {fn run(&mut self,keep:Option<Owner>){CANCEL}}
    Ref{reflector,reflect_output}.run(keep);
}
#[derive(Clone,Copy)]struct Inputs{buttons:u32,left_y:i8}
unsafe fn receipt(state:&mut State,inputs:Inputs,mapped_reflect:bool) {
    let mapped=&inputs as *const Inputs;
    RECEIPT
}
fn poll(state:&mut State,mapped_reflector:Option<i64>) {POLL}
fn observation(frame:f32,status:i32)->Option<(Config,Context,Option<Threat>)> {
    let mut t=Tuning::default();t.set("reflect_default","true").unwrap();
    Some((Config::for_fighter(28,t).unwrap(),Context{kind:28,situation:0,facing:1.,
        can_special:true,gauge:0.,stopped:false,animation:AnimationSample{
            object:1,status,kind:1,frame,auxiliary:0.}},Some(Threat{id:2,generation:0,in_frames:7.,direction:1.})))
}
#[test]fn production_publication_yields_to_every_existing_punish_packet_and_ledge() {
    for competitor in 0..10 {
        let mut state=State::default();let attack=kirby_assist_core::move_input::output([0.,1.],MAPPER_B);
        match competitor {0=>state.oos_output=attack,1=>state.roll_read_output=attack,
            2=>state.spot_output=attack,3=>state.trip_output=attack,4=>state.fair_output=attack,
            5=>state.jab_lock_output=attack,6=>state.trump_output=attack,7=>state.ledge_active=true,
            8=>state.action_owner.set_automatic_suspended(true),_=>{}}
        publish(&mut state,observation(0.,0),false,competitor==9);
        assert_eq!(state.reflect_output.owned,0);assert!(!state.reflector.active());
    }
    let mut state=State::default();publish(&mut state,observation(0.,0),false,false);
    assert_eq!(state.reflect_output.requested,MAPPER_B);
    assert_eq!(state.reflect_output.stick,Some([0.,-1.]));
}
#[test]fn only_final_delivered_down_special_arms_native_bucket_confirmation() {
    for inputs in [Inputs{buttons:0,left_y:-127},Inputs{buttons:MAPPED_SPECIAL,left_y:127},
        Inputs{buttons:MAPPED_SPECIAL,left_y:-127}] {
        let mut state=State::default();publish(&mut state,observation(0.,0),false,false);
        unsafe{receipt(&mut state,inputs,true);}
        publish(&mut state,observation(1.,0x1e3),false,false);
        assert_eq!(state.reflector.active(),inputs.buttons==MAPPED_SPECIAL && inputs.left_y< -63);
    }
}
#[test]fn production_owner_cancellation_clears_input_without_a_late_second_attempt() {
    let mut state=State::default();publish(&mut state,observation(0.,0),false,false);
    unsafe{receipt(&mut state,Inputs{buttons:MAPPED_SPECIAL,left_y:-127},true);}
    cancel(&mut state,Some(Owner::Safety));assert!(!state.reflector.active());
    assert_eq!(state.reflect_output.owned,0);
    publish(&mut state,observation(2.,0),false,false);
    assert_eq!(state.reflect_output.owned,0);
}
#[test]fn production_unseen_delivery_cleanup_releases_cached_input_and_owner() {
    let mut state=State::default();publish(&mut state,observation(0.,0),false,false);
    for tick in 0..24 {poll(&mut state,Some(tick));}
    assert!(!state.reflector.active());assert_eq!(state.reflect_output.owned,0);
    publish(&mut state,observation(1.,0),false,false);
    assert_eq!(state.reflect_output.owned,0);
}
'''.replace('PUBLICATION',publication).replace('CANCEL',cancel).replace('RECEIPT',receipt).replace('POLL',poll)
    # The production cancellation block acts on ordinary fields; dereference
    # the tiny borrowed fixture's output slot when assigning its copied value.
    fixture=fixture.replace('self.reflect_output=ControllerOutput::default();','*self.reflect_output=ControllerOutput::default();')
    deps=ROOT/'crates/kirby_assist_core/target/debug/deps'
    libraries=list(deps.glob('libkirby_assist_core-*.rlib'))
    if not libraries:raise SystemExit('Run the core projectile_reflect test first.')
    library=max(libraries,key=lambda p:p.stat().st_mtime)
    with tempfile.TemporaryDirectory(prefix='kirby-reflect-wiring-') as directory:
        rust=Path(directory)/'wiring.rs';binary=Path(directory)/'wiring-tests'
        rust.write_text(fixture)
        subprocess.run(['rustc','--edition=2021','--test',str(rust),'--extern',
            f'kirby_assist_core={library}','-L',f'dependency={deps}','-o',str(binary)],check=True)
        subprocess.run([str(binary)],check=True)

if __name__=='__main__':main()
