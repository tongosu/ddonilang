// Physical local-host transport only. A positive probe is not a VM permit.
let available;
export function sourceStorageErrorMessage(error){
  const diagnostic=error?.diagnostic;
  if(diagnostic?.source_span && (diagnostic.code==='E_J1_ROLE_AMBIGUOUS'
    || diagnostic.code==='E_MODULE_MISSING'
    || (diagnostic.code==='E_PARSE' || diagnostic.code==='E_BYTES_ARGUMENT_BINDING')
      && diagnostic.phase==='lang_parse')) {
    const {start,end}=diagnostic.source_span;
    return `${diagnostic.message} · ${diagnostic.source_path} · UTF-8 ${start}–${end}`;
  }
  const message=String(error?.message??error);
  if(/^(?:Error:\s*)?E_SOURCE_READ_CONFIRM: phase=cold-confirm cause=RecoveryRequired$/u.test(message)) {
    // Advice for this exact protected Native refusal, never a recovery permit
    // or a claim that the retained attempt is Aborted or safe to execute again.
    return message+' · 첫 실행 시도 뒤 저장 보호 상태를 확인하지 못했습니다. 보관한 원문·등록 자료와 실패 기록은 그대로 유지합니다. 현재 편집은 창을 닫기 전에 ‘DDN 원문 저장’으로 따로 보관하세요. 이는 현재 파일 보관이며 작품 전체·Native 복구·세계 이어 실행이 아닙니다. ID·JSON·저장소를 고치거나 지우지 마세요. 같은 저장본의 재열기·새 실행은 복구 확인 전까지 지원하지 않습니다.';
  }
  if(/^(?:Error:\s*)?E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED$/u.test(message)
    || /^(?:Error:\s*)?(?:E_REPLAY_COMPLETED_RECEIPT_REQUIRED|E_NEW_RECORD_TERMINAL_REQUIRED):[^\n]*\bCustodyConflict\b[^\n]*$/u.test(message)) {
    // Display advice only: this does not identify Pending or grant recovery.
    return message+' · 기록의 완료나 이어 실행을 확인하지 못했습니다. 기록 마치기 중 창이 닫혔다면 기존 저장본과 기록을 지우거나 덮어쓰지 마세요. 원문으로 새로 시작하려면 이 창을 닫고 ‘더 보기 → 작성 정체 자료 → 새 ID로 복제’ 후 저장하세요. 원문만 새 작품으로 복사하며, 이전 진행이나 완료 기록을 복구하는 것은 아닙니다. 자동 복구는 아직 지원하지 않습니다.';
  }
  if(message.replace(/^Error:\s*/u,'')==='E_CODE_SOURCE_REGISTRATION_REQUIRED: 별도 등록이 필요한 원문은 기존 등록 저장 경로를 사용하세요') {
    // Display this exact unsupported custody boundary, never change the raw
    // failure or claim a Root/registration route exists for standalone source.
    return 'E_CODE_SOURCE_REGISTRATION_REQUIRED: 이 단독 원문의 Native 보관은 아직 지원하지 않습니다. 작품 이름 메뉴의 ‘DDN 원문 저장’으로 현재 파일을 따로 보관할 수 있습니다. 현재 파일만 보관하며, 등록 자료나 작품 전체·세계 진행을 저장하는 기능은 아닙니다. 원문을 감싸거나 ID를 고칠 필요는 없습니다.';
  }
  if(message==='E_PROJECT_RUNTIME_ROOT_REQUIRED: 실행 저장에는 명시 얽음 뿌리가 필요합니다') {
    // Advice for this exact Native rejection only; it confers no source-save,
    // root/Start or world authority and never selects a fallback store.
    return message+' · 명시 얽음 없는 단독 원문의 Native 보관은 이 경로에서 아직 지원하지 않습니다. 작품 이름 메뉴의 ‘DDN 원문 저장’으로 현재 파일을 따로 보관할 수 있습니다. 현재 파일만 보관하며, 작품 전체나 세계 진행을 저장하는 기능은 아닙니다.';
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
    const error=new Error(/^E_SOURCE_WORLD_CURRENT:\s*RecoveryRequired$/u.test(detail)
      ?`복구 필요: 저장 상태를 확인하지 못했습니다. 다른 판으로 대체하거나 임의로 복구하지 않았습니다. (${detail})`:detail);
    if((operation==='run_public_callable_edition'&&result.diagnostic?.phase==='runtime_execution')
      ||(operation==='prepare_public_callable_original'&&result.diagnostic?.phase==='lang_parse')
      ||(operation==='inspect_public_callable_original'
        &&(['project_root','lang_lex','lang_parse'].includes(result.diagnostic?.phase)
          ||result.diagnostic?.code==='E_DDL_PUBLIC_BODY_SCOPE'&&result.diagnostic?.phase==='source_prepare'))
      ||(operation==='review_public_callable_edition'
        &&['lang_parse','project_prepare'].includes(result.diagnostic?.phase)))
      error.diagnostic=result.diagnostic;
    throw error;
  }
  return result;
}
