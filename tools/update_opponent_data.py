#!/usr/bin/env python3
"""Offline/public-data importer. The Switch uses generated Rust, never this JSON.

--fetch explicitly accesses UFD; ordinary runs and --check are network-free.
Only factual move/stat fields are retained, not artwork or prose descriptions.
Unknown/compound numeric expressions stay raw and are not invented as constants.
"""
import argparse
from datetime import datetime, timezone
from concurrent.futures import ThreadPoolExecutor
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import urllib.request
import zlib

ROOT = Path(__file__).resolve().parents[1]
SNAPSHOT = ROOT / "data/opponent_reference.json"
OUTPUT = ROOT / "crates/kirby_assist_core/src/opponent_data/generated.rs"
BASE = "https://ultimateframedata.com/"
# Native kinds verified against pinned skyline-smash d2ca90d lua_const.rs.
BASE_NAMES = "mario donkey_kong link samus dark_samus yoshi kirby fox pikachu luigi ness jigglypuff captain_falcon peach daisy bowser sheik zelda dr_mario pichu falco marth lucina young_link ganondorf mewtwo roy chrom mr_game_and_watch meta_knight pit dark_pit zero_suit_samus wario snake ike squirtle ivysaur charizard diddy_kong lucas sonic king_dedede olimar lucario rob toon_link wolf villager mega_man wii_fit_trainer rosalina_and_luma little_mac greninja palutena pac_man robin shulk bowser_jr duck_hunt ryu ken cloud corrin bayonetta inkling ridley simon richter king_k_rool isabelle incineroar mii_brawler mii_swordfighter mii_gunner ice_climbers ice_climbers".split()
DLC_NAMES = "piranha_plant joker hero banjo_and_kazooie terry byleth min_min steve sephiroth pyra mythra kazuya sora".split()
KINDS = dict(enumerate(BASE_NAMES)) | dict(enumerate(DLC_NAMES, 0x51))
ALIASES = {"ice_climbers": ["Popo", "Nana"], "rosalina_and_luma": ["Rosalina", "Rosalina & Luma"],
           "king_dedede": ["Dedede"], "mr_game_and_watch": ["Mr. Game & Watch"],
           "banjo_and_kazooie": ["Banjo & Kazooie"]}

def norm(s):
    return re.sub(r"[^a-z0-9]", "", s.lower())

class Node:
    def __init__(self, tag="root", attrs=()):
        self.tag, self.attrs, self.children = tag, dict(attrs), []
    def text(self):
        return " ".join(" ".join(x.text() if isinstance(x, Node) else x for x in self.children).split())
    def walk(self):
        yield self
        for child in self.children:
            if isinstance(child, Node): yield from child.walk()
    def has(self, cls): return cls in self.attrs.get("class", "").split()

class Document(HTMLParser):
    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.root = Node(); self.stack = [self.root]; self.feed(source)
    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs); self.stack[-1].children.append(node)
        if tag not in {"img", "br", "hr", "meta", "link", "input", "source", "wbr", "area", "embed"}:
            self.stack.append(node)
    def handle_endtag(self, tag):
        for i in range(len(self.stack)-1, 0, -1):
            if self.stack[i].tag == tag:
                del self.stack[i:]; break
    def handle_data(self, data): self.stack[-1].children.append(data)

def get(page):
    req = urllib.request.Request(BASE + page, headers={"User-Agent": "KirbyAssist-reference-import/1.0"})
    with urllib.request.urlopen(req, timeout=30) as response: raw = response.read()
    return raw.decode("utf-8"), hashlib.sha256(raw).hexdigest()

def integer(value):
    return int(value) if re.fullmatch(r"\d{1,4}", value) else None

def spans(value):
    value = value.replace("—", "-").replace("–", "-")
    if not re.fullmatch(r"[\d\s/,()\-]+", value): return []
    result = []
    for a, b in re.findall(r"(\d+)(?:\s*-\s*(\d+))?", value):
        first, last = int(a), int(b or a)
        if not 0 < first <= last <= 1000: return []
        result.append([first, last])
    return result

