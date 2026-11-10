//! Observation contracts passed from the game adapter to policies.

pub mod battle_objects {
    //! Bounded copied identities and candidate selection; no native pointers.
    /// Maximum number of fighter current slots in the roster.
    ///
///  Maximum number of fighter current slots in the roster.
    pub const FIGHTER_CAPACITY: usize = 16;
    /// Object category for fighters.
    ///
///  Object category for fighters.
    pub const OBJECT_CATEGORY_FIGHTER: u32 = 0;
    /// Object category for weapons.
    ///
///  Object category for weapons.
    pub const OBJECT_CATEGORY_WEAPON: u32 = 1;
    /// Object category for items.
    ///
///  Object category for items.
    pub const OBJECT_CATEGORY_ITEM: u32 = 4;
    /// Weapon kind for Pikmin Pikmin.
    ///
///  Weapon kind for Pikmin Pikmin.
    pub const WEAPON_KIND_PIKMIN_PIKMIN: i32 = 0x9d;
    /// Weapon kind for Rosetta Tico.
    ///
///  Weapon kind for Rosetta Tico.
    pub const WEAPON_KIND_ROSETTA_TICO: i32 = 0xe6;
    /// Bitshift amount (28) for extracting the category from an object ID.
    /// That label is a 32-bit number.
    /// Smash stores the object’s category in the first four bits and other identifying information in the remaining 28
    ///
///  Bitshift amount (28) for extracting the category from an object ID. That label is a 32-bit number. Smash stores the object’s category in the first four bits and other identifying information in the remaining 28.
    pub const BITSHIFT_CATEGORY: u32 = 28;
    /// Array holding current fighters in game.
    /// Each slot can hold an optional fighter ID; empty slots are None.
    ///
///  Array holding current fighters in game. Each slot can hold an optional fighter ID; empty slots are None.
    pub type FightersInGame = [Option<u32>; FIGHTER_CAPACITY];

    /// Recieves a mutable reference to a fighter roster and a fighter ID to append.
    ///
///  Recieves a mutable reference to a fighter roster and a fighter ID to append.
    pub fn append_fighter(roster: &mut FightersInGame, id: u32) {
        // Ignore invalid IDs or duplicates.
        if id >> BITSHIFT_CATEGORY != OBJECT_CATEGORY_FIGHTER || roster.contains(&Some(id)) {
            return;
        }
        if let Some(slot) = roster.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(id);
        }
    }

    /// Extra actors must not turn roster ordinals into indices of eight-slot
    /// policy arrays. Keep the nearest relevant candidates when those arrays fill.
    ///
///  Extra actors must not turn roster ordinals into indices of eight-slot policy arrays. Keep the nearest relevant candidates when those arrays fill.
    pub fn insert_nearest<T: Copy, const N: usize>(
        targets: &mut [Option<T>; N],
        candidate: T,
        distance: impl Fn(T) -> f32,
    ) {
        let candidate_distance = distance(candidate);
        if !candidate_distance.is_finite() || candidate_distance < 0. {
            return;
        }
        if let Some(slot) = targets.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(candidate);
            return;
        }
        let Some((farthest, far_distance)) = targets
            .iter()
            .enumerate()
            .filter_map(|(i, t)| t.map(|t| (i, distance(t))))
            .max_by(|a, b| a.1.total_cmp(&b.1))
        else {
            return;
        };
        if candidate_distance < far_distance {
            targets[farthest] = Some(candidate);
        }
    }

    /// Returns the `distance_squared` result.
    ///
    ///
///  Gets the distance squared result.
    pub fn distance_squared(from: [f32; 2], to: [f32; 2]) -> f32 {
        (to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2)
    }

    /// Defines the available `ObjectCategory` cases.
    ///
    ///
///  Lists the available ObjectCategory choices.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u32)]
    pub enum ObjectCategory {
        /// Represents the `Fighter` case of `ObjectCategory`.
        ///
        ///
///  Represents the Fighter option in ObjectCategory.
        Fighter = OBJECT_CATEGORY_FIGHTER,
        /// Represents the `Weapon` case of `ObjectCategory`.
        ///
        ///
///  Represents the Weapon option in ObjectCategory.
        Weapon = OBJECT_CATEGORY_WEAPON,
        /// Represents the `Item` case of `ObjectCategory`.
        ///
        ///
///  Represents the Item option in ObjectCategory.
        Item = OBJECT_CATEGORY_ITEM,
        // ...
    }

    impl ObjectCategory {
        /// Constructs a value from the id.
        ///
        ///
///  Constructs a value from the id.
        pub const fn from_id(id: u32) -> Option<Self> {
            match id >> BITSHIFT_CATEGORY {
                OBJECT_CATEGORY_FIGHTER => Some(Self::Fighter),
                OBJECT_CATEGORY_WEAPON => Some(Self::Weapon),
                OBJECT_CATEGORY_ITEM => Some(Self::Item),
                _ => None,
            }
        }
        /// check to ensure the weapon is not a non-projectile type like Rosetta & Tico,
        /// and that Pikmin are not already attached or in a landing state
        ///
///  check to ensure the weapon is not a non-projectile type like Rosetta & Tico, and that Pikmin are not already attached or in a landing state.
        pub fn is_projectile(id: u32, kind: i32, status: i32) -> bool {
            match Self::from_id(id) {
                Some(Self::Weapon) => {
                    kind != WEAPON_KIND_ROSETTA_TICO
                        && !(kind == WEAPON_KIND_PIKMIN_PIKMIN
                            && matches!(
                                status,
                                crate::native_states::status::LANDING_LIGHT
                                    ..=crate::native_states::status::LANDING_FALL_SPECIAL
                            ))
                }

                Some(Self::Item) => true,

                _ => false,
            }
        }
    }

    /// Entry-based damage attribution cannot distinguish two Ice Climbers on
    /// opposite sides. Preserve agreement; return no invented side on conflict.
    ///
