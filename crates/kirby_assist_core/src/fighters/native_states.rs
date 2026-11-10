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
    /// Defines the `WAIT` constant used by this module.
    ///
    ///
    ///  Provides the WAIT constant value.
    pub const WAIT: i32 = 0x00;
    /// Defines the `WALK` constant used by this module.
    ///
    ///
    ///  Provides the WALK constant value.
    pub const WALK: i32 = 0x01;
    /// Defines the `WALK_BRAKE` constant used by this module.
    ///
    ///
    ///  Provides the WALK BRAKE constant value.
    pub const WALK_BRAKE: i32 = 0x02;
    /// Defines the `DASH` constant used by this module.
    ///
    ///
    ///  Provides the DASH constant value.
    pub const DASH: i32 = 0x03;
    /// Defines the `RUN` constant used by this module.
    ///
    ///
    ///  Provides the RUN constant value.
    pub const RUN: i32 = 0x04;
    /// Defines the `RUN_BRAKE` constant used by this module.
    ///
    ///
    ///  Provides the RUN BRAKE constant value.
    pub const RUN_BRAKE: i32 = 0x05;
    /// Defines the `TURN` constant used by this module.
    ///
    ///
    ///  Provides the TURN constant value.
    pub const TURN: i32 = 0x06;
    /// Defines the `TURN_DASH` constant used by this module.
    ///
    ///
    ///  Provides the TURN DASH constant value.
    pub const TURN_DASH: i32 = 0x07;
    /// Defines the `TURN_RUN` constant used by this module.
    ///
    ///
    ///  Provides the TURN RUN constant value.
    pub const TURN_RUN: i32 = 0x08;
    /// Defines the `TURN_RUN_BRAKE` constant used by this module.
    ///
    ///
    ///  Provides the TURN RUN BRAKE constant value.
    pub const TURN_RUN_BRAKE: i32 = 0x09;
    /// Defines the `JUMP_SQUAT` constant used by this module.
    ///
    ///
    ///  Provides the JUMP SQUAT constant value.
    pub const JUMP_SQUAT: i32 = 0x0a;
    /// Defines the `JUMP` constant used by this module.
    ///
    ///
    ///  Provides the JUMP constant value.
    pub const JUMP: i32 = 0x0b;
    /// Defines the `JUMP_AERIAL` constant used by this module.
    ///
    ///
    ///  Provides the JUMP AERIAL constant value.
    pub const JUMP_AERIAL: i32 = 0x0c;
    /// Defines the `FALL` constant used by this module.
    ///
    ///
    ///  Provides the FALL constant value.
    pub const FALL: i32 = 0x0e;
    /// Defines the `FALL_AERIAL` constant used by this module.
    ///
    ///
    ///  Provides the FALL AERIAL constant value.
    pub const FALL_AERIAL: i32 = 0x0f;
    /// Defines the `FALL_SPECIAL` constant used by this module.
    ///
    ///
    ///  Provides the FALL SPECIAL constant value.
    pub const FALL_SPECIAL: i32 = 0x10;
    /// Defines the `SQUAT` constant used by this module.
    ///
    ///
    ///  Provides the SQUAT constant value.
    pub const SQUAT: i32 = 0x11;
    /// Defines the `SQUAT_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the SQUAT WAIT constant value.
    pub const SQUAT_WAIT: i32 = 0x12;
    /// Defines the `SQUAT_F` constant used by this module.
    ///
    ///
    ///  Provides the SQUAT F constant value.
    pub const SQUAT_F: i32 = 0x13;
    /// Defines the `SQUAT_B` constant used by this module.
    ///
    ///
    ///  Provides the SQUAT B constant value.
    pub const SQUAT_B: i32 = 0x14;
    /// Defines the `SQUAT_RV` constant used by this module.
    ///
    ///
    ///  Provides the SQUAT RV constant value.
    pub const SQUAT_RV: i32 = 0x15;
    /// Defines the `LANDING` constant used by this module.
    ///
    ///
    ///  Provides the LANDING constant value.
    pub const LANDING: i32 = 0x16;
    /// Defines the `LANDING_LIGHT` constant used by this module.
    ///
    ///
    ///  Provides the LANDING LIGHT constant value.
    pub const LANDING_LIGHT: i32 = 0x17;
    /// Defines the `LANDING_ATTACK_AIR` constant used by this module.
    ///
    ///
    ///  Provides the LANDING ATTACK AIR constant value.
    pub const LANDING_ATTACK_AIR: i32 = 0x18;
    /// Defines the `LANDING_FALL_SPECIAL` constant used by this module.
    ///
    ///
    ///  Provides the LANDING FALL SPECIAL constant value.
    pub const LANDING_FALL_SPECIAL: i32 = 0x19;
    /// Defines the `LANDING_DAMAGE_LIGHT` constant used by this module.
    ///
    ///
    ///  Provides the LANDING DAMAGE LIGHT constant value.
    pub const LANDING_DAMAGE_LIGHT: i32 = 0x1a;
    /// Defines the `GUARD_ON` constant used by this module.
    ///
    ///
    ///  Provides the GUARD ON constant value.
    pub const GUARD_ON: i32 = 0x1b;
    /// Defines the `GUARD` constant used by this module.
    ///
    ///
    ///  Provides the GUARD constant value.
    pub const GUARD: i32 = 0x1c;
    /// Defines the `GUARD_OFF` constant used by this module.
    ///
    ///
    ///  Provides the GUARD OFF constant value.
    pub const GUARD_OFF: i32 = 0x1d;
    /// Defines the `GUARD_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the GUARD DAMAGE constant value.
    pub const GUARD_DAMAGE: i32 = 0x1e;
    /// Defines the `ESCAPE` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE constant value.
    pub const ESCAPE: i32 = 0x1f;
    /// Defines the `ESCAPE_F` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE F constant value.
    pub const ESCAPE_F: i32 = 0x20;
    /// Defines the `ESCAPE_B` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE B constant value.
    pub const ESCAPE_B: i32 = 0x21;
    /// Defines the `ESCAPE_AIR` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE AIR constant value.
    pub const ESCAPE_AIR: i32 = 0x22;
    /// Defines the `ESCAPE_AIR_SLIDE` constant used by this module.
    ///
    ///
    ///  Provides the ESCAPE AIR SLIDE constant value.
    pub const ESCAPE_AIR_SLIDE: i32 = 0x23;
    /// Defines the `REBOUND_STOP` constant used by this module.
    ///
    ///
    ///  Provides the REBOUND STOP constant value.
    pub const REBOUND_STOP: i32 = 0x24;
    /// Defines the `REBOUND` constant used by this module.
    ///
    ///
    ///  Provides the REBOUND constant value.
    pub const REBOUND: i32 = 0x25;
    /// Defines the `REBOUND_JUMP` constant used by this module.
    ///
    ///
    ///  Provides the REBOUND JUMP constant value.
    pub const REBOUND_JUMP: i32 = 0x26;
    /// Defines the `ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK constant value.
    pub const ATTACK: i32 = 0x27;
    /// Defines the `ATTACK_100` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK 100 constant value.
    pub const ATTACK_100: i32 = 0x28;
    /// Defines the `ATTACK_DASH` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK DASH constant value.
    pub const ATTACK_DASH: i32 = 0x29;
    /// Defines the `ATTACK_S3` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK S3 constant value.
    pub const ATTACK_S3: i32 = 0x2a;
    /// Defines the `ATTACK_HI3` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK HI3 constant value.
    pub const ATTACK_HI3: i32 = 0x2b;
    /// Defines the `ATTACK_LW3` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK LW3 constant value.
    pub const ATTACK_LW3: i32 = 0x2c;
    /// Defines the `ATTACK_S4_START` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK S4 START constant value.
    pub const ATTACK_S4_START: i32 = 0x2d;
    /// Defines the `ATTACK_S4_HOLD` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK S4 HOLD constant value.
    pub const ATTACK_S4_HOLD: i32 = 0x2e;
    /// Defines the `ATTACK_S4` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK S4 constant value.
    pub const ATTACK_S4: i32 = 0x2f;
    /// Defines the `ATTACK_LW4_START` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK LW4 START constant value.
    pub const ATTACK_LW4_START: i32 = 0x30;
    /// Defines the `ATTACK_LW4_HOLD` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK LW4 HOLD constant value.
    pub const ATTACK_LW4_HOLD: i32 = 0x31;
    /// Defines the `ATTACK_LW4` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK LW4 constant value.
    pub const ATTACK_LW4: i32 = 0x32;
    /// Defines the `ATTACK_HI4_START` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK HI4 START constant value.
    pub const ATTACK_HI4_START: i32 = 0x33;
    /// Defines the `ATTACK_HI4_HOLD` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK HI4 HOLD constant value.
    pub const ATTACK_HI4_HOLD: i32 = 0x34;
    /// Defines the `ATTACK_HI4` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK HI4 constant value.
    pub const ATTACK_HI4: i32 = 0x35;
    /// Defines the `ATTACK_AIR` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK AIR constant value.
    pub const ATTACK_AIR: i32 = 0x36;
    /// Defines the `CATCH` constant used by this module.
    ///
    ///
    ///  Provides the CATCH constant value.
    pub const CATCH: i32 = 0x37;
    /// Defines the `CATCH_PULL` constant used by this module.
    ///
    ///
    ///  Provides the CATCH PULL constant value.
    pub const CATCH_PULL: i32 = 0x38;
    /// Defines the `CATCH_DASH` constant used by this module.
    ///
    ///
    ///  Provides the CATCH DASH constant value.
    pub const CATCH_DASH: i32 = 0x39;
    /// Defines the `CATCH_DASH_PULL` constant used by this module.
    ///
    ///
    ///  Provides the CATCH DASH PULL constant value.
    pub const CATCH_DASH_PULL: i32 = 0x3a;
    /// Defines the `CATCH_TURN` constant used by this module.
    ///
    ///
    ///  Provides the CATCH TURN constant value.
    pub const CATCH_TURN: i32 = 0x3b;
    /// Defines the `CATCH_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the CATCH WAIT constant value.
    pub const CATCH_WAIT: i32 = 0x3c;
    /// Defines the `CATCH_ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the CATCH ATTACK constant value.
    pub const CATCH_ATTACK: i32 = 0x3d;
    /// Defines the `CATCH_CUT` constant used by this module.
    ///
    ///
    ///  Provides the CATCH CUT constant value.
    pub const CATCH_CUT: i32 = 0x3e;
    /// Defines the `CATCH_JUMP` constant used by this module.
    ///
    ///
    ///  Provides the CATCH JUMP constant value.
    pub const CATCH_JUMP: i32 = 0x3f;
    /// Defines the `THROW` constant used by this module.
    ///
    ///
    ///  Provides the THROW constant value.
    pub const THROW: i32 = 0x40;
    /// Defines the `CAPTURE_PULLED` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE PULLED constant value.
    pub const CAPTURE_PULLED: i32 = 0x41;
    /// Defines the `CAPTURE_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE WAIT constant value.
    pub const CAPTURE_WAIT: i32 = 0x42;
    /// Defines the `CAPTURE_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE DAMAGE constant value.
    pub const CAPTURE_DAMAGE: i32 = 0x43;
    /// Defines the `CAPTURE_CUT` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE CUT constant value.
    pub const CAPTURE_CUT: i32 = 0x44;
    /// Defines the `CAPTURE_JUMP` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE JUMP constant value.
    pub const CAPTURE_JUMP: i32 = 0x45;
    /// Defines the `THROWN` constant used by this module.
    ///
    ///
    ///  Provides the THROWN constant value.
    pub const THROWN: i32 = 0x46;
    /// Defines the `DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE constant value.
    pub const DAMAGE: i32 = 0x47;
    /// Defines the `DAMAGE_AIR` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE AIR constant value.
    pub const DAMAGE_AIR: i32 = 0x48;
    /// Defines the `DAMAGE_FLY` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY constant value.
    pub const DAMAGE_FLY: i32 = 0x49;
    /// Defines the `DAMAGE_FLY_ROLL` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY ROLL constant value.
    pub const DAMAGE_FLY_ROLL: i32 = 0x4a;
    /// Defines the `DAMAGE_FLY_METEOR` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY METEOR constant value.
    pub const DAMAGE_FLY_METEOR: i32 = 0x4b;
    /// Defines the `DAMAGE_FLY_REFLECT_LR` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY REFLECT LR constant value.
    pub const DAMAGE_FLY_REFLECT_LR: i32 = 0x4c;
    /// Defines the `DAMAGE_FLY_REFLECT_U` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY REFLECT U constant value.
    pub const DAMAGE_FLY_REFLECT_U: i32 = 0x4d;
    /// Defines the `DAMAGE_FLY_REFLECT_D` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY REFLECT D constant value.
    pub const DAMAGE_FLY_REFLECT_D: i32 = 0x4e;
    /// Defines the `DAMAGE_FALL` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FALL constant value.
    pub const DAMAGE_FALL: i32 = 0x4f;
    /// Defines the `DOWN` constant used by this module.
    ///
    ///
    ///  Provides the DOWN constant value.
    pub const DOWN: i32 = 0x50;
    /// Defines the `DOWN_SPOT` constant used by this module.
    ///
    ///
    ///  Provides the DOWN SPOT constant value.
    pub const DOWN_SPOT: i32 = 0x51;
    /// Defines the `DOWN_CONTINUE` constant used by this module.
    ///
    ///
    ///  Provides the DOWN CONTINUE constant value.
    pub const DOWN_CONTINUE: i32 = 0x52;
    /// Defines the `DOWN_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the DOWN WAIT constant value.
    pub const DOWN_WAIT: i32 = 0x53;
    /// Defines the `DOWN_WAIT_CONTINUE` constant used by this module.
    ///
    ///
    ///  Provides the DOWN WAIT CONTINUE constant value.
    pub const DOWN_WAIT_CONTINUE: i32 = 0x54;
    /// Defines the `DOWN_EAT` constant used by this module.
    ///
    ///
    ///  Provides the DOWN EAT constant value.
    pub const DOWN_EAT: i32 = 0x55;
    /// Defines the `LAY_DOWN` constant used by this module.
    ///
    ///
    ///  Provides the LAY DOWN constant value.
    pub const LAY_DOWN: i32 = 0x56;
    /// Defines the `DOWN_STAND` constant used by this module.
    ///
    ///
    ///  Provides the DOWN STAND constant value.
    pub const DOWN_STAND: i32 = 0x57;
    /// Defines the `DOWN_STAND_FB` constant used by this module.
    ///
    ///
    ///  Provides the DOWN STAND FB constant value.
    pub const DOWN_STAND_FB: i32 = 0x58;
    /// Defines the `DOWN_STAND_ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the DOWN STAND ATTACK constant value.
    pub const DOWN_STAND_ATTACK: i32 = 0x59;
    /// Defines the `DOWN_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the DOWN DAMAGE constant value.
    pub const DOWN_DAMAGE: i32 = 0x5a;
    /// Defines the `DOWN_REFLECT_LR` constant used by this module.
    ///
    ///
    ///  Provides the DOWN REFLECT LR constant value.
    pub const DOWN_REFLECT_LR: i32 = 0x5b;
    /// Defines the `SHIELD_BREAK_FLY` constant used by this module.
    ///
    ///
    ///  Provides the SHIELD BREAK FLY constant value.
    pub const SHIELD_BREAK_FLY: i32 = 0x5c;
    /// Defines the `SHIELD_BREAK_FALL` constant used by this module.
    ///
    ///
    ///  Provides the SHIELD BREAK FALL constant value.
    pub const SHIELD_BREAK_FALL: i32 = 0x5d;
    /// Defines the `SHIELD_BREAK_DOWN` constant used by this module.
    ///
    ///
    ///  Provides the SHIELD BREAK DOWN constant value.
    pub const SHIELD_BREAK_DOWN: i32 = 0x5e;
    /// Defines the `FURAFURA_STAND` constant used by this module.
    ///
    ///
    ///  Provides the FURAFURA STAND constant value.
    pub const FURAFURA_STAND: i32 = 0x5f;
    /// Defines the `FURAFURA` constant used by this module.
    ///
    ///
    ///  Provides the FURAFURA constant value.
    pub const FURAFURA: i32 = 0x60;
    /// Defines the `FURAFURA_END` constant used by this module.
    ///
    ///
    ///  Provides the FURAFURA END constant value.
    pub const FURAFURA_END: i32 = 0x61;
    /// Defines the `DAMAGE_SONG_START` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE SONG START constant value.
    pub const DAMAGE_SONG_START: i32 = 0x62;
    /// Defines the `DAMAGE_SONG` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE SONG constant value.
    pub const DAMAGE_SONG: i32 = 0x63;
    /// Defines the `DAMAGE_SONG_FALL` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE SONG FALL constant value.
    pub const DAMAGE_SONG_FALL: i32 = 0x64;
    /// Defines the `DAMAGE_SONG_END` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE SONG END constant value.
    pub const DAMAGE_SONG_END: i32 = 0x65;
    /// Defines the `BIND` constant used by this module.
    ///
    ///
    ///  Provides the BIND constant value.
    pub const BIND: i32 = 0x66;
    /// Defines the `PASSIVE` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE constant value.
    pub const PASSIVE: i32 = 0x67;
    /// Defines the `PASSIVE_FB` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE FB constant value.
    pub const PASSIVE_FB: i32 = 0x68;
    /// Defines the `PASSIVE_WALL` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE WALL constant value.
    pub const PASSIVE_WALL: i32 = 0x69;
    /// Defines the `PASSIVE_WALL_JUMP` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE WALL JUMP constant value.
    pub const PASSIVE_WALL_JUMP: i32 = 0x6a;
    /// Defines the `PASSIVE_CEIL` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE CEIL constant value.
    pub const PASSIVE_CEIL: i32 = 0x6b;
    /// Defines the `STOP_WALL` constant used by this module.
    ///
    ///
    ///  Provides the STOP WALL constant value.
    pub const STOP_WALL: i32 = 0x6c;
    /// Defines the `STOP_CEIL` constant used by this module.
    ///
    ///
    ///  Provides the STOP CEIL constant value.
    pub const STOP_CEIL: i32 = 0x6d;
    /// Defines the `WALL_JUMP` constant used by this module.
    ///
    ///
    ///  Provides the WALL JUMP constant value.
    pub const WALL_JUMP: i32 = 0x6e;
    /// Defines the `CLIFF_CATCH_MOVE` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF CATCH MOVE constant value.
    pub const CLIFF_CATCH_MOVE: i32 = 0x75;
    /// Defines the `CLIFF_CATCH` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF CATCH constant value.
    pub const CLIFF_CATCH: i32 = 0x76;
    /// Defines the `CLIFF_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF WAIT constant value.
    pub const CLIFF_WAIT: i32 = 0x77;
    /// Defines the `CLIFF_ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF ATTACK constant value.
    pub const CLIFF_ATTACK: i32 = 0x78;
    /// Defines the `CLIFF_CLIMB` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF CLIMB constant value.
    pub const CLIFF_CLIMB: i32 = 0x79;
    /// Defines the `CLIFF_ESCAPE` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF ESCAPE constant value.
    pub const CLIFF_ESCAPE: i32 = 0x7a;
    /// Defines the `CLIFF_JUMP1` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF JUMP1 constant value.
    pub const CLIFF_JUMP1: i32 = 0x7b;
    /// Defines the `CLIFF_JUMP2` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF JUMP2 constant value.
    pub const CLIFF_JUMP2: i32 = 0x7c;
    /// Defines the `CLIFF_JUMP3` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF JUMP3 constant value.
    pub const CLIFF_JUMP3: i32 = 0x7d;
    /// Defines the `OTTOTTO` constant used by this module.
    ///
    ///
    ///  Provides the OTTOTTO constant value.
    pub const OTTOTTO: i32 = 0x7e;
    /// Defines the `OTTOTTO_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the OTTOTTO WAIT constant value.
    pub const OTTOTTO_WAIT: i32 = 0x7f;
    /// Defines the `SLIP` constant used by this module.
    ///
    ///
    ///  Provides the SLIP constant value.
    pub const SLIP: i32 = 0x86;
    /// Defines the `SLIP_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the SLIP DAMAGE constant value.
    pub const SLIP_DAMAGE: i32 = 0x87;
    /// Defines the `SLIP_WAIT` constant used by this module.
    ///
    ///
    ///  Provides the SLIP WAIT constant value.
    pub const SLIP_WAIT: i32 = 0x88;
    /// Defines the `SWIM_DIVE` constant used by this module.
    ///
    ///
    ///  Provides the SWIM DIVE constant value.
    pub const SWIM_DIVE: i32 = 0xab;
    /// Defines the `TREAD_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the TREAD DAMAGE constant value.
    pub const TREAD_DAMAGE: i32 = 0xb9;
    /// Defines the `TREAD_FALL` constant used by this module.
    ///
    ///
    ///  Provides the TREAD FALL constant value.
    pub const TREAD_FALL: i32 = 0xbc;
    /// Defines the `CLUNG_GANON` constant used by this module.
    ///
    ///
    ///  Provides the CLUNG GANON constant value.
    pub const CLUNG_GANON: i32 = 0xbd;
    /// Defines the `SHOULDERED_DONKEY_THROWN` constant used by this module.
    ///
    ///
    ///  Provides the SHOULDERED DONKEY THROWN constant value.
    pub const SHOULDERED_DONKEY_THROWN: i32 = 0xc7;
    /// Defines the `BURY` constant used by this module.
    ///
    ///
    ///  Provides the BURY constant value.
    pub const BURY: i32 = 0xc8;
    /// Defines the `HAMMER_LANDING` constant used by this module.
    ///
    ///
    ///  Provides the HAMMER LANDING constant value.
    pub const HAMMER_LANDING: i32 = 0xdf;
    /// Defines the `ITEM_SHOOT_WALK_BRAKE_F` constant used by this module.
    ///
    ///
    ///  Provides the ITEM SHOOT WALK BRAKE F constant value.
    pub const ITEM_SHOOT_WALK_BRAKE_F: i32 = 0x9d;
    /// Defines the `DEAD` constant used by this module.
    ///
    ///
    ///  Provides the DEAD constant value.
    pub const DEAD: i32 = 0xb5;
    /// Defines the `REBIRTH` constant used by this module.
    ///
    ///
    ///  Provides the REBIRTH constant value.
    pub const REBIRTH: i32 = 0xb6;
    /// Defines the `CATCHED_GANON` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED GANON constant value.
    pub const CATCHED_GANON: i32 = 0xbe;
    /// Defines the `CATCHED_AIR_GANON` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED AIR GANON constant value.
    pub const CATCHED_AIR_GANON: i32 = 0xbf;
    /// Defines the `CATCHED_AIR_FALL_GANON` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED AIR FALL GANON constant value.
    pub const CATCHED_AIR_FALL_GANON: i32 = 0xc0;
    /// Defines the `CATCHED_AIR_END_GANON` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED AIR END GANON constant value.
    pub const CATCHED_AIR_END_GANON: i32 = 0xc1;
    /// Defines the `ICE_JUMP` constant used by this module.
    ///
    ///
    ///  Provides the ICE JUMP constant value.
    pub const ICE_JUMP: i32 = 0xcc;
    /// Defines the `SWALLOWED` constant used by this module.
    ///
    ///
    ///  Provides the SWALLOWED constant value.
    pub const SWALLOWED: i32 = 0xe6;
    /// Defines the `CAPTURE_DAMAGE_YOSHI` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE DAMAGE YOSHI constant value.
    pub const CAPTURE_DAMAGE_YOSHI: i32 = 0xf7;
    /// Defines the `CATCHED_REFLET` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED REFLET constant value.
    pub const CATCHED_REFLET: i32 = 0xfe;
    /// Defines the `CAPTURE_MASTERHAND` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE MASTERHAND constant value.
    pub const CAPTURE_MASTERHAND: i32 = 0xff;
    /// Defines the `KILLER` constant used by this module.
    ///
    ///
    ///  Provides the KILLER constant value.
    pub const KILLER: i32 = 0x100;
    /// Defines the `GIMMICK_EATEN` constant used by this module.
    ///
    ///
    ///  Provides the GIMMICK EATEN constant value.
    pub const GIMMICK_EATEN: i32 = 0x102;
    /// Defines the `GENESIS_SHOOT_START` constant used by this module.
    ///
    ///
    ///  Provides the GENESIS SHOOT START constant value.
    pub const GENESIS_SHOOT_START: i32 = 0x103;
    /// Defines the `GENESIS_GET` constant used by this module.
    ///
    ///
    ///  Provides the GENESIS GET constant value.
    pub const GENESIS_GET: i32 = 0x106;
    /// Defines the `DEMO` constant used by this module.
    ///
    ///
    ///  Provides the DEMO constant value.
    pub const DEMO: i32 = 0x107;
    /// Defines the `KASEY_WARP` constant used by this module.
    ///
    ///
    ///  Provides the KASEY WARP constant value.
    pub const KASEY_WARP: i32 = 0x110;
    /// Defines the `LADDER_CATCH` constant used by this module.
    ///
    ///
    ///  Provides the LADDER CATCH constant value.
    pub const LADDER_CATCH: i32 = 0x111;
    /// Defines the `LADDER_END` constant used by this module.
    ///
    ///
    ///  Provides the LADDER END constant value.
    pub const LADDER_END: i32 = 0x115;
    /// Defines the `CLIFF_ROBBED` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF ROBBED constant value.
    pub const CLIFF_ROBBED: i32 = 0x116;
    /// Defines the `METAKNIGHT_FINAL_DAMAGE_FALL` constant used by this module.
    ///
    ///
    ///  Provides the METAKNIGHT FINAL DAMAGE FALL constant value.
    pub const METAKNIGHT_FINAL_DAMAGE_FALL: i32 = 0x119;
    /// Defines the `FINAL_VISUAL_ATTACK_OTHER` constant used by this module.
    ///
    ///
    ///  Provides the FINAL VISUAL ATTACK OTHER constant value.
    pub const FINAL_VISUAL_ATTACK_OTHER: i32 = 0x11f;
    /// Defines the `PSYCHOBREAK` constant used by this module.
    ///
    ///
    ///  Provides the PSYCHOBREAK constant value.
    pub const PSYCHOBREAK: i32 = 0x147;
    /// Defines the `SAVING_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the SAVING DAMAGE constant value.
    pub const SAVING_DAMAGE: i32 = 0x148;
    /// Defines the `SAVING_DAMAGE_FLY` constant used by this module.
    ///
    ///
    ///  Provides the SAVING DAMAGE FLY constant value.
    pub const SAVING_DAMAGE_FLY: i32 = 0x149;
    /// Defines the `RYU_FINAL_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the RYU FINAL DAMAGE constant value.
    pub const RYU_FINAL_DAMAGE: i32 = 0x14a;
    /// Defines the `RYU_FINAL_DAMAGE_FLY` constant used by this module.
    ///
    ///
    ///  Provides the RYU FINAL DAMAGE FLY constant value.
    pub const RYU_FINAL_DAMAGE_FLY: i32 = 0x14b;
    /// Defines the `SAVING_DAMAGE_AIR` constant used by this module.
    ///
    ///
    ///  Provides the SAVING DAMAGE AIR constant value.
    pub const SAVING_DAMAGE_AIR: i32 = 0x14c;
    /// Defines the `CLOUD_FINAL_DAMAGE_FLY` constant used by this module.
    ///
    ///
    ///  Provides the CLOUD FINAL DAMAGE FLY constant value.
    pub const CLOUD_FINAL_DAMAGE_FLY: i32 = 0x14d;
    /// Defines the `RIDLEY_FINAL_TARGET_END` constant used by this module.
    ///
    ///
    ///  Provides the RIDLEY FINAL TARGET END constant value.
    pub const RIDLEY_FINAL_TARGET_END: i32 = 0x16e;
    /// Defines the `CATCHED_RIDLEY` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED RIDLEY constant value.
    pub const CATCHED_RIDLEY: i32 = 0x171;
    /// Defines the `STABBED_DAMAGE` constant used by this module.
    ///
    ///
    ///  Provides the STABBED DAMAGE constant value.
    pub const STABBED_DAMAGE: i32 = 0x174;
    /// Defines the `SWING_GAOGAEN_CATCHED` constant used by this module.
    ///
    ///
    ///  Provides the SWING GAOGAEN CATCHED constant value.
    pub const SWING_GAOGAEN_CATCHED: i32 = 0x176;
    /// Defines the `CAPTURE_WAIT_OCTOPUS` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE WAIT OCTOPUS constant value.
    pub const CAPTURE_WAIT_OCTOPUS: i32 = 0x18c;
    /// Defines the `SIMON_FINAL_TARGET_START` constant used by this module.
    ///
    ///
    ///  Provides the SIMON FINAL TARGET START constant value.
    pub const SIMON_FINAL_TARGET_START: i32 = 0x18f;
    /// Defines the `METAMON_OUT` constant used by this module.
    ///
    ///
    ///  Provides the METAMON OUT constant value.
    pub const METAMON_OUT: i32 = 0x196;
    /// Defines the `BOSS_DEAD` constant used by this module.
    ///
    ///
    ///  Provides the BOSS DEAD constant value.
    pub const BOSS_DEAD: i32 = 0x198;
    /// Defines the `BRAVE_FINAL_TARGET_END` constant used by this module.
    ///
    ///
    ///  Provides the BRAVE FINAL TARGET END constant value.
    pub const BRAVE_FINAL_TARGET_END: i32 = 0x1a2;
    /// Defines the `DAMAGE_SLEEP_START` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE SLEEP START constant value.
    pub const DAMAGE_SLEEP_START: i32 = 0x1a3;
    /// Defines the `SLEEP_END` constant used by this module.
    ///
    ///
    ///  Provides the SLEEP END constant value.
    pub const SLEEP_END: i32 = 0x1aa;
    /// Defines the `BUDDY_FINAL_TARGET_START` constant used by this module.
    ///
    ///
    ///  Provides the BUDDY FINAL TARGET START constant value.
    pub const BUDDY_FINAL_TARGET_START: i32 = 0x1ab;
    /// Defines the `TANTAN_FINAL_TARGET_END` constant used by this module.
    ///
    ///
    ///  Provides the TANTAN FINAL TARGET END constant value.
    pub const TANTAN_FINAL_TARGET_END: i32 = 0x1ba;
    /// Defines the `GIMMICK_JUMP_BOARD` constant used by this module.
    ///
    ///
    ///  Provides the GIMMICK JUMP BOARD constant value.
    pub const GIMMICK_JUMP_BOARD: i32 = 0x1bb;
    /// Defines the `DAMAGE_FLY_REFLECT_JUMP_BOARD` constant used by this module.
    ///
    ///
    ///  Provides the DAMAGE FLY REFLECT JUMP BOARD constant value.
    pub const DAMAGE_FLY_REFLECT_JUMP_BOARD: i32 = 0x1bd;
    /// Defines the `CATCHED_PICKEL_TROLLEY` constant used by this module.
    ///
    ///
    ///  Provides the CATCHED PICKEL TROLLEY constant value.
    pub const CATCHED_PICKEL_TROLLEY: i32 = 0x1be;
    /// Defines the `EDGE_FINAL_TARGET_END` constant used by this module.
    ///
    ///
    ///  Provides the EDGE FINAL TARGET END constant value.
    pub const EDGE_FINAL_TARGET_END: i32 = 0x1d0;
    /// Defines the `TRAIL_REBOUND` constant used by this module.
    ///
    ///
    ///  Provides the TRAIL REBOUND constant value.
    pub const TRAIL_REBOUND: i32 = 0x1d1;
    /// Defines the `CAPTURE_TRAIL_KEYHOLE` constant used by this module.
    ///
    ///
    ///  Provides the CAPTURE TRAIL KEYHOLE constant value.
    pub const CAPTURE_TRAIL_KEYHOLE: i32 = 0x1d2;
    /// Defines the `STANDBY` constant used by this module.
    ///
    ///
    ///  Provides the STANDBY constant value.
    pub const STANDBY: i32 = 0x1d6;
    /// Defines the `ENTRY` constant used by this module.
    ///
    ///
    ///  Provides the ENTRY constant value.
    pub const ENTRY: i32 = 0x1d9;
    /// Defines the `LOSE` constant used by this module.
    ///
    ///
    ///  Provides the LOSE constant value.
    pub const LOSE: i32 = 0x1db;
    /// Defines the `SPECIAL_N` constant used by this module.
    ///
    ///
    ///  Provides the SPECIAL N constant value.
    pub const SPECIAL_N: i32 = 0x1dc;
    /// Defines the `SPECIAL_S` constant used by this module.
    ///
    ///
    ///  Provides the SPECIAL S constant value.
    pub const SPECIAL_S: i32 = 0x1dd;
    /// Defines the `SPECIAL_HI` constant used by this module.
    ///
    ///
    ///  Provides the SPECIAL HI constant value.
    pub const SPECIAL_HI: i32 = 0x1de;
    /// Defines the `SPECIAL_LW` constant used by this module.
    ///
    ///
    ///  Provides the SPECIAL LW constant value.
    pub const SPECIAL_LW: i32 = 0x1df;
    /// Defines the `FINAL` constant used by this module.
    ///
    ///
    ///  Provides the FINAL constant value.
    pub const FINAL: i32 = 0x1e0;
}