FIELDS = ("startup", "totalframes", "landinglag", "basedamage", "shieldlag", "shieldstun", "advantage", "activeframes", "endlag")
MOTIONS = {"Forward Tilt": ["attack_s3_s"],
           "Up Tilt": ["attack_hi3"], "Down Tilt": ["attack_lw3"],
           "Dash Attack": ["attack_dash"], "Forward Smash": ["attack_s4_s"],
           "Up Smash": ["attack_hi4"], "Down Smash": ["attack_lw4"],
           "Neutral Air": ["attack_air_n"], "Forward Air": ["attack_air_f"],
           "Back Air": ["attack_air_b"], "Up Air": ["attack_air_hi"], "Down Air": ["attack_air_lw"]}

def moves(source):
    out = []
    for node in Document(source).root.walk():
        if not node.has("movecontainer"): continue
        fields = {n.attrs["class"]: n.text() for n in node.walk() if n.attrs.get("class") in (*FIELDS, "movename", "notes")}
        if not fields.get("movename"): continue
        name = fields.pop("movename"); notes = fields.pop("notes", "")
        for key, pattern in {
            'autocancel': r'Autocancels? on frames?\s+([^.;]+)',
            'charge_hold_frame': r'Charge hold is frame\s+(\d+)',
            'landing_hit_frame': r'Landing hit on frame\s+(\d+)',
        }.items():
            found=re.search(pattern,notes,re.I)
            if found: fields[key]=found[1].strip()
        active = spans(fields.get("activeframes", ""))
        total = integer(fields.get("totalframes", ""))
        # Cancel/landing/held/projectile/resource exceptions need explicit review.
        review_notes = re.sub(r"Charge hold is frame \d+\.?", "", notes, flags=re.I)
        uncertain = bool(re.search(r"cancel|projectile|bullet|laser|landing|hold|held|loop|charge|Pikmin|ink|meter|stance|float|jump", review_notes, re.I))
        # Smashes are eligible only after the native release attack status;
        # remaining time is capped by native end/cancel observations as well.
        eligible = name in ("Forward Tilt", "Up Tilt", "Down Tilt", "Dash Attack", "Forward Smash", "Up Smash", "Down Smash") and not uncertain
        out.append(dict(name=name, raw=fields, active=active, total=total,
            landing_lag=integer(fields.get("landinglag", "")),
            motions=MOTIONS.get(name, []), whiff_eligible=bool(eligible and active and total and max(x[1] for x in active) < total)))
    if len(out) < 10: raise ValueError("Frame-data page did not contain enough parsed moves")
    return out

def fetch(date, stats_only=False):
    roster, _ = get("smash")
    links = {norm(n.attrs.get("title", "")): n.attrs.get("href", "").strip("/")
             for n in Document(roster).root.walk() if n.tag == "a" and n.attrs.get("title")}
    resolved = {}
    for slug in sorted(set(KINDS.values())):
        key = norm(slug)
        # Source URLs aren't always the same as the display/native names.
        page = links.get(key) or next((links[norm(a)] for a in ALIASES.get(slug, []) if norm(a) in links), None)
        if not page and slug == "pac_man": page = links.get("pacman")
        if not page: raise ValueError("Unresolved roster URL: " + slug)
        resolved[slug] = page
    stats_html, stats_hash = get("stats.php")
    stats = []
    for table in Document(stats_html).root.walk():
        if table.tag != "table": continue
        rows = [n for n in table.walk() if n.tag == "tr"]
        if not rows: continue
        headers = [n.text() for n in rows[0].walk() if n.tag == "th"]
        char_col = next((i for i, h in enumerate(headers) if norm(h) in ("character", "fighter")), None)
        if char_col is None: continue
        for row in rows[1:]:
            cells = [n.text() for n in row.walk() if n.tag == "td"]
            if len(cells) != len(headers): continue
            stats.append((cells[char_col], norm(table.attrs.get("id", "").removesuffix("table")),
                {norm(h): v for i, (h,v) in enumerate(zip(headers,cells)) if i != char_col and norm(h) != "rank"}))
    def load(item):
        slug, page = item; html, sha = get(page)
        return slug, dict(source=BASE+page, sha256=sha, moves=moves(html))
    # Three bounded downloads, no image requests or crawl beyond the roster.
    if stats_only:
        old=json.loads(SNAPSHOT.read_text())
        pages={f['name']:{k:f[k] for k in ('source','sha256','moves')} for f in old['fighters']}
    else:
        with ThreadPoolExecutor(max_workers=3) as pool: pages = dict(pool.map(load, resolved.items()))
    fighters = []
    for kind, slug in sorted(KINDS.items()):
        aliases = [slug.replace("_", " "), *ALIASES.get(slug, [])]
        if slug == "ice_climbers": aliases = ["Popo" if kind == 0x4b else "Nana", "Ice Climbers",
            "Ice Climbers (leader)" if kind == 0x4b else "Ice Climbers (partner)"]
        wanted = {norm(x) for x in aliases}
        values = [{"key": table+"."+column, "value": value}
            for name, table, fields in stats if norm(name) in wanted
            for column, value in fields.items()]
        fighters.append(dict(kind=kind, name=slug, stats=values, **pages[slug]))
    return dict(schema_version=1, snapshot_date=date, stats_source=BASE+"stats.php", stats_sha256=stats_hash, fighters=fighters)

