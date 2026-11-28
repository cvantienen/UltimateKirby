"""Managed receiver lifecycle tests; no device or existing pairing-file reads."""
import http.client
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from workbench_telemetry import TelemetryViewer


class TelemetryViewerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.config = self.root / 'pairing.txt'
        self.viewer = TelemetryViewer(self.root)

    def test_missing_or_stale_pairing_is_never_inspected(self):
        process = Mock(pid=4321)
        process.poll.return_value = None
        for existing in [False, True]:
            if existing:
                self.config.write_text('obsolete pairing fixture')
            viewer = TelemetryViewer(self.root)
            with patch('workbench_telemetry.subprocess.Popen', return_value=process) as launch, \
                 patch.object(viewer, '_probe', return_value=False), \
                 patch.object(Path, 'is_file', side_effect=AssertionError('No pairing discovery')), \
                 patch.object(Path, 'read_text', side_effect=AssertionError('No pairing read')):
                state = viewer.start()
            self.assertTrue(state['running'])
            self.assertNotIn(str(self.config), launch.call_args.args[0])
            self.assertNotIn('config', state)

    def test_managed_start_readiness_and_stop(self):
        self.config.write_text('test fixture, never read by manager')
        process = Mock(pid=4321)
        process.poll.return_value = None
        with patch('workbench_telemetry.subprocess.Popen', return_value=process) as launch, \
             patch.object(self.viewer, '_probe', return_value=False):
            state = self.viewer.start()
        self.assertTrue(state['running'])
        self.assertFalse(state['ready'])
        argv = launch.call_args.args[0]
        self.assertIn('--offline', argv)
        self.assertEqual(argv[-2], '--')
        self.assertTrue(argv[-1].isdecimal())
        self.assertTrue(launch.call_args.kwargs['start_new_session'])
        with self.assertRaisesRegex(RuntimeError, 'Stop the current'):
            self.viewer.start()
        with patch.object(self.viewer, '_probe', return_value=True):
            self.assertTrue(self.viewer.snapshot()['ready'])
        with patch('workbench_telemetry.os.killpg') as kill:
            state = self.viewer.stop()
        self.assertEqual(kill.call_args.args[0], process.pid)
        process.wait.assert_called_once_with(timeout=3)
        self.assertFalse(state['running'])
        self.assertIsNone(state['url'])

    @unittest.skipUnless(shutil.which('node'), 'Node required for setup JavaScript regression')
    def test_dashboard_setup_posts_empty_body_and_resets_frozen_data(self):
        page = Path(__file__).resolve().parents[3] / 'crates/kirby_assist_core/examples/telemetry_viewer.html'
        html = page.read_text()
        script = html[html.index("$('connect').onsubmit"):html.index('setInterval(render, 100); poll();')]
        harness = '''
const elements = new Proxy({}, {get(target,id){return target[id] ||= {value:'',disabled:false,textContent:'',setAttribute(){}}}});
const $ = id => elements[id];
$('switch-host').value=' 192.168.1.142 ';$('switch-port').value='4243';
const API_BASE='',API_HEADERS={};
let setupGeneration=0,frozen=true,frozenState={},state={},session='old',connected=true;
let history=[1],rateHistory=[1],historySequence=1,lastRate=2,selectedField=3;
const previous=new Map([['old',1]]), highlights=new Map([['old',1]]);
const syncObjects=()=>{},render=()=>{};
let request;
const fetch=async(path,options)=>{request={path,options};return {ok:true}};
'''
        checks = '''
(async()=>{
await $('connect').onsubmit({preventDefault(){}});
if(request.path!=='/connect?endpoint=192.168.1.142:4243')throw Error('Endpoint missing');
if(request.options.method!=='POST'||request.options.body!==''||request.options.headers!==API_HEADERS)throw Error('Invalid request');
if(frozen||state||history.length||rateHistory.length||previous.size||highlights.size||setupGeneration!==1)throw Error('Old observations retained');
if($('connect-submit').disabled)throw Error('Setup remains locked');
fetch=async()=>({ok:false});
await $('connect').onsubmit({preventDefault(){}});
if(!$('connect-status').textContent.includes('failed'))throw Error('Failure hidden');
})().catch(error=>{console.error(error);process.exitCode=1});
'''
        # Keep fetch mutable so the failure branch is exercised after success.
        harness = harness.replace('const fetch=', 'let fetch=')
        result = subprocess.run(['node', '-e', harness + script + checks], capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('not authenticated or encrypted', html)
        self.assertIn('id="connect"', html)

    @unittest.skipUnless(shutil.which('node'), 'Node required for raw dashboard regression')
    def test_raw_dashboard_reports_loss_and_recovers_after_capture_shrinks(self):
        page = Path(__file__).resolve().parents[2] / 'workbench_ui/dashboard.js'
        script = page.read_text()
        harness = '''
const elements = new Proxy({}, {get(target,id){return target[id] ||= {value:'',checked:false,textContent:'',classList:{contains:()=>true}}}});
const $ = id => elements[id];
let incoming = {receiver:{status:'ready'},logs:{connected:true,dropped:12,epoch:3,error:''},
  telemetry:{bytes:20,text:'old capture contents',truncated:false},
  diagnostic:{bytes:0,text:'',truncated:false}};
const api = async()=>incoming;
'''
        checks = '''
(async()=>{
await pollRaw();
clearRaw('telemetry');
incoming.telemetry={bytes:3,text:'new',truncated:false};
await pollRaw();
if($('raw-telemetry').textContent!=='new')throw Error('New capture hidden by old clear cursor');
if(!$('rawStatus').textContent.includes('Native ring dropped 12')||!$('rawStatus').textContent.includes('Log session 3'))throw Error('Loss/session hidden');
})().catch(error=>{console.error(error);process.exitCode=1});
'''
        result = subprocess.run(['node', '-e', harness + script + checks], capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_proxy_is_bounded_to_owned_receiver_routes(self):
        process = Mock()
        process.poll.return_value = None
        self.viewer.process, self.viewer.ready, self.viewer.port = process, True, 8765
        connection = Mock()
        response = connection.getresponse.return_value
        response.status = 200
        response.getheader.return_value = 'application/json'
        response.read.return_value = b'{}'
        with patch('workbench_telemetry.http.client.HTTPConnection', return_value=connection) as create:
            self.assertEqual(self.viewer.request('/state'), (200, 'application/json', b'{}'))
        create.assert_called_once_with('127.0.0.1', 8765, timeout=1)
        connection.request.assert_called_once_with('GET', '/state', headers={
            'Origin': 'http://127.0.0.1:8765', 'Content-Length': '0'})
        connection.close.assert_called_once()
        with self.assertRaises(ValueError):
            self.viewer.request('/connect?endpoint=192.168.1.1:4243')
        with self.assertRaises(ValueError):
            self.viewer.request('/arbitrary-file')
        connection.getresponse.side_effect = http.client.RemoteDisconnected('receiver closed')
        with patch('workbench_telemetry.http.client.HTTPConnection', return_value=connection):
            with self.assertRaisesRegex(RuntimeError, 'HTTP response unavailable'):
                self.viewer.request('/state')
        self.viewer.ready = False
        with self.assertRaises(RuntimeError):
            self.viewer.request('/state')

    def test_failed_start_and_exited_child_are_not_reported_ready(self):
        self.config.touch()
        with patch('workbench_telemetry.subprocess.Popen', side_effect=FileNotFoundError):
            self.assertFalse(self.viewer.start()['running'])
        process = Mock(returncode=1)
        process.poll.return_value = 1
        self.viewer.process = process
        self.viewer.ready = True
        state = self.viewer.snapshot()
        self.assertFalse(state['ready'])
        self.assertIn('code 1', state['status'])


if __name__ == '__main__':
    unittest.main()
