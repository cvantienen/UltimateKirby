# Explicit action ownership

One final selector chooses which assist owns automated buttons and sticks before
the shared input overlay. It uses the existing mapper and observations; it does
not add a hook, force an animation, cancel native move recovery or change physics.

## Interruption rules

| Situation | Behavior |
| --- | --- |
| Held ledge button / ledge mode ON | Suspends all seven Auto react events, automatic grabs, shield backup and grab avoidance immediately, including their opponent searches. They resume fresh checks only after the ledge binding and toggle are both OFF. Explicit recipe previews also suspend automatic responses. |
| Acquired ledge visit | Reserves its setup and configured response against competing configured actions. |
| Ledge response waits, release frames and final input | Remain protected until delivered or cancelled. |
| Response finishes, even if the attack misses | Releases its reservation so enabled assists can match the opponent's current state. Holding L or leaving ledge mode ON does not block them indefinitely. |
| Tech, DI/SDI or mash safety | Retains its safety priority when hit/captured or a native tech is needed. |
| Native-ready incoming-grab dodge | Sampled ahead of new offense. Cannot interrupt committed roll/OOS/ledge/safety input; native dodge startup releases Down/Guard; the saved spot-dodge Up-B recipe may run at first native-ready recovery, within its range. |
| Ready Auto react Up-B versus an unmapped automatic grab | Clears an uncommitted grab or a committed grab approach and its cached output before emitting Up-B in the same observation. Already-delivered R, a committed non-approach grab, native grabs, ledge reservations and other committed modes keep their protection. |
| Shield-grab backup | A legal front grab wins; otherwise a close vulnerable target on either side can receive direct shield Up-B. The short B attempt is exclusive and bounded. Any follow-through is controlled by the edited sequence; there is no implicit drift tail. |
| Manual input during confirmed Up-B drift | Releases the optional sequence drift tail in that observation. The initial B pulse and literal ledge/button tapes retain their protection. |
| Grounded moving grab | May acquire a front shielding, vulnerable target while walking/running/turning or normally landing when native catch readiness is present. Grab requests use neutral stick; acquired ledge, safety and roll Up-B keep their priority. Shield grabs keep neutral axes and roll-safe intent gates. |
| Native grab/catch/hold/pummel/throw | All automatic ownership and ledge/button tapes release. Physical throw selection remains manual; no Up-B or follow-up is mixed into the grab. |
| Delivered automatic R awaiting acceptance | Cannot be replaced by Up-B. After the four-packet input budget, bounded settling owns no movement/buttons. Native grab startup or expiry clears it. |
| Fresh shield/jump during automatic Up-B | All three automatic Up-B owners share manual escape gates. The initial grounded B pulse stays protected; fresh airborne shield may hand back for a manual dodge. |
| Unconfirmed automatic Up-B with frozen clocks | A 24-distinct-delivery watchdog releases input; repeated observations/receipts do not count. Native startup disables this watchdog and advances to the next authored sequence action. |
| Passive walking, crouching or shielding | Cannot steal a committed punish; ledge approach defense remains available during preparation. |
| Other committed sequence | Uses the explicit owner/interruption rules in `action_owner.rs`; unrelated rules do not freely overwrite its inputs. |
| Invalid scope, stale observation, disable or cancellation | Releases ownership and cancels losing plans rather than replaying them later. |

The ledge reservation gates all configured action owners, including future ones
unless explicitly classified as safety/passive. It protects ledge setup from
ordinary roll, OOS, grab-dodge/punish and button sequences. Regrab/vulnerable-hang
responses are part of ledge handling and can replace its drop defense.

Cancellation retains physical button-edge history so holding a button cannot
silently restart a cancelled recipe. Roll rules retain their used-roll marker.
An event that ended while another response owned control is not queued for a
late attack. A still-current qualifying event can match after handoff. Native
startup, shield-drop and recovery rules continue to decide whether an input takes
effect; reference timings do not bypass them.

Selection uses a fixed-size intent list and enum comparisons, without additional
fighter scans, ray queries, logging, network changes or delay estimation.