/// Ground-touch masks from the pinned `skyline-smash` lua_const table.
pub mod ground_touch {
    /// Defines the `DOWN_SIDE` constant used by this module.
    ///
    ///
    ///  Provides the DOWN SIDE constant value.
    pub const DOWN_SIDE: u32 = 0xc8;
    /// Defines the `LEFT_SIDE` constant used by this module.
    ///
    ///
    ///  Provides the LEFT SIDE constant value.
    pub const LEFT_SIDE: u32 = 0x52;
    /// Defines the `RIGHT_SIDE` constant used by this module.
    ///
    ///
    ///  Provides the RIGHT SIDE constant value.
    pub const RIGHT_SIDE: u32 = 0xa4;
    /// Defines the `UP_SIDE` constant used by this module.
    ///
    ///
    ///  Provides the UP SIDE constant value.
    pub const UP_SIDE: u32 = 0x31;
}

/// `SITUATION_KIND_*` values from the pinned `skyline-smash` lua_const table.
pub mod situation {
    /// Defines the `GROUND` constant used by this module.
    ///
    ///
    ///  Provides the GROUND constant value.
    pub const GROUND: i32 = 0;
    /// Defines the `CLIFF` constant used by this module.
    ///
    ///
    ///  Provides the CLIFF constant value.
    pub const CLIFF: i32 = 1;
    /// Defines the `AIR` constant used by this module.
    ///
    ///
    ///  Provides the AIR constant value.
    pub const AIR: i32 = 2;
    /// Adapter sentinel when no native situation observation is available.
    ///
    ///  Adapter sentinel when no native situation observation is available.
    pub const UNAVAILABLE: i32 = -1;

