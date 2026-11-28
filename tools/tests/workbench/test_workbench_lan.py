"""Bounded HTTP peer simulations; no Switch, DNS, proxy or Rust build."""
import json
import socket
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

from workbench import LanClient, LanDatagramPeer, lan_endpoint


def reply(body, status=200, headers=b'', length=None):
    if isinstance(body, dict):
        body = json.dumps(body).encode()
    return (f'HTTP/1.1 {status} Test\r\nContent-Length: {len(body) if length is None else length}\r\nConnection: close\r\n'.encode()
            + headers + b'\r\n' + body)


class Peer:
    def __init__(self, response):
        self.response = response
        self.sent = b''
        self.address = None
        self.timeouts = []

    def __enter__(self):
        return self

    def __exit__(self, *args):
        pass

    def settimeout(self, value):
        self.timeouts.append(value)

    def connect(self, address):
        self.address = address

    def sendall(self, body):
        self.sent += body

    def recv(self, size):
        data, self.response = self.response[:size], self.response[size:]
        return data


class LanTests(unittest.TestCase):
    def setUp(self):
        self.client = LanClient()
        self.peers = []
        self.responses = []
        def factory(*args):
            self.assertEqual(args, (socket.AF_INET, socket.SOCK_STREAM))
            peer = Peer(self.responses.pop(0))
            self.peers.append(peer)
            return peer
        self.patch = patch('workbench.socket.socket', side_effect=factory)
        self.patch.start()
        self.addCleanup(self.patch.stop)
        dns = patch('socket.getaddrinfo', side_effect=AssertionError('DNS forbidden'))
        dns.start()
        self.addCleanup(dns.stop)

    def connect(self):
        self.responses.append(reply({'protocol': 1, 'training': True, 'editing': True}))
        return self.client.perform('connect', host='192.168.1.142', port=4242, token='shared-secret-012345678901234567890')

    def load(self, text=b'[unknown.settings]\nvalue=1\n'):
        self.responses.append(reply(text, headers=b'X-Kirby-Revision: AbC123\r\n'))
        return self.client.perform('load')

    def test_direct_auth_and_timeout_no_proxy_or_dns(self):
        with patch.dict('os.environ', {'HTTP_PROXY': 'http://evil.test', 'ALL_PROXY': 'http://evil.test'}):
            self.connect()
        peer = self.peers[0]
        self.assertEqual(peer.address, ('192.168.1.142', 4242))
        self.assertIn(b'X-Kirby-Token: shared-secret-012345678901234567890\r\n', peer.sent)
        self.assertIn(b'GET /status HTTP/1.1', peer.sent)
        self.assertTrue(all(0 < value <= 2 for value in peer.timeouts))
        self.client.disconnect()
        self.assertEqual(self.client.token, '')
        with self.assertRaisesRegex(RuntimeError, 'Connect explicitly'):
            self.client.perform('logs')
        self.assertEqual(len(self.peers), 1)

    def test_tokenless_http_default_and_explicit_transport_validation(self):
        for token in [None, '']:
            self.responses.append(reply({'protocol': 1, 'training': True, 'editing': True}))
            arguments = {} if token is None else {'token': token}
            self.client.perform('connect', host='10.0.0.1', port=4242, **arguments)
            self.assertNotIn(b'X-Kirby-Token:', self.peers[-1].sent)
        count = len(self.peers)
        with self.assertRaises(RuntimeError):
            self.client.perform('connect', host='10.0.0.1', port=4242, transport='auto')
        self.assertEqual(len(self.peers), count)

    def test_network_errors_are_specific_and_redacted(self):
        for exception, message in [(ConnectionRefusedError('secret draft'), 'connection refused'),
                                   (TimeoutError('secret draft'), 'timed out'),
                                   (OSError('secret draft'), 'network connection failed')]:
            with patch('workbench.socket.socket', side_effect=exception):
                with self.assertRaisesRegex(RuntimeError, message) as caught:
                    self.client.perform('connect', host='10.0.0.1', port=4242)
            self.assertNotIn('secret draft', str(caught.exception))
            self.assertFalse(self.client.connected)

    def test_complete_text_preview_revision_and_save(self):
        self.connect()
        text = self.load()['text']
        with patch('workbench.transfer.validate_sequences') as validate:
            self.client.perform('preview', text=text + '# comment\n')
            validate.assert_called_once_with((text + '# comment\n').encode())
        self.responses.append(reply({'saved': True, 'reloaded': True}))
        self.client.perform('save', text=text + '# comment\n', confirmed=True)
        self.assertIn(b'X-Kirby-Revision: AbC123\r\n', self.peers[-1].sent)
        self.assertTrue(self.peers[-1].sent.endswith((text + '# comment\n').encode()))
        self.assertIsNone(self.client.revision)
        self.assertTrue(all(0 < value <= 10 for value in self.peers[-1].timeouts))
        self.assertGreater(self.peers[-1].timeouts[0], 2)

    def test_save_acknowledgment_loss_requires_reload_not_retry(self):
        self.connect()
        text = self.load()['text']
        with patch('workbench.transfer.validate_sequences'):
            self.client.perform('preview', text=text)
        self.responses.append(reply(b'bad-json'))
        with self.assertRaisesRegex(RuntimeError, 'disk may have changed'):
            self.client.perform('save', text=text, confirmed=True)
        count = len(self.peers)
        with self.assertRaisesRegex(RuntimeError, 'Connect explicitly'):
            self.client.perform('save', text=text, confirmed=True)
        self.assertEqual(len(self.peers), count)

    def test_save_server_failure_warns_disk_may_have_changed(self):
        self.connect()
        text = self.load()['text']
        with patch('workbench.transfer.validate_sequences'):
            self.client.perform('preview', text=text)
        self.responses.append(reply(b'', 500))
        with self.assertRaisesRegex(RuntimeError, 'disk may have changed'):
            self.client.perform('save', text=text, confirmed=True)
        self.assertFalse(self.client.connected)

    def test_disconnect_after_transport_cannot_publish_session_state(self):
        for operation in ['connect', 'load', 'logs']:
            with self.subTest(operation=operation):
                self.connect()
                def completed_request(*args):
                    self.client.disconnect()
                    if operation == 'connect':
                        return b'{"protocol":1,"training":true,"editing":true}', {}
                    if operation == 'logs':
                        return b'{"next":1,"dropped":0,"entries":[]}', {}
                    return b'[kirby.settings]\nx=1\n', {'x-kirby-revision': 'abc'}
                with patch.object(self.client, '_request', side_effect=completed_request):
                    with self.assertRaisesRegex(RuntimeError, 'Session disconnected'):
                        self.client.perform(operation, host='10.0.0.1', port=4242, token='x' * 32)
                self.assertFalse(self.client.connected)
                self.assertIsNone(self.client.original)
                self.assertIsNone(self.client.revision)
                self.assertEqual(self.client.next, 0)

    def test_save_requires_exact_validated_draft_and_stale_save_disconnects(self):
        self.connect()
        text = self.load()['text']
        count = len(self.peers)
        with self.assertRaisesRegex(RuntimeError, 'Preview'):
            self.client.perform('save', text=text, confirmed=True)
        self.assertEqual(len(self.peers), count)
        self.connect()
        text = self.load()['text']
        with patch('workbench.transfer.validate_sequences'):
            self.client.perform('preview', text=text)
        self.responses.append(reply(b'', 409))
        with self.assertRaisesRegex(RuntimeError, 'Stale revision'):
            self.client.perform('save', text=text, confirmed=True)
        self.assertFalse(self.client.connected)
        self.assertIsNone(self.client.revision)
        self.assertIn(b'PUT /sequences', self.peers[-1].sent)

    def test_status_training_gate_and_response_bounds(self):
        for response in [reply({'protocol': 2, 'training': True, 'editing': True}),
                         reply({'protocol': 1, 'training': False, 'editing': True}),
                         reply({'protocol': True, 'training': True, 'editing': True}),
                         reply(b'', length=8192)]:
            self.responses.append(response)
            with self.assertRaises(RuntimeError):
                self.client.perform('connect', host='10.0.0.1', port=4242,
                                    token='x' * 32)
            self.assertFalse(self.client.connected)
            self.assertEqual(self.client.token, '')

    def test_stale_revocation_redirect_timeout_never_retry(self):
        for response in [reply(b'', 409), reply(b'', 403), reply(b'', 302, b'Location: http://evil.test/\r\n')]:
            with self.subTest(response=response):
                self.connect()
                self.responses.append(response)
                with self.assertRaises(RuntimeError):
                    self.client.perform('load')
                count = len(self.peers)
                with self.assertRaisesRegex(RuntimeError, 'Connect explicitly'):
                    self.client.perform('load')
                self.assertEqual(len(self.peers), count)
                self.assertFalse(self.client.connected)
                self.assertEqual(self.client.token, '')
        self.connect()
        with patch('workbench.socket.socket', side_effect=TimeoutError('shared-secret-012345678901234567890')):
            with self.assertRaises(RuntimeError) as error:
                self.client.perform('logs')
        self.assertNotIn('shared-secret-012345678901234567890', str(error.exception))
        self.assertFalse(self.client.connected)

    def test_invalid_bounds_and_framing_disconnect_before_body(self):
        for response in [reply(b'', length=524289), reply(b'bad', length=5),
                         b'HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n',
                         reply(b'x', headers=b'Transfer-Encoding: chunked\r\n'),
                         reply(b'x', headers=b'Content-Length: 1\r\n'),
                         reply(b'\xff', headers=b'X-Kirby-Revision: 1\r\n'),
                         reply(b'x', headers=b'X-Kirby-Revision: nope\r\n')]:
            self.connect()
            self.responses.append(response)
            with self.assertRaises(RuntimeError):
                self.client.perform('load')
            self.assertFalse(self.client.connected)
        self.connect()
        self.responses.append(reply(b'', length=65537))
        with self.assertRaises(RuntimeError):
            self.client.perform('logs')

    def test_log_cursor_bounds_and_untrusted_text(self):
        self.connect()
        text = '<img src=x onerror=alert(1)>'
        self.responses.append(reply({'next': 3, 'dropped': 2, 'entries': [{'id': 3, 'text': text}]}))
        self.assertEqual(self.client.perform('logs')['entries'][0]['text'], text)
        self.responses.append(reply({'next': 3, 'dropped': 2, 'entries': []}))
        self.client.perform('logs')
        self.assertIn(b'GET /logs?after=3 ', self.peers[-1].sent)
        for result in [{'next': True, 'dropped': 0, 'entries': []},
                       {'next': 4, 'dropped': 0, 'entries': [{'id': 4, 'text': 1}]},
                       {'next': 65, 'dropped': 0, 'entries': [{'id': n, 'text': 'x'} for n in range(1, 66)]}]:
            self.connect()
            self.responses.append(reply(result))
            with self.assertRaises(RuntimeError):
                self.client.perform('logs')
        html = Path(__file__).resolve().parents[2].joinpath('workbench_gui.html').read_text()
        draw = html[html.index('function drawLanLogs()'):html.index('function lanClearLogs()')]
        self.assertIn('out.textContent=value', draw)
        self.assertNotIn('innerHTML', draw)

    def test_native_restart_resets_cursor_and_preserves_retained_batch(self):
        self.connect()
        self.client.next = 100
        self.responses.append(reply({'next': 2, 'dropped': 0, 'entries': [{'id': 1, 'text': 'startup'}, {'id': 2, 'text': 'ready'}]}))
        result = self.client.perform('logs')
        self.assertTrue(result['restarted'])
        self.assertEqual(self.client.next, 2)
        self.responses.append(reply({'next': 2, 'dropped': 0, 'entries': []}))
        self.assertFalse(self.client.perform('logs')['restarted'])
        self.assertIn(b'GET /logs?after=2 ', self.peers[-1].sent)

    def test_state_lock_is_not_held_by_slow_lan_request(self):
        self.connect()
        self.assertTrue(self.client.lock.acquire(False))
        try:
            with self.assertRaisesRegex(RuntimeError, 'already running'):
                self.client.perform('logs')
            self.assertTrue(self.client.connected)
            self.client.disconnect()
            self.assertFalse(self.client.connected)
        finally:
            self.client.lock.release()

    def test_reject_public_dns_and_header_injection_without_socket(self):
        for host in ['localhost', '8.8.8.8', '127.0.0.1', '169.254.1.1', '::1', 'https://192.168.1.1']:
            with self.assertRaises(ValueError):
                lan_endpoint(host, 4242)
        for token in ['x\r\nHost: evil', 'with space', 'é']:
            with self.assertRaises(RuntimeError):
                self.client.perform('connect', host='10.0.0.1', port=4242, token=token)
        self.assertEqual(self.peers, [])


