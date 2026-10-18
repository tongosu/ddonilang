// A3-P12 PROVISIONAL: browser storage mechanics only, not an RD-669 receipt.
// Runtime bytes and semantic custody are admitted exclusively by common Rust.
const DB_NAME='ddn.runtime_project.local.v1';
const SCHEMA='ddn.runtime_project.local_record.v1';
const LOGICAL_SCHEMA='ddn.runtime_project.local_record.v2';
const SOURCE_CAPABLE_SCHEMA='ddn.runtime_project.local_record.v3';
const validHex=text=>typeof text==='string' && text.length>0 && text.length<=8192 && /^(?:[0-9a-f]{2})+$/.test(text);
function validate(record) {
  if(!record || ![SCHEMA,LOGICAL_SCHEMA,SOURCE_CAPABLE_SCHEMA].includes(record.schema) || typeof record.project_id!=='string' || !record.project_id
    || !validHex(record.revision_hex) || !(record.parent_hex==='' || validHex(record.parent_hex))
    || typeof record.artifact_text!=='string' || !record.artifact_text
    || new TextEncoder().encode(record.artifact_text).length>64*1024*1024) {
    throw new Error('E_RUNTIME_STORAGE_RECORD: 실행 저장 자료가 올바르지 않습니다.');
  }
  if(record.schema===LOGICAL_SCHEMA && (!Array.isArray(record.checkpoint_bytes)
    || !record.checkpoint_bytes.length || record.checkpoint_bytes.length>3_030_005
    || record.checkpoint_bytes.some(byte=>!Number.isInteger(byte)||byte<0||byte>255)))
    throw new Error('E_RUNTIME_STORAGE_RECORD: 논리 입력 checkpoint 바이트가 올바르지 않습니다.');
  if(record.schema===SCHEMA && record.checkpoint_bytes!==undefined)
    throw new Error('E_RUNTIME_STORAGE_RECORD: 기존 저장 형식에 논리 입력을 숨길 수 없습니다.');
  if(record.schema===SOURCE_CAPABLE_SCHEMA) {
    const keys=['schema','project_id','parent_hex','revision_hex','payload_kind','artifact_text','checkpoint_bytes'];
    if(Object.keys(record).length!==keys.length || keys.some(key=>!Object.hasOwn(record,key))
      || !['SOURCE_ONLY','STATEFUL_FRAME2'].includes(record.payload_kind)
      || (record.payload_kind==='SOURCE_ONLY' && record.checkpoint_bytes!==null)
      || (record.payload_kind==='STATEFUL_FRAME2' && (!Array.isArray(record.checkpoint_bytes)
        || !record.checkpoint_bytes.length || record.checkpoint_bytes.length>3_030_005
        || record.checkpoint_bytes.some(byte=>!Number.isInteger(byte)||byte<0||byte>255)))) {
      throw new Error('E_RUNTIME_STORAGE_SOURCE_ROLE: 원문 전용 저장과 세계 저장의 필드·바이트 역할이 다릅니다.');
    }
  }
  return record;
}
function recordText(record) {
  try{
    // Some browser serializers stall on cycles after a large UTF-8 string.
    // Detect only the object edges JSON visits, without recursion or a depth cap.
    // Shared acyclic children remain legal; Date serializes itself, not its fields.
    const active=new WeakSet(),pending=[{value:record,exit:false}];
    while(pending.length){
      const {value,exit}=pending.pop();
      if(!value||typeof value!=='object'||value instanceof Date)continue;
      if(exit){active.delete(value);continue;}
      if(active.has(value))throw new Error('cyclic storage comparison');
      active.add(value);pending.push({value,exit:true});
      const keys=Object.keys(value);
      for(let index=keys.length-1;index>=0;index--){
        const key=keys[index];
        if(Array.isArray(value)&&(!/^(0|[1-9][0-9]*)$/.test(key)||Number(key)>=value.length))continue;
        pending.push({value:value[key],exit:false});
      }
    }
    return JSON.stringify(record);
  }
  catch{throw new Error('E_RUNTIME_STORAGE_RECORD_ENCODING: 실행 저장 자료를 비교할 수 없습니다.');}
}
function openDatabase() {
  return new Promise((resolve,reject)=>{
    const request=indexedDB.open(DB_NAME,1);
    request.onupgradeneeded=()=>{
      request.result.createObjectStore('heads',{keyPath:'project_id'});
      request.result.createObjectStore('revisions',{keyPath:['project_id','revision_hex']});
    };
    request.onsuccess=()=>resolve(request.result);
    request.onerror=()=>reject(request.error);
    request.onblocked=()=>reject(new Error('E_RUNTIME_STORAGE_BLOCKED: 다른 창이 실행 저장소를 사용 중입니다.'));
  });
}
export function newRuntimeRevisionHex() {
  // Revision locator issued by the storage owner, never source/Definition/PESI.
  return Array.from(crypto.getRandomValues(new Uint8Array(16)),byte=>byte.toString(16).padStart(2,'0')).join('');
}
export async function loadRuntimeProject(projectId,{purpose='runtime'}={}) {
  if(!['runtime','source_readback'].includes(purpose))throw new Error('E_RUNTIME_STORAGE_PURPOSE: 저장 읽기 목적이 올바르지 않습니다.');
  const db=await openDatabase();
  try {return await new Promise((resolve,reject)=>{
    const tx=db.transaction(['heads','revisions'],'readonly');
    let record=null,failure=null;
    const head=tx.objectStore('heads').get(projectId);
    head.onsuccess=()=>{
      if(!head.result)return;
      let expected;
      try{
        record=validate(head.result);
        // Existing VM callers use the default. Never reinterpret a source-only
        // artifact as a saved runtime or synthesize first-use permission.
        if(record.schema===SOURCE_CAPABLE_SCHEMA && record.payload_kind==='SOURCE_ONLY' && purpose!=='source_readback')
          throw new Error('E_RUNTIME_STORAGE_SOURCE_ONLY: 실행 전 원문 저장본입니다. 실행 상태 복원으로 열 수 없습니다.');
        expected=recordText(record);
      }catch(error){failure=error;tx.abort();return;}
      const revision=tx.objectStore('revisions').get([projectId,record.revision_hex]);
      revision.onsuccess=()=>{
        try{if(recordText(revision.result)!==expected)throw new Error('E_RUNTIME_STORAGE_CHAIN: 저장 이력과 현재 자료가 다릅니다.');}
        catch(error){failure=error;tx.abort();}
      };
    };
    tx.oncomplete=()=>resolve(record);
    // A request error bubbles before tx.error is populated. Retain its actual
    // diagnostic and settle only once the transaction has finished aborting.
    tx.onerror=event=>{failure??=event.target?.error??tx.error;};
    tx.onabort=()=>reject(failure??tx.error??new Error('E_RUNTIME_STORAGE_READ_ABORTED'));
  });}finally{db.close();}
}
export async function commitRuntimeProject(record,{expectedParentRecord=undefined}={}) {
  // Capture the caller's bytes before the first await. The admitted record,
  // CAS parent and reread target must not change while IndexedDB is opening.
  record=structuredClone(validate(record));
  // Comparison must be possible before any persistent row is changed.
  const expected=recordText(record);
  const pinnedParent=expectedParentRecord===undefined?undefined:recordText(expectedParentRecord===null?null:structuredClone(validate(expectedParentRecord)));
  const db=await openDatabase();
  try {await new Promise((resolve,reject)=>{
    let tx;
    try{
      tx=db.transaction(['heads','revisions'],'readwrite',{durability:'strict'});
      if(tx.durability!=='strict') {tx.abort();throw new Error('E_RUNTIME_STORAGE_DURABILITY_UNSUPPORTED: 이 브라우저는 엄격한 실행 저장을 지원하지 않습니다.');}
    }catch(error){reject(error);return;}
    let failure=null;
    tx.oncomplete=resolve;
    tx.onerror=event=>{failure??=event.target?.error??tx.error;};
    tx.onabort=()=>reject(failure??tx.error??new Error('E_RUNTIME_STORAGE_WRITE_ABORTED'));
    const head=tx.objectStore('heads').get(record.project_id);
    head.onsuccess=()=>{
      let previous=null;
      try{if(head.result){previous=validate(head.result);recordText(previous);}}
      catch(error){failure=error;tx.abort();return;}
      if((previous?.revision_hex??'')!==record.parent_hex) {
        failure=new Error('E_RUNTIME_STORAGE_STALE: 다른 창에서 저장 이력이 바뀌었습니다. 다시 여세요.');tx.abort();return;
      }
      if(pinnedParent!==undefined && recordText(previous)!==pinnedParent) {
        failure=new Error('E_RUNTIME_STORAGE_STALE: 저장 부모의 원문 바이트가 바뀌었습니다. 다시 여세요.');tx.abort();return;
      }
      if(record.parent_hex===record.revision_hex) {
        failure=new Error('E_RUNTIME_STORAGE_REVISION: 새 저장에는 새 이력 정체가 필요합니다.');tx.abort();return;
      }
      const write=()=>{
        // add never overwrites a previous revision; head + immutable record are atomic.
        try{tx.objectStore('revisions').add(record);tx.objectStore('heads').put(record);}
        catch(error){failure=error;tx.abort();}
      };
      if(!previous){write();return;}
      // CAS alone is insufficient: an inconsistent head/revision is a failed
      // read, not permission to conceal the corruption with a new head.
      // Check inside this same write transaction; no read/write race window.
      const parent=tx.objectStore('revisions').get([record.project_id,previous.revision_hex]);
      parent.onsuccess=()=>{
        try{if(recordText(parent.result)!==recordText(previous))throw new Error('E_RUNTIME_STORAGE_CHAIN: 저장 이력과 현재 자료가 다릅니다.');}
        catch(error){failure=error;tx.abort();return;}
        write();
      };
    };
  });}finally{db.close();}
  const reread=await loadRuntimeProject(record.project_id,{purpose:record.schema===SOURCE_CAPABLE_SCHEMA&&record.payload_kind==='SOURCE_ONLY'?'source_readback':'runtime'});
  if(recordText(reread)!==expected)throw new Error('E_RUNTIME_STORAGE_REREAD: 저장 후 다시 읽은 자료가 다릅니다. 재열기 전 확인이 필요합니다.');
  return reread;
}
export function runtimeStorageRecord(projectId,parentHex,revisionHex,artifactText,checkpointBytes=null,payloadKind=null) {
  // Explicit v3 only. Old records keep their exact old roles; a successful
  // adapter write is never Rust source admission or first-world authority.
  if(payloadKind!==null)return validate({schema:SOURCE_CAPABLE_SCHEMA,project_id:projectId,parent_hex:parentHex,revision_hex:revisionHex,
    payload_kind:payloadKind,artifact_text:artifactText,checkpoint_bytes:checkpointBytes===null?null:Array.from(checkpointBytes)});
  return validate({schema:checkpointBytes===null?SCHEMA:LOGICAL_SCHEMA,project_id:projectId,parent_hex:parentHex,revision_hex:revisionHex,artifact_text:artifactText,
    ...(checkpointBytes===null?{}:{checkpoint_bytes:Array.from(checkpointBytes)})});
}
