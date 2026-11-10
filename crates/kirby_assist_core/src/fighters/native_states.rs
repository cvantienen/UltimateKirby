//! Shared read-only native state vocabulary (skyline-smash d2ca90d lua_const).
//! Own incapacitation and opponent punishability are deliberately distinct.
pub mod fighter_status;
mod kind_statuses;
pub use kind_statuses::up_special_status;
pub mod status {
    //! Generic fighter status IDs copied from the pinned `skyline-smash` table.
    //!
    //! Use these names only for shared statuses. Character-specific status values
    //! can overlap and need a fighter-specific name and identity check.
    pub const WAIT: i32 = 0x00;
    pub const WALK: i32 = 0x01;
    pub const WALK_BRAKE: i32 = 0x02;
    pub const DASH: i32 = 0x03;
    pub const RUN: i32 = 0x04;
    pub const RUN_BRAKE: i32 = 0x05;
    pub const TURN: i32 = 0x06;
    pub const TURN_DASH: i32 = 0x07;
    pub const TURN_RUN: i32 = 0x08;
    pub const TURN_RUN_BRAKE: i32 = 0x09;
    pub const JUMP_SQUAT: i32 = 0x0a;
    pub const JUMP: i32 = 0x0b;
    pub const JUMP_AERIAL: i32 = 0x0c;
    pub const FALL: i32 = 0x0e;
    pub const FALL_AERIAL: i32 = 0x0f;
    pub const FALL_SPECIAL: i32 = 0x10;
    pub const SQUAT: i32 = 0x11;
    pub const SQUAT_WAIT: i32 = 0x12;
    pub const SQUAT_F: i32 = 0x13;
    pub const SQUAT_B: i32 = 0x14;
    pub const SQUAT_RV: i32 = 0x15;
    pub const LANDING: i32 = 0x16;
    pub const LANDING_LIGHT: i32 = 0x17;
    pub const LANDING_ATTACK_AIR: i32 = 0x18;
    pub const LANDING_FALL_SPECIAL: i32 = 0x19;
    pub const LANDING_DAMAGE_LIGHT: i32 = 0x1a;
    pub const GUARD_ON: i32 = 0x1b;
    pub const GUARD: i32 = 0x1c;
    pub const GUARD_OFF: i32 = 0x1d;
    pub const GUARD_DAMAGE: i32 = 0x1e;
    pub const ESCAPE: i32 = 0x1f;
    pub const ESCAPE_F: i32 = 0x20;
    pub const ESCAPE_B: i32 = 0x21;
    pub const ESCAPE_AIR: i32 = 0x22;
    pub const ESCAPE_AIR_SLIDE: i32 = 0x23;
    pub const REBOUND_STOP: i32 = 0x24;
    pub const REBOUND: i32 = 0x25;
    pub const REBOUND_JUMP: i32 = 0x26;
    pub const ATTACK: i32 = 0x27;
    pub const ATTACK_100: i32 = 0x28;
    pub const ATTACK_DASH: i32 = 0x29;
    pub const ATTACK_S3: i32 = 0x2a;
    pub const ATTACK_HI3: i32 = 0x2b;
    pub const ATTACK_LW3: i32 = 0x2c;
    pub const ATTACK_S4_START: i32 = 0x2d;
    pub const ATTACK_S4_HOLD: i32 = 0x2e;
    pub const ATTACK_S4: i32 = 0x2f;
    pub const ATTACK_LW4_START: i32 = 0x30;
    pub const ATTACK_LW4_HOLD: i32 = 0x31;
    pub const ATTACK_LW4: i32 = 0x32;
    pub const ATTACK_HI4_START: i32 = 0x33;
    pub const ATTACK_HI4_HOLD: i32 = 0x34;
    pub const ATTACK_HI4: i32 = 0x35;
    pub const ATTACK_AIR: i32 = 0x36;
    pub const CATCH: i32 = 0x37;
    pub const CATCH_PULL: i32 = 0x38;
    pub const CATCH_DASH: i32 = 0x39;
    pub const CATCH_DASH_PULL: i32 = 0x3a;
    pub const CATCH_TURN: i32 = 0x3b;
    pub const CATCH_WAIT: i32 = 0x3c;
    pub const CATCH_ATTACK: i32 = 0x3d;
    pub const CATCH_CUT: i32 = 0x3e;
    pub const CATCH_JUMP: i32 = 0x3f;
    pub const THROW: i32 = 0x40;
    pub const CAPTURE_PULLED: i32 = 0x41;
    pub const CAPTURE_WAIT: i32 = 0x42;
    pub const CAPTURE_DAMAGE: i32 = 0x43;
    pub const CAPTURE_CUT: i32 = 0x44;
    pub const CAPTURE_JUMP: i32 = 0x45;
    pub const THROWN: i32 = 0x46;
    pub const DAMAGE: i32 = 0x47;
    pub const DAMAGE_AIR: i32 = 0x48;
    pub const DAMAGE_FLY: i32 = 0x49;
    pub const DAMAGE_FLY_ROLL: i32 = 0x4a;
    pub const DAMAGE_FLY_METEOR: i32 = 0x4b;
    pub const DAMAGE_FLY_REFLECT_LR: i32 = 0x4c;
    pub const DAMAGE_FLY_REFLECT_U: i32 = 0x4d;
    pub const DAMAGE_FLY_REFLECT_D: i32 = 0x4e;
    pub const DAMAGE_FALL: i32 = 0x4f;
    pub const DOWN: i32 = 0x50;
    pub const DOWN_SPOT: i32 = 0x51;
    pub const DOWN_CONTINUE: i32 = 0x52;
    pub const DOWN_WAIT: i32 = 0x53;
    pub const DOWN_WAIT_CONTINUE: i32 = 0x54;
    pub const DOWN_EAT: i32 = 0x55;
    pub const LAY_DOWN: i32 = 0x56;
    pub const DOWN_STAND: i32 = 0x57;
    pub const DOWN_STAND_FB: i32 = 0x58;
    pub const DOWN_STAND_ATTACK: i32 = 0x59;
    pub const DOWN_DAMAGE: i32 = 0x5a;
    pub const DOWN_REFLECT_LR: i32 = 0x5b;
    pub const SHIELD_BREAK_FLY: i32 = 0x5c;
    pub const SHIELD_BREAK_FALL: i32 = 0x5d;
    pub const SHIELD_BREAK_DOWN: i32 = 0x5e;
    pub const FURAFURA_STAND: i32 = 0x5f;
    pub const FURAFURA: i32 = 0x60;
    pub const FURAFURA_END: i32 = 0x61;
    pub const DAMAGE_SONG_START: i32 = 0x62;
    pub const DAMAGE_SONG: i32 = 0x63;
    pub const DAMAGE_SONG_FALL: i32 = 0x64;
    pub const DAMAGE_SONG_END: i32 = 0x65;
    pub const BIND: i32 = 0x66;
    pub const PASSIVE: i32 = 0x67;
    pub const PASSIVE_FB: i32 = 0x68;
    pub const PASSIVE_WALL: i32 = 0x69;
    pub const PASSIVE_WALL_JUMP: i32 = 0x6a;
    pub const PASSIVE_CEIL: i32 = 0x6b;
    pub const STOP_WALL: i32 = 0x6c;
    pub const STOP_CEIL: i32 = 0x6d;
    pub const WALL_JUMP: i32 = 0x6e;
    pub const CLIFF_CATCH_MOVE: i32 = 0x75;
    pub const CLIFF_CATCH: i32 = 0x76;
    pub const CLIFF_WAIT: i32 = 0x77;
    pub const CLIFF_ATTACK: i32 = 0x78;
    pub const CLIFF_CLIMB: i32 = 0x79;
    pub const CLIFF_ESCAPE: i32 = 0x7a;
    pub const CLIFF_JUMP1: i32 = 0x7b;
    pub const CLIFF_JUMP2: i32 = 0x7c;
    pub const CLIFF_JUMP3: i32 = 0x7d;
    pub const OTTOTTO: i32 = 0x7e;
    pub const OTTOTTO_WAIT: i32 = 0x7f;
    pub const SLIP: i32 = 0x86;
    pub const SLIP_DAMAGE: i32 = 0x87;
    pub const SLIP_WAIT: i32 = 0x88;
    pub const SWIM_DIVE: i32 = 0xab;
    pub const TREAD_DAMAGE: i32 = 0xb9;
    pub const TREAD_FALL: i32 = 0xbc;
    pub const CLUNG_GANON: i32 = 0xbd;
    pub const SHOULDERED_DONKEY_THROWN: i32 = 0xc7;
    pub const BURY: i32 = 0xc8;
    pub const HAMMER_LANDING: i32 = 0xdf;
    pub const ITEM_SHOOT_WALK_BRAKE_F: i32 = 0x9d;
    pub const DEAD: i32 = 0xb5;
    pub const REBIRTH: i32 = 0xb6;
    pub const CATCHED_GANON: i32 = 0xbe;
    pub const CATCHED_AIR_GANON: i32 = 0xbf;
    pub const CATCHED_AIR_FALL_GANON: i32 = 0xc0;
    pub const CATCHED_AIR_END_GANON: i32 = 0xc1;
    pub const ICE_JUMP: i32 = 0xcc;
    pub const SWALLOWED: i32 = 0xe6;
    pub const CAPTURE_DAMAGE_YOSHI: i32 = 0xf7;
    pub const CATCHED_REFLET: i32 = 0xfe;
    pub const CAPTURE_MASTERHAND: i32 = 0xff;
    pub const KILLER: i32 = 0x100;
    pub const GIMMICK_EATEN: i32 = 0x102;
    pub const GENESIS_SHOOT_START: i32 = 0x103;
    pub const GENESIS_GET: i32 = 0x106;
    pub const DEMO: i32 = 0x107;
    pub const KASEY_WARP: i32 = 0x110;
    pub const LADDER_CATCH: i32 = 0x111;
    pub const LADDER_END: i32 = 0x115;
    pub const CLIFF_ROBBED: i32 = 0x116;
    pub const METAKNIGHT_FINAL_DAMAGE_FALL: i32 = 0x119;
    pub const FINAL_VISUAL_ATTACK_OTHER: i32 = 0x11f;
    pub const PSYCHOBREAK: i32 = 0x147;
    pub const SAVING_DAMAGE: i32 = 0x148;
    pub const SAVING_DAMAGE_FLY: i32 = 0x149;
    pub const RYU_FINAL_DAMAGE: i32 = 0x14a;
    pub const RYU_FINAL_DAMAGE_FLY: i32 = 0x14b;
    pub const SAVING_DAMAGE_AIR: i32 = 0x14c;
    pub const CLOUD_FINAL_DAMAGE_FLY: i32 = 0x14d;
    pub const RIDLEY_FINAL_TARGET_END: i32 = 0x16e;
    pub const CATCHED_RIDLEY: i32 = 0x171;
    pub const STABBED_DAMAGE: i32 = 0x174;
    pub const SWING_GAOGAEN_CATCHED: i32 = 0x176;
    pub const CAPTURE_WAIT_OCTOPUS: i32 = 0x18c;
    pub const SIMON_FINAL_TARGET_START: i32 = 0x18f;
    pub const METAMON_OUT: i32 = 0x196;
    pub const BOSS_DEAD: i32 = 0x198;
    pub const BRAVE_FINAL_TARGET_END: i32 = 0x1a2;
    pub const DAMAGE_SLEEP_START: i32 = 0x1a3;
    pub const SLEEP_END: i32 = 0x1aa;
    pub const BUDDY_FINAL_TARGET_START: i32 = 0x1ab;
    pub const TANTAN_FINAL_TARGET_END: i32 = 0x1ba;
    pub const GIMMICK_JUMP_BOARD: i32 = 0x1bb;
    pub const DAMAGE_FLY_REFLECT_JUMP_BOARD: i32 = 0x1bd;
    pub const CATCHED_PICKEL_TROLLEY: i32 = 0x1be;
    pub const EDGE_FINAL_TARGET_END: i32 = 0x1d0;
    pub const TRAIL_REBOUND: i32 = 0x1d1;
    pub const CAPTURE_TRAIL_KEYHOLE: i32 = 0x1d2;
    pub const STANDBY: i32 = 0x1d6;
    pub const ENTRY: i32 = 0x1d9;
    pub const LOSE: i32 = 0x1db;
    pub const SPECIAL_N: i32 = 0x1dc;
    pub const SPECIAL_S: i32 = 0x1dd;
    pub const SPECIAL_HI: i32 = 0x1de;
    pub const SPECIAL_LW: i32 = 0x1df;
    pub const FINAL: i32 = 0x1e0;
}

