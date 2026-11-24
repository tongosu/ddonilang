import {createCurrentProjectLanguageSupport} from './current_project_languages.js';
import {createMultilangController} from './controller.js';
import {mountMultilangPanel} from './panel.js';
import {ddlEditionLabels,diagnosticPresentation,hostLabels} from './messages.js';
import {textareaSelectionForUtf8Span} from './source_selection.js';
import {createAuthoredSourceStorage} from '../persistence/authored_source_save.js';
import {nativeSourceAvailable,nativeSourceRequest} from '../../runtime/native_source_transport.js';

// Approved R19 raw example sources only. The Rust create/prepare APIs issue all identities.
const J1_PROFILE='v1-core-v25-ja-base-r1';
const MAX_DDL_FOLDER_FILES=256;
const LIMITED_EXAMPLE={"original":{"source_profile":"v1-core-v25","module_name":"","source_snapshot":{"entry_path":"main.ddn","root_path":"root.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\"),(이름=내계산,머리=\"내계산.ddn\",원문=\"내계산.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n"},{"path":"main.ddn","text":"(계산=내계산) 쓰임.\n실행:움직씨 := { ((금액=(12,1) 나눔수, 세액=(3,1) 나눔수) 계산.총액합하다) 보여주기. }.\n"},{"path":"내계산.ddn","text":"(총액합하다=총액합하다) 드러냄.\n(금액:나눔수~에, 세액:나눔수~을~를)\n총액합하다{총액합하기}:나눔수:셈씨 := { 총액합하다 <- 금액 + 세액. }.\n"}]}},"consumer":{"source_profile":"v1-core-v25","module_name":"","source_snapshot":{"entry_path":"main.ddn","root_path":"root.ddn","files":[{"path":"root.ddn","text":"(글판=\"v25\", 모듬=차림[(이름=앱,머리=\"main.ddn\",원문=\"main.ddn\")], 시작=(모듈=앱,정의=실행)) 얽음.\n"},{"path":"main.ddn","text":"(계산=\"./내계산\", 옮김=(대상말씨=\"日本語\", 대상판본=\"1\")) 쓰임.\n실행:움직씨 := { ((金額=(12,1) 有理数, 税額=(3,1) 有理数) 合計する) 보여주기. }.\n"}]}},"ddl":{"path":"일본어.ddl","text":"#!日本語\r\n(金額:有理数~に, 税額:有理数~を)\r\n合計する:有理数:関数 :={...}.\r\n"},"originalPath":"내계산.ddn","consumerPath":"main.ddn"};

