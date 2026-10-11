// A3-P12 PROVISIONAL: browser storage mechanics only, not an RD-669 receipt.
// Runtime bytes and semantic custody are admitted exclusively by common Rust.
const DB_NAME='ddn.runtime_project.local.v1';
const SCHEMA='ddn.runtime_project.local_record.v1';
const validHex=text=>typeof text==='string' && text.length>0 && text.length<=8192 && /^(?:[0-9a-f]{2})+$/.test(text);
function validate(record) {
  if(!record || record.schema!==SCHEMA || typeof record.project_id!=='string' || !record.project_id
    || !validHex(record.revision_hex) || !(record.parent_hex==='' || validHex(record.parent_hex))
    || typeof record.artifact_text!=='string' || !record.artifact_text
    || new TextEncoder().encode(record.artifact_text).length>64*1024*1024) {
    throw new Error('E_RUNTIME_STORAGE_RECORD: 실행 저장 자료가 올바르지 않습니다.');
  }
  return record;
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
export async function loadRuntimeProject(projectId) {
  const db=await openDatabase();
  try {return await new Promise((resolve,reject)=>{
    const tx=db.transaction(['heads','revisions'],'readonly');
    let record=null,failure=null;
    const head=tx.objectStore('heads').get(projectId);
    head.onsuccess=()=>{
      if(!head.result)return;
      try{record=validate(head.result);}catch(error){failure=error;tx.abort();return;}
      const revision=tx.objectStore('revisions').get([projectId,record.revision_hex]);
      revision.onsuccess=()=>{if(JSON.stringify(revision.result)!==JSON.stringify(record)){
        failure=new Error('E_RUNTIME_STORAGE_CHAIN: 저장 이력과 현재 자료가 다릅니다.');tx.abort();
      }};
    };
    tx.oncomplete=()=>resolve(record);
    tx.onerror=()=>reject(failure??tx.error);
    tx.onabort=()=>reject(failure??tx.error??new Error('E_RUNTIME_STORAGE_READ_ABORTED'));
  });}finally{db.close();}
}
export async function commitRuntimeProject(record) {
  validate(record);
  const db=await openDatabase();
  try {await new Promise((resolve,reject)=>{
    let tx;
    try{
      tx=db.transaction(['heads','revisions'],'readwrite',{durability:'strict'});
      if(tx.durability!=='strict') {tx.abort();throw new Error('E_RUNTIME_STORAGE_DURABILITY_UNSUPPORTED: 이 브라우저는 엄격한 실행 저장을 지원하지 않습니다.');}
    }catch(error){reject(error);return;}
    let failure=null;
    tx.oncomplete=resolve;
    tx.onerror=()=>reject(failure??tx.error);
    tx.onabort=()=>reject(failure??tx.error??new Error('E_RUNTIME_STORAGE_WRITE_ABORTED'));
    const head=tx.objectStore('heads').get(record.project_id);
    head.onsuccess=()=>{
      if((head.result?.revision_hex??'')!==record.parent_hex) {
        failure=new Error('E_RUNTIME_STORAGE_STALE: 다른 창에서 저장 이력이 바뀌었습니다. 다시 여세요.');tx.abort();return;
      }
      if(record.parent_hex===record.revision_hex) {
        failure=new Error('E_RUNTIME_STORAGE_REVISION: 새 저장에는 새 이력 정체가 필요합니다.');tx.abort();return;
      }
      // add never overwrites a previous revision; head + immutable record are atomic.
      tx.objectStore('revisions').add(record);
      tx.objectStore('heads').put(record);
    };
  });}finally{db.close();}
  const reread=await loadRuntimeProject(record.project_id);
  if(JSON.stringify(reread)!==JSON.stringify(record))throw new Error('E_RUNTIME_STORAGE_REREAD: 저장 후 다시 읽은 자료가 다릅니다. 재열기 전 확인이 필요합니다.');
  return reread;
}
export function runtimeStorageRecord(projectId,parentHex,revisionHex,artifactText) {
  return validate({schema:SCHEMA,project_id:projectId,parent_hex:parentHex,revision_hex:revisionHex,artifact_text:artifactText});
}
