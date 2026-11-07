# Interactive build and sequence workbench

From the project directory, run:

```sh
python3 tools/workbench.py
```

This opens a small browser GUI served only on your computer, using Python's
standard library. Keep the terminal open; **Close workbench** stops the launcher.
No build or FTP connection happens merely by opening it. All original helper
commands still work independently. This adds no Switch runtime work.

## Quick build and upload

1. On **Ship a version**, enter the version or click **+1** to increase its last
   number. Select **Both switches** (default), **Silent only**, or **Diagnostic only**.
2. Close Smash and open FTP on the selected Switches. Check the ready box once
   for this GUI session.
3. Click **Build + Upload**, or press Enter in the version field. It updates only `[package].version` in `Cargo.toml`,
   builds the selected variants, validates their upload payloads and uploads them
   using the existing safe routes. A failed build prevents all uploads.

The default Cargo home is `/private/tmp/kirby-assist-build-cargo`; expand
**Build & connection settings** to change it or the FTP ports. Settings are
remembered. **Build only**, **Upload built version** and **Verify installed**
are available without the terminal prompt sequence. Upload/verify actions use
the entered built version without changing the source version. A source version
selected before a failed build stays selected for retry; it is not marked shipped.

Regular release uploads preserve active `sequences.txt`, staging a separate
`sequences.upload.txt` as before. **History** retains logs and the last verified
upload for each Switch. A partial upload is reported as a failure with individual
target results; the other Switch is still attempted.

On **Sequences**, choose source and destination, optionally enter a fighter ID,
then **Preview transfer**. **Confirm replacement…** shows the actual destination
and scope before replacing anything. Changing a transfer field invalidates that
preview. Whole-file replacement overwrites every fighter; one-fighter transfer
replaces only that fighter's saved recipes/settings. Validation, backups,
concurrent-edit checks and readback verification use the existing transfer helper.

**Helpers** runs the original helper scripts with entered arguments; start with
`--help`. Switch writes require the ready checkbox, and sequence writes also
require confirmation. Helpers that need interactive terminal input can still be
used through the original terminal interface:

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
- **Build, then review/upload** offers upload only after a successful build.
- **Validate** runs the existing sequences validator against your selected file.
- **Transfer / replace sequences** copies one fighter or a whole file between computer,
  silent and diagnostic endpoints in either direction.
- **Verify installed release** uses the existing read-only verification command.
- **History** shows recent runs, logs and the last verified guided upload per
  Switch. **Original helpers** exposes their existing advanced arguments.

Preferences remember the local sequences path, FTP ports, cargo cache and last
successfully built version. Enter keeps a displayed default; `-` clears the
custom cargo cache. The fixed Switch addresses remain silent `192.168.1.105`
and diagnostic `192.168.1.142`; the default FTP port is `5000`.

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
- Preferences and a lock preventing concurrent workbenches using this directory.

Guided uploads are marked shipped only after the original uploader reports full
verification. Actual payload hashes are retained from its output. Advanced
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
