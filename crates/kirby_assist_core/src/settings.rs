//! Settings shared by ledge and defense-response policies.
use crate::fighter_profiles::{FighterGrabData, LedgeResponses};
use crate::native_input::MAPPER_R;

/// Validated assistance settings. Defaults and response timing are shared by both builds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AssistSettings {
    /// Legacy identity tag; no executable fighter handler is dispatched.
    pub implementation: Option<crate::ledge_trap::ResponseImplementation>,
    /// Native fighter identity for startup-loaded file menus.
    pub fighter_kind: i32,
    /// Indices into the fighter's startup-resolved menu lists. None preserves direct
    /// core/legacy callers that supply settings without using the battle menu.
    pub response_choices: [Option<usize>; crate::fighter_profiles::presets::PresetSlot::COUNT],
    /// Startup-parsed recipes for the five selected compiled or file-added IDs.
    /// Fixed-size copies keep text/file work out of live callbacks.
    pub sequence_overrides: [Option<crate::sequence_config::Sequence>;
        crate::fighter_profiles::presets::PresetSlot::COUNT],
    /// Numeric-only startup adjustments; these cannot register new actions.
    /// Resolved move timings and response switches for our selected fighter.
    pub fighter: FighterGrabData,
    pub responses: LedgeResponses,
    pub enabled: bool,
    /// Enable the selected response to a witnessed generic ledge release.
    pub drop_defense: bool,
    /// Native-time budget to request/confirm the drop punish before shielding.
    // Ranges/heights/stance distances are WORLD units, not stick percentages.
    // Increasing reach only makes the policy willing to attack from farther
    // away; it does not enlarge Kirby's real hitbox and can create more whiffs.
    /// Broad target acquisition limits in world units.
    pub horizontal_range: f32,
    pub vertical_range: f32,
    // Follow limits keep the acquired opponent through its get-up travel;
    // acquisition limits above decide whether to acquire it in the first place.
    pub follow_range: f32,
    pub follow_vertical_range: f32,
    /// Preserve player movement between response inputs.
    pub manual_spacing: bool,
    /// Player owns spacing; normal/jump/grab timing does not test reach.
    pub distance_agnostic: bool,
    /// Baseline horizontal walk strength: normal cap is -0.02, near-full
    /// horizontal cap is +0.03, both bounded to 0.1..0.45.
    pub walk_strength: f32,
    pub auto_position: bool,
    /// Heuristic stage-side stance offset, not a universal optimal location.
    pub stand_distance: f32,
    // Unlike distances, move_strength is normalized stick magnitude (0..1).
    pub move_strength: f32,
    /// Legacy parsed setting. Captures always return control to the player.
    pub throw_outward: bool,
    /// Native mask of the button mapped to Grab; R for this user's controls.
    pub grab_button: u32,
}

impl Default for AssistSettings {
    fn default() -> Self {
        Self {
            implementation: Some(crate::ledge_trap::ResponseImplementation::Kirby),
            fighter_kind: crate::fighter_kinds::KIRBY,
            response_choices: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
            sequence_overrides: [None; crate::fighter_profiles::presets::PresetSlot::COUNT],
            fighter: FighterGrabData::from_kind(6),
            responses: LedgeResponses::default(),
            enabled: false,
            drop_defense: true,
            horizontal_range: 12.0,
            vertical_range: 10.0,
            follow_range: 40.0,
            follow_vertical_range: 48.0,
            manual_spacing: true,
            distance_agnostic: true,
            walk_strength: 0.35,
            auto_position: true,
            stand_distance: 8.0,
            move_strength: 1.0,
            throw_outward: false,
            grab_button: MAPPER_R,
        }
    }
}
