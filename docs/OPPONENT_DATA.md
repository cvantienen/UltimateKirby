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

At plugin startup, `live::install()` calls `load_config()`: it reads and validates
the master `sequences.txt` (legacy `config.txt` fallback). `PROFILES` holds the
identity/reference registry and shared editor templates; a separate compact
indexed library holds saved recipes. Only the selected fighter's owned menus and
rule cache are expanded. A failed configuration/catalog validation aborts installation.
That catalog selects your fighter's file inputs and delays; it is distinct from the
90-kind opponent reference tables already stored in the plugin binary.

The importer verifies native identity/name pairs, including Captain Falcon at
kind 11 and Jigglypuff at kind 12. Host checks compare reference identities for
every registered playable profile so another fighter's grab/movement facts
cannot silently be substituted by an incorrectly ordered source list.

The whiff observer/menu switch are disconnected from the live build. No live
opponent-profile lookup pass runs for that feature. Static reference tables,
exact-kind lookup and bounded cache helpers remain available for future policies
and host tests. Unknown opponents never inherit Kirby reference data. Unused
data may be discarded by the linker; actual NRO/memory cost remains unmeasured.

Ordinary standing/dash/pivot grab reference values are validated and prepared
once before gameplay hooks are installed. The fixed table allocates no heap
memory and avoids repeated move-list scans and numeric parsing during a grab.
It contains exactly the same immutable compiled values; opponent motion, current
positions, velocity, facing, transitions and detection frequency remain live.
Standalone core callers without startup preparation retain the original lookup.

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
timings; input scheduling now comes exclusively from `sequences.txt`.

## Fighter-specific values and remaining estimates

Ordinary grab detection uses each attacker's compiled standing/dash/pivot active
windows and grab reach; post-grab recovery uses the same variant-specific window.
Own grab timings/reach and move windows use the selected fighter's profile. Native
movement observations remain authoritative; saved fighter overrides take precedence.
Rosalina's shield-grab reference is 11 frames. Shared warning horizons, stop gaps
and input strengths are policy settings, not imported physical fighter attributes.
The bundled reference has no hurtbox dimensions, ECB probe geometry or general
smash-hitbox world reach. Those editable estimates are retained without replacing
them with guessed fighter-specific numbers. Standing grab reach remains an estimate
for dash/pivot geometry where separate reach measurements are unavailable.

Startup profile construction now resolves each supported fighter's simple
standing/dash/pivot grab startups, standing active duration/reach, and plain
numeric post-shieldstun grab startup directly from its own reference row. Saved
fighter overrides apply afterward. Compound/missing facts keep an authored
same-fighter value; another fighter's row is never substituted. This does not
replace authored move-part windows or unavailable body geometry with guesses.
The lookup runs during profile construction, not in the per-frame input path.

The fixed native roll/spot/far-roll punish paths are retired. Their generic
12/21-unit smash estimates no longer enter live calculations. Current roll and
spot punishes use the selected fighter's saved rule conditions and input tape;
intentional Kirby/Yoshi-specific helpers remain explicitly fighter-gated. The
inert `LedgeSettings` fixture default is replaced by the exact selected profile
before live observations; invalid identities remain disabled.

Startup validation also rejects a supported profile with missing/mismatched
reference identity. An unavailable fighter is not silently assigned Kirby data.
