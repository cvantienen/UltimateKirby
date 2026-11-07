"""Single-fighter text editing against a read-only Switch snapshot.

Unselected fighter/universal spans are preserved verbatim. Saving uses the
existing validated transfer plan, backup, concurrent-edit check and readback.
This module never writes to FTP itself.
"""
from __future__ import annotations

from datetime import datetime, timezone
import difflib
from pathlib import Path
import re
import secrets

import sequence_transfer as transfer
from sequence_validation import MAX_BYTES, validate_sequences
from sync_sequences import fighter_id, sections

HEADER = re.compile(rb'^\s*\[([^\]\r\n]+)\][ \t]*(?:#[^\r\n]*)?\r?$', re.M)


def index(data: bytes):
    data.decode('utf-8')  # Reject undecodable files rather than rewriting bytes.
    result = []
    for owner, start, end in sections(data):
        matches = list(HEADER.finditer(data[start:end]))
        if len(matches) != 1:
            raise ValueError('Cannot identify sequence section boundaries safely')
        result.append((owner, matches[0][1].decode(), start, end))
    return result


def fighter_sections(data: bytes, fighter: str):
    fighter = fighter_id(fighter)
    return [{'id': name, 'text': data[start:end].decode()}
            for owner, name, start, end in index(data) if owner == fighter]


def merge(data: bytes, fighter: str, drafts: list):
    fighter = fighter_id(fighter)
    if not isinstance(drafts, list) or len(drafts) > 512:
        raise ValueError('Provide at most 512 sections for one fighter')
    replacement = []
    seen = set()
    for draft in drafts:
        if not isinstance(draft, dict) or not isinstance(draft.get('text'), str):
            raise ValueError('Every draft must contain section text')
        raw = draft['text'].encode()
        if len(raw) > MAX_BYTES:
            raise ValueError('Section exceeds sequence file size limit')
        spans = index(raw)
        if len(spans) != 1 or spans[0][0] != fighter:
            raise ValueError(f'Each section must belong only to {fighter}')
        name = spans[0][1]
        if name in seen:
            raise ValueError('Duplicate section: ' + name)
        # Allow comments/blank lines before the header, never orphaned fields.
        prefix = raw[:HEADER.search(raw).start()]
        if any(line.strip() and not line.lstrip().startswith(b'#') for line in prefix.splitlines()):
            raise ValueError('Fields must appear under a section header')
        seen.add(name)
        replacement.append(raw if raw.endswith(b'\n') else raw + b'\n')
    spans = index(data)
    old = [(name, data[start:end]) for owner, name, start, end in spans if owner == fighter]
    # Preserve even a missing final newline for a completely unedited fighter.
    if [draft['text'].encode() for draft in drafts] == [raw for _, raw in old]:
        return data, {'added': [], 'removed': [], 'changed': []}
    body = b''.join(replacement)
    output = bytearray()
    cursor, inserted = 0, False
    for owner, _, start, end in spans:
        if owner != fighter:
            continue
        output.extend(data[cursor:start])
        if not inserted:
            output.extend(body)
            inserted = True
        cursor = end
    output.extend(data[cursor:])
    if not inserted and body:
        if output and not output.endswith(b'\n'):
            output.extend(b'\n')
        output.extend(body)
    updated = bytes(output)
    if len(updated) > MAX_BYTES:
        raise ValueError('Result exceeds sequence file size limit')
    before = dict(old)
    after = {name: updated[start:end] for owner, name, start, end in index(updated) if owner == fighter}
    # The preservation contract is checked independently of the replacement loop.
    other_before = [(name, data[start:end]) for owner, name, start, end in spans if owner != fighter]
    other_after = [(name, updated[start:end]) for owner, name, start, end in index(updated) if owner != fighter]
    if other_before != other_after:
        raise RuntimeError('Another fighter section would change; edit cancelled')
    return updated, {'added': [n for n in after if n not in before],
                     'removed': [n for n in before if n not in after],
                     'changed': [n for n in before if n in after and before[n] != after[n]]}


class Session:
    def __init__(self, directory: Path):
        self.directory = directory
        self.original = None
        self.destination = None
        self.load_id = None
        self.loaded_at = None
        self.pending = None
        self.preview = None

    def loaded(self, data: bytes, destination):
        if len(data) > MAX_BYTES:
            raise ValueError('Sequence file exceeds size limit')
        index(data)
        self.original, self.destination = data, destination
        self.load_id = secrets.token_hex(16)
        self.loaded_at = datetime.now(timezone.utc).isoformat()
        self.pending = self.preview = None

    def metadata(self):
        if self.original is None:
            return None
        return {'load_id': self.load_id, 'loaded_at': self.loaded_at,
                'sha256': transfer.digest(self.original), 'bytes': len(self.original),
                'source': self.destination.label(),
                'fighters': sorted({owner for owner, _, _, _ in index(self.original) if owner != 'universal'}),
                'preview': self.preview}

    def get(self, fighter, load_id):
        if self.original is None or load_id != self.load_id:
            raise ValueError('Read the current file from silent first')
        fighter = fighter_id(fighter)
        return {'load_id': self.load_id, 'fighter': fighter,
                'sections': fighter_sections(self.original, fighter)}

    def prepare(self, fighter, drafts, load_id, request_id):
        self.pending = self.preview = None
        current = self.get(fighter, load_id)
        fighter = current['fighter']
        updated, changes = merge(self.original, fighter, drafts)
        if self.destination.kind=='local' and transfer.is_split_source(Path(self.destination.path)):
            updated=transfer.canonical_bundle(updated)
        validate_sequences(updated)
        folder = self.directory / secrets.token_hex(16)
        folder.mkdir(parents=True)
        source = folder / 'validated.txt'
        source.write_bytes(updated)
        plan = transfer.Plan(transfer.Endpoint('local', str(source)), self.destination, fighter,
            updated, self.original, updated,
            f"Update only {fighter}: {len(changes['added'])} added, {len(changes['changed'])} changed, {len(changes['removed'])} deleted sections. Other fighters/universal settings preserved.")
        before = ''.join(d['text'] for d in current['sections'])
        after = ''.join(d['text'] for d in fighter_sections(updated, fighter))
        diff = ''.join(difflib.unified_diff(before.splitlines(True), after.splitlines(True), fromfile=f'{fighter} / loaded', tofile=f'{fighter} / proposed'))
        token = secrets.token_hex(16)
        self.pending = (token, plan)
        self.preview = {'plan_id': token, 'load_id': self.load_id, 'request_id': request_id,
                        'fighter': fighter, 'description': plan.description, 'changes': changes,
                        'diff': diff[:24000], 'diff_truncated': len(diff) > 24000,
                        'sha256': transfer.digest(updated), 'destination': self.destination.label(),
                        'delete_fighter': bool(current['sections']) and not drafts}
        return self.preview

    def commit(self, plan_id, *, confirmed=False, ready=False):
        if not confirmed or not ready:
            raise ValueError('Confirm the fighter replacement with Smash closed and FTP open')
        if not self.pending or plan_id != self.pending[0]:
            raise ValueError('Preview the fighter edit first')
        plan = self.pending[1]
        result = transfer.commit(plan, self.directory / 'backups')
        self.loaded(plan.updated, plan.destination)
        return result
