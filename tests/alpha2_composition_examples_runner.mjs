import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';

// 실제 원문만 제품 frontdoor에 전달한다. 장면 표본은 정식 chart 지원 증거가 아니다.
assert.ok(process.env.DDN_NATIVE_DIR, 'source-built native directory required');
assert.ok(process.env.DDN_WASM_DIR, 'source-built WASM directory required');
const wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
const wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
await wasm.default({module_or_path:wasmBytes});
const Q=1n<<32n;
function round(n,d) {
  const sign=n<0n?-1n:1n, a=n*sign, quotient=a/d, remainder=a%d;
  return sign*(quotient+(remainder*2n>d || (remainder*2n===d && quotient%2n) ? 1n:0n));
}
const fixed=(n,d=1n)=>round(n*Q,d);
const mul=(a,b)=>round(a*b,Q);
const number=raw=>Number(raw)/Number(Q);
function sampleTime(ticks,end) {
  let raw=0n;
  for(let i=0;i<ticks;i++) {raw+=fixed(1n,50n);if(raw>end*Q) raw=0n;}
  return raw;
}
function scalar(value) {assert.equal(value.kind,'fixed64');return BigInt(value.raw);}
const reports=[];
const examples=process.env.DDN_COMPOSITION_EXAMPLE?[process.env.DDN_COMPOSITION_EXAMPLE]:['parabola','pendulum','linear'];
assert.ok(examples.every(name=>['parabola','pendulum','linear'].includes(name)));
for(const name of examples) {
  const file=path.resolve(`solutions/seamgrim_ui_mvp/samples/alpha2_${name}_composition.ddn`);
  const source=fs.readFileSync(file,'utf8');
  assert.ok(!source.includes('얼개'), 'retired entry surface must not be introduced');
  const objectIds=new Map();
  for(const ticks of name==='linear'?[1]:[1,17,51,101,201]) {
    const summary=JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source,file,ticks,'v1-core-v25'));
    const projection=summary.bogae_projection;
    assert.equal(projection.status,'ready');assert.equal(projection.candidates.length,1);
    const artifact=projection.candidates[0].artifact, relations=artifact.relations;
    const kinds=name==='linear'?['primary','overlay','side',...Array(21).fill('magnifier')]:name==='pendulum'?['primary','overlay','side','side','magnifier','magnifier']:['primary','overlay','side','side','magnifier'];
    assert.deepEqual(relations.map(row=>row.kind),kinds);
    assert.equal(new Set(relations.map(row=>row.relation_id)).size,kinds.length);
    assert.equal(relations[1].order,1);assert.equal(relations[2].placement,'오른쪽');
    assert.deepEqual(relations[0].view.horizontal,relations[1].view.horizontal);
    assert.deepEqual(relations[0].view.vertical,relations[1].view.vertical);
    assert.ok(relations.every(row=>row.time_policy.kind==='common'));
    const magnifiers=relations.filter(row=>row.kind==='magnifier');
    const magnifier=magnifiers[0];
    const target=magnifier.target.element.center.map(BigInt);
    for(const relation of magnifiers) {
      assert.equal(relation.object_identity.identity.scope,'temporary_execution_object');
      assert.ok(relation.object_identity.id);
      const previous=objectIds.get(relation.object_identity.label);
      if(previous) assert.equal(relation.object_identity.id,previous,'pose changes must preserve explicitly bound object identity');
      objectIds.set(relation.object_identity.label,relation.object_identity.id);
      assert.equal(relation.target.kind,'scene_element');
      assert.equal(relation.selection.kind,'primary_scene_element.v1');
      assert.equal(relation.selection.element_indices.length,1);
      assert.deepEqual(artifact.view.elements[relation.selection.element_indices[0]],relation.target.element);
    }
    assert.equal(new Set(magnifiers.map(row=>row.object_identity.id)).size,magnifiers.length,
      'distinct authored objects must not share identity');
    const position=magnifier.view.elements.find(row=>row.kind==='circle').center.map(BigInt);
    assert.deepEqual(position,target.slice(0,2),'sealed magnifier renders precisely its supplied target');
    if(name==='parabola') {
      const t=sampleTime(ticks,2n), x=5n*t, y=10n*t-mul(mul(5n*Q,t),t);
      assert.deepEqual(target,[x,y]);
      assert.equal(relations[2].view.kind,'scene');assert.equal(relations[3].view.kind,'scene');
      assert.equal(relations[3].placement,'오른쪽');
      const xPoints=relations[2].view.elements.filter(row=>row.kind==='circle');
      const points=relations[3].view.elements.filter(row=>row.kind==='circle');
      assert.equal(xPoints.length,22);
      assert.equal(points.length,22);
      for(let n=0;n<=20;n++) {
        const s=fixed(BigInt(n),10n), h=10n*s-mul(mul(5n*Q,s),s);
        assert.deepEqual(xPoints[n].center,[s.toString(),(5n*s).toString()]);
        assert.deepEqual(points[n].center,[s.toString(),h.toString()]);
      }
    } else if(name==='linear') {
      magnifiers.forEach((relation,index)=>{
        const x=fixed(BigInt(index-8),4n);
        assert.deepEqual(relation.target.element.center,[x,2n*x+Q].map(String));
      });
      assert.equal(relations[2].view.kind,'result_table');
      assert.deepEqual(relations[1].view.elements[0].vertices,
        [[0n,Q],[Q,Q],[Q,3n*Q]].map(row=>row.map(String)));
      const table=summary.public_observation_table_projection.tables[0];
      assert.deepEqual(table.columns.map(column=>column.label),['x','y','기울기']);
      assert.equal(table.rows.length,6);
      table.rows.forEach((row,index)=>assert.deepEqual(row.cells.map(cell=>cell.value_canon),
        [String(index-2),String(2*(index-2)+1),'2']));
    } else {
      const t=number(sampleTime(ticks,4n));
      const theta=0.5*Math.cos(Math.sqrt(9.8)*t);
      assert.ok(Math.abs(number(target[0])-Math.sin(theta))<2e-8);
      assert.ok(Math.abs(number(target[1])+Math.cos(theta))<2e-8);
      assert.deepEqual(magnifiers[1].target.element.center,['0','0'],'fixed pivot is a separate selectable target');
      assert.equal(relations[2].view.kind,'scene');
      const anglePoints=relations[2].view.elements.filter(row=>row.kind==='circle');
      const energyPoints=relations[3].view.elements.filter(row=>row.kind==='circle');
      assert.equal(relations[3].placement,'오른쪽');
      assert.equal(anglePoints.length,42);assert.equal(energyPoints.length,42);
      for(let n=0;n<=40;n++) {
        const s=fixed(BigInt(n),10n);
        assert.equal(anglePoints[n].center[0],s.toString());
        assert.ok(Math.abs(number(BigInt(anglePoints[n].center[1]))-0.5*Math.cos(Math.sqrt(9.8)*number(s)))<2e-8);
        assert.deepEqual(energyPoints[n].center,[s.toString(),fixed(49n,40n).toString()]);
      }
    }
    for(const alias of ['ddonirang','ddonilang','또니랑']) {
      const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${alias}.exe`),
        ['실행',file,'--source-profile','v1-core-v25','--마디수',String(ticks),'--json'],
        {encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:32*1024*1024});
      assert.ifError(run.error);assert.equal(run.status,0,`${alias}/${name}/${ticks}: ${run.stderr}`);
      const native=JSON.parse(run.stdout);
      for(const key of ['state_hash','resources','bogae_projection','public_observation_table_projection','work_cost_receipt'])
        assert.deepEqual(native[key],summary[key],`${alias}/${name}/${ticks}: ${key}`);
    }
    reports.push({name,ticks,state_hash:summary.state_hash});
  }
  console.log(JSON.stringify({name,source_sha256:createHash('sha256').update(source).digest('hex'),result:'PASS'}));
}
console.log(JSON.stringify({result:'ALPHA2_COMPOSITION_EXAMPLES_PASS',actual_source:true,native_aliases:3,
  actual_wasm_sha256:createHash('sha256').update(wasmBytes).digest('hex'),runs:reports,
  scope:'Authored scene samples and exact CLI/WASM parity; formal chart constructors/browser/Human/F3 closure not claimed'}));
