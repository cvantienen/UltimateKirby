"""Launcher tests mock native dialogs, frontend startup and device operations."""
from pathlib import Path
import shlex
import subprocess
import sys
from unittest.mock import Mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import pytest
import workbench
from workbench_reminal_ui import launcher


@pytest.fixture
def launch_setup(tmp_path, monkeypatch):
    project = tmp_path / "project with 'quotes'"
    project.mkdir()
    (project / 'Cargo.toml').write_text('[package]\nname="test"\nversion="1.2.3"\n')
    sequences = project / 'deployment/ultimate/kirby_assist/sequences.txt'
    sequences.parent.mkdir(parents=True)
    sequences.write_text('[universal.settings]\nenabled=true\n')
    state = tmp_path / 'state with spaces'
    monkeypatch.setattr(workbench, 'IS_FROZEN', False)
    monkeypatch.setattr(workbench, 'version', lambda: '1.2.3')
    monkeypatch.setattr(workbench, 'PROJECT_ROOT', project)
    monkeypatch.setattr(workbench, 'ROOT', project)
    monkeypatch.setattr(workbench, 'DEFAULT_FILE', sequences)
    for module in (workbench.workbench_history, workbench.sequence_sources, workbench.upload_switch):
        for name in ('ROOT', 'STATE', 'TOOLS_DIR', 'SOURCE', 'GENERATED', 'PROJECT_ROOT', 'DEFAULT_RELEASE_ROOT', 'DEPLOYMENT_SEQUENCES'):
            if hasattr(module, name):
                monkeypatch.setattr(module, name, getattr(module, name))
    import build_variants
    for name in ('ROOT', 'RELEASE_VERSION', 'RELEASE_SUMMARY_NAME'):
        monkeypatch.setattr(build_variants, name, getattr(build_variants, name))
    selected = Mock(return_value='web')
    monkeypatch.setattr(launcher, 'choose_interface', selected)
    terminal = Mock()
    monkeypatch.setattr(launcher, 'launch_terminal', terminal)
    import workbench_gui
    web = Mock()
    monkeypatch.setattr(workbench_gui, 'launch', web)
    monkeypatch.setattr(launcher, 'has_terminal', lambda: True)
    def run(*flags):
        monkeypatch.setattr(sys, 'argv', ['workbench.py', '--project', str(project), '--state-dir', str(state), *flags])
        workbench.main()
    return run, project, state, selected, web, terminal


@pytest.mark.parametrize('flag,target', [('--gui', 'web'), ('--web', 'web'), ('--terminal', 'terminal')])
def test_explicit_modes_bypass_chooser(launch_setup, flag, target):
    run, project, state, chooser, web, terminal = launch_setup
    run(flag)
    chooser.assert_not_called()
    called, other = (web, terminal) if target == 'web' else (terminal, web)
    called.assert_called_once()
    other.assert_not_called()
    backend = called.call_args.args[0]
    assert backend.history.directory == state
    assert workbench.PROJECT_ROOT == project
    assert workbench.sequence_sources.SOURCE == project / 'deployment/ultimate/kirby_assist/sequences.txt'


def test_cancel_does_not_create_state_or_frontend(launch_setup):
    run, _, state, chooser, web, terminal = launch_setup
    chooser.return_value = None
    run('--choose')
    assert not state.exists()
    web.assert_not_called()
    terminal.assert_not_called()


def test_terminal_handoff_precedes_lock_and_backend(launch_setup, monkeypatch):
    run, project, state, chooser, web, terminal = launch_setup
    chooser.return_value = 'terminal'
    monkeypatch.setattr(launcher, 'has_terminal', lambda: False)
    handoff = Mock()
    monkeypatch.setattr(launcher, 'open_terminal', handoff)
    lock = Mock(side_effect=AssertionError('Parent must not acquire lock'))
    monkeypatch.setattr(workbench, 'session_lock', lock)
    run('--choose')
    handoff.assert_called_once_with(project, state)
    assert not state.exists()
    lock.assert_not_called()
    terminal.assert_not_called()
    web.assert_not_called()


def test_child_attaches_tty_and_does_not_relaunch(launch_setup, monkeypatch):
    run, _, _, chooser, web, terminal = launch_setup
    attach = Mock()
    monkeypatch.setattr(launcher, 'attach_terminal', attach)
    handoff = Mock()
    monkeypatch.setattr(launcher, 'open_terminal', handoff)
    run('--terminal', '--terminal-child')
    attach.assert_called_once()
    terminal.assert_called_once()
    chooser.assert_not_called()
    handoff.assert_not_called()


def test_terminal_command_quotes_all_paths_and_uses_bundled_executable(monkeypatch):
    project = Path("/tmp/project 'quoted'; echo nope")
    state = Path('/tmp/state with spaces')
    monkeypatch.setattr(sys, 'frozen', True, raising=False)
    monkeypatch.setattr(sys, 'executable', '/Applications/Workbench app/Contents/MacOS/Workbench')
    argv = shlex.split(launcher.terminal_command(project, state))
    assert argv == [sys.executable, '--terminal', '--terminal-child', '--project', str(project), '--state-dir', str(state)]


@pytest.mark.parametrize('output,code,stderr,expected', [('Web\n', 0, '', 'web'), ('Terminal\n', 0, '', 'terminal'), ('', 1, 'User canceled. (-128)', None)])
def test_native_chooser_result_and_cancel(monkeypatch, output, code, stderr, expected):
    monkeypatch.setattr(sys, 'platform', 'darwin')
    call = Mock(return_value=subprocess.CompletedProcess([], code, stdout=output, stderr=stderr))
    monkeypatch.setattr(launcher.subprocess, 'run', call)
    assert launcher.choose_interface() == expected
    assert call.call_args.args[0] == ['osascript', '-']


