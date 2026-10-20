"""Builder packaging tests; no plugin compilation or network connections."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import build_variants as builder


class BuildTests(unittest.TestCase):
    def test_defaults_and_variant_selection_are_build_only(self):
        self.assertEqual(builder.parse_options([]).variant, "all")
        options = builder.parse_options(["--cargo-home", "/private/tmp/kirby-assist-build-cargo"])
        self.assertEqual(options.variant, "all")
        self.assertEqual(options.cargo_home, Path("/private/tmp/kirby-assist-build-cargo"))
        self.assertEqual(builder.parse_options(["-v", "diagnostic"]).variant, "diagnostic")
        self.assertEqual(builder.parse_options(["-v", "all"]).variant, "all")
        for args in [["-u", "192.0.2.10"], ["--upload", "192.0.2.10"]]:
            with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                builder.parse_options(args)

    def test_uploader_checks_the_same_diagnostic_markers_as_packager(self):
        import upload_switch
        self.assertEqual(upload_switch.LOG_MARKERS, builder.LOG_MARKERS)

    def run_packaging(self, base, selection):
        project = base / "project"
        project.mkdir(exist_ok=True)
        (project / "Cargo.toml").write_text("# source fixture")
        def fake_build(variant, env):
            stage = project / "releases" / variant
            for file, data in ((builder.PLUGIN, f"{variant} NRO".encode()),
                               (builder.CONFIG, f"{variant} config".encode()),
                               (Path(builder.RELEASE_SUMMARY_NAME), builder.variant_release_summary(variant))):
                target = stage / file
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
        with patch.object(builder, "ROOT", project), \
                patch.object(builder, "parse_options", return_value=builder.parse_options(["-v", selection])), \
                patch.object(builder, "read_release_summary", return_value=b"summary"), \
                patch.object(builder, "build_variant", side_effect=fake_build) as build, \
                contextlib.redirect_stdout(io.StringIO()):
            builder.main()
        self.assertFalse(hasattr(builder, "ftplib"))
        return base / f"Latestbuild-{builder.RELEASE_VERSION}", build

    def test_both_variants_get_exactly_four_matching_files_in_separate_folders(self):
        with tempfile.TemporaryDirectory() as temporary:
            latest, build = self.run_packaging(Path(temporary), "all")
            self.assertEqual([call.args[0] for call in build.call_args_list], ["silent", "diagnostic"])
            source_name = f"kirby_assist-{builder.RELEASE_VERSION}-source.zip"
            expected = {"libkirby_assist.nro", "config.txt", builder.RELEASE_SUMMARY_NAME, source_name}
            self.assertEqual({path.name for path in latest.iterdir()}, {"silent", "diagnostic"})
            for variant in ("silent", "diagnostic"):
                folder = latest / variant
                self.assertEqual({path.name for path in folder.iterdir()}, expected)
                self.assertEqual((folder / "libkirby_assist.nro").read_bytes(), f"{variant} NRO".encode())
                self.assertEqual((folder / "config.txt").read_bytes(), f"{variant} config".encode())
                self.assertEqual((folder / builder.RELEASE_SUMMARY_NAME).read_bytes(),
                                 f"Kirby Assist {builder.RELEASE_VERSION}\nBuild: {variant.upper()}\n\nsummary".encode())
                with zipfile.ZipFile(folder / source_name) as archive:
                    self.assertIsNone(archive.testzip())
                    self.assertIn("project/Cargo.toml", archive.namelist())
            self.assertEqual((latest / "silent" / source_name).read_bytes(),
                             (latest / "diagnostic" / source_name).read_bytes())

    def test_single_variant_refresh_preserves_other_variant_and_legacy_root_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            latest, _ = self.run_packaging(base, "all")
            original_silent = {path.name: path.read_bytes() for path in (latest / "silent").iterdir()}
            legacy = latest / "libkirby_assist.nro"
            legacy.write_bytes(b"preserve historical output")
            _, build = self.run_packaging(base, "diagnostic")
            self.assertEqual([call.args[0] for call in build.call_args_list], ["diagnostic"])
            self.assertEqual({path.name: path.read_bytes() for path in (latest / "silent").iterdir()}, original_silent)
            self.assertEqual(legacy.read_bytes(), b"preserve historical output")


if __name__ == "__main__":
    unittest.main()
