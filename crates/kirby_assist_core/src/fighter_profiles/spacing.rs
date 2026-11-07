//! Display-only, qualitative spacing estimates for the compiled response list.
//!
//! These describe the move's useful coverage, not the response's detection
//! radius or a guaranteed collision distance. Animation, opponent hurtboxes,
//! height and aerial drift prevent turning a hitbox radius into a universal
//! maximum. Keep numerical distances out until measured in world coordinates.
//! Nothing here gates an attack or reads live fighter/opponent state.
use super::presets::PresetSlot;
use crate::fighter_kinds as kind;

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
            kind::LITTLE_MAC => "~touching", // Little Mac's intentionally tiny aerial.
            kind::DONKEY_KONG
            | kind::LINK
            | kind::SAMUS
            | kind::DARK_SAMUS
            | kind::BOWSER
            | kind::MARTH
            | kind::LUCINA
            | kind::GANONDORF
            | kind::ROY
            | kind::CHROM
            | kind::IKE
            | kind::CHARIZARD
            | kind::SONIC
            | kind::KING_DEDEDE
            | kind::ROB
            | kind::ROSALINA_AND_LUMA
            | kind::SHULK
            | kind::CLOUD
            | kind::CORRIN
            | kind::RIDLEY
            | kind::KING_K_ROOL => "~mid",
            _ => "~close",
        },
        "down_tilt" | "down_tilt_shield" => match fighter_kind {
            kind::DONKEY_KONG
            | kind::LINK
            | kind::SAMUS
            | kind::DARK_SAMUS
            | kind::YOSHI
            | kind::KIRBY
            | kind::CAPTAIN_FALCON
            | kind::MARTH
            | kind::LUCINA
            | kind::GANONDORF
            | kind::MEWTWO
            | kind::ROY
            | kind::CHROM
            | kind::IKE
            | kind::IVYSAUR
            | kind::CHARIZARD
            | kind::KING_DEDEDE
            | kind::TOON_LINK
            | kind::GRENINJA
            | kind::PALUTENA
            | kind::ROBIN
            | kind::SHULK
            | kind::CLOUD
            | kind::CORRIN
            | kind::RIDLEY
            | kind::KING_K_ROOL
            | kind::ISABELLE
            | kind::MII_SWORDFIGHTER => "~mid / low",
            _ => "~close / low",
        },
        "forward_tilt" | "poke" => match fighter_kind {
            kind::DONKEY_KONG
            | kind::LINK
            | kind::SAMUS
            | kind::DARK_SAMUS
            | kind::MARTH
            | kind::LUCINA
            | kind::GANONDORF
            | kind::MEWTWO
            | kind::IKE
            | kind::CHARIZARD
            | kind::SHULK
            | kind::CLOUD
            | kind::CORRIN
            | kind::RIDLEY
            | kind::SIMON
            | kind::RICHTER
            | kind::KING_K_ROOL
            | kind::MII_SWORDFIGHTER => "~long",
            _ => "~mid",
        },
        "stone" => "~close",
        "inhale" => "~close / front",
        "zair" | "shield_zair" => "~long / front",
        "bomb" => "~near ledge",
        "up_tilt" => "~close / above",
        "up_air" | "shield_up_air" => {
            if fighter_kind == kind::LITTLE_MAC {
                "~touching / above"
            } else {
                "~above"
            }
        }
        "forward_air" if fighter_kind == kind::ISABELLE => "~ranged / front",
        "forward_air" | "shield_fair" => "~mid / front",
        "bair" if fighter_kind == kind::YOSHI => "~close / rear",
        "slingshot" => "~ranged / front",
        "short_hop_dair"
            if matches!(
                fighter_kind,
                kind::DARK_SAMUS
                    | kind::NESS
                    | kind::JIGGLYPUFF
                    | kind::PEACH
                    | kind::DAISY
                    | kind::MARTH
                    | kind::LUCINA
                    | kind::YOUNG_LINK
                    | kind::GANONDORF
                    | kind::MEWTWO
                    | kind::ROY
                    | kind::CHROM
                    | kind::MR_GAME_AND_WATCH
                    | kind::META_KNIGHT
                    | kind::PIT
                    | kind::DARK_PIT
                    | kind::ZERO_SUIT_SAMUS
                    | kind::WARIO
                    | kind::SNAKE
                    | kind::IKE
                    | kind::SQUIRTLE
                    | kind::IVYSAUR
                    | kind::CHARIZARD
                    | kind::DIDDY_KONG
                    | kind::LUCAS
                    | kind::SONIC
                    | kind::KING_DEDEDE
                    | kind::OLIMAR
                    | kind::LUCARIO
                    | kind::ROB
                    | kind::TOON_LINK
                    | kind::WOLF
                    | kind::VILLAGER
                    | kind::MEGA_MAN
                    | kind::WII_FIT_TRAINER
                    | kind::ROSALINA_AND_LUMA
                    | kind::LITTLE_MAC
                    | kind::GRENINJA
                    | kind::PALUTENA
                    | kind::PAC_MAN
                    | kind::ROBIN
                    | kind::SHULK
                    | kind::BOWSER_JR
                    | kind::DUCK_HUNT
                    | kind::RYU
                    | kind::KEN
                    | kind::CLOUD
                    | kind::CORRIN
                    | kind::BAYONETTA
                    | kind::INKLING
                    | kind::RIDLEY
                    | kind::SIMON
                    | kind::RICHTER
                    | kind::KING_K_ROOL
                    | kind::ISABELLE
                    | kind::INCINEROAR
                    | kind::SQUIRTLE_FIGHTER
                    | kind::IVYSAUR_FIGHTER
                    | kind::CHARIZARD_FIGHTER
                    | kind::POKEMON_TRAINER
                    | kind::PYRA_FIGHTER
                    | kind::MYTHRA_FIGHTER
                    | kind::ICE_CLIMBERS_POPO
                    | kind::ICE_CLIMBERS_NANA
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
