const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { JSDOM } = require('jsdom');

const htmlPath = path.resolve(__dirname, '../../crates/kirby_assist_core/examples/telemetry_viewer.html');
const html = fs.readFileSync(htmlPath, 'utf8');
const schema = {
  object: [
    { index: 0, name: 'action_id', label: 'Action ID', kind: 'Unsigned', source: 'native observation' },
    { index: 1, name: 'damage_pct', label: 'Damage percent', kind: 'Float', source: 'native observation' },
    { index: 2, name: 'unavailable_value', label: 'Unavailable value', kind: 'Unsigned', source: 'unavailable getter' },
  ],
  input: [],
  policy: [],
  attack: [],
  features: [{ label: 'Auto dodge', owner: 1, safety: null, configured_index: 3, description: 'Responds to incoming attacks.' }],
  feature: [],
};

function wireValue(display, raw) {
  return raw === undefined ? { display } : { display, raw };
}

function fixtureState({ age = 0 } = {}) {
  const values = [];
  values[0] = wireValue('42', '0x000000000000002a');
  values[1] = wireValue('12.5', '0x0000000041480000');
  return {
    status: 'streaming',
    session: 'session-a',
    packets: 12,
    bytes: 960,
    packets_per_second: 4,
    bytes_per_second: 320,
    gaps: 0,
    sender_dropped: 0,
    reordered: 0,
    rejected: 0,
    wrong_source: 0,
    session_changes: 0,
    uptime_seconds: 4,
    session_history_full: false,
    objects: [
      { object_id: 'object-a', group: 0, category: 1, kind: 2, age_ms: age, sequence: 100, sample_us: 9000, session: 'session-a', values },
      { object_id: 'object-a', group: 10, category: 1, kind: 2, age_ms: age, sequence: 101, sample_us: 9500, session: 'session-a', values: [wireValue('true', '1'), wireValue('1', '1'), wireValue('1', '1'), wireValue('false', '0'), wireValue('false', '0'), wireValue('0', '0')] },
      { object_id: 'object-a', group: 11, category: 1, kind: 2, age_ms: age, sequence: 102, sample_us: 9600, session: 'session-a', mode: 'Reactive', recipe: 'Dodge on threat', values: [wireValue('true', '1'), wireValue('false', '0'), wireValue('0', '0')] },
    ],
  };
}

async function mount({ age = 0, schemaData = schema, stateData = fixtureState({ age }) } = {}) {
  const dom = new JSDOM(html, { runScripts: 'outside-only', url: 'http://localhost/' });
  const { window } = dom;
  const state = stateData;
  window.fetch = async url => ({
    ok: true,
    json: async () => String(url).includes('/schema') ? schemaData : state,
  });
  window.setInterval = () => 1;
  window.setTimeout = () => 1;
  window.clearTimeout = () => {};
  const script = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].at(-1)?.[1];
  assert.ok(script, 'telemetry viewer inline script exists');
  window.eval(script + '\nwindow.testPoll = poll;');
  await new Promise(resolve => setImmediate(resolve));
  await new Promise(resolve => setImmediate(resolve));
  return { dom, window, document: window.document };
}

test('renders feature and observation cards with exact raw bits and unavailable values, without tables', async t => {
  const { dom, document } = await mount();
  t.after(() => dom.window.close());

  const feature = document.querySelector('.feature-card');
  assert.ok(feature);
  assert.match(feature.textContent, /Auto dodge/);
  assert.match(feature.textContent, /Selected proposal \(not delivery\)/);
  assert.match(feature.textContent, /Dodge on threat/);

  const cards = [...document.querySelectorAll('.observation-card')];
  assert.equal(cards.length, 3);
  assert.match(cards[0].textContent, /0x000000000000002a/);
  assert.match(cards[0].textContent, /Action ID/);
  assert.match(cards[2].textContent, /Unknown \/ unavailable/);
  const unavailableRaw = [...cards[2].querySelectorAll('.observation-part')]
    .find(part => part.querySelector('.data-label')?.textContent === 'Exact raw u64 bits');
  assert.equal(unavailableRaw.querySelector('.raw').textContent, '—');
  assert.equal(document.querySelectorAll('table,thead,tbody,tr,td,th').length, 0);
});

