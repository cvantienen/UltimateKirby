"""Adapter: reuse the browser controller, without starting its HTTP server."""
from __future__ import annotations

import json
from pathlib import Path
import threading

import workbench
from workbench_capture import LiveLogs, LOG_DIRECTORY, tail
from workbench_gui import Controller
from workbench_telemetry import TelemetryViewer


class Backend:
    def __init__(self, app):
        self.app = app
        self.controller = Controller(app)
        self.telemetry = TelemetryViewer(workbench.PROJECT_ROOT)
        self.live = LiveLogs(app.lan)
        self.lock = threading.Lock()
        self.closed = False

    def run(self, data):
        if not self.lock.acquire(blocking=False):
            raise RuntimeError("An operation is already running")
        try:
            self.controller.start(data)
            self.controller.thread.join()
            state = self.controller.snapshot()
            if state['status'].startswith('Stopped:'):
                raise RuntimeError(state['status'])
            return state['result']
        finally:
            self.lock.release()

    def lan(self, operation, **data):
        if not self.lock.acquire(blocking=False):
            raise RuntimeError("An operation is already running")
        try:
            if operation == 'connect':
                self.app.lan.disconnect()
                self.app.configure_lan_editor(data['host'], data['port'])
            result = self.app.lan.perform(operation, **data)
            if operation == 'connect' and not self.live.thread.is_alive():
                self.live.thread.start()
            return result
        finally:
            self.lock.release()

    def telemetry_request(self, route, method='GET'):
        code, _, body = self.telemetry.request(route, method)
        if code != 200:
            raise RuntimeError(body.decode(errors='replace')[:1000])
        return json.loads(body)

    def streams(self):
        return {'telemetry': tail(LOG_DIRECTORY / 'telemetry.txt')['text'],
                'diagnostic': tail(LOG_DIRECTORY / 'diagnostic.txt')['text'],
                'live': '\n'.join(e['text'] for e in self.live.snapshot()['entries'])}

    def history_log(self, identifier):
        row = next((r for r in self.controller.rows if r['id'] == identifier), None)
        if not row:
            raise ValueError('Unknown operation')
        path = Path(row['log']).resolve()
        if path.parent != (self.app.history.directory / 'logs').resolve():
            raise ValueError('Log outside workbench directory')
        return tail(path, 160_000)['text']

    def close(self):
        if self.closed:
            return
        if self.controller.thread:
            self.controller.thread.join()
        with self.lock:
            self.telemetry.stop()
            self.app.lan.disconnect()
            if self.live.thread.is_alive():
                self.live.stop()
            self.closed = True
