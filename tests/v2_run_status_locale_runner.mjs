import assert from 'node:assert/strict';
import {runStatusPresentation} from '../solutions/seamgrim_ui_mvp/ui/workbench/display_locale.js';
import {executionStatusLabel} from '../solutions/seamgrim_ui_mvp/ui/workbench/execution_lifecycle.js';
const states=['idle','preparing','running','pausing','paused','stopping','done','failed','history_readonly'];
let matrix=0;
for(const status of states) for(const mode of ['unknown','live','oneshot']) {
 const lifecycle=Object.freeze({status,mode,final_status:status==='done'?'done':undefined});
 const input=Object.freeze({kind:'lifecycle',lifecycle});const raw=JSON.stringify(input);
 assert.equal(runStatusPresentation(input,'ko').text,executionStatusLabel(lifecycle));
 const ja=runStatusPresentation(input,'ja');assert.equal(ja.fallback,false);
 assert.notEqual(ja.text,executionStatusLabel(lifecycle));assert.equal(JSON.stringify(input),raw);matrix++;
}
const stopped=Object.freeze({kind:'lifecycle',lifecycle:Object.freeze({status:'done',final_status:'stopped',mode:'live'})});
assert.equal(runStatusPresentation(stopped,'ko').text,'실행 중지 · 연속 실행');
assert.equal(runStatusPresentation(stopped,'ja').text,'実行停止 · 連続実行');
console.log('PASS lifecycle matrix and stopped mode: '+matrix+' state/mode cases, input immutable');

const preparing=Object.freeze({kind:'lifecycle',lifecycle:Object.freeze({status:'preparing',mode:'live'}),stepInFlight:true});
assert.equal(runStatusPresentation(preparing,'ko').text,'실행 중');
assert.equal(runStatusPresentation(preparing,'ja').text,'実行中');
assert.equal(runStatusPresentation({...preparing,stepInFlight:false},'ja').text,'実行の準備中 · 連続実行');
console.log('PASS actual host-step override retains mode-free display boundary');

const lifecycle=Object.freeze({status:'failed',mode:'oneshot'});
for(const [kind,expected] of [
 ['readiness_blocked','실행 실패 · 원문을 고친 뒤 다시 실행해 주세요.'],
 ['previous_success','이전 성공 기록 보존 · 실행 실패 · 한 번 실행'],
 ['imported_record','불러온 기록입니다. 원문을 실행해 주세요.'],
 ['dirty_source','실행 실패 · 한 번 실행 · 수정한 원문은 새 실행에 적용됩니다.'],
]) {
 const input=Object.freeze({kind,lifecycle});const raw=JSON.stringify(input);
 assert.equal(runStatusPresentation(input,'ko').text,expected);
 assert.equal(runStatusPresentation(input,'ja').fallback,false);assert.equal(JSON.stringify(input),raw);
}
console.log('PASS readiness, retained success, imported and changed-source presentation branches');

const code='E_RAW_TEST_한글_<진단>';
assert.equal(runStatusPresentation({kind:'failure',code},'ko').text,'실행 실패 · '+code);
assert.equal(runStatusPresentation({kind:'failure',code},'ja').text,'実行失敗 · '+code);
assert.equal(runStatusPresentation({kind:'failure'},'ja').text,'実行失敗 · E_PROJECT_RUN_FAILED');
assert.equal(runStatusPresentation({kind:'failure',code:''},'ja').text,'実行失敗 · ');
console.log('PASS exact original failure code including empty/default distinctions');

const unknown={kind:'lifecycle',lifecycle:{status:'future_phase',mode:'live'}};
assert.deepEqual(runStatusPresentation(unknown,'ja'),{text:'[ko] '+executionStatusLabel(unknown.lifecycle),fallback:true});
assert.equal(runStatusPresentation({kind:'lifecycle',lifecycle:{status:'constructor'}},'ja').fallback,true);
assert.deepEqual(runStatusPresentation({kind:'future',original:'원문 <그대로>'},'ja'),{text:'[ko] 원문 <그대로>',fallback:true});
assert.throws(()=>runStatusPresentation(unknown,'en'),RangeError);
console.log('PASS explicit unknown/prototype fallback and unsupported locale refusal');