    // AttackData.situation is a bitmask, not a SITUATION_KIND value.
    /// Defines the `ATTACK_GROUND_MASK` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK GROUND MASK constant value.
    pub const ATTACK_GROUND_MASK: u16 = 1;
    /// Defines the `ATTACK_AIR_MASK` constant used by this module.
    ///
    ///
    ///  Provides the ATTACK AIR MASK constant value.
    pub const ATTACK_AIR_MASK: u16 = 2;
}

/// Native transition-term IDs from the pinned `skyline-smash` constant table.
pub mod transition_terms {
    /// Defines the `PASSIVE` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE constant value.
    pub const PASSIVE: i32 = 0x1e000013;
    /// Defines the `PASSIVE_FB` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE FB constant value.
    pub const PASSIVE_FB: i32 = 0x1e000014;
    /// Defines the `PASSIVE_WALL` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE WALL constant value.
    pub const PASSIVE_WALL: i32 = 0x1e000015;
    /// Defines the `PASSIVE_CEIL` constant used by this module.
    ///
    ///
    ///  Provides the PASSIVE CEIL constant value.
    pub const PASSIVE_CEIL: i32 = 0x1e000018;
    /// Defines the `CONT_ESCAPE` constant used by this module.
    ///
    ///
    ///  Provides the CONT ESCAPE constant value.
    pub const CONT_ESCAPE: i32 = 0x1e000026;
    /// Defines the `CONT_GUARD_ON` constant used by this module.
    ///
    ///
    ///  Provides the CONT GUARD ON constant value.
    pub const CONT_GUARD_ON: i32 = 0x1e000029;
    /// Defines the `CONT_JUMP_SQUAT` constant used by this module.
    ///
    ///
    ///  Provides the CONT JUMP SQUAT constant value.
    pub const CONT_JUMP_SQUAT: i32 = 0x1e00002b;
    /// Defines the `CONT_JUMP_SQUAT_BUTTON` constant used by this module.
    ///
    ///
    ///  Provides the CONT JUMP SQUAT BUTTON constant value.
    pub const CONT_JUMP_SQUAT_BUTTON: i32 = 0x1e00002c;
    /// Defines the `CONT_SQUAT` constant used by this module.
    ///
    ///
    ///  Provides the CONT SQUAT constant value.
    pub const CONT_SQUAT: i32 = 0x1e000034;
    /// Defines the `CONT_ESCAPE_AIR` constant used by this module.
    ///
    ///
    ///  Provides the CONT ESCAPE AIR constant value.
    pub const CONT_ESCAPE_AIR: i32 = 0x1e00003d;
    /// Defines the `CONT_ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the CONT ATTACK constant value.
    pub const CONT_ATTACK: i32 = 0x1e000052;
    /// Defines the `CONT_ATTACK_LW4_START` constant used by this module.
    ///
    ///
    ///  Provides the CONT ATTACK LW4 START constant value.
    pub const CONT_ATTACK_LW4_START: i32 = 0x1e00005f;
    /// Defines the `CONT_SPECIAL_N` constant used by this module.
    ///
    ///
    ///  Provides the CONT SPECIAL N constant value.
    pub const CONT_SPECIAL_N: i32 = 0x1e000063;
    /// Defines the `CONT_SPECIAL_HI` constant used by this module.
    ///
    ///
    ///  Provides the CONT SPECIAL HI constant value.
    pub const CONT_SPECIAL_HI: i32 = 0x1e000065;
    /// Defines the `CONT_SPECIAL_LW` constant used by this module.
    ///
    ///
    ///  Provides the CONT SPECIAL LW constant value.
    pub const CONT_SPECIAL_LW: i32 = 0x1e000066;
    /// Defines the `CONT_CATCH` constant used by this module.
    ///
    ///
    ///  Provides the CONT CATCH constant value.
    pub const CONT_CATCH: i32 = 0x1e00006f;
    /// Defines the `CONT_CATCH_DASH` constant used by this module.
    ///
    ///
    ///  Provides the CONT CATCH DASH constant value.
    pub const CONT_CATCH_DASH: i32 = 0x1e000070;
    /// Defines the `CONT_CATCH_TURN` constant used by this module.
    ///
    ///
    ///  Provides the CONT CATCH TURN constant value.
    pub const CONT_CATCH_TURN: i32 = 0x1e000071;
}

