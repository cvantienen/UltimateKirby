//! Verified kind-specific SPECIAL_HI continuations from skyline-smash d2ca90d.
//! A large custom status number or previous Up-B alone proves nothing.
pub fn up_special_status(kind: i32, status: i32) -> bool {
    if status == super::status::SPECIAL_HI {
        return true;
    }
    match kind {
        crate::fighter_kinds::LINK => matches!(
            status,
            super::fighter_status::LINK_SPECIAL_HI_HOLD
                | super::fighter_status::LINK_SPECIAL_HI_END
        ), // LINK
        crate::fighter_kinds::KIRBY => matches!(
            status,
            super::fighter_status::KIRBY_SPECIAL_HI2
                | super::fighter_status::KIRBY_SPECIAL_HI3
                | super::fighter_status::KIRBY_SPECIAL_HI4
        ), // KIRBY
        crate::fighter_kinds::FOX => matches!(
            status,
            super::fighter_status::FOX_SPECIAL_HI_RUSH
                | super::fighter_status::FOX_SPECIAL_HI_RUSH_END
                | super::fighter_status::FOX_SPECIAL_HI_BOUND
        ), // FOX
        crate::fighter_kinds::PIKACHU => matches!(
            status,
            super::fighter_status::PIKACHU_SPECIAL_HI_WARP
                | super::fighter_status::PIKACHU_SPECIAL_HI_END
        ), // PIKACHU
        crate::fighter_kinds::LUIGI => matches!(
            status,
            super::fighter_status::LUIGI_SPECIAL_HI_DROP
                | super::fighter_status::LUIGI_SPECIAL_HI_FALL
                | super::fighter_status::LUIGI_SPECIAL_HI_LANDING_FALL
        ), // LUIGI
        crate::fighter_kinds::NESS => matches!(
            status,
            super::fighter_status::NESS_SPECIAL_HI_HOLD
                | super::fighter_status::NESS_SPECIAL_HI_END
                | super::fighter_status::NESS_SPECIAL_HI_ATTACK
                | super::fighter_status::NESS_SPECIAL_HI_REFLECT
                | super::fighter_status::NESS_SPECIAL_HI_AGAIN
        ), // NESS
        crate::fighter_kinds::CAPTAIN_FALCON => matches!(
            status,
            super::fighter_status::CAPTAIN_SPECIAL_HI_CLING
                | super::fighter_status::CAPTAIN_SPECIAL_HI_THROW
        ), // CAPTAIN
        crate::fighter_kinds::PEACH | crate::fighter_kinds::DAISY => matches!(
            status,
            super::fighter_status::PEACH_SPECIAL_HI_FALL
                | super::fighter_status::PEACH_SPECIAL_HI_AIR_END
                | super::fighter_status::PEACH_SPECIAL_HI_LANDING
        ), // PEACH / DAISY
        crate::fighter_kinds::BOWSER => matches!(
            status,
            super::fighter_status::KOOPA_SPECIAL_HI_G | super::fighter_status::KOOPA_SPECIAL_HI_A
        ), // KOOPA
        crate::fighter_kinds::SHEIK => matches!(
            status,
            super::fighter_status::SHEIK_SPECIAL_HI_MOVE
                | super::fighter_status::SHEIK_SPECIAL_HI_END
        ), // SHEIK
        crate::fighter_kinds::ZELDA => matches!(
            status,
            super::fighter_status::ZELDA_SPECIAL_HI_2 | super::fighter_status::ZELDA_SPECIAL_HI_3
        ), // ZELDA
        crate::fighter_kinds::FALCO => matches!(
            status,
            super::fighter_status::FALCO_SPECIAL_HI_RUSH
                | super::fighter_status::FALCO_SPECIAL_HI_RUSH_END
                | super::fighter_status::FALCO_SPECIAL_HI_BOUND
        ), // FALCO
        crate::fighter_kinds::GANONDORF => matches!(
            status,
            super::fighter_status::GANON_SPECIAL_HI_CLING
                | super::fighter_status::GANON_SPECIAL_HI_THROW
        ), // GANON
        crate::fighter_kinds::MEWTWO => matches!(
            status,
            super::fighter_status::MEWTWO_SPECIAL_HI_2 | super::fighter_status::MEWTWO_SPECIAL_HI_3
        ), // MEWTWO
        crate::fighter_kinds::ROY | crate::fighter_kinds::CHROM => matches!(
            status,
            super::fighter_status::ROY_SPECIAL_HI_2
                | super::fighter_status::ROY_SPECIAL_HI_3
                | super::fighter_status::ROY_SPECIAL_HI_4
        ), // ROY / CHROM
        crate::fighter_kinds::MR_GAME_AND_WATCH => matches!(
            status,
            super::fighter_status::GAMEWATCH_SPECIAL_HI_FALL
                | super::fighter_status::GAMEWATCH_SPECIAL_HI_CLOSE
        ), // GAMEWATCH
        crate::fighter_kinds::META_KNIGHT => {
            matches!(status, super::fighter_status::METAKNIGHT_SPECIAL_HI_LOOP)
        } // METAKNIGHT
        crate::fighter_kinds::PIT | crate::fighter_kinds::DARK_PIT => matches!(
            status,
            super::fighter_status::PIT_SPECIAL_HI_RUSH
                | super::fighter_status::PIT_SPECIAL_HI_RUSH_END
        ), // PIT / DARK PIT
        crate::fighter_kinds::WARIO => {
            matches!(status, super::fighter_status::WARIO_SPECIAL_HI_JUMP)
        } // WARIO
        crate::fighter_kinds::SNAKE => matches!(
            status,
            super::fighter_status::SNAKE_SPECIAL_HI_HANG
                | super::fighter_status::SNAKE_SPECIAL_HI_CUT
        ), // SNAKE
        crate::fighter_kinds::IKE => matches!(
            status,
            super::fighter_status::IKE_SPECIAL_HI_2
                | super::fighter_status::IKE_SPECIAL_HI_3
                | super::fighter_status::IKE_SPECIAL_HI_4
        ), // IKE
        crate::fighter_kinds::DIDDY_KONG => matches!(
            status,
            super::fighter_status::DIDDY_SPECIAL_HI_CHARGE
                | super::fighter_status::DIDDY_SPECIAL_HI_UPPER
                | super::fighter_status::DIDDY_SPECIAL_HI_HIT_CEIL
                | super::fighter_status::DIDDY_SPECIAL_HI_FALL_ROLL
        ), // DIDDY
        crate::fighter_kinds::LUCAS => matches!(
            status,
            super::fighter_status::LUCAS_SPECIAL_HI_HOLD
                | super::fighter_status::LUCAS_SPECIAL_HI_END
                | super::fighter_status::LUCAS_SPECIAL_HI_ATTACK
                | super::fighter_status::LUCAS_SPECIAL_HI_REFLECT
                | super::fighter_status::LUCAS_SPECIAL_HI_AGAIN
        ), // LUCAS
        crate::fighter_kinds::SONIC => {
            matches!(status, super::fighter_status::SONIC_SPECIAL_HI_JUMP)
        } // SONIC
        crate::fighter_kinds::KING_DEDEDE => matches!(
            status,
            super::fighter_status::DEDEDE_SPECIAL_HI_JUMP
                | super::fighter_status::DEDEDE_SPECIAL_HI_LANDING
                | super::fighter_status::DEDEDE_SPECIAL_HI_HIT
                | super::fighter_status::DEDEDE_SPECIAL_HI_FAILURE
                | super::fighter_status::DEDEDE_SPECIAL_HI_TURN
        ), // DEDEDE
        crate::fighter_kinds::OLIMAR => matches!(
            status,
            super::fighter_status::PIKMIN_SPECIAL_HI_WAIT
                | super::fighter_status::PIKMIN_SPECIAL_HI_LANDING
                | super::fighter_status::PIKMIN_SPECIAL_HI_END
        ), // PIKMIN
        crate::fighter_kinds::LUCARIO => matches!(
            status,
            super::fighter_status::LUCARIO_SPECIAL_HI_RUSH
                | super::fighter_status::LUCARIO_SPECIAL_HI_RUSH_END
                | super::fighter_status::LUCARIO_SPECIAL_HI_BOUND
        ), // LUCARIO
        crate::fighter_kinds::ROB => matches!(
            status,
            super::fighter_status::ROBOT_SPECIAL_HI_KEEP
                | super::fighter_status::ROBOT_SPECIAL_HI_ATTACK
                | super::fighter_status::ROBOT_SPECIAL_HI_ITEM_THROW
                | super::fighter_status::ROBOT_SPECIAL_HI_ITEM_SHOOT_AIR
        ), // ROBOT
        crate::fighter_kinds::WOLF => matches!(
            status,
            super::fighter_status::WOLF_SPECIAL_HI_RUSH
                | super::fighter_status::WOLF_SPECIAL_HI_RUSH_END
                | super::fighter_status::WOLF_SPECIAL_HI_BOUND
        ), // WOLF
        crate::fighter_kinds::VILLAGER => matches!(
            status,
            super::fighter_status::MURABITO_SPECIAL_HI_WAIT
                | super::fighter_status::MURABITO_SPECIAL_HI_FLAP
                | super::fighter_status::MURABITO_SPECIAL_HI_TURN
                | super::fighter_status::MURABITO_SPECIAL_HI_LANDING
                | super::fighter_status::MURABITO_SPECIAL_HI_END
                | super::fighter_status::MURABITO_SPECIAL_HI_DETACH
                | super::fighter_status::MURABITO_SPECIAL_HI_BURST
        ), // MURABITO
        crate::fighter_kinds::MEGA_MAN => {
            matches!(status, super::fighter_status::ROCKMAN_SPECIAL_HI_JUMP)
        } // ROCKMAN
        crate::fighter_kinds::WII_FIT_TRAINER => matches!(
            status,
            super::fighter_status::WIIFIT_SPECIAL_HI_JUMP
                | super::fighter_status::WIIFIT_SPECIAL_HI_END
        ), // WIIFIT
        crate::fighter_kinds::ROSALINA_AND_LUMA => matches!(
            status,
            super::fighter_status::ROSETTA_SPECIAL_HI_JUMP
                | super::fighter_status::ROSETTA_SPECIAL_HI_END
        ), // ROSETTA
        crate::fighter_kinds::LITTLE_MAC => matches!(
            status,
            super::fighter_status::LITTLEMAC_SPECIAL_HI_START
                | super::fighter_status::LITTLEMAC_SPECIAL_HI_JUMP
        ), // LITTLEMAC
        crate::fighter_kinds::GRENINJA => matches!(
            status,
            super::fighter_status::GEKKOUGA_SPECIAL_HI_LOOP
                | super::fighter_status::GEKKOUGA_SPECIAL_HI_END
        ), // GEKKOUGA
        crate::fighter_kinds::PALUTENA => matches!(
            status,
            super::fighter_status::PALUTENA_SPECIAL_HI_2
                | super::fighter_status::PALUTENA_SPECIAL_HI_3
        ), // PALUTENA
        crate::fighter_kinds::PAC_MAN => matches!(
            status,
            super::fighter_status::PACMAN_SPECIAL_HI_LOOP
                | super::fighter_status::PACMAN_SPECIAL_HI_END
        ), // PACMAN
        crate::fighter_kinds::ROBIN => matches!(
            status,
            super::fighter_status::REFLET_SPECIAL_HI_2
                | super::fighter_status::REFLET_SPECIAL_HI_FAIL
        ), // REFLET
        crate::fighter_kinds::SHULK => {
            matches!(status, super::fighter_status::SHULK_SPECIAL_HI_ADD)
        } // SHULK
        crate::fighter_kinds::BOWSER_JR => matches!(
            status,
            super::fighter_status::KOOPAJR_SPECIAL_HI_SHOOT
                | super::fighter_status::KOOPAJR_SPECIAL_HI_FALL
                | super::fighter_status::KOOPAJR_SPECIAL_HI_ATTACK
                | super::fighter_status::KOOPAJR_SPECIAL_HI_ESCAPE
                | super::fighter_status::KOOPAJR_SPECIAL_HI_LANDING
        ), // KOOPAJR
        crate::fighter_kinds::DUCK_HUNT => matches!(
            status,
            super::fighter_status::DUCKHUNT_SPECIAL_HI_FLY
                | super::fighter_status::DUCKHUNT_SPECIAL_HI_END
        ), // DUCKHUNT
        crate::fighter_kinds::RYU | crate::fighter_kinds::KEN => matches!(
            status,
            super::fighter_status::RYU_SPECIAL_HI_JUMP
                | super::fighter_status::RYU_SPECIAL_HI_FALL
                | super::fighter_status::RYU_SPECIAL_HI_LANDING
                | super::fighter_status::RYU_SPECIAL_HI_COMMAND
        ), // RYU / KEN
        crate::fighter_kinds::CLOUD => matches!(
            status,
            super::fighter_status::CLOUD_SPECIAL_HI2
                | super::fighter_status::CLOUD_SPECIAL_HI2_FALL
                | super::fighter_status::CLOUD_SPECIAL_HI2_LANDING
        ), // CLOUD
        crate::fighter_kinds::BAYONETTA => {
            matches!(status, super::fighter_status::BAYONETTA_SPECIAL_HI_JUMP)
        } // BAYONETTA
        crate::fighter_kinds::INKLING => matches!(
            status,
            super::fighter_status::INKLING_SPECIAL_HI_JUMP
                | super::fighter_status::INKLING_SPECIAL_HI_ROT
                | super::fighter_status::INKLING_SPECIAL_HI_STOP_CEIL
                | super::fighter_status::INKLING_SPECIAL_HI_FALL
        ), // INKLING
        crate::fighter_kinds::RIDLEY => matches!(
            status,
            super::fighter_status::RIDLEY_SPECIAL_HI_HOVER
                | super::fighter_status::RIDLEY_SPECIAL_HI_CHARGE_START
                | super::fighter_status::RIDLEY_SPECIAL_HI_CHARGE_F
                | super::fighter_status::RIDLEY_SPECIAL_HI_CHARGE_HI
                | super::fighter_status::RIDLEY_SPECIAL_HI_STOP_CEIL
                | super::fighter_status::RIDLEY_SPECIAL_HI_STOP_WALL
                | super::fighter_status::RIDLEY_SPECIAL_HI_LANDING
                | super::fighter_status::RIDLEY_SPECIAL_HI_END
        ), // RIDLEY
        crate::fighter_kinds::KING_K_ROOL => matches!(
            status,
            super::fighter_status::KROOL_SPECIAL_HI_START
                | super::fighter_status::KROOL_SPECIAL_HI
                | super::fighter_status::KROOL_SPECIAL_HI_AIR_END
                | super::fighter_status::KROOL_SPECIAL_HI_FALL
                | super::fighter_status::KROOL_SPECIAL_HI_LANDING
        ), // KROOL
        crate::fighter_kinds::INCINEROAR => matches!(
            status,
            super::fighter_status::GAOGAEN_SPECIAL_HI_TURN
                | super::fighter_status::GAOGAEN_SPECIAL_HI_FALL
                | super::fighter_status::GAOGAEN_SPECIAL_HI_LOOP
                | super::fighter_status::GAOGAEN_SPECIAL_HI_END
                | super::fighter_status::GAOGAEN_SPECIAL_HI_BOUND
        ), // GAOGAEN
        crate::fighter_kinds::MII_BRAWLER => matches!(
            status,
            super::fighter_status::MIIFIGHTER_SPECIAL_HI1_2
                | super::fighter_status::MIIFIGHTER_SPECIAL_HI1_3
                | super::fighter_status::MIIFIGHTER_SPECIAL_HI1_4
                | super::fighter_status::MIIFIGHTER_SPECIAL_HI2_ATTACK
                | super::fighter_status::MIIFIGHTER_SPECIAL_HI2_END
        ), // MIIFIGHTER
        crate::fighter_kinds::MII_SWORDFIGHTER => matches!(
            status,
            super::fighter_status::MIISWORDSMAN_SPECIAL_HI1_JUMP
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI1_LOOP
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI1_END
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI2_RUSH
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI2_RUSH_END
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI2_BOUND
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI3_HOLD
                | super::fighter_status::MIISWORDSMAN_SPECIAL_HI3_END
        ), // MIISWORDSMAN
        crate::fighter_kinds::MII_GUNNER => matches!(
            status,
            super::fighter_status::MIIGUNNER_SPECIAL_HI2_JUMP
                | super::fighter_status::MIIGUNNER_SPECIAL_HI3_RUSH
                | super::fighter_status::MIIGUNNER_SPECIAL_HI3_RUSH_END
        ), // MIIGUNNER
        crate::fighter_kinds::ICE_CLIMBERS_POPO => matches!(
            status,
            super::fighter_status::POPO_SPECIAL_HI_FAIL
                | super::fighter_status::POPO_SPECIAL_HI_JUMP_PRE
                | super::fighter_status::POPO_SPECIAL_HI_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_JUMP_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_JUMP
                | super::fighter_status::POPO_SPECIAL_HI_CLIFF_COMP_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_CLIFF_PULL_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_CLIFF_COMP
        ), // POPO
        crate::fighter_kinds::ICE_CLIMBERS_NANA => matches!(
            status,
            super::fighter_status::POPO_SPECIAL_HI_FAIL
                | super::fighter_status::POPO_SPECIAL_HI_JUMP_PRE
                | super::fighter_status::POPO_SPECIAL_HI_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_JUMP_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_JUMP
                | super::fighter_status::POPO_SPECIAL_HI_CLIFF_COMP_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_CLIFF_PULL_PARTNER
                | super::fighter_status::POPO_SPECIAL_HI_CLIFF_COMP
        ), // POPO
        crate::fighter_kinds::MII_ENEMY_F => matches!(
            status,
            super::fighter_status::MIIENEMYF_SPECIAL_HI2_2
                | super::fighter_status::MIIENEMYF_SPECIAL_HI2_3
                | super::fighter_status::MIIENEMYF_SPECIAL_HI2_4
                | super::fighter_status::MIIENEMYF_SPECIAL_HI3_ATTACK
                | super::fighter_status::MIIENEMYF_SPECIAL_HI3_END
        ), // MIIENEMYF
        crate::fighter_kinds::MII_ENEMY_S => matches!(
            status,
            super::fighter_status::MIIENEMYS_SPECIAL_HI1_HOLD
                | super::fighter_status::MIIENEMYS_SPECIAL_HI1_END
                | super::fighter_status::MIIENEMYS_SPECIAL_HI2_RUSH
                | super::fighter_status::MIIENEMYS_SPECIAL_HI2_BOUND
                | super::fighter_status::MIIENEMYS_SPECIAL_HI3_LOOP
                | super::fighter_status::MIIENEMYS_SPECIAL_HI3_END
        ), // MIIENEMYS
        crate::fighter_kinds::MII_ENEMY_G => matches!(
            status,
            super::fighter_status::MIIENEMYG_SPECIAL_HI1_JUMP
                | super::fighter_status::MIIENEMYG_SPECIAL_HI2_FAIL
                | super::fighter_status::MIIENEMYG_SPECIAL_HI3_RUSH
                | super::fighter_status::MIIENEMYG_SPECIAL_HI3_RUSH_END
        ), // MIIENEMYG
        crate::fighter_kinds::PIRANHA_PLANT => matches!(
            status,
            super::fighter_status::PACKUN_SPECIAL_HI_END
                | super::fighter_status::PACKUN_SPECIAL_HI_LANDING
        ), // PACKUN
        crate::fighter_kinds::JOKER => matches!(
            status,
            super::fighter_status::JACK_SPECIAL_HI_PULL
                | super::fighter_status::JACK_SPECIAL_HI_PICKUP
                | super::fighter_status::JACK_SPECIAL_HI_THROW
                | super::fighter_status::JACK_SPECIAL_HI_CUT
                | super::fighter_status::JACK_SPECIAL_HI2_RUSH
                | super::fighter_status::JACK_SPECIAL_HI2_END
        ), // JACK
        crate::fighter_kinds::HERO => matches!(
            status,
            super::fighter_status::BRAVE_SPECIAL_HI_HOLD
                | super::fighter_status::BRAVE_SPECIAL_HI_JUMP
        ), // BRAVE
        crate::fighter_kinds::BANJO_AND_KAZOOIE => {
            matches!(status, super::fighter_status::BUDDY_SPECIAL_HI_JUMP)
        } // BUDDY
        crate::fighter_kinds::TERRY => matches!(
            status,
            super::fighter_status::DOLLY_SPECIAL_HI_JUMP
                | super::fighter_status::DOLLY_SPECIAL_HI_FALL
                | super::fighter_status::DOLLY_SPECIAL_HI_LANDING
                | super::fighter_status::DOLLY_SPECIAL_HI_COMMAND
        ), // DOLLY
        crate::fighter_kinds::BYLETH => matches!(
            status,
            super::fighter_status::MASTER_SPECIAL_HI_HIT
                | super::fighter_status::MASTER_SPECIAL_HI_OVERTAKE
                | super::fighter_status::MASTER_SPECIAL_HI_WALL_JUMP
                | super::fighter_status::MASTER_SPECIAL_HI_FAIL
        ), // MASTER
        crate::fighter_kinds::MIN_MIN => matches!(
            status,
            super::fighter_status::TANTAN_SPECIAL_HI_GROUND
                | super::fighter_status::TANTAN_SPECIAL_HI_GROUND_JUMP
                | super::fighter_status::TANTAN_SPECIAL_HI_GROUND_END
                | super::fighter_status::TANTAN_SPECIAL_HI_AIR
                | super::fighter_status::TANTAN_SPECIAL_HI_AIR_REACH
                | super::fighter_status::TANTAN_SPECIAL_HI_AIR_END
        ), // TANTAN
        crate::fighter_kinds::STEVE => matches!(
            status,
            super::fighter_status::PICKEL_SPECIAL_HI_JUMP
                | super::fighter_status::PICKEL_SPECIAL_HI_WAIT
                | super::fighter_status::PICKEL_SPECIAL_HI_GLIDING
                | super::fighter_status::PICKEL_SPECIAL_HI_LANDING
                | super::fighter_status::PICKEL_SPECIAL_HI_FALL
                | super::fighter_status::PICKEL_SPECIAL_HI_FALL_SPECIAL
        ), // PICKEL
        crate::fighter_kinds::SEPHIROTH => matches!(
            status,
            super::fighter_status::EDGE_SPECIAL_HI_RUSH
                | super::fighter_status::EDGE_SPECIAL_HI_CHARGED_RUSH
                | super::fighter_status::EDGE_SPECIAL_HI_LANDING
                | super::fighter_status::EDGE_SPECIAL_HI_END
        ), // EDGE
        crate::fighter_kinds::PYRA => matches!(
            status,
            super::fighter_status::EFLAME_SPECIAL_HI_JUMP
                | super::fighter_status::EFLAME_SPECIAL_HI_LOOP
                | super::fighter_status::EFLAME_SPECIAL_HI_END
        ), // EFLAME
        crate::fighter_kinds::MYTHRA => matches!(
            status,
            super::fighter_status::ELIGHT_SPECIAL_HI_JUMP
                | super::fighter_status::ELIGHT_SPECIAL_HI_ATTACK1
                | super::fighter_status::ELIGHT_SPECIAL_HI_ATTACK2
                | super::fighter_status::ELIGHT_SPECIAL_HI_FINISH
        ), // ELIGHT
        crate::fighter_kinds::KAZUYA => matches!(
            status,
            super::fighter_status::DEMON_SPECIAL_HI_RISE
                | super::fighter_status::DEMON_SPECIAL_HI_FALL
        ), // DEMON
        _ => false,
    }
}
