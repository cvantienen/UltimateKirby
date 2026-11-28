'use strict';
let consoleText = '', consoleFrozen = false, consolePending = null;
let activityChart = null, outcomeChart = null;
function visibleLog(text, filter) {
  const query = filter.trim().toLowerCase();
  return query ? text.split('\n').filter(line => line.toLowerCase().includes(query)).join('\n') : text;
}
function renderConsole() {
  const out = $('output');
  const bottom = out.scrollHeight - out.scrollTop - out.clientHeight < 40;
  const value = visibleLog(consoleText, $('outputFilter').value);
  // Leave a user's selection intact when polling has no visible changes.
  if (out.textContent !== value) out.textContent = value;
  if ($('outputFollow').checked && bottom) out.scrollTop = out.scrollHeight;
  $('outputMeta').textContent = `${value ? value.split('\n').length.toLocaleString() : 0} visible lines${consoleFrozen ? ' · frozen' : ''}`;
}
function setConsoleOutput(value) {
  if (consoleFrozen) { consolePending = value; return; }
  consoleText = value;
  renderConsole();
}
function freezeConsole() {
  consoleFrozen = !consoleFrozen;
  $('outputFreeze').textContent = consoleFrozen ? 'Resume' : 'Freeze';
  $('outputFreeze').setAttribute('aria-pressed', String(consoleFrozen));
  if (!consoleFrozen && consolePending !== null) { consoleText = consolePending; consolePending = null; }
  renderConsole();
}
async function copyLog(id, selected = false) {
  const selection = window.getSelection();
  const value = selected ? selection && $(id).contains(selection.anchorNode) && $(id).contains(selection.focusNode) ? selection.toString() : '' : $(id).textContent;
  if (!value) { error(selected ? 'Select text inside this log first.' : 'No visible log text to copy.'); return; }
  try {
    await navigator.clipboard.writeText(value);
    $('status').textContent = selected ? 'Selected log text copied' : 'Visible log copied';
  } catch (_) { error('Clipboard unavailable. Select the log text and use your system Copy shortcut.'); }
}
function downloadLog(id) {
  const value = $(id).textContent;
  if (!value) { error('No visible log text to export.'); return; }
  const url = URL.createObjectURL(new Blob([value], { type: 'text/plain;charset=utf-8' }));
  const link = document.createElement('a');
  link.href = url; link.download = id.includes('diagnostic') || id === 'lanLogs' ? 'diagnostic-visible.log' : id.includes('telemetry') ? 'telemetry-visible.log' : 'workbench-visible.log';
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function paintActivity(rows) {
  const completed = rows.filter(row => row.status === 'success').length;
  const failed = rows.filter(row => row.status === 'failed').length;
  $('runsTotal').textContent = rows.length.toLocaleString();
  $('runsSuccess').textContent = completed.toLocaleString();
  $('runsFailed').textContent = failed.toLocaleString();
  $('runsOther').textContent = (rows.length - completed - failed).toLocaleString();
  const recent = rows.slice(-12);
  const totals = [completed, failed, rows.length - completed - failed];
  $('chartEmpty').hidden = rows.length > 0;
  $('activityCharts').hidden = rows.length === 0;
  if (!window.Chart || !rows.length) return;
  if (!activityChart) {
    Chart.defaults.color = '#a1aec6';
    Chart.defaults.borderColor = '#25334b';
    Chart.defaults.font.family = 'system-ui';
    activityChart = new Chart($('activityChart'), {
      type: 'bar',
      data: { labels: [], datasets: [{ label: 'Recorded operation', data: [], borderRadius: 4, backgroundColor: [] }] },
      options: { responsive: true, maintainAspectRatio: false, animation: false, plugins: { legend: { display: false }, tooltip: { callbacks: {} } }, scales: { y: { display: false, min: 0, max: 1.3 }, x: { grid: { display: false }, ticks: { maxRotation: 0, autoSkip: true, font: { size: 10 } } } } }
    });
    outcomeChart = new Chart($('outcomeChart'), {
      type: 'doughnut',
      data: { labels: ['Success', 'Failed', 'Other'], datasets: [{ data: totals, backgroundColor: ['#57dfb0', '#ff8396', '#baa4ff'], borderWidth: 0, spacing: 3 }] },
      options: { responsive: true, maintainAspectRatio: false, animation: false, cutout: '76%', plugins: { legend: { position: 'bottom', labels: { boxWidth: 8, padding: 10, font: { size: 10 } } } } }
    });
  }
  // Keep tooltip records aligned with each update rather than the first render.
  activityChart.options.plugins.tooltip.callbacks.label = context => {
    const row = recent[context.dataIndex];
    return row ? `${row.action} · ${row.status} · ${new Date(row.time).toLocaleString()}` : '';
  };
  activityChart.data.labels = recent.map(row => row.details.version || row.action);
  activityChart.data.datasets[0].data = recent.map(() => 1);
  activityChart.data.datasets[0].backgroundColor = recent.map(row => row.status === 'success' ? '#57dfb0' : row.status === 'failed' ? '#ff8396' : '#baa4ff');
  outcomeChart.data.datasets[0].data = totals;
  activityChart.update('none'); outcomeChart.update('none');
}
function initDashboard() {
  const icons = { ship: 'rocket', editor: 'square-pen', telemetry: 'activity', lanEditor: 'radio', raw: 'terminal', breakdown: 'files', sequences: 'arrow-left-right', history: 'history', helpers: 'terminal' };
  try { setSidebar(localStorage.getItem('wb-sidebar-hidden') === 'true'); } catch (_) {}
  initRaw();
  const rawWindow = new URLSearchParams(location.search).get('raw');
  if (Object.hasOwn(rawStreams, rawWindow)) { tab('raw'); focusRaw(rawWindow); setSidebar(true); }
  for (const button of document.querySelectorAll('nav button')) {
    const icon = document.createElement('i'); icon.dataset.lucide = icons[button.dataset.tab]; button.prepend(icon);
  }
  for (const heading of document.querySelectorAll('.tab h2')) {
    heading.classList.add('page-heading');
    const icon = document.createElement('i'); icon.dataset.lucide = icons[heading.closest('.tab').id]; heading.prepend(icon);
  }
  if (window.lucide) lucide.createIcons();
  $('outputFilter').addEventListener('input', renderConsole);
  $('outputWrap').addEventListener('change', () => $('output').classList.toggle('nowrap', !$('outputWrap').checked));
  $('lanWrap').addEventListener('change', () => $('lanLogs').classList.toggle('nowrap', !$('lanWrap').checked));
  $('outputFollow').addEventListener('change', () => { if ($('outputFollow').checked) $('output').scrollTop = $('output').scrollHeight; });
  $('output').addEventListener('scroll', () => { $('outputFollow').checked = $('output').scrollHeight - $('output').scrollTop - $('output').clientHeight < 40; });
}

function setSidebar(hidden) {
  document.body.classList.toggle('sidebar-hidden', hidden);
  $('sidebarToggle').setAttribute('aria-expanded', String(!hidden));
}
function toggleSidebar() {
  const hidden = !document.body.classList.contains('sidebar-hidden');
  setSidebar(hidden);
  try { localStorage.setItem('wb-sidebar-hidden', String(hidden)); } catch (_) {}
}
const rawStreams = {
  telemetry: { text: '', bytes: 0, frozen: false, cleared: 0 },
  diagnostic: { text: '', bytes: 0, frozen: false, cleared: 0 }
};
let rawPolling = false;
function renderRaw(kind) {
  const stream = rawStreams[kind], out = $('raw-' + kind);
  const include = $('raw-' + kind + '-filter').value.trim().toLowerCase();
  const exclude = $('raw-' + kind + '-exclude').value.trim().toLowerCase();
  const lines = stream.text.split('\n').filter(line => (!include || line.toLowerCase().includes(include)) && (!exclude || !line.toLowerCase().includes(exclude)));
  const value = lines.join('\n');
  if (out.textContent !== value) out.textContent = value;
  if ($('raw-' + kind + '-follow').checked) out.scrollTop = out.scrollHeight;
  $('raw-' + kind + '-meta').textContent = `${stream.bytes.toLocaleString()} bytes archived · ${lines.filter(Boolean).length.toLocaleString()} visible lines${stream.truncated ? ' · bounded tail' : ''}${stream.frozen ? ' · display frozen, capture continues' : ''}`;
}
function freezeRaw(kind) {
  const stream = rawStreams[kind]; stream.frozen = !stream.frozen;
  $('raw-' + kind + '-freeze').textContent = stream.frozen ? 'Resume' : 'Freeze';
  $('raw-' + kind + '-freeze').setAttribute('aria-pressed', String(stream.frozen));
  renderRaw(kind);
}
function clearRaw(kind) {
  const stream = rawStreams[kind]; stream.cleared = stream.bytes; stream.text = ''; renderRaw(kind);
}
function openRawWindow(kind) { window.open('/?raw=' + encodeURIComponent(kind), '_blank', 'noopener'); }
function focusRaw(kind) {
  document.body.classList.add('raw-focus');
  for (const pane of document.querySelectorAll('.raw-pane')) pane.classList.toggle('focused', pane.id === 'raw-' + kind + '-pane');
}
async function pollRaw() {
  if (rawPolling || !$('raw')?.classList.contains('active')) return;
  rawPolling = true;
  try {
    const data = await api('/api/raw');
    $('rawStatus').textContent = `${data.receiver.status} · Diagnostic ${data.logs.connected ? 'connected' : 'disconnected'} · Native ring dropped ${data.logs.dropped ?? 'unknown'} · Log session ${data.logs.epoch ?? 'unknown'}${data.logs.error ? ' · ' + data.logs.error : ''}`;
    for (const kind of Object.keys(rawStreams)) {
      const stream = rawStreams[kind], incoming = data[kind];
      if (stream.frozen) continue;
      if (incoming.bytes < stream.bytes) stream.cleared = 0;
      stream.bytes = incoming.bytes; stream.truncated = incoming.truncated;
      // Byte cursors also handle UTF-8 and clearing a currently truncated tail.
      const bytes = new TextEncoder().encode(incoming.text);
      const offset = Math.max(0, stream.cleared - (incoming.bytes - bytes.length));
      stream.text = new TextDecoder().decode(bytes.slice(offset));
      renderRaw(kind);
    }
  } catch (e) { $('rawStatus').textContent = e.message; }
  finally { rawPolling = false; }
}
function initRaw() {
  for (const kind of Object.keys(rawStreams)) {
    for (const suffix of ['filter', 'exclude']) $('raw-' + kind + '-' + suffix).addEventListener('input', () => renderRaw(kind));
    $('raw-' + kind + '-wrap').addEventListener('change', () => $('raw-' + kind).classList.toggle('nowrap', !$('raw-' + kind + '-wrap').checked));
    $('raw-' + kind).addEventListener('scroll', () => {
      const out = $('raw-' + kind);
      $('raw-' + kind + '-follow').checked = out.scrollHeight - out.scrollTop - out.clientHeight < 20;
    });
  }
}
