//! Coalesce unchanged diagnostic snapshots before allocating/formatting strings.
//! Native action edges remain immediate; wall-clock samples explain long waits.
/// Defines the `PERIODIC_DIAGNOSTIC_INTERVAL_MS` constant used by this module.
///
///
///  Provides the PERIODIC DIAGNOSTIC INTERVAL MS constant value.
const PERIODIC_DIAGNOSTIC_INTERVAL_MS: u64 = 500;

/// Defines the `DiagnosticEmission` data structure.
///
///
///  Holds the data for DiagnosticEmission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticEmission {
    /// Stores the `periodic` value in `DiagnosticEmission` as `bool`.
    ///
    ///
    ///  Holds the periodic value used by DiagnosticEmission.
    pub periodic: bool,
    /// Stores the `coalesced_updates` value in `DiagnosticEmission` as `u32`.
    ///
    ///
    ///  Holds the coalesced updates value used by DiagnosticEmission.
    pub coalesced_updates: u32,
}

/// Defines the `DiagnosticGate` data structure.
///
///
///  Holds the data for DiagnosticGate.
pub struct DiagnosticGate<K> {
    /// Stores the `last` value in `DiagnosticGate` as `Option<K>`.
    ///
    ///
    ///  Optionally stores the last for DiagnosticGate; it may be absent.
    last: Option<K>,
    /// Stores the `last_emission_ms` value in `DiagnosticGate` as `Option<u64>`.
    ///
    ///
    ///  Optionally stores the last emission ms for DiagnosticGate; it may be absent.
    last_emission_ms: Option<u64>,
    /// Stores the `coalesced` value in `DiagnosticGate` as `u32`.
    ///
    ///
    ///  Holds the coalesced value used by DiagnosticGate.
    coalesced: u32,
}
impl<K: PartialEq> DiagnosticGate<K> {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    pub const fn new() -> Self {
        Self {
            last: None,
            last_emission_ms: None,
            coalesced: 0,
        }
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    pub fn sample(&mut self, key: K, now_ms: u64, periodic: bool) -> Option<DiagnosticEmission> {
        let changed = self.last.as_ref() != Some(&key);
        let sample_due = periodic
            && self.last_emission_ms.is_none_or(|last| {
                now_ms.saturating_sub(last) >= PERIODIC_DIAGNOSTIC_INTERVAL_MS || now_ms < last
            });
        if changed || sample_due {
            let emission = DiagnosticEmission {
                periodic: !changed,
                coalesced_updates: self.coalesced,
            };
            self.last = Some(key);
            self.last_emission_ms = Some(now_ms);
            self.coalesced = 0;
            Some(emission)
        } else {
            self.coalesced = self.coalesced.saturating_add(1);
            None
        }
    }
    /// Bound bulky progress logs even when readiness/requests chatter. Dedicated
    /// grab traces preserve critical edges separately before this throttle.
    ///
    ///  Bound bulky progress logs even when readiness/requests chatter. Dedicated grab traces preserve critical edges separately before this throttle.
    pub fn sample_throttled(
        &mut self,
        key: K,
        now_ms: u64,
        periodic: bool,
        minimum_ms: u64,
    ) -> Option<DiagnosticEmission> {
        if self
            .last_emission_ms
            .is_some_and(|last| now_ms >= last && now_ms - last < minimum_ms)
        {
            self.coalesced = self.coalesced.saturating_add(1);
            return None;
        }
        self.sample(key, now_ms, periodic)
    }
}
impl<K: PartialEq> Default for DiagnosticGate<K> {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Regression test for `changing_progress_keys_cannot_bypass_the_formatting_rate_limit`.
    ///
    ///
    ///  Checks that changing progress keys cannot bypass the formatting rate limit.
    #[test]
    fn changing_progress_keys_cannot_bypass_the_formatting_rate_limit() {
        let mut gate = DiagnosticGate::new();
        let emissions = (0..=1000)
            .filter_map(|ms| gate.sample_throttled(ms, ms, true, 125))
            .collect::<Vec<_>>();
        assert_eq!(emissions.len(), 9);
        assert_eq!(emissions[1].coalesced_updates, 124);
        // A backwards/reset clock rearms instead of silencing diagnostics.
        assert!(gate.sample_throttled(2000, 0, true, 125).is_some());
    }
    /// Regression test for `waiting_frames_are_coalesced_but_attack_and_hit_events_are_immediate`.
    ///
    ///
    ///  Checks that waiting frames are coalesced but attack and hit events are immediate.
    #[test]
    fn waiting_frames_are_coalesced_but_attack_and_hit_events_are_immediate() {
        let mut gate = DiagnosticGate::new();
        assert!(!gate.sample("roll_wait", 0, true).unwrap().periodic);
        for frame in 1..30 {
            assert!(gate.sample("roll_wait", frame * 16, true).is_none());
        }
        let attack = gate.sample("smash_request", 480, true).unwrap();
        assert!(!attack.periodic);
        assert_eq!(attack.coalesced_updates, 29);
        assert!(!gate.sample("hit_confirmed", 496, true).unwrap().periodic);
    }
    /// Regression test for `verbose_waits_emit_at_most_two_periodic_snapshots_per_second`.
    ///
    ///
    ///  Checks that verbose waits emit at most two periodic snapshots per second.
    #[test]
    fn verbose_waits_emit_at_most_two_periodic_snapshots_per_second() {
        let mut gate = DiagnosticGate::new();
        let emissions = (0..=1000)
            .filter_map(|ms| gate.sample(1, ms, true))
            .collect::<Vec<_>>();
        assert_eq!(emissions.len(), 3); // Initial event, then 500ms and 1000ms.
        assert_eq!(emissions.iter().filter(|event| event.periodic).count(), 2);
        assert_eq!(emissions[1].coalesced_updates, 499);
    }
    /// Regression test for `nonverbose_mode_only_logs_events_and_reset_rearms_first_snapshot`.
    ///
    ///
    ///  Checks that nonverbose mode only logs events and reset rearms first snapshot.
    #[test]
    fn nonverbose_mode_only_logs_events_and_reset_rearms_first_snapshot() {
        let mut gate = DiagnosticGate::new();
        assert!(gate.sample(1, 0, false).is_some());
        assert!(gate.sample(1, 10_000, false).is_none());
        assert!(gate.sample(2, 10_001, false).is_some());
        gate = DiagnosticGate::new();
        assert!(gate.sample(2, 10_002, false).is_some());
    }
}

#[cfg(feature = "diagnostic-logging")]
pub mod modes {
    //! Diagnostic-only copied evidence. No game reads, formatting or allocation in sample.
    use crate::{action_owner::Owner, native_input::*};
    use std::fmt::Write;

