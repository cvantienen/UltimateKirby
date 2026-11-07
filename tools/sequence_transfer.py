"""Prepared, validated local/Switch sequence transfers. Existing sync helpers stay unchanged."""
from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
import ftplib
import os
from pathlib import Path, PurePosixPath
import stat
import tempfile

from sequence_validation import MAX_BYTES, validate_sequences
from sequence_sources import assemble, is_split_source, canonical_bundle, replace_sources
from sync_sequences import ACTIVE, replace_fighter, store_verified
from upload_all_switches import TARGETS
from workbench_history import digest

STAGED = "/ultimate/kirby_assist/sequences.upload.txt"


@dataclass(frozen=True)
class Endpoint:
    kind: str
    path: str
    port: int = 5000

    def __post_init__(self):
        if self.kind not in ("local", "silent", "diagnostic"):
            raise ValueError("Endpoint must be local, silent or diagnostic")
        if self.kind != "local" and self.path not in (ACTIVE, STAGED):
            raise ValueError("Only active/staged sequence files may be transferred")
        if not 1 <= self.port <= 65535:
            raise ValueError("FTP port must be 1..65535")
        if self.kind == "local":
            object.__setattr__(self, "path", str(Path(self.path).expanduser().resolve()))

    @property
    def host(self) -> str:
        return next(host for host, variant in TARGETS if variant == self.kind)

    def label(self) -> str:
        return self.path if self.kind == "local" else f"{self.kind} {self.host}:{self.port}{self.path}"


def connect(endpoint: Endpoint):
    ftp = ftplib.FTP()
    try:
        ftp.connect(endpoint.host, endpoint.port, timeout=15)
        ftp.login()
        return ftp
    except BaseException:
        ftp.close()
        raise


def remote_read(ftp, path: str) -> bytes | None:
    data = bytearray()
    def receive(block):
        if len(data) + len(block) > MAX_BYTES:
            raise RuntimeError("Remote sequences file exceeds 512 KiB")
        data.extend(block)
    try:
        ftp.retrbinary("RETR " + path, receive)
    except (ftplib.error_perm, ftplib.error_temp) as error:
        if str(error)[:3] not in ("450", "550"):
            raise
        names = ftp.nlst(str(PurePosixPath(path).parent))
        if any(PurePosixPath(n.rstrip("/")).name == PurePosixPath(path).name for n in names):
            raise RuntimeError(f"Existing destination cannot be read: {path}") from error
        return None
    return bytes(data)


def read(endpoint: Endpoint) -> bytes | None:
    if endpoint.kind != "local":
        with connect(endpoint) as ftp:
            return remote_read(ftp, endpoint.path)
    path = Path(endpoint.path)
    try:
        if is_split_source(path):
            return assemble(path)
        with path.open("rb") as handle:
            data = handle.read(MAX_BYTES + 1)
    except FileNotFoundError:
        return None
    if len(data) > MAX_BYTES:
        raise RuntimeError("Local sequence file exceeds 512 KiB")
    return data


@dataclass(frozen=True)
class Plan:
    source: Endpoint
    destination: Endpoint
    fighter: str | None
    source_data: bytes
    original: bytes | None
    updated: bytes
    description: str

    def details(self) -> dict:
        return {"source": self.source.label(), "destination": self.destination.label(),
                "fighter": self.fighter, "sha256": digest(self.updated), "bytes": len(self.updated),
                "original_sha256": digest(self.original) if self.original is not None else None}