///  Entry-based damage attribution cannot distinguish two Ice Climbers on opposite sides. Preserve agreement; return no invented side on conflict.
    #[derive(Default)]
    pub struct EntryDirection {
        /// Stores the `side` value in `EntryDirection` as `Option<f32>`.
        ///
        ///
///  Optionally stores the side for EntryDirection; it may be absent.
        side: Option<f32>,
        /// Stores the `conflict` value in `EntryDirection` as `bool`.
        ///
        ///
///  Holds the conflict value used by EntryDirection.
        conflict: bool,
    }
    impl EntryDirection {
        /// Records the time of the latest fighter observation.
        ///
        ///
///  Remembers when a fighter was last seen.
        pub fn observe(&mut self, direction: f32) {
            if !direction.is_finite() || self.side.is_some_and(|side| side != direction) {
                self.conflict = true;
            }
            self.side = Some(direction);
        }
        /// Returns an optional `direction` value.
        ///
        ///
///  Gets direction if it is available.
        pub fn direction(&self) -> Option<f32> {
            if self.conflict {
                None
            } else {
                self.side
            }
        }
    }
}
pub mod gameplay {
    //! Shared gameplay observations and controller-input requests.
    //!
    //! These values are copied between automation policies and the live adapter.
    //! They contain no game pointers and do not write native fighter state.

    /// Maximum number of animation forecast frames considered for timing calculations.
    ///
///  Maximum number of animation forecast frames considered for timing calculations.
    const MAX_ANIMATION_FORECAST_FRAMES: f32 = 120.0;

    /// Read-only animation timing; remaining values are measured in game frames.
    /// This struct captures the timing information for a single animation,
    ///  including its current frame, rate of progress, and remaining duration.
    ///
///  Read-only animation timing; remaining values are measured in game frames. This struct captures the timing information for a single animation, including its current frame, rate of progress, and remaining duration.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MotionTiming {
        /// Animation Hash40 identifier, not a status ID or frame count.
        ///
///  Animation Hash40 identifier, not a status ID or frame count.
        pub kind: u64,
        /// Animation frame, which can be fractional.
        ///
///  Animation frame, which can be fractional.
        pub frame: f32,
        /// Converts animation progress into game-frame time.
        ///
///  Converts animation progress into game-frame time.
        pub rate: f32,
        /// None means unknown; Some(0.0) means the animation already ended.
        ///
///  None means unknown; Some(0.0) means the animation already ended.
        pub remaining: Option<f32>,
    }

    /// Convert an animation-frame deadline into game-frame time.
    ///
    /// Invalid or past deadlines remain unknown instead of being treated as ready.
    ///
///  Convert an animation-frame deadline into game-frame time. Invalid or past deadlines remain unknown instead of being treated as ready.
    pub fn frames_until(frame: f32, deadline: f32, rate: f32) -> Option<f32> {
        if !frame.is_finite()
            || frame < 0.0
            || !deadline.is_finite()
            || !rate.is_finite()
            || rate <= 0.0
        {
            return None;
        }
        let remaining = (deadline - frame) / rate;
        (deadline > 0.0 && (0.0..=MAX_ANIMATION_FORECAST_FRAMES).contains(&remaining))
            .then_some(remaining)
    }

    /// Validated native air kinetics for short movement forecasts.
    ///
///  Validated native air kinetics for short movement forecasts.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AirPhysics {
        /// Stores the `gravity` value in `AirPhysics` as `f32`.
        ///
        ///
///  Holds the gravity value used by AirPhysics.
        pub gravity: f32,
        /// Stores the `terminal_speed` value in `AirPhysics` as `f32`.
        ///
        ///
///  Holds the terminal speed value used by AirPhysics.
        pub terminal_speed: f32,
    }
    impl AirPhysics {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
///  Creates a new value.
        pub fn new(gravity: f32, terminal_speed: f32) -> Option<Self> {
            (gravity.is_finite()
                && (0.001..=2.0).contains(&gravity)
                && terminal_speed.is_finite()
                && (0.1..=10.0).contains(&terminal_speed))
            .then_some(Self {
                gravity,
                terminal_speed,
            })
        }
    }

    /// Return time to the earlier animation end or native cancel deadline.
    ///
///  Return time to the earlier animation end or native cancel deadline.
    pub fn recovery_frames(frame: f32, end: f32, cancel: f32, rate: f32) -> Option<f32> {
        let end = frames_until(frame, end, rate)?;
        if cancel.is_finite() && cancel > 0.0 {
            if cancel >= frame + end * rate {
                return Some(end);
            }
            if cancel <= frame {
                return Some(0.0);
            }
            Some(frames_until(frame, cancel, rate)?.min(end))
        } else {
            Some(end)
        }
    }

    /// Read-only opponent facts copied by the platform adapter.
    ///
///  Read-only opponent facts copied by the platform adapter.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Opponent {
        /// Stores the `id` value in `Opponent` as `u32`.
        ///
        ///
///  Holds the id value used by Opponent.
        pub id: u32,
        /// Stores the `status` value in `Opponent` as `i32`.
        ///
        ///
///  Holds the status value used by Opponent.
        pub status: i32,
        /// Stores the `previous_status` value in `Opponent` as `i32`.
        ///
        ///
///  Holds the previous status value used by Opponent.
        pub previous_status: i32,
        /// Stores the `situation` value in `Opponent` as `i32`.
        ///
        ///
///  Holds the situation value used by Opponent.
        pub situation: i32,
        /// Stores the `position` value in `Opponent` as `[f32; 2]`.
        ///
        ///
