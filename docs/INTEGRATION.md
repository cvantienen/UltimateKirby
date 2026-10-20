# Raw controller build: install, behavior, and verification

## Current status

This build contains actual Switch hooks and raw controller-memory stick/button writes. It is no longer the empty plugin shell described in the project's first version.

**It compiles, but has not been tested on a physical Switch. Treat it as an on-device test build, not a confirmed fully working release.** A successful build cannot verify the latest executable's internal layouts, actual controller routing, or auto-tech timing. The diagnostic build's status log makes the next verification concrete. The silent build intentionally has no plugin log.

This is a direct on-Switch mod. No extra controller hardware, Wi-Fi transport, or computer during play is required.

The target is an actual Switch, the selected player input slot, and ZR mapped to Shield. Record the exact installed Smash version when reporting a problem. The unique instruction-signature search avoids assuming an old mapper address, but does not validate every internal structure for an arbitrary game version.

## What the buttons really are

The output hook runs immediately before Smash's controller-mapping function. It temporarily overlays the controller's raw held, previous, just-pressed, and just-released button words. A is bit 5, B bit 6, and ZR bit 11 in this internal structure; these are different from Nintendo HID button positions. `native_input.rs` explicitly translates A/B/X/Y/L/R/ZL/ZR requests to these internal bits.

Mash changes left-stick floats and matching direction bits. With takeover enabled it clears physical action buttons and C-stick input during the cycle; tech still requests only ZR. Manual turbo/ledge controls share the stick overlay and reserve their D-pad triggers. Connection fields stay intact. The original mapper consumes the modified trace using the player's normal profile, then the hook restores the physical controller structure so it is not reused as physical input on the next update. The live output does not depend on the SDK's single-state versus bulk HID-read path.

Smash then handles those bits through its own normal input code, including your controller mapping. This is why ZR must actually be assigned to Shield. There is no forced fighter-status change, direct action-command injection, clatter-timer reduction, or bypass of game lockouts.

These are **software changes to raw controller memory consumed by the game**. They are not actual USB/Bluetooth packets from a controller. There is no claim of electrical identity, universal indistinguishability, or identical behavior across other plugins that read input at a different stage. Hardware equivalence has not been measured on your device.

