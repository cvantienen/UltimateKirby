# Project Instructions

## Project Scope

- Primary Rust development is located in `crates/kirby_assist_core/`.
- Rust source files: `crates/kirby_assist_core/src/`.
- Crate documentation: `crates/kirby_assist_core/docs/`.
- Make code and documentation changes in these locations when relevant.
- Do not overlook existing Rust modules, files, or crate functionality when implementing changes.
- Do not assume files outside these directories are part of the task. If additional files, dependencies, workspace configuration, or external documentation need to be read or modified, ask for permission first.
- Read only the specific files necessary to understand and implement the requested change.

## Project Maintenance

- Append a short summary of completed changes to the `[package].description` in the relevant `Cargo.toml`. Preserve all existing description text; never replace, trim, reorder, or remove previous summaries.
- Do not bump versions unless explicitly requested.
- Keep automation input-only. Never implement assistance by modifying native physics, hitboxes, move definitions, animation rates, or fighter status. Read-only native observations are allowed; automated sequences must simulate controller inputs.
- Do not build the mod, deploy it, or upload anything to the Nintendo Switch unless explicitly requested.
- Documentation builds are allowed when validating documentation changes.

## Efficient Agent Workflow

- Reuse existing project documentation, implementations, and commands. Avoid rediscovering information already documented.
- Read only relevant files. Avoid recursively scanning the repository or opening unrelated modules.
- Before implementing, inspect the immediate dependencies and existing patterns necessary for the task.
- Prefer targeted edits over unnecessary refactoring or rewriting entire files.
- Avoid duplicating existing functionality, constants, types, or utilities.
- Ask before expanding the scope or accessing unrelated directories.
- Do not generate lengthy plans for straightforward changes. Plan only when the task involves substantial architectural changes or multiple dependent steps.
- Run the smallest relevant check first. Prefer targeted Rust tests and checks over workspace-wide builds.
- Do not run expensive builds, full test suites, or deployment commands unless necessary or requested.
- Do not repeat checks that have already passed unless subsequent changes could affect them.

## Completion

- Briefly report what changed, which checks were performed, and any remaining issues.
- Mention any assumptions or unresolved questions.
- Do not provide lengthy explanations of unchanged code or routine operations.