def quote(s): return json.dumps(s, ensure_ascii=False)
def opt(v): return "None" if v is None else "Some("+str(v)+")"
def atomic_write(path, text):
    temporary=path.with_suffix(path.suffix+'.tmp')
    temporary.write_text(text);temporary.replace(path)
def render(data):
    out = ["// Generated by tools/update_opponent_data.py; do not edit.", "use super::{Reference, MoveData, Span, Stat, Physics};", "pub static REFERENCES: &[Reference] = &["]
    for f in data["fighters"]:
        values = {s['key']: s['value'] for s in f['stats']}
        keys = dict(weight='weight.weight',walk_speed='walkspeed.walkspeed',initial_dash='dashandrun.initialdash',run_speed='dashandrun.runspeed',air_speed='airspeed.airspeed',gravity='gravityt.gravity',fall_speed='fallspeed.regularfall',fast_fall_speed='fallspeed.fastfall',full_hop_height='jumpheight.fullhop',short_hop_height='jumpheight.shorthop',air_accel_base='airacceleration.base',air_accel_add='airacceleration.additional',air_accel_max='airacceleration.max')
        physics = ','.join(k+': '+('Some('+str(float(values[v]))+')' if re.fullmatch(r'\d+(?:\.\d+)?',values.get(v,'')) else 'None') for k,v in keys.items())
        out.append(f'Reference {{ kind: {f["kind"]}, name: {quote(f["name"])}, source: {quote(f["source"])}, physics: Physics {{ {physics} }}, stats: &[')
        out += [f'Stat {{ key: {quote(s["key"])}, value: {quote(s["value"])} }},' for s in f["stats"]]
        out.append('], moves: &[')
        for m in f["moves"]:
            active = ",".join(f"Span {{ first: {a}, last: {b} }}" for a,b in m["active"])
            raw = ",".join(f"Stat {{ key: {quote(k)}, value: {quote(v)} }}" for k,v in m["raw"].items())
            motions = ",".join(str((len(x.encode())<<32)|zlib.crc32(x.encode())) for x in m["motions"])
            out.append(f'MoveData {{ name: {quote(m["name"])}, total: {opt(m["total"])}, landing_lag: {opt(m["landing_lag"])}, active: &[{active}], motions: &[{motions}], whiff_eligible: {str(m["whiff_eligible"]).lower()}, raw: &[{raw}] }},')
        out.append('] },')
    out.append('];\n')
    return "\n".join(out)

def main():
    p=argparse.ArgumentParser(); p.add_argument("--fetch",action="store_true"); p.add_argument("--stats-only",action="store_true"); p.add_argument("--check",action="store_true"); p.add_argument("--date",default=datetime.now(timezone.utc).date().isoformat()); args=p.parse_args()
    if args.fetch or args.stats_only:
        if args.check: p.error("--fetch and --check are separate operations")
        data=fetch(args.date,args.stats_only); SNAPSHOT.parent.mkdir(parents=True,exist_ok=True)
        render(data)
        atomic_write(SNAPSHOT,json.dumps(data,ensure_ascii=False,indent=2)+"\n")
    else: data=json.loads(SNAPSHOT.read_text())
    text=render(data)
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text()!=text: raise SystemExit("Opponent generated data is stale")
    else:
        OUTPUT.parent.mkdir(parents=True,exist_ok=True); atomic_write(OUTPUT,text)
    print(f'{len(data["fighters"])} exact-kind reference profiles, {sum(len(f["moves"]) for f in data["fighters"])} move entries, {len(text.encode())} generated bytes')
    missing=[f['name'] for f in data['fighters'] if not f['stats']]
    if missing: print("No matched stats: "+", ".join(missing))
if __name__ == "__main__": main()
