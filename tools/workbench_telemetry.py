"""Own a local, read-only telemetry receiver for one workbench session."""
from __future__ import annotations

import http.client
import os
from pathlib import Path
import signal
import socket
import shutil
import sys
import subprocess
import threading
import time


class TelemetryViewer:
    def __init__(self, root: Path):
        self.root = root
        self.lock = threading.RLock()
        self.process = None
        self.port = None
        self.status = 'Not started'
        self.ready = False
        self.started = None

    def start(self):
        with self.lock:
            if self.process is not None and self.process.poll() is None:
                raise RuntimeError('Stop the current telemetry receiver before starting another')
            self.ready = False
            self.port = None
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as reservation:
                reservation.bind(('127.0.0.1', 0))
                self.port = reservation.getsockname()[1]
            argv = ['cargo', 'run', '--offline', '--manifest-path',
                    str(self.root / 'crates/kirby_assist_core/Cargo.toml'),
                    '--features', 'diagnostic-logging', '--example', 'telemetry_viewer', '--',
                    str(self.port)]
            from workbench_capture import LOG_DIRECTORY
            cargo = shutil.which('cargo')
            if cargo is None and sys.platform == 'darwin':
                bundled_cargo = Path.home() / '.cargo' / 'bin' / 'cargo'
                cargo = str(bundled_cargo) if bundled_cargo.is_file() else None
            if cargo is None:
                self.status = 'Could not find Cargo. Install/configure the Rust host toolchain, then retry.'
                return self.snapshot()
            argv[0] = cargo
            environment = {**os.environ, 'KIRBY_TELEMETRY_CAPTURE': str(LOG_DIRECTORY / 'telemetry.txt')}
            try:
                self.process = subprocess.Popen(argv, cwd=self.root, stdin=subprocess.DEVNULL,
                                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                                env=environment, start_new_session=True)
            except OSError:
                self.process = None
                self.status = 'Could not start Cargo. Install/configure the existing Rust host toolchain, then retry.'
            else:
                self.started = time.monotonic()
                self.status = 'Starting local receiver (first launch may compile the host example)…'
            return self.snapshot()

    def _probe(self):
        try:
            connection = http.client.HTTPConnection('127.0.0.1', self.port, timeout=0.15)
            try:
                connection.request('GET', '/schema')
                response = connection.getresponse()
                return response.status == 200
            finally:
                connection.close()
        except (OSError, http.client.HTTPException):
            return False

    def request(self, route, method='GET'):
        if route not in {'/', '/schema', '/state'} and not (method == 'POST' and route.startswith('/connect?endpoint=')):
            raise ValueError('Unknown telemetry route')
        with self.lock:
            if not self.ready or self.process is None or self.process.poll() is not None:
                raise RuntimeError('Telemetry receiver is not ready')
            port = self.port
        connection = http.client.HTTPConnection('127.0.0.1', port, timeout=1)
        try:
            connection.request(method, route, headers={'Origin': f'http://127.0.0.1:{port}',
                                                       'Content-Length': '0'})
            response = connection.getresponse()
            body = response.read(2 * 1024 * 1024 + 1)
            if len(body) > 2 * 1024 * 1024:
                raise ValueError('Oversized telemetry response')
            return response.status, response.getheader('Content-Type', 'text/plain'), body
        finally:
            connection.close()

    def snapshot(self):
        with self.lock:
            running = self.process is not None and self.process.poll() is None
            if self.process is not None and not running:
                self.ready = False
                self.status = (f'Receiver exited (code {self.process.returncode}). Check capture disk space/permissions, the local HTTP port '
                               'and offline Cargo dependencies. Run the standalone '
                               'viewer command in a terminal for detailed errors; no automatic retry.')
            elif running and not self.ready:
                self.ready = self._probe()
                if self.ready:
                    self.status = 'Receiver ready · waiting for explicitly activated Switch telemetry'
                elif time.monotonic() - self.started > 120:
                    self.status = 'Still starting. Check the standalone command for build/cache or bind problems.'
            return {'running': running, 'ready': self.ready, 'status': self.status,
                    'url': f'http://127.0.0.1:{self.port}/' if self.ready else None}

    def stop(self):
        with self.lock:
            process, self.process = self.process, None
            if process is not None:
                # Cargo and its receiver share this new, session-owned process group.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    process.wait(timeout=3)
            self.ready = False
            self.status = 'Receiver stopped · Switch activation is unchanged'
            return self.snapshot()
