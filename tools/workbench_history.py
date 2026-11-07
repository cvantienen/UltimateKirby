"""Host-only operation journal and content-addressed artifact snapshots."""
from __future__ import annotations

import codecs
import contextlib
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / ".mod-workbench"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class Tee:
    def __init__(self, terminal, log):
        self.terminal, self.log = terminal, log

    def write(self, text):
        self.log.write(text)
        self.log.flush()
        return self.terminal.write(text)

    def flush(self):
        self.terminal.flush()
        self.log.flush()


def receipts(path: Path) -> dict:
    manifests, verified = {}, []
    complete = False
    with path.open(errors="replace") as log:
        for line in log:
            match = re.match(r"\s+(/.+) \(([\d,]+) bytes; SHA-256 ([0-9a-f]{64})\)", line)
            if match:
                manifests[match[1]] = {"path": match[1], "bytes": int(match[2].replace(",", "")), "sha256": match[3]}
            if line.startswith("Uploaded and verified "):
                verified.append(line.strip().removeprefix("Uploaded and verified "))
            if line.startswith("FTP update complete: all "):
                complete = True
    return {"payloads": list(manifests.values()), "verified_files": verified,
            "verified_deployment": complete}


class History:
    def __init__(self, directory: Path = STATE):
        self.directory = directory
        self.path = directory / "history.jsonl"
        (directory / "logs").mkdir(parents=True, exist_ok=True)

    def append(self, event: dict):
        with self.path.open("a") as handle:
            handle.write(json.dumps(event, ensure_ascii=True) + "\n")
            handle.flush()
            os.fsync(handle.fileno())

    def archive(self, data: bytes) -> dict:
        sha = digest(data)
        folder = self.directory / "artifacts"
        folder.mkdir(exist_ok=True)
        path = folder / sha
        if path.exists():
            if digest(path.read_bytes()) != sha:
                raise RuntimeError(f"Stored artifact is corrupt: {path}")
        else:
            with path.open("xb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
        return {"sha256": sha, "bytes": len(data), "archive": str(path)}

    def perform(self, action: str, details: dict, operation):
        token = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S-") + uuid.uuid4().hex[:10]
        log = self.directory / "logs" / (token + ".log")
        base = {"id": token, "action": action, "time": datetime.now(timezone.utc).isoformat(),
                "details": details, "log": str(log)}
        # Fail before any operation if the journal/log cannot be written.
        with log.open("x") as handle:
            self.append({**base, "status": "started"})
            try:
                with contextlib.redirect_stdout(Tee(sys.stdout, handle)), contextlib.redirect_stderr(Tee(sys.stderr, handle)):
                    result = operation() or {}
            except BaseException as error:
                handle.write(f"\n{type(error).__name__}: {error}\n")
                handle.flush()
                self.append({**base, "status": "interrupted" if isinstance(error, KeyboardInterrupt) else "failed",
                             "error": str(error), "receipt": receipts(log)})
                raise
        receipt = receipts(log)
        archived = {a["sha256"]: a["archive"] for a in details.get("artifacts", [])}
        for payload in receipt["payloads"]:
            payload["archive"] = archived.get(payload["sha256"])
        self.append({**base, "status": "success", "result": result, "receipt": receipt})
        print(f"History/log: {log}")
        return result

    def rows(self) -> list[dict]:
        rows = {}
        if self.path.exists():
            for number, line in enumerate(self.path.read_text().splitlines(), 1):
                try:
                    event = json.loads(line)
                    rows[event["id"]] = event
                except (ValueError, KeyError, TypeError):
                    print(f"History line {number} is incomplete/unreadable; retained logs may help.")
        return list(rows.values())


def command(argv: list[str], *, interactive: bool = True) -> dict:
    """Keep CLI confirmations; GUI closes stdin to avoid invisible prompts. No shell."""
    child = subprocess.Popen(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                             stdin=None if interactive else subprocess.DEVNULL)
    decoder = codecs.getincrementaldecoder("utf-8")("replace")
    try:
        assert child.stdout is not None
        while block := child.stdout.read1(8192):
            sys.stdout.write(decoder.decode(block))
            sys.stdout.flush()
        sys.stdout.write(decoder.decode(b"", final=True))
        code = child.wait()
    except BaseException:
        child.terminate()
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
        raise
    finally:
        if child.stdout is not None:
            child.stdout.close()
    if code:
        raise RuntimeError(f"Helper exited {code}; inspect the operation log before retrying.")
    return {"exit_code": code}
