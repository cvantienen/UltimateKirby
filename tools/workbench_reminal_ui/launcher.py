"""Dependency-free interface selection and macOS terminal handoff."""
from __future__ import annotations

from pathlib import Path
import shlex
import subprocess
import sys

CHOOSER = '''
set answer to display dialog "Choose your Workbench interface." with title "Kirby Assist Workbench" buttons {"Cancel", "Terminal", "Web"} default button "Web" cancel button "Cancel"
return button returned of answer
'''
TERMINAL = '''
on run argv
    tell application "Terminal"
        activate
        do script (item 1 of argv)
    end tell
end run
'''


def choose_interface():
    if sys.platform != 'darwin':
        raise RuntimeError('The native interface chooser requires macOS; use --web or --terminal')
    result = subprocess.run(['osascript', '-'], input=CHOOSER, capture_output=True, text=True, check=False)
    if result.returncode:
        if '(-128)' in result.stderr:
            return None
        raise RuntimeError('Could not show the interface chooser: ' + result.stderr.strip())
    choice = {'Web': 'web', 'Terminal': 'terminal', 'Cancel': None}
    if result.stdout.strip() not in choice:
        raise RuntimeError('The interface chooser returned an unexpected selection')
    return choice[result.stdout.strip()]


def has_terminal():
    return sys.stdin is not None and sys.stdout is not None and sys.stdin.isatty() and sys.stdout.isatty()


def attach_terminal():
    """Windowed PyInstaller replaces stdio with None, even when launched in Terminal."""
    if has_terminal():
        return
    try:
        incoming = open('/dev/tty', 'r', encoding='utf-8')
        try:
            outgoing = open('/dev/tty', 'w', encoding='utf-8', buffering=1)
        except BaseException:
            incoming.close()
            raise
    except OSError as error:
        raise RuntimeError('The terminal child has no interactive terminal; launch with --terminal from Terminal.app') from error
    sys.stdin, sys.stdout, sys.stderr = incoming, outgoing, outgoing


def terminal_command(project: Path, state: Path):
    if getattr(sys, 'frozen', False):
        argv = [sys.executable]
    else:
        argv = [sys.executable, str(Path(__file__).resolve().parents[1] / 'workbench.py')]
    argv.extend(['--terminal', '--terminal-child', '--project', str(project), '--state-dir', str(state)])
    # A shell is required by Terminal's do script; every argument is individually quoted.
    return shlex.join(argv)


def open_terminal(project: Path, state: Path):
    if sys.platform != 'darwin':
        raise RuntimeError('No interactive terminal attached; run --terminal inside a terminal')
    result = subprocess.run(['osascript', '-', terminal_command(project, state)], input=TERMINAL,
                            capture_output=True, text=True, check=False)
    if result.returncode:
        raise RuntimeError('Could not open Terminal.app: ' + result.stderr.strip())


def check_terminal_bundle():
    from .app import TerminalWorkbench
    import textual_plotext
    if not (Path(__file__).resolve().parent / TerminalWorkbench.CSS_PATH).is_file():
        raise RuntimeError('Bundled terminal stylesheet is missing')


def launch_terminal(backend):
    if not has_terminal():
        raise RuntimeError('Textual requires an interactive terminal')
    try:
        from .app import TerminalWorkbench
    except ImportError as error:
        raise RuntimeError('Terminal UI dependencies are missing. Install tools/workbench_reminal_ui/requirements.txt with this Python, or rebuild the app with tools/requirements-macos.txt') from error
    app = TerminalWorkbench(backend)
    try:
        app.run()
    finally:
        app.backend.close()
