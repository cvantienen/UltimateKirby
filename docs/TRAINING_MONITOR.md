# Training and system monitoring

**R3 alone** toggles Kirby Assist's menu in every build. D-pad Up/Down chooses a
row, Left/Right cycles it. L still holds ledge autopilot; Y attempts the selected
finisher; X is the manual short-hop shortcut. R3 no longer runs turbo teabag and
Hold L3 without L for turbo teabag. Keep R3 unassigned in Training's custom shortcut settings.

In diagnostic Training compatibility, default **ZL+Up load / ZL+Down save**
passes the shortcut through only while held, keeps our panel visible, and resumes
on release without an R3 tap. Saved choices and manually hidden visibility stay
unchanged. Holding L through the load resumes ledge practice once native control
returns; Y is not automatically rearmed. This assumes the standard ZL save/load
bindings, not arbitrary custom Training shortcuts.

Training menu/record shortcuts still hand the controller back until you close
their menu or recording and tap R3. Silent builds retain normal controls;
they do not activate this diagnostic handoff path.

## Monitor controls can overlap

The classic Status Monitor exit shortcut is **L+Down+R3**, according to its
[configuration documentation](https://github.com/masagrator/Status-Monitor-Overlay/blob/master/docs/config.md).
Kirby Assist ignores that chord for menu toggling. Diagnostic Training hands
that chord through until a fresh R3 resumes our tools. Ordinary R3 alone still
opens/closes the menu. An OS overlay that has captured the controller can prevent
Smash and both game mods from receiving input; changing our hook cannot recover
buttons the game never receives. This is a likely conflict, not a diagnosis of
the particular installed console.

For an existing classic Status Monitor installation, a separate exit chord can
be set in `/config/status-monitor/config.ini` without replacing other settings:

```ini
[status-monitor]
key_combo=ZL+ZR+PLUS
```

That is an example exit chord, not an automatic installation. Change only that
key in the existing section; also keep your overlay launch shortcut distinct.
Pressing L+Down before completing the old R3 chord can already start a trump;
using a distinct monitor chord avoids that partial-press overlap too.
Ultrahand exposes **Key Combo** in settings and per-overlay launch combos via Y,
as documented by [Ultrahand](https://github.com/ppkantorski/Ultrahand-Overlay).
Plus may pause Training/hand it input; close that menu and resume us with R3.

Use a compact stats display at a low refresh rate for CPU/GPU load, temperatures,
fan and memory, positioned away from our top-left menu. Exit the interactive overlay menu before testing game controls.
First test with monitoring closed, then with only the stats display. Compare
the same Training scene, fighter and CPU behavior; temperature alone does not
measure the plugin's CPU cost. This change does not install a monitor, alter
system clocks, or modify Training/Tesla/Ultrahand files on the Switch.

The original monitor is archived and points to
[Status Monitor Deux](https://github.com/masagrator/Status-Monitor-Deux), which
has configurable exit shortcuts. CPU/GPU/temperature monitoring is distinct from
FPS features requiring SaltyNX. No additional game plugin is necessary merely
to make our menu work; avoid changing multiple monitoring components at once
while isolating the input problem.
