# Kirby Assist terminal workbench

Independent Python/Textual alternative to the existing browser GUI. The spelling
`workbench_reminal_ui` is intentional, matching the requested directory.
Nothing in the original GUI or its assets is modified. This frontend imports the
existing Controller, Workbench, sequence editor, journal, telemetry and capture
services; fixes to those shared services remain available to both interfaces.

## Run

Python 3.10+ and the existing project's helper/toolchain prerequisites are required.
From the project root:

```sh
python3 -m venv tools/workbench_reminal_ui/.venv
tools/workbench_reminal_ui/.venv/bin/python -m pip install -r tools/workbench_reminal_ui/requirements.txt
tools/workbench_reminal_ui/.venv/bin/python tools/workbench_reminal_ui/__main__.py
```

Or from `tools`:

```sh
workbench_reminal_ui/.venv/bin/python -m workbench_reminal_ui
```

Optional `--project /path/to/UltimateKirby` and `--state-dir /path/to/state` select
a checkout and journal. The default checkout is the one containing this frontend.
Set the project environment **before** importing the shared services; the launcher
handles this. State inside the checkout must be under `.mod-workbench` to avoid
including journal/backups in release archives.

The app shares preferences, history and backups with the browser GUI. Only one
workbench can use a state directory at once, enforced by the existing session lock.
Close the browser workbench before opening this alternative. Do not point separate
sessions at the same devices concurrently, even with different state directories.

## Unified launcher and macOS app

Normal macOS startup of `workbench.py` now opens a native Web / Terminal / Cancel
selection window. `--choose` explicitly requests it. On other platforms the default
remains Web. These modes bypass the chooser:

```sh
# From tools, use a Python that has the terminal requirements installed:
workbench_reminal_ui/.venv/bin/python workbench.py --web
workbench_reminal_ui/.venv/bin/python workbench.py --terminal
workbench_reminal_ui/.venv/bin/python workbench.py --choose
```

`--gui` remains the Web alias; `--cli` and `--history` retain their original behavior.
The standalone terminal launcher delegates to the same project initialization and
session-lock path, so both interfaces operate on the selected checkout consistently.

The existing macOS builder now includes both interfaces and their dependencies.
Install `tools/requirements-macos.txt` in the packaging Python before running the
existing builder. Its install/signing/rollback behavior is unchanged. The package
verification checks CLI startup plus terminal imports and bundled stylesheet without
opening either interface or contacting devices.

A Finder-launched app has no terminal. Choosing Terminal hands the selected project
and state paths to Terminal.app using individually shell-quoted arguments and the
**packaged executable**, then exits the parent before acquiring any lock. The child
reattaches `/dev/tty` because the windowed PyInstaller runtime may clear Python's
standard streams. A macOS Automation permission prompt for Terminal may appear;
refusal produces an error, not a retry or fallback. Web launches directly. The
chooser itself starts no receiver, HTTP server, build, upload or device connection.

No macOS package was built during this implementation; actual frozen-terminal
attachment and Finder/Terminal handoff remain to be verified on a packaged app.

## Menu

| Group | Capabilities |
| --- | --- |
| Overview | Local project/state paths, last verified deployments, recent outcome chart, safety reminders |
| Deploy | Patch increment, target selection, build + upload, build only, upload existing release, installed verification |
| Sequences / Fighter editor | Load local/active/staged file, select and open fighter, edit sections, preview diff, confirmed save, revert |
| Sequences / Transfer | Validate local file, whole-file or fighter transfer between all existing endpoints, preview and confirmed replacement |
| Sequences / Split | Preview combined-file breakdown, confirmed per-fighter source replacement |
| Monitor / Training LAN | Explicit HTTP/UDP connection, optional non-persisted token, complete-file load/edit/validate/preview/save/reload |
| Monitor / Telemetry | Explicit local receiver startup/shutdown, explicit Switch subscription, packet/byte rate charts, schema-labeled object fields, raw values and schema details |
| Monitor / Live & raw logs | Live Diagnostic logs, raw telemetry and Diagnostic captures; include/exclude, freeze, wrap, follow, clear view, expand, clipboard, export |
| Activity | Searchable full recorded operation timeline, receipts/details, retained log inspection |
| Helpers | Every original registered helper, help and argument entry, confirmation before execution |
| Settings | Validated FTP addresses/ports and Cargo cache; persistent shared preferences |