def test_native_failures_are_not_retried_or_silently_fallback(monkeypatch):
    monkeypatch.setattr(sys, 'platform', 'darwin')
    monkeypatch.setattr(launcher.subprocess, 'run', Mock(return_value=subprocess.CompletedProcess([], 1, stdout='', stderr='Not authorized')))
    with pytest.raises(RuntimeError, match='chooser'):
        launcher.choose_interface()
    with pytest.raises(RuntimeError, match='Terminal.app'):
        launcher.open_terminal(Path('/tmp/project'), Path('/tmp/state'))


def test_history_and_cli_keep_original_behavior(launch_setup, monkeypatch):
    run, _, _, chooser, web, terminal = launch_setup
    history, cli = Mock(), Mock()
    monkeypatch.setattr(workbench.Workbench, 'show_history', history)
    monkeypatch.setattr(workbench.Workbench, 'loop', cli)
    run('--history')
    run('--cli')
    history.assert_called_once()
    cli.assert_called_once()
    chooser.assert_not_called()
    web.assert_not_called()
    terminal.assert_not_called()


def test_real_lock_rejects_second_frontend(launch_setup):
    run, _, state, _, _, terminal = launch_setup
    with workbench.session_lock(state):
        with pytest.raises(SystemExit):
            run('--terminal')
    terminal.assert_not_called()


def test_windowed_child_restores_interactive_stdio(monkeypatch):
    import io
    incoming = io.StringIO()
    outgoing = io.StringIO()
    incoming.isatty = lambda: True
    outgoing.isatty = lambda: True
    monkeypatch.setattr(sys, 'stdin', None)
    monkeypatch.setattr(sys, 'stdout', None)
    monkeypatch.setattr(sys, 'stderr', None)
    opened = Mock(side_effect=[incoming, outgoing])
    monkeypatch.setattr('builtins.open', opened)
    launcher.attach_terminal()
    assert launcher.has_terminal()
    assert sys.stdout is sys.stderr is outgoing
    assert opened.call_args_list[0].args[:2] == ('/dev/tty', 'r')


def test_child_without_tty_reports_error_instead_of_relaunch(monkeypatch):
    monkeypatch.setattr(launcher, 'has_terminal', lambda: False)
    monkeypatch.setattr('builtins.open', Mock(side_effect=OSError('No controlling terminal')))
    with pytest.raises(RuntimeError, match='no interactive terminal'):
        launcher.attach_terminal()


def test_non_macos_default_is_web_and_mac_default_is_chooser(launch_setup, monkeypatch):
    run, _, _, chooser, web, terminal = launch_setup
    monkeypatch.setattr(sys, 'platform', 'linux')
    run()
    web.assert_called_once()
    chooser.assert_not_called()
    web.reset_mock()
    monkeypatch.setattr(sys, 'platform', 'darwin')
    chooser.return_value = None
    run()
    chooser.assert_called_once()
    web.assert_not_called()
    terminal.assert_not_called()


def test_packaging_checks_terminal_dependencies_without_opening_frontend(launch_setup, monkeypatch):
    run, _, state, chooser, web, terminal = launch_setup
    probe = Mock()
    monkeypatch.setattr(launcher, 'check_terminal_bundle', probe)
    run('--check-terminal-bundle')
    probe.assert_called_once()
    chooser.assert_not_called()
    terminal.assert_not_called()
    web.assert_not_called()
    assert not state.exists()


def test_builder_collects_both_interfaces_and_terminal_resources(tmp_path, monkeypatch):
    import build_workbench_macos as packaging
    monkeypatch.setattr(packaging.sys, 'platform', 'darwin')
    monkeypatch.setattr(packaging, 'TOOLS', tmp_path)
    monkeypatch.setattr(packaging, 'PROJECT_ROOT', tmp_path.parent)
    monkeypatch.setattr(packaging, 'INSTALLED_ROOT', tmp_path / 'Applications')
    monkeypatch.setattr(packaging.importlib, 'import_module', Mock())
    monkeypatch.setattr(packaging.shutil, 'which', lambda name: '/usr/bin/cargo')
    icon = tmp_path / 'icon.webp'
    icon.touch()
    monkeypatch.setattr(packaging, 'make_icon', Mock())
    captured = []
    class CapturedBuild(Exception):
        pass
    def stop_at_build(command, **kwargs):
        captured.append(command)
        raise CapturedBuild()
    monkeypatch.setattr(packaging.subprocess, 'run', stop_at_build)
    with pytest.raises(CapturedBuild):
        packaging.build(icon, 'Test Workbench')
    command = captured[0]
    assert '--windowed' in command
    imports = [command[i + 1] for i, value in enumerate(command) if value == '--hidden-import']
    assert 'workbench_gui' in imports
    assert 'workbench_reminal_ui.app' in imports
    assert 'workbench_reminal_ui.launcher' in imports
    packages = [command[i + 1] for i, value in enumerate(command) if value == '--collect-all']
    assert set(packages) == {'textual', 'textual_plotext', 'rich_pixels', 'plotext'}
    data = [command[i + 1] for i, value in enumerate(command) if value == '--add-data']
    assert any('workbench_gui.html' in item for item in data)
    assert any('app.tcss' in item and 'workbench_reminal_ui' in item for item in data)