///  Stores the position collection for Opponent.
        pub position: [f32; 2],
        /// Aggregate native hit status, not an individual hurtbox model.
        ///
///  Aggregate native hit status, not an individual hurtbox model.
        pub vulnerable: bool,
        /// Stores the `velocity` value in `Opponent` as `[f32; 2]`.
        ///
        ///
///  Stores the velocity collection for Opponent.
        pub velocity: [f32; 2],
        /// Only ledge policies use this fallback anchor when the fighter hangs.
        ///
///  Only ledge policies use this fallback anchor when the fighter hangs.
        pub cliff_position: Option<[f32; 2]>,
        /// Sanitized native motion metadata; None means prediction is unavailable.
        ///
///  Sanitized native motion metadata; None means prediction is unavailable.
        pub vulnerable_in: Option<f32>,
        /// Stores the `motion` value in `Opponent` as `Option<MotionTiming>`.
        ///
        ///
///  Optionally stores the motion for Opponent; it may be absent.
        pub motion: Option<MotionTiming>,
        /// Present for get-up attacks; None preserves conservative block-only behavior.
        ///
///  Present for get-up attacks; None preserves conservative block-only behavior.
        pub attack_active: Option<bool>,
        /// Stores the `air_physics` value in `Opponent` as `Option<AirPhysics>`.
        ///
        ///
///  Optionally stores the air physics for Opponent; it may be absent.
        pub air_physics: Option<AirPhysics>,
        /// Native cancel deadline, separate from intermediate jump animation ends.
        ///
///  Native cancel deadline, separate from intermediate jump animation ends.
        pub actionable_in: Option<f32>,
    }

    /// Controlled fighter's native state, copied by the platform adapter.
    ///
///  Controlled fighter's native state, copied by the platform adapter.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct GameplayContext {
        /// Whether the controlled fighter is currently holding an input.
        ///
///  Whether the controlled fighter is currently holding an input.
        pub held: bool,
        /// Whether the controlled fighter is currently in active gameplay.
        ///
///  Whether the controlled fighter is currently in active gameplay.
        pub gameplay: bool,
        /// Changes only on fresh physical gameplay input, never simulated output.
        ///
///  Changes only on fresh physical gameplay input, never simulated output.
        pub manual_input_epoch: u64,
        /// Native readiness for this consumer's intended grounded action.
        ///
///  Native readiness for this consumer's intended grounded action.
        pub can_act: bool,
        /// The assist may own inputs, including during its guard/grab/tilt animation.
        ///
///  The assist may own inputs, including during its guard/grab/tilt animation.
        pub can_control: bool,
        /// Native hitstop: preserve an existing shield, but start no new actions.
        ///
///  Native hitstop: preserve an existing shield, but start no new actions.
        pub stopped: bool,
        /// The controlled fighter's current status kind.
        ///
///  The controlled fighter's current status kind.
        pub status: i32,
        /// The controlled fighter's current situation kind.
        ///
///  The controlled fighter's current situation kind.
        pub situation: i32,
        /// Current animation frame. Together with the motion kind it distinguishes
        /// progress when a target holds one pose, and is part of duplicate-observation reuse.
        ///
///  Current animation frame. Together with the motion kind it distinguishes progress when a target holds one pose, and is part of duplicate-observation reuse.
        pub motion_frame: f32,
        /// Current motion (Hash40) identifier.
        ///
///  Current motion (Hash40) identifier.
        pub motion_kind: u64,
        /// Current animation rate.
        ///
///  Current animation rate.
        pub motion_rate: f32,
        /// Current position in world coordinates.
        ///
///  Current position in world coordinates.
        pub position: [f32; 2],
        /// Current velocity in world coordinates.
        ///
///  Current velocity in world coordinates.
        pub velocity: [f32; 2],
        /// Native ground deceleration; None avoids assuming a stopping distance.
        ///
///  Native ground deceleration; None avoids assuming a stopping distance.
        pub ground_brake: Option<f32>,
        /// Facing uses its sign (+right/-left); position/velocity remain absolute
        /// world axes. Multiplying dx by facing asks if the target is in front.
        ///
///  Facing uses its sign (+right/-left); position/velocity remain absolute world axes. Multiplying dx by facing asks if the target is in front.
        pub facing: f32,
        /// The controlled fighter's previous status kind.
        ///
///  The controlled fighter's previous status kind.
        pub previous_status: i32,
        /// Whether the controlled fighter can currently perform a grab.
        ///
///  Whether the controlled fighter can currently perform a grab.
        pub can_grab: bool,
        /// Whether the controlled fighter can currently perform a tilt.
        ///
///  Whether the controlled fighter can currently perform a tilt.
        pub can_tilt: bool,
        /// Whether the controlled fighter can currently perform a down tilt.
        ///
///  Whether the controlled fighter can currently perform a down tilt.
        pub can_down_tilt: bool,
        /// Whether the controlled fighter can currently perform a down special.
        ///
///  Whether the controlled fighter can currently perform a down special.
        pub can_down_special: bool,
        /// Whether the controlled fighter can currently perform a dash attack.
        ///
///  Whether the controlled fighter can currently perform a dash attack.
        pub can_dash_attack: bool,
        /// Whether the controlled fighter can currently perform a down smash.
        ///
///  Whether the controlled fighter can currently perform a down smash.
        pub can_down_smash: bool,
        /// Explicit additional delay in game frames; live observations supply zero.
        ///
///  Explicit additional delay in game frames; live observations supply zero.
        pub input_delay: f32,
        /// Whether the controlled fighter can currently perform a jump.
        ///
///  Whether the controlled fighter can currently perform a jump.
        pub can_jump: bool,
        /// Whether the controlled fighter can currently perform an air attack.
        ///
