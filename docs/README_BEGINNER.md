# Kirby Assist: a beginner's guide to the code and real input integration

For the current file layout, a ledge-response walkthrough, and how loaded
fighter profiles fit, start with [the source map](CODE_MAP.md).

## 1. What you have right now

The project now has a decision system **and an implemented Switch input path**. The live path observes the controlled Kirby and applies raw mash and ZR button states to raw controller memory before mapping. It is a compiled test candidate that still needs verification on the actual Switch. Choose a [silent or diagnostic build](BUILD_VARIANTS.md), then start with [INTEGRATION.md](INTEGRATION.md) to install it; you do not need to write an adapter yourself for the included tools.

This guide teaches the reusable Rust core and a pretend adapter so you can understand and extend it. The generic `AssistRuntime` path shown below is educational. The live code in `src/live.rs` instead passes requests between a game-state callback and a raw controller-mapping callback; [the live architecture](INTEGRATION.md#the-actual-live-architecture) explains that difference.

| Part | Exists? | What it does |
| --- | --- | --- |
| Auto-tech and auto-mash settings | Yes | Describe when to propose Shield or Attack. |
| Scenario priority, timing, and merging | Yes | Select and combine logical assistance with player input. |
| Demonstration adapter | Yes | Simulate game data using ordinary variables. |
| Live Switch hooks | Yes, device verification pending | Observe human Kirby and modify raw controller button states and press/release history. |
| SD-card config loader | Yes | Set tool toggles, controller ID, and timing at startup. |
| In-game settings menu | No | No interactive settings menu is implemented. |
| Scenario-triggered recorded playback | No | Sequence sampling exists, but recording/playback is not connected. |

There is no training-mode gate. That does not establish tested compatibility in every mode.

## 2. Start by running something you can see

From this repository's root directory, run:

```sh
cargo run --manifest-path crates/kirby_assist_core/Cargo.toml --example walkthrough
```

Open `crates/kirby_assist_core/examples/walkthrough.rs` from the project root beside its output. It contains a complete **pretend** adapter and uses the same `AssistRuntime` a live adapter would use.

In this example, the human holds Jump throughout. The pretend game reports these situations:

| Situation | Scenario frame | Assist adds | Final buttons |
| --- | --- | --- | --- |
| Clatter | 0 | Attack | Jump + Attack |
| Clatter | 1 | Nothing | Jump |
| Clatter | 2 | Nothing | Jump |
| Clatter | 3 | Attack | Jump + Attack |
| None | 0 | Nothing | Jump |
| Clatter again | 0 | Attack | Jump + Attack |
| Tech | 0 | Shield | Jump + Shield |
| Tech | 1 | Nothing | Jump |

The final line disables assistance and confirms that the human's Jump is still preserved. This example is about inspecting input values; holding Jump is just an easy way to see that merging works.

Changing `DemoAdapter.output` only changes a variable. No real controller, game, or emulator is controlled by this program.

## 3. Understand the two Rust crates

A **crate** is a Rust compilation unit. A package's `Cargo.toml` describes its name, dependencies, and build settings. Cargo is Rust's build/package tool.

```text
kirby_assist/
├── Cargo.toml                  Main package: Switch plugin
├── src/lib.rs                  Skyline startup function; calls live::install()
└── crates/kirby_assist_core/
    ├── Cargo.toml              Separate package: reusable Rust logic
    ├── examples/               Programs you can run on your Mac
    └── src/
        ├── lib.rs              Exposes the core's public API
        ├── tools.rs            AutoTech and AutoMash presets
        ├── config.rs           Rules, enable flags, and timing
        ├── state.rs            Scenarios, observations, adapter contracts
        ├── assist.rs           Chooses what to propose
        ├── input.rs            Input values and merging
        ├── runtime.rs          Connects reads, evaluation, and writes
        └── sequence.rs         Samples hand-authored input sequences
```

The main package depends on the core through a local path in its `Cargo.toml`. The core does not depend on Skyline. That separation lets most logic run and be tested on your computer, while platform-specific code stays in the plugin.

There is no root Cargo workspace declaration here. Use `--manifest-path` when you mean to run the nested core package. Formatting the main package alone does not replace formatting the nested core package.

`cargo run --example walkthrough` finds `examples/walkthrough.rs`. That file has a normal `fn main()` and runs as a host program. The plugin's `#[skyline::main(...)]` function is called by Skyline when the plugin loads into the game; it is not your per-frame update loop.

### Reading the live ledge assist

For the code that actually runs on Switch, start with `crates/kirby_assist_core/src/ledge_trap.rs`. `src/live.rs` collects native observations, calls that policy, and passes its requested controls to `ManualOverlay`. The generic teaching flow below illustrates the reusable engine; it is not the exact hook sequence of the live mod.

Read `sample`, then `evaluate`, then the response selected by `OptionPlan`.
Responses now live in `ledge_trap/aerial.rs`, `roll.rs`, `shield.rs` and
`grounded.rs`, all as methods on the same `LedgeTrap` instance. Normal get-up
defaults to short-hop nair; ledge jump also requests neutral jump+A together.
Roll moves to a common coverage point before timing down smash. Get-up attack
uses shield and a block response; confirmed capture returns control to you.
`observe_action` checks startup, actual hit/capture and whiff recovery;
`choose_response` remembers the opponent's option even when Kirby is busy.
`AttackWindow` in `prediction.rs` calculates timing and optional approximate
reach. A requested button is an attempt: an attack animation is different from
a confirmed hit, and a grab animation is different from capturing an opponent.
`GetupAttack` still supports legacy down/forward tilts for normal get-up configs.

`LedgeContext` is the input data, `LedgeSettings` is the tuning, and `LedgeOutput` is the proposed result. `requested` names the buttons to hold; `owned` names the buttons the assist replaces, including releases. Native status constants give names to game state numbers. Host regression examples live separately in `src/ledge_trap/tests.rs` inside the core crate.

See [LEDGE_TRAP.md](LEDGE_TRAP.md) for the function map, controls, and configuration. The removed forward-air settings in older SD configs are ignored so the other tools can still load.

## 4. Follow one frame through the system

Here is the intended live flow. For the teaching example, the adapter stores pretend values. A synchronous adapter would have this flow; the asynchronous Switch flow is documented separately in INTEGRATION.md:

```text
Smash: current fighter state + player's freshly mapped input
                         │
                         ▼
                 Game adapter
                         │
          RunContext + FighterObservation + ControllerFrame
                         │
                         ▼
               AssistRuntime::process_frame
                         │
                         ▼
                 AssistEngine::evaluate
                  │ scope and enable checks
                  │ highest-priority active scenario
                  │ enabled rule for that scenario
                  │ timing check
                  ▼
                    InputRecipe
                         │ merge with fresh player input
                         ▼
                    FrameDecision
                         │
                         ▼
               adapter.write_merged_input
                         │
                         ▼
          Smash consumes this frame's modified input
```

Take clatter frame 3 as an example:

1. The adapter identifies your controlled Kirby and a mashable condition.
2. It reports `Scenario::Clatter` with `frames_active: 3` and the human's current input.
3. The runtime calls the engine with those values and your configuration.
4. The engine checks that assistance is enabled and the fighter/controller matches the scope.
5. It selects Clatter and looks up that scenario's enabled rule.
6. The default mash interval is three frames. Frame 3 is an action frame, so the rule supplies an Attack recipe.
7. `arbitrate` combines human buttons with Attack. It leaves the player's right stick and triggers alone.
8. The runtime passes the resulting frame to the adapter's write method.
9. On frame 4, the mash timing is waiting. The runtime writes the fresh human frame, without an assist Attack contribution.

The engine is **stateless**: it does not remember the previous call. Scenario counters come from the adapter. `AssistRuntime` holds configuration and permissions, but does not advance those counters either.

A new Clatter occurrence must start at frame zero again. Otherwise “run once on entry” actions and delays will behave incorrectly.

## 5. Rust syntax you will see here

### Variables, structs, and defaults

```rust
let mut runtime = AssistRuntime::default();
runtime.config.enabled = true;
```

`let` creates a local variable. `mut` allows it to be changed. `AssistRuntime` is a **struct**, a named group of fields. `::default()` calls the type's default constructor; `.config.enabled` accesses nested fields.

Defaults start assistance off, so you explicitly enable it. Creating a local runtime inside a startup function would not retain it for future frames. The live integration keeps configuration in `OnceLock` and shared session state behind a `Mutex`; its callbacks use that retained state.

This syntax fills unspecified fields with defaults:

```rust
let mash = AutoMash {
    interval_frames: 4,
    ..AutoMash::default()
};
```

Here you change the interval and keep the default Attack button.

### Enums and match

`Scenario` is an **enum**: one value can be `Tech`, `Clatter`, `Ledge`, and so on. `Scenario::None` means no active situation. `ScenarioAction` is another enum; it distinguishes a controller recipe from the reserved playback action.

A `match` expression handles enum variants explicitly. In `assist.rs`, it chooses whether to merge a recipe or return the “playback backend missing” result.

`Scenario::None` and `Option::None` are different enum variants that happen to have the same short name.

### Methods, impl, and traits

An `impl SomeType` block defines methods on that type. For example, `AutoMash::rule()` converts settings into a `ScenarioRule`.

A **trait** describes methods another type promises to provide. The core's adapter contracts are:

| Trait | Method | Meaning |
| --- | --- | --- |
| `GameObservationSource` | `observe_fighter` | Supply normalized active scenarios and counters. |
| `InputBridge` | `read_human_input` | Supply this frame's player input before assist changes. |
| `InputBridge` | `write_merged_input` | Apply the final input for this frame. |
| `GameAdapter` | `read_context` | Identify fighter kind and local human control. |

`GameAdapter: GameObservationSource + InputBridge` means an adapter must also implement those two traits. The demo implements all three using stored values. These traits are the synchronous abstraction. The implemented raw-memory path uses separate game-state and controller-mapping callbacks rather than implementing these traits directly. See the [Rust Book's trait chapter](https://doc.rust-lang.org/book/ch10-02-traits.html) for the language concept.

### Borrowing: &self and &mut self

`&self` borrows an object to read it. `&mut self` borrows it with permission to change it. `&mut adapter` lets `process_frame` use the adapter and eventually call its writing method without taking ownership of the adapter.

In `adapter: &mut impl GameAdapter`, `impl GameAdapter` means “a concrete type implementing this trait.” It does not automatically construct a game adapter for you.

### Option and the question mark

`Option<T>` is either `Some(value)` or `None`. Adapter reads return `None` when the requested data is unavailable.

In the runtime:

```rust
let context = adapter.read_context()?;
```

`?` unwraps `Some(context)` and continues. If the result is `None`, it immediately returns `None` from `process_frame`. If fighter context or observation is missing but fresh human input is available, the runtime writes that human input to release the previous assist. If human input itself is unavailable, the adapter needs its own watchdog.

This is a normal unavailable-state path, not an exception. The adapter still owns cleanup of previously cached game output.

### Other common words

| Syntax | Meaning here |
| --- | --- |
| `pub` | Other modules/crates may access this item. |
| `use ...` | Bring a name into scope. |
| `crate::...` | Refer to something in the current crate. |
| `Self` | The type whose implementation you are inside. |
| `#[derive(...)]` | Ask Rust to generate standard behavior, such as debug printing or copying. |
| `const` | A constant value, or a function usable in constant expressions when its arguments permit. |
| `u16`, `u32`, `i16` | Fixed-width unsigned/signed integers. |
| `[T; N]` | An array containing exactly N values of type T. |
| `assert!(condition)` | Stop with a panic if a programmer expectation is false. |
| `#[test]` | Mark a function for `cargo test`. |

`assert!` is useful in demos and tests. A live settings UI should handle a failed configuration operation and report it instead of panicking inside a hook.

## 6. Configure tools and understand their limits

```rust
use kirby_assist_core::{AssistRuntime, AutoMash, AutoTech, Scenario};

let mut runtime = AssistRuntime::default();
runtime.config.enabled = true;
assert!(runtime.config.set_rule(AutoTech::default().rule()));
assert!(runtime.config.set_rule(AutoMash {
    interval_frames: 4,
    ..AutoMash::default()
}.rule()));

assert!(runtime.config.set_rule_enabled(Scenario::Tech, false));
assert!(runtime.config.set_rule_enabled(Scenario::Tech, true));
```

Each `.rule()` builds ordinary configuration. It does not start a background task or register a hook. There is one rule per scenario; setting another Tech rule replaces the existing Tech rule. A disabled rule keeps its settings; `remove_rule` removes them.

`AutoTech` means “add Shield when the adapter reports Tech and its timing is due.” It does not detect the floor, predict impact, determine lockout, or prove that a tech is allowed. Those are game-state integration responsibilities.

The generic teaching preset `AutoMash` means “pulse selected buttons during Clatter.” The live Switch translation now uses that scenario decision to produce a left-stick direction instead. Neither path directly shortens game timers. A valid input may still fail to cause an action because of the fighter's state or the game's input processing.

Buttons are combined as sets. Human Jump plus assist Attack becomes Jump + Attack. The core never removes a player-held button. Consequently, if you already hold Attack, an assist Attack pulse cannot create a new Attack press after a release; it stays held. 

That paragraph describes the generic logical core. The direct Switch path additionally implements `takeover`: it suppresses physical action buttons/C-stick during analog mash, or releases ZR during a tech attempt when a fresh press is needed. The live mash stick is owned only while its mashable scenario is active. 

Once a fighter has been associated, the mapper resolves its live accessor by object ID and refreshes detection before consuming that same update’s input. It does not retain a pointer across updates. For auto tech, ZR is pressed immediately if the last game-visible ZR state is released; only a held ZR needs a release update first. 

The live path caches both results of one update: `update_with_buttons` returns a ZR button request (zero during mash), and `mash_stick()` returns an optional left-stick direction. Clatter cycles Right → Up → Left → Down on distinct input updates. `None` means return the fresh physical stick. The mapper supplies matching digital direction bits as well as normalized stick floats. With takeover enabled, action buttons and C-stick are cleared during mash, then restored when it ends. See [ANALOG_MASH.md](ANALOG_MASH.md).

`InputRecipe::NONE` means “add nothing,” not “clear the player's controller.” Left-stick replacement requires an explicit permission flag. The symbolic stick positions such as `LEFT` use tiny logical values; their physical scale is not defined by the core. The adapter must choose and document a consistent scale for both human input and requested directions before enabling directional tools.

### Timing

For the generic teaching preset only, a mash rule with interval 3 produces:

```text
Scenario frame:  0  1  2  3  4  5  6
Assist Attack:   X  .  .  X  .  .  X
```

`RuleTiming` has a delay, active duration, and repeat interval. `repeat_every_frames: 0` means one activation period. `AutoMash` clamps its interval to at least 2 so its one-frame pulses have a gap. Custom `RuleTiming` does not enforce gaps: a duration at least as long as its repeat interval produces continuous activity after the delay.

The live analog cycle advances every distinct mashable update, without this button-release interval. Old SD button-mash interval/group settings are ignored.

### Priority

The engine selects the highest-priority observed scenario **before** it looks for an enabled rule. Tech wins over Clatter; defensive scenarios win over Grounded/Airborne.

If Tech is observed but its tool is disabled, Clatter does not take over that frame. If a selected rule is waiting for its delay, another rule does not take over either. This is the current deliberate priority policy.

Observations have room for three simultaneous samples, while configuration has room for eleven scenario rules. These are different capacities. A live observer must not silently discard a higher-priority condition when it has more than three candidates.

## 7. What “real controller input” means for this plugin

For this project, the intended integration modifies the input **Smash consumes inside the game**. It does not physically push a controller button, send Bluetooth commands, or automatically act as a USB controller for other software.

Conceptually, there are several input layers:

```text
Physical controller
    → raw hardware buttons
    → player's control mapping
    → game action buttons/sticks
    → input history, command recognition, fighter behavior
```

Our `ButtonSet::ATTACK` is a logical request, not a hardware mask or a complete Smash structure. The live raw-input policy explicitly translates Attack to native A and Shield to native ZR. It translates those requests to Smash’s internal raw button bits and modifies held/previous/pressed/released states just before the normal mapping function; the full implementation and compatibility limits are in INTEGRATION.md. The separate [manual-control example used live](TURBO_CONTROLS.md) now demonstrates D-pad consumption and temporary normalized native left-stick ownership.

The upstream Training Modpack defines separate raw-controller and mapped-input structures. Its mapped structure uses action flags and signed stick fields. This is useful evidence for why an explicit translation layer is necessary; do not cast our `ControllerFrame` into its structure. [Upstream input definitions](https://github.com/jugeeya/UltimateTrainingModpack/blob/main/src/common/input.rs)

The core also does not represent every native input flag. A real adapter should preserve unmodeled fields/bits from the original native frame. Reconstructing the entire native controller frame using only our limited logical fields could discard unrelated player inputs. For an initial button-only tool, preserve the native frame and translate only the intended assist contribution.

## 8. Integration milestones: how to understand and verify the live code

The included raw-button integration now implements the connection discussed in these milestones. Treat this section as background for maintainers and future adapter work. The current install procedure and exact remaining device checks are in [INTEGRATION.md](INTEGRATION.md).

### Step 1: Choose and record the target environment

Record your Smash version, Skyline/runtime version, relevant mod dependencies, and whether testing on hardware or an emulator. Select exact dependency revisions and hook definitions compatible with that environment.

The main manifest depends on `skyline` and the local core. The live code uses a narrow, explicitly sourced set of Smash foreign declarations in `src/game_ffi.rs` instead of adding the entire bindings crate. Skyline supplies the hooking infrastructure; foreign declarations describe game-facing functions. The upstream [Skyline Rust library](https://github.com/ultimate-research/skyline-rs) and [skyline-smash repository](https://github.com/ultimate-research/skyline-smash) are starting points to inspect. Adding a dependency alone does not implement input injection.

### Step 2: Prove the plugin loads and retain configuration

Build the silent variant with `cargo skyline build --release`, or the diagnostic variant with `cargo skyline build --release --features diagnostic-logging`, from the project folder. The optimized Switch plugin is written to:

```text
target/aarch64-skyline-switch/release/libkirby_assist.nro
```

With a compatible Skyline installation, the plugin destination for Smash is:

```text
SD:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
```

Skyline documents the general [runtime and plugin installation layout](https://github.com/skyline-dev/skyline). When using the diagnostic variant, confirm the startup log appears before diagnosing tool behavior. The silent variant intentionally creates no plugin log.

Then retain one configured runtime for the callbacks that will use it. Decide how configuration is accessed across callbacks; use synchronization if callbacks can overlap. Avoid an unprotected mutable global. The current `src/lib.rs` calls `live::install()`, which initializes retained configuration/shared state and installs the hooks after signature validation.

### Step 3: Read the correct fighter without changing input

The game-facing integration is `src/live.rs`; its narrow foreign declarations are in `src/game_ffi.rs`. Read those files alongside these responsibilities.

Implement context and scenario observation using verified bindings. Establish the relationship between the selected controller, player slot, fighter entry, and live battle object. These identifiers are not interchangeable. Keep a separate observation/counter state for each controlled fighter.

When developing with the diagnostic feature enabled, start by logging changes: valid fighter acquired, local player identified, Clatter entered/exited, object replaced. Return `None` when the object is unavailable, dead/replaced as appropriate, disconnected, or outside gameplay. Do not retain raw object pointers across lifetimes without a verified validity mechanism.

A `RunContext` has no controller ID field. If you want assistance on only one of several local players, selecting the correct player belongs in the adapter, before you create that context.

### Step 4: Maintain scenario counters at the correct cadence

Use a verified simulation-frame identity and update each active scenario once per relevant frame. A callback's name or the fact it is called often does not prove it fires once per simulation frame.

For each scenario, keep whether it was active last frame and its counter. On entry use 0; while continuously active increment once; on exit clear it. Reset on fighter replacement, match reset, or target loss. Define how pauses, hitstop, and repeated hook calls affect the counters so the timing matches the chosen input-consumption point.

### Step 5: Find and verify a human-input hook

You need a hook at a point where you can capture fresh player input and modify the frame the game will actually consume. Verify its signature, native layout, player/controller indexing, and ordering relative to input-history/command processing for your exact build.

As research examples, the Training Modpack's [training integration](https://github.com/jugeeya/UltimateTrainingModpack/blob/main/src/training/mod.rs) hooks game control functions, while its [input recording code](https://github.com/jugeeya/UltimateTrainingModpack/blob/main/src/training/input_record.rs) handles mapped inputs and a separate CPU-controls path. These are distinct mechanisms. A CPU-only hook is insufficient for your human-controlled fighter. Their training-mode checks are upstream behavior, not a requirement of our core.

Do not substitute “force the fighter into an attack status” for supplying input. That bypasses the normal input path and can behave differently from a button press. Likewise, changing only a held-button query may leave trigger/release queries and command buffers inconsistent.

This guide deliberately does not invent an offset, pointer layout, or universally valid hook signature. Those details must come from verified source and the target binary. A guessed hook can compile successfully and still crash or do nothing.

### Step 6: Implement translation and a one-frame write

For the selected player and frame:

1. Preserve the original native input before assist changes.
2. Convert the fields needed by the core into a `ControllerFrame`.
3. Supply the matching observation/context snapshot through your adapter.
4. Call the retained runtime's `process_frame` exactly once.
5. In `write_merged_input`, translate the assist result back into the appropriate current native input, preserving unmodeled native information.
6. Allow the game to consume that input in the correct hook order. Never feed a previous assist-modified frame back as fresh human input.

For an initial implementation, support button-only recipes first. The current manual turbo control is an explicitly separate directional path with layout checks and host tests; its hardware behavior still needs verification. Read `manual_controls.rs` and [TURBO_CONTROLS.md](TURBO_CONTROLS.md) before adding more directional tools.

Implementing an empty `write_merged_input` satisfies Rust's trait checker but controls nothing. The demo's assignment to `output` demonstrates the call path, not game integration.

Keep unsafe pointer access and foreign-function calls in the integration layer. `unsafe` means Rust cannot verify those operations' guarantees; it does not validate the layout or pointer for you. Never let a panic unwind across a foreign callback.

### Step 7: Prove a real press and release before auto tech

Use a controlled test with one selected fighter: verify that a single assist button press reaches the game, then verify the following fresh-human frame releases it. Verify that the other players and your movement remain unaffected. This validates the input path independently of scenario detection.

Next, enable Clatter-only mash and verify entry, pulse gaps, exit, disable, and re-entry. Then implement and test the actual Tech-window observer. Auto tech requires correct timing and eligibility; pressing Shield indiscriminately during tumble can request an air dodge or miss the tech window.

On missing state, the runtime skips its write. The adapter must clear its own cached contributions and avoid replaying an old merged frame when gameplay resumes. Verify death/respawn, match transitions, pause/resume, and controller changes.

Using a reproducible local test while developing does not add a training-mode restriction to the plugin. Test additional intended modes after the input path works; mode-independent policy alone cannot establish mode compatibility.

### Step 8: Add convenient settings

The live integration now reads `SD:/ultimate/kirby_assist/config.txt` at startup. It includes a global enable switch, individual tool toggles, controller selection, and timing. An in-game settings menu would be a future extension; restart Smash after editing the existing file.

The first useful live milestone is **one correct player's Clatter detection plus one verified Attack pulse and release**. After that, auto mash is a small extension. Reliable auto tech additionally needs its more demanding observer and timing validation.

## 9. Extend the tools without rewriting the adapter

For a scenario already represented by `Scenario`, add a rule:

```rust
use kirby_assist_core::{ButtonSet, InputRecipe, RuleTiming, Scenario, ScenarioRule};

let ledge_jump = ScenarioRule::controller(
    Scenario::Ledge,
    RuleTiming::after(5),
    InputRecipe::buttons(ButtonSet::JUMP),
);
// Insert into your existing configuration: runtime.config.set_rule(ledge_jump).
```

This waits five observed ledge frames and proposes one Jump frame. It does not prove that the game accepts that input in every ledge state.

For a new kind of scenario, extend the enum, its priority function, rule capacity if necessary, the adapter's detection/counter logic, and relevant tests. If a new tool needs multi-frame memory beyond a scenario counter, design that state explicitly. `InputSequence` can sample a sequence, but no existing rule automatically starts or cancels it. `PlaybackSlot` is also a placeholder, not working playback.

## 10. Diagnose problems in layers

| Symptom | Check first |
| --- | --- |
| No plugin startup log | Expected in the silent build; use diagnostic before investigating loading. |
| Startup log, no tool behavior | Read status.log for signature validation, installed hooks, controller association, and fighter observation. |
| Runtime says `Disabled` | `runtime.config.enabled`. |
| `ScopeRejected` | Local-human flag and Kirby requirement. |
| `NoActiveScenario` | Observation and scenario detection. |
| `NoRule(...)` | Selected scenario's enabled rule; priority may block broader rules. |
| `WaitingForTiming(...)` | Scenario counter, delay, duration, interval. |
| `ProposedController(...)`, but no game action | Native translation, hook order, input write, game eligibility. A proposal is not proof of an accepted action. |
| Attack stays held | Fresh human input vs merged output, native-frame cleanup, player-held Attack. |
| Delays run too fast | Duplicate updates per simulation frame. |
| First occurrence works, next does not | Scenario exit/re-entry counter reset. |
| Another player is affected | Fighter/controller routing in the adapter. |
| Crashes during transitions | Object lifetime, native layout, hook signature, shared-state access. |

Log scenario/status changes and occasional snapshots instead of printing every hook invocation. Keep enough information to distinguish “not detected,” “not proposed,” “not injected,” and “injected but not accepted.”

## 11. Commands and what they actually prove

Run these from the repository root:

```sh
# Logic tests on your computer; no Switch needed.
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml

# Rust lint checks for the core and its examples.
cargo clippy --manifest-path crates/kirby_assist_core/Cargo.toml --all-targets -- -D warnings

# Format both packages.
cargo fmt --all
cargo fmt --manifest-path crates/kirby_assist_core/Cargo.toml

# Run the pretend adapter walkthrough.
cargo run --manifest-path crates/kirby_assist_core/Cargo.toml --example walkthrough

# Compile the optimized silent Switch plugin.
cargo skyline build --release

# Compile the optimized diagnostic plugin instead.
cargo skyline build --release --features diagnostic-logging
```

`cargo` runs Rust tools; `skyline` selects the Switch-specific build tool; `build` compiles the project; `--release` enables the optimized profile. `--features diagnostic-logging` includes the optional logger. Rust `#[cfg(feature = "diagnostic-logging")]` removes guarded code from the silent build at compile time; a config setting cannot bring that code back. Run this in the folder containing the main `Cargo.toml`. The Skyline CLI and its Switch toolchain must already be configured.

After changing Rust code, build again and copy the newly generated `target/aarch64-skyline-switch/release/libkirby_assist.nro` into your SD card's `atmosphere/contents/01006A800016E000/romfs/skyline/plugins/` directory while Smash is closed. Replace the existing file and restart Smash. Building alone does not update the Switch or the downloadable ZIP. For a fresh copyable folder and ZIP, follow [the packaging commands](INTEGRATION.md#rebuild-the-sd-card-packages).

Changing supported numbers in `SD:/ultimate/kirby_assist/config.txt` needs a game restart. Fighter functions, menu registrations and compiled timing defaults are Rust source changes and require an authorized rebuild/redeploy. There is no fighter JSON file. Editing a README needs neither a rebuild nor a game restart.

Passing the tests proves the tested logical behavior. Building the `.nro` proves compilation for the target. Only testing a concrete live adapter in the target game can demonstrate that the game receives and responds to the intended input.

For learning Rust beyond this project, the official [Rust Book](https://doc.rust-lang.org/book/) is a useful companion. Start with variables, structs, enums, ownership/borrowing, and traits, then reread the walkthrough with those concepts in mind.

For held-L ledge autopilot, read [LEDGE_TRAP.md](LEDGE_TRAP.md).
Held-L3 Kirby recovery cover is documented in [TWO_FRAME.md](TWO_FRAME.md). These guides walk through observations, state and input composition. Source
changes need a separately built and installed NRO.
