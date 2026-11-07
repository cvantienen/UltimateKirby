"""Host-side fighter source assembly. No game-time includes or file discovery."""
from __future__ import annotations

import argparse
from pathlib import Path
import re
import tempfile
import os

ROOT = Path(__file__).resolve().parents[1]
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
        if total > MAX_BYTES:
            raise RuntimeError('Assembled sequence file exceeds 512 KiB')
    output = b''.join(pieces).rstrip() + b'\n'
    if len(output)>MAX_BYTES:
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


def canonical_bundle(data: bytes) -> bytes:
    blocks=split_bundle(data)
    if 'universal' not in blocks:
        raise RuntimeError('Split-source import requires universal settings/templates')
    return b''.join(
        blocks[name].rstrip() + b'\n\n'
        for name in ['universal'] + sorted(name for name in blocks if name != 'universal')
    ).rstrip() + b'\n'


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
    args = parser.parse_args()
    try:
        if args.output.resolve() == args.source.resolve():
            raise RuntimeError('Output must not overwrite the universal source')
        if args.import_file:
            if args.check:
                raise RuntimeError('--check cannot be combined with --import-file')
            from sequence_validation import validate_sequences
            raw = args.import_file.read_bytes()
            validate_sequences(raw)
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