///  Whether the controlled fighter can currently perform an air attack.
        pub can_air_attack: bool,
        /// The controlled fighter's current air physics, if applicable.
        ///
///  The controlled fighter's current air physics, if applicable.
        pub air_physics: Option<AirPhysics>,
        /// The controlled fighter's short hop speed, if applicable.
        ///
///  The controlled fighter's short hop speed, if applicable.
        pub short_hop_speed: Option<f32>,
        /// The controlled fighter's full hop speed, if applicable.
        ///
///  The controlled fighter's full hop speed, if applicable.
        pub full_hop_speed: Option<f32>,
        /// The number of frames the controlled fighter is in shield stun, if applicable.
        ///
///  The number of frames the controlled fighter is in shield stun, if applicable.
        pub shield_stun_frames: Option<i32>,
        /// The ID of the attacker causing the shield stun, if applicable.
        ///
///  The ID of the attacker causing the shield stun, if applicable.
        pub shield_attacker: Option<u32>,
        /// Native hit infliction during Kirby's current assisted attack.
        ///
///  Native hit infliction during Kirby's current assisted attack.
        pub hit_confirmed: bool,
        /// Ground support along a short swept lookahead, left then right.
        ///
///  Ground support along a short swept lookahead, left then right.
        pub safe_ground: [bool; 2],
    }

    /// Proposed controller inputs; the live adapter arbitrates and maps the request.
    ///
///  Proposed controller inputs; the live adapter arbitrates and maps the request.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct ControllerOutput {
        /// Whether to consume the trigger for this update.
        ///
///  Whether to consume the trigger for this update.
        pub consume_trigger: bool,
        /// The desired stick position for this update, if any.
        ///
///  The desired stick position for this update, if any.
        pub stick: Option<[f32; 2]>,
        /// Buttons to hold for this update (a mask of native controller bits).
        ///
///  Buttons to hold for this update (a mask of native controller bits).
        pub requested: u32,
        /// Buttons to replace, including buttons that should currently be released.
        ///
///  Buttons to replace, including buttons that should currently be released.
        pub owned: u32,
        /// Preserve a committed short-hop decision with this cached output until mapped.
        ///
///  Preserve a committed short-hop decision with this cached output until mapped.
        pub native_short_hop: bool,
    }
    impl ControllerOutput {
        /// No proposal; usable in `const` contexts where `Default` is not.
        ///
///  No proposal; usable in const contexts where Default is not.
        pub const NONE: Self = Self {
            consume_trigger: false,
            stick: None,
            requested: 0,
            owned: 0,
            native_short_hop: false,
        };
    }
}
pub mod observation {
    //! One observation's lazy, fixed-size fighter facts. Never retain native pointers
    //! or reuse these facts across callbacks. Readiness queries remain live; the
    //! separate floor cache only reuses an identical query inside this observation.
    use std::cell::Cell;

    /// Input proposals belong to exactly one selected-controller update. Wall-clock
    /// freshness is only a secondary timeout; it cannot make an old packet current.
    ///
///  Input proposals belong to exactly one selected-controller update. Wall-clock freshness is only a secondary timeout; it cannot make an old packet current.
    pub fn input_update_current(evaluated: Option<i64>, current: i64) -> bool {
        evaluated == Some(current)
    }

    /// Reject setup that began before a settings/cancellation fence or mapper update.
    ///
///  Reject setup that began before a settings/cancellation fence or mapper update.
    pub fn observation_inputs_current(
        captured_sequence: u64,
        current_sequence: u64,
        captured_tick: i64,
        current_tick: i64,
    ) -> bool {
        captured_sequence == current_sequence && input_update_current(Some(captured_tick), current_tick)
    }

    /// A command callback may upgrade a mapper-only sample once, including when
    /// no policy needs a roster. Discovery capability is distinct from roster data.
    ///
///  A command callback may upgrade a mapper-only sample once, including when no policy needs a roster. Discovery capability is distinct from roster data.
    #[derive(Clone, Copy, Default)]
    pub struct ObservationSetup {
        /// Stores the `discovered` value in `ObservationSetup` as `Option<i64>`.
        ///
        ///
///  Optionally stores the discovered for ObservationSetup; it may be absent.
        discovered: Option<i64>,
    }
    impl ObservationSetup {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
///  Creates a new value.
        pub const fn new() -> Self {
            Self { discovered: None }
        }
        /// Determines whether the `needed` condition holds.
        ///
        ///
///  Checks whether needed.
        pub fn needed(&self, tick: i64, evaluated: bool, discover: bool) -> bool {
            !evaluated || (discover && self.discovered != Some(tick))
        }
        /// Updates the `complete` state using the supplied inputs.
        ///
        ///
///  Updates complete.
        pub fn complete(&mut self, tick: i64, discover: bool) {
            if discover {
                self.discovered = Some(tick);
            }
        }
        /// Clears the remembered fighter observation.
        ///
        ///
///  Forgets the last fighter observation.
        pub fn clear(&mut self) {
            self.discovered = None;
        }
    }

    /// Exact floor-query reuse within ONE synchronous observation. Store copied
    /// values, never a pointer, and discard at callback exit so moving platforms
    /// and later frames always get new collision queries.
    ///
///  Exact floor-query reuse within ONE synchronous observation. Store copied values, never a pointer, and discard at callback exit so moving platforms and later frames always get new collision queries.
    type GroundSupportCache = Option<(u32, [f32; 2], f32, [bool; 2])>;

