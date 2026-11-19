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

    def test_sequence_source_import_needs_confirmation_but_check_does_not(self):
        with patch.object(self.app, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'Confirm sequence'):
                self.gui.perform({'action': 'helper', 'helper': 'sequence_sources.py',
                                  'arguments': '--import-file /tmp/downloaded.txt'})
            run.assert_not_called()
            self.gui.perform({'action': 'helper', 'helper': 'sequence_sources.py', 'arguments': '--check'})
            run.assert_called_once()


EDITED = b"[universal.settings]\nx=1\n[kirby.neutral.a]\nstart_delay=0\nactions=a:4\n[yoshi.roll.p]\nstart_delay=0\nactions=b:4\n"


class BreakdownTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'kirby_assist' / 'sequences.txt'
        (self.source.parent / 'fighters').mkdir(parents=True)
        self.source.write_bytes(b'[universal.settings]\nx=1\n')
        (self.source.parent / 'fighters' / 'yoshi.txt').write_bytes(b'[yoshi.roll.p]\nstart_delay=0\nactions=b:4\n')
        (self.source.parent / 'fighters' / 'retired.txt').write_bytes(b'[retired.neutral.a]\nactions=wait:0\n')
        self.app = workbench.Workbench(History(self.root / 'state'))
        self.gui = gui.Controller(self.app)
        validator = patch.object(gui.transfer, 'validate_sequences')
        validator.start()
        self.addCleanup(validator.stop)

    def preview(self, combined):
        with patch.object(gui.transfer, 'read', return_value=combined), contextlib.redirect_stdout(io.StringIO()):
            return self.gui.perform({'action': 'breakdown-preview', 'breakdown_endpoint': 'silent',
                                     'breakdown_file': '', 'breakdown_destination': str(self.source)})

    def test_preview_classifies_new_replaced_and_removed_files_without_writing(self):
        plan = self.preview(EDITED)
        self.assertEqual([e['fighter'] for e in plan['created']], ['kirby'])
        self.assertEqual([e['fighter'] for e in plan['removed']], ['retired'])
        # Identical fighter bodies are reported as unchanged rather than rewritten.
        self.assertEqual([e['fighter'] for e in plan['unchanged']], ['yoshi'])
        self.assertFalse((self.source.parent / 'fighters' / 'kirby.txt').exists())
        self.assertTrue((self.source.parent / 'fighters' / 'retired.txt').exists())

    def test_commit_writes_sources_only_after_a_matching_confirmed_plan(self):
        plan = self.preview(EDITED)
        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(ValueError, 'Preview the breakdown'):
                self.gui.perform({'action': 'breakdown-commit', 'plan_id': 'stale', 'confirmed': True})
            with self.assertRaisesRegex(ValueError, 'Confirm replacing'):
                self.gui.perform({'action': 'breakdown-commit', 'plan_id': plan['plan_id']})
            self.assertTrue((self.source.parent / 'fighters' / 'retired.txt').exists())
            result = self.gui.perform({'action': 'breakdown-commit', 'plan_id': plan['plan_id'], 'confirmed': True})
        self.assertEqual(result['created'], ['kirby'])
        self.assertEqual(result['removed'], ['retired'])
        self.assertFalse((self.source.parent / 'fighters' / 'retired.txt').exists())
        self.assertIn(b'actions=a:4', (self.source.parent / 'fighters' / 'kirby.txt').read_bytes())
        self.assertEqual(gui.sources.assemble(self.source), gui.sources.canonical_bundle(EDITED))
        self.assertTrue(Path(result['backup']).is_dir())

    def test_breakdown_requires_a_split_source_destination(self):
        plain = self.root / 'loose.txt'
        plain.write_bytes(EDITED)
        with patch.object(gui.transfer, 'read', return_value=EDITED), contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaisesRegex(RuntimeError, 'fighters/ directory|deployment/ultimate'):
                self.gui.perform({'action': 'breakdown-preview', 'breakdown_endpoint': 'silent',
                                  'breakdown_file': '', 'breakdown_destination': str(plain)})


class FighterEditorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.file = self.root / 'sequences.txt'
        self.file.write_bytes(EDITED)
        self.app = workbench.Workbench(History(self.root / 'state'))
        self.gui = gui.Controller(self.app)
        for module in (gui.fighter_editor, gui.transfer):
            validator = patch.object(module, 'validate_sequences')
            validator.start()
            self.addCleanup(validator.stop)

    def load(self, endpoint='local'):
        with contextlib.redirect_stdout(io.StringIO()):
            return self.gui.perform({'action': 'editor-load', 'editor_endpoint': endpoint,
                                     'editor_file': str(self.file)})

    def edit(self, text, fighter='kirby'):
        with contextlib.redirect_stdout(io.StringIO()):
            self.gui.perform({'action': 'editor-fighter', 'fighter': fighter})
            return self.gui.perform({'action': 'editor-preview', 'fighter': fighter, 'text': text})

    def test_local_edit_saves_one_fighter_without_the_switch_ready_flag(self):
        metadata = self.load()
        self.assertEqual(metadata['fighters'], ['kirby', 'yoshi'])
        self.assertFalse(metadata['requires_ready'])
        preview = self.edit('[kirby.neutral.a]\nstart_delay=0\nactions=a:12\n')
        self.assertEqual(preview['changes']['changed'], ['kirby.neutral.a'])
        with contextlib.redirect_stdout(io.StringIO()):
            result = self.gui.perform({'action': 'editor-commit', 'plan_id': preview['plan_id'], 'confirmed': True})
        self.assertTrue(result['verified'])
        self.assertEqual(self.file.read_bytes(), EDITED.replace(b'a:4', b'a:12'))

    def test_switch_destination_still_requires_closed_game_and_confirmation(self):
        with patch.object(gui.transfer, 'read', return_value=EDITED):
            self.assertTrue(self.load('silent')['requires_ready'])
            preview = self.edit('[kirby.neutral.a]\nstart_delay=0\nactions=a:12\n')
            self.assertTrue(preview['requires_ready'])
            with contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaisesRegex(ValueError, 'Close Smash'):
                    self.gui.perform({'action': 'editor-commit', 'plan_id': preview['plan_id'], 'confirmed': True})
                with self.assertRaisesRegex(ValueError, 'Confirm the fighter'):
                    self.gui.perform({'action': 'editor-commit', 'plan_id': preview['plan_id'], 'ready': True})

    def test_edits_cannot_touch_another_fighter_or_orphan_fields(self):
        self.load()
        for text in ('[yoshi.roll.p]\nstart_delay=0\nactions=b:9\n', 'actions=a:1\n'):
            with self.assertRaises(ValueError):
                self.edit(text)
        self.assertEqual(self.file.read_bytes(), EDITED)

    def test_commit_rejects_a_stale_plan_and_requires_a_loaded_file(self):
        with self.assertRaisesRegex(ValueError, 'Load a sequence file'):
            self.gui.perform({'action': 'editor-fighter', 'fighter': 'kirby'})
        self.load()
        preview = self.edit('[kirby.neutral.a]\nstart_delay=0\nactions=a:12\n')
        with contextlib.redirect_stdout(io.StringIO()):
            self.gui.perform({'action': 'editor-discard'})
            with self.assertRaisesRegex(ValueError, 'Preview the fighter edit'):
                self.gui.perform({'action': 'editor-commit', 'plan_id': preview['plan_id'], 'confirmed': True})
        self.assertEqual(self.file.read_bytes(), EDITED)

    def test_emptying_a_fighter_deletes_only_their_sections(self):
        self.load()
        preview = self.edit('')
        self.assertTrue(preview['delete_fighter'])
        with contextlib.redirect_stdout(io.StringIO()):
            self.gui.perform({'action': 'editor-commit', 'plan_id': preview['plan_id'], 'confirmed': True})
        data = self.file.read_bytes()
        self.assertNotIn(b'kirby', data)
        self.assertIn(b'[yoshi.roll.p]\nstart_delay=0\nactions=b:4', data)
        self.assertIn(b'[universal.settings]', data)

    def test_split_repository_sources_are_edited_per_fighter_file(self):
        source = self.root / 'kirby_assist'
        (source / 'fighters').mkdir(parents=True)
        universal = source / 'sequences.txt'
        universal.write_bytes(b'[universal.settings]\nx=1\n')
        (source / 'fighters' / 'kirby.txt').write_bytes(b'[kirby.neutral.a]\nstart_delay=0\nactions=a:4\n')
        (source / 'fighters' / 'yoshi.txt').write_bytes(b'[yoshi.roll.p]\nstart_delay=0\nactions=b:4\n')
        self.file = universal
        self.load()
        preview = self.edit('[kirby.neutral.a]\nstart_delay=0\nactions=a:12\n')
        with contextlib.redirect_stdout(io.StringIO()):
            self.gui.perform({'action': 'editor-commit', 'plan_id': preview['plan_id'], 'confirmed': True})
        self.assertIn(b'actions=a:12', (source / 'fighters' / 'kirby.txt').read_bytes())
        self.assertEqual((source / 'fighters' / 'yoshi.txt').read_bytes(),
                         b'[yoshi.roll.p]\nstart_delay=0\nactions=b:4\n')


if __name__ == '__main__':
    with contextlib.redirect_stdout(io.StringIO()):
        unittest.main()