class UdpLanTests(unittest.TestCase):
    def test_real_udp_chunks_retries_peer_filtering_and_single_save(self):
        header = LanDatagramPeer.header
        original = '[unknown.settings]\nvalue=1\n' + '# preserved\n' * 220
        updated = original + '# new draft\n'
        requests, saves, errors = [], [], []
        transfers = {}
        stop = threading.Event()
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server, \
             socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as rogue:
            server.bind(('127.0.0.1', 0))
            server.settimeout(.1)
            dropped_ack = set()
            dropped_fetch = set()

            def serve():
                try:
                    while not stop.is_set():
                        try:
                            packet, source = server.recvfrom(1042)
                        except TimeoutError:
                            continue
                        magic, kind, transfer_id, index, total = header.unpack(packet[:17])
                        self.assertEqual(magic, b'KALE')
                        self.assertNotEqual(transfer_id, 0)
                        if kind == 1:
                            self.assertTrue(1 <= total <= 520)
                            self.assertLessEqual(len(packet) - 17, 1024)
                            transfer = transfers.setdefault(transfer_id, {'chunks': {}, 'total': total})
                            self.assertEqual(transfer['total'], total)
                            transfer['chunks'][index] = packet[17:]
                            ack = header.pack(b'KALE', 2, transfer_id, index, total)
                            # A connected UDP socket must ignore even a valid ACK from another endpoint.
                            rogue.sendto(ack, source)
                            server.sendto(header.pack(b'KALE', 2, transfer_id ^ 1, index, total), source)
                            server.sendto(header.pack(b'KALE', 2, transfer_id, index + 1, total), source)
                            request = b''.join(transfer['chunks'][n] for n in sorted(transfer['chunks']))
                            is_save = request.startswith(b'PUT ')
                            if is_save and index + 1 == total and transfer_id not in dropped_ack:
                                dropped_ack.add(transfer_id)
                            else:
                                server.sendto(ack, source)
                            if len(transfer['chunks']) == total and 'response' not in transfer:
                                requests.append((transfer_id, request))
                                if request.startswith(b'GET /status '):
                                    response = reply({'protocol': 1, 'training': True, 'editing': True})
                                elif request.startswith(b'GET /sequences '):
                                    response = reply(original.encode(), headers=b'X-Kirby-Revision: aBc012\r\n')
                                elif is_save:
                                    self.assertIn(b'X-Kirby-Revision: aBc012\r\n', request)
                                    saves.append(request.split(b'\r\n\r\n', 1)[1])
                                    response = reply({'saved': True, 'reloaded': True})
                                else:
                                    raise AssertionError('Unexpected request')
                                self.assertNotIn(b'X-Kirby-Token:', request)
                                transfer['response'] = response
                                transfer['ready_at'] = time.monotonic() + (.6 if is_save else 0)
                        else:
                            self.assertEqual((kind, total, packet[17:]), (3, 0, b''))
                            transfer = transfers[transfer_id]
                            if time.monotonic() < transfer['ready_at']:
                                continue
                            # Lost fetches are safe to repeat using the original request ID.
                            if (transfer_id, index) not in dropped_fetch:
                                dropped_fetch.add((transfer_id, index))
                                continue
                            response = transfer['response']
                            count = (len(response) + 1023) // 1024
                            server.sendto(header.pack(b'KALE', 4, transfer_id, index + 1, count)
                                          + b'out of order', source)
                            server.sendto(header.pack(b'KALE', 4, transfer_id, index, count)
                                          + response[index * 1024:(index + 1) * 1024], source)
                except Exception as error:
                    errors.append(error)

            thread = threading.Thread(target=serve, daemon=True)
            thread.start()
            client = LanClient()
            try:
                with patch('workbench.lan_endpoint', return_value=server.getsockname()):
                    self.assertTrue(client.perform('connect', host='10.0.0.1', port=4242,
                                                   transport='udp')['connected'])
                self.assertEqual(client.perform('load')['text'], original)
                with patch('workbench.transfer.validate_sequences'):
                    client.perform('preview', text=updated)
                self.assertEqual(client.perform('save', text=updated, confirmed=True),
                                 {'saved': True, 'reloaded': True})
                self.assertEqual(saves, [updated.encode()])
                self.assertEqual(len(requests), 3)
                self.assertEqual(len({transfer_id for transfer_id, _ in requests}), 3)
            finally:
                client.disconnect()
                stop.set()
                thread.join(timeout=2)
            self.assertFalse(thread.is_alive())
            self.assertEqual(errors, [])

    def test_lost_udp_save_response_never_resubmits_or_changes_transport(self):
        header = LanDatagramPeer.header
        stop = threading.Event()
        request_ids, fetches = set(), []
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server:
            server.bind(('127.0.0.1', 0))
            server.settimeout(.1)
            def serve():
                while not stop.is_set():
                    try:
                        packet, source = server.recvfrom(1042)
                    except TimeoutError:
                        continue
                    _, kind, transfer_id, index, total = header.unpack(packet[:17])
                    if kind == 1:
                        request_ids.add(transfer_id)
                        server.sendto(header.pack(b'KALE', 2, transfer_id, index, total), source)
                    else:
                        fetches.append(transfer_id)
            thread = threading.Thread(target=serve, daemon=True)
            thread.start()
            client = LanClient()
            client.host, client.port = server.getsockname()
            client.transport, client.connected = 'udp', True
            client.original = client.preview_text = '[unknown.settings]\nvalue=1\n'
            client.revision = 'abc'
            try:
                with self.assertRaisesRegex(RuntimeError, 'timed out.*disk may have changed'):
                    client.perform('save', text=client.preview_text, confirmed=True)
                count = len(request_ids)
                with self.assertRaisesRegex(RuntimeError, 'Connect explicitly'):
                    client.perform('save', text='anything', confirmed=True)
                self.assertEqual(count, 1)
                self.assertEqual(request_ids, set(fetches))
                self.assertGreater(len(fetches), 3)
                self.assertEqual(client.transport, 'udp')
            finally:
                stop.set()
                thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_udp_ack_timeout_is_bounded_without_http_fallback(self):
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as server:
            server.bind(('127.0.0.1', 0))
            server.settimeout(.1)
            with patch('workbench.lan_endpoint', return_value=server.getsockname()):
                client = LanClient()
                with self.assertRaisesRegex(RuntimeError, 'timed out'):
                    client.perform('connect', host='10.0.0.1', port=4242, transport='udp')
            packets = []
            while True:
                try:
                    packets.append(server.recv(1042))
                except TimeoutError:
                    break
            self.assertEqual(len(packets), 3)
            self.assertEqual(len(set(packets)), 1)
            self.assertFalse(client.connected)


