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
//! Technical: Defines the pinned native types, offsets, and read-only game calls
//! used by the Switch integration.
//! Plain English: This is the narrow interface Kirby Assist uses to inspect Smash.
use std::ffi::c_void;

/// Run one native ground ray and account for it in enabled probes.
/// `ray_displacement` is the ray's travel from `ray_origin`, not a second
/// endpoint.
/// Calls the pinned native ray-collision routine using the supplied origin and displacement.
/// # Safety
/// `module_accessor` must point to a live game object for the duration of the
/// call. `ray_origin` and `ray_displacement` must point to initialized,
/// correctly laid-out `Vector2` values that remain valid until the game call
/// returns. The running game must match the pinned ABI documented above.
#[inline]
pub unsafe fn ray_check(
    module_accessor: *mut Object,
    ray_origin: *const Vector2,
    ray_displacement: *const Vector2,
    check_passable_ground: bool,
) -> u64 {
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::ray();
    native_ray_check(
        module_accessor,
        ray_origin,
        ray_displacement,
        check_passable_ground,
    )
}
// We treat game-owned objects as opaque pointers. Rust must not allocate/free
// them or assume they remain valid across loading, respawn, or match changes.
/// Opaque stand-in for a game-owned object allocation.
pub type Object = c_void;
/// Byte offset of the battle-object ID in the pinned native layout.
pub const BATTLE_OBJECT_ID_OFFSET_BYTES: usize = 8;
/// Index of the most recently recorded previous status.
pub const LATEST_PREVIOUS_STATUS_HISTORY_INDEX: u32 = 0;
// Aggregate main, gravity, and damage knockback energy in sum-speed queries.
/// Native reserve selector that aggregates all kinetic energy sources.
pub const ALL_KINETIC_ENERGY_RESERVES: i32 = -1;

/// Read the object ID from the pinned BattleObject layout.
/// Loads the object identifier at the verified battle-object byte offset.
/// # Safety
/// `object` must be a live, non-null game object with the pinned ABI layout.
#[inline]
pub unsafe fn battle_object_id(object: *mut Object) -> u32 {
    object
        .cast::<u8>()
        .add(BATTLE_OBJECT_ID_OFFSET_BYTES)
        .cast::<u32>()
        .read()
}

