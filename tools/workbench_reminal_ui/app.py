from __future__ import annotations

from collections import deque
import json
from pathlib import Path
import re
import time
import csv
import math
from copy import deepcopy

from PIL import Image, ImageDraw
from rich_pixels import Pixels

from PIL import Image, ImageDraw
from rich_pixels import Pixels
from textual import work
from textual.app import App, ComposeResult
from rich.text import Text
from textual.theme import Theme
from textual.containers import Horizontal, HorizontalScroll, Vertical, VerticalScroll
from textual.widgets import (Button, Checkbox, Collapsible, DataTable, Footer, Header, Input,
                             Label, Select, Sparkline, Static, TabbedContent, TabPane, TextArea)

from textual_plotext import PlotextPlot

import workbench
import sequence_sources
from .backend import Backend
from .widgets import Confirm, LogPane
from .telemetry import assistance_summary, observation_rows, schema_key

ENDPOINTS = [(label, value) for label, value in (
    ('Local file', 'local'), ('Silent · active', 'silent'),
    ('Diagnostic · active', 'diagnostic'), ('Silent · staged', 'silent-staged'),
    ('Diagnostic · staged', 'diagnostic-staged'))]


def mascot():
    image = Image.new('RGB', (36, 20), '#101827')
    draw = ImageDraw.Draw(image)
    draw.ellipse((2, 12, 17, 18), fill='#e54d88')
    draw.ellipse((19, 12, 34, 18), fill='#e54d88')
    draw.ellipse((6, 1, 30, 17), fill='#f4a7c6')
    draw.ellipse((13, 5, 15, 11), fill='#18203b')
    draw.ellipse((21, 5, 23, 11), fill='#18203b')
    draw.point((14, 6), fill='white')
    draw.point((22, 6), fill='white')
    draw.arc((16, 10, 21, 14), 0, 180, fill='#8d3657')
    return Pixels.from_image(image)


def mascot():
    image = Image.new('RGB', (36, 20), '#101827')
    draw = ImageDraw.Draw(image)
    draw.ellipse((2, 12, 17, 18), fill='#e54d88')
    draw.ellipse((19, 12, 34, 18), fill='#e54d88')
    draw.ellipse((6, 1, 30, 17), fill='#f4a7c6')
    draw.ellipse((13, 5, 15, 11), fill='#18203b')
    draw.ellipse((21, 5, 23, 11), fill='#18203b')
    draw.point((14, 6), fill='white')
    draw.point((22, 6), fill='white')
    draw.arc((16, 10, 21, 14), 0, 180, fill='#8d3657')
    return Pixels.from_image(image)


def field(label, name, value='', *, password=False):
    with Horizontal(classes='form-row'):
        yield Label(label)
        yield Input(str(value), id=name, password=password)


def endpoint(label, name, value='local'):
    with Horizontal(classes='form-row'):
        yield Label(label)
        yield Select(ENDPOINTS, value=value if value in dict(ENDPOINTS).values() else 'local',
                     allow_blank=False, id=name)


def buttons(*items):
    with HorizontalScroll(classes='buttons'):
        for name, label in items:
            symbol = ('↗' if name in {'ship', 'upload'} else '✓' if name == 'verify' or 'validate' in name
                      else '≡' if 'preview' in name else '↓' if 'export' in name else '↻' if 'load' in name
                      else '■' if name.endswith('stop') or name.endswith('disconnect') else '›')
            yield Button(f'{symbol} {label}', id=name, variant='primary' if name.endswith('preview') else 'default')


