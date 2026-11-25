# Shared observations and game progress

The live adapter creates one `FrameReads` inside each validated local-fighter
observation. All assistance policies share this instance. It lazily copies
status, situation, motion/frame/rate, position, total velocity, facing, team,
stop, previous-status, animation-end, capture-link and fighter-kind facts into
sixteen fixed identity slots.
Repeated field reads reuse their first value. A last-identity cursor avoids
rescanning the slot table for consecutive fields of the same fighter. No heap allocation, new hook,
worker, network activity or eager full-roster read is added. Unexpected extra
identities read through without evicting cached fighters.

The snapshot belongs to **one callback**, not one mapper tick or match. It is
dropped on return; the next observation reads fresh facts even if its mapper
tick is unchanged. Only copied values and IDs are stored, never native pointers.
Existing session, identity and routing checks remain. This is a reuse boundary,
not a promise that individual native reads form an atomic engine snapshot.

## Native observation evidence and limits

The local binding reference is skyline-smash revision
`d2ca90d9ecffdf9bf58efe6bcf4dd49f62ea1435`, `src/cpp.rs` and
`src/lua_const.rs`. The narrow declarations are in [game_ffi.rs](../src/game_ffi.rs).
**This identifies a binding revision, not a verified supported Smash version.**
No exact executable/version certification was found in these declarations; the
mapper signature check does not validate other native structures. No hardware
validation was performed. All native use below inherits that compatibility limit.
No new offsets, declarations, or native writes were added for this work.

In the table, **direct** means an existing read, not a guarantee of ABI validity;
**estimate** means policy inference. Costs are query counts/structure, not measured
Switch timing. Native pointers require a freshly resolved active object, matching
object ID and synchronous lifetime. Missing/invalid observations are not zero-sized
hurtboxes or proof that an attack is harmless.

