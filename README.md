# Kirby Assist

Kirby Assist is a Rust plugin for Super Smash Bros. Ultimate. It adds configurable
controller-input assists for a human-controlled Kirby. The plugin targets a
Nintendo Switch running Skyline and uses the Player 1 controller.

> **Testing status:** Host tests check the shared logic, but they do not verify
> gameplay or compatibility on a Switch. Test device changes carefully.

## Features

- Auto mash and auto tech
- Directional turbo crouch
- Configurable ledge assist and ledge trump
- Silent and diagnostic plugin builds

Settings are read from `SD:/ultimate/kirby_assist/config.txt` when Smash starts.
Restart the game after changing the file. Install only one build variant.

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

- [ ] Fix ledge-trap homing issues.
- [ ] Fix predicted grab-startup issues.
- [ ] Validate the ledge-drop shield-defense option.
- [ ] Add dynamic ledge-trap options loaded from config for each character.

## Documentation

- [Controls and ledge assist](docs/LEDGE_TRAP.md)
- [Directional turbo](docs/TURBO_CONTROLS.md)
- [Analog mash](docs/ANALOG_MASH.md)
- [Build variants](docs/BUILD_VARIANTS.md)
- [Rust beginner guide](docs/README_BEGINNER.md)
- [All project documentation](docs/index.md)
