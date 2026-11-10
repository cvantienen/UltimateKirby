//! Observation contracts passed from the game adapter to policies.

pub mod battle_objects {
    //! Bounded copied identities and candidate selection; no native pointers.
    /// Maximum number of fighter current slots in the roster.
    /// Plain English: Maximum number of fighter current slots in the roster.
    pub const FIGHTER_CAPACITY: usize = 16;
    /// Object category for fighters.
    /// Plain English: Object category for fighters.
    pub const OBJECT_CATEGORY_FIGHTER: u32 = 0;
    /// Object category for weapons.
    /// Plain English: Object category for weapons.
    pub const OBJECT_CATEGORY_WEAPON: u32 = 1;
    /// Object category for items.
    /// Plain English: Object category for items.
    pub const OBJECT_CATEGORY_ITEM: u32 = 4;
    /// Weapon kind for Pikmin Pikmin.
    /// Plain English: Weapon kind for Pikmin Pikmin.
    pub const WEAPON_KIND_PIKMIN_PIKMIN: i32 = 0x9d;
    /// Weapon kind for Rosetta Tico.
    /// Plain English: Weapon kind for Rosetta Tico.
    pub const WEAPON_KIND_ROSETTA_TICO: i32 = 0xe6;
    /// Bitshift amount (28) for extracting the category from an object ID.
    /// That label is a 32-bit number.
    /// Smash stores the object’s category in the first four bits and other identifying information in the remaining 28
    /// Plain English: Bitshift amount (28) for extracting the category from an object ID. That label is a 32-bit number. Smash stores the object’s category in the first four bits and other identifying information in the remaining 28.
    pub const BITSHIFT_CATEGORY: u32 = 28;
    /// Array holding current fighters in game.
    /// Each slot can hold an optional fighter ID; empty slots are None.
    /// Plain English: Array holding current fighters in game. Each slot can hold an optional fighter ID; empty slots are None.
    pub type FightersInGame = [Option<u32>; FIGHTER_CAPACITY];

    /// Recieves a mutable reference to a fighter roster and a fighter ID to append.
    /// Plain English: Recieves a mutable reference to a fighter roster and a fighter ID to append.
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
    /// Plain English: Extra actors must not turn roster ordinals into indices of eight-slot policy arrays. Keep the nearest relevant candidates when those arrays fill.
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

    /// Technical: Returns the `distance_squared` result.
    ///
    /// Plain English: Gets the distance squared result.
    pub fn distance_squared(from: [f32; 2], to: [f32; 2]) -> f32 {
        (to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2)
    }

    /// Technical: Defines the available `ObjectCategory` cases.
    ///
    /// Plain English: Lists the available ObjectCategory choices.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u32)]
    pub enum ObjectCategory {
        /// Technical: Represents the `Fighter` case of `ObjectCategory`.
        ///
        /// Plain English: Represents the Fighter option in ObjectCategory.
        Fighter = OBJECT_CATEGORY_FIGHTER,
        /// Technical: Represents the `Weapon` case of `ObjectCategory`.
        ///
        /// Plain English: Represents the Weapon option in ObjectCategory.
        Weapon = OBJECT_CATEGORY_WEAPON,
        /// Technical: Represents the `Item` case of `ObjectCategory`.
        ///
        /// Plain English: Represents the Item option in ObjectCategory.
        Item = OBJECT_CATEGORY_ITEM,
        // ...
    }

    impl ObjectCategory {
        /// Technical: Constructs a value from the id.
        ///
        /// Plain English: Constructs a value from the id.
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
        /// Plain English: check to ensure the weapon is not a non-projectile type like Rosetta & Tico, and that Pikmin are not already attached or in a landing state.
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
    /// Plain English: Entry-based damage attribution cannot distinguish two Ice Climbers on opposite sides. Preserve agreement; return no invented side on conflict.
    #[derive(Default)]
    pub struct EntryDirection {
        /// Technical: Stores the `side` value in `EntryDirection` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the side for EntryDirection; it may be absent.
        side: Option<f32>,
        /// Technical: Stores the `conflict` value in `EntryDirection` as `bool`.
        ///
        /// Plain English: Holds the conflict value used by EntryDirection.
        conflict: bool,
    }
    impl EntryDirection {
        /// Technical: Records the time of the latest fighter observation.
        ///
        /// Plain English: Remembers when a fighter was last seen.
        pub fn observe(&mut self, direction: f32) {
            if !direction.is_finite() || self.side.is_some_and(|side| side != direction) {
                self.conflict = true;
            }
            self.side = Some(direction);
        }
        /// Technical: Returns an optional `direction` value.
        ///
        /// Plain English: Gets direction if it is available.
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
    /// Plain English: Maximum number of animation forecast frames considered for timing calculations.
    const MAX_ANIMATION_FORECAST_FRAMES: f32 = 120.0;

