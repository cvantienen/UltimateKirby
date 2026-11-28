# Project Instructions

## Project Scope
- Primary development: `crates/kirby_assist_core/` and root `src/`.

- Source reads, searches, and edits may cover task-relevant Rust files in root `src/` and any crate without additional permission.
- Scope glob/grep explicitly to task-relevant directories and `*.rs`; use bounded line ranges, not whole large files or recursive repository listings. Read immediate dependencies only as needed.
- Task-relevant tests, dependency manifests, and configuration may be inspected and edited without additional permission. Documentation and logs still require explicit task authorization or permission before access. These instructions are exempt. Do not read or update Cargo descriptions for unrelated code changes.
- Documentation, when authorized: `crates/kirby_assist_core/docs/`. Ask before expanding scope.
- These are agent workflow restrictions, not a filesystem sandbox; search exclusions do not enforce access control.

## Development Rules
- **Documentation:** Only create, update, or reorganize documentation when explicitly requested. Do not spend credits reviewing or maintaining docs during code-only tasks.
- Do not bump versions unless requested.
- Keep automation input-only. Never modify native physics, hitboxes, move definitions, animation rates, or fighter status. Assistance must simulate controller inputs.
- Do not build, deploy, or upload to the Switch unless requested.

## Efficient Workflow
- Default to a small-task workflow: locate the relevant code, read the smallest necessary ranges, make the change, and run one focused check when needed. No repository tours, architecture audits, or speculative exploration.
- Reuse evidence already in the conversation. Do not reread files or repeat searches unless an edit or new evidence makes it necessary.
- Start with named files or a scoped filename glob. Use content searches only within the relevant files/directory, capped at 30 matching lines. Read roughly 40–100 lines around a match; expand only when needed.
- Do not use workspace-wide semantic searches, recursive listings, whole-file dumps, or raw log dumps. Request file paths/counts instead of contents when locating code; inspect only the relevant part of oversized tool output.
- Handle ordinary tasks directly. Do not launch subagents unless explicitly requested or substantial independent work requires separate context; never delegate a small lookup, edit, or check.
- Work on the current code task only. Do not query session history, previous chats, memories, GitHub, remote repositories, web, or cloud services unless explicitly requested. Use only current-chat evidence and task-relevant local files.
- Never create repositories, branches, commits, pull requests, or issues, or push changes, unless requested.
- Do not invoke MCP tools, custom skills, toolboxes, or extension-provided AI tools for routine local edits unless required by higher-priority instructions. Prefer scoped local file tools and the smallest necessary terminal check.
- Batch independent lookups/checks and keep outputs bounded. Before expanding scope, identify the specific missing evidence; ask permission where required.
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
