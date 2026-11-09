# Training and system monitoring

**Minus** opens the ledge home or closes any menu in every build. D-pad Up/Down
chooses a row; Left/Right changes its value or opens a page link. L still holds ledge autopilot; hidden-menu X/Y use the
current fighter’s recipes, or Normal button for X short hop / ordinary Y. Minus does not run turbo teabag.
Hold the button assigned to Built-in: Turbo t-bag (L3 by default) for fast crouching. Keep Minus unassigned in Training's custom shortcut settings.

In Training compatibility (silent and diagnostic), default **ZL+Up load / ZL+Down save**
passes the shortcut through only while held, keeps our panel visible, and resumes
on release without a Minus tap. Saved choices and manually hidden visibility stay
unchanged. Holding L through the load resumes ledge practice once native control
returns; Y is not automatically rearmed. This assumes the standard ZL save/load
bindings, not arbitrary custom Training shortcuts.

Training menu/record shortcuts still hand the controller back until you close
their menu or recording and tap Minus. Both builds use this handoff when the
Training Modpack is installed and the game is in Training. Silent does not enable
logging or diagnostic collection. Other battle modes keep their normal controls.

## Testing feature cost in diagnostics

1. Use the diagnostic version and a repeatable Training setup: same stage,
   fighters, opponents and actions. Leave **Spot**, **Grab**, **Trump** and
   **Tech** OFF for a baseline. Keep the same other modes enabled in each test.
2. Open the menu with Minus, open the Modes page and enable **Performance gauge**. After about five seconds, a compact card shows sampled average and
   peak local-observation time in microseconds. Full `PERFORMANCE` reports are
   written to `sd:/ultimate/kirby_assist/performance.log` with ranked bars, calls and rays.
   `RUNTIME HEALTH` reports in the same file cover mapping, menu, rendering, saves and logger pressure.
3. Enable one feature, repeat its actual trigger for 20–30 seconds, then turn it
   OFF and repeat the baseline. Ignore the first five-second window after a
   change because it may contain both settings. Repeat idle, triggered and OFF
   tests; an enabled reaction reader may legitimately keep cheap status checks.
4. Compare **All local observation work** average/peak and the relevant row's
   sampled calls/rays. Tech should have no prediction calls/rays outside eligible
   tumble; disabled grab mode and an unassigned ledge-trump built-in should have
   none. Floor rays can still belong to other enabled positioning or movement
   helpers. OFF-by-default modes
   do not silently re-enable through old SD settings.
5. Hide the menu with Minus to compare actual gameplay, then turn **Perf** OFF
   to compare instrumentation overhead. Test silent separately: it excludes the
   diagnostic logger and performance instrumentation entirely. Use the in-game
   master OFF state as a vanilla-input comparison, with the same game workload.

The gauge samples one in 16 accepted local observations; it can miss isolated
spikes. Bars show elapsed time relative to measured observation work, **not system
CPU utilization**. Rows overlap where one measured operation calls another; do
not add their percentages. The original observation gauge remains separate from the new mapping, mod-draw,
save-worker and logger timings. Neither covers the entire game renderer, GPU or system. Zero sampled calls can mean inactive or not
sampled. Frame/FPS and CPU/GPU/fan monitoring provide the separate system picture.
1,000 microseconds is 1 millisecond; a 60 FPS frame lasts about 16.67 milliseconds,
shared with the rest of Smash. Temperature takes time to stabilize and does not
identify a particular feature. Timing windows reset on gauge toggles/session end; cumulative health events and
high-water marks persist until the plugin restarts. Stale observation cards clear
after observations stop.

## Runtime health and the two logs

Diagnostic builds keep two files under `sd:/ultimate/kirby_assist/`:

- `performance.log`: enabled Perf observation gauges and runtime health reports.
- `status.log`: startup, configuration, action/calibration evidence and other diagnostics.

