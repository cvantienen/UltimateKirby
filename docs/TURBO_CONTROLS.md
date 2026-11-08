# Button recipes and turbo controls

**Minus** opens/closes the menu. Outside Training, R3 switches between the
lightweight ledge selector and Buttons / Assists. Training retains editing tools.

**Turbo t-bag** is a built-in button role, not a D-pad macro or global toggle.
Hold whichever button slot is assigned **Built-in: Turbo t-bag**; the default
binding is **L3**. It starts with a crouch and alternates one crouch input and
one slight facing-direction creep input per local update. Releasing the button
stops it. The optional `turbo_walk_strength` setting controls the creep amount.
Grounded movement readiness and higher-priority input owners can suppress it.
L3 never becomes the ledge-assist trigger in the Brother control layout. In
Training menus, L3 retains its menu action and does not run the gameplay helper.

The built-in role can be selected for any configurable button slot: X, Y, the
four D-pad directions, L3 or R3. Change the slot's choice in the Buttons editor;
select **Normal button** on the old slot if moving the helper elsewhere. D-pad
slots are ordinary configurable buttons; there is no separate D-pad turbo mode.
Legacy `r3_turbo`, `l3_turbo`, `dpad_turbo` and `turbo_phase_frames` config keys
are accepted for old files but no longer select, enable or slow the helper.

X, Y and all four D-pad directions have fighter-specific input recipes. X
defaults to **Normal button**, which passes the physical input through without
short-hop assistance. Configure a recipe such as `short_hop:1` when you want
short-hop behavior. Recipes can run once per press or loop while held;
tap/hold actions are configured in their recipe editor.

The old Y native Finisher selector, A+B jab-to-grab and hard-coded D-pad
slide/B-reverse/trump bindings are removed. Assist modes use their ON/OFF menu
rows. Existing reaction-rule toggle groups are separate metadata and remain
compatible; they are not generic button-to-mode bindings.

See [controls](CONTROLS.md), [input recipes](SEQUENCE_FILES.md),
[assist rules](REACTION_RULES.md) and [posture helpers](INPUT_RECIPES.md).
