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
    /// Fighter is standing in their neutral idle state.
    pub const WAIT: i32 = 0x00;
    /// Fighter is walking.
    pub const WALK: i32 = 0x01;
    /// Fighter is decelerating after walking.
    pub const WALK_BRAKE: i32 = 0x02;
    /// Fighter is in the initial dash.
    pub const DASH: i32 = 0x03;
    /// Fighter is running.
    pub const RUN: i32 = 0x04;
    /// Fighter is decelerating after a run.
    pub const RUN_BRAKE: i32 = 0x05;
    /// Fighter is turning around while grounded.
    pub const TURN: i32 = 0x06;
    /// Fighter is reversing direction with an initial dash.
    pub const TURN_DASH: i32 = 0x07;
    /// Fighter is turning around while running.
    pub const TURN_RUN: i32 = 0x08;
    /// Fighter is braking after turning during a run.
    pub const TURN_RUN_BRAKE: i32 = 0x09;
    /// Fighter is in jumpsquat before leaving the ground.
    pub const JUMP_SQUAT: i32 = 0x0a;
    /// Fighter is airborne from a ground jump.
    pub const JUMP: i32 = 0x0b;
    /// Fighter is airborne from a midair jump.
    pub const JUMP_AERIAL: i32 = 0x0c;
    /// Fighter is in an ordinary airborne fall.
    pub const FALL: i32 = 0x0e;
    /// Fighter is falling after a midair jump.
    pub const FALL_AERIAL: i32 = 0x0f;
    /// Fighter is in special fall and usually cannot act.
    pub const FALL_SPECIAL: i32 = 0x10;
    /// Fighter is entering a crouch.
    pub const SQUAT: i32 = 0x11;
    /// Fighter is holding a crouch.
    pub const SQUAT_WAIT: i32 = 0x12;
    /// Fighter is leaving crouch while moving forward.
    pub const SQUAT_F: i32 = 0x13;
    /// Fighter is leaving crouch while moving backward.
    pub const SQUAT_B: i32 = 0x14;
    /// Fighter is standing up from crouch.
    pub const SQUAT_RV: i32 = 0x15;
    /// Fighter is in normal landing lag.
    pub const LANDING: i32 = 0x16;
    /// Fighter is completing a light landing with little or no lag.
    pub const LANDING_LIGHT: i32 = 0x17;
    /// Fighter is in an aerial attack's landing lag.
    pub const LANDING_ATTACK_AIR: i32 = 0x18;
    /// Fighter is landing from special fall.
    pub const LANDING_FALL_SPECIAL: i32 = 0x19;
    /// Fighter is lightly landing while in a damage reaction.
    pub const LANDING_DAMAGE_LIGHT: i32 = 0x1a;
    /// Fighter is raising their shield.
    pub const GUARD_ON: i32 = 0x1b;
    /// Fighter is holding shield.
    pub const GUARD: i32 = 0x1c;
    /// Fighter is dropping their shield.
    pub const GUARD_OFF: i32 = 0x1d;
    /// Fighter's shield has just been struck and is in shieldstun.
    pub const GUARD_DAMAGE: i32 = 0x1e;
    /// Fighter is performing a spot dodge.
    pub const ESCAPE: i32 = 0x1f;
    /// Fighter is rolling forward.
    pub const ESCAPE_F: i32 = 0x20;
    /// Fighter is rolling backward.
    pub const ESCAPE_B: i32 = 0x21;
    /// Fighter is performing a neutral air dodge.
    pub const ESCAPE_AIR: i32 = 0x22;
    /// Fighter is performing a directional air dodge.
    pub const ESCAPE_AIR_SLIDE: i32 = 0x23;
    /// Fighter is frozen at the start of a rebound or grab-clank reaction.
    pub const REBOUND_STOP: i32 = 0x24;
    /// Fighter is recoiling after a grab or tether rebounds.
    pub const REBOUND: i32 = 0x25;
    /// Fighter is leaving a rebound reaction with a jump.
    pub const REBOUND_JUMP: i32 = 0x26;
    /// Fighter is performing a neutral attack or jab.
    pub const ATTACK: i32 = 0x27;
    /// Fighter is performing a rapid jab.
    pub const ATTACK_100: i32 = 0x28;
    /// Fighter is performing a dash attack.
    pub const ATTACK_DASH: i32 = 0x29;
    /// Fighter is performing a forward tilt.
    pub const ATTACK_S3: i32 = 0x2a;
    /// Fighter is performing an up tilt.
    pub const ATTACK_HI3: i32 = 0x2b;
    /// Fighter is performing a down tilt.
    pub const ATTACK_LW3: i32 = 0x2c;
    /// Fighter is starting a forward smash.
    pub const ATTACK_S4_START: i32 = 0x2d;
    /// Fighter is charging a forward smash.
    pub const ATTACK_S4_HOLD: i32 = 0x2e;
    /// Fighter is releasing a forward smash.
    pub const ATTACK_S4: i32 = 0x2f;
    /// Fighter is starting a down smash.
    pub const ATTACK_LW4_START: i32 = 0x30;
    /// Fighter is charging a down smash.
    pub const ATTACK_LW4_HOLD: i32 = 0x31;
    /// Fighter is releasing a down smash.
    pub const ATTACK_LW4: i32 = 0x32;
    /// Fighter is starting an up smash.
    pub const ATTACK_HI4_START: i32 = 0x33;
    /// Fighter is charging an up smash.
    pub const ATTACK_HI4_HOLD: i32 = 0x34;
    /// Fighter is releasing an up smash.
    pub const ATTACK_HI4: i32 = 0x35;
    /// Fighter is performing an aerial attack.
    pub const ATTACK_AIR: i32 = 0x36;
    /// Fighter is attempting a standing grab.
    pub const CATCH: i32 = 0x37;
    /// Fighter is pulling in an opponent caught by a standing grab.
    pub const CATCH_PULL: i32 = 0x38;
    /// Fighter is attempting a dash grab.
    pub const CATCH_DASH: i32 = 0x39;
    /// Fighter is pulling in an opponent caught by a dash grab.
    pub const CATCH_DASH_PULL: i32 = 0x3a;
    /// Fighter is attempting a pivot grab.
    pub const CATCH_TURN: i32 = 0x3b;
    /// Fighter is holding a grabbed opponent.
    pub const CATCH_WAIT: i32 = 0x3c;
    /// Fighter is pummeling a grabbed opponent.
    pub const CATCH_ATTACK: i32 = 0x3d;
    /// Fighter's grab has been released or broken.
    pub const CATCH_CUT: i32 = 0x3e;
    /// Fighter is jumping while maintaining a special capture.
    pub const CATCH_JUMP: i32 = 0x3f;
    /// Fighter is performing a throw.
    pub const THROW: i32 = 0x40;
    /// Fighter is being pulled into an opponent's grab.
    pub const CAPTURE_PULLED: i32 = 0x41;
    /// Fighter is being held by an opponent's grab.
    pub const CAPTURE_WAIT: i32 = 0x42;
    /// Fighter is taking damage while captured, such as from a pummel.
    pub const CAPTURE_DAMAGE: i32 = 0x43;
    /// Fighter is being released from a grab.
    pub const CAPTURE_CUT: i32 = 0x44;
    /// Fighter is being carried through a jumping capture.
    pub const CAPTURE_JUMP: i32 = 0x45;
    /// Fighter is in the victim state of a throw.
    pub const THROWN: i32 = 0x46;
    /// Fighter is in grounded hitstun.
    pub const DAMAGE: i32 = 0x47;
    /// Fighter is in airborne hitstun without tumble.
    pub const DAMAGE_AIR: i32 = 0x48;
    /// Fighter is launched into tumble.
    pub const DAMAGE_FLY: i32 = 0x49;
    /// Fighter is launched in a rolling tumble animation.
    pub const DAMAGE_FLY_ROLL: i32 = 0x4a;
    /// Fighter is launched downward by a meteor smash.
    pub const DAMAGE_FLY_METEOR: i32 = 0x4b;
    /// Fighter has bounced from a wall after being launched.
    pub const DAMAGE_FLY_REFLECT_LR: i32 = 0x4c;
    /// Fighter has bounced from a ceiling after being launched.
    pub const DAMAGE_FLY_REFLECT_U: i32 = 0x4d;
    /// Fighter has bounced from the ground after being launched.
    pub const DAMAGE_FLY_REFLECT_D: i32 = 0x4e;
    /// Fighter is falling in tumble after hitstun.
    pub const DAMAGE_FALL: i32 = 0x4f;
    /// Fighter has just entered a knocked-down state.
    pub const DOWN: i32 = 0x50;
    /// Fighter is settling onto the floor after knockdown.
    pub const DOWN_SPOT: i32 = 0x51;
    /// Fighter's knockdown animation is continuing.
    pub const DOWN_CONTINUE: i32 = 0x52;
    /// Fighter is lying prone after being knocked down.
    pub const DOWN_WAIT: i32 = 0x53;
    /// Fighter remains in the prone wait state.
    pub const DOWN_WAIT_CONTINUE: i32 = 0x54;
    /// Fighter is prone and interacting with nearby food.
    pub const DOWN_EAT: i32 = 0x55;
    /// Fighter is lying face-up or face-down on the stage.
    pub const LAY_DOWN: i32 = 0x56;
    /// Fighter is performing a neutral get-up.
    pub const DOWN_STAND: i32 = 0x57;
    /// Fighter is performing a get-up roll.
    pub const DOWN_STAND_FB: i32 = 0x58;
    /// Fighter is performing a get-up attack.
    pub const DOWN_STAND_ATTACK: i32 = 0x59;
    /// Fighter is taking damage while downed.
    pub const DOWN_DAMAGE: i32 = 0x5a;
    /// Fighter has bounced sideways while downed.
    pub const DOWN_REFLECT_LR: i32 = 0x5b;
    /// Fighter is launched upward by a shield break.
    pub const SHIELD_BREAK_FLY: i32 = 0x5c;
    /// Fighter is falling after the shield-break launch.
    pub const SHIELD_BREAK_FALL: i32 = 0x5d;
    /// Fighter has landed after a shield break.
    pub const SHIELD_BREAK_DOWN: i32 = 0x5e;
    /// Fighter is entering the shield-break dizzy state.
    pub const FURAFURA_STAND: i32 = 0x5f;
    /// Fighter is stunned and wobbling after a shield break.
    pub const FURAFURA: i32 = 0x60;
    /// Fighter is recovering from shield-break stun.
    pub const FURAFURA_END: i32 = 0x61;
    /// Fighter is entering a sleep-inducing song reaction.
    pub const DAMAGE_SONG_START: i32 = 0x62;
    /// Fighter is asleep from a song effect such as Sing.
    pub const DAMAGE_SONG: i32 = 0x63;
    /// Fighter is falling while affected by a sleep-inducing song.
    pub const DAMAGE_SONG_FALL: i32 = 0x64;
    /// Fighter is waking from a song-induced sleep.
    pub const DAMAGE_SONG_END: i32 = 0x65;
    /// Fighter is immobilized by a binding effect.
    pub const BIND: i32 = 0x66;
    /// Fighter is performing a neutral floor tech.
    pub const PASSIVE: i32 = 0x67;
    /// Fighter is performing a forward or backward tech roll.
    pub const PASSIVE_FB: i32 = 0x68;
    /// Fighter is teching against a wall.
    pub const PASSIVE_WALL: i32 = 0x69;
    /// Fighter is wall-teching with a jump.
    pub const PASSIVE_WALL_JUMP: i32 = 0x6a;
    /// Fighter is teching against a ceiling.
    pub const PASSIVE_CEIL: i32 = 0x6b;
    /// Fighter has collided with a wall without teching.
    pub const STOP_WALL: i32 = 0x6c;
    /// Fighter has collided with a ceiling without teching.
    pub const STOP_CEIL: i32 = 0x6d;
    /// Fighter is performing a wall jump.
    pub const WALL_JUMP: i32 = 0x6e;
    /// Fighter is moving into position to grab a ledge.
    pub const CLIFF_CATCH_MOVE: i32 = 0x75;
    /// Fighter has just caught a ledge.
    pub const CLIFF_CATCH: i32 = 0x76;
    /// Fighter is hanging from a ledge.
    pub const CLIFF_WAIT: i32 = 0x77;
    /// Fighter is performing a ledge attack.
    pub const CLIFF_ATTACK: i32 = 0x78;
    /// Fighter is performing a neutral get-up from the ledge.
    pub const CLIFF_CLIMB: i32 = 0x79;
    /// Fighter is rolling onto the stage from the ledge.
    pub const CLIFF_ESCAPE: i32 = 0x7a;
    /// Fighter is beginning a ledge jump.
    pub const CLIFF_JUMP1: i32 = 0x7b;
    /// Fighter is in the airborne phase of a ledge jump.
    pub const CLIFF_JUMP2: i32 = 0x7c;
    /// Fighter is finishing a ledge jump.
    pub const CLIFF_JUMP3: i32 = 0x7d;
    /// Fighter is entering the teeter animation at a platform edge.
    pub const OTTOTTO: i32 = 0x7e;
    /// Fighter is teetering at a platform edge.
    pub const OTTOTTO_WAIT: i32 = 0x7f;
    /// Fighter has tripped and is falling down.
    pub const SLIP: i32 = 0x86;
    /// Fighter is taking damage during a trip.
    pub const SLIP_DAMAGE: i32 = 0x87;
    /// Fighter is lying down after a trip.
    pub const SLIP_WAIT: i32 = 0x88;
    /// Fighter is submerged or drowning in swimmable water.
    pub const SWIM_DIVE: i32 = 0xab;
    /// Fighter has been footstooled.
    pub const TREAD_DAMAGE: i32 = 0xb9;
    /// Fighter is falling after a footstool.
    pub const TREAD_FALL: i32 = 0xbc;
    /// Fighter is held by Ganondorf's command grab.
    pub const CLUNG_GANON: i32 = 0xbd;
    /// Fighter is being thrown from Donkey Kong's cargo carry.
    pub const SHOULDERED_DONKEY_THROWN: i32 = 0xc7;
    /// Fighter is buried in the ground.
    pub const BURY: i32 = 0xc8;
    /// Fighter is landing while wielding the Hammer item.
    pub const HAMMER_LANDING: i32 = 0xdf;
    /// Fighter is braking a forward walk while firing a shooting item.
    pub const ITEM_SHOOT_WALK_BRAKE_F: i32 = 0x9d;
    /// Fighter is KO'd and awaiting the next stock or result.
    pub const DEAD: i32 = 0xb5;
    /// Fighter is on the respawn platform with temporary invincibility.
    pub const REBIRTH: i32 = 0xb6;
    /// Fighter is caught by Ganondorf's grounded Flame Choke.
    pub const CATCHED_GANON: i32 = 0xbe;
    /// Fighter is caught by Ganondorf's aerial Flame Choke.
    pub const CATCHED_AIR_GANON: i32 = 0xbf;
    /// Fighter is falling while held by aerial Flame Choke.
    pub const CATCHED_AIR_FALL_GANON: i32 = 0xc0;
    /// Fighter is released at the end of aerial Flame Choke.
    pub const CATCHED_AIR_END_GANON: i32 = 0xc1;
    /// Fighter is breaking free from a frozen state.
    pub const ICE_JUMP: i32 = 0xcc;
    /// Fighter has been swallowed by Kirby or King Dedede.
    pub const SWALLOWED: i32 = 0xe6;
    /// Fighter is trapped and taking damage inside Yoshi's egg.
    pub const CAPTURE_DAMAGE_YOSHI: i32 = 0xf7;
    /// Fighter is caught by Robin's Nosferatu.
    pub const CATCHED_REFLET: i32 = 0xfe;
    /// Fighter is captured by Master Hand.
    pub const CAPTURE_MASTERHAND: i32 = 0xff;
    /// Fighter is being launched by a Bullet Bill transformation.
    pub const KILLER: i32 = 0x100;
    /// Fighter has been eaten by a stage gimmick.
    pub const GIMMICK_EATEN: i32 = 0x102;
    /// Fighter is entering the launch sequence for the Genesis item.
    pub const GENESIS_SHOOT_START: i32 = 0x103;
    /// Fighter is being caught by the Genesis item's effect.
    pub const GENESIS_GET: i32 = 0x106;
    /// Fighter is controlled by a scripted cinematic sequence.
    pub const DEMO: i32 = 0x107;
    /// Fighter is being warped by a stage or scripted effect.
    pub const KASEY_WARP: i32 = 0x110;
    /// Fighter is grabbing onto a ladder.
    pub const LADDER_CATCH: i32 = 0x111;
    /// Fighter is leaving a ladder.
    pub const LADDER_END: i32 = 0x115;
    /// Fighter has lost the ledge because another fighter took it.
    pub const CLIFF_ROBBED: i32 = 0x116;
    /// Fighter is falling as a victim of Meta Knight's Final Smash.
    pub const METAKNIGHT_FINAL_DAMAGE_FALL: i32 = 0x119;
    /// Fighter is a victim in another fighter's cinematic Final Smash.
    pub const FINAL_VISUAL_ATTACK_OTHER: i32 = 0x11f;
    /// Fighter is caught in Mewtwo's Psystrike Final Smash.
    pub const PSYCHOBREAK: i32 = 0x147;
    /// Fighter is taking grounded damage in a scripted saving/cinematic sequence.
    pub const SAVING_DAMAGE: i32 = 0x148;
    /// Fighter is launched during a scripted saving/cinematic sequence.
    pub const SAVING_DAMAGE_FLY: i32 = 0x149;
    /// Fighter is caught in Ryu's grounded Final Smash sequence.
    pub const RYU_FINAL_DAMAGE: i32 = 0x14a;
    /// Fighter is launched by Ryu's Final Smash sequence.
    pub const RYU_FINAL_DAMAGE_FLY: i32 = 0x14b;
    /// Fighter is taking airborne damage in a scripted saving/cinematic sequence.
    pub const SAVING_DAMAGE_AIR: i32 = 0x14c;
    /// Fighter is launched as a victim of Cloud's Final Smash.
    pub const CLOUD_FINAL_DAMAGE_FLY: i32 = 0x14d;
    /// Fighter is leaving Ridley's Final Smash victim sequence.
    pub const RIDLEY_FINAL_TARGET_END: i32 = 0x16e;
    /// Fighter is caught by Ridley's Space Pirate Rush.
    pub const CATCHED_RIDLEY: i32 = 0x171;
    /// Fighter is pinned and taking damage from a stabbing capture.
    pub const STABBED_DAMAGE: i32 = 0x174;
    /// Fighter is caught in Incineroar's Alolan Whip.
    pub const SWING_GAOGAEN_CATCHED: i32 = 0x176;
    /// Fighter is held by the Octopus transformation.
    pub const CAPTURE_WAIT_OCTOPUS: i32 = 0x18c;
    /// Fighter is entering Simon or Richter's Final Smash victim sequence.
    pub const SIMON_FINAL_TARGET_START: i32 = 0x18f;
    /// Fighter is leaving a Ditto-related transformation or scripted state.
    pub const METAMON_OUT: i32 = 0x196;
    /// Fighter is in a boss-style defeated state.
    pub const BOSS_DEAD: i32 = 0x198;
    /// Fighter is leaving Hero's Final Smash victim sequence.
    pub const BRAVE_FINAL_TARGET_END: i32 = 0x1a2;
    /// Fighter is beginning a sleep reaction.
    pub const DAMAGE_SLEEP_START: i32 = 0x1a3;
    /// Fighter is waking from sleep.
    pub const SLEEP_END: i32 = 0x1aa;
    /// Fighter is entering Banjo & Kazooie's Final Smash victim sequence.
    pub const BUDDY_FINAL_TARGET_START: i32 = 0x1ab;
    /// Fighter is leaving Min Min's Final Smash victim sequence.
    pub const TANTAN_FINAL_TARGET_END: i32 = 0x1ba;
    /// Fighter is being launched by a stage jump board.
    pub const GIMMICK_JUMP_BOARD: i32 = 0x1bb;
    /// Fighter is bouncing from a jump board while launched.
    pub const DAMAGE_FLY_REFLECT_JUMP_BOARD: i32 = 0x1bd;
    /// Fighter is trapped in Steve's minecart.
    pub const CATCHED_PICKEL_TROLLEY: i32 = 0x1be;
    /// Fighter is leaving Sephiroth's Final Smash victim sequence.
    pub const EDGE_FINAL_TARGET_END: i32 = 0x1d0;
    /// Fighter is recoiling from Sora's move-specific rebound.
    pub const TRAIL_REBOUND: i32 = 0x1d1;
    /// Fighter is captured in Sora's Keyhole Final Smash sequence.
    pub const CAPTURE_TRAIL_KEYHOLE: i32 = 0x1d2;
    /// Fighter is inactive and waiting outside ordinary gameplay.
    pub const STANDBY: i32 = 0x1d6;
    /// Fighter is playing their match-entry animation.
    pub const ENTRY: i32 = 0x1d9;
    /// Fighter is in a loss/result state.
    pub const LOSE: i32 = 0x1db;
    /// Fighter is performing a neutral special.
    pub const SPECIAL_N: i32 = 0x1dc;
    /// Fighter is performing a side special.
    pub const SPECIAL_S: i32 = 0x1dd;
    /// Fighter is performing an up special.
    pub const SPECIAL_HI: i32 = 0x1de;
    /// Fighter is performing a down special.
    pub const SPECIAL_LW: i32 = 0x1df;
    /// Fighter is performing a Final Smash.
    pub const FINAL: i32 = 0x1e0;
}

