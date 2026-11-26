# Starting cast: choose exact profiles, not legacy timing tables

The original eight are Mario (`mario`), Donkey Kong (`donkey_kong`), Link (`link`),
Samus (`samus`), Yoshi (`yoshi`), Kirby (`kirby`), Fox (`fox`) and Pikachu
(`pikachu`). Each has its own exact-kind profile and generated page in
[Fighters](FIGHTERS.md). There is no starting-cast fallback or shared copied
startup table; each file supplies five ledge rows and optional button/reaction records.

## First edit

Open the fighter's page, read its supplied overrides and actual action column,
then use [the configuration walkthrough](FIGHTER_CONFIGS.md). For example,
`[donkey_kong.neutral.my_response]` belongs to DK; `[donkey.neutral...]` does not
match the registered profile name. Start with an existing action, add an explicit
wait/range only if needed, save in Training and wait for success. External file
changes need restart. A label/default is configuration, not a native move promise.

Kirby's trip/fair/idle-crouch helpers and Yoshi's tilt-facing support are separate
from ledge recipes; see [Kirby/Yoshi](KIRBY_YOSHI_LEDGE.md). Samus tether/projectile
recipes require different state/travel assumptions; see [Samus](SAMUS_LEDGE.md).
Other starting fighters inherit neither standalone Lucina counter nor Game & Watch
projectile-defense adapters. Universal held tools have separate eligibility gates.

Use [all tuning keys](FIGHTER_SETTINGS.md) for estimates and
[execution/calibration](FIGHTER_EXECUTION.md) for frame accounting. Native
[Fighter Data](FIGHTER_DATA.md) is independent of input scheduling; a wider active
window, faster character or larger estimate does not guarantee contact. No old
compiled lead, adaptive hop or roll endpoint is hidden behind a current label.