class TerminalWorkbench(App):
    TITLE = 'Kirby Assist · Terminal Workbench'
    CSS_PATH = 'app.tcss'
    BINDINGS = [('ctrl+q', 'safe_quit', 'Quit safely'), ('ctrl+l', 'current_log', 'Current log'),
                ('ctrl+o', 'toggle_output', 'Toggle output'),
                ('f1', 'help', 'Help'), ('ctrl+s', 'preview_draft', 'Preview draft')]

    def __init__(self, backend_app, *, backend=None):
        super().__init__()
        self.backend = backend or Backend(backend_app)
        self.model = backend_app
        self.running_operation = False
        self.result_id = None
        self.history_fingerprint = None
        self.history_selection = None
        self.editor_plan = None
        self.loaded_editor_fighter = None
        self.loaded_editor_form = None
        self.editor_preview_text = None
        self.transfer_plan = None
        self.transfer_form = None
        self.split_plan = None
        self.split_form = None
        self.lan_preview_text = None
        self.telemetry_schema = {}
        self.telemetry_data = {}
        self.telemetry_received = None
        self.receiver_status = 'Receiver not started'
        self.receiver_available = False
        self.telemetry_samples = deque(maxlen=120)
        self.telemetry_frozen = None
        self.telemetry_rows = []
        self.telemetry_projected = []
        self.selected_telemetry_field = None
        self.field_samples = deque(maxlen=120)
        self.telemetry_object = None
        self.telemetry_previous = {}
        self.telemetry_changes = {}
        self.telemetry_session = None
        self.telemetry_selection_options = None
        self.closing = False
        self.operation_status = None
        self.polling = False
        self.poll_error = None

    def compose(self) -> ComposeResult:
        p = self.model.preferences
        default = str(workbench.DEFAULT_FILE)
        yield Header()
        yield Static('Ready · no build, upload, receiver or device connection on startup', id='status', markup=False)
        with TabbedContent(id='menu'):
            with TabPane('◆ Overview', id='overview'):
                with VerticalScroll():
                    yield Static('◆ KIRBY ASSIST / WORKBENCH\nDeploy · Sequences · Training monitors · Activity · Helpers', id='welcome-text')
                    with Horizontal(id='outcome-cards'):
                        for name, caption in [('runs', '≡ Recorded'), ('success', '✓ Successful'), ('failed', '! Failed'), ('other', '◷ Other')]:
                            with Vertical(classes='stat-card', id=f'card-{name}'):
                                yield Label(caption)
                                yield Static('0', id=f'outcome-{name}', markup=False)
                    yield Static(f'Project: {workbench.PROJECT_ROOT}\nJournal: {self.model.history.directory}', markup=False)
                    yield Static('', id='overview-state', markup=False)
                    yield Label('Recorded outcomes · recent 30 operations', classes='section-title')
                    yield PlotextPlot(id='activity-chart')
                    yield Static('Safety: opening this app does not start Cargo or connect to a Switch.\n'
                                 'Build writes the requested version to Cargo.toml. Upload may synchronize Silent sources.\n'
                                 'All replacements retain the existing backend validation, backups and verification.\n'
                                 'Ctrl+S previews the current editor draft. Ctrl+Q waits for active work.', markup=False)
            with TabPane('↗ Deploy', id='deploy'):
                with VerticalScroll():
                    yield Label('Release pipeline', classes='section-title')
                    yield from field('Release version (written only when you build)', 'version', workbench.version() or '')
                    yield from buttons(('bump', 'Increment patch'))
                    yield Label('Target')
                    yield Select([('Both Switches', 'all'), ('Silent only', 'silent'), ('Diagnostic only', 'diagnostic')],
                                 value=p.get('gui_target', 'all'), allow_blank=False, id='target')
                    yield Static('Diagnostic-only uploads preserve sequences. Other uploads use the authoritative Silent snapshot.', markup=False)
                    yield from buttons(('ship', 'Build + upload…'), ('build', 'Build only…'),
                                       ('upload', 'Upload built…'), ('verify', 'Verify installed'))
            with TabPane('≡ Sequences', id='sequences'):
                with TabbedContent(id='sequence-menu'):
                    with TabPane('Fighter editor', id='fighter-editor'):
                        with VerticalScroll():
                            yield from endpoint('Read from', 'editor_endpoint', p.get('gui_editor_endpoint', 'local'))
                            yield from field('Local sequence file', 'editor_file', p.get('gui_editor_file', default))
                            yield from buttons(('editor-load', 'Read file'))
                            yield Label('Fighter in loaded snapshot')
                            yield Select([], id='editor_fighter', prompt='Load a file first')
                            yield from buttons(('editor-fighter', 'Open fighter'), ('editor-preview', 'Preview changes'),
                                               ('editor-commit', 'Save fighter…'), ('editor-discard', 'Revert draft'))
                            yield TextArea('', id='editor_text', show_line_numbers=True, tab_behavior='indent')
                            yield TextArea('', id='editor_diff', read_only=True)
                    with TabPane('Transfer & validate', id='transfer'):
                        with VerticalScroll():
                            yield from endpoint('Source', 'source', p.get('gui_source', 'local'))
                            yield from field('Source local file', 'source_file', p.get('gui_source_file', default))
                            yield from endpoint('Destination', 'destination', p.get('gui_destination', 'silent'))
                            yield from field('Destination local file', 'destination_file', p.get('gui_destination_file', default))
                            yield from field('Fighter (blank transfers whole file)', 'fighter', p.get('gui_fighter', ''))
                            yield from buttons(('validate', 'Validate local source'), ('preview', 'Preview transfer'), ('commit', 'Replace destination…'))
                            yield TextArea('', id='transfer_diff', read_only=True)
                    with TabPane('Split source files', id='split'):
                        with VerticalScroll():
                            yield from endpoint('Combined file source', 'breakdown_endpoint', p.get('gui_breakdown_endpoint', 'silent'))
                            yield from field('Local combined file', 'breakdown_file', p.get('gui_breakdown_file', default))
                            yield from field('Per-fighter destination', 'breakdown_destination', p.get('gui_breakdown_destination', str(sequence_sources.SOURCE)))
                            yield from buttons(('breakdown-preview', 'Preview split'), ('breakdown-commit', 'Write source files…'))
                            yield TextArea('', id='split_diff', read_only=True)
            with TabPane('◈ Monitor', id='monitor'):
                with TabbedContent(id='monitor-menu'):
                    with TabPane('Training LAN editor', id='lan-editor'):
                        with VerticalScroll():
                            yield Static('Diagnostic Training only. Connect explicitly; complete-file save requires validation and exact-draft confirmation.', markup=False)
                            yield from field('Numeric private Switch IPv4', 'lan_host', p.get('lan_editor_host', ''))
                            yield from field('LAN port', 'lan_port', p.get('lan_editor_port', 4242))
                            yield Label('Transport')
                            yield Select([('HTTP', 'http'), ('UDP', 'udp')], value='udp', allow_blank=False, id='lan_transport')
                            yield from field('Optional shared token (never persisted)', 'lan_token', '', password=True)
                            yield from buttons(('lan-connect', 'Connect explicitly'), ('lan-disconnect', 'Disconnect'),
                                               ('lan-load', 'Load complete file'), ('lan-preview', 'Validate + preview'), ('lan-save', 'Save + reload…'))
                            yield Static('', id='lan_status', markup=False)
                            yield TextArea('', id='lan_text', show_line_numbers=True, tab_behavior='indent')
                            yield TextArea('', id='lan_diff', read_only=True)
                    with TabPane('◈ Telemetry', id='telemetry'):
                        with VerticalScroll():
                            yield Static('Read-only · trusted private LAN · default packets are unauthenticated and unencrypted. Receiver startup is explicit; Switch streaming is never activated here.', classes='safety-note', markup=False)
                            yield from field('Switch IPv4:port', 'telemetry_endpoint', '192.168.1.142:4243')
                            yield from buttons(('telemetry-start', 'Start receiver…'), ('telemetry-stop', 'Stop receiver'), ('telemetry-connect', 'Subscribe…'))
                            yield Static('Receiver not started', id='telemetry_status', markup=False)
                            yield Static('Packets/s — · KiB/s — · Objects 0 · Gaps 0', id='telemetry_metrics', markup=False)
                            with Horizontal(classes='metric-trends'):
                                yield Sparkline([], id='packet-trend')
                                yield Sparkline([], id='byte-trend')
                            with Horizontal(classes='form-row'):
                                yield Label('Object / publication')
                                yield Select([('All objects', '*')], value='*', allow_blank=False, id='telemetry_object_select')
                                yield Select([('All groups', '*')], value='*', allow_blank=False, id='telemetry_group')
                            with HorizontalScroll(classes='buttons'):
                                yield Checkbox('Freeze values', id='telemetry_freeze')
                                yield Checkbox('Schema reference', id='telemetry_schema_mode')
                                yield Checkbox('Available only', id='telemetry_available')
                            with TabbedContent(id='telemetry-pages'):
                                with TabPane('≡ Observations', id='observations'):
                                    yield Input(placeholder='Filter name, source, type, value, mode or recipe', id='telemetry_filter')
                                    yield Static('', id='telemetry_row_count', markup=False)
                                    yield DataTable(id='telemetry_table', cursor_type='row', zebra_stripes=True)
                                    with Collapsible(title='◇ Selected field · exact bits & source', collapsed=False):
                                        yield TextArea('', id='telemetry_details', read_only=True)
                                        yield from buttons(('telemetry-copy', 'Copy exact raw bits'))
                                    with Collapsible(title='↓ Export visible observations'):
                                        yield Input(placeholder='New CSV file path · existing files are never overwritten', id='telemetry_export_path')
                                        yield from buttons(('telemetry-export', 'Export visible CSV'))
                                with TabPane('◆ Assistance', id='assistance'):
                                    yield Input(placeholder='Find assistance tool', id='feature_filter')
                                    yield Checkbox('Only enabled / proposing tools', id='feature_active')
                                    yield TextArea('', id='fighter_summary', read_only=True)
                                    yield DataTable(id='feature_table', cursor_type='row', zebra_stripes=True)
                                    yield Static('Selection is a proposal, not delivery. Final mapper output is separate evidence, not proof Smash executed the action. Short pulses can be missed.', classes='safety-note', markup=False)
                                with TabPane('▥ Charts', id='charts'):
                                    yield Select([('Accepted packets / second', 'packets_per_second'), ('Accepted payload KiB / second', 'bytes_per_second')],
                                                 value='packets_per_second', allow_blank=False, id='telemetry_metric')
                                    yield PlotextPlot(id='telemetry-chart')
                                    yield Select([('Select a numeric field', '')], value='', allow_blank=False, id='telemetry_graph')
                                    yield PlotextPlot(id='field-chart')
                                    yield from buttons(('telemetry-clear-charts', 'Clear charts'))
                                    yield Static('Receiver time · not game FPS or network latency. Gaps are unavailable/stale observations; missing values are never plotted as zero.', markup=False)
                                with TabPane('◇ Reference', id='wire-reference'):
                                    yield Static('', id='telemetry_diagnostics', markup=False)
                                    yield TextArea('', id='telemetry_schema', read_only=True)
                                    yield Static('Fresh <250 ms; expired ≥1 s. Frozen values retain their snapshot while ages advance. sample_us is a sender publication timestamp, not a game frame or synchronized latency. Sequence gaps are unseen positions, not confirmed network loss; sender queue drops are separate. Object IDs are not player slots. All groups and exact wire types come from the receiver schema.', classes='safety-note', markup=False)
                    with TabPane('Live & raw logs', id='streams'):
                        with VerticalScroll():
                            yield Static('Filters, freeze and clear affect display only; durable captures continue. Export never overwrites files.', markup=False)
                            with TabbedContent(id='stream-pages'):
                                with TabPane('◆ Live game log', id='live-stream'):
                                    yield Static('', id='live_log_status', markup=False)
                                    yield LogPane('Diagnostic · live game logs', 'live-log')
                                with TabPane('◈ Raw telemetry', id='raw-telemetry-stream'):
                                    yield LogPane('Telemetry · exact UDP payload and decoded fields', 'telemetry-log')
                                with TabPane('◇ Raw events', id='raw-diagnostic-stream'):
                                    yield LogPane('Diagnostic · raw native event batches', 'diagnostic-log')
            with TabPane('◷ Activity', id='activity'):
                with VerticalScroll():
                    yield Input(placeholder='Find action, target, version or outcome', id='history_filter')
                    yield DataTable(id='history_table', cursor_type='row', zebra_stripes=True)
                    yield from buttons(('current-log', 'Return to current operation'))
                    yield TextArea('', id='history_details', read_only=True)
            with TabPane('› Helpers', id='helpers'):
                with VerticalScroll():
                    yield Static('Original helper scripts. Arguments are parsed without a shell. Review write actions before confirming.', markup=False)
                    yield Label('Helper')
                    yield Select([(name, name) for name in workbench.HELPERS], value=workbench.HELPERS[0], allow_blank=False, id='helper')
                    yield from field('Arguments', 'arguments', '--help')
                    yield from buttons(('helper-help', 'Show help'), ('helper-run', 'Run helper…'))
            with TabPane('◇ Settings', id='settings'):
                with VerticalScroll():
                    yield Label('FTP routing & build cache', classes='section-title')
                    for name in ('silent', 'diagnostic'):
                        yield from field(f'{name.title()} FTP IPv4', f'{name}_host', self.model.host(name))
                        yield from field(f'{name.title()} FTP port', f'{name}_port', self.model.port(name))
                    yield from field('Cargo cache', 'cargo_home', p.get('cargo_home', '/private/tmp/kirby-assist-build-cargo'))
                    yield from buttons(('settings-save', 'Save settings'))
                    yield Static('Settings use the same preferences as the browser GUI. Tokens and editor drafts are never saved as preferences.', markup=False)
        yield Checkbox('FTP ready: Smash closed; FTP open on the involved Switches', id='ready')
        yield LogPane('Operation output · current or selected history log', 'operation-log')
        yield Footer()

    def on_mount(self):
        self.register_theme(Theme(name='workbench-black', primary='#70d7ff', secondary='#bcacff', accent='#f3a2d6',
                                  foreground='#edf1f7', background='#08090b', surface='#101216', panel='#171a20',
                                  success='#78e2b0', warning='#ffd078', error='#ff8c9c', dark=True))
        self.theme = 'workbench-black'
        for plot in self.query(PlotextPlot):
            plot.plt.canvas_color('black')
            plot.plt.axes_color('black')
            plot.plt.ticks_color('white')
        for widget in self.query(TextArea):
            widget.theme = 'monokai'
        self.query_one('#history_table', DataTable).add_columns('Time', 'Action', 'Outcome', 'Target / version', 'Verified')
        self.query_one('#telemetry_table', DataTable).add_columns('Object', 'Group', 'Field', 'Value', 'Age / state', 'Mode / recipe')
        self.query_one('#feature_table', DataTable).add_columns('Tool', 'Preference', 'Proposal evidence', 'Mode', 'Recipe', 'Constraints', 'Purpose / limits')
        self.update_state()
        self.set_interval(0.5, self.refresh_backend)
        self.set_interval(0.2, self.refresh_output)

    def refresh_output(self):
        if self.closing or not self.query('#telemetry_table'):
            return
        if not self.screen.is_modal and self.telemetry_data:
            self.update_telemetry()
        if self.running_operation and not self.screen.is_modal:
            state = self.backend.controller.snapshot()
            self.query_one('#operation-log', LogPane).update_text(state['output'])

    def value(self, name):
        widget = self.query_one('#' + name)
        return widget.value if isinstance(widget, (Input, Select, Checkbox)) else widget.text

    def text(self, name, value):
        self.query_one('#' + name, TextArea).load_text(value)

    def common(self):
        return {key: self.value(key) for key in ('version', 'target', 'silent_host', 'silent_port',
                                                'diagnostic_host', 'diagnostic_port', 'cargo_home', 'ready')}

    def editor_values(self):
        return {k: self.value(k) for k in ('editor_endpoint', 'editor_file')}

    def invalidate_editor_source(self):
        if self.loaded_editor_form != self.editor_values():
            self.editor_plan = None
            self.loaded_editor_fighter = None

    def transfer_values(self):
        return {k: self.value(k) for k in ('source', 'source_file', 'destination', 'destination_file', 'fighter')}

    def split_values(self):
        return {k: self.value(k) for k in ('breakdown_endpoint', 'breakdown_file', 'breakdown_destination')}

    def ask_confirmation(self, title, details, callback):
        self.push_screen(Confirm(title, details), lambda confirmed: callback() if confirmed else None)

    def request_run(self, action, extra=None):
        self.execute('run', {'action': action, **self.common(), **(extra or {})})

    @work(group='operations', exit_on_error=False)
    async def execute(self, kind, payload):
        if self.running_operation or self.closing:
            self.notify('Wait for the current operation', severity='warning')
            return
        self.running_operation = True
        self.history_selection = None
        self.query_one('#status', Static).update('Running: ' + str(payload.get('action', kind)))
        self.set_operation_buttons(True)
        try:
            import asyncio
            result = await asyncio.to_thread(self.dispatch, kind, payload)
            self.consume_result(kind, payload, result)
            self.operation_status = 'Complete: ' + str(payload.get('action', kind))
            self.notify('Operation complete')
        except Exception as error:
            self.operation_status = str(error)
            self.query_one('#status', Static).update(str(error))
            self.notify(str(error), severity='error', timeout=8)
        finally:
            self.running_operation = False
            self.set_operation_buttons(False)
            self.update_state()

    def set_operation_buttons(self, disabled):
        for button in self.query(Button):
            if button.id in OPERATION_BUTTONS:
                button.disabled = disabled

    def dispatch(self, kind, payload):
        if kind == 'run':
            return self.backend.run(payload)
        if kind == 'lan':
            return self.backend.lan(payload['operation'], **{k: v for k, v in payload.items() if k != 'operation'})
        if kind == 'telemetry-start':
            return self.backend.telemetry.start()
        if kind == 'telemetry-stop':
            return self.backend.telemetry.stop()
        if kind == 'telemetry-connect':
            endpoint_value = payload['endpoint'].strip()
            if not re.fullmatch(r'\d{1,3}(?:\.\d{1,3}){3}:\d{1,5}', endpoint_value):
                raise ValueError('Use a numeric private IPv4:port')
            return self.backend.telemetry_request('/connect?endpoint=' + endpoint_value, 'POST')
        if kind == 'history':
            return self.backend.history_log(payload['id'])
        if kind == 'settings':
            for name in ('silent', 'diagnostic'):
                from upload_switch import ftp_ip
                ftp_ip(payload[name + '_host'].strip())
                port = int(payload[name + '_port'])
                if not 1 <= port <= 65535:
                    raise ValueError('FTP ports must be 1..65535')
            if payload['silent_host'].strip() == payload['diagnostic_host'].strip():
                raise ValueError('Silent and Diagnostic must have distinct addresses')
            previous = dict(self.model.preferences)
            try:
                self.model.preferences.update(payload)
                self.model.save_preferences()
            except Exception:
                self.model.preferences = previous
                raise
            return {'saved': True}
        raise ValueError('Unknown terminal operation')

    def consume_result(self, kind, payload, result):
        if kind == 'history':
            self.history_selection = payload['id']
            self.query_one('#operation-log', LogPane).update_text(result)
            return
        if kind == 'lan':
            operation = payload['operation']
            if operation == 'load':
                self.text('lan_text', result['text'])
                self.lan_preview_text = None
            elif operation == 'preview':
                self.lan_preview_text = payload['text']
            elif operation in {'connect', 'disconnect', 'save'}:
                self.lan_preview_text = None
            self.text('lan_diff', json.dumps(result, indent=2, ensure_ascii=False))
            return
        if kind in {'telemetry-start', 'telemetry-stop'}:
            self.receiver_status = result['status']
            self.receiver_available = result['ready']
            self.update_telemetry()
        if kind == 'telemetry-connect':
            self.telemetry_data = {}
            self.telemetry_frozen = None
            self.query_one('#telemetry_freeze', Checkbox).value = False
            self.telemetry_samples.clear()
            self.field_samples.clear()
            self.telemetry_previous.clear()
            self.telemetry_changes.clear()
            self.selected_telemetry_field = None
            self.telemetry_object = None
            self.update_telemetry()
        if kind != 'run':
            return
        self.result_id = self.backend.controller.snapshot()['result_id']
        action = payload['action']
        if action == 'editor-load':
            self.editor_plan = None
            self.loaded_editor_fighter = None
            self.loaded_editor_form = {k: payload[k] for k in self.editor_values()}
            self.text('editor_text', '')
            self.text('editor_diff', json.dumps(result, indent=2))
            self.query_one('#editor_fighter', Select).set_options([(f, f) for f in result['fighters']])
        elif action == 'editor-fighter':
            self.editor_plan = None
            self.loaded_editor_fighter = result['fighter']
            self.text('editor_text', result['text'])
            self.text('editor_diff', '')
        elif action == 'editor-preview':
            self.editor_plan = result
            self.editor_preview_text = payload['text']
            self.text('editor_diff', result['description'] + '\n\n' + result['diff'])
        elif action in {'editor-commit', 'editor-discard'}:
            self.editor_plan = None
            fighter = self.backend.controller.editor_fighter or self.value('editor_fighter')
            if fighter is not Select.BLANK and self.backend.controller.editor.original is not None:
                current = self.backend.controller.editor.get(fighter, self.backend.controller.editor.load_id)
                self.text('editor_text', current['text'])
            self.text('editor_diff', json.dumps(result, indent=2))
        elif action == 'preview':
            self.transfer_plan = self.backend.controller.plan_id
            self.transfer_form = {k: payload[k] for k in self.transfer_values()}
            self.text('transfer_diff', json.dumps(result, indent=2, ensure_ascii=False))
        elif action == 'commit':
            self.transfer_plan = None
            self.text('transfer_diff', json.dumps(result, indent=2, ensure_ascii=False))
        elif action == 'breakdown-preview':
            self.split_plan = result['plan_id']
            self.split_form = {k: payload[k] for k in self.split_values()}
            self.text('split_diff', result['description'])
        elif action == 'breakdown-commit':
            self.split_plan = None
            self.text('split_diff', json.dumps(result, indent=2, ensure_ascii=False))

    def on_button_pressed(self, event: Button.Pressed):
        name = event.button.id
        if self.running_operation or self.closing:
            return
        if name in {'ship', 'build', 'upload'}:
            data = {'action': name, **self.common()}
            if name in {'ship', 'upload'}:
                data['sequences_confirmed'] = True
            self.ask_confirmation('Confirm release operation',
                                  json.dumps(data, indent=2) + '\n\nBuild writes the manifest version. Upload may synchronize Silent sources and replace active files.',
                                  lambda: self.execute('run', data))
        elif name == 'verify':
            self.request_run(name)
        elif name == 'bump':
            value = self.value('version')
            if re.fullmatch(r'\d+\.\d+\.\d+', value):
                major, minor, patch = value.split('.')
                self.query_one('#version', Input).value = f'{major}.{minor}.{int(patch) + 1}'
            else:
                self.notify('Patch increment needs a plain major.minor.patch version', severity='warning')
        elif name == 'editor-load':
            self.editor_plan = None
            self.request_run(name, {k: self.value(k) for k in ('editor_endpoint', 'editor_file')})
        elif name in {'editor-fighter', 'editor-preview'}:
            if self.loaded_editor_form != self.editor_values():
                self.notify('Read the selected file before opening or previewing a fighter', severity='warning')
                return
            if self.value('editor_fighter') is Select.BLANK:
                self.notify('Load a file and choose a fighter first', severity='warning')
                return
            if name == 'editor-preview' and self.loaded_editor_fighter != self.value('editor_fighter'):
                self.notify('Open the selected fighter before editing its draft', severity='warning')
                return
            self.editor_plan = None
            self.request_run(name, {'fighter': self.value('editor_fighter'), 'text': self.value('editor_text')})
        elif name == 'editor-commit':
            if (self.loaded_editor_form != self.editor_values() or not self.editor_plan
                    or self.editor_preview_text != self.value('editor_text')):
                self.notify('Preview this exact fighter draft first', severity='warning')
                return
            data = {'plan_id': self.editor_plan['plan_id'], 'confirmed': True}
            self.ask_confirmation('Save fighter replacement', self.value('editor_diff'), lambda: self.request_run(name, data))
        elif name == 'editor-discard':
            self.request_run(name)
        elif name == 'validate':
            self.request_run(name, {'file': self.value('source_file')})
        elif name == 'preview':
            self.transfer_plan = None
            self.request_run(name, self.transfer_values())
        elif name == 'commit':
            if not self.transfer_plan or self.transfer_form != self.transfer_values():
                self.notify('Preview the current source and destination first', severity='warning')
                return
            data = {'plan_id': self.transfer_plan, 'confirmed': True}
            self.ask_confirmation('Replace sequence destination', self.value('transfer_diff'), lambda: self.request_run(name, data))
        elif name == 'breakdown-preview':
            self.split_plan = None
            self.request_run(name, self.split_values())
        elif name == 'breakdown-commit':
            if not self.split_plan or self.split_form != self.split_values():
                self.notify('Preview the current split first', severity='warning')
                return
            data = {'plan_id': self.split_plan, 'confirmed': True}
            self.ask_confirmation('Replace per-fighter sources', self.value('split_diff'), lambda: self.request_run(name, data))
        elif name and name.startswith('lan-'):
            operation = name.removeprefix('lan-')
            data = {'operation': operation}
            if operation == 'connect':
                data.update(host=self.value('lan_host'), port=self.value('lan_port'),
                            transport=self.value('lan_transport'), token=self.value('lan_token'))
            elif operation in {'preview', 'save'}:
                data['text'] = self.value('lan_text')
            if operation == 'save':
                if self.lan_preview_text != data['text']:
                    self.notify('Validate and preview this exact complete file first', severity='warning')
                    return
                data['confirmed'] = True
                self.ask_confirmation('Save complete file and reload Diagnostic catalog', self.value('lan_diff'), lambda: self.execute('lan', data))
            else:
                self.execute('lan', data)
        elif name == 'telemetry-export':
            try:
                path = self.value('telemetry_export_path').strip()
                if not path:
                    raise ValueError('Enter a new CSV file path')
                with Path(path).expanduser().open('x', newline='', encoding='utf-8') as file:
                    writer = csv.writer(file)
                    writer.writerow(['Object', 'Group', 'Field', 'Value', 'Age / state', 'Mode / recipe', 'Raw bits'])
                    writer.writerows(self.telemetry_rows)
                self.notify('Visible telemetry exported')
            except (ValueError, OSError) as error:
                self.notify(str(error), severity='error')
        elif name == 'telemetry-clear-charts':
            self.telemetry_samples.clear()
            self.field_samples.clear()
            self.update_telemetry()
        elif name == 'telemetry-copy':
            row = next((r for r in self.telemetry_projected if self.selected_telemetry_field ==
                        (r['object'], r['group'], r['field']['index'])), None)
            if row and row['value']:
                self.copy_to_clipboard(str(row['value'].get('raw', '')))
                self.notify('Copied exact raw bits (terminal clipboard support required)')
            else:
                self.notify('Select an available field first', severity='warning')
        elif name in {'telemetry-start', 'telemetry-connect'}:
            data = {'endpoint': self.value('telemetry_endpoint')}
            details = ('Start the existing offline Cargo telemetry receiver. It may compile the host example.'
                       if name == 'telemetry-start' else 'Activate read-only telemetry subscription to ' + data['endpoint'])
            self.ask_confirmation('Telemetry', details, lambda: self.execute(name, data))
        elif name == 'telemetry-stop':
            self.execute(name, {})
        elif name in {'helper-help', 'helper-run'}:
            data = {'helper': self.value('helper'), 'arguments': '--help' if name == 'helper-help' else self.value('arguments')}
            if name == 'helper-help':
                self.request_run('helper', data)
            else:
                data['confirmed'] = True
                self.ask_confirmation('Run original helper', json.dumps(data, indent=2) + '\n\nReview flags and destinations. Backend readiness checks still apply.',
                                      lambda: self.request_run('helper', data))
        elif name == 'settings-save':
            self.execute('settings', {k: self.value(k) for k in ('silent_host', 'silent_port', 'diagnostic_host', 'diagnostic_port', 'cargo_home')})
        elif name == 'current-log':
            self.action_current_log()

    def on_text_area_changed(self, event: TextArea.Changed):
        if event.text_area.id == 'editor_text' and event.text_area.text != self.editor_preview_text:
            self.editor_plan = None
        if event.text_area.id == 'lan_text' and event.text_area.text != self.lan_preview_text:
            self.lan_preview_text = None

    def on_input_changed(self, event: Input.Changed):
        if event.input.id == 'editor_file':
            self.invalidate_editor_source()
        elif event.input.id == 'history_filter':
            self.history_fingerprint = None
            self.update_history()
        elif event.input.id in {'telemetry_filter', 'feature_filter'}:
            self.update_telemetry()

    def on_checkbox_changed(self, event: Checkbox.Changed):
        if event.checkbox.id == 'telemetry_freeze':
            if event.value:
                self.telemetry_frozen = (deepcopy(self.telemetry_data), time.monotonic() - self.telemetry_elapsed() / 1000, self.receiver_available)
            else:
                self.telemetry_frozen = None
            self.update_telemetry()
        elif event.checkbox.id in {'feature_active', 'telemetry_schema_mode', 'telemetry_available'}:
            self.update_telemetry()

    def telemetry_elapsed(self):
        return int((time.monotonic() - self.telemetry_received) * 1000) if self.telemetry_received is not None else 0

    def on_select_changed(self, event: Select.Changed):
        if event.select.id == 'editor_endpoint':
            self.invalidate_editor_source()
        elif event.select.id == 'editor_fighter':
            self.editor_plan = None
        elif event.select.id in {'telemetry_metric', 'telemetry_object_select', 'telemetry_group', 'telemetry_graph'}:
            if event.select.id in {'telemetry_object_select', 'telemetry_group'}:
                self.selected_telemetry_field = None
                self.field_samples.clear()
            elif event.select.id == 'telemetry_graph':
                selected = str(event.value)
                self.selected_telemetry_field = (selected.split('|')[0], *map(int, selected.split('|')[1:])) if selected else None
                self.field_samples.clear()
            self.update_telemetry()

    def on_data_table_row_selected(self, event: DataTable.RowSelected):
        if self.running_operation:
            return
        key = str(event.row_key.value)
        if event.data_table.id == 'history_table':
            row = next((r for r in self.backend.controller.rows if r['id'] == key), None)
            self.text('history_details', json.dumps(row, indent=2, ensure_ascii=False))
            self.execute('history', {'id': key})
        elif event.data_table.id == 'telemetry_table':
            object_id, group, index = key.split('|')
            data = self.telemetry_frozen[0] if self.telemetry_frozen else self.telemetry_data
            obj = next((o for o in data.get('objects', []) if str(o['object_id']) == object_id and str(o['group']) == group), {})
            self.selected_telemetry_field = (object_id, int(group), int(index))
            self.telemetry_object = object_id
            self.field_samples.clear()
            self.update_telemetry()

    def update_state(self):
        state = self.backend.controller.snapshot()
        if not self.running_operation:
            self.query_one('#status', Static).update(self.operation_status or state['status'])
        if not self.history_selection:
            self.query_one('#operation-log', LogPane).update_text(state['output'])
        lan = state['lan_editor']
        live = self.backend.live.snapshot()
        self.query_one('#lan_status', Static).update(f"{lan['status']} · editing={lan['editing']} · dropped={live['dropped']}\n{live['error']}")
        self.query_one('#live_log_status', Static).update(f"Native live-ring loss: {live['dropped']} · {live['error']} · display controls do not stop capture")
        shipped = '\n'.join(f"{name}: verified {row['details'].get('version', '')} · {row['time']}" for name, row in state['shipped'].items())
        self.query_one('#overview-state', Static).update(f"{state['status']}\nLAN: {lan['status']}\nLast verified deployment:\n{shipped or 'None recorded'}")
        self.update_history()

    def update_history(self):
        rows = self.backend.controller.rows
        search = self.value('history_filter').casefold()
        fingerprint = (json.dumps(rows, default=str), search)
        if fingerprint == self.history_fingerprint:
            return
        self.history_fingerprint = fingerprint
        success = sum(r.get('status') == 'success' for r in rows)
        failed = sum(r.get('status') == 'failed' for r in rows)
        for name, count in [('runs', len(rows)), ('success', success), ('failed', failed), ('other', len(rows) - success - failed)]:
            self.query_one('#outcome-' + name, Static).update(str(count))
        table = self.query_one('#history_table', DataTable)
        table.clear()
        for row in reversed(rows):
            if search and search not in json.dumps(row, default=str).casefold():
                continue
            details = row.get('details', {})
            verified = row.get('receipt', {}).get('verified_deployment', False)
            table.add_row(str(row.get('time', '')), str(row.get('action', '')), str(row.get('status', '')),
                          f"{details.get('target', '')} {details.get('version', '')}", 'yes' if verified else '—', key=row['id'])
        plot = self.query_one('#activity-chart', PlotextPlot)
        recent = rows[-30:]
        labels = ['success', 'failed', 'started', 'interrupted']
        plot.plt.clear_data()
        plot.plt.bar(labels, [sum(r.get('status') == label for r in recent) for label in labels], color=['green', 'red', 'cyan', 'yellow'])
        plot.plt.title('Recent operation outcomes')
        plot.refresh()

    @work(group='poll', exit_on_error=False)
    async def refresh_backend(self):
        if self.polling or self.running_operation or self.closing or self.screen.is_modal or not self.query('#telemetry_table'):
            return
        self.polling = True
        try:
            import asyncio
            streams, receiver, schema, data = await asyncio.to_thread(self.poll_backend)
            if self.closing or not self.query('#telemetry_table'):
                return
            for name, text in streams.items():
                self.query_one('#' + name + '-log', LogPane).update_text(text)
            self.receiver_status = receiver['status']
            self.receiver_available = receiver['ready']
            if schema:
                self.telemetry_schema = schema
                self.text('telemetry_schema', json.dumps(schema, indent=2, ensure_ascii=False))
            if data:
                if data.get('session') != self.telemetry_session:
                    self.telemetry_session = data.get('session')
                    self.telemetry_samples.clear()
                    self.field_samples.clear()
                    self.telemetry_previous.clear()
                    self.telemetry_changes.clear()
                self.telemetry_data = data
                self.telemetry_received = time.monotonic()
                self.telemetry_samples.append(data)
            self.record_field_sample()
            self.update_telemetry()
            self.update_state()
            self.poll_error = None
        except Exception as error:
            if self.closing or not self.query('#telemetry_table'):
                return
            self.receiver_available = False
            self.receiver_status = 'Monitoring unavailable: ' + str(error)
            self.update_telemetry()
            if str(error) != self.poll_error:
                self.notify('Monitor refresh: ' + str(error), severity='warning')
                self.poll_error = str(error)
        finally:
            self.polling = False

    def poll_backend(self):
        receiver = self.backend.telemetry.snapshot()
        schema = data = None
        if receiver['ready']:
            if not self.telemetry_schema:
                schema = self.backend.telemetry_request('/schema')
            data = self.backend.telemetry_request('/state')
        return self.backend.streams(), receiver, schema, data

    def record_field_sample(self):
        if not self.selected_telemetry_field or self.telemetry_frozen or self.value('telemetry_schema_mode'):
            return
        object_id, group, index = self.selected_telemetry_field
        obj = next((o for o in self.telemetry_data.get('objects', []) if o['object_id'] == object_id and o['group'] == group), None)
        value = None
        if self.receiver_available and obj and not obj['stale'] and obj['age_ms'] + self.telemetry_elapsed() < 250:
            fields = obj.get('values', [])
            if index < len(fields) and fields[index]:
                try:
                    value = float(fields[index]['display'])
                    if not math.isfinite(value):
                        value = None
                except (ValueError, TypeError):
                    pass
        self.field_samples.append((time.monotonic(), value))

    def sync_telemetry_selectors(self, data):
        objects = sorted({str(o['object_id']) for o in data.get('objects', [])})
        features = self.telemetry_schema.get('features', [])
        start = self.telemetry_schema.get('feature_group_start', 11)
        groups = [('Object state', '0'), *[(f'Attack slot {i}', str(i + 1)) for i in range(8)],
                  ('Controller input', '9'), ('Assist policy', '10'),
                  *[(f['label'], str(start + i)) for i, f in enumerate(features)]]
        numeric = []
        for obj in data.get('objects', []):
            for field in self.telemetry_schema.get(schema_key(obj['group']), []):
                if field.get('kind') in {'Float', 'Signed', 'Unsigned'} and not field.get('source', '').startswith('unavailable'):
                    numeric.append((f"{obj['object_id']} / {obj['group']} · {field.get('label') or field.get('name')}", f"{obj['object_id']}|{obj['group']}|{field['index']}"))
        fingerprint = (objects, groups, numeric)
        if fingerprint == self.telemetry_selection_options:
            return
        self.telemetry_selection_options = fingerprint
        for name, options, fallback in [('telemetry_object_select', [('All objects', '*'), *[(o, o) for o in objects]], '*'),
                                        ('telemetry_group', [('All groups', '*'), *groups], '*'),
                                        ('telemetry_graph', [('Select a numeric field', ''), *numeric], '')]:
            widget = self.query_one('#' + name, Select)
            old = widget.value
            widget.set_options(options)
            widget.value = old if old in {value for _, value in options} else fallback

    def update_telemetry(self):
        table = self.query_one('#telemetry_table', DataTable)
        cursor = table.cursor_row
        table.clear()
        search = self.value('telemetry_filter').casefold()
        schema = self.telemetry_schema
        elapsed_ms = self.telemetry_elapsed()
        display_data, available = self.telemetry_data, self.receiver_available
        reference = self.value('telemetry_schema_mode')
        if self.telemetry_frozen:
            display_data, received_at, available = self.telemetry_frozen
            elapsed_ms = int((time.monotonic() - received_at) * 1000)
        self.sync_telemetry_selectors(display_data)
        self.telemetry_rows = []
        self.telemetry_projected = observation_rows(schema, display_data, elapsed_ms, available,
            object_id=str(self.value('telemetry_object_select')), group=str(self.value('telemetry_group')),
            reference=reference, available_only=self.value('telemetry_available'))
        now = time.monotonic()
        for row in self.telemetry_projected:
            field, value, obj = row['field'], row['value'], row['record'] or {}
            label = field.get('label') or field.get('name', f"Field {field['index']}")
            decoded = '— (schema)' if reference else 'Unknown / unavailable' if value is None else str(value.get('display', '')) + (f" · {value['meaning']}" if value.get('meaning') else '')
            status = row['state']
            if self.telemetry_frozen and status == 'live':
                status = 'frozen'
            cells = (row['object'], str(row['group']), label, decoded,
                     f"{row['age']} ms · {status}" if row['age'] is not None else status,
                     f"{obj.get('mode') or ''} / {obj.get('recipe') or ''}")
            if search and search not in (' '.join(cells) + ' ' + str(field.get('name', '')) + ' ' + str(field.get('kind', '')) + ' ' + str(field.get('source', ''))).casefold():
                continue
            raw = value.get('raw') if value else None
            if status == 'live':
                if row['key'] in self.telemetry_previous and self.telemetry_previous[row['key']] != raw:
                    self.telemetry_changes[row['key']] = now + .5
                self.telemetry_previous[row['key']] = raw
            tone = '#ffd078' if status in {'stale', 'unavailable'} else '#bcacff' if status in {'schema', 'frozen'} else '#78e2b0'
            value_tone = '#ffd078' if self.telemetry_changes.get(row['key'], 0) > now else '#edf1f7'
            table.add_row(*cells[:3], Text(decoded, style=value_tone), Text(cells[4], style=tone), cells[5], key=row['key'])
            self.telemetry_rows.append((*cells, str(raw or '')))
        self.query_one('#telemetry_row_count', Static).update(f'{len(self.telemetry_rows)} / {len(self.telemetry_projected)} fields shown · unknown ≠ false or zero')
        selected = next((r for r in self.telemetry_projected if self.selected_telemetry_field == (r['object'], r['group'], r['field']['index'])), None)
        if selected:
            field, value, record = selected['field'], selected['value'], selected['record'] or {}
            self.text('telemetry_details', json.dumps({'field': field, 'value': value, 'state': selected['state'], 'age_ms': selected['age'],
                'publication': {key: record.get(key) for key in ['object_id', 'group', 'category', 'kind', 'session', 'sequence', 'sample_us']}}, indent=2, ensure_ascii=False))
        else:
            self.text('telemetry_details', 'Select a field row to inspect its type, source, exact raw bits and publication metadata.')
        if table.row_count:
            table.move_cursor(row=min(cursor, table.row_count - 1))
        data = self.telemetry_data
        live_elapsed = self.telemetry_elapsed()
        rate = data.get('packets_per_second')
        byte_rate = data.get('bytes_per_second')
        object_count = len({o['object_id'] for o in data.get('objects', []) if o.get('age_ms', 0) + live_elapsed < 1000})
        self.query_one('#telemetry_metrics', Static).update(f"Packets/s {rate if rate is not None else '—'} · KiB/s {round(byte_rate / 1024, 2) if byte_rate is not None else '—'} · Objects {object_count} · Gaps {data.get('gaps', 0)}")
        self.query_one('#packet-trend', Sparkline).data = [s['packets_per_second'] for s in self.telemetry_samples if isinstance(s.get('packets_per_second'), (int, float)) and math.isfinite(s['packets_per_second'])]
        self.query_one('#byte-trend', Sparkline).data = [s['bytes_per_second'] / 1024 for s in self.telemetry_samples if isinstance(s.get('bytes_per_second'), (int, float)) and math.isfinite(s['bytes_per_second'])]
        self.query_one('#telemetry_diagnostics', Static).update('\n'.join(f'{name}: {data.get(name, "—")}' for name in
            ['packets', 'bytes', 'gaps', 'sender_dropped', 'reordered', 'rejected', 'wrong_source', 'session_changes', 'session', 'uptime_seconds', 'session_history_full']))
        self.query_one('#telemetry_status', Static).update(self.receiver_status)
        if data:
            stream_status = data.get('status') if self.receiver_available and live_elapsed < 250 else 'stale / retained snapshot'
            self.query_one('#telemetry_status', Static).update(
                f"{self.receiver_status}\n{stream_status} · packets={data.get('packets')} · gaps={data.get('gaps')} · "
                f"rejected={data.get('rejected')} · sender dropped={data.get('sender_dropped')}\n"
                f"Session={data.get('session')} · bytes={data.get('bytes')} · reordered={data.get('reordered')} · "
                f"wrong source={data.get('wrong_source')} · session changes={data.get('session_changes')}")
        metric = self.value('telemetry_metric')
        plot = self.query_one('#telemetry-chart', PlotextPlot)
        plot.plt.clear_data()
        values = [float(s[metric]) / (1024 if metric == 'bytes_per_second' else 1) for s in self.telemetry_samples
                  if isinstance(s.get(metric), (int, float)) and math.isfinite(s[metric])]
        if values:
            plot.plt.plot(values, color='cyan')
        plot.plt.title(metric.replace('_', ' ').title() + ' · last 120 refreshes')
        plot.refresh()
        if self.telemetry_frozen:
            self.query_one('#telemetry_status', Static).update(self.receiver_status + '\n❚❚ Frozen values · ages advance; live metrics and capture continue')
        field_plot = self.query_one('#field-chart', PlotextPlot)
        field_plot.plt.clear_data()
        segment_x, segment_y = [], []
        now = time.monotonic()
        previous_time = None
        for timestamp, value in ([] if reference else self.field_samples):
            if now - timestamp > 60:
                continue
            if previous_time is not None and timestamp - previous_time >= 1.5:
                if segment_x:
                    field_plot.plt.plot(segment_x, segment_y)
                    segment_x, segment_y = [], []
            previous_time = timestamp
            if value is None:
                if segment_x:
                    field_plot.plt.plot(segment_x, segment_y)
                    segment_x, segment_y = [], []
            else:
                segment_x.append(timestamp - now)
                segment_y.append(value)
        if segment_x:
            field_plot.plt.plot(segment_x, segment_y)
        field_plot.plt.title('Selected numeric field · seconds ago · stale/unavailable gaps')
        field_plot.refresh()
        objects = display_data.get('objects', [])
        selected_object = self.value('telemetry_object_select')
        object_id = selected_object if selected_object != '*' else self.telemetry_object or (objects[0]['object_id'] if objects else '')
        summary, feature_rows = assistance_summary(schema, display_data, object_id, elapsed_ms, available and not reference)
        self.text('fighter_summary', summary)
        feature_table = self.query_one('#feature_table', DataTable)
        feature_table.clear()
        search = self.value('feature_filter').casefold()
        only_active = self.value('feature_active')
        for row in feature_rows:
            if search and search not in row[0].casefold():
                continue
            if only_active and row[1] != 'Enabled preference' and row[2] not in {'Selected proposal (not delivery)', 'Proposed; not selected'}:
                continue
            preference_tone = '#78e2b0' if row[1] == 'Enabled preference' else '#ffd078' if row[1] == 'Unknown' else '#a3adbc'
            evidence_tone = '#78e2b0' if row[2] == 'Selected proposal (not delivery)' else '#ffd078' if row[2] == 'Proposed; not selected' else '#a3adbc'
            feature_table.add_row(row[0], Text(row[1], style=preference_tone), Text(row[2], style=evidence_tone), *row[3:])

    def action_toggle_output(self):
        self.query_one('#operation-log', LogPane).toggle_class('hidden')

    def action_current_log(self):
        self.history_selection = None
        self.query_one('#operation-log', LogPane).remove_class('hidden')
        self.update_state()

    def action_preview_draft(self):
        menu = self.query_one('#menu', TabbedContent).active
        if menu == 'sequences':
            active = self.query_one('#sequence-menu', TabbedContent).active
            action = {'fighter-editor': 'editor-preview', 'transfer': 'preview', 'split': 'breakdown-preview'}[active]
        elif menu == 'monitor' and self.query_one('#monitor-menu', TabbedContent).active == 'lan-editor':
            action = 'lan-preview'
        else:
            self.notify('Open an editor or sequence tool to preview a draft')
            return
        self.query_one('#' + action, Button).press()

    def action_help(self):
        self.ask_confirmation('Keyboard & safety guide',
                              'Tab / Shift+Tab: move focus\nArrow keys: choose tabs, rows and select options\nEnter: activate\n'
                              'Ctrl+S: preview active draft\nCtrl+L: current operation log\nCtrl+Q: safe exit\nEscape: cancel dialog\n\n'
                              'Select a history row to inspect its receipt and retained log.\n'
                              'Telemetry rows expose exact raw values in the detail pane.\n'
                              'Drafts must be previewed again after edits. FTP readiness never bypasses confirmation.\n'
                              'Log copy uses terminal clipboard support. Export uses a new local file only.\n'
                              'Freeze/clear/filter never erase capture files or stop collection.', lambda: None)

    @work(group='shutdown', exit_on_error=False)
    async def action_safe_quit(self):
        import asyncio
        if self.closing:
            return
        self.closing = True
        if self.running_operation:
            self.notify('Waiting for active work to finish; transfers are not interrupted', timeout=8)
            while self.running_operation:
                await asyncio.sleep(0.1)
        self.set_operation_buttons(True)
        await asyncio.to_thread(self.backend.close)
        self.exit()


OPERATION_BUTTONS = {'ship', 'build', 'upload', 'verify', 'editor-load', 'editor-fighter',
                     'editor-preview', 'editor-commit', 'editor-discard', 'validate', 'preview', 'commit',
                     'breakdown-preview', 'breakdown-commit', 'lan-connect', 'lan-disconnect', 'lan-load',
                     'lan-preview', 'lan-save', 'telemetry-start', 'telemetry-stop', 'telemetry-connect',
                     'helper-help', 'helper-run', 'settings-save', 'current-log'}
