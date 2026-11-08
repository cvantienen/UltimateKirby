# Ledge input timelines

For a complete copy/paste reference, use the [sequence text syntax cheat sheet](SEQUENCE_SYNTAX_CHEATSHEET.md) or its [searchable HTML version](SEQUENCE_SYNTAX_CHEATSHEET.html).

The editable repository sources are split under `deployment/ultimate/kirby_assist/`:

- `sequences.txt`: universal settings and named generic Training templates.
- `fighters/<fighter>.txt`: all of that fighter's settings, ledge/button recipes and automatic reactions, for example `fighters/bowser.txt`.
- `assembled/sequences.txt`: generated combined catalog for validation/tests. Edit the source files above; generated edits are replaced on assembly.

Build and sequence-upload tools assemble the current sources automatically into
one Switch `sd:/ultimate/kirby_assist/sequences.txt`. Smash still reads one file
at startup, with no includes, extra disk reads or parsing during matches.
The `[universal.settings]` section takes precedence over the legacy `config.txt`.
All existing recipe values were preserved when splitting the files.

Pyra and Mythra have separate native identities and separate fighter files.
Their initial five `custom` ledge slots contain `wait:0`, providing editable menu
entries without selecting attacks or swaps for the player. Author their actions
and automatic events in Training; unconfigured automatic attack/reflect modes
stay inert. A form change reloads only the newly selected form's saved choices.

To assemble or check without building/uploading the mod:

```sh
python3 tools/sequence_sources.py
python3 tools/sequence_sources.py --check
```

Training saves still update the Switch's combined file. Download that file before
rebuilding if it has newer edits. A Workbench transfer to the repository source
path splits it back into fighter files with backups and readback checks. Or import
an already downloaded copy explicitly:

```sh
python3 tools/sequence_sources.py --import-file /path/to/downloaded-sequences.txt
```

Imports back up replaced sources under `.sequence-sync-backups/`; do not replace
the universal source directly with an entire combined Switch file.

Fighter banners group settings, ledge responses, button recipes and mode
sequences. Saving, renaming, deleting and transferring recipes in a master file
refreshes these headers and keeps each fighter together. Choice order within a
category stays unchanged. Organization runs only during explicit file operations,
not during gameplay. This format requires an updated mod build; do not send the
new master file to an older installed build.

`sd:/ultimate/kirby_assist/sequences.txt` controls ordered controller inputs after
code recognizes the opponent's ledge option. It can add choices or replace an
existing choice. Recognition, target identity, controller routing, menu controls,
trigger release, match resets and interruption handling stay in code.

The file is the sole option catalog and loads/validates once at startup. Inactive
fighters retain compact indexed definitions containing only their actual actions.
Only the selected fighter gets expanded owned menus and automatic-rule state;
changing fighters or ending a match releases the old live menus and rules.
Generic editor templates are shared once. Activation uses the in-memory index,
without reading fighter files or scanning the cast during gameplay. Ordinary
timeline steps use fixed numeric inputs; idle menu polls allocate nothing.
Missing sections mean missing options; a missing file leaves empty ledge menus. One invalid section rejects
the file and prevents hook installation; validate before uploading.

## Automatic projectile reflection

Projectile defense is an independent **Auto-reflect** Assist row. It currently
supports only Mr. Game & Watch and requires an explicit fighter settings section:

```ini
[mr_game_and_watch.settings]
reflect_default = true
reflect_startup = 6
reflect_margin = 2
reflect_max_hold = 45
reflect_half_width = 5
reflect_height = 16
reflect_half_depth = 5
```

Bucket starts absorbing/reflecting on native frame 6; the configuration cannot
make that earlier. See [Fighter Data](FIGHTER_DATA.md) for the canonical
fighter-specific move facts.
`reflect_margin` adds an early activation allowance to the current straight-line
contact prediction. It inserts no wait. `reflect_max_hold` bounds a confirmed
response using observed native/projectile progress. The three body values are
prediction estimates, not changes to the native hitbox. Exact setting bounds
are listed in the [syntax cheat sheet](SEQUENCE_SYNTAX_CHEATSHEET.md).

The existing ATTACK observer records at most 32 projectile identities while needed,
including a fresh generation for each native attack. Reused weapon/item IDs and
new attack phases do not inherit a previous shot's duplicate guard. There is no
post-reflection cooldown: a new shot is eligible on the first observation where
Bucket is ready and not full. Duplicate observations of the same attack stay
suppressed, including after a cancelled input.
The detector revalidates identity, active attack, team, situation and native
reflectable/absorbable flags; direct melee attacks are excluded. A swept 3D
envelope uses current relative velocity and at most 16 active hitbox geometry
reads per observation. There is no roster search, file I/O, allocation or network
work in this detector. Disabled/unconfigured/unsupported scopes make no native
projectile queries. Dead projectile IDs are retired from the shared ring.

Only native-ready movement or ordinary jump/fall may start Bucket. Holding
shield, attacks, grabs, rolls, tumble, capture, recovery, safety input and ledge
trapping suppress admission. Already committed punishes retain ownership. The
grounded mode can turn toward a rear shot before Down+B; an airborne rear shot
is ignored rather than forcing a turn/recovery input. An active automatic bucket
does not switch targets or replay a consumed attack after cancellation.
Confirmation requires a final delivered Down+Special packet and observed native
Bucket startup. Missing acceptance has both native-progress and delivery bounds.
After its initial pulse, a delayed native receipt keeps automatic ownership
reserved while leaving physical movement free and sending no repeated special
input. A last-resort mapper-poll bound also releases frozen/unseen startup or
turn requests; duplicate copies of one poll do not spend that bound twice.

Absorbable attacks are absorbed; eligible nonabsorbable shots may be reflected.
A gauge of three or more suspends the mode: it does not dump oil to empty Bucket.
It releases on catch/reflect/end, vanished threat, manual takeover or interruption.
Master OFF, spectating, match exit and fighter changes use the normal teardown.

This is prediction from live hitboxes, not guaranteed immunity: shots already too
close for startup, curved/bouncing paths, reflector limits, unknown/preexisting
projectiles not observed by ATTACK, crowd limits and unsuitable spacing can miss.
The body envelope and delivery timing need on-device calibration. Additional
fighters require a verified move/resource adapter; copying these settings into an
unsupported fighter does not enable it. Rebuild the mod before deploying the new
settings, since older plugins do not recognize the new keys.

## Auto react sequence group

