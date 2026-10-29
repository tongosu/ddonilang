// Actual executable boundary; no DDN rewriting or test-only lowering.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import assert from 'node:assert/strict';
const [repo,executables,out,legacyFixtures]=process.argv.slice(2);
assert(repo&&executables&&out&&legacyFixtures,'repo executable-directory fresh-output-directory fixed-legacy-fixture-directory');
fs.mkdirSync(out); // Preserve previous evidence; require a fresh exact directory.
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const write=(name,data)=>{const file=path.join(out,name);fs.writeFileSync(file,data,{flag:'wx'});return file;};
const json=(name,value)=>write(name,JSON.stringify(value));
const sample=name=>path.join(repo,'solutions/seamgrim_ui_mvp/samples',name);
const readJson=file=>JSON.parse(fs.readFileSync(file,'utf8'));
const held={'다시시작':false,'떨어뜨리기':false,'아래':false,'오른쪽':false,'왼쪽':false,'회전':false};
const logical={name:'logical',source:sample('alpha2_logical_tetris.ddn'),binding:readJson(sample('alpha2_logical_tetris.binding.json')),
    snapshots:[{madi:0,events:[],held},{madi:1,events:[{ordinal:0,channel:'왼쪽',event:{kind:'Press'}}],held:{...held,'왼쪽':true}},
    {madi:2,events:[{ordinal:0,channel:'왼쪽',event:{kind:'Release'}},{ordinal:1,channel:'회전',event:{kind:'Press'}}],held:{...held,'회전':true}}]};
const manipulation={name:'manipulation',source:sample('alpha4_manipulation_point.ddn'),binding:readJson(sample('alpha4_manipulation_point.binding.json')),
    snapshots:[{madi:0,held:{},events:[
        {ordinal:0,channel:'반지름',event:{kind:'Number',payload:{raw:'4294967296',phase:'Set'}}},
        ...['Begin','Move','End'].map((phase,i)=>({ordinal:i+1,channel:'위치',event:{kind:'Coordinate',payload:{raw:[String(i*1073741824),'0'],phase}}}))]}]};
const results=[];
process.on('uncaughtException',error=>{
    write('failure.json',JSON.stringify({scope:'RAW_FAILURE_PRESERVED',completed_workflows:results,error:String(error),stack:error.stack},null,2));
    console.error(error);process.exitCode=1;
});
for(const entry of [logical,manipulation]){
    const raw=fs.readFileSync(entry.source),sourceSha=sha(raw);
    entry.binding.source_revision='sha256:'+sourceSha; // Explicit caller source binding; raw source unchanged.
    const binding=json(entry.name+'-binding.json',entry.binding);
    const request=json(entry.name+'-request.json',{binding:entry.binding,snapshots:entry.snapshots});
    const wrong=json(entry.name+'-wrong-binding.json',{...entry.binding,initial_rng:entry.binding.initial_rng+1});
    for(const name of ['ddonilang','ddonirang','또니랑']){
        const exe=path.join(executables,name+'.exe');
        const invoke=(label,args,success=true)=>{
            const result=spawnSync(exe,args,{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000,windowsHide:true});
            if(result.error)throw result.error;
            write(`${entry.name}-${name}-${label}.stdout`,result.stdout);write(`${entry.name}-${name}-${label}.stderr`,result.stderr);
            if(success)assert.equal(result.status,0,result.stderr);else assert.notEqual(result.status,0,'must explicitly reject');
            return success?JSON.parse(result.stdout):result;
        };
        const tape=path.join(out,`${entry.name}-${name}-v3.bin`),legacy=path.join(out,`${entry.name}-${name}-legacy.bin`);
        const original=invoke('record',[entry.name+'-record',entry.source,request,tape,'--format=v3']);
        assert(fs.readFileSync(tape).subarray(0,18).equals(Buffer.from('DDN_INPUT_TAPE_V3\n')));
        const replayed=invoke('replay',[entry.name+'-replay',entry.source,tape,binding,'--format=v3']);
        assert.deepEqual(replayed,original);
        const normalTape=path.join(out,`${entry.name}-${name}-default-v3.bin`);
        const normal=invoke('default-record',[entry.name+'-record',entry.source,request,normalTape]);
        assert.deepEqual(normal,original,'default and explicit V3 must use the same semantic owner');
        assert.deepEqual(fs.readFileSync(normalTape),fs.readFileSync(tape));
        const legacyBytes=fs.readFileSync(path.join(legacyFixtures,`${entry.name}-${name}-legacy.bin`));
        write(path.basename(legacy),legacyBytes); // Preserve fixed old bytes; never reconstruct events.
        invoke('reject-legacy',[entry.name+'-replay',entry.source,legacy,binding,'--format=v3'],false);
        invoke('reject-legacy-default',[entry.name+'-replay',entry.source,legacy,binding],false);
        invoke('reject-binding',[entry.name+'-replay',entry.source,tape,wrong,'--format=v3'],false);
        results.push({case:entry.name,exe:name,source:entry.source,source_sha256:sourceSha,exe_sha256:sha(fs.readFileSync(exe)),
            tape_sha256:sha(fs.readFileSync(tape)),state_hash:original.state_hash,madi_count:original.madi_count,
            replay_equal:true,default_explicit_v3_equal:true,legacy_fixture_sha256:sha(legacyBytes),legacy_implicit_read_rejected:true,wrong_binding_rejected:true});
    }
}
write('result.json',JSON.stringify({scope:'DEBUG_CLI3_ACTUAL_RAW_RECORD_REPLAY_NOT_WASM_OR_RELEASE',results},null,2));
console.log(JSON.stringify({workflows:results.length,failures:0,report:path.join(out,'result.json')}));
