#!/usr/bin/env python3
"""
Smush Info Live Stats Dashboard
-------------------------------
Listens for newline-delimited JSON from the smush_info Skyline plugin and shows
EVERYTHING it sends: per-player cards, a live table of every field, charts of any
numeric field over time, an event log, and the raw packet stream. Sessions can be
recorded to JSONL files for later analysis.

Run:   python live_smash_info.py --switch-ip <switch-ip> [--tcp-port 4242] [--web-port 8080]
       python live_smash_info.py --listen
"""
import argparse
import asyncio
import json
import re
import socket
import time
from collections import deque
from datetime import datetime
from pathlib import Path

from nicegui import app, ui

# ----------------------------------------------------------------------------
# Config
# ----------------------------------------------------------------------------
parser = argparse.ArgumentParser()
parser.add_argument("--tcp-port", type=int, default=4242)
parser.add_argument("--web-port", type=int, default=8080)
parser.add_argument("--switch-ip", help="IP of the Switch running smush_info")
parser.add_argument("--listen", action="store_true",
                    help="Listen for the plugin to connect to this Mac instead of connecting to the Switch")
args, _ = parser.parse_known_args()
if not args.listen and not args.switch_ip:
    parser.error("--switch-ip is required unless --listen is selected")

SWITCH_IP = args.switch_ip       # the Switch hosts the smush_info TCP server
LISTEN_MODE = args.listen
LISTEN_HOST = "0.0.0.0"          # only used with --listen
TCP_PORT = args.tcp_port
WEB_PORT = args.web_port
SESSION_DIR = Path(__file__).parent / "sessions"

# Heuristics (tweak to match smush_info's real field names)
PLAYER_RE = re.compile(
    r"(?:^|\.)(?:p|player|players|fighter|fighters|entry|entries)_?\[?(\d+)\]?(?:\.(.+))?$", re.I)
DAMAGE_RE = re.compile(r"damage|percent", re.I)
STOCK_RE = re.compile(r"stock", re.I)
EVENT_RE = re.compile(r"stock|stage|status|state|action|kind|character|fighter|mode|winner|result|game", re.I)


# ----------------------------------------------------------------------------
# Helpers
# ----------------------------------------------------------------------------
def flatten(obj, prefix=""):
    """Turn nested JSON into {'players[0].damage': 12.0, ...}."""
    out = {}
    if isinstance(obj, dict):
        if not obj and prefix:
            out[prefix] = "{}"
        for k, v in obj.items():
            out.update(flatten(v, f"{prefix}.{k}" if prefix else str(k)))
    elif isinstance(obj, list):
        if not obj and prefix:
            out[prefix] = "[]"
        for i, v in enumerate(obj):
            out.update(flatten(v, f"{prefix}[{i}]"))
    else:
        out[prefix or "(value)"] = obj
    return out