Automatic responses use one `reaction = EVENT:MAX_X:MAX_Y` sequence per event
under each fighter's `.roll.` category. The shared Auto react group contains
`auto_roll`, `auto_spot_dodge`, `auto_air_dodge`, `auto_grab`,
`auto_shield_grab`, `auto_grab_dodged` and `auto_shield_backup`. All accept the
normal action syntax and editor/saving path. Duplicate events are rejected.

For example, `[bowser.roll.auto_grab]` has
`reaction = auto_grab:35:24` and `actions = input:r:0:0:4`; its shielded partner
uses `auto_shield_grab:25:10`. The grab-dodge follow-up uses
`auto_grab_dodged:25:10` independently of enemy spot-dodge reactions.
These are activation thresholds, not modified hitboxes.
Only `auto_grab` requires enemy shield. `auto_shield_grab` can select any
vulnerable, grabbable front opponent while you shield. If no legal grab wins,
`auto_shield_backup` uses its separate limits and native readiness.

The migration preserves all existing ledge/button recipes and timing, relocates
the effective automatic-grab ranges for all 77 fighters, and copies Bowser's
previous grab-dodge follow-up exactly. It does not enable previously absent
roll/dodge responses on other fighters. Missing new entries retain old-file
fallbacks. Recognition, native readiness/delivery, priority and native grab/throw
handoff remain input-only code safeguards. See [Auto react](REACTION_RULES.md).

## Button bindings and built-in actions

Button section rows are `x`, `y`, `dpad_up`, `dpad_down`, `dpad_left`,
`dpad_right`, `l3`, and `r3`. Trigger rows `l`/`r`/`zl`/`zr` (including
`lz`/`rz`) are retired and ignored, so old files remain loadable but cannot remap
triggers. Literal trigger inputs inside an action remain supported normally.
The eight customizable rows use the same recipe syntax as other input sequences.
Existing saved recipes remain valid. Legacy `builtin = short_hop` entries now
behave as `builtin = normal`; create a `short_hop` recipe when you want an
automated short-hop press.

A native helper is selected with `builtin = normal`, `ledge_assist` or
`turbo_teabag`. These choices call the existing helper, not a sequence of
attacks. They cannot contain `actions`, reaction conditions, startup delays,
`while_held` or `ground_only`. `start_delay` is not required for a built-in.

Use an ordinary recipe when you want a short-hop input on a button:

```ini
[bowser.r3.short_hop]
name = Short hop
start_delay = 0
actions = short_hop:1
default = true

[bowser.x.normal]
name = Normal X
builtin = normal
default = true
```

Keep only one `default = true` per fighter/button row. The Training editor's
save-as-default action handles this automatically. R3/L3 are reserved for menu
navigation while visible; their gameplay assignments work when the menu is hidden.
Normal means physical passthrough, including ordinary X jumps. With no saved
button default, X uses short hop and L3 uses turbo teabag. Default-layout L is
fixed walk/ledge assistance and is never a saved binding. D-pad Down and other unassigned buttons
are Normal.
These new slots/built-in entries require the corresponding updated mod build.

## Sections and frame timing

```ini
[kirby.drop.jump_up_b]
name = Hop into Up B
start_delay = 0
timeout = 180
spatial_timeout = 30
actions = Short Hop, wait:2, up_special:1, wait:12, down_special:1
```

Frame 0 is the first eligible observation of the recognized option. This recipe
holds Jump on frame 0, releases on frames 1–2, sends Up+B on frame 3, releases on
frames 4–15, then sends Down+B on frame 16. No action-readiness or grounded-only
gate is inserted. The game may buffer or reject an input during recovery; the
file does not wait for acceptance or automatically retry it.

The header is `[fighter.situation.choice_id]`. Situations are `neutral`, `jump`,
`roll`, `attack`, and `drop`: they describe the **opponent's** option. Fighter IDs
come from the configured roster. File order is menu order, and IDs are unique
within each row; duplicate sections are errors. `name` (or `label`) supplies a display
label of up to 64 bytes. Open the menu and select a newly added choice; it does
not select itself. File choices display `[FILE]` in battle.

Actions execute strictly left to right, including multiple attacks and waits
between any steps. Bare inputs last **one native frame**. `move:FRAMES` overrides
the duration (1–255). `wait:N` releases buttons and neutralizes both sticks for
N frames (0–255); `release` is `wait:1`. Adjacent inputs sharing a button keep it
held: insert a release when you want another press. `wait:0` consumes no frame.

Section keys are independent of property order. `start_delay` is required; other keys are optional:

| Key | Default | Meaning |
| --- | --- | --- |
| `start_delay` | Required | Neutral frames before the first step; 0–3600 |
| `default` | false | Initial selected option; one true per fighter/recipe row |
| `timeout` | 3600 | Whole-timeline native-frame limit; 1–3600 |
| `spatial_timeout` | 60 | Per-step limit for Face, distance Walk/Run and range checks; 1–3600 |

There are at most 32 **expanded** steps, 2112 sections, and 32 ledge choices per
fighter/situation. Dash Attack expands to two steps; a checked grab adds a range
step; a turnaround prefix adds Face. Files are limited to 512 KiB. Duplicate
normalized headers/properties, unknown inputs and invalid numbers are errors.
Names normalize case, spaces and hyphens; labels keep their authored spelling.
Comments start with `#`. IDs/labels marked `test` / `[TEST]` are diagnostic-only.

## Universal moves and raw controls

All named general moves use the same input mapping for every configured fighter.
They work on the ground or in the air without checking native readiness, hitbox
startup, invincibility, range or whether an attack actually started. The resulting
move still depends on the game's current state, stick/button history and control
mapping, just like physical inputs. Timings and holds belong in the recipe.

| Action | Canonical input / default duration |
| --- | --- |
| `jump`, `short_hop`, `Short Hop` | X / 1 frame |
| `full_hop:N` | X / N frames; an explicit duration is required |
| `short_hop_nair/fair/bair/uair/dair` | X+A plus aerial direction / 1 frame |
| `jab`, `attack`, `nair` | Neutral A / 1 frame |
| `forward_tilt`, `up_tilt`, `down_tilt` | A + 60% directional stick / 1 frame |
| `forward_smash`, `up_smash`, `down_smash` | A + full directional stick / 1 frame |
| `fair`, `bair`, `uair`, `dair` | A + full aerial direction / 1 frame |
| `neutral_special`, `side_special`, `up_special`, `down_special` | B + appropriate stick / 1 frame |
| `dash_attack[:DASH_FRAMES:ATTACK_FRAMES]` | Full facing stick, then facing stick+A / 1+1 frames |
| `shield`, `air_dodge` | Neutral ZR / 1 frame |
| `roll_forward`, `roll_backward`, `spot_dodge` | ZR + appropriate stick / 1 frame |
| `grab` | R / 1 frame, no range check |
| `dash`, `crouch`, `fast_fall` | Appropriate full stick / 1 frame |
| `a`, `b`, `x`, `y`, `l`, `r`, `zl`, `zr` | Corresponding canonical button / 1 frame |