// UI adapter only. The Rust codec owns whole-project identity and Native owns storage/CAS.
export function installMultilangHost({root, runtimeBridge, readContext, rememberNativeSave,
  storage = createAuthoredSourceStorage(runtimeBridge), nativeAvailable = nativeSourceAvailable}) {
  const document = root.ownerDocument;
  const load = document.createElement('button'); load.type = 'button';
  load.dataset.testid = 'multilang-load-current';
  const status = document.createElement('p'); status.setAttribute('role','status'); status.dataset.testid = 'multilang-host-status';
  const area = document.createElement('div'); area.className = 'multilang-consumer';
  const example=document.createElement('button');example.type='button';
  example.dataset.testid='multilang-limited-example';
  root.append(load,example,status,area);
  let context = null, limitedContext=null, epoch = 0, disposed = false, loading = false;
  const receipts = new Map();
  const languageSupport=createCurrentProjectLanguageSupport({runtimeBridge,readContext,storage});
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
    return {sourceText:files[0].text,path:captured.path,sourceProfileIdentity:captured.profile,
      ...(captured.profile===J1_PROFILE?{projectPreparation:true}:{})};
  }
  function singleSource(captured) {
    // Existing sourceAnalysis is a single-file frontdoor. Never flatten module/DDL bundles into it.
    if (captured.artifact.source_snapshot.files.length !== 1 || captured.artifact.source_snapshot.root_path) {
      throw refusal('E_MULTILANG_BUNDLE_ANALYSIS_UNSUPPORTED: 여러 파일의 공통 분석 연결은 아직 지원하지 않습니다. 전체 작품 원문 저장은 별도입니다.','unsupported');
    }
  }
  function limitedCaptured(value,role) {
    const captured=limitedContext;
    if (!captured || disposed || value.limitedRole!==role
        || value.path!==(role==='original'?LIMITED_EXAMPLE.originalPath:LIMITED_EXAMPLE.consumerPath)
        || value.sourceProfileIdentity!==(role==='original'?captured.original.artifact:captured.consumer.artifact).source_profile) {
      throw new Error('E_MULTILANG_LIMITED_CONTEXT_REQUIRED');
    }
    return captured;
  }
  function sameLimited(captured) {
    if (disposed || limitedContext!==captured) throw new Error('E_MULTILANG_CONTEXT_CHANGED');
  }
  async function editedLimited(captured,base,value) {
    const snapshot=structuredClone(base.artifact.source_snapshot);
    const rows=snapshot.files.filter(file=>file.path===value.path);
    if (rows.length!==1) throw new Error('E_MULTILANG_DOCUMENT_CONTEXT');
    rows[0].text=value.sourceText;
    const updated=await operate('update',JSON.stringify({original_json:base.text,source_snapshot:snapshot}));
    sameLimited(captured);const checked=await admitted(updated);sameLimited(captured);return checked;
  }
  const bridge = {
    readOtherSource,
    async prepareSelectedProject({document:value}) {
      const captured=requireDocument(value);
      if (captured.profile!==J1_PROFILE || value.projectPreparation!==true) {
        throw refusal('E_MULTILANG_PROJECT_PROFILE_UNSUPPORTED','unsupported');
      }
      const savedText=captured.artifact.source_snapshot.files.find(file=>file.path===value.path)?.text;
      const nativeSource = captured.nativeSource;
      if (nativeSource?.status === 'rejected') throw refusal(nativeSource.error);
      if (nativeSource && value.sourceText === savedText) {
        if (nativeSource.status === 'accepted') {
          return {sourceArtifactText:nativeSource.sourceArtifactText,restoredNativeSource:true};
        }
        // The Rust probe distinguishes ordinary projects from registered source-only projects.
        const required = await operate('source_probe',JSON.stringify({artifact_json_utf8:captured.artifactText}));
        unchanged(captured);
        if (required !== 'false') throw refusal(nativeSource.error,'supply_missing');
      }
      const snapshot=structuredClone(captured.artifact.source_snapshot);
      const rows=snapshot.files.filter(file=>file.path===value.path);
      if (rows.length!==1) throw new Error('E_MULTILANG_DOCUMENT_CONTEXT');
      rows[0].text=value.sourceText;
      const updated=await operate('update',JSON.stringify({original_json:captured.artifactText,source_snapshot:snapshot}));
      unchanged(captured);
      const checked=await admitted(updated);unchanged(captured);selected(checked.artifact,captured);
      // Detached source-only preparation: never apply to the active editor/world or save here.
      const sourceArtifactText=await operate('source_prepare',JSON.stringify({artifact_json_utf8:checked.text,
        expected_parent_hex:'',revision_hex:'17',previous_source_artifact_utf8:null}));
      unchanged(captured);
      // Rust successfully parsed the complete original but needs no registered
      // source-only carrier. Keep that actual original for the ordinary VM.
      if(sourceArtifactText==='null')return {sourceArtifactText:checked.text,originalSourceExecution:true};
      return {sourceArtifactText};
    },
    async runSelectedProject({document:value,preparation}) {
      const captured=requireDocument(value);
      if (captured.profile!==J1_PROFILE || value.projectPreparation!==true) {
        throw refusal('E_MULTILANG_PROJECT_PROFILE_UNSUPPORTED','unsupported');
      }
      if(preparation.originalSourceExecution===true){
        const session=await runtimeBridge.createSession({ddnSourceText:value.sourceText,
          sourceProfileIdentity:captured.profile,projectRuntimeArtifactText:preparation.sourceArtifactText});
        try {
          unchanged(captured);const frame=await session.step({n:1,sourceText:value.sourceText});unchanged(captured);
          const texts=frame?.output_display?.texts;
          if(!Array.isArray(texts)||!texts.every(text=>typeof text==='string'))throw new Error('E_MULTILANG_RESULT_OUTPUT_REQUIRED');
          return {...frame,output_log_texts:structuredClone(texts)};
        } finally {await session.close();}
      }
      const raw=await operate('public_callable_source_run',JSON.stringify({source_artifact_utf8:preparation.sourceArtifactText}));
      unchanged(captured);return JSON.parse(raw);
    },
    async prepareLimitedOriginal({document:value}) {
      const captured=limitedCaptured(value,'original');
      const original=await editedLimited(captured,captured.original,value);
      const sourceArtifactText=await operate('first_callable_original_source_prepare',JSON.stringify({
        original_project_json_utf8:original.text,original_source_path:value.path,
        ddl_source:LIMITED_EXAMPLE.ddl,revision_hex:'01'}));
      sameLimited(captured);
      return {sourceArtifactText,originalProjectText:original.text,originalSourcePath:value.path};
    },
    async prepareLimitedConsumer({document:value,original}) {
      const captured=limitedCaptured(value,'consumer');
      const consumer=await editedLimited(captured,captured.consumer,value);
      const sourceArtifactText=await operate('first_callable_source_prepare',JSON.stringify({
        original_project_json_utf8:original.originalProjectText,original_source_artifact_utf8:original.sourceArtifactText,
        original_source_path:original.originalSourcePath,ddl_source:LIMITED_EXAMPLE.ddl,
        consumer_project_json_utf8:consumer.text,consumer_source_path:value.path,revision_hex:'02'}));
      sameLimited(captured);return {sourceArtifactText};
    },
    async runLimitedConsumer({document:value,preparation}) {
      const captured=limitedCaptured(value,'consumer');
      const raw=await operate('first_callable_source_run',JSON.stringify({source_artifact_utf8:preparation.sourceArtifactText}));
      sameLimited(captured);return JSON.parse(raw);
    },
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
      let nativeSource={status:'supply_missing',error:'E_MULTILANG_SAVED_SOURCE_REQUIRED'};
      const saved = await storage.save(checked.text,()=>unchanged(captured),{
        previousArtifactText:captured.artifactText,retainedLocator:captured.locator,
        receiveSourceArtifact:captured.profile===J1_PROFILE ? value=>{nativeSource=value;} : null,
      });
      unchanged(captured);
      const locator = saved?.locator && JSON.parse(saved.locator);
      if (locator?.native_source_project_id !== captured.projectId) {
        throw new Error('E_MULTILANG_NATIVE_READBACK: Native 저장 응답을 확인하지 못했습니다.');
      }
      // Keep only the actual Native locator. No localStorage fallback and no active editor/VM apply.
      captured.artifact = checked.artifact; captured.artifactText = checked.text; captured.locator = saved.locator;
      if(captured.profile===J1_PROFILE)captured.nativeSource=nativeSource;
      receipts.set(saved.locator,{context:captured,epoch:captured.epoch});
      rememberNativeSave?.({projectId:captured.projectId,nativeLocator:saved.locator});
      return {locator:saved.locator,...(captured.profile===J1_PROFILE?{recheckRequired:true}:{})};
    },
    async openDocument(receipt) {
      const owned = receipts.get(receipt?.locator);
      if (!owned || owned.context !== context || owned.epoch !== context.epoch) throw new Error('E_MULTILANG_LOCATOR_CONTEXT');
      const captured = owned.context; unchanged(captured);
      if (!await nativeAvailable()) throw new Error('E_MULTILANG_NATIVE_REQUIRED');
      unchanged(captured);
      const locator = JSON.parse(receipt.locator);
      if (locator.native_source_project_id !== captured.projectId) throw new Error('E_MULTILANG_LOCATOR_CONTEXT');
      let nativeSource={status:'supply_missing',error:'E_MULTILANG_SAVED_SOURCE_REQUIRED'};
      const text = await storage.reopen(captured.projectId,()=>unchanged(captured),locator,{
        receiveSourceArtifact:captured.profile===J1_PROFILE ? value=>{nativeSource=value;} : null,
      });
      unchanged(captured);
      const checked = await admitted(text); unchanged(captured);
      const value = selected(checked.artifact,captured);
      captured.artifactText = checked.text; captured.artifact = checked.artifact;
      captured.locator = JSON.stringify(locator);
      if(captured.profile===J1_PROFILE)captured.nativeSource=nativeSource;
      return value;
    },
  }});
  function readOtherSource(value, path) {
    if (value?.projectPreparation !== true || value.path === path) return null;
    try {
      const captured = requireDocument(value);
      return captured.artifact.source_snapshot.files.find(file => file.path === path)?.text ?? null;
    } catch {
      return null;
    }
  }
  const panel = mountMultilangPanel(area,controller,{readOtherSource});
  let hostStatus = null;
  function renderHost(locale) {
    const labels = hostLabels[locale];
    load.textContent = labels.load;
    example.textContent = labels.example;
    status.textContent = hostStatus?.raw ?? labels[hostStatus?.key] ?? '';
  }
  function setHostStatus(next) {
    hostStatus = next;
    renderHost(controller.snapshot().locale);
  }
  const unsubscribeHost = controller.subscribe(state => renderHost(state.locale));
  async function loadCurrent({quietWhenMissing = false} = {}) {
    if (loading) return;
    loading = true; load.disabled = true;
    try {
      const raw = readContext(), originalToken = token(raw);
      if (context && context.token !== originalToken) {
        const language = context.profile===J1_PROFILE ? 'ja' : 'ko';
        context = null;
        receipts.clear();
        setHostStatus(null);
        controller.clearDocument(language,{invalidatePending:true});
      }
      if (Object.values(controller.snapshot().slots).some(slot=>slot.busy)) return;
      if (!raw?.artifactText) {
        if (quietWhenMissing) return;
        throw new Error('E_MULTILANG_PROJECT_REQUIRED: 작성 정체 자료에서 정체를 만들거나 Native 저장본을 먼저 여세요.');
      }
      if (raw.running) throw new Error('E_MULTILANG_PAUSE_REQUIRED: 원래 작품을 일시정지한 뒤 가져오세요.');
      const checked = await admitted(raw.artifactText);
      if (disposed || token(readContext()) !== originalToken) throw new Error('E_MULTILANG_CONTEXT_CHANGED');
      if (raw.profile !== checked.artifact.source_profile || !sameSnapshot(raw.sourceSnapshot,checked.artifact.source_snapshot)) {
        throw new Error('E_MULTILANG_PROJECT_SOURCE_CHANGED: 전체 작품 원문과 정체 자료가 다릅니다. 기존 원문 저장 경로에서 먼저 확인하세요.');
      }
      const captured = {token:originalToken,epoch:++epoch,artifact:checked.artifact,artifactText:checked.text,
        projectId:checked.artifact.authored_identity.logical_dependency_id,profile:raw.profile,path:raw.path,locator:raw.locator??''};
      if(captured.profile===J1_PROFILE) {
        let locator;try{locator=JSON.parse(captured.locator);}catch{}
        if(locator?.native_source_project_id===captured.projectId) {
          let nativeSource={status:'supply_missing',error:'E_MULTILANG_SAVED_SOURCE_REQUIRED'};
          const current=()=>{if(disposed||token(readContext())!==originalToken)throw new Error('E_MULTILANG_CONTEXT_CHANGED');};
          const savedText=await storage.reopen(captured.projectId,current,locator,{receiveSourceArtifact:value=>{nativeSource=value;}});
          current();
          if(savedText!==checked.text)throw new Error('E_MULTILANG_PROJECT_SOURCE_CHANGED');
          captured.nativeSource=nativeSource;
        }
      }
      const value = selected(checked.artifact,captured);
      context = captured; limitedContext=null; receipts.clear();
      const language=captured.profile===J1_PROFILE?'ja':'ko';
      controller.clearDocument(language==='ja'?'ko':'ja');
      controller.setDocument(language,value,{replaceDocument:true});
      setHostStatus({key:captured.profile===J1_PROFILE ? 'loadedJ1' : 'loadedKorean'});
    } catch (error) { setHostStatus({raw:String(error?.message ?? error)}); }
    finally { loading = false; load.disabled = false; }
  }
  async function loadLimitedExample() {
    if (loading || Object.values(controller.snapshot().slots).some(slot=>slot.busy)) return;
    loading=true;load.disabled=true;example.disabled=true;
    try {
      const original=await admitted(await operate('create',JSON.stringify(LIMITED_EXAMPLE.original)));
      const consumer=await admitted(await operate('create',JSON.stringify(LIMITED_EXAMPLE.consumer)));
      if (disposed) return;
      const captured={original,consumer,epoch:++epoch};limitedContext=captured;context=null;receipts.clear();
      for (const [language,role,owner,path] of [['ko','original',original,LIMITED_EXAMPLE.originalPath],
          ['ja','consumer',consumer,LIMITED_EXAMPLE.consumerPath]]) {
        const rows=owner.artifact.source_snapshot.files.filter(file=>file.path===path);
        if (rows.length!==1) throw new Error('E_MULTILANG_DOCUMENT_CONTEXT');
        controller.setDocument(language,{sourceText:rows[0].text,path,
          sourceProfileIdentity:owner.artifact.source_profile,limitedRole:role},{replaceDocument:true});
      }
      setHostStatus({key:'loadedExample'});
    } catch(error) {setHostStatus({raw:String(error?.message??error)});}
    finally {loading=false;load.disabled=false;example.disabled=false;}
  }
  example.addEventListener('click',loadLimitedExample);
  load.addEventListener('click',loadCurrent);
  const savedV4=mountSavedPublicCallable(root,operate,controller);
  return Object.freeze({loadCurrent,loadLimitedExample,refreshLanguageSupport:()=>languageSupport.refresh(),languageSupportState:()=>languageSupport.getState(),dispose() {
    disposed=true; languageSupport.dispose(); unsubscribeHost(); savedV4(); load.removeEventListener('click',loadCurrent); panel.dispose(); controller.dispose();
    context=null;limitedContext=null;receipts.clear();example.removeEventListener('click',loadLimitedExample);
    example.remove();load.remove();status.remove();area.remove();
  }});
}

