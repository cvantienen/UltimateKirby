# Maintaining the documentation

The maintained site guides live in `docs/`; `index.md` is the homepage.
The canonical beginner Rust/API tutorial also lives in
`crates/kirby_assist_core/docs/DEVELOPER_GUIDE.md`. Keep that one detailed tutorial
and point site readers through [Development](DEVELOPMENT.md), rather than copying
it into every feature page. Run the following commands from the project root,
beside `Cargo.toml` and `mkdocs.yml`.

## Set up Python once

```sh
python3 -m venv .venv-docs
.venv-docs/bin/python -m pip install -r requirements-docs.txt
```

Use the virtual environment so documentation packages stay separate from your
system Python. To update the pinned MkDocs version, edit `requirements-docs.txt`,
reinstall it, and run the strict build below.

## Preview while editing

```sh
.venv-docs/bin/python -m mkdocs serve
```

Open `http://127.0.0.1:8000/`. Saving a Markdown file refreshes the preview.
Stop the preview with Ctrl+C.

## Check and build

```sh
.venv-docs/bin/python -m mkdocs build --strict
```

By default generated HTML goes into `site/`. For an isolated validation run use
`--site-dir PATH` with a caller-approved output directory instead; do not inspect
or edit generated site output as source. Warnings such as missing pages or broken
anchors fail the strict build. Both `site/` and `.venv-docs/` are ignored by Git
and excluded from source-backup ZIPs. This command builds locally; it does not
publish a website or change the Switch plugin.

## Add or edit a guide

1. Edit Markdown in `docs/`, or create a new `.md` file there.
2. Add general pages to `nav` in `mkdocs.yml`. For fighter guides, link them from
   `FIGHTERS.md` and list their paths in `not_in_nav` so they stay out of the sidebar.
3. Link guides relative to each other, for example `[Ledge trap](LEDGE_TRAP.md)`.
4. Use inline code for project-relative source/config paths outside `docs/`;
   those files are not published as documentation assets.
5. Preview the result and run the strict build.

Record planned work using the [maintainer backlog](TODO.md). Preserve historical
notes and mark source verification separately from device validation. Never turn
an old success count or a source-level refactor into a fresh hardware claim.
Distinguish Rust fallback defaults from actual supplied configuration and
fighter overrides; their values can differ.

Each registered fighter has a page under `docs/fighters/`, linked from the
[Fighters overview](FIGHTERS.md). Refresh its generated choices/defaults and
file menus/schedules and reference data after changing `sequences.txt` or fighter reference declarations:

```sh
python3 tools/update_fighter_docs.py
python3 tools/update_fighter_docs.py --check
```

This reads static Rust declarations without compiling the mod. It rejects
unrecognized catalog syntax before writing pages. Keep authored notes outside
the generated markers; the updater preserves them. Add new guides below the
overview rather than putting every fighter in the sidebar.

## Release documentation

Release changes are a separate, explicitly requested task. A docs-only update
must not bump versions, append Cargo descriptions, build or upload the plugin,
or refresh historical release artifacts. Preserve existing uncommitted edits.

When preparing an authorized new release, change `[package].version` in the root `Cargo.toml`.
Append a small summary of completed changes to `[package].description`; preserve
all existing text and earlier summaries. Do not replace, reorder or remove old
entries. Keep the docs aligned with each behavior/tooling change. The builder reads both
fields from Cargo and generates `releases/RELEASE_SUMMARY-<version>.txt` from the
description automatically; manual edits to that generated file are overwritten. The [release builder](BUILD_VARIANTS.md) packages the complete
`docs/` tree and MkDocs setup alongside the release summary. New Markdown pages
are included automatically; no Python guide list needs updating.

Source backups include the Markdown, MkDocs config and dependency file, while
excluding generated HTML and the Python environment. Historical release folders
and source archives remain snapshots; edit the maintained `docs/` tree for the
next release.

## Standalone cheat sheets

The Markdown sequence-syntax, editor/action and reaction-rule references are the
source for their styled searchable HTML copies. After changing a reference, run:

```sh
.venv-docs/bin/python tools/update_cheatsheets.py
.venv-docs/bin/python tools/update_cheatsheets.py --check
```

Then validate navigation and links with `.venv-docs/bin/python -m mkdocs build --strict`.
Start the local site with `.venv-docs/bin/python -m mkdocs serve`.
