//! Select compiled executable response logic by our fighter’s native kind.
//! Register both a fighter profile and its implementation when adding a fighter.
use super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};

pub(crate) mod bayonetta;
pub(crate) mod bowser;
pub(crate) mod bowser_jr;
pub(crate) mod captain;
pub(crate) mod charizard;
pub(crate) mod chrom;
pub(crate) mod cloud;
mod common;
pub(crate) mod corrin;
pub(crate) mod daisy;
pub(crate) mod dark_pit;
pub(crate) mod dark_samus;
pub(crate) mod diddy_kong;
pub(crate) mod donkey;
pub(crate) mod dr_mario;
pub(crate) mod duck_hunt;
pub(crate) mod falco;
pub(crate) mod fox;
pub(crate) mod ganondorf;
pub(crate) mod greninja;
pub(crate) mod ice_climbers;
pub(crate) mod ice_climbers_nana;
pub(crate) mod ike;
pub(crate) mod incineroar;
pub(crate) mod inkling;
pub(crate) mod isabelle;
pub(crate) mod ivysaur;
pub(crate) mod jigglypuff;
pub(crate) mod ken;
pub(crate) mod king_dedede;
pub(crate) mod kirby;
pub(crate) mod krool;
pub(crate) mod link;
pub(crate) mod littlemac;
pub(crate) mod lucario;
pub(crate) mod lucas;
pub(crate) mod lucina;
pub(crate) mod luigi;
pub(crate) mod mario;
pub(crate) mod marth;
pub(crate) mod mega_man;
pub(crate) mod meta_knight;
pub(crate) mod mewtwo;
pub(crate) mod mii_brawler;
pub(crate) mod mii_gunner;
pub(crate) mod mii_swordfighter;
pub(crate) mod mr_game_and_watch;
pub(crate) mod ness;
pub(crate) mod olimar;
pub(super) mod one_shot;
pub(crate) mod pacman;
pub(crate) mod palutena;
pub(crate) mod peach;
pub(crate) mod pichu;
pub(crate) mod pikachu;
pub(crate) mod pit;
pub(crate) mod richter;
pub(crate) mod ridley;
pub(crate) mod rob;
pub(crate) mod robin;
pub(crate) mod rosalina_and_luma;
pub(crate) mod roy;
pub(crate) mod ryu;
pub(crate) mod samus;
pub(crate) mod sheik;
pub(crate) mod shulk;
pub(crate) mod simon;
pub(crate) mod snake;
pub(crate) mod sonic;
pub(crate) mod squirtle;
mod standard;
pub(crate) mod toon_link;
mod variety;
pub(crate) mod villager;
pub(crate) mod wario;
pub(crate) mod wii_fit_trainer;
pub(crate) mod wolf;
pub(crate) mod yoshi;
pub(crate) mod young_link;
pub(crate) mod zelda;
pub(crate) mod zero_suit_samus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseImplementation {
    Kirby,
    Luigi,
    CaptainFalcon,
    Mario,
    DonkeyKong,
    Link,
    Samus,
    Yoshi,
    Fox,
    Pikachu,
    Sonic,
    KingKRool,
    Isabelle,
    PacMan,
    LittleMac,
    DarkSamus,
    Ness,
    Jigglypuff,
    Peach,
    Daisy,
    Bowser,
    Sheik,
    Zelda,
    DrMario,
    Pichu,
    Falco,
    Marth,
    Lucina,
    YoungLink,
    Ganondorf,
    Mewtwo,
    Roy,
    Chrom,
    MrGameAndWatch,
    MetaKnight,
    Pit,
    DarkPit,
    ZeroSuitSamus,
    Wario,
    Snake,
    Ike,
    Squirtle,
    Ivysaur,
    Charizard,
    DiddyKong,
    Lucas,
    KingDedede,
    Olimar,
    Lucario,
    Rob,
    ToonLink,
    Wolf,
    Villager,
    MegaMan,
    WiiFitTrainer,
    RosalinaAndLuma,
    Greninja,
    Palutena,
    Robin,
    Shulk,
    BowserJr,
    DuckHunt,
    Ryu,
    Ken,
    Cloud,
    Corrin,
    Bayonetta,
    Inkling,
    Ridley,
    Simon,
    Richter,
    Incineroar,
    MiiBrawler,
    MiiSwordfighter,
    MiiGunner,
    IceClimbers,
    IceClimbersNana,
}
impl ResponseImplementation {
    pub fn for_kind(kind: i32) -> Option<Self> {
        match kind {
            0 => Some(Self::Mario),
            1 => Some(Self::DonkeyKong),
            2 => Some(Self::Link),
            3 => Some(Self::Samus),
            5 => Some(Self::Yoshi),
            7 => Some(Self::Fox),
            8 => Some(Self::Pikachu),
            6 => Some(Self::Kirby),
            9 => Some(Self::Luigi),
            11 => Some(Self::CaptainFalcon),
            crate::fighter_profiles::sonic::FIGHTER_KIND => Some(Self::Sonic),
            crate::fighter_profiles::krool::FIGHTER_KIND => Some(Self::KingKRool),
            crate::fighter_profiles::isabelle::FIGHTER_KIND => Some(Self::Isabelle),
            crate::fighter_profiles::pacman::FIGHTER_KIND => Some(Self::PacMan),
            crate::fighter_profiles::littlemac::FIGHTER_KIND => Some(Self::LittleMac),
            4 => Some(Self::DarkSamus),
            10 => Some(Self::Ness),
            12 => Some(Self::Jigglypuff),
            13 => Some(Self::Peach),
            14 => Some(Self::Daisy),
            15 => Some(Self::Bowser),
            16 => Some(Self::Sheik),
            17 => Some(Self::Zelda),
            18 => Some(Self::DrMario),
            19 => Some(Self::Pichu),
            20 => Some(Self::Falco),
            21 => Some(Self::Marth),
            22 => Some(Self::Lucina),
            23 => Some(Self::YoungLink),
            24 => Some(Self::Ganondorf),
            25 => Some(Self::Mewtwo),
            26 => Some(Self::Roy),
            27 => Some(Self::Chrom),
            28 => Some(Self::MrGameAndWatch),
            29 => Some(Self::MetaKnight),
            30 => Some(Self::Pit),
            31 => Some(Self::DarkPit),
            32 => Some(Self::ZeroSuitSamus),
            33 => Some(Self::Wario),
            34 => Some(Self::Snake),
            35 => Some(Self::Ike),
            36 => Some(Self::Squirtle),
            37 => Some(Self::Ivysaur),
            38 => Some(Self::Charizard),
            39 => Some(Self::DiddyKong),
            40 => Some(Self::Lucas),
            42 => Some(Self::KingDedede),
            43 => Some(Self::Olimar),
            44 => Some(Self::Lucario),
            45 => Some(Self::Rob),
            46 => Some(Self::ToonLink),
            47 => Some(Self::Wolf),
            48 => Some(Self::Villager),
            49 => Some(Self::MegaMan),
            50 => Some(Self::WiiFitTrainer),
            51 => Some(Self::RosalinaAndLuma),
            53 => Some(Self::Greninja),
            54 => Some(Self::Palutena),
            56 => Some(Self::Robin),
            57 => Some(Self::Shulk),
            58 => Some(Self::BowserJr),
            59 => Some(Self::DuckHunt),
            60 => Some(Self::Ryu),
            61 => Some(Self::Ken),
            62 => Some(Self::Cloud),
            63 => Some(Self::Corrin),
            64 => Some(Self::Bayonetta),
            65 => Some(Self::Inkling),
            66 => Some(Self::Ridley),
            67 => Some(Self::Simon),
            68 => Some(Self::Richter),
            71 => Some(Self::Incineroar),
            72 => Some(Self::MiiBrawler),
            73 => Some(Self::MiiSwordfighter),
            74 => Some(Self::MiiGunner),
            75 => Some(Self::IceClimbers),
            76 => Some(Self::IceClimbersNana),
            _ => None,
        }
    }
    pub(super) fn respond(
        self,
        trap: &mut LedgeTrap,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        match self {
            Self::DarkSamus => common::dispatch(&dark_samus::MENU, trap, settings, ctx, enemy, out),
            Self::Ness => common::dispatch(&ness::MENU, trap, settings, ctx, enemy, out),
            Self::Jigglypuff => {
                common::dispatch(&jigglypuff::MENU, trap, settings, ctx, enemy, out)
            }
            Self::Peach => common::dispatch(&peach::MENU, trap, settings, ctx, enemy, out),
            Self::Daisy => common::dispatch(&daisy::MENU, trap, settings, ctx, enemy, out),
            Self::Bowser => common::dispatch(&bowser::MENU, trap, settings, ctx, enemy, out),
            Self::Sheik => common::dispatch(&sheik::MENU, trap, settings, ctx, enemy, out),
            Self::Zelda => common::dispatch(&zelda::MENU, trap, settings, ctx, enemy, out),
            Self::DrMario => common::dispatch(&dr_mario::MENU, trap, settings, ctx, enemy, out),
            Self::Pichu => common::dispatch(&pichu::MENU, trap, settings, ctx, enemy, out),
            Self::Falco => common::dispatch(&falco::MENU, trap, settings, ctx, enemy, out),
            Self::Marth => common::dispatch(&marth::MENU, trap, settings, ctx, enemy, out),
            Self::Lucina => common::dispatch(&lucina::MENU, trap, settings, ctx, enemy, out),
            Self::YoungLink => common::dispatch(&young_link::MENU, trap, settings, ctx, enemy, out),
            Self::Ganondorf => common::dispatch(&ganondorf::MENU, trap, settings, ctx, enemy, out),
            Self::Mewtwo => common::dispatch(&mewtwo::MENU, trap, settings, ctx, enemy, out),
            Self::Roy => common::dispatch(&roy::MENU, trap, settings, ctx, enemy, out),
            Self::Chrom => common::dispatch(&chrom::MENU, trap, settings, ctx, enemy, out),
            Self::MrGameAndWatch => {
                common::dispatch(&mr_game_and_watch::MENU, trap, settings, ctx, enemy, out)
            }
            Self::MetaKnight => {
                common::dispatch(&meta_knight::MENU, trap, settings, ctx, enemy, out)
            }
            Self::Pit => common::dispatch(&pit::MENU, trap, settings, ctx, enemy, out),
            Self::DarkPit => common::dispatch(&dark_pit::MENU, trap, settings, ctx, enemy, out),
            Self::ZeroSuitSamus => {
                common::dispatch(&zero_suit_samus::MENU, trap, settings, ctx, enemy, out)
            }
            Self::Wario => common::dispatch(&wario::MENU, trap, settings, ctx, enemy, out),
            Self::Snake => common::dispatch(&snake::MENU, trap, settings, ctx, enemy, out),
            Self::Ike => common::dispatch(&ike::MENU, trap, settings, ctx, enemy, out),
            Self::Squirtle => common::dispatch(&squirtle::MENU, trap, settings, ctx, enemy, out),
            Self::Ivysaur => common::dispatch(&ivysaur::MENU, trap, settings, ctx, enemy, out),
            Self::Charizard => common::dispatch(&charizard::MENU, trap, settings, ctx, enemy, out),
            Self::DiddyKong => common::dispatch(&diddy_kong::MENU, trap, settings, ctx, enemy, out),
            Self::Lucas => common::dispatch(&lucas::MENU, trap, settings, ctx, enemy, out),
            Self::KingDedede => {
                common::dispatch(&king_dedede::MENU, trap, settings, ctx, enemy, out)
            }
            Self::Olimar => common::dispatch(&olimar::MENU, trap, settings, ctx, enemy, out),
            Self::Lucario => common::dispatch(&lucario::MENU, trap, settings, ctx, enemy, out),
            Self::Rob => common::dispatch(&rob::MENU, trap, settings, ctx, enemy, out),
            Self::ToonLink => common::dispatch(&toon_link::MENU, trap, settings, ctx, enemy, out),
            Self::Wolf => common::dispatch(&wolf::MENU, trap, settings, ctx, enemy, out),
            Self::Villager => common::dispatch(&villager::MENU, trap, settings, ctx, enemy, out),
            Self::MegaMan => common::dispatch(&mega_man::MENU, trap, settings, ctx, enemy, out),
            Self::WiiFitTrainer => {
                common::dispatch(&wii_fit_trainer::MENU, trap, settings, ctx, enemy, out)
            }
            Self::RosalinaAndLuma => {
                common::dispatch(&rosalina_and_luma::MENU, trap, settings, ctx, enemy, out)
            }
            Self::Greninja => common::dispatch(&greninja::MENU, trap, settings, ctx, enemy, out),
            Self::Palutena => common::dispatch(&palutena::MENU, trap, settings, ctx, enemy, out),
            Self::Robin => common::dispatch(&robin::MENU, trap, settings, ctx, enemy, out),
            Self::Shulk => common::dispatch(&shulk::MENU, trap, settings, ctx, enemy, out),
            Self::BowserJr => common::dispatch(&bowser_jr::MENU, trap, settings, ctx, enemy, out),
            Self::DuckHunt => common::dispatch(&duck_hunt::MENU, trap, settings, ctx, enemy, out),
            Self::Ryu => common::dispatch(&ryu::MENU, trap, settings, ctx, enemy, out),
            Self::Ken => common::dispatch(&ken::MENU, trap, settings, ctx, enemy, out),
            Self::Cloud => common::dispatch(&cloud::MENU, trap, settings, ctx, enemy, out),
            Self::Corrin => common::dispatch(&corrin::MENU, trap, settings, ctx, enemy, out),
            Self::Bayonetta => common::dispatch(&bayonetta::MENU, trap, settings, ctx, enemy, out),
            Self::Inkling => common::dispatch(&inkling::MENU, trap, settings, ctx, enemy, out),
            Self::Ridley => common::dispatch(&ridley::MENU, trap, settings, ctx, enemy, out),
            Self::Simon => common::dispatch(&simon::MENU, trap, settings, ctx, enemy, out),
            Self::Richter => common::dispatch(&richter::MENU, trap, settings, ctx, enemy, out),
            Self::Incineroar => {
                common::dispatch(&incineroar::MENU, trap, settings, ctx, enemy, out)
            }
            Self::MiiBrawler => {
                common::dispatch(&mii_brawler::MENU, trap, settings, ctx, enemy, out)
            }
            Self::MiiSwordfighter => {
                common::dispatch(&mii_swordfighter::MENU, trap, settings, ctx, enemy, out)
            }
            Self::MiiGunner => common::dispatch(&mii_gunner::MENU, trap, settings, ctx, enemy, out),
            Self::IceClimbers => {
                common::dispatch(&ice_climbers::MENU, trap, settings, ctx, enemy, out)
            }
            Self::IceClimbersNana => {
                common::dispatch(&ice_climbers_nana::MENU, trap, settings, ctx, enemy, out)
            }
            Self::Mario => common::dispatch(&mario::MENU, trap, settings, ctx, enemy, out),
            Self::DonkeyKong => common::dispatch(&donkey::MENU, trap, settings, ctx, enemy, out),
            Self::Link => common::dispatch(&link::MENU, trap, settings, ctx, enemy, out),
            Self::Samus => common::dispatch(&samus::MENU, trap, settings, ctx, enemy, out),
            Self::Yoshi => common::dispatch(&yoshi::MENU, trap, settings, ctx, enemy, out),
            Self::Fox => common::dispatch(&fox::MENU, trap, settings, ctx, enemy, out),
            Self::Pikachu => common::dispatch(&pikachu::MENU, trap, settings, ctx, enemy, out),
            Self::KingKRool => common::dispatch(&krool::MENU, trap, settings, ctx, enemy, out),
            Self::PacMan => common::dispatch(&pacman::MENU, trap, settings, ctx, enemy, out),
            Self::LittleMac => common::dispatch(&littlemac::MENU, trap, settings, ctx, enemy, out),
            Self::Isabelle => common::dispatch(&isabelle::MENU, trap, settings, ctx, enemy, out),
            Self::Sonic => common::dispatch(&sonic::MENU, trap, settings, ctx, enemy, out),
            Self::Kirby => kirby::respond(trap, settings, ctx, enemy, out),
            Self::Luigi => common::dispatch(&luigi::MENU, trap, settings, ctx, enemy, out),
            Self::CaptainFalcon => {
                common::dispatch(&captain::MENU, trap, settings, ctx, enemy, out)
            }
        }
    }
}