test('shows stale status while retaining the selected field reference and exact bits', async t => {
  const { dom, document } = await mount({ age: 300 });
  t.after(() => dom.window.close());

  assert.ok(document.querySelector('.observation-card.stale'));
  assert.match(document.querySelector('.observation-card .chip').textContent, /Stale/);
  const fieldButton = document.querySelector('[data-field-index="0"]');
  fieldButton.click();
  assert.ok(document.querySelector('.observation-card.selected'));
  const reference = document.querySelector('#field-reference');
  const referenceDetails = [...reference.querySelectorAll('dt')].map((dt, index) => [dt.textContent, dt.nextElementSibling.textContent]);
  assert.deepEqual(referenceDetails.find(([label]) => label === 'Wire index'), ['Wire index', '0']);
  assert.match(document.querySelector('#field-reference').textContent, /0x000000000000002a/);
});

test('search narrows cards by field metadata and can restore the full schema list', async t => {
  const { dom, document, window } = await mount();
  t.after(() => dom.window.close());

  const search = document.querySelector('#search');
  search.value = 'damage_pct';
  search.dispatchEvent(new window.Event('input', { bubbles: true }));
  assert.equal(document.querySelectorAll('.observation-card').length, 1);
  assert.match(document.querySelector('.observation-card').textContent, /Damage percent/);
  assert.match(document.querySelector('#row-count').textContent, /1 \/ 3 schema fields shown/);

  search.value = '';
  search.dispatchEvent(new window.Event('input', { bubbles: true }));
  assert.equal(document.querySelectorAll('.observation-card').length, 3);
});
test('active flags omit false and stale evidence; assist filters and metadata toggles remain display-only', async t => {
  schema.object.push({index:3,name:'can_guard',label:'Can shield',kind:'Boolean',source:'native observation'});
  const original = fixtureState;
  const dom = new JSDOM(html, {runScripts:'outside-only',url:'http://localhost/'});
  t.after(()=>{dom.window.close();schema.object.pop();});
  const w=dom.window, state=original();
  state.objects[0].values[3]=wireValue('true','1');
  w.fetch=async url=>({ok:true,json:async()=>String(url).includes('/schema')?schema:state});
  w.setInterval=()=>1;w.setTimeout=()=>1;w.clearTimeout=()=>{};
  w.eval([...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].at(-1)[1]);
  await new Promise(resolve=>setImmediate(resolve));
  const d=w.document;
  assert.match(d.getElementById('active-flags').textContent,/Can shield/);
  d.getElementById('feature-search').value='no match';
  d.getElementById('feature-search').dispatchEvent(new w.Event('input'));
  assert.equal(d.querySelectorAll('.feature-card').length,0);
  d.getElementById('technical').checked=true;
  d.getElementById('technical').dispatchEvent(new w.Event('change'));
  assert.equal(d.body.classList.contains('technical-hidden'),false);
  state.objects[0].values[3]=wireValue('false','0');
  d.getElementById('feature-search').dispatchEvent(new w.Event('input'));assert.doesNotMatch(d.getElementById('active-flags').textContent,/Can shield/);
  state.objects[0].values[3]=wireValue('true','1');state.objects[0].age_ms=300;
  d.getElementById('feature-search').dispatchEvent(new w.Event('input'));assert.doesNotMatch(d.getElementById('active-flags').textContent,/Can shield/);
});