/// Ground-touch masks from the pinned `skyline-smash` lua_const table.
pub mod ground_touch {
    /// Collision mask for surfaces touching the fighter's lower or side sensors.
    pub const DOWN_SIDE: u32 = 0xc8;
    /// Collision mask for surfaces touching the fighter's left or side sensors.
    pub const LEFT_SIDE: u32 = 0x52;
    /// Collision mask for surfaces touching the fighter's right or side sensors.
    pub const RIGHT_SIDE: u32 = 0xa4;
    /// Collision mask for surfaces touching the fighter's upper or side sensors.
    pub const UP_SIDE: u32 = 0x31;
}

/// `SITUATION_KIND_*` values from the pinned `skyline-smash` lua_const table.
pub mod situation {
    /// Fighter is standing on the stage or another solid surface.
    pub const GROUND: i32 = 0;
    /// Fighter is attached to a ledge.
    pub const CLIFF: i32 = 1;
    /// Fighter is airborne.
    pub const AIR: i32 = 2;
    /// Adapter could not read a valid native situation for the fighter.
    pub const UNAVAILABLE: i32 = -1;

    // AttackData.situation is a bitmask, not a SITUATION_KIND value.
    /// Attack hitbox can connect with grounded targets.
    pub const ATTACK_GROUND_MASK: u16 = 1;
    /// Attack hitbox can connect with airborne targets.
    pub const ATTACK_AIR_MASK: u16 = 2;
}

