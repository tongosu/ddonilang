// Persistent IDs are supplied only by the common Rust codec after an explicit
// user operation. Local storage keys and download names are locators, not IDs.
import {createAuthoredSourceStorage} from './persistence/authored_source_save.js';
import {nativeSourceAvailable,nativeSourceRequest,sourceStorageErrorMessage} from '../runtime/native_source_transport.js';
export function installAuthoredProjectPanel({root, bridge, host, capture, apply, saveRuntime, reopenRuntime, exportRuntime,saveProject,onSourceReopened=()=>{},ownsLogicalSource=()=>false}) {
  const status=root.querySelector('[data-authored-status]');
  const identity=root.querySelector('[data-authored-identity]');
  const contents=root.querySelector('[data-authored-contents]');
  const fileInput=root.querySelector('input[type=file]');
  const key='ddn.authored_project.last.v1';
  const operate=(operation,text)=>bridge.authoredProjectOperation({operation,text});
  const sourceStorage=createAuthoredSourceStorage(bridge);
  let artifactText='', baseline='', busy=false, sourceContextActive=false;
  // Keep the physical selection returned to this open work. The shared last
  // locator is a reopening convenience, not another tab's authoring baseline.
  let retainedSourceLocator='',nominalSourceArtifactText='';
  function publishSourceMaterial(result){
    nominalSourceArtifactText='';
    if(result?.status==='accepted'){try{if(JSON.parse(result.sourceArtifactText).schema==='ddn.nominal_source_only_artifact.v1')nominalSourceArtifactText=result.sourceArtifactText;}catch{}}
  }
  const pendingInitialSources=new Map();
  function sourceLocator(projectId='') {
    for (const text of [retainedSourceLocator,localStorage.getItem(key)??'']) {
      try {
        const locator=JSON.parse(text);
        const id=locator.native_source_project_id??locator.source_project_id
          ??locator.authored_identity?.logical_dependency_id;
        if (!projectId||id===projectId) return text;
      } catch {}
    }
    return '';
  }
  function rememberSourceLocator(text) {
    retainedSourceLocator=text;
    localStorage.setItem(key,text);
  }

  function show(text, message) {
    const artifact=JSON.parse(text);
    artifactText=text;
    identity.textContent=artifact.authored_identity.logical_dependency_id;
    contents.value=text;
    status.textContent=message;
    const targets=root.querySelector('[data-authored-rename-target]');
    if(targets) {
      targets.replaceChildren();
      for(const module of artifact.authored_identity.modules ?? []) {
        for(const [target_kind, values] of [['composition',module.compositions ?? []],['binding',module.bindings ?? []]]) {
          for(const target of values) {
            const option=document.createElement('option');
            option.value=JSON.stringify({owner_module_id:module.id,target_kind,target_id:target.id});
            option.textContent=`${module.name} · ${target.definition} (${target_kind==='composition'?'짜임':'정의'})`;
            targets.append(option);
          }
        }
      }
    }
  }
  function unchanged(token) {
    if(JSON.stringify(capture())!==token) throw new Error('원문이나 선택 작품이 바뀌었습니다. 다시 시도하세요.');
  }
  async function perform(action,{propagate=false}={}) {
    if(busy) {if(propagate)throw new Error('E_SOURCE_SAVE_BUSY: 다른 작성 작업을 마친 뒤 저장하세요.');return;}
    const errorContext=JSON.stringify(capture());
    busy=true;
    // Disabling the focused save button can emit focusout(null), which the
    // input host correctly treats as loss. Stay inside this admitted drawer.
    if(root.contains(root.ownerDocument.activeElement)) {
      if(!root.hasAttribute('tabindex'))root.tabIndex=-1;
      root.focus({preventScroll:true});
    }
    root.querySelectorAll('button').forEach(button=>button.disabled=true);
    try { return await action(); }
    catch(error) {
      // A failed request still belongs to its original source and project.
      if(JSON.stringify(capture())===errorContext) status.textContent=`저장 자료 오류: ${sourceStorageErrorMessage(error)}`;
      if(propagate)throw error;
    }
    finally {busy=false;root.querySelectorAll('button').forEach(button=>button.disabled=false);fileInput.value='';}
  }
  async function openText(text, message, token,projectContext) {
    const admitted=await operate('open',text);
    unchanged(token);
    const expected=await apply(JSON.parse(admitted),projectContext); // Admission first; no execution on open.
    const expectedToken=JSON.stringify(expected);
    unchanged(expectedToken);
    baseline=expectedToken;
    sourceContextActive=false;nominalSourceArtifactText='';
    show(admitted,message);
  }
  root.querySelector('[data-authored-create]').onclick=()=>perform(async()=>{
    const captured=capture(), token=JSON.stringify(captured);
    if(captured.source_snapshot.files.length!==1 && !captured.source_snapshot.root_path) throw new Error('여러 파일은 실행할 얽음 뿌리를 명시해야 합니다. 파일 이름으로 찾지 않습니다.');
    const text=await operate('create',JSON.stringify({
      source_profile:captured.source_profile,source_snapshot:captured.source_snapshot,module_name:captured.source_snapshot.root_path?'':'앱',
    }));
    unchanged(token);
    baseline=token;
    sourceContextActive=false;
    show(text,'명시 작품 정체를 만들었습니다. 아직 이 기기에 저장하지 않았습니다.');
  });
  async function sourceSaveCandidate() {
    const captured=capture(),token=JSON.stringify(captured),original=artifactText;
    const assertCurrent=()=>{unchanged(token);if(artifactText!==original)throw new Error('E_PROJECT_EDIT_SOURCE_CONFLICT: 작성 기준이 바뀌었습니다.');};
    let snapshot=captured.source_snapshot;
    const prior=JSON.parse(original).source_snapshot;
    const oldLock=prior.files.find(file=>file.path==='ddn.lock');
    const currentLock=snapshot.files.find(file=>file.path==='ddn.lock');
    // Only a generated lock left unchanged by the user may be refreshed.
    // Explicit lifecycle changes already carry their own checked new lock.
    if(oldLock&&currentLock?.text===oldLock.text&&prior.root_path===snapshot.root_path
        &&prior.entry_path===snapshot.entry_path&&prior.files.length===snapshot.files.length
        &&prior.files.every(file=>snapshot.files.some(next=>next.path===file.path))
        &&prior.files.filter(file=>file.path.startsWith('vendor/')).every(file=>
          snapshot.files.find(next=>next.path===file.path).text===file.text)
        &&prior.files.some(file=>snapshot.files.find(next=>next.path===file.path).text!==file.text)) {
      const projected=JSON.parse(await operate('source_project',original));assertCurrent();
      const previousLock=JSON.parse(await bridge.projectGajiLock(projected));assertCurrent();
      const texts=new Map(snapshot.files.map(file=>[file.path,file.text]));
      const current={...projected,files:projected.files.filter(file=>file.path!=='ddn.lock')
        .map(file=>({...file,utf8_text:texts.get(file.path)}))};
      const lockText=(await bridge.projectGajiLock(current)).trimEnd()+'\n';assertCurrent();
      const nextLock=JSON.parse(lockText);
      if(JSON.stringify(nextLock.packages)!==JSON.stringify(previousLock.packages)
          ||nextLock.root_path!==previousLock.root_path||nextLock.entry_path!==previousLock.entry_path)
        throw new Error('E_LOCK_HASH_MISMATCH: 의존 판본 변경은 가지 관리의 명시 변경을 사용하세요.');
      snapshot={...snapshot,files:snapshot.files.map(file=>file.path==='ddn.lock'?{...file,text:lockText}:file)};
    }
    const text=await operate('update',JSON.stringify({original_json:original,source_snapshot:snapshot}));assertCurrent();
    if(currentLock&&currentLock.text!==oldLock?.text){
      const projected=JSON.parse(await operate('source_project',text));assertCurrent();
      await bridge.projectGajiLock(projected);assertCurrent();
    }
    return {text,token,original};
  }
  async function saveSourceAction(prepared=null) {
    if(!artifactText) throw new Error('먼저 정체를 만들거나 저장 자료를 여세요.');
    const captured=capture(), token=JSON.stringify(captured), prior=JSON.parse(baseline);
    if(captured.selected_project!==prior.selected_project || captured.source_profile!==prior.source_profile) throw new Error('다른 작품이나 프로필로 바뀌었습니다. 저장 자료를 다시 여세요.');
    const candidate=prepared??await sourceSaveCandidate();
    unchanged(candidate.token);
    if(candidate.original!==artifactText)throw new Error('E_PROJECT_EDIT_SOURCE_CONFLICT: 작성 기준이 바뀌었습니다.');
    const text=candidate.text;
    unchanged(token);
    let supplied=null;
    const sourceSaved=await sourceStorage.save(text,()=>unchanged(token),{previousArtifactText:artifactText,retainedLocator:sourceLocator(JSON.parse(artifactText).authored_identity.logical_dependency_id),receiveSourceArtifact:value=>{supplied=value;}});
    unchanged(token);
    rememberSourceLocator(sourceSaved?.locator??text);
    if(sourceSaved&&(sourceSaved.requires_new_run||text!==artifactText)){
      // Only an acknowledged source CAS invalidates the current VM. A failed
      // save never applies a new project or clears the running world's view.
      const expected=await apply(JSON.parse(text),{sourceOnlyFreshExecution:Boolean(sourceSaved?.locator&&JSON.parse(sourceSaved.locator).native_source_project_id)});
      baseline=JSON.stringify(expected);unchanged(baseline);
    }else baseline=token;
    sourceContextActive=Boolean(sourceSaved);
    publishSourceMaterial(supplied);
    show(text,sourceSaved?.message??'이 기기에 작성 자료를 저장했습니다. 다시 열어도 같은 ID를 사용합니다.');
    return sourceSaved;
  }
  root.querySelector('[data-authored-save]').onclick=()=>perform(saveSourceAction);
  for(const [selector,operation] of [['[data-authored-runtime-save]',saveRuntime],['[data-authored-runtime-reopen]',reopenRuntime],['[data-authored-runtime-export]',exportRuntime]]) {
    root.querySelector(selector)?.addEventListener('click',async()=>{
      // Native source and world use the same top-level storage owner. Do not
      // send the drawer's buttons to a separate IndexedDB runtime history.
      try {
        if(selector==='[data-authored-runtime-save]'&&await nativeSourceAvailable()) {
          const result=await saveProject?.();
          status.textContent=result?.message??'상단 저장과 같은 경로로 처리했습니다.';
          return;
        }
      }catch(error){status.textContent=`저장 자료 오류: ${sourceStorageErrorMessage(error)}`;return;}
      return perform(async()=>{
      if(!artifactText || JSON.stringify(capture())!==baseline)throw new Error('먼저 현재 작품 원문을 갱신 저장하세요.');
      if(!operation)throw new Error('E_PROJECT_RUNTIME_UI_UNSUPPORTED');
      status.textContent=await operation(artifactText);
      });
    });
  }
  root.querySelector('[data-authored-register]').onclick=()=>perform(async()=>{
    if(!artifactText) throw new Error('먼저 작품 정체를 만들거나 저장 자료를 여세요.');
    const captured=capture(), token=JSON.stringify(captured), prior=JSON.parse(baseline);
    if(captured.selected_project!==prior.selected_project || captured.source_profile!==prior.source_profile) throw new Error('다른 작품으로 바뀌었습니다. 저장 자료를 다시 여세요.');
    const module_name=root.querySelector('[data-authored-module]').value.trim();
    const source_path=root.querySelector('[data-authored-source]').value.trim() || captured.source_snapshot.entry_path;
    const composition_definition=root.querySelector('[data-authored-composition]').value.trim();
    if(!module_name || !composition_definition) throw new Error('소유 모듈과 보개짜임 정의를 명시하세요.');
    const updated=await operate('update',JSON.stringify({original_json:artifactText,source_snapshot:captured.source_snapshot}));
    unchanged(token);
    const text=await operate('register_bogae',JSON.stringify({original_json:updated,module_name,
      source_path,composition_definition,make_entry:true}));
    unchanged(token);
    const expected=await apply(JSON.parse(text));
    const expectedToken=JSON.stringify(expected);
    unchanged(expectedToken);
    baseline=expectedToken;
    show(text,'보개 시작을 명시 등록했습니다. 실행은 직접 시작하세요. 이 기기 저장은 별도입니다.');
  });
  async function reopenSourceAction() {
    const token=JSON.stringify(capture()), text=sourceLocator(artifactText?JSON.parse(artifactText).authored_identity.logical_dependency_id:'');
    if(!text) throw new Error('이 기기에 저장한 작성 자료가 없습니다.');
    const locator=JSON.parse(text);
    if((Object.keys(locator).length===1 && typeof locator.source_project_id==='string')||typeof locator.native_source_project_id==='string') {
      let supplied=null;
      const admitted=await sourceStorage.reopen(locator.source_project_id??locator.native_source_project_id,()=>unchanged(token),locator,{receiveSourceArtifact:value=>{supplied=value;}});
      const message=locator.first_failure_recovery===true ? '보존 원문·등록자료를 다시 읽었습니다. 이전 실행 시도와 실패 기록은 유지했습니다. ▶ 실행을 누르면 같은 정체로 새 시도를 시작합니다. 아직 실행하지 않았습니다.' : locator.model_source_selection ? '보관한 모형 원문을 다시 읽었습니다. 결과·세계 진행은 복원하지 않았습니다. 검사는 직접 시작하세요.' : '저장 원문과 등록 자료를 다시 읽었습니다. 실행하지 않았습니다.';
      await openText(admitted,message,token,{sourceOnlyFreshExecution:typeof locator.native_source_project_id==='string'});publishSourceMaterial(supplied);sourceContextActive=true;
      rememberSourceLocator(JSON.stringify(locator));
      onSourceReopened();
      return {message};
    }
    await openText(text,'저장 자료를 다시 열었습니다. 실행은 직접 시작하세요.',token);
    return null;
  }
  root.querySelector('[data-authored-reopen]').onclick=()=>perform(reopenSourceAction);
  root.querySelector('[data-authored-compare-saved]')?.addEventListener('click',()=>perform(async()=>{
    if(!artifactText)throw new Error('먼저 이 작품을 Native에 저장하고 실행하세요.');
    if(ownsLogicalSource())throw new Error('논리 입력의 완료 기록은 입력 기록에서 재현 확인하세요.');
    const token=JSON.stringify(capture()),source=artifactText;
    if(token!==baseline)throw new Error('원문이 바뀌었습니다. 저장된 원문을 연 뒤 결과를 대조하세요.');
    if(!await nativeSourceAvailable())throw new Error('이 대조는 Native 실행 프로그램의 실제 저장본이 필요합니다.');
    unchanged(token);status.textContent='저장된 원 시작·입력·출력을 확인하는 중입니다. 아직 대조가 끝나지 않았습니다.';
    const checked=await nativeSourceRequest('general_recording_content',source);
    unchanged(token);
    if(artifactText!==source||checked.kind!=='native_general_recording_content'||checked.authored_project_json_utf8!==source)
      throw new Error('E_GENERAL_RECORDING_SOURCE_CHANGED: 대조할 저장 원문이 바뀌었습니다.');
    const replay=JSON.parse(await bridge.compareGeneralOriginalRecording({bundleHex:checked.recording_bundle_hex}));
    unchanged(token);if(artifactText!==source)throw new Error('E_GENERAL_RECORDING_SOURCE_CHANGED');
    const fields=['accepted_count_decimal','result_identity','source_pesi','status'];
    for(const value of [checked.comparison,replay]){
      if(!value||Object.keys(value).sort().join('|')!==fields.join('|')||fields.some(key=>typeof value[key]!=='string'))
        throw new Error('E_GENERAL_RECORDING_REPORT: 결과 대조 응답이 맞지 않습니다.');
    }
    if(replay.status!=='RUNNING_RESULT_COMPARED'||!/^\d{1,20}$/u.test(replay.accepted_count_decimal)||fields.some(key=>replay[key]!==checked.comparison[key]))
      throw new Error('E_GENERAL_RECORDING_RESULT_MISMATCH: 원 입력으로 대조한 결과가 다릅니다.');
    const count=BigInt(replay.accepted_count_decimal);
    const position=count===0n?'아직 실행한 마디 없음(0마디)':`마디 ${count-1n}까지(${count}마디)`;
    status.textContent=`저장한 ${position}의 세계와 출력이 일치합니다. 진행 중인 기록의 대조이며 정상 종료 영수증 확인은 아닙니다.`;
  }));
  root.querySelector('[data-authored-clone]').onclick=()=>perform(async()=>{
    if(!artifactText) throw new Error('복제할 작성 자료가 없습니다.');
    const token=JSON.stringify(capture());
    if(token!==baseline) throw new Error('원문이 바뀌었습니다. 저장된 원문을 다시 연 뒤 복제하세요.');
    const text=await operate('clone',artifactText);
    unchanged(token);
    const expected=await apply(JSON.parse(text));
    const expectedToken=JSON.stringify(expected);
    unchanged(expectedToken);
    baseline=expectedToken;
    sourceContextActive=false;
    show(text,'새 ID로 복제했습니다. 원본 자료는 바꾸지 않았습니다.');
  });
  root.querySelector('[data-authored-export]').onclick=()=>perform(async()=>{
    if(sourceContextActive)throw new Error('E_SOURCE_SAVE_EXPORT_UNSUPPORTED: 등록 자료를 버리는 작성 자료 내보내기는 하지 않습니다. 원문 저장본 파일 내보내기는 아직 연결 중입니다.');
    if(!artifactText || JSON.stringify(capture())!==baseline) throw new Error('내보낼 저장 자료가 없거나 원문이 바뀌었습니다.');
    const token=baseline, text=await operate('open',artifactText);
    unchanged(token);
    await host.downloadTextFile({text,filename:'작품-작성자료.json',mimeType:'application/json'});
    status.textContent='Rust가 검증한 작성 자료를 내보냈습니다.';
  });
  root.querySelector('[data-authored-import]').onclick=()=>fileInput.click();
  async function changeIdentityLocator(operation, fields) {
    if(!artifactText) throw new Error('먼저 저장 자료를 여세요.');
    const captured=capture(), token=JSON.stringify(captured), prior=JSON.parse(baseline);
    if(captured.selected_project!==prior.selected_project || captured.source_profile!==prior.source_profile) throw new Error('다른 작품으로 바뀌었습니다. 저장 자료를 다시 여세요.');
    let source_snapshot=captured.source_snapshot;
    if(operation==='relocate') {
      // Exact user-supplied path mapping, never infer a move from file names.
      // References in a multi-module root must already be edited by the author.
      const {old_path,new_path}=fields;
      if(!source_snapshot.files.some(file=>file.path===old_path)) throw new Error('이전 경로에 원문이 없습니다.');
      if(source_snapshot.files.some(file=>file.path===new_path)) throw new Error('새 경로에 원문이 이미 있습니다.');
      source_snapshot={...source_snapshot,
        entry_path:source_snapshot.entry_path===old_path?new_path:source_snapshot.entry_path,
        ...(source_snapshot.root_path?{root_path:source_snapshot.root_path===old_path?new_path:source_snapshot.root_path}:{}),
        files:source_snapshot.files.map(file=>file.path===old_path?{...file,path:new_path}:file)};
    }
    const text=await operate(operation,JSON.stringify({original_json:artifactText,source_snapshot,...fields}));
    unchanged(token);
    const expected=await apply(JSON.parse(text)), expectedToken=JSON.stringify(expected);
    unchanged(expectedToken);
    baseline=expectedToken;
    show(text,'명시 변경을 검증했습니다. 기존 ID는 유지했고 이 기기 저장은 별도입니다.');
  }
  root.querySelector('[data-authored-rename]')?.addEventListener('click',()=>perform(async()=>{
    const target=root.querySelector('[data-authored-rename-target]').value;
    const new_name=root.querySelector('[data-authored-new-name]').value.trim();
    if(!target || !new_name) throw new Error('바꿀 정의와 새 이름을 명시하세요. 원문의 선언과 참조도 먼저 수정하세요.');
    await changeIdentityLocator('rename',{...JSON.parse(target),new_name});
  }));
  root.querySelector('[data-authored-relocate]')?.addEventListener('click',()=>perform(async()=>{
    const old_path=root.querySelector('[data-authored-old-path]').value.trim();
    const new_path=root.querySelector('[data-authored-new-path]').value.trim();
    if(!old_path || !new_path || old_path===new_path) throw new Error('이전 경로와 새 경로를 명시하세요. 얽음의 경로 참조는 먼저 수정하세요.');
    await changeIdentityLocator('relocate',{old_path,new_path});
  }));
  const importFile=file=>perform(async()=>{
    if(!file) return;
    if(file.size>64*1024*1024) throw new Error('저장 자료가 64MiB를 넘습니다.');
    const token=JSON.stringify(capture());
    // ignoreBOM=true preserves the character: Rust rejects it, never strips it.
    const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(await file.arrayBuffer());
    await openText(text,'파일을 열었습니다. 영구 ID는 보존했고 실행하지 않았습니다.',token);
  },{propagate:true});
  fileInput.onchange=()=>importFile(fileInput.files[0]).catch(()=>{});
  return {
    importFile,
    reopenNativeLocator:(text,projectContext)=>perform(async()=>{
      const token=JSON.stringify(capture()),locator=JSON.parse(text);
      if(typeof locator.native_source_project_id!=='string'||typeof locator.authored_project_json_utf8!=='string')throw new Error('E_NATIVE_SOURCE_LOCATOR_REQUIRED');
      if(projectContext?.expectedNativeProjectId&&locator.native_source_project_id!==projectContext.expectedNativeProjectId)
        throw new Error('E_NATIVE_WORK_LOCATOR_MISMATCH: 목록과 저장 위치의 작품 ID가 다릅니다. 다른 작품으로 대체하지 않습니다.');
      let supplied=null;
      const admitted=await sourceStorage.reopen(locator.native_source_project_id,()=>unchanged(token),locator,{receiveSourceArtifact:value=>{supplied=value;}});
      const message=locator.first_failure_recovery===true ? '보존 원문·등록자료를 다시 읽었습니다. 이전 실행 시도와 실패 기록은 유지했습니다. ▶ 실행을 누르면 같은 정체로 새 시도를 시작합니다. 아직 실행하지 않았습니다.' : 'Native 저장 원문을 다시 열었습니다. 실행하지 않았습니다.';
      await openText(admitted,message,token,{...projectContext,sourceOnlyFreshExecution:true});
      publishSourceMaterial(supplied);sourceContextActive=true;rememberSourceLocator(JSON.stringify(locator));return {message};
    },{propagate:true}),
    createAndSaveSource:projectContext=>perform(async()=>{
      if(!await nativeSourceAvailable())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: Native 시작 프로그램에서 저장하세요.');
      const captured=capture(),token=JSON.stringify(captured);
      if(!captured.source_snapshot.root_path && (captured.source_profile!=='v1-core-v25'||captured.source_snapshot.files.length!==1))throw new Error('E_PROJECT_RUNTIME_ROOT_REQUIRED: 등록/여러 파일의 저장 작품은 명시 뿌리가 필요합니다.');
      let pending=pendingInitialSources.get(captured.selected_project);
      if(pending&&pending.token!==token)throw new Error('E_NATIVE_SOURCE_PENDING_CHANGED: 이전 원문 판의 저장 확인이 남아 있습니다. 현재 편집은 유지했습니다. 저장 요청 당시 원문을 확인한 뒤 같은 판을 다시 저장하세요.');
      if(!pending){
        if(pendingInitialSources.size>=8)throw new Error('E_NATIVE_SOURCE_PENDING_LIMIT: 확인되지 않은 원문 저장이 8건입니다. 기존 판의 저장 확인부터 마치세요.');
        const text=await operate('create',JSON.stringify({source_profile:captured.source_profile,source_snapshot:captured.source_snapshot,module_name:captured.source_snapshot.root_path?'':'앱'}));
        unchanged(token);pending={token,text};pendingInitialSources.set(captured.selected_project,pending);
      }
      const text=pending.text;
      let supplied=null;
      let saved;
      try {
        saved=await sourceStorage.save(text,()=>unchanged(token),{receiveSourceArtifact:value=>{supplied=value;}});
      } catch(error) {
        if(error?.sourceSaveProbeRejected&&pendingInitialSources.get(captured.selected_project)===pending)
          pendingInitialSources.delete(captured.selected_project);
        throw error;
      }
      unchanged(token);
      if(!saved?.locator||!JSON.parse(saved.locator).native_source_project_id)throw new Error('E_NATIVE_SOURCE_SAVE_READBACK');
      // Publish a new work only after the Native owner acknowledges the exact
      // source. No old world, browser checkpoint or grant is migrated.
      const expected=await apply(JSON.parse(text),{...projectContext,sourceOnlyFreshExecution:true});
      baseline=JSON.stringify(expected);unchanged(baseline);
      rememberSourceLocator(saved.locator);publishSourceMaterial(supplied);sourceContextActive=true;
      const message='내 작업의 원문을 Native 저장소에서 확인했습니다. ▶로 실행하세요.';
      show(text,message);pendingInitialSources.delete(captured.selected_project);return {...saved,message};
    },{propagate:true}),
    saveIfRegisteredSource:async()=>{
      if(!artifactText)return null;
      const current=capture(),prior=JSON.parse(baseline);
      if(current.selected_project!==prior.selected_project||current.source_profile!==prior.source_profile)return null;
      const token=JSON.stringify(current);
      const candidate=await sourceSaveCandidate();unchanged(token);
      const required=await operate('source_probe',JSON.stringify({artifact_json_utf8:candidate.text}));unchanged(token);
      if(required!=='true'&&!sourceContextActive&&!(await nativeSourceAvailable()))return null;
      unchanged(token);
      return perform(()=>saveSourceAction(candidate),{propagate:true});
    },
    reopenIfRegisteredSource:async(projectIdentity='',{allowLast=false}={})=>{
      let locator;try{locator=JSON.parse(sourceLocator(artifactText?JSON.parse(artifactText).authored_identity.logical_dependency_id:'')||'null');}catch{return null;}
      const savedId=locator?.source_project_id??locator?.native_source_project_id;
      if(typeof savedId!=='string')return null;
      const sameCurrent=sourceContextActive&&baseline&&capture().selected_project===JSON.parse(baseline).selected_project;
      if(!allowLast&&!sameCurrent&&projectIdentity!==savedId)return null;
      return perform(reopenSourceAction,{propagate:true});
    },
    getNominalSourceArtifact:()=>{
      if(!sourceContextActive||!nominalSourceArtifactText||!baseline||JSON.stringify(capture())!==baseline)return '';
      try{const saved=JSON.parse(nominalSourceArtifactText);return saved.authored_project_json_utf8===artifactText?nominalSourceArtifactText:'';}catch{return '';}
    },
    getArtifactText:()=>artifactText,
    getNativeSourceLocator:()=>{
      if(!sourceContextActive||!artifactText)return '';
      const id=JSON.parse(artifactText).authored_identity.logical_dependency_id,text=sourceLocator(id);
      try{return JSON.parse(text).native_source_project_id===id?text:'';}catch{return '';}
    },

    hasSourceEdits:()=>Boolean(artifactText&&baseline&&capture().selected_project===JSON.parse(baseline).selected_project&&JSON.stringify(capture())!==baseline),
    nativeSourceRunOptions:()=>{
      let locator;try{locator=JSON.parse(sourceLocator(artifactText?JSON.parse(artifactText).authored_identity.logical_dependency_id:'')||'null');}catch{return {};}
      if(!artifactText||locator?.native_source_project_id!==JSON.parse(artifactText).authored_identity.logical_dependency_id||locator.code_source_selection||locator.nominal_source_selection)return {};
      return {nativeSourceFirstRetry:locator.first_failure_recovery===true&&locator.authoring_selection?.kind===2,nativeSourceNewRun:locator.requires_new_run===true,nativeSourceSelectionJson:JSON.stringify(locator.authoring_selection??null)};
    },
    rememberNativeSelection:selection=>{
      if(!selection||!artifactText||JSON.stringify(capture())!==baseline)return;
      let locator;try{locator=JSON.parse(sourceLocator(artifactText?JSON.parse(artifactText).authored_identity.logical_dependency_id:'')||'null');}catch{return;}
      if(locator?.native_source_project_id!==JSON.parse(artifactText).authored_identity.logical_dependency_id)return;
      rememberSourceLocator(JSON.stringify({...locator,authoring_selection:selection,requires_new_run:false,first_failure_recovery:false}));
    },
    hasNativeSourceContext:()=>{
      if(!sourceContextActive||!artifactText)return false;
      try{return JSON.parse(sourceLocator(artifactText?JSON.parse(artifactText).authored_identity.logical_dependency_id:'')||'null')?.native_source_project_id===JSON.parse(artifactText).authored_identity.logical_dependency_id;}
      catch{return false;}
    },
    getRunProjectArtifact:()=>{
      if(!artifactText) return '';
      const artifact=JSON.parse(artifactText), current=capture(), prior=JSON.parse(baseline);
      if(current.selected_project!==prior.selected_project || !artifact.source_snapshot?.root_path) return '';
      if(JSON.stringify(current)!==baseline) throw new Error('E_PROJECT_RUNTIME_SOURCE_CHANGED: 원문이 바뀌었습니다. 작품 원문을 갱신 저장한 뒤 실행하세요.');
      return artifactText;
    },
    getRunArtifact:()=>{
      if(!artifactText) return '';
      const artifact=JSON.parse(artifactText), current=capture(), prior=JSON.parse(baseline);
      if(current.selected_project!==prior.selected_project) return '';
      // Standalone code custody is not authored/world execution authority.
      try{if(JSON.parse(sourceLocator(artifact.authored_identity.logical_dependency_id)||'null')?.code_source_selection||JSON.parse(sourceLocator(artifact.authored_identity.logical_dependency_id)||'null')?.nominal_source_selection)return '';}catch{}
      if(artifact.schema!=='ddn.authored_project_artifact.v2') return '';
      if(JSON.stringify(current)!==baseline) throw new Error('E_AUTHORED_BOGAE_SOURCE: 원문이 바뀌었습니다. 기존 ID를 유지하여 저장한 뒤 실행하세요.');
      return artifactText;
    },
  };
}