/// Native work IDs used by the read-only Switch adapter.
pub mod work_ids {
    /// Defines the `FIGHTER_INSTANCE_WORK_ID_INT_DOWN_DAMAGE_COUNT` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER INSTANCE WORK ID INT DOWN DAMAGE COUNT constant value.
    pub const FIGHTER_INSTANCE_WORK_ID_INT_DOWN_DAMAGE_COUNT: i32 = 0x1000006a;
    /// Defines the `FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER INSTANCE WORK ID INT SUCCEED ATTACKER ENTRY ID constant value.
    pub const FIGHTER_INSTANCE_WORK_ID_INT_SUCCEED_ATTACKER_ENTRY_ID: i32 = 0x100000b3;
    /// Defines the `FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER INSTANCE WORK ID FLOAT DAMAGE REACTION FRAME constant value.
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME: i32 = 0x13;
    /// Defines the `FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER INSTANCE WORK ID FLOAT DAMAGE SPEED RESERVED X constant value.
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X: i32 = 0x15;
    /// Defines the `FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER INSTANCE WORK ID FLOAT DAMAGE SPEED RESERVED Y constant value.
    pub const FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y: i32 = 0x16;
    /// Defines the `FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER GAMEWATCH INSTANCE WORK ID FLOAT SPECIAL LW GAUGE constant value.
    pub const FIGHTER_GAMEWATCH_INSTANCE_WORK_ID_FLOAT_SPECIAL_LW_GAUGE: i32 = 0x4c;
    /// Defines the `FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER STATUS CLIFF WORK FLOAT HIT NORMAL FRAME constant value.
    pub const FIGHTER_STATUS_CLIFF_WORK_FLOAT_HIT_NORMAL_FRAME: i32 = 0x1000007;
    /// Defines the `FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER STATUS GUARD DAMAGE WORK INT STIFF FRAME constant value.
    pub const FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME: i32 = 0x1100000e;
    /// Defines the `FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER POPO INSTANCE WORK ID INT PARTNER OBJECT ID constant value.
    pub const FIGHTER_POPO_INSTANCE_WORK_ID_INT_PARTNER_OBJECT_ID: i32 = 0x100000be;
    /// Defines the `FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER STATUS DAMAGE FLAG FLY DISABLE PASSIVE constant value.
    pub const FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE: i32 = 0x21000016;
    /// Defines the `FIGHTER_STATUS_DAMAGE_FLAG_END_REACTION` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER STATUS DAMAGE FLAG END REACTION constant value.
    pub const FIGHTER_STATUS_DAMAGE_FLAG_END_REACTION: i32 = 0x2100000d;
    /// Defines the `FIGHTER_STATUS_DAMAGE_FLAG_REACTION_CANCEL` constant used by this module.
    ///
    ///
    ///  Provides the FIGHTER STATUS DAMAGE FLAG REACTION CANCEL constant value.
    pub const FIGHTER_STATUS_DAMAGE_FLAG_REACTION_CANCEL: i32 = 0x2100000c;
}