class LoopbackLanTests(unittest.TestCase):
    def test_real_http_tokenless_connect(self):
        headers = []
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_GET(self):
                headers.append(dict(self.headers))
                body = b'{"protocol":1,"training":true,"editing":true}'
                self.send_response(200)
                self.send_header('Content-Length', str(len(body)))
                self.send_header('Connection', 'close')
                self.end_headers()
                self.wfile.write(body)
        with ThreadingHTTPServer(('127.0.0.1', 0), Handler) as server:
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            client = LanClient()
            try:
                with patch('workbench.lan_endpoint', return_value=server.server_address):
                    self.assertTrue(client.perform('connect', host='10.0.0.1', port=4242)['connected'])
                self.assertEqual(len(headers), 1)
                self.assertNotIn('X-Kirby-Token', headers[0])
            finally:
                client.disconnect()
                server.shutdown()
                thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_real_http_session_framing_auth_revision_save_and_logs(self):
        token = 'loopback-fixture-secret-0123456789012345'
        original = '[unknown.settings]\nvalue=1\n'
        updated = original + '# preserved complete-file draft\n'
        requests = []
        saved = []

        class Handler(BaseHTTPRequestHandler):
            protocol_version = 'HTTP/1.1'

            def log_message(self, *args):
                pass

            def handle_request(self):
                size = int(self.headers['Content-Length'])
                body = self.rfile.read(size)
                requests.append((self.command, self.path, dict(self.headers), body))
                if self.headers.get('X-Kirby-Token') != token:
                    payload, status = b'', 403
                elif self.path == '/status':
                    payload = json.dumps({'protocol': 1, 'training': True, 'editing': True}).encode()
                    status = 200
                elif self.path == '/sequences' and self.command == 'GET':
                    payload, status = original.encode(), 200
                elif self.path == '/sequences' and self.command == 'PUT':
                    if self.headers.get('X-Kirby-Revision') != 'aBc012':
                        payload, status = b'', 409
                    else:
                        saved.append(body)
                        payload, status = b'{"saved":true,"reloaded":true}', 200
                elif self.path == '/logs?after=0':
                    payload = json.dumps({'next': 1, 'dropped': 0,
                                          'entries': [{'id': 1, 'text': 'startup <not HTML>'}]}).encode()
                    status = 200
                else:
                    payload, status = b'', 404
                self.send_response(status)
                self.send_header('Content-Length', str(len(payload)))
                self.send_header('Connection', 'close')
                if self.path == '/sequences' and self.command == 'GET':
                    self.send_header('X-Kirby-Revision', 'aBc012')
                self.end_headers()
                self.wfile.write(payload)
                self.close_connection = True

            do_GET = handle_request
            do_PUT = handle_request

        client = LanClient()
        with ThreadingHTTPServer(('127.0.0.1', 0), Handler) as server:
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                # Only endpoint validation is substituted; all socket I/O is real.
                with patch('workbench.lan_endpoint', return_value=('127.0.0.1', server.server_port)):
                    self.assertTrue(client.perform('connect', host='192.168.1.142',
                                                   port=4242, token=token)['connected'])
                self.assertEqual(client.perform('load'), {'text': original, 'revision': 'aBc012'})
                with patch('workbench.transfer.validate_sequences') as validate:
                    preview = client.perform('preview', text=updated)
                    validate.assert_called_once_with(updated.encode())
                self.assertIn('+# preserved complete-file draft', preview['diff'])
                self.assertEqual(client.perform('save', text=updated, confirmed=True),
                                 {'saved': True, 'reloaded': True})
                self.assertEqual(saved, [updated.encode()])
                self.assertEqual(client.perform('logs')['entries'],
                                 [{'id': 1, 'text': 'startup <not HTML>'}])
                client.perform('disconnect')
                with self.assertRaisesRegex(RuntimeError, 'Connect explicitly'):
                    client.perform('logs')
                self.assertEqual(client.token, '')
                self.assertEqual([(method, path) for method, path, _, _ in requests],
                                 [('GET', '/status'), ('GET', '/sequences'),
                                  ('PUT', '/sequences'), ('GET', '/logs?after=0')])
                for _, _, headers, body in requests:
                    self.assertEqual(headers['X-Kirby-Token'], token)
                    self.assertEqual(headers['Connection'], 'close')
                    self.assertEqual(int(headers['Content-Length']), len(body))
                    self.assertEqual(headers['Host'], f'127.0.0.1:{server.server_port}')
                self.assertEqual(requests[2][2]['X-Kirby-Revision'], 'aBc012')
            finally:
                client.disconnect()
                server.shutdown()
                thread.join(timeout=3)
                self.assertFalse(thread.is_alive())


if __name__ == '__main__':
    unittest.main()