| Observation / actual binding | Meaning, identity and validity | Cost / use / evidence limit |
|---|---|---|
| `AttackModule::is_attack`, `attack_data` | Current indexed attack; copy the record immediately. Endpoints are joint-relative; size is the stored native size. Null/inactive/nonfinite records provide no usable geometry. | Eight slots checked per source; at most 16 geometry records per scan. Direct prefix; no per-slot future schedule or generation token. |
| `ModelModule::joint_global_position_with_offset`, `PostureModule::scale`, `lr`, `pos_x/y/z` | Current joint-offset transform, model scale, facing, world root. Current active object only; no cached native pointer. Nonfinite output is rejected. | One or two transforms per eligible record, one lazy source-scale read. Shared by guard, counter and reflection in [hitboxes.rs](../src/live/hitboxes.rs). The binding names/signatures are verified locally; the boolean transform semantics, radius scaling convention and zero-second-endpoint sentinel are existing assumptions, not newly proven native facts. A capsule ending at local zero remains ambiguous. |
| `HitModule::get_total_status` (`hit_status`) | Existing aggregate protection query, index 0 used by target selection. Not individual vulnerable-part geometry and not future protection expiry. | Demand-driven direct read. Existing ledge/grab protection gates retained; not used to declare an entire defender immune. Cached binding also declares `get_status(accessor, int, int)`, but part/index meanings are not established locally. |
| Hurtbox candidates | Cached `HitModule::get_part_size` and `get_center_pos` return `u64`; no verified geometry layout, radii/endpoints, part enumeration or lifetime contract was established. | **Not integrated.** Do not cast these returns to invented hurtbox structures. Configured body envelopes remain estimates, including crouching, aerial and changing poses. |
| `GroundModule::get_rhombus` | Cached binding returns `*const Vector4f`; the component semantics and shape lifetime are not established here. Collision body is not a vulnerable hurtbox. | **Not integrated.** Cannot justify replacing body envelopes with this pointer. |
| `MotionModule::motion_kind/frame/rate/end_frame`, `StatusModule::status_kind/prev_status_kind`, `situation_kind` | Current Hash40 motion, fractional animation frame, playback rate, status/situation; previous status is history index 0. Rate converts animation-frame deadlines into game-frame estimates. | Lazy callback-local copies keyed by object ID. Invalid clocks remain unknown; transition/rewind is not a global game clock. No promise of unobserved animation frames. |
| `WorkModule::is_enable_transition_term`, `CancelModule::is_enable_cancel`, native cancel/normal-hit-frame queries | Current permission vs motion metadata deadline. Status, situation, capture and stop remain separate gates. | Live demand-driven reads, not cached permission. Counter still requires `CONT_SPECIAL_LW`; permission/input delivery does not prove special activation or counter success. |
| `KineticModule::get_sum_speed_x/y`, native parameters | Current aggregate XY velocity, world units/game frame; `AirPhysics` validates gravity and terminal-speed parameters. | Lazy copied speed, bounded parameter reads. Linear hitbox sweep is an **estimate**, not observed joint velocity, acceleration, homing, rotation or landing. Stopped guard sources and defender have zero forecast movement. |
| `StopModule::is_stop/is_damage/get_hit_stop_real_frame`, reaction and reserved-damage-speed WorkModule keys | Current stop, victim hitstop countdown and status-scoped reaction/launch values. Reserved launch is pending movement, not movement during freeze. | Lazy/direct reads in existing tech/damage adapters. Animation and stop gates stay separate from controller-poll watchdogs. DI/SDI remain disabled by `AUTO_DAMAGE_STICK_ENABLED = false`. |
| `GroundModule::ray_check`, passable-ground/cliff queries | World XY origin plus **displacement**, present terrain support; passability is not hurtbox status. | Existing bounded support/tech rays; identical support probes shared within callback. Cached bindings also expose `get_touch_normal` / `ray_check_hit_pos_normal`; contact selectors/output semantics were not validated for new use. No new terrain or normal claims. |
| `sv_battle_object::is_active/module_accessor/kind`, manager entries/current fighter ID, `TeamModule::team_no`, `LinkModule::is_link` | Current object vs player-entry identity; categories distinguish fighter/weapon/item. Capture slots 0/2 are distinct from partner links. | Revalidate ID/team before source reads; callback-local roster of up to 16 fighters. No accessor survives callbacks. Transformations/respawn must continue through existing identity/session fences. |
| Native `ATTACK` observer and `Sources` | Bounded weapon/item ID plus software generation advanced by observed ATTACK commands, not a native allocation-generation ID. | 32 remembered sources, current active/team checks; absent unobserved sources cannot be enumerated. Reuse without an observed ATTACK remains a gap. `TeamModule::team_owner_id` exists in cached bindings, but owner-ID meaning during reflection/transfer is not proven here and no owner-chain inference was added. |
| `AttackModule::is_infliction`, damage attacker-entry key, shield status | Fresh hit-mask result and existing entry attribution; shieldstun alone does not identify an attack slot or attacker. | Demand-driven direct reads. Conflicting entry directions yield unknown. No verified collision-record layout or counter-success signal was added; native down-special startup is not a successful counter. |
| Controller prefix, `MappedInputs`, `ControlModule::check_button_on`, button history | Physical held/previous/edge bits, mapped packet and game-consumed buttons are different stages. History describes polls, not simulation progress. | Existing copied controller data and receipt/ownership checks. Prefix provenance is Training Modpack `077a8e43b11d1f61ce57169156fb82e92e784c79`; compile-time offsets do not certify the executable. No new command-buffer memory reads. |
| Local `Ultimate_stats.json` via `fighter_data` | Catalog active windows/damage presence tied to fighter and mapped motion. | Parsed once; lookup only when needed. **Estimate:** current-window remaining time, not native per-slot expiry, reach, future pose or guaranteed collision. Unknown timing never admits counter; a geometrically earlier unknown-timing attacker blocks a later candidate. |

### Consumer decisions

- Ledge shielding and out-of-shield threat detection share the bounded active
  geometry scan. Fighter/weapon/item geometry is checked **before** root-based
  startup/recovery limits, so disjoints are not culled by distant owners.
- Projectile defense also checks active geometry independently of owner-root
  distance. A truncated scan withholds selection rather than claiming the first
  projectile was found. Current team/capability/situation and tracked-generation
  checks remain in place. Non-nominal defender scale still withholds reflection.
- Counter retains current active-window bounds and native readiness. Unknown
  timing can veto an earlier contact but cannot create an affirmative counter.
  Non-nominal defender scale now withholds rather than using nominal body bounds.
