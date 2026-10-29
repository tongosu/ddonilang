import assert from 'node:assert/strict';
import {renderAuthoredBogae,createAuthoredCompositionHost,pickAuthoredTarget,observationRowsForDisplay,observationCellDisplay} from '../solutions/seamgrim_ui_mvp/ui/workbench/authored_bogae_renderer.js';
import {normalizeWasmStatePayload} from '../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js';
const lines=[];
const fixedColumn={dtype:'fixed64'};
for(const [exact,expected] of [['0.9800000009126961231231689453125','0.98'],['1.640000001527369022369384765625','1.64'],['0.0078125','0.007812'],['0.0234375','0.023438'],['-0.0234375','-0.023438'],['0.00000001','0']]) {
  const cell={status:'present',value_canon:exact,value_text:exact},beforeCell=JSON.stringify(cell);
  assert.equal(observationCellDisplay(cell,fixedColumn),expected);
  assert.equal(JSON.stringify(cell),beforeCell);
}
for(const dtype of ['string','rational','integer']) assert.equal(observationCellDisplay({status:'present',value_canon:'1.6400000015',value_text:'1.6400000015'},{dtype}),'1.6400000015');
const ctx={fillText(text){lines.push(text);}};
const projection={status:'ready',candidates:[{authored_digest:'rust-only-digest',
  artifact:{identity:{scope:'temporary_execution'},view:{kind:'result_table'},relations:[]}}]};
const table={projection_hash:'rust-table-hash',tables:[{columns:[{name:'측정'}],
  rows:[{cells:[{status:'present',value_text:'3'}]}]}]};
const timed={rows:[{committed_madi:0,cells:['old']},{committed_madi:1,cells:['a']},{committed_madi:1,cells:['b']}]};
const timedBefore=JSON.stringify(timed);
assert.deepEqual(observationRowsForDisplay(timed),timed.rows.slice(1));
assert.equal(observationRowsForDisplay(timed,'history'),timed.rows);
assert.equal(observationRowsForDisplay(table.tables[0]),table.tables[0].rows);
assert.equal(JSON.stringify(timed),timedBefore,'presentation filtering never edits recorded rows');
const input={schema:'seamgrim.engine_response.v0',state_hash:'world-hash',
  bogae_projection:projection,public_observation_table_projection:table};
const before=JSON.stringify(input);
const normalized=normalizeWasmStatePayload(input);
assert.deepEqual(normalized.bogae_projection,projection);
assert.deepEqual(normalized.public_observation_table_projection,table);
const model=renderAuthoredBogae(ctx,600,400,normalized);
assert.equal(model.authored_digest,'rust-only-digest');
assert.equal(model.row_count,1);
assert.deepEqual(lines,['측정','3']);
assert.equal(JSON.stringify(input),before,'rendering must not mutate the state');
// A3-TIME-P1: a historical view must never silently draw the current host table.
const historical=structuredClone(input);
historical.bogae_projection.candidates[0].artifact.resolved_madi=0;
historical.bogae_projection.candidates[0].artifact.view.table_projection={projection_hash:'past',tables:[{columns:[{name:'측정'}],rows:[{cells:[{status:'present',value_text:'1'}]}]}]};
lines.length=0;
assert.equal(renderAuthoredBogae(ctx,600,400,historical).table_projection_hash,'past');
assert.deepEqual(lines,['측정','1']);
delete historical.bogae_projection.candidates[0].artifact.view.table_projection;
assert.equal(renderAuthoredBogae(ctx,600,400,historical).diagnostic.code,'E_BOGAE_TIME_OUT_OF_RANGE');
assert.equal(renderAuthoredBogae(ctx,600,400,{resources:{value_json:{fake:3}}}),null);
lines.length=0;
const rejected=renderAuthoredBogae(ctx,600,400,{bogae_projection:{status:'selection_required',
  candidates:[projection.candidates[0],projection.candidates[0]],
  diagnostic:{code:'E_BOGAE_SELECTION_REQUIRED',message:'명시 선택 필요'}}});
assert.equal(rejected.authored_digest,undefined,'no auto-selection by order or digest');
assert.deepEqual(lines,['명시 선택 필요']);
const ellipses=[];
const planeCtx={...ctx,save(){},restore(){},beginPath(){},rect(){},clip(){},stroke(){},
  ellipse(...args){ellipses.push(args);}};
