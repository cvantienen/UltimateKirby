"""Headless frontend tests: no builds, receivers or device connections."""
import asyncio
from pathlib import Path
import sys
import threading
from unittest.mock import Mock, patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import pytest
from textual.widgets import Input, Select, TextArea, TabbedContent, Button, Checkbox, DataTable
from workbench import Workbench
from workbench_history import History
from workbench_reminal_ui.app import TerminalWorkbench
from workbench_reminal_ui.backend import Backend
from workbench_reminal_ui.widgets import Confirm, LogPane


@pytest.fixture
def model(tmp_path, monkeypatch):
    import workbench
    monkeypatch.setattr(workbench, 'version', lambda: '1.2.3')
    return Workbench(History(tmp_path / 'state'))


@pytest.mark.asyncio
async def test_idle_start_and_navigation(model):
    app = TerminalWorkbench(model)
    app.backend.telemetry.start = Mock(side_effect=AssertionError('Unexpected receiver start'))
    model.lan.perform = Mock(side_effect=AssertionError('Unexpected LAN request'))
    async with app.run_test(size=(110, 45)) as pilot:
        await pilot.pause()
        assert app.backend.controller.thread is None
        assert not app.backend.live.thread.is_alive()
        for tab in ['deploy', 'sequences', 'monitor', 'activity', 'helpers', 'settings']:
            app.query_one('#menu', TabbedContent).active = tab
            await pilot.pause()
        await pilot.press('f1')
        await pilot.pause()
        assert isinstance(app.screen, Confirm)
        await pilot.press('escape')
        await pilot.pause()
        assert not app.screen.is_modal
    app.backend.close()
    app.backend.telemetry.start.assert_not_called()
    model.lan.perform.assert_not_called()


@pytest.mark.asyncio
async def test_transfer_preview_invalidation_and_confirmation(model):
    app = TerminalWorkbench(model)
    app.backend.run = Mock(return_value={'saved': True})
    async with app.run_test(size=(130, 55)) as pilot:
        app.query_one('#menu', TabbedContent).active = 'sequences'
        app.query_one('#sequence-menu', TabbedContent).active = 'transfer'
        app.transfer_plan = 'valid-token'
        app.transfer_form = app.transfer_values()
        app.query_one('#destination', Select).value = 'diagnostic'
        app.query_one('#commit', Button).press()
        await pilot.pause()
        assert not app.screen.is_modal
        app.backend.run.assert_not_called()
        app.transfer_form = app.transfer_values()
        app.query_one('#commit', Button).press()
        await pilot.pause()
        assert isinstance(app.screen, Confirm)
        await pilot.press('escape')
        await pilot.pause()
        app.backend.run.assert_not_called()
        app.query_one('#commit', Button).press()
        await pilot.pause()
        await pilot.click('#confirm')
        await app.workers.wait_for_complete()
        data = app.backend.run.call_args.args[0]
        assert data['action'] == 'commit'
        assert data['plan_id'] == 'valid-token'
        assert data['confirmed'] is True
        assert app.transfer_plan is None
    app.backend.close()


@pytest.mark.asyncio
async def test_editing_invalidates_exact_draft_preview(model):
    app = TerminalWorkbench(model)
    async with app.run_test(size=(120, 50)) as pilot:
        app.query_one('#editor_text', TextArea).load_text('draft')
        await pilot.pause()
        app.editor_plan = {'plan_id': 'token'}
        app.editor_preview_text = 'draft'
        app.query_one('#editor_text', TextArea).load_text('changed draft')
        app.lan_preview_text = 'old complete file'
        app.query_one('#lan_text', TextArea).load_text('new complete file')
        await pilot.pause()
        assert app.editor_plan is None
        assert app.lan_preview_text is None
    app.backend.close()


