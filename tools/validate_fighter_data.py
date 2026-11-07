#!/usr/bin/env python3
"""Validate the canonical, offline Ultimate fighter stats and frame-data file."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATA_FILE = ROOT / "data/Ultimate_stats.json"
EXPECTED_KINDS = set(range(77)) | set(range(81, 94))


def validate(data):
    if data.get("schema_version") != 1:
        raise ValueError("Unsupported fighter data schema version")
    fighters = data.get("fighters")
    if not isinstance(fighters, list) or len(fighters) != len(EXPECTED_KINDS):
        raise ValueError("Fighter data must contain exactly 90 fighter records")

    kinds = []
    for fighter in fighters:
        kind = fighter.get("kind")
        if not isinstance(kind, int) or kind not in EXPECTED_KINDS:
            raise ValueError(f"Unknown native fighter kind: {kind!r}")
        kinds.append(kind)
        if not isinstance(fighter.get("name"), str) or not fighter["name"]:
            raise ValueError(f"Fighter kind {kind} has no name")
        if not isinstance(fighter.get("stats"), list) or not fighter["stats"]:
            raise ValueError(f"Fighter kind {kind} has no stats")
        for stat in fighter["stats"]:
            if not isinstance(stat.get("key"), str) or not stat["key"]:
                raise ValueError(f"Fighter kind {kind} has an invalid stat key")
            if not isinstance(stat.get("value"), str):
                raise ValueError(f"Fighter kind {kind} has a non-text stat value")
        if not isinstance(fighter.get("moves"), list) or not fighter["moves"]:
            raise ValueError(f"Fighter kind {kind} has no frame data")

        for move in fighter["moves"]:
            if not isinstance(move.get("name"), str) or not move["name"]:
                raise ValueError(f"Fighter kind {kind} has an unnamed move")
            if not isinstance(move.get("raw"), dict):
                raise ValueError(f"Fighter kind {kind} move {move['name']} has invalid raw data")
            if not all(isinstance(key, str) and isinstance(value, str)
                       for key, value in move["raw"].items()):
                raise ValueError(f"Fighter kind {kind} move {move['name']} has non-text raw data")
            if not isinstance(move.get("active"), list):
                raise ValueError(f"Fighter kind {kind} move {move['name']} has invalid active frames")
            for span in move["active"]:
                if (not isinstance(span, list) or len(span) != 2
                        or not all(isinstance(frame, int) for frame in span)
                        or span[0] <= 0 or span[0] > span[1]):
                    raise ValueError(
                        f"Fighter kind {kind} move {move['name']} has invalid active span {span!r}"
                    )
            if not isinstance(move.get("motions"), list) or not all(
                    isinstance(name, str) for name in move["motions"]):
                raise ValueError(f"Fighter kind {kind} move {move['name']} has invalid motions")
            if move.get("whiff_eligible") and (
                    not move["active"] or not isinstance(move.get("total"), int)):
                raise ValueError(f"Fighter kind {kind} move {move['name']} has invalid recovery facts")

    if set(kinds) != EXPECTED_KINDS or kinds != sorted(kinds):
        raise ValueError("Fighter records must contain every supported kind exactly once, in order")
    return len(fighters), sum(len(fighter["moves"]) for fighter in fighters)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--file", type=Path, default=DATA_FILE)
    args = parser.parse_args()
    try:
        data = json.loads(args.file.read_text(encoding="utf-8"))
        fighter_count, move_count = validate(data)
    except (OSError, json.JSONDecodeError, ValueError, TypeError) as error:
        parser.exit(1, f"Invalid fighter data: {error}\n")
    print(f"Validated {fighter_count} fighters and {move_count} moves in {args.file}")


if __name__ == "__main__":
    main()
