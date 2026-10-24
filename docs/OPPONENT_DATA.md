# Compiled opponent reference data

The reference snapshot contains **90 native fighter kinds and 4,076 move
entries**, including DLC opponents and separate Ice Climber identities. It does
not add playable automation for those fighters: the first whiff policies are
Kirby and Yoshi only. Data was retrieved on **2026-10-23** from
[Ultimate Frame Data](https://ultimateframedata.com/smash) and its
[fighter statistics](https://ultimateframedata.com/stats.php).

## What is stored

- Move startup, total animation frames, discrete active intervals, landing lag,
  base damage, shield lag/stun, shield advantage and listed endlag.
- Listed autocancel, charge-hold and landing-hit frame facts where parseable.
- Weight, walk/run/initial-dash/air speed, gravity, fall/fast-fall speed, full/short
  jump height and air acceleration as typed optional numbers.
- Additional source-table cells: jump durations, landing, dodge/roll, grab-range,
  ledge and out-of-shield references, retaining compound values as text.
- Per-page source URLs and SHA-256 provenance in the source snapshot.

This is **reference data**, not every engine parameter. It contains no complete
3D hitbox/hurtbox geometry, knockback formula model or exhaustive native motion
mapping. Missing, compound and ambiguous fields remain unknown/raw, never zero
or another fighter's number. For example, the source table lacks a simple
Sephiroth air-speed entry and Byleth gravity entry; these stay `None`.
Baseline stats do not account for instance modifiers, equipment, scaling or
other gameplay mods. Native instance observations take priority in policies.

## Match behavior and performance

Everything is compiled into read-only Rust tables. **No JSON, website, network
connection, SD-card file or directory is read when a match starts.** The source
JSON is a developer snapshot, not deployed runtime configuration.

At plugin startup, `live::install()` calls `load_config()`: it reads the ordinary
`config.txt`, constructs the existing compiled ledge-response catalog once,
validates it and stores it in `PROFILES: OnceLock<FighterProfiles>` before hooks
are installed. A failed configuration/catalog validation aborts installation.
That catalog selects your fighter's compiled responses; it is distinct from the
90-kind opponent reference tables already stored in the plugin binary.

The whiff observer/menu switch are disconnected from the live build. No live
opponent-profile lookup pass runs for that feature. Static reference tables,
exact-kind lookup and bounded cache helpers remain available for future policies
and host tests. Unknown opponents never inherit Kirby reference data. Unused
data may be discarded by the linker; actual NRO/memory cost remains unmeasured.

## Source and maintenance

| File/API | Purpose |
| --- | --- |
| `data/opponent_reference.json` | Reproducible source snapshot; factual fields only |
| `tools/update_opponent_data.py` | Explicit developer-time importer/generator |
| `opponent_data/generated.rs` | Checked-in compiled static facts |
| `opponent_data::profile(kind)` | Exact-kind lookup, no fallback |
| `FighterProfile::reference()` | Existing fighter profile's reference facts |
| `MoveData::recovery()` | Pure validation of a known native recovery window |

Commands from the repository root:

```sh
# Network-free regeneration and drift check:
python3 tools/update_opponent_data.py
python3 tools/update_opponent_data.py --check
python3 -m unittest tools.test_update_opponent_data

# Explicit developer-time public-data refresh; not performed by Cargo/the game:
python3 tools/update_opponent_data.py --fetch
# Refresh only stats without fetching move pages:
python3 tools/update_opponent_data.py --stats-only
```

Review source changes, unknowns and whiff eligibility after any refresh. Never
infer all multihit gaps are active or map an ambiguous move variant by picking
its fastest row. The imported data does not silently replace existing ledge
timings; those remain their current compiled response tuning.