// A recorded DDL consumer has its own source envelope and execution owner.
// It cannot enter the ordinary editable-project path without losing its binding.
function mountSavedPublicCallable(root,operate,controller) {
  let locale=controller.snapshot().locale;
  const persistentLabels=[];
  const formLabels=[];
  const optionLabels=[];
  function bind(node,key,{attribute,args=[],group=persistentLabels}={}) {
    const render=()=>{
      const value=ddlEditionLabels[locale][key];
      const message=typeof value==='function'?value(...args):value;
      if(attribute)node.setAttribute(attribute,message);
      else node.textContent=message;
    };
    group.push(render);
    render();
  }
  let statusMessage={key:'chooseFile',args:[]};
  function renderStatus() {
    if(statusMessage.editionDiagnostic){
      const {diagnostic,consumer}=statusMessage.editionDiagnostic;
      const {code,message,source_path,source_span}=diagnostic;
      const guidance=code==='E_MULTILANG_FULLWIDTH_SYNTAX'||(!consumer&&code==='E_DDL_PUBLIC_PIN_OWNER')
        ?diagnosticPresentation(diagnostic,locale).guidance
        :consumer?ddlEditionLabels[locale].editionConsumerGuidance:ddlEditionLabels[locale].editionDdlSurfaceGuidance;
      const diagnosticCode=message===code||message.includes(code+':')?'':code+' | ';
      status.textContent=message+'\n'+diagnosticCode+source_path+' · UTF-8 '+source_span.start+'–'+source_span.end+'\n'+guidance;
      return;
    }
    if(statusMessage.runtime){
      const {code,message,source_path,source_span}=statusMessage.runtime;
      status.textContent=message+'\n'+code+' | '+source_path+' · UTF-8 '+source_span.start+'–'+source_span.end
        +'\n'+ddlEditionLabels[locale].runtimeSourceGuidance;
      return;
    }
    if(statusMessage.raw!==undefined){
      const code=statusMessage.raw.match(/^E_[A-Z0-9_]+/u)?.[0];
      const hint=ddlEditionLabels[locale].errors[code];
      status.textContent=hint?statusMessage.raw+'\n'+hint:statusMessage.raw;
      return;
    }
    const value=ddlEditionLabels[locale][statusMessage.key];
    status.textContent=typeof value==='function'?value(...statusMessage.args):value;
  }
  function setStatus(key,...args) {
    clearRuntimeLocation();
    statusMessage={key,args};
    renderStatus();
  }

  const document=root.ownerDocument;
  const section=document.createElement('section');
  bind(section,'section',{attribute:'aria-label'});
  const file=document.createElement('input');
  file.type='file'; file.accept='.json,.detjson,application/json';
  bind(file,'file',{attribute:'aria-label'});
  const status=document.createElement('p'); status.setAttribute('role','status');
  renderStatus();
  const source=document.createElement('pre'); source.hidden=true;
  const run=document.createElement('button'); run.type='button'; run.disabled=true;
  bind(run,'run');
  const runtimeJump=document.createElement('button'); runtimeJump.type='button'; runtimeJump.hidden=true;
  bind(runtimeJump,'consumerSetupJump');
  const runtimeSource=document.createElement('textarea'); runtimeSource.readOnly=true;
  runtimeSource.rows=5; runtimeSource.hidden=true;
  bind(runtimeSource,'consumerSetupDiagnosticSource',{attribute:'aria-label'});
  let runtimeSelection=null;
  function clearRuntimeLocation() {
    runtimeSelection=null;
    runtimeJump.hidden=runtimeSource.hidden=true;
    runtimeSource.value='';
  }
  runtimeJump.addEventListener('click',()=>{
    if(!runtimeSelection)return;
    runtimeSource.hidden=false;
    runtimeSource.focus();
    runtimeSource.setSelectionRange(runtimeSelection.start,runtimeSelection.end);
  });
  const output=document.createElement('pre'); output.hidden=true;
  output.setAttribute('aria-live','polite');
  const save=document.createElement('button'); save.type='button';
  bind(save,'save');
  const refresh=document.createElement('button'); refresh.type='button';
  bind(refresh,'refresh');
  const projects=document.createElement('select');
  bind(projects,'projects',{attribute:'aria-label'});
  const reopen=document.createElement('button'); reopen.type='button';
  bind(reopen,'reopen');
  const exportSaved=document.createElement('button'); exportSaved.type='button';
  bind(exportSaved,'exportSaved');
  const workflowHint=document.createElement('p');
  bind(workflowHint,'workflow');
  // A revision is selected and read back before Run becomes available.
  const editionControls=document.createElement('fieldset');
  editionControls.hidden=true;
  const editionTitle=document.createElement('legend'); bind(editionTitle,'title');
  const editionSelection=document.createElement('select');
  bind(editionSelection,'selection',{attribute:'aria-label'});
  const openEdition=document.createElement('button'); openEdition.type='button'; bind(openEdition,'open');
  const editEdition=document.createElement('details');
  const editTitle=document.createElement('summary'); bind(editTitle,'update');
  const form=document.createElement('div');
  editEdition.append(editTitle,form);
  const originalReview=document.createElement('section');
  bind(originalReview,'originalCompare',{attribute:'aria-label'});
  const originalTitle=document.createElement('h4'); bind(originalTitle,'originalCompare');
  const originalNote=document.createElement('p'); bind(originalNote,'originalNote');
  const originalFolder=document.createElement('input');
  originalFolder.type='file'; originalFolder.setAttribute('webkitdirectory',''); originalFolder.multiple=true;
  bind(originalFolder,'originalFolder',{attribute:'aria-label'});
  const originalStatus=document.createElement('p'); originalStatus.setAttribute('role','status');
  const originalResults=document.createElement('ul');
  const openNewDraft=document.createElement('button');
  openNewDraft.type='button'; openNewDraft.hidden=true;
  bind(openNewDraft,'originalNewDraft');
  openNewDraft.addEventListener('click',()=>{
    initialAuthoring.open=true;
    initialFolder.focus();
  });
  originalReview.append(originalTitle,originalNote,originalFolder,originalStatus,originalResults,openNewDraft);
  const compareStored=document.createElement('button');
  compareStored.type='button'; bind(compareStored,'storedComparison');
  const storedComparison=document.createElement('section');
  bind(storedComparison,'storedComparison',{attribute:'aria-label'});
  const storedViews=['storedPrevious','storedCurrent'].map(key=>{
    const heading=document.createElement('h4'); bind(heading,key);
    const body=document.createElement('pre');
    storedComparison.append(heading,body);
    return body;
  });
  function clearStoredComparison() {
    storedComparison.hidden=true;
    for(const body of storedViews)body.textContent='';
  }
  clearStoredComparison();
  editionControls.append(editionTitle,editionSelection,openEdition,compareStored,storedComparison,originalReview,editEdition);

  let originalComparison=null;
  function renderOriginalComparison() {
    const labels=ddlEditionLabels[locale];
    openNewDraft.hidden=true;
    originalResults.replaceChildren();
    originalStatus.textContent='';
    if(originalComparison===null)return;
    if(originalComparison.error){originalStatus.textContent=labels.originalError; return;}
    const counts={originalSame:0,originalChanged:0,originalMissing:0,originalAdded:0};
    for(const {path,result} of originalComparison.rows){
      counts[result]++;
      const row=document.createElement('li');
      row.textContent=path+' — '+labels[result];
      originalResults.append(row);
    }
    originalStatus.textContent=labels.originalSummary(counts.originalSame,counts.originalChanged,counts.originalMissing,counts.originalAdded);
    if(counts.originalChanged||counts.originalMissing||counts.originalAdded){
      originalStatus.textContent+=' '+labels.originalDifferenceGuidance;
      openNewDraft.hidden=false;
    }
  }
  function clearOriginalComparison() {
    originalComparison=null;
    originalFolder.value='';
    renderOriginalComparison();
  }
  originalFolder.addEventListener('change',()=>{
    const files=Array.from(originalFolder.files);
    if(!files.length)return;
    void perform(async()=>{
      try {
        if(files.length>MAX_DDL_FOLDER_FILES)throw new Error('E_SOURCE_SAVE_REQUEST_LIMIT');
        const byPath=folderFiles(files);
        const rows=[];
        // Check recorded files by bytes. An unrecorded path is informational:
        // its contents are never read, parsed or sent to Native.
        for(const {path,bytes,sha256} of recorded.originalFiles){
          const file=byPath.get(path);
          byPath.delete(path);
          let result='originalMissing';
          if(file){
            result='originalChanged';
            if(file.size===bytes){
              const digest=await crypto.subtle.digest('SHA-256',await file.arrayBuffer());
              const hex=Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join('');
              if(hex===sha256)result='originalSame';
            }
          }
          rows.push({path,result});
        }
        for(const path of [...byPath.keys()].sort())rows.push({path,result:'originalAdded'});
        if(!disposed)originalComparison={rows};
      } catch {
        if(!disposed)originalComparison={error:true};
      } finally {
        if(!disposed){originalFolder.value=''; renderOriginalComparison();}
      }
    });
  });

  function field(key,value,multiline=false,args=[]) {
    const row=document.createElement('label');
    const input=document.createElement(multiline?'textarea':'input');
    input.value=value;
    if(multiline){input.rows=6;input.cols=42;}
    const caption=document.createElement('span');
    bind(caption,key,{args,group:formLabels});
    row.append(caption,' ',input);
    const line=document.createElement('div'); line.append(row);
    form.append(line);
    return input;
  }
  function editedText(input,original) {
    // A textarea exposes LF even when the loaded source used CRLF.
    return input.value===original.replace(/\r\n|\r/g,'\n')?original:input.value;
  }
  function comparisonText(value,consumerPath) {
    const rows=value.surfaces.map(row=>row.public_name+' → '+row.surface+'\n'+row.pins.map(
      ([original,surface,particle])=>'  '+original+' → '+surface+' ('+particle+')',
    ).join('\n')).join('\n');
    return [value.edition,rows,value.ddl_source.path,value.ddl_source.text,consumerPath,value.consumer_source_utf8].join('\n\n');
  }
  function editionData(projectHex,reply) {
    if(reply.kind!=='public_callable_edition_view'||reply.project_hex!==projectHex
        ||typeof reply.revision_hex!=='string'||typeof reply.has_previous!=='boolean'
        ||typeof reply.authoring?.consumer_source_utf8!=='string'
        ||!Array.isArray(reply.authoring.original_files)||!reply.authoring.original_files.every(file=>
          typeof file?.path==='string'&&Number.isSafeInteger(file.bytes)&&file.bytes>=0&&/^[0-9a-f]{64}$/u.test(file.sha256)))
      throw new Error('E_DDL_EDITION_READBACK');
    return reply.authoring;
  }
  function renderEdition(projectHex,selection,reply) {
    const data=editionData(projectHex,reply);
    clearStoredComparison();
    recorded={projectHex,selection,revision:reply.revision_hex,nativeReadback:true,hasPrevious:reply.has_previous,
      originalFiles:data.original_files,consumer:{path:data.consumer_path,text:data.consumer_source_utf8}};
    clearOriginalComparison();
    editionControls.hidden=false;
    editionSelection.replaceChildren();
    optionLabels.length=0;
    const options=reply.has_previous?['current','previous']:['current'];
    for(const value of options){
      const option=document.createElement('option'); option.value=value;
      bind(option,value,{group:optionLabels}); editionSelection.append(option);
    }
    editionSelection.value=selection;
    source.textContent=data.consumer_path+'\n'+data.consumer_source_utf8; source.hidden=false;
    setStatus('opened',data.edition);
    output.hidden=true; output.textContent='';
    editEdition.hidden=reply.has_previous;
    formLabels.length=0;
    form.replaceChildren();
    if(reply.has_previous)return;
    const edition=field('newEdition','');
    const ddl=field('ddl',data.ddl_source.text,true);
    const consumer=field('consumer',data.consumer_source_utf8,true);
    const surfaces=data.surfaces.map(row=>({
      name:row.public_name,
      surface:field('callable',row.surface,false,[row.public_name]),
      pins:row.pins.map(([name,value,particle])=>[name,field('pin',value,false,[row.public_name,name]),particle]),
    }));
    const review=document.createElement('button'); review.type='button'; bind(review,'review',{group:formLabels});
    const update=document.createElement('button'); update.type='button'; bind(update,'commit',{group:formLabels});
    const comparison=document.createElement('section');
    bind(comparison,'comparison',{attribute:'aria-label',group:formLabels});
    const changeHeading=document.createElement('h4');
    bind(changeHeading,'reviewChanges',{group:formLabels});
    const changedFields=document.createElement('ul');
    comparison.append(changeHeading,changedFields);
    const views=[];
    for(const key of ['before','candidate']){
      const heading=document.createElement('h4'); bind(heading,key,{group:formLabels});
      const body=document.createElement('pre');
      comparison.append(heading,body);
      views.push(body);
    }
    let candidateLocation=null;
    function sourceJump(input,label) {
      const button=document.createElement('button'); button.type='button';
      bind(button,label,{group:formLabels});
      button.addEventListener('click',()=>{
        if(candidateLocation?.input!==input)return;
        input.focus();
        input.setSelectionRange(candidateLocation.start,candidateLocation.end);
      });
      return button;
    }
    const ddlJump=sourceJump(ddl,'initialDiagnosticJump');
    const consumerJump=sourceJump(consumer,'consumerSetupJump');
    form.append(review,ddlJump,consumerJump,comparison,update);
    const staticLabelCount=formLabels.length;
    let reviewed=null;
    let editGeneration=0;
    function invalidateReview() {
      reviewed=null;
      candidateLocation=null;
      ddlJump.hidden=consumerJump.hidden=true;
      update.dataset.reviewPending='true';
      update.disabled=true;
      comparison.hidden=true;
      formLabels.length=staticLabelCount;
      changedFields.replaceChildren();
      for(const body of views)body.textContent='';
    }
    invalidateReview();
    for(const input of form.querySelectorAll('input,textarea')){
      input.addEventListener('input',()=>{
        editGeneration++;
        if(reviewed||candidateLocation)invalidateReview();
        setStatus('reviewRequired');
      });
    }
    function proposal() {
      return {
        project_hex:projectHex,expected_revision_hex:reply.revision_hex,edition:edition.value,
        surfaces:surfaces.map(row=>({public_name:row.name,surface:row.surface.value,
          pins:row.pins.map(([name,input,particle])=>[name,input.value,particle])})),
        ddl_source:{path:data.ddl_source.path,text:editedText(ddl,data.ddl_source.text)},
        consumer_source_utf8:editedText(consumer,data.consumer_source_utf8),
      };
    }
    function renderChangedFields(candidate) {
      function add(key,...args) {
        const row=document.createElement('li');
        bind(row,key,{args,group:formLabels});
        changedFields.append(row);
      }
      add('reviewVersion',data.edition,candidate.edition);
      for(const [index,before] of data.surfaces.entries()){
        const after=candidate.surfaces[index];
        if(before.surface!==after.surface)add('reviewCallable',before.public_name,before.surface,after.surface);
        for(const [pinIndex,[name,value]] of before.pins.entries()){
          const next=after.pins[pinIndex][1];
          if(value!==next)add('reviewPin',before.public_name,name,value,next);
        }
      }
      if(data.ddl_source.text!==candidate.ddl_source.text)add('reviewDdlText');
      if(data.consumer_source_utf8!==candidate.consumer_source_utf8)add('reviewConsumerText');
    }
    const currentEdition=()=>recorded?.revision===reply.revision_hex&&editionSelection.value==='current';
    review.addEventListener('click',()=>void perform(async()=>{
      if(!currentEdition())return;
      invalidateReview();
      setStatus('reviewRequired');
      const candidate=proposal();
      const generation=editGeneration;
      const stillCurrent=()=>!disposed&&currentEdition()&&generation===editGeneration
        &&JSON.stringify(candidate)===JSON.stringify(proposal());
      let result;
      try {
        result=await request('review_public_callable_edition',{candidate});
      } catch(error) {
        if(!stillCurrent())return;
        const diagnostic=error?.diagnostic;
        if(!diagnostic?.source_span)throw error;
        const isDdl=diagnostic.source_path===candidate.ddl_source.path&&diagnostic.phase==='lang_parse'
          &&['E_MULTILANG_FULLWIDTH_SYNTAX','E_MULTILANG_UNREGISTERED_SURFACE','E_DDL_PUBLIC_PIN_OWNER'].includes(diagnostic.code);
        const isConsumer=diagnostic.source_path===data.consumer_path
          &&(['lang_parse','project_prepare'].includes(diagnostic.phase))&&(
            ['E_DDL_ACTIVATION_PARSE','E_DDL_ACTIVATION_OWNER_OR_PROFILE','E_DDL_ORIGINAL_MODULE_MISMATCH',
              'E_MULTILANG_SUCCESSOR_HEADER_SCOPE'].includes(diagnostic.code)
            ||Object.hasOwn(ddlEditionLabels[locale].consumerDiagnosticGuidance,diagnostic.code));
        if(!isDdl&&!isConsumer||isDdl&&isConsumer)throw error;
        const text=isDdl?candidate.ddl_source.text:candidate.consumer_source_utf8;
        const selection=textareaSelectionForUtf8Span(text,{...diagnostic.source_span,encoding:'utf-8-byte'});
        if(!selection)throw error;
        candidateLocation={...selection,input:isDdl?ddl:consumer};
        (isDdl?ddlJump:consumerJump).hidden=false;
        statusMessage={editionDiagnostic:{diagnostic,consumer:isConsumer}};
        renderStatus();
        return;
      }
      if(!stillCurrent())return;
      if(result.kind!=='public_callable_edition_review'||result.project_hex!==projectHex
          ||result.revision_hex!==reply.revision_hex||!/^[a-f0-9]{64}$/u.test(result.review_hash))
        throw new Error('E_DDL_EDITION_READBACK');
      reviewed={candidate,hash:result.review_hash};
      renderChangedFields(candidate);
      views[0].textContent=comparisonText(data,data.consumer_path);
      views[1].textContent=comparisonText(candidate,data.consumer_path);
      comparison.hidden=false;
      update.dataset.reviewPending='false';
      setStatus('reviewed');
    }));
    update.addEventListener('click',()=>void perform(async()=>{
      if(!currentEdition()||!reviewed)return;
      if(JSON.stringify(proposal())!==JSON.stringify(reviewed.candidate)){
        invalidateReview();
        setStatus('reviewRequired');
        return;
      }
      const {candidate,hash}=reviewed;
      invalidateReview();
      const changed=await request('update_public_callable_edition',{candidate,review_hash:hash});
      if(!disposed)renderEdition(projectHex,'current',changed);
    }));
  }
  async function readEdition(projectHex,selection) {
    const reply=await request('read_public_callable_edition',{project_hex:projectHex,selection});
    if(!disposed)renderEdition(projectHex,selection,reply);
  }
  compareStored.addEventListener('click',()=>void perform(async()=>{
    const opened=recorded;
    if(!opened?.hasPrevious||opened.selection!==editionSelection.value)return;
    clearStoredComparison();
    const {projectHex}=opened;
    const previous=await request('read_public_callable_edition',{project_hex:projectHex,selection:'previous'});
    const current=await request('read_public_callable_edition',{project_hex:projectHex,selection:'current'});
    if(disposed)return;
    const before=editionData(projectHex,previous);
    const after=editionData(projectHex,current);
    const selected=opened.selection==='previous'?previous:current;
    if(!previous.has_previous||!current.has_previous||previous.revision_hex===current.revision_hex
        ||selected.revision_hex!==opened.revision||recorded!==opened||editionSelection.value!==opened.selection)
      throw new Error('E_DDL_EDITION_READBACK');
    const texts=[comparisonText(before,before.consumer_path),comparisonText(after,after.consumer_path)];
    for(const [index,body] of storedViews.entries())body.textContent=texts[index];
    storedComparison.hidden=false;
  }));
  function invalidateOpenedEdition() {
    if(recorded?.nativeReadback)recorded.stale=true;
    source.hidden=true; source.textContent='';
    editEdition.open=false;
    clearStoredComparison();
    output.hidden=true; output.textContent='';
    clearOriginalComparison();
  }
  editionSelection.addEventListener('change',()=>{
    invalidateOpenedEdition();
    setStatus('selectionPending');
    setControls();
  });
  openEdition.addEventListener('click',()=>void perform(async()=>{
    const projectHex=recorded?.projectHex;
    if(projectHex)await readEdition(projectHex,editionSelection.value);
  }));
  function folderFiles(files){
    if(!files.length)return new Map();
    const top=files[0].webkitRelativePath.split('/')[0]+'/';
    const byPath=new Map();
    for(const file of files){
      const relative=file.webkitRelativePath;
      if(!relative.startsWith(top)||relative===top)throw new Error('E_DDL_ORIGINAL_FOLDER_PATH');
      const path=relative.slice(top.length);
      if(byPath.has(path))throw new Error('E_DDL_ORIGINAL_FOLDER_PATH');
      byPath.set(path,file);
    }
    return byPath;
  }
  async function folderCreateRequest(selected,source_profile,root_path,entry_path) {
    const size=[...selected.values()].reduce((total,file)=>total+file.size,0);
    if(selected.size>MAX_DDL_FOLDER_FILES||size>1_048_576)throw new Error('E_SOURCE_SAVE_REQUEST_LIMIT');
    const decoder=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true});
    const files=[];
    for(const [path,file] of selected)files.push({path,text:decoder.decode(await file.arrayBuffer())});
    return JSON.stringify({source_profile,module_name:'',source_snapshot:{root_path,entry_path,files}});
  }
  const initialAuthoring=document.createElement('details');
  const initialTitle=document.createElement('summary'); bind(initialTitle,'initialTitle');
  const initialHint=document.createElement('p'); bind(initialHint,'initialHint');
  const initialFolder=document.createElement('input');
  initialFolder.type='file'; initialFolder.setAttribute('webkitdirectory',''); initialFolder.multiple=true;
  bind(initialFolder,'initialFolder',{attribute:'aria-label'});
  function initialSelect(key) {
    const label=document.createElement('label');
    const caption=document.createElement('span'); bind(caption,key);
    const select=document.createElement('select');
    label.append(caption,' ',select);
    return [label,select];
  }
  const [profileLabel,initialProfile]=initialSelect('initialProfile');
  for(const profile of ['v1-core-v25',J1_PROFILE]){
    const option=document.createElement('option'); option.value=profile; option.textContent=profile; initialProfile.append(option);
  }
  const [rootLabel,initialRoot]=initialSelect('initialRoot');
  const [entryLabel,initialEntry]=initialSelect('initialEntry');
  const [moduleLabel,initialModule]=initialSelect('initialModule');
  const inspect=document.createElement('button'); inspect.type='button'; bind(inspect,'initialInspect');
  const selectedOriginalFolder=document.createElement('p');
  selectedOriginalFolder.hidden=true;
  selectedOriginalFolder.setAttribute('role','status');
  const initialStatus=document.createElement('p'); initialStatus.setAttribute('role','status');
  const initialJump=document.createElement('button');
  initialJump.type='button'; initialJump.hidden=true;
  bind(initialJump,'initialScopeJump');
  const initialSource=document.createElement('textarea');
  initialSource.readOnly=true; initialSource.hidden=true; initialSource.rows=5;
  bind(initialSource,'initialScopeSource',{attribute:'aria-label'});
  initialJump.addEventListener('click',()=>{
    const location=initialInspection?.location;
    if(!location)return;
    initialSource.value=location.text;
    initialSource.hidden=false;
    initialSource.focus();
    initialSource.setSelectionRange(location.start,location.end);
  });
  const initialRows=document.createElement('ul');
  initialAuthoring.append(initialTitle,initialHint,initialFolder,selectedOriginalFolder,profileLabel,rootLabel,entryLabel,moduleLabel,
    inspect,initialStatus,initialJump,initialSource,initialRows);
  let initialFiles=new Map(),initialInspection=null,originalFolderName='';
  const pathSelectors=[initialRoot,initialEntry,initialModule];
  const initialPlaceholders=[];
  function renderInitialInspection(){
    const labels=ddlEditionLabels[locale];
    selectedOriginalFolder.hidden=!originalFolderName;
    selectedOriginalFolder.textContent=originalFolderName?labels.initialSelectedFolder(originalFolderName,initialFiles.size):'';
    for(const placeholder of initialPlaceholders)placeholder.textContent=labels.initialChoose;
    initialRows.replaceChildren();
    initialStatus.textContent='';
    initialJump.hidden=!initialInspection?.location;
    if(initialJump.hidden){
      initialSource.hidden=true;
      initialSource.value='';
    }
    if(initialInspection===null)return;
    if(initialInspection.error){
      const diagnostic=initialInspection.diagnostic;
      const detail=diagnostic?.code==='E_MULTILANG_FULLWIDTH_SYNTAX'&&typeof diagnostic.message==='string'
        ?diagnostic.message:initialInspection.error;
      initialStatus.textContent=labels.initialError+'\n'+detail;
      if(initialInspection.error.startsWith('E_RESOLVED_SUPPLY: E_DDL_PUBLIC_BODY_SCOPE:'))
        initialStatus.textContent+='\n'+labels.initialBodyScopeGuidance;
      else if(['E_J1_UNREGISTERED_SURFACE','E_MULTILANG_FULLWIDTH_SYNTAX'].includes(initialInspection.diagnostic?.code))
        initialStatus.textContent+='\n'+diagnosticPresentation(initialInspection.diagnostic,locale).guidance;
      const location=initialInspection.location;
      if(location)initialStatus.textContent+='\n'+location.path+' · UTF-8 '+location.byteStart+'–'+location.byteEnd;
      return;
    }
    initialStatus.textContent=labels.initialFound(initialInspection.module,initialInspection.definitions.length);
    for(const row of initialInspection.definitions){
      const item=document.createElement('li');
      item.textContent=row.public_name+' → '+row.internal_name+' ('+row.pins.join(', ')+') — '+row.source_path;
      initialRows.append(item);
    }
  }
  function resetInitialPaths() {
    initialPlaceholders.length=0;
    for(const select of pathSelectors){
      const placeholder=document.createElement('option'); placeholder.value='';
      initialPlaceholders.push(placeholder); select.replaceChildren(placeholder);
      for(const path of [...initialFiles.keys()].sort()){
        const option=document.createElement('option'); option.value=path; option.textContent=path; select.append(option);
      }
    }
    renderInitialInspection();
  }
  resetInitialPaths();
  const mapping=document.createElement('fieldset');
  mapping.hidden=true;
  const mappingTitle=document.createElement('legend'); bind(mappingTitle,'mappingTitle');
  const mappingHint=document.createElement('p'); bind(mappingHint,'mappingHint');
  const mappingFields=document.createElement('div');
  const prepare=document.createElement('button'); prepare.type='button'; bind(prepare,'initialPrepare');
  const mappingStatus=document.createElement('p'); mappingStatus.setAttribute('role','status');
  const mappingJump=document.createElement('button');
  mappingJump.type='button'; mappingJump.hidden=true; bind(mappingJump,'initialDiagnosticJump');
  mappingJump.addEventListener('click',()=>{
    const selection=mappingMessage?.selection;
    if(!mappingInput||!selection)return;
    mappingInput.ddl.focus();
    mappingInput.ddl.setSelectionRange(selection.start,selection.end);
  });
  mapping.append(mappingTitle,mappingHint,mappingFields,prepare,mappingStatus,mappingJump);
  initialAuthoring.append(mapping);
  const mappingLabels=[];
  let mappingInput=null,preparedOriginal=null,mappingMessage=null;
  let originalRevision=0;
  function renderMappingStatus(){
    const labels=ddlEditionLabels[locale];
    const error=mappingMessage?.error;
    mappingJump.hidden=!mappingMessage?.selection;
    const guidance=error==='파서 오류: E_MULTILANG_UNREGISTERED_SURFACE'
      ?'\n'+labels.initialUnregisteredSurfaceGuidance
      :mappingMessage?.presentationDiagnostic?'\n'+diagnosticPresentation(mappingMessage.presentationDiagnostic,locale).guidance:'';
    mappingStatus.textContent=error
      ?labels.initialPrepareError+'\n'+error+guidance
      :mappingMessage?.key?labels[mappingMessage.key]:'';
  }
  function resetMapping() {
    originalRevision++;
    resetConsumer();
    mapping.hidden=true;
    mappingFields.replaceChildren();
    mappingLabels.length=0;
    mappingInput=preparedOriginal=mappingMessage=null;
    renderMappingStatus();
  }
  function mappingField(key,args=[],multiline=false) {
    const label=document.createElement('label');
    const caption=document.createElement('span'); bind(caption,key,{args,group:mappingLabels});
    const input=document.createElement(multiline?'textarea':'input');
    if(multiline){input.rows=6;input.cols=42;}
    label.append(caption,' ',input);
    const line=document.createElement('div'); line.append(label); mappingFields.append(line);
    input.addEventListener('input',()=>{
      originalRevision++;
      if(preparedOriginal)mappingMessage={key:'initialPreparationStale'};
      else if(mappingMessage?.error)mappingMessage=null;
      if(preparedOriginal)resetConsumer();
      preparedOriginal=null;
      renderMappingStatus();
      setControls();
    });
    return input;
  }
  function openMapping(inspection) {
    resetMapping();
    const path=mappingField('initialDdlPath');
    const ddl=mappingField('initialDdlText',[],true);
    const surfaces=inspection.definitions.map(row=>({
      publicName:row.public_name, surface:mappingField('callable',[row.public_name]),
      pins:row.pins.map(pin=>({
        name:pin, surface:mappingField('pin',[row.public_name,pin]),
        particle:mappingField('initialParticle',[row.public_name,pin]),
      })),
    }));
    mappingInput={inspection,path,ddl,surfaces};
    mapping.hidden=false;
  }
  prepare.addEventListener('click',()=>void perform(async()=>{
    if(!mappingInput||preparedOriginal)return;
    const revision=originalRevision;
    const current=()=>!disposed&&originalRevision===revision;
    const {inspection,path,ddl,surfaces}=mappingInput;
    const ddlSource={path:path.value,text:ddl.value};
    mappingMessage=null;
    renderMappingStatus();
    try {
      const reply=await request('prepare_public_callable_original',{
        original_project_json_utf8:inspection.original_project_json_utf8,
        original_source_path:inspection.source_path,
        ddl_source:ddlSource,
        surfaces:surfaces.map(row=>({public_name:row.publicName,surface:row.surface.value,
          pins:row.pins.map(pin=>[pin.name,pin.surface.value,pin.particle.value])})),
      });
      if(!current())return;
      if(reply.kind!=='public_callable_original_prepared'||typeof reply.original_record_json_utf8!=='string')
        throw new Error('E_DDL_ORIGINAL_INSPECTION');
      preparedOriginal=reply.original_record_json_utf8;
      mappingMessage={key:'initialPrepared'};
    } catch(error) {
      if(!current())return;
      const diagnostic=error?.diagnostic;
      const selection=diagnostic?.phase==='lang_parse'
        &&['E_MULTILANG_UNREGISTERED_SURFACE','E_MULTILANG_FULLWIDTH_SYNTAX','E_DDL_PUBLIC_PIN_OWNER'].includes(diagnostic.code)
        &&diagnostic.source_path===ddlSource.path&&diagnostic.source_span
        ?textareaSelectionForUtf8Span(ddlSource.text,{...diagnostic.source_span,encoding:'utf-8-byte'}):null;
      const message=String(error?.message??error);
      const code=diagnostic?.code??message.replace(/^파서 오류: /u,'');
      const registrationMismatch=code==='E_DDL_PUBLIC_COVERAGE'||code==='E_DDL_PUBLIC_PIN_OWNER';
      const presentationDiagnostic=registrationMismatch?(diagnostic??{code})
        :selection&&code==='E_MULTILANG_FULLWIDTH_SYNTAX'?diagnostic:null;
      mappingMessage={error:message,selection,presentationDiagnostic};
    }
    renderMappingStatus();
  }));
  const consumerSetup=document.createElement('fieldset');
  const consumerTitle=document.createElement('legend'); bind(consumerTitle,'consumerSetupTitle');
  const consumerHint=document.createElement('p'); bind(consumerHint,'consumerSetupHint');
  const consumerFolder=document.createElement('input');
  consumerFolder.type='file'; consumerFolder.setAttribute('webkitdirectory',''); consumerFolder.multiple=true;
  bind(consumerFolder,'consumerSetupFolder',{attribute:'aria-label'});
  const [consumerProfileLabel,consumerProfile]=initialSelect('consumerSetupProfile');
  for(const profile of ['v1-core-v25',J1_PROFILE]){
    const option=document.createElement('option'); option.value=profile; option.textContent=profile; consumerProfile.append(option);
  }
  const [consumerRootLabel,consumerRoot]=initialSelect('consumerSetupRoot');
  const [consumerEntryLabel,consumerEntry]=initialSelect('consumerSetupEntry');
  const [consumerPathLabel,consumerPath]=initialSelect('consumerSetupPath');
  const [logicPathLabel,logicPath]=initialSelect('consumerSetupLogicPath');
  const runJoint=document.createElement('button'); runJoint.type='button'; bind(runJoint,'consumerSetupJointRun');
  const jointOutput=document.createElement('pre'); jointOutput.hidden=true;
  const checkConsumer=document.createElement('button'); checkConsumer.type='button'; bind(checkConsumer,'consumerSetupCheck');
  const consumerStatus=document.createElement('p'); consumerStatus.setAttribute('role','status');
  const consumerJump=document.createElement('button'); consumerJump.type='button'; consumerJump.hidden=true;
  bind(consumerJump,'consumerSetupJump');
  const consumerSource=document.createElement('textarea'); consumerSource.readOnly=true;
  consumerSource.rows=5; consumerSource.hidden=true;
  bind(consumerSource,'consumerSetupDiagnosticSource',{attribute:'aria-label'});
  consumerSetup.append(consumerTitle,consumerHint,consumerFolder,consumerProfileLabel,consumerRootLabel,
    consumerEntryLabel,consumerPathLabel,logicPathLabel,checkConsumer,runJoint,consumerStatus,consumerJump,consumerSource,jointOutput);
  initialAuthoring.append(consumerSetup);
  const consumerSelectors=[consumerRoot,consumerEntry,consumerPath];
  let consumerFiles=new Map(),consumerMessage=null,consumerStaged=null,consumerRevision=0,jointProject=null;
  consumerJump.addEventListener('click',()=>{
    const source=consumerMessage?.source;
    if(!source)return;
    consumerSource.value=source.text;
    consumerSource.hidden=false;
    consumerSource.focus();
    consumerSource.setSelectionRange(source.selection.start,source.selection.end);
  });
  const consumerPlaceholders=[];
  function renderConsumerStatus() {
    const labels=ddlEditionLabels[locale];
    for(const placeholder of consumerPlaceholders)placeholder.textContent=labels.initialChoose;
    jointOutput.textContent=consumerMessage?.output?.join('\n')??'';
    jointOutput.hidden=consumerMessage?.output===undefined;
    consumerJump.hidden=!consumerMessage?.source;
    if(consumerJump.hidden){
      consumerSource.hidden=true;
      consumerSource.value='';
    }
    if(consumerMessage?.error){
      const diagnostic=consumerMessage.diagnostic;
      const message=consumerMessage.error;
      const hasCode=diagnostic?.code&&(message===diagnostic.code||message.startsWith(diagnostic.code+':'));
      const location=diagnostic?.source_path
        ?(hasCode?'':diagnostic.code+' | ')+diagnostic.source_path+(diagnostic.source_span
          ?' · UTF-8 '+diagnostic.source_span.start+'–'+diagnostic.source_span.end:''):'';
      const logicSelection=consumerMessage.logicSelection
        ?labels.consumerSetupLogicSelection+' '+consumerMessage.logicSelection:'';
      // The activation wrapper retains the parser's byte span. Use the
      // selected character only to choose a hint, leaving the diagnostic intact.
      const selected=consumerMessage.source;
      const fullwidthLetter=diagnostic?.code==='E_DDL_ACTIVATION_PARSE'
        && selected && /^[Ａ-Ｚａ-ｚ]$/u.test(selected.text.replace(/\r\n?/gu,'\n').slice(selected.selection.start,selected.selection.end));
      const guidance=fullwidthLetter
        ?diagnosticPresentation({code:'E_MULTILANG_FULLWIDTH_SYNTAX'},locale).guidance
        :Object.hasOwn(labels.consumerDiagnosticGuidance,diagnostic?.code)
        ?labels.consumerDiagnosticGuidance[diagnostic.code]
        :consumerMessage.source&&['E_MULTILANG_FULLWIDTH_SYNTAX','E_J1_UNREGISTERED_SURFACE'].includes(diagnostic?.code)
          ?diagnosticPresentation(diagnostic,locale).guidance:null;
      consumerStatus.textContent=[diagnostic?.phase==='runtime_execution'
        ?labels.consumerSetupExecutionRejected:labels.consumerSetupRejected,consumerMessage.error,location,logicSelection,consumerMessage.logicSelection?labels.consumerSetupLogicGuidance:null,guidance].filter(Boolean).join('\n');
    } else {
      consumerStatus.textContent=consumerMessage?.key?labels[consumerMessage.key]:'';
    }
  }
  function invalidateConsumer() {
    consumerRevision++;
    if(consumerStaged!==null&&recorded?.artifactText===consumerStaged){
      clear();
      setStatus('stagedInvalidated');
    }
    consumerMessage=consumerStaged=jointProject=null;
    renderConsumerStatus();
  }
  function resetConsumerPaths() {
    consumerPlaceholders.length=0;
    for(const select of [...consumerSelectors,logicPath]){
      const placeholder=document.createElement('option'); placeholder.value='';
      consumerPlaceholders.push(placeholder); select.replaceChildren(placeholder);
      for(const path of [...consumerFiles.keys()].sort()){
        const option=document.createElement('option'); option.value=path; option.textContent=path; select.append(option);
      }
    }
    renderConsumerStatus();
  }
  function resetConsumer() {
    invalidateConsumer();
    consumerFiles.clear();
    resetConsumerPaths();
  }
  consumerFolder.addEventListener('change',()=>{
    invalidateConsumer();
    try {consumerFiles=folderFiles(Array.from(consumerFolder.files));}
    catch(error){consumerFiles=new Map(); consumerMessage={error:String(error?.message??error)};}
    consumerFolder.value='';
    resetConsumerPaths();
    setControls();
  });
  for(const select of [consumerProfile,...consumerSelectors,logicPath]){
    select.addEventListener('change',()=>{invalidateConsumer(); setControls();});
  }
  runJoint.addEventListener('click',()=>void perform(async()=>{
    const original=preparedOriginal, userPath=consumerPath.value, builtinPath=logicPath.value;
    if(!original||!userPath||!builtinPath||userPath===builtinPath)return;
    const revision=consumerRevision;
    const current=()=>!disposed&&consumerRevision===revision;
    consumerMessage=null;
    renderConsumerStatus();
    let sourceFiles=jointProject?.files;
    try {
      if(!jointProject){
        const create=await folderCreateRequest(consumerFiles,consumerProfile.value,consumerRoot.value,consumerEntry.value);
        if(!current())return;
        sourceFiles=JSON.parse(create).source_snapshot.files;
        const text=await operate('create',create);
        if(!current())return;
        jointProject={text,files:sourceFiles};
      }
      const reply=JSON.parse(await operate('linked_callable_source_run',JSON.stringify({
        original_record_json_utf8:original,
        consumer_project_json_utf8:jointProject.text,
        consumer_source_path:userPath,
        logic_consumer_source_path:builtinPath,
      })));
      if(!current())return;
      if(reply.source_only!==true||!Array.isArray(reply.output_log_texts)
          ||reply.output_log_texts.some(line=>typeof line!=='string')){
        throw new Error('E_DDL_CONSUMER_EXECUTION');
      }
      consumerMessage={key:'consumerSetupJointDone',output:reply.output_log_texts};
    } catch(error) {
      if(!current())return;
      const diagnostic=error?.diagnostic;
      const file=sourceFiles?.find(file=>file.path===diagnostic?.source_path);
      const selection=file&&diagnostic.source_span
        ?textareaSelectionForUtf8Span(file.text,{encoding:'utf-8-byte',...diagnostic.source_span}):null;
      const logicSelection=jointProject&&error?.code==='E_LOGIC_DEPENDENCY'&&!diagnostic?.source_path
        &&sourceFiles?.some(file=>file.path===builtinPath)?builtinPath:null;
      consumerMessage={error:String(error?.diagnosticMessage??error?.message??error),diagnostic,logicSelection,source:selection?{text:file.text,selection}:null};
    }
    if(current())renderConsumerStatus();
  }));
  checkConsumer.addEventListener('click',()=>void perform(async()=>{
    if(!preparedOriginal)return;
    invalidateConsumer();
    const revision=consumerRevision;
    const original=preparedOriginal, path=consumerPath.value;
    const current=()=>!disposed&&consumerRevision===revision;
    try {
      const create=await folderCreateRequest(consumerFiles,consumerProfile.value,consumerRoot.value,consumerEntry.value);
      if(!current())return;
      const reply=await request('prepare_public_callable_consumer',{
        original_record_json_utf8:original,create_request_utf8:create,consumer_source_path:path,
      });
      if(!current())return;
      if(reply.kind==='public_callable_consumer_rejected'){
        const diagnostic=reply.diagnostic;
        const file=JSON.parse(create).source_snapshot.files.find(file=>file.path===diagnostic?.source_path);
        const selection=file&&diagnostic.source_span
          ?textareaSelectionForUtf8Span(file.text,{encoding:'utf-8-byte',...diagnostic.source_span}):null;
        consumerMessage={error:reply.message,diagnostic,source:selection?{text:file.text,selection}:null};
      }else{
        if(reply.kind!=='public_callable_consumer_prepared'||typeof reply.source_artifact_utf8!=='string')
          throw new Error('E_DDL_CONSUMER_PREPARATION');
        await openArtifact(reply.source_artifact_utf8,null,current);
        if(!current())return;
        consumerStaged=reply.source_artifact_utf8;
        consumerMessage={key:'consumerSetupPrepared'};
      }
    }catch(error){
      if(!current())return;
      consumerMessage={error:String(error?.message??error)};
    }
    if(current())renderConsumerStatus();
  }));
  initialFolder.addEventListener('change',()=>{
    resetMapping();
    initialInspection=null;
    originalFolderName='';
    try {
      const files=Array.from(initialFolder.files);
      initialFiles=folderFiles(files);
      if(initialFiles.size)originalFolderName=files[0].webkitRelativePath.split('/')[0];
    }
    catch(error){initialFiles=new Map(); initialInspection={error:String(error?.message??error)};}
    initialFolder.value='';
    resetInitialPaths();
    setControls();
  });
  for(const select of [initialProfile,...pathSelectors]){
    select.addEventListener('change',()=>{resetMapping();initialInspection=null;renderInitialInspection();setControls();});
  }
  inspect.addEventListener('click',()=>void perform(async()=>{
    resetMapping();
    initialInspection=null;
    renderInitialInspection();
    const revision=originalRevision, sourcePath=initialModule.value;
    const current=()=>!disposed&&originalRevision===revision;
    let inspectedFiles=[];
    try {
      const create=await folderCreateRequest(initialFiles,initialProfile.value,initialRoot.value,initialEntry.value);
      if(!current())return;
      inspectedFiles=JSON.parse(create).source_snapshot.files;
      const reply=await request('inspect_public_callable_original',{
        create_request_utf8:create,
        original_source_path:sourcePath,
      });
      if(!current())return;
      if(reply.kind!=='public_callable_original_inspection'||reply.source_path!==sourcePath
          ||typeof reply.module!=='string'||!Array.isArray(reply.definitions)
          ||typeof reply.original_project_json_utf8!=='string')throw new Error('E_DDL_ORIGINAL_INSPECTION');
      initialInspection=reply;
      openMapping(reply);
    } catch(error) {
      if(!current())return;
      const diagnostic=error?.diagnostic;
      const span=diagnostic?.source_span;
      const file=inspectedFiles.find(row=>row.path===diagnostic?.source_path);
      const selection=file&&span?textareaSelectionForUtf8Span(file.text,{...span,encoding:'utf-8-byte'}):null;
      const location=selection?{...selection,path:file.path,text:file.text,byteStart:span.start,byteEnd:span.end}:null;
      initialInspection={error:String(error?.message??error),diagnostic,location};
    }
    renderInitialInspection();
  }));
  section.append(workflowHint,initialAuthoring,file,save,refresh,projects,reopen,exportSaved,status,source,editionControls,run,runtimeJump,runtimeSource,output);
  root.append(section);

  let recorded=null,disposed=false,busy=false;
  const setControls=()=>{
    file.disabled=refresh.disabled=projects.disabled=busy;
    consumerSetup.hidden=!preparedOriginal;
    consumerFolder.disabled=consumerProfile.disabled=busy;
    for(const select of [...consumerSelectors,logicPath])select.disabled=busy||consumerFiles.size===0;
    const ready=Boolean(preparedOriginal)&&consumerSelectors.every(select=>select.value);
    checkConsumer.disabled=busy||!ready||consumerStaged!==null||Boolean(logicPath.value);
    runJoint.disabled=busy||!ready||!logicPath.value||logicPath.value===consumerPath.value;
    initialFolder.disabled=initialProfile.disabled=busy;
    for(const select of pathSelectors)select.disabled=busy||initialFiles.size===0;
    inspect.disabled=busy||pathSelectors.some(select=>!select.value);
    const inputs=[...mappingFields.querySelectorAll('input,textarea')];
    for(const input of inputs)input.disabled=busy;
    prepare.disabled=busy||!mappingInput||Boolean(preparedOriginal)||inputs.some(input=>!input.value.trim());
    const unresolvedSelection=recorded?.nativeReadback&&(recorded.stale||editionSelection.value!==recorded.selection);
    run.disabled=busy||recorded===null||unresolvedSelection;
    save.disabled=busy||!recorded?.artifactText||recorded.nativeReadback;
    openEdition.disabled=editionSelection.disabled=busy;
    compareStored.hidden=!recorded?.hasPrevious;
    compareStored.disabled=busy||unresolvedSelection;
    originalFolder.disabled=busy||!recorded?.originalFiles||unresolvedSelection;
    openNewDraft.disabled=originalFolder.disabled;
    for(const control of form.querySelectorAll('input,textarea,button'))control.disabled=busy||unresolvedSelection||control.dataset.reviewPending==='true';
    reopen.disabled=busy||projects.value==='';
    exportSaved.disabled=busy||!recorded?.nativeReadback||recorded.hasPrevious||unresolvedSelection;
  };
  setControls();
  projects.addEventListener('change',()=>{
    if(recorded?.nativeReadback&&projects.value!==recorded.projectHex){
      clear();
      setStatus('projectSelectionPending');
    }
    setControls();
  });
  async function perform(action) {
    if(busy||disposed)return;
    busy=true; setControls();
    try {await action();}
    catch(error) {
      if(!disposed){
        if(recorded?.nativeReadback&&['E_DDL_EDITION_PARENT_MISMATCH',
          'E_DDL_EDITION_REVISION_SELECTION'].includes(error?.message))
          invalidateOpenedEdition();
        clearRuntimeLocation();
        statusMessage={raw:String(error?.message??error)};
        renderStatus();
      }
    }
    finally {busy=false;if(!disposed)setControls();}
  }
  async function request(operation,fields={}) {
    if(!await nativeSourceAvailable())throw new Error('E_MULTILANG_NATIVE_REQUIRED');
    // These private source-only operations carry a sealed artifact or a saved
    // project selector. They do not send ordinary authored-project JSON.
    return nativeSourceRequest(operation,undefined,fields);
  }
  function clear() {
    clearRuntimeLocation();
    recorded=null;
    clearStoredComparison();
    clearOriginalComparison();
    editionControls.hidden=true; editEdition.open=false;
    formLabels.length=optionLabels.length=0; form.replaceChildren(); editionSelection.replaceChildren();
    source.hidden=true; source.textContent='';
    output.hidden=true; output.textContent='';
  }
  async function openArtifact(artifactText,expectedProject=null,isCurrent) {
    let envelope;
    try {
      envelope=JSON.parse(artifactText);
    } catch(error) {
      if(expectedProject!==null||!(error instanceof SyntaxError))throw error;
      throw new Error('E_DDL_ARTIFACT_UNREADABLE');
    }
    if(envelope?.schema!=='ddn.source_only_project_artifact.v1')
      throw new Error('E_DDL_V4_SOURCE_ARTIFACT_REQUIRED');
    const checkedText=await operate('source_open',JSON.stringify({
      artifact_text:artifactText,project_hex:envelope.project_hex,
      parent_hex:envelope.parent_hex,revision_hex:envelope.revision_hex,
    }));
    if(disposed||isCurrent?.()===false)return;
    const checked=JSON.parse(checkedText);
    if(checked.schema!=='ddn.authored_project_artifact.v4')
      throw new Error('E_DDL_V4_SOURCE_ARTIFACT_REQUIRED');
    const projectHex=checked.authored_identity.logical_dependency_id;
    if(expectedProject!==null&&expectedProject!==projectHex)throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
    const path=checked.public_callable_binding.consumer_source_path;
    const entry=checked.source_snapshot.files.find(row=>row.path===path);
    if(!entry)throw new Error('E_DDL_CONSUMER_SOURCE_REQUIRED');
    clear();
    recorded={artifactText,projectHex,consumer:entry,nativeReadback:expectedProject!==null};
    source.textContent=path+'\n'+entry.text; source.hidden=false;
    setStatus('imported');
  }
  let renderProjectPlaceholder;
  async function refreshProjects(selected=projects.value) {
    const reply=await request('list_public_callable_sources');
    if(disposed)return;
    if(reply.kind!=='public_callable_source_list'||!Array.isArray(reply.projects)
        ||!reply.projects.every(id=>typeof id==='string'&&/^[0-9a-f]{64}$/u.test(id)))
      throw new Error('E_NATIVE_SOURCE_LOCATOR_FIELDS');
    projects.replaceChildren();
    const placeholder=document.createElement('option');
    placeholder.value=''; renderProjectPlaceholder=()=>{placeholder.textContent=ddlEditionLabels[locale].projectPlaceholder;};
    renderProjectPlaceholder();
    projects.append(placeholder);
    for(const id of reply.projects){
      const option=document.createElement('option');
      option.value=id; option.textContent=id;
      projects.append(option);
    }
    projects.value=reply.projects.includes(selected)?selected:'';
    if(recorded?.nativeReadback&&projects.value!==recorded.projectHex)clear();
  }
  file.addEventListener('change',()=>{
    const selected=file.files[0];
    if(!selected)return;
    void perform(async()=>{
      clear();
      setStatus('checking');
      try {
        if(selected.size>8_000_000)throw new Error('E_SOURCE_ARTIFACT_LIMIT');
        const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(await selected.arrayBuffer());
        // The CLI appends one line terminator to its stdout record. Keep
        // all other bytes for the existing canonical artifact check.
        const artifactText=text.endsWith('\r\n')?text.slice(0,-2):text.endsWith('\n')?text.slice(0,-1):text;
        await openArtifact(artifactText);
      } finally {if(!disposed)file.value='';}
    });
  });
  refresh.addEventListener('click',()=>void perform(async()=>{
    await refreshProjects();
    if(!disposed)setStatus('listed');
  }));
  save.addEventListener('click',()=>void perform(async()=>{
    if(recorded===null)return;
    const {artifactText,projectHex}=recorded;
    const reply=await request('save_public_callable_source',{source_artifact_utf8:artifactText});
    if(disposed)return;
    if(reply.kind!=='public_callable_source_saved'||reply.project_hex!==projectHex)
      throw new Error('E_NATIVE_SOURCE_SAVE_READBACK');
    setStatus('saved');
  }));
  reopen.addEventListener('click',()=>void perform(async()=>{
    const projectHex=projects.value;
    if(!projectHex)return;
    clear();
    await readEdition(projectHex,'current');
  }));
  exportSaved.addEventListener('click',()=>void perform(async()=>{
    if(!recorded?.nativeReadback||recorded.hasPrevious)return;
    const projectHex=recorded.projectHex;
    const reply=await request('load_public_callable_source',{project_hex:projectHex});
    if(disposed)return;
    if(reply.kind!=='public_callable_source_current'||reply.project_hex!==projectHex
        ||typeof reply.source_artifact_utf8!=='string')throw new Error('E_NATIVE_SOURCE_READBACK');
    const url=URL.createObjectURL(new Blob([reply.source_artifact_utf8],{type:'application/json'}));
    const link=document.createElement('a');
    link.href=url; link.download='source-artifact.json';
    section.append(link);
    link.click(); link.remove();
    setTimeout(()=>URL.revokeObjectURL(url),0);
  }));
  run.addEventListener('click',()=>void perform(async()=>{
    if(recorded===null||recorded.stale)return;
    output.hidden=true; output.textContent='';
    clearRuntimeLocation();
    let result;
    try {
      result=recorded.nativeReadback
        ?await request('run_public_callable_edition',{project_hex:recorded.projectHex,
          selection:recorded.selection,expected_revision_hex:recorded.revision})
        :JSON.parse(await operate('public_callable_source_run',JSON.stringify({source_artifact_utf8:recorded.artifactText})));
    } catch(error) {
      if(disposed)return;
      const diagnostic=error?.diagnostic;
      const consumer=recorded.consumer;
      if(diagnostic?.phase!=='runtime_execution'||diagnostic.source_path!==consumer?.path||!diagnostic.source_span)
        throw error;
      const selection=textareaSelectionForUtf8Span(consumer.text,{encoding:'utf-8-byte',...diagnostic.source_span});
      if(!selection)throw error;
      runtimeSelection=selection;
      runtimeSource.value=consumer.text;
      runtimeJump.hidden=false;
      statusMessage={runtime:diagnostic};
      renderStatus();
      return;
    }
    if(disposed)return;
    if(result.source_only!==true||!Array.isArray(result.output_log_texts)||!result.output_log_texts.every(row=>typeof row==='string'))
      throw new Error('E_MULTILANG_RESULT_OUTPUT_REQUIRED');
    output.textContent=result.output_log_texts.join('\n'); output.hidden=false;
    setStatus('result');
  }));
  const unsubscribe=controller.subscribe(state=>{
    if(locale===state.locale)return;
    locale=state.locale;
    for(const render of [...persistentLabels,...formLabels,...optionLabels])render();
    renderProjectPlaceholder?.();
    renderOriginalComparison();
    renderInitialInspection();
    for(const render of mappingLabels)render();
    renderMappingStatus();
    renderConsumerStatus();
    renderStatus();
  });
  return ()=>{disposed=true; unsubscribe(); recorded=null; section.remove();};
}
