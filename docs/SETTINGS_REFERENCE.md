# Settings and modes: a beginner's configuration guide

This guide explains **where a setting belongs, when it takes effect, and how to
change or extend it safely**. You do not need to know Rust to edit a saved text
configuration. You do need a new plugin build to change executable Rust code;
editing a source file does not update an already installed plugin.

Kirby Assist requests controller input. A setting can change a prediction,
selection, or requested stick/button; it cannot change native hitboxes, physics,
move speed, invincibility, damage, or whether Smash accepts the request.

## 1. Understand the four configuration layers

| Layer | Where it lives | What it controls | When changes take effect |
| --- | --- | --- | --- |
| Shared startup settings | `[universal.settings]` in `SD:/ultimate/kirby_assist/sequences.txt` | Controller selection/layout, master installation, shared mash/tech/DI/SDI preferences, legacy ledge defaults | Restart Smash after a text edit |
| Fighter settings | `[kirby.settings]`, `[bowser.settings]`, and equivalent sections in the same file | Fighter-specific predictions, reaches, startup switches and supported helper tuning | Text edit: restart. Training save: wait for successful save/publication; see each setting's live/startup distinction |
| Saved recipes and assist rules | `[fighter.situation.choice_id]` and `[template.id]` | Literal input tapes, per-button choices, ledge responses and event-triggered actions | Text edit: restart. Training editor: preview/edit in memory, then save explicitly |
| Match-session choices | Menu ON/OFF rows and selected choices | Current match's active tools and selected responses | Immediate when admitted by the current controller/menu state; ordinary transient choices reset with session cleanup |

**The shared section is not merged with `config.txt`.** The loader first parses
`sequences.txt`. If it contains `[universal.settings]`, that section is the shared
configuration and `config.txt` is not read. If it does not, the loader tries
`SD:/ultimate/kirby_assist/config.txt`. If that file is also missing, Rust fallback
defaults are used. Missing `sequences.txt` means no saved recipe catalog; there is
no compiled ledge-response tape that silently replaces it.

The file supplied with this checkout and Rust's fallback defaults are different.
For example, the supplied shared section enables Auto-tech, DI and SDI, while the
Rust fallback defaults disable all three. Your installed file can differ from
both. A fighter override or current match toggle can change the effective result
again. Always inspect the actual file and current menu instead of assuming that
“default” means one universal value.

## 2. Make a first safe text edit

1. Close Smash before replacing its active configuration file. Keep a backup of
   the file you intend to edit, not only a repository copy.
2. Find the correct section. Shared settings go under `[universal.settings]`;
   fighter tuning goes under `[fighter.settings]`. A recipe's `actions` does not
   belong in either settings section.
3. Change one setting. Use exact keys and ordinary `true`/`false` values. Keep
   unrelated sections and comments.
