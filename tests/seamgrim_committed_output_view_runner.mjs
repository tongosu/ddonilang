import assert from 'node:assert/strict';
import fs from 'node:fs';
import { selectCommittedOutputView } from '../solutions/seamgrim_ui_mvp/ui/workbench/committed_output_view.js';
import { extractCanonicalViewAuthorityInputsFromState } from '../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js';
import { buildCanonicalViewPayloadSet } from '../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js';
const frame = {state_hash:'s',source_hash:'p',madi:3};
const make = (family,content) => ({schema:'seamgrim.semantic_view_payload.v1_candidate',family,
  status:'available',identity:{state_hash:'s',source_hash:'p',tick_id:3},
  provenance:{classification:'AUTHORITATIVE_STRUCTURED'},content});
const console = make('console',{ordered_entries:[{text:'100'},{text:'200'}]});
const space2d = make('space2d',{space2d:{schema:'seamgrim.space2d.v0',drawlist:[{kind:'point',x:1,y:2}]}});
assert.deepEqual(selectCommittedOutputView({console},frame).lines,['100','200']);
assert.equal(selectCommittedOutputView({console,space2d},frame).family,'space2d');
assert.deepEqual(selectCommittedOutputView({console,space2d},frame).lines,['100','200']);
assert.deepEqual(selectCommittedOutputView({console:{...console,identity:{...console.identity,state_hash:'old'}},space2d},frame).lines,[]);
assert.equal(selectCommittedOutputView({console},null),null);
for (const patch of [{state_hash:'other'},{source_hash:'other'},{tick_id:4}]) {
  assert.equal(selectCommittedOutputView({console:{...console,identity:{...console.identity,...patch}}},frame),null);
}
assert.equal(selectCommittedOutputView({console:{...console,status:'unavailable'}},frame),null);
assert.equal(selectCommittedOutputView({space2d:{...space2d,provenance:{}}},frame),null);
assert.equal(selectCommittedOutputView({},frame),null);
if (process.argv.includes('--grid-stdin')) {
  const state = JSON.parse(fs.readFileSync(0, 'utf8'));
  const payloads = buildCanonicalViewPayloadSet(extractCanonicalViewAuthorityInputsFromState(state));
  assert.equal(payloads.space2d.status, 'available', JSON.stringify(payloads.space2d.diagnostic));
  const selected = selectCommittedOutputView(payloads, {
    state_hash: state.state_hash, source_hash: state.source_hash, madi: state.tick_id,
  });
  assert.equal(selected.family, 'space2d');
  assert.equal(selected.space2d.drawlist.length, 64);
  assert.deepEqual(selected.space2d.drawlist, state.view_meta.space2d.drawlist);
  process.stdout.write('actual_grid_native_projection_to_canonical_selector: PASS 64\n');
}
process.stdout.write('committed_output_view: PASS\n');