    /// Defines the `GroundSupportSnapshot` data structure.
    ///
    ///
///  Holds the data for GroundSupportSnapshot.
    #[derive(Default)]
    pub struct GroundSupportSnapshot {
        // `Cell` lets this synchronous snapshot cache copied results through a
        // shared reference; it is interior mutability, not cross-thread locking.
        /// Stores the `last` value in `GroundSupportSnapshot` as `Cell<GroundSupportCache>`.
        ///
        ///
///  Holds the last value used by GroundSupportSnapshot.
        last: Cell<GroundSupportCache>,
    }
    impl GroundSupportSnapshot {
        /// Returns the `read` result.
        ///
        ///
///  Gets the read result.
        pub fn read(
            &self,
            object: u32,
            position: [f32; 2],
            vx: f32,
            query: impl FnOnce() -> [bool; 2],
        ) -> [bool; 2] {
            if let Some((id, at, speed, supported)) = self.last.get() {
                if id == object && at == position && speed == vx {
                    return supported;
                }
            }
            let supported = query();
            self.last.set(Some((object, position, vx, supported)));
            supported
        }
    }
    /// Field identifiers for fighter facts. Used as indices into the `facts` array of a `Fighter`.
    ///
///  Field identifiers for fighter facts. Used as indices into the facts array of a Fighter.
    #[derive(Clone, Copy)]
    #[repr(usize)]
    pub enum Field {
        /// Represents the `Status` case of `Field`.
        ///
        ///
///  Represents the Status option in Field.
        Status,
        /// Represents the `Situation` case of `Field`.
        ///
        ///
///  Represents the Situation option in Field.
        Situation,
        /// Represents the `Motion` case of `Field`.
        ///
        ///
///  Represents the Motion option in Field.
        Motion,
        /// Represents the `Frame` case of `Field`.
        ///
        ///
///  Represents the Frame option in Field.
        Frame,
        /// Represents the `Rate` case of `Field`.
        ///
        ///
///  Represents the Rate option in Field.
        Rate,
        /// Represents the `X` case of `Field`.
        ///
        ///
///  Represents the X option in Field.
        X,
        /// Represents the `Y` case of `Field`.
        ///
        ///
///  Represents the Y option in Field.
        Y,
        /// Represents the `Vx` case of `Field`.
        ///
        ///
///  Represents the Vx option in Field.
        Vx,
        /// Represents the `Vy` case of `Field`.
        ///
        ///
///  Represents the Vy option in Field.
        Vy,
        /// Represents the `Facing` case of `Field`.
        ///
        ///
///  Represents the Facing option in Field.
        Facing,
        /// Represents the `Team` case of `Field`.
        ///
        ///
///  Represents the Team option in Field.
        Team,
        /// Represents the `Stop` case of `Field`.
        ///
        ///
///  Represents the Stop option in Field.
        Stop,
        /// Represents the `PreviousStatus` case of `Field`.
        ///
        ///
///  Represents the PreviousStatus option in Field.
        PreviousStatus,
        /// Represents the `EndFrame` case of `Field`.
        ///
        ///
///  Represents the EndFrame option in Field.
        EndFrame,
        /// Represents the `CaptureLink` case of `Field`.
        ///
        ///
///  Represents the CaptureLink option in Field.
        CaptureLink,
        /// Represents the `Kind` case of `Field`.
        ///
        ///
///  Represents the Kind option in Field.
        Kind,
    }
    /// Defines the `FIELDS` constant used by this module.
    ///
    ///
///  Provides the FIELDS constant value.
    const FIELDS: usize = Field::Kind as usize + 1;
    /// Defines the available `Fact` cases.
    ///
    ///
///  Lists the available Fact choices.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Fact {
        /// Represents the `Int` case of `Fact`.
        ///
        ///
///  Represents the Int option in Fact.
        Int(
            /// Carries the observed int fact value in `Fact`.
            ///
            ///
///  Holds the observed int fact value for this `Fact` choice.
            i32,
        ),
        /// Represents the `Motion` case of `Fact`.
        ///
        ///
///  Represents the Motion option in Fact.
        Motion(
            /// Carries the observed motion fact value in `Fact`.
            ///
            ///
///  Holds the observed motion fact value for this `Fact` choice.
            u64,
        ),
        /// Represents the `Float` case of `Fact`.
        ///
        ///
///  Represents the Float option in Fact.
        Float(
            /// Carries the observed float fact value in `Fact`.
            ///
            ///
///  Holds the observed float fact value for this `Fact` choice.
            f32,
        ),
        /// Represents the `Bool` case of `Fact`.
        ///
        ///
///  Represents the Bool option in Fact.
        Bool(
            /// Carries the observed bool fact value in `Fact`.
            ///
            ///
///  Holds the observed bool fact value for this `Fact` choice.
            bool,
        ),
    }
    impl Fact {
        /// Returns an optional `int` value.
        ///
        ///
///  Gets int if it is available.
        fn int(self) -> Option<i32> {
            match self {
                Self::Int(value) => Some(value),
                _ => None,
            }
        }
        /// Returns the motion value when the observation contains one.
        ///
        ///
///  Gets the motion value if it is available.
        fn motion(self) -> Option<u64> {
            match self {
                Self::Motion(value) => Some(value),
                _ => None,
            }
        }
        /// Returns the floating-point value when the fact contains one.
        ///
        ///
///  Gets the number if this fact stores one.
        fn float(self) -> Option<f32> {
            match self {
                Self::Float(value) => Some(value),
                _ => None,
            }
        }
    }
    /// A fighter in the observation snapshot, storing its identity and copied facts.
    ///
