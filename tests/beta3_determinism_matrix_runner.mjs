// Real product entrypoints only. No DDN lowering, synthetic world, or hash repair.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL, fileURLToPath} from 'node:url';

const [stageArg, outputArg, selected='numeric'] = process.argv.slice(2);
assert(stageArg && outputArg, 'stage output [numeric|subject_atom|derived|double_pendulum|multifile]');
const stage=path.resolve(stageArg), out=path.resolve(outputArg);
assert(!fs.existsSync(out), 'evidence directory must be new');
fs.mkdirSync(out,{recursive:true});
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const sample=path.join(stage,'solutions/seamgrim_ui_mvp/samples');
const sha=x=>createHash('sha256').update(x).digest('hex');
const aliases=['ddonilang','ddonirang','또니랑'];
const groups=['numeric','subject_atom','derived','logical_tetris','double_pendulum','multifile'];
const routes=[...aliases,'WASM','Workbench'];
const methods=['continuous','cold_resume','repeat'];
const target=JSON.parse(fs.readFileSync(path.join(stage,'BETA3_DEVELOPMENT_MANIFEST.json'),'utf8'));
assert.match(target.fixed_commit??'',/^[a-f0-9]{40}$/);
const matrix={scope:'WO23_F1_IN_PROGRESS_NOT_COMPLETE_ROW_OR_HUMAN',baseline_product:'6b2e640f8333a03f5cbd76eead0654d44abb00d7',
  cells:groups.flatMap(group=>routes.flatMap(route=>methods.map(method=>({group,route,method,status:'NOT_RUN'})))),
  invariants:{pacing:'NOT_RUN',view_toggles:'NOT_RUN',wall_clock:'NOT_RUN'},runs:[],verdict:'INCOMPLETE'};
