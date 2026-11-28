"""Host-side fighter source assembly. No game-time includes or file discovery."""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import tempfile
import os

ROOT = Path(os.environ.get('KIRBY_ASSIST_PROJECT_ROOT', Path(__file__).resolve().parents[1])).expanduser().resolve()
SOURCE = ROOT / 'deployment/ultimate/kirby_assist/sequences.txt'
GENERATED = SOURCE.parent / 'assembled/sequences.txt'
MAX_BYTES = 512 * 1024
ID = re.compile(r'[a-z][a-z0-9_]*\Z')


def split_bundle(data: bytes) -> dict[str, bytes]:
    """Move complete sections, preserving their properties and order verbatim."""
    data.decode('utf-8')
    starts, offset, decoration = [], 0, None
    for line in data.splitlines(keepends=True):
        stripped = line.strip()
        if stripped == b'# ========================================================================' or (stripped.startswith(b'# ---- ') and stripped.endswith(b' ----')):
            if decoration is None:
                decoration = offset
        header = line.split(b'#', 1)[0].strip()
        if header.startswith(b'[') and header.endswith(b']'):
            owner = header[1:-1].split(b'.', 1)[0].decode().strip()
            if not ID.fullmatch(owner):
                raise RuntimeError(f'Invalid section owner: {owner}')
            starts.append(('universal' if owner in ('universal', 'template') else owner,
                           decoration if decoration is not None else offset))
            decoration = None
        offset += len(line)
    if not starts:
        raise RuntimeError('No configuration sections found')
    result = {}
    for i, (owner, start) in enumerate(starts):
        end = starts[i + 1][1] if i + 1 < len(starts) else len(data)
        if i == 0:
            start = 0
        result[owner] = result.get(owner, b'') + data[start:end]
    return result


def is_split_source(path: Path) -> bool:
    return path.name == 'sequences.txt' and (path.parent / 'fighters').is_dir()


def assemble(path: Path = SOURCE) -> bytes:
    """Read universal settings/templates plus sorted, strictly owned fighter files."""
    path = Path(path)
    data = path.read_bytes()
    if len(data)>MAX_BYTES:
        raise RuntimeError('Sequence file exceeds 512 KiB')
    if not is_split_source(path):
        return data
    if set(split_bundle(data)) != {'universal'}:
        raise RuntimeError('sequences.txt must contain only universal settings and templates; move fighter sections into fighters/<name>.txt')
    pieces = [data.rstrip() + b'\n\n']
    total=len(pieces[0])
    seen = set()
    for file in sorted((path.parent / 'fighters').glob('*.txt')):
        owner = file.stem
        if file.is_symlink() or not ID.fullmatch(owner) or owner in ('universal', 'template'):
            raise RuntimeError(f'Invalid fighter source: {file.name}')
        raw = file.read_bytes()
        if len(raw) > MAX_BYTES or set(split_bundle(raw)) != {owner}:
            raise RuntimeError(f'{file.name} must contain only [{owner}.*] sections')
        pieces.append(raw.rstrip() + b'\n\n')
        total+=len(pieces[-1])
    output = b''.join(pieces).rstrip() + b'\n'
    # The 512 KiB cap applies to the bytes the Switch receives. Repo-only
    # comments are stripped before upload, so they must not consume that budget.
    if len(strip_comments(output)) > MAX_BYTES:
        raise RuntimeError('Assembled sequence file exceeds 512 KiB')
    for line in output.decode().splitlines():
        header = line.split('#', 1)[0].strip()
        if header.startswith('[') and header.endswith(']'):
            if header in seen:
                raise RuntimeError(f'Duplicate section {header}')
            seen.add(header)
    return output


def write_generated(data: bytes, output: Path = GENERATED) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(dir=output.parent, prefix=output.name + '.')
    try:
        with os.fdopen(fd, 'wb') as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(name, output)
    finally:
        Path(name).unlink(missing_ok=True)


def strip_comments(data: bytes) -> bytes:
    """Drop repo-only comments/blank lines for the Switch copy.

    The mod's parser already ignores everything from '#' onward, so removing
    them changes no option, value or ordering. Editable sources keep their
    documentation; only the uploaded bytes shrink.
    """
    text = data.decode('utf-8')
    out = []
    for line in text.splitlines():
        if not line.strip():
            # Keep separation that already existed, collapsed to one blank line.
            if out and out[-1]:
                out.append('')
            continue
        kept = line.split('#', 1)[0].rstrip()
        if kept:
            out.append(kept)
    while out and not out[-1]:
        out.pop()
    result = '\n'.join(out)
    if result and text.endswith('\n'):
        result += '\n'
    return result.encode('utf-8')