const plane={kind:'scene',spatial_profile:'plane2',horizontal:['-21474836480','21474836480'],
  vertical:['-21474836480','21474836480'],elements:[{kind:'circle',center:['0','0'],radius:'8589934592'}]};
const planeState={bogae_projection:{...projection,candidates:[{...projection.candidates[0],artifact:{identity:{scope:'temporary_execution'},view:plane,relations:[]}}]}};
const planeBefore=JSON.stringify(planeState);
assert.equal(renderAuthoredBogae(planeCtx,600,400,planeState).family,'scene');
assert.deepEqual(ellipses[0].slice(0,4),[300,200,72.8,72.8]);
assert.equal(JSON.stringify(planeState),planeBefore);
const contrastState=structuredClone(planeState);
contrastState.bogae_projection.candidates[0].artifact.relations=[{relation_id:'comparison',kind:'overlay',order:1,view:plane}];
const strokes=[];
const contrastBefore=JSON.stringify(contrastState);
renderAuthoredBogae({...planeCtx,stroke(){strokes.push(this.strokeStyle);}},600,400,contrastState);
assert.deepEqual(strokes,['#215b8b','#b45309'],'local overlay contrast changes no authored geometry');
assert.equal(JSON.stringify(contrastState),contrastBefore);
assert.equal(contrastState.bogae_projection.candidates[0].artifact.relations[0].view,plane);
const selectedPlane={...planeState,bogae_projection:{...planeState.bogae_projection,
  candidates:[{...planeState.bogae_projection.candidates[0],artifact:{...planeState.bogae_projection.candidates[0].artifact,
    relations:[{relation_id:'selected',kind:'magnifier',object_identity:{id:'object'},selection:{element_indices:[0]}}]}}]}};