Aliases include `ftilt/utilt/dtilt`, `fsmash/usmash/dsmash`, `Neutral Air`,
`Up B`, `Down B`, `Final Cutter` and `Stone`. Special aliases always mean that
fighter's directional B input; `Stone` does not restrict the fighter to Kirby.
`turnaround_` before a move inserts a Face Backward step before that input.

```ini
[mario.neutral.dash_attack]
start_delay = 0
name = Dash Attack
# Dash for 2 frames, then hold forward+A for 1 frame.
actions = face:opponent, dash_attack:2:1

[kirby.attack.shield_then_up_b]
start_delay = 0
name = Timed shield into Up B
# Fixed timing; this does not wait for a block.
actions = shield:15, release, up_special:2
```

For exact button/stick combinations:

| Syntax | Meaning |
| --- | --- |
| `press:A+B[:FRAMES]` | Button chord with neutral sticks; default 1 frame |
| `hold:B:FRAMES` | Same input representation, with an explicit hold |
| `input:A+X:X_AXIS:Y_AXIS:FRAMES` | Button chord and left stick together (world axes) |
| `facing_input:A:X_AXIS:Y_AXIS:FRAMES` | Same, with horizontal axis relative to facing |
| `stick:X_AXIS:Y_AXIS:FRAMES` | Left stick only |
| `cstick:X_AXIS:Y_AXIS:FRAMES` | Right stick axes and native direction bits |
| `wait:N`, `release[:N]` | Neutral sticks and release owned buttons |

Button names in chords: A, B, X, Y, L, R, ZL, ZR, `attack`, `special`, `jump`,
`shield`, `grab`, or `none`. Axes are **world-relative** values from -1 to 1.
Named move horizontal directions are relative to facing, captured at step entry.
`move:FRAMES:STRENGTH` optionally scales the directional stick (0–100%); e.g.
`side_special:1:50%` uses a one-frame, half-strength side+B input.
C-stick actions obey the player's C-stick configuration. Named tilt/smash/aerial
inputs do not override the game's interpretation; use raw inputs to tune them.

## Spatial actions

Only these steps use observation checks; they use existing copied facts rather
than adding opponent scans or ground rays inside the executor.

- `face:DIRECTION`: capture the direction at entry and wait for observed facing.
- `walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]`: travel up to the specified distance;
  optional opponent-only gap accounts for observed speed/braking. Grounded only;
  aborts at unsafe ground. Strength 10–95%, distance 0–100 units, gap 0–60 units.
- `run:DIRECTION:UNITS`: distance-based full stick, grounded with safe-ground checks.
- `walk_to_opponent:SPEED:FRAMES:GAP`: follow the existing opponent observation
  until within the gap or the explicit time limit expires. Strength 10–95%.
- `drift_to_opponent:SPEED:FRAMES:GAP`: airborne approach, strength 10–100%; the
  configured time includes any remaining jump squat. Add waits first if needed.
- `drift:DIRECTION:STRENGTH:FRAMES` (also `drift_for`): horizontal stick for that
  **duration**. Backward captures the opposite of facing when that action starts,
  so a turnaround cannot reverse it repeatedly. Other directions retain their
  existing per-update resolution. No attack-start gate.
- `shield_until_block:MAX_FRAMES`: hold ZR through a confirmed native block and shieldstun; advance when native guard resumes, or cancel at the file timeout.
- `grab:fighter_range` / `grab:WORLD_UNITS`: an explicit range wait, then one R
  input; no native grab-readiness check. The range step uses `spatial_timeout`.
- `range:fighter_range_instant`: check the current target's horizontal distance
  once against this fighter's configured standing-grab reach. Continue in the
  same observation when in range; cancel the remaining sequence immediately
  when outside reach or the current target is unavailable. No range wait.
- `grab:fighter_range_instant`: the immediate check followed by one R input.
  Use this version for roll-punish grabs that should hand control back if too far.

Directions: `opponent` / `toward_opponent`, `away` / `away_from_opponent`, `ledge`,
`inward`, `outward`, `facing`, `backward`. Toward/away resolve to neutral when
horizontal positions overlap. Target loss keeps the last observed position.
Spatial-step timeout cancels rather than launching a late follow-up. Raw input
steps do not inherit the ground checks or timeouts of spatial steps.

## Fighter-specific X/Y recipes

X, Y and all four D-pad directions have separate recipe lists for each fighter, using the same actions,
start delay and per-action timing controls as ledge recipes:

```ini
[kirby.x.my_route]
name = My X route
default = true
start_delay = 2
actions = short_hop, wait:3, hold:a:4
```

Use `[yoshi.x.ID]`, `[kirby.y.ID]`, `[kirby.dpad_up.ID]`,
`[kirby.dpad_down.ID]`, `[yoshi.dpad_left.ID]` and `[yoshi.dpad_right.ID]` for the other lists. Nothing is shared or
copied between fighters. Without an explicit `default = true`, a button starts
on its built-in default (X short hop, L3 turbo) or **Normal button**
for the other sources. Y and D-pad Up/Left/Right pass through unless configured. Startup validation accepts this permanent Normal button selection even after a non-default recipe has been saved; a default is not required for button lists.
With the menu hidden, a fresh single-button press runs the selected recipe once.
Simultaneous binding buttons do not launch multiple timelines; chords inside a
sequence remain supported. Holding the
button does not repeat it or leak a jump after completion. In the menu, X remains
Test and Y remains Save in the editor. R3 on Normal button opens a separate
Custom template; Normal button remains permanently available.

Old `[universal.x.ID]` and `[universal.y.ID]` sections remain ignored and are
preserved during saves. They never populate fighter lists. Fighter-specific X/Y
sections validate normally. Literal X/Y inputs inside any recipe remain valid.

## Editor frame summary

The editor shows **Input schedule: Nf** for fixed-duration recipes and annotates
slots with `F1`, `F7-10`, etc. This display is one-based: **F1 is the first eligible
recognition update**, corresponding to the zero-based frame 0 used in the example
above. `start_delay = 0` adds no initial wait; a start delay of 4 starts the first
step at F5 before any slot-specific delay/check. Recognition itself can occur
after the opponent animation has already begun.

