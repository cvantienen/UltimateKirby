# Fighter profile and data catalogs

The runtime has 79 registered fighter-profile identities for menu/configuration
behavior. Fighter statistics and move-frame facts are a separate catalog:
`data/Ultimate_stats.json` contains 90 exact
native-kind records across the roster, including DLC fighters.

## One canonical stats source

The core embeds and validates the JSON once at startup. Lookups use the native
fighter kind, regardless of whether that fighter is controlled by the player or
observed as an opponent. Unknown kinds and missing or compound facts remain
unavailable; neither another fighter's row nor a generic numeric default is
substituted.

`fighter_profiles` contains role/profile behavior, while `fighter_data` owns
stats and move-frame access. The profile registry and stats catalog are not
interchangeable, and profile-specific menu configuration does not create a
second source of frame data.

## Input configuration

Ledge-response lists, defaults and input schedules are maintained in
[`sequences.txt`](SEQUENCE_FILES.md). Individual fighter guides describe those
file-authored choices and link their reference facts back to the canonical
dataset. Input sequences simulate controller input only; they do not edit
native moves, physics, hitboxes or fighter status.

## Checks

From the repository root:

```sh
python3 tools/validate_fighter_data.py
python3 tools/update_fighter_docs.py --check
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
```

Update fighter-specific stats and frame facts only in the canonical JSON, then
run the data validator and regenerate/check the fighter pages.
