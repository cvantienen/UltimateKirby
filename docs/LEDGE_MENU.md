# Saved ledge menu

Hold **L** for ledge autopilot. Tap **Y** for a neutral-B reverse attempt;
press **X** for a paired X+ZL short hop (both mapped to Jump). Hold **R3 alone**
for fast crouch/walk-cancel turbo teabag. **Hold L and click R3** to show/hide the menu. **Minus**
toggles auto-mash once per press, whether the menu is open or closed. While visible, **Up/Down**
selects one of five rows (Neutral, Jump, Roll, Attack, Drop), and **Left/Right**
cycles its saved responses. While hidden, Left/Right slides, Down starts ledge
trump, and Up is normal. Directions held when closing must be released before
starting gameplay tools. L+R3 is reserved for menu toggling and does not teabag.
Hold **L3 as Kirby** for [down-tilt recovery cover](TWO_FRAME.md); release to stop.
Auto-mash initially uses the startup configuration (enabled in the supplied
config). Its toggle is shared across fighters and survives stocks and matches
until the plugin reloads. A nonselectable `Auto-mash: ON/OFF` line shows the saved
state. This cached line adds no live status queries. Disabling mash immediately
releases cached mash stick/button takeover while preserving auto-tech.

The title-free panel stays in the bottom-left corner, shows the selected row in
gold, and displays option counts. It starts visible in each battle, remains
through stock loss until you hide it, and closes outside battle. Menu navigation
also works during entry/countdown once the local fighter is known.

The panel uses a 360-pixel reference width at 720p, down from 410, and text size
19 rather than 17, scaled with display height. A small offset text pass gives a
heavier appearance without loading another font. Row/count columns are narrower;
long saved choices wrap within the option column.

Common move labels are abbreviated: `Dtilt`, `Ftilt`, `Utilt`, `Dsmash`, `Fsmash`,
`Fair`, `Uair`, `SH Zair` (short-hop zair), and `Homing Msl` (homing missile).
`Shield Nair` means shield then nair; `Dtilt > Shield` keeps the opposite order.
Near/stay/resource/risky labels retain their meaning. The spacing hints remain
unchanged. These display-only aliases are resolved when the cached snapshot
changes; move IDs, functions, selection order and timing are unaffected.

Move names include qualitative spacing estimates, for example `Nair [~close]`,
`Down tilt [~mid / low]`, or `Short-hop zair [~long / front]`. These are rough
coverage cues, not measured maximum distances. Close means nearby, mid means a
moderate melee gap, and long means extended reach; these are comparative bands,
not a shared numerical scale across fighters. Above/below/front describe where
the target needs to be when the hitbox is active. Offstage identifies a drop
chase; ranged identifies projectile coverage, whose travel time still matters.
Grabs, passive shield, and automatic roll approaches omit the range hint.
The `~` explicitly marks an estimate. Opponent size, height, drift, attack phase,
and positioning can change actual contact. Unsupported future actions display
`~spacing?` until reviewed. Numerical maximums require in-game measurements.
These hints live in `fighter_profiles/spacing.rs` and are appended only when
the existing menu snapshot changes; they add no per-frame distance queries and
do not change response timing or inputs.

The menu contains only fighter-specific ledge choices. Combo playback, fighter
combo definitions, and Y-mode selection have been removed. All responses are
compiled Rust functions; changing a choice changes a saved index and cached
settings. No fighter JSON or runtime scripts are loaded.

The adapter caches labels until a menu revision and resolves response metadata
only after fighter/choice changes. Drawing reads a cached snapshot, with no
native fighter or opponent queries. Full ledge/movement observations, including
ground-support rays, run only while ledge, slide or trump needs them.
A released trigger still permits an in-flight slide/trump to finish. Active
move timing is not throttled. Baseline mash, tech and SDI remain automatic.

This caching limits repeated configuration work; device heat and frame time
have not been profiled. Source edits do not update an installed NRO. See
[current controls](CONTROLS.md) and [fighter guides](FIGHTERS.md); rebuild and
install separately to use source changes.


## Frozen diagnostic calibration results

Diagnostic builds provide a [frozen result card](CALIBRATION_CARD.md) for all
five ledge response lists. The compact card appears beside the menu after an
attempt ends; L+R3 hides/shows both. It records available configured timing,
request/mapping/startup evidence, root spacing and waiting gates. Full detail
is written to `status.log`. Missing metadata stays unknown; configured frames
are not measured hitboxes. Silent builds exclude this feature.