The controller prefix and mapper hook are checked against the [Training Modpack input definitions](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/common/input.rs). The ownership-chain accessor follows [HDR's pinned implementation](https://github.com/HDR-Development/HewDraw-Remix/blob/a8f98eace1c6a7b45c88cb3f0ac1ef05426624dc/dynamic/src/ext.rs).

### Why the previous build could do nothing

The old observer treated `ControlModule + 8` as a controller index. The upstream accessor instead follows `ModuleAccessor + 0x48 → ControlModule + 0x118 → +0x58 → +8 → index at +8`. Reading the wrong field prevented the selected fighter from publishing mash/tech requests. A regression test now constructs the ownership chain with a deliberately different value at the former address.

The old live path also required a particular single-state SDK HID hook to run; output now happens directly at the mapper. The tech policy no longer adds a release delay when the previous game-visible ZR state was already released. There is no game-mode or Ready/Go restriction; active local-human Kirby selection, pause/stop handling, and legal scenario detection still apply.

## Installation with an existing compatible Skyline setup

1. Fully close Smash.
2. Choose `releases/kirby_assist-silent.zip` or `releases/kirby_assist-diagnostic.zip`. Copy its `atmosphere` and `ultimate` directories to the SD-card root, preserving the directory structure. Install only one; see [build variants](BUILD_VARIANTS.md).
3. Confirm there is only one `libkirby_assist.nro` installed. Do not leave an older differently named copy of this mod in the plugins directory.
4. Use local controller 1 (`player_index = 0`) with the required control mappings. Physical Npad identity selects the controller; its mapper slot is followed dynamically. Controller style is unrestricted.
5. Start Smash and select your fighter. The code has no character or training-mode activation check. Move timings remain tuned for Kirby.

The ZIP contains:

```text
atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libkirby_assist.nro
ultimate/kirby_assist/config.txt
INSTALL.txt
VALIDATION.json
README.md
README_BEGINNER.md
INTEGRATION.md
BUILD_VARIANTS.md
TURBO_CONTROLS.md
LEDGE_TRAP.md
ANALOG_MASH.md
```

It does not include Skyline, Atmosphere, other plugins, game data, or an installer that changes your Switch. Skyline documents its general [plugin installation directory](https://github.com/skyline-dev/skyline).

## Settings

Edit `SD:/ultimate/kirby_assist/config.txt`, then restart Smash. No Rust editing is needed.

| Setting | Default | Meaning |
| --- | --- | --- |
| `enabled` | `true` | Install/enable the live hooks. `false` skips hook installation at startup. |
| `auto_mash` | `true` | Request mash inputs while the game reports a positive clatter timer. |
| `takeover` | `false` | During analog mash, suppress physical action buttons and C-stick; during tech, own only ZR. The supplied config enables it. |
| `auto_tech` | `true` | Request raw ZR on predicted contact, then throttle retries with the configured cooldown. |
| `tech_debug` | `false` | Diagnostic build only: log native gate reasons and ZR edges. Its supplied config enables this; silent compiles the logger out. |
| `player_index` | `0` | Local physical controller number, 0 through 7. Its mapper slot is discovered dynamically; this is not the Arena battle position. |
| `tech_lookahead_frames` | `3` | Collision-ray prediction horizon, 1 through 5; the included config uses 5. |
| `tech_cooldown_frames` | `40` | Minimum observed input updates between automated ZR presses. |

The sample config is `deployment/ultimate/kirby_assist/config.txt` in the project root. Missing config uses the table defaults. The supplied config enables `takeover` for stick-only mashing and sets lookahead to five. The removed `alternate_mash`, `mash_buttons_per_frame`, and `mash_interval_frames` keys are ignored if present in an older config. Diagnostic packages enable `tech_debug`; silent packages disable it and cannot log even if it is changed to true. Invalid or unknown keys stop installation and write a diagnostic, rather than silently using mistaken settings. Keys are read once at startup.

Auto mash supplies the left stick, not an Attack-button binding. Auto tech always requests raw ZR, so ZR must be assigned to Shield. Ledge responses retain their A=Attack/R=Grab requirements.

## Directional turbo control

Hold D-pad Left/Right for rapid crouch/release with slight left/right movement. Releasing returns current physical stick input; both held cancels movement. It runs only during normal grounded movement for the selected Kirby. See [settings, implementation, and device checks](TURBO_CONTROLS.md). Both logging variants include it.

## What each tool does

### Auto mash

The observer checks `ControlModule::get_clatter_time(object, 0)`. A positive value selects the mashable Clatter scenario. The live policy emits **Right → Up → Left → Down**, one normalized full-strength direction per distinct non-stopped native progress sample (animation or remaining clatter timer). It emits no mash action buttons. Duplicate observations reuse the direction; exit, disable, reassignment, and clock reset cancel/restart the cycle. Hitstop publishes no mash stick and freezes progression.

With the supplied `takeover = true`, physical A/B/X/Y/L/R/ZL/ZR and C-stick input are suppressed during the cycle to avoid mixing action inputs with stick mashing. With takeover false, physical buttons/C-stick remain available; the automated left stick still runs. At scenario exit, the mapper uses the current physical sticks/buttons. Mash has priority over D-pad stick tools while trapped; tech keeps its existing priority and ZR timing.

The original mapper consumes the normalized floats and consistent directional bits, after which the physical controller memory is restored. The mod never subtracts from the clatter timer. The game decides how much each input counts. Mapper updates are not a verified simulation clock, and faster actual escapes require device comparison. See [ANALOG_MASH.md](ANALOG_MASH.md).

### Auto tech

The observer checks airborne damage-fly/reflect/fall states and the damage flag disabling passive/tech behavior. For an already associated fighter, it refreshes prediction before the same mapper invocation consumes the requested button. This removes v2's extra mapping-update delay. The game callback remains responsible for initially discovering the controlled fighter.

Collision prediction sweeps along velocity up to the configured horizon, with three probes for each approaching floor/wall/ceiling direction. Offset floor/ceiling rays and several wall heights approximate the fighter's footprint rather than relying on a single center line. The supplied horizon is five updates instead of v2's three. These are fixed approximate offsets, not the game's exact collision shape.

Prediction no longer waits for the current passive transition term to be enabled: an early physical-style ZR press can reach the game's normal input buffer before contact. It still excludes the native flag disabling passive behavior, and the game retains eligibility, lockouts, and untechable outcomes. The mod does not force a tech status. When a predicted opportunity is ready, it requests ZR and starts the configured cooldown. If an opportunity begins while cooling down and remains present, it can now trigger when the cooldown finishes. A sustained opportunity can be retried after the configured cooldown; requests are not spammed every update.

This is a velocity-based collision heuristic, not a perfect simulation of future movement or the fighter's collision shape. Moving platforms, slopes, unusual collision geometry, special grabs, hitstop, buffering, and polling latency can affect the result. The game still decides whether the input results in a tech. No mod can make a raw ZR press guarantee a legal tech when the game rejects that input.

The 40-update cooldown is the mod's request-throttling setting. It is not presented as the game's precise lockout duration and does not overwrite the game's timers. With takeover enabled, ZR is requested immediately if its previous game-visible state is released. If ZR was held, the policy first requests a release update, then presses on the next observed opportunity update. With takeover disabled, a physically held ZR prevents a fresh automated press. A one-update opportunity can therefore be handled immediately when ZR is released, but may still be missed when a held ZR needs releasing first.

## The actual live architecture

```text
Game ControlModule query hook (returns original command flags unchanged)
    discovers human fighter IDs and their controller ownership slots
    → reads native state and publishes copied input proposals
                               │
                               ▼
Raw controller-mapping hook
    captures fresh physical buttons; advances selected input-update counter
    → binds the selected physical controller to its current mapper slot
    → refreshes that slot's callback-discovered, revalidated fighter
    → overlays left-stick floats/direction bits and raw button history
    → calls original mapper → restores physical controller structure
                               │
                               ▼
Smash's normal mapping, buffering, eligibility, and fighter logic
```

`AssistRuntime` and the pretend-adapter examples remain useful for learning and host testing. The live path instead uses `RawAutoControls`/`AssistEngine` and overlays native stick values/button words. It does not rebuild the native controller from the core's limited `ControllerFrame`.

The selected mapper's invocation counter advances on each matching connected
controller-mapping call. Live fighter command callbacks validate identity, query
native observations and publish copied proposals. They also cache human fighter
IDs by input slot, even before the selected controller's mapper is known. The
mapper re-resolves and validates only a recently discovered fighter ID, then
refreshes its state before mapping, as the working 1.0.4 path did. It never
enumerates the fighter manager's roster.

The mapper selects physical Npad identity instead of assuming mapper index 0.
The shared observation path validates active identity,
controller ownership, and human control before publishing. Character kind and
roster enumeration do not gate independent controls. A 250 ms observation gap
resets the route, object association, overlay edge history, and tool state.
Session tickets reject in-flight observations from an earlier epoch. Inactive
objects and accessor identity mismatches are rejected before controller-chain
access. Controller samples allow 250 ms of scheduling delay; attack proposals
still expire after 35 ms. A mapper poll advancing during a native query no longer
discards that observation; publication uses the latest mapper tick and rejects
superseded native observations. Physical input passes through
when no fresh eligible snapshot exists. Binding consumption uses the recent human
route rather than the shorter attack-proposal expiry. A reentrancy guard prevents
nested native observations during synchronous refresh. Native callbacks bootstrap
discovery; input-callback timing and the first actionable frame need hardware
measurement. These observed updates are not a verified global simulation clock.

Player routing follows configured physical controller → actual mapper slot →
human fighter ownership chain. CPU fighters are excluded. Character and battle
mode do not gate activation. Current move timings and ranges remain tuned for Kirby.

## Read the status log before changing code

Use the **diagnostic** variant for this section. Silent does not create/truncate or append to this log; an unchanged older log may remain on the SD card.

After launching Smash and entering a match, inspect:

```text
SD:/ultimate/kirby_assist/status.log
```

| Message | What it establishes |
| --- | --- |
| `NOT INSTALLED: ... signature missing or ambiguous` | This executable/mod layout is unsupported by the scanned hook. No hooks were installed. |
| `NOT INSTALLED: FighterManager ...` | The required singleton could not be resolved. No hooks were installed. |
| `invalid config line ...` | Fix that line and restart Smash. |
| `hooks installed` | The scanner succeeded and hook installation was requested. This is not proof of correct gameplay. |
| `mapper observed index ...` | Shows the controller ID, style, connection, and configured ID; useful for correcting routing. |
| `Pro Controller npad ... mapped ... raw mapper reads confirmed` | The configured Pro Controller has reached the mapper hook. |
| `fighter controller ownership chain unavailable ...` | No valid index was resolved yet; persistent absence of fighter confirmation needs investigation. |
| `observing human fighter ...` | The selected controller's fighter is reaching the observer, regardless of character. |
| `scenario: mashable=true ...` | The native clatter timer is positive. |
| `predicted_tech_collision=true` | The heuristic found an eligible nearby collision. |
| `raw controller mapper button overlay confirmed` | The raw button trace was changed before mapping. This does not prove the game accepted an action. |

The diagnostic first line identifies `Kirby Assist diagnostic build v17`. An earlier version identifies an older plugin. An unchanged old file is expected when switching to silent.

The log resets at launch. It records scenario changes and first confirmations, not a full per-frame trace. A startup crash may occur before a usable log if another mod/runtime is incompatible; a signature match cannot validate every ABI field.

### Diagnose an online ledge-jump nair

Ledge-jump cover supplies neutral jump+attack together, using the game's normal
short-hop attack input. It no longer waits for a second delayed observation to
begin sending A. Once committed, it finishes from Kirby's own state, independent
of target lookup or aerial-readiness flags. Normal-getup and shield-hop nairs
share that follow-through. None of these paths reads the online input queue.

In a newly built diagnostic plugin, `short-hop mapped pair` reports whether the
original mapper produced both JUMP and ATTACK with neutral stick. A missing bit
points to mapping/input generation; both bits present confirms mapping only.
The following ledge observations must show native status `0x33` with motion
`0xc3a4e2597` (`attack_air_n`) to establish nair acceptance. Hit confirmation is a
separate event. Keep the full request/mapping/native-state sequence when comparing
offline and Arena failures. The 35 ms request watchdog is a wall-clock freshness
limit, not a game-frame timer or a measurement of network delay.

Source edits do not update an installed NRO. Build and install the diagnostic
variant only when requested, then restart Smash before comparing results.

### Diagnose the reported spike test

A spike is not automatically a legal tech opportunity. In Ultimate, a meteor hit on an already-grounded standing opponent cannot be floor-teched. An airborne hit into the floor is a different case and can still be untechable depending on the native collision/knockback rules. Compare a manually successful ZR tech in the exact same setup before treating the failed automation as a timing failure. [SmashWiki tech mechanics](https://www.ssbwiki.com/Tech)

In the diagnostic build with `tech_debug = true`, the mod queues state-change diagnostics after the mapper consumes the input. A bounded background writer batches console/SD writes; the gameplay callbacks do not perform that I/O. Ledge wait snapshots are capped at two per second; action changes and input edges remain immediate queue events:

| Diagnostic | Meaning |
| --- | --- |
| `native_passive_disabled` | The checked native flag blocks the prediction; the mod leaves it unchanged. |
| `not_airborne` | The current detector requires an airborne tumble state. |
| `outside_supported_tumble` | Damage/knockdown status is outside the supported tumble range. |
| `hitstop_wait` | No late-hitstop tech buffer is ready; cooldown history is retained. Late damage-hitstop may predict and buffer a tech. |
| `no_collision_predicted` | The probes did not find a surface in the prediction horizon. |
| `collision_predicted` | Contact was predicted; includes estimated contact time, hitstop remaining, learned delay, request mask and scheduler cooldown. |
| `tech ZR applied` | A raw ZR press edge was supplied to the mapper during a predicted tech opportunity. This does not prove a successful tech. |
| `tech mapped: fresh GUARD edge confirmed` | The original mapper produced a fresh shield press. Our cooldown starts here; native tech acceptance remains separate. |
| `tech_status_observed` | The fighter entered a native tech status. This does not establish whether the mod or a human caused it. |

If an airborne collision that you can tech manually still fails, provide the full log plus the stage, percent, attack, and whether Kirby was grounded or airborne when hit. Do not shorten the cooldown blindly: additional shield presses can interact with native lockouts. To stop the extra logging after testing, set `tech_debug = false` and restart.

## On-device acceptance checks still required

First verify mash with a normal mashable grab/bury state while not physically holding A. Compare escape behavior with `auto_mash = false` and `true`, restarting for each configuration. Check the status log for both scenario detection and raw mapper button overlay.

Then isolate tech: set `auto_mash = false`, leave `auto_tech = true`, and use a repeatable tumble into a flat floor. Verify an actual tech occurs, ZR does not remain held afterwards, and ordinary aerial movement does not receive unrelated air dodges. Check wall/ceiling cases separately. A prediction log without a successful tech means timing, eligibility, or collision geometry still needs investigation.

Also verify movement preservation, held A/B/ZR, pause/resume, death/respawn, match exit, controller disconnect/reconnect, and another player's isolation. Compare raw native samples from physical A/B/ZR presses against generated ones if exact game-visible trace equivalence is required; no such capture has been taken in this environment.

To disable the mod, set `enabled = false` and restart, or remove its `.nro` while Smash is closed. Keep your existing Skyline setup.

## Source verification references

These describe the ABI inputs used to write this integration; they are not proof of compatibility with every game version:

- [Skyline-smash declarations and symbol names, revision d2ca90d](https://github.com/ultimate-research/skyline-smash/blob/d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435/src/cpp.rs). The code uses a narrow set of these foreign declarations in `src/game_ffi.rs`.
- [Skyline-smash status/transition constants at that revision](https://github.com/ultimate-research/skyline-smash/blob/d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435/src/lua_const.rs).
- [Controller, wrapper, and ControlModule prefix definitions, Training Modpack revision 077a8e4](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/common/input.rs).
- [Input-mapper signature bytes at that revision](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/common/offsets.rs).
- [Input-mapper hook signature at that revision](https://github.com/jugeeya/UltimateTrainingModpack/blob/077a8e43b11d1f61ce57169156fb82e92e784c79/src/training/mod.rs).
- [ModuleAccessor layout showing the ControlModule pointer](https://github.com/HDR-Development/HewDraw-Remix/blob/pre-release/fighters/common/src/function_hooks/mod.rs). The prefix places ControlModule at `+0x48`; the index is resolved through the ownership chain, not read directly from ControlModule.

Native layout reads are isolated in `src/live.rs` and `crates/kirby_assist_core/src/native_input.rs`. The mapper is installed only after one aligned instruction-signature match is found. There is no hard-coded-address fallback. Other internal layouts are sourced as described above and remain a compatibility risk until checked on the target executable.

## Build from source

Run from the project root (the folder containing the main `Cargo.toml`). The Skyline CLI and its Switch toolchain must already be configured. The default silent build command is `cargo skyline build --release`. Add `--features diagnostic-logging` for diagnostic. Use `python3 tools/build_variants.py` to build/package both separately; see [BUILD_VARIANTS.md](BUILD_VARIANTS.md).

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
cargo clippy --manifest-path crates/kirby_assist_core/Cargo.toml --all-targets -- -D warnings
cargo skyline clippy
cargo skyline build --release
```

The release plugin is `target/aarch64-skyline-switch/release/libkirby_assist.nro`. If the user's existing Cargo cache has permission errors, an isolated `CARGO_HOME` in a writable temporary directory can be used for these Skyline commands; changing the project source is not needed to work around that cache problem.

## Rebuild the SD-card packages

Use the helper so one build does not overwrite the other variant's release files:

```sh
python3 tools/build_variants.py
```

It builds both optimized variants and produces `releases/kirby_assist-silent.zip` and `releases/kirby_assist-diagnostic.zip`, plus separate copyable folders. For one variant use `--variant silent` or `--variant diagnostic`. Both install to the same plugin path; replace rather than rename/load both. The helper checks binary logging markers and package contents and never installs to your Switch.

See [build variants](BUILD_VARIANTS.md) for individual Cargo commands, the optional isolated Cargo cache, exact logging scope, and hardware checks. The old numbered candidate ZIPs are previous builds, not the new dual-variant outputs.

## Opponent ledge observations (source addition)

The mapper enumerates fighter-manager entries and tracks one nearby non-team opponent through its ledge option. Hold Y for conditional automatic responses with manual capped spacing; hold L3 separately for turbo creep/teabag. Releasing each button disengages its own tool. Y is consumed before jump mapping. D-pad Left/Right now request short-hop/horizontal-air-dodge slide attempts. The supplied mode does not navigate or turn Kirby for you. Mash/tech retain priority. See [LEDGE_TRAP.md](LEDGE_TRAP.md) for controls and limits. These new bindings are source-only and have not been built. The older local builds under `target/manual-spacing/` and existing release packages retain their previous controls.

Observations now include target motion kind/frame/rate/end, native previous status, invincibility expiry, attack presence, root positions/velocity, Kirby's velocity and hit flag, shield history, catch readiness, and floor support. All game queries remain outside STATE's mutex. Four added read-only declarations match the pinned primary bindings: MotionModule rate/end_frame and AttackModule is_infliction_status/is_attack. The HIT mask is 0x2. No status/timer/velocity writes were added.

The default stance is now 8 units inward, with a short momentum lookahead. Acquired targets use separate follow-up limits. Root displacement over the same advancing animation supplements missing kinetic travel; repeated polls of one animation frame reuse the estimate. Rate-adjusted motion deadlines feed a linear active-window/protection model. This is not a hurtbox simulator.

Normal get-up pursues range before preparing down tilt or optional downward-angled forward tilt. Jumps are remembered even while Kirby is busy; native previous status catches a short missed transition. Up tilt predicts the rising target across active frames. Roll grabs can be requested before status exit when known protection expiry and predicted reach support standing/dash/pivot startup. Unknown expiry requires actual vulnerability. Tilt whiffs now retry after recovery, within three observed animation attempts; native hit infliction completes the attack phase. v17 returns to the original ledge stance after recovery, even if the hit launched the target out of tracking range. Confirmed captures request a horizontal throw toward the acquired ledge anchor in catch-wait when `ledge_throw_outward=true`. No throw is guessed without that anchor. Hanging checks use the actual status range, so ordinary idle no longer cancels the prepared normal-getup punish. Native previous-status recovery now includes roll and normal get-up. The shield-grab predictor includes remaining shieldstun and future protection/reach; tilt retries need a release without an additional timer.

Shield retains its continuous ZR hold and release-aware R buffer. Pushback pursuit commits through guard-off without re-raising shield. A witnessed get-up attack whose hitboxes clear can also receive a whiff punish; empty startup alone is insufficient. Hitbox gaps, later multi-hits, and multi-opponent hit confirmation remain limitations. Native action acceptance and first hittable collision need Switch validation.

Diagnostic snapshots add target motion/rate, previous status, observed speed, selected grounded move, and own hit confirmation. Detailed snapshots can update on target animation frames when tech_debug is true. Silent excludes their construction. Both variants keep the existing analog mash, tech, and turbo behavior. Copy the updated config with the new plugin; an old explicit stance=6 overrides the new default.

Controller routing now follows the working 1.0.4 physical-controller selection:
`player_index=0` selects the Player 1 Npad and follows its actual mapper slot.
The legacy `npad_id` key remains an alias. Controller style and character are
unrestricted. Unlike 1.0.4, bootstrap discovery does not enumerate fighters in
the mapper: native callbacks cache human routes and roster IDs, and synchronous
refresh revalidates those IDs. Install the rebuilt plugin before testing.

## Online warm-up transition hardening (source update)

The reported symptom is a crash between online warm-up and the fighter reveal.
The previous mapper path could enumerate partially initialized fighter entries
or follow a cached fighter's module pointers during that transition. This audit
removed fighter queries from the mapper, added session invalidation, moved
active/accessor identity checks ahead of controller
pointer traversal, and limited ledge opponent scans to held-Y operation. Invalid
roster counts and stale fighter-entry identities now fail closed. Nonfinite poses
do not reach native ground rays, and missing motion metadata does not invoke a
protection-frame query with an invalid motion hash. Pointer-chain
null/sentinel/alignment checks do not establish memory lifetime; that is why
native callbacks and the lifecycle gates are also required. The later
mode-compatibility fix stopped using the global ready-go flags as hard gates:
the mapper now samples Player 1 independently of those flags or manager roster
readiness. The native callback validates active identity and human input ownership.
Roster enumeration is used only for ledge opponent selection. The CPU/human
information query also runs during a revalidated pre-map refresh. Observation gaps expire
the session across loading transitions.

Diagnostic stdout errors are ignored instead of using `println!`, whose output
failure could panic across a native hook. The diagnostic performance follow-up
moves file/console writes into a bounded background logger; see
[diagnostic performance](BUILD_VARIANTS.md#diagnostic-performance). File I/O errors remain nonfatal.

Host tests cover warm-up/loading/match session tickets and invalid controller
chain addresses. Both root compilation paths are checked without generating an
NRO. There is no crash report or on-device verification of this source update,
so these are fixes for identified hazards, not proof of the exact crash cause or
a guarantee of online stability. The patch does not bypass network validation,
change simulation states, or establish network compatibility on every game/mod
combination. Rebuild and install the patch before retesting the transition.
