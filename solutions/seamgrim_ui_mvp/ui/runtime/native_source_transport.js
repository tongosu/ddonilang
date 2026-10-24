// Physical local-host transport only. A positive probe is not a VM permit.
let available;
export function sourceStorageErrorMessage(error){
  const message=String(error?.message??error);
  if(/^(?:Error:\s*)?E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED$/u.test(message)
    || /^(?:Error:\s*)?(?:E_REPLAY_COMPLETED_RECEIPT_REQUIRED|E_NEW_RECORD_TERMINAL_REQUIRED):[^\n]*\bCustodyConflict\b[^\n]*$/u.test(message)) {
    // Display advice only: this does not identify Pending or grant recovery.
    return message+' · 기록의 완료나 이어 실행을 확인하지 못했습니다. 기록 마치기 중 창이 닫혔다면 기존 저장본과 기록을 지우거나 덮어쓰지 마세요. 원문으로 새로 시작하려면 이 창을 닫고 ‘더 보기 → 작성 정체 자료 → 새 ID로 복제’ 후 저장하세요. 원문만 새 작품으로 복사하며, 이전 진행이나 완료 기록을 복구하는 것은 아닙니다. 자동 복구는 아직 지원하지 않습니다.';
  }
  if(message==='E_NATIVE_SOURCE_SOURCE_CHANGED') {
    return message+' · 이 ID의 저장 원문과 다릅니다. 기존 저장본은 보존했습니다. 저장되지 않은 편집을 먼저 따로 보관하세요. ‘저장 자료 다시 열기’로 마지막 저장 자료를 확인하거나, 파일을 다시 가져온 뒤 ‘새 ID로 복제’ → 저장을 선택하세요.';
  }
  if(message==='E_NATIVE_SOURCE_AUTHORING_SELECTION_REQUIRED: 원문을 열 때 받은 기준 판으로 수정 저장하세요') {
    return message+' · 기존 저장본은 보존했습니다. 저장되지 않은 편집을 먼저 따로 보관하세요. ‘저장 자료 다시 열기’로 마지막 저장 자료를 확인하거나, 파일을 다시 가져온 뒤 ‘새 ID로 복제’ → 저장을 선택하세요.';
  }
  return /^(?:(?:TypeError|Error):\s*)?(?:Failed to execute ['"]send['"] on ['"]XMLHttpRequest['"]|NetworkError\b|Failed to fetch\b|fetch failed\b|Network request failed\b)/iu.test(message)
    ?'저장 프로그램과 연결이 끊겼습니다. 저장 완료를 확인하지 못했습니다. 현재 편집을 보관하고, 시작 프로그램을 다시 실행한 뒤 저장본을 확인하세요. 성공으로 처리하거나 다른 저장본으로 대체하지 않았습니다.'
    :message;
}
export async function nativeSourceAvailable(){
  if(!available)available=fetch('/',{credentials:'same-origin',cache:'no-store'})
    .then(response=>{if(!response.ok)throw new Error('E_NATIVE_SOURCE_PROBE: 저장 제공자를 확인할 수 없습니다. 기존 저장본을 보존했습니다.');return response.headers.get('X-DDN-Native-Source')==='v1';})
    .catch(error=>{available=null;throw error;});
  return available;
}
export async function nativeSourceRequest(operation,artifactText,fields={}){
  const response=await fetch('/__ddn/vc04/source',{method:'POST',credentials:'same-origin',cache:'no-store',
    headers:{'Content-Type':'application/json'},body:JSON.stringify({...fields,operation,authored_project_json_utf8:artifactText})});
  const result=await response.json();
  if(!response.ok){
    const detail=String(result.error??'E_NATIVE_SOURCE_REQUEST_REJECTED');
    // Only this exact Native failure owner is relabelled; program text that
    // happens to mention RecoveryRequired must not acquire recovery meaning.
    throw new Error(/^E_SOURCE_WORLD_CURRENT:\s*RecoveryRequired$/u.test(detail)
      ?`복구 필요: 저장 상태를 확인하지 못했습니다. 다른 판으로 대체하거나 임의로 복구하지 않았습니다. (${detail})`:detail);
  }
  return result;
}
