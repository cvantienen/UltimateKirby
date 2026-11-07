//! Configured spot-dodge punish, using existing bounded recovery observations.
use crate::{defense_response::Response, ledge_trap::{LedgeContext,LedgeOutput,LedgeSettings,Opponent},
    option_read::{Kind,Target},reaction_rules::{Resolved,Trigger}};
#[derive(Clone,Copy)]
struct Seen { id:u32,status:i32,motion:u64,frame:f32,used:bool }
#[derive(Default)]
pub struct Punisher { object:Option<u32>, seen:[Option<Seen>;8], response:Response }
impl Punisher {
    pub fn target(&self)->Option<u32>{self.response.target()}
    pub fn committed(&self)->bool{self.target().is_some()}
    pub fn needs_ground(&self)->bool{self.response.needs_ground()}
    pub fn walking(&self)->bool{self.response.walking()}
    pub fn attack_hold(&self)->bool{self.response.attack_hold()}
    pub fn right_stick(&self)->Option<[f32;2]>{self.response.right_stick()}
    pub fn cancel_pending(&mut self){self.response.cancel();}
    pub fn sample(&mut self,enabled:bool,object:u32,rules:Option<&Resolved>,settings:LedgeSettings,
        c:LedgeContext,targets:[Option<Target>;8],follow:Option<Opponent>)->LedgeOutput {
        if self.object!=Some(object){*self=Self::default();self.object=Some(object);}
        let old=self.seen;
        self.seen=std::array::from_fn(|i|targets[i].filter(|t|t.kind==Kind::Spot&&t.status==0x1f
            &&t.motion!=0&&t.rate.is_finite()&&t.rate>0.&&t.frame.is_finite()&&t.frame>=0.
            &&t.position.iter().chain(t.velocity.iter()).all(|v|v.is_finite()))
            .map(|t|Seen{id:t.id,status:t.status,motion:t.motion,frame:t.frame,
                used:old.iter().flatten().any(|p|p.id==t.id&&p.status==t.status&&p.motion==t.motion&&t.frame>=p.frame&&p.used)}));
        if !enabled||rules.is_none()||!c.gameplay {self.cancel_pending();return LedgeOutput::default();}
        let mut following=follow.filter(|e|Some(e.id)==self.target());
        if !self.committed()&&!c.stopped {
            let rules=rules.unwrap();
            let mut best=None::<(u8,usize,Opponent)>;
            for (i,target) in targets.iter().enumerate() {
                let (Some(t),Some(seen))=(target,self.seen[i])else{continue;};
                if seen.used{continue;}
                let enemy=Opponent{id:t.id,status:t.status,situation:0,position:t.position,
                    velocity:t.velocity,motion:Some(crate::ledge_trap::MotionTiming{kind:t.motion,frame:t.frame,rate:t.rate,remaining:t.closes.is_finite().then_some(t.closes)}),vulnerable:t.opens.is_finite()&&t.opens<=0.&&t.closes.is_finite()&&t.closes>0.,vulnerable_in:t.opens.is_finite().then_some(t.opens),..Default::default()};
                if let Some(sequence)=crate::defense_response::select(rules,Trigger::SpotDodge,c,enemy) {
                    let priority=sequence.reaction.priority;
                    if best.is_none_or(|(p,_,_)|priority<p){best=Some((priority,i,enemy));}
                }
            }
            if let Some((_,index,enemy))=best {
                if self.response.start(rules,Trigger::SpotDodge,c,enemy){
                    self.seen[index].as_mut().unwrap().used=true;following=Some(enemy);
                }
            }
        }
        self.response.sample(true,settings,c,following)
    }
}
