from __future__ import annotations

from pathlib import Path

from textual.app import ComposeResult
from textual.containers import Horizontal, HorizontalScroll, Vertical
from textual.screen import ModalScreen
from textual.widgets import Button, Checkbox, Input, Label, RichLog, Static, TextArea


def tail_overlap(previous, current):
    """Longest previous suffix matching the new tail prefix, in linear time."""
    if not current:
        return 0
    prefix = [0] * len(current)
    matched = 0
    for index in range(1, len(current)):
        while matched and current[index] != current[matched]:
            matched = prefix[matched - 1]
        if current[index] == current[matched]:
            matched += 1
        prefix[index] = matched
    matched = 0
    for char in previous:
        while matched and (matched == len(current) or char != current[matched]):
            matched = prefix[matched - 1]
        if char == current[matched]:
            matched += 1
    return matched


class Confirm(ModalScreen[bool]):
    BINDINGS = [('escape', 'cancel', 'Cancel')]

    def __init__(self, title, details):
        super().__init__()
        self.title_text, self.details = title, details

    def compose(self) -> ComposeResult:
        with Vertical(id='confirmation'):
            yield Label(self.title_text, classes='section-title')
            yield TextArea(self.details, read_only=True, id='confirmation-details')
            yield Static('Review the destination and changes. This action may write files or contact a device.', markup=False)
            with Horizontal(classes='buttons'):
                yield Button('Cancel', id='cancel')
                yield Button('Confirm operation', id='confirm', variant='warning')

    def on_button_pressed(self, event: Button.Pressed):
        self.dismiss(event.button.id == 'confirm')

    def action_cancel(self):
        self.dismiss(False)


class LogViewer(ModalScreen):
    BINDINGS = [('escape', 'close', 'Return')]

    def __init__(self, pane):
        super().__init__()
        self.pane = pane

    def compose(self):
        with Vertical(id='log-viewer'):
            yield Label(self.pane.caption, classes='section-title')
            with Horizontal(classes='buttons'):
                yield Button('‹ Return · Escape', id='close-viewer')
                yield Button('⧉ Copy selection', id='copy-selection')
            yield TextArea(self.pane.display_text, read_only=True, id='expanded-log-text')

    def on_button_pressed(self, event):
        if event.button.id == 'copy-selection':
            self.app.copy_to_clipboard(self.query_one('#expanded-log-text', TextArea).selected_text)
            self.app.notify('Copied selected text (terminal clipboard support required)')
        else:
            self.dismiss()

    def action_close(self):
        self.dismiss()


class LogPane(Vertical):
    """Bounded display controls never stop the underlying durable capture."""
    def __init__(self, title, name):
        super().__init__(id=name, classes='log-pane')
        self.caption = title
        self.raw = ''
        self.clear_offset = 0
        self.rendered = None
        self.display_text = ''
        self.frozen = False

    def compose(self):
        yield Label(self.caption, classes='section-title')
        with Horizontal(classes='filters'):
            yield Input(placeholder='Include text', id=f'{self.id}-include')
            yield Input(placeholder='Exclude text', id=f'{self.id}-exclude')
        with HorizontalScroll(classes='buttons'):
            yield Checkbox('Freeze', id=f'{self.id}-freeze')
            yield Checkbox('Wrap', value=True, id=f'{self.id}-wrap')
            yield Checkbox('Follow', value=True, id=f'{self.id}-follow')
            yield Button('↻ Clear view', id=f'{self.id}-clear')
            yield Button('⧉ Copy', id=f'{self.id}-copy')
            yield Button('↗ Expand', id=f'{self.id}-expand')
        yield Static('0 visible lines · capture unaffected', id=f'{self.id}-meta', classes='log-meta', markup=False)
        yield RichLog(highlight=False, markup=False, wrap=True, max_lines=5000)
        with Horizontal(classes='filters'):
            yield Input(placeholder='Export visible log to a new local file', id=f'{self.id}-path')
            yield Button('Export', id=f'{self.id}-export')

    def update_text(self, text):
        if self.frozen:
            return
        current = text[-256 * 1024:]
        if current != self.raw and self.clear_offset:
            overlap = tail_overlap(self.raw, current)
            self.clear_offset = max(0, self.clear_offset - (len(self.raw) - overlap))
        self.raw = current
        self.redraw()

    def redraw(self):
        text = self.raw[self.clear_offset:]
        include = self.query_one(f'#{self.id}-include', Input).value.casefold()
        exclude = self.query_one(f'#{self.id}-exclude', Input).value.casefold()
        visible = '\n'.join(line for line in text.splitlines()
                            if (not include or include in line.casefold())
                            and (not exclude or exclude not in line.casefold()))
        log = self.query_one(RichLog)
        wrap = self.query_one(f'#{self.id}-wrap', Checkbox).value
        follow = self.query_one(f'#{self.id}-follow', Checkbox).value
        fingerprint = (visible, wrap)
        if fingerprint == self.rendered:
            return
        position = log.scroll_y
        log.wrap = wrap
        log.clear()
        log.write(visible, scroll_end=follow)
        if not follow:
            log.scroll_to(y=position, animate=False)
        self.display_text, self.rendered = visible, fingerprint
        self.query_one(f'#{self.id}-meta', Static).update(f'{len(visible.splitlines())} visible lines · {"frozen display" if self.frozen else "live display"} · capture unaffected')

    def on_input_changed(self, event: Input.Changed):
        if event.input.id in {f'{self.id}-include', f'{self.id}-exclude'}:
            self.redraw()
            event.stop()

    def on_checkbox_changed(self, event: Checkbox.Changed):
        if event.checkbox.id == f'{self.id}-freeze':
            self.frozen = event.value
            self.rendered = None
            self.redraw()
        else:
            self.redraw()
        event.stop()

    def on_button_pressed(self, event: Button.Pressed):
        event.stop()
        action = event.button.id.removeprefix(f'{self.id}-')
        if action == 'clear':
            self.clear_offset = len(self.raw)
            self.redraw()
        elif action == 'copy':
            self.app.copy_to_clipboard(self.display_text)
            self.app.notify('Copied visible lines (terminal clipboard support required)')
        elif action == 'expand':
            self.app.push_screen(LogViewer(self))
        elif action == 'export':
            path = self.query_one(f'#{self.id}-path', Input).value.strip()
            if not path:
                self.app.notify('Enter an export file path', severity='warning')
                return
            try:
                with Path(path).expanduser().open('x', encoding='utf-8') as file:
                    file.write(self.display_text)
            except OSError as error:
                self.app.notify(str(error), severity='error')
            else:
                self.app.notify('Visible lines exported; existing files are never overwritten')