The total includes sequence start delay, per-slot delays/waits and timed button,
C-stick and drift holds. It is the input schedule, not attack startup, recovery,
FAF or a guaranteed hit timing. The completion label is the next input-control
update after those scheduled frames. A shorter configured sequence timeout is
shown instead of promising completion. Native hitstop/paused progress can extend
wall-clock duration without spending scheduled frames.

Face, distance walking/running, approach, range and shield-until-block checks
can depend on observations or finish early. These show **variable checks**;
subsequent frames are earliest positions marked `F…+ after checks`, not fixed
promises. Existing maximum duration/spatial timeout fields still apply. Empty
unused slots do not add time. No native reads, rendering-time calculation or
execution/timing change was added; the summary rebuilds with the cached editor.

## Mode bindings

The Button configs page selects saved/custom input sequences or **Normal button**.
There are currently no modes, Hold/Toggle choices, or Mode/Activation editor fields
on any X/Y/D-pad row. In Training, R3 edits action slots and Y starts saving immediately.
Minus opens the ledge home or closes any page. Use the Button recipes / Assist
modes / Ledge traps rows with Left/Right to switch pages directly. Automatic features are controlled
only on the Modes ON/OFF page, independently of button assignments.

For backward compatibility, known older `mode = ...` / `activation = ...`
sections still parse, but are excluded from the loaded button choices. An old
mode selected as a default falls back to Normal button; saved input sequences
remain available. The original file is not rewritten or deleted. Buttons cannot
activate, hold, toggle, or override any mode through these legacy entries.
Unknown/malformed sections still fail ordinary validation. New button modes will
be added only when explicitly configured in a future change. The identity
registry remains for compatibility, not as a selectable menu catalog.

Unsaved edits/selections last only for the match. Explicit Training Save uses
the same immediate worker, backups and validation as input sequences.

Menu navigation never runs a D-pad binding; held menu directions require a
release before gameplay can use them. Empty lists remain valid and select Normal
button, including after adding non-default mode/sequence entries. No universal
button lists are created or migrated.

## Editing button holds and checks

In compact, highlight a ledge or button recipe and press **R3** to edit it.
Choose the action, then adjust **Hold buttons: Nf** underneath it with Left/Right.
For example, choose **Hold A button** and set Hold buttons to 30 for a 30-frame
A hold. B, jump, shield, grab and the other input presets have the same duration
setting (1–255 frames). **Delay** waits before the action; **Hold buttons** keeps
its buttons/stick pressed during the action. For example, `hold:a:60` holds A
for 60 native-progress frames. A gap wholly inside an already-active button hold
keeps that input down without generating repeated press edges. Gaps crossing an
action boundary, release or start delay still cancel, as do invalid clocks,
damage and normal sequence cancellation. Hitstop pauses the configured budget.
An active neutral-A hold also retains ordinary mapped Attack/AttackRaw held bits
in the existing mapper result, guarded by current sequence ownership and freshness.
This adds no repeated press edges or attack commands and stops when the hold ends
or is cancelled. It uses no new hook, native query, worker or logging.
Holding A does not guarantee rapid
jab: the fighter's normal game behavior still decides which attack occurs.
The compact selector only changes saved recipes; durations are edited with R3.
In Training, press **Y** to save immediately to the active SD file. Keep Smash
open until the menu reports success/error; retry another Save after completion.
Unsaved edits are discarded at match end.
Existing sequences are not replaced by this update.

The editor keeps each numbered move beside its settings; the pinned **STEP N / M**
header identifies the active step even while scrolling. **Action slots** at the
bottom grows the recipe up to 32 displayed slots. It shrinks only unused trailing
slots. Per-action delays count toward the existing 32 expanded-action limit;
changes exceeding it are refused rather than truncating the recipe.

Explicit condition choices can be inserted anywhere in a recipe:

| Menu check | File action |
| --- | --- |
| I am grounded / airborne | `check:self_grounded` / `check:self_airborne` |
| Enemy grounded / airborne | `check:enemy_grounded` / `check:enemy_airborne` |
| Enemy shielding / not shielding | `check:enemy_shielding` / `check:enemy_not_shielding` |
| Enemy in front / behind | `check:enemy_in_front` / `check:enemy_behind` |
| Enemy X distance within / outside | `check:enemy_within:12` / `check:enemy_outside:12` |

**Check wait limit** is 0–255 frames. With 0, a false condition ends the rest of
the sequence immediately. A positive value waits up to that many native-progress
frames, then ends the sequence if still false. A true condition proceeds to the
next step on the same update. Add the wait as the final file parameter, for
example `check:enemy_within:12:20` or `check:enemy_shielding:8`. These are linear
requirements, not an if/else branch or attack prediction. Enemy checks need a
fresh observation of the original target; cached/missing opponents do not pass.
Distance is horizontal root-position separation in game units (0–100), not
hitbox reach. Front/behind uses current facing. Self checks need no opponent.

For example, `check:enemy_shielding:8, grab:4` waits briefly for a confirmed shield,
then sends Grab; it ends without grabbing if that condition never becomes true.
`check:self_airborne, nair:4` sends Nair immediately only if already airborne.
The checks do not add floor rays or native queries. A positive wait makes the
schedule variable; ordinary input actions remain literal ungated inputs.

Reactive steps are permanent action choices:

- **Wait for shield hit**: shield through a confirmed block and shieldstun, then
  continue when guard resumes; Max wait bounds the attempt.
- **Face direction**: choose Opponent, Away or another direction below it; wait
  for observed facing before continuing.
- **Check opponent distance**: wait until the opponent is within the editable
  horizontal range. This does not press Grab by itself.
- **Check fighter grab reach**: wait within the fighter's configured standing-grab
  reach, then continue. Add **Grab / Zair** next to actually press Grab.
- **Check fighter grab reach instantly**: check once and cancel the rest of the
  recipe if out of range. Add **Grab / Zair** next to press Grab when in range.
  This is useful for roll punishes that should not stand still waiting for reach.
- **No range check**: continue immediately without a range gate.

The original range checks use the sequence's existing spatial timeout (60
native-progress frames by default). They send neutral input while waiting for
horizontal distance, not for a grab animation or successful grab. Hitstop pauses
that clock, so wall-clock time can be longer. The instant version adds no wait;
any earlier Wait, Face, movement or start delay still runs as configured.
Existing ledge recipes retain their original waiting checks. These steps observe
existing game facts and simulate inputs; they do not change moves or physics.
The imported Kirby **Grab / rear nair** recipe is `shield_until_block:90, grab`;
its name does not add an automatic rear-Nair branch.

