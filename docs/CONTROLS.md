# Current controls

These bindings describe the current source with the supplied configuration.
Use the selected local controller (`player_index = 0` by default).

While spacing manually with ledge autopilot held, normal horizontal walking is
capped at **0.33** stick; nearly full horizontal input uses **0.50** with the
default config. The fast cap requires at least 95% horizontal and no more than
15% vertical input. Lighter stick inputs and vertical control remain available.

Kirby's separate [Roll read](ROLL_READ.md) row defaults to Down smash. Select Off to disable it, or Grab
to react to ordinary onstage rolls approaching/passing through Kirby, without
holding L. The existing Roll row still controls ledge roll get-up responses.

| Input | Action |
| --- | --- |
| Hold L | Ledge autopilot using the five saved response choices |
| D-pad Up/Down, menu visible | Select ledge lists; Kirby/Yoshi also have Roll read; Kirby/Yoshi have Finisher; Auto-mash is shared |
| D-pad Left/Right, menu visible | Cycle the selected row's saved responses |
| D-pad Left/Right, menu hidden | Start/repeat a downward air-dodge slide; L is not required |
| D-pad Down, menu hidden | Attempt a ledge trump |
| D-pad Up, menu hidden | Normal mapped input; no mode cycling |
| R3 | Show/hide the menu |
| Hold L3 without L | Turbo teabag; release to stop |
| Auto-mash row, Left/Right | Toggle mash ON/OFF |
| Press X | X short-hop tap on the ground |
| Tap A+B as Kirby, neutral stick | Jab 1 → ordinary grab attempt; A alone stays normal (A+B grab row ON) |
| Press Y as Kirby | Selected Finisher attempt: Fair 1 > Fsmash or Dair > Dsmash |
| Press Y as Yoshi | Short-hop Bair drag > Fsmash attempt; fresh early hit required |
| Hold L as Kirby | Ledge responses plus in-range down-tilt recovery cover |

Kirby recovery cover runs within held-L autopilot with no separate R3 binding
or toggle. Other fighters keep their existing ledge responses. The **Finisher**
row selects a Y starter/confirmed-smash attempt; release and repress Y to retry. Y is
reserved on other fighters until modes are configured. Yoshi also has a
hit-confirmed bair drag-down attempt. Whiff punish is removed from the live
build/menu for now. B-reverse is unbound;
combo playback remains removed.

Held L also enables the [grounded defensive shield](LEDGE_TRAP.md#held-l-defensive-shield)
when nearby active or known imminent fighter attacks enter an approximate danger envelope. This
works without a ledge grab. Releasing L disengages it; committed punishes retain
priority. Projectiles and grabs are not detected by this check.
Auto-duck and its hitbox-reader hook are disabled, even with an older config
that sets `kirby_auto_duck = true`. See [Finisher controls and limits](FINISHERS.md).

## Controller mapping

### Training Modpack installed

Only when **diagnostic build + Training Modpack detected + Training Mode**:

- **Click R3:** show/hide Kirby Assist's menu; D-pad Up/Down selects a row and
  Left/Right changes it. Release R3 before toggling again.
- All configured Kirby Assist gameplay tools remain enabled: **L** autopilot,
  **Y** finisher, **X** short hop, mash/tech/SDI, slides, follow-ups
  and configured Roll read. Auto-mash has its own ON/OFF menu row.
- Default **ZL+Up load / ZL+Down save** keeps our panel visible and resumes
  automatically on release, including held-L ledge practice. No R3 tap needed.
- Real **Minus/Plus** or known Training menu/recording chords hand the complete
  controller back to Training and cancel pending assist input. After closing
  Training's menu/recording, **tap R3** to resume with our menu visible.
- Leave R3 unassigned in Training's custom shortcuts. Custom Training chords
  beyond the defaults are not automatically detected.

This uses stock Training Modpack and no additional hook/companion. Silent builds,
standalone Switches and non-Training modes use **R3** for our menu. See
[compatibility and test steps](INTEGRATION.md#training-modpack-compatibility).

Map **A to Attack**, **B to Special**, **R to Grab**, **ZR to Shield**, and
**both X and ZL to Jump**. Kirby's forward-facing attack-cancel bair also needs
**C-stick set to Tilt Attack**. Keep Tap Jump off for directional input recipes.
The plugin changes raw inputs before your mapping, so a different mapping can
change or prevent the intended action.

## Menu behavior

The compact, title-free menu starts visible in the top-left corner of battle.
It survives stock loss until hidden and closes outside battle. It shows five
saved response lists, available fighter modes, option counts and editable Auto-mash.
It uses compact move names with the existing spacing hints, larger/heavier text
and a narrower panel. `SH` means short hop, `Msl` missile, and `Dtilt/Ftilt/Utilt`
mean down/forward/up tilt; `Fair/Uair` mean forward/up air.
Ledge response lists have no Off choices. Roll read and Auto-mash have explicit
Off choices; no live combo-step indicators are shown. Choices come from the
current fighter's compiled profile; selecting one changes cached settings.

Opening/closing the menu or changing a saved choice cancels pending manual-tool
inputs. Release directions held while closing before using a hidden-menu tool.
R3 toggles visibility and no longer triggers turbo teabag. D-pad directions,
shortcuts and higher-priority mash/tech can suspend L autopilot.

The mash toggle survives stocks and matches until the plugin reloads. It does
not disable tech. Automatic tech and SDI otherwise follow their startup settings;
SDI hands back physical control when you deliberately move the stick in hitstop.

See [the menu](LEDGE_MENU.md), [shortcuts](TURBO_CONTROLS.md),
[slides](SLIDE_CONTROLS.md), [ledge trump](LEDGE_TRUMP.md),
[recovery cover](TWO_FRAME.md), and [fighters](FIGHTERS.md).

Kirby also has **Idle crouch** and **Dtilt face** ON/OFF rows, initially OFF.
See [input recipes and posture](INPUT_RECIPES.md). For overlay shortcut conflicts
and stats setup, see [Training and monitoring](TRAINING_MONITOR.md). R3 alone
replaces the old two-stick chord and no longer runs turbo teabag.

Yoshi: enable the separate **Jab lock** row (initially OFF) for automatic
confirmed down-tilt chases. Techs get grab attempts; missed techs get jab-lock
attempts into down smash. See [down-tilt chase](JAB_LOCK.md).