No backend capability is intentionally removed. Browser-specific separate windows
are replaced by expandable terminal panes. Browser selection-copy is replaced by
terminal-native selection and a visible-log clipboard button. Clipboard support
depends on the terminal's OSC52 policy; export is always available. Telemetry uses
a schema-driven field explorer with exact values, a selected numeric-field history
chart with stale/unavailable gaps, decoded-observation freeze and visible CSV export.
The assistance table reproduces configured preference, proposing/selected evidence,
known constraints, mode/recipe and purpose; fresh game flags and final mapper values
appear in its summary. These observations never claim successful game execution. Charts use textual-plotext; the welcome art uses rich-pixels and an original locally generated Pillow image.

## Keyboard and layout

- Tab / Shift+Tab: focus; arrows: select tabs/options/table rows; Enter: activate.
- Ctrl+S: preview the active sequence or LAN draft.
- Ctrl+L: return the output pane to the current operation.
- Ctrl+O: hide/show the persistent output pane for extra editor space.
- Ctrl+Q: safe exit; waits for active operations, stops owned receiver/capture.
- F1: guide; Escape: cancel a modal confirmation.

Expanded logs open a full-screen, read-only snapshot; Escape returns to the live
pane. Decoded telemetry freeze holds the displayed age at freeze time, explicitly
labeled as historical; ordinary cached observations continue aging locally. Select
a telemetry row to choose its object for assistance analysis and its numeric field
for history plotting. Missing, stale and nonfinite samples leave chart gaps.

A 120×45 terminal is recommended. Tabs and action rows scroll horizontally on
smaller terminals. Editors show line numbers; logs treat peer output as plain text,
not Rich markup. Select a telemetry row for its object's raw JSON; select a history
row for its receipt and retained output. Export creates a **new** file and never
overwrites an existing path.

## Safety and persistence

Startup does not build, deploy, upload, start a receiver or contact a Switch.
Starting the existing telemetry receiver is explicit and may compile its offline
Cargo host example. Training activation/subscription is explicit and read-only.
Build/deployment actions retain the existing backend behavior, including writing
the requested manifest version and authoritative Silent sequence synchronization.

File replacements require preview tokens and a separate confirmation. Fighter and
LAN saves invalidate previews after draft changes; transfers/splits reject changed
forms. Changing fighter selection requires opening that fighter before previewing
its text. FTP readiness is independent of write confirmation. LAN editing uses the
backend revision/exact-text checks and never automatically retries a failed save.

Existing journals, recovery snapshots and capture files stay in their established
locations; no second data store is introduced. Drafts are in memory and lost on
exit; use previews/saves or explicit export where appropriate. Filters/freezes and
clear-view never stop capture or erase stored logs. No native game physics, status,
hitboxes or move definitions are changed by this frontend.

## Tests

From `tools`:

```sh
workbench_reminal_ui/.venv/bin/python -m pip install -r workbench_reminal_ui/requirements-dev.txt
workbench_reminal_ui/.venv/bin/python -m pytest -c workbench_reminal_ui/pytest.ini -q workbench_reminal_ui/test_terminal_ui.py
```

Headless Textual tests use temporary local journals and mocked operations. They
cover idle startup/navigation, preview invalidation/confirmation, log controls and
non-overwriting export, settings persistence/validation, serialized backend work,
schema-driven telemetry and safe retained-log lookup. They never build, upload or
connect to a Switch.
