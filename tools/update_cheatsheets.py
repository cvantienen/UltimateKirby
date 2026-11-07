#!/usr/bin/env python3
"""Regenerate the standalone searchable editor/rule cheat sheets from Markdown.

Run from the repository root:
    .venv-docs/bin/python tools/update_cheatsheets.py
    .venv-docs/bin/python tools/update_cheatsheets.py --check

Uses Markdown, already supplied by the MkDocs documentation environment. No
Switch access, gameplay changes, network requests or mod builds are performed.
"""
from __future__ import annotations
import argparse
from pathlib import Path
import markdown

ROOT = Path(__file__).resolve().parents[1]
PAGES = ("EDITOR_ACTION_CHEATSHEET", "REACTION_RULE_CHEATSHEET", "SEQUENCE_SYNTAX_CHEATSHEET")
STYLE = """*{box-sizing:border-box}body{margin:0;background:#edf2f7;color:#182c42;font:15px/1.6 system-ui,sans-serif}main{max-width:1500px;margin:28px auto;padding:28px;background:white;border:1px solid #d9e3ef;border-radius:16px}h1{font-size:30px;margin-top:0}h2{margin-top:32px;color:#174c68}h3{color:#235369}a{color:#006f87}code{background:#eaf2f6;border-radius:4px;padding:2px 5px;font:13px/1.6 ui-monospace,monospace}pre{background:#102a3c;color:#e7f5ff;padding:18px;overflow:auto;border-radius:10px}pre code{background:transparent;color:inherit;padding:0}table{width:100%;border-collapse:collapse;font-size:14px;margin:14px 0 26px}th{text-align:left;background:#123c52;color:white;padding:12px;vertical-align:top}td{border-bottom:1px solid #dbe6ed;padding:11px;vertical-align:top}tbody tr:nth-child(even){background:#f3f7fa}td:first-child{font-weight:600;min-width:165px}td code{overflow-wrap:anywhere}blockquote{border-left:4px solid #3b9aa4;margin-left:0;padding-left:16px;color:#43576b}.search{position:sticky;top:0;z-index:1;background:#edf2f7ed;backdrop-filter:blur(6px);padding:14px max(20px,calc((100vw - 1444px)/2));border-bottom:1px solid #c9d8e6;display:flex;align-items:center;gap:14px}input{width:min(700px,80vw);font:inherit;padding:10px 14px;border:1px solid #a6bece;border-radius:9px}.search span{font-size:13px;color:#38556c}tr[hidden]{display:none}@media(max-width:850px){main{padding:16px;margin:12px}table{display:block;overflow-x:auto}h1{font-size:24px}.search{padding:10px}td{min-width:180px}}@media print{body,main{background:white}main{border:0;padding:0;margin:0}.search{display:none}table{font-size:10px}td,th{padding:6px}pre{white-space:pre-wrap}h2{break-after:avoid}tr{break-inside:avoid}tr[hidden]{display:table-row}}"""
SCRIPT = """const input=document.getElementById('query'),rows=[...document.querySelectorAll('tbody tr')],count=document.getElementById('count');function filter(){const terms=input.value.toLowerCase().trim().split(/\\s+/).filter(Boolean);let visible=0;for(const row of rows){const text=row.textContent.toLowerCase();row.hidden=!terms.every(term=>text.includes(term));if(!row.hidden)visible++;}count.textContent=visible+' / '+rows.length+' rows';}input.addEventListener('input',filter);filter();"""


def render(source: str) -> str:
    title = source.splitlines()[0].removeprefix("# ")
    content = markdown.markdown(source, extensions=["tables", "fenced_code"])
    return (f'<!doctype html>\n<html lang="en"><head><meta charset="utf-8">'
            f'<meta name="viewport" content="width=device-width,initial-scale=1">'
            f'<title>{title}</title><style>{STYLE}</style></head><body>'
            '<div class="search"><input id="query" type="search" aria-label="Filter reference rows" '
            'placeholder="Find a move, setting, syntax or example…"><span id="count"></span></div>'
            f'<main>{content}</main><script>{SCRIPT}</script></body></html>\n')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated HTML is stale")
    args = parser.parse_args()
    changed = []
    for page in PAGES:
        source = (ROOT / "docs" / f"{page}.md").read_text()
        destination = ROOT / "docs" / f"{page}.html"
        expected = render(source)
        if not destination.exists() or destination.read_text() != expected:
            changed.append(destination.name)
            if not args.check:
                destination.write_text(expected)
    if args.check and changed:
        raise SystemExit("Stale cheat sheets: " + ", ".join(changed))
    print(("Checked" if args.check else "Refreshed") + f" {len(PAGES)} cheat sheets; {len(changed)} changed.")


if __name__ == "__main__":
    main()
