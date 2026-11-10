//! Verified kind-specific native status tables from skyline-smash d2ca90d:
//! Final Smash phases, SPECIAL_HI continuations, inactive swap-form standby and
//! forced-state statuses. Private status numbers overlap across fighters, and a
//! large custom number or previous Up-B alone proves nothing: look a status up
//! only for the native kind that owns it.
use super::fighter_status as fs;
use crate::fighter_kinds as kind;

/// Inactive swap forms and Final Smash phases; do not apply these numbers to
/// another fighter where they may mean ordinary attacks or recovery.
///
///  Inactive swap forms and Final Smash phases; do not apply these numbers to another fighter where they may mean ordinary attacks or recovery.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub(super) fn noncombat_status(fighter: i32, status: i32) -> bool {
    final_statuses(fighter).contains(&status) || standby_statuses(fighter).contains(&status)
}

/// Kind-specific stun/exit states that end an input sequence.
///
///  Kind-specific stun/exit states that end an input sequence.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub(super) fn forced_status(fighter: i32, status: i32) -> bool {
    forced_statuses(fighter).contains(&status)
}

/// Ordinary SPECIAL_HI plus the verified kind-specific continuations.
///
///  Ordinary SPECIALHI plus the verified kind-specific continuations.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn up_special_status(fighter: i32, status: i32) -> bool {
    status == super::status::SPECIAL_HI || up_special_statuses(fighter).contains(&status)
}

/// Returns the `standby_statuses` result.
///
/// Gets the standby statuses result.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `&'static [i32]` value.
fn standby_statuses(fighter: i32) -> &'static [i32] {
    match fighter {
        kind::SQUIRTLE => &[fs::PZENIGAME_SPECIAL_LW_STANDBY],
        kind::IVYSAUR => &[fs::PFUSHIGISOU_SPECIAL_LW_STANDBY],
        kind::CHARIZARD => &[fs::PLIZARDON_SPECIAL_LW_STANDBY],
        kind::PYRA => &[fs::EFLAME_SPECIAL_LW_STANDBY],
        kind::MYTHRA => &[fs::ELIGHT_SPECIAL_LW_STANDBY],
        _ => &[],
    }
}

/// Returns the `forced_statuses` result.
///
/// Gets the forced statuses result.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `&'static [i32]` value.
fn forced_statuses(fighter: i32) -> &'static [i32] {
    match fighter {
        kind::DIDDY_KONG => &[
            fs::DIDDY_SPECIAL_S_DAMAGE_FALL,
            fs::DIDDY_SPECIAL_HI_CHARGE_DAMAGE,
            fs::DIDDY_SPECIAL_HI_UPPER_DAMAGE,
        ],
        kind::GRENINJA => &[fs::GEKKOUGA_SPECIAL_HI_WALL_DAMAGE],
        kind::BOWSER_JR => &[fs::KOOPAJR_SPECIAL_HI_DAMAGE_END],
        kind::SQUIRTLE => &[fs::PZENIGAME_SPECIAL_LW_OUT],
        kind::IVYSAUR => &[fs::PFUSHIGISOU_SPECIAL_LW_OUT],
        kind::CHARIZARD => &[fs::PLIZARDON_SPECIAL_LW_OUT],
        kind::PYRA => &[fs::EFLAME_SPECIAL_LW_OUT],
        kind::MYTHRA => &[fs::ELIGHT_SPECIAL_LW_OUT],
        _ => &[],
    }
}

