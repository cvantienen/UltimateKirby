# Fighters

Every configured fighter has its own page, linked below. Individual pages stay
off the sidebar; shared guides provide additional execution notes.
Every configured fighter has compiled response functions for **Neutral, Jump,
Roll, Attack and Drop**. Open the menu with **R3** and use the D-pad to select
and cycle its saved choices. See [current controls](CONTROLS.md).

All non-DLC playable fighter forms have profiles, including echoes, Pokémon
forms and Miis. Valid unconfigured fighter kinds use the Kirby fallback; that
does not make Kirby's move timings appropriate for those fighters. There is no
runtime fighter JSON, combo library or action interpreter.

The [compiled reference dataset](OPPONENT_DATA.md) retains baseline weight,
movement and move facts for 90 native kinds, including DLC opponents. These facts
do not add ledge recipes. Automatic whiff punishing is currently disconnected
from the live build/menu; each fighter page retains its reference stats.

<!-- BEGIN INDIVIDUAL FIGHTER LINKS -->

## Individual fighter pages

Each registered fighter form has its own source-based reference, including
all five saved lists, default choices, move windows and grab data.

| Fighter | Fighter | Fighter |
| --- | --- | --- |
| [Bayonetta](fighters/bayonetta.md) | [Bowser](fighters/bowser.md) | [Bowser Jr.](fighters/bowser_jr.md) |
| [Captain Falcon](fighters/captain.md) | [Charizard](fighters/charizard.md) | [Chrom](fighters/chrom.md) |
| [Cloud](fighters/cloud.md) | [Corrin](fighters/corrin.md) | [Daisy](fighters/daisy.md) |
| [Dark Pit](fighters/dark_pit.md) | [Dark Samus](fighters/dark_samus.md) | [Diddy Kong](fighters/diddy_kong.md) |
| [Donkey Kong](fighters/donkey.md) | [Dr. Mario](fighters/dr_mario.md) | [Duck Hunt](fighters/duck_hunt.md) |
| [Falco](fighters/falco.md) | [Fox](fighters/fox.md) | [Ganondorf](fighters/ganondorf.md) |
| [Greninja](fighters/greninja.md) | [Ice Climbers (Nana)](fighters/ice_climbers_nana.md) | [Ice Climbers (Popo)](fighters/ice_climbers.md) |
| [Ike](fighters/ike.md) | [Incineroar](fighters/incineroar.md) | [Inkling](fighters/inkling.md) |
| [Isabelle](fighters/isabelle.md) | [Ivysaur](fighters/ivysaur.md) | [Jigglypuff](fighters/jigglypuff.md) |
| [Ken](fighters/ken.md) | [King Dedede](fighters/king_dedede.md) | [King K. Rool](fighters/krool.md) |
| [Kirby](fighters/kirby.md) | [Link](fighters/link.md) | [Little Mac](fighters/littlemac.md) |
| [Lucario](fighters/lucario.md) | [Lucas](fighters/lucas.md) | [Lucina](fighters/lucina.md) |
| [Luigi](fighters/luigi.md) | [Mario](fighters/mario.md) | [Marth](fighters/marth.md) |
| [Mega Man](fighters/mega_man.md) | [Meta Knight](fighters/meta_knight.md) | [Mewtwo](fighters/mewtwo.md) |
| [Mii Brawler](fighters/mii_brawler.md) | [Mii Gunner](fighters/mii_gunner.md) | [Mii Swordfighter](fighters/mii_swordfighter.md) |
| [Mr. Game & Watch](fighters/mr_game_and_watch.md) | [Ness](fighters/ness.md) | [Olimar](fighters/olimar.md) |
| [Pac-Man](fighters/pacman.md) | [Palutena](fighters/palutena.md) | [Peach](fighters/peach.md) |
| [Pichu](fighters/pichu.md) | [Pikachu](fighters/pikachu.md) | [Pit](fighters/pit.md) |
| [R.O.B.](fighters/rob.md) | [Richter](fighters/richter.md) | [Ridley](fighters/ridley.md) |
| [Robin](fighters/robin.md) | [Rosalina & Luma](fighters/rosalina_and_luma.md) | [Roy](fighters/roy.md) |
| [Ryu](fighters/ryu.md) | [Samus](fighters/samus.md) | [Sheik](fighters/sheik.md) |
| [Shulk](fighters/shulk.md) | [Simon](fighters/simon.md) | [Snake](fighters/snake.md) |
| [Sonic](fighters/sonic.md) | [Squirtle](fighters/squirtle.md) | [Toon Link](fighters/toon_link.md) |
| [Villager](fighters/villager.md) | [Wario](fighters/wario.md) | [Wii Fit Trainer](fighters/wii_fit_trainer.md) |
| [Wolf](fighters/wolf.md) | [Yoshi](fighters/yoshi.md) | [Young Link](fighters/young_link.md) |
| [Zelda](fighters/zelda.md) | [Zero Suit Samus](fighters/zero_suit_samus.md) |  |

<!-- END INDIVIDUAL FIGHTER LINKS -->

## Shared and detailed guides

| Fighter or group | Guide |
| --- | --- |
| Kirby and Yoshi | [Current choices, nair nudges, bair and roll-grab changes](KIRBY_YOSHI_LEDGE.md) |
| Mario, DK, Link, Samus, Yoshi, Kirby, Fox and Pikachu | [Starting-roster options and frame windows](STARTING_CAST.md) |
| Luigi and Captain Falcon | [Ledge profiles](FIGHTER_ADDITIONS.md) |
| Samus and Dark Samus | [Zair, missiles, aerials, roll grab and drop Bomb](SAMUS_LEDGE.md) |
| Sonic | [Ledge options](SONIC_LEDGE.md) |
| King K. Rool and Isabelle | [Ledge options and Isabelle's five-row slingshot choices](KROOL_ISABELLE_LEDGE.md) |
| Pac-Man and Little Mac | [Ledge options](PACMAN_MAC_LEDGE.md) |
| Every configured fighter | [Full non-DLC roster, defaults and data audit](NON_DLC_ROSTER.md) |

Kirby also has a [forward-air first-hit landing follow-up](KIRBY_FAIR_CONFIRM.md).

## Configure or add responses

Use [compiled fighter profiles](FIGHTER_CONFIGS.md) for the file layout and
`SavedChoice` registration, and [execution and calibration](FIGHTER_EXECUTION.md)
for hop choice, drift, movement and timing. The [source map](CODE_MAP.md) follows
one response from observation through the controller mapper.

New fighters and action sequences require Rust changes and a rebuild. Shared
`config.txt` settings load once at startup; in-match choices only select existing
functions and cached numeric settings. Only the selected response runs.

Frame intervals document compiled data, not universal collision or optimality.
Reach and root-motion forecasts are approximate. Projectile travel, opponent
steering, native readiness, stage geometry and online delay can defeat a punish.
Check [diagnostic grab traces](BUILD_VARIANTS.md#diagnostic-performance) when
testing a miss, and distinguish the source version from the installed NRO.
