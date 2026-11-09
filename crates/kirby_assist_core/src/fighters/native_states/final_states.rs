//! Pinned kind-specific Final Smash phases; do not apply these numbers to
//! another fighter where they may mean ordinary attacks or recovery.
pub(super) fn final_status(kind: i32, status: i32) -> bool {
    match kind {
        crate::fighter_kinds::DONKEY_KONG => matches!(
            status,
            super::fighter_status::DONKEY_FINAL_ATTACK | super::fighter_status::DONKEY_FINAL_END
        ), // DONKEY
        crate::fighter_kinds::LINK => matches!(
            status,
            super::fighter_status::LINK_FINAL_DASH | super::fighter_status::LINK_FINAL_COMBO
        ), // LINK
        crate::fighter_kinds::SAMUS => matches!(
            status,
            super::fighter_status::SAMUS_FINAL_SZEROSUIT
                | super::fighter_status::SAMUS_FINAL_START_G
                | super::fighter_status::SAMUS_FINAL_START_A
                | super::fighter_status::SAMUS_FINAL_LOOP_G
                | super::fighter_status::SAMUS_FINAL_LOOP_A
                | super::fighter_status::SAMUS_FINAL_END_G
                | super::fighter_status::SAMUS_FINAL_END_A
        ), // SAMUS
        crate::fighter_kinds::YOSHI => matches!(
            status,
            super::fighter_status::YOSHI_FINAL_DASH
                | super::fighter_status::YOSHI_FINAL_DASH_END
                | super::fighter_status::YOSHI_FINAL_HIT
                | super::fighter_status::YOSHI_FINAL_SCENE01
                | super::fighter_status::YOSHI_FINAL_END
        ), // YOSHI
        crate::fighter_kinds::KIRBY => matches!(
            status,
            super::fighter_status::KIRBY_FINAL_CALLING
                | super::fighter_status::KIRBY_FINAL_UP_NABE
                | super::fighter_status::KIRBY_FINAL_ATTACK
                | super::fighter_status::KIRBY_FINAL_DOWN_NABE
                | super::fighter_status::KIRBY_FINAL_END
        ), // KIRBY
        crate::fighter_kinds::FOX => matches!(
            status,
            super::fighter_status::FOX_FINAL_READY
                | super::fighter_status::FOX_FINAL_SCENE01
                | super::fighter_status::FOX_FINAL_END
        ), // FOX
        crate::fighter_kinds::PIKACHU => matches!(
            status,
            super::fighter_status::PIKACHU_FINAL_DASH
                | super::fighter_status::PIKACHU_FINAL_DASH_END
                | super::fighter_status::PIKACHU_FINAL_HIT
                | super::fighter_status::PIKACHU_FINAL_ATTACK
                | super::fighter_status::PIKACHU_FINAL_ATTACK_2
                | super::fighter_status::PIKACHU_FINAL_ATTACK_FINISH
                | super::fighter_status::PIKACHU_FINAL_RETURN
                | super::fighter_status::PIKACHU_FINAL_END
        ), // PIKACHU
        crate::fighter_kinds::LUIGI => matches!(
            status,
            super::fighter_status::LUIGI_FINAL_VACUUM
                | super::fighter_status::LUIGI_FINAL_SHOOT_READY
                | super::fighter_status::LUIGI_FINAL_SHOOT
                | super::fighter_status::LUIGI_FINAL_END
                | super::fighter_status::LUIGI_FINAL_FAIL
        ), // LUIGI
        crate::fighter_kinds::NESS => matches!(status, super::fighter_status::NESS_FINAL_ATTACK), // NESS
        crate::fighter_kinds::CAPTAIN_FALCON => matches!(
            status,
            super::fighter_status::CAPTAIN_FINAL_RIDE
                | super::fighter_status::CAPTAIN_FINAL_SCENE01
                | super::fighter_status::CAPTAIN_FINAL_SCENE02
                | super::fighter_status::CAPTAIN_FINAL_SCENE03
                | super::fighter_status::CAPTAIN_FINAL_END
        ), // CAPTAIN
        crate::fighter_kinds::JIGGLYPUFF => matches!(
            status,
            super::fighter_status::PURIN_FINAL_WAIT | super::fighter_status::PURIN_FINAL_END
        ), // PURIN
        crate::fighter_kinds::BOWSER => matches!(status, super::fighter_status::KOOPA_FINAL_END), // KOOPA
        crate::fighter_kinds::SHEIK => matches!(
            status,
            super::fighter_status::SHEIK_FINAL_START
                | super::fighter_status::SHEIK_FINAL_DASH
                | super::fighter_status::SHEIK_FINAL_DASH_END
                | super::fighter_status::SHEIK_FINAL_HIT
                | super::fighter_status::SHEIK_FINAL_ATTACK
                | super::fighter_status::SHEIK_FINAL_FINISH
                | super::fighter_status::SHEIK_FINAL_END
        ), // SHEIK
        crate::fighter_kinds::ZELDA => matches!(
            status,
            super::fighter_status::ZELDA_FINAL_START
                | super::fighter_status::ZELDA_FINAL_LOOP
                | super::fighter_status::ZELDA_FINAL_FINISH
                | super::fighter_status::ZELDA_FINAL_END
        ), // ZELDA
        crate::fighter_kinds::FALCO => matches!(
            status,
            super::fighter_status::FALCO_FINAL_READY
                | super::fighter_status::FALCO_FINAL_SCENE01
                | super::fighter_status::FALCO_FINAL_END
        ), // FALCO
        crate::fighter_kinds::MARTH => matches!(
            status,
            super::fighter_status::MARTH_FINAL_DASH
                | super::fighter_status::MARTH_FINAL_DASH_END
                | super::fighter_status::MARTH_FINAL_END
        ), // MARTH
        crate::fighter_kinds::GANONDORF => matches!(
            status,
            super::fighter_status::GANON_FINAL_ATTACK | super::fighter_status::GANON_FINAL_END
        ), // GANON
        crate::fighter_kinds::MEWTWO => matches!(status, super::fighter_status::MEWTWO_FINAL_END), // MEWTWO
        crate::fighter_kinds::ROY | crate::fighter_kinds::CHROM => matches!(
            status,
            super::fighter_status::ROY_FINAL_END
                | super::fighter_status::ROY_FINAL_DASH
                | super::fighter_status::ROY_FINAL_DASH_END
                | super::fighter_status::ROY_FINAL_ATTACK
        ), // ROY / CHROM
        crate::fighter_kinds::MR_GAME_AND_WATCH => matches!(
            status,
            super::fighter_status::GAMEWATCH_FINAL_WAIT
                | super::fighter_status::GAMEWATCH_FINAL_WALK
                | super::fighter_status::GAMEWATCH_FINAL_DASH
                | super::fighter_status::GAMEWATCH_FINAL_RUN
                | super::fighter_status::GAMEWATCH_FINAL_RUN_BRAKE
                | super::fighter_status::GAMEWATCH_FINAL_TURN
                | super::fighter_status::GAMEWATCH_FINAL_TURN_DASH
                | super::fighter_status::GAMEWATCH_FINAL_TURN_RUN
                | super::fighter_status::GAMEWATCH_FINAL_JUMP_SQUAT
                | super::fighter_status::GAMEWATCH_FINAL_JUMP
                | super::fighter_status::GAMEWATCH_FINAL_JUMP_AERIAL
                | super::fighter_status::GAMEWATCH_FINAL_FALL
                | super::fighter_status::GAMEWATCH_FINAL_FALL_AERIAL
                | super::fighter_status::GAMEWATCH_FINAL_PASS
                | super::fighter_status::GAMEWATCH_FINAL_LANDING
                | super::fighter_status::GAMEWATCH_FINAL_LANDING_LIGHT
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_DIVE
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_RISE
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_WAIT
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_WALK
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_DASH
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_RUN
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_RUN_BRAKE
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_TURN
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_TURN_DASH
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_TURN_RUN
                | super::fighter_status::GAMEWATCH_FINAL_SWIM_JUMP_SQUAT
                | super::fighter_status::GAMEWATCH_FINAL_END
        ), // GAMEWATCH
        crate::fighter_kinds::META_KNIGHT => matches!(
            status,
            super::fighter_status::METAKNIGHT_FINAL_HIT_WAIT
                | super::fighter_status::METAKNIGHT_FINAL_SQUAT
                | super::fighter_status::METAKNIGHT_FINAL_JUMP
                | super::fighter_status::METAKNIGHT_FINAL_ATTACK
                | super::fighter_status::METAKNIGHT_FINAL_END
        ), // METAKNIGHT
        crate::fighter_kinds::PIT | crate::fighter_kinds::DARK_PIT => matches!(
            status,
            super::fighter_status::PIT_FINAL_WAIT
                | super::fighter_status::PIT_FINAL_ATTACK
                | super::fighter_status::PIT_FINAL_END
        ), // PIT / DARK PIT
        crate::fighter_kinds::ZERO_SUIT_SAMUS => matches!(
            status,
            super::fighter_status::SZEROSUIT_FINAL_LOCK_ON
                | super::fighter_status::SZEROSUIT_FINAL_END
        ), // SZEROSUIT
        crate::fighter_kinds::WARIO => matches!(
            status,
            super::fighter_status::WARIO_FINAL_DASH
                | super::fighter_status::WARIO_FINAL_DASH_END
                | super::fighter_status::WARIO_FINAL_READY
                | super::fighter_status::WARIO_FINAL_SCENE
                | super::fighter_status::WARIO_FINAL_END
        ), // WARIO
        crate::fighter_kinds::SNAKE => matches!(
            status,
            super::fighter_status::SNAKE_FINAL_LOCK_ON
                | super::fighter_status::SNAKE_FINAL_CAMERA_ZOOM
                | super::fighter_status::SNAKE_FINAL_END
                | super::fighter_status::SNAKE_FINAL_APPEAL
        ), // SNAKE
        crate::fighter_kinds::IKE => matches!(
            status,
            super::fighter_status::IKE_FINAL_HIT
                | super::fighter_status::IKE_FINAL_MOVE
                | super::fighter_status::IKE_FINAL_ATTACK
                | super::fighter_status::IKE_FINAL_FALL
                | super::fighter_status::IKE_FINAL_END
        ), // IKE
        crate::fighter_kinds::DIDDY_KONG => matches!(
            status,
            super::fighter_status::DIDDY_FINAL_START
                | super::fighter_status::DIDDY_FINAL_WAIT_FLY
                | super::fighter_status::DIDDY_FINAL_FLY
                | super::fighter_status::DIDDY_FINAL_READY_CHARGE
                | super::fighter_status::DIDDY_FINAL_CHARGE
                | super::fighter_status::DIDDY_FINAL_FINISH_ATTACK
                | super::fighter_status::DIDDY_FINAL_READY_END
                | super::fighter_status::DIDDY_FINAL_END
        ), // DIDDY
        crate::fighter_kinds::LUCAS => matches!(status, super::fighter_status::LUCAS_FINAL_ATTACK), // LUCAS
        crate::fighter_kinds::SONIC => matches!(status, super::fighter_status::SONIC_FINAL_END), // SONIC
        crate::fighter_kinds::KING_DEDEDE => matches!(
            status,
            super::fighter_status::DEDEDE_FINAL_READY
                | super::fighter_status::DEDEDE_FINAL_SCENE01
                | super::fighter_status::DEDEDE_FINAL_SCENE02
                | super::fighter_status::DEDEDE_FINAL_SCENE03
                | super::fighter_status::DEDEDE_FINAL_END
        ), // DEDEDE
        crate::fighter_kinds::OLIMAR => matches!(
            status,
            super::fighter_status::PIKMIN_FINAL_FLY
                | super::fighter_status::PIKMIN_FINAL_WAIT
                | super::fighter_status::PIKMIN_FINAL_FALL
                | super::fighter_status::PIKMIN_FINAL_END
        ), // PIKMIN
        crate::fighter_kinds::LUCARIO => matches!(
            status,
            super::fighter_status::LUCARIO_FINAL_ENTRY
                | super::fighter_status::LUCARIO_FINAL_HOLD
                | super::fighter_status::LUCARIO_FINAL_ATTACK
                | super::fighter_status::LUCARIO_FINAL_END
        ), // LUCARIO
        crate::fighter_kinds::ROB => matches!(
            status,
            super::fighter_status::ROBOT_FINAL_LOOP | super::fighter_status::ROBOT_FINAL_END
        ), // ROBOT
        crate::fighter_kinds::WOLF => matches!(
            status,
            super::fighter_status::WOLF_FINAL_JUMP
                | super::fighter_status::WOLF_FINAL_CONTROL
                | super::fighter_status::WOLF_FINAL_END
        ), // WOLF
        crate::fighter_kinds::VILLAGER => matches!(
            status,
            super::fighter_status::MURABITO_FINAL_SURPRISE
                | super::fighter_status::MURABITO_FINAL_MONEY
                | super::fighter_status::MURABITO_FINAL_CHEER
                | super::fighter_status::MURABITO_FINAL_HAPPY
                | super::fighter_status::MURABITO_FINAL_END
        ), // MURABITO
        crate::fighter_kinds::MEGA_MAN => matches!(
            status,
            super::fighter_status::ROCKMAN_FINAL_INHALE
                | super::fighter_status::ROCKMAN_FINAL_SCENE01
                | super::fighter_status::ROCKMAN_FINAL_SCENE02
                | super::fighter_status::ROCKMAN_FINAL_SCENE05
                | super::fighter_status::ROCKMAN_FINAL_END
        ), // ROCKMAN
        crate::fighter_kinds::WII_FIT_TRAINER => matches!(
            status,
            super::fighter_status::WIIFIT_FINAL_HOLD | super::fighter_status::WIIFIT_FINAL_END
        ), // WIIFIT
        crate::fighter_kinds::LITTLE_MAC => matches!(
            status,
            super::fighter_status::LITTLEMAC_FINAL_DASH
                | super::fighter_status::LITTLEMAC_FINAL_DASH_END
                | super::fighter_status::LITTLEMAC_FINAL_HIT
                | super::fighter_status::LITTLEMAC_FINAL_ATTACK
                | super::fighter_status::LITTLEMAC_FINAL_END
        ), // LITTLEMAC
        crate::fighter_kinds::GRENINJA => matches!(
            status,
            super::fighter_status::GEKKOUGA_FINAL_WAIT | super::fighter_status::GEKKOUGA_FINAL_END
        ), // GEKKOUGA
        crate::fighter_kinds::PALUTENA => matches!(
            status,
            super::fighter_status::PALUTENA_FINAL_WAIT
                | super::fighter_status::PALUTENA_FINAL_BEAM_START
                | super::fighter_status::PALUTENA_FINAL_BEAM
                | super::fighter_status::PALUTENA_FINAL_BEAM_END
        ), // PALUTENA
        crate::fighter_kinds::PAC_MAN => matches!(
            status,
            super::fighter_status::PACMAN_FINAL_MOVE | super::fighter_status::PACMAN_FINAL_END
        ), // PACMAN
        crate::fighter_kinds::ROBIN => matches!(
            status,
            super::fighter_status::REFLET_FINAL_START
                | super::fighter_status::REFLET_FINAL_FAIL
                | super::fighter_status::REFLET_FINAL_READY
                | super::fighter_status::REFLET_FINAL_END
                | super::fighter_status::REFLET_FINAL_CHROM_ENTRY
                | super::fighter_status::REFLET_FINAL_CHROM_DASH
                | super::fighter_status::REFLET_FINAL_CHROM_LAUNCH
                | super::fighter_status::REFLET_FINAL_CHROM_MISS
                | super::fighter_status::REFLET_FINAL_CHROM_RUSH
                | super::fighter_status::REFLET_FINAL_CHROM_READY
                | super::fighter_status::REFLET_FINAL_CHROM_ATTACK
                | super::fighter_status::REFLET_FINAL_CHROM_EXIT
                | super::fighter_status::REFLET_FINAL_HIT
                | super::fighter_status::REFLET_FINAL_MOVE
                | super::fighter_status::REFLET_FINAL_ATTACK
        ), // REFLET
        crate::fighter_kinds::SHULK => matches!(
            status,
            super::fighter_status::SHULK_FINAL_READY
                | super::fighter_status::SHULK_FINAL_SCENE_ENTRY
                | super::fighter_status::SHULK_FINAL_SCENE_ATTACK
                | super::fighter_status::SHULK_FINAL_END
        ), // SHULK
        crate::fighter_kinds::BOWSER_JR => matches!(
            status,
            super::fighter_status::KOOPAJR_FINAL_DRAW | super::fighter_status::KOOPAJR_FINAL_END
        ), // KOOPAJR
        crate::fighter_kinds::DUCK_HUNT => matches!(
            status,
            super::fighter_status::DUCKHUNT_FINAL_READY
                | super::fighter_status::DUCKHUNT_FINAL_SCENE01
                | super::fighter_status::DUCKHUNT_FINAL_SCENE02
                | super::fighter_status::DUCKHUNT_FINAL_SCENE03
                | super::fighter_status::DUCKHUNT_FINAL_SCENE04
                | super::fighter_status::DUCKHUNT_FINAL_SCENE05
                | super::fighter_status::DUCKHUNT_FINAL_END
        ), // DUCKHUNT
        crate::fighter_kinds::RYU => matches!(
            status,
            super::fighter_status::RYU_FINAL_JUMP
                | super::fighter_status::RYU_FINAL_FALL
                | super::fighter_status::RYU_FINAL_LANDING
                | super::fighter_status::RYU_FINAL_AIR_END
                | super::fighter_status::RYU_FINAL_HIT
                | super::fighter_status::RYU_FINAL2
                | super::fighter_status::RYU_FINAL2_FALL
                | super::fighter_status::RYU_FINAL2_LANDING
                | super::fighter_status::RYU_FINAL2_AIR_END
        ), // RYU
        crate::fighter_kinds::KEN => matches!(
            status,
            super::fighter_status::RYU_FINAL_JUMP
                | super::fighter_status::RYU_FINAL_FALL
                | super::fighter_status::RYU_FINAL_LANDING
                | super::fighter_status::RYU_FINAL_AIR_END
                | super::fighter_status::RYU_FINAL_HIT
                | super::fighter_status::RYU_FINAL2
                | super::fighter_status::RYU_FINAL2_FALL
                | super::fighter_status::RYU_FINAL2_LANDING
                | super::fighter_status::RYU_FINAL2_AIR_END
        ), // RYU
        crate::fighter_kinds::CLOUD => matches!(
            status,
            super::fighter_status::CLOUD_FINAL_DASH
                | super::fighter_status::CLOUD_FINAL_DASH_END
                | super::fighter_status::CLOUD_FINAL_HIT
                | super::fighter_status::CLOUD_FINAL_MOVE
                | super::fighter_status::CLOUD_FINAL_ATTACK
                | super::fighter_status::CLOUD_FINAL_FALL
                | super::fighter_status::CLOUD_FINAL_END
                | super::fighter_status::CLOUD_FINAL2
                | super::fighter_status::CLOUD_FINAL2_DASH
                | super::fighter_status::CLOUD_FINAL2_DASH_END
                | super::fighter_status::CLOUD_FINAL2_HIT
                | super::fighter_status::CLOUD_FINAL2_MOVE
                | super::fighter_status::CLOUD_FINAL2_ATTACK
                | super::fighter_status::CLOUD_FINAL2_FALL
                | super::fighter_status::CLOUD_FINAL2_END
        ), // CLOUD
        crate::fighter_kinds::CORRIN => matches!(
            status,
            super::fighter_status::KAMUI_FINAL_READY
                | super::fighter_status::KAMUI_FINAL_SCENE_ENTRY
                | super::fighter_status::KAMUI_FINAL_SCENE_ATTACK
                | super::fighter_status::KAMUI_FINAL_SCENE_ATTACK_WAIT
                | super::fighter_status::KAMUI_FINAL_END
        ), // KAMUI
        crate::fighter_kinds::BAYONETTA => matches!(
            status,
            super::fighter_status::BAYONETTA_FINAL_START
                | super::fighter_status::BAYONETTA_FINAL_SCENE01
                | super::fighter_status::BAYONETTA_FINAL_END
                | super::fighter_status::BAYONETTA_FINAL_END_AFTER
        ), // BAYONETTA
        crate::fighter_kinds::RIDLEY => matches!(
            status,
            super::fighter_status::RIDLEY_FINAL_DASH
                | super::fighter_status::RIDLEY_FINAL_DASH_END
                | super::fighter_status::RIDLEY_FINAL_READY
                | super::fighter_status::RIDLEY_FINAL_SCENE01
                | super::fighter_status::RIDLEY_FINAL_SCENE02
                | super::fighter_status::RIDLEY_FINAL_SCENE03
                | super::fighter_status::RIDLEY_FINAL_SCENE04
                | super::fighter_status::RIDLEY_FINAL_SCENE05
                | super::fighter_status::RIDLEY_FINAL_END
        ), // RIDLEY
        crate::fighter_kinds::SIMON => matches!(
            status,
            super::fighter_status::SIMON_FINAL_READY
                | super::fighter_status::SIMON_FINAL_SCENE01
                | super::fighter_status::SIMON_FINAL_SCENE02
                | super::fighter_status::SIMON_FINAL_SCENE03
                | super::fighter_status::SIMON_FINAL_END
        ), // SIMON
        crate::fighter_kinds::KING_K_ROOL => matches!(
            status,
            super::fighter_status::KROOL_FINAL_DASH
                | super::fighter_status::KROOL_FINAL_DASH_END
                | super::fighter_status::KROOL_FINAL_READY
                | super::fighter_status::KROOL_FINAL_SCENE01
                | super::fighter_status::KROOL_FINAL_SCENE02
                | super::fighter_status::KROOL_FINAL_SCENE03
                | super::fighter_status::KROOL_FINAL_SCENE04
                | super::fighter_status::KROOL_FINAL_END
        ), // KROOL
        crate::fighter_kinds::INCINEROAR => matches!(
            status,
            super::fighter_status::GAOGAEN_FINAL_DASH
                | super::fighter_status::GAOGAEN_FINAL_DASH_END
                | super::fighter_status::GAOGAEN_FINAL_READY
                | super::fighter_status::GAOGAEN_FINAL_SCENE01
                | super::fighter_status::GAOGAEN_FINAL_SCENE02
                | super::fighter_status::GAOGAEN_FINAL_SCENE03
                | super::fighter_status::GAOGAEN_FINAL_END
        ), // GAOGAEN
        crate::fighter_kinds::MII_BRAWLER => matches!(
            status,
            super::fighter_status::MIIFIGHTER_FINAL_HIT
                | super::fighter_status::MIIFIGHTER_FINAL_MOVE
                | super::fighter_status::MIIFIGHTER_FINAL_ATTACK
                | super::fighter_status::MIIFIGHTER_FINAL_END
        ), // MIIFIGHTER
        crate::fighter_kinds::MII_SWORDFIGHTER => matches!(
            status,
            super::fighter_status::MIISWORDSMAN_FINAL_HOLD
                | super::fighter_status::MIISWORDSMAN_FINAL_END
        ), // MIISWORDSMAN
        crate::fighter_kinds::MII_GUNNER => matches!(
            status,
            super::fighter_status::MIIGUNNER_FINAL_LOOP
                | super::fighter_status::MIIGUNNER_FINAL_END
        ), // MIIGUNNER
        crate::fighter_kinds::ICE_CLIMBERS_POPO => matches!(
            status,
            super::fighter_status::POPO_FINAL_PARTNER | super::fighter_status::POPO_FINAL_HANG
        ), // POPO
        crate::fighter_kinds::ICE_CLIMBERS_NANA => matches!(
            status,
            super::fighter_status::POPO_FINAL_PARTNER | super::fighter_status::POPO_FINAL_HANG
        ), // POPO
        crate::fighter_kinds::MII_ENEMY_S => matches!(
            status,
            super::fighter_status::MIIENEMYS_FINAL_HOLD
                | super::fighter_status::MIIENEMYS_FINAL_END
        ), // MIIENEMYS
        crate::fighter_kinds::JOKER => matches!(
            status,
            super::fighter_status::JACK_FINAL_TURN_DASH
                | super::fighter_status::JACK_FINAL_DASH_END
                | super::fighter_status::JACK_FINAL_READY
                | super::fighter_status::JACK_FINAL_SCENE01
                | super::fighter_status::JACK_FINAL_SCENE02
                | super::fighter_status::JACK_FINAL_END
        ), // JACK
        crate::fighter_kinds::HERO => matches!(
            status,
            super::fighter_status::BRAVE_FINAL_READY
                | super::fighter_status::BRAVE_FINAL_SCENE01
                | super::fighter_status::BRAVE_FINAL_END
        ), // BRAVE
        crate::fighter_kinds::BANJO_AND_KAZOOIE => matches!(
            status,
            super::fighter_status::BUDDY_FINAL_READY
                | super::fighter_status::BUDDY_FINAL_SCENE01
                | super::fighter_status::BUDDY_FINAL_SCENE02
                | super::fighter_status::BUDDY_FINAL_SCENE03
                | super::fighter_status::BUDDY_FINAL_SCENE04
                | super::fighter_status::BUDDY_FINAL_END
        ), // BUDDY
        crate::fighter_kinds::TERRY => matches!(
            status,
            super::fighter_status::DOLLY_FINAL_READY
                | super::fighter_status::DOLLY_FINAL_SCENE01
                | super::fighter_status::DOLLY_FINAL_SCENE02
                | super::fighter_status::DOLLY_FINAL_SCENE03
                | super::fighter_status::DOLLY_FINAL_SCENE04
                | super::fighter_status::DOLLY_FINAL_SCENE05
                | super::fighter_status::DOLLY_FINAL_END
        ), // DOLLY
        crate::fighter_kinds::BYLETH => matches!(
            status,
            super::fighter_status::MASTER_FINAL_READY
                | super::fighter_status::MASTER_FINAL_SCENE01
                | super::fighter_status::MASTER_FINAL_SCENE02
                | super::fighter_status::MASTER_FINAL_SCENE03
                | super::fighter_status::MASTER_FINAL_END
        ), // MASTER
        crate::fighter_kinds::MIN_MIN => matches!(
            status,
            super::fighter_status::TANTAN_FINAL_READY
                | super::fighter_status::TANTAN_FINAL_SCENE01
                | super::fighter_status::TANTAN_FINAL_SCENE02
                | super::fighter_status::TANTAN_FINAL_SCENE03
                | super::fighter_status::TANTAN_FINAL_END
        ), // TANTAN
        crate::fighter_kinds::STEVE => matches!(
            status,
            super::fighter_status::PICKEL_FINAL_READY
                | super::fighter_status::PICKEL_FINAL_SCENE01
                | super::fighter_status::PICKEL_FINAL_END
        ), // PICKEL
        crate::fighter_kinds::SEPHIROTH => matches!(
            status,
            super::fighter_status::EDGE_FINAL_READY
                | super::fighter_status::EDGE_FINAL_SCENE01
                | super::fighter_status::EDGE_FINAL_END
        ), // EDGE
        crate::fighter_kinds::PYRA => matches!(
            status,
            super::fighter_status::EFLAME_FINAL_READY
                | super::fighter_status::EFLAME_FINAL_SCENE01
                | super::fighter_status::EFLAME_FINAL_SCENE02
                | super::fighter_status::EFLAME_FINAL_END
        ), // EFLAME
        crate::fighter_kinds::MYTHRA => matches!(
            status,
            super::fighter_status::ELIGHT_FINAL_READY
                | super::fighter_status::ELIGHT_FINAL_SCENE01
                | super::fighter_status::ELIGHT_FINAL_SCENE02
                | super::fighter_status::ELIGHT_FINAL_END
                | super::fighter_status::ELIGHT_FINAL_SCENE03
                | super::fighter_status::ELIGHT_FINAL_SCENE04
        ), // ELIGHT
        crate::fighter_kinds::KAZUYA => matches!(
            status,
            super::fighter_status::DEMON_FINAL_READY
                | super::fighter_status::DEMON_FINAL_SCENE01
                | super::fighter_status::DEMON_FINAL_END
        ), // DEMON
        crate::fighter_kinds::SORA => matches!(
            status,
            super::fighter_status::TRAIL_FINAL_READY
                | super::fighter_status::TRAIL_FINAL_SCENE01
                | super::fighter_status::TRAIL_FINAL_SCENE02
                | super::fighter_status::TRAIL_FINAL_SCENE03
                | super::fighter_status::TRAIL_FINAL_SCENE04
                | super::fighter_status::TRAIL_FINAL_END
        ), // TRAIL
        _ => false,
    }
}