def is_num(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def type_name(v):
    if v is None:
        return "null"
    if isinstance(v, bool):
        return "bool"
    return type(v).__name__


def fmt(v, limit=120):
    if isinstance(v, float):
        return f"{v:.3f}".rstrip("0").rstrip(".")
    s = str(v)
    return s if len(s) <= limit else s[:limit] + "…"


def local_ips():
    ips = set()
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        s.connect(("10.255.255.255", 1))  # no packet is actually sent
        ips.add(s.getsockname()[0])
        s.close()
    except OSError:
        pass
    return sorted(ips) or ["127.0.0.1"]


class Feed:
    """Ring buffer with sequence numbers so each browser tab can pull only new lines."""

    def __init__(self, n):
        self.items = deque(maxlen=n)
        self.seq = 0

    def add(self, text):
        self.seq += 1
        self.items.append((self.seq, text))

    def since(self, seq, limit=25):
        new = [t for s, t in self.items if s > seq]
        return new[-limit:], self.seq


class KeyStat:
    __slots__ = ("last", "count", "changes", "last_change", "vmin", "vmax")

    def __init__(self, val, now):
        self.last = val
        self.count = 0
        self.changes = 0
        self.last_change = now
        self.vmin = val if is_num(val) else None
        self.vmax = val if is_num(val) else None


class Derived:
    """Running match statistics for one player."""

    def __init__(self):
        self.dmg = None
        self.stocks = None
        self.peak = 0.0
        self.hits_taken = 0
        self.dmg_taken = 0.0
        self.stocks_lost = 0


# ----------------------------------------------------------------------------
# Data store
# ----------------------------------------------------------------------------
class Store:
    def __init__(self):
        self.raw = Feed(300)
        self.events = Feed(500)
        self.recording = True
        self._fh = None
        self.session_path = None
        self.connections = 0
        self.peers = []
        self.reset()

    def reset(self):
        self.t0 = time.time()
        self.packets = 0
        self.bad_packets = 0
        self.bytes = 0
        self.recent = deque(maxlen=600)
        self.last_packet = None
        self.stats = {}      # path -> KeyStat
        self.history = {}    # path -> deque[(t, v)] (recorded on change)
        self.players = {}    # idx -> {field: value}
        self.globals = {}    # non-player fields
        self.derived = {}    # idx -> Derived

    # ---- connection lifecycle
    def on_connect(self, peer):
        self.connections += 1
        self.peers.append(str(peer))
        self.events.add(f"🔌 Connected: {peer}")
        if self.recording:
            self._open_file()

    def on_disconnect(self, peer):
        self.connections = max(0, self.connections - 1)
        if str(peer) in self.peers:
            self.peers.remove(str(peer))
        self.events.add(f"❌ Disconnected: {peer}")
        if self.connections == 0:
            self._close_file()

    def _open_file(self):
        if self._fh:
            return
        SESSION_DIR.mkdir(exist_ok=True)
        self.session_path = SESSION_DIR / f"session_{datetime.now():%Y%m%d_%H%M%S}.jsonl"
        self._fh = open(self.session_path, "a", encoding="utf-8")

    def _close_file(self):
        if self._fh:
            self._fh.close()
            self._fh = None

    def set_recording(self, on):
        self.recording = on
        if not on:
            self._close_file()
        elif self.connections:
            self._open_file()

    # ---- ingest
    def ingest_line(self, line: bytes):
        text = line.decode("utf-8", errors="replace").strip()
        if not text:
            return
        self.raw.add(text[:1500])
        try:
            data = json.loads(text)
        except json.JSONDecodeError:
            self.bad_packets += 1
            self.events.add(f"⚠ Non-JSON line: {text[:100]}")
            return
        self.ingest(data, len(line))

    def ingest(self, data, nbytes):
        now = time.time()
        self.packets += 1
        self.bytes += nbytes
        self.recent.append(now)
        self.last_packet = now

        flat = flatten(data)
        players, globals_ = {}, {}
        for path, val in flat.items():
            self._track(path, val, now)
            m = PLAYER_RE.search(path)
            if m:
                players.setdefault(m.group(1), {})[m.group(2) or "(value)"] = val
            else:
                globals_[path] = val
        self.players, self.globals = players, globals_
        for idx, fields in players.items():
            self._derive(idx, fields)

        if self.recording and self._fh:
            self._fh.write(json.dumps({"t": round(now - self.t0, 4), "data": data}) + "\n")
            self._fh.flush()

    def _track(self, path, val, now):
        st = self.stats.get(path)
        first = st is None
        if first:
            st = self.stats[path] = KeyStat(val, now)
        changed = (not first) and val != st.last
        st.count += 1
        if changed:
            st.changes += 1
            st.last_change = now
            if (EVENT_RE.search(path) and not isinstance(val, float)) or isinstance(val, (str, bool)) or val is None:
                self.events.add(f"{path}: {fmt(st.last, 40)} → {fmt(val, 40)}")
        if is_num(val):
            st.vmin = val if st.vmin is None else min(st.vmin, val)
            st.vmax = val if st.vmax is None else max(st.vmax, val)
            if first or changed:
                h = self.history.setdefault(path, deque(maxlen=3000))
                h.append((now - self.t0, val))
        st.last = val

    def _derive(self, idx, fields):
        d = self.derived.setdefault(idx, Derived())
        dmg = next((v for k, v in fields.items() if DAMAGE_RE.search(k) and is_num(v)), None)
        stk = next((v for k, v in fields.items() if STOCK_RE.search(k) and is_num(v)), None)
        if dmg is not None:
            if d.dmg is not None and dmg > d.dmg:
                delta = dmg - d.dmg
                d.hits_taken += 1
                d.dmg_taken += delta
                if delta >= 1:
                    self.events.add(f"💥 P{idx} took {delta:.1f}% → {dmg:.1f}%")
            d.peak = max(d.peak, dmg)
            d.dmg = dmg
        if stk is not None:
            if d.stocks is not None and stk < d.stocks:
                d.stocks_lost += d.stocks - stk
                self.events.add(f"☠ P{idx} lost a stock ({stk} left, peak {d.peak:.0f}%)")
            d.stocks = stk

    # ---- views
    def pps(self):
        now = time.time()
        return sum(1 for t in self.recent if now - t <= 2.0) / 2.0

    def dealt_by(self, idx):
        return sum(d.dmg_taken for k, d in self.derived.items() if k != idx)


store = Store()


# ----------------------------------------------------------------------------
# TCP server (started ONCE at app startup, not per page load)
# ----------------------------------------------------------------------------
async def handle_client(reader, writer):
    peer = writer.get_extra_info("peername")
    store.on_connect(peer)
    try:
        while True:
            line = await reader.readline()
            if not line:
                break
            store.ingest_line(line)
    except (ConnectionError, asyncio.IncompleteReadError):
        pass
    except ValueError as e:  # line longer than the stream limit
        store.events.add(f"⚠ Stream error: {e}")
    finally:
        writer.close()
        store.on_disconnect(peer)


_tasks = []


async def connect_loop():
    """Connect to the Switch and keep reconnecting (game closed, Switch asleep, etc.)."""
    announced = False
    while True:
        try:
            reader, writer = await asyncio.open_connection(SWITCH_IP, TCP_PORT, limit=2 ** 24)
        except OSError as e:
            if not announced:
                store.events.add(f"⏳ Can't reach {SWITCH_IP}:{TCP_PORT} ({e.strerror or e}); retrying…")
                announced = True
            await asyncio.sleep(2)
            continue
        announced = False
        await handle_client(reader, writer)  # returns when the connection drops
        await asyncio.sleep(1)


async def start_tcp():
    if LISTEN_MODE:
        await asyncio.start_server(handle_client, LISTEN_HOST, TCP_PORT, limit=2 ** 24)
        ips = ", ".join(f"{ip}:{TCP_PORT}" for ip in local_ips())
        print(f"📡 Listening on {LISTEN_HOST}:{TCP_PORT} (point the plugin at: {ips})")
    else:
        _tasks.append(asyncio.create_task(connect_loop()))
        print(f"🔗 Connecting to smush_info on the Switch at {SWITCH_IP}:{TCP_PORT} (auto-retry)")


app.on_startup(start_tcp)
app.on_shutdown(store._close_file)


# ----------------------------------------------------------------------------
# UI
# ----------------------------------------------------------------------------
PLAYER_BORDERS = ["border-blue-500", "border-red-500", "border-green-500", "border-yellow-400",
                  "border-purple-500", "border-pink-500", "border-cyan-400", "border-orange-500"]

DATA_COLUMNS = [
    {"name": "key", "label": "Field", "field": "key", "align": "left", "sortable": True},
    {"name": "value", "label": "Current value", "field": "value", "align": "left", "sortable": True},
    {"name": "type", "label": "Type", "field": "type", "sortable": True},
    {"name": "min", "label": "Min", "field": "min", "sortable": True},
    {"name": "max", "label": "Max", "field": "max", "sortable": True},
    {"name": "changes", "label": "Changes", "field": "changes", "sortable": True},
    {"name": "updates", "label": "Updates", "field": "updates", "sortable": True},
    {"name": "age", "label": "Last change (s ago)", "field": "age", "sortable": True},
]
FIELD_COLUMNS = [
    {"name": "key", "label": "Field", "field": "key", "align": "left"},
    {"name": "value", "label": "Value", "field": "value", "align": "right"},
]


@ui.page("/")
def main_dashboard():
    ui.dark_mode(True)
    ui.query("body").classes("bg-slate-900 text-slate-100")
    ui_state = {"ids": None, "cards": {}, "raw_seq": 0, "ev_seq": 0, "chart_keys": None, "auto_picked": False}

    with ui.column().classes("w-full max-w-7xl mx-auto p-4 gap-4"):
        # ---- Header
        with ui.row().classes("w-full justify-between items-center bg-slate-800 p-4 rounded-xl shadow-lg"):
            with ui.row().classes("items-center gap-3"):
                ui.label("🎮 SMUSH INFO STATS").classes("text-xl font-black tracking-wider text-amber-400")
                status_badge = ui.badge("Waiting…", color="red").classes("px-3 py-1 font-bold")
            header_lbl = ui.label("").classes("text-sm font-mono text-slate-400")
            with ui.row().classes("items-center gap-3"):
                ui.switch("Record JSONL", value=store.recording,
                          on_change=lambda e: store.set_recording(e.value))

                def download_session():
                    p = store.session_path
                    if p and Path(p).exists():
                        try:
                            ui.download(Path(p).read_bytes(), Path(p).name)
                        except Exception as ex:  # noqa: BLE001
                            ui.notify(f"Download failed: {ex}", type="negative")
                    else:
                        ui.notify("No session file yet", type="warning")

                ui.button("Download session", on_click=download_session).props("outline dense")
                ui.button("Reset stats", on_click=lambda: (store.reset(), ui.notify("Stats reset"))) \
                    .props("outline dense color=red")
        listen_lbl = ui.label("").classes("text-xs text-slate-500 -mt-2")

        # ---- Tabs
        with ui.tabs().classes("w-full") as tabs:
            t_over = ui.tab("Overview")
            t_data = ui.tab("All Data")
            t_chart = ui.tab("Charts")
            t_events = ui.tab("Events")
            t_raw = ui.tab("Raw Stream")

        with ui.tab_panels(tabs, value=t_over).classes("w-full bg-transparent"):
            # Overview
            with ui.tab_panel(t_over).classes("p-0 gap-4"):
                players_row = ui.row().classes("w-full gap-4 items-stretch no-wrap overflow-x-auto")
                ui.label("Global / match fields").classes("text-xs font-bold uppercase tracking-widest text-slate-500 mt-4")
                globals_table = ui.table(columns=FIELD_COLUMNS, rows=[], row_key="key") \
                    .props("dense flat hide-header hide-pagination").classes("w-full bg-slate-800")
                globals_table.pagination = 0

            # All Data
            with ui.tab_panel(t_data).classes("p-0"):
                search = ui.input("Filter fields…").props("dense outlined clearable").classes("w-72 mb-2")
                data_table = ui.table(columns=DATA_COLUMNS, rows=[], row_key="key") \
                    .props("dense flat").classes("w-full bg-slate-800")
                data_table.pagination = 0
                search.bind_value_to(data_table, "filter")

            # Charts
            with ui.tab_panel(t_chart).classes("p-0"):
                chart_select = ui.select([], multiple=True, label="Numeric fields to plot", with_input=True) \
                    .props("use-chips outlined dense").classes("w-full")
                chart = ui.echart({
                    "backgroundColor": "transparent",
                    "tooltip": {"trigger": "axis"},
                    "legend": {"textStyle": {"color": "#cbd5e1"}},
                    "grid": {"left": 50, "right": 30, "bottom": 60},
                    "xAxis": {"type": "value", "name": "seconds", "axisLabel": {"color": "#94a3b8"}},
                    "yAxis": {"type": "value", "scale": True, "axisLabel": {"color": "#94a3b8"}},
                    "dataZoom": [{"type": "inside"}, {"type": "slider"}],
                    "series": [],
                }).classes("w-full h-96")

            # Events
            with ui.tab_panel(t_events).classes("p-0"):
                events_log = ui.log(max_lines=500).classes(
                    "w-full h-96 bg-slate-950 border border-slate-800 rounded-lg p-3 font-mono text-xs text-amber-300")

            # Raw
            with ui.tab_panel(t_raw).classes("p-0"):
                raw_log = ui.log(max_lines=300).classes(
                    "w-full h-96 bg-slate-950 border border-slate-800 rounded-lg p-3 font-mono text-xs text-emerald-400")

    # ---- Refresh logic ------------------------------------------------------
    def rebuild_cards(ids):
        players_row.clear()
        ui_state["cards"] = {}
        ui_state["ids"] = ids
        with players_row:
            if not ids:
                ui.label("No per-player fields detected yet. Check the All Data tab to see what's arriving.") \
                    .classes("text-slate-500 text-sm")
            for n, idx in enumerate(ids):
                with ui.card().classes(f"flex-1 min-w-72 bg-slate-800 border-l-8 {PLAYER_BORDERS[n % len(PLAYER_BORDERS)]} p-4 rounded-xl shadow-2xl items-center"):
                    ui.label(f"Player {idx}").classes("text-lg font-black tracking-wide text-slate-300")
                    pct = ui.label("–").classes("text-6xl font-extrabold my-1 transition-colors duration-150")
                    stocks = ui.label("").classes("text-xl tracking-widest")
                    stats = ui.label("").classes("text-xs text-slate-300 text-center bg-slate-900/60 px-3 py-1 rounded-lg")
                    tbl = ui.table(columns=FIELD_COLUMNS, rows=[], row_key="key") \
                        .props("dense flat hide-header hide-pagination").classes("w-full bg-transparent")
                    tbl.pagination = 0
                    ui_state["cards"][idx] = {"pct": pct, "stocks": stocks, "stats": stats, "tbl": tbl}

    def refresh():
        # Header
        connected = store.connections > 0
        status_badge.set_text("Connected" if connected else "Disconnected")
        status_badge.props(f'color={"green" if connected else "red"}')
        age = f"{time.time() - store.last_packet:.1f}s ago" if store.last_packet else "never"
        header_lbl.set_text(
            f"{store.packets:,} packets · {store.pps():.0f}/s · {store.bytes / 1024:.1f} KB · "
            f"{len(store.stats)} fields · {store.bad_packets} bad · last {age}")
        rec = f" · recording → {Path(store.session_path).name}" if (store._fh and store.session_path) else ""
        where = (f"Listening on {', '.join(f'{ip}:{TCP_PORT}' for ip in local_ips())}" if LISTEN_MODE
                 else f"Connecting to Switch {SWITCH_IP}:{TCP_PORT}")
        listen_lbl.set_text(f"{where}{' · peers: ' + ', '.join(store.peers) if store.peers else ''}{rec}")

        # Feeds (cheap, always on)
        new, ui_state["ev_seq"] = store.events.since(ui_state["ev_seq"])
        for line in new:
            events_log.push(f"[{datetime.now():%H:%M:%S}] {line}")
        new, ui_state["raw_seq"] = store.raw.since(ui_state["raw_seq"], limit=15)
        for line in new:
            raw_log.push(line)

        tab = tabs.value
        # Overview
        if tab == "Overview":
            ids = sorted(store.players, key=lambda x: int(x))
            if ids != ui_state["ids"]:
                rebuild_cards(ids)
            for idx in ids:
                c, d = ui_state["cards"][idx], store.derived.get(idx)
                if d is None:
                    continue
                if d.dmg is not None:
                    c["pct"].set_text(f"{d.dmg:.1f}%")
                    color = "text-red-500" if d.dmg > 100 else ("text-orange-400" if d.dmg > 50 else "text-white")
                    c["pct"].classes(replace=f"text-6xl font-extrabold my-1 {color}")
                if d.stocks is not None:
                    c["stocks"].set_text("● " * max(0, int(d.stocks)) or "—")
                c["stats"].set_text(
                    f"Peak {d.peak:.0f}% · Hits taken {d.hits_taken} · Dmg taken {d.dmg_taken:.0f}% · "
                    f"Dmg dealt {store.dealt_by(idx):.0f}% · Stocks lost {d.stocks_lost}")
                c["tbl"].rows = [{"key": k, "value": fmt(v)} for k, v in store.players[idx].items()]
                c["tbl"].update()
            globals_table.rows = [{"key": k, "value": fmt(v)} for k, v in store.globals.items()]
            globals_table.update()

        # All data
        elif tab == "All Data":
            now = time.time()
            rows = []
            for k, st in store.stats.items():
                rows.append({
                    "key": k, "value": fmt(st.last), "type": type_name(st.last),
                    "min": fmt(st.vmin) if st.vmin is not None else "",
                    "max": fmt(st.vmax) if st.vmax is not None else "",
                    "changes": st.changes, "updates": st.count, "age": round(now - st.last_change, 1),
                })
            data_table.rows = rows
            data_table.update()

        # Charts
        elif tab == "Charts":
            keys = sorted(store.history)
            if keys != ui_state["chart_keys"]:
                ui_state["chart_keys"] = keys
                chart_select.set_options(keys, value=chart_select.value)
            if not ui_state["auto_picked"] and keys:
                pick = [k for k in keys if DAMAGE_RE.search(k)][:4] or keys[:3]
                chart_select.value = pick
                ui_state["auto_picked"] = True
            now_t = time.time() - store.t0
            series = []
            for k in chart_select.value or []:
                pts = [[round(t, 2), v] for t, v in store.history.get(k, [])]
                if pts:
                    pts.append([round(now_t, 2), pts[-1][1]])  # extend flat line to "now"
                series.append({"type": "line", "name": k, "step": "end", "showSymbol": False, "data": pts})
            chart.options["series"] = series
            chart.update()

    ui.timer(0.5, refresh)


ui.run(title="Smush Info Stats", port=WEB_PORT, dark=True, reload=False, show=False)