def canonical_bundle(data: bytes) -> bytes:
    blocks=split_bundle(data)
    if 'universal' not in blocks:
        raise RuntimeError('Split-source import requires universal settings/templates')
    return b''.join(
        blocks[name].rstrip() + b'\n\n'
        for name in ['universal'] + sorted(name for name in blocks if name != 'universal')
    ).rstrip() + b'\n'


def restore_comments(reference: bytes, authoritative: bytes) -> bytes:
    """Reapply repository-only comments to matching sections and settings."""
    section_comments: dict[str, list[bytes]] = {}
    section_inline: dict[str, bytes] = {}
    setting_comments: dict[tuple[str, str], list[bytes]] = {}
    setting_inline: dict[tuple[str, str], bytes] = {}
    global_comments: list[bytes] = []
    section = None
    pending: list[bytes] = []
    for text_line in reference.decode('utf-8').splitlines(keepends=True):
        line = text_line.encode('utf-8')
        content, marker, comment = line.partition(b'#')
        stripped = content.strip()
        inline = b'#' + comment.rstrip(b'\r\n') if marker else b''
        if stripped.startswith(b'[') and stripped.endswith(b']'):
            if section is None:
                global_comments.extend(pending)
            else:
                section_comments[section] = pending
            section = stripped.decode()
            if inline:
                section_inline[section] = inline
            pending = []
        elif not stripped and marker:
            pending.append(line)
        elif stripped:
            if section is None:
                global_comments.extend(pending)
            else:
                key = stripped.split(b'=', 1)[0].strip().decode()
                identity = (section, key)
                if pending:
                    setting_comments[identity] = pending
                if inline:
                    setting_inline[identity] = inline
            pending = []
    if section is None:
        global_comments.extend(pending)

    output: list[bytes] = []
    section = None
    emitted_global = False
    for line in authoritative.splitlines(keepends=True):
        content, marker, _comment = line.partition(b'#')
        stripped = content.strip()
        if stripped.startswith(b'[') and stripped.endswith(b']'):
            if not emitted_global:
                output.extend(global_comments)
                emitted_global = True
            section = stripped.decode()
            output.extend(section_comments.get(section, ()))
            inline = section_inline.get(section)
            if inline and not marker:
                line = line.rstrip(b'\r\n') + b' ' + inline + (b'\n' if line.endswith(b'\n') else b'')
        elif stripped and section is not None:
            identity = (section, stripped.split(b'=', 1)[0].strip().decode())
            output.extend(setting_comments.get(identity, ()))
            inline = setting_inline.get(identity)
            if inline and not marker:
                line = line.rstrip(b'\r\n') + b' ' + inline + (b'\n' if line.endswith(b'\n') else b'')
        output.append(line)
    if not emitted_global:
        output.extend(global_comments)
    return b''.join(output)


def plan_breakdown(source: Path, data: bytes) -> dict:
    """Describe the per-file result of splitting a combined file; writes nothing."""
    canonical = canonical_bundle(data)
    if len(canonical) > MAX_BYTES:
        raise RuntimeError('Sequence import exceeds 512 KiB')
    blocks = split_bundle(canonical)
    fighters = source.parent / 'fighters'
    files = {source: blocks.pop('universal')}
    files.update({fighters / (name + '.txt'): body for name, body in blocks.items()})
    existing = {path for path in [source, *fighters.glob('*.txt')] if path.is_file()}
    created, updated, unchanged = [], [], []
    for path, body in sorted(files.items()):
        name = 'universal' if path == source else path.stem
        count = len(re.findall(rb'^\[', body, re.M))
        entry = {'fighter': name, 'path': str(path), 'bytes': len(body), 'sections': count}
        if path not in existing:
            created.append(entry)
        elif path.read_bytes() != body:
            updated.append(entry)
        else:
            unchanged.append(entry)
    removed = [{'fighter': path.stem, 'path': str(path),
                'bytes': path.stat().st_size,
                'sections': len(re.findall(rb'^\[', path.read_bytes(), re.M))}
               for path in sorted(existing - set(files))]
    return {'source': str(source), 'bytes': len(canonical),
            'sections': len(re.findall(rb'^\[', canonical, re.M)),
            'fighters': len(files) - 1, 'created': created, 'updated': updated,
            'unchanged': unchanged, 'removed': removed, 'data': canonical}


