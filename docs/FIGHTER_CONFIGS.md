# Configure and extend fighter responses

## The two layers

`fighter_profiles.rs` registers 90 exact native identities. Each profile initially
has **empty menus** and reference-derived grab facts. The startup sequence catalog
supplies labels, order, selected defaults and literal controller timelines.
There is no compiled fallback recipe; missing selected inputs produce no response.
`fighter_tuning.rs` resolves policy overrides, not move definitions.

Repository source files are `deployment/ultimate/kirby_assist/fighters/<name>.txt`
and the shared master `sequences.txt`; assembly creates the active catalog. These
are configuration assets, not Rust modules. The active SD path is
`sd:/ultimate/kirby_assist/sequences.txt`. Reading a repository snapshot does not
prove which catalog is installed. [Sequence workflow](SEQUENCE_FILES.md) explains
safe staging/replacement and persistence; do not upload while editing a draft.

## Change an existing option, step by step

1. Find the fighter's registered **name**, not its display nickname. Captain
   Falcon uses `captain_falcon`; DK uses `donkey_kong`; K. Rool uses `krool`;
   Pac-Man uses `pacman`; Little Mac uses `littlemac`. Per-fighter pages print
   the exact section prefix.
2. In Training open Minus and choose the fighter/row or button binding. Select
   the desired recipe. Read its **actions**, not just its label: a saved name
   can intentionally differ from what the tape requests.
3. Edit start delay and actions. `start_delay=0` starts immediately after
   recognition; a wait belongs at the point where a release/delay is intended.
   A move's native startup is not automatically inserted into the tape.
4. Add range/facing/airborne guards only when required. A failed predicate may
   wait or cancel according to its action; use the documented instant variant
   when stale execution would be unsafe. An explicit `range:20` does not inherit
   `grab_range`, and tuning cannot rewrite it.
5. Save with Y alone and wait for success. A menu draft is not automatically
   persistent. X reset uses the configured startup value. Changing external
   text requires a restart; current in-menu edits and future startup defaults
   are distinct. See [menu](LEDGE_MENU.md).
6. Test from both sides at repeatable spacing. A one-frame button request is
   not proof of native acceptance, and a projectile input is not projectile arrival.

<a id="recipes-and-variants"></a>

## Add a file option

A small example, not a supplied default or guaranteed punish:

```ini
[kirby.neutral.my_nair]
name = My delayed Nair
start_delay = 2
default = false
timeout = 240
spatial_timeout = 60
actions = Short Hop, wait:1, Neutral Air, Drift:Opponent:25%:8
```

Use a unique ID. `start_delay` is required (0–3600 native frames). File order is
menu order; at most one **regular** recipe can be the default in a row. Without
an explicit default, the first regular recipe wins. Active reaction records are
not manual defaults. Each preset slot supports at most 32 regular choices;
button slots append built-in/custom templates separately. Do not count those
appended templates as file-authored ledge choices.

For button defaults, an explicit saved selection wins, then the first manual
recipe, then the builtin fallback. An empty L3 falls back to Auto Defense;
D-pad Down to Turbo; other empty buttons to Short hop, except supported empty
R3 uses Counter. The legacy serialized builtin value `normal` displays **Short
hop**; it is not a separate passthrough option. Builtin roles and current bindings
are documented in [Controls](CONTROLS.md) and the settings reference.

A bare move/button input lasts one frame; `wait:N` releases N frames. Adjacent
inputs sharing a button keep it held, so insert a release for a new press edge.
All hop, attack, drift and hold durations remain explicit and Smash decides
whether inputs buffer or execute. Neither animation-rate edits nor native
status/hitbox changes are permitted. [Syntax reference](SEQUENCE_FILES.md) covers
raw controller axes, motion-relative directions, waits and bounded predicates.

## Add a current automatic reaction

Current simple events are enemy roll, spot dodge, air dodge, shield grab, shield
backup and after dodging an enemy grab. There is one configured sequence per
event/fighter, not a priority-ranked list of alternatives. `auto_grab` is retired
and rejected; do not copy it from an old guide. Automatic-grab admission requires
your shield; generic unshielded approach grabs are removed.

```ini
[yoshi.roll.my_roll_response]
name = Roll starter
start_delay = 0
reaction = auto_roll:30:12
actions = check:enemy_in_front:0, range:fighter_range_instant, press:r:4
```

30/12 are explicit horizontal/height limits. The example does not inherit a
compiled roll destination or guarantee a catch. Current simple editor fields
are max X and height; legacy rule metadata is not the same configuration model.
Keep ledge Roll and ordinary onstage roll distinct. [Reactions](REACTION_RULES.md),
[grab avoidance](GRAB_DODGE.md) and [automatic grabs](OUT_OF_SHIELD.md) document
admission and ownership. Enabling a mode without a configured response cannot
create an attack.

## Configure fighter policy

Use `[fighter.settings]` for exactly the 83 accepted keys in
[Fighter settings](FIGHTER_SETTINGS.md). Booleans are literal true/false;
numbers must be finite/in bounds. Menu increments are not file quantization.
Omitting a key means fallback; zero is not a general reset. Body/scan/probe
values estimate observations, while a move selector selects an input pattern.
Startup preferences and current switches are separate.

## Extend the Rust code

A Rust beginner can read `FighterProfile` as a struct bundling identity, grab
facts and menu lists. `Option<T>` means present (`Some`) or unavailable (`None`);
unknown data must remain unavailable, not guessed. `BTreeMap<i32,FighterProfile>`
indexes by exact native kind; `SavedChoice` holds an ID/name and validated tape.
The constructor macro creates the same profile shape for each registered fighter.

For a genuinely new identity:

1. Verify its pinned native kind and add an explicit constant/constructor entry.
   The profile builder must not borrow another fighter's data or options.
2. Coordinate the canonical data record and supported response implementation.
   `validate()` checks exact identity/data agreement, registered implementation,
   grab facts, choice IDs/defaults and list limits. Missing/mismatched facts fail.
3. Add configuration under the exact profile name. A constructor alone leaves
   all menus empty. Preserve form/partner selection isolation.
4. Add focused profile/lookup/parser fixtures for known, unknown and swapped kinds.
   Never change native physics, move definitions, animation rates or fighter status.

For an existing fighter, prefer new recipes over new Rust handlers. A new shared
input primitive belongs in the executor/parser/editor with matching tests and
save/serialization support; a new counter/reflector needs a verified native
move/resource adapter and eligibility/receipt/cancellation tests. A tuning key
alone cannot implement it. Append stable tuning fields rather than reordering
saved indices; see the schema extension checklist in Fighter settings.

## Validation boundaries

With authorized asset access, `python3 tools/update_fighter_docs.py` refreshes
all generated snapshots and `--check` detects drift;
`python3 tools/validate_fighter_data.py` checks the canonical data schema.
The generator is static, does not execute Rust and is not a native-match validator.
Use focused host parser/runner tests for behavior changes, not a Switch build
unless separately requested. Device delivery, native acceptance, resource setup,
spacing and online timing remain calibration work.