Imported button combinations/stick values that do not match a named preset now
show their buttons, X/Y percentages and facing-relative or screen-relative axes.
Their original input remains in that recipe's action choices after cycling away,
so it can be selected again. Saved input values reconstruct those choices after
restart. These are literal inputs, not hidden conditional handlers.

## File-only menus and migrated defaults

All 1,515 roster choices are now ordinary file entries, followed by your existing
custom sections. `default = true` marks the initial choice in each of the five
rows. Deleting an entry removes it; there is no Rust fallback or `code:ID` route.
The old conditional recipes have become visible input templates, so their labels
do not imply hidden branches or perfect spacing. Tune the explicit 30-frame
down-tilt/shield wait, 22-unit cross-up walk and other values as needed.

The old config.txt ledge move-choice/timing keys are ignored. Fighter reference
statistics still exist for other tools; they do not choose or delay file inputs.
List file options and initial delays without building the Switch mod:

```sh
cargo run --offline --manifest-path crates/kirby_assist_core/Cargo.toml --example validate_sequences -- --list-options deployment/ultimate/kirby_assist/assembled/sequences.txt kirby
```

## Online timing and limits

The scheduler uses the existing native animation-progress observation converted
by motion rate, **not mapper-poll counts or wall-clock sleeps**. Frame zero emits
the first input immediately. Duplicate polls reuse output; hitstop freezes the
cursor while preserving held input. Native motion transitions and loops preserve
progress; a native observation gap over 1.25 frames or invalid clock cancels the
timeline instead of skipping taps or bursting overdue steps. Trigger release,
disabling the tool, match reset, ordinary capture, hitstun or death also cancels.
Completion releases the inputs and hands control back for this trigger session.

For a zero-rate pose, the scheduler can use the locked opponent’s advancing
native motion clock instead. That fallback reads only the locked actor, without
a roster scan, hitbox/readiness lookup or ground ray. The two clocks are never
added together; missing samples/hitstop rebase the fallback rather than charging
a catch-up gap. Floating-point roundoff is normalized at exact frame boundaries.

No guessed online latency compensation is inserted. Online input delay affects
when the game consumes inputs, as it does with physical buttons; it is not added
again to the file's waits. This is an observation clock, not a true simulation
frame counter: frozen/missing observations cannot prove that a game frame passed.
Host tests cover duplicates, stop, rates, gaps and 0–12-frame delay metadata;
Switch/offline/online testing is still required to verify actual delivery timing.

## Migration, validation and upload

This executor requires rebuilding/installing the updated mod before uploading
new recipes. Older installed NROs can reject newly supported inputs. Compared
with the old one-attack recipe system, multiple attacks, jump → Up B, arbitrary
wait placement and grounded/airborne specials now parse. Input steps no longer
retry, wait for native acceptance, or import fighter startup data. `drift`'s last
number is now a duration, not an attack-animation deadline. Recalibrate existing
waits/drift after migrating. The compiled option catalog and `code:ID` route are removed.

```sh
python3 tools/upload_sequences.py --dry-run
```

This checks the exact deployment bytes with the current Rust parser, menu limits,
and both build filters, without contacting either Switch. Invalid sections stop
uploads. To upload only the file, explicitly run `python3 tools/upload_sequences.py`
with Smash closed and FTP open. It targets both saved IPs on port 5000, attempts
both, and verifies SHA-256 read-back; it builds/uploads no mod. By default, full
uploads and the sequence-only tool send an inactive `sequences.upload.txt` copy,
preserving the active `sequences.txt`. Restarting does not activate that copy.

To replace the active file while uploading a prepared release, close Smash on
both Switches and use:

```sh
python3 tools/upload_all_switches.py -v 2.3.1 --replace-sequences --smash-closed
```

Use your prepared release's version. The same flags work with `upload_switch.py`
for one Switch. Existing active files are backed up to
`/ultimate/kirby_assist/bak/sequences.txt` and verified before any installed file
is replaced; the uploaded configuration is verified by SHA-256 read-back before
the plugin is uploaded. `--replace-sequences --dry-run` previews the active paths
without connecting; `--replace-sequences --verify-only` reads without writing.
The sequence-only equivalent is `python3 tools/upload_sequences.py --replace --smash-closed`.

Full uploads assemble the current universal settings and `fighters/*.txt` at
upload time, rather than using a stale build-folder or generated copy. A valid
local file does not prove an older installed mod supports it.

### Copy sequences between Switches

Save your edits with **Y**, close Smash on both Switches, and start both FTP
servers. Choose the source containing the edits you want to keep:

```sh
# Diagnostic → silent (also the default without --from)
python3 tools/sync_sequences.py --from diagnostic

# Silent → diagnostic
python3 tools/sync_sequences.py --from silent
```

The source is selected manually; the script does not guess which copy is newer.
It copies the active `/ultimate/kirby_assist/sequences.txt` between diagnostic
`.142` and silent `.105`. The source file and project's deployment file stay
unchanged. Unlike ordinary suffixed uploads, this command intentionally replaces
the destination Switch's **active** sequences. It validates the downloaded bytes
for both builds before connecting to the destination. Diagnostic-only recipes
remain subject to the silent build's existing filtering.

Before replacement, it keeps unique local snapshots under
`.sequence-sync-backups/`, named for each Switch, and verifies the previous
destination file at `/ultimate/kirby_assist/sequences.sync.bak`. Backup failure
prevents replacement. The script verifies the transfer with SHA-256 and attempts
to restore the previous file after a failed write or read-back check; if
restoration also fails, it prints the recovery paths. Restart Smash on the
destination after success. An identical file requires no writes. `--dry-run`
downloads/validates the selected source without connecting to the destination or
writing anything; `--port` changes the default FTP port of 5000.


### Transfer one fighter between Switches

Close Smash on both Switches, leave FTP open, and choose the copy to keep:

```sh
# Ask for direction and fighter interactively
python3 tools/sync_sequences.py --interactive

# Preview Kirby from diagnostic -> silent (reads both; writes nothing)
python3 tools/sync_sequences.py --from diagnostic --fighter kirby --dry-run

# Replace silent Kirby with diagnostic Kirby
python3 tools/sync_sequences.py --from diagnostic --fighter kirby

# Replace diagnostic Yoshi with silent Yoshi
python3 tools/sync_sequences.py --from silent --fighter yoshi
```