test('feature-specific evidence separates shared owners and policy stick/frame is not delivery', async t => {
  const schemaData = structuredClone(schema), stateData = fixtureState();
  schemaData.features = [
    {label:'Dodge follow-up',owner:4,safety:null,configured_index:16,description:'Input only'},
    {label:'Other option read',owner:4,safety:null,configured_index:17,description:'Input only'},
    {label:'Grab dodge',owner:8,safety:null,configured_index:18,description:'Independent GrabDodge owner'},
  ];
  const policy = stateData.objects[1].values;
  policy[1] = wireValue('8','0x8'); policy[2] = wireValue('12','0xc');
  policy[11] = wireValue('123','0x7b'); policy[12] = wireValue('-1','0xffffffffffffffff'); policy[13] = wireValue('0','0x0');
  stateData.objects[2].values.push(wireValue('false','0x0'),wireValue('false','0x0'));
  stateData.objects.push({...structuredClone(stateData.objects[2]), group:12, values:[wireValue('true','1'),null,null,wireValue('true','1'),wireValue('false','0')]});
  stateData.objects.push({...structuredClone(stateData.objects[2]), group:13, values:[wireValue('true','1'),null,wireValue('0','0')]});
  const {dom,document,window} = await mount({schemaData,stateData}); t.after(()=>dom.window.close());
  const cards = [...document.querySelectorAll('.feature-card')];
  assert.match(cards[0].textContent,/No current proposal/);
  assert.match(cards[1].textContent,/Proposed; not selected/);
  assert.match(cards[2].textContent,/Selected proposal \(not delivery\)/);
  const summary = document.querySelector('#fighter-summary').textContent;
  assert.match(summary,/Policy native frame123/);
  assert.match(summary,/Proposed stick X \(quantized; not delivery\)-1/);
  stateData.objects[2].values = stateData.objects[2].values.slice(0,3);
  stateData.objects[3].values = stateData.objects[3].values.slice(0,3);
  document.querySelector('#feature-search').dispatchEvent(new window.Event('input'));
  assert.match(document.querySelector('.feature-card').textContent,/Unknown \/ stale/);
});

test('missing, stale and malformed evidence stays unknown and only reported blockers are labeled', async t => {
  const schemaData = structuredClone(schema), stateData = fixtureState();
  schemaData.feature = [{index:0,name:'configured',kind:'Boolean',source:'preference'}];
  const detail = stateData.objects[2];
  detail.values[1] = wireValue('true','1'); detail.values[2] = wireValue('38','0x26');
  const {dom,document,window} = await mount({schemaData,stateData}); t.after(()=>dom.window.close());
  let card = document.querySelector('.feature-card');
  assert.match(card.textContent,/Unknown constraint bits: 0x26/);
  assert.doesNotMatch(card.textContent,/Animation stopped|Native readiness not confirmed/);
  assert.doesNotMatch(document.querySelector('#active-flags').textContent,/configured/);
  detail.values[2] = wireValue('bad','not hex');
  stateData.objects[1].values[1] = wireValue('bad','not hex');
  document.querySelector('#feature-search').dispatchEvent(new window.Event('input'));
  card = document.querySelector('.feature-card');
  assert.match(card.textContent,/Constraint evidence unavailable/);
  assert.match(card.textContent,/Proposed; selection unknown/);
  detail.stale = true;
  document.querySelector('#feature-search').dispatchEvent(new window.Event('input'));
  assert.match(document.querySelector('.feature-card').textContent,/Details unknown \/ stale/);
  delete detail.age_ms;
  stateData.objects[1].stale = true;
  document.querySelector('#feature-search').dispatchEvent(new window.Event('input'));
  assert.match(document.querySelector('.feature-card').textContent,/Unknown \/ stale/);
  assert.doesNotMatch(document.querySelector('.feature-card').textContent,/Reactive/);
});

test('session changes discard frozen observations and disconnects do not report fresh metrics', async t => {
  const stateData = fixtureState();
  const {dom,document,window} = await mount({stateData}); t.after(()=>dom.window.close());
  document.querySelector('#pause').click();
  assert.match(document.querySelector('#display-status').textContent,/Frozen/);
  const next = fixtureState(); next.session = 'session-b'; next.objects[0].values[0] = wireValue('77','0x4d');
  window.fetch = async url => ({ok:true,json:async()=>String(url).includes('/schema')?schema:next});
  await window.testPoll();
  assert.doesNotMatch(document.querySelector('#display-status').textContent,/Frozen/);
  assert.match(document.querySelector('.observation-card').textContent,/77/);
  window.fetch = async()=>{throw Error('disconnected');};
  await window.testPoll();
  assert.equal(document.querySelector('#packet-rate').textContent,'—');
  assert.match(document.querySelector('.observation-card .chip').textContent,/Stale/);
  assert.match(document.querySelector('.feature-card').textContent,/Unknown \/ stale/);
});
