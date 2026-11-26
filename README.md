# Kirby Assist

Kirby Assist is an input-only Rust Skyline plugin for Super Smash Bros. Ultimate.
It reads gameplay observations and requests simulated controller inputs for the
selected local human fighter; it does not modify native moves, hitboxes, physics,
or fighter status.

The [project documentation](docs/index.md) is the maintained guide for setup,
controls, configuration, features, architecture, development, and releases.
Start with the [installation guide](docs/INTEGRATION.md), then the detailed
[settings and modes walkthrough](docs/SETTINGS_REFERENCE.md). It explains shared
settings, fighter overrides, temporary match switches, exact configuration keys,
units, saving, and troubleshooting. Use the [fighter settings guide](docs/FIGHTER_SETTINGS.md)
and [sequence/editor guides](docs/SEQUENCE_FILES.md) for individual values and
input tapes.

New to Rust? Read [Development](docs/DEVELOPMENT.md) and the
[core developer guide](crates/kirby_assist_core/docs/DEVELOPER_GUIDE.md) before
changing a setting or mode in code. They cover crates/modules, data types,
configuration-to-policy wiring, private unit tests, input ownership, and safe
extension workflows. The [architecture walkthrough](docs/ARCHITECTURE.md)
shows how observations become controller requests.

Dedicated guides explain [Auto-Spacing & Attack](docs/AUTO_SPACING.md),
[Counter](docs/AUTO_COUNTER.md), [Auto Defense](docs/AUTO_DEFENSE.md),
[projectile reflection](docs/PROJECTILE_REFLECT.md), and the other
[assist features](docs/index.md#understand-a-feature). Text configuration edits
and Rust implementation edits are different workflows: the former can be
loaded after a restart; the latter require a separately built/installed plugin.

## Commands by workflow

Run commands from the repository root. The build/upload examples below are
manual deployment workflows, not prerequisites for reading or editing docs.
Do not build or upload while Smash is running; preserve the active SD
configuration and choose a single plugin variant deliberately.

**Build the optimized Switch plugin (silent):**
```sh
cargo skyline build --release --no-default-features --features imgui-menu
```

**Build both packaged variants (silent and diagnostic):**
```sh
python3 tools/build_variants.py
```

**Upload a built silent variant to your Switch** (replace both placeholders; close Smash and start the Switch FTP server first):
```sh
python3 tools/upload_switch.py <SWITCH_IP> -v <VERSION> -s
```

**Upload a diagnostic variant instead:**
```sh
python3 tools/upload_switch.py <SWITCH_IP> -v <VERSION> -d
```

**Run the core Rust tests offline:**
```sh
cargo test --manifest-path crates/kirby_assist_core/Cargo.toml --offline
```

**Run the diagnostic-feature host suite:**
```sh
cargo test --offline --manifest-path crates/kirby_assist_core/Cargo.toml --all-features --no-fail-fast
```

**Check formatting for a touched Rust file:**
```sh
rustfmt --check --edition 2021 --config skip_children=true path/to/changed.rs
```

Existing whole-file formatting differences remain in some sources. Avoid
rewriting unrelated code solely to make a repository-wide formatting check pass.
New external unit-test files should be fully formatted.

**Run core lint checks offline:**
```sh
cargo clippy --offline --manifest-path crates/kirby_assist_core/Cargo.toml --all-features --all-targets
```

The existing lint baseline has warnings. `-D warnings` is stricter than that
baseline and is not a claim that all existing warnings are already resolved.

**Build the documentation strictly:**
```sh
.venv-docs/bin/python -m mkdocs build --strict
```

**Run the local documentation preview:**
```sh
.venv-docs/bin/python -m mkdocs serve
```

**Check your branch and changed files before committing:**
```sh
git status --short --branch
```

**Stage only reviewed files and commit them** (replace the example paths):
```sh
git add path/to/reviewed-file path/to/another-reviewed-file
git diff --cached --check
git diff --cached
git commit -m "Describe the specific change"
```

Do not stage unrelated work, generated artifacts, private data or secrets.
See the [GitHub workflow](docs/GITHUB_WORKFLOW.md) for the complete process.

**Push the current branch to GitHub:**
```sh
git push -u origin HEAD
```

**Create a PR targeting `main`** (after pushing a new branch):
```sh
gh pr create --base main --title "state updates" --body "state updates"
```

**Open the PR for your current branch in a browser:**
```sh
gh pr view --web
```

The upload `<VERSION>` must match the version in the root `Cargo.toml` and the build folder produced for that version.