The script prints source, destination and fighter. `--fighter` replaces **every
section for that fighter**, including all ledge categories and X/Y/D-pad recipes,
names, defaults, actions and saved timings. Destination-only recipes for that
fighter are removed. Other fighters and destination preamble settings stay intact;
source preamble settings are not copied. Use file names such as `ness`, `kirby`,
`yoshi` or `captain_falcon` (spaces/hyphens also normalize to underscores).
A missing source fighter is an error, never a request to erase the destination.
Only settings actually saved in sequences.txt transfer; in-memory edits do not.

Fighter transfers read and validate both files and the merged result before any
SD write. They use the same unique local snapshots, verified SD backup, read-back
verification and failure rollback as whole-file sync. A changed destination during
preparation stops replacement. This check cannot make concurrent editing safe:
keep Smash closed and avoid other FTP writers throughout. Restart the destination
Smash after success. Validation checks current source code, not whether an older
installed NRO supports every action. Without `--fighter`, existing whole-file
sync behavior remains; whole-file dry runs read only the source, fighter dry runs
read both Switches. Neither dry run writes backups or SD files.


### In-game save safety

Editing, saving, renaming, adding and deleting are Training-only in both builds.
**Y Save** immediately starts one short-lived background worker. There is no
pending-save queue and no need to exit Training. Until the result is consumed,
a second Save/Delete returns a busy/retry message rather than storing another
snapshot. Visible menu pages pause editing/navigation until completion is consumed,
so fast input cannot change the target of an in-progress save. The exact compiled
draft is saved; an invalid/oversized draft reports an error instead of falling back
to an older recipe. Keep Smash open until the success/error appears. Closing the editor
or ending Training does not cancel a write that already started.
If the cached menu is being refreshed, the worker waits for that short refresh;
the first Save request and its exact snapshot are retained without blocking input.
Successful completion promotes that snapshot to the editor's saved baseline and
refreshes runtime recipe caches, so reopening does not require repeating an edit.
Newer unsaved edits are preserved when an older save finishes.

The worker validates the catalog, writes and verifies a temporary file, backs up
the previous file and publishes through the existing recovery-safe replacement
path. It has a 2 MiB stack, capped 512 KiB reads and exits after one operation.
Malformed action counts are rejected before slicing/serialization. If a worker
exits without publishing completion, saving is unlocked and an unconfirmed-save
error is shown rather than leaving the editor permanently busy.
No SD read/parse/write runs on the input/render callback. The worker refreshes
the owned saved catalog only after successful replacement; old catalogs are
freed outside the short publication lock. It never publishes leaked startup
menu references. Training writes can cause brief SD/CPU activity while active.
Leaving Training releases unchanged/saved editor records, undo/clipboard data and
temporary tool lists; unsaved or invalid drafts stay available. Lightweight input
polls reuse button-role mappings until their configuration changes. Match-end
cleanup releases session selections and resolved rule/menu caches once the match
guard confirms the battle has ended, preserving them through death and respawn.

**Step tools → Recipe options → Save as startup default** saves the current recipe and selects it for the
next match in its ledge/button category. Other defaults in that same category
are cleared; other fighters/categories and actions are preserved. Normal button
and empty templates are not saved recipes. The default save includes the current
edited actions, delays, hold durations, name and loop setting. Wait for **Saved as
startup default** before leaving Training. Online has no save controls.
Button categories without an explicit default start as **Normal button**, both on
a cold game start and after an in-game saved-catalog reload.

Per-fighter menu/editor state is discarded at confirmed match end. Each new
match starts from current saved recipes/defaults. If native match teardown deferred
cleanup, the next match discards those stale menus before loading its profile.
A save completion is matched to its recipe ID before updating menu bookkeeping;
a reused draft index in a new match cannot mark a different recipe as saved.
Unsaved edits and temporary
match selections do not survive. The owned saved catalog and frozen startup
reference data remain configuration, not a retained fighter editor roster.

Explicit **X: Test** also runs an edited Custom/new draft when the fighter has
no loaded ledge recipes and Ledge Trap mode is OFF. Only that test is enabled;
it does not turn on automatic ledge trapping. The start refreshes the existing
bounded opponent roster so a first test has a target without a prior ledge visit.
Rows with no file entries show empty Custom templates until configured. External FTP replacement of
sequences.txt requires restarting Smash; in-game saves refresh the saved catalog.


In either build, **Step tools → Recipe options** opens list management for the selected recipe: Rename, Add,
Duplicate, Delete or Save as startup default. Accepting a name returns to the
editor; press **Y** to save it with the current actions. Confirmed Delete starts
an immediate write. Wait for each result before saving another recipe. A saved
Custom can be renamed/edited/deleted; blank templates and Normal button remain
permanent.


A fighter may start with no ledge sections and acquire its first recipe in the
in-game editor. Startup-empty lists no longer disable held-binding trapping. Only the
recognized option needs a configured recipe: a Ness neutral recipe works with
Jump/Roll/Attack/Drop blank. Blank categories remain no-response choices; no
compiled moves are substituted. Saving alone does not enable the Ledge trap mode;
hold fixed L or another assigned ledge binding or enable that mode and leave action editing before testing.


### Repeated button taps

Choose the general **Repeated taps** action in a recipe. Its settings are:

- **Tap input:** A, B, Forward tilt, Up tilt or Down tilt.
- **Delay between taps:** released frames between presses, 1–255. The displayed
  `every Nf` interval includes the press and release (a 1-frame press plus an
  11-frame delay repeats every 12 frames).
- **Repeat for:** total duration of this block, 1–3600 native-progress frames.

There is no separate down-tilt repeat action. Select Down tilt through Tap input.
Newly selected inputs use one-frame presses. Tilt choices use A with the existing
0.6-strength tilt direction; forward is relative to facing at this action's start.
Buttons and stick return to neutral between presses. Recovery and grounded/airborne
state still determine which native move, if any, comes out; no readiness gate is
added. The picker deliberately excludes jump/shield/grab, combinations, adjustable
stick axes and tap hold length. Existing saved tap values still load and execute
unchanged; choosing a different Tap input applies the new one-frame preset.

The file syntax remains `tap:BUTTONS:X:Y:HOLD:RELEASE:TOTAL` for compatibility:

```ini
# Down-tilt inputs every 12 frames for a 120-frame block
actions = tap:a:0:-0.6:1:11:120
```

