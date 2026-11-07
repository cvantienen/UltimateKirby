"""Loopback-only browser UI for workbench.py; standard-library host tooling.

No build/FTP on startup. One background operation at a time; original helpers
supply build validation, safe routing, backups, verification and journal logs.
Active sequence replacement requires a prepared plan and explicit confirmation.
"""
from __future__ import annotations

import contextlib
import io
import json
import os
from pathlib import Path
import re
import secrets
import shlex
import stat
import tempfile
import threading
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit, parse_qs

import sequence_transfer as transfer
from toml_compat import tomllib
import upload_switch
from workbench import DEFAULT_FILE, HELPERS, ROOT, TARGETS, helper, version


def set_version(manifest: Path, release: str):
    """Change only the literal package version, atomically; preserve all other TOML."""
    if not re.fullmatch(r'\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?', release):
        raise ValueError("Build version must look like 2.2.5")
    upload_switch.release_version(release)
    original = manifest.read_bytes()
    text = original.decode()
    before = tomllib.loads(text)
    header = re.search(r'^\[package\][ \t]*\r?\n', text, re.M)
    match = re.search(r'^(version\s*=\s*")([^"]+)(")', text[header.end():], re.M) if header else None
    if not match or match[2] != before['package']['version']:
        raise RuntimeError("Cannot identify the package version safely")
    start = header.end() + match.start(2)
    updated = text[:start] + release + text[start + len(match[2]):]
    expected = tomllib.loads(text)
    expected['package']['version'] = release
    if tomllib.loads(updated) != expected:
        raise RuntimeError("Version edit would change other manifest settings")
    if updated == text:
        return {"version": release, "unchanged": True}
    descriptor, name = tempfile.mkstemp(prefix='.workbench-version-', dir=manifest.parent)
    try:
        with os.fdopen(descriptor, 'wb') as out:
            out.write(updated.encode())
            out.flush()
            os.fsync(out.fileno())
        os.chmod(name, stat.S_IMODE(manifest.stat().st_mode))
        if manifest.read_bytes() != original:
            raise RuntimeError("Cargo.toml changed during version edit; retry")
        os.replace(name, manifest)
    finally:
        Path(name).unlink(missing_ok=True)
    return {"previous": before['package']['version'], "version": release}


class Output(io.TextIOBase):
    def __init__(self):
        self.lock = threading.Lock()
        self.text = ''

    def write(self, text):
        with self.lock:
            self.text = (self.text + text)[-160_000:]
        return len(text)

    def flush(self):
        pass

    def value(self):
        with self.lock:
            return self.text


