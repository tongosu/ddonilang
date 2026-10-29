import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createViewPresentation} from '../solutions/seamgrim_ui_mvp/ui/workbench/view_presentation.js';
import {displayMessage} from '../solutions/seamgrim_ui_mvp/ui/workbench/display_locale.js';

// Exercise the actual presentation factory, never a second value evaluator.
globalThis.document={documentElement:{dataset:{displayLocale:'ko'}},addEventListener(){}};
let frame;let logical=true;
const state={semanticViewPayload:{ordered_observations:[{name:"횟수",value:4,value_text:"4",unit:"",observation_id:"counter",state_hash_participation:true}],committed_frame:{state_hash:"blake3:committed"},semantic_payload_hash:"test"}};
const box={dataset:{},innerHTML:'',querySelectorAll(){return [];}};
const escaped=text=>String(text).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
const view=createViewPresentation({activeFrame:()=>frame,currentLogicalPresentation:()=>logical?({value:frame.wasm_state}):null,activeRun:()=>null,localProjectRunEvidenceFromProject:()=>null,
  els:{semanticObservations:box,canvas:{addEventListener(){}}},escapeHtml:escaped,formatNumber:String,state});
const make=()=>({madi:2,state_hash:'blake3:committed',wasm_state:{tick_id:2,
  resources:{fixed64:{횟수:'3'},value_json:{결과:{type:'rational_v1',version:1,resource_value_tag:'10'},
    output_log:[{kind:'output',tick:1,line_no:1,text:'old'},
      {kind:'output',tick:2,line_no:2,text:'1/3'},{kind:'warn',tick:2,line_no:3,text:'not a value'}]}},
  output_display:{schema:'ddn.output-display.v1',world_state_hash:'blake3:committed'}}});
frame=make();const before=JSON.stringify(frame);view.renderSemanticObservations({});
assert.ok(box.innerHTML.includes('>1/3</span>'));assert.ok(!box.innerHTML.includes('old'));
assert.ok(!box.innerHTML.includes('not a value'));assert.equal(JSON.stringify(frame),before);
assert.ok(box.innerHTML.includes('data-state-hash-participation="0"'));
frame=make();frame.wasm_state.output_display.world_state_hash='blake3:stale';view.renderSemanticObservations({});
assert.ok(!box.innerHTML.includes('data-committed-output='));
frame=make();frame.wasm_state.resources.value_json.결과={type:'unverified'};view.renderSemanticObservations({});
assert.ok(!box.innerHTML.includes('data-committed-output='));
frame=make();frame.wasm_state.resources.value_json.output_log[1].text='<img onerror="bad">';view.renderSemanticObservations({});
assert.ok(!box.innerHTML.includes('<img'));assert.ok(box.innerHTML.includes('&lt;img'));
frame=make();for(let i=0;i<10;i++)frame.wasm_state.resources.value_json.output_log.push({kind:'output',tick:2,line_no:4+i,text:String(i)});
view.renderSemanticObservations({});assert.equal((box.innerHTML.match(/data-committed-output=/g)||[]).length,4);
document.documentElement.dataset.displayLocale='ja';frame=make();view.renderSemanticObservations({});
assert.ok(box.innerHTML.includes('>出力</strong>'));assert.ok(box.innerHTML.includes('>1/3</span>'));
assert.equal(displayMessage('output','ko').text,'출력');
if(process.env.DDN_S5_ACTUAL_FRAME){
  frame=JSON.parse(fs.readFileSync(process.env.DDN_S5_ACTUAL_FRAME,'utf8')).snapshot.activeFrame;
  document.documentElement.dataset.displayLocale='ko';view.renderSemanticObservations({});
  assert.ok(box.innerHTML.includes('>30/1</span>'));
}
logical=false;frame=make();view.renderSemanticObservations({});
assert.ok(box.innerHTML.includes('>1/3</span>'));assert.ok(box.innerHTML.includes('runtime')===false);
state.semanticViewPayload.ordered_observations=[];view.renderSemanticObservations({});
assert.ok(box.innerHTML.includes('>1/3</span>'));
// Ordinary Rust-formatted output is useful even without graph observations.
// It remains presentation-only and must never publish a different frame's text.
logical=false;state.semanticViewPayload.ordered_observations=[];
frame=make();delete frame.wasm_state.resources.value_json.결과;
frame.wasm_state.resources.fixed64={};
frame.wasm_state.resources.value_json.output_log[1].text='7';
const plainBefore=JSON.stringify(frame);view.renderSemanticObservations({});
assert.ok(box.innerHTML.includes('>7</span>'));
assert.ok(!box.innerHTML.includes('이번 실행에는 관찰값이 없습니다.'));
assert.equal(JSON.stringify(frame),plainBefore);
frame.wasm_state.output_display.world_state_hash='blake3:stale';view.renderSemanticObservations({});
assert.ok(!box.innerHTML.includes('data-committed-output='));
frame.wasm_state.output_display.world_state_hash='blake3:committed';
state.semanticViewPayload.ordered_observations=[{name:'횟수',value:4,value_text:'4',unit:'',observation_id:'counter',state_hash_participation:true}];
view.renderSemanticObservations({});assert.ok(!box.innerHTML.includes('data-committed-output='));
state.semanticViewPayload.ordered_observations=[];frame.wasm_state.resources.value_json.output_log=[];
view.renderSemanticObservations({});assert.ok(!box.innerHTML.includes('data-committed-output='));
console.log('EXACT_OUTPUT_PRESENTATION_GUARDS_PASS');