///  A fighter in the observation snapshot, storing its identity and copied facts.
    struct Fighter {
        // Cache facts lazily so sharing a snapshot does not force unrelated
        // policies to perform every possible native observation.
        /// Stores the `id` value in `Fighter` as `Cell<Option<u32>>`.
        ///
        ///
///  Holds the id value used by Fighter.
        id: Cell<Option<u32>>,
        /// Stores the `facts` value in `Fighter` as `[Cell<Option<Fact>>; FIELDS]`.
        ///
        ///
///  Stores the facts collection for Fighter.
        facts: [Cell<Option<Fact>>; FIELDS],
    }
    /// At most eight fighter identities; an unexpected extra identity reads through.
    /// Construction allocates no heap memory and does not read the game eagerly.
    ///
///  At most eight fighter identities; an unexpected extra identity reads through. Construction allocates no heap memory and does not read the game eagerly.
    pub type ObservationSnapshot = ObservationSnapshotSized<8>;
    /// Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
    ///
///  Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
    pub struct ObservationSnapshotSized<const N: usize> {
        /// Stores the `fighters` value in `ObservationSnapshotSized` as `[Fighter; N]`.
        ///
        ///
///  Stores the fighters collection for ObservationSnapshotSized.
        fighters: [Fighter; N],
        /// Stores the `last` value in `ObservationSnapshotSized` as `Cell<Option<usize>>`.
        ///
        ///
///  Holds the last value used by ObservationSnapshotSized.
        last: Cell<Option<usize>>,
    }
    impl<const N: usize> Default for ObservationSnapshotSized<N> {
        /// Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
        ///
///  Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
        fn default() -> Self {
            Self {
                last: Cell::new(None),
                fighters: std::array::from_fn(|_| Fighter {
                    id: Cell::new(None),
                    facts: std::array::from_fn(|_| Cell::new(None)),
                }),
            }
        }
    }
    impl<const N: usize> ObservationSnapshotSized<N> {
        /// A copied motion clock from an already observed other fighter. No native
        /// calls or additional roster scan; passive frozen poses cannot time a tail.
        ///
///  A copied motion clock from an already observed other fighter. No native calls or additional roster scan; passive frozen poses cannot time a tail.
        pub fn known_progress_clock(&self, own: u32) -> Option<(u32, i32, u64, f32, f32)> {
            self.fighters.iter().find_map(|f| {
                let id = f.id.get().filter(|id| *id != own)?;
                let fact = |field: Field| f.facts[field as usize].get();
                let status = fact(Field::Status)?.int()?;
                if matches!(
                    status,
                    crate::native_states::GUARD_ON
                        | crate::native_states::GUARD
                        | crate::native_states::status::CLIFF_WAIT
                        | crate::native_states::status::OTTOTTO
                        | crate::native_states::status::OTTOTTO_WAIT
                ) || fact(Field::Stop) == Some(Fact::Bool(true))
                {
                    return None;
                }
                let motion = fact(Field::Motion)?.motion()?;
                let frame = fact(Field::Frame)?.float()?;
                let rate = fact(Field::Rate)?.float()?;
                crate::timing::clock_known(motion, frame, rate)
                    .then_some((id, status, motion, frame, rate))
            })
        }
        /// Already-read statuses only: no native calls, eager reads or allocation.
        /// Event consumers can distinguish a known end from a missing observation.
        ///
///  Already-read statuses only: no native calls, eager reads or allocation. Event consumers can distinguish a known end from a missing observation.
        pub fn known_statuses(&self) -> [Option<(u32, i32)>; N] {
            std::array::from_fn(|i| {
                let entry = &self.fighters[i];
                let id = entry.id.get()?;
                Some((id, entry.facts[Field::Status as usize].get()?.int()?))
            })
        }
        /// Most consumers read several fields from the same fighter in a row.
        /// This cursor only speeds slot lookup; fact lifetime/overflow stay exact.
        ///
///  Most consumers read several fields from the same fighter in a row. This cursor only speeds slot lookup; fact lifetime/overflow stay exact.
        pub fn read(&self, id: u32, field: Field, read: impl FnOnce() -> Fact) -> Fact {
            let index = self
                .last
                .get()
                .filter(|i| self.fighters[*i].id.get() == Some(id))
                .or_else(|| self.fighters.iter().position(|e| e.id.get() == Some(id)))
                .or_else(|| self.fighters.iter().position(|e| e.id.get().is_none()));
            let Some(index) = index else {
                return read();
            };
            self.last.set(Some(index));
            let entry = &self.fighters[index];
            entry.id.set(Some(id));
            let fact = &entry.facts[field as usize];
            if let Some(value) = fact.get() {
                return value;
            }
            let value = read();
            fact.set(Some(value));
            value
        }
    }

    /// One synchronous observation's roster IDs. Empty/unavailable is cached too.
    /// No native pointers, allocation, frame throttling, or cross-frame lifetime.
    ///
///  One synchronous observation's roster IDs. Empty/unavailable is cached too. No native pointers, allocation, frame throttling, or cross-frame lifetime.
    pub struct RosterSnapshot<const N: usize = 8> {
        /// Stores the `value` value in `RosterSnapshot` as `Cell<Option<Option<[Option<u32>; N]>>>`.
        ///
        ///
///  Holds the value value used by RosterSnapshot.
        value: Cell<Option<Option<[Option<u32>; N]>>>,
    }
    impl<const N: usize> Default for RosterSnapshot<N> {
        /// Returns the default value for this type.
        ///
        ///
///  Returns the default value for this type.
        fn default() -> Self {
            Self {
                value: Cell::new(None),
            }
        }
    }
    impl<const N: usize> RosterSnapshot<N> {
        /// Returns an optional `read` value.
        ///
        ///
///  Gets read if it is available.
        pub fn read(
            &self,
            query: impl FnOnce() -> Option<[Option<u32>; N]>,
        ) -> Option<[Option<u32>; N]> {
            if let Some(value) = self.value.get() {
                return value;
            }
            let value = query();
            self.value.set(Some(value));
            value
        }
    }
}
pub(crate) mod projectile_geometry {
    //! Continuous swept capsule bounds against an observed body envelope.
    use crate::autopilot_guard::Hitbox;

