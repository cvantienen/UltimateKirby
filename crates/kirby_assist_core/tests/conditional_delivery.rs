//! Conditional actions use copied facts and continue on the satisfying sample.
use kirby_assist_core::{
    ledge_trap::{InputTimeline,LedgeContext,LedgeSettings,Opponent,MotionTiming},
    native_input::*,sequence_config::Catalog,
    reaction_rules::{Candidate,Engine,Resolved},roll_read::Target,
};
fn context()->LedgeContext {LedgeContext{held:true,gameplay:true,can_act:true,motion_kind:1,motion_frame:10.,motion_rate:1.,facing:1.,safe_ground:[true;2],..Default::default()}}
fn enemy(x:f32)->Opponent {Opponent{id:2,position:[x,0.],motion:Some(MotionTiming{kind:2,frame:10.,rate:1.,remaining:None}),..Default::default()}}
fn timeline(actions:&str,e:Opponent)->InputTimeline {
    let catalog=Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\nactions={actions}\n")).unwrap();
    InputTimeline::new(catalog.entries()[0].sequence,e)
}
fn rules(event:&str,actions:&str)->Resolved {
    let catalog=Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\nreaction={event}:40:20\nactions={actions}\n")).unwrap();
    let mut rules=Resolved::default();rules.add(Candidate{slot:2,index:0,sequence:catalog.entries()[0].sequence}).unwrap();rules
}
fn react(rules:&Resolved,status:i32,own_status:i32)->kirby_assist_core::ledge_trap::LedgeOutput {
    let mut targets=[None;8];targets[0]=Some(Target{id:2,status,motion:2,frame:10.,rate:1.,position:[10.,0.],velocity_x:0.,vulnerable_in:None,actionable_in:None});
    Engine::default().sample_simple(true,rules,LedgeSettings::default(),LedgeContext{status:own_status,..context()},targets,None)
}
#[test]
fn satisfied_self_enemy_and_distance_checks_add_no_input_frame(){
    for (check,own_situation,other_situation,status,x) in [
        ("self_grounded:0",0,0,0,10.),("self_airborne:0",2,0,0,10.),
        ("enemy_grounded:0",0,0,0,10.),("enemy_airborne:0",0,2,0x22,10.),
        ("enemy_shielding:0",0,0,0x1c,10.),("enemy_not_shielding:0",0,0,0,10.),
        ("enemy_in_front:0",0,0,0,10.),("enemy_behind:0",0,0,0,-10.),
        ("enemy_within:10:0",0,0,0,10.),("enemy_outside:5:0",0,0,0,10.),
    ] {
        let other=Opponent{situation:other_situation,status,..enemy(x)};
        let mut t=timeline(&format!("check:{check},up_b:4"),other);
        let out=t.sample(LedgeSettings::default(),LedgeContext{situation:own_situation,..context()},Some(other),None,0.);
        assert_eq!(out.requested,MAPPER_B,"{check}");
        assert_eq!(out.stick,Some([0.,1.]));
    }
}
#[test]
fn satisfied_face_and_approach_keep_direct_out_of_shield_up_b_immediate(){
    for actions in ["face:opponent,up_b:4","face:forward,check:enemy_within:20:0,up_b:4","walk_to_opponent:50%:10:20,up_b:4","chase_if_far:20:1:14,up_b:4"] {
        let out=react(&rules("auto_roll",actions),0x20,0x1c);
        assert_eq!(out.requested,MAPPER_B|MAPPER_ZR,"{actions}");
        assert_eq!(out.stick,Some([0.,1.]));
    }
    // An actual turnaround or approach still must release shield normally.
    let other=enemy(-10.);let mut t=timeline("face:opponent,up_b:4",other);
    let out=t.sample(LedgeSettings::default(),LedgeContext{status:0x1c,..context()},Some(other),None,0.);
    assert_eq!(out.requested,0);assert_eq!(out.stick,Some([0.;2]));
}
#[test]
fn copied_conditions_refresh_during_a_duplicate_own_animation_sample(){
    let c=context();let e=enemy(10.);
    let mut drift=timeline("drift:opponent:1:4,up_b:4",e);
    assert_eq!(drift.sample(LedgeSettings::default(),c,Some(e),None,0.).stick,Some([1.,0.]));
    assert_eq!(drift.sample(LedgeSettings::default(),c,Some(enemy(-10.)),None,0.).stick,Some([-1.,0.]));
    let mut face=timeline("face:opponent,up_b:4",enemy(-10.));
    assert_eq!(face.sample(LedgeSettings::default(),c,Some(enemy(-10.)),None,0.).requested,0);
    assert_eq!(face.sample(LedgeSettings::default(),c,Some(e),None,0.).requested,MAPPER_B);
    let mut approach=timeline("walk_to_opponent:50%:10:5,up_b:4",e);
    assert_eq!(approach.sample(LedgeSettings::default(),c,Some(e),None,0.).stick,Some([0.5,0.]));
    assert_eq!(approach.sample(LedgeSettings::default(),c,Some(enemy(4.)),None,0.).requested,MAPPER_B);
    let mut check=timeline("check:enemy_within:5:10,up_b:4",e);
    assert_eq!(check.sample(LedgeSettings::default(),c,Some(e),None,0.).requested,0);
    assert_eq!(check.sample(LedgeSettings::default(),c,Some(enemy(4.)),None,0.).requested,MAPPER_B);
}
#[test]
fn first_air_dodge_observation_has_airborne_data_before_any_follow_sample(){
    let r=rules("auto_air_dodge","check:enemy_airborne:0,up_b:4");
    for status in [0x22,0x23] {assert_eq!(react(&r,status,0).requested,MAPPER_B);}
    let ground=rules("auto_roll","check:enemy_grounded:0,up_b:4");
    for status in [0x20,0x21] {assert_eq!(react(&ground,status,0).requested,MAPPER_B);}
}
#[test]
fn cstick_up_smash_retains_guard_without_an_added_shield_drop(){
    let out=react(&rules("auto_roll","cstick:0:1:4"),0x20,0x1c);
    assert_eq!(out.requested,MAPPER_RIGHT_STICK_UP|MAPPER_ZR);
}

#[test]
fn canonical_zl_is_an_out_of_shield_jump_not_a_shield_or_unrequested_jump(){
    let out=react(&rules("auto_roll","input:zl:0:0:1"),0x20,0x1c);
    assert_eq!(out.requested,MAPPER_ZL|MAPPER_ZR);
}