/// Ground-touch masks from the pinned `skyline-smash` lua_const table.
pub mod ground_touch {
    pub const DOWN_SIDE: u32 = 0xc8;
    pub const LEFT_SIDE: u32 = 0x52;
    pub const RIGHT_SIDE: u32 = 0xa4;
    pub const UP_SIDE: u32 = 0x31;
}

/// `SITUATION_KIND_*` values from the pinned `skyline-smash` lua_const table.
pub mod situation {
    pub const GROUND: i32 = 0;
    pub const CLIFF: i32 = 1;
    pub const AIR: i32 = 2;
    /// Adapter sentinel when no native situation observation is available.
    pub const UNAVAILABLE: i32 = -1;

    // AttackData.situation is a bitmask, not a SITUATION_KIND value.
    pub const ATTACK_GROUND_MASK: u16 = 1;
    pub const ATTACK_AIR_MASK: u16 = 2;
}

/// Native transition-term IDs from the pinned `skyline-smash` constant table.
pub mod transition_terms {
    pub const PASSIVE: i32 = 0x1e000013;
    pub const PASSIVE_FB: i32 = 0x1e000014;
    pub const PASSIVE_WALL: i32 = 0x1e000015;
    pub const PASSIVE_CEIL: i32 = 0x1e000018;
    pub const CONT_ESCAPE: i32 = 0x1e000026;
    pub const CONT_GUARD_ON: i32 = 0x1e000029;
    pub const CONT_JUMP_SQUAT: i32 = 0x1e00002b;
    pub const CONT_JUMP_SQUAT_BUTTON: i32 = 0x1e00002c;
    pub const CONT_SQUAT: i32 = 0x1e000034;
    pub const CONT_ESCAPE_AIR: i32 = 0x1e00003d;
    pub const CONT_ATTACK: i32 = 0x1e000052;
    pub const CONT_ATTACK_LW4_START: i32 = 0x1e00005f;
    pub const CONT_SPECIAL_N: i32 = 0x1e000063;
    pub const CONT_SPECIAL_HI: i32 = 0x1e000065;
    pub const CONT_SPECIAL_LW: i32 = 0x1e000066;
    pub const CONT_CATCH: i32 = 0x1e00006f;
    pub const CONT_CATCH_DASH: i32 = 0x1e000070;
    pub const CONT_CATCH_TURN: i32 = 0x1e000071;
}

