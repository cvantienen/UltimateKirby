# Saved ledge menu

**R3** opens/closes the menu in every build, including diagnostic Training compatibility. The former L+R3 binding is removed. While
visible, **Up/Down** selects a row and **Left/Right** changes its choice.
**Auto-mash** is now an editable **ON/OFF** row, shared across fighters and
preserved through stocks/matches until the plugin reloads. Minus does not
change mash. Disabling mash releases its cached input without disabling tech.

Hold **L** for ledge autopilot (including Kirby in-range recovery cover),
press **Y** for Kirby's selected Finisher attempt, and press **X** for
X short-hop tap. With the menu hidden, sides slide, Down trumps and Up
stays normal. Directions held while closing must be released before gameplay
uses them. R3 is reserved for the menu. Hold L3 without L for turbo teabag.

The menu is a **transparent single column in the top-left corner**, with no
panel backdrop, border or title. Each cached line contains its category, saved
move and option count. At 720p it uses 19-pixel text, four-pixel row spacing and
three-pixel inner padding, scaled with display height. Its anchor is 0.5% from
the left edge and 0.8% from the top, with a minimum four-pixel left margin. Width follows
the longest cached line; each row stays on one line without word wrapping. Dark opposing shadows and a small bold pass
keep text legible over bright/dark stages. Gold plus a `>` marker identifies
selection; mode rows use lavender, and Auto-mash uses green ON/red OFF when not
selected. Rows stay on a single line; bracketed spacing hints are removed.

It begins visible each battle, stays through stock loss until hidden, and closes
outside battle. Editing also works during entry/countdown once your fighter is
known. Lines are rebuilt only on selection revisions. No emoji font, texture,
asset loading, animation or live gameplay read was added; actual rendering cost
still needs device measurement. The diagnostic result card retains its own dark
backdrop for detailed calibration text.

Common move labels are abbreviated: `Dtilt`, `Ftilt`, `Utilt`, `Dsmash`, `Fsmash`,
`Fair`, `Uair`, `SH Zair` (short-hop zair), and `Homing Msl` (homing missile).
`Shield Nair` means shield then nair; `Dtilt > Shield` keeps the opposite order.
Near/stay/resource/risky recipe labels retain their meaning. Display-only
aliases are resolved when the cached snapshot
changes; move IDs, functions, selection order and timing are unaffected.

Move names show the compact saved action without appended coverage estimates,
for example `Nair`, `Dtilt` and `SH Zair`. The old spacing-hint helper remains
available in source but is no longer called by the menu. This display change
does not alter response timing, selection order or input behavior.

The five ledge lists remain fighter-specific. Kirby also has the separate
[Roll read](ROLL_READ.md) selector: Off / Down smash / Grab for ordinary onstage rolls.
It defaults to Down smash and does not change the ledge Roll list. Combo playback, fighter
combo definitions remain removed. Kirby's **Finisher** row contains
Fair 1 > Fsmash and Dair > Dsmash. Yoshi's row offers Bair drag > Fsmash.
Automatic whiff punishing is currently removed from the live build/menu.
Press Y for one manual attempt; holding never loops.
See [confirmation, timing and limitations](FINISHERS.md). Auto-duck is disabled.
Y choices live in `button_modes.rs`; future compiled functions can be
added to that list. Other fighters do not show the Y row until configured.
All responses are
compiled Rust functions; changing a choice changes a saved index and cached
settings. No fighter JSON or runtime scripts are loaded.

To add a future Y mode, add its enum/label and fighter availability to
`crates/kirby_assist_core/src/button_modes.rs`, then route its held input to a
dedicated core policy in `src/live.rs`. Keep release/edit cancellation and the
single shared final input overlay. The menu reads the compiled list only on a
saved-selection revision; it does not execute mode functions while drawing.

The adapter caches labels until a menu revision and resolves response metadata
only after fighter/choice changes. Drawing reads a cached snapshot, with no
native fighter or opponent queries. Full ledge/movement observations, including
ground-support rays, run only while ledge, slide or trump needs them. Held-Y
defense uses its own small observation without ground-support rays. Enabled
Roll read adds its own small status-filtered roster check without ground rays.
A released trigger still permits an in-flight slide/trump to finish. Active
move timing is not throttled. Baseline mash, tech and SDI remain automatic.

This caching limits repeated configuration work; device heat and frame time
have not been profiled. Source edits do not update an installed NRO. See
[current controls](CONTROLS.md) and [fighter guides](FIGHTERS.md); rebuild and
install separately to use source changes.


## Frozen diagnostic calibration results

Diagnostic builds provide a [frozen result card](CALIBRATION_CARD.md) for all
five ledge response lists. The compact card appears beside the menu after an
attempt ends; R3 hides/shows both. It records available configured timing,
request/mapping/startup evidence, root spacing and waiting gates. Full detail
is written to `status.log`. Missing metadata stays unknown; configured frames
are not measured hitboxes. Silent builds exclude this feature.

R3 close requests update a separate visibility flag before attempting the cached-text lock, so the next draw skips the menu even if its previous snapshot is still in use. Closing preserves selections; release R3 before pressing it again to reopen.

Kirby additionally has **Idle crouch**, **Dtilt face**, and **A+B grab** toggle
rows, initially ON. Holding L suspends idle crouch until release. Tap neutral
A+B for jab 1 into an ordinary grab; A alone stays normal. Yoshi shares the **Roll read** Off/Down smash/Grab row with his own active
windows. The [posture/input guide](INPUT_RECIPES.md) explains their priorities.

Yoshi also has a separate **Jab lock** ON/OFF row, initially OFF. It chases only
his own confirmed down-tilt hits and takes priority over background Roll read.
See [Yoshi down-tilt chase](JAB_LOCK.md) for reactions and calibration.
