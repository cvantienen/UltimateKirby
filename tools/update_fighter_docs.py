#!/usr/bin/env python3
"""Refresh fighter pages from file menus and independent reference facts, without building.

This deliberately supports only the static catalog syntax used in this repo.
Unknown expressions fail before any pages are written; it never executes Rust.
Keep authored notes outside the generated markers. Run --check to detect drift.
"""
import argparse
import configparser
from functools import lru_cache
import json
from pathlib import Path
import re
from sequence_sources import assemble

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates/kirby_assist_core/src"
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


@lru_cache(maxsize=1)
def sequence_catalog():
    parser = configparser.RawConfigParser(strict=True)
    parser.read_string(assemble(ROOT / "deployment/ultimate/kirby_assist/sequences.txt").decode())
    return parser


def file_menu(module):
    profile = source(PROFILES / (module + ".rs"))
    name = re.search(r'name:\s*"([^"]+)"', profile)[1]
    result = [{"selected": 0, "options": []} for _ in ROWS]
    for section in sequence_catalog().sections():
        parts = section.split(".")
        # Fighter policy sections and button recipes are not ledge-response rows.
        if len(parts) != 3 or parts[1] not in [r.lower() for r in ROWS]:
            continue
        fighter, situation, choice = parts
        if fighter != name: continue
        row = result[[r.lower() for r in ROWS].index(situation)]
        values = sequence_catalog()[section]
        if values.get("reaction", "off").split(":", 1)[0].strip() != "off":
            continue  # Assist rules are not manual ledge choices/defaults.
        if values.getboolean("default", fallback=False): row["selected"] = len(row["options"])
        row["options"].append({"id": choice, "name": values.get("name", choice).strip('"'),
                               "start_delay": values["start_delay"], "actions": values["actions"]})
    return result


def file_assists(module):
    profile = source(PROFILES / (module + ".rs"))
    name = re.search(r'name:\s*"([^"]+)"', profile)[1]
    rules = []
    for section in sequence_catalog().sections():
        if section.split(".")[0] != name:
            continue
        values = sequence_catalog()[section]
        fields = values.get("reaction", "off").split(":")
        if fields[0].strip() == "off":
            continue
        simple = fields[0].strip() in {"auto_roll", "auto_spot_dodge", "auto_air_dodge", "auto_grab", "auto_shield_grab", "auto_grab_dodged", "auto_shield_backup"}
        priority = 1 if simple else int(fields[6])
        rules.append((fields[0], priority, values.get("name", section).strip('"'), section))
    return sorted(rules, key=lambda r: (r[0], r[1]))


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
    source_name = re.search(r'name:\s*"([^"]+)"', profile)[1]
    kind = re.search(r"fighter_kind:\s*(\w+)", profile)[1]
    if not re.fullmatch(r"0x[\da-fA-F]+|\d+", kind):
        kind = re.search(r"const\s+" + kind + r"\s*:\s*i32\s*=\s*(0x[\da-fA-F]+|\d+)", profile)[1]
    kind = str(int(kind, 0))
    menu = file_menu(module)
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
             "The lightweight selector starts visible; **Minus** hides/shows it. Training opens with **Minus**.",
             "Up/Down selects a row; Left/Right cycles its",
             "choices. Hold **L** to run the selected responses. Defaults are bold.", "",
             "| Row | Saved choices, in menu order |", "| --- | --- |"]
    for name, row in zip(ROWS, menu):
        names = [menu_names().get(choice['name'], choice['name']) for choice in row['options']]
        choices = ["**%s**" % name if index == row['selected'] else name
                   for index, name in enumerate(names)]
        lines.append("| %s | %s |" % (name, "; ".join(choices)))
    lines += ["", "These are the compact UI names; SH means short hop and Msl means missile.",
              "Input timelines hand back control after their last step; the game decides acceptance.",
              "Selecting a choice never changes native moves, hitboxes, physics or animation speed.", "",
              "## Native move reference (not scheduling)", "",
              "These independent game facts do not schedule file inputs. They are inclusive animation-frame spans, not input-to-hit",
              "delays or proof that every phase of a multihit move stays active.", "",
              "| Move | Stored frames |", "| --- | --- |"]
    # Put assist lists beside, never inside, the manual ledge-choice table.
    marker = lines.index("## Native move reference (not scheduling)")
    assists = ["## Saved assist rules", "",
               "These rules have separate ON/OFF assists and Training L3/R3 rule lists;",
               "they are not selected ledge/button recipes. Auto react has one sequence per event;",
               "priority numbers apply only to legacy rule lists. Vulnerable-ledge punishment is retired.", "",
               "| Assist trigger | Priority | Saved rule | Section |", "| --- | --- | --- | --- |"]
    rules = file_assists(module)
    for trigger, priority, label, section in rules:
        shown_priority = "—" if trigger in {"auto_roll", "auto_spot_dodge", "auto_air_dodge", "auto_grab", "auto_shield_grab", "auto_grab_dodged", "auto_shield_backup"} else priority
        assists.append(f"| `{trigger}` | {shown_priority} | {label} | `{section}` |")
    if not rules:
        assists.append("| — | — | No saved assist rules in the supplied master | — |")
    lines[marker:marker] = assists + [""]
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
              "The file decides whether to request an explicit range check. No native timing gate",
              "is inferred from this table; input delivery and the game's buffering rules still matter.", "",
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
    lines += [f"Option definitions and input delays come from `fighters/{source_name}.txt`, assembled into the Switch’s `sequences.txt`. Each option",
              "has an explicit `start_delay`; waits between actions are literal native frames.",
              "No compiled fighter recipe, adaptive-hop policy or native move window schedules them.", "",
              "| Situation / ID | Start delay | Ordered inputs |", "| --- | --- | --- |"]
    for situation, row in zip(ROWS, menu):
        for choice in row["options"]:
            lines.append("| %s / `%s` | %s | `%s` |" % (situation, choice["id"], choice["start_delay"], choice["actions"]))
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
              "- Recipes and overrides: `deployment/ultimate/kirby_assist/fighters/<fighter>.txt`; universal settings/templates: `deployment/ultimate/kirby_assist/sequences.txt`.",
              "- Shared input runner: `crates/kirby_assist_core/src/ledge_trap.rs`.",
              "", "[Adding a response](../FIGHTER_CONFIGS.md) explains registration and shared helpers.",
              "Recipes are saved file configuration. External file edits need a Smash restart;",
              "Rust/reference-data changes need a separately authorized build/install.",
              "Refresh this reference with `python3 tools/update_fighter_docs.py` after catalog edits.", "", END]
    return "\n".join(lines) + "\n", int(kind)


def outputs():
    catalog = (CORE / "fighter_profiles.rs").read_text()
    modules = re.findall(r"\b(\w+)::profile(?:\(\))?\s*,", catalog)
    if not modules:
        raise ValueError("No registered fighter constructors found; refusing to erase fighter links")
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
             "all five file-authored lists, selected defaults, input schedules and reference facts.", "",
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
