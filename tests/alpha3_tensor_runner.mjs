// RD-385..392/510/674/682/683; A3-P6 resource profile remains PROVISIONAL.
// These are unchanged authored sources passed through each product frontdoor.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {verifyTensorPhysicalCase,poisonTensorTransport} from './alpha3_tensor_physical_oracle.mjs';
assert.ok(process.env.DDN_NATIVE_DIR && process.env.DDN_WASM_DIR);
const wasmDir=process.env.DDN_WASM_DIR;
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))});
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-tensor-'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const json=v=>typeof v==='string'?JSON.parse(v):v;
const prefix='판 <- (차림=차림[차림[1,2,3],차림[4,5,6]],원소씨=셈수,모양=(2,3)) 텐서로.\n';
const cases=[
  {id:'slice-and-persistent',source:prefix+`옛판 <- 판. 가운데 <- 판[:,1..2].
새열 <- (차림=차림[차림[8],차림[9]],원소씨=셈수,모양=(2,1)) 텐서로.
판[:,1..2] <- 새열. 판[1,2] <- 7. 판[0..1,:][0][0] <- 10.
새열 -> 판[:,1..2]. 검산 <- 판[0,0]. 옛값 <- 옛판[0,0].`,expected:{검산:'10',옛값:'1'}},
  {id:'scalar-rankzero-empty',source:`영차원 <- (차림=7,원소씨=셈수,모양=()) 텐서로.
벡터 <- (차림=차림[1,2],원소씨=셈수,모양=(2,)) 텐서로.
빈판 <- (차림=() 차림,원소씨=셈수,모양=(0,3)) 텐서로.
검산 <- (텐서=영차원) 차림으로. 셀 <- 벡터[1].`,expected:{검산:'7',셀:'2'}},
  {id:'dynamic-scalar-static-range',source:prefix+'축 <- 1. 행 <- 판[축,:]. 검산 <- 행[2]. 빈판 <- 판[2..2,:].',expected:{검산:'6'}},
  ...[
    ['shape-replace','판 <- (차림=차림[1,2],원소씨=셈수,모양=(2,)) 텐서로.'],
    ['broadcast','판[:] <- 9.'],
    ['wrong-slice-shape','판[:,1..2] <- (차림=차림[8,9],원소씨=셈수,모양=(2,)) 텐서로.'],
    ['too-many-axes','검산 <- 판[0,0,0].'],
    ['fractional-axis','검산 <- 판[0.5,0].'],
    ['negative-axis','검산 <- 판[-1,0].'],
    ['out-of-range','검산 <- 판[2,0].'],
    ['reversed-static-range','검산 <- 판[2..1,:].'],
    ['negative-static-range','검산 <- 판[-1..1,:].'],
    ['dynamic-range-bound','끝 <- 1. 검산 <- 판[0..끝,:].'],
    ['wrong-element-family','다른판 <- (차림=차림[1,2],원소씨=글,모양=(2,)) 텐서로.'],
    ['ragged','다른판 <- (차림=차림[차림[1],차림[2,3]],원소씨=셈수,모양=(2,2)) 텐서로.'],
  ].map(([id,invalid])=>({id,source:prefix+invalid,error:true})),
];
const rows=[];
for(const c of cases){
  const file=path.join(dir,c.id+'.ddn');fs.writeFileSync(file,c.source,'utf8');
  let canonicalSource;
  if(!c.error){
    // VC14 complete-row canon evidence uses the product frontdoors, not a
    // harness rewrite. Canon must retain static shape/Selection distinctions.
    canonicalSource=wasm.wasm_canon_ddn_with_source_profile(c.source,'v1-core-v25');
    assert.equal(wasm.wasm_canon_ddn_with_source_profile(canonicalSource,'v1-core-v25'),canonicalSource,c.id+'/WASM canon idempotence');
    for(const name of ['ddonirang','ddonilang','또니랑']){
      const canon=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['개발','정본',file,'--source-profile','v1-core-v25'],{encoding:'utf8',windowsHide:true,timeout:60000});
      assert.ifError(canon.error);assert.equal(canon.status,0,canon.stderr);
      assert.equal(canon.stdout.trimEnd(),canonicalSource.trimEnd(),c.id+'/Native WASM canon');
    }
  }
  let nativeResult,nativeError,physicalOracle;
  for(const name of ['ddonirang','ddonilang','또니랑']){
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['실행',file,'--source-profile','v1-core-v25','--마디수','1','--json'],{encoding:'utf8',windowsHide:true,timeout:60000,maxBuffer:16*1024*1024});
    assert.ifError(run.error);
    if(c.error){
      assert.notEqual(run.status,0,c.id+'/explicit refusal');
      const diagnostic=JSON.parse((run.stderr||run.stdout).trim());
      if(nativeError)assert.deepEqual(diagnostic,nativeError,c.id+'/Native3 diagnostic');nativeError=diagnostic;
    }else{
      assert.equal(run.status,0,run.stderr);const result=JSON.parse(run.stdout);
      for(const [name,value] of Object.entries(c.expected))assert.equal(result.resources.fixed64[name],value,c.id+'/'+name);
      assert.equal(result.resources.value_det.판?.type??result.resources.value_det.영차원?.type,'tensor_v1');
      if(nativeResult)assert.deepEqual(result,nativeResult,c.id+'/Native3');nativeResult=result;
    }
  }
  if(c.error){
    let error;
    try{wasm.wasm_run_summary_with_source_profile_json(c.source,file,1,'v1-core-v25');assert.fail('must reject');}catch(e){error=json(e);}
    if(nativeError.schema==='ddn.source_analysis_snapshot.candidate.v1'){
      // Native execution and Workbench first enter the common typed analysis
      // frontdoor. The lower-level summary API wraps parse failures in an
      // execution diagnostic; it is not the source-analysis wire.
      const analyzed=json(wasm.wasm_source_analysis_snapshot_json(JSON.stringify({source_text:c.source,path:file,analysis_generation:0,source_profile_identity:'v1-core-v25'})));
      assert.deepEqual(analyzed,nativeError,c.id+'/CLI Workbench WASM analysis exact');
      assert.ok(error.code,c.id+'/lower-level summary also explicitly rejects');
    }else assert.deepEqual(error,nativeError,c.id+'/CLI-WASM exact runtime diagnostic');
  }else{
    const result=json(wasm.wasm_run_summary_with_source_profile_json(c.source,file,1,'v1-core-v25'));
    assert.equal(result.state_hash,nativeResult.state_hash,c.id+'/world');
    assert.deepEqual(result.resources,nativeResult.resources,c.id+'/resources');
    assert.deepEqual(result.output_display,nativeResult.output_display,c.id+'/output');
    physicalOracle=verifyTensorPhysicalCase(c.id,result.resources);
    assert.deepEqual(verifyTensorPhysicalCase(c.id,nativeResult.resources),physicalOracle,c.id+'/Native WASM physical oracle');
    const canonicalResult=json(wasm.wasm_run_summary_with_source_profile_json(canonicalSource,file,1,'v1-core-v25'));
    assert.equal(canonicalResult.state_hash,result.state_hash,c.id+'/canon preserves world');
    assert.deepEqual(canonicalResult.resources,result.resources,c.id+'/canon preserves tensor shape and values');
    for(const leaf of Object.values(result.resources.value_det).filter(v=>v?.type==='tensor_v1')){
      const carrier=Buffer.from(leaf.carrier_base64url,'base64url');
      assert.equal(carrier.length,leaf.carrier_byte_length);
      assert.equal(sha(carrier),leaf.carrier_sha256);
      assert.equal(sha(Buffer.concat([Buffer.from([0x0e]),carrier])),leaf.resource_value_sha256);
    }
  }
  const vm=new wasm.DdnWasmVm('');
  try{
    if(c.error && nativeError.schema==='ddn.source_analysis_snapshot.candidate.v1'){
      vm.update_logic_with_source_profile(prefix,'v1-core-v25');vm.reset(false);vm.step_one();
      const saved=vm.get_state_json(),hash=vm.get_state_hash();
      assert.throws(()=>vm.update_logic_with_source_profile(c.source,'v1-core-v25'));
      assert.equal(vm.get_state_json(),saved,c.id+'/failed compile preserves whole active state');
      assert.equal(vm.get_state_hash(),hash,c.id+'/failed compile preserves world');
      rows.push({id:c.id,result:'EXPLICIT_REJECT',source_sha256:sha(c.source),canonical_source_sha256:null,canon_product_parity:'NOT_APPLICABLE',state_hash:null,diagnostic:nativeError,failed_compile_atomic:true});
      continue;
    }
    vm.update_logic_with_source_profile(c.source,'v1-core-v25');vm.reset(false);
    const before=vm.get_state_json(),beforeHash=vm.get_state_hash();
    if(c.error){assert.throws(()=>vm.step_one());assert.equal(vm.get_state_json(),before,c.id+'/whole madi atomic');assert.equal(vm.get_state_hash(),beforeHash);}
    else{
      vm.step_one();const saved=vm.get_state_json(),hash=vm.get_state_hash();
      vm.reset(false);vm.restore_state(saved);assert.equal(vm.get_state_hash(),hash,c.id+'/restore');
      // get_state_json also projects ephemeral debug evidence. restore_state
      // intentionally resets that local field (wasm_api.rs), not world bytes.
      const restored=vm.get_state_json();
      const worldBefore=json(saved),worldAfter=json(restored);
      function withoutDebug(v){if(!v||typeof v!=='object')return;delete v.debug_pause;for(const child of Object.values(v))withoutDebug(child);}
      assert.deepEqual(worldAfter.debug_pause,{triggered:false,reasons:[]},c.id+'/local debug reset');
      withoutDebug(worldBefore);withoutDebug(worldAfter);
      assert.deepEqual(worldAfter,worldBefore,c.id+'/all non-debug snapshot fields');
      const corrupt=json(saved);
      function poisonLeaf(v){if(!v||typeof v!=='object')return false;if(v.type==='tensor_v1'){v.carrier_sha256='0'.repeat(64);return true;}return Object.values(v).some(poisonLeaf);}
      function poison(v){if(!v||typeof v!=='object')return false;if(v.value_det&&poisonLeaf(v.value_det))return true;return Object.values(v).some(poison);}
      assert.ok(poison(corrupt));assert.throws(()=>vm.restore_state(JSON.stringify(corrupt)));
      assert.equal(vm.get_state_hash(),hash,c.id+'/failed restore hash');
      assert.equal(vm.get_state_json(),restored,c.id+'/failed restore whole state');
      if(c.id==='slice-and-persistent'){
        for(const kind of ['element-identity','shape-count','element-value-tag']){
          const poisoned=json(saved);
          let mutations=0;
          function visit(value){
            if(!value||typeof value!=='object')return;
            if(value.type==='tensor_v1'){poisonTensorTransport(value,kind);mutations++;return;}
            for(const child of Object.values(value))visit(child);
          }
          visit(poisoned);assert.ok(mutations,kind+'/actual saved Tensor coverage');
          assert.throws(()=>vm.restore_state(JSON.stringify(poisoned)),kind+'/typed admission rejects recomputed digests');
          assert.equal(vm.get_state_hash(),hash,kind+'/world atomic');
          assert.equal(vm.get_state_json(),restored,kind+'/whole state atomic');
          vm.restore_state(saved);assert.equal(vm.get_state_hash(),hash,kind+'/exact restore GREEN');
          assert.equal(vm.get_state_json(),restored,kind+'/restored state GREEN');
        }
        physicalOracle.typed_mutations=['element-identity','shape-count','element-value-tag'];
      }
    }
  }finally{vm.free();}
  rows.push({id:c.id,result:c.error?'EXPLICIT_REJECT':'PASS',physical_oracle:physicalOracle??null,source_sha256:sha(c.source),canonical_source_sha256:canonicalSource?sha(canonicalSource):null,canon_product_parity:c.error?'NOT_APPLICABLE':'PASS',state_hash:nativeResult?.state_hash??null,diagnostic:nativeError??null});
}
const report={status:'PASS',ssot:'RD-385..392/510/674/682/683',provisional:'A3-P6',native_aliases:3,actual_wasm:true,wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),rows};
fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify({...report,evidence_directory:dir}));
