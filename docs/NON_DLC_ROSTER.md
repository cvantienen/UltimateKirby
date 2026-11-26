# Roster profiles and canonical data (including DLC)

The historical filename does **not** mean non-DLC-only support. The current
`fighter_profiles!` declaration registers **90** identities: all DLC fighters,
echoes, Pokémon forms, Miis and both Ice Climbers identities. Registration is
not a count of fighter-select portraits, nor proof of a populated recipe.
Native IDs 0–76 and 81–93 are registered; non-playable kinds 77–80 are omitted.
The native lookup upper bound is 94, so a kind in range can still lack a profile.
Other fighter-object/trainer/random constants are not automatic aliases.

## Three things not to confuse

| Layer | Meaning | Missing value |
| --- | --- | --- |
| Profile | Exact identity and menu/configuration namespace | No substituted profile or ledge response |
| Embedded fighter data | Exact-kind stats/move-frame facts | Unknown/ambiguous, never copied from another fighter |
| Startup sequence file | Selected labels and controller timelines | Empty/no selected tape, not a compiled move |

The core expects 90 embedded records, but profile validation checks exact
identity/data correspondence separately. Data lookups apply equally to player
and opponent roles. A DLC data row does not automatically supply a counter,
reflector or ledge script. A Nana profile is not partner-control automation.
Pyra/Mythra retain separate selections and settings across form identity changes.

## Configure and verify

Open [your fighter](FIGHTERS.md), follow [configuration](FIGHTER_CONFIGS.md),
then review [tuning](FIGHTER_SETTINGS.md) and [data lifecycle](FIGHTER_DATA.md).
Generated pages show repository assets, not installed/unsaved state. Unknown
compound frames or conflicting variants stay unknown; do not infer continuous
coverage across a multihit gap. Author input timing explicitly and calibrate.

For authorized reference-data/catalog changes, run the data validator and
fighter-documentation generator/check. These offline checks establish schema
and documentation consistency, not native hit reliability. Do not build, upload
or modify canonical assets as part of a documentation-only update.