// Current evidence identity only; original raw fixtures, comparison channels,
// reference construction and every acceptance assertion below stay unchanged.
matrix.baseline_product=target.fixed_commit;matrix.target=target;
matrix.native_binary_sha256=Object.fromEntries(aliases.map(alias=>[alias,sha(fs.readFileSync(path.join(stage,'bin',alias+'.exe')))]));
const flush=()=>fs.writeFileSync(path.join(out,'B3_DETERMINISM_MATRIX.json'),JSON.stringify(matrix,null,2)+'\n');
flush();
let sequence=0,vm;
const parse=value=>typeof value==='string'?JSON.parse(value):value;
function cli(alias,args){
  const result=spawnSync(path.join(stage,'bin',alias+'.exe'),['작품자료',...args],
    {encoding:'utf8',windowsHide:true,cwd:stage,timeout:180000,maxBuffer:32*1024*1024});
  const prefix=path.join(out,String(++sequence).padStart(3,'0')+'_'+alias);
  fs.writeFileSync(prefix+'.stdout',result.stdout??'');fs.writeFileSync(prefix+'.stderr',result.stderr??'');
  matrix.runs.push({alias,args,status:result.status,stdout_sha256:sha(result.stdout??''),stderr_sha256:sha(result.stderr??'')});flush();
  assert.ifError(result.error);assert.equal(result.status,0,result.stderr);
  return JSON.parse(result.stdout);
}
function mark(route,method,evidence){
  const cell=matrix.cells.find(c=>c.group===selected&&c.route===route&&c.method===method);
  assert(cell);Object.assign(cell,{status:'PASS',...evidence});flush();
}
// Compare typed/public channels separately; never normalize numbers or omit a
// missing channel to hide a Native/WASM boundary mismatch.
function compareSummary(actual,expected){
  assert.equal(actual.state_hash,expected.state_hash,'state_hash');
  assert.deepEqual(actual.output_display,expected.output_display,'output_display');
  for(const channel of ['fixed64','handle','json','value','value_json']){
    assert.deepEqual(actual.resources[channel],expected.resources[channel],`resources.${channel}`);
  }
  assert.deepEqual(actual.state?.snapshot_v2?.resources?.value_det??actual.resources.value_det,
    expected.resources.value_det,'typed resource values');
  assert.deepEqual(actual.bogae_projection,expected.bogae_projection,'declared bogae result');
}
try{
  let sourceSnapshot;
  if(selected==='numeric'||selected==='multifile') sourceSnapshot=JSON.parse(fs.readFileSync(path.join(sample,
    selected==='numeric'?'beta1_typed_tuple/authored-project.json':'beta1_logical_counter/authored-project.json'),'utf8')).source_snapshot;
  else if(selected==='subject_atom') sourceSnapshot={root_path:'root.ddn',entry_path:'main.ddn',files:[
    {path:'root.ddn',text:'(글판="v25", 모듬=차림[(이름=앱, 머리="main.ddn", 원문="main.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.\n'},
    {path:'main.ddn',text:'명찰:이름씨 := { 이름:글. }. 병사:임자 := { 이름:글. }. 채비 { 횟수:바른수 := 0. }. 실행:움직씨 := { (매마디)마다 { 횟수 <- 횟수 + 1. 표적 <- (이름="철수")인 병사. 색 <- #빨강. 횟수 보여주기. }. }.\n'}]};
  else if(selected==='derived') sourceSnapshot={root_path:'root.ddn',entry_path:'main.ddn',files:[
    {path:'root.ddn',text:'(글판="v25", 모듬=차림[(이름=앱, 머리="main.ddn", 원문="main.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.\n'},
    {path:'main.ddn',text:fs.readFileSync(path.join(sample,'beta2_derived/main.ddn'),'utf8')}]};
  else if(selected==='double_pendulum')sourceSnapshot={root_path:'root.ddn',entry_path:'alpha3_double_pendulum_lab.ddn',files:[
    {path:'root.ddn',text:fs.readFileSync(path.join(sample,'alpha3_double_pendulum_lab.project.ddn'),'utf8')},
    ...['alpha3_double_pendulum_lab.ddn','alpha4_drawing_common.ddn'].map(name=>({path:name,text:fs.readFileSync(path.join(sample,name),'utf8')}))]};
  else throw Error('This execution slice does not yet supply '+selected+'; its cells remain NOT_RUN.');
  matrix.raw_source_files=sourceSnapshot.files.map(f=>({path:f.path,sha256:sha(f.text)}));
  const request=path.join(out,'create.json');
  fs.writeFileSync(request,JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:sourceSnapshot}));
  const created=cli(aliases[0],['만들기',request]);
  const artifact=path.join(out,'authored-project.json');fs.writeFileSync(artifact,JSON.stringify(created));
  assert.deepEqual(created.source_snapshot,sourceSnapshot,'raw source bytes retained');
  const continuous=cli(aliases[0],['실행',artifact,'--마디수','7']);
  const continuous8=cli(aliases[0],['실행',artifact,'--마디수','8']);
  const store=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-b3-determinism-'));
  matrix.store=store;
  const source=cli(aliases[0],['원문저장',artifact,'--판','01','--저장소',store]);
  const sourceFile=path.join(out,'source-report.json');fs.writeFileSync(sourceFile,JSON.stringify(source));
  const saved=path.join(out,'saved3.json');
  const first=cli(aliases[0],['첫실행',sourceFile,'--판','02','--저장소',store,'--마디수','3','--저장파일',saved]);
  assert.equal(fs.readFileSync(saved,'utf8'),first.saved_artifact_json_utf8);
  const savedBytes=fs.readFileSync(saved),savedHash=sha(savedBytes);
  const resumeArgs=['원문이어실행',saved,'--부모','01','--판','02','--저장소',store,'--허가',source.grant_locator,'--마디수','4'];
  for(const alias of aliases){
    const direct=cli(alias,['실행',artifact,'--마디수','7']);compareSummary(direct,continuous);
    mark(alias,'continuous',{state_hash:direct.state_hash});
    const cold=cli(alias,resumeArgs);compareSummary(cold,continuous);
    mark(alias,'cold_resume',{state_hash:cold.state_hash,saved_sha256:savedHash,split:'3+4=7',new_os_process:true});
    const repeated=cli(alias,resumeArgs);assert.deepEqual(repeated,cold,'same saved identity replay complete summary');
    mark(alias,'repeat',{state_hash:repeated.state_hash,complete_summary_equal:true});
  }
  assert.equal(sha(fs.readFileSync(saved)),savedHash,'cold execution never rewrites source snapshot');
  const wasmDir=path.join(stage,'solutions/seamgrim_ui_mvp/ui/wasm');
  const bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
  matrix.wasm_sha256=sha(bytes);
  const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
  await wasm.default({module_or_path:bytes});
  let wasmSequence=0;
  const runWasm=()=>{
    vm=wasm.DdnWasmVm.from_saved_project_artifact_json(savedBytes.toString('utf8'),'01','02');
    let frame;for(let i=0;i<4;i++)frame=parse(vm.step_one());
    fs.writeFileSync(path.join(out,`WASM_COLD_${++wasmSequence}.json`),JSON.stringify(frame));
    compareSummary(frame,continuous);
    const exported=vm.export_saved_project_artifact_json('02','03');
    const state=parse(vm.get_state_json());vm.free();vm=null;
    return {frame,exported,state};
  };
  const wasmFirst=runWasm();mark('WASM','cold_resume',{state_hash:wasmFirst.frame.state_hash,saved_sha256:savedHash,split:'3+4=7'});
  const wasmRepeat=runWasm();assert.deepEqual(wasmRepeat,wasmFirst);
  mark('WASM','repeat',{state_hash:wasmRepeat.frame.state_hash,full_frame_state_and_saved_artifact_equal:true});
  vm=wasm.DdnWasmVm.from_authored_project_json(JSON.stringify(created));
  let wasmContinuous;for(let i=0;i<7;i++)wasmContinuous=parse(vm.step_one());
  fs.writeFileSync(path.join(out,'WASM_CONTINUOUS.json'),JSON.stringify(wasmContinuous));
  compareSummary(wasmContinuous,continuous);vm.free();vm=null;
  mark('WASM','continuous',{state_hash:wasmContinuous.state_hash,same_authored_identity:true});
  // Fresh WASM reopening of its real saved output, not JSON state injection.
  vm=wasm.DdnWasmVm.from_saved_project_artifact_json(wasmFirst.exported,'02','03');
  compareSummary(parse(vm.step_one()),continuous8);vm.free();vm=null;
  matrix.wasm_resave_cold_next_equal=true;
  matrix.verdict='PARTIAL_PASS';
}catch(error){matrix.verdict='FAIL';matrix.error=String(error.stack??error);process.exitCode=1;}
finally{vm?.free();flush();console.log(JSON.stringify({out,group:selected,verdict:matrix.verdict,error:matrix.error}));}
