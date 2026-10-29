import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const wasmDir=path.resolve(process.env.DDN_WASM_DIR),nativeEvidence=path.resolve(process.env.DDN_DERIVED_NATIVE_EVIDENCE);
const output=path.resolve(process.env.DDN_DERIVED_WASM_OUTPUT);assert(!fs.existsSync(output));fs.mkdirSync(output,{recursive:true});
let currentCase='startup';
process.on('uncaughtExceptionMonitor',error=>fs.writeFileSync(path.join(output,'FAILURE.json'),JSON.stringify({currentCase,error:String(error.stack??error)},null,2)));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:wasmBytes});
const rows=[];
function compare(state,baseline,label) {
    assert.equal(state.state_hash,baseline.state_hash,label+' hash');
    assert.deepEqual(state.output_display,baseline.output_display,label+' output');
    // Compare the same complete resource contract, not a display projection
    // (the existing WASM display omits Native's det/schema helper fields).
    assert.deepEqual(state.state.snapshot_v2.resources,baseline.engine_state.state.snapshot_v2.resources,label+' canonical resource snapshot');
    for(const lane of ['fixed64','handle','json','value','value_json'])
        assert.deepEqual(state.resources[lane],baseline.resources[lane],label+' display lane '+lane);
}
for(const kind of ['scalar','fraction_tuple']) {
    currentCase=kind;
    const fixture=path.join(nativeEvidence,'ddonilang-'+kind);
    const artifact=fs.readFileSync(path.join(fixture,'authored-project.json'),'utf8');
    const baseline=JSON.parse(fs.readFileSync(path.join(fixture,'continuous7.stdout'),'utf8'));
    const baseline8=JSON.parse(fs.readFileSync(path.join(fixture,'continuous8.stdout'),'utf8'));
    const vm=wasm.DdnWasmVm.from_authored_project_json(artifact);let restored,reopened;
    try {
        for(let i=0;i<3;i++)vm.step_one();
        const saved=vm.export_saved_project_artifact_json('','51');fs.writeFileSync(path.join(output,kind+'-saved3.json'),saved);
        for(let i=0;i<4;i++)vm.step_one();
        let state=JSON.parse(vm.get_state_json());
        compare(state,baseline,kind+' Native/WASM');
        restored=wasm.DdnWasmVm.from_saved_project_artifact_json(saved,'','51');
        for(let i=0;i<4;i++)restored.step_one();
        state=JSON.parse(restored.get_state_json());
        compare(state,baseline,kind+' WASM cold');
        const resaved=restored.export_saved_project_artifact_json('51','52');fs.writeFileSync(path.join(output,kind+'-saved7.json'),resaved);
        reopened=wasm.DdnWasmVm.from_saved_project_artifact_json(resaved,'51','52');
        assert.equal(reopened.get_state_hash(),baseline.state_hash);
        reopened.step_one();state=JSON.parse(reopened.get_state_json());
        compare(state,baseline8,kind+' resaved cold +1');
        rows.push({kind,state_hash_7:baseline.state_hash,state_hash_8:baseline8.state_hash,
            native_same_source_artifact_sha256:sha(artifact),cold_3_plus_4_equals_7:true,resaved7_cold1_equals_8:true,
            exact_resources_and_output_equal:true});
    } finally {reopened?.free();restored?.free();vm.free();}
}
for(const [id,source,code] of [
    ['cycle','앞값 <<- 뒷값 + 1.0. 뒷값 <<- 앞값 + 1.0.','E_DERIVED_CYCLE'],
    ['duplicate','결과 <<- 1.0. 결과 <<- 2.0.','E_DERIVED_DUPLICATE'],
    ['direct','결과 <<- 1.0. 결과 <- 2.0.','E_DERIVED_DIRECT_WRITE'],
    ['input','결과 <<- 입력키.','E_DERIVED_EFFECT'],
    ['host-default','결과 <<- 프레임수.','E_DERIVED_DEPENDENCY_REQUIRED'],
    ['nested','(매마디)마다 { 결과 <<- 1.0. }.','E_DERIVED_SCOPE'],
    ['bidirectional','앞값 <<->> 뒷값.','E_BIDIRECTIONAL_BINDING_UNSUPPORTED'],
]) {
    currentCase=id;
    const file=path.join(output,id+'.ddn');fs.writeFileSync(file,source);
    assert.throws(()=>wasm.wasm_run_summary_with_source_profile_json(source,file,1,'v1-core-v25'),error=>String(error).includes(code));
    for(const alias of ['ddonilang','ddonirang','또니랑']) {
        const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],{encoding:'utf8',timeout:120000});
        fs.writeFileSync(path.join(output,id+'-'+alias+'.stderr'),run.stderr??'');
        assert.equal(run.error,undefined);assert.notEqual(run.status,0);assert.equal(run.stdout,'');assert(run.stderr.includes(code),run.stderr);
    }
}
fs.writeFileSync(path.join(output,'RESULT.json'),JSON.stringify({claim:'DERIVED_WASM_NATIVE_CLI3_STORAGE_AND_REJECTION_NOT_CHROME_OR_FULL_BETA2',
    wasm_sha256:sha(wasmBytes),rows,raw_rejections_cli3_wasm:7},null,2));
console.log('PASS: Native/WASM scalar + rational Tuple save/cold/resave parity; raw rejection cases7 × CLI3/WASM');
