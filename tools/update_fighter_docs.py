#!/usr/bin/env python3
"""Refresh fighter reference pages from compiled Rust declarations, without building.

This deliberately supports only the static catalog syntax used in this repo.
Unknown expressions fail before any pages are written; it never executes Rust.
Keep authored notes outside the generated markers. Run --check to detect drift.
"""
import argparse
from functools import lru_cache
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates/kirby_assist_core/src"
HANDLERS = CORE / "ledge_trap/fighters"
PROFILES = CORE / "fighter_profiles"
BEGIN = "<!-- BEGIN GENERATED FIGHTER REFERENCE -->"
END = "<!-- END GENERATED FIGHTER REFERENCE -->"
NAMES = {
    "captain": "Captain Falcon", "donkey": "Donkey Kong", "krool": "King K. Rool",
    "littlemac": "Little Mac", "pacman": "Pac-Man", "rob": "R.O.B.",
    "mr_game_and_watch": "Mr. Game & Watch", "bowser_jr": "Bowser Jr.",
    "dr_mario": "Dr. Mario", "rosalina_and_luma": "Rosalina & Luma",
    "ice_climbers": "Ice Climbers (Popo)", "ice_climbers_nana": "Ice Climbers (Nana)",
}

@lru_cache(maxsize=1)
def opponent_reference():
    snapshot = json.loads((ROOT / "data/opponent_reference.json").read_text())
    return {f["kind"]: f for f in snapshot["fighters"]}
ROWS = ["Neutral", "Jump", "Roll", "Attack", "Drop"]


def source(path):
    # Remove // comments only outside strings, preserving URLs/menu labels.
    return re.sub(r'"(?:\\.|[^"\\])*"|//[^\n]*',
                  lambda m: "" if m[0].startswith("//") else m[0], path.read_text())


def split_top(text, separator=","):
    parts, start, depth, quoted, escaped = [], 0, 0, False, False
    for index, char in enumerate(text):
        if quoted:
            if escaped: escaped = False
            elif char == "\\": escaped = True
            elif char == '"': quoted = False
            continue
        if char == '"': quoted = True
        elif char in "([{": depth += 1
        elif char in ")]}": depth -= 1
        elif char == separator and depth == 0:
            parts.append(text[start:index].strip()); start = index + 1
    parts.append(text[start:].strip())
    return [part for part in parts if part]


@lru_cache(None)
def constants(module):
    text = source(HANDLERS / (module + ".rs"))
    result = {}
    for match in re.finditer(r"\b(?:const|static)\s+(\w+)\s*:[^=]+?=", text):
        result[match[1]] = split_top(text[match.end():], ";")[0]
    return result


def fields(expression):
    body = expression[expression.index("{") + 1:expression.rindex("}")]
    return dict(part.split(":", 1) for part in split_top(body))


def field(expression, key):
    return {name.strip(): value.strip() for name, value in fields(expression).items()}[key]