const highlighted=renderAuthoredBogae(planeCtx,600,400,selectedPlane,{selectedRelationIds:new Set(['selected'])});
assert.deepEqual(highlighted.selected_object_ids,['object']);
assert.deepEqual(ellipses.at(-1).slice(0,4),[300,200,76.8,76.8],'local ring follows the selected authored object');
ellipses.length=0;
plane.horizontal=['9223372036854775806','9223372036854775807'];
plane.elements[0].center[0]='9223372036854775806';
renderAuthoredBogae(planeCtx,600,400,planeState);
assert.ok(ellipses[0].every(Number.isFinite),'one-raw-bit bounds must not collapse under JS number conversion');
const pathPoints=[];
plane.horizontal=['-21474836480','21474836480'];
plane.elements=[{kind:'polygon',vertices:[['0','0'],['8589934592','0'],['8589934592','8589934592']]}];
renderAuthoredBogae({...planeCtx,moveTo(...p){pathPoints.push(['move',...p]);},lineTo(...p){pathPoints.push(['line',...p]);},closePath(){pathPoints.push(['close']);}},600,400,planeState);
const expectedPath=[['move',300,200],['line',372.8,200],['line',372.8,127.2],['close']];
assert.equal(pathPoints.length,expectedPath.length);
pathPoints.forEach((point,index)=>{
  assert.equal(point[0],expectedPath[index][0]);
  point.slice(1).forEach((pixel,axis)=>assert.ok(Math.abs(pixel-expectedPath[index][axis+1])<1e-9));
}); // Pixel projection only; authored raw geometry comparisons remain exact.
// DOM lifecycle unit only: supplied projection fixtures are not world execution evidence.
class Node {
  constructor(tag) { this.tag=tag; this.children=[]; this.dataset={}; this.style={setProperty(){},removeProperty(){}}; this.listeners={}; }
  append(...nodes) { for(const node of nodes) {node.attachments=(node.attachments??0)+1;node.remove();node.parent=this;this.children.push(node);} }
  remove() { if(this.parent) this.parent.children=this.parent.children.filter(node=>node!==this);this.parent=null; }
  replaceChildren() { for(const node of [...this.children]) node.remove(); }
  querySelectorAll(tag) {return this.children.flatMap(node=>[...(node.tag===tag?[node]:[]),...node.querySelectorAll(tag)]);}
  querySelector(tag) {return this.querySelectorAll(tag)[0];}
  addEventListener(event,callback) {this.listeners[event]=callback;}
  removeEventListener(event,callback) {if(this.listeners[event]===callback) delete this.listeners[event];}
  getBoundingClientRect() {return {left:0,top:0,width:400,height:400};}
  focus() {this.focused=true;}
  setAttribute(name,value) {this[name]=value;}
  getContext() {return {...ctx,clearRect(){},fillRect(){}};}
}
globalThis.document={createElement:tag=>new Node(tag)};
const host=createAuthoredCompositionHost(()=>{});
const stage=new Node('main'),toolbar=new Node('nav'),list=new Node('aside');
const primary={relation_id:'p',kind:'primary',view:{kind:'result_table'}};
const side=id=>({relation_id:id,kind:'side',placement:'오른쪽',view:{kind:'result_table'}});
function update(relations) {
  const wasmState={...normalized,bogae_projection:{...projection,candidates:[{...projection.candidates[0],
    artifact:{...projection.candidates[0].artifact,relations}}]}};
  const before=JSON.stringify(wasmState);
  const result=host.render({projectId:'same-project',wasmState,list,stage,toolbar});
  assert.equal(JSON.stringify(wasmState),before);
  return result;
}
update([primary,side('s1'),side('s2')]);
const edgeHandle=stage.querySelectorAll('button').find(node=>node.dataset.authoredSideEdge==='right');
assert.equal(edgeHandle.hidden,true,'an open dock must not be covered by its edge handle');
for(const panel of stage.querySelectorAll('section'))panel.querySelectorAll('button').find(node=>node.dataset.authoredSideClose).listeners.click();
update([primary,side('s1'),side('s2')]);
assert.equal(edgeHandle.hidden,false,'closing all sides exposes the reopening handle');
edgeHandle.listeners.click();
update([primary,side('s1'),side('s2')]);
assert.equal(edgeHandle.hidden,true,'reopening hides the duplicate handle again');
assert.equal(stage.dataset.sideInstanceCount,'2');
const retained=stage.querySelectorAll('section').find(node=>node.dataset.authoredRelationId==='s2');
const initialAttachments=retained.attachments;
update([primary,side('s1'),side('s2')]);
assert.equal(retained.attachments,initialAttachments,'live frames must not reinsert Close button DOM');
toolbar.querySelectorAll('button').find(node=>node.dataset.authoredRelationToggle==='s2').listeners.click();
assert.equal(update([primary,side('s2')]).sideCount,0);
assert.equal(stage.querySelectorAll('section').length,1);
assert.equal(stage.querySelector('section'),retained);
assert.equal(retained.hidden,true);
assert.equal(toolbar.querySelectorAll('button').filter(node=>node.dataset.authoredRelationToggle).length,1);
assert.equal(update([primary,side('s3'),side('s2')]).sideCount,1);
assert.equal(stage.querySelectorAll('section').find(node=>node.dataset.authoredRelationId==='s2'),retained);
assert.equal(update([primary]).sideCount,0);
assert.equal(stage.querySelectorAll('section').length,0);
assert.equal(toolbar.querySelectorAll('button').filter(node=>node.dataset.authoredRelationToggle).length,0);
assert.equal(host.hidden.size,0);
const magnifier=target=>({relation_id:'m1',kind:'magnifier',target_digest:target,recipe:{seal:'sealed-recipe'},
  target:{kind:'list',values:[{kind:'fixed64',raw:'-8589934592'},{kind:'fixed64',raw:'1'}]},view:{kind:'result_table'}});