    /// Computes the earliest time of intersection between a swept capsule and an axis-aligned bounding box.
    ///
///  Computes the earliest time of intersection between a swept capsule and an axis-aligned bounding box.
    pub(crate) fn swept_capsule_aabb(
        hit: Hitbox,
        body_lo: [f32; 3],
        body_hi: [f32; 3],
        velocity: [f32; 2],
        horizon: f32,
    ) -> Option<f32> {
        let relative = [
            hit.velocity[0] - velocity[0],
            hit.velocity[1] - velocity[1],
            0.,
        ];
        let mut enter = 0.0f32;
        let mut leave = horizon;
        for axis in 0..3 {
            let lo = hit.start[axis].min(hit.end[axis]) - hit.radius;
            let hi = hit.start[axis].max(hit.end[axis]) + hit.radius;
            let speed = relative[axis];
            if speed.abs() < 0.0001 {
                if hi < body_lo[axis] || lo > body_hi[axis] {
                    return None;
                }
            } else {
                let a = (body_lo[axis] - hi) / speed;
                let b = (body_hi[axis] - lo) / speed;
                enter = enter.max(a.min(b));
                leave = leave.min(a.max(b));
                if enter > leave {
                    return None;
                }
            }
        }
        if leave < 0. || enter > horizon {
            return None;
        }
        Some(enter)
    }
}
pub mod state {
    //! Normalize game facts into small Rust values that policy can understand.
    //! Scenario names describe observations, not game commands. Most variants are
    //! extension points; the current live adapter publishes Clatter and Tech only.
    //!
    //! Traits at the bottom define a contract for the generic runtime/examples.
    //! They do not automatically read game memory or install native callbacks.
    use crate::input::ControllerFrame;

    /// simple label that tells the generic assist engine what it knows about the fighter
    /// It allows the engine to differentiate between known fighters, unknown fighters, and Kirby specifically.
    /// *** 
    /// Need to analyse more, may be extended in the future to include other specific fighters and unique modes/modifiers.
    ///
///  simple label that tells the generic assist engine what it knows about the fighter It allows the engine to differentiate between known fighters, unknown fighters, and Kirby specifically.  Need to analyse more, may be extended.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum FighterKind {
        /// Represents the `Kirby` case of `FighterKind`.
        ///
        ///
///  Represents the Kirby option in FighterKind.
        Kirby,
        /// Represents the `Other` case of `FighterKind`.
        ///
        ///
///  Represents the Other option in FighterKind.
        Other,
        /// Represents the `Unknown` case of `FighterKind`.
        ///
        ///
///  Represents the Unknown option in FighterKind.
        Unknown,
    }

    /// A normalized situation that can activate an assistance rule.
    ///
    /// The priority order follows the Training Modpack's public mash selection
    /// flow: specific defensive situations win over broad grounded/airborne ones.
    ///
///  A normalized situation that can activate an assistance rule. The priority order follows the Training Modpack's public mash selection flow: specific defensive situations win over broad grounded/airborne ones.
    #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
    pub enum Scenario {
        /// Represents the `Tech` case of `Scenario`.
        ///
        ///
///  Represents the Tech option in Scenario.
        Tech,
        /// Represents the `Clatter` case of `Scenario`.
        ///
        ///
///  Represents the Clatter option in Scenario.
        Clatter,
        /// Represents the `Tumble` case of `Scenario`.
        ///
        ///
///  Represents the Tumble option in Scenario.
        Tumble,
        /// Represents the `Hitstun` case of `Scenario`.
        ///
        ///
///  Represents the Hitstun option in Scenario.
        Hitstun,
        /// Represents the `Parry` case of `Scenario`.
        ///
        ///
///  Represents the Parry option in Scenario.
        Parry,
        /// Represents the `Footstool` case of `Scenario`.
        ///
        ///
///  Represents the Footstool option in Scenario.
        Footstool,
        /// Represents the `Landing` case of `Scenario`.
        ///
        ///
///  Represents the Landing option in Scenario.
        Landing,
        /// Represents the `Ledge` case of `Scenario`.
        ///
        ///
///  Represents the Ledge option in Scenario.
        Ledge,
        /// Represents the `Grounded` case of `Scenario`.
        ///
        ///
///  Represents the Grounded option in Scenario.
        Grounded,
        /// Represents the `Airborne` case of `Scenario`.
        ///
        ///
///  Represents the Airborne option in Scenario.
        Airborne,
        // Sentinel for an unused observation/rule slot; it is not an active tool.
        #[default]
        /// Represents the `None` case of `Scenario`.
        ///
        ///
///  Represents the None option in Scenario.
        None,
    }

    impl Scenario {
        /// Lower values have higher precedence when multiple samples are active.
        ///
///  Lower values have higher precedence when multiple samples are active.
        pub const fn priority(self) -> u8 {
            match self {
                Self::Tech => 0,
                Self::Clatter => 1,
                Self::Tumble => 2,
                Self::Hitstun => 3,
                Self::Parry => 4,
                Self::Footstool => 5,
                Self::Landing => 6,
                Self::Ledge => 7,
                Self::Grounded => 8,
                Self::Airborne => 9,
                Self::None => u8::MAX,
            }
        }
    }

    /// A situation supplied by a live or pretend game-state adapter.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    // Age belongs to a particular occurrence, not match time: active -> inactive
    // -> active restarts at zero. The live ScenarioCounter implements that contract.
    ///