Both files reset at plugin startup, so copy them before restarting. One existing
background worker handles both buffered files; gameplay hooks never write logs.
Normal silent builds exclude all these counters, timers, files and the logger worker.

With Perf ON, the health report appears roughly every five seconds even if no
observation gauge was produced. It includes:

- Sampled mapping hook, menu-input and mod draw-callback times; mapping includes
  original game mapping and nested observations, not just mod overhead.
- Menu-text rebuilds, publish retries, recipe settings resolutions, profile
  switches, state resets, observation passes/reuse and opponent roster scans.
- Retained profile/editor-recipe/draft counts and estimated owned payload bytes,
  plus current/peak menu-view payload. These are recomputed on revisions, not
  by walking every cached recipe on each idle input poll.
- Pending saves, current/peak active save workers, completions/failures, worker
  duration and the reserved 2 MiB stack per active worker.
- Logger queue depth/high-water mark, dropped messages and formatting/flush times.
- Maximum draw-callback interval and counts at least 25/50/100 ms apart. These
  gaps include pauses, transitions and missed callbacks; they are not FPS.

Hot-path durations sample one call in sixteen. Sparse rebuilds, saves and log
flushes are timed individually. Counters are fixed-size; timing updates use
nonblocking locks and report skipped updates on contention. Formatting runs in
the logger worker. The new diagnostics do not change action delays, observation
frequency, tech scheduling, movement or game physics. Instrumentation has its
own diagnostic-only overhead; compare Perf ON/OFF in the same scenario.

Capture a fresh-match baseline, then switch fighters repeatedly, return to an
already-used fighter and reproduce the stutter. Keep Perf enabled for each
fighter being compared. Compare the cumulative counts and `(+window delta)`
values between reports. Opening new fighters/recipes intentionally retains their
settings; revisiting the same recipes or sitting idle should not continually
increase retained recipe/draft counts. Rebuilds should settle when navigation
stops. Active save workers should return to zero after completion; queue growth,
drops, large flush times or rising callback times indicate where to investigate.
State resets and callback gaps outside battle can be normal, so compare the same
battle phase. Memory values exclude allocator/BTree-node overhead, immutable
startup catalogs, ImGui/game/SDK allocations and total system RAM. They help
identify retained growth; they cannot by themselves prove an arbitrary heap leak
or identify a thermal/GPU issue.

## External monitor controls

