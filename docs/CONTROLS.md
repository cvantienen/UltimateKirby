# Current controls

Spot punish (reads an opponent's spot dodge) and Auto-tech are available in both
silent and diagnostic builds and start ON. Both retain independent ON/OFF rows.
Spot punish needs a saved rule; it no longer has a compiled smash fallback.
**Grab dodge + punish** and **Out of shield** are available in both builds, initially ON with the supplied defaults; only diagnostics expose **Perf**.
Ledge trump is available on the Modes ON/OFF page in both builds and starts OFF.
There are no mode choices on the Button configs page. Regular Roll read and ledge-trap
responses remain available. Auto-tech starts ON with the supplied config; an
explicit `auto_tech = false` still starts it OFF. Use
Up/Down to select it and Left/Right to toggle it. OFF skips its collision rays.

These bindings describe the current source with the supplied configuration.
Use the selected local controller (`player_index = 0` by default).

While spacing manually with ledge autopilot held, normal horizontal walking is
capped at **0.33** stick; nearly full horizontal input uses **0.50** with the
default config. The fast cap requires at least 95% horizontal and no more than
15% vertical input. Lighter stick inputs and vertical control remain available.

Kirby's separate [Roll read](ROLL_READ.md) row defaults to Down smash. Select Off to disable it, or Grab
to react to ordinary onstage rolls approaching/passing through Kirby, without
holding fixed L or another assigned ledge binding. The existing Roll row still controls ledge roll get-up responses.

| Input | Action |
| --- | --- |
| Hold L | Fixed walk/ledge autopilot using the five saved response choices |
| D-pad Up/Down, compact or modes page | Select the previous/next row |
| D-pad Left/Right, compact or modes page | Change the hovered recipe or mode (Left previous, Right next; either toggles ON/OFF) |
| D-pad, recipe editor | Up/Down selects a field; Left/Right adjusts its value. Choose a different recipe in compact |
| D-pad, menu hidden | Current fighter’s configured input sequence; Normal button passes through |
| Minus | Open the ledge home from hidden; close from any page |
| Hold Plus+Minus outside battle (~0.35 seconds) | Toggle the entire mod ON/OFF; works on Arena character select/results |
| Hold L3 without the ledge-assist binding | Turbo teabag; release to stop |
| Auto-mash row, Left/Right | Toggle mash ON/OFF |
| Press X | Saved X binding; defaults to Built-in: Short hop when unconfigured |
| Press Y | Current fighter’s Y recipe; Normal button passes through |
| L3/R3 on a sequence-driven assist in Training | Open its separate priority-ordered rule list |

The A+B jab-to-grab shortcut and old Y Finisher binding are removed. X/Y/D-pad
and L3/R3 bindings are fighter specific. L/R/ZL/ZR are not custom binding slots.
L retains its legacy fixed walk/ledge role on the default layout; R/ZL/ZR retain
the game's normal configured actions. Old saved trigger-binding sections are
ignored at startup and cannot consume or remap triggers. **Normal button** passes the
physical button through to the game; it does not apply short-hop assistance. Recipes run once per fresh press or loop while held,
according to the recipe setting. Generic button-to-mode assignments are removed;
legacy reaction-rule toggle groups remain supported separately. Hard-coded
D-pad slide, B-reverse and Down-trump bindings are removed.

Sequence-based assists have separate lists; their rules never appear as normal
ledge/button choices. In Training highlight Roll reactions, Out of shield,
Grab dodge + punish and press L3/R3. Up/Down selects,
A/R3 edits, X adds, Y manages and B/L3 returns. In the editor Y saves and R3/L3
returns to the list. Each assist permits eight priority-ordered rules.

With **Modes → Grab dodge** ON, ordinary standing, dash and pivot grabs are
forecast from that opponent fighter's compiled grab startup and reach. The
assist requests Down+Guard up to six advancing native frames before contact
whenever the game reports that a grounded spot dodge can start. A proposal
suppressed by a higher-priority owner remains eligible during the same grab;
the attempt is consumed only after native spot-dodge startup or a terminal
loss of the threat/readiness window. It does not automatically punish afterward.
Command grabs are not modeled. See [Grab dodge](GRAB_DODGE.md).

Effective ledge autopilot activation (fixed L or another assigned ledge binding, or its ON toggle) also enables the [grounded defensive shield](LEDGE_TRAP.md#held-l-defensive-shield)
when nearby active or known imminent fighter attacks enter an approximate danger envelope. This
works without a ledge grab. Releasing the ledge binding disengages it when the toggle is OFF; committed punishes retain
priority. Projectiles and grabs are not detected by this check.
Auto-duck and its hitbox-reader hook are disabled, even with an older config
that sets `kirby_auto_duck = true`. The old [finisher reference](FINISHERS.md) is historical.

Menus appear in the **top-left in silent builds** and **top-right in diagnostic
builds**, using dark rounded panels and a teal selection highlight. Training edit/save control hints are visible; lightweight gameplay selectors
remain compact. Save/error statuses, action-slot numbers and edited values remain
visible.

## Controller mapping

### Training Modpack installed

Only when **diagnostic build + Training Modpack detected + Training Mode**:

- **Press Minus:** open the ledge home; close from any page.
  Open the Button recipes / Assist modes / Ledge traps rows with Left/Right
  to switch pages directly.
  **Press R3 on compact:** open the highlighted recipe directly for editing.
  **R3 in the editor:** return to compact. **Minus:** close.
  The compact view shows Neutral, Jump, Roll, Getup Attack and Drop
  together; it switches saved responses without editing or managing recipes.
  **Hold X** still runs ledge trapping with the compact selector visible. D-pad
  navigation pauses it; releasing the D-pad resumes it without releasing X.
  The assigned ledge binding takes over from a running recipe test.
  **L3** tests/stops the highlighted ledge recipe in Training’s compact selector. Ordinary A/B and
  other gameplay buttons pass through; a manual action cancels a running test.
  The first page contains five ledge lists; the second contains eight button bindings (X/Y, D-pad, L3/R3). The modes page shows
  every available mode and its current
  ON/OFF state or selected behavior; fighter/build restrictions still apply.
  On compact and modes pages, Up/Down selects a row and Left/Right changes it.
  Action editing retains Up/Down for fields and Left/Right for values. Release
  each control before pressing it again. Menu navigation blocks
  and cancels D-pad specials, including held directions through closing/release.
- All configured Kirby Assist gameplay tools remain enabled: **X** autopilot,
  **L** short hop, mash/tech/SDI, slides, follow-ups
  and configured Roll read. Auto-mash has its own ON/OFF menu row.
- Default **ZL+Up load / ZL+Down save** keeps our panel visible and resumes
  automatically on release, including held ledge-binding practice. No R3 tap needed.
- Real **Plus** or known Training menu/recording chords hand the complete
  controller back to Training and cancel pending assist input. After closing
  Training's menu/recording, **tap Minus** to resume with our menu visible.
- Leave Minus unassigned in Training's custom shortcuts. Custom Training chords
  beyond the defaults are not automatically detected.

This uses stock Training Modpack and no additional hook/companion. Silent builds,
standalone Switches and non-Training modes use **Minus** for our menu. See
[compatibility and test steps](INTEGRATION.md#training-modpack-compatibility).

Map **A to Attack**, **B to Special**, **R to Grab**, **ZR to Shield**, and
**both X and ZL to Jump**. Kirby's forward-facing attack-cancel bair also needs
**C-stick set to Tilt Attack**. Keep Tap Jump off for directional input recipes.
Recipes change raw inputs before your mapping, so a different mapping can
change or prevent their intended actions. The built-in jump helper also marks
the final mapped Jump input so its physical assignment does not depend on X being Jump.

Set `control_layout = brother` for R/ZR Jump, ZL Shield, L Special, X Grab and
A Attack. Trigger custom mappings are disabled in this layout too. Saved X/Y,
D-pad and stick-click choices remain effective; L3 can still be assigned a helper. Assisted canonical inputs are
translated back to those physical buttons, including R+ZR for a two-button
short hop; Minus remains the menu control.

## Remap built-in button actions

Open **Minus → Button configs**, highlight a button and cycle its choices.
Alongside saved recipes and **Normal button**, every button offers:

| Built-in choice | Existing behavior |
| --- | --- |
| Short hop | One jump pulse then release; fresh airborne presses still jump normally |
| Ledge assist / walk | Hold to slow walking, apply the saved ledge-distance limit and activate ledge trapping; the ON toggle still keeps trapping active independently |
| Turbo t-bag | Hold to run the existing turbo teabag helper; release to stop |

On the default layout, **L is fixed Ledge assist / walk**. Customizable defaults
are **X = Short hop** and **L3 = Turbo t-bag**. Y and D-pad Left/Right/Up retain their
saved choices; unassigned D-pad Down and other buttons are
Normal. Explicit saved defaults win.
Select a built-in on any other button to move it, and select Normal on the old
button if desired. Multiple sources share one helper, never multiple executions.

In Training, press R3 on the selected built-in to open its information page,
then **Y** to save it as that fighter/button's startup default. Built-in actions
have no editable attack tape; custom recipes retain the ordinary editor and loop
settings. R3/L3 remain menu controls while the menu is visible. Plus/Minus are
not assignable gameplay bindings. Master enable and native safety still apply.
An assigned held ledge/turbo helper enables its own action while held, so a legacy
ledge/turbo enable flag cannot silently disable the selected binding.
The Brother layout uses the same new physical helper defaults. Its ordinary
game-action translation is unchanged; button labels refer to physical buttons.

## Runtime master switch

Auto-mash starts OFF in both builds. Enable it explicitly from its menu row if
wanted; enabling the runtime master does not turn auto-mash on by itself.

On Arena character select or results, hold **Plus+Minus together for about
0.35 seconds** to toggle Kirby Assist. Release both for at least 0.15 seconds
before toggling again. Taps, partial releases, missing controller samples and a
chord carried out of a match do not toggle the mod.
The battle lock is independent of menu visibility/activity and remains active
when the master switch is OFF, including intro/gameplay and pauses. A small
read-only battle check and a render-requested native heartbeat maintain this
lock; neither runs gameplay policies. The selector also rechecks manager flags
outside mapper callbacks, at four Hz
while locked or during the explicit toggle chord, to unlock on character select.
While OFF, pending actions, input overlays
and diagnostics cards are cleared, and the controller mapper passes through
unchanged. ON restores all configured policies with their saved individual
settings; it does not force optional toggles ON or select conflicting recipes.
OFF also discards cached controller/fighter associations, tracking receipts,
session menus and Training handoffs. Re-enabling requires a fresh validated
association before assistance resumes; an old in-flight observation cannot
restore a cancelled action. Optional mapper/draw profiling, projectile tracking
and periodic diagnostic reports are gated off too. Only the small local
battle/master-gesture check remains so the mod can be turned back on. A save
explicitly started before disabling is allowed to finish safely.
After a successful toggle, a tiny translucent
**Kirby Assist: ON** or **Kirby Assist: OFF** confirmation appears in the top-left
for 1.8 seconds. It is not left visible across unrelated game screens.
The startup `enabled = false` setting remains a permanent no-hook disable until
the game is restarted with the setting changed.

Arena spectating does not authorize assistance merely because a controller is
connected or a fighter is marked human. The selected mapper requires a fresh
callback-discovered fighter ID, matching native controller ownership and a live
identity recheck before menus, analyses or input overlays run. Missing, expired,
inactive, recycled, CPU and other-controller routes pass through to the native
mapper. Losing participation clears the local session once; it leaves the master
ON/OFF preference intact. These checks use existing local game observations,
with no network/service queries, new scene hooks or native gameplay writes.

## Menu behavior

The lightweight menu starts visible on its ledge page at every battle entry,
including a reused selector from an earlier intro/match. It draws option rows
only, with no character/page heading or control hints. **Minus** hides or shows it
during the current battle. Outside Training it shows five ledge choices;
**R3** switches to/from a separate Buttons / Assists page. Up/Down navigates and
Left/Right changes saved choices/toggles. The text-only selector is top left in
silent and top right in diagnostic; closing skips menu drawing.

Training retains the full editor: page links, R3 editing, step tools and
B Add/Duplicate/Rename/Delete/**Save as startup default**. **Y** saves immediately
using one temporary worker; wait for completion before another save. No queue or
outside-battle wait. At confirmed match end, menu/editor data is freed and next
match loads saved recipe defaults. Unsaved edits/selections are discarded.

Opening/closing the menu or changing a saved choice cancels pending manual-tool
inputs. Release directions held while closing before using a hidden-menu tool.
Minus toggles visibility and no longer triggers turbo teabag. D-pad directions,
native safety responses can interrupt when needed. An acquired ledge setup is
protected from competing configured actions until its response finishes.

Temporary mode toggles survive stocks but reset with the match menu session. It does
not disable tech. Automatic tech and SDI otherwise follow their startup settings;
DI/SDI respect a stick already held or moved outside 20% during hitstop, retaining manual control for that hit. SDI stops for the final three launch-input frames; automatic DI defaults to Survival. Legacy Adaptive also uses survival behavior. Explicit Escape remains available but can worsen survival. See [DI/SDI behavior](DI_SDI.md).

See [the menu](LEDGE_MENU.md), [shortcuts](TURBO_CONTROLS.md),
[slides](SLIDE_CONTROLS.md), [ledge trump](LEDGE_TRUMP.md),
[recovery cover](TWO_FRAME.md), and [fighters](FIGHTERS.md).

Kirby has **Idle crouch** and Kirby/Yoshi have **Dtilt face** ON/OFF rows, initially ON.
Idle crouch requires an enemy within 45 horizontal / 20 vertical units and starts
immediately when Kirby is idle; passable platforms remain excluded.
See [input recipes and posture](INPUT_RECIPES.md). For overlay shortcut conflicts
and stats setup, see [Training and monitoring](TRAINING_MONITOR.md). Minus
replaces the old two-stick chord and no longer runs turbo teabag.

Yoshi: enable the separate **Jab lock** row (initially OFF) for automatic
nearby tumble/knockdown/tech chases, including late Nair setups. Techs get grab
attempts; missed techs get jab-lock attempts into down smash. See [tech chase](JAB_LOCK.md).


The experimental automatic Spacing assist, its settings and dedicated hitbox/
movement prediction code are removed. This does not change manual walking,
file-authored movement actions or held ledge-binding positioning and defense.

## Menu availability and saves

The selector is battle-only; the full editor is Training-only. Arena lobbies,
character select and title screens have no editor or last-used-fighter view.
Minus closes/opens the selector; R3 changes the two lightweight match pages.
Plus+Minus master toggle remains available outside battle.

Training Save starts immediately on one short-lived worker. Another Save/Delete
while busy is rejected with a retry message; no pending snapshots accumulate.
Keep Smash open until completion. Verified backups and replacement recovery
remain enabled. Saved recipes/defaults survive; transient fighter menus do not.

All fighters have **Out of shield** and **Grab dodge + punish** mode rows,
initially ON with the supplied defaults. Their `shield_hit` / `grab_dodged`
response recipes are configured in Training. Fighter settings saves the per-fighter
startup switches and grab forecast window. See [OOS](OUT_OF_SHIELD.md) and
[Grab dodge](GRAB_DODGE.md).


Button recipe editors include **Button activation** near the bottom: **Run once** (existing behavior) or **Loop while held**. The latter
repeats the full configured X/Y/D-pad recipe while its assigned button is held,
and cancels on release. Use **Repeated taps → Tap input: Down tilt** for repeated down-tilt inputs.
Its picker offers only A, B and the three tilts, with delay and total duration. See [hold-to-repeat settings](SEQUENCE_FILES.md#hold-a-binding-to-repeat-its-recipe).


In recipe editing, **A** opens Step tools (grouped action picker, insert before/
after, duplicate, move, delete and one-step Undo). **ZL/ZR** jumps between steps;
**hold R + Up/Down** moves the selected step with its delay/settings. These inputs
are captured only by the editor and require release before returning to gameplay.
Tool/picker **B** cancels/back; normal-editor B retains recipe-list management.
Holding Up/Down scrolls fields or picker rows faster. See [faster editing](LEDGE_MENU.md#faster-recipe-editing).

## Buttons during automatic input sequences

A committed input owner finishes before another configured button sequence can
start. Buttons pressed during that ownership are not queued; release and press
again to use them afterward. Short-hop and turbo helpers also require release
if pressed during a committed sequence. Damage/tech safety preserves ordinary
jump input, including built-in jump bindings and unconditional one-frame
jump-only recipes, so recovery inputs can still buffer. The held ledge-assist helper and menu navigation
remain available. A real ledge acquisition and native safety can still preempt
automatic reactions; general punish modes cannot interrupt a committed ledge
sequence or automatic grab. These rules use existing observations, not added
frame waits or online wall-clock compensation.

Fixed native jump triggers (Default ZL; Brother R/ZR) suspend automatic punish
modes and return their controls immediately. A fresh shield press cancels an
active Auto react tape except its initial grounded B input. In the air, fresh
shield still hands back control for an air dodge. An already held shield allows
its OOS reaction.
This exception does not mix inputs into committed ledge or explicit button tapes.
Game inputs already queued online and native recovery cannot be undone.

Universal built-in helpers resolve from the captured physical button once, so
assigning a helper to another retained custom button does not remap it again. Held ledge
assistance remains active throughout its own response. Short hop preserves its
own held source while delivering the hop, consumes the held tail, and permits
a fresh airborne jump after release. Unrelated input tapes still block new
button sequences until release. Visible-menu D-pad navigation takes priority
over a helper assigned to that D-pad direction; the ledge binding remains usable in the compact
selector. Saved recipe selections and defaults are not reset by these fixes.

An always-on ledge watch without an acquired ledge response yields to manual
short hop and turbo t-bag. A real reserved ledge visit or committed ledge sequence keeps priority.

Built-in jump press/release and walk/turbo stick overrides are retained in the
final mapped controller packet, after physical-source consumption. This only
runs for the selected built-in owner. Sequence-generated controller buttons
retain their ordinary configured control-layout meaning and never invoke custom
button assignments. Normal/manual input and competing sequence owners bypass
this built-in delivery step. No game statuses, moves or physics are modified.

The held walk helper continues to cap manual movement after a ledge response
finishes; the response completing does not turn off a still-held walk modifier.

The turbo helper alternates Down/release per local input update. Fixed L retains
legacy autopilot takeover in hidden and compact menus; it cannot be reassigned
to short hop or another recipe.

L3 is the only default turbo t-bag binding in gameplay. In Training’s compact
selector, L3 tests the hovered recipe and R3 opens editing; X remains available
for its assigned helper. Full-editor Test controls are unchanged.