## Verification and extensions

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline --test action_ownership
```

See [assist rules](REACTION_RULES.md), [ledge controls](LEDGE_MENU.md#l-and-the-autopilot-toggle)
and [input timing](SEQUENCE_FILES.md). Host tests cannot establish actual Switch
performance or prove that a specific input was accepted in-game.

## Configured automatic sequences

The Auto react editor groups roll, enemy spot/air dodge, automatic unshielded and
shielded grab, after dodging enemy grab, and shield backup. Grouping configuration
does not merge their enable switches or owners. Native grab/backup defaults keep
their existing delivery paths; edited actions commit one shared input timeline.
A committed edited response cannot be replaced by a competing new reaction.
Queued R in either grab mode or a configured grab-dodge punish preserves a
bounded receipt reservation with zero input while settling. It cannot fall
through to another B/Up step or resume a hidden script after native catch.

Shielded and unshielded grabs use separate target policies: only the unshielded
path requires enemy shield. A legal shielded front grab wins before the close
shield backup; rear/ungrabbable/out-of-grab-range targets can use backup only
inside its own X/Y limits. Existing usable roll/dodge reactions retain priority
before new grabs. A delivered Grab cannot switch to Special while its receipt
is unresolved, and a committed backup cannot switch back to Grab midway.

# Automode authoring guide

This guide follows the path from a fighter entering battle to a controller
overlay being delivered. It is the safest map for adding an automode because
each stage has one responsibility:

```text
startup configuration
    -> command observation
    -> copied native facts
    -> policy/state-machine proposal
    -> ownership arbitration
    -> controller overlay for one mapper call
    -> physical input restoration
```

An automode must observe native state and emit controller input only. It must
not write fighter status, physics, hitboxes, animation rate, or move data.

## Battle lifecycle

### 1. Startup prepares immutable inputs

`src/live.rs::install` validates fighter data, loads the startup configuration,
fighter profiles, and saved sequences, then installs the command observer. The
mapper hook is installed later, after an eligible fighter callback has located
the supported mapper.

Treat startup data as immutable during battle. Persistent changes go through
the existing settings-menu save path; per-frame code must only use a copied
configuration snapshot.

### 2. The command hook admits one local fighter

`src/live.rs::observe_commands` calls `live::observation::observe_fighter` for
the game command category. `observe_fighter` rejects inactive runtime/config
states and non-local or stale routes before doing policy work.

For an admitted fighter it:

1. resolves the selected fighter/profile and session ticket;
2. reads short-lived native facts into copied values;
3. snapshots saved policy settings;
4. derives cheap mode gates;
5. performs only the additional observations currently needed; and
6. publishes copied contexts and policy outputs to `State`.

No native pointer survives the callback. A mode should add only the facts it
needs to `Collected`, a mode-specific context, or an existing context.

### 3. A policy proposes logical input

Policies return `gameplay::ControllerOutput`. The important fields are:

- `requested`: controller buttons to press for this update;
- `owned`: buttons whose physical input the mode temporarily controls; and
- `stick`: an optional logical left-stick position.

The core crate decides *what it wants to press*. The live crate decides whether
that proposal is current, wins arbitration, and can be applied to the selected
controller.

Stateful modes retain their small state machine in `State`, together with the
latest `ControllerOutput`. They must handle duplicate observations: returning
the previous output for an identical native sample prevents a second callback
from manufacturing another button edge.

### 4. The mapper picks one owner and applies it briefly

Before the native controller mapper runs, `live::mapper::prepare_mapper_overlay`
copies the physical controller frame. `compose_overlay` samples helpers,
arbitrates their `action_owner::Owner` intents, and selects one output.

Exclusive owners receive the whole gameplay-button set and a neutral/default
stick when necessary. This is deliberate: a release frame in an automated
sequence still belongs to that sequence, rather than leaking a physical input.
The original controller data is restored immediately after the native mapper;
only the mapped result remains.

Add a new owner when the mode has a multi-frame commitment that must not mix
with another automode. Use an existing owner only when the lifetime and input
conflict rules are genuinely identical. Safety modes retain the highest
priority; do not bypass `ActionOwnership`.

## Universal automodes

Use a universal mode when its trigger and response are meaningful for all
fighters, or can be described entirely by shared observations.

Keep its policy in `crates/kirby_assist_core/src/policies/` or `shared/`:

1. define a small copied context with only required facts;
2. make validation explicit (`finite`, active, grounded, etc.);
3. return `ControllerOutput`, never native writes;
4. add tests with fabricated contexts; and
5. wire native observation and output delivery in `src/live/`.

Examples include auto-tech, DI/SDI, generic reaction rules, and automatic
defensive policies. The generic `AssistEngine` is useful for stateless,
scenario-based rules: it selects an enabled `ScenarioRule` and produces an
`InputRecipe`. The live adapter still supplies observations and performs final
controller arbitration.

Use an existing generic policy before adding another top-level mode. A new
scenario, rule, or sequence configuration is usually smaller and safer than a
new state machine when only timing or the requested input differs.

## Character-specific automodes

Use a character-specific mode when recognition depends on a fighter's native
status/motion semantics or the response has fighter-specific lifecycle rules.

The Kirby trip follow-up is the reference implementation:

```text
config.down_tilt_trip_smash
    + fighter kind == KIRBY
    + Kirby side/down tilt
    + target trip evidence
    -> TripFollowup queues
    -> owns controls through tilt lag
    -> repeatedly buffers forward-smash input
    -> releases on smash startup or interruption
