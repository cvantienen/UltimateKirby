# Saved ledge menu

See the [complete editor action cheat sheet](EDITOR_ACTION_CHEATSHEET.md) for
every picker entry, its behavior, corresponding file syntax and examples. A
[styled searchable/printable table](EDITOR_ACTION_CHEATSHEET.html) is also available.

Spot punish is available and defaults ON in silent and diagnostic builds.
**Grab dodge + punish** and **Out of shield** are available in both builds and
start ON with the supplied defaults. Configure their `grab_dodged` / `shield_hit`
responses in the Training reaction editor, and save per-fighter startup switches
in Fighter settings. No matching response means no automatic follow-up. Diagnostics also
expose **Perf**. Ledge trump is available in both builds as **Built-in: Ledge
trump** in Button configs; it is not an ON/OFF mode and is unassigned by default.
Temporary choices/mode toggles last only for this match. Each new match starts
from saved recipe defaults and normal mode startup settings. **Perf** samples local observation
work and shows a compact timing card below the menu after about five seconds;
full ranked gauges, sampled collision-ray counts and runtime health reports go to
`performance.log`. Other diagnostic evidence remains in `status.log`.
See [performance testing](TRAINING_MONITOR.md#testing-feature-cost-in-diagnostics).
Regular Roll read and the fighter-specific ledge responses remain available.
Auto-tech is available for all fighters as the compact **Tech:ON/OFF** row,
initially ON with the supplied config. An explicit `auto_tech = false` still
starts it OFF. Its temporary setting lasts through the current match. Select it with Up/Down and toggle with Left/Right.

The lightweight menu starts visible on the ledge page at every battle entry. It contains option rows only: no character/menu heading or control-hint footer. Training starts hidden and keeps its editor heading and hints. **Minus** opens/closes either menu in every build, including silent and diagnostic Training compatibility. The former L+R3 binding is removed. While
visible in Training, **Up/Down** selects a row and **Left/Right** changes its choice.
Minus always opens the ledge home from hidden and closes from any page.
Outside Training, Minus opens only the five ledge choices. **R3** switches
between this list and a separate **Buttons / Assists** list; Minus closes either
page and next opens ledge choices. Both pages use cached plain text, with up to
nine rows around the cursor on the auxiliary page. **Hold L + D-pad** to navigate
or change values; hold Up/Down to scroll rows. Without L, configured D-pad actions
work while the menu stays open. Release a navigation direction before using it
as a gameplay binding. R3 and Minus work without L. The panel fits its text,
wraps long names and omits the ledge-option counters. There are no editor,
preview, CRUD or save controls outside Training.

Training retains the full layout: the ledge home has **Button recipes** and
**Assist modes** links (A/R3 or Left/Right opens them), **R3** edits,
**Recipe options** inside Step tools manages Add/Duplicate/Rename/Delete,
**Y** saves immediately, and **X** tests inside the editor. In the compact
selector **L3** tests/stops the highlighted recipe. Unsaved Training drafts can be selected
for testing but are discarded when the match ends. Save before leaving Training.
Outside active battles, hold Plus+Minus together for 0.35 seconds to toggle the
runtime master, then release both before another toggle. The independent match
lock blocks this gesture during play even when the master state is OFF.
A small top-left **Kirby Assist: ON/OFF** confirmation appears for 1.8 seconds,
then clears so it does not remain across unrelated screens.
**Auto-mash** is now an editable **ON/OFF** row, shared across fighters and
preserved through stocks, then reset to its normal startup setting next match. Minus does not
change mash. Disabling mash releases its cached input without disabling tech.

Hold the assigned ledge binding (**X** by default) for ledge autopilot (including Kirby in-range recovery cover),
use the selected fighter-specific **X/Y/D-pad** bindings with the menu hidden.
**Normal button** restores the physical button's ordinary game input. Built-in
L is fixed walk/ledge assistance; customizable defaults are X short hop and L3 turbo. The Turbo t-bag built-in can be assigned to any button slot, and L3 is never remapped to the ledge-assist role. Directions held while closing must be released before gameplay
uses them. Minus is reserved for the menu. Hold the assigned Turbo t-bag button for fast crouching.

The menu uses a fixed-width dark rounded card, with a teal highlight and a
separate navigation area. Silent stays at the **top left**; diagnostic stays at
the **top right**. Compact shows the selected option for every row. The wider
editor keeps numbered action summaries and their settings together, marks edited
steps, and highlights the exact selected value. The pinned header identifies
**STEP N / M** while the action list scrolls. **Action slots** at the bottom
expands the editor up to 32; reducing it removes only unused trailing slots.
No images, animation, font loading or additional UI library is used.

It begins hidden each battle, stays through stock loss until hidden, and closes
outside battle. In Training, editing also works during entry/countdown once your fighter is
known. Lines are rebuilt only on selection revisions. No emoji font, texture,
asset loading, animation or live gameplay read was added; actual rendering cost
still needs device measurement. The diagnostic result card retains its own dark
backdrop for detailed calibration text.

Common move labels are abbreviated: `Dtilt`, `Ftilt`, `Utilt`, `Dsmash`, `Fsmash`,
`Fair`, `Uair`, `SH Zair` (short-hop zair), and `Homing Msl` (homing missile).
`Shield Nair` means shield then nair; `Dtilt > Shield` keeps the opposite order.
Near/stay/resource/risky recipe labels retain their meaning. Display-only
aliases are resolved when the cached snapshot
changes; move IDs, functions, selection order and timing are unaffected.

Move names show the compact saved action without appended coverage estimates,
for example `Nair`, `Dtilt` and `SH Zair`. The old spacing-hint helper remains
available in source but is no longer called by the menu. This display change
does not alter response timing, selection order or input behavior.

The five ledge lists remain fighter-specific. Onstage
[roll reactions](ROLL_READ.md) use saved sequences and do not change the ledge
Roll list. The old
Y native combo/Finisher selector and binding remain removed. The compact selector
and editor use directly linked pages: five ledge lists, fighter-specific X/Y/D-pad
bindings, and available mode ON/OFF settings. Button bindings select only input sequences or Normal button; hard-coded D-pad B-reverse/Down-trump are removed.
No universal button recipes or shared edit cache remain. Menus are available
only in battle; arena/character-select editing and its SDK capture hooks are removed.
Automatic whiff punishing and auto-duck have been removed.

**Auto-reflect** appears in Assist modes only for a supported fighter with an
explicit `reflect_default` setting. Game & Watch is the first configured fighter,
enabled at startup in his supplied file. Left/Right toggles the current match's
mode; Training's Fighter settings edits and saves its startup default, prediction
envelope and timing allowance. Other fighters never inherit Bucket input.

This mode watches eligible incoming weapon/item hitboxes and requests Down+B
only from native-ready movement, ordinary jumps/falls, or its own already accepted
Bucket. It yields to manual action buttons, held shield, committed punishes/grabs,
safety, editor/preview controls and all held/toggled ledge trapping. Grounded rear
shots may require a confirmed turn first; rear shots in the air do not trigger it.
Full Bucket, catch/reflect/end states, lost threats and interruptions release the
request. It never automatically discharges oil. See
[projectile configuration and limits](SEQUENCE_FILES.md#automatic-projectile-reflection).

All five ledge response lists are built exclusively from the selected fighter's
validated in-memory `sequences.txt` definitions on activation. File order,
`name`, `default`, `start_delay` and ordered actions define
the menu and its behavior. No compiled choices are appended and missing entries
have no fallback. Changing a selection caches that file timeline; no files or
strings are read while playing. Diagnostic `[TEST]` sections are ordinary file
choices filtered by build type. See [the action/timing guide](SEQUENCE_FILES.md).
Silent builds filter test IDs and `[TEST]` labels at startup. Long editor recipes
scroll vertically while the selected step remains identified in the pinned header.

The adapter caches labels until a menu revision and resolves response metadata
only after fighter/choice changes. Drawing reads a cached snapshot, with no
native fighter or opponent queries. Full ledge/movement observations, including
ground-support rays, run only while ledge or trump needs them. Enabled
Roll reactions use shared opponent observations without ground rays.
A released trigger still permits an in-flight trump to finish. Active
move timing is not throttled. Baseline mash, tech and SDI remain automatic.

This caching limits repeated configuration work; device heat and frame time
have not been profiled. Source edits do not update an installed NRO. See
[current controls](CONTROLS.md) and [fighter guides](FIGHTERS.md); rebuild and
install separately to use source changes.


## Frozen diagnostic calibration results

Diagnostic builds provide a [frozen result card](CALIBRATION_CARD.md) for all
five ledge response lists. The compact card appears beside the menu after an
attempt ends; Minus hides/shows both. It records available configured timing,
request/mapping/startup evidence, root spacing and waiting gates. Full detail
is written to `status.log`. Missing metadata stays unknown; configured frames
are not measured hitboxes. Silent builds exclude this feature.

Minus close requests update a separate visibility flag before attempting the cached-text lock, so the next draw skips the menu even if its previous snapshot is still in use. Closing preserves selections; release Minus before pressing it again to reopen.

Kirby additionally has **Idle crouch**; all fighters have a **Tilt face** mode
and independently configurable move settings. Both modes start ON; down tilt
is selected by default only for Kirby/Yoshi. Holding L suspends idle crouch
until release. The A+B
jab-to-grab helper and its row are removed; A/B keep normal game behavior. The
[posture/input guide](INPUT_RECIPES.md) explains their priorities.

Yoshi also has a separate **Jab lock** ON/OFF row, initially OFF. It chases only
his own confirmed down-tilt hits and takes priority over background Roll read.


## Shared editing and save limits

Silent and diagnostic use the same Training-only recipe manager. **Y Save**
starts one temporary worker immediately in Training; there is no pending-save
queue or outside-battle wait. While a save runs, the visible editor waits without queuing invisible edits
or repeated Save/Delete requests. Other explicit save requests are rejected
as busy; no second snapshot is retained. Keep Smash running until
success or error appears. SD reads, validation, backup and verified replacement
run on that worker, not on the input/render callback. Its 2 MiB stack is released
after completion and worker cleanup; silent gains no logging or performance timers.

An available local Training editor can save even while gameplay input is
temporarily unavailable. Press **Y alone**; holding another button shows a
release-buttons message instead of silently ignoring Save. This also applies
to Step tools and input pickers. Saves remain unavailable outside Training.
Large valid files keep their compact layout if decorative section headings
would exceed 512 KiB; formatting does not reject an otherwise valid edit.

In **Step tools → Recipe options → Save as startup default**, the current recipe is saved and its category
gets exactly one `default = true`; other choices in that fighter/category become
nondefault. This works for ledge and button recipes. The permanent Normal button
and untouched empty Custom template cannot be saved this way. See
[save safety](SEQUENCE_FILES.md#in-game-save-safety).

Successful writes replace the compact indexed saved catalog on the save worker.
Only the selected fighter expands owned live recipes and automatic rules; other
fighters retain dormant definitions. Fighter changes release the previous live
menu and rule cache. Diagnostic names are copied into bounded owned labels so
reports remain valid after those menus are released. At match end all per-fighter
menu records, selections, drafts, Undo, name edits, temporary mode toggles and
render snapshots are released. Death/respawn alone does not discard them; cleanup
waits for the existing match guard to unlock. Fresh matches load current saved
recipes/defaults from the catalog, without gameplay SD reads or repeated static
menu publication. The startup reference data and saved catalog remain loaded as
configuration. External FTP edits still require restarting Smash.

Explicit **Check** actions can require grounded/airborne states, shielding,
facing-relative enemy position or horizontal distance. Zero wait requires the
condition immediately; a positive wait bounds the wait and ends the remaining
recipe on failure. A true check continues on the same update. These use copied
facts without new native queries; enemy checks require a fresh observation of the
same target. See [checks and button holds](SEQUENCE_FILES.md#editing-button-holds-and-checks).


## Faster recipe editing

Both builds share the same editor controls and wider corner-anchored editor card.
The selected step expands its settings onto separate lines; other steps stay
compact. The pinned step header and scrolling focus identify the current value.
Compact ledge selection and its gameplay controls are unchanged.

| Editor control | Action |
| --- | --- |
| A | Open Step tools for the current step |
| ZL / ZR | Jump directly to the previous / next step, skipping its settings |
| Hold R + Up / Down | Move the selected step up / down; its delay/settings move with it |
| Up / Down | Select a field; holding scrolls faster |
| Left / Right | Adjust the field; numeric holds accelerate |
| B | Back to parent tools; from editor fields, back to the selector |
| R3 | Done: return to compact or the assist rule list, closing any popup |
| Y / X | Explicit Save / Test of the current recipe, including from Step tools/pickers |

Step tools offer **Choose action**, **Insert before**, **Insert after**,
**Duplicate step**, **Move up/down**, **Delete step**, **Undo last edit**,
**Copy step**, **Paste step before/after**, and **Recipe options**. The step
clipboard carries the action, settings and before-delay between this fighter’s
recipes in the current editor session. Pasting uses the same limits and Undo
as other edits. Recipe options opens rename/add/duplicate/delete/default controls.
Delete removes the selected action and its own delay, shifting later steps
without changing their settings. Insert creates a blank slot without adding
frames. Duplicate copies the selected action and delay. Limits are checked
before changes are accepted; the previous recipe stays intact on rejection.
Undo stores only one previous recipe snapshot for that fighter's editor and
covers numerical, action-selection and structural changes. It is not an
unbounded history and does not undo a completed SD save automatically; press Y
to save an undone recipe if desired.

Choose action opens presets grouped as attacks/aerials/tilts,
jumps/jump attacks, specials, defense, movement, controller inputs, checks and
repeated taps. Each fixed entry appears in one category; dynamic saved inputs
remain in their separate group.
**Left/Right** changes group, **Up/Down** chooses an action (hold to scroll),
**A** applies, and **B** cancels. Browsing and cancelling do not change a recipe;
reselecting its current action preserves customized values. Choosing a different
action keeps the selected step’s delay and, where compatible, its hold duration.
Short/Full hop explicitly choose 1/4-frame pulses. Selecting a saved imported
input restores that saved payload, including its hold duration.
**B** returns to the parent tools or editor; from editor fields it returns to
the selector. Onstage reaction pickers omit actions requiring a ledge anchor.
Save/Test rejects incomplete conditional blocks and invalid reactions without
starting playback or writing a file; an incomplete draft suspends only its own
automatic rule. Rename preserves existing Unicode characters and edits whole
characters; X inserts after the cursor, with the existing 64-byte name limit.

**Buttons** on an input opens eight independent A/B/X/Y/L/R/ZL/ZR toggles.
Use A or Left/Right to toggle, Up/Down to select, and B to return. **Controller
input** also exposes both sticks and their independent screen/facing/opponent
horizontal references, allowing movement + C-stick + buttons simultaneously.
This is canonical input; the game’s configured mappings/state determine the move.
There are no separate duplicate Hold A, Fsmash/Usmash/Dsmash, Dash+A or specific
button-combination shortcuts. Choose the combined A entries or Controller input.
Legacy saved actions remain compatible. The legacy Ledge fallback direction is
not offered for new selections; use Toward saved ledge when that is intended.

Picker processing runs only in the editor. Idle tool/picker frames reuse cached
UI text; no new gameplay observations, native hooks, polling workers, files or
background processing are added. Edits touch at most 32 slots. The one undo
snapshot is allocated only by an edit and included in diagnostic retained-memory
accounting. Existing save behavior and gameplay clocks remain unchanged. Controller input
uses the existing literal timeline and mapper, with no readiness checks or game
writes; target-based references use the existing observed opponent. Source checks cannot replace on-device layout/performance testing.


## Copy another recipe as a template

Open the **destination** recipe with R3, press **A**, and choose **Copy recipe
from...** in Step tools. **Left/Right** selects any of the current fighter's
five ledge categories or six X/Y/D-pad categories, skipping empty groups;
**Up/Down** chooses a source.
**A** replaces the destination's complete action sequence and timings in memory;
**B** cancels. Use Undo last edit if needed, then **Y** explicitly saves.
To create a new option rather than edit an existing one, use Step tools →
Recipe options → Add first, or
open a Custom template, then copy into it.

Copies include all actions, before-delays, sequence start delay, timeout and
spatial timeout. Unsaved source edits are used. The source recipe is untouched;
the destination keeps its name, identity/default selection and independent Run
once / Loop while held setting. A ledge destination never gains button looping.
Normal button, empty templates, deleted entries and the destination itself are
excluded. Sources are limited to this fighter; no cross-fighter lookup occurs.

Only source identities/names are collected when explicitly opening the chooser.
Actions are resolved on confirmation, not copied into a retained roster. The
chooser list is freed on apply/cancel/editor close or input reset, and included
in diagnostic retained-memory accounting. Idle frames do not rebuild it. Copying
uses the existing bounded Undo and Save paths without files, game queries or
additional background work.

## Hidden-menu work

Recipe editing is available only in actual Training Mode, in both builds.
Outside Training, the two-page plain-text selector never opens editor/list tools
or preview tests. Visible text is rebuilt on revisions; hidden menus skip ImGui
context/IO access. The shared renderer/callback remains loaded, so this is not a
claim of zero renderer cost or a measured cure for heat.

During a locked match the outside-battle master gesture skips native HID reads
and retains its existing 250 ms unlock probe. Immediate Training-save completion
uses a one-result flag; no pending queue, SD polling or idle worker runs online.
Gameplay clocks, move detection and simulated inputs are unchanged.

Training permission is copied before the shared state lock and reused by the
shortcut and menu policies. New battle entry cancels a stale deferred cleanup
request before the new menu session is used. Neither change adds native writes,
SD work or gameplay timing adjustments; neither establishes the cause of a GPU
crash observed while entering Training.

The graphics draw callback does not apply save results or release fighter/editor
rosters. One-shot maintenance runs on the existing mapper callback before its
state lock; confirmed outside-match cleanup and immediate Training-save results
retain their guards. First fighter entry uses startup menu references directly
until a successful in-game save changes the owned catalog. This avoids redundant
catalog scans/temporary allocations before any editing has happened. These are
source corrections to callback workload; a GPU-crash cause is not established by
compilation or host tests alone.

## Reaction recipe settings

Current automatic recipes are grouped under **Assist modes → Auto react →
L3/R3**, including enemy roll/spot/air dodge, unshielded and shielded grab,
after dodging enemy grab, and shield backup. Automatic grabs and Grab avoidance
rows open subsets of the same cached recipes. There is one sequence per event;
legacy priority lists remain separate. A/R3 edits, X adds, Y manages and B/L3 returns.
See [rule setup](REACTION_RULES.md#configure-in-training).

### Reaction activation groups

New rules use their assist's Menu ON/OFF control. Existing saved roll rules may
still use a Toggle X/Y/D-pad control group; those groups reserve that button and
start OFF each match. This legacy reaction metadata is different from the
retired `mode = ...` button binding. Defensive rules use their own mode ON/OFF.
Actions copied as a template preserve the destination activation and identity.
Ordinary ledge/button editors do not offer reaction tagging.

## Fighter settings in Training

Open **Minus → Fighter settings** on the ledge home, then press **Left/Right**
or **R3** to enter. This separate page exists only in Training, in both builds.
Up/Down selects a setting; Left/Right adjusts it, **X** restores that setting's
configured startup value, **Y alone** saves the fighter's overrides, and **R3** returns
to ledge choices. Minus closes. Existing defaults and each explicit recipe's
movement/timing values remain unchanged unless edited. Ledge, button-sequence and reaction-rule responses are
suspended while this settings page owns the controls.

| Setting | Meaning |
| --- | --- |
| Walking stick % | Baseline for the restored legacy walking curve: partial horizontal input uses a slightly lower cap, and full horizontal stick uses 50%. Fine input and crouching are preserved. |
| Grab reach estimate | 1–60 world-unit estimate used by `grab:fighter_range` and derived grab-approach calculations. Does not extend an actual grab. |
| Ledge acquire X / Y | Horizontal/vertical target acquisition limits, used when **Use acquire/follow limits** is ON. |
| Ledge follow X / Y | Limits for retaining an acquired ledge target, with that same switch ON. |
| Ledge stand distance | Stop gap for held ledge-binding manual walking toward the known ledge; also the automatic stance offset. Toggling autopilot ON alone does not limit movement. |
| Auto-position stick % | Stick strength for that automatic stance path. |
| Manual walking cap | Existing manual-spacing switch; ON permits the walking-speed cap. |
| Auto ledge stance | Existing automatic positioning switch. Eligible automatic positioning also requires manual spacing OFF and acquire/follow limits ON; file recipes continue to own their movement. |
| Use acquire/follow limits | ON enables the four distance limits. OFF retains existing distance-agnostic tracking. |

Each field shows its configured value and whether it uses a default or an override.
The displayed walking baseline uses the legacy curve, including its 50% full-stick
cap. Saved values above 45% remain accepted but clamp to 45% for the partial-stick
baseline. The previous exact-cap override has been removed.
Distances use world units; percentages limit simulated analog input, not native
fighter speed. Explicit `walk`, `drift`, grab numeric checks and other recipe
parameters retain their own values. Startup/delay/frame timing remains in recipes.
Additional analytical settings are exposed where relevant to the fighter:

| Setting group | What the values mean |
| --- | --- |
| Shield warning X/Y and scan X/Y | Approach warning envelope and candidate cutoff, not attack reach. |
| Shield body width/height/depth | Estimated controlled-fighter envelope used for incoming-hit prediction. |
| Shield forecast and tail hold | Prediction horizon and brief retention after a threat. |
| Grab-dodge padding/height | Extra contact allowance and vertical tolerance around attacker grab estimates. |
| Tech body/probe offsets and padding | Read-only floor/wall/ceiling prediction probe placement. |
| Jab-lock reach/chase distance/height | Reach estimates and chase/watch bounds. |
| Crouch range/height | When the nearby-opponent crouch helper can apply. |

Unedited values preserve the existing policy. Some defaults are generic
estimates, not measured hurtbox dimensions; move/grab facts use fighter reference
data where available. These settings change the assist's decisions, never actual
hitboxes, grab length, movement physics or native move timings. Conditional
assist-rule range/side/priority remain per rule, separate from these defaults.

Y uses the existing one-at-a-time Training save worker and the same verified
`sequences.txt` temporary/backup replacement. There is no autosave, new hook,
background polling or new opponent scan. Overrides are resolved with selected
recipes on configuration changes; matches reuse copied numbers. Enabling existing
automatic positioning can enable its existing collision work. The page itself
adds no collision work, and unedited settings preserve prior behavior.

The **Ledge trap mode** switch keeps the assist active without holding the ledge binding. That binding remains
reserved; with the toggle ON it adds the manual walking limits without restarting
the already-active autopilot. Either activation source disables Auto react,
automatic grabs/shield backup and grab avoidance, including their opponent searches.
Their saved switches stay unchanged; fresh checks resume when both ledge sources are OFF. Always-on assist continues the existing tracking/defense work; it has a
larger active duty cycle than holding the binding only at ledge. CPU observation timings
alone cannot establish total console load or explain temperature.

<a id="manual-walking-stop-distance"></a>
### L and the autopilot toggle

| Ledge trap mode toggle | Ledge binding released | Ledge binding held |
| --- | --- | --- |
| OFF | Ledge autopilot inactive | Ledge tracking/responses plus eligible grounded walking-speed and ledge-gap limits |
| ON | Ledge tracking/responses active; no manual walking limits | Same ongoing autopilot plus eligible walking-speed and ledge-gap limits |

The original walking curve is restored: fine input/crouching is preserved,
partial input uses the configured baseline minus 0.02 (clamped 0.1–0.45), and full
horizontal stick uses 0.5. Ledge stand distance releases only outward walking input
near an already-tracked ledge, using copied velocity for a small look-ahead.
Walking away remains available. No known ledge means no distance stop; momentum
can carry the fighter beyond the gap. Explicit recipe movement, attack direction
and existing post-response hand-back retain their priority.

The assigned ledge-binding source is consumed with the menu hidden and on both
lightweight selector pages, including R3 transitions, preventing its native game
action from leaking. Compact navigation pauses/resumes held ledge-binding readiness; full editing suspends
playback. While the toggle is ON, pressing L does not cancel/restart a running
ledge response. Existing passive ledge shielding, when enabled in configuration,
follows effective autopilot activation rather than requiring physical L.

Held L and toggle activation feed one assist; no duplicate tracker is started.
B inputs explicitly requested by a recipe remain available. The distance stop
uses the known anchor and existing local observations, with no new ledge search.

### Editing hints and settings-save isolation

Training recipe editors, step tools and add/rename/delete dialogs show their
control hints again. Fighter settings keeps navigation/reset/save/back hints
visible alongside save status. Quick gameplay selectors retain their compact
presentation; hint strings are cached when the menu changes.

On Fighter settings, release other buttons and press **Y alone** to save. A chord
is rejected with an on-screen message. A settings-only save uses the existing
temporary worker and verified backup/replacement transaction, but publishes only
that fighter's numeric override record. It does not replace the in-memory recipe
catalog or request recipe-overlay rebuilding. File validation still covers all
recipes before replacement. Ordinary recipe saves retain their existing path.

### Saved-ledge directions and facing blocks

In a movement step, change **Direction** to **Toward saved ledge** or **Away from
saved ledge**. Existing strength/duration/distance controls still apply. The
**Conditions / checks** picker also contains **If facing ledge**, **If back to
ledge**, and **End facing block**. Place actions between each opening marker and
its end marker. Only the block matching your sequence-start facing executes.
No nested blocks; missing anchors stop safely. See
[facing-dependent recipe examples](SEQUENCE_FILES.md#facing-dependent-ledge-recipes).
These use copied facing/position and the existing anchor, with bounded forward
skips; no new hooks, SD reads, collision searches or automatic move-data changes.

### Repeated saves without edits

If the recipe's actions/timing and requested name are already saved, pressing Y
reports **Already saved; file unchanged**. This still validates the file, but
skips temporary-file replacement, backup rotation and live recipe-catalog
publication. Existing comments/formatting and the previous backup are preserved.
Actual edits, renames, deletions and startup-default changes retain their normal
validated save path. A successful validation does not identify the cause of a
native game or GPU crash.

The build helper currently prints a reminder to revisit the deferred
Training-to-online GPU crash investigation. Fresh-boot Elite worked in the
reported test; that observation does not establish the cause. The reminder does
not install a test config or change Deluxe/overclock settings.

### One response per ledge visit

A get-up response is consumed when that visit first starts its selected recipe.
Releasing/reholding the ledge binding, a menu edit, or another assist interrupting it cannot
restart the same visit's sequence just because the opponent is still standing
with a recorded previous get-up status. An already-handled visit waits for a new
ledge visit rather than reserving another response. The original response's
inputs/timing are unchanged while it remains active.

A newly observed ledge grab rearms the opponent. A newly entered native cliff
get-up state can also establish a visit when its hang was missed; a standing
fighter's previous-status field alone cannot. The tracker holds at most eight
small opponent records, uses existing observations, and is cleared for a new
match or controlled fighter/object. Explicit editor tests and button-loop recipes
keep their independent activation behavior.

Held-L walking uses only the current/most recently observed opponent cliff
position retained by this tracker. It never falls back to the automatic stance
finder or a platform-floor edge. The remembered edge survives L release and
ordinary assist cancellation within the match. A newly observed visit with no
known anchor clears the old edge until an actual cliff position is observed.
No anchor means speed limiting only; no guessed distance stop. No additional
stage probes, roster scans, save workers or physics changes are introduced.

### Save worker lifetime

Explicit Training saves retain their temporary worker handle until it exits.
Cached menu-text replacement and cleanup are deferred during the save; the
existing panel stays visible, and visible-editor inputs wait without queuing
hidden edits or another held Save. Normal gameplay detection is not paused.
Completion polling checks that the worker has finished before joining it; file
I/O stays off the input/render threads. Match-end cached-text cleanup is retried
once saving finishes. Temporary-file verification and recovery backups remain.

This hardening addresses the observed save-start/menu-text destruction overlap.
The silent crash report showed a null write while freeing cached C strings;
on-device confirmation is still required to establish that the crash is resolved.

### Training frame references

While editing a recipe, focus a move or its settings to see the selected
fighter's startup, active frames, total frames and landing lag (when available).
The action picker previews the highlighted move before applying it. Ground/air
aliases show both move references; specials label the referenced form. Missing
or ambiguous data is not substituted with another fighter's timing. Editing
custom stick values can make an input ambiguous, in which case no named-move
hint is shown. Hold duration is controller-input time, not attack startup.

Neutral get-up, ledge roll, jump and get-up attack recipes also show a cached
roster average for total duration and invulnerability end, with the total-duration
range. These are reference entries, not the current opponent's exact timing or
a countdown from recognition. Actual invulnerability and recovery can vary.
Existing sequence schedule hints still describe input delivery only.

References use the already compiled fighter tables. Their text is rebuilt with
the existing menu cache on selection/edit changes; the small roster summary is
computed once lazily. No new hooks, opponent scans, file reads or gameplay-frame
calculations are added. Hints do not appear in lightweight battle selectors and
do not change saved recipes or automation.

Diagnostic save statistics report active workers and their peak; obsolete save-queue counters were removed because Training saves use one immediate worker rather than a queue.

### Legacy separate assist rule lists

The current seven-event Auto react list is described above. Older files and
legacy tools retain the following separate rule-list vocabulary.

Sequence-driven assists have their own lists: Roll reactions, Out of shield,
Spot punish and Grab dodge + punish. In Training, highlight the assist
and press L3 or R3. Up/Down selects a rule, A/R3 edits, X adds, Y manages
(rename/add/duplicate/delete), and B/L3 returns. While editing, R3/L3 returns to
that list. Rules are ordered by priority, with at most eight per assist. The
assist's ON/OFF row controls normal Menu rules; it is not a selected ledge option.

Assist rules are excluded from the ledge and button selectors, their startup
defaults and ordinary input playback. Existing saved sections and their IDs are
preserved and routed by their reaction tag. The text organizer gives each assist
its own group. Explicit Training preview is still available, without publishing
that rule as a normal ledge response afterward. Standalone helpers such as
tech/mash and facing retain their existing ON/OFF behavior.

Assist rows are grouped with sequence-driven reactions first, followed by ledge
activation, safety helpers, fighter helpers and diagnostics.
No extra hooks, opponent scans, rays or save workers are added. Rule resolution
and list formatting happen on configuration/input changes, not idle list polls.

**Spot punish** also has its own L3/R3 rule list and `spot_dodge` trigger. This watches an opponent's dodge, unlike Grab dodge + punish. Create a rule to enable a response; no hard-coded smash fallback is used. See [Spot punish](SPOT_PUNISH.md).

### Cached text after saving

Menu rows now use explicit independent C-string allocations rather than an
in-place `Vec<String>` to `Vec<CString>` collection. Empty rows retain their NUL
byte, interior-NUL text is skipped, and cache allocation failures keep the last
complete snapshot. This targets the observed main-thread cached-text cleanup
fault after save completion. File backup/verification and save-worker isolation
remain in place. On-device confirmation is still required; portable stress tests
and matching crash offsets alone do not establish the underlying target-library
cause or guarantee that a reported crash is eliminated.

See [every Fighter Settings option explained in simple terms](FIGHTER_SETTINGS.md),
including what increasing each value does and how acquire/follow differ.

The fixed legacy Roll read row and its seven punish/roll estimate settings are
retired and hidden. Old keys still parse; they cannot activate automation or
affect custom rule timings. See [retired settings](FIGHTER_SETTINGS.md#retired-fixed-punish-settings).

## Non-Training selector refresh and switching

The match selector keeps cached text for its five ledge rows or Buttons/Assists
page. Cursor-only navigation updates focus metadata under the existing snapshot
lock, reusing the text allocations. Page changes, actual recipe switches and
ON/OFF changes invalidate the values; live roll-feedback changes do not. Hidden
menus still skip drawing. Deferred match cleanup prevents reusing an earlier
match's snapshot even for the same fighter.

Opening, navigating, changing selector pages and closing the lightweight menu do
not cancel an ordinary held ledge-binding ledge response. Consumed selector D-pad also keeps
always-on autopilot active; hidden gameplay D-pad retains its previous behavior. An existing explicit button preview
still stops when the menu takes over. Changing a recipe or mode retains the
existing cancellation of old pending inputs before the new configuration runs.
Cycling a one-choice list is a no-op. Unavailable/unsaved choices are not numbered
as the first saved recipe. Button recipes and assist rows retain fresh-press
controls; holding a direction does not repeatedly toggle them.

These changes leave Training editing, saves, sequence files, detection clocks,
mode priorities and render-host lifetime unchanged. They reduce redundant menu
work, not the cost of the enabled gameplay modes, and are not proof of a specific
FPS or temperature improvement on hardware.

In the Training compact selector, L3 tests/stops the hovered recipe and R3
opens editing. X retains its saved choice (short hop when unconfigured). L is the fixed
walk/ledge trigger and has no editable binding row. Full-editor
X Test is unchanged. Hidden-menu L3 defaults to turbo t-bag; unassigned D-pad Down is Normal.


## Menu maintenance checks

Switching from Training to the lightweight selector restores the original manual
choices before discarding the assist editor's temporary selection. It closes
rule lists, action tools, copy/picker dialogs and previews, resets editor repeat
state, and cancels the old preview on that mapper sample. Draft recipe contents
remain available until the ordinary match cleanup; no editor, save or list tools
become active in the battle selector.

Automatic grabs and Grab avoidance now open their sequence subsets. B/L3
returns from a list to the same Assist modes row. On the + Add row, Left/Right
chooses a missing event. General Fighter settings still returns to ledge home;
legacy grab-range sliders are hidden when explicit grab sequences configure them.

Closing the selector hides it immediately. Snapshot text is released through the
existing save-safe, nonblocking cleanup gate. Once that cleanup finishes, hidden
binding/preview updates skip the snapshot and save locks entirely. Visible
lightweight cursor moves still reuse cached strings; the draw callback still
uses the existing immutable snapshot. Master-switch polling and battle-presence
checks remain independent of selector visibility.

Portable checks cover both core feature variants, all 77 fighter selector pages,
retired-row filtering, Training-to-battle transitions, editor commands, binding
quarantine, save/recovery and cached text ownership. They do not verify the
Switch renderer, native input acceptance or a particular hardware frame cost.

Generic named starters from `[template.NAME]` in the universal source appear in
**Copy recipe / use template... → Generic templates**. Copying leaves the starter
unchanged and preserves the destination name, activation and reaction conditions.
See [template configuration](SEQUENCE_FILES.md#named-generic-training-templates).

The match-start reset reapplies visibility after the actual scene is known, so a
selector reused from an intro or previous battle does not remain hidden or retain
the old auxiliary page. Minus still controls visibility for the rest of that battle.
Cached cursor updates reuse their strings. Page, fighter, generation and layout
changes require matching cached content; a busy Training save cannot display an
old Training card/hints in a match. The lightweight renderer is exercised on the
host with its production snapshot builder and recorded ImGui text/stack calls.
