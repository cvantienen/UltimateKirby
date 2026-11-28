"""Launch from tools with python -m workbench_reminal_ui, or run this file."""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import sys


def main():
    parser = argparse.ArgumentParser(description="Kirby Assist terminal workbench (idle on startup)")
    parser.add_argument("--project", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--state-dir", type=Path, help="Shared journal/preferences; only one workbench may use it")
    args = parser.parse_args()
    project = args.project.expanduser().resolve()
    state = (args.state_dir or Path(os.environ.get("KIRBY_ASSIST_STATE_DIR", project / ".mod-workbench"))).expanduser().resolve()
    if not (project / "Cargo.toml").is_file() or not (project / "tools/build_variants.py").is_file():
        parser.error("Choose a valid UltimateKirby project root")
    if state.is_relative_to(project) and ".mod-workbench" not in state.relative_to(project).parts:
        parser.error("Keep state in .mod-workbench or outside the project")
    os.environ["KIRBY_ASSIST_PROJECT_ROOT"] = str(project)
    os.environ["KIRBY_ASSIST_STATE_DIR"] = str(state)
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    import workbench
    sys.argv = [sys.argv[0], '--terminal', '--project', str(project), '--state-dir', str(state)]
    workbench.main()


if __name__ == "__main__":
    main()
