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
  return {
    async save(artifactText,unchanged,{previousArtifactText='',retainedLocator=''}={}) {
      const projectId=JSON.parse(artifactText).authored_identity.logical_dependency_id;
      unchanged();
      const requires=await operate('source_probe',{artifact_json_utf8:artifactText});unchanged();
      const previousRequires=previousArtifactText&&previousArtifactText!==artifactText
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
        }:{};
        unchanged();const saved=await nativeSourceRequest('save_source',artifactText,fields);unchanged();
        if(saved.kind!=='native_source_saved'||saved.authored_project_json_utf8!==artifactText)throw new Error('E_NATIVE_SOURCE_SAVE_READBACK');
        return {locator:JSON.stringify({native_source_project_id:projectId,authored_project_json_utf8:artifactText,
          authoring_selection:saved.authoring_selection??locator?.authoring_selection??null,requires_new_run:saved.requires_new_run===true}),
          requires_new_run:saved.requires_new_run===true,
          message:saved.requires_new_run ? '수정 원문을 저장했습니다. 이전 실행은 보존했습니다. ▶로 새 실행을 시작하세요.' : 'Native에 원문·등록 자료를 저장하고 다시 확인했습니다. 실행은 직접 시작하세요.'};
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
    async reopen(projectId,unchanged,locator=null) {
      if(locator?.native_source_project_id===projectId){
        if(!await nativeSourceAvailable())throw new Error('E_NATIVE_SOURCE_HOST_REQUIRED: 같은 Native 작업대에서 다시 여세요.');
        unchanged();const read=await nativeSourceRequest('load',locator.authored_project_json_utf8);unchanged();
        if(read.kind!=='native_source_current'&&read.authored_project_json_utf8!==locator.authored_project_json_utf8)throw new Error('E_NATIVE_SOURCE_READBACK');
        if(JSON.parse(read.authored_project_json_utf8).authored_identity.logical_dependency_id!==projectId)throw new Error('E_NATIVE_SOURCE_READBACK_PROJECT');
        // Persist only actual protected-owner readback, not caller fields.
        if(read.authoring_selection)Object.assign(locator,{authored_project_json_utf8:read.authored_project_json_utf8,
          authoring_selection:read.authoring_selection,requires_new_run:read.requires_new_run===true});
        return read.authored_project_json_utf8;
      }
      const record=await loadRuntimeProject(projectId,{purpose:'source_readback'});unchanged();
      if(!record)throw new Error('E_SOURCE_SAVE_MISSING: 원문 저장본이 없습니다. ID를 다시 만들지 않습니다.');
      const admitted=await open(record);unchanged();return admitted;
    },
  };
}