class Controller:
    def __init__(self, app):
        self.app = app
        self.app.noninteractive = True
        self.lock = threading.Lock()
        self.busy = False
        self.status = 'Ready'
        self.output = Output()
        self.thread = None
        self.plan = None
        self.plan_id = None
        self.preview = None
        self.rows = app.history.rows()
        self.result = None

    def snapshot(self):
        with self.lock:
            def compact(row):
                return {**{k: row.get(k) for k in ('id', 'action', 'status', 'time', 'log')},
                        'details': {k: v for k, v in row.get('details', {}).items()
                                    if k in {'version', 'target', 'source', 'destination', 'fighter'}},
                        'receipt': {'verified_deployment': row.get('receipt', {}).get('verified_deployment', False)}}
            shipped = {}
            for row in self.rows:
                if row.get('action') == 'upload' and row.get('status') == 'success' and row.get('receipt', {}).get('verified_deployment'):
                    shipped[row['details']['target']] = compact(row)
            return {"busy": self.busy, "status": self.status, "output": self.output.value(),
                    "result": self.result, "preview": self.preview, "plan_id": self.plan_id,
                    "history": [compact(r) for r in self.rows[-30:]], "shipped": shipped}

    def start(self, data):
        if not isinstance(data, dict) or data.get('action') not in {
            'ship', 'build', 'upload', 'verify', 'validate', 'preview', 'commit', 'helper'}:
            raise ValueError('Unknown operation')
        with self.lock:
            if self.busy:
                raise RuntimeError('An operation is already running')
            self.busy = True
            self.status = 'Running: ' + data['action']
            self.result = None
            self.output = Output()
            # Prevent a previous preview being mistaken for a newly failed one.
            if data['action'] == 'preview':
                self.plan = self.plan_id = self.preview = None
            self.thread = threading.Thread(target=self._worker, args=(dict(data),), daemon=False)
            self.thread.start()

    def _worker(self, data):
        try:
            with contextlib.redirect_stdout(self.output), contextlib.redirect_stderr(self.output):
                result = self.perform(data)
            status = 'Complete'
        except Exception as error:
            self.output.write(f'\nStopped: {error}\n')
            result, status = None, 'Stopped: ' + str(error)
        finally:
            try:
                rows = self.app.history.rows()
            except (OSError, ValueError) as error:
                rows = self.rows
                self.output.write(f'\nCould not refresh history: {error}\n')
            with self.lock:
                self.rows, self.result, self.status, self.busy = rows, result, status, False

    def endpoint(self, kind, path):
        if kind == 'local':
            if not path or not str(path).strip():
                raise ValueError('Choose a local sequence file')
            return transfer.Endpoint('local', str(path))
        if kind not in {'silent', 'diagnostic', 'silent-staged', 'diagnostic-staged'}:
            raise ValueError('Unknown sequence endpoint')
        name = kind.split('-')[0]
        return transfer.Endpoint(name, transfer.STAGED if kind.endswith('-staged') else transfer.ACTIVE, self.app.port(name))

    def perform(self, data):
        action = data['action']
        if action in {'ship', 'upload'} and data.get('ready') is not True:
            raise ValueError('Close Smash and open FTP, then check the ready box')
        for key in ('silent_port', 'diagnostic_port'):
            port = int(data.get(key, self.app.preferences.get(key, 5000)))
            if not 1 <= port <= 65535:
                raise ValueError('FTP port must be 1..65535')
            self.app.preferences[key] = port
        cache = str(data.get('cargo_home', self.app.preferences.get('cargo_home', '/private/tmp/kirby-assist-build-cargo'))).strip()
        self.app.preferences['cargo_home'] = cache
        self.app.save_preferences()
        if action in {'ship', 'build', 'upload', 'verify'}:
            release = upload_switch.release_version(str(data.get('version', '')).strip())
            target = data.get('target', 'all')
            if target not in {'all', 'silent', 'diagnostic'}:
                raise ValueError('Choose Both, Silent or Diagnostic')
            targets = [t for t in TARGETS if target == 'all' or t[1] == target]
            self.app.preferences['gui_target'] = target
            self.app.save_preferences()
            if action in {'ship', 'build'}:
                self.app.history.perform('set-version', {'version': release}, lambda: set_version(ROOT / 'Cargo.toml', release))
                self.app.build_release(target, cache)
                if version() != release:
                    raise RuntimeError('Source version changed during build; upload cancelled')
            if action in {'ship', 'upload', 'verify'}:
                # Validate every selected payload before writing to any Switch.
                for host, name in targets:
                    self.app.run('upload-preview', helper('upload_switch.py', host, '-v', release,
                        '-s' if name == 'silent' else '-d', '--port', self.app.port(name), '--dry-run'),
                        {'version': release, 'target': name, 'host': host})
                if not self.app.upload_release(release, targets, verify=action == 'verify'):
                    raise RuntimeError('Some Switches failed; inspect individual upload results')
            return {'version': release, 'targets': [name for _, name in targets]}
        if action == 'validate':
            path = Path(data.get('file', '')).expanduser().resolve()
            self.app.preferences['sequences'] = str(path)
            self.app.save_preferences()
            return self.app.run('validate', helper('sequence_validation.py', path), {'file': str(path)})
        if action == 'preview':
            source = self.endpoint(data.get('source'), data.get('source_file'))
            destination = self.endpoint(data.get('destination'), data.get('destination_file'))
            fighter = str(data.get('fighter', '')).strip() or None
            plans = []
            def prepare():
                plan = transfer.prepare(source, destination, fighter)
                plans.append(plan)
                return plan.details()
            self.app.history.perform('sequence-preview', {'source': source.label(), 'destination': destination.label(), 'fighter': fighter}, prepare)
            self.plan = plans[0]
            for key in ('source', 'destination', 'source_file', 'destination_file', 'fighter'):
                self.app.preferences['gui_' + key] = data.get(key, '')
            self.app.save_preferences()
            self.plan_id = secrets.token_hex(16)
            self.preview = {**self.plan.details(), 'description': self.plan.description,
                            'requires_ready': source.kind != 'local' or destination.kind != 'local',
                            'form': {k: data.get(k, '') for k in ('source', 'destination', 'source_file', 'destination_file', 'fighter')}}
            return self.preview
        if action == 'commit':
            if not self.plan or data.get('plan_id') != self.plan_id or data.get('confirmed') is not True:
                raise ValueError('Preview the transfer and confirm its replacement first')
            if (self.plan.source.kind != 'local' or self.plan.destination.kind != 'local') and data.get('ready') is not True:
                raise ValueError('Close Smash on involved Switches and open FTP first')
            result = self.app.history.perform('sequence-transfer', self.plan.details(),
                lambda: transfer.commit(self.plan, self.app.history.directory / 'sequence-backups'))
            self.plan = self.plan_id = self.preview = None
            return result
        if action == 'helper':
            name = data.get('helper')
            if name not in HELPERS:
                raise ValueError('Unknown original helper')
            args = shlex.split(str(data.get('arguments', '--help')))
            readonly = any(a in args for a in ('-h', '--help', '--dry-run', '--verify-only'))
            writes = not readonly and (name.startswith('upload') or name == 'sync_sequences.py'
                or name == 'build_variants.py' and any(a in args for a in ('--upload', '-upload')))
            sequence_write = writes and name in {'sync_sequences.py', 'upload_sequences.py'}
            if writes and data.get('ready') is not True:
                raise ValueError('Close Smash and open FTP first')
            if sequence_write and data.get('confirmed') is not True:
                raise ValueError('Confirm sequence transfer/replacement before running this helper')
            return self.app.run('advanced', helper(name, *args), {'helper': name})
        raise ValueError('Unknown action')