    /// Read-only animation timing; remaining values are measured in game frames.
    /// This struct captures the timing information for a single animation,
    ///  including its current frame, rate of progress, and remaining duration.
    /// Plain English: Read-only animation timing; remaining values are measured in game frames. This struct captures the timing information for a single animation, including its current frame, rate of progress, and remaining duration.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MotionTiming {
        /// Animation Hash40 identifier, not a status ID or frame count.
        /// Plain English: Animation Hash40 identifier, not a status ID or frame count.
        pub kind: u64,
        /// Animation frame, which can be fractional.
        /// Plain English: Animation frame, which can be fractional.
        pub frame: f32,
        /// Converts animation progress into game-frame time.
        /// Plain English: Converts animation progress into game-frame time.
        pub rate: f32,
        /// None means unknown; Some(0.0) means the animation already ended.
        /// Plain English: None means unknown; Some(0.0) means the animation already ended.
        pub remaining: Option<f32>,
    }

    /// Convert an animation-frame deadline into game-frame time.
    ///
    /// Invalid or past deadlines remain unknown instead of being treated as ready.
    /// Plain English: Convert an animation-frame deadline into game-frame time. Invalid or past deadlines remain unknown instead of being treated as ready.
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
    /// Plain English: Validated native air kinetics for short movement forecasts.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AirPhysics {
        /// Technical: Stores the `gravity` value in `AirPhysics` as `f32`.
        ///
        /// Plain English: Holds the gravity value used by AirPhysics.
        pub gravity: f32,
        /// Technical: Stores the `terminal_speed` value in `AirPhysics` as `f32`.
        ///
        /// Plain English: Holds the terminal speed value used by AirPhysics.
        pub terminal_speed: f32,
    }
    impl AirPhysics {
        /// Technical: Constructs a new value with the type’s initial state.
        ///
        /// Plain English: Creates a new value.
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
    /// Plain English: Return time to the earlier animation end or native cancel deadline.
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
    /// Plain English: Read-only opponent facts copied by the platform adapter.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct Opponent {
        /// Technical: Stores the `id` value in `Opponent` as `u32`.
        ///
        /// Plain English: Holds the id value used by Opponent.
        pub id: u32,
        /// Technical: Stores the `status` value in `Opponent` as `i32`.
        ///
        /// Plain English: Holds the status value used by Opponent.
        pub status: i32,
        /// Technical: Stores the `previous_status` value in `Opponent` as `i32`.
        ///
        /// Plain English: Holds the previous status value used by Opponent.
        pub previous_status: i32,
        /// Technical: Stores the `situation` value in `Opponent` as `i32`.
        ///
        /// Plain English: Holds the situation value used by Opponent.
        pub situation: i32,
        /// Technical: Stores the `position` value in `Opponent` as `[f32; 2]`.
        ///
        /// Plain English: Stores the position collection for Opponent.
        pub position: [f32; 2],
        /// Aggregate native hit status, not an individual hurtbox model.
        /// Plain English: Aggregate native hit status, not an individual hurtbox model.
        pub vulnerable: bool,
        /// Technical: Stores the `velocity` value in `Opponent` as `[f32; 2]`.
        ///
        /// Plain English: Stores the velocity collection for Opponent.
        pub velocity: [f32; 2],
        /// Only ledge policies use this fallback anchor when the fighter hangs.
        /// Plain English: Only ledge policies use this fallback anchor when the fighter hangs.
        pub cliff_position: Option<[f32; 2]>,
        /// Sanitized native motion metadata; None means prediction is unavailable.
        /// Plain English: Sanitized native motion metadata; None means prediction is unavailable.
        pub vulnerable_in: Option<f32>,
        /// Technical: Stores the `motion` value in `Opponent` as `Option<MotionTiming>`.
        ///
        /// Plain English: Optionally stores the motion for Opponent; it may be absent.
        pub motion: Option<MotionTiming>,
        /// Present for get-up attacks; None preserves conservative block-only behavior.
        /// Plain English: Present for get-up attacks; None preserves conservative block-only behavior.
        pub attack_active: Option<bool>,
        /// Technical: Stores the `air_physics` value in `Opponent` as `Option<AirPhysics>`.
        ///
        /// Plain English: Optionally stores the air physics for Opponent; it may be absent.
        pub air_physics: Option<AirPhysics>,
        /// Native cancel deadline, separate from intermediate jump animation ends.
        /// Plain English: Native cancel deadline, separate from intermediate jump animation ends.
        pub actionable_in: Option<f32>,
    }

