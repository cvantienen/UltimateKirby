# Compiled fighter ledge responses

All fighter moves, response functions and menu lists are compiled Rust code.
Only shared settings and supported numeric adjustments load from config.txt at
startup. There is no fighter JSON, combo library, or runtime action interpreter.

All non-DLC fighter forms now have their own profiles, including echoes,
Pokémon forms and the three Miis. See [the roster/data audit](NON_DLC_ROSTER.md)
for defaults, frame windows, movement references and resource exceptions. Other
valid unconfigured kinds retain the prior Kirby fallback; invalid identities stay inert and globally
disabled tools stay disabled.

| Source | Role |
| --- | --- |
| `fighter_profiles.rs` | Register/validate fighter move data and response lists once |
| `fighter_profiles/<fighter>.rs` | Register one fighter |
| `fighter_profiles/<fighter>/timing.rs` | Native startup/active windows |
| `ledge_trap/fighters/<fighter>.rs` | Named response handlers and static choices |
| `fighter_profiles/presets.rs` | Saved list/function/preparation types |
| `fighter_profiles/spacing.rs` | Display-only compact names and qualitative spacing hints |
| `ledge_menu.rs` | Five independent saved indices and physical navigation |
| `src/live/ledge_menu.rs` | Cache resolved settings and battle UI revisions |
| `src/live/ledge_menu/menu.rs` | Draw cached labels only |

## Recipes and variants

The flow is:

```text
our fighter kind -> compiled fighter catalog
opponent ledge option -> that fighter’s response list
selected index -> registered Rust response function
compiled timings + numeric adjustments + native observations -> ordinary inputs
```

Kirby’s catalog contains separate functions such as:

```rust
respond_to_neutral_getup_nair(...)
respond_to_neutral_getup_down_tilt(...)
respond_to_neutral_getup_forward_tilt(...)
respond_to_neutral_getup_stone(...)
respond_to_jump_getup_nair(...)
respond_to_roll_down_smash(...)
respond_to_getup_attack_grab_rear_nair(...)
respond_to_getup_attack_grab_rear_shield(...)
respond_to_ledge_drop_shield(...)
respond_to_ledge_drop_down_tilt_then_shield(...)
```

A `SavedChoice` binds an ID/label, response function, and preparation function.
Preparation resolves the selected action’s native-readiness metadata and timing
adjustments when the choice changes. The response function owns the action
sequence. Functions may reuse the same geometry, input, shielding or timing
helpers without duplicating the whole state machine.

For example, a future roll dash attack belongs beside Kirby’s roll down-smash
function, with its own preparation and registration:

```rust
SavedChoice::new(
    "dash_attack",
    "Dash attack",
    respond_to_roll_dash_attack,
    prepare_roll_dash_attack,
)
```

That is an extension example, **not an implemented choice**. Implement and test
the native readiness, input, action confirmation and timing first. No string
in a settings file can point to a new function or bypass that work.

## Tuning and extending

Supported keys in config.txt adjust numeric timing settings on the next game
start; they cannot add functions or fighters. A new response needs a named Rust
handler, its readiness/timing preparation, and a SavedChoice registration in the
appropriate fighter list. A new fighter needs move data, response lists and a
catalog registration. These source changes require an authorized rebuild.

Old combo_* keys are ignored for compatibility with existing SD config files.
They cannot start any action. See [controls](LEDGE_MENU.md),
[the starting cast](STARTING_CAST.md), [Samus variants](SAMUS_LEDGE.md), and
[Sonic variants](SONIC_LEDGE.md), and [K. Rool/Isabelle variants](KROOL_ISABELLE_LEDGE.md), and
[Pac-Man/Little Mac variants](PACMAN_MAC_LEDGE.md).

See [fighter execution and calibration](FIGHTER_EXECUTION.md) for hop policies,
movement ownership, and what remains unmeasured.
