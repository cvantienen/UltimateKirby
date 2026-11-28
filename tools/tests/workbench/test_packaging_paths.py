import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[2]
REPO = TOOLS.parent


class PackagedPathTests(unittest.TestCase):
    def test_selected_project_root_is_used_by_core_helper_modules(self):
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            (project / "Cargo.toml").write_text('[package]\nname = "fixture"\nversion = "1.2.3"\n')
            env = {**os.environ, "KIRBY_ASSIST_PROJECT_ROOT": str(project)}
            check = """
import build_variants, sequence_sources, upload_switch, workbench_history
project = Path(os.environ['KIRBY_ASSIST_PROJECT_ROOT'])
workbench_history.ROOT = project
sequence_sources.ROOT = project
sequence_sources.SOURCE = project / 'deployment/ultimate/kirby_assist/sequences.txt'
upload_switch.PROJECT_ROOT = project
upload_switch.DEPLOYMENT_SEQUENCES = sequence_sources.SOURCE
build_variants.ROOT = project
assert build_variants.ROOT == sequence_sources.ROOT == upload_switch.PROJECT_ROOT == workbench_history.ROOT == project
assert sequence_sources.SOURCE == project / 'deployment/ultimate/kirby_assist/sequences.txt'
assert upload_switch.DEPLOYMENT_SEQUENCES == sequence_sources.SOURCE
"""
            result = subprocess.run([sys.executable, "-c", f"import os; from pathlib import Path;\n{check}"], cwd=REPO,
                                    env={**env, "PYTHONPATH": str(TOOLS)}, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_packager_includes_the_toml_stdlib_import(self):
        builder = (TOOLS / "build_workbench_macos.py").read_text()
        self.assertIn('"tomllib"', builder)
        self.assertIn('"build_variants"', builder)

    def test_frozen_helper_routes_run_through_the_bundled_entrypoint(self):
        check = "import sys, workbench; workbench.IS_FROZEN=True; print(workbench.helper('build_variants.py', '--variant', 'silent'))"
        result = subprocess.run([sys.executable, "-c", check], cwd=REPO,
                                env={**os.environ, "PYTHONPATH": str(TOOLS)}, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--workbench-helper", result.stdout)
        self.assertIn("build_variants", result.stdout)
        self.assertIn("--variant", result.stdout)


if __name__ == "__main__":
    unittest.main()
