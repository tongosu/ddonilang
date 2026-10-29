import {createMultilangController} from './controller.js';
import {mountMultilangPanel} from './panel.js';
import {createAuthoredSourceStorage} from '../persistence/authored_source_save.js';
import {nativeSourceAvailable} from '../../runtime/native_source_transport.js';

// UI adapter only. The Rust codec owns whole-project identity and Native owns storage/CAS.
export function installMultilangHost({root, runtimeBridge, readContext,
  storage = createAuthoredSourceStorage(runtimeBridge), nativeAvailable = nativeSourceAvailable}) {
  const document = root.ownerDocument;
  const load = document.createElement('button'); load.type = 'button';
  load.textContent = '현재 작품 원문 가져오기'; load.dataset.testid = 'multilang-load-current';
  const status = document.createElement('p'); status.setAttribute('role','status'); status.dataset.testid = 'multilang-host-status';
  const area = document.createElement('div'); area.className = 'multilang-consumer';
  root.append(load, status, area);
  let context = null, epoch = 0, disposed = false, loading = false;
  const receipts = new Map();
  const token = value => JSON.stringify(value);
  const operate = async (operation, value) => runtimeBridge.authoredProjectOperation({operation,text:value});
  function refusal(message,presentationStatus) { const error=new Error(message); error.presentationStatus=presentationStatus; return error; }
  function sameSnapshot(a,b) {
    return a?.entry_path === b?.entry_path && (a?.root_path??null) === (b?.root_path??null)
      && Array.isArray(a?.files) && a.files.length === b?.files?.length
      && a.files.every((file,index) => file.path === b.files[index].path && file.text === b.files[index].text);
  }
  function unchanged(captured) {
    if (disposed || context !== captured || token(readContext()) !== captured.token) {
      throw new Error('E_MULTILANG_CONTEXT_CHANGED: 원래 작품·문서·판본이 바뀌었습니다. 다시 가져오세요.');
    }
  }
  function requireDocument(value) {
    const captured = context;
    if (!captured) throw new Error('E_MULTILANG_PROJECT_REQUIRED: 먼저 현재 작품을 가져오세요.');
    unchanged(captured);
    if (value.path !== captured.path || value.sourceProfileIdentity !== captured.profile) {
      throw new Error('E_MULTILANG_DOCUMENT_CONTEXT: 원래 문서·프로필과 다릅니다.');
    }
    return captured;
  }
  async function admitted(text) {
    const opened = await operate('open',text);
    if (typeof opened !== 'string') throw new Error('E_MULTILANG_ARTIFACT_RESPONSE');
    const artifact = JSON.parse(opened);
    if (!artifact.authored_identity?.logical_dependency_id || !artifact.source_profile
        || !Array.isArray(artifact.source_snapshot?.files)) throw new Error('E_MULTILANG_ARTIFACT_CONTEXT');
    return {text:opened, artifact};
  }
  function selected(artifact, captured) {
    if (artifact.authored_identity.logical_dependency_id !== captured.projectId
        || artifact.source_profile !== captured.profile) throw new Error('E_MULTILANG_SAVED_CONTEXT');
    const files = artifact.source_snapshot.files.filter(file => file.path === captured.path);
    if (files.length !== 1 || typeof files[0].text !== 'string') throw new Error('E_MULTILANG_SAVED_DOCUMENT');
    return {sourceText:files[0].text,path:captured.path,sourceProfileIdentity:captured.profile};
  }
  function singleSource(captured) {
    // Existing sourceAnalysis is a single-file frontdoor. Never flatten module/DDL bundles into it.
    if (captured.artifact.source_snapshot.files.length !== 1 || captured.artifact.source_snapshot.root_path) {
      throw refusal('E_MULTILANG_BUNDLE_ANALYSIS_UNSUPPORTED: 여러 파일의 공통 분석 연결은 아직 지원하지 않습니다. 전체 작품 원문 저장은 별도입니다.','unsupported');
    }
  }
  const bridge = {
    async sourceAnalysis(request) {
      const captured = requireDocument({path:request.path,sourceProfileIdentity:request.source_profile_identity});
      singleSource(captured);
      const result = await runtimeBridge.sourceAnalysis(request); unchanged(captured); return result;
    },
    async runSummaryWithSourceProfile(request) {
      const captured = requireDocument({path:request.sourcePath,sourceProfileIdentity:request.sourceProfileIdentity});
      singleSource(captured);
      const result = await runtimeBridge.runSummaryWithSourceProfile(request); unchanged(captured); return result;
    },
  };
  const controller = createMultilangController({runtimeBridge:bridge, persistence:{
    async saveDocument(value) {
      const captured = requireDocument(value);
      if (!await nativeAvailable()) throw new Error('E_MULTILANG_NATIVE_REQUIRED: Native 작업대에서만 저장합니다. 브라우저 저장으로 대체하지 않습니다.');
      unchanged(captured);
      const snapshot = structuredClone(captured.artifact.source_snapshot);
      const files = snapshot.files.filter(file => file.path === value.path);
      if (files.length !== 1) throw new Error('E_MULTILANG_DOCUMENT_CONTEXT');
      files[0].text = value.sourceText;
      const updated = await operate('update',JSON.stringify({original_json:captured.artifactText,source_snapshot:snapshot}));
      unchanged(captured);
      const checked = await admitted(updated); unchanged(captured); selected(checked.artifact,captured);
      if (checked.text !== captured.artifactText) {
        let prior; try { prior=JSON.parse(captured.locator); } catch {}
        if (prior?.native_source_project_id !== captured.projectId || !prior.authoring_selection) {
          throw refusal('E_MULTILANG_NATIVE_SELECTION_REQUIRED: 수정 저장에는 Native에서 받은 기준 판이 필요합니다. 원래 작품에서 저장·실행한 뒤 다시 가져오세요.','supply_missing');
        }
      }
      const saved = await storage.save(checked.text,()=>unchanged(captured),{
        previousArtifactText:captured.artifactText,retainedLocator:captured.locator,
      });
      unchanged(captured);
      const locator = saved?.locator && JSON.parse(saved.locator);
      if (locator?.native_source_project_id !== captured.projectId) {
        throw new Error('E_MULTILANG_NATIVE_READBACK: Native 저장 응답을 확인하지 못했습니다.');
      }
      // Keep only the actual Native locator. No localStorage fallback and no active editor/VM apply.
      captured.artifact = checked.artifact; captured.artifactText = checked.text; captured.locator = saved.locator;
      receipts.set(saved.locator,{context:captured,epoch:captured.epoch});
      return {locator:saved.locator};
    },
    async openDocument(receipt) {
      const owned = receipts.get(receipt?.locator);
      if (!owned || owned.context !== context || owned.epoch !== context.epoch) throw new Error('E_MULTILANG_LOCATOR_CONTEXT');
      const captured = owned.context; unchanged(captured);
      if (!await nativeAvailable()) throw new Error('E_MULTILANG_NATIVE_REQUIRED');
      unchanged(captured);
      const locator = JSON.parse(receipt.locator);
      if (locator.native_source_project_id !== captured.projectId) throw new Error('E_MULTILANG_LOCATOR_CONTEXT');
      const text = await storage.reopen(captured.projectId,()=>unchanged(captured),locator);
      unchanged(captured);
      const checked = await admitted(text); unchanged(captured);
      const value = selected(checked.artifact,captured);
      captured.artifactText = checked.text; captured.artifact = checked.artifact;
      captured.locator = JSON.stringify(locator);
      return value;
    },
  }});
  const panel = mountMultilangPanel(area,controller);
  async function loadCurrent() {
    if (loading || controller.snapshot().slots.ko.busy) return;
    loading = true; load.disabled = true;
    try {
      const raw = readContext(), originalToken = token(raw);
      if (!raw?.artifactText) throw new Error('E_MULTILANG_PROJECT_REQUIRED: 작성 정체 자료에서 정체를 만들거나 Native 저장본을 먼저 여세요.');
      if (raw.running) throw new Error('E_MULTILANG_PAUSE_REQUIRED: 원래 작품을 일시정지한 뒤 가져오세요.');
      const checked = await admitted(raw.artifactText);
      if (disposed || token(readContext()) !== originalToken) throw new Error('E_MULTILANG_CONTEXT_CHANGED');
      if (raw.profile !== checked.artifact.source_profile || !sameSnapshot(raw.sourceSnapshot,checked.artifact.source_snapshot)) {
        throw new Error('E_MULTILANG_PROJECT_SOURCE_CHANGED: 전체 작품 원문과 정체 자료가 다릅니다. 기존 원문 저장 경로에서 먼저 확인하세요.');
      }
      const captured = {token:originalToken,epoch:++epoch,artifact:checked.artifact,artifactText:checked.text,
        projectId:checked.artifact.authored_identity.logical_dependency_id,profile:raw.profile,path:raw.path,locator:raw.locator??''};
      const value = selected(checked.artifact,captured);
      context = captured; receipts.clear();
      controller.setDocument('ko',value,{replaceDocument:true});
      status.textContent = '전체 작품에서 선택 원문을 가져왔습니다. 원래 편집기와 세계는 변경하지 않습니다. 일본어 등록·함수 연결은 미지원입니다.';
    } catch (error) { status.textContent = String(error?.message ?? error); }
    finally { loading = false; load.disabled = false; }
  }
  load.addEventListener('click',loadCurrent);
  return Object.freeze({loadCurrent,dispose() {
    disposed=true; load.removeEventListener('click',loadCurrent); panel.dispose(); controller.dispose();
    context=null; receipts.clear(); load.remove(); status.remove(); area.remove();
  }});
}
