# Project maintenance
- Append a small summary of completed changes to `[package].description` in
  `Cargo.toml`. Preserve all existing description text; do not replace, trim,
  reorder or remove older summaries. Do not bump the version unless requested.
- Keep automation input-only. Never implement assistance by changing native
  physics, hitboxes, move definitions, animation rates or fighter status.
  Read-only native observations are allowed; sequences simulate controller input.
- Do not build the mod or upload to the Switch unless the user explicitly asks.
  Documentation builds are allowed when validating documentation changes.

## Efficient agent workflow

- Reuse
  existing project docs and commands rather than re-explaining them.
- Run the smallest relevant check first. Use the core crate tests for portable
  Rust logic; use broader checks only when the change warrants them.
- Keep final responses to the result, checks run, and any remaining caveat.
