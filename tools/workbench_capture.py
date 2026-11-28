"""Durable local stream capture and bounded, display-only tails."""
from __future__ import annotations

from collections import deque
import json
import os
from pathlib import Path
import threading

LOG_DIRECTORY = Path(os.environ.get('KIRBY_ASSIST_LOG_DIR', Path(__file__).resolve().parent / 'logs')).expanduser().resolve()


def append_record(path: Path, record: dict):
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    payload = (json.dumps(record, ensure_ascii=True, separators=(',', ':')) + '\n').encode()
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        remaining = memoryview(payload)
        while remaining:
            written = os.write(descriptor, remaining)
            if written <= 0:
                raise OSError('Capture write made no progress')
            remaining = remaining[written:]
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    directory = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def tail(path: Path, limit=256 * 1024):
    try:
        with path.open('rb') as handle:
            handle.seek(0, os.SEEK_END)
            size = handle.tell()
            handle.seek(max(0, size - limit))
            data = handle.read(limit)
        if size > limit:
            data = data.partition(b'\n')[2]
        return {'text': data.decode('utf-8', errors='replace'), 'bytes': size,
                'truncated': size > limit}
    except FileNotFoundError:
        return {'text': '', 'bytes': 0, 'truncated': False}


class LiveLogs:
    """Drain the native ring independently of browser filters, tabs and freezes."""
    def __init__(self, client):
        self.client = client
        self.lock = threading.Lock()
        self.entries = deque(maxlen=2048)
        self.dropped = 0
        self.error = ''
        self.generation = None
        self.epoch = 0
        self.stop_event = threading.Event()
        self.thread = threading.Thread(target=self.run, daemon=True)

    def poll(self):
        if not self.client.connected or self.client.lock.locked():
            return
        generation = self.client.generation
        try:
            result = self.client.perform('logs')
            with self.lock:
                if self.generation != generation or result['restarted']:
                    self.entries.clear()
                    self.epoch += 1
                self.generation = generation
                self.entries.extend(result['entries'])
                while self.entries and sum(len(e['text']) for e in self.entries) > 256 * 1024:
                    self.entries.popleft()
                self.dropped = result['dropped']
                self.error = ''
        except RuntimeError as error:
            with self.lock:
                self.error = str(error)

    def snapshot(self):
        with self.lock:
            return {'entries': list(self.entries), 'dropped': self.dropped, 'error': self.error,
                    'epoch': self.epoch, 'connected': self.client.connected}

    def run(self):
        while not self.stop_event.wait(0.1):
            self.poll()

    def stop(self):
        self.stop_event.set()
        self.thread.join(timeout=3)