@pytest.mark.asyncio
async def test_log_filters_freeze_clear_and_non_overwriting_export(model, tmp_path):
    app = TerminalWorkbench(model)
    async with app.run_test(size=(120, 50)) as pilot:
        pane = app.query_one('#operation-log', LogPane)
        pane.update_text('Alpha\nBeta private\nBeta visible')
        app.query_one('#operation-log-include', Input).value = 'beta'
        app.query_one('#operation-log-exclude', Input).value = 'private'
        await pilot.pause()
        assert pane.display_text == 'Beta visible'
        app.query_one('#operation-log-freeze', Checkbox).value = True
        await pilot.pause()
        pane.update_text('ignored while frozen')
        assert pane.display_text == 'Beta visible'
        destination = tmp_path / 'export.txt'
        app.query_one('#operation-log-path', Input).value = str(destination)
        app.query_one('#operation-log-export', Button).press()
        await pilot.pause()
        assert destination.read_text() == 'Beta visible'
        pane.display_text = 'must not overwrite'
        app.query_one('#operation-log-export', Button).press()
        await pilot.pause()
        assert destination.read_text() == 'Beta visible'
        app.query_one('#operation-log-clear', Button).press()
        await pilot.pause()
        assert pane.display_text == ''
        assert pane.raw == 'Alpha\nBeta private\nBeta visible'
    app.backend.close()


@pytest.mark.asyncio
async def test_settings_validation_is_atomic_and_persistent(model):
    app = TerminalWorkbench(model)
    data = {'silent_host': '10.0.0.1', 'diagnostic_host': '10.0.0.2',
            'silent_port': '5000', 'diagnostic_port': '6000', 'cargo_home': '/tmp/cache'}
    app.dispatch('settings', data)
    assert Workbench(History(model.history.directory)).preferences['diagnostic_port'] == '6000'
    previous = dict(model.preferences)
    with pytest.raises(ValueError):
        app.dispatch('settings', {**data, 'diagnostic_port': '0'})
    assert model.preferences == previous
    app.backend.close()


def test_backend_reuses_controller_and_serializes_operations(model):
    backend = Backend(model)
    entered, release = threading.Event(), threading.Event()
    def perform(data):
        entered.set()
        release.wait(3)
        return {'action': data['action']}
    backend.controller.perform = perform
    result = []
    thread = threading.Thread(target=lambda: result.append(backend.run({'action': 'validate'})))
    thread.start()
    assert entered.wait(2)
    try:
        with pytest.raises(RuntimeError, match='already running'):
            backend.run({'action': 'validate'})
    finally:
        release.set()
        thread.join(3)
        backend.close()
    assert result == [{'action': 'validate'}]


@pytest.mark.asyncio
async def test_schema_driven_telemetry_and_history_log(model, tmp_path):
    model.history.perform('example', {'target': 'diagnostic', 'version': '1.2.3'}, lambda: print('retained output'))
    app = TerminalWorkbench(model)
    async with app.run_test(size=(140, 60)) as pilot:
        app.telemetry_schema = {'object': [{'index': 0, 'label': 'Native state'}]}
        app.telemetry_data = {'objects': [{'object_id': '0x1', 'group': 0, 'age_ms': 10, 'stale': False,
                                         'values': [{'display': '5', 'raw': '0x5', 'meaning': 'guard'}]}]}
        app.telemetry_samples.append({'packets_per_second': 12})
        app.update_telemetry()
        table = app.query_one('#telemetry_table', DataTable)
        assert table.row_count == 1
        assert 'guard' in table.get_row_at(0)[3]
        app.query_one('#telemetry_filter', Input).value = 'unmatched'
        await pilot.pause()
        assert table.row_count == 0
        row = app.backend.controller.rows[0]
        assert 'retained output' in app.backend.history_log(row['id'])
        app.backend.controller.rows.append({'id': 'unsafe', 'log': str(tmp_path / 'outside.txt')})
        with pytest.raises(ValueError, match='outside'):
            app.backend.history_log('unsafe')
    app.backend.close()