/// Native transition-term IDs from the pinned `skyline-smash` constant table.
pub mod transition_terms {
    /// Native transition permission for a neutral floor tech.
    pub const PASSIVE: i32 = 0x1e000013;
    /// Native transition permission for a forward or backward tech roll.
    pub const PASSIVE_FB: i32 = 0x1e000014;
    /// Native transition permission for a wall tech.
    pub const PASSIVE_WALL: i32 = 0x1e000015;
    /// Native transition permission for a ceiling tech.
    pub const PASSIVE_CEIL: i32 = 0x1e000018;
    /// Native transition permission to continue into a spot dodge.
    pub const CONT_ESCAPE: i32 = 0x1e000026;
    /// Native transition permission to raise shield.
    pub const CONT_GUARD_ON: i32 = 0x1e000029;
    /// Native transition permission to begin jumpsquat from stick input.
    pub const CONT_JUMP_SQUAT: i32 = 0x1e00002b;
    /// Native transition permission to begin jumpsquat from a jump button.
    pub const CONT_JUMP_SQUAT_BUTTON: i32 = 0x1e00002c;
    /// Native transition permission to continue into crouch.
    pub const CONT_SQUAT: i32 = 0x1e000034;
    /// Native transition permission to perform an air dodge.
    pub const CONT_ESCAPE_AIR: i32 = 0x1e00003d;
    /// Native transition permission to perform a neutral attack.
    pub const CONT_ATTACK: i32 = 0x1e000052;
    /// Native transition permission to start a down smash.
    pub const CONT_ATTACK_LW4_START: i32 = 0x1e00005f;
    /// Native transition permission to perform a neutral special.
    pub const CONT_SPECIAL_N: i32 = 0x1e000063;
    /// Native transition permission to perform an up special.
    pub const CONT_SPECIAL_HI: i32 = 0x1e000065;
    /// Native transition permission to perform a down special.
    pub const CONT_SPECIAL_LW: i32 = 0x1e000066;
    /// Native transition permission to perform a standing grab.
    pub const CONT_CATCH: i32 = 0x1e00006f;
    /// Native transition permission to perform a dash grab.
    pub const CONT_CATCH_DASH: i32 = 0x1e000070;
    /// Native transition permission to perform a pivot grab.
    pub const CONT_CATCH_TURN: i32 = 0x1e000071;
}

