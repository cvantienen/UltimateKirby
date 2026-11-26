# Fighters: identity, configuration and support

The source registers **90 exact fighter-profile identities**, including all DLC
fighters, echoes, the three Pokémon forms, all three Miis, and Popo/Nana. This
is not a count of fighter-select portraits: a form/partner can have its own
native kind. The contiguous kind limit is 94 (Sora + 1); four non-playable IDs
inside that range are not registered profiles. Unknown kinds and missing
sections never borrow Kirby or another fighter's actions.

A **profile** tells the runtime which fighter it is and supplies independent
grab facts. A **recipe** tells it which controller inputs to request. The 90
profiles begin with empty menu lists in Rust; `sequences.txt` supplies options
at startup. Registration alone never enables an attack or creates a default
script. Five ledge rows are Neutral, Jump, Roll, Attack and Drop; eight profile
button slots are X, Y, D-pad directions, L3 and R3. Current control binding
support also includes configured hold/toggle tools; see [controls](CONTROLS.md).

Standalone auto-counter currently supports Lucina; standalone projectile defense
supports Game & Watch; Kirby-specific trip/fair follow-ups stay Kirby-only.
Universal held tools use separate gates and do not imply those adapters exist
for every fighter. Tilt-facing schema rows are shown for Kirby/Yoshi/Lucina.
[All settings and prerequisites](FIGHTER_SETTINGS.md) separate capability from
saved preference and current activation.

## Beginner path

1. Find your exact fighter below. Read **Configure**, supplied overrides and
   support limits before selecting a label.
2. Follow [configuration](FIGHTER_CONFIGS.md) to edit an existing option or make
   a new one. Use [sequence syntax](SEQUENCE_FILES.md), not a Rust move table.
3. Use [tuning](FIGHTER_SETTINGS.md) for prediction/spacing settings. A numeric
   reach is an estimate, not a hitbox edit; an at-startup flag is not the current
   mode switch.
4. Save in Training and wait for the result. External changes require restart;
   an installed file may differ from the repository snapshot.
5. [Calibrate execution](FIGHTER_EXECUTION.md) at repeatable spacing. No snapshot
   demonstrates on-device or online hit reliability.

<!-- BEGIN INDIVIDUAL FIGHTER LINKS -->

## Individual fighter pages

Each registered fighter form has its own source-based reference, including
all five file-authored lists, supplied defaults, input schedules and canonical fighter facts. These tables are repository snapshots, not live menu state.