/// Native work IDs used by the read-only Switch adapter.
pub mod work_ids {
    pub const FIGHTER_INSTANCE_WORK_ID_INT_DOWN_DAMAGE_COUNT: i32 = 0x1000006a;
    pub const FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID: i32 = 0x100000b3;
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME: i32 = 0x13;
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X: i32 = 0x15;
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y: i32 = 0x16;
    pub const FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE: i32 = 0x4c;
    pub const FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME: i32 = 0x1000007;
    pub const FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME: i32 = 0x1100000e;
    pub const FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID: i32 = 0x100000be;
    pub const FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE: i32 = 0x21000016;
    pub const FIGHTER_STATUS_DAMAGE_FLAG_END_REACTION: i32 = 0x2100000d;
    pub const FIGHTER_STATUS_DAMAGE_FLAG_REACTION_CANCEL: i32 = 0x2100000c;
}

// Common FIGHTER_STATUS_KIND values from the pinned native status table.
pub const GUARD_ON: i32 = status::GUARD_ON;
pub const GUARD: i32 = status::GUARD;
pub const GUARD_OFF: i32 = status::GUARD_OFF;
pub const GUARD_DAMAGE: i32 = status::GUARD_DAMAGE;
pub const ESCAPE: i32 = status::ESCAPE;
pub const ESCAPE_F: i32 = status::ESCAPE_F;
pub const ESCAPE_B: i32 = status::ESCAPE_B;
pub const ESCAPE_AIR: i32 = status::ESCAPE_AIR;
pub const ESCAPE_AIR_SLIDE: i32 = status::ESCAPE_AIR_SLIDE;
pub const CATCH: i32 = status::CATCH;
pub const CATCH_PULL: i32 = status::CATCH_PULL;
pub const CATCH_DASH: i32 = status::CATCH_DASH;
pub const CATCH_DASH_PULL: i32 = status::CATCH_DASH_PULL;
pub const CATCH_TURN: i32 = status::CATCH_TURN;
pub const THROW: i32 = status::THROW;
pub const CAPTURE_PULLED: i32 = status::CAPTURE_PULLED;
pub const THROWN: i32 = status::THROWN;

