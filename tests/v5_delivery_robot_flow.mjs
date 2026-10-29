import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { resolve, dirname } from 'node:path';
const req=createRequire(import.meta.url), bin=process.env.DDN_PRODUCT_BIN_DIR, wasm=process.env.DDN_PRODUCT_WASM_NODE, out=process.env.DDN_V5_EVIDENCE;
assert.ok(bin&&wasm&&out,'고정 Native/WASM/근거 경로를 명시하세요.');
const api=req(resolve(wasm)), project=resolve(dirname(fileURLToPath(import.meta.url)),'../solutions/seamgrim_ui_mvp/samples/v5_delivery_robot');
const root=readFileSync(resolve(project,'root.ddn'),'utf8'), original=readFileSync(resolve(project,'main.ddn'),'utf8'), rows=[];
for(const [name,text,ticks,expected,arrived] of [['initial',original,1,0.0625,false],['speed1of2',original.replace('속도:수 := 1 / 16.','속도:수 := 1 / 2.'),16,8,true]]){
 const dir=resolve(out,name);mkdirSync(dir,{recursive:true});const files=[{path:'root.ddn',text:root},{path:'main.ddn',text}];
 const created=api.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:'root.ddn',entry_path:'main.ddn',files}}));
 const artifact=api.authored_project_operation('register_bogae',JSON.stringify({original_json:created,module_name:'앱',source_path:'main.ddn',composition_definition:'배달짜임',make_entry:true}));
 writeFileSync(resolve(dir,'registered.json'),artifact);assert.equal(api.authored_project_operation('open',artifact),artifact);
 const v={schema:'ddn.virtual_project_source.v1',root_path:'root.ddn',entry_path:'main.ddn',files:files.map(f=>({path:f.path,role:f.path==='root.ddn'?'root':'ddn_source',utf8_text:f.text}))};
 const linked=JSON.parse(api.wasm_project_prepare_link_json(JSON.stringify(v),'실행',true,false));
 const node=JSON.parse(api.wasm_run_summary_with_authored_bogae_json(linked.linked_execution_source,linked.target_path,ticks,'v1-core-v25',artifact));
 writeFileSync(resolve(dir,'node.json'),JSON.stringify(node,null,2));assert.equal(node.bogae_projection.status,'ready');assert.equal(node.bogae_projection.persistent_identity,true);
 const final=node.final_row??node.runtime_final_row;assert.equal(final.이동칸,expected);assert.equal(final.도착,arrived);
 for(const alias of ['ddonilang','ddonirang','또니랑']){
  const p=spawnSync(resolve(bin,alias+'.exe'),['작품자료','실행',resolve(dir,'registered.json'),'--마디수',String(ticks)],{encoding:'utf8',timeout:180000,maxBuffer:16*1024*1024});
  writeFileSync(resolve(dir,alias+'.stdout'),p.stdout??'');writeFileSync(resolve(dir,alias+'.stderr'),p.stderr??'');assert.equal(p.status,0,p.stderr);const native=JSON.parse(p.stdout);writeFileSync(resolve(dir,alias+'.json'),JSON.stringify(native,null,2));
  for(const k of ['state_hash','resources','bogae_projection'])assert.deepEqual(native[k],node[k],`${name}/${alias}/${k}`);
  rows.push({name,alias,ticks,position:expected,arrived,persistent_projection:true,state_hash:native.state_hash});
 }
}
writeFileSync(resolve(out,'FOCUSED.json'),JSON.stringify({scope:'same actual root/main → common create/register/link → Native3/Node; no test source lowering',rows},null,2));console.log('Registered root composition: 2 scenarios × Native3/Node persistent projection/state parity PASS');