/// Returns the `final_statuses` result.
///
/// Gets the final statuses result.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `&'static [i32]` value.
fn final_statuses(fighter: i32) -> &'static [i32] {
    match fighter {
        kind::DONKEY_KONG => &[fs::DONKEY_FINAL_ATTACK, fs::DONKEY_FINAL_END],
        kind::LINK => &[fs::LINK_FINAL_DASH, fs::LINK_FINAL_COMBO],
        kind::SAMUS => &[
            fs::SAMUS_FINAL_SZEROSUIT,
            fs::SAMUS_FINAL_START_G,
            fs::SAMUS_FINAL_START_A,
            fs::SAMUS_FINAL_LOOP_G,
            fs::SAMUS_FINAL_LOOP_A,
            fs::SAMUS_FINAL_END_G,
            fs::SAMUS_FINAL_END_A,
        ],
        kind::YOSHI => &[
            fs::YOSHI_FINAL_DASH,
            fs::YOSHI_FINAL_DASH_END,
            fs::YOSHI_FINAL_HIT,
            fs::YOSHI_FINAL_SCENE01,
            fs::YOSHI_FINAL_END,
        ],
        kind::KIRBY => &[
            fs::KIRBY_FINAL_CALLING,
            fs::KIRBY_FINAL_UP_NABE,
            fs::KIRBY_FINAL_ATTACK,
            fs::KIRBY_FINAL_DOWN_NABE,
            fs::KIRBY_FINAL_END,
        ],
        kind::FOX => &[
            fs::FOX_FINAL_READY,
            fs::FOX_FINAL_SCENE01,
            fs::FOX_FINAL_END,
        ],
        kind::PIKACHU => &[
            fs::PIKACHU_FINAL_DASH,
            fs::PIKACHU_FINAL_DASH_END,
            fs::PIKACHU_FINAL_HIT,
            fs::PIKACHU_FINAL_ATTACK,
            fs::PIKACHU_FINAL_ATTACK_2,
            fs::PIKACHU_FINAL_ATTACK_FINISH,
            fs::PIKACHU_FINAL_RETURN,
            fs::PIKACHU_FINAL_END,
        ],
        kind::LUIGI => &[
            fs::LUIGI_FINAL_VACUUM,
            fs::LUIGI_FINAL_SHOOT_READY,
            fs::LUIGI_FINAL_SHOOT,
            fs::LUIGI_FINAL_END,
            fs::LUIGI_FINAL_FAIL,
        ],
        kind::NESS => &[fs::NESS_FINAL_ATTACK],
        kind::CAPTAIN_FALCON => &[
            fs::CAPTAIN_FINAL_RIDE,
            fs::CAPTAIN_FINAL_SCENE01,
            fs::CAPTAIN_FINAL_SCENE02,
            fs::CAPTAIN_FINAL_SCENE03,
            fs::CAPTAIN_FINAL_END,
        ],
        kind::JIGGLYPUFF => &[fs::PURIN_FINAL_WAIT, fs::PURIN_FINAL_END],
        kind::BOWSER => &[fs::KOOPA_FINAL_END],
        kind::SHEIK => &[
            fs::SHEIK_FINAL_START,
            fs::SHEIK_FINAL_DASH,
            fs::SHEIK_FINAL_DASH_END,
            fs::SHEIK_FINAL_HIT,
            fs::SHEIK_FINAL_ATTACK,
            fs::SHEIK_FINAL_FINISH,
            fs::SHEIK_FINAL_END,
        ],
        kind::ZELDA => &[
            fs::ZELDA_FINAL_START,
            fs::ZELDA_FINAL_LOOP,
            fs::ZELDA_FINAL_FINISH,
            fs::ZELDA_FINAL_END,
        ],
        kind::FALCO => &[
            fs::FALCO_FINAL_READY,
            fs::FALCO_FINAL_SCENE01,
            fs::FALCO_FINAL_END,
        ],
        kind::MARTH => &[
            fs::MARTH_FINAL_DASH,
            fs::MARTH_FINAL_DASH_END,
            fs::MARTH_FINAL_END,
        ],
        kind::GANONDORF => &[fs::GANON_FINAL_ATTACK, fs::GANON_FINAL_END],
        kind::MEWTWO => &[fs::MEWTWO_FINAL_END],
        kind::ROY | kind::CHROM => &[
            fs::ROY_FINAL_END,
            fs::ROY_FINAL_DASH,
            fs::ROY_FINAL_DASH_END,
            fs::ROY_FINAL_ATTACK,
        ],
        kind::MR_GAME_AND_WATCH => &[
            fs::GAMEWATCH_FINAL_WAIT,
            fs::GAMEWATCH_FINAL_WALK,
            fs::GAMEWATCH_FINAL_DASH,
            fs::GAMEWATCH_FINAL_RUN,
            fs::GAMEWATCH_FINAL_RUN_BRAKE,
            fs::GAMEWATCH_FINAL_TURN,
            fs::GAMEWATCH_FINAL_TURN_DASH,
            fs::GAMEWATCH_FINAL_TURN_RUN,
            fs::GAMEWATCH_FINAL_JUMP_SQUAT,
            fs::GAMEWATCH_FINAL_JUMP,
            fs::GAMEWATCH_FINAL_JUMP_AERIAL,
            fs::GAMEWATCH_FINAL_FALL,
            fs::GAMEWATCH_FINAL_FALL_AERIAL,
            fs::GAMEWATCH_FINAL_PASS,
            fs::GAMEWATCH_FINAL_LANDING,
            fs::GAMEWATCH_FINAL_LANDING_LIGHT,
            fs::GAMEWATCH_FINAL_SWIM_DIVE,
            fs::GAMEWATCH_FINAL_SWIM_RISE,
            fs::GAMEWATCH_FINAL_SWIM_WAIT,
            fs::GAMEWATCH_FINAL_SWIM_WALK,
            fs::GAMEWATCH_FINAL_SWIM_DASH,
            fs::GAMEWATCH_FINAL_SWIM_RUN,
            fs::GAMEWATCH_FINAL_SWIM_RUN_BRAKE,
            fs::GAMEWATCH_FINAL_SWIM_TURN,
            fs::GAMEWATCH_FINAL_SWIM_TURN_DASH,
            fs::GAMEWATCH_FINAL_SWIM_TURN_RUN,
            fs::GAMEWATCH_FINAL_SWIM_JUMP_SQUAT,
            fs::GAMEWATCH_FINAL_END,
        ],
        kind::META_KNIGHT => &[
            fs::METAKNIGHT_FINAL_HIT_WAIT,
            fs::METAKNIGHT_FINAL_SQUAT,
            fs::METAKNIGHT_FINAL_JUMP,
            fs::METAKNIGHT_FINAL_ATTACK,
            fs::METAKNIGHT_FINAL_END,
        ],
        kind::PIT | kind::DARK_PIT => {
            &[fs::PIT_FINAL_WAIT, fs::PIT_FINAL_ATTACK, fs::PIT_FINAL_END]
        }
        kind::ZERO_SUIT_SAMUS => &[fs::SZEROSUIT_FINAL_LOCK_ON, fs::SZEROSUIT_FINAL_END],
        kind::WARIO => &[
            fs::WARIO_FINAL_DASH,
            fs::WARIO_FINAL_DASH_END,
            fs::WARIO_FINAL_READY,
            fs::WARIO_FINAL_SCENE,
            fs::WARIO_FINAL_END,
        ],
        kind::SNAKE => &[
            fs::SNAKE_FINAL_LOCK_ON,
            fs::SNAKE_FINAL_CAMERA_ZOOM,
            fs::SNAKE_FINAL_END,
            fs::SNAKE_FINAL_APPEAL,
        ],
        kind::IKE => &[
            fs::IKE_FINAL_HIT,
            fs::IKE_FINAL_MOVE,
            fs::IKE_FINAL_ATTACK,
            fs::IKE_FINAL_FALL,
            fs::IKE_FINAL_END,
        ],
        kind::DIDDY_KONG => &[
            fs::DIDDY_FINAL_START,
            fs::DIDDY_FINAL_WAIT_FLY,
            fs::DIDDY_FINAL_FLY,
            fs::DIDDY_FINAL_READY_CHARGE,
            fs::DIDDY_FINAL_CHARGE,
            fs::DIDDY_FINAL_FINISH_ATTACK,
            fs::DIDDY_FINAL_READY_END,
            fs::DIDDY_FINAL_END,
        ],
        kind::LUCAS => &[fs::LUCAS_FINAL_ATTACK],
        kind::SONIC => &[fs::SONIC_FINAL_END],
        kind::KING_DEDEDE => &[
            fs::DEDEDE_FINAL_READY,
            fs::DEDEDE_FINAL_SCENE01,
            fs::DEDEDE_FINAL_SCENE02,
            fs::DEDEDE_FINAL_SCENE03,
            fs::DEDEDE_FINAL_END,
        ],
        kind::OLIMAR => &[
            fs::PIKMIN_FINAL_FLY,
            fs::PIKMIN_FINAL_WAIT,
            fs::PIKMIN_FINAL_FALL,
            fs::PIKMIN_FINAL_END,
        ],
        kind::LUCARIO => &[
            fs::LUCARIO_FINAL_ENTRY,
            fs::LUCARIO_FINAL_HOLD,
            fs::LUCARIO_FINAL_ATTACK,
            fs::LUCARIO_FINAL_END,
        ],
        kind::ROB => &[fs::ROBOT_FINAL_LOOP, fs::ROBOT_FINAL_END],
        kind::WOLF => &[
            fs::WOLF_FINAL_JUMP,
            fs::WOLF_FINAL_CONTROL,
            fs::WOLF_FINAL_END,
        ],
        kind::VILLAGER => &[
            fs::MURABITO_FINAL_SURPRISE,
            fs::MURABITO_FINAL_MONEY,
            fs::MURABITO_FINAL_CHEER,
            fs::MURABITO_FINAL_HAPPY,
            fs::MURABITO_FINAL_END,
        ],
        kind::MEGA_MAN => &[
            fs::ROCKMAN_FINAL_INHALE,
            fs::ROCKMAN_FINAL_SCENE01,
            fs::ROCKMAN_FINAL_SCENE02,
            fs::ROCKMAN_FINAL_SCENE05,
            fs::ROCKMAN_FINAL_END,
        ],
        kind::WII_FIT_TRAINER => &[fs::WIIFIT_FINAL_HOLD, fs::WIIFIT_FINAL_END],
        kind::LITTLE_MAC => &[
            fs::LITTLEMAC_FINAL_DASH,
            fs::LITTLEMAC_FINAL_DASH_END,
            fs::LITTLEMAC_FINAL_HIT,
            fs::LITTLEMAC_FINAL_ATTACK,
            fs::LITTLEMAC_FINAL_END,
        ],
        kind::GRENINJA => &[fs::GEKKOUGA_FINAL_WAIT, fs::GEKKOUGA_FINAL_END],
        kind::PALUTENA => &[
            fs::PALUTENA_FINAL_WAIT,
            fs::PALUTENA_FINAL_BEAM_START,
            fs::PALUTENA_FINAL_BEAM,
            fs::PALUTENA_FINAL_BEAM_END,
        ],
        kind::PAC_MAN => &[fs::PACMAN_FINAL_MOVE, fs::PACMAN_FINAL_END],
        kind::ROBIN => &[
            fs::REFLET_FINAL_START,
            fs::REFLET_FINAL_FAIL,
            fs::REFLET_FINAL_READY,
            fs::REFLET_FINAL_END,
            fs::REFLET_FINAL_CHROM_ENTRY,
            fs::REFLET_FINAL_CHROM_DASH,
            fs::REFLET_FINAL_CHROM_LAUNCH,
            fs::REFLET_FINAL_CHROM_MISS,
            fs::REFLET_FINAL_CHROM_RUSH,
            fs::REFLET_FINAL_CHROM_READY,
            fs::REFLET_FINAL_CHROM_ATTACK,
            fs::REFLET_FINAL_CHROM_EXIT,
            fs::REFLET_FINAL_HIT,
            fs::REFLET_FINAL_MOVE,
            fs::REFLET_FINAL_ATTACK,
        ],
        kind::SHULK => &[
            fs::SHULK_FINAL_READY,
            fs::SHULK_FINAL_SCENE_ENTRY,
            fs::SHULK_FINAL_SCENE_ATTACK,
            fs::SHULK_FINAL_END,
        ],
        kind::BOWSER_JR => &[fs::KOOPAJR_FINAL_DRAW, fs::KOOPAJR_FINAL_END],
        kind::DUCK_HUNT => &[
            fs::DUCKHUNT_FINAL_READY,
            fs::DUCKHUNT_FINAL_SCENE01,
            fs::DUCKHUNT_FINAL_SCENE02,
            fs::DUCKHUNT_FINAL_SCENE03,
            fs::DUCKHUNT_FINAL_SCENE04,
            fs::DUCKHUNT_FINAL_SCENE05,
            fs::DUCKHUNT_FINAL_END,
        ],
        kind::RYU | kind::KEN => &[
            fs::RYU_FINAL_JUMP,
            fs::RYU_FINAL_FALL,
            fs::RYU_FINAL_LANDING,
            fs::RYU_FINAL_AIR_END,
            fs::RYU_FINAL_HIT,
            fs::RYU_FINAL2,
            fs::RYU_FINAL2_FALL,
            fs::RYU_FINAL2_LANDING,
            fs::RYU_FINAL2_AIR_END,
        ],
        kind::CLOUD => &[
            fs::CLOUD_FINAL_DASH,
            fs::CLOUD_FINAL_DASH_END,
            fs::CLOUD_FINAL_HIT,
            fs::CLOUD_FINAL_MOVE,
            fs::CLOUD_FINAL_ATTACK,
            fs::CLOUD_FINAL_FALL,
            fs::CLOUD_FINAL_END,
            fs::CLOUD_FINAL2,
            fs::CLOUD_FINAL2_DASH,
            fs::CLOUD_FINAL2_DASH_END,
            fs::CLOUD_FINAL2_HIT,
            fs::CLOUD_FINAL2_MOVE,
            fs::CLOUD_FINAL2_ATTACK,
            fs::CLOUD_FINAL2_FALL,
            fs::CLOUD_FINAL2_END,
        ],
        kind::CORRIN => &[
            fs::KAMUI_FINAL_READY,
            fs::KAMUI_FINAL_SCENE_ENTRY,
            fs::KAMUI_FINAL_SCENE_ATTACK,
            fs::KAMUI_FINAL_SCENE_ATTACK_WAIT,
            fs::KAMUI_FINAL_END,
        ],
        kind::BAYONETTA => &[
            fs::BAYONETTA_FINAL_START,
            fs::BAYONETTA_FINAL_SCENE01,
            fs::BAYONETTA_FINAL_END,
            fs::BAYONETTA_FINAL_END_AFTER,
        ],
        kind::RIDLEY => &[
            fs::RIDLEY_FINAL_DASH,
            fs::RIDLEY_FINAL_DASH_END,
            fs::RIDLEY_FINAL_READY,
            fs::RIDLEY_FINAL_SCENE01,
            fs::RIDLEY_FINAL_SCENE02,
            fs::RIDLEY_FINAL_SCENE03,
            fs::RIDLEY_FINAL_SCENE04,
            fs::RIDLEY_FINAL_SCENE05,
            fs::RIDLEY_FINAL_END,
        ],
        kind::SIMON => &[
            fs::SIMON_FINAL_READY,
            fs::SIMON_FINAL_SCENE01,
            fs::SIMON_FINAL_SCENE02,
            fs::SIMON_FINAL_SCENE03,
            fs::SIMON_FINAL_END,
        ],
        kind::KING_K_ROOL => &[
            fs::KROOL_FINAL_DASH,
            fs::KROOL_FINAL_DASH_END,
            fs::KROOL_FINAL_READY,
            fs::KROOL_FINAL_SCENE01,
            fs::KROOL_FINAL_SCENE02,
            fs::KROOL_FINAL_SCENE03,
            fs::KROOL_FINAL_SCENE04,
            fs::KROOL_FINAL_END,
        ],
        kind::INCINEROAR => &[
            fs::GAOGAEN_FINAL_DASH,
            fs::GAOGAEN_FINAL_DASH_END,
            fs::GAOGAEN_FINAL_READY,
            fs::GAOGAEN_FINAL_SCENE01,
            fs::GAOGAEN_FINAL_SCENE02,
            fs::GAOGAEN_FINAL_SCENE03,
            fs::GAOGAEN_FINAL_END,
        ],
        kind::MII_BRAWLER => &[
            fs::MIIFIGHTER_FINAL_HIT,
            fs::MIIFIGHTER_FINAL_MOVE,
            fs::MIIFIGHTER_FINAL_ATTACK,
            fs::MIIFIGHTER_FINAL_END,
        ],
        kind::MII_SWORDFIGHTER => &[fs::MIISWORDSMAN_FINAL_HOLD, fs::MIISWORDSMAN_FINAL_END],
        kind::MII_GUNNER => &[fs::MIIGUNNER_FINAL_LOOP, fs::MIIGUNNER_FINAL_END],
        kind::ICE_CLIMBERS_POPO | kind::ICE_CLIMBERS_NANA => {
            &[fs::POPO_FINAL_PARTNER, fs::POPO_FINAL_HANG]
        }
        kind::MII_ENEMY_S => &[fs::MIIENEMYS_FINAL_HOLD, fs::MIIENEMYS_FINAL_END],
        kind::JOKER => &[
            fs::JACK_FINAL_TURN_DASH,
            fs::JACK_FINAL_DASH_END,
            fs::JACK_FINAL_READY,
            fs::JACK_FINAL_SCENE01,
            fs::JACK_FINAL_SCENE02,
            fs::JACK_FINAL_END,
        ],
        kind::HERO => &[
            fs::BRAVE_FINAL_READY,
            fs::BRAVE_FINAL_SCENE01,
            fs::BRAVE_FINAL_END,
        ],
        kind::BANJO_AND_KAZOOIE => &[
            fs::BUDDY_FINAL_READY,
            fs::BUDDY_FINAL_SCENE01,
            fs::BUDDY_FINAL_SCENE02,
            fs::BUDDY_FINAL_SCENE03,
            fs::BUDDY_FINAL_SCENE04,
            fs::BUDDY_FINAL_END,
        ],
        kind::TERRY => &[
            fs::DOLLY_FINAL_READY,
            fs::DOLLY_FINAL_SCENE01,
            fs::DOLLY_FINAL_SCENE02,
            fs::DOLLY_FINAL_SCENE03,
            fs::DOLLY_FINAL_SCENE04,
            fs::DOLLY_FINAL_SCENE05,
            fs::DOLLY_FINAL_END,
        ],
        kind::BYLETH => &[
            fs::MASTER_FINAL_READY,
            fs::MASTER_FINAL_SCENE01,
            fs::MASTER_FINAL_SCENE02,
            fs::MASTER_FINAL_SCENE03,
            fs::MASTER_FINAL_END,
        ],
        kind::MIN_MIN => &[
            fs::TANTAN_FINAL_READY,
            fs::TANTAN_FINAL_SCENE01,
            fs::TANTAN_FINAL_SCENE02,
            fs::TANTAN_FINAL_SCENE03,
            fs::TANTAN_FINAL_END,
        ],
        kind::STEVE => &[
            fs::PICKEL_FINAL_READY,
            fs::PICKEL_FINAL_SCENE01,
            fs::PICKEL_FINAL_END,
        ],
        kind::SEPHIROTH => &[
            fs::EDGE_FINAL_READY,
            fs::EDGE_FINAL_SCENE01,
            fs::EDGE_FINAL_END,
        ],
        kind::PYRA => &[
            fs::EFLAME_FINAL_READY,
            fs::EFLAME_FINAL_SCENE01,
            fs::EFLAME_FINAL_SCENE02,
            fs::EFLAME_FINAL_END,
        ],
        kind::MYTHRA => &[
            fs::ELIGHT_FINAL_READY,
            fs::ELIGHT_FINAL_SCENE01,
            fs::ELIGHT_FINAL_SCENE02,
            fs::ELIGHT_FINAL_END,
            fs::ELIGHT_FINAL_SCENE03,
            fs::ELIGHT_FINAL_SCENE04,
        ],
        kind::KAZUYA => &[
            fs::DEMON_FINAL_READY,
            fs::DEMON_FINAL_SCENE01,
            fs::DEMON_FINAL_END,
        ],
        kind::SORA => &[
            fs::TRAIL_FINAL_READY,
            fs::TRAIL_FINAL_SCENE01,
            fs::TRAIL_FINAL_SCENE02,
            fs::TRAIL_FINAL_SCENE03,
            fs::TRAIL_FINAL_SCENE04,
            fs::TRAIL_FINAL_END,
        ],
        _ => &[],
    }
}