/// Ordinary jump/fall states only: never recovery, tumble, attack or air dodge.
pub fn jump_or_fall(status: i32) -> bool {
    matches!(
        status,
        status::JUMP | status::JUMP_AERIAL | status::FALL | status::FALL_AERIAL
    )
}

/// Special captures/final victims are not ordinary combat targets, even when a
/// transient LinkModule capture link is absent or uses a move-specific link.
pub fn captured_or_scripted(status: i32) -> bool {
    matches!(status,
        status::CAPTURE_PULLED..=status::THROWN
        | status::CLUNG_GANON..=status::SHOULDERED_DONKEY_THROWN
        | status::SWALLOWED..=status::CAPTURE_DAMAGE_YOSHI
        | status::CATCHED_REFLET..=status::CAPTURE_MASTERHAND
        | status::GIMMICK_EATEN
        | status::DEMO..=status::KASEY_WARP
        | status::CLIFF_ROBBED..=status::METAKNIGHT_FINAL_DAMAGE_FALL
        | status::FINAL_VISUAL_ATTACK_OTHER..=status::PSYCHOBREAK
        | status::RYU_FINAL_DAMAGE..=status::RYU_FINAL_DAMAGE_FLY
        | status::CLOUD_FINAL_DAMAGE_FLY..=status::RIDLEY_FINAL_TARGET_END
        | status::CATCHED_RIDLEY..=status::STABBED_DAMAGE
        | status::SWING_GAOGAEN_CATCHED..=status::CAPTURE_WAIT_OCTOPUS
        | status::SIMON_FINAL_TARGET_START..=status::BRAVE_FINAL_TARGET_END
        | status::BUDDY_FINAL_TARGET_START..=status::TANTAN_FINAL_TARGET_END
        | status::CATCHED_PICKEL_TROLLEY..=status::EDGE_FINAL_TARGET_END
        | status::CAPTURE_TRAIL_KEYHOLE..=status::STANDBY
        | status::ENTRY..=status::LOSE
        | status::FINAL)
}

