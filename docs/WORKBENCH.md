# Interactive build and sequence workbench

From the project directory, run:

```sh
python3 tools/workbench.py
```

This opens a small dark-themed browser GUI served only on your computer, using
Python's standard library. Keep the terminal open; **Close** stops the launcher.
No build or FTP connection happens merely by opening it. All original helper
commands still work independently. This adds no Switch runtime work.

Six tabs cover the whole workflow, and the window reopens on the tab you used
last:

| Tab | Use it to |
| --- | --- |
| **Ship** | Set the version and build/upload to either or both Switches |
| **Edit fighter** | Change one fighter's sections with a diff preview |
| **Break down** | Split a combined `sequences.txt` into `fighters/*.txt` |
| **Transfer** | Copy a whole file or one fighter between computer and Switches |
| **History** | Review shipped versions, past runs and their logs |
| **Helpers** | Run the original scripts with their own arguments |

The header shows the current `Cargo.toml` version, the **Ship** tab shows the
last verified upload for each Switch, and the output console stays visible at
the bottom of every tab.

## Choose the workflow before pressing a button

This is host tooling, not a Rust gameplay module. Opening the GUI does not build
or connect, but explicit reads of Switch files do contact FTP; previews can also
run the host Rust sequence parser. `--dry-run` means no FTP only where the exact
helper promises it. For the beginner runtime/tutorial use
`crates/kirby_assist_core/docs/DEVELOPER_GUIDE.md` and [Development](DEVELOPMENT.md).

| Workflow | Configuration authority | Writes / limit |
| --- | --- | --- |
| Build only | Current repository split sources | Version/build outputs; no guided upload |
| Guided Ship / Upload built | Validated Silent active snapshot | Imports repository sources and replaces selected active Switch files |
| Direct `upload_switch.py` default | Current repository sources | Stages inactive `sequences.upload.txt` alongside release payload |
| Transfer whole file | Explicit reviewed source | Replaces all destination recipes/settings |
| Transfer one fighter | Explicit reviewed source fighter | Preserves other fighters; destination merge requires existing file |
| Verify installed | Expected selected payload | Reads remote bytes; not a gameplay or compatibility test |

Do not mix these workflows assuming all uploads preserve active settings. Keep
Smash closed for device writes and review both configuration scope and binary
variant. A docs-only task must not launch any of these operations.

## Quick build and upload

1. On **Ship**, enter the version or click **+1** to increase its last
   number. Select **Both switches** (default), **Silent only**, or **Diagnostic only**.
2. Close Smash and open FTP on the selected Switches. Check the ready box once
   for this GUI session.
3. Click **Build + Upload**, or press Enter in the version field. It updates
   `[package].version`, temporarily stages the validated Silent configuration for
   the build/preflight, then imports that configuration into split sources and
   uploads selected variants with active sequence replacement. A failed build
   prevents the guided upload stage; staging restoration is a separate local step.

The tool's historical default Cargo home is
`/private/tmp/kirby-assist-build-cargo`; before running it under a workflow that
forbids temporary-directory writes, change **Build & connection settings** to an
approved project-local cache such as `.cargo-home`. This documentation update
does not run the tool or alter its default. The same panel controls FTP ports. Settings are
remembered. **Build only**, **Upload built** and **Verify installed**
are available without the terminal prompt sequence. Upload/verify actions use
the entered built version without changing the source version. A source version
selected before a failed build stays selected for retry; it is not marked shipped.

**Guided Ship and Upload built replace active `sequences.txt`.** They first read
and validate Silent's active file as the authoritative configuration, stage that
snapshot during build/upload preflight, then import it into repository split
sources and replace active files on the chosen targets. Silent must be reachable
for that guided synchronization even if only Diagnostic is selected. This differs
from the direct upload helper's default inactive staging. **History** retains logs and the last verified
upload for each Switch. A partial upload is reported as a failure with individual
target results; the other Switch is still attempted.

## Editing one fighter

**Edit fighter** changes a single fighter's recipes and settings without copying
whole files. Choose the file — the repository source, or a Switch active/staged
file — and click **Read file**. That one read becomes the snapshot every later
step compares against; the fighter list is filled from it.

Pick a fighter, click **Open fighter**, and its sections appear in the text box.
**Preview changes** validates the result with the current parser and shows a
unified diff of only that fighter. **Save fighter…** then writes it. Other
fighters and `[universal.settings]` are preserved and checked independently; an
edit that would alter another fighter is rejected before any write.