This occupies one execution step. Total includes presses and releases; it may
truncate the final press. Hitstop pauses the frame budget, duplicate observations
do not add taps, and missed native updates cancel rather than catch up edges.
Existing cancellation and timeout rules apply. No hooks, native queries, readiness
checks, physics changes, runtime allocations or workers are added. An updated mod
build is required; older builds may reject `tap` syntax.

The independent **Button activation** setting applies to every custom X/Y/D-pad
recipe, whether or not it contains Repeated taps. Choose **Run once** or **Loop
while held**; looping repeats the full sequence, including its actions and delays.

### Hold a binding to repeat its recipe

On **Button recipes**, choose an X/Y/D-pad recipe, press R3, then scroll to
**Button activation** near the bottom (above Action slots). Set it to **Loop while held**. The default remains **Run once**, preserving
existing recipes. This setting is not offered for ledge-trap categories.

For down-tilt spam, select **Repeated taps**, choose **Down tilt** as Tap input, set its delay,
and enable Loop while held. With the menu closed, hold the assigned button:
the whole recipe repeats, including its start delay, until release. Releasing it
cancels the sequence on the next input mapping update; no new press is queued.
The repeat action's total duration is one block/pass, not a held-binding cutoff.
Loop transitions use the same native clock without an extra mapper-frame gap;
completed passes rebase elapsed time so a long hold does not accumulate timeout.
The timeout still bounds an individual pass. Normal damage, scene/menu changes,
competing bound buttons and higher-priority cancellation still apply. Release
and press again after cancellation. Explicit X Test can also loop a held recipe;
press X again or leave the test to stop it.

```ini
[kirby.y.repeat_down_tilt]
name = Repeat down tilt
start_delay = 0
while_held = true
actions = tap:a:0:-0.6:1:11:120
```

This sends down-tilt inputs every 12 native-progress frames while Y is held.
For a uniform repeated-tap interval across passes, make TOTAL a multiple of
HOLD+RELEASE (120 is a multiple of 12). The script does not insert a release at
loop boundaries; explicit release gaps remain necessary for a new button edge.
Zero-time loops are stopped rather than spinning. No readiness checks or native
move/physics changes are introduced. In-game activation changes are stored with
the recipe by normal Save; installing the updated mod is required first.


### Insert, delete, reorder and find actions

Inside the editor, **A** opens Step tools: choose an action, insert before/after,
duplicate, move, delete, copy/paste a step, or undo the last edit. Step tools →
Recipe options opens rename/add/duplicate/delete controls. B goes back; R3
finishes editing even from a popup. Y/X explicitly saves/tests the current
recipe from tools or pickers too. **R+Up/Down** directly reorders the
selected step, carrying its before-delay and settings. **ZL/ZR** jumps between
steps without visiting every field. Hold Up/Down to scroll fields or picker rows.

