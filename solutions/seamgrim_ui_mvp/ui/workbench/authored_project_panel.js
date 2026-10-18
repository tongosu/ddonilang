// Persistent IDs are supplied only by the common Rust codec after an explicit
// user operation. Local storage keys and download names are locators, not IDs.
import {createAuthoredSourceStorage} from './persistence/authored_source_save.js';
export function installAuthoredProjectPanel({root, bridge, host, capture, apply, saveRuntime, reopenRuntime, exportRuntime}) {
  const status=root.querySelector('[data-authored-status]');
  const identity=root.querySelector('[data-authored-identity]');
  const contents=root.querySelector('[data-authored-contents]');
  const fileInput=root.querySelector('input[type=file]');
  const key='ddn.authored_project.last.v1';
  const operate=(operation,text)=>bridge.authoredProjectOperation({operation,text});
  const sourceStorage=createAuthoredSourceStorage(bridge);
  let artifactText='', baseline='', busy=false, sourceContextActive=false;
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
    busy=true;
    // Disabling the focused save button can emit focusout(null), which the
    // input host correctly treats as loss. Stay inside this admitted drawer.
    if(root.contains(root.ownerDocument.activeElement)) {
      if(!root.hasAttribute('tabindex'))root.tabIndex=-1;
      root.focus({preventScroll:true});
    }
    root.querySelectorAll('button').forEach(button=>button.disabled=true);
    try { return await action(); }
    catch(error) { status.textContent=`저장 자료 오류: ${String(error?.message ?? error)}`;if(propagate)throw error; }
    finally {busy=false;root.querySelectorAll('button').forEach(button=>button.disabled=false);fileInput.value='';}
  }
  async function openText(text, message, token) {
    const admitted=await operate('open',text);
    unchanged(token);
    const expected=await apply(JSON.parse(admitted)); // Admission first; no execution on open.
    const expectedToken=JSON.stringify(expected);
    unchanged(expectedToken);
    baseline=expectedToken;
    sourceContextActive=false;
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
  async function saveSourceAction() {
    if(!artifactText) throw new Error('먼저 정체를 만들거나 저장 자료를 여세요.');
    const captured=capture(), token=JSON.stringify(captured), prior=JSON.parse(baseline);
    if(captured.selected_project!==prior.selected_project || captured.source_profile!==prior.source_profile) throw new Error('다른 작품이나 프로필로 바뀌었습니다. 저장 자료를 다시 여세요.');
    const text=await operate('update',JSON.stringify({original_json:artifactText,source_snapshot:captured.source_snapshot}));
    unchanged(token);
    const sourceSaved=await sourceStorage.save(text,()=>unchanged(token),{previousArtifactText:artifactText,retainedLocator:localStorage.getItem(key)??''});
    unchanged(token);
    localStorage.setItem(key,sourceSaved?.locator??text);
    baseline=token;
    sourceContextActive=Boolean(sourceSaved);
    show(text,sourceSaved?.message??'이 기기에 작성 자료를 저장했습니다. 다시 열어도 같은 ID를 사용합니다.');
    return sourceSaved;
  }
  root.querySelector('[data-authored-save]').onclick=()=>perform(saveSourceAction);
  for(const [selector,operation] of [['[data-authored-runtime-save]',saveRuntime],['[data-authored-runtime-reopen]',reopenRuntime],['[data-authored-runtime-export]',exportRuntime]]) {
    root.querySelector(selector)?.addEventListener('click',()=>perform(async()=>{
      if(!artifactText || JSON.stringify(capture())!==baseline)throw new Error('먼저 현재 작품 원문을 갱신 저장하세요.');
      if(!operation)throw new Error('E_PROJECT_RUNTIME_UI_UNSUPPORTED');
      status.textContent=await operation(artifactText);
    }));
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
    const token=JSON.stringify(capture()), text=localStorage.getItem(key);
    if(!text) throw new Error('이 기기에 저장한 작성 자료가 없습니다.');
    const locator=JSON.parse(text);
    if((Object.keys(locator).length===1 && typeof locator.source_project_id==='string')||typeof locator.native_source_project_id==='string') {
      const admitted=await sourceStorage.reopen(locator.source_project_id??locator.native_source_project_id,()=>unchanged(token),locator);
      await openText(admitted,'저장 원문과 등록 자료를 다시 읽어 열었습니다. 실행하지 않았습니다.',token);sourceContextActive=true;
      return {message:'저장 원문과 등록 자료를 다시 읽었습니다. 실행하지 않았습니다.'};
    }
    await openText(text,'저장 자료를 다시 열었습니다. 실행은 직접 시작하세요.',token);
    return null;
  }
  root.querySelector('[data-authored-reopen]').onclick=()=>perform(reopenSourceAction);
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
  fileInput.onchange=()=>perform(async()=>{
    const file=fileInput.files[0];if(!file) return;
    if(file.size>64*1024*1024) throw new Error('저장 자료가 64MiB를 넘습니다.');
    const token=JSON.stringify(capture());
    // ignoreBOM=true preserves the character: Rust rejects it, never strips it.
    const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(await file.arrayBuffer());
    await openText(text,'파일을 열었습니다. 영구 ID는 보존했고 실행하지 않았습니다.',token);
  });
  return {
    saveIfRegisteredSource:async()=>{
      if(!artifactText)return null;
      const current=capture(),prior=JSON.parse(baseline);
      if(current.selected_project!==prior.selected_project||current.source_profile!==prior.source_profile)return null;
      const token=JSON.stringify(current);
      const updated=await operate('update',JSON.stringify({original_json:artifactText,source_snapshot:current.source_snapshot}));unchanged(token);
      const required=await operate('source_probe',JSON.stringify({artifact_json_utf8:updated}));unchanged(token);
      if(required!=='true'&&!sourceContextActive)return null;
      return perform(saveSourceAction,{propagate:true});
    },
    reopenIfRegisteredSource:async(projectIdentity='')=>{
      let locator;try{locator=JSON.parse(localStorage.getItem(key)??'null');}catch{return null;}
      const savedId=locator?.source_project_id??locator?.native_source_project_id;
      if(typeof savedId!=='string')return null;
      const sameCurrent=sourceContextActive&&baseline&&capture().selected_project===JSON.parse(baseline).selected_project;
      if(!sameCurrent&&projectIdentity!==savedId)return null;
      return perform(reopenSourceAction,{propagate:true});
    },
    getArtifactText:()=>artifactText,
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
      if(artifact.schema!=='ddn.authored_project_artifact.v2') return '';
      if(JSON.stringify(current)!==baseline) throw new Error('E_AUTHORED_BOGAE_SOURCE: 원문이 바뀌었습니다. 기존 ID를 유지하여 저장한 뒤 실행하세요.');
      return artifactText;
    },
  };
}