@pytest.mark.asyncio
async def test_local_fighter_roundtrip_keeps_other_fighters_and_blocks_wrong_selection(model, tmp_path):
    import fighter_editor
    import sequence_transfer
    source = tmp_path / 'sequences.txt'
    original = '[universal.settings]\nx=1\n[kirby.neutral.a]\nstart_delay=0\nactions=a:4\n[yoshi.roll.p]\nstart_delay=0\nactions=b:4\n'
    source.write_text(original)
    app = TerminalWorkbench(model)
    with patch.object(fighter_editor, 'validate_sequences'), patch.object(sequence_transfer, 'validate_sequences'):
        async with app.run_test(size=(130, 55)) as pilot:
            app.query_one('#menu', TabbedContent).active = 'sequences'
            app.query_one('#editor_file', Input).value = str(source)
            app.query_one('#editor-load', Button).press()
            await pilot.pause()
            await app.workers.wait_for_complete()
            app.query_one('#editor_fighter', Select).value = 'kirby'
            app.query_one('#editor-fighter', Button).press()
            await pilot.pause()
            await app.workers.wait_for_complete()
            app.query_one('#editor_text', TextArea).load_text('[kirby.neutral.a]\nstart_delay=0\nactions=a:12\n')
            await pilot.pause()
            app.query_one('#editor_fighter', Select).value = 'yoshi'
            await pilot.pause()
            app.query_one('#editor-preview', Button).press()
            await pilot.pause()
            assert app.editor_plan is None
            app.query_one('#editor_fighter', Select).value = 'kirby'
            await pilot.pause()
            app.query_one('#editor-preview', Button).press()
            await pilot.pause()
            await app.workers.wait_for_complete()
            assert app.editor_plan
            app.query_one('#editor-commit', Button).press()
            await pilot.pause()
            assert isinstance(app.screen, Confirm)
            await pilot.click('#confirm')
            await app.workers.wait_for_complete()
            assert source.read_text() == original.replace('a:4', 'a:12')
            assert app.backend.controller.result['verified']
            assert app.editor_plan is None
    app.backend.close()


@pytest.mark.asyncio
async def test_live_output_and_safe_exit_wait_for_active_operation(model):
    entered, release = threading.Event(), threading.Event()
    app = TerminalWorkbench(model)
    def perform(data):
        print('operation progress')
        entered.set()
        release.wait(3)
        return {'done': True}
    app.backend.controller.perform = perform
    async with app.run_test(size=(120, 50)) as pilot:
        app.request_run('validate', {'file': 'unused'})
        assert await asyncio.to_thread(entered.wait, 2)
        app.refresh_output()
        assert 'operation progress' in app.query_one('#operation-log', LogPane).display_text
        app.action_safe_quit()
        await asyncio.sleep(0.05)
        assert not app.backend.closed
        release.set()
        await app.workers.wait_for_complete()
        assert app.backend.closed


@pytest.mark.asyncio
async def test_upload_confirmation_includes_sequence_sync_consent(model):
    app = TerminalWorkbench(model)
    app.backend.run = Mock(return_value={'targets': ['silent']})
    async with app.run_test(size=(130, 55)) as pilot:
        app.query_one('#menu', TabbedContent).active = 'deploy'
        app.query_one('#ready', Checkbox).value = True
        app.query_one('#upload', Button).press()
        await pilot.pause()
        assert isinstance(app.screen, Confirm)
        app.backend.run.assert_not_called()
        await pilot.click('#confirm')
        await app.workers.wait_for_complete()
        data = app.backend.run.call_args.args[0]
        assert data['sequences_confirmed'] is True
        assert data['ready'] is True
    app.backend.close()


