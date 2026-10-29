// Original source modules, actual project authoring and CLI3; not a WASM/browser verdict.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const [root,bin,out]=process.argv.slice(2);assert(root&&bin&&out);fs.mkdirSync(out);
const buildKind=process.env.DDN_NATIVE_BUILD_KIND??'release';assert(['release','debug'].includes(buildKind));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const report={scope:`ACTUAL_${buildKind.toUpperCase()}_CLI3_UNMODIFIED_LAB_MODULES_NOT_WASM_CHROME_OR_MATH_REVALIDATION`,native_build_kind:buildKind,cases:[]};
let serial=0;
function call(name,args){
 const result=spawnSync(path.join(bin,name+'.exe'),args,{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:64e6});assert.ifError(result.error);
 const stem=path.join(out,String(++serial).padStart(2,'0')+'-'+name);fs.writeFileSync(stem+'.stdout',result.stdout);fs.writeFileSync(stem+'.stderr',result.stderr);
 assert.equal(result.status,0,result.stderr);return JSON.parse(result.stdout);
}
function json(name,value){const file=path.join(out,name);fs.writeFileSync(file,JSON.stringify(value),{flag:'wx'});return file;}
try{
 for(const [entry,composition] of [['alpha3_double_pendulum_lab','진자실험짜임'],['alpha4_lrl_lab','LRL실험짜임']]){
  const rootPath=entry+'.project.ddn',entryPath=entry+'.ddn';
  const files=[rootPath,entryPath,'alpha4_drawing_common.ddn'].map(file=>({path:file,text:fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/samples',file),'utf8')}));
  const request=json(entry+'-create.json',{source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:rootPath,entry_path:entryPath,files}});
  const created=call('ddonilang',['작품자료','만들기',request]);
  const registration=json(entry+'-register.json',{original_json:JSON.stringify(created),module_name:'앱',source_path:entryPath,composition_definition:composition,make_entry:true});
  const authored=call('ddonilang',['작품자료','보개등록',registration]);
  assert.deepEqual(authored.source_snapshot.files,files,'project authoring must not rewrite the original modules');
  const artifact=json(entry+'-authored.json',authored);let baseline=null;const aliases=[];
  for(const name of ['ddonilang','ddonirang','또니랑']){
   const actual=call(name,['작품자료','실행',artifact,'--마디수','1']);
   assert.equal(actual.bogae_projection.persistent_identity,true);
   assert.deepEqual(actual.bogae_projection.candidates[0].artifact.relations.map(row=>row.kind),['primary','overlay','side','side']);
   if(baseline)for(const key of ['state_hash','resources','output_log_texts','bogae_projection','work_cost_receipt'])assert.deepEqual(actual[key],baseline[key],entry+': '+name+' '+key);
   else baseline=actual;
   aliases.push({name,exe_sha256:sha(fs.readFileSync(path.join(bin,name+'.exe')))});
  }
  report.cases.push({entry,source_files:files.map(file=>({path:file.path,sha256:sha(file.text)})),state_hash:baseline.state_hash,relations:['primary','overlay','side','side'],aliases});
 }
 report.verdict='PASS';
}catch(error){report.verdict='FAIL';report.error=String(error.stack??error);process.exitCode=1;}
fs.writeFileSync(path.join(out,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({out,verdict:report.verdict,error:report.error}));