The action picker groups the available presets, including checks and preserved
imported inputs. Left/Right chooses a group; Up/Down selects an action; A applies;
B cancels without changing values. Existing direct Left/Right action cycling
still works. Selected-step settings appear on separate lines while other steps
remain compact. Insertion alone adds no timing; deletion removes that step and
its before-delay. Expanded 32-step limits are enforced before applying edits.
Changes remain in memory until explicit Save. See [editor controls](LEDGE_MENU.md#faster-recipe-editing).


### Reuse a recipe as a starter

In the destination editor, **A → Copy recipe / use template...** browses any configured
ledge or X/Y/D-pad recipe for the current fighter. Left/Right chooses category,
Up/Down chooses source, A applies the whole sequence, B returns. Empty source
categories are skipped. Copy step and Paste step before/after reuse one action
with its settings and before-delay across recipes. Current unsaved
source edits are included. This replaces the destination actions/delays/start
and timeout settings, keeps its name and activation, and leaves the source
unchanged. Undo and explicit Save still apply. Add a new recipe first if you
want to keep the destination's existing version too. No SD changes occur until
immediate Training Save processing. See [template copying](LEDGE_MENU.md#copy-another-recipe-as-a-template).

### Named generic Training templates

Add templates to the universal source using the same action and timing syntax:

```ini
[template.short_shield]
name = Short shield
start_delay = 0
actions = input:zr:0:0:4
```

In the Training editor's step tools, choose **Copy recipe / use template...**,
then **Generic templates** with Left/Right. Select a name with Up/Down and apply
with A; B cancels. Add a new recipe first to retain the original. Applying copies
actions and timing settings while keeping the destination name, activation and
reaction/range conditions. Undo and explicit Save work as usual. Templates are
inert: they are never fighter options, automatic fallbacks or mode bindings.
There are at most 64 templates, within the existing file/section limits. The
lightweight menu has no template editor or hint rendering.

## Reaction metadata

A recipe may include `reaction = roll:grounded:0:20:12:front:1` to attach an
automatic roll event and conditions. The seven fields are trigger, own state,
minimum X distance, maximum X distance, height difference, enemy side and priority.
No reaction key means no automatic rule. Each assist family allows eight rules
(up to 32 per fighter). They appear only in their separate assist lists; lower
priority numbers are checked first. Menu roll rules start OFF; defensive mode
startup switches follow the fighter/universal settings. Existing roll-button
toggle groups remain supported. Training edits/save/copy use the same master file. Actions and move
timings remain in that recipe. See [reaction rules](REACTION_RULES.md).

Reaction activation can be specified as an optional eighth field, e.g.
`reaction = roll:grounded:0:20:12:front:1:toggle_x`. Omitted control retains Menu
behavior. Allowed controls are menu/toggle_x/toggle_y/toggle_up/toggle_down/
toggle_left/toggle_right. Recipes sharing a button form one group. This metadata
reserves that button; it is not a legacy `mode = ...` binding. See
[reaction activation](REACTION_RULES.md#menu-versus-button-activation).

## Per-fighter ledge policy overrides

New builds accept an optional `[fighter.settings]` section, separate from recipes:

```ini
[yoshi.settings]
walk_strength = 0.28
grab_range = 14
horizontal_range = 18
vertical_range = 12
follow_range = 40
follow_vertical_range = 48
use_range_limits = true
manual_spacing = true
```

The [Training fighter-settings page](LEDGE_MENU.md#fighter-settings-in-training)
can edit/save these values. Omitted properties inherit existing configured
values. `walk_strength` is 0.1–0.5; `grab_range` is 1–60. The four range fields
and `stand_distance` accept 1–150 world units; `move_strength` is 0.1–1.0.
`manual_spacing`, `auto_position`, and `use_range_limits` accept `true`/`false`.
Unknown/duplicate keys, duplicate settings sections, non-finite and out-of-range
numbers fail validation. Ordinary recipe saves preserve these sections and
fighter-specific transfers include them.

Walking strength supplies the baseline for the original manual walking curve
during active ledge assistance; full horizontal stick retains the legacy 50% cap. Grab range changes
the `fighter_range` estimate, not hitboxes. Explicit recipe distances/strengths
still take precedence. No settings section changes native physics, moves or
startup timings. Install a matching new plugin before using these sections;
older plugins do not understand this syntax. Existing files without them retain
their previous behavior.

## Validate a file locally

From the project directory, check the default deployment file without FTP:

```sh
python3 tools/sequence_validation.py
```

Check any saved copy, including paths with spaces:

```sh
python3 tools/sequence_validation.py "/path/to/sequences.txt"
python3 tools/upload_sequences.py --file "/path/to/sequences.txt" --check-only
```

These use the current Rust parser, including fighter settings, reaction activation,
recipe syntax, numeric bounds and menu-capacity/default checks. Success reports
recipe and fighter-settings section counts; syntax errors show the line and section
when available. They run a host validator, not an installable mod build.

Preview an upload without connecting, or upload after validation:

```sh
python3 tools/upload_sequences.py --dry-run
python3 tools/upload_sequences.py
```

Default uploads (or explicit `--upload`) write **sequences.upload.txt** on both Switches, preserving active
**sequences.txt**. Restarting does not activate the suffixed copy. `--file PATH`
can also select the source for these commands. New fighter settings syntax requires
the matching updated plugin; file validation cannot verify what plugin is installed.

To replace the active file instead, close Smash on **both** Switches and run:

```sh
python3 tools/upload_sequences.py --replace --smash-closed
```

This validates the selected bytes, rechecks source/destination, saves unique local
snapshots under `.sequence-sync-backups/upload-sequences/`, backs up an existing
active file to `sequences.txt.workbench.bak`, and verifies replacement by read-back.
It reuses the workbench transfer/recovery path; a failed write attempts to restore
and verify the previous file. Each Switch is reported separately. `--replace`
replaces the **whole file**, including all fighters and settings; use Workbench
fighter transfer for a single-character update. `--replace --dry-run` previews
without a Switch connection. `--file PATH` selects the source for either mode.

`stand_distance` supplies the held-binding manual walking stop gap from the known ledge,
and the existing automatic stance offset. The walking-speed/gap limits require
holding the ledge binding even when autopilot is toggled ON; releasing it leaves normal manual
movement available while autopilot continues tracking/responding. Missing anchors
and momentum mean this is an input aid, not a physical barrier. See
[L and the autopilot toggle](LEDGE_MENU.md#l-and-the-autopilot-toggle).

## Facing-dependent ledge recipes

Walk distance, Run distance, Drift and Face offer two explicit anchor-only
**Direction** values: **Toward saved ledge** (`saved_ledge`) and **Away from saved
ledge** (`away_from_ledge`). They use the ledge already retained by the ledge-trap
engine, not the opponent's changing position. Unknown/non-finite anchors end the
sequence instead of guessing. The old `ledge` direction keeps its existing
opponent-position fallback for compatibility. Recipe movement remains independent
of automatic stance positioning.

```ini
[yoshi.roll.ledge_walk_example]
name = Walk away from saved ledge
start_delay = 0
actions = walk:away_from_ledge:4:50%, face:opponent, grab:fighter_range
```

`walk:away_from_ledge:4:50%` walks four horizontal world units at 50% stick input.
`run:saved_ledge:8` runs eight units toward that ledge.
`drift:away_from_ledge:25%:20` requests 25% stick away for twenty native frames.
These are simulated inputs, not position/physics changes.

Under **Conditions / checks**, insert **If facing ledge**, **If back to ledge**
and **End facing block** to put different responses in the same get-up recipe:

```ini
[yoshi.neutral.facing_example]
name = Grab facing ledge / Nair facing away
start_delay = 0
actions = if_facing_ledge, grab:4, end_facing, if_back_to_ledge, short_hop_nair:4, end_facing
```

At the sequence's first observation, remember whether your facing points toward
or away from the saved ledge. Execute only matching blocks; skip the other block,
including its waits, without consuming frames. Turning during one block cannot
enable the opposite block later. Unconditional steps outside blocks always run.
Every opening marker requires `end_facing`; nesting is rejected. Markers count
against the existing 32-action limit. Validation rejects broken blocks before
saving/uploading; invalid in-memory previews cannot execute.

The editor shows conditional timing rather than summing both branches as a fixed
schedule. Missing anchors or ambiguous facing end a sequence that needs a facing
block. On-stage roll reaction rules cannot use these blocks/directions because
they have no ledge anchor. Button recipes/previews also need a valid anchor;
there is no background search or persistent cross-match ledge cache.

Facing the ledge is different from facing an opponent who rolled across you.
For a grab that should simply turn toward the opponent, use `face:opponent`
before `grab:fighter_range`. The facing predicate itself adds no wait, but the
actual turnaround and attack still follow native game input/animation rules.
Install a matching new build before using this syntax; older builds reject it.

Save-as-default releases its source catalog before validating the replacement;
recipe saves release the finished validation catalog before formatting. These
reduce transient save memory without weakening validation. The persistence API
also rejects assist rules as ledge/button startup defaults. These source-level
checks do not establish the cause or resolution of a reported Switch crash;
matching the installed build and latest crash report is still required.

### Ground-only safety

`ground_only = true` ends remaining or held recipe inputs when your fighter is
airborne. It defaults false and is retained by editor saves. Assist editors expose
it as **Stop if airborne**. This does not alter jumps, gravity or recovery; it
only stops simulated inputs. A My state: grounded reaction still filters only
startup unless this flag is enabled. Use the current build before installing a
file containing this property.

### Combined controller input

`controller_input:BUTTONS:LX:LY:RX:RY:FRAMES[:LEFT_REF[:RIGHT_REF]]` sends
buttons and both sticks together. Axes are -1..1, hold is 1..255 frames, and
references are screen/facing/opponent (default screen for both). References only
mirror the horizontal axis, captured at step entry; vertical remains up/down.
Opponent references require the existing locked observation. All ordinary
canonical gameplay buttons can be combined with `+`; `none` is stick-only.
Example: `controller_input:a+b:0.5:0:-1:0:4:opponent:facing`.

Training’s Controller input choice exposes these values and button toggles. The
new token requires an updated plugin. Existing input/facing_input/opponent_input,
C-stick, range:ignore, shield-until-manual and legacy direction tokens remain
readable without changing their inputs/timings.