// Common FIGHTER_STATUS_KIND values from the pinned native status table.
/// Defines the `GUARD_ON` constant used by this module.
///
/// Provides the GUARD ON constant value.
pub const GUARD_ON: i32 = status::GUARD_ON;
/// Defines the `GUARD` constant used by this module.
///
/// Provides the GUARD constant value.
pub const GUARD: i32 = status::GUARD;
/// Defines the `GUARD_OFF` constant used by this module.
///
/// Provides the GUARD OFF constant value.
pub const GUARD_OFF: i32 = status::GUARD_OFF;
/// Defines the `GUARD_DAMAGE` constant used by this module.
///
/// Provides the GUARD DAMAGE constant value.
pub const GUARD_DAMAGE: i32 = status::GUARD_DAMAGE;
/// Defines the `ESCAPE` constant used by this module.
///
/// Provides the ESCAPE constant value.
pub const ESCAPE: i32 = status::ESCAPE;
/// Defines the `ESCAPE_F` constant used by this module.
///
/// Provides the ESCAPE F constant value.
pub const ESCAPE_F: i32 = status::ESCAPE_F;
/// Defines the `ESCAPE_B` constant used by this module.
///
/// Provides the ESCAPE B constant value.
pub const ESCAPE_B: i32 = status::ESCAPE_B;
/// Defines the `ESCAPE_AIR` constant used by this module.
///
/// Provides the ESCAPE AIR constant value.
pub const ESCAPE_AIR: i32 = status::ESCAPE_AIR;
/// Defines the `ESCAPE_AIR_SLIDE` constant used by this module.
///
/// Provides the ESCAPE AIR SLIDE constant value.
pub const ESCAPE_AIR_SLIDE: i32 = status::ESCAPE_AIR_SLIDE;
/// Defines the `CATCH` constant used by this module.
///
/// Provides the CATCH constant value.
pub const CATCH: i32 = status::CATCH;
/// Defines the `CATCH_PULL` constant used by this module.
///
/// Provides the CATCH PULL constant value.
pub const CATCH_PULL: i32 = status::CATCH_PULL;
/// Defines the `CATCH_DASH` constant used by this module.
///
/// Provides the CATCH DASH constant value.
pub const CATCH_DASH: i32 = status::CATCH_DASH;
/// Defines the `CATCH_DASH_PULL` constant used by this module.
///
/// Provides the CATCH DASH PULL constant value.
pub const CATCH_DASH_PULL: i32 = status::CATCH_DASH_PULL;
/// Defines the `CATCH_TURN` constant used by this module.
///
/// Provides the CATCH TURN constant value.
pub const CATCH_TURN: i32 = status::CATCH_TURN;
/// Defines the `THROW` constant used by this module.
///
/// Provides the THROW constant value.
pub const THROW: i32 = status::THROW;
/// Defines the `CAPTURE_PULLED` constant used by this module.
///
/// Provides the CAPTURE PULLED constant value.
pub const CAPTURE_PULLED: i32 = status::CAPTURE_PULLED;
/// Defines the `THROWN` constant used by this module.
///
/// Provides the THROWN constant value.
pub const THROWN: i32 = status::THROWN;

