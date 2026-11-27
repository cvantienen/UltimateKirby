# Interactive build and sequence workbench

From the project directory, run:

```sh
python3 tools/workbench.py
```

This opens a small dark-themed browser GUI served only on your computer, using
Python's standard library. Keep the terminal open; **Close** stops the launcher.
No build or FTP connection happens merely by opening it. All original helper
commands still work independently. Training LAN connects only on explicit request to the separately activated Diagnostic server.

Seven tabs cover the whole workflow, and the window reopens on the tab you used
last:

| Tab | Use it to |
| --- | --- |
| **Ship** | Set the version and build/upload to either or both Switches |
| **Edit fighter** | Change one fighter's sections with a diff preview |
| **Training LAN** | Explicit Diagnostic connection, complete-file editor and bounded live logs |
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

## Training LAN editor and live Diagnostic logs

**ON immediately returns to OFF:** first create the SD pairing file below.
Without it the listener deliberately stops; it cannot accept unpaired edits or
logs. The Training row now retains startup errors (missing/invalid setup, bind
failure, or expired Training observation) so troubleshooting does not depend on
connecting to a server that failed to start. Correct setup and activate again;
there is no automatic retry. A bind error usually means `switch_ip` is not the
Switch's current address or the chosen HTTP port is already in use. Workbench
connection failures can also mean a wrong computer IP/token or LAN isolation.

Manual setup only; Workbench does not deploy configuration, discover devices or
activate the native listener. Use an isolated trusted LAN and router/firewall rules.
Private numeric addressing is not proof of a local route or internet isolation;
this feature does not control Nintendo services or other mods. The native
listener uses the installed Switch Rust TCP backend, whose first socket use may
request OS network access. That initialization runs only on the LAN worker, not
on controller/render callbacks; it cannot be forcibly cancelled while inside
the OS. Revocation is rechecked before serving requests. No private/guessed
native networking bindings or shared-service finalization are introduced.

1. On the Diagnostic Switch SD card, create
   `sd:/ultimate/kirby_assist/lan_editor.txt` with the exact Switch and computer
   RFC1918 IPv4 addresses and a separate HTTP port (not FTP):

   ```ini
   switch_ip=192.168.1.142
   computer_ip=192.168.1.20
   port=4242
   token=REPLACE_WITH_GENERATED_SECRET
   ```

   Generate your own secret locally when setting up (do not use the placeholder):
   `python3 -c 'import secrets; print(secrets.token_urlsafe(32))'`.
   Native tokens require 32–128 ASCII letters/digits/`-`/`_`. Protect the SD
   configuration and never share the token in screenshots or logs.
   On a mounted SD card the file is `/ultimate/kirby_assist/lan_editor.txt`;
   `sd:/` is the game's path prefix, not an SD directory named `sd:`. Use plain
   UTF-8 text and ensure the filename is not accidentally `lan_editor.txt.txt`.
   Find the Switch IPv4 in System Settings → Internet → Connection Status.
   On macOS find the computer IPv4 in System Settings → Network → your active
   Wi-Fi/Ethernet connection → Details → TCP/IP. Both devices must use the same
   trusted LAN; guest Wi-Fi/client isolation can prevent their communication.
   These are examples: replace both addresses with your actual addresses.
2. Start the Diagnostic build, enter Training, press Minus to open the editor,
   select **FIGHTER**, and enable **LAN editor HTTP** with A or Left/Right.
   Keep the editor open for saving. **STARTING** means setup/network initialization
   is in progress; **LISTENING** means you can connect from Workbench. If it
   returns to OFF, the row retains the failure reason. Finish/discard local
   unsaved recipe/tuning drafts first; they cannot overlap a remote write.
   Silent has no diagnostic listener. Leaving Training revokes permission.
3. Open **Training LAN**, enter the **Switch** address, HTTP port and shared
   secret; press **Connect explicitly**. Workbench saves only host/port. The
   secret stays in process memory and is cleared on disconnect; it never enters
   the journal or browser storage. HTTP is plaintext on the trusted LAN, not TLS.
4. **Load complete file**, preserve unknown/unrelated sections, edit fighter
   recipes or tuning, then **Validate + preview** with the existing local parser.
   `[universal.settings]` and templates are immutable startup contracts: native
   rejects their changes even if local syntax validation succeeds. **Save +
   reload…** confirms the exact validated text and uses the opaque loaded
   revision. Saving requires the native editor open; stale revisions (409) or
   revoked access (403) disconnect instead of overwriting. Reconnect and load
   again manually. Unsaved browser text remains available to copy/reconcile.
5. Save acknowledgment confirms disk save and catalog reload, **not gameplay
   success**. The next mapper processes native pending-assist cancellation and
   menu-selection reset from reread disk; it may not have run when HTTP returns.
   Load again before making another edit. No FTP/build-ready checkbox is needed.

All HTTP calls go through the loopback Workbench backend to one configured
numeric RFC1918 Switch address with `X-Kirby-Token`. There is no DNS, proxy,
redirect following, discovery, automatic reconnect or persistent log worker.
GET requests have a two-second total transport deadline; PUT allows ten seconds
for validation/SD save and acknowledgment. Lost save acknowledgment does not mean
the disk is unchanged: reconnect and reload before deciding what to do; never
automatically retry a save. Sequence reads/writes are
bounded to 512 KiB, status/save replies below 8 KiB, and log replies to 64 KiB / 64
entries. A transport/protocol error clears the session; only explicit connect
starts another one. State polling uses a separate lock and is not held during
Switch I/O. Backend loopback Host/Origin/session-token protections remain intact.

Logs poll while explicitly connected and not paused, including startup/errors.
**Clear view** is local only; **Pause polling** stops log reads, not gameplay.
The text filter and log renderer use plain text, never peer-supplied HTML. The
browser view keeps at most 512 entries / 128 KiB. **Native live-ring loss
(overwrites/truncation)** counts loss in the bounded native live ring, not the
source queues. Source queue losses appear as separate event lines. A lower valid native cursor indicates a server
restart; Workbench resets its view/cursor and accepts the sorted retained batch.
An **event** means an assist decision/request;
**mapper delivery** means controller input reached the mapper, not that the game
accepted a move. **Game confirmation** needs a later observed state. Missing
observations or dropped/gapped queue entries cannot prove success or failure.

`python3 tools/workbench.py --cli` also offers **Training LAN editor / live logs**:
explicit connect, read a complete file to a new computer file, preview/save that
edited file, read one new log batch, and disconnect. No secret command-line
argument is used; the terminal asks privately. No automatic writes or retries.

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