update([primary,magnifier('target-1')]);
assert.equal(stage.dataset.magnifierInstanceCount,'0','no automatic opening');
const open=()=>toolbar.querySelectorAll('button').find(node=>node.dataset.relationKind==='magnifier').listeners.click();
open();update([primary,magnifier('target-1')]);
assert.equal(stage.dataset.magnifierInstanceCount,'1');
const firstMagnifier=stage.querySelectorAll('section')[0];
assert.equal(firstMagnifier.querySelector('p').textContent,'현재 값: [-2, 1/4294967296]');
assert.deepEqual(JSON.parse(firstMagnifier.querySelector('pre').textContent),magnifier('target-1').target);
update([primary,magnifier('target-1')]);
assert.equal(stage.querySelectorAll('section')[0],firstMagnifier);
update([primary,magnifier('target-2')]);
assert.equal(stage.dataset.magnifierInstanceCount,'0','changed target ends old instance');
assert.equal(firstMagnifier.parent,null);
open();update([primary,magnifier('target-2')]);
const secondMagnifier=stage.querySelectorAll('section')[0];
assert.notEqual(secondMagnifier,firstMagnifier);
secondMagnifier.querySelector('button').listeners.click();update([primary,magnifier('target-2')]);
assert.equal(stage.dataset.magnifierInstanceCount,'0','close terminates, not hides');
open();update([primary,magnifier('target-2')]);
assert.notEqual(stage.querySelectorAll('section')[0],secondMagnifier);
update([primary]);assert.equal(stage.querySelectorAll('section').length,0);
const stable=raw=>({...magnifier(`pose-${raw}`),object_identity:{id:'declared-object',label:'공'},
  target:{kind:'fixed64',raw}});
update([primary,stable('1')]);open();update([primary,stable('1')]);
const moving=stage.querySelector('section');
update([primary,stable('2')]);
assert.equal(stage.querySelector('section'),moving,'new pose keeps the same explicitly bound object instance');
assert.equal(moving.querySelector('pre').textContent,JSON.stringify(stable('2').target,null,2));
assert.deepEqual([...host.selectedRelationIds],['m1']);
update([primary,{...stable('2'),object_identity:{id:'different-object',label:'추'}}]);
assert.equal(stage.dataset.magnifierInstanceCount,'0','different object ends the previous instance');
assert.equal(moving.parent,null);
update([primary]);
const pickingView={kind:'scene',horizontal:['-4294967296','4294967296'],vertical:['-4294967296','4294967296'],
  elements:[{kind:'circle',center:['0','0'],radius:'429496730'}]};
const pickedRelation={relation_id:'pick',kind:'magnifier',selection:{kind:'primary_scene_element.v1',element_indices:[0]}};
assert.equal(pickAuthoredTarget(pickingView,[pickedRelation],400,400,200,200),pickedRelation);
assert.equal(pickAuthoredTarget(pickingView,[pickedRelation],400,400,10,10),null);
assert.equal(pickAuthoredTarget(pickingView,[pickedRelation,{...pickedRelation,relation_id:'ambiguous'}],400,400,200,200),null);
assert.equal(pickAuthoredTarget(pickingView,[{...pickedRelation,selection:undefined}],400,400,200,200),null,'no guessed coordinate target');
const pickCanvas=new Node('canvas');pickCanvas.width=400;pickCanvas.height=400;
const twoView={...pickingView,elements:[pickingView.elements[0],{...pickingView.elements[0],center:['2147483648','0']}]};
const twoTargets=[0,1].map(index=>({...magnifier(`picked-${index}`),relation_id:`pick-${index}`,
  selection:{kind:'primary_scene_element.v1',element_indices:[index]},target:{kind:'scene_element',element:twoView.elements[index]}}));
const pickState={...normalized,bogae_projection:{...projection,candidates:[{...projection.candidates[0],artifact:{...projection.candidates[0].artifact,view:twoView,relations:[primary,...twoTargets]}}]}};
const pickStateBefore=JSON.stringify(pickState);
const renderPick=()=>host.render({projectId:'picking-project',wasmState:pickState,list,stage,toolbar,canvas:pickCanvas});
renderPick();assert.equal(stage.dataset.magnifierInstanceCount,'0');
pickCanvas.listeners.click({clientX:200,clientY:200});renderPick();
const oldPick=stage.querySelector('section');assert.equal(oldPick.dataset.authoredMagnifierId,'pick-0');
pickCanvas.listeners.click({clientX:291,clientY:200});renderPick();
assert.equal(oldPick.parent,null);
assert.equal(stage.querySelector('section').dataset.authoredMagnifierId,'pick-1');
assert.equal(stage.dataset.magnifierInstanceCount,'1');
assert.equal(JSON.stringify(pickState),pickStateBefore);
host.reset();assert.equal(pickCanvas.listeners.click,undefined,'reset removes owned event handler');
console.log(JSON.stringify({result:'PASS',scope:'presentation, explicit hit testing, stable object pose updates and DOM lifecycle only; activation commit atomicity not tested'}));
