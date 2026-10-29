// P1 transport successor: exact byte preservation, no parser/edition inference.
import assert from 'node:assert/strict';
import {readDdnSourceFile} from '../solutions/seamgrim_ui_mvp/ui/workbench/source_file.js';
const file=(name,bytes)=>({name,size:bytes.length,arrayBuffer:async()=>Uint8Array.from(bytes).buffer});
let cases=0;
for(const source of ['', 'ASCII\r\n', '한글 😀\n', 'A한글\uFEFF끝\n']){
  assert.deepEqual(await readDdnSourceFile(file('원문.ddn',Buffer.from(source))),{name:'원문.ddn',source});cases++;
}
for(const [name,bytes,code] of [
  ['깨짐.ddn',[0xff],'E_SOURCE_UTF8_INVALID'],
  ['BOM.ddn',[0xef,0xbb,0xbf,65],'E_SOURCE_UTF8_BOM_FORBIDDEN'],
  ['원문.json',[65],'E_SOURCE_FILE_NAME'],
  ['../원문.ddn',[65],'E_SOURCE_FILE_NAME'],
  ['C:원문.ddn',[65],'E_SOURCE_FILE_NAME'],
]){
  await assert.rejects(readDdnSourceFile(file(name,bytes)),new RegExp(code));cases++;
}
const exact=Buffer.alloc(2_000_000,32);
assert.equal((await readDdnSourceFile(file('정확.ddn',exact))).source.length,2_000_000);cases++;
await assert.rejects(readDdnSourceFile(file('넘침.ddn',Buffer.alloc(2_000_001,32))),/E_SOURCE_SNAPSHOT_LIMIT/);cases++;
await assert.rejects(readDdnSourceFile({name:'실패.ddn',arrayBuffer:async()=>{throw Error('read failed');}}),/read failed/);cases++;
console.log(JSON.stringify({status:'PASS',cases,scope:'transport only; common Rust/browser gates separate'}));