@pytest.mark.asyncio
@pytest.mark.parametrize('button,kind,action,confirmed', [
    ('build', 'run', 'build', True), ('ship', 'run', 'ship', True),
    ('verify', 'run', 'verify', False), ('validate', 'run', 'validate', False),
    ('preview', 'run', 'preview', False), ('breakdown-preview', 'run', 'breakdown-preview', False),
    ('helper-help', 'run', 'helper', False), ('helper-run', 'run', 'helper', True),
    ('lan-connect', 'lan', 'connect', False), ('lan-disconnect', 'lan', 'disconnect', False),
    ('lan-load', 'lan', 'load', False), ('lan-preview', 'lan', 'preview', False),
    ('telemetry-start', 'telemetry-start', None, True),
    ('telemetry-stop', 'telemetry-stop', None, False),
    ('telemetry-connect', 'telemetry-connect', None, True),
    ('settings-save', 'settings', None, False),
])
async def test_menu_actions_route_to_expected_services(model, button, kind, action, confirmed):
    app = TerminalWorkbench(model)
    app.execute = Mock()
    async with app.run_test(size=(130, 55)) as pilot:
        app.query_one('#' + button, Button).press()
        await pilot.pause()
        if confirmed:
            assert isinstance(app.screen, Confirm)
            app.execute.assert_not_called()
            await pilot.click('#confirm')
            await pilot.pause()
        actual_kind, payload = app.execute.call_args.args
        assert actual_kind == kind
        if action:
            assert payload.get('action', payload.get('operation')) == action
    app.backend.close()


@pytest.mark.asyncio
async def test_split_and_lan_save_require_current_previews(model):
    app = TerminalWorkbench(model)
    app.execute = Mock()
    async with app.run_test(size=(130, 55)) as pilot:
        app.split_plan = 'split-token'
        app.split_form = app.split_values()
        app.query_one('#breakdown_file', Input).value = 'changed-source'
        app.query_one('#breakdown-commit', Button).press()
        await pilot.pause()
        app.execute.assert_not_called()
        assert not app.screen.is_modal
        app.split_form = app.split_values()
        app.query_one('#breakdown-commit', Button).press()
        await pilot.pause()
        await pilot.click('#confirm')
        await pilot.pause()
        assert app.execute.call_args.args[1]['plan_id'] == 'split-token'
        app.execute.reset_mock()
        app.query_one('#lan_text', TextArea).load_text('complete file')
        await pilot.pause()
        app.query_one('#lan-save', Button).press()
        await pilot.pause()
        app.execute.assert_not_called()
        app.lan_preview_text = 'complete file'
        app.query_one('#lan-save', Button).press()
        await pilot.pause()
        await pilot.click('#confirm')
        await pilot.pause()
        assert app.execute.call_args.args == ('lan', {'operation': 'save', 'text': 'complete file', 'confirmed': True})
    app.backend.close()


def test_lan_reconnect_reuses_capture_thread_and_never_persists_token(model):
    backend = Backend(model)
    backend.live.thread = Mock()
    backend.live.thread.is_alive.side_effect = [False, True]
    model.lan.perform = Mock(return_value={'connected': True, 'editing': True})
    payload = {'host': '10.0.0.8', 'port': 4242, 'transport': 'udp', 'token': 'A' * 32}
    backend.lan('connect', **payload)
    backend.lan('connect', **payload)
    backend.live.thread.start.assert_called_once()
    assert model.preferences['lan_editor_host'] == '10.0.0.8'
    assert 'token' not in model.preferences
    model.lan.perform.assert_called_with('connect', **payload)
    backend.live.thread.is_alive.side_effect = None
    backend.live.thread.is_alive.return_value = False
    backend.close()


@pytest.mark.asyncio
async def test_failed_operation_releases_frontend_and_preserves_error(model):
    app = TerminalWorkbench(model)
    app.backend.run = Mock(side_effect=RuntimeError('expected operation failure'))
    async with app.run_test(size=(120, 50)) as pilot:
        app.request_run('validate', {'file': 'unused'})
        await pilot.pause()
        await app.workers.wait_for_complete()
        assert not app.running_operation
        assert not app.query_one('#validate', Button).disabled
        app.update_state()
        assert app.operation_status == 'expected operation failure'
    app.backend.close()


