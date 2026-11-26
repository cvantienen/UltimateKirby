# Current controls

Use this page for physical buttons and mode activation. Use
[Settings and modes](SETTINGS_REFERENCE.md) for the complete configuration
layers, accepted keys, defaults, bounds and extension workflow; use
[Ledge menu](LEDGE_MENU.md) for exact page-specific edit/save controls.

Both silent and diagnostic builds contain gameplay assists. Diagnostic-only
rows such as **Perf** do not become available by enabling a text key in a silent
build. A mode being ON means its eligibility checks may run, not that a move is
guaranteed. Recipe-driven reactions need a matching saved action tape.

**Defaults have three meanings:** Rust fallback, the supplied SD configuration,
and the current match's choices. Auto-tech is OFF in Rust fallback but ON in
the supplied universal section. DI/SDI have the same distinction. Auto-mash is
OFF in both. Fighter overrides and current menu switches can differ again.
Always check your actual active file and menu.

Ledge trump is an assignable **Built-in: Ledge trump**, not the retired standalone
ledge-trump mode. Auto-Spacing & Attack, Counter and Auto Defense are distinct
assignable helpers with hold/toggle variants. Their detailed guides explain
support, prediction and interruption limits.

These bindings describe the current source with the supplied configuration.
Use the selected local controller (`player_index = 0` by default).

While spacing manually with ledge autopilot held, normal horizontal walking is
capped at **0.33** stick; nearly full horizontal input uses **0.50** with the
default config. The fast cap requires at least 95% horizontal and no more than
15% vertical input. Lighter stick inputs and vertical control remain available.

Onstage [roll reactions](ROLL_READ.md) use saved reaction sequences; the old
fixed grab/down-smash selector is removed. The separate ledge Roll row still
controls responses to ledge roll get-ups.

| Input | Action |
| --- | --- |
| Hold L | Fixed walk/ledge autopilot using the five saved response choices |
| D-pad Up/Down, any selector page | Select the previous/next visible row; holding repeats with acceleration |
| D-pad Left/Right, any selector page | Change the hovered recipe or mode; hold to cycle recipes, but ON/OFF toggles change once per press |
| ZL/ZR, Training selector or fighter settings | Previous/next grouped category: Ledge, Buttons, Reactions, Assists, Fighter; works without scrolling to the top |
| Hold R + D-pad, Training selector | Jump eight rows with Up/Down or eight saved choices with Left/Right |
| Hold R + Up/Down, fighter settings | Jump eight visible settings, stopping at the beginning/end; ordinary Up/Down still wraps |
| ZL/ZR or hold R + Up/Down, Training rule list | Jump eight rules, stopping at the beginning/end; hold ordinary Up/Down to scroll |
| D-pad, recipe editor | Up/Down selects a field; Left/Right adjusts its value. Choose a different recipe in compact |
| D-pad, menu hidden | Current fighter's saved recipe/helper; D-pad Down fallback is Turbo, other empty slots use Short hop |
| Minus | Open the ledge home from hidden; close from any page |
| Hold Plus+Minus outside battle (~0.35 seconds) | Toggle the entire mod ON/OFF; works on Arena character select/results |
| Hold the button assigned to Built-in: Turbo t-bag (D-pad Down fallback; saved choices can differ) | Fast crouch/creep; release to stop |
| Auto-mash row, Left/Right | Toggle mash ON/OFF |
| Press X | Current fighter's saved X binding; empty-slot helper fallback is Short hop |
| Press Y | Current fighter's saved Y recipe/helper; inspect its row rather than assuming passthrough |
| L3/R3 on a sequence-driven assist in Training | Open its separate priority-ordered rule list |

The A+B jab-to-grab shortcut and old Y Finisher binding are removed. X/Y/D-pad
and L3/R3 bindings are fighter specific. L/R/ZL/ZR are not custom binding slots.
L retains its legacy fixed walk/ledge role on the default layout; R/ZL/ZR retain
the game's normal configured actions. Old saved trigger-binding sections are
ignored at startup and cannot consume or remap triggers. The internal built-in
`normal` token is displayed as **Short hop**; it is not a separate physical
passthrough choice. Configure an explicit tape when you need different input
behavior, and verify the current slot's displayed helper or saved recipe. Recipes run
once per fresh press or loop while held, according to the recipe setting.
Generic button-to-mode assignments are removed;
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
priority. Tracked weapon/item hitboxes may also be checked; grabs are not detected.
Auto-duck has been removed; an older `kirby_auto_duck` setting has no effect.
The old Y Finisher binding is retired; Y uses its selected saved recipe or
current helper fallback. See [sequence files](SEQUENCE_FILES.md).