/// Returns the `up_special_statuses` result.
///
/// Gets the up special statuses result.
/// ### Arguments
/// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `&'static [i32]` value.
fn up_special_statuses(fighter: i32) -> &'static [i32] {
    match fighter {
        kind::LINK => &[fs::LINK_SPECIAL_HI_HOLD, fs::LINK_SPECIAL_HI_END],
        kind::KIRBY => &[
            fs::KIRBY_SPECIAL_HI2,
            fs::KIRBY_SPECIAL_HI3,
            fs::KIRBY_SPECIAL_HI4,
        ],
        kind::FOX => &[
            fs::FOX_SPECIAL_HI_RUSH,
            fs::FOX_SPECIAL_HI_RUSH_END,
            fs::FOX_SPECIAL_HI_BOUND,
        ],
        kind::PIKACHU => &[fs::PIKACHU_SPECIAL_HI_WARP, fs::PIKACHU_SPECIAL_HI_END],
        kind::LUIGI => &[
            fs::LUIGI_SPECIAL_HI_DROP,
            fs::LUIGI_SPECIAL_HI_FALL,
            fs::LUIGI_SPECIAL_HI_LANDING_FALL,
        ],
        kind::NESS => &[
            fs::NESS_SPECIAL_HI_HOLD,
            fs::NESS_SPECIAL_HI_END,
            fs::NESS_SPECIAL_HI_ATTACK,
            fs::NESS_SPECIAL_HI_REFLECT,
            fs::NESS_SPECIAL_HI_AGAIN,
        ],
        kind::CAPTAIN_FALCON => &[fs::CAPTAIN_SPECIAL_HI_CLING, fs::CAPTAIN_SPECIAL_HI_THROW],
        kind::PEACH | kind::DAISY => &[
            fs::PEACH_SPECIAL_HI_FALL,
            fs::PEACH_SPECIAL_HI_AIR_END,
            fs::PEACH_SPECIAL_HI_LANDING,
        ],
        kind::BOWSER => &[fs::KOOPA_SPECIAL_HI_G, fs::KOOPA_SPECIAL_HI_A],
        kind::SHEIK => &[fs::SHEIK_SPECIAL_HI_MOVE, fs::SHEIK_SPECIAL_HI_END],
        kind::ZELDA => &[fs::ZELDA_SPECIAL_HI_2, fs::ZELDA_SPECIAL_HI_3],
        kind::FALCO => &[
            fs::FALCO_SPECIAL_HI_RUSH,
            fs::FALCO_SPECIAL_HI_RUSH_END,
            fs::FALCO_SPECIAL_HI_BOUND,
        ],
        kind::GANONDORF => &[fs::GANON_SPECIAL_HI_CLING, fs::GANON_SPECIAL_HI_THROW],
        kind::MEWTWO => &[fs::MEWTWO_SPECIAL_HI_2, fs::MEWTWO_SPECIAL_HI_3],
        kind::ROY | kind::CHROM => &[
            fs::ROY_SPECIAL_HI_2,
            fs::ROY_SPECIAL_HI_3,
            fs::ROY_SPECIAL_HI_4,
        ],
        kind::MR_GAME_AND_WATCH => &[
            fs::GAMEWATCH_SPECIAL_HI_FALL,
            fs::GAMEWATCH_SPECIAL_HI_CLOSE,
        ],
        kind::META_KNIGHT => &[fs::METAKNIGHT_SPECIAL_HI_LOOP],
        kind::PIT | kind::DARK_PIT => &[fs::PIT_SPECIAL_HI_RUSH, fs::PIT_SPECIAL_HI_RUSH_END],
        kind::WARIO => &[fs::WARIO_SPECIAL_HI_JUMP],
        kind::SNAKE => &[fs::SNAKE_SPECIAL_HI_HANG, fs::SNAKE_SPECIAL_HI_CUT],
        kind::IKE => &[
            fs::IKE_SPECIAL_HI_2,
            fs::IKE_SPECIAL_HI_3,
            fs::IKE_SPECIAL_HI_4,
        ],
        kind::DIDDY_KONG => &[
            fs::DIDDY_SPECIAL_HI_CHARGE,
            fs::DIDDY_SPECIAL_HI_UPPER,
            fs::DIDDY_SPECIAL_HI_HIT_CEIL,
            fs::DIDDY_SPECIAL_HI_FALL_ROLL,
        ],
        kind::LUCAS => &[
            fs::LUCAS_SPECIAL_HI_HOLD,
            fs::LUCAS_SPECIAL_HI_END,
            fs::LUCAS_SPECIAL_HI_ATTACK,
            fs::LUCAS_SPECIAL_HI_REFLECT,
            fs::LUCAS_SPECIAL_HI_AGAIN,
        ],
        kind::SONIC => &[fs::SONIC_SPECIAL_HI_JUMP],
        kind::KING_DEDEDE => &[
            fs::DEDEDE_SPECIAL_HI_JUMP,
            fs::DEDEDE_SPECIAL_HI_LANDING,
            fs::DEDEDE_SPECIAL_HI_HIT,
            fs::DEDEDE_SPECIAL_HI_FAILURE,
            fs::DEDEDE_SPECIAL_HI_TURN,
        ],
        kind::OLIMAR => &[
            fs::PIKMIN_SPECIAL_HI_WAIT,
            fs::PIKMIN_SPECIAL_HI_LANDING,
            fs::PIKMIN_SPECIAL_HI_END,
        ],
        kind::LUCARIO => &[
            fs::LUCARIO_SPECIAL_HI_RUSH,
            fs::LUCARIO_SPECIAL_HI_RUSH_END,
            fs::LUCARIO_SPECIAL_HI_BOUND,
        ],
        kind::ROB => &[
            fs::ROBOT_SPECIAL_HI_KEEP,
            fs::ROBOT_SPECIAL_HI_ATTACK,
            fs::ROBOT_SPECIAL_HI_ITEM_THROW,
            fs::ROBOT_SPECIAL_HI_ITEM_SHOOT_AIR,
        ],
        kind::WOLF => &[
            fs::WOLF_SPECIAL_HI_RUSH,
            fs::WOLF_SPECIAL_HI_RUSH_END,
            fs::WOLF_SPECIAL_HI_BOUND,
        ],
        kind::VILLAGER => &[
            fs::MURABITO_SPECIAL_HI_WAIT,
            fs::MURABITO_SPECIAL_HI_FLAP,
            fs::MURABITO_SPECIAL_HI_TURN,
            fs::MURABITO_SPECIAL_HI_LANDING,
            fs::MURABITO_SPECIAL_HI_END,
            fs::MURABITO_SPECIAL_HI_DETACH,
            fs::MURABITO_SPECIAL_HI_BURST,
        ],
        kind::MEGA_MAN => &[fs::ROCKMAN_SPECIAL_HI_JUMP],
        kind::WII_FIT_TRAINER => &[fs::WIIFIT_SPECIAL_HI_JUMP, fs::WIIFIT_SPECIAL_HI_END],
        kind::ROSALINA_AND_LUMA => &[fs::ROSETTA_SPECIAL_HI_JUMP, fs::ROSETTA_SPECIAL_HI_END],
        kind::LITTLE_MAC => &[
            fs::LITTLEMAC_SPECIAL_HI_START,
            fs::LITTLEMAC_SPECIAL_HI_JUMP,
        ],
        kind::GRENINJA => &[fs::GEKKOUGA_SPECIAL_HI_LOOP, fs::GEKKOUGA_SPECIAL_HI_END],
        kind::PALUTENA => &[fs::PALUTENA_SPECIAL_HI_2, fs::PALUTENA_SPECIAL_HI_3],
        kind::PAC_MAN => &[fs::PACMAN_SPECIAL_HI_LOOP, fs::PACMAN_SPECIAL_HI_END],
        kind::ROBIN => &[fs::REFLET_SPECIAL_HI_2, fs::REFLET_SPECIAL_HI_FAIL],
        kind::SHULK => &[fs::SHULK_SPECIAL_HI_ADD],
        kind::BOWSER_JR => &[
            fs::KOOPAJR_SPECIAL_HI_SHOOT,
            fs::KOOPAJR_SPECIAL_HI_FALL,
            fs::KOOPAJR_SPECIAL_HI_ATTACK,
            fs::KOOPAJR_SPECIAL_HI_ESCAPE,
            fs::KOOPAJR_SPECIAL_HI_LANDING,
        ],
        kind::DUCK_HUNT => &[fs::DUCKHUNT_SPECIAL_HI_FLY, fs::DUCKHUNT_SPECIAL_HI_END],
        kind::RYU | kind::KEN => &[
            fs::RYU_SPECIAL_HI_JUMP,
            fs::RYU_SPECIAL_HI_FALL,
            fs::RYU_SPECIAL_HI_LANDING,
            fs::RYU_SPECIAL_HI_COMMAND,
        ],
        kind::CLOUD => &[
            fs::CLOUD_SPECIAL_HI2,
            fs::CLOUD_SPECIAL_HI2_FALL,
            fs::CLOUD_SPECIAL_HI2_LANDING,
        ],
        kind::BAYONETTA => &[fs::BAYONETTA_SPECIAL_HI_JUMP],
        kind::INKLING => &[
            fs::INKLING_SPECIAL_HI_JUMP,
            fs::INKLING_SPECIAL_HI_ROT,
            fs::INKLING_SPECIAL_HI_STOP_CEIL,
            fs::INKLING_SPECIAL_HI_FALL,
        ],
        kind::RIDLEY => &[
            fs::RIDLEY_SPECIAL_HI_HOVER,
            fs::RIDLEY_SPECIAL_HI_CHARGE_START,
            fs::RIDLEY_SPECIAL_HI_CHARGE_F,
            fs::RIDLEY_SPECIAL_HI_CHARGE_HI,
            fs::RIDLEY_SPECIAL_HI_STOP_CEIL,
            fs::RIDLEY_SPECIAL_HI_STOP_WALL,
            fs::RIDLEY_SPECIAL_HI_LANDING,
            fs::RIDLEY_SPECIAL_HI_END,
        ],
        kind::KING_K_ROOL => &[
            fs::KROOL_SPECIAL_HI_START,
            fs::KROOL_SPECIAL_HI,
            fs::KROOL_SPECIAL_HI_AIR_END,
            fs::KROOL_SPECIAL_HI_FALL,
            fs::KROOL_SPECIAL_HI_LANDING,
        ],
        kind::INCINEROAR => &[
            fs::GAOGAEN_SPECIAL_HI_TURN,
            fs::GAOGAEN_SPECIAL_HI_FALL,
            fs::GAOGAEN_SPECIAL_HI_LOOP,
            fs::GAOGAEN_SPECIAL_HI_END,
            fs::GAOGAEN_SPECIAL_HI_BOUND,
        ],
        kind::MII_BRAWLER => &[
            fs::MIIFIGHTER_SPECIAL_HI1_2,
            fs::MIIFIGHTER_SPECIAL_HI1_3,
            fs::MIIFIGHTER_SPECIAL_HI1_4,
            fs::MIIFIGHTER_SPECIAL_HI2_ATTACK,
            fs::MIIFIGHTER_SPECIAL_HI2_END,
        ],
        kind::MII_SWORDFIGHTER => &[
            fs::MIISWORDSMAN_SPECIAL_HI1_JUMP,
            fs::MIISWORDSMAN_SPECIAL_HI1_LOOP,
            fs::MIISWORDSMAN_SPECIAL_HI1_END,
            fs::MIISWORDSMAN_SPECIAL_HI2_RUSH,
            fs::MIISWORDSMAN_SPECIAL_HI2_RUSH_END,
            fs::MIISWORDSMAN_SPECIAL_HI2_BOUND,
            fs::MIISWORDSMAN_SPECIAL_HI3_HOLD,
            fs::MIISWORDSMAN_SPECIAL_HI3_END,
        ],
        kind::MII_GUNNER => &[
            fs::MIIGUNNER_SPECIAL_HI2_JUMP,
            fs::MIIGUNNER_SPECIAL_HI3_RUSH,
            fs::MIIGUNNER_SPECIAL_HI3_RUSH_END,
        ],
        kind::ICE_CLIMBERS_POPO | kind::ICE_CLIMBERS_NANA => &[
            fs::POPO_SPECIAL_HI_FAIL,
            fs::POPO_SPECIAL_HI_JUMP_PRE,
            fs::POPO_SPECIAL_HI_PARTNER,
            fs::POPO_SPECIAL_HI_JUMP_PARTNER,
            fs::POPO_SPECIAL_HI_JUMP,
            fs::POPO_SPECIAL_HI_CLIFF_COMP_PARTNER,
            fs::POPO_SPECIAL_HI_CLIFF_PULL_PARTNER,
            fs::POPO_SPECIAL_HI_CLIFF_COMP,
        ],
        kind::MII_ENEMY_F => &[
            fs::MIIENEMYF_SPECIAL_HI2_2,
            fs::MIIENEMYF_SPECIAL_HI2_3,
            fs::MIIENEMYF_SPECIAL_HI2_4,
            fs::MIIENEMYF_SPECIAL_HI3_ATTACK,
            fs::MIIENEMYF_SPECIAL_HI3_END,
        ],
        kind::MII_ENEMY_S => &[
            fs::MIIENEMYS_SPECIAL_HI1_HOLD,
            fs::MIIENEMYS_SPECIAL_HI1_END,
            fs::MIIENEMYS_SPECIAL_HI2_RUSH,
            fs::MIIENEMYS_SPECIAL_HI2_BOUND,
            fs::MIIENEMYS_SPECIAL_HI3_LOOP,
            fs::MIIENEMYS_SPECIAL_HI3_END,
        ],
        kind::MII_ENEMY_G => &[
            fs::MIIENEMYG_SPECIAL_HI1_JUMP,
            fs::MIIENEMYG_SPECIAL_HI2_FAIL,
            fs::MIIENEMYG_SPECIAL_HI3_RUSH,
            fs::MIIENEMYG_SPECIAL_HI3_RUSH_END,
        ],
        kind::PIRANHA_PLANT => &[fs::PACKUN_SPECIAL_HI_END, fs::PACKUN_SPECIAL_HI_LANDING],
        kind::JOKER => &[
            fs::JACK_SPECIAL_HI_PULL,
            fs::JACK_SPECIAL_HI_PICKUP,
            fs::JACK_SPECIAL_HI_THROW,
            fs::JACK_SPECIAL_HI_CUT,
            fs::JACK_SPECIAL_HI2_RUSH,
            fs::JACK_SPECIAL_HI2_END,
        ],
        kind::HERO => &[fs::BRAVE_SPECIAL_HI_HOLD, fs::BRAVE_SPECIAL_HI_JUMP],
        kind::BANJO_AND_KAZOOIE => &[fs::BUDDY_SPECIAL_HI_JUMP],
        kind::TERRY => &[
            fs::DOLLY_SPECIAL_HI_JUMP,
            fs::DOLLY_SPECIAL_HI_FALL,
            fs::DOLLY_SPECIAL_HI_LANDING,
            fs::DOLLY_SPECIAL_HI_COMMAND,
        ],
        kind::BYLETH => &[
            fs::MASTER_SPECIAL_HI_HIT,
            fs::MASTER_SPECIAL_HI_OVERTAKE,
            fs::MASTER_SPECIAL_HI_WALL_JUMP,
            fs::MASTER_SPECIAL_HI_FAIL,
        ],
        kind::MIN_MIN => &[
            fs::TANTAN_SPECIAL_HI_GROUND,
            fs::TANTAN_SPECIAL_HI_GROUND_JUMP,
            fs::TANTAN_SPECIAL_HI_GROUND_END,
            fs::TANTAN_SPECIAL_HI_AIR,
            fs::TANTAN_SPECIAL_HI_AIR_REACH,
            fs::TANTAN_SPECIAL_HI_AIR_END,
        ],
        kind::STEVE => &[
            fs::PICKEL_SPECIAL_HI_JUMP,
            fs::PICKEL_SPECIAL_HI_WAIT,
            fs::PICKEL_SPECIAL_HI_GLIDING,
            fs::PICKEL_SPECIAL_HI_LANDING,
            fs::PICKEL_SPECIAL_HI_FALL,
            fs::PICKEL_SPECIAL_HI_FALL_SPECIAL,
        ],
        kind::SEPHIROTH => &[
            fs::EDGE_SPECIAL_HI_RUSH,
            fs::EDGE_SPECIAL_HI_CHARGED_RUSH,
            fs::EDGE_SPECIAL_HI_LANDING,
            fs::EDGE_SPECIAL_HI_END,
        ],
        kind::PYRA => &[
            fs::EFLAME_SPECIAL_HI_JUMP,
            fs::EFLAME_SPECIAL_HI_LOOP,
            fs::EFLAME_SPECIAL_HI_END,
        ],
        kind::MYTHRA => &[
            fs::ELIGHT_SPECIAL_HI_JUMP,
            fs::ELIGHT_SPECIAL_HI_ATTACK1,
            fs::ELIGHT_SPECIAL_HI_ATTACK2,
            fs::ELIGHT_SPECIAL_HI_FINISH,
        ],
        kind::KAZUYA => &[fs::DEMON_SPECIAL_HI_RISE, fs::DEMON_SPECIAL_HI_FALL],
        _ => &[],
    }
}