def prepare(source: Endpoint, destination: Endpoint, fighter: str | None = None) -> Plan:
    if source == destination:
        raise RuntimeError("Source and destination are identical")
    data = read(source)
    if data is None:
        raise RuntimeError("Source file is missing")
    validate_sequences(data)
    original = read(destination)
    if fighter:
        if original is None:
            raise RuntimeError("Create a whole destination file first; fighter merge requires an existing file")
        validate_sequences(original)
        updated, added, removed = replace_fighter(data, original, fighter)
        description = f"Replace {removed} sections for {fighter} with {added} source sections; other fighters preserved"
    else:
        updated = data
        description = "Replace the WHOLE file (all fighters/settings)"
    if destination.kind=="local" and is_split_source(Path(destination.path)):
        updated=canonical_bundle(updated)
    validate_sequences(updated)
    return Plan(source, destination, fighter, data, original, updated, description)


def commit(plan: Plan, backup_root: Path) -> dict:
    if plan.destination.kind=="local" and is_split_source(Path(plan.destination.path)) and canonical_bundle(plan.updated)!=plan.updated:
        raise RuntimeError("Split-source destination needs a fresh canonical transfer preview; nothing overwritten")
    # Recheck both sides after the user reviews the immutable preview.
    if read(plan.source) != plan.source_data:
        raise RuntimeError("Source changed after preview; prepare again")
    if read(plan.destination) != plan.original:
        raise RuntimeError("Destination changed after preview; nothing overwritten")
    if plan.original == plan.updated:
        print("Destination already matches; no write needed.")
        return {"unchanged": True, **plan.details()}
    backup_root.mkdir(parents=True, exist_ok=True)
    snapshot = Path(tempfile.mkdtemp(prefix=datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S-"), dir=backup_root))
    for name, data in (("source.txt", plan.source_data), ("destination-before.txt", plan.original), ("result.txt", plan.updated)):
        if data is not None:
            with (snapshot / name).open("xb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
    print(f"Recovery snapshots: {snapshot}")
    endpoint = plan.destination
    if endpoint.kind == "local" and is_split_source(Path(endpoint.path)):
        replace_sources(Path(endpoint.path),plan.updated,snapshot/'sources',expected=plan.original)
    elif endpoint.kind == "local":
        path = Path(endpoint.path)
        descriptor, temp_name = tempfile.mkstemp(prefix=path.name + ".workbench-", dir=path.parent)
        temporary = Path(temp_name)
        try:
            with os.fdopen(descriptor, "wb") as handle:
                handle.write(plan.updated)
                handle.flush()
                os.fsync(handle.fileno())
            if path.exists():
                temporary.chmod(stat.S_IMODE(path.stat().st_mode))
            if read(endpoint) != plan.original:
                raise RuntimeError("Local destination changed before replacement")
            os.replace(temporary, path)
            if read(endpoint) != plan.updated:
                raise RuntimeError(f"Local verification failed; recovery copy: {snapshot}")
        finally:
            temporary.unlink(missing_ok=True)
    else:
        with connect(endpoint) as ftp:
            if remote_read(ftp, endpoint.path) != plan.original:
                raise RuntimeError("Switch file changed before backup; nothing overwritten")
            backup = endpoint.path + ".workbench.bak"
            if plan.original is not None:
                store_verified(ftp, backup, plan.original)
            if remote_read(ftp, endpoint.path) != plan.original:
                raise RuntimeError("Switch file changed during backup; active write cancelled")
            try:
                store_verified(ftp, endpoint.path, plan.updated)
            except (OSError, EOFError, ftplib.Error, RuntimeError, KeyboardInterrupt) as error:
                if plan.original is not None:
                    try:
                        store_verified(ftp, endpoint.path, plan.original)
                    except (OSError, EOFError, ftplib.Error, RuntimeError) as restore:
                        raise RuntimeError(f"Transfer and rollback failed; destination may be incomplete. Restore {snapshot / 'destination-before.txt'}. {restore}") from error
                    raise RuntimeError(f"Transfer failed; original restored and verified. {error}") from error
                raise RuntimeError(f"Transfer failed; newly created file may be incomplete. Source snapshot: {snapshot}") from error
    print("Transfer complete; destination bytes verified.")
    return {"verified": True, "backup": str(snapshot), **plan.details()}
