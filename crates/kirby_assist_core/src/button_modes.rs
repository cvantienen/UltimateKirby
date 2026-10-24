//! Small compiled action list for the swappable Y binding. Add future actions
//! here, then route their inputs explicitly in live.rs; no runtime scripts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum YMode {
    #[default]
    FairSmash,
    DairSmash,
    YoshiBairSmash,
}
impl YMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::FairSmash => "Fair 1 > Fsmash",
            Self::DairSmash => "Dair > Dsmash",
            Self::YoshiBairSmash => "Bair drag > Fsmash",
        }
    }
    pub fn options(kirby: bool) -> &'static [Self] {
        if kirby {
            &[Self::FairSmash, Self::DairSmash]
        } else {
            &[]
        }
    }
    pub fn cycle(self, forward: bool, kirby: bool) -> Self {
        self.cycle_for_kind(forward, if kirby { 6 } else { -1 })
    }
    pub fn options_for_kind(kind: i32) -> &'static [Self] {
        match kind {
            6 => &[Self::FairSmash, Self::DairSmash],
            5 => &[Self::YoshiBairSmash],
            _ => &[],
        }
    }
    pub fn supports(self, kind: i32) -> bool {
        Self::options_for_kind(kind).contains(&self)
    }
    pub fn cycle_for_kind(self, forward: bool, kind: i32) -> Self {
        let options = Self::options_for_kind(kind);
        if options.is_empty() {
            return self;
        }
        let index = options.iter().position(|v| *v == self).unwrap_or(0);
        options[(index + if forward { 1 } else { options.len() - 1 }) % options.len()]
    }
}