4. Validate the file using the existing sequence validator described in
   [Sequence files](SEQUENCE_FILES.md#validate-before-installing).
5. Install only the configuration change using your normal reviewed workflow,
   then restart Smash. This guide does not require a plugin rebuild for text
   changes.
6. Test a repeatable situation in Training. Compare the current mode's ON/OFF
   state, the intended input, and actual native acceptance separately.

A minimal shared section for learning without optional damage-stick assistance:

```ini
[universal.settings]
enabled = true
player_index = 0
control_layout = default
auto_mash = false
auto_tech = false
auto_di = false
auto_sdi = false
```

This is a **complete shared settings source**, not a patch merged into
`config.txt`. Omitted keys use Rust defaults. It does not define ledge or button
recipes. Add them separately using [Sequence files](SEQUENCE_FILES.md).

## 3. Units and vocabulary

- **Frames:** a duration or deadline interpreted by the consuming policy. Native
  animation progress, a simulation frame ID, mapper updates and wall-clock
  milliseconds are not interchangeable. See [Observations and timing](OBSERVATIONS_AND_TIMING.md).
- **World units:** distances between observed reference positions. Increasing a
  reach estimate permits farther attempts; it does not lengthen a real grab.
- **Stick strength:** normalized input magnitude. `0.35` is partial stick input,
  not a 35% modification to the game's movement speed.
- **Startup default:** the value used to initialize a new session. It is not
  necessarily the current match's toggle.
- **Admission:** the checks that decide whether an enabled tool may propose
  input now. ON is permission to consider a tool, not an instruction to attack
  every frame.
- **Ownership:** a selected input request can reserve specific controls or a
  release frame. Other tools do not simply add all their buttons together.
- **Legacy/retired:** accepted for old-file compatibility. A parsed key or token
  can exist without activating a current gameplay feature.

## 4. Shared settings reference

The following keys are accepted by the current `LiveConfig` parser. Values in
“Rust fallback” apply only when a key is absent from the chosen shared source.
“Supplied” describes the repository's current master shared section, not a
promise about a user's installed SD card.

### Installation, routing and input layout

| Key | Rust fallback | Supplied | Accepted value / meaning |
| --- | --- | --- | --- |
| `enabled` | `true` | `true` | Boolean. `false` prevents startup hook installation; changing it requires restart. The outside-battle runtime gesture is a separate temporary switch |
| `player_index` | `0` | `0` | Integer `0..7`: physical local controller identity, not fighter-manager position or Arena participant number. `npad_id` is an alias |
| `control_layout` | `default` | `default` | `default` or `brother`, case-insensitive values. Selects translation of canonical assisted inputs; does not alter Smash's saved control profile |
| `takeover` | `false` | `true` | Boolean. Allows the active raw mash/tech helper to release relevant physical controls. It does not make every feature own every input |
| `sequence_input_delay_frames` | `0` | omitted → `0` | Integer `0..120`. Explicit sequence queue compensation; no automatic network-delay measurement or guaranteed online synchronization |

Use [Controls](CONTROLS.md#controller-mapping) before changing layout. A valid
configuration cannot verify that the game's C-stick is set to Tilt Attack.

### Mash, tech and damage-stick assistance

| Key | Rust fallback | Supplied | Accepted value / meaning |
| --- | --- | --- | --- |
| `auto_mash` | `false` | `false` | Boolean. Enables analog-direction requests in supported native clatter states; not ordinary attack-button spam |
| `auto_tech` | `false` | `true` | Boolean. Enables collision prediction and eligible Guard-edge scheduling; does not force a tech |
| `tech_debug` | `false` | `true` | Boolean. Extra diagnostic evidence where logging is compiled in. Silent builds do not become diagnostic through this key |
| `tech_cooldown_frames` | `40` | `40` | Unsigned 16-bit integer `0..65535`. Parser storage limit, not the game's native lockout; lowering it cannot bypass eligibility |
| `tech_lookahead_frames` | `3` | `5` | Finite number `1..5`. Prediction horizon used by native collision queries, not a guaranteed successful-tech window |
| `auto_di` | `false` | `true` | Boolean. Enables automatic DI preference; fighter `auto_di_default` can override it |
| `auto_di_mode` | `survival` | `survival` | Exact lowercase `survival`, `adaptive`, or `escape`. Adaptive is retained compatibility behavior; consult the current [DI/SDI guide](DI_SDI.md) |
| `auto_di_survival_speed` | `3` | `3` | Finite number `0.1..30`. Survival-mode threshold/tuning, not native velocity modification |
| `auto_sdi` | `false` | `true` | Boolean. Enables supported SDI stick requests; fighter `auto_sdi_default` can override it |
| `auto_spot_dodge` | `true` | `true` | Boolean. Shared grab-dodge startup preference; fighter `grab_dodge_default` can override it |

Read [Auto-tech](AUTO_TECH.md), [Analog mash](ANALOG_MASH.md),
[DI/SDI](DI_SDI.md), and [Grab dodge](GRAB_DODGE.md) for gates, timing,
manual-input interaction and tuning trade-offs. The integer parser accepting a
value is not evidence that every extreme is useful.

### Physical tilt and Kirby follow-ups

| Key | Rust fallback | Supplied | Accepted value / meaning |
| --- | --- | --- | --- |
| `auto_turn_tilts_active` | `false` | omitted → `false` | Boolean for the shared physical-tilt correction path. Historical misspelling `auto_turn_tilit_active` is accepted |
| `auto_turn_tilts` | all directions OFF | omitted → all OFF | Directions separated by commas, spaces or `\|`: `down`/`d`/`dtilt`, `forward`/`side`/`f`/`ftilt`, `up`/`u`/`utilt`; `none` or `off` disables. Empty or unknown selections are errors. Distinct from fighter `tilt_turn_*` settings |
| `down_tilt_trip_smash` | `true` | `true` | Boolean. Permits the supported confirmed-trip follow-up path, not a guaranteed trip or hit |
| `kirby_fair1_smash` | `true` | `true` | Boolean. Permits Kirby's first-Fair-hit follow-up logic; later Fair hits are not equivalent triggers |

The fighter menu's Tilt face switch and directional overrides are a separate
configuration layer. Use [Fighter settings](FIGHTER_SETTINGS.md) for current
per-fighter controls and [Kirby Fair confirm](KIRBY_FAIR_CONFIRM.md) for evidence
and cancellation requirements.

### Legacy shared ledge and turbo base settings

These keys supply base ledge settings. Fighter `[fighter.settings]` values and
saved response choices remain separate. There is no shared `actions` field.

| Key | Rust fallback | Supplied | Accepted value / meaning |
| --- | --- | --- | --- |
| `autopilot_shield` | `true` | `true` | Boolean. Permits approximate incoming-attack defense during effective ledge assistance |
| `ledge_trap` | `false` | `true` | Boolean. Base ledge enablement; held helper activation and current menu state also participate |
| `ledge_drop_defense` | `true` | `true` | Boolean. Enables the selected generic drop response; not a native invincibility change |
| `ledge_trap_range` | `12` | `20` | Finite `1..20` horizontal acquisition distance |
| `ledge_trap_vertical_range` | `10` | `10` | Finite `1..20` vertical acquisition distance |
| `ledge_follow_range` | `40` | `40` | Finite `1..100` horizontal retention distance |
| `ledge_follow_vertical_range` | `48` | `48` | Finite `1..100` vertical retention distance |
| `ledge_distance_agnostic` | `true` | `true` | Boolean. Disables the base acquire/follow distance limits when true; fighter `use_range_limits` expresses the inverse preference |
| `ledge_manual_spacing` | `true` | `true` | Boolean. Held walking-cap path; distinct from the Auto-Spacing & Attack built-in |
| `ledge_walk_strength` | `0.35` | `0.35` | Finite `0.1..0.45` partial-stick walking baseline |
| `ledge_auto_position` | `true` | `false` | Boolean. Legacy ledge stance positioning, subject to other mode/range/manual-spacing gates |
| `ledge_stand_distance` | `8` | `10` | Finite `2..15` world units from the known ledge stance |
| `ledge_move_strength` | `1` | `0.65` | Finite `0.1..1` positioning stick strength |
| `ledge_grab_button` | `R` | `R` | `R`, `L`, or `ZL`, case-insensitive. Canonical grab mask for supported older helper callers; explicit tapes have their own button masks |
| `ledge_throw_outward` | `false` | `false` | Legacy accepted preference; capture hands control back rather than executing a universal auto-throw |
| `turbo_walk_strength` | `0.45` | `0.35` | Finite `0.05..0.6` facing creep strength for the assigned Turbo t-bag helper |

Acquisition means finding an opponent; following means retaining that opponent.
Neither changes the range in an explicit `range:...` recipe step. See
[Ledge trap](LEDGE_TRAP.md) and [Turbo controls](TURBO_CONTROLS.md).

### Compatibility-only keys: do not tune these for new behavior

Accepted, type-validated but retired switches include `kirby_auto_duck`,
`dpad_b_reverse`, `dpad_slide`, `r3_turbo`, `l3_turbo`, and `dpad_turbo`. Turbo is
activated through its selected button built-in, not those old switches.

The legacy numeric parser also validates these ignored values:

| Key | Accepted compatibility value |
| --- | --- |
| `ledge_roll_dash_range`, `ledge_roll_smash_range` | finite `1..20` |
| `ledge_tilt_range`, `ledge_grab_range` | finite `1..20` |
| `ledge_roll_distance` | `auto` (case-insensitive) or finite `2..100` |
| `ledge_anti_air_range` | finite `1..20` |
| `ledge_anti_air_height` | finite `1..40` |
| `turbo_phase_frames` | unsigned integer `1..10` |

Other retired families are accepted and ignored: `combo_enabled`,
`combo_starter`, `combo_max_start_percent`, `combo_grab_range`,
`combo_fair_range`; `mash_interval_frames`, `alternate_mash`,
`mash_buttons_per_frame`; `ledge_nair_hit_lead_frames`,
`ledge_stone_hit_lead_frames`, `ledge_stone_startup_frames`,
`ledge_drop_request_timeout_frames`, `ledge_roll_hit_window_frames`,
`ledge_grab_buffer_frames`, `ledge_getup_attack`; `ledge_jump_response`,
`ledge_fair_range`, `ledge_fair_vertical_range`, `ledge_fair_delay_frames`,
`ledge_fair_lead_frames`, `ledge_fair_jump_squat_buffer`,
`ledge_fair_buffer_frame`.

Accepted does not mean active. For new files, use explicit recipes and current
fighter settings rather than copying obsolete tuning blocks.

## 5. Which guide covers each mode?

| Feature or menu concept | How it is configured | Detailed guide |
| --- | --- | --- |
| Ledge trap / five ledge choices | Match toggle or fixed/assigned ledge helper, saved ledge recipes, fighter stance/ranges | [Ledge trap](LEDGE_TRAP.md), [Ledge menu](LEDGE_MENU.md) |
| Auto-tech | Shared startup preference and current mode; fighter collision probes | [Auto-tech](AUTO_TECH.md) |
| Auto-mash | Shared preference and current mode; takeover affects physical inputs | [Analog mash](ANALOG_MASH.md) |
| DI / SDI | Shared preferences and fighter startup overrides | [DI/SDI](DI_SDI.md) |
| Idle crouch / Tilt face | Supported posture mode and fighter directional settings | [Input recipes](INPUT_RECIPES.md) |
| Roll reactions / Spot punish / Shield pressure | Enabled assist families and saved reaction sequences | [Reaction rules](REACTION_RULES.md) and feature guides |
| Grab dodge + punish | Startup/current switch, incoming-grab estimates and optional after-dodge recipe | [Grab dodge](GRAB_DODGE.md) |
| Out of shield | Startup/current switch and supported saved shield responses | [Out of shield](OUT_OF_SHIELD.md) |
| Auto-Spacing & Attack | Assign hold/toggle built-in and tune fighter settings | [Auto-spacing](AUTO_SPACING.md) |
| Counter | Assign hold/toggle built-in; supported fighter counter facts and tuning | [Auto-counter](AUTO_COUNTER.md) |
| Auto Defense | Assign hold/toggle built-in; incoming-attack defense tuning | [Auto Defense](AUTO_DEFENSE.md) |
| Projectile reflect | Supported fighter switch/reflect settings; no generic universal counter recipe | [Projectile reflect](PROJECTILE_REFLECT.md) |
| Ledge trump | Assign its button built-in; not the retired ledge-trump mode | [Ledge trump](LEDGE_TRUMP.md) |
| Turbo t-bag | Assign its button built-in and set creep strength | [Turbo controls](TURBO_CONTROLS.md) |
| Training test / diagnostics | Training editor controls; diagnostic build-only evidence where indicated | [Training monitor](TRAINING_MONITOR.md), [Calibration](CALIBRATION_CARD.md) |

The built-in token `normal` means **Short hop**, not a separate physical
passthrough entry. Empty slot fallbacks use L3 Auto Defense, D-pad Down Turbo,
other slots Short hop, and a supported Counter fallback for empty R3; saved
explicit defaults and the first manual recipe take precedence.

The menu can contain compatibility identities that are no longer offered as
normal configurable button modes. Do not infer support merely from an enum or
parser token. The current menu, parser validation and live admission path must
all agree.

## 6. Save from Training without losing work

Use [Ledge menu](LEDGE_MENU.md) for exact page-specific controls. The general
workflow is: choose the fighter and row, edit one value or tape, test a safe
repeatable case, request Save, and wait for the completion message. A busy worker
rejects additional save requests rather than accumulating unlimited snapshots.
A displayed edit and a successful disk save are different states.

Save a recipe and save it **as startup default** when you want it selected in a
new session; ordinary current-match selection alone is not a permanent default.
Save fighter tuning for numeric/boolean settings. Do not assume editing a
`*_default` setting immediately flips every related live switch. Keep Smash open
until the save completes. Do not overwrite the SD file from a computer while an
in-game save is working; conflict/backup recovery checks are not a substitute
for avoiding concurrent edits.

## 7. Extend a setting in Rust: the beginner checklist

Rust source is divided into a **portable core crate** and a **native adapter**.
A crate is a compilation unit; a module groups related code. A `struct` stores
named values, an `enum` represents alternatives, `Option<T>` means a value can
be absent, and `Result<T, E>` means an operation can succeed or report an error.
See [Development](DEVELOPMENT.md) for worked code-reading and test tutorials.

For a new shared setting:

1. Identify the policy that will consume it. Do not add a checkbox with no
   behavior behind it.
2. Add a correctly typed field and deliberate fallback default to `LiveConfig`
   or its appropriate nested settings type in core `src/live_input.rs`.
3. Add an exact text-key arm to `LiveConfig::apply`; keep existing serialized
   keys unchanged. Add finite/range validation to `validate` where needed.
4. Wire the native publication/consumer so a parsed value reaches the policy.
   Avoid file parsing, allocation, blocking I/O or new native queries per frame.
5. If editable in the menu, update the relevant menu row, value binding,
   presentation and save serialization. Parsing alone does not expose a menu.
6. Add focused parser and policy tests. Private tests remain external unit-test
   files declared at their original private module scope; public contracts use
   integration tests.
7. Update the example configuration and this reference with units, bounds,
   default, activation conditions and trade-offs. Do not change installed assets
   or build/upload a Switch artifact merely to document the source change.

For fighter tuning, follow [Fighter additions](FIGHTER_ADDITIONS.md) and
[Fighter settings](FIGHTER_SETTINGS.md): its indexed key/default/bounds/menu/save
contracts must stay aligned. For a new action or mode, follow the distinct
[sequence extension](SEQUENCE_FILES.md) and [reaction extension](REACTION_RULES.md)
checklists. Reuse an existing policy contract instead of exposing private
internals merely to make a test or UI field convenient.

## 8. Diagnose a setting that seems to do nothing

- Verify the **active SD file**, section and exact spelling, not just a local
  source copy. Shared `[universal.settings]` may shadow `config.txt`.
- Distinguish a stored startup default from a current match switch.
- Confirm the controlled fighter supports that helper and the assigned button
  is not currently consumed by visible-menu navigation.
- Check admission: grounded/readiness/target/clock/visibility/range/ownership
  gates can correctly suppress an enabled feature.
- Check whether the key is compatibility-only or whether a missing recipe
  leaves the tool with no action to execute.
- Verify actual controller mappings and separate requested input, mapped input,
  native startup and a real hit/capture/tech outcome.
- For source changes, verify the installed artifact was rebuilt separately.
  Host tests cannot prove Switch ABI compatibility or online timing.
