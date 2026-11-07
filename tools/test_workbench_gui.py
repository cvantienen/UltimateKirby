"""GUI safety checks with fake builds/uploads and temporary local files only."""
import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import workbench
import workbench_gui as gui
from workbench_history import History


class GuiTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.app = workbench.Workbench(History(self.root / 'state'))
        self.gui = gui.Controller(self.app)

    def test_manifest_version_changes_only_requested_literal_and_is_atomic(self):
        manifest = self.root / 'Cargo.toml'
        original = b'[package]\nname="test"\nversion = "2.2.4" # retain comment\ndescription="version 2.2.4 remains here"\n[lib]\ncrate-type=["rlib"]\n'
        manifest.write_bytes(original)
        gui.set_version(manifest, '2.2.5')
        self.assertEqual(manifest.read_bytes(), original.replace(b'version = "2.2.4"', b'version = "2.2.5"'))
        saved = manifest.read_bytes()
        with patch.object(gui.os, 'replace', side_effect=OSError('no rename')):
            with self.assertRaises(OSError):
                gui.set_version(manifest, '2.2.6')
        self.assertEqual(manifest.read_bytes(), saved)
        self.assertEqual(list(self.root.glob('.workbench-version-*')), [])
        for version in ['../2.2.7', '2.2', '$(touch x)', '2.2.5\n']:
            with self.assertRaises(ValueError):
                gui.set_version(manifest, version)
        self.assertEqual(manifest.read_bytes(), saved)

    def test_build_failure_prevents_upload_and_does_not_fall_back_to_old_release(self):
        with patch.object(gui, 'set_version', return_value={}), patch.object(self.app, 'build_release', side_effect=RuntimeError('build failed')), patch.object(self.app, 'upload_release') as upload:
            with self.assertRaisesRegex(RuntimeError, 'build failed'):
                self.gui.perform({'action': 'ship', 'version': '2.2.5', 'target': 'all', 'ready': True})
            upload.assert_not_called()

    def test_one_click_ship_builds_requested_target_then_previews_fixed_routes(self):
        events = []
        with patch.object(gui, 'set_version', side_effect=lambda *a: events.append('version')), patch.object(gui, 'version', return_value='2.2.5'), patch.object(self.app, 'build_release', side_effect=lambda *a: events.append(('build', a))), patch.object(self.app, 'run', side_effect=lambda *a: events.append(('preview', a))), patch.object(self.app, 'upload_release', return_value=True) as upload:
            self.gui.perform({'action': 'ship', 'version': '2.2.5', 'target': 'all', 'ready': True, 'cargo_home': '/tmp/cache'})
        self.assertEqual(events[0], 'version')
        self.assertEqual(events[1], ('build', ('all', '/tmp/cache')))
        previews = [e[1][1] for e in events if isinstance(e, tuple) and e[0] == 'preview']
        self.assertEqual(len(previews), 2)
        self.assertTrue(all('--dry-run' in p for p in previews))
        upload.assert_called_once_with('2.2.5', list(workbench.TARGETS), verify=False)

    def test_upload_requires_ready_and_source_version_change_blocks_ship(self):
        with patch.object(self.app, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'Close Smash'):
                self.gui.perform({'action': 'upload', 'version': '2.2.5'})
            run.assert_not_called()
        with patch.object(gui, 'set_version', return_value={}), patch.object(gui, 'version', return_value='9.9.9'), patch.object(self.app, 'build_release'), patch.object(self.app, 'upload_release') as upload:
            with self.assertRaisesRegex(RuntimeError, 'changed during build'):
                self.gui.perform({'action': 'ship', 'version': '2.2.5', 'ready': True})
            upload.assert_not_called()

    def test_sequence_replace_requires_matching_preview_confirmation_and_closed_game(self):
        self.gui.plan = SimpleNamespace(source=SimpleNamespace(kind='local'), destination=SimpleNamespace(kind='silent'), details=lambda: {})
        self.gui.plan_id = 'preview-id'
        with patch.object(gui.transfer, 'commit', return_value={'verified': True}) as commit:
            for values in [{}, {'plan_id': 'wrong', 'confirmed': True, 'ready': True}, {'plan_id': 'preview-id', 'confirmed': False, 'ready': True}, {'plan_id': 'preview-id', 'confirmed': True, 'ready': False}]:
                with self.assertRaises(ValueError):
                    self.gui.perform({'action': 'commit', **values})
            commit.assert_not_called()
            self.gui.perform({'action': 'commit', 'plan_id': 'preview-id', 'confirmed': True, 'ready': True})
            commit.assert_called_once()
        self.assertIsNone(self.gui.plan_id)

    def test_helpers_preserve_original_whitelist_and_sequence_confirmation(self):
        with patch.object(self.app, 'run') as run:
            with self.assertRaises(ValueError):
                self.gui.perform({'action': 'helper', 'helper': '../not-allowed.py'})
            with self.assertRaisesRegex(ValueError, 'Confirm sequence'):
                self.gui.perform({'action': 'helper', 'helper': 'upload_sequences.py', 'arguments': '--replace --smash-closed', 'ready': True})
            run.assert_not_called()
            self.gui.perform({'action': 'helper', 'helper': 'upload_sequences.py', 'arguments': '--help'})
            self.assertIn('--help', run.call_args.args[1])

    def test_new_job_cannot_overlap_and_failure_releases_busy_state(self):
        self.gui.busy = True
        with self.assertRaisesRegex(RuntimeError, 'already running'):
            self.gui.start({'action': 'build'})
        self.gui.busy = False
        with patch.object(self.gui, 'perform', side_effect=RuntimeError('fake failure')):
            self.gui.start({'action': 'build'})
            self.gui.thread.join(3)
        self.assertFalse(self.gui.snapshot()['busy'])
        self.assertIn('fake failure', self.gui.snapshot()['status'])

    def test_last_shipped_is_retained_beyond_recent_history_window(self):
        record = {'id': 'old', 'action': 'upload', 'status': 'success', 'details': {'version': '2.2.5', 'target': 'silent'}, 'receipt': {'verified_deployment': True}}
        self.gui.rows = [record] + [{'id': str(i), 'action': 'validate', 'status': 'success'} for i in range(35)]
        snapshot = self.gui.snapshot()
        self.assertEqual(len(snapshot['history']), 30)
        self.assertEqual(snapshot['shipped']['silent']['details']['version'], '2.2.5')


if __name__ == '__main__':
    with contextlib.redirect_stdout(io.StringIO()):
        unittest.main()