@pytest.mark.asyncio
async def test_editor_source_changes_block_old_snapshot_save(model):
    app = TerminalWorkbench(model)
    app.execute = Mock()
    async with app.run_test(size=(130, 55)) as pilot:
        app.loaded_editor_form = app.editor_values()
        app.loaded_editor_fighter = 'kirby'
        app.editor_plan = {'plan_id': 'old-file-token'}
        app.editor_preview_text = ''
        app.query_one('#editor_file', Input).value = '/new/file.txt'
        await pilot.pause()
        assert app.editor_plan is None
        assert app.loaded_editor_fighter is None
        app.query_one('#editor-commit', Button).press()
        app.query_one('#editor-fighter', Button).press()
        await pilot.pause()
        app.execute.assert_not_called()
        assert not app.screen.is_modal
        app.loaded_editor_form = app.editor_values()
        app.editor_plan = {'plan_id': 'old-endpoint-token'}
        app.query_one('#editor_endpoint', Select).value = 'diagnostic'
        await pilot.pause()
        assert app.editor_plan is None
    app.backend.close()


@pytest.mark.asyncio
async def test_all_attack_slots_are_labeled_and_cached_telemetry_ages(model):
    import time
    app = TerminalWorkbench(model)
    async with app.run_test(size=(130, 55)) as pilot:
        app.telemetry_schema = {'attack': [{'index': 0, 'label': 'Damage'}]}
        app.telemetry_data = {'status': 'streaming', 'objects': [
            {'object_id': '0x1', 'group': group, 'age_ms': 10, 'stale': False,
             'values': [{'display': '12', 'raw': '0xc'}]} for group in range(1, 9)]}
        app.receiver_available = True
        app.telemetry_received = time.monotonic() - 1
        app.update_telemetry()
        table = app.query_one('#telemetry_table', DataTable)
        assert table.row_count == 8
        assert all(table.get_row_at(i)[2] == 'Damage' for i in range(8))
        assert all('unavailable' in str(table.get_row_at(i)[4]) for i in range(8))
        assert all(table.get_row_at(i)[3].plain == 'Unknown / unavailable' for i in range(8))
        app.consume_result('telemetry-stop', {}, {'ready': False, 'status': 'Receiver stopped'})
        app.query_one('#telemetry_filter', Input).value = 'damage'
        await pilot.pause()
        assert app.receiver_status == 'Receiver stopped'
        assert table.row_count == 8
        assert all('unavailable' in str(table.get_row_at(i)[4]) for i in range(8))
    app.backend.close()


@pytest.mark.asyncio
async def test_clear_survives_rotated_tail_and_expand_uses_full_screen(model):
    from workbench_reminal_ui.widgets import LogViewer
    app = TerminalWorkbench(model)
    async with app.run_test(size=(80, 24)) as pilot:
        pane = app.query_one('#operation-log', LogPane)
        pane.update_text('old prefix\nretained boundary')
        pane.query_one('#operation-log-clear', Button).press()
        await pilot.pause()
        pane.update_text('old prefix\nretained boundary\nnew entry')
        assert pane.display_text.strip() == 'new entry'
        pane.update_text('old prefix\nretained boundary\nnew entry\nnext entry')
        pane.update_text('retained boundary\nnew entry\nnext entry')
        assert 'retained boundary' not in pane.display_text
        assert 'next entry' in pane.display_text
        first = pane.display_text
        pane.redraw()
        assert pane.display_text == first
        pane.query_one('#operation-log-expand', Button).press()
        await pilot.pause()
        assert isinstance(app.screen, LogViewer)
        assert app.screen.query_one('#expanded-log-text', TextArea).region.height >= 8
        await pilot.press('escape')
        await pilot.pause()
        assert not app.screen.is_modal
    app.backend.close()


