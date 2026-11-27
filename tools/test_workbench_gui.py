"""GUI safety checks with fake builds/uploads and temporary local files only."""
import contextlib
import io
import json
import http.client
import queue
import re
import threading
from urllib.parse import urlsplit
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import workbench
import sequence_sources as sources
import workbench_gui as gui
from workbench_history import History


class GuiTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.app = workbench.Workbench(History(self.root / 'state'))
        self.gui = gui.Controller(self.app)

    def test_loopback_server_auth_and_dedicated_lan_route(self):
        urls = queue.Queue()
        servers = []
        server_type = gui.ThreadingHTTPServer
        def create_server(*args):
            server = server_type(*args)
            servers.append(server)
            return server
        errors = []
        def serve():
            try:
                gui.launch(self.app)
            except BaseException as error:
                errors.append(error)
        with patch.object(gui.webbrowser, 'open', side_effect=urls.put), \
             patch.object(gui, 'TelemetryViewer') as viewer_type, \
             patch.object(gui, 'ThreadingHTTPServer', side_effect=create_server), \
             patch.object(self.app.lan, 'perform', return_value={'next': 0, 'dropped': 0, 'entries': []}) as perform, \
             contextlib.redirect_stdout(io.StringIO()):
            viewer = viewer_type.return_value
            viewer.snapshot.return_value = {'ready': False, 'running': False, 'status': 'test'}
            viewer.start.return_value = viewer.snapshot.return_value
            viewer.stop.return_value = viewer.snapshot.return_value
            thread = threading.Thread(target=serve)
            thread.start()
            address = urlsplit(urls.get(timeout=3))
            def request(method, path, data=None, token=None, host=None):
                with contextlib.closing(http.client.HTTPConnection(address.hostname, address.port, timeout=3)) as connection:
                    headers = {'Host': host or address.netloc}
                    if token:
                        headers['X-Workbench-Token'] = token
                    body = None if data is None else json.dumps(data)
                    connection.request(method, path, body, headers)
                    response = connection.getresponse()
                    return response.status, response.read().decode()
            try:
                code, html = request('GET', '/')
                self.assertEqual(code, 200)
                token = re.search(r'const TOKEN=("[0-9a-f]+")', html)[1]
                token = json.loads(token)
                self.assertEqual(request('GET', '/api/state')[0], 403)
                self.assertEqual(request('GET', '/api/state', token='é')[0], 403)
                self.assertEqual(request('GET', '/api/state', token=token, host='evil.test')[0], 403)
                self.assertEqual(request('POST', '/api/lan', {'operation': 'logs'}, token)[0], 200)
                perform.assert_called_once_with('logs')
                perform.reset_mock()
                self.assertEqual(request('POST', '/api/lan', {'operation': 'connect',
                                        'host': '10.0.0.20', 'port': 4242, 'transport': 'udp'}, token)[0], 200)
                perform.assert_called_once_with('connect', host='10.0.0.20', port=4242, transport='udp')
                self.assertIn('<option value="udp" selected>', html)
                self.assertNotIn('lanToken', html)
                self.assertFalse(self.app.history.path.exists())
                self.assertEqual(request('POST', '/api/lan', [], token)[0], 400)
                viewer.start.assert_called_once_with()
                viewer.request.return_value = (200, 'text/html', b"const API_BASE = '', API_HEADERS = {};")
                self.assertEqual(request('GET', '/telemetry/?session=wrong')[0], 403)
                code, embedded = request('GET', '/telemetry/?session=' + token)
                self.assertEqual(code, 200)
                self.assertIn("'/api/telemetry/view'", embedded)
                self.assertIn(token, embedded)
                viewer.request.return_value = (200, 'application/json', b'{"objects":[]}')
                self.assertEqual(request('GET', '/api/telemetry/view/state')[0], 403)
                self.assertEqual(request('GET', '/api/telemetry/view/state', token=token)[0], 200)
                self.assertEqual(request('POST', '/api/telemetry/view/connect?endpoint=192.168.1.142:4243', token=token)[0], 200)
                viewer.request.assert_called_with('/connect?endpoint=192.168.1.142:4243', 'POST')
                self.assertEqual(request('POST', '/api/telemetry', {'operation': 'stop'})[0], 403)
                self.assertEqual(request('POST', '/api/telemetry', {'operation': 'stop'}, token)[0], 200)
                self.assertEqual(request('POST', '/api/telemetry', {'operation': 'start'}, token)[0], 200)
                viewer.start.assert_called_with()
                self.assertNotIn('telemetryConfig', html)
                self.assertIn('No config file or manual key is needed', html)
                self.assertEqual(request('POST', '/api/telemetry', [], token)[0], 400)
                self.assertEqual(request('POST', '/api/close', {}, token)[0], 200)
            finally:
                servers[0].shutdown()
                thread.join(5)
            self.assertFalse(thread.is_alive())
            self.assertEqual(errors, [])

    @unittest.skipUnless(shutil.which('node'), 'Node required for browser JavaScript regression')
    def test_browser_disconnect_ignores_pending_load_and_poll_state(self):
        html = Path(gui.__file__).with_suffix('.html').read_text()
        script = html[html.index('let lanConnected='):html.index('showLanSetup(INITIAL.lan_editor);')]
        harness = '''
const elements = new Proxy({}, {get(target, id){return target[id] ||= {value:'',checked:false,disabled:false,textContent:'',addEventListener(){}}}});
const $ = id => elements[id];
const document = {querySelectorAll(){return []}};
let busy=false;
let pending;
const api = (path,data) => data.operation==='disconnect' ? Promise.resolve({connected:false}) : new Promise(resolve=>{pending=resolve});
const error = () => {};
const refresh = async () => {};
'''
        cases = '''
(async()=>{
lanConnected=true;
const load=lanLoad();
await lanDisconnect();
pending({text:'old session',revision:'abc'});
await load;
if(lanLoaded || $('lanText').value==='old session')throw Error('Late load restored disconnected session');
lanConnected=true;
const connect=lanConnect();
await lanDisconnect();
pending({connected:true,editing:true});
await connect;
if(lanConnected)throw Error('Late connect resurrected session');
busy=true;lanConnected=true;lanControls();
if(!$('lanConnect').disabled || !$('lanText').disabled)throw Error('LAN controls ignore main job');
})().catch(error=>{console.error(error);process.exitCode=1});
'''
        result = subprocess.run(['node', '-e', harness + script + cases], capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_lan_setup_is_local_only_persists_without_connecting(self):
        with patch('socket.socket', side_effect=AssertionError('No network sockets')), \
             patch('socket.getaddrinfo', side_effect=AssertionError('No DNS')), \
             patch.object(self.app, 'host', side_effect=AssertionError('No FTP settings')), \
             patch.object(self.app, 'run', side_effect=AssertionError('No helper execution')):
            result = self.gui.perform({'action': 'lan-editor-configure',
                                       'lan_host': '192.168.1.20', 'lan_port': '4242'})
            self.assertEqual(result['host'], '192.168.1.20')
            self.assertTrue(result['available'])
            self.assertFalse(result['connected'])
            self.assertFalse(result['listening'])
            restarted = workbench.Workbench(self.app.history)
            self.assertEqual(restarted.lan_editor_state(), result)
            self.assertEqual(self.gui.snapshot()['lan_editor'], result)
            self.assertEqual(self.app.history.rows(), [])

    def test_invalid_lan_setup_cannot_change_saved_preferences(self):
        self.app.configure_lan_editor('10.0.0.20', 4242)
        saved = self.app.preferences_path.read_bytes()
        for host, port in [('localhost', 4242), ('https://192.168.1.20', 4242),
                           ('8.8.8.8', 4242), ('127.0.0.1', 4242), ('169.254.1.2', 4242),
                           ('::ffff:192.168.1.20', 4242), ('192.168.1.20', 0),
                           ('192.168.1.20', 65536), ('192.168.1.20', True),
                           ('192.168.1.20', 42.5), ('192.168.1.20', '1e3')]:
            with self.subTest(host=host, port=port), self.assertRaises(ValueError):
                self.gui.perform({'action': 'lan-editor-configure', 'lan_host': host, 'lan_port': port})
            self.assertEqual(self.app.preferences_path.read_bytes(), saved)
        before = dict(self.app.preferences)
        with patch.object(self.app, 'save_preferences', side_effect=OSError('disk full')):
            with self.assertRaises(OSError):
                self.app.configure_lan_editor('10.0.0.21', 4243)
        self.assertEqual(self.app.preferences, before)

    def test_lan_cannot_use_journal_worker_and_state_does_not_wait_for_network(self):
        with patch('socket.socket', side_effect=AssertionError('No network sockets')):
            for operation in ['start', 'connect', 'pair', 'apply', 'save']:
                data = {'action': f'lan-editor-{operation}'}
                with self.assertRaises(ValueError):
                    self.gui.start(data)
                with self.assertRaisesRegex(ValueError, 'dedicated'):
                    self.gui.perform(data)
            self.app.lan.lock.acquire()
            try:
                self.assertFalse(self.gui.snapshot()['lan_editor']['connected'])
            finally:
                self.app.lan.lock.release()
            self.assertFalse(self.gui.busy)
            self.assertIsNone(self.gui.thread)

    def test_terminal_lan_explicit_connect_disconnect_without_transfer(self):
        with patch.object(workbench, 'ask', side_effect=['172.16.1.20', '4242', 'udp']), \
             patch.object(workbench, 'choose', return_value=3), \
             patch.object(self.app.lan, 'perform', return_value={'connected': True}) as perform, \
             contextlib.redirect_stdout(io.StringIO()):
            self.app.lan.connected = True
            self.app.lan_editor()
        self.assertEqual(self.app.lan_editor_state()['host'], '172.16.1.20')
        perform.assert_called_once_with('connect', host='172.16.1.20', port='4242', transport='udp')
        self.assertFalse(self.app.lan.connected)
        self.assertNotIn('token', self.app.preferences_path.read_text())

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
        with patch.object(gui, 'set_version', return_value={}), patch.object(self.app, 'prepare_silent_sequences', return_value={'original': sources.assemble(sources.SOURCE), 'data': sources.assemble(sources.SOURCE)}), patch.object(self.app, 'build_release', side_effect=RuntimeError('build failed')), patch.object(self.app, 'upload_release') as upload:
            with self.assertRaisesRegex(RuntimeError, 'build failed'):
                self.gui.perform({'action': 'ship', 'version': '2.2.5', 'target': 'all', 'ready': True})
            upload.assert_not_called()

    def test_one_click_ship_builds_requested_target_then_previews_fixed_routes(self):
        events = []
        plan = {'original': sources.assemble(sources.SOURCE), 'data': sources.assemble(sources.SOURCE), 'origin': 'silent'}
        with patch.object(gui, 'set_version', side_effect=lambda *a: events.append('version')), patch.object(gui, 'version', return_value='2.2.5'), patch.object(self.app, 'prepare_silent_sequences', side_effect=lambda: (events.append('sync') or dict(plan))),  patch.object(self.app.history, 'perform', side_effect=lambda action, details, operation: operation()), patch.object(self.app, 'build_release', side_effect=lambda *a: events.append(('build', a))), patch.object(self.app, 'run', side_effect=lambda *a: events.append(('preview', a))), patch.object(self.app, 'upload_release', return_value=True) as upload, patch.object(self.app, 'apply_silent_sequences', side_effect=lambda *a: events.append('apply')):
            self.gui.perform({'action': 'ship', 'version': '2.2.5', 'target': 'all', 'ready': True, 'sequences_confirmed': True, 'silent_host': '192.168.1.106', 'diagnostic_host': '192.168.1.142', 'cargo_home': '/tmp/cache'})
        self.assertIn('version', events)
        self.assertIn(('build', ('all', '/tmp/cache')), events)
        self.assertIn('apply', events)
        previews = [e[1][1] for e in events if isinstance(e, tuple) and e[0] == 'preview']
        self.assertEqual(len(previews), 2)
        self.assertTrue(all('--dry-run' in p for p in previews))
        self.assertTrue(all('--replace-sequences' in p for p in previews))
        upload.assert_called_once_with('2.2.5', list(workbench.TARGETS), verify=False, sequences_synced=True, sequence_plan=plan)

    def test_diagnostic_only_skips_silent_sync_and_preserves_device_sequences(self):
        for action in ('ship', 'upload', 'verify'):
            with self.subTest(action=action), \
                 patch.object(gui, 'set_version', return_value={}), \
                 patch.object(gui, 'version', return_value='2.2.5'), \
                 patch.object(self.app, 'prepare_silent_sequences') as prepare, \
                 patch.object(self.app, 'apply_silent_sequences') as apply, \
                 patch.object(self.app, 'build_release') as build, \
                 patch.object(self.app, 'run') as preview, \
                 patch.object(self.app, 'upload_release', return_value=True) as upload:
                self.gui.perform({'action': action, 'version': '2.2.5', 'target': 'diagnostic', 'ready': True})
                prepare.assert_not_called()
                apply.assert_not_called()
                self.assertEqual(build.call_count, int(action == 'ship'))
                argv = preview.call_args.args[1]
                self.assertIn('--preserve-sequences', argv)
                self.assertNotIn('--replace-sequences', argv)
                self.assertEqual([name for _, name in upload.call_args.args[1]], ['diagnostic'])
                self.assertIsNone(upload.call_args.kwargs['sequence_plan'])

    def test_ship_synchronizes_silent_before_build_and_uses_configured_targets(self):
        events = []
        plan = {'original': sources.assemble(sources.SOURCE), 'data': sources.assemble(sources.SOURCE), 'origin': 'silent'}
        with (patch.object(gui, 'set_version', return_value={}),
              patch.object(gui, 'version', return_value='2.2.5'),
              patch.object(self.app, 'prepare_silent_sequences', side_effect=lambda: (events.append('sync') or dict(plan))),
              patch.object(self.app.history, 'perform', side_effect=lambda action, details, operation: events.append(action) or operation()),
              patch.object(self.app, 'build_release', side_effect=lambda *args: events.append('build')),
              patch.object(self.app, 'run', side_effect=lambda *args: events.append('preview')),
              patch.object(self.app, 'upload_release', return_value=True) as upload,
              patch.object(self.app, 'apply_silent_sequences', side_effect=lambda *args: events.append('apply'))):
            self.gui.perform({'action': 'ship', 'version': '2.2.5', 'target': 'all', 'ready': True,
                              'silent_host': '10.0.0.10', 'diagnostic_host': '10.0.0.11'})
        self.assertIn('sync', events)
        self.assertIn('build', events)
        self.assertEqual(upload.call_args.args[1], [('10.0.0.10', 'silent'), ('10.0.0.11', 'diagnostic')])
        self.assertTrue(upload.call_args.kwargs['verify'] is False)
        self.assertTrue(upload.call_args.kwargs['sequences_synced'])
        self.assertIn('apply', events)

    def test_configured_switch_ips_must_be_distinct_ipv4_addresses(self):
        for values in ({'silent_host': 'bad-ip'}, {'silent_host': '10.0.0.2', 'diagnostic_host': '10.0.0.2'}):
            with self.subTest(values=values), self.assertRaises(ValueError):
                self.gui.perform({'action': 'validate', **values})

    def test_upload_requires_ready_and_source_version_change_blocks_ship(self):
        with patch.object(self.app, 'run') as run:
            with self.assertRaisesRegex(ValueError, 'Close Smash'):
                self.gui.perform({'action': 'upload', 'version': '2.2.5'})
            run.assert_not_called()
        with patch.object(gui, 'set_version', return_value={}), patch.object(gui, 'version', return_value='9.9.9'), patch.object(self.app, 'prepare_silent_sequences', return_value={'original': sources.assemble(sources.SOURCE), 'data': sources.assemble(sources.SOURCE), 'origin': 'silent'}), patch.object(self.app, 'run'), patch.object(self.app, 'build_release'), patch.object(self.app, 'upload_release') as upload, patch.object(self.app, 'apply_silent_sequences'): 
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