Editing the repository `sequences.txt` writes back through the split sources, so
the edit itself lands only in `fighters/<fighter>.txt`. As with any source
import, a few other fighter files can pick up trailing-blank-line normalization;
recipe content and section order are unchanged. Clearing the text box entirely
deletes that fighter from the file. **Revert edits** restores the loaded text
and drops any pending preview. Changing the file, fighter or text invalidates
a preview.

A local repository edit does not need the ready checkbox. A Switch destination
still requires Smash closed, FTP open and the ready box on **Ship**.
Saving keeps a recovery backup and verifies the written bytes, and `fighter-load`,
`fighter-preview` and `fighter-save` are recorded in History.

The same flow is item 5 in `--cli`, which opens `$VISUAL`/`$EDITOR` on a scratch
copy (or waits while you edit it) and asks for `SAVE` after showing the diff.

## Breaking a combined file into fighter sources

**Break down** splits one combined `sequences.txt` into the individual
`fighters/<fighter>.txt` files the build assembles from. Use it after
downloading Training edits from a Switch, or to import any new sequence file
into the repository sources.

Choose the combined file (a Switch active/staged file or a file on this
computer), confirm the repository universal source to write beside, then click
**Preview split**. The preview lists each fighter file as new, replaced,
deleted or unchanged, with its section count — nothing is written yet.
**Write source files…** then applies it after a confirmation that names the
deletions.

Fighters present in `fighters/` but absent from the combined file are deleted,
so the sources always match the file you split. Every replaced file is copied
to `.mod-workbench/source-breakdowns/<timestamp>/` first, and the sources are
re-assembled and compared against the imported bytes afterwards; a mismatch
rolls the whole import back.

The same flow is **Break down a combined file into fighter sources** in `--cli`,
which prints the summary and asks for `BREAKDOWN`. The standalone helper also
previews without writing:

```sh
python3 tools/sequence_sources.py --import-file /path/to/downloaded.txt --dry-run
python3 tools/sequence_sources.py --import-file /path/to/downloaded.txt
```

On **Transfer**, choose source and destination, optionally enter a fighter ID,
then **Preview transfer**. **Confirm replacement…** shows the actual destination
and scope before replacing anything. Changing a transfer field invalidates that
preview. Whole-file replacement overwrites every fighter; one-fighter transfer
replaces only that fighter's saved recipes/settings. Validation, backups,
concurrent-edit checks and readback verification use the existing transfer helper.

**Helpers** runs the original helper scripts with entered arguments; start with
`--help`. This is explicit command execution, not a harmless documentation link;
read the exact helper flags before authorizing builds or transfers. Switch writes require the ready checkbox, and sequence writes also
require confirmation. `sequence_sources.py` is available here for assembling and
checking the split sources; its `--import-file` rewrites the repository fighter
files, so it is confirmed like other sequence writes while `--check` is not.
Helpers that need interactive terminal input can still be used through the
original terminal interface:

```sh
python3 tools/workbench.py --cli
```

Only one operation runs at a time. Build/upload output streams into the window
and the existing journal. Closing the launcher waits for an active operation to
finish rather than interrupting a transfer. `--gui` explicitly selects the GUI;
`--history` remains a read-only terminal command.

## Terminal operations (`--cli`)

- **Build** selects silent, diagnostic or both and delegates to the existing
  build helper. It requires `BUILD` confirmation and does not upload.
- **Upload a built release** previews the existing uploader plan for each chosen
  Switch. Type `UPLOAD` only with Smash fully closed and FTP open. Each target
  has a separate result, so one failure cannot be mistaken for success on both.
- **Ship: build, then review/upload** offers upload only after a successful build.
- **Validate** runs the existing sequences validator against your selected file.
- **Edit one fighter's sequences** reads a chosen file once, opens that fighter's
  sections in `$VISUAL`/`$EDITOR`, shows a diff and asks for `SAVE` before writing.
- **Break down a combined file into fighter sources** splits one `sequences.txt`
  into `fighters/*.txt`, prints the per-file summary and asks for `BREAKDOWN`.
- **Transfer / replace sequences** copies one fighter or a whole file between computer,
  silent and diagnostic endpoints in either direction.
- **Verify installed release** uses the existing read-only verification command.
- **History** shows recent runs, logs and the last verified guided upload per
  Switch. **Original helpers** exposes their existing advanced arguments.