    /// Defines the `OWNER_COUNT` constant used by this module.
    ///
    ///
    ///  Provides the OWNER COUNT constant value.
    const OWNER_COUNT: usize = 13;
    /// Defines the `REPORT_TEXT_CAPACITY_BYTES` constant used by this module.
    ///
    ///
    ///  Provides the REPORT TEXT CAPACITY BYTES constant value.
    const REPORT_TEXT_CAPACITY_BYTES: usize = 1200;
    /// Sets the lower bound for report interval ms.
    ///
    ///
    ///  Holds the MIN REPORT INTERVAL MS value.
    const MIN_REPORT_INTERVAL_MS: u64 = 250;
    /// Defines the `LABEL_CAPACITY_BYTES` constant used by this module.
    ///
    ///
    ///  Provides the LABEL CAPACITY BYTES constant value.
    const LABEL_CAPACITY_BYTES: usize = 64;
    /// Defines the `CONFIGURATION_ROW_COUNT` constant used by this module.
    ///
    ///
    ///  Provides the CONFIGURATION ROW COUNT constant value.
    const CONFIGURATION_ROW_COUNT: usize = 25;
    /// Defines the `LEDGE_HELP` constant used by this module.
    ///
    ///
    ///  Provides the LEDGE HELP constant value.
    const LEDGE_HELP: &str = "Hold the ledge tool; waits for the matching ledge option, safe spacing and protection to end.";
    /// Defines the `RECIPE_HELP` constant used by this module.
    ///
    ///
    ///  Provides the RECIPE HELP constant value.
    const RECIPE_HELP: &str = "An assigned recipe runs once per fresh button press while the menu is closed; Native preserves the existing control.";
    /// Defines the `BINDING_HELP` constant used by this module.
    ///
    ///
    ///  Provides the BINDING HELP constant value.
    const BINDING_HELP: &str =
        "Binding for this D-pad direction; Normal keeps the existing control.";
    /// Row order must match the root menu's configuration rows. An empty label
    /// marks a removed slot that is kept only so later rows stay aligned.
    ///
    ///  Row order must match the root menu's configuration rows. An empty label marks a removed slot that is kept only so later rows stay aligned.
    const CONFIGURATION_ROWS: [(&str, &str); CONFIGURATION_ROW_COUNT] = [
        ("Neutral getup", LEDGE_HELP),
        ("Jump getup", LEDGE_HELP),
        ("Ledge roll", LEDGE_HELP),
        ("Getup attack", LEDGE_HELP),
        ("Ledge drop", LEDGE_HELP),
        ("Onstage roll read", "Watches approaching/crossing ground rolls; chooses the configured punish inside recovery and reach."),
        ("Y binding", "Existing Y mode, used while Y sequence is set to Native."),
        ("Auto mash", "Acts only during supported capture/mash states, then returns physical control."),
        ("Idle crouch", "Grounded idle within 45 horizontal / 20 vertical units of an enemy; movement, actions or held L suspend crouching."),
        ("Turnaround tilt", "Aims a physical down tilt toward the enemy; does not generate an attack press."),
        ("", ""),
        ("Spot-dodge punish", "Reads enemy spot-dodge recovery; selects near down smash or farther forward smash."),
        ("Grab dodge", "Detects ordinary grab threats, requests spot dodge, then watches recovery for a smash punish."),
        ("Auto tech", "Handles native tech windows; physical input can still override."),
        ("Ledge trump", "Uses the held trump trigger, a known floor edge and native movement readiness."),
        ("Performance gauge", "Records the training performance gauge to performance.log."),
        ("X button recipe", RECIPE_HELP),
        ("Y button recipe", RECIPE_HELP),
        ("D-pad Up binding", BINDING_HELP),
        ("D-pad Down binding", BINDING_HELP),
        ("D-pad Left binding", BINDING_HELP),
        ("D-pad Right binding", BINDING_HELP),
        ("Ledge trap mode", "Uses the saved ledge response; waits for protection, reach and native readiness."),
        ("Out of shield", "Confirms shield contact, retains defense through stun/multihits and selects the fighter-specific in-range response; yields to ledge responses."),
        ("Shield pressure", "Watches enemy shields and shield-origin rolls; selects a configured pressure response without interrupting protected ledge actions."),
    ];
    /// Defines the `RESULT_TOP_MARGIN` constant used by this module.
    ///
    ///
    ///  Provides the RESULT TOP MARGIN constant value.
    const RESULT_TOP_MARGIN: f32 = 14.0;
    /// Defines the `RESULT_BOTTOM_MARGIN` constant used by this module.
    ///
    ///
    ///  Provides the RESULT BOTTOM MARGIN constant value.
    const RESULT_BOTTOM_MARGIN: f32 = 8.0;
    /// Sets the lower bound for result height.
    ///
    ///
    ///  Holds the MIN RESULT HEIGHT value.
    const MIN_RESULT_HEIGHT: f32 = 40.0;
    /// Defines the `SAFETY_TECH_GUARD` constant used by this module.
    ///
    ///
    ///  Provides the SAFETY TECH GUARD constant value.
    pub const SAFETY_TECH_GUARD: u8 = 1 << 0;
    /// Defines the `SAFETY_MASH` constant used by this module.
    ///
    ///
    ///  Provides the SAFETY MASH constant value.
    pub const SAFETY_MASH: u8 = 1 << 1;
    /// Defines the `SAFETY_SDI` constant used by this module.
    ///
    ///
    ///  Provides the SAFETY SDI constant value.
    pub const SAFETY_SDI: u8 = 1 << 2;
    /// Defines the `SAFETY_DI` constant used by this module.
    ///
    ///
    ///  Provides the SAFETY DI constant value.
    pub const SAFETY_DI: u8 = 1 << 3;

