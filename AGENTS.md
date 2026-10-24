# Project maintenance

- Keep maintained documentation in `docs/` aligned with behavior, controls,
  configuration and tooling changes. Fighter pages belong under `docs/fighters/`
  and must be linked from the Fighters overview, not individually in the sidebar.
  After catalog/timing edits, run `python3 tools/update_fighter_docs.py` and its
  `--check` mode; keep fighter-specific authored execution notes current too.
- Append a small summary of completed changes to `[package].description` in
  `Cargo.toml`. Preserve all existing description text; do not replace, trim,
  reorder or remove older summaries. Do not bump the version unless requested.
- Do not build the mod or upload to the Switch unless the user explicitly asks.
  Documentation builds are allowed when validating documentation changes.

## Efficient agent workflow

- Search narrowly, then read only the relevant files and surrounding code.
  Avoid dumping the repository, generated files, archives, or full diffs unless
  the task requires them.
- Follow the relevant code path instead of scanning unrelated subsystems. Reuse
  existing project docs and commands rather than re-explaining them.
- Run the smallest relevant check first. Use the core crate tests for portable
  Rust logic; use broader checks only when the change warrants them.
- Keep final responses to the result, checks run, and any remaining caveat.
