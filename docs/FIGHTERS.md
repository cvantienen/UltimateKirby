# Fighters

Every configured fighter has file-authored **Neutral, Jump, Roll, Attack and
Drop** options. Open the menu with **Minus** and use the D-pad to select them.
All labels, defaults, initial delays and ordered input timing come from
`sequences.txt`; there are no compiled fighter option tables or fallback recipes.

All non-DLC playable fighter forms have identities, including echoes, Pokémon
forms and Miis. Pyra and Mythra are also registered as separate native forms,
bringing the configured registry to 79 identities. Their separate `pyra.txt` and
`mythra.txt` files begin with five editable, nonattacking ledge slots; configure
their actions in Training. Automatic punish recipes stay inactive until authored,
and neither form inherits Game & Watch's reflector or Bowser's specials.
Missing identities/sections have no fallback option. The file
controls controller inputs only; it never changes native moves or physics.

The [compiled reference dataset](OPPONENT_DATA.md) retains baseline weight,
movement and move facts for 90 native kinds, including DLC opponents. These facts
do not add ledge recipes. Automatic whiff punishing is currently disconnected
from the live build/menu; each fighter page retains its reference stats.

<!-- BEGIN INDIVIDUAL FIGHTER LINKS -->

## Individual fighter pages

Each registered fighter form has its own source-based reference, including
all five file-authored lists, selected defaults, input schedules and reference facts.

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
| [Mr. Game & Watch](fighters/mr_game_and_watch.md) | [Mythra](fighters/mythra.md) | [Ness](fighters/ness.md) |
| [Olimar](fighters/olimar.md) | [Pac-Man](fighters/pacman.md) | [Palutena](fighters/palutena.md) |
| [Peach](fighters/peach.md) | [Pichu](fighters/pichu.md) | [Pikachu](fighters/pikachu.md) |
| [Pit](fighters/pit.md) | [Pyra](fighters/pyra.md) | [R.O.B.](fighters/rob.md) |
| [Richter](fighters/richter.md) | [Ridley](fighters/ridley.md) | [Robin](fighters/robin.md) |
| [Rosalina & Luma](fighters/rosalina_and_luma.md) | [Roy](fighters/roy.md) | [Ryu](fighters/ryu.md) |
| [Samus](fighters/samus.md) | [Sheik](fighters/sheik.md) | [Shulk](fighters/shulk.md) |
| [Simon](fighters/simon.md) | [Snake](fighters/snake.md) | [Sonic](fighters/sonic.md) |
| [Squirtle](fighters/squirtle.md) | [Toon Link](fighters/toon_link.md) | [Villager](fighters/villager.md) |
| [Wario](fighters/wario.md) | [Wii Fit Trainer](fighters/wii_fit_trainer.md) | [Wolf](fighters/wolf.md) |
| [Yoshi](fighters/yoshi.md) | [Young Link](fighters/young_link.md) | [Zelda](fighters/zelda.md) |
| [Zero Suit Samus](fighters/zero_suit_samus.md) |  |  |

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

Use [file-authored fighter options](FIGHTER_CONFIGS.md) and the
[complete action/timing syntax](SEQUENCE_FILES.md) to add or edit responses.
`start_delay` is required for every option; waits schedule each following input.
Only shared recognition, observation, input routing and spatial predicates stay
in code. New input primitives need Rust changes; new scripts need only a file edit
and restart after installing the updated executor.

Frame intervals document compiled data, not universal collision or optimality.
Reach and root-motion forecasts are approximate. Projectile travel, opponent
steering, native readiness, stage geometry and online delay can defeat a punish.
Check [diagnostic grab traces](BUILD_VARIANTS.md#diagnostic-performance) when
testing a miss, and distinguish the source version from the installed NRO.