/// Native work IDs used by the read-only Switch adapter.
pub mod work_ids {
    /// Work integer storing repeated damage received while the fighter is downed.
    pub const FIGHTER_INSTANCE_WORK_ID_INT_DOWN_DAMAGE_COUNT: i32 = 0x1000006a;
    /// Work integer identifying the fighter credited as the latest successful attacker.
    pub const FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID: i32 = 0x100000b3;
    /// Work float storing remaining frames of the current damage reaction.
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME: i32 = 0x13;
    /// Work float storing reserved horizontal launch speed from damage.
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X: i32 = 0x15;
    /// Work float storing reserved vertical launch speed from damage.
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y: i32 = 0x16;
    /// Mr. Game & Watch work float storing Oil Panic's absorbed-power gauge.
    pub const FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE: i32 = 0x4c;
    /// Ledge-status work float tracking normal ledge invincibility timing.
    pub const FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME: i32 = 0x1000007;
    /// Shield-damage work integer storing remaining shieldstun frames.
    pub const FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME: i32 = 0x1100000e;
    /// Ice Climbers work integer identifying the partner's battle object.
    pub const FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID: i32 = 0x100000be;
    /// Damage-state flag indicating that launch tumble cannot be teched.
    pub const FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE: i32 = 0x21000016;
    /// Damage-state flag indicating that the forced damage reaction has ended.
    pub const FIGHTER_STATUS_DAMAGE_FLAG_END_REACTION: i32 = 0x2100000d;
    /// Damage-state flag indicating that the damage reaction may be canceled.
    pub const FIGHTER_STATUS_DAMAGE_FLAG_REACTION_CANCEL: i32 = 0x2100000c;
}

