import tempfile
import plistlib
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch, Mock

import build_workbench_macos as packaging


class MacOSPackagingTests(unittest.TestCase):
    def test_selected_image_is_converted_to_macos_icns(self):
        from PIL import Image

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "alternate.webp"
            Image.new("RGBA", (32, 48), (20, 80, 160, 255)).save(source)
            image_set = root / "generated"
            image_set.mkdir()
            def record_iconset(command, check):
                files = sorted(path.name for path in Path(command[3]).glob("*.png"))
                (Path(command[5])).write_bytes(b"test icns")
                record_iconset.files = files
                return Mock(returncode=0)
            with patch.object(packaging.subprocess, "run", side_effect=record_iconset) as run:
                packaging.make_icon(source, image_set)
            self.assertEqual(len(record_iconset.files), 12)
            self.assertIn("icon_512x512@2x.png", record_iconset.files)
            self.assertEqual(run.call_args.args[0][:3], ["iconutil", "-c", "icns"])
            self.assertEqual((image_set / "AppIcon.icns").read_bytes(), b"test icns")

    def test_icon_directory_accepts_image_content_without_matching_extension(self):
        from PIL import Image

        for image_format in ("JPEG", "PNG", "WEBP", "GIF", "BMP"):
            with self.subTest(image_format=image_format), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / ".DS_Store").write_bytes(b"not an image")
                (root / "a-invalid.png").write_bytes(b"not an image either")
                (root / "a-subdirectory").mkdir()
                source = root / "custom-name.unknown"
                Image.new("RGB", (32, 48)).save(source, format=image_format)
                self.assertEqual(packaging.resolve_icon(root), source)

    def test_icon_directory_selects_first_readable_image_alphabetically(self):
        from PIL import Image

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ("z.png", "a.jpg"):
                Image.new("RGB", (32, 48)).save(root / name)
            self.assertEqual(packaging.resolve_icon(root), root / "a.jpg")
            self.assertEqual(packaging.resolve_icon(root / "z.png"), root / "z.png")

    def test_icon_directory_requires_a_readable_image_in_that_directory(self):
        from PIL import Image

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "nested").mkdir()
            Image.new("RGB", (32, 48)).save(root / "nested" / "image.png")
            (root / "invalid.png").write_bytes(b"invalid")
            with self.assertRaisesRegex(FileNotFoundError, "No readable app icon image"):
                packaging.resolve_icon(root)
            with self.assertRaisesRegex(FileNotFoundError, "not found"):
                packaging.resolve_icon(root / "missing.png")

    def test_builder_rejects_non_macos_build_hosts(self):
        with patch.object(packaging.sys, "platform", "linux"):
            with self.assertRaisesRegex(RuntimeError, "requires macOS"):
                packaging.build(packaging.DEFAULT_ICON, "test-app")

    def test_final_bundle_is_signed_after_metadata_and_icon_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            app = Path(directory) / 'Workbench.app'
            resources = app / 'Contents' / 'Resources'
            resources.mkdir(parents=True)
            manifest = app / 'Contents' / 'Info.plist'
            manifest.write_bytes(plistlib.dumps({'CFBundleExecutable': 'Workbench',
                                                 'LSMinimumSystemVersion': '13.0'}))
            icon = Path(directory) / 'custom.webp'
            icon.write_bytes(b'custom icon')
            calls = []
            def run(command, **kwargs):
                calls.append(command)
                if '--sign' in command:
                    self.assertTrue(plistlib.loads(manifest.read_bytes())['NSHighResolutionCapable'])
                    self.assertEqual((resources / 'app_icon_source.webp').read_bytes(), b'custom icon')
                return subprocess.CompletedProcess(command, 0, stdout='usage: Workbench --project PROJECT')
            with patch.object(packaging.subprocess, 'run', side_effect=run):
                packaging.finalize_bundle(app, icon)
            self.assertIn('--sign', calls[0])
            self.assertIn('--verify', calls[1])
            self.assertEqual(calls[2][-1], '--help')
            self.assertEqual(calls[3][-1], '--check-terminal-bundle')
            self.assertEqual(plistlib.loads(manifest.read_bytes())['LSMinimumSystemVersion'], '13.0')

    def test_invalid_signature_blocks_startup_verification(self):
        with patch.object(packaging.subprocess, 'run',
                          side_effect=subprocess.CalledProcessError(1, 'codesign')) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                packaging.verify_bundle(Path('/Applications/Workbench.app'))
            self.assertEqual(run.call_count, 1)


if __name__ == "__main__":
    unittest.main()