The classic Status Monitor exit shortcut is **L+Down+R3**, according to its
[configuration documentation](https://github.com/masagrator/Status-Monitor-Overlay/blob/master/docs/config.md).
Kirby Assist ignores that chord for menu toggling. Diagnostic Training hands
that chord through until a fresh Minus resumes our tools. Ordinary Minus still
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
D-pad directions now use user-configured bindings; keep monitor chords distinct
from any buttons assigned to sequences or modes.
Ultrahand exposes **Key Combo** in settings and per-overlay launch combos via Y,
as documented by [Ultrahand](https://github.com/ppkantorski/Ultrahand-Overlay).
Plus may pause Training/hand it input; close that menu and resume us with Minus.

Use a compact stats display at a low refresh rate for CPU/GPU load, temperatures,
fan and memory, positioned away from our top-edge menu. Exit the interactive overlay menu before testing game controls.
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

## Labelled latency and workload trials

Diagnostic builds add **Modes → Test delay label**. It starts **Unset**; Left/Right
cycles manual labels for external delay **0–30 frames**. This field does not
change Training Modpack, introduce delay, compensate inputs or read its private
settings. Match it manually to the external delay you selected. Label changes
start a new Perf epoch so five-second performance windows are not mixed across
trials. The label is fighter-specific and retained in memory until restart.

For a quick comparison:

1. Set the Training Modpack input delay, close its menu and resume this assist
   with Minus if Training's sticky handoff is active.
2. Set the matching **Test delay label** and turn **Performance gauge ON**.
3. Close the menu; keep fighter, stage, selected recipe and enabled modes the
   same. Keep the first setup/editing window separate from later windows.
4. Repeat the same interaction for roughly 15–20 seconds at each chosen delay,
   such as 0f, 4f and 8f. Include both successful and failed neutral-getup attempts.
5. Keep the two existing files: `performance.log` and `status.log`. Both now
   include `TEST CONTEXT` / Perf-epoch markers for grouping the trials.

Training Modpack's reviewed implementation queues mapped controller inputs.
Hook order can put its delay queue before or after our mapping acknowledgment;
`MAPPED ACK LIMIT` explicitly records that limitation. Our input trace compares
request, mapping acknowledgment and observed attack stamps, with wall milliseconds
and target-animation frame deltas separately. Native deltas are unknown when the
opponent's status/motion/rate changes. An acknowledgment is not proof of final
engine consumption; neither time measurement is a network ping or measured
online delay. Fixed Training delay also does not reproduce online packet jitter,
stalls, callback scheduling or total system workload.

### Reading the summaries

`performance.log` now starts each health window with `TRIAGE`, `REVIEW [CODE]`
and `RATES`. Flags are investigation hints, never automatic proof of a cause:

| Code | Evidence to inspect |
| --- | --- |
| `INPUT_EXPIRY` | One or more episodes where the 35ms freshness guard suppressed stale input |
| `SEQUENCE_GAP` / `SEQUENCE_CLOCK` | A recipe cancelled after missed native updates, an invalid clock or timeout; inspect its `SEQUENCE END` |
| `PUBLICATION` | An observation was superseded/session changed before publishing; transitions may explain it |
| `EVALUATION_RATE` | More than 1.5 full local evaluations per draw callback; compare the same scene at 0f, since callbacks are not simulation frames |
| `MENU_REBUILDS` | Frequent menu text rebuilding; expected during edits, suspicious if untouched |
| `MAPPING_TIME` / `DRAW_TIME` | A sampled call exceeded 2ms; mapping is inclusive of vanilla/nested work, drawing excludes the rest of the renderer |
| `LOGGER_COST` / `LOGGER_PRESSURE` | A diagnostic batch exceeded 5ms, a queue approached capacity, or messages were dropped; instrumentation may affect evidence |
| `SAVE_WORKER` | Multiple active workers, or an active worker across multiple windows without completion |
| `RETAINED_GROWTH` | Estimated retained payload rose for three windows without changed item counts, text rebuilds or profile switches; not a complete heap/leak detector |
| `CALLBACK_GAP` | A draw-callback gap exceeded 100ms; pauses/loading/stalls are possible explanations, not an FPS measurement |

The rates distinguish full local evaluations from preliminary observation passes,
duplicate skips, roster scans and draw callbacks. Do not add overlapping inclusive
timing rows or treat them as system CPU utilization/temperature/GPU load. A quiet
triage window rules out none of the unmeasured system work. Thresholds are review
thresholds, not gameplay limits and do not change any detector's cadence.

All new counter fields, timestamps, summaries and native termination telemetry are
diagnostic-only. Fixed counters stay on existing paths; formatting and reporting
use the existing worker and bounded queue. No new hook, game-state query, worker,
file or network output is added by these improvements. Silent logging remains
compiled out. No timing/detection/physics adjustment is made from a review flag.
## GPU-crash isolation

A GPU error followed by a system-thread abort does not identify the editor,
save worker, Deluxe or ImGui as the cause. Keep the matching Atmosphere crash
report and error reports; a normal CPU-side mod timing sample cannot diagnose
GPU command corruption or clock stability.

The optional `deployment/troubleshooting/deluxe-vanilla-no-overclock.toml` preset
sets Deluxe menu/offline/online render profiles to Vanilla and disables Deluxe's
overclock requests. Normal Kirby upload tools do not install it. With Smash
fully closed, back up an existing `/ultimate/ssbu_online_deluxe/config.toml`
before using the preset there. Restart Smash and retry the same Training setup.
If earlier elevated clocks may have survived a crash, reboot before comparing.
Other clock plugins/sysmodules are independent; this does not guarantee stock
clock speeds or a fix. Restore the previous file afterward, or remove only the
test config if no file existed before. Do not change profiles, plugins and
recipes together during the comparison.

## Training-to-online crash source review

The observed save-associated main-thread null-pointer reports and the later
GPU/SDK-abort report have different signatures; file validation alone cannot
establish their causes. Fresh-boot Elite working without entering Training is
useful transition evidence, not proof that sequence-file contents caused the GPU
fault. No Deluxe configuration change is part of the current source investigation.

In the inspected shared `imgui-smash` source (revision
`bb47b205ed0be2fb864269ee2d37fc886b81221e`), the renderer resizes buffers in
`imgui-smash/src/imgui_backend/imgui_impl_nvn.cpp`. It finalizes old vertex/index
buffers there, while `MemoryBuffer.cpp::Finalize` frees backing memory before
finalizing its pool/buffer objects. No GPU completion wait was found around that
resize path. Shared command-buffer submission and reuse also need review.
This is a candidate lifetime/synchronization hazard when editor geometry changes;
it is not yet a proven match to the installed host or the crash.

The ImGui host is a separate NRO shared by Kirby Assist and Deluxe. Rebuilding
Kirby alone does not patch it. A renderer fix must be matched to the installed
host source and checked for correct resource lifetime/queue synchronization,
rather than guessing at hooks, disabling clocks or claiming that a valid saved
file proves the renderer safe. Reading the installed host over FTP remains
read-only; applying any replacement requires separate explicit approval.

### Installed-host verification and prepared patch

The read-only installed host copy identifies ImGui `1.92.7 WIP`, has 1,261,568
bytes, and SHA-256
`61e19e1b593826b1216228c6536cf2603b512154a8fecc548203e245bb86ed3e`.
It matches the earlier saved host binary byte-for-byte. Disassembly of its resize
calls and finalizer confirms the same backing-allocation-first release order at
NRO-relative function offset `0x14c80`; this is no longer only a source-checkout
observation. This verifies that specific pattern, not complete build provenance.

A narrowly scoped source correction is prepared at
`patches/imgui-smash-buffer-release-order.patch` in the project repository.
It finalizes the buffer, then its pool, then frees backing memory. A host C++
lifetime-contract harness rejects the old order and accepts the new order;
`git apply --check` passes against the reference checkout. No new hooks,
per-frame work, clock/profile changes or Switch writes are involved.

The patch is not an installable NRO, and rebuilding Kirby alone does not include
it. No shared-host build/deployment has been performed. The absence of GPU
completion synchronization around resize/reuse still needs investigation, so
the Training-to-online GPU crash is **not confirmed fixed** by this source patch.

## In-match work reuse

The gameplay adapter shares roster IDs and lazy fighter facts within each
accepted observation. Roster reuse holds eight copied IDs, allocates no heap
memory, and ends at callback exit. Every consumer still validates live object
identity and team; moving fighters and floors are observed again next time.

Selected sequence settings reuse the existing match-owned menu resolution.
Fighter defaults are reapplied when the fighter/choice/recipe revision changes
or when no resolved menu is available, rather than rebuilding the same profile
for every controller poll. Automatic-grab close/far tuning resolves once per
observation. Floor probes remain live and are requested for relevant movement
or shield/contact candidates; unrelated rolls do not request grab-approach rays.
The retired configured grab-dodge follow-up no longer builds an unused attacker
clock/context, and the Training compatibility path does not copy identical
settings or query the scene solely for an identity conversion.

Tech prediction, mash, DI/SDI, grab avoidance, ledge acquisition/defense/trump,
button sequences retain their native observation/input cadence. There is no
new frame skipping, polling throttle, wall-time catch-up, network query, physics
change or persistent opponent-state cache. Same-observation floor/fact reuse
cannot explain total console heat or GPU load; compare measured costs on device
before assigning a thermal improvement to these changes.
