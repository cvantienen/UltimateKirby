# Roll reactions

The live roll assist uses **your saved reaction sequences for the selected fighter**.
Open **Assist modes → Roll reactions → L3/R3** in Training to edit its separate
priority-ordered list. See [reaction rules](REACTION_RULES.md) and the
[rule cheat sheet](REACTION_RULE_CHEATSHEET.md).

Roll reactions starts ON by default. A matching enabled recipe is required to
produce inputs. Switching its Menu group OFF disables those rules; it does not
reactivate an old hard-coded grab/down-smash helper. Explicit button-controlled
roll groups retain their own activation state.

Use Side and distance conditions to select front/back or close/far responses.
`roll` detects the roll; `roll_vulnerable` additionally requires observed
vulnerability. Inputs, delays and reach checks come from the chosen recipe.
General roll rules do not replace a protected ledge response; incoming-grab
defense and native safety retain their documented priority.

The old Kirby/Yoshi fixed Roll read and far-roll smash fallback are removed from
live execution, and their unused portable prototype has been removed. Seven old
`punish_*` / `roll_*` fighter-setting keys
remain readable for file compatibility and are hidden/inactive; configure the
current rule's range, side, height, priority and actions instead.