    /// Defines the `OWNERS` constant used by this module.
    ///
    ///
    ///  Provides the OWNERS constant value.
    pub const OWNERS: [Owner; OWNER_COUNT] = [
        Owner::Safety,
        Owner::OptionRead,
        Owner::Ledge,
        Owner::RollRead,
        Owner::Trip,
        Owner::Fair,
        Owner::Trump,
        Owner::Defense,
        Owner::Turbo,
        Owner::Posture,
        Owner::OutOfShield,
        Owner::GrabDodge,
        Owner::Reflect,
    ];
    /// Tests whether the requested bit is set in the value.
    ///
    ///
    ///  Checks whether this bit is on.
    pub fn bit(owner: Owner) -> u32 {
        1 << OWNERS.iter().position(|v| *v == owner).unwrap()
    }
    /// Returns the display name associated with the value.
    ///
    ///
    ///  Gets the name shown for this value.
    pub fn name(owner: Owner) -> &'static str {
        match owner {
            Owner::Safety => "Tech / mash / SDI / DI",
            Owner::Reflect => "Projectile reflect / absorb",
            Owner::GrabDodge => "Incoming grab dodge",
            Owner::OptionRead => "Configured dodge follow-up / recovery punish",
            Owner::Ledge => "Ledge trapping",
            Owner::RollRead => "Roll / shield-pressure rules",
            Owner::Trip => "Trip follow-up",
            Owner::Fair => "Fair-hit follow-up",
            Owner::Trump => "Ledge trump",
            Owner::Defense => "Defensive spot dodge / shield / duck",
            Owner::Turbo => "Turbo movement",
            Owner::Posture => "Idle crouch / turnaround tilt",
            Owner::OutOfShield => "Out of shield",
        }
    }
    /// Describe each tool's prerequisites without guessing a particular failed gate.
    ///
    ///  Describe each tool's prerequisites without guessing a particular failed gate.
    pub fn purpose(owner: Owner) -> &'static str {
        match owner {
            Owner::OutOfShield => "Confirms shield contact, retains defense through stun/multihits and selects the fighter-specific in-range response; yields to ledge responses.",
            Owner::Reflect => "Uses the configured native reflector for an incoming eligible projectile; yields to ledge, safety, manual attacks and committed punish/grab input. A full bucket is never discharged automatically.",
            Owner::Safety => "Handles native tech windows, capture escape, hitstop SDI and launch DI; physical stick override can cancel DI.",
            Owner::GrabDodge => "Requests a native-ready spot dodge for an incoming ordinary grab; may preempt general punishes, but yields to protected ledge responses and safety.",
            Owner::OptionRead => "Runs configured spot-dodge/grab-dodge follow-ups and retained recovery punishes; yields to incoming grab defense.",
            Owner::RollRead => "Watches approaching/crossing rolls; primes the chosen smash/grab only inside its timing and reach window.",
            Owner::Ledge => "Uses the saved response for neutral/jump/roll/attack/drop; waits for protection, reach and native readiness.",
            Owner::Trip => "Requires an observed trip/hit before buffering its grounded follow-up.",
            Owner::Fair => "A fresh Kirby fair-one hit arms the landing smash; shield/stale flags do not arm it.",
            Owner::Trump => "Uses the held trump trigger, a known floor edge and native movement readiness.",
            Owner::Defense => "Defensive shield/duck proposals yield to reads and ledge sequences; low hitboxes cannot safely be ducked.",
            Owner::Turbo => "Repeats the configured movement while its trigger and native movement scope permit it.",
            Owner::Posture => "Crouches while grounded and idle near an enemy, or aims configured physical tilts; movement/buttons suspend idle crouch.",
        }
    }
    /// Returns the display name for the observed motion.
    ///
    ///
    ///  Gets the motion’s display name.
    pub fn motion_name(status: i32, motion: u64) -> &'static str {
        /// Defines the `ATTACK_AIR_NAMES` constant used by this module.
        ///
        ///
        ///  Provides the ATTACK AIR NAMES constant value.
        const ATTACK_AIR_NAMES: [(u64, &str); 5] = [
            (crate::timing::motion_hash("attack_air_n"), "neutral air"),
            (crate::timing::motion_hash("attack_air_f"), "forward air"),
            (crate::timing::motion_hash("attack_air_b"), "back air"),
            (crate::timing::motion_hash("attack_air_hi"), "up air"),
            (crate::timing::motion_hash("attack_air_lw"), "down air"),
        ];
        if status == crate::native_states::status::ATTACK_AIR {
            if let Some((_, name)) = ATTACK_AIR_NAMES.iter().find(|(hash, _)| *hash == motion) {
                return name;
            }
        }
        native_name(status)
    }
    /// Maps the native action identifier to its diagnostic label.
    ///
    ///
    ///  Gets the readable name for the native action.
    fn native_action(status: i32) -> u16 {
        match status {
            crate::native_states::status::JUMP_SQUAT
                ..=crate::native_states::status::FALL_SPECIAL => NATIVE_ACTION_AIRBORNE,
            crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE => {
                NATIVE_ACTION_SHIELD
            }
            crate::native_states::ESCAPE => NATIVE_ACTION_SPOT_DODGE,
            crate::native_states::ESCAPE_F..=crate::native_states::ESCAPE_AIR_SLIDE => {
                NATIVE_ACTION_ROLL_OR_AIR_DODGE
            }
            crate::native_states::status::ATTACK..=crate::native_states::status::ATTACK_HI4 => {
                NATIVE_ACTION_GROUND_ATTACK
            }
            crate::native_states::status::ATTACK_AIR => NATIVE_ACTION_AERIAL_ATTACK,
            crate::native_states::status::CATCH..=crate::native_states::status::CATCH_JUMP => {
                NATIVE_ACTION_GRAB
            }
            crate::native_states::status::DAMAGE..=crate::native_states::status::PASSIVE_CEIL => {
                NATIVE_ACTION_DAMAGE_OR_TECH
            }
            crate::native_states::status::CLIFF_CATCH_MOVE
                ..=crate::native_states::status::CLIFF_JUMP2 => NATIVE_ACTION_LEDGE_ACTION,
            _ => NATIVE_ACTION_NONE,
        }
    }
    /// Defines the `NATIVE_ACTION_NONE` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION NONE constant value.
    const NATIVE_ACTION_NONE: u16 = 0;
    /// Defines the `NATIVE_ACTION_AIRBORNE` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION AIRBORNE constant value.
    const NATIVE_ACTION_AIRBORNE: u16 = 1;
    /// Defines the `NATIVE_ACTION_SHIELD` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION SHIELD constant value.
    const NATIVE_ACTION_SHIELD: u16 = 2;
    /// Defines the `NATIVE_ACTION_SPOT_DODGE` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION SPOT DODGE constant value.
    const NATIVE_ACTION_SPOT_DODGE: u16 = 4;
    /// Defines the `NATIVE_ACTION_ROLL_OR_AIR_DODGE` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION ROLL OR AIR DODGE constant value.
    const NATIVE_ACTION_ROLL_OR_AIR_DODGE: u16 = 8;
    /// Defines the `NATIVE_ACTION_GROUND_ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION GROUND ATTACK constant value.
    const NATIVE_ACTION_GROUND_ATTACK: u16 = 16;
    /// Defines the `NATIVE_ACTION_AERIAL_ATTACK` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION AERIAL ATTACK constant value.
    const NATIVE_ACTION_AERIAL_ATTACK: u16 = 32;
    /// Defines the `NATIVE_ACTION_GRAB` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION GRAB constant value.
    const NATIVE_ACTION_GRAB: u16 = 64;
    /// Defines the `NATIVE_ACTION_DAMAGE_OR_TECH` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION DAMAGE OR TECH constant value.
    const NATIVE_ACTION_DAMAGE_OR_TECH: u16 = 128;
    /// Defines the `NATIVE_ACTION_LEDGE_ACTION` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION LEDGE ACTION constant value.
    const NATIVE_ACTION_LEDGE_ACTION: u16 = 256;
    /// Defines the `NATIVE_ACTION_LABELS` constant used by this module.
    ///
    ///
    ///  Provides the NATIVE ACTION LABELS constant value.
    const NATIVE_ACTION_LABELS: [(u16, &str); 9] = [
        (NATIVE_ACTION_AIRBORNE, "jump/fall"),
        (NATIVE_ACTION_SHIELD, "shield"),
        (NATIVE_ACTION_SPOT_DODGE, "spot dodge"),
        (NATIVE_ACTION_ROLL_OR_AIR_DODGE, "roll/air dodge"),
        (NATIVE_ACTION_GROUND_ATTACK, "ground attack"),
        (NATIVE_ACTION_AERIAL_ATTACK, "aerial attack"),
        (NATIVE_ACTION_GRAB, "grab/capture"),
        (NATIVE_ACTION_DAMAGE_OR_TECH, "damage/knockdown"),
        (NATIVE_ACTION_LEDGE_ACTION, "ledge action"),
    ];
    /// Buttons whose labels a mode's requests are summarized by.
    ///
    ///  Buttons whose labels a mode's requests are summarized by.
    const REQUEST_LABELS: [(u32, &str); 5] = [
        (MAPPER_X | MAPPER_ZL, "Jump"),
        (MAPPER_A, "Attack"),
        (MAPPER_R, "Grab"),
        (MAPPER_B, "Special"),
        (MAPPER_ZR, "Shield"),
    ];
    /// Maps the native identifier to its diagnostic name.
    ///
    ///
    ///  Gets the readable name for the native value.
    pub fn native_name(status: i32) -> &'static str {
        match status {
            crate::native_states::status::WAIT => "standing",
            crate::native_states::status::WALK..=crate::native_states::status::WALK_BRAKE => {
                "walking"
            }
            crate::native_states::status::DASH..=crate::native_states::status::TURN_RUN_BRAKE => {
                "running / turning"
            }
            crate::native_states::status::JUMP_SQUAT => "jump squat",
            crate::native_states::status::JUMP..=crate::native_states::status::FALL_SPECIAL => {
                "airborne"
            }
            crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV => {
                "crouching"
            }
            crate::native_states::status::LANDING
                ..=crate::native_states::status::LANDING_ATTACK_AIR => "landing",
            crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE => {
                "shield / shield stun"
            }
            crate::native_states::ESCAPE => "spot dodge",
            crate::native_states::ESCAPE_F..=crate::native_states::ESCAPE_B => "roll dodge",
            crate::native_states::ESCAPE_AIR..=crate::native_states::ESCAPE_AIR_SLIDE => {
                "air dodge"
            }
            crate::native_states::status::ATTACK => "jab",
            crate::native_states::status::ATTACK_100
                ..=crate::native_states::status::ATTACK_DASH => "jab follow-up / dash attack",
            crate::native_states::status::ATTACK_S3 => "forward tilt",
            crate::native_states::status::ATTACK_HI3 => "up tilt",
            crate::native_states::status::ATTACK_LW3 => "down tilt",
            crate::native_states::status::ATTACK_S4_START
                ..=crate::native_states::status::ATTACK_S4 => "forward smash",
            crate::native_states::status::ATTACK_LW4_START
                ..=crate::native_states::status::ATTACK_LW4 => "down smash",
            crate::native_states::status::ATTACK_HI4_START
                ..=crate::native_states::status::ATTACK_HI4 => "up smash",
            crate::native_states::status::ATTACK_AIR => "aerial attack",
            crate::native_states::CATCH => "standing grab startup",
            crate::native_states::CATCH_DASH => "dash grab startup",
            crate::native_states::CATCH_TURN => "pivot grab startup",
            crate::native_states::CATCH_PULL
            | crate::native_states::CATCH_DASH_PULL
            | crate::native_states::status::CATCH_WAIT..=crate::native_states::status::CATCH_JUMP => {
                "grab pull / hold / pummel"
            }
            crate::native_states::status::SHIELD_BREAK_FLY
                ..=crate::native_states::status::PASSIVE_CEIL => "knockdown / getup",
            crate::native_states::status::DAMAGE
                ..=crate::native_states::status::DOWN_REFLECT_LR => "damage / hitstun",
            crate::native_states::status::CLIFF_CATCH_MOVE
                ..=crate::native_states::status::CLIFF_JUMP2 => "ledge action",
            _ => "other native action",
        }
    }

    /// Defines the `Snapshot` data structure.
    ///
    ///
    ///  Holds the data for Snapshot.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Snapshot {
        /// Stores the `winner` value in `Snapshot` as `Option<Owner>`.
        ///
        ///
        ///  Optionally stores the winner for Snapshot; it may be absent.
        pub winner: Option<Owner>,
        /// Stores the `present` value in `Snapshot` as `u32`.
        ///
        ///
        ///  Holds the present value used by Snapshot.
        pub present: u32,
        /// Stores the `committed` value in `Snapshot` as `bool`.
        ///
        ///
        ///  Holds the committed value used by Snapshot.
        pub committed: bool,
        /// Stores the `requested` value in `Snapshot` as `u32`.
        ///
        ///
        ///  Holds the requested value used by Snapshot.
        pub requested: u32,
        /// Quantized directions only; analog jitter must not create log spam.
        ///
        ///  Quantized directions only; analog jitter must not create log spam.
        pub stick: Option<[i8; 2]>,
        /// Stores the `status` value in `Snapshot` as `i32`.
        ///
        ///
        ///  Holds the status value used by Snapshot.
        pub status: i32,
        /// Stores the `motion` value in `Snapshot` as `u64`.
        ///
        ///
        ///  Holds the motion value used by Snapshot.
        pub motion: u64,
        /// Stores the `native_frame` value in `Snapshot` as `u16`.
        ///
        ///
        ///  Holds the native frame value used by Snapshot.
        pub native_frame: u16,
        /// Stores the `stopped` value in `Snapshot` as `bool`.
        ///
        ///
        ///  Holds the stopped value used by Snapshot.
        pub stopped: bool,
        /// Stores the `can_act` value in `Snapshot` as `bool`.
        ///
        ///
        ///  Holds the can act value used by Snapshot.
        pub can_act: bool,
        /// Stores the `hit` value in `Snapshot` as `Option<bool>`.
        ///
        ///
        ///  Optionally stores the hit for Snapshot; it may be absent.
        pub hit: Option<bool>,
        /// Stores the `valid` value in `Snapshot` as `bool`.
        ///
        ///
        ///  Holds the valid value used by Snapshot.
        pub valid: bool,
        /// Flags are the `SAFETY_*` constants in this module.
        ///
        ///  Flags are the SAFETY constants in this module.
        pub safety: u8,
    }
    /// Defines the `Evidence` data structure.
    ///
    ///
    ///  Holds the data for Evidence.
    #[derive(Clone, Copy, Default)]
    struct Evidence {
        /// Stores the `proposed` value in `Evidence` as `bool`.
        ///
        ///
        ///  Holds the proposed value used by Evidence.
        proposed: bool,
        /// Stores the `selected` value in `Evidence` as `u16`.
        ///
        ///
        ///  Holds the selected value used by Evidence.
        selected: u16,
        /// Stores the `requests` value in `Evidence` as `u32`.
        ///
        ///
        ///  Holds the requests value used by Evidence.
        requests: u32,
        /// Stores the `stick` value in `Evidence` as `Option<[i8; 2]>`.
        ///
        ///
        ///  Optionally stores the stick for Evidence; it may be absent.
        stick: Option<[i8; 2]>,
    }
    /// Defines the `Report` data structure.
    ///
    ///
    ///  Holds the data for Report.
    #[derive(Clone, Copy)]
    pub struct Report {
        /// Stores the `first` value in `Report` as `Snapshot`.
        ///
        ///
        ///  Holds the first value used by Report.
        pub first: Snapshot,
        /// Stores the `last` value in `Report` as `Snapshot`.
        ///
        ///
        ///  Holds the last value used by Report.
        pub last: Snapshot,
        /// Stores the `at_ms` value in `Report` as `u64`.
        ///
        ///
        ///  Holds the at ms value used by Report.
        pub at_ms: u64,
        /// Stores the `changes` value in `Report` as `u16`.
        ///
        ///
        ///  Holds the changes value used by Report.
        changes: u16,
        /// Stores the `modes` value in `Report` as `[Evidence; OWNERS.len()]`.
        ///
        ///
        ///  Stores the modes collection for Report.
        modes: [Evidence; OWNERS.len()],
        /// Stores the `hit_seen` value in `Report` as `bool`.
        ///
        ///
        ///  Holds the hit seen value used by Report.
        hit_seen: bool,
        /// Stores the `hit_sampled` value in `Report` as `bool`.
        ///
        ///
        ///  Holds the hit sampled value used by Report.
        hit_sampled: bool,
        /// Stores the `native_actions` value in `Report` as `u16`.
        ///
        ///
        ///  Holds the native actions value used by Report.
        native_actions: u16,
    }
    impl Report {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        fn new(snapshot: Snapshot, at_ms: u64) -> Self {
            Self {
                first: snapshot,
                last: snapshot,
                at_ms,
                changes: 0,
                modes: [Evidence::default(); OWNERS.len()],
                hit_seen: false,
                hit_sampled: false,
                native_actions: native_action(snapshot.status),
            }
        }
        /// Records the time of the latest fighter observation.
        ///
        ///
        ///  Remembers when a fighter was last seen.
        fn observe(&mut self, s: Snapshot) {
            self.native_actions |= native_action(s.status);
            self.last = s;
            self.changes = self.changes.saturating_add(1);
            self.hit_seen |= s.hit == Some(true);
            self.hit_sampled |= s.hit.is_some();
            for (index, owner) in OWNERS.iter().enumerate() {
                let e = &mut self.modes[index];
                e.proposed |= s.present & (1 << index) != 0;
                if s.winner == Some(*owner) {
                    e.selected = e.selected.saturating_add(1);
                    e.requests |= s.requested;
                    e.stick = s.stick;
                }
            }
        }
        /// Compact menu snapshot, formatted only by the background writer.
        ///
        ///  Compact menu snapshot, formatted only by the background writer.
        pub fn card_text(&self) -> String {
            let s = self.last;
            let owner = s.winner.map_or("Physical controls", name);
            let state = if !s.valid {
                "Paused: gameplay/input observation unavailable"
            } else if s.stopped {
                "Native stop: retaining input; timers frozen"
            } else if s.requested != 0 {
                "Requesting input; waiting for native startup"
            } else if s.committed {
                "Sequence active: preparing or releasing input"
            } else if s.winner.is_some() {
                "Watching / preparing"
            } else {
                "No assisted input pending"
            };
            let mut text = format!(
                "LATEST SNAPSHOT +{}ms\n{}\n{}\nGame: {} f{}\n",
                self.at_ms,
                owner,
                state,
                motion_name(s.status, s.motion),
                s.native_frame
            );
            let waiting = OWNERS
                .iter()
                .filter(|owner| s.present & bit(**owner) != 0 && Some(**owner) != s.winner)
                .count();
            if waiting > 0 {
                let _ = writeln!(text, "Other proposals yielding priority: {waiting}");
            }
            let _ = writeln!(
                text,
                "Hit: {}. Full details: status.log",
                if self.hit_seen {
                    "flag observed; victim unverified"
                } else {
                    "not confirmed"
                }
            );
            text
        }
        /// Called by the background writer, never the input hook.
        ///
        ///  Called by the background writer, never the input hook.
        pub fn format(&self) -> String {
            let mut text = String::with_capacity(REPORT_TEXT_CAPACITY_BYTES);
            let last = self.last;
            let winner = winner_name(last.winner);
            let _ = writeln!(
                text,
                "MODE EXPLANATION +{}ms ({} changes batched)",
                self.at_ms, self.changes
            );
            let _ = writeln!(text, "Now: {winner}. Why: {}.", why_text(last));
            if self.first.winner != last.winner {
                let _ = writeln!(text, "Control changed: {} -> {}. Earlier control ended or yielded to priority; this alone does not establish a hit or miss.", winner_name(self.first.winner), winner);
            }

            let _ = writeln!(
                text,
                "Fighter: {} -> {} (status {:#x}); {}.",
                native_name(self.first.status),
                native_name(last.status),
                last.status,
                if last.stopped {
                    "native stop is active; timers must not advance"
                } else if last.can_act {
                    "ready for a new grounded action"
                } else {
                    "new grounded actions are unavailable during this animation"
                }
            );
            let _ = writeln!(
                text,
                "Latest animation: {} f{} (motion {:#x}).",
                motion_name(last.status, last.motion),
                last.native_frame,
                last.motion
            );
            let observed: Vec<_> = NATIVE_ACTION_LABELS
                .into_iter()
                .filter_map(|(mask, label)| (self.native_actions & mask != 0).then_some(label))
                .collect();
            let _ = writeln!(text, "Game states observed in batch: {}. These are observations, not proof a particular request caused them.", if observed.is_empty() { "no action startup sampled".to_owned() } else { observed.join(", ") });
            for (index, owner) in OWNERS.iter().enumerate() {
                let e = self.modes[index];
                if !e.proposed && e.selected == 0 {
                    continue;
                }
                let _ = writeln!(text, "  {} — {}", name(*owner), purpose(*owner));
                let _ = write!(text, "    Input: ");
                write_input_evidence(&mut text, e, last.valid);
            }
            if last.safety != 0 {
                let _ = writeln!(
                    text,
                    "Safety flags: tech={}, mash={}, SDI={}, DI={}.",
                    last.safety & SAFETY_TECH_GUARD != 0,
                    last.safety & SAFETY_MASH != 0,
                    last.safety & SAFETY_SDI != 0,
                    last.safety & SAFETY_DI != 0
                );
            }
            let _ = write!(text, "Evidence: {}. Requests above describe inputs sent before mapping; they do not prove the selected move started or hit. Button names are a union across the batch, not necessarily simultaneous presses. Native state is the latest copied observation. +ms is wall-clock time, not a simulation-frame count.",
                if self.hit_seen { "a hit flag was observed; victim and responsible mode are unverified" }
                else if !self.hit_sampled { "hit evidence was not sampled for these observations" }
                else { "no hit flag was observed in this batch" });
            text
        }
    }

    /// Returns the diagnostic label for the selected winner.
    ///
    ///
    ///  Gets the name shown for the selected winner.
    fn winner_name(winner: Option<Owner>) -> &'static str {
        winner.map_or("physical controls", name)
    }

    /// Formats the explanation associated with the decision.
    ///
    ///
    ///  Builds the text explaining the decision.
    fn why_text(last: Snapshot) -> &'static str {
        if !last.valid {
            "assistance paused: observation or gameplay scope is unavailable"
        } else if last.winner == Some(Owner::Safety) {
            "native tech, mash, SDI or launch DI has emergency priority"
        } else if last.committed {
            "a committed sequence retains its input and release frames"
        } else if last.winner == Some(Owner::Ledge) && last.requested == 0 {
            "ledge watch/preparation is reserved ahead of optional helpers"
        } else if last.winner == Some(Owner::RollRead) && last.requested == 0 {
            "roll-read preparation owns the stick before requesting an attack"
        } else if last.winner == Some(Owner::Posture) {
            "no higher-priority mode needs control; this helper only aims the stick, leaving attack buttons physical"
        } else if last.winner.is_some() {
            "this eligible proposal won input priority"
        } else {
            "no mode currently proposes an eligible input"
        }
    }

    /// Writes the input evidence.
    ///
    ///
    ///  Writes the input evidence.
    fn write_input_evidence(text: &mut String, e: Evidence, valid: bool) {
        if e.selected == 0 {
            let _ = writeln!(
                text,
                "{}",
                if !valid {
                    "proposed, but never applied: gameplay or observation scope was unavailable."
                } else {
                    "proposed, but never applied in this batch; another mode retained priority."
                }
            );
            return;
        }
        let _ = write!(text, "selected; requested button types during batch: ");
        if e.requests == 0 {
            let _ = write!(text, "no action buttons (preparation, hold or release)");
        } else {
            let mut separator = "";
            for (mask, label) in REQUEST_LABELS {
                if e.requests & mask != 0 {
                    let _ = write!(text, "{separator}{label}");
                    separator = ", ";
                }
            }
            let _ = write!(text, " (raw {:#x})", e.requests);
        }
        let _ = write!(text, ". Stick: ");
        match e.stick {
            None => {
                let _ = write!(text, "physical");
            }
            Some([x, y]) => {
                let horizontal = match x {
                    -1 => "left",
                    1 => "right",
                    _ => "neutral horizontal",
                };
                let vertical = match y {
                    -1 => "down",
                    1 => "up",
                    _ => "neutral vertical",
                };
                let _ = write!(text, "{horizontal}, {vertical}");
            }
        }
        let _ = writeln!(text, ".");
    }

    /// Defines the `Recorder` data structure.
    ///
    ///
    ///  Holds the data for Recorder.
    pub struct Recorder {
        /// Stores the `previous` value in `Recorder` as `Option<Snapshot>`.
        ///
        ///
        ///  Optionally stores the previous for Recorder; it may be absent.
        previous: Option<Snapshot>,
        /// Stores the `pending` value in `Recorder` as `Option<Report>`.
        ///
        ///
        ///  Optionally stores the pending for Recorder; it may be absent.
        pending: Option<Report>,
        /// Stores the `last_ms` value in `Recorder` as `Option<u64>`.
        ///
        ///
        ///  Optionally stores the last ms for Recorder; it may be absent.
        last_ms: Option<u64>,
    }
    impl Default for Recorder {
        /// Returns the default value for this type.
        ///
        ///
        ///  Returns the default value for this type.
        fn default() -> Self {
            Self::new()
        }
    }
    impl Recorder {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        pub const fn new() -> Self {
            Self {
                previous: None,
                pending: None,
                last_ms: None,
            }
        }
        /// At most four fixed-size batches per second. Unchanged polls do no work.
        ///
        ///  At most four fixed-size batches per second. Unchanged polls do no work.
        pub fn sample(&mut self, snapshot: Snapshot, ms: u64) -> Option<Report> {
            if self.last_ms.is_some_and(|last| ms < last) {
                *self = Self::new();
            }
            let relevant = snapshot.present != 0
                || snapshot.winner.is_some()
                || self
                    .previous
                    .is_some_and(|p| p.present != 0 || p.winner.is_some());
            // Animation frame numbers enrich reports but must not make a stable
            // crouch/watch mode produce four log messages every second forever.
            let changed = self.previous.map(|mut previous| {
                previous.native_frame = snapshot.native_frame;
                previous
            }) != Some(snapshot);
            if relevant && changed {
                self.pending
                    .get_or_insert_with(|| Report::new(self.previous.unwrap_or(snapshot), ms))
                    .observe(snapshot);
            }
            self.previous = Some(snapshot);
            if self
                .last_ms
                .is_none_or(|last| ms.saturating_sub(last) >= MIN_REPORT_INTERVAL_MS)
            {
                // Move the report out so it is emitted only once; later changes
                // can build a fresh pending report without cloning the payload.
                if let Some(report) = self.pending.take() {
                    self.last_ms = Some(ms);
                    return Some(Report {
                        at_ms: ms,
                        ..report
                    });
                }
            }
            None
        }
    }

    /// Keep completed results below the menu; hide rather than overlap if it fills the screen.
    ///
    ///  Keep completed results below the menu; hide rather than overlap if it fills the screen.
    pub fn left_result_space(
        display_height: f32,
        scale: f32,
        menu_height: f32,
    ) -> Option<(f32, f32)> {
        if !display_height.is_finite()
            || !scale.is_finite()
            || !menu_height.is_finite()
            || display_height <= 0.0
            || scale <= 0.0
            || menu_height < 0.0
        {
            return None;
        }
        let y = RESULT_TOP_MARGIN * scale + menu_height;
        let remaining = display_height - y - RESULT_BOTTOM_MARGIN * scale;
        (remaining >= MIN_RESULT_HEIGHT * scale).then_some((y, remaining))
    }

    /// A copied menu name survives selected-fighter menu teardown without leaking
    /// strings. Configuration/report updates allocate no label storage on frames.
    ///
    ///  A copied menu name survives selected-fighter menu teardown without leaking strings. Configuration/report updates allocate no label storage on frames.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Label {
        /// Stores the `bytes` value in `Label` as `[u8; LABEL_CAPACITY_BYTES]`.
        ///
        ///
        ///  Stores the bytes collection for Label.
        bytes: [u8; LABEL_CAPACITY_BYTES],
        /// Stores the `len` value in `Label` as `usize`.
        ///
        ///
        ///  Holds the len value used by Label.
        len: usize,
    }
    impl Label {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        pub fn new(text: &str) -> Self {
            let mut len = text.len().min(LABEL_CAPACITY_BYTES);
            while !text.is_char_boundary(len) {
                len -= 1;
            }
            let mut label = Self {
                bytes: [0; LABEL_CAPACITY_BYTES],
                len,
            };
            label.bytes[..len].copy_from_slice(&text.as_bytes()[..len]);
            label
        }
        /// Returns the string representation of the value.
        ///
        ///
        ///  Gets the value as text.
        pub fn as_str(&self) -> &str {
            std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
        }
        /// Checks whether empty.
        ///
        ///
        ///  Checks whether empty.
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
    impl std::fmt::Display for Label {
        /// Formats the value using the requested formatter.
        ///
        ///
        ///  Writes the value in its display form.
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    /// Defines the `Configuration` data structure.
    ///
    ///
    ///  Holds the data for Configuration.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Configuration {
        /// Stores the `fighter` value in `Configuration` as `&'static str`.
        ///
        ///
        ///  Holds the fighter value used by Configuration.
        pub fighter: &'static str,
        /// Stores the `rows` value in `Configuration` as `[Label; CONFIGURATION_ROW_COUNT]`.
        ///
        ///
        ///  Stores the rows collection for Configuration.
        pub rows: [Label; CONFIGURATION_ROW_COUNT],
        /// Stores the `auto_reflect` value in `Configuration` as `bool`.
        ///
        ///
        ///  Holds the auto reflect value used by Configuration.
        pub auto_reflect: bool,
    }
    impl Configuration {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        pub fn new(fighter: &'static str, rows: [&str; CONFIGURATION_ROW_COUNT]) -> Self {
            Self {
                fighter,
                rows: rows.map(Label::new),
                auto_reflect: false,
            }
        }
        /// Returns a copy configured with the reflect response.
        ///
        ///
        ///  Creates a copy that includes the reflect response.
        pub fn with_reflect(mut self, enabled: bool) -> Self {
            self.auto_reflect = enabled;
            self
        }
        /// Formats the value for presentation or persistence.
        ///
        ///
        ///  Turns the value into formatted text.
        pub fn format(&self) -> String {
            let mut text = format!("SAVED MODES for {} (changed configuration)\n", self.fighter);
            if self.fighter == "mr_game_and_watch" {
                let _=writeln!(text,"  Auto-reflect: {}. Native-ready projectile defense only; full bucket and ledge mode suspend it.",if self.auto_reflect {"ON"}else{"OFF"});
            }
            for ((label, help), value) in CONFIGURATION_ROWS.iter().zip(self.rows) {
                if !label.is_empty() {
                    let _ = writeln!(text, "  {label}: {value}\n    {help}");
                }
            }
            text.push_str("Priority: emergency safety > spot/grab/roll reads > ledge sequences > other punishes/tools > defensive helpers > turbo > crouch/tilt. Saved choices are available options; an eligible trigger and native state are still required.");
            text
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Sets the upper bound for recorder size bytes.
        ///
        ///
        ///  Holds the MAX RECORDER SIZE BYTES value.
        const MAX_RECORDER_SIZE_BYTES: usize = 1024;
        /// Sets the upper bound for report size bytes.
        ///
        ///
        ///  Holds the MAX REPORT SIZE BYTES value.
        const MAX_REPORT_SIZE_BYTES: usize = 768;

        /// Regression test for `copied_labels_survive_menu_teardown_and_truncate_on_utf8_boundaries`.
        ///
        ///
        ///  Checks that copied labels survive menu teardown and truncate on utf 8 boundaries.
        #[test]
        fn copied_labels_survive_menu_teardown_and_truncate_on_utf8_boundaries() {
            let name = "é".repeat(40);
            let label = Label::new(&name);
            drop(name);
            assert_eq!(label.as_str(), "é".repeat(32));
            assert_eq!(label.to_string(), label.as_str());
            assert!(Label::new("").is_empty());
            let menu_name = String::from("Saved custom option");
            let configuration =
                Configuration::new("Bowser", [menu_name.as_str(); CONFIGURATION_ROW_COUNT]);
            drop(menu_name);
            assert_eq!(configuration.rows[0].as_str(), "Saved custom option");
        }
        /// Captures the current values into a snapshot.
        ///
        ///
        ///  Saves a snapshot of the current values.
        fn snapshot() -> Snapshot {
            Snapshot {
                winner: Some(Owner::Ledge),
                present: bit(Owner::Ledge) | bit(Owner::Posture),
                committed: false,
                requested: 0,
                stick: None,
                status: crate::native_states::status::WAIT,
                motion: 1,
                native_frame: 0,
                stopped: false,
                can_act: true,
                hit: Some(false),
                valid: true,
                safety: 0,
            }
        }
        /// Regression test for `snapshots_explain_every_owner_and_preserve_intermediate_native_startup`.
        ///
        ///
        ///  Checks that snapshots explain every owner and preserve intermediate native startup.
        #[test]
        fn snapshots_explain_every_owner_and_preserve_intermediate_native_startup() {
            for owner in OWNERS {
                let mut r = Recorder::new();
                let s = Snapshot {
                    winner: Some(owner),
                    present: bit(owner),
                    ..snapshot()
                };
                let report = r.sample(s, 0).unwrap();
                assert!(report.format().contains(purpose(owner)));
                assert!(report.card_text().contains(name(owner)));
            }
            let mut r = Recorder::new();
            r.sample(snapshot(), 0);
            r.sample(
                Snapshot {
                    status: crate::native_states::status::ATTACK_AIR,
                    motion: crate::timing::motion_hash("attack_air_f"),
                    native_frame: 10,
                    ..snapshot()
                },
                10,
            );
            let report = r.sample(snapshot(), 250).unwrap();
            assert!(report.format().contains("aerial attack"));
            assert!(report.format().contains("not proof"));
            assert!(report.card_text().contains("Hit: not confirmed"));
            assert_eq!(report.at_ms, 250);
            assert_eq!(
                motion_name(
                    crate::native_states::status::ATTACK_AIR,
                    crate::timing::motion_hash("attack_air_lw"),
                ),
                "down air"
            );
        }
        /// Regression test for `animation_frame_only_changes_do_not_spam_stable_modes`.
        ///
        ///
        ///  Checks that animation frame only changes do not spam stable modes.
        #[test]
        fn animation_frame_only_changes_do_not_spam_stable_modes() {
            let mut r = Recorder::new();
            r.sample(snapshot(), 0);
            for frame in 1..1000 {
                assert!(r
                    .sample(
                        Snapshot {
                            native_frame: frame,
                            ..snapshot()
                        },
                        u64::from(frame)
                    )
                    .is_none());
            }
        }
        /// Regression test for `duplicate_polls_are_free_and_batches_have_a_hard_rate_limit`.
        ///
        ///
        ///  Checks that duplicate polls are free and batches have a hard rate limit.
        #[test]
        fn duplicate_polls_are_free_and_batches_have_a_hard_rate_limit() {
            let mut recorder = Recorder::new();
            assert!(recorder.sample(snapshot(), 0).is_some());
            for ms in 1..250 {
                assert!(recorder.sample(snapshot(), ms).is_none());
            }
            let mut count = 1;
            for ms in 250..=1250 {
                let s = Snapshot {
                    requested: if ms % 2 == 0 { MAPPER_A } else { 0 },
                    ..snapshot()
                };
                count += usize::from(recorder.sample(s, ms).is_some());
            }
            assert_eq!(count, 6);
            assert!(recorder.sample(snapshot(), 0).is_some());
        }
        /// Regression test for `buffered_press_survives_release_in_the_same_batch_without_claiming_a_hit`.
        ///
        ///
        ///  Checks that buffered press survives release in the same batch without claiming a hit.
        #[test]
        fn buffered_press_survives_release_in_the_same_batch_without_claiming_a_hit() {
            let mut r = Recorder::new();
            r.sample(snapshot(), 0);
            r.sample(
                Snapshot {
                    requested: MAPPER_X | MAPPER_A,
                    committed: true,
                    ..snapshot()
                },
                1,
            );
            r.sample(
                Snapshot {
                    status: crate::native_states::status::JUMP_SQUAT,
                    can_act: false,
                    ..snapshot()
                },
                16,
            );
            let report = r.sample(snapshot(), 250).unwrap().format();
            assert!(report.contains("Jump, Attack"));
            assert!(report.contains("never applied"));
            assert!(report.contains("do not prove"));
            assert!(report.contains("no hit flag"));
        }
        /// Regression test for `every_owner_and_configured_row_has_a_readable_label`.
        ///
        ///
        ///  Checks that every owner and configured row has a readable label.
        #[test]
        fn every_owner_and_configured_row_has_a_readable_label() {
            assert_eq!(
                OWNERS.iter().fold(0, |mask, owner| mask | bit(*owner)),
                (1u32 << OWNERS.len()) - 1
            );
            for owner in OWNERS {
                assert!(!name(owner).is_empty());
            }
            let text = Configuration::new("yoshi", ["OFF"; CONFIGURATION_ROW_COUNT]).format();
            assert!(text.contains("Neutral getup: OFF"));
        }
        /// Regression test for `configuration_rows_remain_aligned_after_removing_jab_grab`.
        ///
        ///
        ///  Checks that configuration rows remain aligned after removing jab grab.
        #[test]
        fn configuration_rows_remain_aligned_after_removing_jab_grab() {
            let rows = [
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
            ];
            let text = Configuration::new("yoshi", rows).format();
            for expected in [
                "Turnaround tilt: 9",
                "Spot-dodge punish: 11",
                "X button recipe: 16",
                "D-pad Right binding: 21",
                "Ledge trap mode: 22",
                "Out of shield: 23",
                "Shield pressure: 24",
            ] {
                assert!(text.contains(expected), "missing {expected}");
            }
        }
        /// Regression test for `left_result_stacks_below_menu_or_hides_instead_of_overlapping`.
        ///
        ///
        ///  Checks that left result stacks below menu or hides instead of overlapping.
        #[test]
        fn left_result_stacks_below_menu_or_hides_instead_of_overlapping() {
            for height in [480.0_f32, 720.0, 1080.0, 2160.0] {
                let scale = (height / 720.0).clamp(0.65, 3.0);
                let menu = 330.0 * scale;
                let (y, available) = left_result_space(height, scale, menu).unwrap();
                assert!(y > 8.0 * scale + menu);
                assert!(y + available <= height);
                assert!(left_result_space(height, scale, height).is_none());
            }
            assert!(left_result_space(f32::NAN, 1.0, 100.0).is_none());
        }
        /// Regression test for `disabled_modes_do_not_log_manual_play_and_evidence_storage_is_bounded`.
        ///
        ///
        ///  Checks that disabled modes do not log manual play and evidence storage is bounded.
        #[test]
        fn disabled_modes_do_not_log_manual_play_and_evidence_storage_is_bounded() {
            let mut r = Recorder::new();
            for ms in 0..1000 {
                let idle = Snapshot {
                    winner: None,
                    present: 0,
                    status: ms as i32,
                    ..snapshot()
                };
                assert!(r.sample(idle, ms).is_none());
            }
            assert!(std::mem::size_of::<Recorder>() <= MAX_RECORDER_SIZE_BYTES);
            assert!(std::mem::size_of::<Report>() <= MAX_REPORT_SIZE_BYTES);
        }
        /// Regression test for `unknown_hit_evidence_and_invalid_scope_are_explained_without_inventing_a_blocker`.
        ///
        ///
        ///  Checks that unknown hit evidence and invalid scope are explained without inventing a blocker.
        #[test]
        fn unknown_hit_evidence_and_invalid_scope_are_explained_without_inventing_a_blocker() {
            let mut r = Recorder::new();
            let report = r
                .sample(
                    Snapshot {
                        winner: None,
                        valid: false,
                        hit: None,
                        ..snapshot()
                    },
                    0,
                )
                .unwrap()
                .format();
            assert!(report.contains("scope was unavailable"));
            assert!(report.contains("hit evidence was not sampled"));
            assert!(!report.contains("another mode retained priority"));
        }
    }
}
