# Maintaining the documentation

All maintained guides live in `docs/`; `index.md` is the homepage. Run the
following commands from the project root, beside `Cargo.toml` and `mkdocs.yml`.

## Set up Python once

```sh
python3 -m venv .venv-docs
.venv-docs/bin/python -m pip install -r requirements-docs.txt
```

Use the virtual environment so documentation packages stay separate from your
system Python. To update the pinned MkDocs version, edit `requirements-docs.txt`,
reinstall it, and run the strict build below.

This workspace's environment was created with `/usr/bin/python3` (Python 3.9),
after Homebrew Python environment setup failed. If `python3 -m venv` fails in
`ensurepip` or cannot import `pyexpat`, use `/usr/bin/python3 -m venv .venv-docs` when
creating a fresh environment. The project does not change system Python.

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

The generated HTML goes into `site/`. Warnings such as missing pages or broken
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

Record planned work using the [TODO template](TODO.md). State which release a
behavior applies to, and distinguish host checks from Switch measurements.

Each registered fighter has a page under `docs/fighters/`, linked from the
[Fighters overview](FIGHTERS.md). Refresh its generated choices/defaults and
frame data after changing compiled fighter declarations:

```sh
python3 tools/update_fighter_docs.py
python3 tools/update_fighter_docs.py --check
```

This reads static Rust declarations without compiling the mod. It rejects
unrecognized catalog syntax before writing pages. Keep authored notes outside
the generated markers; the updater preserves them. Add new guides below the
overview rather than putting every fighter in the sidebar.

## Release documentation

Change `[package].version` in the root `Cargo.toml` for a new release.
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