// Common FIGHTER_STATUS_KIND values from the pinned native status table.
/// Fighter is raising their shield.
pub const GUARD_ON: i32 = status::GUARD_ON;
/// Fighter is holding shield.
pub const GUARD: i32 = status::GUARD;
/// Fighter is dropping their shield.
pub const GUARD_OFF: i32 = status::GUARD_OFF;
/// Fighter's shield has just been struck and is in shieldstun.
pub const GUARD_DAMAGE: i32 = status::GUARD_DAMAGE;
/// Fighter is performing a spot dodge.
pub const ESCAPE: i32 = status::ESCAPE;
/// Fighter is rolling forward.
pub const ESCAPE_F: i32 = status::ESCAPE_F;
/// Fighter is rolling backward.
pub const ESCAPE_B: i32 = status::ESCAPE_B;
/// Fighter is performing a neutral air dodge.
pub const ESCAPE_AIR: i32 = status::ESCAPE_AIR;
/// Fighter is performing a directional air dodge.
pub const ESCAPE_AIR_SLIDE: i32 = status::ESCAPE_AIR_SLIDE;
/// Fighter is attempting a standing grab.
pub const CATCH: i32 = status::CATCH;
/// Fighter is pulling in an opponent caught by a standing grab.
pub const CATCH_PULL: i32 = status::CATCH_PULL;
/// Fighter is attempting a dash grab.
pub const CATCH_DASH: i32 = status::CATCH_DASH;
/// Fighter is pulling in an opponent caught by a dash grab.
pub const CATCH_DASH_PULL: i32 = status::CATCH_DASH_PULL;
/// Fighter is attempting a pivot grab.
pub const CATCH_TURN: i32 = status::CATCH_TURN;
/// Fighter is performing a throw.
pub const THROW: i32 = status::THROW;
/// Fighter is being pulled into an opponent's grab.
pub const CAPTURE_PULLED: i32 = status::CAPTURE_PULLED;
/// Fighter is in the victim state of a throw.
pub const THROWN: i32 = status::THROWN;

/// Whether the fighter is in an ordinary jump or fall, excluding recovery,
/// tumble, attacks, and air dodges.
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
