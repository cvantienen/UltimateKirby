const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { JSDOM } = require('jsdom');
function dashboard() {
  const initial = { version: '1.2.3', target: 'all', helpers: [], lan_editor: { connected: false }, source: 'local', destination: 'local', editor_endpoint: 'local', breakdown_endpoint: 'local' };
  const html = fs.readFileSync(path.join(__dirname, '../workbench_gui.html'), 'utf8').replace('__TOKEN__', '"test"').replace('__INITIAL__', JSON.stringify(initial));
  const dom = new JSDOM(html, { runScripts: 'outside-only', url: 'http://127.0.0.1:12345/' });
  const window = dom.window;
  window.setInterval = () => 1;
  window.fetch = () => new Promise(() => {});
  const vm = require('node:vm');
  vm.runInContext(fs.readFileSync(path.join(__dirname, 'vendor/lucide.js'), 'utf8'), dom.getInternalVMContext());
  vm.runInContext(fs.readFileSync(path.join(__dirname, 'vendor/chart.js'), 'utf8'), dom.getInternalVMContext());
  vm.runInContext(fs.readFileSync(path.join(__dirname, 'dashboard.js'), 'utf8'), dom.getInternalVMContext());
  vm.runInContext([...window.document.scripts].find(script => script.textContent.includes('const TOKEN=')).textContent, dom.getInternalVMContext());
  return window;
}
test('compact console filters, freezes latest output and preserves literal log content', () => {
  const w = dashboard(), d = w.document;
  w.setConsoleOutput('build ok\nERROR <img src=x>\nlast line');
  d.getElementById('outputFilter').value = 'error';
  d.getElementById('outputFilter').dispatchEvent(new w.Event('input'));
  assert.equal(d.getElementById('output').textContent, 'ERROR <img src=x>');
  assert.equal(d.getElementById('output').children.length, 0);
  w.freezeConsole(); w.setConsoleOutput('ERROR first'); w.setConsoleOutput('ERROR latest');
  assert.equal(d.getElementById('output').textContent, 'ERROR <img src=x>');
  w.freezeConsole();
  assert.equal(d.getElementById('output').textContent, 'ERROR latest');
  assert.match(d.getElementById('outputMeta').textContent, /1 visible lines/);
  w.close();
});
test('copy selection is scoped to its log, and visible export uses filtered text', async () => {
  const w = dashboard(), d = w.document;
  let copied = '', exported;
  Object.defineProperty(w.navigator, 'clipboard', { value: { writeText: async value => { copied = value; } } });
  w.URL.createObjectURL = blob => { exported = blob; return 'blob:test'; };
  w.URL.revokeObjectURL = () => {};
  w.HTMLAnchorElement.prototype.click = () => {};
  w.setConsoleOutput('first\nsecond');
  const range = d.createRange(); range.selectNodeContents(d.getElementById('output'));
  w.getSelection().addRange(range);
  await w.copyLog('output', true); assert.equal(copied, 'first\nsecond');
  w.getSelection().removeAllRanges();
  range.selectNodeContents(d.getElementById('status')); w.getSelection().addRange(range);
  copied = ''; await w.copyLog('output', true); assert.equal(copied, '');
  d.getElementById('outputFilter').value = 'second'; w.renderConsole();
  await w.copyLog('output'); assert.equal(copied, 'second');
  w.downloadLog('output'); assert.equal(exported.size, 6);
  w.close();
});
test('history cards and charts use actual outcomes, and timeline search hides unmatched runs', () => {
  const w = dashboard(), d = w.document, charts = [];
  w.Chart = class { static defaults = { font: {} }; constructor(element, config) { Object.assign(this, config); charts.push(this); } update() {} };
  const rows = [
    { id: 'a', action: 'build', status: 'success', time: '2026-01-01', details: { version: '1.0.0' } },
    { id: 'b', action: 'upload', status: 'failed', time: '2026-01-02', details: { version: '1.0.1', target: 'silent' } },
  ];
  w.drawHistory(rows, {});
  assert.equal(d.getElementById('runsSuccess').textContent, '1');
  assert.equal(d.getElementById('runsFailed').textContent, '1');
  assert.deepEqual(Array.from(charts[1].data.datasets[0].data), [1, 1, 0]);
  d.getElementById('historyFilter').value = 'silent'; w.filterHistory();
  assert.equal([...d.getElementById('historyRows').children].filter(row => !row.hidden).length, 1);
  assert.equal(d.querySelectorAll('table').length, 0);
  w.close();
});
test('raw panes filter independently, freeze display, clear by byte cursor and retain literal text', async () => {
  const w = dashboard(), d = w.document;
  w.TextEncoder = TextEncoder; w.TextDecoder = TextDecoder;
  w.tab('raw');
  let telemetry = 'first Ω\n<img src=x> shield\n', diagnostic = 'mapper one\nmapper two\n';
  w.fetch = async () => ({ok:true,json:async()=>({
    telemetry:{text:telemetry,bytes:Buffer.byteLength(telemetry),truncated:false},
    diagnostic:{text:diagnostic,bytes:Buffer.byteLength(diagnostic),truncated:false},
    logs:{connected:true,error:''},receiver:{status:'Ready'}
  })});
  await w.pollRaw();
  d.getElementById('raw-telemetry-filter').value = 'shield';
  d.getElementById('raw-telemetry-filter').dispatchEvent(new w.Event('input'));
  assert.equal(d.getElementById('raw-telemetry').textContent, '<img src=x> shield');
  assert.equal(d.getElementById('raw-telemetry').children.length, 0);
  assert.equal(d.getElementById('raw-diagnostic').textContent, diagnostic);
  w.freezeRaw('telemetry'); telemetry += 'new shield\n'; diagnostic += 'mapper three\n';
  await w.pollRaw();
  assert.equal(d.getElementById('raw-telemetry').textContent, '<img src=x> shield');
  assert.match(d.getElementById('raw-diagnostic').textContent, /mapper three/);
  w.freezeRaw('telemetry'); await w.pollRaw();
  w.clearRaw('telemetry'); telemetry += 'after clear shield\n'; await w.pollRaw();
  assert.equal(d.getElementById('raw-telemetry').textContent, 'after clear shield');
  assert.match(d.getElementById('raw-diagnostic').textContent, /mapper one/);
  w.toggleSidebar(); assert.ok(d.body.classList.contains('sidebar-hidden'));
  assert.equal(d.getElementById('sidebarToggle').getAttribute('aria-expanded'), 'false');
  w.toggleSidebar(); assert.equal(d.getElementById('sidebarToggle').getAttribute('aria-expanded'), 'true');
  w.close();
});