- All three adapters share copied geometry conversion and finite-output rejection.
  They keep their distinct threat/admission rules and bounded budgets. Guard can
  conservatively warn on budget exhaustion; counter/reflect cannot claim timing.
- Ledge positioning, sequence range settings, roll following and grab approach
  retain root displacement for navigation/configured acquisition. Existing grab
  and protection estimates are **not** upgraded to proven body contact: no safe
  hurtbox/catch geometry was established. Tech retains its terrain/support probes,
  not attack geometry. No live DI/SDI path was activated for testing.

Host regressions execute production adapters with native stubs for distant roots,
relative/frozen movement, unknown timing among multiple opponents, scale rejection,
transformed capsules, invalid depth, situation/team filters, lifecycle generations
and bounded scans. Stubs prove adapter behavior, **not** the native transform,
endpoint sentinel, capability offsets or executable compatibility. Pose-aware
hurtboxes, actual ownership transfer, future attack rotation/expiry and native
success attribution remain unverified; do not claim those are solved.

## Coverage

- All fighters' ledge responses, grab movement and recovery coverage.
- Ledge trump and onstage roll reads.
- Autopilot shielding, shortcuts and fair/trip confirmations.
- Tech/captor timing and SDI.

Readiness/transition terms, hit confirmations, active attacks, collision queries
and protection metadata remain live, demand-driven reads. The snapshot never
crosses the existing observation/publication fence.

## Input and diagnostic work

X/Y/D-pad input sequences are fighter-specific. Modes use only ON/OFF rows. No shared-button publication/cache or cross-fighter
copying remains. The runtime resolves five ledge plus six button slots only after
selection/edit changes; ordinary polls check numeric button identities. The
outside-battle editor, last-used-fighter tracking and SDK controller interception
hooks are removed. Only an explicitly pending save can request a safe-scene
check outside battle; there is no lobby menu input polling.
Diagnostic configuration rows rebuild only after menu revisions, fighter changes
or changes to shared Auto-mash. Actual changed reports still use the existing
bounded logger. These changes do not slow input polling or detection, and do not
cache readiness, hit confirmation or attack-active flags.

## Callback overhead and hidden menus

The command-observation path refreshes validated human-controller routing and
local presence before its existing duplicate guard. Fighter-profile/menu settings
and assist setup now run only after that guard accepts the selected local fighter.
The existing exception still lets command callbacks discover opponents after a
mapper-only observation. Session, identity and publication checks remain in place;
no observations are throttled and no facts are cached across native callbacks.

The hidden Kirby Assist menu returns before ImGui context/window/draw calls.
A visible menu reuses its cached strings; only changed revisions rebuild them.
The master-switch toast uses an atomic pending flag so its empty cache needs no
mutex on ordinary render callbacks. Publishing and expiring a toast update the
flag under the toast lock, preserving notifications. Master-switch gesture and
battle-presence checks still run as before. These optimizations do not disable
or measure the shared ImGui host's own frame/GPU work or Deluxe's renderer.

Ledge shield geometry reads an opponent's scale only when an active hitbox needs
a geometry test, once per opponent in that pass. Active-hitbox geometry is no
longer rejected by owner-root scan ranges; startup/recovery warnings retain those
limits. The shared ATTACK observer
tracks bounded projectile sources for ledge shielding and projectile reflection.
D-pad sequence bindings remain fixed numeric checks over six button slots,
with recipes resolved on selection/edit changes rather than every button poll.
Removing those bindings is not expected to remove substantial ongoing work.

These are source-level overhead reductions, not measured improvements to Switch
temperature or FPS. Host tests/type checks cannot establish the cause of heat
when Deluxe and Kirby Assist are loaded together.

## Consistent progress without changing timings

`timing::motion_progress` classifies initial observations, duplicate poses,
advancement, status/motion transitions, rewinds and invalid clocks. Elapsed
motion time uses the previous sample's rate. Each caller retains its explicit
initial/transition, hitstop, rewind and timeout policy. Tech cooldowns retain
their stable-rate basis; ledge velocity estimates retain their current-rate
basis. These policies are not silently recalibrated.

