"""Capture regressions using synthetic streams and temporary files only."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from workbench import LanClient
from workbench_capture import append_record, LiveLogs, tail


class CaptureTests(unittest.TestCase):
    def test_append_sync_and_bounded_tail_survive_reopen(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'logs' / 'diagnostic.txt'
            with patch('workbench_capture.os.fsync', wraps=__import__('os').fsync) as sync:
                append_record(path, {'text': 'first\nrecord Ω'})
                append_record(path, {'text': 'second'})
                self.assertEqual(sync.call_count, 4)
            rows = [json.loads(line) for line in path.read_text().splitlines()]
            self.assertEqual([row['text'] for row in rows], ['first\nrecord Ω', 'second'])
            limited = tail(path, limit=24)
            self.assertTrue(limited['truncated'])
            self.assertEqual(json.loads(limited['text']), {'text': 'second'})
            self.assertEqual(tail(path)['bytes'], path.stat().st_size)
            self.assertEqual(path.stat().st_mode & 0o777, 0o600)

    def test_native_batch_is_saved_before_cursor_is_published(self):
        client = LanClient()
        client.connected = True
        client.host, client.port = '192.168.1.10', 4242
        with tempfile.TemporaryDirectory() as directory:
            client.capture_path = Path(directory) / 'diagnostic.txt'
            batch = {'entries': [{'id': 1, 'text': 'mapper input'}], 'next': 1, 'dropped': 0}
            with patch.object(client, '_request', return_value=(json.dumps(batch).encode(), {})):
                client.perform('logs')
            self.assertEqual(client.next, 1)
            self.assertEqual(json.loads(client.capture_path.read_text())['entries'], batch['entries'])
            batch['entries'][0]['id'] = batch['next'] = 2
            with patch.object(client, '_request', return_value=(json.dumps(batch).encode(), {})), \
                 patch('workbench_capture.append_record', side_effect=OSError('disk full')):
                with self.assertRaisesRegex(RuntimeError, 'without advancing'):
                    client.perform('logs')
            self.assertFalse(client.connected)
            self.assertEqual(len(client.capture_path.read_text().splitlines()), 1)

    def test_background_drain_retains_view_and_resets_on_session_change(self):
        client = LanClient()
        client.connected = True
        monitor = LiveLogs(client)
        batches = [
            {'entries': [{'id': 3, 'text': 'first'}], 'next': 3, 'dropped': 0},
            {'entries': [{'id': 4, 'text': 'second'}], 'next': 4, 'dropped': 2},
            {'entries': [{'id': 1, 'text': 'new session'}], 'next': 1, 'dropped': 0}]
        with patch.object(client, '_request', side_effect=[(json.dumps(b).encode(), {}) for b in batches]):
            monitor.poll()
            monitor.poll()
            self.assertEqual([e['text'] for e in monitor.snapshot()['entries']], ['first', 'second'])
            epoch = monitor.snapshot()['epoch']
            monitor.poll()
        self.assertGreater(monitor.snapshot()['epoch'], epoch)
        self.assertEqual(monitor.snapshot()['entries'], [{'id': 1, 'text': 'new session'}])


if __name__ == '__main__':
    unittest.main()