/// Abort existing input tapes on forced states, without disabling native
/// mash/tech safety. Do not resume an old attack after a stun/capture ends.
pub fn sequence_interrupted(status: i32) -> bool {
    captured_or_scripted(status)
        || matches!(status,
        status::REBOUND_STOP..=status::REBOUND_JUMP
        | status::DAMAGE..=status::DOWN_WAIT_CONTINUE
        | status::DOWN_DAMAGE..=status::BIND
        | status::SLIP..=status::SLIP_WAIT
        | status::SWIM_DIVE..=status::REBIRTH
        | status::TREAD_DAMAGE..=status::TREAD_FALL
        | status::BURY..=status::HAMMER_LANDING
        | status::KILLER
        | status::GENESIS_SHOOT_START..=status::GENESIS_GET
        | status::LADDER_CATCH..=status::LADDER_END
        | status::SAVING_DAMAGE..=status::SAVING_DAMAGE_FLY
        | status::SAVING_DAMAGE_AIR
        | status::METAMON_OUT..=status::BOSS_DEAD
        | status::DAMAGE_SLEEP_START..=status::SLEEP_END
        | status::GIMMICK_JUMP_BOARD..=status::DAMAGE_FLY_REFLECT_JUMP_BOARD
        | status::TRAIL_REBOUND)
}

/// These vulnerable bodies may be hit by a configured shield backup, but an
/// ordinary grab must not be chosen for their downed/buried/frozen states.
pub fn ordinary_grab_immune(status: i32) -> bool {
    matches!(
        status,
        status::DOWN..=status::LAY_DOWN
            | status::DOWN_DAMAGE..=status::DOWN_REFLECT_LR
            | status::SHIELD_BREAK_DOWN
            | status::SLIP..=status::SLIP_WAIT
            | status::BURY..=status::ICE_JUMP
    )
}

/// Private status values overlap across fighters. Apply these forced-state
/// meanings only to the native kind that owns them.
pub fn fighter_sequence_interrupted(kind: i32, status: i32) -> bool {
    sequence_interrupted(status)
        || fighter_noncombat(kind, status)
        || kind_statuses::forced_status(kind, status)
}

/// Inactive swap forms and known Final Smash actors are not ordinary punish
/// targets, even through a transient missing capture link.
pub fn fighter_noncombat(kind: i32, status: i32) -> bool {
    kind_statuses::noncombat_status(kind, status)
}
