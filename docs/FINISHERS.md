# Retired native Y finisher binding

The former Y Finisher/combo selector is removed. X, Y and D-pad buttons now use
fighter-specific saved input recipes or **Normal button**. Normal X uses the
short-hop shortcut; normal Y passes through unchanged. Old native Y combo choices
are not available on Button configs.

Build your own input sequence in Training and save it with Y in the editor.
Use explicit checks, delays and movement where needed; a sequence is simulated
controller input, not a guaranteed combo or a replacement for native move rules.
Existing automatic hit-confirm helpers are separate assists, not the old Y menu.

See [current controls](CONTROLS.md), [editor actions](EDITOR_ACTION_CHEATSHEET.md)
and [sequence syntax](SEQUENCE_FILES.md).