def evaluate(module, expression):
    expression = expression.strip().removeprefix("&").strip()
    if expression in constants(module):
        return evaluate(module, constants(module)[expression])
    standard = re.fullmatch(r"(?:super::)?standard::menu\((.*)\)", expression, re.S)
    if standard:
        neutral, attack, up_b = split_top(standard[1])
        names = ["NEUTRAL_OPTIONS", "JUMP_OPTIONS", "ROLL_OPTIONS",
                 "ATTACK_UP_B_OPTIONS" if up_b == "true" else "ATTACK_OPTIONS", "DROP_OPTIONS"]
        return [{"selected": int(neutral) if i == 0 else int(attack) if i == 3 else 0,
                 "options": evaluate("standard", name)} for i, name in enumerate(names)]
    alias = re.fullmatch(r"super::(\w+)::MENU", expression)
    if alias:
        return evaluate(alias[1], "MENU")
    reference = re.fullmatch(r"(\w+)\.lists\[(\d+)\](?:\.options(?:\[(\d+)\])?)?", expression)
    if reference:
        value = evaluate(module, reference[1])[int(reference[2])]
        if ".options" in expression:
            value = value["options"]
            if reference[3] is not None: value = value[int(reference[3])]
        return value
    item = re.fullmatch(r"(\w+)\[(\d+)\]", expression)
    if item:
        return evaluate(module, item[1])[int(item[2])]
    if expression.startswith("MenuPresets {"):
        return evaluate(module, field(expression, "lists"))
    if expression.startswith("SavedList {"):
        return {"selected": int(field(expression, "selected")),
                "options": evaluate(module, field(expression, "options"))}
    if expression.startswith("SavedList::new("):
        return {"selected": 0, "options": evaluate(module, expression[len("SavedList::new("):-1])}
    if expression.startswith("SavedChoice::new("):
        args = split_top(expression[len("SavedChoice::new("):-1])
        assert len(args) == 4, expression
        return {"id": json.loads(args[0]), "name": json.loads(args[1]),
                "handler": args[2], "prepare": args[3]}
    if expression.startswith("SavedChoice {"):
        parts = split_top(expression[expression.index("{")+1:expression.rindex("}")])
        base = next(part[2:].strip() for part in parts if part.startswith(".."))
        choice = dict(evaluate(module, base))
        for part in parts:
            if part.startswith(".."): continue
            key, value = [s.strip() for s in part.split(":", 1)]
            key = "handler" if key == "respond" else key
            choice[key] = json.loads(value) if key in ("id", "name") else value
        return choice
    if expression.startswith("[") and expression.endswith("]"):
        return [evaluate(module, item) for item in split_top(expression[1:-1])]
    raise ValueError("Unsupported static catalog expression in %s: %s" % (module, expression[:160]))


def number(text, key):
    match = re.search(r"\b" + key + r"\s*:\s*([\d.]+)", text)
    if not match: raise ValueError("Missing numeric field " + key)
    return match[1]


def title(module):
    return NAMES.get(module, module.replace("_", " ").title())


@lru_cache(None)
def menu_names():
    text = source(PROFILES / "spacing.rs")
    body = text[text.index("pub fn menu_name"):text.index("pub fn hint")]
    strings = r'"(?:\\.|[^"\\])*"'
    return {json.loads(left): json.loads(right) for left, right in
            re.findall(r"(" + strings + r")\s*=>\s*(" + strings + r")", body)}