Menus appear in the **top-left in silent builds** and **top-right in diagnostic
builds**, using translucent dark rounded panels and distinct sky-blue, lavender, amber, rose and green category accents. Training edit/save control hints are visible; lightweight gameplay selectors
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
  The effective held ledge helper can remain active with the compact selector
  visible. Do not assume it is X: fixed/default-layout L and any saved helper
  assignment determine activation. D-pad navigation takes menu precedence.
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
- Supported gameplay tools retain their own startup/current switches and
  admission rules. Fixed L is not universally remapped to short hop and X is
  not universally autopilot; use the current fighter's button choices.
  Hard-coded slides and the old Y finisher are retired. Auto-mash remains an
  independent optional switch.
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
Alongside saved recipes, the current selectable built-in list offers:

| Built-in choice | Existing behavior |
| --- | --- |
| Short hop | One jump pulse then release; fresh airborne presses remain eligible for ordinary jump handling |
| Turbo t-bag | Hold its assigned button to alternate crouch and slight facing creep; release to stop |
| Ledge trump | Assigned-button activation of the supported ledge-trump input helper |
| Auto-Spacing & Attack (hold / toggle) | Enables the current fighter's spacing/attack helper through the selected activation style |
| Counter (hold / toggle) | Activates the supported counter predictor; currently Lucina-only counter facts |
| Auto Defense (hold / toggle) | Activates incoming-attack defensive input; distinct from Counter and OOS |

Legacy **Ledge assist / walk** remains loadable for old files but is not in the
current selectable built-in list. Fixed L retains its default-layout role.
There is no separate **Normal button** passthrough variant in the current
built-in picker: the serialized token `normal` selects **Short hop**. Older
documentation that described it as passthrough does not match the current code.

On the default layout, **L is fixed Ledge assist / walk**. Saved explicit
startup defaults and the first available manual recipe are resolved before
fallback helpers. The built-in fallback uses **D-pad Down = Turbo t-bag** and
**L3 = Auto Defense**; other slots use the short-hop built-in fallback, with a
supported Counter fallback for R3 where the selector applies it. Your supplied
fighter recipe lists can select different actual choices. Read the visible
button row rather than assuming a fallback is your installed default.
The Turbo helper can be assigned to any supported custom slot; it does not
replace fixed L's role. Select a different saved recipe/helper on the old slot
when moving an assignment; selecting the internal Normal helper means Short
hop, not physical passthrough. Multiple sources share one helper, never multiple executions.

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

Outside Training, the lightweight menu starts visible on its ledge page at
battle entry. Training starts with the full menu hidden; press Minus to open it. It draws option rows
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
selected DI/SDI responses own gameplay input rather than yielding to physical
stick movement. SDI stops for the final three launch-input frames; automatic DI
defaults to Survival. Legacy Adaptive also uses survival behavior. Explicit
Escape remains available but can worsen survival. See [DI/SDI behavior](DI_SDI.md).

See [the menu](LEDGE_MENU.md), [shortcuts](TURBO_CONTROLS.md),
[ledge trump](LEDGE_TRUMP.md), [recovery cover](TWO_FRAME.md), and
[fighters](FIGHTERS.md).

Kirby has **Idle crouch**. Every fighter has a **Tilt face** ON/OFF row,
initially ON; its per-fighter move settings default to down tilt for Kirby/Yoshi
and OFF otherwise.
Idle crouch requires an enemy within 45 horizontal / 20 vertical units and starts
immediately when Kirby is idle; passable platforms remain excluded.
See [input recipes and posture](INPUT_RECIPES.md). For overlay shortcut conflicts
and stats setup, see [Training and monitoring](TRAINING_MONITOR.md). Minus
replaces the old two-stick chord and no longer runs turbo teabag.

Yoshi: enable the separate **Jab lock** row (initially OFF) for automatic
nearby tumble/knockdown/tech chases, including late Nair setups. Techs get grab
attempts.


The current **Auto-Spacing & Attack** built-in is active code, distinct from
manual held-L walking and explicit Walk/Run/Drift recipe steps. It has
fighter-specific settings and hold/toggle activation; see
[Auto-Spacing & Attack](AUTO_SPACING.md). Counter and Auto Defense are separate
helpers, not alternative names for that spacing policy. See
[Counter](AUTO_COUNTER.md) and [Auto Defense](AUTO_DEFENSE.md).

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

Do not assume L3 is a universal turbo binding: its built-in fallback is Auto
Defense, and saved fighter choices can override fallbacks. In Training's compact
selector, L3 tests the hovered recipe and R3 opens editing; physical bindings
have page-specific menu precedence. See [Ledge menu](LEDGE_MENU.md) before
attempting a gameplay helper while an editor page is open.