    /// Controlled fighter's native state, copied by the platform adapter.
    /// Plain English: Controlled fighter's native state, copied by the platform adapter.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct GameplayContext {
        /// Whether the controlled fighter is currently holding an input.
        /// Plain English: Whether the controlled fighter is currently holding an input.
        pub held: bool,
        /// Whether the controlled fighter is currently in active gameplay.
        /// Plain English: Whether the controlled fighter is currently in active gameplay.
        pub gameplay: bool,
        /// Changes only on fresh physical gameplay input, never simulated output.
        /// Plain English: Changes only on fresh physical gameplay input, never simulated output.
        pub manual_input_epoch: u64,
        /// Native readiness for this consumer's intended grounded action.
        /// Plain English: Native readiness for this consumer's intended grounded action.
        pub can_act: bool,
        /// The assist may own inputs, including during its guard/grab/tilt animation.
        /// Plain English: The assist may own inputs, including during its guard/grab/tilt animation.
        pub can_control: bool,
        /// Native hitstop: preserve an existing shield, but start no new actions.
        /// Plain English: Native hitstop: preserve an existing shield, but start no new actions.
        pub stopped: bool,
        /// The controlled fighter's current status kind.
        /// Plain English: The controlled fighter's current status kind.
        pub status: i32,
        /// The controlled fighter's current situation kind.
        /// Plain English: The controlled fighter's current situation kind.
        pub situation: i32,
        /// Current animation frame. Together with the motion kind it distinguishes
        /// progress when a target holds one pose, and is part of duplicate-observation reuse.
        /// Plain English: Current animation frame. Together with the motion kind it distinguishes progress when a target holds one pose, and is part of duplicate-observation reuse.
        pub motion_frame: f32,
        /// Current motion (Hash40) identifier.
        /// Plain English: Current motion (Hash40) identifier.
        pub motion_kind: u64,
        /// Current animation rate.
        /// Plain English: Current animation rate.
        pub motion_rate: f32,
        /// Current position in world coordinates.
        /// Plain English: Current position in world coordinates.
        pub position: [f32; 2],
        /// Current velocity in world coordinates.
        /// Plain English: Current velocity in world coordinates.
        pub velocity: [f32; 2],
        /// Native ground deceleration; None avoids assuming a stopping distance.
        /// Plain English: Native ground deceleration; None avoids assuming a stopping distance.
        pub ground_brake: Option<f32>,
        /// Facing uses its sign (+right/-left); position/velocity remain absolute
        /// world axes. Multiplying dx by facing asks if the target is in front.
        /// Plain English: Facing uses its sign (+right/-left); position/velocity remain absolute world axes. Multiplying dx by facing asks if the target is in front.
        pub facing: f32,
        /// The controlled fighter's previous status kind.
        /// Plain English: The controlled fighter's previous status kind.
        pub previous_status: i32,
        /// Whether the controlled fighter can currently perform a grab.
        /// Plain English: Whether the controlled fighter can currently perform a grab.
        pub can_grab: bool,
        /// Whether the controlled fighter can currently perform a tilt.
        /// Plain English: Whether the controlled fighter can currently perform a tilt.
        pub can_tilt: bool,
        /// Whether the controlled fighter can currently perform a down tilt.
        /// Plain English: Whether the controlled fighter can currently perform a down tilt.
        pub can_down_tilt: bool,
        /// Whether the controlled fighter can currently perform a down special.
        /// Plain English: Whether the controlled fighter can currently perform a down special.
        pub can_down_special: bool,
        /// Whether the controlled fighter can currently perform a dash attack.
        /// Plain English: Whether the controlled fighter can currently perform a dash attack.
        pub can_dash_attack: bool,
        /// Whether the controlled fighter can currently perform a down smash.
        /// Plain English: Whether the controlled fighter can currently perform a down smash.
        pub can_down_smash: bool,
        /// Explicit additional delay in game frames; live observations supply zero.
        /// Plain English: Explicit additional delay in game frames; live observations supply zero.
        pub input_delay: f32,
        /// Whether the controlled fighter can currently perform a jump.
        /// Plain English: Whether the controlled fighter can currently perform a jump.
        pub can_jump: bool,
        /// Whether the controlled fighter can currently perform an air attack.
        /// Plain English: Whether the controlled fighter can currently perform an air attack.
        pub can_air_attack: bool,
        /// The controlled fighter's current air physics, if applicable.
        /// Plain English: The controlled fighter's current air physics, if applicable.
        pub air_physics: Option<AirPhysics>,
        /// The controlled fighter's short hop speed, if applicable.
        /// Plain English: The controlled fighter's short hop speed, if applicable.
        pub short_hop_speed: Option<f32>,
        /// The controlled fighter's full hop speed, if applicable.
        /// Plain English: The controlled fighter's full hop speed, if applicable.
        pub full_hop_speed: Option<f32>,
        /// The number of frames the controlled fighter is in shield stun, if applicable.
        /// Plain English: The number of frames the controlled fighter is in shield stun, if applicable.
        pub shield_stun_frames: Option<i32>,
        /// The ID of the attacker causing the shield stun, if applicable.
        /// Plain English: The ID of the attacker causing the shield stun, if applicable.
        pub shield_attacker: Option<u32>,
        /// Native hit infliction during Kirby's current assisted attack.
        /// Plain English: Native hit infliction during Kirby's current assisted attack.
        pub hit_confirmed: bool,
        /// Ground support along a short swept lookahead, left then right.
        /// Plain English: Ground support along a short swept lookahead, left then right.
        pub safe_ground: [bool; 2],
    }

    /// Proposed controller inputs; the live adapter arbitrates and maps the request.
    /// Plain English: Proposed controller inputs; the live adapter arbitrates and maps the request.
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct ControllerOutput {
        /// Whether to consume the trigger for this update.
        /// Plain English: Whether to consume the trigger for this update.
        pub consume_trigger: bool,
        /// The desired stick position for this update, if any.
        /// Plain English: The desired stick position for this update, if any.
        pub stick: Option<[f32; 2]>,
        /// Buttons to hold for this update (a mask of native controller bits).
        /// Plain English: Buttons to hold for this update (a mask of native controller bits).
        pub requested: u32,
        /// Buttons to replace, including buttons that should currently be released.
        /// Plain English: Buttons to replace, including buttons that should currently be released.
        pub owned: u32,
        /// Preserve a committed short-hop decision with this cached output until mapped.
        /// Plain English: Preserve a committed short-hop decision with this cached output until mapped.
        pub native_short_hop: bool,
    }
    impl ControllerOutput {
        /// No proposal; usable in `const` contexts where `Default` is not.
        /// Plain English: No proposal; usable in const contexts where Default is not.
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
    /// Plain English: Input proposals belong to exactly one selected-controller update. Wall-clock freshness is only a secondary timeout; it cannot make an old packet current.
    pub fn input_update_current(evaluated: Option<i64>, current: i64) -> bool {
        evaluated == Some(current)
    }

    /// Reject setup that began before a settings/cancellation fence or mapper update.
    /// Plain English: Reject setup that began before a settings/cancellation fence or mapper update.
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
    /// Plain English: A command callback may upgrade a mapper-only sample once, including when no policy needs a roster. Discovery capability is distinct from roster data.
    #[derive(Clone, Copy, Default)]
    pub struct ObservationSetup {
        /// Technical: Stores the `discovered` value in `ObservationSetup` as `Option<i64>`.
        ///
        /// Plain English: Optionally stores the discovered for ObservationSetup; it may be absent.
        discovered: Option<i64>,
    }
    impl ObservationSetup {
        /// Technical: Constructs a new value with the type’s initial state.
        ///
        /// Plain English: Creates a new value.
        pub const fn new() -> Self {
            Self { discovered: None }
        }
        /// Technical: Determines whether the `needed` condition holds.
        ///
        /// Plain English: Checks whether needed.
        pub fn needed(&self, tick: i64, evaluated: bool, discover: bool) -> bool {
            !evaluated || (discover && self.discovered != Some(tick))
        }
        /// Technical: Updates the `complete` state using the supplied inputs.
        ///
        /// Plain English: Updates complete.
        pub fn complete(&mut self, tick: i64, discover: bool) {
            if discover {
                self.discovered = Some(tick);
            }
        }
        /// Technical: Clears the remembered fighter observation.
        ///
        /// Plain English: Forgets the last fighter observation.
        pub fn clear(&mut self) {
            self.discovered = None;
        }
    }

    /// Exact floor-query reuse within ONE synchronous observation. Store copied
    /// values, never a pointer, and discard at callback exit so moving platforms
    /// and later frames always get new collision queries.
    /// Plain English: Exact floor-query reuse within ONE synchronous observation. Store copied values, never a pointer, and discard at callback exit so moving platforms and later frames always get new collision queries.
    type GroundSupportCache = Option<(u32, [f32; 2], f32, [bool; 2])>;

    /// Technical: Defines the `GroundSupportSnapshot` data structure.
    ///
    /// Plain English: Holds the data for GroundSupportSnapshot.
    #[derive(Default)]
    pub struct GroundSupportSnapshot {
        // `Cell` lets this synchronous snapshot cache copied results through a
        // shared reference; it is interior mutability, not cross-thread locking.
        /// Technical: Stores the `last` value in `GroundSupportSnapshot` as `Cell<GroundSupportCache>`.
        ///
        /// Plain English: Holds the last value used by GroundSupportSnapshot.
        last: Cell<GroundSupportCache>,
    }
    impl GroundSupportSnapshot {
        /// Technical: Returns the `read` result.
        ///
        /// Plain English: Gets the read result.
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
    /// Plain English: Field identifiers for fighter facts. Used as indices into the facts array of a Fighter.
    #[derive(Clone, Copy)]
    #[repr(usize)]
    pub enum Field {
        /// Technical: Represents the `Status` case of `Field`.
        ///
        /// Plain English: Represents the Status option in Field.
        Status,
        /// Technical: Represents the `Situation` case of `Field`.
        ///
        /// Plain English: Represents the Situation option in Field.
        Situation,
        /// Technical: Represents the `Motion` case of `Field`.
        ///
        /// Plain English: Represents the Motion option in Field.
        Motion,
        /// Technical: Represents the `Frame` case of `Field`.
        ///
        /// Plain English: Represents the Frame option in Field.
        Frame,
        /// Technical: Represents the `Rate` case of `Field`.
        ///
        /// Plain English: Represents the Rate option in Field.
        Rate,
        /// Technical: Represents the `X` case of `Field`.
        ///
        /// Plain English: Represents the X option in Field.
        X,
        /// Technical: Represents the `Y` case of `Field`.
        ///
        /// Plain English: Represents the Y option in Field.
        Y,
        /// Technical: Represents the `Vx` case of `Field`.
        ///
        /// Plain English: Represents the Vx option in Field.
        Vx,
        /// Technical: Represents the `Vy` case of `Field`.
        ///
        /// Plain English: Represents the Vy option in Field.
        Vy,
        /// Technical: Represents the `Facing` case of `Field`.
        ///
        /// Plain English: Represents the Facing option in Field.
        Facing,
        /// Technical: Represents the `Team` case of `Field`.
        ///
        /// Plain English: Represents the Team option in Field.
        Team,
        /// Technical: Represents the `Stop` case of `Field`.
        ///
        /// Plain English: Represents the Stop option in Field.
        Stop,
        /// Technical: Represents the `PreviousStatus` case of `Field`.
        ///
        /// Plain English: Represents the PreviousStatus option in Field.
        PreviousStatus,
        /// Technical: Represents the `EndFrame` case of `Field`.
        ///
        /// Plain English: Represents the EndFrame option in Field.
        EndFrame,
        /// Technical: Represents the `CaptureLink` case of `Field`.
        ///
        /// Plain English: Represents the CaptureLink option in Field.
        CaptureLink,
        /// Technical: Represents the `Kind` case of `Field`.
        ///
        /// Plain English: Represents the Kind option in Field.
        Kind,
    }
    /// Technical: Defines the `FIELDS` constant used by this module.
    ///
    /// Plain English: Provides the FIELDS constant value.
    const FIELDS: usize = Field::Kind as usize + 1;
    /// Technical: Defines the available `Fact` cases.
    ///
    /// Plain English: Lists the available Fact choices.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Fact {
        /// Technical: Represents the `Int` case of `Fact`.
        ///
        /// Plain English: Represents the Int option in Fact.
        Int(
            /// Technical: Carries the observed int fact value in `Fact`.
            ///
            /// Plain English: Holds the observed int fact value for this `Fact` choice.
            i32,
        ),
        /// Technical: Represents the `Motion` case of `Fact`.
        ///
        /// Plain English: Represents the Motion option in Fact.
        Motion(
            /// Technical: Carries the observed motion fact value in `Fact`.
            ///
            /// Plain English: Holds the observed motion fact value for this `Fact` choice.
            u64,
        ),
        /// Technical: Represents the `Float` case of `Fact`.
        ///
        /// Plain English: Represents the Float option in Fact.
        Float(
            /// Technical: Carries the observed float fact value in `Fact`.
            ///
            /// Plain English: Holds the observed float fact value for this `Fact` choice.
            f32,
        ),
        /// Technical: Represents the `Bool` case of `Fact`.
        ///
        /// Plain English: Represents the Bool option in Fact.
        Bool(
            /// Technical: Carries the observed bool fact value in `Fact`.
            ///
            /// Plain English: Holds the observed bool fact value for this `Fact` choice.
            bool,
        ),
    }
    impl Fact {
        /// Technical: Returns an optional `int` value.
        ///
        /// Plain English: Gets int if it is available.
        fn int(self) -> Option<i32> {
            match self {
                Self::Int(value) => Some(value),
                _ => None,
            }
        }
        /// Technical: Returns the motion value when the observation contains one.
        ///
        /// Plain English: Gets the motion value if it is available.
        fn motion(self) -> Option<u64> {
            match self {
                Self::Motion(value) => Some(value),
                _ => None,
            }
        }
        /// Technical: Returns the floating-point value when the fact contains one.
        ///
        /// Plain English: Gets the number if this fact stores one.
        fn float(self) -> Option<f32> {
            match self {
                Self::Float(value) => Some(value),
                _ => None,
            }
        }
    }
    /// A fighter in the observation snapshot, storing its identity and copied facts.
    /// Plain English: A fighter in the observation snapshot, storing its identity and copied facts.
    struct Fighter {
        // Cache facts lazily so sharing a snapshot does not force unrelated
        // policies to perform every possible native observation.
        /// Technical: Stores the `id` value in `Fighter` as `Cell<Option<u32>>`.
        ///
        /// Plain English: Holds the id value used by Fighter.
        id: Cell<Option<u32>>,
        /// Technical: Stores the `facts` value in `Fighter` as `[Cell<Option<Fact>>; FIELDS]`.
        ///
        /// Plain English: Stores the facts collection for Fighter.
        facts: [Cell<Option<Fact>>; FIELDS],
    }
    /// At most eight fighter identities; an unexpected extra identity reads through.
    /// Construction allocates no heap memory and does not read the game eagerly.
    /// Plain English: At most eight fighter identities; an unexpected extra identity reads through. Construction allocates no heap memory and does not read the game eagerly.
    pub type ObservationSnapshot = ObservationSnapshotSized<8>;
    /// Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
    /// Plain English: Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
    pub struct ObservationSnapshotSized<const N: usize> {
        /// Technical: Stores the `fighters` value in `ObservationSnapshotSized` as `[Fighter; N]`.
        ///
        /// Plain English: Stores the fighters collection for ObservationSnapshotSized.
        fighters: [Fighter; N],
        /// Technical: Stores the `last` value in `ObservationSnapshotSized` as `Cell<Option<usize>>`.
        ///
        /// Plain English: Holds the last value used by ObservationSnapshotSized.
        last: Cell<Option<usize>>,
    }
    impl<const N: usize> Default for ObservationSnapshotSized<N> {
        /// Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
        /// Plain English: Observation snapshot for a fixed number of fighters. Stores copied facts and identities for passive analysis.
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
        /// Plain English: A copied motion clock from an already observed other fighter. No native calls or additional roster scan; passive frozen poses cannot time a tail.
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
        /// Plain English: Already-read statuses only: no native calls, eager reads or allocation. Event consumers can distinguish a known end from a missing observation.
        pub fn known_statuses(&self) -> [Option<(u32, i32)>; N] {
            std::array::from_fn(|i| {
                let entry = &self.fighters[i];
                let id = entry.id.get()?;
                Some((id, entry.facts[Field::Status as usize].get()?.int()?))
            })
        }
        /// Most consumers read several fields from the same fighter in a row.
        /// This cursor only speeds slot lookup; fact lifetime/overflow stay exact.
        /// Plain English: Most consumers read several fields from the same fighter in a row. This cursor only speeds slot lookup; fact lifetime/overflow stay exact.
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
    /// Plain English: One synchronous observation's roster IDs. Empty/unavailable is cached too. No native pointers, allocation, frame throttling, or cross-frame lifetime.
    pub struct RosterSnapshot<const N: usize = 8> {
        /// Technical: Stores the `value` value in `RosterSnapshot` as `Cell<Option<Option<[Option<u32>; N]>>>`.
        ///
        /// Plain English: Holds the value value used by RosterSnapshot.
        value: Cell<Option<Option<[Option<u32>; N]>>>,
    }
    impl<const N: usize> Default for RosterSnapshot<N> {
        /// Technical: Returns the default value for this type.
        ///
        /// Plain English: Returns the default value for this type.
        fn default() -> Self {
            Self {
                value: Cell::new(None),
            }
        }
    }
    impl<const N: usize> RosterSnapshot<N> {
        /// Technical: Returns an optional `read` value.
        ///
        /// Plain English: Gets read if it is available.
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
    /// Plain English: Computes the earliest time of intersection between a swept capsule and an axis-aligned bounding box.
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
    /// Plain English: simple label that tells the generic assist engine what it knows about the fighter It allows the engine to differentiate between known fighters, unknown fighters, and Kirby specifically.  Need to analyse more, may be extended.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum FighterKind {
        /// Technical: Represents the `Kirby` case of `FighterKind`.
        ///
        /// Plain English: Represents the Kirby option in FighterKind.
        Kirby,
        /// Technical: Represents the `Other` case of `FighterKind`.
        ///
        /// Plain English: Represents the Other option in FighterKind.
        Other,
        /// Technical: Represents the `Unknown` case of `FighterKind`.
        ///
        /// Plain English: Represents the Unknown option in FighterKind.
        Unknown,
    }

    /// A normalized situation that can activate an assistance rule.
    ///
    /// The priority order follows the Training Modpack's public mash selection
    /// flow: specific defensive situations win over broad grounded/airborne ones.
    /// Plain English: A normalized situation that can activate an assistance rule. The priority order follows the Training Modpack's public mash selection flow: specific defensive situations win over broad grounded/airborne ones.
    #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
    pub enum Scenario {
        /// Technical: Represents the `Tech` case of `Scenario`.
        ///
        /// Plain English: Represents the Tech option in Scenario.
        Tech,
        /// Technical: Represents the `Clatter` case of `Scenario`.
        ///
        /// Plain English: Represents the Clatter option in Scenario.
        Clatter,
        /// Technical: Represents the `Tumble` case of `Scenario`.
        ///
        /// Plain English: Represents the Tumble option in Scenario.
        Tumble,
        /// Technical: Represents the `Hitstun` case of `Scenario`.
        ///
        /// Plain English: Represents the Hitstun option in Scenario.
        Hitstun,
        /// Technical: Represents the `Parry` case of `Scenario`.
        ///
        /// Plain English: Represents the Parry option in Scenario.
        Parry,
        /// Technical: Represents the `Footstool` case of `Scenario`.
        ///
        /// Plain English: Represents the Footstool option in Scenario.
        Footstool,
        /// Technical: Represents the `Landing` case of `Scenario`.
        ///
        /// Plain English: Represents the Landing option in Scenario.
        Landing,
        /// Technical: Represents the `Ledge` case of `Scenario`.
        ///
        /// Plain English: Represents the Ledge option in Scenario.
        Ledge,
        /// Technical: Represents the `Grounded` case of `Scenario`.
        ///
        /// Plain English: Represents the Grounded option in Scenario.
        Grounded,
        /// Technical: Represents the `Airborne` case of `Scenario`.
        ///
        /// Plain English: Represents the Airborne option in Scenario.
        Airborne,
        // Sentinel for an unused observation/rule slot; it is not an active tool.
        #[default]
        /// Technical: Represents the `None` case of `Scenario`.
        ///
        /// Plain English: Represents the None option in Scenario.
        None,
    }

    impl Scenario {
        /// Lower values have higher precedence when multiple samples are active.
        /// Plain English: Lower values have higher precedence when multiple samples are active.
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
    /// Plain English: A situation supplied by a live or pretend game-state adapter.
    pub struct ScenarioSample {
        /// Technical: Stores the `scenario` value in `ScenarioSample` as `Scenario`.
        ///
        /// Plain English: Holds the scenario value used by ScenarioSample.
        pub scenario: Scenario,
        /// Frame zero is the first frame in this scenario.
        /// Plain English: Frame zero is the first frame in this scenario.
        pub frames_active: u32,
    }

    impl ScenarioSample {
        /// Technical: Defines the `EMPTY` constant used by this module.
        ///
        /// Plain English: Provides the EMPTY constant value.
        pub const EMPTY: Self = Self {
            scenario: Scenario::None,
            frames_active: 0,
        };

        /// Technical: Constructs a new value with the type’s initial state.
        ///
        /// Plain English: Creates a new value.
        pub const fn new(scenario: Scenario, frames_active: u32) -> Self {
            Self {
                scenario,
                frames_active,
            }
        }
    }

    /// A small snapshot with room for overlapping conditions (for example, tumble
    /// and hitstun). The engine chooses the highest-priority active scenario.
    /// Plain English: A small snapshot with room for overlapping conditions (for example, tumble and hitstun). The engine chooses the highest-priority active scenario.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct FighterObservation {
        /// Technical: Stores the `active` value in `FighterObservation` as `[ScenarioSample; Self::MAX_SAMPLES]`.
        ///
        /// Plain English: Stores the active collection for FighterObservation.
        pub active: [ScenarioSample; Self::MAX_SAMPLES],
    }

    impl FighterObservation {
        /// Technical: Sets the upper bound for samples.
        ///
        /// Plain English: Holds the MAX SAMPLES value.
        pub const MAX_SAMPLES: usize = 3;

        /// Technical: Defines the `EMPTY` constant used by this module.
        ///
        /// Plain English: Provides the EMPTY constant value.
        pub const EMPTY: Self = Self {
            active: [ScenarioSample::EMPTY; Self::MAX_SAMPLES],
        };

        /// Technical: Returns the `one` result.
        ///
        /// Plain English: Gets the one result.
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
        /// Technical: Returns an optional `highest_priority` value.
        ///
        /// Plain English: Gets highest priority if it is available.
        pub fn highest_priority(self) -> Option<ScenarioSample> {
            self.active
                .into_iter()
                .filter(|sample| sample.scenario != Scenario::None)
                .min_by_key(|sample| sample.scenario.priority())
        }
    }

    impl Default for FighterObservation {
        /// Technical: Returns the default value for this type.
        ///
        /// Plain English: Returns the default value for this type.
        fn default() -> Self {
            Self::EMPTY
        }
    }

    /// Facts that must be established before the engine may propose an action.
    /// Plain English: Facts that must be established before the engine may propose an action.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct RunContext {
        /// Technical: Stores the `is_locally_human_controlled` value in `RunContext` as `bool`.
        ///
        /// Plain English: Holds the is locally human controlled value used by RunContext.
        pub is_locally_human_controlled: bool,
        /// Technical: Stores the `fighter` value in `RunContext` as `FighterKind`.
        ///
        /// Plain English: Holds the fighter value used by RunContext.
        pub fighter: FighterKind,
    }

    /// Boundary to implement only after a current, documented state-reading route
    /// has been verified. No game hook or memory layout is assumed here.
    // A trait describes methods a type must supply. It is not a concrete adapter;
    // examples/walkthrough.rs implements these methods using pretend data. &self
    // permits reads; &mut self on write_merged_input permits changing adapter state.
    /// Plain English: Boundary to implement only after a current, documented state-reading route has been verified. No game hook or memory layout is assumed here.
    pub trait GameObservationSource {
        /// Technical: Observes the fighter.
        ///
        /// Plain English: Observes the fighter.
        fn observe_fighter(&self) -> Option<FighterObservation>;
    }

    /// Boundary to implement only after a current, documented input route has been
    /// verified. The core requires a read-then-write flow so human input can be
    /// preserved during arbitration.
    /// Plain English: Boundary to implement only after a current, documented input route has been verified. The core requires a read-then-write flow so human input can be preserved during arbitration.
    pub trait InputBridge {
        /// Technical: Reads the human input.
        ///
        /// Plain English: Reads the human input.
        fn read_human_input(&self) -> Option<ControllerFrame>;
        /// Technical: Writes the merged input.
        ///
        /// Plain English: Writes the merged input.
        fn write_merged_input(&mut self, frame: ControllerFrame);
    }

    /// Combined adapter used by the frame runner. Return `None` when no valid
    /// fighter/controller is available (for example during loading or in menus).
    /// Plain English: Combined adapter used by the frame runner. Return None when no valid fighter/controller is available (for example during loading or in menus).
    pub trait GameAdapter: GameObservationSource + InputBridge {
        /// Technical: Reads the context.
        ///
        /// Plain English: Reads the context.
        fn read_context(&self) -> Option<RunContext>;
    }

    // Compiled only by cargo test on the host; these assertions are not live
    // Switch logging or gameplay callbacks. Add regressions here for new behavior.
    #[cfg(test)]
    mod tests {
        use super::*;

        /// Technical: Regression test for `specific_scenario_beats_broad_scenario`.
        ///
        /// Plain English: Checks that specific scenario beats broad scenario.
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