| Fighter | Fighter | Fighter |
| --- | --- | --- |
| [Banjo And Kazooie](fighters/banjo_and_kazooie.md) | [Bayonetta](fighters/bayonetta.md) | [Bowser](fighters/bowser.md) |
| [Bowser Jr.](fighters/bowser_jr.md) | [Byleth](fighters/byleth.md) | [Captain Falcon](fighters/captain.md) |
| [Charizard](fighters/charizard.md) | [Chrom](fighters/chrom.md) | [Cloud](fighters/cloud.md) |
| [Corrin](fighters/corrin.md) | [Daisy](fighters/daisy.md) | [Dark Pit](fighters/dark_pit.md) |
| [Dark Samus](fighters/dark_samus.md) | [Diddy Kong](fighters/diddy_kong.md) | [Donkey Kong](fighters/donkey.md) |
| [Dr. Mario](fighters/dr_mario.md) | [Duck Hunt](fighters/duck_hunt.md) | [Falco](fighters/falco.md) |
| [Fox](fighters/fox.md) | [Ganondorf](fighters/ganondorf.md) | [Greninja](fighters/greninja.md) |
| [Hero](fighters/hero.md) | [Ice Climbers (Nana)](fighters/ice_climbers_nana.md) | [Ice Climbers (Popo)](fighters/ice_climbers.md) |
| [Ike](fighters/ike.md) | [Incineroar](fighters/incineroar.md) | [Inkling](fighters/inkling.md) |
| [Isabelle](fighters/isabelle.md) | [Ivysaur](fighters/ivysaur.md) | [Jigglypuff](fighters/jigglypuff.md) |
| [Joker](fighters/joker.md) | [Kazuya](fighters/kazuya.md) | [Ken](fighters/ken.md) |
| [King Dedede](fighters/king_dedede.md) | [King K. Rool](fighters/krool.md) | [Kirby](fighters/kirby.md) |
| [Link](fighters/link.md) | [Little Mac](fighters/littlemac.md) | [Lucario](fighters/lucario.md) |
| [Lucas](fighters/lucas.md) | [Lucina](fighters/lucina.md) | [Luigi](fighters/luigi.md) |
| [Mario](fighters/mario.md) | [Marth](fighters/marth.md) | [Mega Man](fighters/mega_man.md) |
| [Meta Knight](fighters/meta_knight.md) | [Mewtwo](fighters/mewtwo.md) | [Mii Brawler](fighters/mii_brawler.md) |
| [Mii Gunner](fighters/mii_gunner.md) | [Mii Swordfighter](fighters/mii_swordfighter.md) | [Min Min](fighters/min_min.md) |
| [Mr. Game & Watch](fighters/mr_game_and_watch.md) | [Mythra](fighters/mythra.md) | [Ness](fighters/ness.md) |
| [Olimar](fighters/olimar.md) | [Pac-Man](fighters/pacman.md) | [Palutena](fighters/palutena.md) |
| [Peach](fighters/peach.md) | [Pichu](fighters/pichu.md) | [Pikachu](fighters/pikachu.md) |
| [Piranha Plant](fighters/piranha_plant.md) | [Pit](fighters/pit.md) | [Pyra](fighters/pyra.md) |
| [R.O.B.](fighters/rob.md) | [Richter](fighters/richter.md) | [Ridley](fighters/ridley.md) |
| [Robin](fighters/robin.md) | [Rosalina & Luma](fighters/rosalina_and_luma.md) | [Roy](fighters/roy.md) |
| [Ryu](fighters/ryu.md) | [Samus](fighters/samus.md) | [Sephiroth](fighters/sephiroth.md) |
| [Sheik](fighters/sheik.md) | [Shulk](fighters/shulk.md) | [Simon](fighters/simon.md) |
| [Snake](fighters/snake.md) | [Sonic](fighters/sonic.md) | [Sora](fighters/sora.md) |
| [Squirtle](fighters/squirtle.md) | [Steve](fighters/steve.md) | [Terry](fighters/terry.md) |
| [Toon Link](fighters/toon_link.md) | [Villager](fighters/villager.md) | [Wario](fighters/wario.md) |
| [Wii Fit Trainer](fighters/wii_fit_trainer.md) | [Wolf](fighters/wolf.md) | [Yoshi](fighters/yoshi.md) |
| [Young Link](fighters/young_link.md) | [Zelda](fighters/zelda.md) | [Zero Suit Samus](fighters/zero_suit_samus.md) |

<!-- END INDIVIDUAL FIGHTER LINKS -->

## Group-specific walkthroughs

- [Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md): separate follow-ups, tilts and explicit drift.
- [Starting cast](STARTING_CAST.md): exact profile keys rather than copied timing defaults.
- [Luigi/Falcon](FIGHTER_ADDITIONS.md): registration and distinct grab facts.
- [Samus/Dark Samus](SAMUS_LEDGE.md): tether airborne guards and projectile travel.
- [K. Rool/Isabelle](KROOL_ISABELLE_LEDGE.md): disconnected phases and slingshot direction.
- [Sonic](SONIC_LEDGE.md): charge/recovery assumptions.
- [Pac-Man/Mac](PACMAN_MAC_LEDGE.md): resource and grounded-versus-aerial choices.
- [Roster/data distinction](NON_DLC_ROSTER.md) and [data lifecycle](FIGHTER_DATA.md).

Generated tables are current **repository asset snapshots**. Bold means the file's
saved default; labels may differ from their actions, so inspect the input column.
Move-frame data is an independent reference, not a live input schedule. New
options need a file edit; new input primitives or native adapters require reviewed
Rust work and a separately authorized build/install. No Switch build, upload or
native move changes are part of this documentation update.