def describe_breakdown(plan: dict) -> str:
    """One readable summary of a breakdown plan for previews and confirmations."""
    lines = [f"Break down {plan['sections']} sections / {plan['bytes']} bytes "
             f"into {plan['fighters']} fighter files plus universal settings:"]
    for label, key in (('New', 'created'), ('Replaced', 'updated'), ('DELETED', 'removed')):
        entries = plan[key]
        if entries:
            names = ', '.join(f"{e['fighter']} ({e['sections']})" for e in entries[:12])
            more = f" … +{len(entries) - 12} more" if len(entries) > 12 else ''
            lines.append(f"  {label} {len(entries)}: {names}{more}")
    if plan['unchanged']:
        lines.append(f"  Unchanged {len(plan['unchanged'])}")
    if plan['removed']:
        lines.append('  Files above marked DELETED are not present in the new file.')
    return '\n'.join(lines)


def replace_sources(source: Path, data: bytes, backup: Path, expected: bytes | None = None) -> None:
    """Explicit, backed-up source import with concurrent-edit checks and rollback."""
    data=canonical_bundle(data)
    if len(data)>MAX_BYTES:
        raise RuntimeError('Sequence import exceeds 512 KiB')
    blocks=split_bundle(data)
    fighters=source.parent/'fighters'
    fighters.mkdir(parents=True,exist_ok=True)
    files={source:blocks.pop('universal')}
    files.update({fighters/(name+'.txt'):body for name,body in blocks.items()})
    old={p:p.read_bytes() for p in [source,*fighters.glob('*.txt')] if p.exists()}
    backup.mkdir(parents=True,exist_ok=True)
    for file,body in old.items():
        write_generated(body,backup/('universal.txt' if file==source else file.name))
    if expected is not None and assemble(source)!=expected:
        raise RuntimeError('Split sources changed before import; nothing overwritten')
    try:
        for file,body in files.items():
            if old.get(file)!=body:
                write_generated(body,file)
        for file in old:
            if file not in files:
                file.unlink()
        if assemble(source)!=data:
            raise RuntimeError('Split-source readback did not match imported data')
    except BaseException:
        for file in set(old)|set(files):
            if file in old:
                write_generated(old[file],file)
            else:
                file.unlink(missing_ok=True)
        raise


def main():
    parser = argparse.ArgumentParser(description='Assemble universal/fighter sequence sources; no mod build or upload.')
    parser.add_argument('--source', type=Path, default=SOURCE)
    parser.add_argument('--output', type=Path, default=GENERATED)
    parser.add_argument('--check', action='store_true', help='check the generated bundle is current without writing')
    parser.add_argument('--import-file', type=Path, help='split a downloaded combined file back into these source files')
    parser.add_argument('--dry-run', action='store_true',
                        help='with --import-file, show the per-fighter breakdown without writing')
    args = parser.parse_args()
    try:
        if args.output.resolve() == args.source.resolve():
            raise RuntimeError('Output must not overwrite the universal source')
        if args.dry_run and not args.import_file:
            raise RuntimeError('--dry-run only applies to --import-file')
        if args.import_file:
            if args.check:
                raise RuntimeError('--check cannot be combined with --import-file')
            from sequence_validation import validate_sequences
            raw = args.import_file.read_bytes()
            validate_sequences(raw)
            plan = plan_breakdown(args.source, raw)
            print(describe_breakdown(plan))
            if args.dry_run:
                parser.exit(0, 'Dry run: no source file was written.\n')
            backup_root=ROOT/'.sequence-sync-backups'/'source-imports'
            backup_root.mkdir(parents=True,exist_ok=True)
            backup=Path(tempfile.mkdtemp(prefix='import-',dir=backup_root))
            replace_sources(args.source,raw,backup)
            print(f'Imported sources; backup: {backup}')
        data = assemble(args.source)
        from sequence_validation import validate_sequences
        validate_sequences(data)
        if args.check:
            if not args.output.exists() or args.output.read_bytes() != data:
                raise RuntimeError('Assembled bundle is stale; run python3 tools/sequence_sources.py')
            print('Assembled bundle is current.')
        else:
            write_generated(data, args.output)
            print(f'Assembled {len(data)} bytes: {args.output}')
    except (OSError, RuntimeError, ValueError) as error:
        parser.exit(1, str(error) + '\n')


if __name__ == '__main__':
    main()
