# Project Instructions

## Scope and Safety

- Primary development: portable Rust in `crates/kirby_assist_core/` and native integration in root `src/`. Task-relevant Rust in any crate, tests, manifests and configuration may be inspected and edited without additional permission.
- Keep automation input-only. Never modify native physics, hitboxes, move definitions, animation rates or fighter status. Assistance must simulate controller inputs.
- Preserve existing behavior outside the requested change, public contracts, feature gates and native/portable boundaries. Reuse established implementations; avoid unrelated refactoring, formatting or cleanup.
- Do not bump versions, change unrelated Cargo descriptions, build/deploy/upload to the Switch, or create repositories, branches, commits, pull requests or issues unless requested. Do not push unless requested.
- Documentation and existing logs require explicit task authorization before access. Only change documentation when requested; authorized documentation defaults to `crates/kirby_assist_core/docs/` unless the user explicitly includes other paths. Instruction files are exempt.
- Honor access restrictions and exclusions. Generated/build artifacts, assets, deployment data and unrelated directories are not discovery targets. Search exclusions are not a filesystem sandbox.

## Current-State, Affected-Rust Coverage

- Start from the request, named files and a scoped working-tree status/diff. Preserve all pre-existing edits, including staged/untracked work; never reset or overwrite them to obtain a clean baseline.
- Treat current on-disk code as authoritative. Reuse current-chat evidence while it remains valid; refresh only ranges changed since inspection or needed to resolve new evidence. Do not assume previous reports or file maps describe the current implementation.
- Before editing, trace the affected behavior: definitions, callers, consumers and relevant tests. Follow dependencies across root `src/` and any affected crate until every changed contract has its consumers accounted for. This is not limited to initially named files.
- For cross-cutting Rust changes, first obtain a filenames-only inventory of the relevant roots and search all candidate files for affected symbols/contracts. Narrow the implementation reads; do not reread every Rust file for a local fix. Full-repository review is for explicitly requested audits or genuinely repository-wide changes.
- Follow only applicable paths: configuration defaults -> parsing/validation -> persistence/menu -> runtime consumers; controller requests -> arbitration/ownership -> mapping/cancellation; public types/signatures -> imports/reexports/callers and feature-gated variants. Check all affected paths rather than only the first working call site.
- Preserve timing, frame/clock units, request lifetimes, cancellation rules, missing-observation handling and user-input priority unless changing them is the task. Do not bypass runtime safety gates to make an assist appear to work.
- Include affected private unit tests under each package's `tests/unit/`, integration tests and conditional builds. Use scoped searches to check for stale call sites, duplicated behavior and obsolete assumptions after a shared change.
- Keep coverage proportional: a small task needs no ledger. For a large change, retain a compact list of affected paths and unresolved dependencies, not a repeated source dump. Never claim repository-wide coverage from a targeted check.

## Efficient Inspection and Editing

- Use a short inspect -> edit -> verify loop. Start with named files or scoped filename globs; use scoped symbol/reference tools when available. No repository tours or speculative architecture reviews.
- Scope Rust searches to task-relevant directories and `*.rs`; use separate filters for related tests/configuration. Initially cap content results at 30 lines and read roughly 40–100 lines around matches; expand when needed to understand the complete function or contract.
- A capped result is not exhaustive evidence. When completeness matters, obtain matching filenames/counts first and inspect remaining relevant matches in bounded batches.
- Avoid workspace-wide semantic searches, recursive listings, whole large-file dumps and raw log dumps. If output is saved to a file, search it and read only the necessary ranges.
- Make precise, coherent edits and save each completed logical chunk to disk before starting another. Prefer editing existing blocks over regenerating whole files. Do not combine unrelated work into a giant patch or script.
- Batch independent lookups when useful; keep results bounded. Do not repeat reads/checks without a changed input, new failure or specific missing fact. Ask only about material ambiguity, new authorization or a genuine blocker.
- Handle ordinary work directly. Delegate only explicitly requested or substantial independent work, with bounded ownership; do not duplicate an agent's investigation or launch agents for small lookups/checks.
- Use current-chat evidence and task-relevant local files. Do not query session history, previous chats, memories, GitHub, remote repositories, web or cloud services unless requested.
- Prefer local file tools and existing tooling. Do not invoke MCP tools, custom skills, toolboxes or extension-provided AI tools for routine local edits unless higher-priority instructions require them.

## Bounded Output and Interruption Recovery

- Reduce avoidable context growth, not correctness: no huge generated replies, repeated plans, source echoes, broad search dumps or repeated progress narration. Split large results/edits into useful bounded chunks; never truncate required implementation or evidence merely to stay brief.
- For verbose checks, retain full output in a session artifact and return the exit status plus a short summary/failure excerpt. Do not pipe away failures or confuse truncated output with a passed check.
- Give long-running checks a realistic wait time. If a command remains running, retain its handle and read its eventual result; do not rerun it, poll repeatedly or kill it solely because the first wait expired. Respect user cancellation.
- Only for long or interrupted tasks, maintain a small non-Markdown checkpoint in the session artifact directory: objective, edited paths, completed checks/results, pending work and active handles. Update at meaningful milestones, not after every tool call; do not add tracking files to the repository.
- After interruption, inspect the saved checkpoint and scoped current diff, verify which commands actually completed, then resume the missing work. Do not restart the entire investigation or assume unsaved/planned changes were applied.
- Do not add speculative retries, extra agents or model calls as safeguards. On rate limits/timeouts, honor any retry delay and use a bounded retry only when safe; report a persistent blocker rather than loop. Check for partial effects before repeating a write.
- These instructions cannot override provider rate/token limits or prevent every interruption. Do not change editor/provider settings, permissions or model selection unless requested.

## Focused Validation

- Choose the smallest existing check that exercises the changed behavior. Add a few regression cases for introduced/fixed behavior and meaningful edge cases; do not weaken existing assertions or add tests of trivial constants/accessors.
- For portable logic, prefer a targeted host command such as `cargo test --offline --manifest-path crates/kirby_assist_core/Cargo.toml --test <target>` or `--lib <filter>`. Choose selectors from the actual test layout; include relevant feature flags when the change affects gated behavior.
- Host compilation required by a focused test/check is allowed. No standalone build or full/workspace suite by default; broaden only when requested or necessary for affected shared contracts or targeted failures. Explain the reason briefly. A core host check does not validate native Switch integration.
- Use existing format/lint tools on the changed scope without mass-formatting unrelated files. Inspect the final scoped diff for accidental behavior changes and whitespace errors.
- Do not install/restore dependencies unless manifests changed or the chosen check failed for missing dependencies. Never report an unavailable check as passed.
- Documentation-only tasks need no code build/tests unless existing documentation checks cover the edit. Reuse valid completed results; rerun only checks whose evidence was invalidated by subsequent edits.

## Completion

- Confirm edits are saved and affected paths/checks are accounted for. Distinguish your changes from pre-existing work and passed checks from unverified behavior.
- Briefly summarize changes, affected coverage, checks and remaining issues. If interrupted or blocked, state exactly what is saved and what remains; never label partial work complete.
