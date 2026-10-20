//! Minimal game ABI declarations checked against skyline-smash revision
//! d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435 (MIT).
//! Game version compatibility still requires on-device verification.
//!
//! FFI means Foreign Function Interface: these declarations describe functions
//! already provided by the game. They do not contain their implementations.
//! `link_name` identifies the C++ export; Rust names below make callers readable.
//! `extern "C"` selects the calling convention, not a network connection.
//!
//! To add a game query, verify its symbol, parameter types, and return type in
//! compatible upstream bindings first. A declaration that compiles can still
//! describe the wrong native ABI. Keep new declarations narrow and read-only.
use std::ffi::c_void;
// We treat game-owned objects as opaque pointers. Rust must not allocate/free
// them or assume they remain valid across loading, respawn, or match changes.
pub type Object = c_void;
// Verified in the pinned lua_const.rs; HIT is 0x2 (0x1 is not a hit mask).
pub const COLLISION_KIND_MASK_HIT: i32 = 0x2;
// The game's Vector2 contains two adjacent floats. repr(C) fixes their layout
// so pointers passed to ray_check refer to the format the game expects.
#[repr(C)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
// Native by-value Vector3f is three adjacent floats (12 bytes), checked against
// the pinned phx::Vector3f declaration. It is borrowed data, not a game object.
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
const _: () = {
    assert!(std::mem::size_of::<Vector3>() == 12);
};
// Calling these functions is unsafe: the caller must establish that the game
// object is live and that the executable matches the assumed native interface.
// Reading guide for a declaration below:
// * `pub fn` makes its Rust name callable from live.rs.
// * `*mut Object` is a raw address of a game allocation, not an owned Rust value.
// * The return type describes native data; status IDs are integers, not buttons.
// Declarations do not execute queries: a call such as game::status_kind(object)
// in live.rs does. Changing a type here changes how that call crosses the ABI.
extern "C" {
    // Identity, state, and timers used to decide whether assistance applies.
    #[link_name = "\u{1}_ZN3app8lua_bind36ControlModule__get_clatter_time_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn clatter_time(module_accessor: *mut Object, arg2: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind30StatusModule__status_kind_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn status_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind33StatusModule__situation_kind_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn situation_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24StopModule__is_stop_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn is_stop(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__is_flag_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn is_flag(module_accessor: *mut Object, flag: i32) -> bool;
    // Read-only ledge observations, verified in the same pinned cpp.rs bindings.
    #[link_name = "\u{1}_ZN3app8lua_bind22PostureModule__lr_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn facing(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind24TeamModule__team_no_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn team(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind32HitModule__get_total_status_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn hit_status(module_accessor: *mut Object, index: i32) -> u64;
    // Status and motion are different: "ledge roll" is a status, while motion
    // identifies the character's animation and its current fractional frame.
    // Policy uses both so repeated input polls do not imply animation progress.
    // Read animation timing for ledge prediction and invincibility expiry.
    #[link_name = "\u{1}_ZN3app8lua_bind30MotionModule__motion_kind_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn motion_kind(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind24MotionModule__frame_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn motion_frame(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind23MotionModule__rate_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn motion_rate(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind28MotionModule__end_frame_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn motion_end_frame(module_accessor: *mut Object) -> f32;
    // Infliction asks whether Kirby registered a hit; is_attack asks whether
    // an opponent currently has an active attack slot. Starting an animation
    // alone proves neither a successful hit nor a safely ended get-up attack.
    #[link_name = "\u{1}_ZN3app8lua_bind39AttackModule__is_infliction_status_implEPNS_26BattleObjectModuleAccessorEj"]
    pub fn attack_infliction(module_accessor: *mut Object, mask: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind28AttackModule__is_attack_implEPNS_26BattleObjectModuleAccessorEib"]
    pub fn attack_active(module_accessor: *mut Object, index: i32, alternate: bool) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35StatusModule__prev_status_kind_implEPNS_26BattleObjectModuleAccessorEj"]
    pub fn previous_status(module_accessor: *mut Object, index: u32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__get_int_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn work_int(module_accessor: *mut Object, key: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind26WorkModule__get_float_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn work_float(module_accessor: *mut Object, key: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind42WorkModule__is_enable_transition_term_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn transition_enabled(module_accessor: *mut Object, key: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35CancelModule__is_enable_cancel_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn cancel_enabled(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind36GroundModule__hang_cliff_pos_3f_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn cliff_position(module_accessor: *mut Object) -> Vector3;
    #[link_name = "\u{1}_ZN3app8lua_bind50FighterMotionModuleImpl__get_hit_normal_frame_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Eb"]
    pub fn normal_hit_frame(module_accessor: *mut Object, motion: u64, alternate: bool) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind46FighterMotionModuleImpl__get_cancel_frame_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Eb"]
    pub fn cancel_frame(module_accessor: *mut Object, motion: u64, alternate: bool) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind32WorkModule__get_param_float_implEPNS_26BattleObjectModuleAccessorEmm"]
    pub fn param_float(module_accessor: *mut Object, parent: u64, child: u64) -> f32;
    // WorkModule keys and transition-term IDs are version-specific constants
    // supplied by callers. They are not config values or offsets to invent.
    // Motion/position queries feed prediction; the mod never writes velocity.
    #[link_name = "\u{1}_ZN3app8lua_bind35KineticModule__get_sum_speed_x_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn speed_x(module_accessor: *mut Object, reserve_type: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind35KineticModule__get_sum_speed_y_implEPNS_26BattleObjectModuleAccessorEi"]
    pub fn speed_y(module_accessor: *mut Object, reserve_type: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_x_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn pos_x(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_y_implEPNS_26BattleObjectModuleAccessorE"]
    pub fn pos_y(module_accessor: *mut Object) -> f32;
    // ray_check receives an origin and a displacement (the ray's travel),
    // rather than two endpoints. Both Vector2 values must stay alive for the call.
    #[link_name = "\u{1}_ZN3app8lua_bind28GroundModule__ray_check_implEPNS_26BattleObjectModuleAccessorERKN3phx8Vector2fES6_b"]
    pub fn ray_check(
        module_accessor: *mut Object,
        arg2: *const Vector2,
        arg3: *const Vector2,
        arg4: bool,
    ) -> u64;
    // Re-resolve an accessor from a checked active object ID each update.
    // The resulting pointer is borrowed from the game, not owned by this plugin.
    #[link_name = "\u{1}_ZN3app16sv_battle_object15module_accessorEj"]
    pub fn module_accessor(object_id: u32) -> *mut Object;
    #[link_name = "\u{1}_ZN3app16sv_battle_object9is_activeEj"]
    pub fn is_active(arg1: u32) -> bool;
    // Enumerate current fighters directly. Ledge opponents may not poll the
    // hooked command category while hanging, so callbacks alone are insufficient.
    #[link_name = "\u{1}_ZN3app8lua_bind32FighterManager__entry_count_implEPNS_14FighterManagerE"]
    pub fn entry_count(manager: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind33FighterManager__get_entry_id_implEPNS_14FighterManagerEi"]
    pub fn entry_id(manager: *mut Object, ordinal: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind38FighterManager__get_fighter_entry_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    pub fn fighter_entry(manager: *mut Object, entry: i32) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind37FighterEntry__current_fighter_id_implEPNS_12FighterEntryE"]
    pub fn current_fighter_id(entry: *mut Object) -> u64;
    // Manager queries exclude CPU-controlled actors and result-screen activity.
    #[link_name = "\u{1}_ZN3app8lua_bind44FighterManager__get_fighter_information_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    pub fn fighter_information(arg1: *mut Object, arg2: i32) -> *mut Object;
    #[link_name = "\u{1}_ZN3app8lua_bind41FighterInformation__is_operation_cpu_implEPNS_18FighterInformationE"]
    pub fn is_cpu(arg1: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35FighterManager__is_result_mode_implEPNS_14FighterManagerE"]
    pub fn is_result(arg1: *mut Object) -> bool;
}