```

The main pieces are:

- `fighters/kirby.rs`: `TripContext` and the pure `TripFollowup` state machine.
- `src/live/kirby_followups.rs`: converts copied live facts to `TripContext`.
- `src/live/observation.rs`: gates the watch, collects targets only when the
  watch is active, and samples the state machine.
- `src/live/state.rs`: stores the machine and current output, and cancels it
  when another owner wins.
- `src/live/mapper.rs`: submits `Owner::Trip` to arbitration and selects its
  output when it wins.

### Recipe for a new character

1. **Choose the scope.** State exactly what starts the mode, what confirms it,
   what cancels it, and what native status proves success. Do not use a timer
   as proof that a move started.
2. **Build the core machine.** Add a module under `fighters/` with a copied
   context and a `Default` state machine. Keep it independent of Skyline and
   raw controller memory. Give it tests for success, interruption, a new
   fighter object, duplicate observations, and stale/missing targets.
3. **Add a narrow live adapter.** Put fighter-kind checking and raw native
   reads in `src/live/`. Convert them immediately into copied context fields.
   Gate expensive target/hitbox scans behind the mode's cheap status gate.
4. **Store and arbitrate it.** Add state and output fields to `State`; cancel
   them in the existing reset/suspension paths. Add an `Owner` and priority if
   the mode owns a sequence across frames. Wire the owner to
   `mapper_action_intents` and `selected_owner_output`.
5. **Configure it at the right level.** A global startup switch belongs in
   `LiveConfig`, as `down_tilt_trip_smash` does. A per-fighter choice belongs
   in `fighter_tuning`/fighter profiles. When extending tuning, append new
   schema fields; do not reorder existing stable indexes, keys, labels, or
   hints because saved configuration relies on them.
6. **Keep delivery input-only.** Express the response as `ControllerOutput`
   (`requested`, `owned`, and `stick`). Let the mapper overlay deliver and
   restore it. Never add a native gameplay write to make a response easier.

## A closely related, non-automatic example

`tilt_turn_down` is a per-fighter input helper, not an automatic attack. It
uses a real physical down-tilt intent and merely aims the stick toward the
opponent; it never creates an A or C-stick attack press. Kirby and Yoshi have
this enabled by default through `fighter_tuning`.

This is a good template for character-specific *input assistance*. The Kirby
trip follow-up is the template for a character-specific *automatic sequence*.

## Pre-merge checklist

- Native observations are read-only and copied before state is retained.
- Every state machine has explicit cancellation and duplicate-poll behavior.
- The mode yields to the established ownership/priority system.
- Button/stick output is expressed only as controller input.
- Configuration is immutable during battle and keeps serialized schema order.
- Core logic has host-testable unit tests, including interruption regressions.
- Live integration scans only what the mode's active gate requires.