def launch(app):
    controller = Controller(app)
    token = secrets.token_hex(32)
    html = Path(__file__).with_suffix('.html').read_text().replace('__TOKEN__', json.dumps(token))
    initial = {'version': version(), 'cargo_home': app.preferences.get('cargo_home', '/private/tmp/kirby-assist-build-cargo'),
               'target': app.preferences.get('gui_target', 'all'), 'file': app.preferences.get('sequences', str(DEFAULT_FILE)),
               'source': app.preferences.get('gui_source', 'local'), 'destination': app.preferences.get('gui_destination', 'silent'),
               'source_file': app.preferences.get('gui_source_file', app.preferences.get('sequences', str(DEFAULT_FILE))),
               'destination_file': app.preferences.get('gui_destination_file', app.preferences.get('sequences', str(DEFAULT_FILE))),
               'fighter': app.preferences.get('gui_fighter', ''),
               'silent_port': app.port('silent'), 'diagnostic_port': app.port('diagnostic'), 'helpers': list(HELPERS)}
    html = html.replace('__INITIAL__', json.dumps(initial).replace('<', '\\u003c'))

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def respond(self, code, data, content_type='application/json'):
            payload = data.encode() if isinstance(data, str) else json.dumps(data).encode()
            self.send_response(code)
            self.send_header('Content-Type', content_type + '; charset=utf-8')
            self.send_header('Content-Length', str(len(payload)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.send_header('Content-Security-Policy', "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'")
            self.end_headers()
            self.wfile.write(payload)

        def allowed(self, api=True):
            authority = f'127.0.0.1:{self.server.server_port}'
            if self.headers.get('Host') != authority or self.headers.get('Origin', 'http://' + authority) != 'http://' + authority:
                self.respond(403, {'error': 'Local workbench requests only'})
                return False
            if api and not secrets.compare_digest(self.headers.get('X-Workbench-Token', ''), token):
                self.respond(403, {'error': 'Invalid session token'})
                return False
            return True

        def do_GET(self):
            route = urlsplit(self.path)
            if not self.allowed(api=route.path != '/'):
                return
            if route.path == '/':
                self.respond(200, html, 'text/html')
            elif route.path == '/api/state':
                self.respond(200, controller.snapshot())
            elif route.path == '/api/log':
                id = parse_qs(route.query).get('id', [''])[0]
                row = next((r for r in controller.rows if r['id'] == id), None)
                if not row:
                    self.respond(404, {'error': 'Unknown operation'})
                else:
                    path = Path(row['log']).resolve()
                    if path.parent != (app.history.directory / 'logs').resolve():
                        self.respond(403, {'error': 'Log outside workbench directory'})
                    else:
                        try:
                            with path.open('rb') as log:
                                log.seek(0, 2); size = log.tell(); log.seek(max(0, size - 160_000))
                                self.respond(200, {'output': log.read().decode(errors='replace')})
                        except OSError as error:
                            self.respond(400, {'error': str(error)})
            else:
                self.respond(404, {'error': 'Not found'})

        def do_POST(self):
            if not self.allowed():
                return
            try:
                size = int(self.headers.get('Content-Length', '0'))
                if not 0 < size <= 65536:
                    raise ValueError('Invalid request size')
                data = json.loads(self.rfile.read(size))
                if self.path == '/api/run':
                    controller.start(data)
                    self.respond(202, {'started': True})
                elif self.path == '/api/close':
                    if controller.snapshot()['busy']:
                        raise RuntimeError('Wait for the current operation before closing')
                    self.respond(200, {'closed': True})
                    threading.Thread(target=self.server.shutdown, daemon=True).start()
                else:
                    self.respond(404, {'error': 'Not found'})
            except (ValueError, RuntimeError) as error:
                self.respond(400, {'error': str(error)})

    with ThreadingHTTPServer(('127.0.0.1', 0), Handler) as server:
        server.daemon_threads = True
        url = f'http://127.0.0.1:{server.server_port}/'
        print('Workbench GUI:', url, flush=True)
        print('Keep this terminal open. Use Close workbench in the window to exit.', flush=True)
        try:
            webbrowser.open(url)
            server.serve_forever(poll_interval=0.3)
        except KeyboardInterrupt:
            print('\nClosing after any active operation finishes; transfers are not interrupted.', flush=True)
        finally:
            if controller.thread:
                controller.thread.join()