///  A situation supplied by a live or pretend game-state adapter.
    pub struct ScenarioSample {
        /// Stores the `scenario` value in `ScenarioSample` as `Scenario`.
        ///
        ///
///  Holds the scenario value used by ScenarioSample.
        pub scenario: Scenario,
        /// Frame zero is the first frame in this scenario.
        ///
///  Frame zero is the first frame in this scenario.
        pub frames_active: u32,
    }

    impl ScenarioSample {
        /// Defines the `EMPTY` constant used by this module.
        ///
        ///
///  Provides the EMPTY constant value.
        pub const EMPTY: Self = Self {
            scenario: Scenario::None,
            frames_active: 0,
        };

        /// Constructs a new value with the type’s initial state.
        ///
        ///
///  Creates a new value.
        pub const fn new(scenario: Scenario, frames_active: u32) -> Self {
            Self {
                scenario,
                frames_active,
            }
        }
    }

    /// A small snapshot with room for overlapping conditions (for example, tumble
    /// and hitstun). The engine chooses the highest-priority active scenario.
    ///
///  A small snapshot with room for overlapping conditions (for example, tumble and hitstun). The engine chooses the highest-priority active scenario.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct FighterObservation {
        /// Stores the `active` value in `FighterObservation` as `[ScenarioSample; Self::MAX_SAMPLES]`.
        ///
        ///
///  Stores the active collection for FighterObservation.
        pub active: [ScenarioSample; Self::MAX_SAMPLES],
    }

    impl FighterObservation {
        /// Sets the upper bound for samples.
        ///
        ///
///  Holds the MAX SAMPLES value.
        pub const MAX_SAMPLES: usize = 3;

        /// Defines the `EMPTY` constant used by this module.
        ///
        ///
///  Provides the EMPTY constant value.
        pub const EMPTY: Self = Self {
            active: [ScenarioSample::EMPTY; Self::MAX_SAMPLES],
        };

        /// Returns the `one` result.
        ///
        ///
///  Gets the one result.
        pub const fn one(scenario: Scenario, frames_active: u32) -> Self {
            Self {
                active: [
                    ScenarioSample::new(scenario, frames_active),
                    ScenarioSample::EMPTY,
                    ScenarioSample::EMPTY,
                ],
            }
        }

        // Return the sample WITH its age; choosing only its name would lose the
        // timing information needed by delayed/pulsing rules.
        /// Returns an optional `highest_priority` value.
        ///
        ///
///  Gets highest priority if it is available.
        pub fn highest_priority(self) -> Option<ScenarioSample> {
            self.active
                .into_iter()
                .filter(|sample| sample.scenario != Scenario::None)
                .min_by_key(|sample| sample.scenario.priority())
        }
    }

    impl Default for FighterObservation {
        /// Returns the default value for this type.
        ///
        ///
///  Returns the default value for this type.
        fn default() -> Self {
            Self::EMPTY
        }
    }

    /// Facts that must be established before the engine may propose an action.
    ///
///  Facts that must be established before the engine may propose an action.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct RunContext {
        /// Stores the `is_locally_human_controlled` value in `RunContext` as `bool`.
        ///
        ///
///  Holds the is locally human controlled value used by RunContext.
        pub is_locally_human_controlled: bool,
        /// Stores the `fighter` value in `RunContext` as `FighterKind`.
        ///
        ///
///  Holds the fighter value used by RunContext.
        pub fighter: FighterKind,
    }

    /// Boundary to implement only after a current, documented state-reading route
    /// has been verified. No game hook or memory layout is assumed here.
    // A trait describes methods a type must supply. It is not a concrete adapter;
    // examples/walkthrough.rs implements these methods using pretend data. &self
    // permits reads; &mut self on write_merged_input permits changing adapter state.
    ///
///  Boundary to implement only after a current, documented state-reading route has been verified. No game hook or memory layout is assumed here.
    pub trait GameObservationSource {
        /// Observes the fighter.
        ///
        ///
///  Observes the fighter.
        fn observe_fighter(&self) -> Option<FighterObservation>;
    }

    /// Boundary to implement only after a current, documented input route has been
    /// verified. The core requires a read-then-write flow so human input can be
    /// preserved during arbitration.
    ///
///  Boundary to implement only after a current, documented input route has been verified. The core requires a read-then-write flow so human input can be preserved during arbitration.
    pub trait InputBridge {
        /// Reads the human input.
        ///
        ///
///  Reads the human input.
        fn read_human_input(&self) -> Option<ControllerFrame>;
        /// Writes the merged input.
        ///
        ///
///  Writes the merged input.
        fn write_merged_input(&mut self, frame: ControllerFrame);
    }

    /// Combined adapter used by the frame runner. Return `None` when no valid
    /// fighter/controller is available (for example during loading or in menus).
    ///
///  Combined adapter used by the frame runner. Return None when no valid fighter/controller is available (for example during loading or in menus).
    pub trait GameAdapter: GameObservationSource + InputBridge {
        /// Reads the context.
        ///
        ///
///  Reads the context.
        fn read_context(&self) -> Option<RunContext>;
    }

    // Compiled only by cargo test on the host; these assertions are not live
    // Switch logging or gameplay callbacks. Add regressions here for new behavior.
    #[cfg(test)]
    mod tests {
        use super::*;

        /// Regression test for `specific_scenario_beats_broad_scenario`.
        ///
        ///
///  Checks that specific scenario beats broad scenario.
        #[test]
        fn specific_scenario_beats_broad_scenario() {
            let observation = FighterObservation {
                active: [
                    ScenarioSample::new(Scenario::Grounded, 12),
                    ScenarioSample::new(Scenario::Tumble, 3),
                    ScenarioSample::EMPTY,
                ],
            };

            assert_eq!(
                observation.highest_priority(),
                Some(ScenarioSample::new(Scenario::Tumble, 3))
            );
        }
    }
}