Ledge attempts/grab delivery, fair confirms, shortcuts, guard holds,
roll reads and recovery cover share this arithmetic. The
`AnimationClock` uses it for event deduplication, retaining native clatter-timer
and identity changes. Mash, turbo and ledge trump receive that event
clock as before. Generic confirmed sequences supplied actual simulation frame
numbers use `simulation_elapsed`. Direct attack-frame and native hitstop-timer
checks remain direct checks, rather than being converted into a different clock.

This is not a global simulation counter: a motion change still contributes the
existing one-frame transition allowance. Animation data cannot reconstruct every
unseen game update. Owning policies handle hitstop before spending sequence time.

No waiting frames, learned delay offsets or move-timing changes are added. The
35 ms request freshness and 250 ms observation/session fences are unchanged.
Network packets and arena delay settings are untouched. Switch testing remains
necessary to measure processing cost and actual online input acceptance.

## Extending the system

1. Pass the current `&FrameReads` to new native observers, rather than creating
   an independent cache. Reacquire and validate accessors as usual.
   To cache a new native fact, add a `Field` variant in
   `crates/kirby_assist_core/src/shared/observations.rs` (keeping `Kind` last,
   because `FIELDS` sizes the array from its index) and one line to the
   `cached_reads!` block in `src/live/frame.rs`. The macro generates the
   read-through accessor, so a field and its `Fact` variant cannot drift apart.
   Reads needing extra arguments or their own cell, such as `ground_support`,
   `ground_brake`, `damage_launch` and `previous_status`, stay hand-written.
2. Keep readiness/contact checks live; publish copied portable contexts.
3. Use `motion_progress` / `motion_elapsed` for motion-relative time and define
   stopped, initial, transition and rewind behavior explicitly.
4. Use `simulation_elapsed` only for a supplied real simulation-frame counter.
5. Test duplicate polls, hitstop, rate changes, loops, identity changes, rejected
   inputs and the existing attack deadline. Do not introduce callback-count
   timers or sleeps for action sequencing.

## Verification limits

`tests/shared_observations.rs` covers lazy reuse, fresh callbacks, overflow,
re-entry, unchanged valid elapsed arithmetic, event deduplication and simulation
counter separation. Existing action tests cover request timing and acceptance.
Native helpers are also host metadata-checked against actual FFI signatures.

Action ownership now resolves committed proposals before the final input
overlay; see [rules and actual live-wiring tests](ACTION_OWNERSHIP.md). This does
not add timing offsets or replace native readiness checks. No Switch plugin
was built as part of the host-only verification.

For this observation change, 70 focused host tests passed (native observation,
reflection adapter, guard, shared observations, counter and capsule geometry).
The broader `projectile_reflect` and `projectile_reflect_rearm` integration suites
have seven failures that also reproduce on an isolated unchanged HEAD; they were
not changed here. This is not a clean full-suite certification. Host checks do not
establish on-console acceptance, exact collision geometry or online outcomes.

Six button identities are checked per input sample. Inactive bindings add no opponent scans, rays or separate policy timelines. Only a pressed input sequence runs its existing timeline. Hold/Toggle mode processing and mode override fields are removed; legacy saved mode sections are excluded at startup without rewriting the source file.

Raw-input calibration descriptions now distinguish an unavailable move model
from input delivery; absent optional readiness/opponent observations alone are
not treated as proof of a missed raw input.

## Startup stack bound

Fighter identities are constructed one at a time from a static table of function
pointers. Do not restore an array of all 77 full profiles followed by `collect`:
with eleven menu lists that array alone was roughly 25 KiB on the target and
nested under config/parser frames on a 64 KiB loader stack. A silent startup
crash report showed SP below its stack region during allocation. This change
preserves all profiles, file entries and defaults, removes the bulk temporary
and bulk-sort collection, and adds an optimized host 64 KiB-stack regression.
No worker or gameplay timing change is involved; native restart still needs
verification after installing the rebuilt binary.

Tilt face requests opponent-facing reads while physical tilt intent or grounded
tilt recovery is present, when at least one move is enabled. Yoshi has no idle-crouch
reads. Enabled Yoshi Jab lock checks the bounded roster for nearby native
tumble/down/tech states; full recovery facts require an eligible state within
60 horizontal / 24 vertical units. Floor-support rays require a relevant target
and grounded Yoshi. The disabled chase adds no such reads. No new worker/hook
or physics/status/hitbox writes are used.

