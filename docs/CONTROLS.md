# Current controls

These bindings describe the current source with the supplied configuration.
Use the selected local controller (`player_index = 0` by default).

| Input | Action |
| --- | --- |
| Hold L | Ledge autopilot using the five saved response choices |
| Hold L and click R3 | Show or hide the menu |
| D-pad Up/Down, menu visible | Select Neutral, Jump, Roll, Attack or Drop |
| D-pad Left/Right, menu visible | Cycle the selected row's saved responses |
| D-pad Left/Right, menu hidden | Start/repeat a downward air-dodge slide; L is not required |
| D-pad Down, menu hidden | Attempt a ledge trump |
| D-pad Up, menu hidden | Normal mapped input; no mode cycling |
| Minus | Toggle automatic mash; its ON/OFF value appears in the menu |
| Hold R3 without L | Turbo teabag with slight forward creep |
| Press X | Paired X+ZL short-hop attempt on the ground |
| Press Y | One neutral-B reverse attempt opposite current fighter facing |
| Hold L3 as Kirby | Down-tilt recovery cover; release to stop |

L3 has no corresponding recovery-cover assist on other fighters. Holding L3 as
Kirby selects recovery cover even if L is also held. Y does not select a mode,
run a combo or enable two-frame cover. Combo playback is removed.

## Controller mapping

Map **A to Attack**, **B to Special**, **R to Grab**, **ZR to Shield**, and
**both X and ZL to Jump**. Kirby's forward-facing attack-cancel bair also needs
**C-stick set to Tilt Attack**. Keep Tap Jump off for directional input recipes.
The plugin changes raw inputs before your mapping, so a different mapping can
change or prevent the intended action.

## Menu behavior

The compact, title-free menu starts visible in the bottom-left corner of battle.
It survives stock loss until hidden and closes outside battle. It shows five
saved response lists, the selected row and option counts, plus mash ON/OFF.
It uses compact move names with the existing spacing hints, larger/heavier text
and a narrower panel. `SH` means short hop, `Msl` missile, and `Dtilt/Ftilt/Utilt`
mean down/forward/up tilt; `Fair/Uair` mean forward/up air.
There are no Off choices or live combo-step indicators. Choices come from the
current fighter's compiled profile; selecting one changes cached settings.

Opening/closing the menu or changing a saved choice cancels pending manual-tool
inputs. Release directions held while closing before using a hidden-menu tool.
L+R3 is reserved for the menu chord and does not teabag. D-pad directions,
shortcuts and higher-priority mash/tech can suspend L autopilot.

The mash toggle survives stocks and matches until the plugin reloads. It does
not disable tech. Automatic tech and SDI otherwise follow their startup settings;
SDI hands back physical control when you deliberately move the stick in hitstop.

See [the menu](LEDGE_MENU.md), [shortcuts](TURBO_CONTROLS.md),
[slides](SLIDE_CONTROLS.md), [ledge trump](LEDGE_TRUMP.md),
[recovery cover](TWO_FRAME.md), and [fighters](FIGHTERS.md).
