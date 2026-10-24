//! Display-only, qualitative spacing estimates for the compiled response list.
//!
//! These describe the move's useful coverage, not the response's detection
//! radius or a guaranteed collision distance. Animation, opponent hurtboxes,
//! height and aerial drift prevent turning a hitbox radius into a universal
//! maximum. Keep numerical distances out until measured in world coordinates.
//! Nothing here gates an attack or reads live fighter/opponent state.
use super::presets::PresetSlot;

/// Compact display aliases only. IDs, handlers and timing stay unchanged.
/// The UI resolves these when its cached snapshot changes, never per game frame.
pub fn menu_name(name: &str) -> &str {
    match name {
        "Down tilt" => "Dtilt",
        "Forward tilt" => "Ftilt",
        "Forward tilt 1" => "Ftilt 1",
        "Up tilt" => "Utilt",
        "Grounded up tilt" => "Ground Utilt",
        "Forward air" => "Fair",
        "Forward air 1" => "Fair 1",
        "Up air" => "Uair",
        "Down smash" => "Dsmash",
        "Forward smash" => "Fsmash",
        "Down tilt > shield" => "Dtilt > Shield",
        "Shield > nair" => "Shield Nair",
        "Shield > fair" => "Shield Fair",
        "Shield > fair 1" => "Shield Fair 1",
        "Shield > up air" => "Shield Uair",
        "Shield > zair" => "Shield Zair",
        "Shield > Up-B" => "Shield Up-B",
        "Shield > slingshot" => "Shield Sling",
        "Shield > trampoline" => "Shield Tramp",
        "Shield > fair (Pikmin)" => "Shield Pikmin Fair",
        "Shield > up air (Pikmin)" => "Shield Pikmin Uair",
        "Grab / rear nair" => "Grab/rear Nair",
        "Grab / rear shield" => "Grab/rear Guard",
        "Short-hop zair" => "SH Zair",
        "Short-hop dair" => "SH Dair",
        "Short-hop bite" => "SH Bite",
        "Hop nair (near)" => "Near Nair",
        "Hop fair (near)" => "Near Fair",
        "Hop fair 1 (near)" => "Near Fair 1",
        "Hop dair (near)" => "Near Dair",
        "Hop fair (Pikmin)" => "Pikmin Fair",
        "Hop dair (Pikmin)" => "Pikmin Dair",
        "Neutral-B / Inhale" => "Inhale",
        "Down-B / Stone" => "Stone",
        "Immediate down-B" => "Bomb (now)",
        "Homing missile" => "Homing Msl",
        "Super missile" => "Super Msl",
        "Homing (stay back)" => "Homing (stay)",
        "Super (stay back)" => "Super (stay)",
        "Flame fair (stay back)" => "Flame Fair (stay)",
        "Headbutt f-smash" => "Head Fsmash",
        "Belly flop (risky)" => "Flop (risky)",
        "Dair shred (risky)" => "Dair (risky)",
        "Meteor chase (risky)" => "Meteor (risky)",
        "Turnip spike (risky)" => "Turnip (risky)",
        "Tiny up-air (joke)" => "Tiny Uair (joke)",
        _ => name,
    }
}

/// An empty hint deliberately leaves grabs, shield and automatic roll approaches
/// alone: their starting distance is not the move's final striking distance.
/// Fighter kind is the resolved profile kind, so fallback profiles stay coherent.
pub fn hint(fighter_kind: i32, slot: PresetSlot, choice: &str) -> &'static str {
    // Missile roll choices deliberately stay back instead of using the roll
    // approach, so retain their ranged hint even in that row.
    if matches!(
        choice,
        "homing_missile" | "super_missile" | "area_smash" | "projectile_fair"
    ) {
        return "~ranged / travel";
    }
    if slot == PresetSlot::Roll
        || matches!(choice, "shield" | "grab" | "shield_grab" | "shield_rear")
    {
        return "";
    }
    match choice {
        "nair" | "shield_nair" | "drop_nair" => match fighter_kind {
            0x34 => "~touching", // Little Mac's intentionally tiny aerial.
            1 | 2 | 3 | 4 | 0xf | 0x15 | 0x16 | 0x18 | 0x1a | 0x1b | 0x23 | 0x26 | 0x29 | 0x2a
            | 0x2d | 0x33 | 0x39 | 0x3e | 0x3f | 0x42 | 0x45 => "~mid",
            _ => "~close",
        },
        "down_tilt" | "down_tilt_shield" => match fighter_kind {
            1 | 2 | 3 | 4 | 5 | 6 | 11 | 0x15 | 0x16 | 0x18 | 0x19 | 0x1a | 0x1b | 0x23 | 0x25
            | 0x26 | 0x2a | 0x2e | 0x35 | 0x36 | 0x38 | 0x39 | 0x3e | 0x3f | 0x42 | 0x45 | 0x46
            | 0x49 => "~mid / low",
            _ => "~close / low",
        },
        "forward_tilt" | "poke" => match fighter_kind {
            1 | 2 | 3 | 4 | 0x15 | 0x16 | 0x18 | 0x19 | 0x23 | 0x26 | 0x39 | 0x3e | 0x3f | 0x42
            | 0x43 | 0x44 | 0x45 | 0x49 => "~long",
            _ => "~mid",
        },
        "stone" => "~close",
        "inhale" => "~close / front",
        "zair" | "shield_zair" => "~long / front",
        "bomb" => "~near ledge",
        "up_tilt" => "~close / above",
        "up_air" | "shield_up_air" => {
            if fighter_kind == 0x34 {
                "~touching / above"
            } else {
                "~above"
            }
        }
        "forward_air" if fighter_kind == 0x46 => "~ranged / front",
        "forward_air" | "shield_fair" => "~mid / front",
        "bair" if fighter_kind == 5 => "~close / rear",
        "slingshot" => "~ranged / front",
        "short_hop_dair"
            if matches!(
                fighter_kind,
                4 | 10
                    | 12
                    | 13
                    | 14
                    | 15
                    | 16
                    | 17
                    | 18
                    | 19
                    | 20
                    | 21
                    | 22
                    | 23
                    | 24
                    | 25
                    | 26
                    | 27
                    | 28
                    | 29
                    | 30
                    | 31
                    | 32
                    | 33
                    | 34
                    | 35
                    | 36
                    | 37
                    | 38
                    | 39
                    | 40
                    | 42
                    | 43
                    | 44
                    | 45
                    | 46
                    | 47
                    | 48
                    | 49
                    | 50
                    | 51
                    | 53
                    | 54
                    | 56
                    | 57
                    | 58
                    | 59
                    | 60
                    | 61
                    | 62
                    | 63
                    | 64
                    | 65
                    | 66
                    | 67
                    | 68
                    | 71
                    | 72
                    | 73
                    | 74
                    | 75
                    | 76
            ) =>
        {
            "~below / near edge"
        }
        "short_hop_dair" => "~below / offstage",
        "drop_fair" => "~front / offstage",
        "belly_flop" if slot == PresetSlot::Drop => "~below / offstage",
        "belly_flop" => "~mid",
        "party_popper" => "~close / front",
        "ghost" | "ghosts" => "~long",
        "down_smash" => "~mid / low",
        "forward_smash" | "low_forward_smash" => "~mid / front",
        "sweep" => "~mid / low",
        "shield_up_b" => "~close / above",
        // A new action without reviewed metadata should never inherit another
        // move's estimate just because it occupies the same menu row.
        _ => "~spacing?",
    }
}