// Verified in the pinned lua_const.rs; HIT is 0x2 (0x1 is not a hit mask).
/// Native collision-mask bit identifying hit contact.
pub const COLLISION_KIND_MASK_HIT: i32 = 0x2;
// The game's Vector2 contains two adjacent floats. repr(C) fixes their layout
// so pointers passed to ray_check refer to the format the game expects.
#[repr(C)]
/// ABI-compatible pair of native floating-point coordinates.
pub struct Vector2 {
    /// Horizontal component in the native two-float vector.
    pub x: f32,
    /// Vertical component in the native two-float vector.
    pub y: f32,
}
// Native by-value Vector3f is three adjacent floats (12 bytes), checked against
// the pinned phx::Vector3f declaration. It is borrowed data, not a game object.
#[repr(C)]
/// ABI-compatible triple of native floating-point coordinates.
pub struct Vector3 {
    /// Horizontal component of the native three-float vector.
    pub x: f32,
    /// Vertical component of the native three-float vector.
    pub y: f32,
    /// Depth component of the native three-float vector.
    pub z: f32,
}
extern "C" {
    // Pinned GroundUtility binding and GROUND_DEAD_AREA_CHECK_RESULT enum.
    // Read-only point classification; no Lua state or module offsets required.
    #[link_name = "\u{1}_ZN3app13GroundUtility15check_dead_areaERKN3phx8Vector3fE"]
    /// Classifies a world-space point against native blast/dead areas.
    pub fn check_dead_area(point: *const Vector3) -> i32;
}
// Read-only AttackData prefix from the pinned skyline-smash cpp.rs. Never
// retain this pointer across native calls; copy only currently active slots.
#[repr(C)]
#[derive(Clone, Copy)]
/// Copyable prefix of native attack geometry and damage fields.
pub struct AttackGeometry {
    /// First endpoint's local X coordinate.
    pub x: f32,
    /// First endpoint's local Y coordinate.
    pub y: f32,
    /// First endpoint's local Z coordinate.
    pub z: f32,
    /// Padding word in the pinned native structure.
    pub padding: u32,
    /// Optional second endpoint's local X coordinate.
    pub x2: f32,
    /// Optional second endpoint's local Y coordinate.
    pub y2: f32,
    /// Optional second endpoint's local Z coordinate.
    pub z2: f32,
    /// Second padding word in the pinned native structure.
    pub padding2: u32,
    /// Native attack power value.
    pub power: f32,
    /// Native hitbox radius/size.
    pub size: f32,
    /// Native launch-angle setting.
    pub angle: i32,
    /// Knockback growth parameter.
    pub kbg: i32,
    /// Fixed knockback parameter.
    pub fkb: i32,
    /// Base knockback parameter.
    pub bkb: i32,
    /// Native slip chance/value.
    pub slip: f32,
    /// Native hitlag multiplier.
    pub hitlag: f32,
    /// Native SDI multiplier.
    pub sdi: f32,
    /// Native joint hash used to transform local geometry to world space.
    pub node: u64,
    /// Native attack situation mask.
    pub check_type: u16,
    /// Native attack situation/category mask.
    pub situation: u16,
    // Full AttackData field offsets verified against
    // https://github.com/WuBoytH/smash-rs/blob/master/src/app/modules/attack.rs
    // the pinned SDK describes only the geometry prefix above. Read bytes,
    // not Rust bools, so an unexpected native value cannot cause bool UB.
    /// Opaque bytes covering native attack capability fields before flags.
    pub capability_padding: [u8; 17],
    /// Native reflectable capability byte.
    pub reflectable: u8,
    /// Native absorbable capability byte.
    pub absorbable: u8,
    /// Native direct-attack capability byte.
    pub direct: u8,
}
/// Expected native byte offset of the hitbox size field.
const ATTACK_GEOMETRY_SIZE_FIELD_OFFSET_BYTES: usize = 0x24;
/// Expected native byte offset of the joint hash field.
const ATTACK_GEOMETRY_NODE_FIELD_OFFSET_BYTES: usize = 0x48;
/// Expected native byte offset of the situation mask field.
const ATTACK_GEOMETRY_SITUATION_FIELD_OFFSET_BYTES: usize = 0x52;
/// Expected native byte offset of the reflectable capability byte.
const ATTACK_GEOMETRY_REFLECTABLE_FIELD_OFFSET_BYTES: usize = 0x65;
/// Expected native byte offset of the absorbable capability byte.
const ATTACK_GEOMETRY_ABSORBABLE_FIELD_OFFSET_BYTES: usize = 0x66;
/// Expected native byte offset of the direct-attack capability byte.
const ATTACK_GEOMETRY_DIRECT_FIELD_OFFSET_BYTES: usize = 0x67;
/// Expected size of the native two-float vector.
const VECTOR2_SIZE_BYTES: usize = 8;
/// Expected size of the native three-float vector.
const VECTOR3_SIZE_BYTES: usize = 12;
/// Expected byte size of the copied native attack-geometry prefix.
const ATTACK_GEOMETRY_SIZE_BYTES: usize = 0x68;
// Compile-time assertions validate Rust's declared layout against the pinned ABI.
const _: () = {
    assert!(std::mem::offset_of!(AttackGeometry, size) == ATTACK_GEOMETRY_SIZE_FIELD_OFFSET_BYTES);
    assert!(std::mem::offset_of!(AttackGeometry, node) == ATTACK_GEOMETRY_NODE_FIELD_OFFSET_BYTES);
    assert!(
        std::mem::offset_of!(AttackGeometry, situation)
            == ATTACK_GEOMETRY_SITUATION_FIELD_OFFSET_BYTES
    );
    assert!(
        std::mem::offset_of!(AttackGeometry, reflectable)
            == ATTACK_GEOMETRY_REFLECTABLE_FIELD_OFFSET_BYTES
    );
    assert!(
        std::mem::offset_of!(AttackGeometry, absorbable)
            == ATTACK_GEOMETRY_ABSORBABLE_FIELD_OFFSET_BYTES
    );
    assert!(
        std::mem::offset_of!(AttackGeometry, direct) == ATTACK_GEOMETRY_DIRECT_FIELD_OFFSET_BYTES
    );
};
extern "C" {
    #[link_name = "\u{1}_ZN3app10sv_animcmd6ATTACKEP9lua_State"]
    /// Invokes the game's native ATTACK script command.
    pub fn attack_command(lua_state: u64);
    #[link_name = "\u{1}_ZN3app9sv_system29battle_object_module_accessorEP9lua_State"]
    /// Resolves the battle-object accessor associated with a Lua state.
    pub fn lua_object(lua_state: u64) -> *mut Object;
    #[link_name = "\u{1}_ZN3app8lua_bind30AttackModule__attack_data_implEPNS_26BattleObjectModuleAccessorEib"]
    /// Returns a pointer to one native attack-geometry record.
    pub fn attack_geometry(
        module_accessor: *mut Object,
        attack_slot: i32,
        use_secondary_slot: bool,
    ) -> *const AttackGeometry;
    #[link_name = "\u{1}_ZN3app8lua_bind51ModelModule__joint_global_position_with_offset_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ERKNS3_8Vector3fERS5_b"]
    /// Transforms a local joint-relative offset into world coordinates.
    pub fn joint_position(
        module_accessor: *mut Object,
        joint_hash: u64,
        local_offset: *const Vector3,
        world_position: *mut Vector3,
        apply_transform: bool,
    );
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_z_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native depth coordinate.
    pub fn pos_z(module_accessor: *mut Object) -> f32;
}
const _: () = {
    assert!(std::mem::size_of::<Vector2>() == VECTOR2_SIZE_BYTES);
    assert!(std::mem::size_of::<Vector3>() == VECTOR3_SIZE_BYTES);
    assert!(std::mem::size_of::<AttackGeometry>() == ATTACK_GEOMETRY_SIZE_BYTES);
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
    // Pinned skyline-smash cpp.rs, app::smashball::is_training_mode.
    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    /// Queries whether the current scene is Training Mode.
    pub fn is_training_mode() -> bool;
    // Capture-link queries, checked against the pinned cpp.rs bindings.
    // LINK_NO_CAPTURE = 0. Resolve IDs anew; never retain the returned accessor.
    #[link_name = "\u{1}_ZN3app8lua_bind24LinkModule__is_link_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Tests whether the object has a native link of the requested kind.
    pub fn is_link(module_accessor: *mut Object, link_kind: i32) -> bool;
    // Identity, state, and timers used to decide whether assistance applies.
    #[link_name = "\u{1}_ZN3app8lua_bind36ControlModule__get_clatter_time_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads the native clatter timer for a specified clatter kind.
    pub fn clatter_time(module_accessor: *mut Object, clatter_kind: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind30StatusModule__status_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's current native status ID.
    pub fn status_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind33StatusModule__situation_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's current native situation ID.
    pub fn situation_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24StopModule__is_stop_implEPNS_26BattleObjectModuleAccessorE"]
    /// Queries whether native stop is active for this object.
    pub fn is_stop(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__is_flag_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads a native work-module flag by ID.
    pub fn is_flag(module_accessor: *mut Object, flag: i32) -> bool;
    // Same pinned bindings: damage hitstop distinguishes the victim from an
    // attacker's hitlag and from shield/global freeze. Remaining time dedups SDI.
    #[link_name = "\u{1}_ZN3app8lua_bind26StopModule__is_damage_implEPNS_26BattleObjectModuleAccessorE"]
    /// Checks whether this object is in damage stop.
    pub fn is_damage_stop(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind40StopModule__get_hit_stop_real_frame_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads remaining native hitstop frames for the object.
    pub fn hitstop_remaining(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app16sv_battle_object4kindEj"]
    /// Resolves a fighter-kind ID from a battle-object ID.
    pub fn fighter_kind(battle_object_id: u32) -> i32;
    // Read-only ledge observations, verified in the same pinned cpp.rs bindings.
    #[link_name = "\u{1}_ZN3app8lua_bind22PostureModule__lr_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native facing-direction scalar.
    pub fn facing(module_accessor: *mut Object) -> f32;
    // Read-only size/floor checks for the narrow Kirby duck allowlist, verified
    // against the same pinned cpp.rs. Down input must not drop through a platform.
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__scale_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native model scale.
    pub fn scale(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind37GroundModule__is_passable_ground_implEPNS_26BattleObjectModuleAccessorE"]
    /// Queries whether the object is on passable ground.
    pub fn passable_ground(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind24TeamModule__team_no_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native team identifier.
    pub fn team(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind32HitModule__get_total_status_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads one native hit-status value by index.
    pub fn hit_status(module_accessor: *mut Object, hit_status_index: i32) -> u64;
    // Status and motion are different: "ledge roll" is a status, while motion
    // identifies the character's animation and its current fractional frame.
    // Policy uses both so repeated input polls do not imply animation progress.
    // Read animation timing for ledge prediction and invincibility expiry.
    #[link_name = "\u{1}_ZN3app8lua_bind30MotionModule__motion_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the current native motion hash.
    pub fn motion_kind(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind24MotionModule__frame_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the current fractional frame within the native motion.
    pub fn motion_frame(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind23MotionModule__rate_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native motion playback rate.
    pub fn motion_rate(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind28MotionModule__end_frame_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native end-frame value for the current motion.
    pub fn motion_end_frame(module_accessor: *mut Object) -> f32;
    // Infliction asks whether Kirby registered a hit; is_attack asks whether
    // an opponent currently has an active attack slot. Starting an animation
    // alone proves neither a successful hit nor a safely ended get-up attack.
    // Fresh collision on this update; unlike is_infliction_status, this cannot
    // carry an earlier fair hit forward into a later hitbox interval.
    #[link_name = "\u{1}_ZN3app8lua_bind32AttackModule__is_infliction_implEPNS_26BattleObjectModuleAccessorEj"]
    /// Queries whether the specified collision type occurred now.
    pub fn attack_infliction_now(module_accessor: *mut Object, collision_mask: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind28AttackModule__is_attack_implEPNS_26BattleObjectModuleAccessorEib"]
    /// Checks whether the indexed native attack slot is active.
    pub fn attack_active(module_accessor: *mut Object, attack_slot: i32, alternate: bool) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35StatusModule__prev_status_kind_implEPNS_26BattleObjectModuleAccessorEj"]
    /// Reads a prior status from the indexed native history entry.
    pub fn previous_status(module_accessor: *mut Object, history_index: u32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__get_int_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads a signed integer from the object's native work module.
    pub fn work_int(module_accessor: *mut Object, work_id: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind26WorkModule__get_float_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads a floating-point value from the object's native work module.
    pub fn work_float(module_accessor: *mut Object, work_id: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind42WorkModule__is_enable_transition_term_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Queries whether a native transition term is enabled.
    pub fn transition_enabled(module_accessor: *mut Object, transition_term: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35CancelModule__is_enable_cancel_implEPNS_26BattleObjectModuleAccessorE"]
    #[allow(dead_code)] // Used by the parked offline whiff-observation prototype.
    /// Reads the native cancel-enabled flag for an object.
    pub fn cancel_enabled(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind36GroundModule__hang_cliff_pos_3f_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native cliff position in world coordinates.
    pub fn cliff_position(module_accessor: *mut Object) -> Vector3;
    #[link_name = "\u{1}_ZN3app8lua_bind50FighterMotionModuleImpl__get_hit_normal_frame_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Eb"]
    /// Reads native normal-hit timing for a specified motion.
    pub fn normal_hit_frame(module_accessor: *mut Object, motion_hash: u64, alternate: bool)
        -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind46FighterMotionModuleImpl__get_cancel_frame_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Eb"]
    /// Reads the native cancel-frame value for a specified motion.
    pub fn cancel_frame(module_accessor: *mut Object, motion_hash: u64, alternate: bool) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind32WorkModule__get_param_float_implEPNS_26BattleObjectModuleAccessorEmm"]
    /// Reads a native floating-point parameter by table and key.
    pub fn param_float(
        module_accessor: *mut Object,
        parameter_table_hash: u64,
        parameter_hash: u64,
    ) -> f32;
    // WorkModule keys and transition-term IDs are version-specific constants
    // supplied by callers. They are not config values or offsets to invent.
    // Motion/position queries feed prediction; the mod never writes velocity.
    #[link_name = "\u{1}_ZN3app8lua_bind35KineticModule__get_sum_speed_x_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads horizontal speed for the selected kinetic reserve.
    pub fn speed_x(module_accessor: *mut Object, reserve_type: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind35KineticModule__get_sum_speed_y_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads vertical speed for the selected kinetic reserve.
    pub fn speed_y(module_accessor: *mut Object, reserve_type: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_x_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native X position.
    pub fn pos_x(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_y_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native Y position.
    pub fn pos_y(module_accessor: *mut Object) -> f32;
    // ray_check receives an origin and a displacement (the ray's travel),
    // rather than two endpoints. Both Vector2 values must stay alive for the call.
    #[link_name = "\u{1}_ZN3app8lua_bind28GroundModule__ray_check_implEPNS_26BattleObjectModuleAccessorERKN3phx8Vector2fES6_b"]
    /// Performs the native ground-ray query underlying [`ray_check`].
    fn native_ray_check(
        module_accessor: *mut Object,
        ray_origin: *const Vector2,
        ray_displacement: *const Vector2,
        check_passable_ground: bool,
    ) -> u64;
    // Re-resolve an accessor from a checked active object ID each update.
    // The resulting pointer is borrowed from the game, not owned by this plugin.
    #[link_name = "\u{1}_ZN3app16sv_battle_object15module_accessorEj"]
    /// Resolves a current module accessor from a battle-object ID.
    pub fn module_accessor(object_id: u32) -> *mut Object;
    #[link_name = "\u{1}_ZN3app16sv_battle_object9is_activeEj"]
    /// Returns whether the specified battle object ID corresponds to an active object.
    /// Checks whether the battle-object ID currently names an active object.
    pub fn is_active(battle_object_id: u32) -> bool;
    // Enumerate current fighters directly. Ledge opponents may not poll the
    // hooked command category while hanging, so callbacks alone are insufficient.
    #[link_name = "\u{1}_ZN3app8lua_bind32FighterManager__entry_count_implEPNS_14FighterManagerE"]
    /// Reads the fighter-entry count from the native manager.
    pub fn entry_count(manager: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind33FighterManager__get_entry_id_implEPNS_14FighterManagerEi"]
    /// Resolves a fighter ordinal to its native player-entry ID.
    pub fn entry_id(manager: *mut Object, fighter_ordinal: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind38FighterManager__get_fighter_entry_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    /// Reads the native fighter-entry pointer value for an entry ID.
    pub fn fighter_entry(manager: *mut Object, entry_id: i32) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind37FighterEntry__current_fighter_id_implEPNS_12FighterEntryE"]
    /// Reads the current fighter object ID from a native entry.
    pub fn current_fighter_id(fighter_entry: *mut Object) -> u64;
    // Manager queries exclude CPU-controlled actors and result-screen activity.
    #[link_name = "\u{1}_ZN3app8lua_bind44FighterManager__get_fighter_information_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    /// Resolves native fighter-information data for a player entry.
    pub fn fighter_information(manager: *mut Object, entry_id: i32) -> *mut Object;
    #[link_name = "\u{1}_ZN3app8lua_bind41FighterInformation__is_operation_cpu_implEPNS_18FighterInformationE"]
    /// Reads the CPU-control flag from native fighter information.
    pub fn is_cpu(fighter_information: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35FighterManager__is_result_mode_implEPNS_14FighterManagerE"]
    /// Queries whether the battle manager is in the results scene.
    pub fn is_result(manager: *mut Object) -> bool;
    // Pinned cpp.rs: FighterManager::is_ready_go, read-only battle phase.
    #[link_name = "\u{1}_ZN3app8lua_bind32FighterManager__is_ready_go_implEPNS_14FighterManagerE"]
    /// Queries the manager's native Ready-Go state.
    pub fn is_ready_go(manager: *mut Object) -> bool;
}