/// Ordinary jump/fall states only: never recovery, tumble, attack or air dodge.
///
///  Ordinary jump/fall states only: never recovery, tumble, attack or air dodge.
pub fn jump_or_fall(status: i32) -> bool {
    matches!(
        status,
        status::JUMP | status::JUMP_AERIAL | status::FALL | status::FALL_AERIAL
    )
}

/// Special captures/final victims are not ordinary combat targets, even when a
/// transient LinkModule capture link is absent or uses a move-specific link.
///
///  Special captures/final victims are not ordinary combat targets, even when a transient LinkModule capture link is absent or uses a move-specific link.
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
///
///  Abort existing input tapes on forced states, without disabling native mash/tech safety. Do not resume an old attack after a stun/capture ends.
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
///
///  These vulnerable bodies may be hit by a configured shield backup, but an ordinary grab must not be chosen for their downed/buried/frozen states.
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
///
///  Private status values overlap across fighters. Apply these forced-state meanings only to the native kind that owns them.
pub fn fighter_sequence_interrupted(kind: i32, status: i32) -> bool {
    sequence_interrupted(status)
        || fighter_noncombat(kind, status)
        || kind_statuses::forced_status(kind, status)
}

/// Inactive swap forms and known Final Smash actors are not ordinary punish
/// targets, even through a transient missing capture link.
///
///  Inactive swap forms and known Final Smash actors are not ordinary punish targets, even through a transient missing capture link.
pub fn fighter_noncombat(kind: i32, status: i32) -> bool {
    kind_statuses::noncombat_status(kind, status)
}
