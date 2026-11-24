import {loadRuntimeProject,commitRuntimeProject,runtimeStorageRecord,newRuntimeRevisionHex} from './runtime_project_store.js';
import {nativeSourceAvailable,nativeSourceRequest} from '../../runtime/native_source_transport.js';
// The localStorage entry is a locator only. Full source material lives in the
// same strict IndexedDB head/revision transaction and is checked by Rust.
export function createAuthoredSourceStorage(bridge) {
  const operate=(operation,value)=>bridge.authoredProjectOperation({operation,text:JSON.stringify(value)});
  function sourceRecord(record) {
    if(record?.schema!=='ddn.runtime_project.local_record.v3'||record.payload_kind!=='SOURCE_ONLY')
      throw new Error('E_SOURCE_SAVE_WORLD_PARENT_UNSUPPORTED: 기존 세계 저장을 원문 저장으로 바꾸지 않습니다.');
    return record;
  }
  const open=record=>operate('source_open',{artifact_text:sourceRecord(record).artifact_text,
    project_hex:record.project_id,parent_hex:record.parent_hex,revision_hex:record.revision_hex});
  async function receiveNativeSource(reply,artifactText,receive,unchanged) {
    if(typeof receive!=='function')return;
    let result;
    if(!Object.hasOwn(reply,'source_artifact_utf8')) {
      result={status:'supply_missing',error:'E_MULTILANG_SAVED_SOURCE_REQUIRED: 저장된 등록 자료가 공급되지 않았습니다.'};
    } else try {
      const text=reply.source_artifact_utf8;
      if(typeof text!=='string'||new TextEncoder().encode(text).byteLength>8*1024*1024)throw new Error('E_NATIVE_SOURCE_CAPTURE_FIELDS');
      const source=JSON.parse(text),project=JSON.parse(artifactText);
      if(!['ddn.source_only_project_artifact.v1','ddn.model_source_only_artifact.v1','ddn.code_source_only_artifact.v1','ddn.nominal_source_only_artifact.v1'].includes(source.schema)||source.project_hex!==project.authored_identity.logical_dependency_id
          ||source.profile!==project.source_profile)throw new Error('E_NATIVE_SOURCE_CAPTURE_CONTEXT');
      const opened=await operate('source_open',{artifact_text:text,project_hex:source.project_hex,
        parent_hex:source.parent_hex,revision_hex:source.revision_hex});
      unchanged();
      if(opened!==artifactText)throw new Error('E_NATIVE_SOURCE_CAPTURE_READBACK');
      result={status:'accepted',sourceArtifactText:text};
    } catch(error) {result={status:'rejected',error:String(error?.message??error)};}
    unchanged();
    if(reply.source_kind==='NOMINAL_SOURCE_ONLY'&&result.status!=='accepted')throw new Error(result.error??'E_NOMINAL_SOURCE_CAPTURE_REQUIRED');
    receive(result);unchanged();
  }
  return {
    async save(artifactText,unchanged,{previousArtifactText='',retainedLocator='',receiveSourceArtifact=null}={}) {
      const projectId=JSON.parse(artifactText).authored_identity.logical_dependency_id;
      unchanged();
      let requires;
      try {
        requires=await operate('source_probe',{artifact_json_utf8:artifactText});
      } catch(error) {
        // A typed rejection here precedes any storage request. An uncertain
        // failure later in the save must retain the existing pending receipt.
        const diagnostic=error?.diagnostic;
        if(diagnostic?.code==='E_J1_ROLE_AMBIGUOUS' || diagnostic?.source_span
          && (diagnostic.phase==='lang_parse' || diagnostic.code==='E_MODULE_MISSING'))error.sourceSaveProbeRejected=true;
        throw error;
      }
      unchanged();
      let recordedLocator=null;try{recordedLocator=JSON.parse(retainedLocator);}catch{}
      const nativeCodeParent=recordedLocator?.native_source_project_id===projectId
        &&recordedLocator.authored_project_json_utf8===previousArtifactText&&(recordedLocator.code_source_selection||recordedLocator.nominal_source_selection);
      // The recorded parent may no longer pass today's grammar. This locator
      // only selects the Native route; its actual grant/head/CAS are checked by
      // Native. The current candidate above always requires a fresh probe.
      const previousRequires=previousArtifactText&&previousArtifactText!==artifactText&&!nativeCodeParent
        ?await operate('source_probe',{artifact_json_utf8:previousArtifactText}):requires;unchanged();
      let linked=false;try{const locator=JSON.parse(retainedLocator);linked=(locator?.source_project_id??locator?.native_source_project_id)===projectId;}catch{}
      if(await nativeSourceAvailable()){
        const nativeLinked=(()=>{try{return JSON.parse(retainedLocator)?.native_source_project_id===projectId;}catch{return false;}})();
        if(!nativeLinked){
          const browserPrior=await loadRuntimeProject(projectId,{purpose:'source_readback'});unchanged();
          if(browserPrior)throw new Error('E_NATIVE_SOURCE_MIGRATION_UNSUPPORTED: 기존 브라우저 저장본은 보존했습니다. Native 저장으로 자동 이관하지 않습니다.');
        }
        let locator=null;try{locator=JSON.parse(retainedLocator);}catch{}
        const fields=previousArtifactText&&previousArtifactText!==artifactText ? {
          previous_authored_project_json_utf8:previousArtifactText,
          authoring_selection:locator?.authoring_selection??null,
          ...(locator?.model_source_selection?{model_source_selection:locator.model_source_selection}:{}),
          ...(locator?.code_source_selection?{code_source_selection:locator.code_source_selection}:{}),
          ...(locator?.nominal_source_selection?{nominal_source_selection:locator.nominal_source_selection}:{}),
        }:{};
        unchanged();const saved=await nativeSourceRequest('save_source',artifactText,fields);unchanged();
        if(saved.kind!=='native_source_saved'||saved.authored_project_json_utf8!==artifactText)throw new Error('E_NATIVE_SOURCE_SAVE_READBACK');
        await receiveNativeSource(saved,artifactText,receiveSourceArtifact,unchanged);
        return {locator:JSON.stringify({native_source_project_id:projectId,authored_project_json_utf8:artifactText,
          authoring_selection:saved.authoring_selection??locator?.authoring_selection??null,model_source_selection:saved.model_source_selection??locator?.model_source_selection,code_source_selection:saved.code_source_selection??locator?.code_source_selection,nominal_source_selection:saved.nominal_source_selection??locator?.nominal_source_selection,requires_new_run:saved.requires_new_run===true}),
          requires_new_run:saved.requires_new_run===true,
          message:saved.source_kind==='NOMINAL_SOURCE_ONLY' ? 'Native에 원문·명목 등록자료를 저장하고 다시 확인했습니다. 결과·세계 진행은 저장하지 않았습니다. 실행은 직접 시작하세요.' : saved.source_kind==='CODE_SOURCE_ONLY' ? 'Native에 일반 원문을 내구 저장하고 다시 확인했습니다. 결과·세계는 저장하지 않았습니다. 실행은 직접 시작하세요.' : saved.source_kind==='MODEL_SOURCE_ONLY' ? 'Native에 모형 원문을 보관했습니다. 결과·세계 진행은 보관하지 않았습니다. 분석·실행은 직접 시작하세요.' : saved.requires_new_run ? '수정 원문을 저장했습니다. 이전 실행은 보존했습니다. ▶로 새 실행을 시작하세요.' : 'Native에 원문·등록 자료를 저장하고 다시 확인했습니다. 실행은 직접 시작하세요.'};
      }
      if(requires!=='true'&&previousRequires!=='true'&&!linked)return null;
      const prior=await loadRuntimeProject(projectId,{purpose:'source_readback'});unchanged();
      // Ordinary existing world records keep their established authoring path;
      // never reinterpret them as source-only or fabricate a source parent.
      if(prior&&prior.payload_kind!=='SOURCE_ONLY')throw new Error('E_SOURCE_SAVE_WORLD_PARENT_UNSUPPORTED: 기존 세계 저장을 원문 저장으로 바꾸지 않습니다.');
      if(prior)await open(prior);unchanged();
      const sealed=await operate('source_prepare',{artifact_json_utf8:artifactText,expected_parent_hex:prior?.revision_hex??'',
        revision_hex:newRuntimeRevisionHex(),previous_source_artifact_utf8:prior?.artifact_text??null});unchanged();
      if(sealed==='null')return null;
      const candidate=JSON.parse(sealed);
      if(candidate.project_hex!==projectId)throw new Error('E_SOURCE_SAVE_PROJECT: 저장 후보가 다른 작품입니다.');
      let record;
      if(prior?.artifact_text===sealed)record=await loadRuntimeProject(projectId,{purpose:'source_readback'});
      else record=await commitRuntimeProject(runtimeStorageRecord(projectId,candidate.parent_hex,candidate.revision_hex,sealed,null,'SOURCE_ONLY'),{expectedParentRecord:prior});
      unchanged();
      const admitted=await open(record);unchanged();
      if(admitted!==artifactText)throw new Error('E_SOURCE_SAVE_READBACK: 다시 읽은 원문이 현재 저장 원문과 다릅니다.');
      return {locator:JSON.stringify({source_project_id:projectId}),message:'원문·등록 자료를 저장하고 다시 읽었습니다. 세계는 실행하지 않았습니다.'};
    },
    async reopen(projectId,unchanged,locator=null,{receiveSourceArtifact=null}={}) {
      if(locator?.native_source_project_id===projectId){
        if(!await nativeSourceAvailable())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: 같은 Native 작업대에서 다시 여세요.');
        // V4 has an existing protected source owner. Choose it by admitted
        // schema before reading; never retry a general-reader rejection as V4.
        const selectedProject=JSON.parse(locator.authored_project_json_utf8);
        if(selectedProject.schema==='ddn.authored_project_artifact.v4') {
          unchanged();const read=await nativeSourceRequest('load_public_callable_source',undefined,{project_hex:projectId});unchanged();
          if(read.kind!=='public_callable_source_current'||read.project_hex!==projectId
              ||typeof read.source_artifact_utf8!=='string')throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
          const source=JSON.parse(read.source_artifact_utf8);
          const opened=await operate('source_open',{artifact_text:read.source_artifact_utf8,
            project_hex:source.project_hex,parent_hex:source.parent_hex,revision_hex:source.revision_hex});unchanged();
          if(JSON.parse(opened).authored_identity.logical_dependency_id!==projectId)throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
          await receiveNativeSource(read,opened,receiveSourceArtifact,unchanged);
          return opened;
        }
        unchanged();const read=await nativeSourceRequest('load',locator.authored_project_json_utf8,locator.nominal_source_selection?{nominal_source_selection:locator.nominal_source_selection}:{});unchanged();
        if(read.kind!=='native_source_current'&&read.authored_project_json_utf8!==locator.authored_project_json_utf8)throw new Error('E_NATIVE_SOURCE_READBACK');
        if(JSON.parse(read.authored_project_json_utf8).authored_identity.logical_dependency_id!==projectId)throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
        if(read.first_failure_recovery===true&&(read.kind!=='native_source_current'||read.requires_new_run!==true||read.authoring_selection?.kind!==2))throw new Error('E_NATIVE_SOURCE_RECOVERY_READBACK');
        await receiveNativeSource(read,read.authored_project_json_utf8,receiveSourceArtifact,unchanged);
        // Persist only actual protected-owner readback, not caller fields.
        if(read.authoring_selection||['MODEL_SOURCE_ONLY','CODE_SOURCE_ONLY','NOMINAL_SOURCE_ONLY'].includes(read.source_kind))Object.assign(locator,{authored_project_json_utf8:read.authored_project_json_utf8,
          authoring_selection:read.authoring_selection,...(read.source_kind==='MODEL_SOURCE_ONLY'?{model_source_selection:read.model_source_selection}:read.source_kind==='CODE_SOURCE_ONLY'?{code_source_selection:read.code_source_selection}:read.source_kind==='NOMINAL_SOURCE_ONLY'?{nominal_source_selection:read.nominal_source_selection}:{}),requires_new_run:read.requires_new_run===true,first_failure_recovery:read.first_failure_recovery===true});
        return read.authored_project_json_utf8;
      }
      const record=await loadRuntimeProject(projectId,{purpose:'source_readback'});unchanged();
      if(!record)throw new Error('E_SOURCE_SAVE_MISSING: 원문 저장본이 없습니다. ID를 다시 만들지 않습니다.');
      const admitted=await open(record);unchanged();return admitted;
    },
  };
}
