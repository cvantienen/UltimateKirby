#!/usr/bin/env python3
"""Local web, Textual and numbered-terminal build/upload/sequence workbench.

Usage from the project directory:
    python3 tools/workbench.py
    python3 tools/workbench.py --cli
    python3 tools/workbench.py --history
    python3 tools/workbench.py --web
    python3 tools/workbench.py --terminal

Default macOS startup asks Web or Terminal in a native chooser (Cancel does nothing).
Other platforms default to Web; --choose requires macOS. Explicit flags bypass the
chooser. Textual needs its requirements and an interactive terminal; packaged app
selection opens Terminal.app before acquiring the shared-state lock. --gui remains
an alias for --web. Web opens a local browser GUI: enter a version (or +1), select Switches,
confirm Smash closed/FTP open once, then click Build + Upload. The build version
is written to Cargo.toml before building. Existing build/upload helpers perform
the work and retain their validation/backups. GUI opening never builds/uploads.
Edit fighter changes one fighter's sections against a single snapshot, with a
diff preview before writing; Sequences has a preview and explicit confirmation
before replacement; Helpers exposes original scripts. --cli keeps the original
numbered prompt below.

In --cli, replace the active sequences.txt on a Switch:
    Choose "Transfer / replace sequences", select the local source file,
    select the destination Switch and "ACTIVE sequences.txt", then choose
    "Whole file". Review the preview and type TRANSFER to confirm replacement.
    Close Smash and leave FTP open before transferring. Whole-file replacement
    overwrites all active recipes/settings; choose "One fighter" instead to
    replace only that fighter while preserving the others. Validation, recovery
    snapshots and destination readback verification run before completion.

Sources and destinations can also be computer files or the other Switch.
STAGED sequences.upload.txt is inactive: uploading a release preserves active
sequences.txt, and restarting does not activate the staged copy.

The prompt stays open until Exit. Local history/preferences/backups live in
.mod-workbench/ (override with --state-dir). Existing standalone helpers remain
available; no Switch release build or device write occurs merely by opening this
launcher. The GUI also starts the read-only telemetry receiver (offline Cargo may
compile its host example). Training telemetry has its own sidebar dashboard;
Switch streaming still requires explicit activation. Closing the GUI stops its
receiver. --cli and --history do not start it. The macOS app uses a saved project
checkout and keeps logs/history outside the selected checkout.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import ftplib
import json
import os
import ipaddress
import secrets
import struct
import subprocess
import socket
import threading
import time
import difflib
import re
from pathlib import Path
import shlex
import sys
import tempfile

if getattr(sys, "frozen", False):
    os.environ.setdefault("KIRBY_ASSIST_STATE_DIR", str(Path.home() / "Library/Application Support/Kirby Assist Workbench"))
    os.environ.setdefault("KIRBY_ASSIST_LOG_DIR", str(Path(os.environ["KIRBY_ASSIST_STATE_DIR"]) / "logs"))

import fighter_editor
import sequence_sources as sources
import sequence_transfer as transfer
from toml_compat import tomllib
import upload_switch
import workbench_history
from workbench_capture import LOG_DIRECTORY
from upload_all_switches import TARGETS
from workbench_history import History, ROOT, STATE, command
import sequence_sources

TOOLS_DIR = Path(getattr(sys, "_MEIPASS", Path(__file__).resolve().parent)).resolve()
IS_FROZEN = bool(getattr(sys, "frozen", False))
PROJECT_ROOT = ROOT
if IS_FROZEN:
    ROOT = Path(os.environ.get("KIRBY_ASSIST_PROJECT_ROOT", Path.home() / "UltimateKirby")).expanduser().resolve()
    STATE = Path(os.environ.get("KIRBY_ASSIST_STATE_DIR", Path.home() / "Library/Application Support/Kirby Assist Workbench")).expanduser().resolve()
    os.environ.setdefault("KIRBY_ASSIST_STATE_DIR", str(STATE))
    os.environ.setdefault("KIRBY_ASSIST_LOG_DIR", str(STATE / "logs"))
DEFAULT_FILE = ROOT / "deployment/ultimate/kirby_assist/assembled/sequences.txt"

HELPERS = ("build_variants.py", "upload_all_switches.py", "upload_switch.py",
           "upload_sequences.py", "sync_sequences.py", "sequence_validation.py",
           "sequence_sources.py")
# The assembled bundle is the default transfer file. It is the exact file that
# is deployed and is regenerated from the canonical fighter sources.


def choose(title, items, *, back=True, back_label="Back"):
    print(f"\n{title}")
    for index, item in enumerate(items, 1):
        print(f"  {index}. {item}")
    if back:
        print(f"  0. {back_label}")
    while True:
        answer = input("> ").strip().lower()
        if back and answer in ("0", "b", "back", "q"):
            return None
        if answer.isdigit() and 1 <= int(answer) <= len(items):
            return int(answer) - 1
        print("Choose a listed number.")


def ask(label, default=""):
    value = input(f"{label}{' [' + str(default) + ']' if default != '' else ''}: ").strip()
    return value or str(default)


def confirm(label, token):
    print(label)
    return input(f"Type {token} to continue; anything else cancels: ").strip() == token


def version():
    # Re-read so editing the manifest while the launcher stays open is supported.
    try:
        return str(tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"])
    except (OSError, ValueError, KeyError):
        if not IS_FROZEN:
            raise
        return ""


def helper(name, *args):
    if name not in HELPERS:
        raise ValueError("Unknown helper")
    script_root = PROJECT_ROOT if (PROJECT_ROOT / "tools" / name).is_file() else TOOLS_DIR
    if IS_FROZEN:
        helper_module = {"build_variants.py": "build_variants", "upload_all_switches.py": "upload_all_switches",
                         "upload_switch.py": "upload_switch", "upload_sequences.py": "upload_sequences",
                         "sync_sequences.py": "sync_sequences", "sequence_validation.py": "sequence_validation",
                         "sequence_sources.py": "sequence_sources"}.get(name)
        if helper_module is None:
            raise ValueError("Unknown bundled helper")
        os.environ["KIRBY_ASSIST_FROZEN_ROOT"] = str(PROJECT_ROOT)
        return [sys.executable, "--workbench-helper", helper_module, *map(str, args)]
    return [sys.executable, str(script_root / name), *map(str, args)]


class LanDatagramPeer:
    """Carry one framed editor request over acknowledged, bounded UDP fragments."""
    header = struct.Struct('>4sBQHH')

    def __init__(self, remaining, response_limit):
        self.remaining = remaining
        self.response_limit = response_limit
        self.transfer_id = secrets.randbelow(2**64 - 1) + 1
        self.response_total = None
        self.response_index = 0
        self.received_size = 0
        self.buffer = b''

    def __enter__(self):
        self.peer = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        return self

    def __exit__(self, *args):
        self.peer.close()

    def settimeout(self, value):
        self.peer.settimeout(value)

    def connect(self, address):
        self.peer.connect(address)

    def exchange(self, kind, index, total, payload, response_kind, attempts=None):
        packet = self.header.pack(b'KALE', kind, self.transfer_id, index, total) + payload
        attempt = 0
        while attempts is None or attempt < attempts:
            attempt += 1
            self.peer.settimeout(self.remaining())
            if self.peer.send(packet) != len(packet):
                raise OSError('Incomplete UDP datagram send')
            fragment_deadline = time.monotonic() + min(.15, self.remaining())
            while True:
                wait = min(fragment_deadline - time.monotonic(), self.remaining())
                if wait <= 0:
                    break
                self.peer.settimeout(wait)
                try:
                    response = self.peer.recv(1042)
                except TimeoutError:
                    break
                if not 17 <= len(response) <= 1041:
                    continue
                magic, received_kind, transfer_id, received_index, received_total = self.header.unpack(response[:17])
                if (magic, received_kind, transfer_id, received_index) != (b'KALE', response_kind, self.transfer_id, index):
                    continue
                if response_kind in (2, 6):
                    if received_total == total and len(response) == 17:
                        return b'', received_total
                elif 1 <= received_total <= 520 and index < received_total:
                    if self.response_total is not None and received_total != self.response_total:
                        raise ValueError('Inconsistent LAN response fragments')
                    return response[17:], received_total
        raise TimeoutError()

    def sendall(self, body):
        if not body or len(body) > 512 * 1024 + 4096:
            raise ValueError('Oversized LAN request')
        total = (len(body) + 1023) // 1024
        for index in range(total):
            self.exchange(1, index, total, body[index * 1024:(index + 1) * 1024], 2, attempts=3)

    def recv(self, size):
        if not self.buffer:
            if self.response_total is not None and self.response_index == self.response_total:
                self.exchange(5, self.response_index, 0, b'', 6, attempts=3)
                return b''
            chunk, total = self.exchange(3, self.response_index, 0, b'', 4)
            if not chunk or (self.response_index + 1 < total and len(chunk) != 1024):
                raise ValueError('Invalid LAN response fragment size')
            self.response_total = total
            self.response_index += 1
            self.received_size += len(chunk)
            if self.received_size > self.response_limit:
                raise ValueError('Oversized LAN response')
            self.buffer = chunk
        result, self.buffer = self.buffer[:size], self.buffer[size:]
        return result


class LanClient:
    """Explicit diagnostic session; fragment retries never repeat a request."""
    def __init__(self):
        self.capture_path = None
        self.capture_dropped = None
        self.lock = threading.Lock()
        self.state_lock = threading.RLock()
        self.connected = False
        self.host, self.port, self.token = '', 4242, ''
        self.transport = 'udp'
        self.editing = False
        self.revision = None
        self.original = None
        self.preview_text = None
        self.next = 0
        self.generation = 0

    def disconnect(self):
        with self.state_lock:
            self.generation += 1
            self.connected = self.editing = False
            self.token = ''
            self.revision = self.original = self.preview_text = None
            self.next = 0
            self.capture_dropped = None
        return {'connected': False}

    @contextmanager
    def publish(self, generation):
        with self.state_lock:
            if generation != self.generation:
                raise RuntimeError('Session disconnected')
            yield

    def _request(self, method, path, limit, body=b'', revision=None):
        generation = self.generation
        deadline = time.monotonic() + (10 if method == 'PUT' or limit >= 512 * 1024 else 2)
        def remaining():
            if generation != self.generation:
                raise RuntimeError('Session disconnected')
            value = deadline - time.monotonic()
            if value <= 0:
                raise TimeoutError()
            return value
        headers = (f'{method} {path} HTTP/1.1\r\nHost: {self.host}:{self.port}\r\n'
                   f'Connection: close\r\nContent-Length: {len(body)}\r\n')
        if self.token:
            headers += f'X-Kirby-Token: {self.token}\r\n'
        if revision is not None:
            headers += f'X-Kirby-Revision: {revision}\r\n'
        connection = LanDatagramPeer(remaining, limit + 8196)
        with connection as peer:
            peer.settimeout(remaining())
            peer.connect((self.host, self.port))
            peer.settimeout(remaining())
            peer.sendall(headers.encode('ascii') + b'\r\n' + body)
            received = b''
            while b'\r\n\r\n' not in received:
                peer.settimeout(remaining())
                chunk = peer.recv(1024)
                if not chunk:
                    raise ValueError('Incomplete HTTP headers')
                received += chunk
                if len(received.split(b'\r\n\r\n', 1)[0]) > 8192:
                    raise ValueError('Oversized HTTP headers')
            head, payload = received.split(b'\r\n\r\n', 1)
            lines = head.decode('ascii').split('\r\n')
            status = re.fullmatch(r'HTTP/1\.[01] ([0-9]{3})(?: [^\r\n]*)?', lines[0])
            if not status:
                raise ValueError('Invalid HTTP status')
            fields = {}
            for line in lines[1:]:
                key, separator, value = line.partition(':')
                key = key.lower()
                if not separator or key in fields:
                    raise ValueError('Invalid or duplicate HTTP header')
                fields[key] = value.strip()
            length = fields.get('content-length', '')
            if not length.isascii() or not length.isdecimal() or int(length) > limit:
                raise ValueError('Missing or oversized Content-Length')
            if fields.get('connection', '').lower() != 'close' or 'transfer-encoding' in fields:
                raise ValueError('Unsupported HTTP framing')
            length = int(length)
            if len(payload) > length:
                raise ValueError('Unexpected HTTP bytes')
            while len(payload) < length:
                peer.settimeout(remaining())
                chunk = peer.recv(min(8192, length - len(payload)))
                if not chunk:
                    raise ValueError('Incomplete HTTP body')
                payload += chunk
            if peer.recv(1):
                raise ValueError('Unexpected trailing response bytes')
        if generation != self.generation:
            raise RuntimeError('Session disconnected')
        code = int(status[1])
        if code != 200:
            if method == 'PUT' and code not in {403, 409}:
                raise ValueError('Save not acknowledged')
            messages = {403: 'Training access revoked or editor closed', 409: 'Stale revision; reconnect and reload before editing'}
            raise RuntimeError(messages.get(code, f'HTTP {code}; redirects and retries are disabled'))
        return payload, fields

    def perform(self, operation, **data):
        if operation == 'disconnect':
            return self.disconnect()
        if not self.lock.acquire(blocking=False):
            raise RuntimeError('A LAN request is already running')
        save_started = False
        generation = self.generation
        try:
            if operation == 'connect':
                with self.state_lock:
                    self.disconnect()
                    generation = self.generation
                    self.host, self.port = lan_endpoint(data.get('host'), data.get('port'))
                    token = data.get('token', '')
                    if not isinstance(token, str) or (token and not re.fullmatch(r'[A-Za-z0-9_-]{32,128}', token)):
                        raise ValueError('Token must be 32..128 ASCII letters, digits, hyphens or underscores')
                    transport = data.get('transport', 'udp')
                    if transport != 'udp':
                        raise ValueError('Only UDP transport is supported')
                    self.token, self.transport = token, transport
                payload, _ = self._request('GET', '/status', 8191)
                status = json.loads(payload)
                if not isinstance(status, dict) or type(status.get('protocol')) is not int or status['protocol'] != 1 or status.get('training') is not True or type(status.get('editing')) is not bool:
                    raise ValueError('Unsupported protocol or Training not active')
                with self.publish(generation):
                    self.connected, self.editing = True, status['editing']
                return {'connected': True, 'editing': self.editing}
            if not self.connected:
                raise RuntimeError('Connect explicitly first')
            if operation == 'load':
                payload, headers = self._request('GET', '/sequences', 512 * 1024)
                revision = headers.get('x-kirby-revision', '')
                if not re.fullmatch(r'[0-9a-fA-F]{1,128}', revision):
                    raise ValueError('Missing or invalid sequence revision')
                text = payload.decode('utf-8')
                with self.publish(generation):
                    self.original, self.revision, self.preview_text = text, revision, None
                return {'text': text, 'revision': revision}
            if operation in {'preview', 'save'}:
                text = data.get('text')
                if self.original is None or not isinstance(text, str):
                    raise ValueError('Load the complete file first')
                body = text.encode('utf-8')
                if not body.strip() or len(body) > 512 * 1024:
                    raise ValueError('File must be nonempty and at most 512 KiB')
                if operation == 'preview':
                    try:
                        transfer.validate_sequences(body)
                    except Exception:
                        raise RuntimeError('Local sequence validation failed; disconnected, correct the draft and reconnect') from None
                    with self.publish(generation):
                        if not self.connected:
                            raise RuntimeError('Session disconnected')
                        self.preview_text = text
                        original = self.original
                    diff = ''.join(difflib.unified_diff(original.splitlines(True), text.splitlines(True), fromfile='Switch', tofile='draft'))
                    return {'diff': diff[:64000], 'truncated': len(diff) > 64000}
                if data.get('confirmed') is not True or self.preview_text != text:
                    raise RuntimeError('Preview this exact text and explicitly confirm save')
                save_started = True
                payload, _ = self._request('PUT', '/sequences', 8191, body, self.revision)
                result = json.loads(payload)
                if not isinstance(result, dict) or result.get('saved') is not True or result.get('reloaded') is not True:
                    raise ValueError('Save/reload not acknowledged; reload before retrying')
                with self.publish(generation):
                    self.revision = self.original = self.preview_text = None
                return result
            if operation == 'logs':
                payload, _ = self._request('GET', f'/logs?after={self.next}', 65536)
                result = json.loads(payload)
                def u64(value):
                    return type(value) is int and 0 <= value <= 2**64 - 1
                if not isinstance(result, dict) or not u64(result.get('next')) or not u64(result.get('dropped')):
                    raise ValueError('Invalid log cursor')
                entries = result.get('entries')
                if not isinstance(entries, list) or len(entries) > 64:
                    raise ValueError('Invalid log batch')
                restarted = result['next'] < self.next
                previous = 0 if restarted else self.next
                for entry in entries:
                    if not isinstance(entry, dict) or not u64(entry.get('id')) or not isinstance(entry.get('text'), str) or not previous < entry['id'] <= result['next']:
                        raise ValueError('Invalid log entry')
                    previous = entry['id']
                with self.publish(generation):
                    if self.capture_path is not None and (entries or restarted or result['dropped'] != self.capture_dropped):
                        from workbench_capture import append_record
                        try:
                            append_record(self.capture_path, {
                                'received_at': datetime.now(timezone.utc).isoformat(),
                                'source': f'{self.host}:{self.port}', 'generation': generation,
                                **result, 'restarted': restarted})
                        except OSError as error:
                            raise RuntimeError('Diagnostic capture could not be synced to disk; disconnected without advancing the cursor') from error
                    self.capture_dropped = result['dropped']
                    self.next = result['next']
                return {**result, 'restarted': restarted}
            raise ValueError('Unknown LAN operation')
        except Exception as error:
            self.disconnect()
            # Peer content, draft text and credentials never enter error messages/history.
            if isinstance(error, RuntimeError):
                raise
            if isinstance(error, ConnectionRefusedError):
                reason = 'LAN connection refused; check the address, port, transport and Training diagnostic server'
            elif isinstance(error, TimeoutError):
                reason = 'LAN request timed out; check the address, transport, firewall and Training diagnostic server'
            elif isinstance(error, OSError):
                reason = 'LAN network connection failed; check the address and network connectivity'
            else:
                reason = 'LAN request failed local validation or peer protocol validation'
            if save_started:
                raise RuntimeError(f'{reason}. Save acknowledgment lost or invalid; disk may have changed. Reconnect and reload; never retry automatically') from None
            raise RuntimeError(f'{reason}; disconnected, reconnect manually') from None
        finally:
            self.lock.release()


def lan_endpoint(host, port):
    if not isinstance(host, str):
        raise ValueError('Use the Switch numeric private IPv4 address')
    try:
        address = ipaddress.IPv4Address(host.strip())
    except ipaddress.AddressValueError:
        raise ValueError('Use a numeric private IPv4 address, not a hostname or URL') from None
    if not any(address in ipaddress.IPv4Network(network) for network in ('10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16')):
        raise ValueError('Only RFC1918 private IPv4 addresses are allowed')
    if isinstance(port, bool) or not isinstance(port, (str, int)) or not str(port).isascii() or not str(port).isdecimal() or not 1 <= int(port) <= 65535:
        raise ValueError('LAN port must be an integer from 1 to 65535')
    return str(address), int(port)


class Workbench:
    def __init__(self, history: History):
        self.history = history
        self.preferences_path = history.directory / "preferences.json"
        self.preferences = {}
        self.noninteractive = False
        self.lan = LanClient()
        from workbench_capture import LOG_DIRECTORY
        self.lan.capture_path = LOG_DIRECTORY / 'diagnostic.txt'
        if self.preferences_path.exists():
            try:
                self.preferences = json.loads(self.preferences_path.read_text())
                if not isinstance(self.preferences, dict):
                    raise ValueError("preferences must be an object")
            except (ValueError, OSError) as error:
                print(f"Preferences could not be read; using defaults: {error}")
                self.preferences = {}

    def save_preferences(self):
        temp = self.preferences_path.with_suffix(".tmp")
        with temp.open("w") as handle:
            json.dump(self.preferences, handle, indent=2)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp, self.preferences_path)

    def local_path(self, label, default=None, *, force_default=False):
        default = str(default or DEFAULT_FILE)
        saved = default if force_default else self.preferences.get("sequences", default)
        path = Path(ask(label, saved)).expanduser().resolve()
        self.preferences["sequences"] = str(path)
        self.save_preferences()
        return path

    def port(self, target):
        port = int(self.preferences.get(target + "_port", 5000))
        if not 1 <= port <= 65535:
            raise ValueError("FTP port must be 1..65535")
        return port

    def host(self, target):
        saved = self.preferences.get(target + "_host")
        host = str(saved or next(host for host, name in TARGETS if name == target))
        try:
            address = ipaddress.ip_address(host)
        except ValueError as error:
            raise ValueError(f"{target.title()} address must be a valid IP address") from error
        if address.version != 4:
            raise ValueError("Switch addresses must be IPv4")
        if target == "silent" and host == self.host("diagnostic"):
            raise ValueError("Silent and Diagnostic Switch addresses must be different")
        return host

    def targets(self):
        return tuple((self.host(name), name) for name in ("silent", "diagnostic"))

    def prepare_silent_sequences(self):
        """Download and validate Silent's active file without mutating repository files."""
        endpoint = transfer.Endpoint("silent", transfer.ACTIVE, self.port("silent"), self.host("silent"))
        downloaded = transfer.read(endpoint)
        if downloaded is None:
            raise RuntimeError("Silent active sequences.txt is missing; repository was not changed")
        current = sources.assemble(sources.SOURCE)
        restored = sources.restore_comments(current, downloaded)
        transfer.validate_sequences(restored)
        plan = sources.plan_breakdown(sources.SOURCE, restored)
        plan["origin"] = endpoint.label()
        plan["original"] = current
        plan["original_files"] = {str(path): path.read_bytes() for path in [sources.SOURCE, *sources.SOURCE.parent.joinpath('fighters').glob('*.txt')] if path.is_file()}
        plan["original_fighters"] = {str(path): data for path, data in ((Path(name), value) for name, value in plan["original_files"].items()) if path != sources.SOURCE}
        return {key: value for key, value in plan.items() if key != "original_files"}

    def rollback_silent_sequences(self, plan):
        """Restore split sources and assembled output after an unsuccessful deployment."""
        current = sources.assemble(sources.SOURCE)
        if current == plan["original"]:
            return
        rollback = Path(tempfile.mkdtemp(
            prefix="silent-sync-rollback-", dir=self.history.directory))
        sources.replace_sources(sources.SOURCE, plan["original"], rollback, expected=current)
        for filename, data in plan.get("original_fighters", {}).items():
            path = Path(filename)
            if path.exists() and path.is_file():
                sources.write_generated(data, path)
        for path in sources.SOURCE.parent.joinpath('fighters').glob('*.txt'):
            if str(path) not in plan.get("original_fighters", {}):
                path.unlink()
        sources.write_generated(plan["original"])

    @contextmanager
    def stage_silent_sequences(self, plan):
        """Temporarily expose a reviewed Silent snapshot to build/upload preflight."""
        current = sources.assemble(sources.SOURCE)
        if current != plan["original"]:
            raise RuntimeError("Repository sequence sources changed after Silent preview; nothing overwritten")
        if not plan.get("data"):
            plan["data"] = current
        generated = sources.GENERATED
        previous_generated = generated.read_bytes() if generated.exists() else None
        scratch = Path(tempfile.mkdtemp(prefix="silent-sequence-stage-", dir=self.history.directory))
        sources.replace_sources(sources.SOURCE, plan["data"], scratch / "before")
        try:
            sources.write_generated(plan["data"])
            yield
        finally:
            sources.replace_sources(sources.SOURCE, plan["original"], scratch / "restore")
            for filename, data in plan.get("original_fighters", {}).items():
                path = Path(filename)
                if path.exists():
                    sources.write_generated(data, path)
            for path in sources.SOURCE.parent.joinpath('fighters').glob('*.txt'):
                if str(path) not in plan.get("original_fighters", {}):
                    path.unlink()
            if previous_generated is None:
                generated.unlink(missing_ok=True)
            else:
                sources.write_generated(previous_generated)

    def apply_silent_sequences(self, plan):
        """Apply a reviewed Silent snapshot to split sources and assembled deployment file."""
        current = sources.assemble(sources.SOURCE)
        if current != plan["original"]:
            raise RuntimeError("Repository sequence sources changed after Silent preview; nothing overwritten")
        if current == plan["data"]:
            return {"unchanged": True, "source": plan["origin"], "destination": str(sources.SOURCE)}
        backup_root = self.history.directory / "source-breakdowns"
        backup_root.mkdir(parents=True, exist_ok=True)
        backup = Path(tempfile.mkdtemp(
            prefix=datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S-"), dir=backup_root))
        sources.replace_sources(sources.SOURCE, plan["data"], backup, expected=current)
        try:
            sources.write_generated(plan["data"])
        except BaseException:
            rollback = Path(tempfile.mkdtemp(prefix="silent-sync-rollback-", dir=backup_root))
            sources.replace_sources(sources.SOURCE, current, rollback)
            raise
        print(f"Synchronized Silent active sequences into {plan['fighters']} fighter sources; backup: {backup}")
        return {"source": plan["origin"], "destination": str(sources.SOURCE), "backup": str(backup),
                "fighters": plan["fighters"], "sections": plan["sections"], "bytes": plan["bytes"]}

    def run(self, action, argv, details=None):
        details = {**(details or {}), "command": argv}
        print("\nCommand:", shlex.join(argv))
        return self.history.perform(action, details, lambda: self.execute(argv))

    def execute(self, argv):
        if IS_FROZEN and Path(argv[0]).name == "cargo":
            cargo_bin = Path.home() / ".cargo" / "bin"
            os.environ["PATH"] = os.pathsep.join([str(cargo_bin), os.environ.get("PATH", "")])
            result = subprocess.run(argv, cwd=PROJECT_ROOT, text=True, check=False)
            if result.returncode:
                raise RuntimeError(f"Cargo exited {result.returncode}; inspect the terminal output")
            return {"exit_code": result.returncode}
        return command(argv, interactive=False) if self.noninteractive else command(argv)

    def build(self):
        selected = choose("Build which variants?", ["Both", "Silent", "Diagnostic"])
        if selected is None:
            return False
        variant = ("all", "silent", "diagnostic")[selected]
        cache = ask("Cargo home (Enter = shown value; - = normal cache)", self.preferences.get("cargo_home", "/private/tmp/kirby-assist-build-cargo" if Path("/private/tmp/kirby-assist-build-cargo").is_dir() else ""))
        if cache == "-":
            cache = ""
        self.preferences["cargo_home"] = cache
        self.save_preferences()
        argv = helper("build_variants.py", "--variant", variant)
        if cache:
            argv += ["--cargo-home", str(Path(cache).expanduser())]
        print("\n", shlex.join(argv))
        if not confirm("Build only; this will NOT upload.", "BUILD"):
            return False
        return self.build_release(variant, cache)

    def build_release(self, variant, cache):
        """Prompt-free build shared by GUI and CLI; archive only successful outputs."""
        if variant not in ("all", "silent", "diagnostic"):
            raise ValueError("Invalid build variant selection")
        argv = helper("build_variants.py", "--variant", variant)
        if cache:
            argv += ["--cargo-home", str(Path(cache).expanduser())]
        built_version = version()
        variants = ["silent", "diagnostic"] if variant == "all" else [variant]
        details = {"command": argv, "version": built_version, "variants": variants}
        def operation():
            result = self.execute(argv)
            artifacts = []
            for name in variants:
                folder = upload_switch.selected_source(built_version, name)
                for path in sorted(folder.iterdir()):
                    if path.is_file() and not path.is_symlink():
                        artifacts.append({"variant": name, "source": str(path), **self.history.archive(path.read_bytes())})
            return {**result, "artifacts": artifacts}
        self.history.perform("build", details, operation)
        self.preferences["last_built_version"] = built_version
        self.save_preferences()
        return True

    def upload(self, *, verify=False):
        release = upload_switch.release_version(ask("Built version", self.preferences.get("last_built_version", version())))
        selected = choose("Target", ["Both saved Switches", "Silent only", "Diagnostic only"])
        if selected is None:
            return
        routes = self.targets()
        targets = list(routes) if selected == 0 else [(host, name) for host, name in routes if name == ("silent" if selected == 1 else "diagnostic")]
        message = ("Upload Diagnostic release only. Close Smash and open FTP on Diagnostic; its current sequences are preserved and Silent is not contacted."
                   if selected == 2 else "Silent active sequences.txt becomes the repository source of truth. Close Smash and open FTP on Silent and the selected Switches; selected active files will be replaced after verified backups.")
        if not verify and not confirm(message, "UPLOAD"):
            return False
        return self.upload_release(release, targets, verify=verify)

    def upload_release(self, release, targets, *, verify=False, sequences_synced=False, sequence_plan=None):
        """Fixed route uploads, per-target history and independent failure handling."""
        upload_switch.release_version(release)
        if not targets or any(target not in self.targets() for target in targets):
            raise ValueError("Only the saved Switch/variant routes may be used")
        preserve_sequences = all(name == 'diagnostic' for _, name in targets)
        if not verify and not sequences_synced and not preserve_sequences:
            sequence_plan = self.prepare_silent_sequences()
            with self.stage_silent_sequences(sequence_plan):
                for host, name in targets:
                    preview = helper("upload_switch.py", host, "-v", release,
                                     "-s" if name == "silent" else "-d", "--port", self.port(name),
                                     "--replace-sequences", "--allow-diagnostic-on-silent-ip", "--dry-run")
                    self.run("upload-preview", preview, {"version": release, "target": name, "host": host})
            self.history.perform("silent-sequence-sync", {"source": sequence_plan["origin"], "destination": str(sources.SOURCE)}, lambda: self.apply_silent_sequences(sequence_plan))
            sequences_synced = True
        failures = []
        uploaded = []
        for host, name in targets:
            argv = helper("upload_switch.py", host, "-v", release, "-s" if name == "silent" else "-d", "--port", self.port(name), "--preserve-sequences" if preserve_sequences else "--replace-sequences", "--allow-diagnostic-on-silent-ip")
            if verify:
                argv.append("--verify-only")
            else:
                argv.append("--smash-closed")
            details = {"version": release, "target": name, "host": host, "command": argv}
            def operation(argv=argv, details=details, name=name):
                # Keep local payload snapshots; actual uploaded hashes are read
                # separately from the original helper's output, never guessed.
                details["artifacts"] = [{"remote": "/" + str(path), **self.history.archive(data)}
                    for path, data in upload_switch.upload_payloads(upload_switch.selected_source(release, name), replace_sequences=not preserve_sequences, preserve_sequences=preserve_sequences)]
                return self.execute(argv)
            try:
                self.history.perform("verify" if verify else "upload", details, operation)
                if not verify:
                    uploaded.append(name)
                    if name == "silent" and sequence_plan:
                        sequence_plan["silent_uploaded"] = True
            except (OSError, RuntimeError) as error:
                failures.append(name)
                print(f"{name}: {error}")
                if not verify and name == "silent" and sequence_plan:
                    try:
                        self.rollback_silent_sequences(sequence_plan)
                        sequences_synced = False
                    except (OSError, RuntimeError) as rollback_error:
                        print(f"Repository rollback failed after Silent upload failure: {rollback_error}")
        if failures and not verify and sequences_synced and sequence_plan and not sequence_plan.get("silent_uploaded"):
            try:
                self.rollback_silent_sequences(sequence_plan)
            except (OSError, RuntimeError) as error:
                print(f"Repository rollback failed after upload failure: {error}")
        print("Finished." if not failures else "Failed/unconfirmed targets: " + ", ".join(failures))
        return not failures

    def endpoint(self, label):
        item = choose(label, ["Local computer file", "Silent active/staged sequences", "Diagnostic active/staged sequences"])
        if item is None:
            return None
        if item == 0:
            # Transfers always start from the deployed assembled bundle; a
            # different path can still be entered explicitly at the prompt.
            return transfer.Endpoint("local", str(self.local_path("Local file", DEFAULT_FILE, force_default=True)))
        name = "silent" if item == 1 else "diagnostic"
        file = choose("Which sequence file?", ["ACTIVE sequences.txt (in-game saves)", "STAGED sequences.upload.txt (not active)"])
        if file is None:
            return None
        return transfer.Endpoint(name, transfer.ACTIVE if file == 0 else transfer.STAGED,
                                 self.port(name), self.host(name))

    def sequences(self):
        source = self.endpoint("Transfer FROM")
        if source is None:
            return
        destination = self.endpoint("Transfer TO")
        if destination is None:
            return
        scope = choose("Transfer scope", ["One fighter: replace ALL their recipes/settings", "Whole file: replace every fighter/settings"])
        if scope is None:
            return
        fighter = ask("Fighter ID (e.g. yoshi, kirby, captain_falcon)", self.preferences.get("fighter", "")) if scope == 0 else None
        if scope == 0 and not fighter:
            print("No fighter selected; cancelled.")
            return
        if fighter:
            self.preferences["fighter"] = fighter
            self.save_preferences()
        plans = []
        def preview():
            plan = transfer.prepare(source, destination, fighter)
            plans.append(plan)
            print(f"FROM: {source.label()}\nTO:   {destination.label()}\n{plan.description}\nResult: {len(plan.updated)} bytes; SHA-256 {transfer.digest(plan.updated)}")
            print("Validation uses the current source parser; installed plugin compatibility is not inferred.")
            return plan.details()
        self.history.perform("sequence-preview", {"source": source.label(), "destination": destination.label(), "fighter": fighter}, preview)
        plan = plans[0]
        if not confirm("Replace the previewed destination with verified backups?\nFor any Switch involved, close Smash and leave FTP open first.", "TRANSFER"):
            return
        self.history.perform("sequence-transfer", plan.details(),
            lambda: transfer.commit(plan, self.history.directory / "sequence-backups"))

    def breakdown_plan(self, endpoint, source=None):
        """Read a combined file and describe the fighter files it would produce."""
        source = Path(source or sources.SOURCE)
        if not sources.is_split_source(source):
            raise RuntimeError(
                "Choose the repository deployment/ultimate/kirby_assist/sequences.txt; "
                "its fighters/ directory receives the individual files")
        data = transfer.read(endpoint)
        if data is None:
            raise RuntimeError(f"No sequence file to break down at {endpoint.label()}")
        transfer.validate_sequences(data)
        if source.resolve() == sources.SOURCE.resolve():
            data = sources.restore_comments(sources.assemble(source), data)
            transfer.validate_sequences(data)
        plan = sources.plan_breakdown(source, data)
        plan["origin"] = endpoint.label()
        return plan

    def breakdown_commit(self, plan):
        """Replace the split sources from a reviewed plan, with backups and readback."""
        source = Path(plan["source"])
        backup_root = self.history.directory / "source-breakdowns"
        backup_root.mkdir(parents=True, exist_ok=True)
        backup = Path(tempfile.mkdtemp(
            prefix=datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S-"), dir=backup_root))
        sources.replace_sources(source, plan["data"], backup)
        written = len(plan["created"]) + len(plan["updated"])
        print(f"Wrote {written} source files; {len(plan['removed'])} removed. Backup: {backup}")
        return {"source": str(source), "origin": plan.get("origin"), "backup": str(backup),
                "created": [e["fighter"] for e in plan["created"]],
                "updated": [e["fighter"] for e in plan["updated"]],
                "removed": [e["fighter"] for e in plan["removed"]],
                "sections": plan["sections"], "bytes": plan["bytes"]}

    def breakdown(self):
        """Terminal flow: pick a combined file, review the split, then confirm."""
        endpoint = self.endpoint("Break down WHICH combined sequence file")
        if endpoint is None:
            return
        destination = Path(ask("Repository universal source", str(sources.SOURCE))).expanduser().resolve()
        plans = []
        def preview():
            plan = self.breakdown_plan(endpoint, destination)
            plans.append(plan)
            print("\n" + sources.describe_breakdown(plan))
            return {k: v for k, v in plan.items() if k != "data"}
        self.history.perform("breakdown-preview",
                             {"source": endpoint.label(), "destination": str(destination)}, preview)
        plan = plans[0]
        if not confirm("Replace the repository fighter sources with this breakdown?\n"
                       "Existing files are backed up first.", "BREAKDOWN"):
            return
        self.history.perform("breakdown", {"source": endpoint.label(), "destination": str(destination)},
                             lambda: self.breakdown_commit(plan))

    def edit_fighter(self):
        """Edit one fighter's sections in $EDITOR against a single read snapshot."""
        endpoint = self.endpoint("Edit which sequence file")
        if endpoint is None:
            return
        session = fighter_editor.Session(self.history.directory / "fighter-edits")
        metadata = self.history.perform("fighter-load", {"source": endpoint.label()},
                                        lambda: session.load(endpoint))
        print(f"Loaded {metadata['bytes']} bytes; {len(metadata['fighters'])} fighters: "
              + ", ".join(metadata["fighters"]))
        fighter = ask("Fighter ID to edit", self.preferences.get("fighter", ""))
        if not fighter:
            print("No fighter selected; cancelled.")
            return
        current = session.get(fighter, session.load_id)
        self.preferences["fighter"] = current["fighter"]
        self.save_preferences()
        if not current["sections"]:
            print(f"{current['fighter']} has no sections in this file; new sections may be added.")
        scratch = session.directory / f"{current['fighter']}.edit.txt"
        scratch.parent.mkdir(parents=True, exist_ok=True)
        scratch.write_text(current["text"])
        editor = os.environ.get("VISUAL") or os.environ.get("EDITOR")
        if editor:
            command([*shlex.split(editor), str(scratch)])
        else:
            print(f"\nSet $EDITOR to open this automatically. Edit then save:\n  {scratch}")
            input("Press Enter when the file is saved: ")
        preview = self.history.perform(
            "fighter-preview", {"source": endpoint.label(), "fighter": current["fighter"]},
            lambda: session.prepare_text(current["fighter"], scratch.read_text(),
                                         session.load_id, "cli"))
        print(f"\n{preview['description']}")
        print(preview["diff"] or "No textual change in this fighter.")
        if preview["diff_truncated"]:
            print("(diff truncated)")
        if not confirm(f"Write only {current['fighter']} to {endpoint.label()}?"
                       + ("\nClose Smash on that Switch and leave FTP open first." if session.requires_ready() else ""),
                       "SAVE"):
            session.discard()
            return
        self.history.perform("fighter-save",
            {"destination": endpoint.label(), "fighter": current["fighter"], "sha256": preview["sha256"]},
            lambda: session.commit(preview["plan_id"], confirmed=True, ready=True))

    def show_history(self):
        rows = self.history.rows()
        if not rows:
            print("No workbench operations recorded yet.")
            return
        shipped = {}
        for row in rows:
            details = row.get("details", {})
            if row.get("action") == "upload" and row.get("status") == "success" and row.get("receipt", {}).get("verified_deployment"):
                shipped[details.get("target", "unknown")] = row
        for target, row in shipped.items():
            print(f"Last VERIFIED {target} upload: {row['details'].get('version')} at {row['time']}")
        print("\nRecent operations (started with no completion = unknown/interrupted):")
        for row in rows[-20:]:
            details = row.get("details", {})
            print(f"{row['id']} | {row['action']} | {row['status']} | {details.get('version','')} {details.get('target','')}")
            if details.get("source") or details.get("destination"):
                print(f"  {details.get('source','')} -> {details.get('destination','')} | {details.get('fighter') or 'whole file'}")
            print("  log:", row["log"])
        print("Full history:", self.history.path)
        print("Artifact snapshots:", self.history.directory / "artifacts")

    def advanced(self):
        item = choose("Original helpers (all original options remain available)", list(HELPERS))
        if item is None:
            return
        name = HELPERS[item]
        args = shlex.split(ask("Arguments (use --help to see original options)", "--help"))
        readonly = any(flag in args for flag in ("--help", "-h", "--dry-run", "--verify-only", "--check-only", "--check")) or name == "sequence_validation.py"
        writes = not readonly and (name.startswith("upload") or name == "sync_sequences.py" or any(flag in args for flag in ("--upload", "-upload")))
        imports = not readonly and name == "sequence_sources.py" and any(a.startswith("--import-file") for a in args)
        argv = helper(name, *args)
        print(shlex.join(argv))
        if writes:
            prompt, token = "Confirm Smash CLOSED before any Switch write. Original helper confirmations remain enabled.", "WRITE"
        elif imports:
            prompt, token = "This replaces the repository fighter sources from the given file; backups are kept.", "IMPORT"
        else:
            prompt, token = "Run this original helper?", "RUN"
        if not confirm(prompt, token):
            return
        self.run("advanced", argv)

    def settings(self):
        self.preferences["sequences"] = str(self.local_path("Default local sequences file"))
        self.preferences["cargo_home"] = ask("Default Cargo home (Enter = shown; - = normal)", self.preferences.get("cargo_home", ""))
        if self.preferences["cargo_home"] == "-":
            self.preferences["cargo_home"] = ""
        for name in ("silent", "diagnostic"):
            configured_host = ask(f"{name.title()} Switch IPv4", self.host(name))
            try:
                if ipaddress.ip_address(configured_host).version != 4:
                    raise ValueError
            except ValueError as error:
                raise ValueError("Switch addresses must be valid IPv4 addresses") from error
            self.preferences[name + "_host"] = configured_host
            port = int(ask(f"{name} FTP port", self.port(name)))
            if not 1 <= port <= 65535:
                raise ValueError("Port must be 1..65535")
            self.preferences[name + "_port"] = port
        if self.host("silent") == self.host("diagnostic"):
            raise ValueError("Silent and Diagnostic Switch addresses must be different")
        self.save_preferences()

    def build_then_upload(self):
        if self.build():
            self.upload()

    def lan_editor_state(self):
        return {"available": True, "connected": self.lan.connected, "listening": False,
                "editing": self.lan.editing,
                "status": "Connected" if self.lan.connected else "Disconnected; connect manually",
                "host": self.preferences.get("lan_editor_host", ""),
                "port": self.preferences.get("lan_editor_port", 4242)}

    def configure_lan_editor(self, host, port):
        """Persist only the numeric Switch endpoint; never the shared secret."""
        host, port = lan_endpoint(host, port)
        previous = dict(self.preferences)
        self.preferences.update(lan_editor_host=host, lan_editor_port=port)
        try:
            self.save_preferences()
        except OSError:
            self.preferences = previous
            raise
        return self.lan_editor_state()

    def lan_editor(self):
        state = self.lan_editor_state()
        print("TRAINING LAN: activate Diagnostic server in Training; edits require editor open.")
        host = ask("Switch private IPv4 (empty cancels)", state["host"])
        if not host:
            return
        port = ask("Diagnostic LAN port (not FTP)", state["port"])
        self.configure_lan_editor(host, port)
        try:
            print(self.lan.perform('connect', host=host, port=port, transport='udp'))
            while self.lan.connected:
                action = choose("LAN actions", ["Read complete file to computer", "Preview and save computer file", "Read new logs", "Disconnect"])
                if action is None or action == 3:
                    return
                if action == 0:
                    result = self.lan.perform('load')
                    path = Path(ask("Output file")).expanduser()
                    with path.open('x', encoding='utf-8', newline='') as output:
                        output.write(result['text'])
                    print("Loaded revision", result['revision'])
                elif action == 1:
                    path = Path(ask("Edited complete file")).expanduser()
                    with path.open('rb') as source:
                        body = source.read(512 * 1024 + 1)
                    if len(body) > 512 * 1024:
                        raise ValueError("File exceeds 512 KiB")
                    text = body.decode('utf-8')
                    print(self.lan.perform('preview', text=text)['diff'])
                    if confirm("Save and reload on Switch?", "SAVE"):
                        print(self.lan.perform('save', text=text, confirmed=True))
                elif action == 2:
                    result = self.lan.perform('logs')
                    print("Queue dropped:", result['dropped'])
                    for entry in result['entries']:
                        print(entry['id'], json.dumps(entry['text'], ensure_ascii=True))
        finally:
            self.lan.disconnect()

    def validate(self):
        self.run("validate", helper("sequence_validation.py", self.local_path("File to validate")))

    def menu(self):
        """Named menu so adding an operation cannot renumber the others."""
        return (
            ("Ship: build, then review/upload", self.build_then_upload),
            ("Build only", self.build),
            ("Upload a built release", self.upload),
            ("Verify installed release (read-only)", lambda: self.upload(verify=True)),
            ("Edit one fighter's sequences", self.edit_fighter),
            ("Break down a combined file into fighter sources", self.breakdown),
            ("Transfer / replace sequences (computer / Switches)", self.sequences),
            ("Validate a sequences file", self.validate),
            ("History / shipped versions / file logs", self.show_history),
            ("Preferences", self.settings),
            ("Original helpers / advanced options", self.advanced),
            ("Training LAN editor / live logs", self.lan_editor),
        )

    def loop(self):
        while True:
            try:
                print(f"\nKIRBY ASSIST WORKBENCH | source {version()}")
                entries = self.menu()
                item = choose("Choose an operation", [label for label, _ in entries], back_label="Exit")
                if item is None:
                    return
                entries[item][1]()
            except EOFError:
                return
            except KeyboardInterrupt:
                print("\nCancelled. Check the operation log if a transfer/upload had started.")
            except (OSError, RuntimeError, ValueError, ftplib.Error, argparse.ArgumentTypeError) as error:
                print(f"\nOperation stopped: {error}\nThe workbench remains open; inspect history before retrying writes.")


@contextmanager
def session_lock(directory):
    directory.mkdir(parents=True, exist_ok=True)
    with (directory / "session.lock").open("a") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise RuntimeError("Another workbench is already running with this history directory")
        yield


def main():
    global PROJECT_ROOT, ROOT, STATE, DEFAULT_FILE
    if getattr(sys, "frozen", False) and len(sys.argv) > 2 and sys.argv[1] == "--workbench-helper":
        import runpy
        module = {"build_variants": "build_variants", "upload_all_switches": "upload_all_switches",
                  "upload_switch": "upload_switch", "upload_sequences": "upload_sequences",
                  "sync_sequences": "sync_sequences", "sequence_validation": "sequence_validation",
                  "sequence_sources": "sequence_sources"}.get(sys.argv[2])
        if module is None:
            raise SystemExit("Unknown bundled helper")
        project_root = os.environ.get("KIRBY_ASSIST_FROZEN_ROOT", "")
        if not project_root:
            project_file = Path.home() / "Library/Application Support/Kirby Assist Workbench/project-root.txt"
            if project_file.is_file():
                project_root = project_file.read_text().strip()
        if project_root:
            os.environ["KIRBY_ASSIST_PROJECT_ROOT"] = project_root
        sys.argv = [module, *sys.argv[3:]]
        runpy.run_module(module, run_name="__main__")
        return
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--state-dir", type=Path, default=STATE, help="local history/preferences/backups directory")
    parser.add_argument("--history", action="store_true", help="show history and exit; no build/FTP")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--cli", action="store_true", help="original numbered terminal prompt")
    mode.add_argument("--gui", "--web", dest="gui", action="store_true", help="launch the browser GUI directly")
    mode.add_argument("--terminal", action="store_true", help="launch the Textual terminal interface")
    mode.add_argument("--choose", action="store_true", help="choose Web or Terminal in a native macOS window")
    parser.add_argument("--terminal-child", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--check-terminal-bundle", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--project", type=Path, help="UltimateKirby project checkout to operate on")
    args = parser.parse_args()
    if args.terminal_child and (not args.terminal or args.history):
        parser.error('--terminal-child requires --terminal and cannot be used with --history')
    try:
        from workbench_reminal_ui import launcher
        if args.check_terminal_bundle:
            launcher.check_terminal_bundle()
            return
        interface = 'history' if args.history else 'cli' if args.cli else 'web' if args.gui else 'terminal' if args.terminal else None
        if interface is None:
            interface = launcher.choose_interface() if args.choose or sys.platform == 'darwin' else 'web'
            if interface is None:
                return
        if args.terminal_child:
            launcher.attach_terminal()
        configured_state = Path(os.environ.get("KIRBY_ASSIST_STATE_DIR", STATE)).expanduser().resolve()
        state_dir = args.state_dir.expanduser().resolve() if args.state_dir.expanduser().resolve() != STATE else configured_state
        project_config = state_dir / "project-root.txt"
        saved_project = project_config.read_text().strip() if IS_FROZEN and project_config.is_file() else ""
        project = args.project.expanduser().resolve() if args.project else Path(saved_project or os.environ.get("KIRBY_ASSIST_PROJECT_ROOT", PROJECT_ROOT)).expanduser().resolve()
        if IS_FROZEN and not args.project and (not saved_project or not (project / "Cargo.toml").is_file()):
            selection = subprocess.run(
                ["osascript", "-e", 'POSIX path of (choose folder with prompt "Select your UltimateKirby project folder")'],
                capture_output=True, text=True, check=False)
            if selection.returncode:
                raise RuntimeError("Project folder selection was cancelled")
            project = Path(selection.stdout.strip()).expanduser().resolve()
        manifest = project / "Cargo.toml"
        if not manifest.is_file() or not (project / "deployment/ultimate/kirby_assist/sequences.txt").is_file():
            raise RuntimeError("Select the root of a valid UltimateKirby project checkout (Cargo.toml and the deployment sequence source are required)")
        if not (project / "tools" / "build_variants.py").is_file() and not (TOOLS_DIR / "build_variants.py").is_file():
            raise RuntimeError("No build helper is available for the selected project")
        if state_dir.is_relative_to(project) and ".mod-workbench" not in state_dir.relative_to(project).parts:
            raise RuntimeError("Keep history in .mod-workbench or outside the project so build source archives cannot include it")
        if interface == 'terminal' and not launcher.has_terminal():
            launcher.open_terminal(project, state_dir)
            return
        if IS_FROZEN:
            project_config.parent.mkdir(parents=True, exist_ok=True)
            project_config.write_text(str(project) + "\n")
            os.environ["KIRBY_ASSIST_PROJECT_ROOT"] = str(project)
            os.environ["KIRBY_ASSIST_FROZEN_ROOT"] = str(project)
        PROJECT_ROOT = ROOT = project
        DEFAULT_FILE = project / "deployment/ultimate/kirby_assist/assembled/sequences.txt"
        workbench_history.ROOT = project
        workbench_history.STATE = state_dir
        workbench_history.TOOLS_DIR = TOOLS_DIR
        sequence_sources.ROOT = project
        sequence_sources.SOURCE = project / "deployment/ultimate/kirby_assist/sequences.txt"
        sequence_sources.GENERATED = sequence_sources.SOURCE.parent / "assembled/sequences.txt"
        sources.ROOT = project
        sources.SOURCE = sequence_sources.SOURCE
        sources.GENERATED = sequence_sources.GENERATED
        upload_switch.PROJECT_ROOT = project
        upload_switch.DEFAULT_RELEASE_ROOT = project.parent
        upload_switch.DEPLOYMENT_SEQUENCES = sequence_sources.SOURCE
        import build_variants
        build_variants.ROOT = project
        build_variants.RELEASE_VERSION = version()
        build_variants.RELEASE_SUMMARY_NAME = f"RELEASE_SUMMARY-{build_variants.RELEASE_VERSION}.txt"
        with session_lock(state_dir):
            app = Workbench(History(state_dir))
            if interface == 'history':
                app.show_history()
            elif interface == 'cli':
                app.loop()
            elif interface == 'terminal':
                launcher.launch_terminal(app)
            else:
                from workbench_gui import launch
                launch(app)
    except (OSError, RuntimeError) as error:
        if IS_FROZEN and not args.check_terminal_bundle:
            message = str(error).replace("\\", "\\\\").replace('"', '\\"')
            subprocess.run(["osascript", "-e", f'display dialog "{message}" with title "Kirby Assist Workbench" buttons {{"OK"}} default button "OK"'], check=False)
            return
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
