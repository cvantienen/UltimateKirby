# File-authored fighter ledge options

`deployment/ultimate/kirby_assist/sequences.txt` is the sole ledge option catalog.
It contains all five menus for each configured fighter, their labels/defaults,
initial input delays, ordered actions and waits. Rust contains no fighter option
tables or fallback response routines. Removing a section removes that option;
a missing file leaves the ledge menus empty and ledge assist inactive.

<a id="recipes-and-variants"></a>

```ini
[kirby.neutral.nair_wait]
name = NAIR LATE
start_delay = 0
default = false
timeout = 3600
spatial_timeout = 60
actions = Short Hop, wait:5, Neutral Air, Drift:Opponent:25%:20
```

Every section requires `start_delay` (0–3600 native frames): zero means the first
input starts immediately after recognition. Each bare input lasts one frame;
`wait:N` inserts N released-input frames exactly where it appears. Durations can
be authored as `move:N`, `hold:BUTTON:N` or a combined raw `input`. No fighter
startup/active-window metadata or config.txt timing overrides schedule the inputs.
The game still decides whether the requested physical input buffers or executes.
Nothing here edits native move startup, animation speed, physics or hitboxes.

`default = true` selects the initial choice for that fighter/situation. Use at
most one default in each row; without it, the first file entry is selected.
File order is menu order. Existing custom recipes are kept after the migrated
roster recipes. IDs and defaults are validated before hooks install.

The supplied roster entries are explicit input templates, not promises of
identical behavior to the retired conditional engine. In particular, the file
states the 30-frame down-tilt-to-shield template wait, the 22-unit cross-up walk,
and every shield/block timeout. Tune these for your intended timing/spacing.
Native block waiting is explicit as `shield_until_block:FRAMES`; no hidden
fighter routine runs. `code:ID` entries have been removed.

Rust retains shared opponent/ledge recognition, raw controller routing, native
progress timing, optional spatial/block predicates, interruption handling and
session reset. `fighter_profiles` contains fighter identities and independent
reference facts needed by other tools; its move windows do not schedule ledge
inputs. The old config.txt move choice/timing keys are ignored for compatibility.

Run `python3 tools/upload_sequences.py --dry-run` after edits. The SD path is
`sd:/ultimate/kirby_assist/sequences.txt`; restart Smash after changing it. This
migration needs an updated mod build installed before the new file is used.
See [the complete action vocabulary and timing rules](SEQUENCE_FILES.md).

## Defensive startup settings

Training **Fighter settings** includes three additional saved input-policy values:

| Menu setting | Text key | Meaning |
| --- | --- | --- |
| Grab dodge + punish at startup | `grab_dodge_default = true` | Start the mode ON/OFF for this fighter in a fresh match. Falls back to global `auto_spot_dodge`. |
| Out of shield at startup | `out_of_shield_default = true` | Start OOS ON/OFF for this fighter; default ON. No configured response means no OOS observation work. |
| Grab dodge forecast frames | `grab_dodge_lookahead = 6` | Contact forecast window, 1–8 frames; default 6. Larger can dodge earlier. |

Store these under `[bowser.settings]`, `[yoshi.settings]`, etc. Save in Training
with Y and wait for completion. They affect controller policy only. Response
range/height/side, priority, actions and delays belong to the recipe's Auto reaction
settings, so different responses can use different ranges. These settings do not
change native shieldstun, dodge intangibility, movement physics or grab hitboxes.

See [every Fighter Settings option explained in simple terms](FIGHTER_SETTINGS.md),
including what increasing each value does and how acquire/follow differ.