@pytest.mark.asyncio
async def test_decoded_freeze_csv_and_numeric_history_gaps(model, tmp_path):
    import time
    import csv
    app = TerminalWorkbench(model)
    async with app.run_test(size=(130, 55)) as pilot:
        app.telemetry_schema = {'attack': [{'index': 0, 'label': 'Damage'}]}
        app.receiver_available = True
        app.telemetry_received = time.monotonic()
        app.telemetry_data = {'objects': [{'object_id': '0x1', 'group': 2, 'age_ms': 0, 'stale': False,
                                          'values': [{'display': '12', 'raw': '0xc'}]}]}
        app.selected_telemetry_field = ('0x1', 2, 0)
        app.record_field_sample()
        assert app.field_samples[-1][1] == 12
        app.query_one('#telemetry_freeze', Checkbox).value = True
        await pilot.pause()
        app.telemetry_data['objects'][0]['values'][0]['display'] = '24'
        app.update_telemetry()
        assert app.telemetry_rows[0][3] == '12'
        app.query_one('#telemetry_export_path', Input).value = str(tmp_path / 'observations.csv')
        app.query_one('#telemetry-export', Button).press()
        await pilot.pause()
        with (tmp_path / 'observations.csv').open() as file:
            rows = list(csv.reader(file))
        assert rows[1][3] == '12'
        assert rows[1][-1] == '0xc'
        app.query_one('#telemetry_freeze', Checkbox).value = False
        await pilot.pause()
        app.receiver_available = False
        app.record_field_sample()
        assert app.field_samples[-1][1] is None
        app.update_telemetry()
        assert app.telemetry_rows[0][3] == '24'
    app.backend.close()


def test_schema_projection_covers_all_groups_and_unavailable_fields():
    from workbench_reminal_ui.telemetry import observation_rows
    field = {'index': 0, 'name': 'value', 'label': 'Value', 'kind': 'Float', 'source': 'native'}
    schema = {key: [field] for key in ['object', 'attack', 'input', 'policy', 'feature']}
    schema['features'] = [{'label': f'Tool {i}'} for i in range(18)]
    rows = observation_rows(schema, {}, 0, True, reference=True, available_only=True)
    assert {r['group'] for r in rows} == set(range(29))
    assert all(r['state'] == 'schema' and r['value'] is None for r in rows)
    data = {'objects': [{'object_id': '42', 'group': 0, 'age_ms': 10, 'stale': False, 'values': [None]}]}
    rows = observation_rows(schema, data, 0, True)
    assert len(rows) == 1 and rows[0]['state'] == 'unavailable'
    assert not observation_rows(schema, data, 0, True, available_only=True)
    data['objects'][0]['values'][0] = {'display': '0', 'raw': '0x0'}
    assert observation_rows(schema, data, 0, True)[0]['value']['display'] == '0'
    assert observation_rows(schema, data, 1000, True)[0]['value'] is None
    assert not observation_rows(schema, data, 0, True, object_id='other')


@pytest.mark.asyncio
async def test_telemetry_selectors_reference_freeze_and_chart_controls(model):
    import time
    app = TerminalWorkbench(model)
    app.backend.polling = True
    async with app.run_test(size=(100, 35)) as pilot:
        app.telemetry_schema = {'object': [{'index': 0, 'name': 'damage', 'label': 'Damage', 'kind': 'Float', 'source': 'native'}]}
        app.telemetry_data = {'packets_per_second': 12, 'bytes_per_second': 2048, 'objects': [
            {'object_id': oid, 'group': 0, 'age_ms': 0, 'stale': False, 'values': [{'display': '12', 'raw': '0xc'}]}
            for oid in ['42', '43']]}
        app.receiver_available = True
        app.telemetry_received = time.monotonic()
        app.update_telemetry()
        app.query_one('#telemetry_object_select', Select).value = '42'
        await pilot.pause()
        assert {r[0] for r in app.telemetry_rows} == {'42'}
        app.query_one('#telemetry_graph', Select).value = '42|0|0'
        await pilot.pause()
        assert app.selected_telemetry_field == ('42', 0, 0)
        assert 'native' in app.query_one('#telemetry_details', TextArea).text
        app.query_one('#telemetry_schema_mode', Checkbox).value = True
        await pilot.pause()
        assert app.telemetry_rows[0][3] == '— (schema)'
        app.query_one('#telemetry_schema_mode', Checkbox).value = False
        app.query_one('#telemetry_freeze', Checkbox).value = True
        await pilot.pause()
        snapshot, received, available = app.telemetry_frozen
        app.telemetry_frozen = (snapshot, received - 2, available)
        app.update_telemetry()
        assert app.telemetry_rows[0][3] == 'Unknown / unavailable'
        app.telemetry_samples.append({'packets_per_second': 5})
        app.field_samples.append((time.monotonic(), 12))
        app.query_one('#telemetry-clear-charts', Button).press()
        await pilot.pause()
        assert not app.telemetry_samples and not app.field_samples
    app.backend.close()