def render(module):
    profile = source(PROFILES / (module + ".rs"))
    kind = re.search(r"fighter_kind:\s*(\w+)", profile)[1]
    if not re.fullmatch(r"0x[\da-fA-F]+|\d+", kind):
        kind = re.search(r"const\s+" + kind + r"\s*:\s*i32\s*=\s*(0x[\da-fA-F]+|\d+)", profile)[1]
    kind = str(int(kind, 0))
    menu = evaluate(module, "MENU")
    assert len(menu) == 5, module
    for row in menu:
        assert row["options"] and 0 <= row["selected"] < len(row["options"]), module
        assert len({c['id'] for c in row['options']}) == len(row['options']), module
    timing = source(PROFILES / module / "timing.rs")
    data = timing[timing.index("LEDGE_DATA:"):]
    lines = ["# " + title(module), "", BEGIN, "",
             "[All fighters](../FIGHTERS.md) · [Controls](../CONTROLS.md)", "",
             "Native fighter kind **%s** (`%s`). This reference follows the compiled source;" % (kind, module),
             "it is not a device-calibration result. No fighter file loads during a match.", "",
             "## Saved ledge responses", "",
             "Open the menu with **R3**.",
             "Up/Down selects a row; Left/Right cycles its",
             "choices. Hold **L** to run the selected responses. Defaults are bold.", "",
             "| Row | Saved choices, in menu order |", "| --- | --- |"]
    for name, row in zip(ROWS, menu):
        names = [menu_names().get(choice['name'], choice['name']) for choice in row['options']]
        choices = ["**%s**" % name if index == row['selected'] else name
                   for index, name in enumerate(names)]
        lines.append("| %s | %s |" % (name, "; ".join(choices)))
    lines += ["", "These are the compact UI names; SH means short hop and Msl means missile.",
              "Compact move labels match the menu without spacing hints. Capture returns control for manual",
              "pummels/throws. Selecting a choice does not add a combo or change native hitboxes.", "",
              "## Compiled move windows", "",
              "These are stored inclusive animation-frame spans. They are not total input-to-hit",
              "delays or proof that every phase of a multihit move stays active.", "",
              "| Move | Stored frames |", "| --- | --- |"]
    for key, label in [("neutral_air", "Neutral air"), ("down_tilt", "Down tilt"),
                       ("forward_tilt", "Forward tilt"), ("down_smash", "Down smash")]:
        match = re.search(key + r"\s*:\s*MoveFrames\s*\{([^}]+)\}", data, re.S)
        if not match: raise ValueError(module + " missing " + key)
        lines.append("| %s | %s–%s |" % (label, number(match[1], "first"), number(match[1], "last")))
    rear = re.search(r"down_smash_back:\s*Some\(MoveFrames\s*\{([^}]+)\}", data, re.S)
    if rear: lines.append("| Down-smash rear window | %s–%s |" % (number(rear[1], "first"), number(rear[1], "last")))
    for move in re.finditer(r"pub const (\w+): MoveFrames = MoveFrames\s*\{([^}]+)\}", timing, re.S):
        lines.append("| %s | %s–%s |" % (move[1].replace('_', ' ').title(), number(move[2], "first"), number(move[2], "last")))
    for key, label in [("down_special_startup", "Grounded down-special startup"),
                       ("up_special_startup", "Grounded up-special startup")]:
        special = re.search(key + r":\s*Some\((\d+)\)", data)
        if special: lines.append("| %s | %s |" % (label, special[1]))
    grab = re.search(r"grab:\s*GrabTiming\s*\{([^}]+)\}", data, re.S)[1]
    lines += ["", "### Grab data", "", "| Setting | Compiled value |", "| --- | --- |"]
    for key, label in [("standing", "Standing startup"), ("dash", "Dash startup"),
                       ("turn_or_shield", "Pivot startup"), ("shield", "Shield-grab startup"),
                       ("active_frames", "Active duration"), ("standing_range", "Standing reach reference (world units)")]:
        lines.append("| %s | %s |" % (label, number(grab, key)))
    lines += ["", "Reach is an approximate root-based reference, not an animated grab/hurtbox model.",
              "Shared roll grabs test predicted range/facing, height and vulnerability on the",
              "same active frame. Native catch readiness and input delivery still matter.", "",
              "## Execution and calibration", ""]
    facts = opponent_reference().get(int(kind))
    if facts:
        table = {s["key"]: s["value"] for s in facts["stats"]}
        section = ["## Baseline reference stats", "",
                   "Compiled [opponent/move reference](../OPPONENT_DATA.md); baseline facts, not",
                   "instance modifiers or a replacement for existing ledge tuning.", "",
                   "| Stat | Reference |", "| --- | --- |"]
        for key, label in [("weight.weight", "Weight"), ("walkspeed.walkspeed", "Walk speed"),
                           ("dashandrun.initialdash", "Initial dash"), ("dashandrun.runspeed", "Run speed"),
                           ("airspeed.airspeed", "Air speed"), ("gravityt.gravity", "Gravity"),
                           ("fallspeed.regularfall", "Fall speed"), ("fallspeed.fastfall", "Fast fall"),
                           ("jumpheight.shorthop", "Short-hop height"), ("jumpheight.fullhop", "Full-hop height")]:
            section.append("| %s | %s |" % (label, table.get(key, "Unknown")))
        section += ["", "[Move source](%s) · [Stats source](https://ultimateframedata.com/stats.php)" % facts["source"], ""]
        lines[-2:-2] = section
    if "AerialExecution::SHORT" in profile:
        lines += ["The profile declares short hops for common nair responses."]
    else:
        lines += ["The profile declares adaptive short/full-hop selection for generic Jump coverage;",
                  "common Neutral and shield nairs remain short hops. A named handler can override",
                  "that policy, including grounded attacks and immediate short-hop recipes."]
    related = {"kirby": "KIRBY_YOSHI_LEDGE.md", "yoshi": "KIRBY_YOSHI_LEDGE.md",
               "samus": "SAMUS_LEDGE.md", "dark_samus": "SAMUS_LEDGE.md", "sonic": "SONIC_LEDGE.md",
               "krool": "KROOL_ISABELLE_LEDGE.md", "isabelle": "KROOL_ISABELLE_LEDGE.md",
               "pacman": "PACMAN_MAC_LEDGE.md", "littlemac": "PACMAN_MAC_LEDGE.md",
               "luigi": "FIGHTER_ADDITIONS.md", "captain": "FIGHTER_ADDITIONS.md"}
    if module in related:
        lines += ["", "See [this fighter's detailed execution notes](../%s) for its specific sequences," % related[module],
                  "timing overrides, drift and exceptions."]
    if module.startswith("ice_climbers"):
        lines += ["", "Nana has a registered native identity, not an extra selectable fighter or partner",
                  "AI/desync tool. The selected-human controller checks still apply."]
    lines += ["", "Spacing, resource requirements, opponent steering, stage geometry and online",
              "input arrival still affect contact. There is no learned-delay compensation.",
              "Use the [diagnostic result card](../CALIBRATION_CARD.md),",
              "[execution/calibration](../FIGHTER_EXECUTION.md) and the",
              "[roster exceptions](../NON_DLC_ROSTER.md#native-state-and-resource-exceptions)",
              "before treating an option as reliable in a particular setup.", "",
              "## Source and edits", "",
              "Paths are relative to the repository root:", "",
              "- Registration: `crates/kirby_assist_core/src/fighter_profiles/%s.rs`." % module,
              "- Move data: `crates/kirby_assist_core/src/fighter_profiles/%s/timing.rs`." % module,
              "- Menu and handlers: `crates/kirby_assist_core/src/ledge_trap/fighters/%s.rs`." % module,
              "", "[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.",
              "These are compiled choices; source changes need a separately authorized build/install.",
              "Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.", "", END]
    return "\n".join(lines) + "\n", int(kind)