Auto-tech retains one copied floor-motion sample during eligible non-stopped
tumble. Recent matching native progress can enable up to five short descending
floor sweeps per body offset (15 floor rays maximum); missing/invalid trends
retain linear prediction. No extra object scan, physics write, worker or learned
network delay is added. See [floor-tech prediction](AUTO_TECH.md).

Optional Out of shield uses the native GUARD_DAMAGE attacker ID, shieldstun count
and existing transition/motion reads for one validated fighter. It never selects
an attacker by proximity. Held-L approach warnings now cover verified compiled
attack windows and bounded live active-hitbox geometry beyond the root warning
envelope (16 active slots maximum, no weapon/item scan). Ledge get-up responses
and committed text timelines retain priority. See [OOS](OUT_OF_SHIELD.md) and
[held-L shielding](LEDGE_TRAP.md).

Dtilt face Down intent includes physical right-stick axes captured before native
mapping, not only digital aliases. Standing first-flick detection uses the same
existing observation refresh. Only the winning grounded helper may angle that
existing down C-stick flick; both axes are restored after mapping. No new scan,
native query, worker, attack edge or intentional wait was added.

Grab dodge + punish is an ON-by-default ON/OFF row in both builds. It reuses the existing
status-gated eight-fighter ordinary-grab reader and now covers the full six-frame
forecast with at most 13 half-frame arithmetic tests per candidate. Its optional avoided-grab response now uses the same file timeline as OOS,
following only the tracked grabber after native spot-dodge acceptance. The forecast
window is saved per fighter; at its maximum of eight frames it uses at most
17 arithmetic tests per candidate. No configured follow-up means dodge only. OOS starts the first file response matching range/side after native shieldstun,
or hands back shield/stick control immediately when no response matches. It no
longer selects moves using compiled fighter startup/recovery timing.
Diagnostic-only `MOVEMENT OVERRIDE` entries name the owner when a full forward
stick is overridden near-neutral/opposite, once per owner/episode change with no
new native reads. See [Grab dodge](GRAB_DODGE.md) and [OOS](OUT_OF_SHIELD.md).

Neutral raw-input diagnostic reports now survive short recipe completion and
record first native attack activity plus first activity during target vulnerability.
Two fixed-size copied snapshots include target motion frame/protection/recovery
and relative gap/height. Native attack-active queries are diagnostic-only, bounded
to eight slots per eligible observation, outside STATE locks. Formatting remains
on the existing logger worker; there is no input-timing/acceptance/recipe change.
See [neutral miss evidence](CALIBRATION_CARD.md#neutral-get-up-attacks-that-execute-but-miss).

Diagnostic health reports now add worker-formatted review flags and per-second
rates for full local evaluations, mapping, roster discovery and draw callbacks.
Additional fixed events count discarded publications, stale-input episodes and
native timeline clock cancellations. Retained/save trend hints use the existing
bounded estimates and worker gauges; they do not claim system heat or heap usage.
A diagnostic-only manual test-delay label starts fresh Perf epochs without
changing timing. Frozen input-path records keep request/ack/native evidence and
clock-safe frame deltas separate from wall time and external latency labels.
See [test workflow and review codes](TRAINING_MONITOR.md#labelled-latency-and-workload-trials).

Static guard-on/held-shield poses may report a positive motion rate without
advancing their frame. Those known states may use a fresh locked opponent motion
clock, as teeter poses already do. Roll target discovery and follow snapshots
carry the real native clock from the first match; own/opponent deltas are never
added together. Stop and missing-sample gaps rebase rather than fabricate time.

## Confirmed event endings

The per-observation fact cache can export statuses it has already read. Automatic
roll and incoming-grab histories use these copied facts to distinguish a known
native event end from a missing observation. Consumed events survive observation
gaps; confirmed status changes allow later events to rearm. This export performs
no native query, roster pass, cross-frame cache reuse or heap allocation. Missing
fields remain unknown, including when a candidate was intentionally not read.