Preferences remember the local sequences path, edited file/fighter, FTP ports,
cargo cache and last successfully built version. Enter keeps a displayed default;
`-` clears the custom cargo cache. The workbench's default file is the assembled bundle at
`deployment/ultimate/kirby_assist/assembled/sequences.txt`; source-aware helpers
resolve it through the split sources. Defaults select silent `192.168.1.106`
and diagnostic `192.168.1.142`, with FTP port `5000`. Saved GUI preferences can
change addresses/ports; the current code validates distinct IPv4 routes.
Do not treat those local defaults as verified reachable devices.

## Sequence transfers

The repository's `deployment/ultimate/kirby_assist/sequences.txt` is the universal
source beside `fighters/*.txt`. Reading that path assembles all fighters; writing
a downloaded combined file to it imports the individual sources with backups and
readback verification. Build/upload/validation commands use current sources,
not an older generated bundle. See [source layout and named templates](SEQUENCE_FILES.md).

To replace the active Switch file, choose **Transfer / replace sequences → FROM
Local computer file → TO the Switch → ACTIVE sequences.txt → Whole file**.
Review the preview, then type `TRANSFER`. This replaces every active recipe and
saved fighter setting; choose **One fighter** to preserve the other fighters.

Choose FROM, TO, then one fighter or the whole file. A fighter transfer replaces
all that fighter's recipes and saved settings while preserving other fighters.
Use the fighter ID from your section headers. The destination must already exist
for a fighter merge; a whole-file transfer can create a file.

Switch endpoints offer **active `sequences.txt`** and **staged
`sequences.upload.txt`**. Staged files do not become active automatically.
Validation uses the current source parser; it does not prove compatibility with
an older installed plugin.

Review the destination and result hash before typing `TRANSFER`. Close Smash on
involved Switches first so in-game saves cannot race the transfer. The workbench
rechecks both files, makes local recovery snapshots and verifies destination
bytes after writing. Existing remote destinations also receive a verified
`.workbench.bak` before replacement. A failed remote replacement attempts to
restore the original; inspect the reported recovery path if restoration fails.
A failed first upload to a nonexistent destination can leave a partial file.

## History and recovery

Local state lives in `.mod-workbench/`:

- `history.jsonl`: operation status, version, target, hashes and results.
- `logs/`: original helper output, including failures.
- `artifacts/`: content-addressed copies of build/upload payloads.
- `sequence-backups/`: source, prior destination and proposed transfer bytes.
- `fighter-edits/`: validated single-fighter results and their recovery backups.
- `source-breakdowns/`: the fighter sources replaced by each breakdown import.
- Preferences and a lock preventing concurrent workbenches using this directory.

Guided uploads are marked shipped only after the original uploader reports full
verification. During Silent synchronization, repository sources are backed up and
staged for preflight, with restoration around that temporary stage. Failed guided
uploads may attempt repository rollback when Silent was not successfully updated;
inspect the reported per-target and rollback outcomes. These steps are not an
atomic transaction across the repository and two consoles. Actual payload hashes are retained from its output. Hash equality proves byte
identity for that payload/readback, not native input acceptance, compatibility or
protection against later edits. Advanced
helper runs are logged but do not populate the guided last-shipped summary.
Manual commands run outside the workbench are not imported into history. A run
without a completion record has unknown status; verify before retrying writes.

History, artifacts and transfer backups are excluded from source release ZIPs.
Keep them for recovery; they can grow with new builds. To inspect history without
building or contacting FTP:

```sh
python3 tools/workbench.py --history
```

See [build variants](BUILD_VARIANTS.md) for package contents and existing direct
build/upload commands. Original helper confirmations and validation remain in
place. The workbench does not deploy unrelated plugin patches automatically.

## Direct sequence upload helper

The standalone helper also offers both destinations:

```sh
python3 tools/upload_sequences.py --upload
python3 tools/upload_sequences.py --replace --smash-closed
```

The first stages `sequences.upload.txt`, preserving the active file. The second
backs up and replaces the whole active file using the shared transfer/recovery
path; close Smash on both Switches first. Add `--file PATH` to select a source or
`--dry-run` to preview without connecting. No flag keeps the original staged-copy
behavior. Guided Workbench transfers still provide endpoint/fighter selection and
history; direct helper uploads do not create Workbench shipped-release records.
