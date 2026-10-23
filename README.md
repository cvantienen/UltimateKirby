# Kirby Assist

Kirby Assist is a Rust plugin for Super Smash Bros. Ultimate. It adds configurable
controller-input assists for the selected human-controlled fighter. The plugin targets a
Nintendo Switch running Skyline and uses the Player 1 controller.

> **Testing status:** Host tests check the shared logic, but they do not verify
> gameplay or compatibility on a Switch. Test device changes carefully.

## Features

- Auto mash and auto tech
- R3 turbo teabag, X short hop, Y B-reverse and Kirby L3 recovery cover
- Configurable ledge assist and ledge trump
- Silent and diagnostic plugin builds

Settings are read from `SD:/ultimate/kirby_assist/config.txt` when Smash starts.
All non-DLC fighter profiles, menu lists and action sequences are compiled Rust.
No fighter JSON is loaded. Restart after editing shared settings; new Rust
responses require a separate rebuild/install. Install only one build variant.
See [current controls](docs/CONTROLS.md) and [fighters](docs/FIGHTERS.md).

## Quick start

Run the host-side tests from the project root:

```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml
```

Build the Switch plugin with a configured Skyline toolchain:

```sh
cargo skyline build --release
```

See [installation and configuration](docs/INTEGRATION.md) for setup details.

## TODO

- [ ] Investigate doubles connection error.
- [ ] Review menu size on device.
- [ ] Future combo work; live combo playback is currently removed.
- [ ] Extend and calibrate fighter response options.
- [x] Superseded fighter JSON with compiled per-fighter response functions and timing data.

## Documentation

- [Controls and ledge assist](docs/LEDGE_TRAP.md)
- [Directional turbo](docs/TURBO_CONTROLS.md)
- [Analog mash](docs/ANALOG_MASH.md)
- [Build variants](docs/BUILD_VARIANTS.md)
- [Rust beginner guide](docs/README_BEGINNER.md)
- [Source map](docs/CODE_MAP.md)
- [Compiled fighter settings](docs/FIGHTER_CONFIGS.md)
- [All project documentation](docs/index.md)
