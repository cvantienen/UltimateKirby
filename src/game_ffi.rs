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
///
/// `ray_displacement` is the ray's travel from `ray_origin`, not a second
/// endpoint.
/// Calls the pinned native ray-collision routine using the supplied origin and displacement.
///
///  Checks whether a game object intersects a ray of a given length.
///
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
///
///  Represents a Smash object without letting Rust manage its memory.
pub type Object = c_void;
/// Byte offset of the battle-object ID in the pinned native layout.
///
///  Where the object's ID is stored in its game memory.
pub const BATTLE_OBJECT_ID_OFFSET_BYTES: usize = 8;
/// Index of the most recently recorded previous status.
///
///  Selects the latest status transition in the game's history.
pub const LATEST_PREVIOUS_STATUS_HISTORY_INDEX: u32 = 0;
// Aggregate main, gravity, and damage knockback energy in sum-speed queries.
/// Native reserve selector that aggregates all kinetic energy sources.
///
///  Tells speed queries to include every movement/knockback source.
pub const ALL_KINETIC_ENERGY_RESERVES: i32 = -1;

/// Read the object ID from the pinned BattleObject layout.
/// Loads the object identifier at the verified battle-object byte offset.
///
///  Gets the game's numeric ID for this object.
///
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
///
///  Selects actual hits rather than other collision events.
pub const COLLISION_KIND_MASK_HIT: i32 = 0x2;
/// Native button identifier for the guard input.
///
///  Identifies the game's shield button.
pub const CONTROL_PAD_BUTTON_GUARD: i32 = 0x3;
/// Native button identifier for the held-guard state.
///
///  Identifies the game's sustained shield input.
pub const CONTROL_PAD_BUTTON_GUARD_HOLD: i32 = 0x10;
// The game's Vector2 contains two adjacent floats. repr(C) fixes their layout
// so pointers passed to ray_check refer to the format the game expects.
#[repr(C)]
/// ABI-compatible pair of native floating-point coordinates.
///
///  A two-dimensional position or direction used by Smash.
pub struct Vector2 {
    /// Horizontal component in the native two-float vector.
    ///
    ///  The vector's left/right value.
    pub x: f32,
    /// Vertical component in the native two-float vector.
    ///
    ///  The vector's up/down value.
    pub y: f32,
}
// Native by-value Vector3f is three adjacent floats (12 bytes), checked against
// the pinned phx::Vector3f declaration. It is borrowed data, not a game object.
#[repr(C)]
/// ABI-compatible triple of native floating-point coordinates.
///
///  A three-dimensional position or direction used by Smash.
pub struct Vector3 {
    /// Horizontal component of the native three-float vector.
    ///
    ///  The vector's left/right value.
    pub x: f32,
    /// Vertical component of the native three-float vector.
    ///
    ///  The vector's up/down value.
    pub y: f32,
    /// Depth component of the native three-float vector.
    ///
    ///  The vector's forward/back value.
    pub z: f32,
}
extern "C" {
    #[link_name = "\u{1}_ZN3app8lua_bind35ControlModule__check_button_on_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Queries whether a native button is currently active for an object.
    ///
    ///  Checks whether the game says a button is pressed.
    pub fn button_on(module_accessor: *mut Object, button_id: i32) -> bool;
    // Read-only terrain contact / animated ECB base accessors; pinned cpp.rs.
    #[link_name = "\u{1}_ZN3app8lua_bind27GroundModule__is_touch_implEPNS_26BattleObjectModuleAccessorEj"]
    /// Tests native ground-contact bits for the supplied touch mask.
    ///
    ///  Checks whether the object is touching the requested ground surface.
    pub fn ground_touch(module_accessor: *mut Object, touch_mask: u32) -> bool;
    #[link_name = "\u{1}_ZN3app11FighterUtil23is_touch_passive_groundERNS_26BattleObjectModuleAccessorEj"]
    /// Queries passive-ground contact using the pinned FighterUtil binding.
    ///
    ///  Checks whether the fighter can passively contact that surface.
    pub fn passive_ground_touch(module_accessor: *mut Object, touch_mask: u32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind31GroundModule__get_offset_x_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native ground collision offset on the X axis.
    ///
    ///  Gets the object's horizontal ground offset.
    pub fn ground_offset_x(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind31GroundModule__get_offset_y_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native ground collision offset on the Y axis.
    ///
    ///  Gets the object's vertical ground offset.
    pub fn ground_offset_y(module_accessor: *mut Object) -> f32;
    // Pinned GroundUtility binding and GROUND_DEAD_AREA_CHECK_RESULT enum.
    // Read-only point classification; no Lua state or module offsets required.
    #[link_name = "\u{1}_ZN3app13GroundUtility15check_dead_areaERKN3phx8Vector3fE"]
    /// Classifies a world-space point against native blast/dead areas.
    ///
    ///  Checks whether the point is outside the stage's safe area.
    pub fn check_dead_area(point: *const Vector3) -> i32;
}
// Read-only AttackData prefix from the pinned skyline-smash cpp.rs. Never
// retain this pointer across native calls; copy only currently active slots.
#[repr(C)]
#[derive(Clone, Copy)]
/// Copyable prefix of native attack geometry and damage fields.
///
///  A local copy of the active hitbox shape and damage information.
pub struct AttackGeometry {
    /// First endpoint's local X coordinate.
    ///
    ///  Horizontal position of the hitbox's first point.
    pub x: f32,
    /// First endpoint's local Y coordinate.
    ///
    ///  Vertical position of the hitbox's first point.
    pub y: f32,
    /// First endpoint's local Z coordinate.
    ///
    ///  Depth of the hitbox's first point.
    pub z: f32,
    /// Padding word in the pinned native structure.
    ///
    ///  Unused bytes that preserve the game layout.
    pub padding: u32,
    /// Optional second endpoint's local X coordinate.
    ///
    ///  Horizontal position of the hitbox's other point.
    pub x2: f32,
    /// Optional second endpoint's local Y coordinate.
    ///
    ///  Vertical position of the hitbox's other point.
    pub y2: f32,
    /// Optional second endpoint's local Z coordinate.
    ///
    ///  Depth of the hitbox's other point.
    pub z2: f32,
    /// Second padding word in the pinned native structure.
    ///
    ///  Unused bytes that preserve the game layout.
    pub padding2: u32,
    /// Native attack power value.
    ///
    ///  The attack's damage amount.
    pub power: f32,
    /// Native hitbox radius/size.
    ///
    ///  How large the hitbox is.
    pub size: f32,
    /// Native launch-angle setting.
    ///
    ///  The direction the attack launches a target.
    pub angle: i32,
    /// Knockback growth parameter.
    ///
    ///  How much the attack's knockback scales with damage.
    pub kbg: i32,
    /// Fixed knockback parameter.
    ///
    ///  The attack's fixed knockback component.
    pub fkb: i32,
    /// Base knockback parameter.
    ///
    ///  The attack's baseline knockback component.
    pub bkb: i32,
    /// Native slip chance/value.
    ///
    ///  The attack's tripping value.
    pub slip: f32,
    /// Native hitlag multiplier.
    ///
    ///  How much hit pause the attack creates.
    pub hitlag: f32,
    /// Native SDI multiplier.
    ///
    ///  How much the defender can influence position during hit pause.
    pub sdi: f32,
    /// Native joint hash used to transform local geometry to world space.
    ///
    ///  Identifies the body part the hitbox follows.
    pub node: u64,
    /// Native attack situation mask.
    ///
    ///  Says whether the hitbox applies to grounded or airborne targets.
    pub check_type: u16,
    /// Native attack situation/category mask.
    ///
    ///  Indicates which target situations this hitbox can affect.
    pub situation: u16,
    // Full AttackData field offsets verified against
    // https://github.com/WuBoytH/smash-rs/blob/master/src/app/modules/attack.rs
    // the pinned SDK describes only the geometry prefix above. Read bytes,
    // not Rust bools, so an unexpected native value cannot cause bool UB.
    /// Opaque bytes covering native attack capability fields before flags.
    ///
    ///  Reserved hitbox data kept only to preserve the game's layout.
    pub capability_padding: [u8; 17],
    /// Native reflectable capability byte.
    ///
    ///  Indicates whether a projectile can be reflected.
    pub reflectable: u8,
    /// Native absorbable capability byte.
    ///
    ///  Indicates whether a projectile can be absorbed.
    pub absorbable: u8,
    /// Native direct-attack capability byte.
    ///
    ///  Indicates whether the attack is a direct hitbox.
    pub direct: u8,
}
/// Expected native byte offset of the hitbox size field.
///
///  Confirms where the hitbox radius is located in game memory.
const ATTACK_GEOMETRY_SIZE_FIELD_OFFSET_BYTES: usize = 0x24;
/// Expected native byte offset of the joint hash field.
///
///  Confirms where the hitbox's attached body part is stored.
const ATTACK_GEOMETRY_NODE_FIELD_OFFSET_BYTES: usize = 0x48;
/// Expected native byte offset of the situation mask field.
///
///  Confirms where the hitbox target-situation flags are stored.
const ATTACK_GEOMETRY_SITUATION_FIELD_OFFSET_BYTES: usize = 0x52;
/// Expected native byte offset of the reflectable capability byte.
///
///  Confirms where the reflectable flag is stored.
const ATTACK_GEOMETRY_REFLECTABLE_FIELD_OFFSET_BYTES: usize = 0x65;
/// Expected native byte offset of the absorbable capability byte.
///
///  Confirms where the absorbable flag is stored.
const ATTACK_GEOMETRY_ABSORBABLE_FIELD_OFFSET_BYTES: usize = 0x66;
/// Expected native byte offset of the direct-attack capability byte.
///
///  Confirms where the direct-hit flag is stored.
const ATTACK_GEOMETRY_DIRECT_FIELD_OFFSET_BYTES: usize = 0x67;
/// Expected size of the native two-float vector.
///
///  Ensures a 2D vector has the size Smash expects.
const VECTOR2_SIZE_BYTES: usize = 8;
/// Expected size of the native three-float vector.
///
///  Ensures a 3D vector has the size Smash expects.
const VECTOR3_SIZE_BYTES: usize = 12;
/// Expected byte size of the copied native attack-geometry prefix.
///
///  Ensures hitbox data matches Smash's memory layout.
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
    ///
    ///  Runs the attack command defined by the current animation script.
    pub fn attack_command(lua_state: u64);
    #[link_name = "\u{1}_ZN3app9sv_system29battle_object_module_accessorEP9lua_State"]
    /// Resolves the battle-object accessor associated with a Lua state.
    ///
    ///  Finds the game object that owns this script state.
    pub fn lua_object(lua_state: u64) -> *mut Object;
    #[link_name = "\u{1}_ZN3app8lua_bind30AttackModule__attack_data_implEPNS_26BattleObjectModuleAccessorEib"]
    /// Returns a pointer to one native attack-geometry record.
    ///
    ///  Gets hitbox data for the requested attack slot.
    pub fn attack_geometry(
        module_accessor: *mut Object,
        attack_slot: i32,
        use_secondary_slot: bool,
    ) -> *const AttackGeometry;
    #[link_name = "\u{1}_ZN3app8lua_bind51ModelModule__joint_global_position_with_offset_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40ERKNS3_8Vector3fERS5_b"]
    /// Transforms a local joint-relative offset into world coordinates.
    ///
    ///  Finds where a point attached to a body part is in the stage.
    pub fn joint_position(
        module_accessor: *mut Object,
        joint_hash: u64,
        local_offset: *const Vector3,
        world_position: *mut Vector3,
        apply_transform: bool,
    );
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_z_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native depth coordinate.
    ///
    ///  Gets how far forward or back the object is.
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
    ///
    ///  Checks whether Smash is in Training Mode.
    pub fn is_training_mode() -> bool;
    // Capture-link queries, checked against the pinned cpp.rs bindings.
    // LINK_NO_CAPTURE = 0. Resolve IDs anew; never retain the returned accessor.
    #[link_name = "\u{1}_ZN3app8lua_bind24LinkModule__is_link_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Tests whether the object has a native link of the requested kind.
    ///
    ///  Checks whether this object is linked to another object in that way.
    pub fn is_link(module_accessor: *mut Object, link_kind: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35LinkModule__get_node_object_id_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads the object ID associated with a native link slot.
    ///
    ///  Gets the ID of the object connected through this link.
    pub fn link_object_id(module_accessor: *mut Object, link_kind: i32) -> u64;
    // Identity, state, and timers used to decide whether assistance applies.
    #[link_name = "\u{1}_ZN3app8lua_bind36ControlModule__get_clatter_time_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads the native clatter timer for a specified clatter kind.
    ///
    ///  Gets the remaining mash/escape timer for the requested state.
    pub fn clatter_time(module_accessor: *mut Object, clatter_kind: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind30StatusModule__status_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's current native status ID.
    ///
    ///  Gets the fighter's current action or state code.
    pub fn status_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind33StatusModule__situation_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's current native situation ID.
    ///
    ///  Gets whether the object is grounded, airborne, or elsewhere.
    pub fn situation_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind38StatusModule__prev_situation_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's previous native situation ID.
    ///
    ///  Gets the ground/air situation from before the current one.
    pub fn previous_situation_kind(module_accessor: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24StopModule__is_stop_implEPNS_26BattleObjectModuleAccessorE"]
    /// Queries whether native stop is active for this object.
    ///
    ///  Checks whether the object is currently frozen by game stop.
    pub fn is_stop(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__is_flag_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads a native work-module flag by ID.
    ///
    ///  Checks whether a particular game-state flag is set.
    pub fn is_flag(module_accessor: *mut Object, flag: i32) -> bool;
    // Same pinned bindings: damage hitstop distinguishes the victim from an
    // attacker's hitlag and from shield/global freeze. Remaining time dedups SDI.
    #[link_name = "\u{1}_ZN3app8lua_bind26StopModule__is_damage_implEPNS_26BattleObjectModuleAccessorE"]
    /// Checks whether this object is in damage stop.
    ///
    ///  Tells whether the fighter is frozen by being hit.
    pub fn is_damage_stop(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind40StopModule__get_hit_stop_real_frame_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads remaining native hitstop frames for the object.
    ///
    ///  Gets how much hit pause remains.
    pub fn hitstop_remaining(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app16sv_battle_object4kindEj"]
    /// Resolves a fighter-kind ID from a battle-object ID.
    ///
    ///  Finds which character owns this fighter object.
    pub fn fighter_kind(battle_object_id: u32) -> i32;
    // Read-only ledge observations, verified in the same pinned cpp.rs bindings.
    #[link_name = "\u{1}_ZN3app8lua_bind22PostureModule__lr_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native facing-direction scalar.
    ///
    ///  Gets which direction the fighter faces.
    pub fn facing(module_accessor: *mut Object) -> f32;
    // Read-only size/floor checks for the narrow Kirby duck allowlist, verified
    // against the same pinned cpp.rs. Down input must not drop through a platform.
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__scale_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native model scale.
    ///
    ///  Gets the object's current size multiplier.
    pub fn scale(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind37GroundModule__is_passable_ground_implEPNS_26BattleObjectModuleAccessorE"]
    /// Queries whether the object is on passable ground.
    ///
    ///  Checks whether the platform beneath the fighter can be dropped through.
    pub fn passable_ground(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind24TeamModule__team_no_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native team identifier.
    ///
    ///  Gets which team this object belongs to.
    pub fn team(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind32HitModule__get_total_status_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads one native hit-status value by index.
    ///
    ///  Gets the requested invulnerability or hit-status state.
    pub fn hit_status(module_accessor: *mut Object, hit_status_index: i32) -> u64;
    // Status and motion are different: "ledge roll" is a status, while motion
    // identifies the character's animation and its current fractional frame.
    // Policy uses both so repeated input polls do not imply animation progress.
    // Read animation timing for ledge prediction and invincibility expiry.
    #[link_name = "\u{1}_ZN3app8lua_bind30MotionModule__motion_kind_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the current native motion hash.
    ///
    ///  Identifies the animation currently playing.
    pub fn motion_kind(module_accessor: *mut Object) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind24MotionModule__frame_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the current fractional frame within the native motion.
    ///
    ///  Gets how far the current animation has progressed.
    pub fn motion_frame(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind23MotionModule__rate_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native motion playback rate.
    ///
    ///  Gets how fast the current animation is playing.
    pub fn motion_rate(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind28MotionModule__end_frame_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native end-frame value for the current motion.
    ///
    ///  Gets the animation's configured ending frame.
    pub fn motion_end_frame(module_accessor: *mut Object) -> f32;
    // Infliction asks whether Kirby registered a hit; is_attack asks whether
    // an opponent currently has an active attack slot. Starting an animation
    // alone proves neither a successful hit nor a safely ended get-up attack.
    // Fresh collision on this update; unlike is_infliction_status, this cannot
    // carry an earlier fair hit forward into a later hitbox interval.
    #[link_name = "\u{1}_ZN3app8lua_bind32AttackModule__is_infliction_implEPNS_26BattleObjectModuleAccessorEj"]
    /// Queries whether the specified collision type occurred now.
    ///
    ///  Checks whether this object hit or contacted something this update.
    pub fn attack_infliction_now(module_accessor: *mut Object, collision_mask: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind28AttackModule__is_attack_implEPNS_26BattleObjectModuleAccessorEib"]
    /// Checks whether the indexed native attack slot is active.
    ///
    ///  Checks whether the requested hitbox is currently active.
    pub fn attack_active(module_accessor: *mut Object, attack_slot: i32, alternate: bool) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35StatusModule__prev_status_kind_implEPNS_26BattleObjectModuleAccessorEj"]
    /// Reads a prior status from the indexed native history entry.
    ///
    ///  Gets an earlier fighter state from the status history.
    pub fn previous_status(module_accessor: *mut Object, history_index: u32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind24WorkModule__get_int_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads a signed integer from the object's native work module.
    ///
    ///  Gets an integer game-state value by its ID.
    pub fn work_int(module_accessor: *mut Object, work_id: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind26WorkModule__get_float_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads a floating-point value from the object's native work module.
    ///
    ///  Gets a decimal game-state value by its ID.
    pub fn work_float(module_accessor: *mut Object, work_id: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind42WorkModule__is_enable_transition_term_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Queries whether a native transition term is enabled.
    ///
    ///  Checks whether the game currently allows the requested transition.
    pub fn transition_enabled(module_accessor: *mut Object, transition_term: i32) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35CancelModule__is_enable_cancel_implEPNS_26BattleObjectModuleAccessorE"]
    #[allow(dead_code)] // Used by the parked offline whiff-observation prototype.
    /// Reads the native cancel-enabled flag for an object.
    ///
    ///  Checks whether the fighter can cancel its current action.
    pub fn cancel_enabled(module_accessor: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind36GroundModule__hang_cliff_pos_3f_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the native cliff position in world coordinates.
    ///
    ///  Gets the location of the nearby ledge point.
    pub fn cliff_position(module_accessor: *mut Object) -> Vector3;
    #[link_name = "\u{1}_ZN3app8lua_bind50FighterMotionModuleImpl__get_hit_normal_frame_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Eb"]
    /// Reads native normal-hit timing for a specified motion.
    ///
    ///  Gets when the move's normal hit timing occurs.
    pub fn normal_hit_frame(module_accessor: *mut Object, motion_hash: u64, alternate: bool)
        -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind46FighterMotionModuleImpl__get_cancel_frame_implEPNS_26BattleObjectModuleAccessorEN3phx6Hash40Eb"]
    /// Reads the native cancel-frame value for a specified motion.
    ///
    ///  Gets the frame when that move can be canceled.
    pub fn cancel_frame(module_accessor: *mut Object, motion_hash: u64, alternate: bool) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind32WorkModule__get_param_float_implEPNS_26BattleObjectModuleAccessorEmm"]
    /// Reads a native floating-point parameter by table and key.
    ///
    ///  Gets a configured decimal value from game parameter data.
    pub fn param_float(
        module_accessor: *mut Object,
        parameter_table_hash: u64,
        parameter_hash: u64,
    ) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind30WorkModule__get_param_int_implEPNS_26BattleObjectModuleAccessorEmm"]
    /// Reads a native integer parameter by table and key.
    ///
    ///  Gets a configured whole number from game parameter data.
    pub fn param_int(
        module_accessor: *mut Object,
        parameter_table_hash: u64,
        parameter_hash: u64,
    ) -> i32;
    // WorkModule keys and transition-term IDs are version-specific constants
    // supplied by callers. They are not config values or offsets to invent.
    // Motion/position queries feed prediction; the mod never writes velocity.
    #[link_name = "\u{1}_ZN3app8lua_bind35KineticModule__get_sum_speed_x_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads horizontal speed for the selected kinetic reserve.
    ///
    ///  Gets the object's left/right movement speed.
    pub fn speed_x(module_accessor: *mut Object, reserve_type: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind35KineticModule__get_sum_speed_y_implEPNS_26BattleObjectModuleAccessorEi"]
    /// Reads vertical speed for the selected kinetic reserve.
    ///
    ///  Gets the object's up/down movement speed.
    pub fn speed_y(module_accessor: *mut Object, reserve_type: i32) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_x_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native X position.
    ///
    ///  Gets its horizontal location.
    pub fn pos_x(module_accessor: *mut Object) -> f32;
    #[link_name = "\u{1}_ZN3app8lua_bind25PostureModule__pos_y_implEPNS_26BattleObjectModuleAccessorE"]
    /// Reads the object's native Y position.
    ///
    ///  Gets its vertical location.
    pub fn pos_y(module_accessor: *mut Object) -> f32;
    // ray_check receives an origin and a displacement (the ray's travel),
    // rather than two endpoints. Both Vector2 values must stay alive for the call.
    #[link_name = "\u{1}_ZN3app8lua_bind28GroundModule__ray_check_implEPNS_26BattleObjectModuleAccessorERKN3phx8Vector2fES6_b"]
    /// Performs the native ground-ray query underlying [`ray_check`].
    ///
    ///  Checks whether a short line reaches stage ground.
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
    ///
    ///  Finds the live game object for this ID; callers revalidate it.
    pub fn module_accessor(object_id: u32) -> *mut Object;
    #[link_name = "\u{1}_ZN3app16sv_battle_object9is_activeEj"]
    /// Returns whether the specified battle object ID corresponds to an active object.
    /// Checks whether the battle-object ID currently names an active object.
    ///
    ///  Checks whether that game object is still alive and present.
    pub fn is_active(battle_object_id: u32) -> bool;
    // Enumerate current fighters directly. Ledge opponents may not poll the
    // hooked command category while hanging, so callbacks alone are insufficient.
    #[link_name = "\u{1}_ZN3app8lua_bind32FighterManager__entry_count_implEPNS_14FighterManagerE"]
    /// Reads the fighter-entry count from the native manager.
    ///
    ///  Gets how many player entries are in the current match.
    pub fn entry_count(manager: *mut Object) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind33FighterManager__get_entry_id_implEPNS_14FighterManagerEi"]
    /// Resolves a fighter ordinal to its native player-entry ID.
    ///
    ///  Gets the player slot for a fighter-list position.
    pub fn entry_id(manager: *mut Object, fighter_ordinal: i32) -> i32;
    #[link_name = "\u{1}_ZN3app8lua_bind38FighterManager__get_fighter_entry_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    /// Reads the native fighter-entry pointer value for an entry ID.
    ///
    ///  Finds the game record for a player's fighter slot.
    pub fn fighter_entry(manager: *mut Object, entry_id: i32) -> u64;
    #[link_name = "\u{1}_ZN3app8lua_bind37FighterEntry__current_fighter_id_implEPNS_12FighterEntryE"]
    /// Reads the current fighter object ID from a native entry.
    ///
    ///  Gets which fighter currently occupies this player slot.
    pub fn current_fighter_id(fighter_entry: *mut Object) -> u64;
    // Manager queries exclude CPU-controlled actors and result-screen activity.
    #[link_name = "\u{1}_ZN3app8lua_bind44FighterManager__get_fighter_information_implEPNS_14FighterManagerENS_14FighterEntryIDE"]
    /// Resolves native fighter-information data for a player entry.
    ///
    ///  Finds the game record that describes this player's fighter.
    pub fn fighter_information(manager: *mut Object, entry_id: i32) -> *mut Object;
    #[link_name = "\u{1}_ZN3app8lua_bind41FighterInformation__is_operation_cpu_implEPNS_18FighterInformationE"]
    /// Reads the CPU-control flag from native fighter information.
    ///
    ///  Checks whether this fighter is controlled by the game.
    pub fn is_cpu(fighter_information: *mut Object) -> bool;
    #[link_name = "\u{1}_ZN3app8lua_bind35FighterManager__is_result_mode_implEPNS_14FighterManagerE"]
    /// Queries whether the battle manager is in the results scene.
    ///
    ///  Checks whether the match has moved to its results screen.
    pub fn is_result(manager: *mut Object) -> bool;
    // Pinned cpp.rs: FighterManager::is_ready_go, read-only battle phase.
    #[link_name = "\u{1}_ZN3app8lua_bind32FighterManager__is_ready_go_implEPNS_14FighterManagerE"]
    /// Queries the manager's native Ready-Go state.
    ///
    ///  Checks whether the match has begun.
    pub fn is_ready_go(manager: *mut Object) -> bool;
}
