# Project Instructions

## Project Scope
- Primary development: `crates/kirby_assist_core/`.
- Default source reads/searches: only task-relevant `crates/kirby_assist_core/src/**/*.rs`. Do not read root `src/` or other crates.
- Scope glob/grep explicitly to that directory and `*.rs`; use bounded line ranges, not whole large files or recursive repository listings. Read immediate dependencies only as needed.
- Non-Rust files, tests outside `src/`, documentation, logs, and configuration require explicit task authorization or permission before access. Exceptions: these instructions and the crate `Cargo.toml` for the required description update.
- Documentation, when authorized: `crates/kirby_assist_core/docs/`. Ask before expanding scope.
- These are agent workflow restrictions, not a filesystem sandbox; search exclusions do not enforce access control.

## Development Rules
- **Documentation:** Only create, update, or reorganize documentation when explicitly requested. Do not spend credits reviewing or maintaining docs during code-only tasks.
- Do not bump versions unless requested.
- Keep automation input-only. Never modify native physics, hitboxes, move definitions, animation rates, or fighter status. Assistance must simulate controller inputs.
- Do not build, deploy, or upload to the Switch unless requested.

## Efficient Workflow
- Minimize token usage: solve locally with the smallest relevant evidence. No speculative exploration, repeated reads, or raw log/output dumps.
- Handle ordinary Rust tasks directly; do not launch subagents unless requested or substantial independent work requires them. Give delegated tasks the same read scope and bounded output.
- Batch independent lookups/checks; reuse results. Broaden searches only when targeted evidence is insufficient.
- Project instructions cannot remove tool-schema overhead. Keep unnecessary MCP tools disabled in this workspace; re-enable only when needed, then start a fresh session.
- Reuse existing implementations, documentation, and established patterns instead of rediscovering them.
- Make targeted changes. Avoid unnecessary refactoring, rewriting, or expanding scope.
- Do not generate lengthy plans for straightforward tasks.
- Avoid reviewing unrelated modules or explaining existing code unless necessary.
- Run only the smallest relevant checks. dont ever build, workspace-wide tests, and repeated checks unless requested or necessary.
- Do not perform additional cleanup or improvements outside the requested task.

## Completion
- Briefly summarize changes, checks performed, and any remaining issues.
- Keep responses concise and avoid lengthy explanations.