def outputs():
    catalog = (CORE / "fighter_profiles.rs").read_text()
    modules = re.findall(r"(\w+)::profile\(\)", catalog)
    assert len(modules) == len(set(modules)), "Duplicate registration"
    result, identities = {}, set()
    for module in modules:
        content, kind = render(module)
        assert kind not in identities, "Duplicate native kind"
        identities.add(kind)
        path = ROOT / "docs/fighters" / (module + ".md")
        if path.exists():
            old = path.read_text()
            if BEGIN not in old or END not in old: raise ValueError("Missing generated markers: " + str(path))
            generated = content[content.index(BEGIN):content.index(END) + len(END)]
            content = old[:old.index(BEGIN)] + generated + old[old.index(END) + len(END):]
        result[path] = content
    index = ["<!-- BEGIN INDIVIDUAL FIGHTER LINKS -->", "", "## Individual fighter pages", "",
             "Each registered fighter form has its own source-based reference, including",
             "all five saved lists, default choices, move windows and grab data.", "",
             "| Fighter | Fighter | Fighter |", "| --- | --- | --- |"]
    links = ["[%s](fighters/%s.md)" % (title(m), m) for m in sorted(modules, key=title)]
    for i in range(0, len(links), 3): index.append("| " + " | ".join((links[i:i+3] + [""] * 3)[:3]) + " |")
    index += ["", "<!-- END INDIVIDUAL FIGHTER LINKS -->"]
    hub = ROOT / "docs/FIGHTERS.md"
    old = hub.read_text()
    begin, end = index[0], index[-1]
    block = "\n".join(index)
    if begin in old:
        updated = old[:old.index(begin)] + block + old[old.index(end) + len(end):]
    else:
        updated = old.replace("## Choose a guide", block + "\n\n## Shared and detailed guides")
    result[hub] = updated
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Check without modifying pages")
    args = parser.parse_args()
    generated = outputs()  # Resolve/validate everything before modifying a page.
    changed = [p for p, text in generated.items() if not p.exists() or p.read_text() != text]
    if args.check and changed:
        raise SystemExit("Stale fighter docs: " + ", ".join(str(p.relative_to(ROOT)) for p in changed))
    if not args.check:
        for path in changed:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(generated[path])
    print("%s %d fighter pages and overview; %d changed." % ("Checked" if args.check else "Refreshed", len(generated)-1, len(changed)))


if __name__ == "__main__":
    main()