def test_assistance_analysis_distinguishes_proposal_from_delivery():
    from workbench_reminal_ui.telemetry import assistance_summary
    def record(group, values):
        return {'object_id': '0x1', 'group': group, 'age_ms': 0, 'stale': False, 'values': values}
    def value(display, raw):
        return {'display': display, 'raw': raw}
    policy = [value('true', '0x1'), value('4', '0x4'), value('4', '0x4')]
    detail = [value('true', '0x1'), value('false', '0x0'), value('16', '0x10')]
    schema = {'features': [{'label': 'Test tool', 'description': 'Input-only assistance', 'owner': 4,
                            'safety': None, 'configured_index': 20}], 'feature_group_start': 11}
    data = {'objects': [record(10, policy), record(11, detail)]}
    summary, rows = assistance_summary(schema, data, '0x1', 0, True)
    assert rows[0][1] == 'Enabled preference'
    assert rows[0][2] == 'Selected proposal (not delivery)'
    assert 'Required hold not pressed' in rows[0][5]
    assert 'do not confirm game execution' in summary
    _, stale_rows = assistance_summary(schema, data, '0x1', 1000, True)
    assert stale_rows[0][2] == 'Unknown / stale'
    assert stale_rows[0][1] == 'Unknown'


@pytest.mark.asyncio
async def test_compact_dashboard_preserves_safe_actions_and_navigation(model):
    app = TerminalWorkbench(model)
    async with app.run_test(size=(120, 45)) as pilot:
        from textual.widgets import Static
        app.query_one('#menu', TabbedContent).active = 'deploy'
        await pilot.pause()
        assert len(app.query('#mascot')) == 0
        assert 'Build + upload…' in str(app.query_one('#ship', Button).label)
        assert 'Build only…' in str(app.query_one('#build', Button).label)
        assert app.query_one('#ship', Button).region.height == 1
        assert app.query_one('#operation-log', LogPane).region.height == 8
        assert len(app.query('#telemetry-pages TabPane')) == 4
        assert len(app.query('#stream-pages TabPane')) == 3
        assert not app.query_one('#ship', Button).has_class('-warning')
        assert not app.query_one('#verify', Button).has_class('-success')
        assert not app.query_one('#telemetry-start', Button).has_class('-primary')
        assert 'KIRBY ASSIST' in app.query_one('#welcome-text', Static).content
        app.query_one('#menu', TabbedContent).active = 'monitor'
        app.query_one('#monitor-menu', TabbedContent).active = 'telemetry'
        await pilot.resize_terminal(80, 24)
        await pilot.pause()
        app.action_toggle_output()
        await pilot.pause()
        for page in ['observations', 'assistance', 'charts', 'wire-reference']:
            app.query_one('#telemetry-pages', TabbedContent).active = page
            await pilot.pause()
            assert app.query_one('#telemetry-pages', TabbedContent).region.width <= 80
        app.query_one('#monitor-menu', TabbedContent).active = 'streams'
        for page in ['live-stream', 'raw-telemetry-stream', 'raw-diagnostic-stream']:
            app.query_one('#stream-pages', TabbedContent).active = page
            await pilot.pause()
            assert app.query_one('#stream-pages', TabbedContent).region.width <= 80
    app.backend.close()
