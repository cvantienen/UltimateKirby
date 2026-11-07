# Fighter Data

[`data/Ultimate_stats.json`](../data/Ultimate_stats.json) is the single source
of truth for fighter statistics and move frame data. It contains 90 exact
native-kind records, including DLC fighters and both Ice Climbers identities.
Data lookup is keyed by native fighter kind, not by whether the fighter is
controlled by the player or observed as an opponent.

## Data lifecycle

The core embeds this JSON file with `include_str!`, parses and validates it once
at startup, and retains the parsed records for the lifetime of the process.
There is no generated Rust copy, runtime file/network access, remote fallback,
or substitution of one fighter's values for another's. Invalid or incomplete
embedded data stops plugin installation with an explicit error.

`fighter_data::all()` exposes the roster, and `fighter_data::fighter(kind)`
returns the exact row or `None` for an unknown native kind. Each row contains
the raw per-fighter stats and every move record in the file; active-frame spans,
landing lag, motion identifiers, and raw source fields are retained. Compound
and missing values stay as supplied/unknown rather than becoming guessed
numbers. A feature must decline a decision when the required fact is absent or
ambiguous.

`FighterProfile::fighter_data()` resolves the same row for a profile. Fighter
profile initialization and reference-hint rendering use this lookup, so
player-controlled and opponent fighters share the same data path. Native
observations such as current position, motion, facing, and readiness remain
authoritative for live state.

The dataset currently has 4,076 move records. It is not a complete hitbox,
hurtbox, collision, or per-instance physics model. Numeric reference stats do
not include stage conditions, equipment, scaling, or other gameplay mods.
Values unavailable in the JSON are intentionally not supplied by defaults.

## Files and checks

| File/API | Responsibility |
| --- | --- |
| `data/Ultimate_stats.json` | Canonical roster stats and move-frame facts |
| `crates/kirby_assist_core/src/fighter_data.rs` | Parse, validate, and expose exact-kind data |
| `tools/validate_fighter_data.py` | Offline schema and roster validation |
| `FighterProfile::fighter_data()` | Resolve the canonical row for an initialized profile |
| `MoveData::recovery()` | Validate a recovery observation against that fighter's move data |

Run from the repository root:

```sh
python3 tools/validate_fighter_data.py
python3 -m unittest tools.test_validate_fighter_data
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
```

Missing motion mappings, unavailable spans, compound values, and duplicate move
variants are not inferred from another fighter or selected arbitrarily. Update
the canonical JSON only with reviewed fighter-specific facts, then run all
three checks.
