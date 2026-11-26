# Fighter data: read-only native reference facts

`data/Ultimate_stats.json` is the canonical embedded statistics/move-frame asset.
The core requires schema version **1** and **90 records**; this is separate from
the profile/menu registry. The data is keyed by exact native kind for both player
and opponent. It is not live hitbox geometry, a per-instance physics model, an
input recipe or a remote fallback service.

## Lifecycle in plain Rust

`include_str!` embeds the asset in the program at build time. Editing the SD
sequence file does not change it. `OnceLock<Result<Vec<FighterData>,String>>`
parses once and remembers either the roster or the error. A `Vec` is a growable
list; `Result` means success (`Ok`) or an explanatory failure (`Err`).
`initialize()` validates before live hooks. There is no battle-time JSON read,
network request, generated Rust mirror or other-fighter substitution.

Parsing verifies schema/count, nonempty identity/stats/moves, nonempty stat keys,
positive ordered active intervals, then strictly increasing unique native kinds.
Readable motion names are hashed once. `fighter(kind)` uses binary search and
returns `Option<&FighterData>`: `Some` exact record or `None`. It never resolves
an unknown kind to Kirby. `all()` expects successful initialization and panics
on invalid embedded data; do not call it as an error-recovery fallback.

## What the public fields mean

| Type/field | Interpretation |
| --- | --- |
| `Stat { key, value }` | Named raw string fact; numeric parsing may fail |
| `FighterData { kind, name, stats, moves }` | One exact native identity |
| `MoveData.total` | Optional total animation frames, not always available |
| `landing_lag` | Optional aerial landing lag |
| `active: Vec<Span>` | Separate inclusive first/last hitbox intervals |
| `motions` | Native motion hashes identifying candidate moves |
| `whiff_eligible` | Permission for a narrow recovery estimate, not proof of whiff |
| `raw` | Original named move facts retained as strings |
| `GrabReference` | First/last active frame and optional approximate root reach |

`numeric_stat` only parses a raw string; callers must reject nonfinite/out-of-range
values as needed. `named_move` and `move_for_motion` require exactly one match;
zero or conflicting variants yield `None`. `last_active` finds the latest span
end; it does not join gaps into continuous coverage. For spans 6–7 and 15–21,
frames 8–14 are not active. Missing/compound values are not invented.

## Grab facts and their limits

`prepare_grab_references()` derives a compact exact-kind cache once. `ordinary_grab`
accepts only standing/dash/pivot grab statuses. Before cache initialization it
uses the same uncached lookup. Matching variants with different active intervals
are ambiguous and return `None`; validated first/last values must lie in 1–120.
Root reach uses a finite numeric `grabs.grabrange` in 1–60 when supplied. Unknown
reach becomes 0 in a profile's display/default field, not a fabricated catch range.
Shield-grab startup similarly needs a valid 1–120 numeric reference.

These facts estimate root positions, not animated hands/hurtboxes. In the current
ordinary-grab helper, standing/dash/pivot share the available numeric reach field;
do not invent distinct dash reach from a fighter-page label. Tuning's automatic
far-range formula uses returned reference values and never resolves far below close.
An explicit fighter override changes policy estimates only.

## Recovery estimates are deliberately narrow

`MoveData::recovery(observation)` handles only recognized grounded dash attack,
forward/up/down tilt and forward/up/down smash. Status/motion must match; the move
must be whiff-eligible, vulnerable, past all active frames, not currently active,
not already contacted and not cancelable. Observed frame/end must be finite;
end must agree with catalog total within two frames. Remaining native progress
must be positive. Failing any condition yields no estimate. This API existing
does not mean a retired automatic whiff-punish mode is active.

`FighterDataCache` has eight controller slots. `bind(slot,id,kind)` reuses a row
only when both object ID and kind match; identity changes resolve afresh and
unknown kinds clear the entry. Invalid slots return `None`. It stores copied
identity/fact references, not a native mutable fighter pointer.

## Update or extend

Only change canonical records with reviewed fighter-specific facts and explicit
permission. Preserve kind order, unique identities, raw uncertainty and separated
active spans. A new public helper should return `Option` when facts are ambiguous;
never choose an arbitrary variant or another fighter's values. Add focused
lookup/variant/cache tests, then validate the data and regenerate fighter snapshots.
New records need coordination with profile validation; the count constant alone
cannot register a fighter.

Authorized offline commands are `python3 tools/validate_fighter_data.py` and
`python3 tools/update_fighter_docs.py --check`. Generator tests verify tables
against the canonical asset. Host checks establish schema/reference consistency,
not Switch input delivery, hitboxes or online contact. This documentation update
reads assets only; it neither edits them nor builds/deploys a new embedded catalog.
