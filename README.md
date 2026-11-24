# Kirby Assist

Kirby Assist is an input-only Rust Skyline plugin for Super Smash Bros. Ultimate.
It reads gameplay observations and requests simulated controller inputs for the
selected local human fighter; it does not modify native moves, hitboxes, physics,
or fighter status.

The [project documentation](docs/index.md) is the maintained guide for setup,
controls, configuration, features, architecture, development, and releases.
Start with the [installation guide](docs/INTEGRATION.md) or the
[architecture walkthrough](docs/ARCHITECTURE.md).

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

**Check Rust formatting:**
```sh
cargo fmt --all -- --check
```

**Run core lint checks offline:**
```sh
cargo clippy --manifest-path crates/kirby_assist_core/Cargo.toml --all-targets --offline -- -D warnings
```

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

**Stage all changes and commit them** (review `git status` first):
```sh
git add -A && git commit -m "state updates"
```

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