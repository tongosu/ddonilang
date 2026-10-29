// Public product APIs only. This consumer does not parse/lower/evaluate DDN.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash,webcrypto} from 'node:crypto';
const [wasmModule,bundlePath,output]=process.argv.slice(2);
const sha=raw=>createHash('sha256').update(raw).digest('hex');
const root=path.dirname(path.resolve(bundlePath));
function readFile(descriptor){
  const file=path.resolve(root,descriptor.path),relative=path.relative(root,file);
  if(path.isAbsolute(descriptor.path)||relative==='..'||relative.startsWith('..'+path.sep)||path.isAbsolute(relative))throw Error('검사 묶음 밖 경로');
  const raw=fs.readFileSync(file);if(sha(raw)!==descriptor.sha256)throw Error('고정 입력·기대 해시 불일치');return raw;
}
globalThis.crypto??=webcrypto;
const api=await import(pathToFileURL(path.resolve(wasmModule)));
const wasmBytes=fs.readFileSync(path.join(path.dirname(wasmModule),'ddonirang_tool_bg.wasm'));
await api.default({module_or_path:wasmBytes});
const bundle=JSON.parse(fs.readFileSync(bundlePath)),checks=[];
if(bundle.version!==1)throw Error('미지원 검사 묶음 판본');
for(const test of bundle.cases){
  const sourceRaw=readFile(test.source);
  let source=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(sourceRaw);
  const expected=JSON.parse(readFile(test.expected));
  if(sha(Buffer.from(source))!==sha(sourceRaw))throw Error('원문 bytes가 달라졌습니다');
  let actual=null,error=null;
  try{
    if(test.kind==='saved-proof'){
      const opened=JSON.parse(source);
      if(Object.keys(opened).sort().join(',')!=='authored_project_json_utf8,kind'||opened.kind!=='native_source_saved')throw Error('원문만 재열린 Native 결과가 아닙니다');
      const project=JSON.parse(opened.authored_project_json_utf8);
      if(project.source_profile!=='v1-core-v25')throw Error('지원 원문 판본이 다릅니다');
      const files=project.source_snapshot.files.filter(file=>file.path===test.product_path);
      if(files.length!==1||typeof files[0].text!=='string'||sha(Buffer.from(files[0].text))!==test.expected_source_sha256)throw Error('재열린 정확 원문·경로가 다릅니다');
      source=files[0].text;
    }
    if(test.kind==='project'){
      const prepared=JSON.parse(api.wasm_project_prepare_link_json(api.authored_project_operation('source_project',source),'',true,true));
      actual=JSON.parse(api.wasm_run_summary_with_authored_bogae_json(prepared.linked_execution_source,prepared.target_path,test.ticks,prepared.source_profile_identity,source,undefined,undefined));
    }else if(test.kind==='source'){
      actual=JSON.parse(api.wasm_run_summary_with_source_profile_json(source,test.product_path,test.ticks,'v1-core-v25'));
    }else if(test.kind==='proof'||test.kind==='saved-proof'){
      actual=JSON.parse(api.wasm_verify_assumption_source_json(source,test.product_path));
    }else throw Error('미지원 검사 종류');
  }catch(cause){error=String(cause);}
  const stable=value=>{
    if(Array.isArray(value))return value.map(stable);
    if(value&&typeof value==='object')return Object.fromEntries(Object.keys(value).sort().map(key=>[key,stable(value[key])]));
    return value;
  };
  const equal=(a,b)=>JSON.stringify(stable(a))===JSON.stringify(stable(b));
  const proof=test.kind==='proof'||test.kind==='saved-proof';
  const proofOracle=!proof||(expected.assumption_dependent===true&&expected.unconditional_proof_issued===false&&Number.isSafeInteger(expected.conditional_definition_count)&&expected.conditional_definition_count>0);
  const good=test.reject_contains?actual===null&&error?.includes(test.reject_contains):error===null&&proofOracle&&(test.kind==='project'?equal(actual,expected):proof?Object.entries(expected).every(([k,v])=>equal(actual[k],v)):equal(actual.output_display?.texts,expected));
  checks.push({id:test.id,row:test.row,path:'Node WASM',gate:proof?'WASM fresh check':'WASM',status:good?'PASS':'FAIL',source_sha256:sha(Buffer.from(source)),native_capture_sha256:test.kind==='saved-proof'?sha(sourceRaw):undefined,actual,error,expected});
}
fs.writeFileSync(output,JSON.stringify({wasm_sha256:sha(wasmBytes),wasm_glue_sha256:sha(fs.readFileSync(wasmModule)),checks},null,2)+'\n');
console.log(JSON.stringify({checks:checks.length,failed:checks.filter(c=>c.status==='FAIL').length}));
