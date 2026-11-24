// Product adapter: only the common project frontdoor can admit a candidate.
export function createEditorCoCreationAdapter({readContext, replaceSource, validateProject, assertCanEdit, readCompletedRuns = () => [], selectCompletedRunPair = () => { throw Error("완료 실행 공급이 없습니다."); }}) {
  const same=(a,b)=>a?.projectId===b?.projectId && a?.documentId===b?.documentId
    && a?.revision===b?.revision && a?.source===b?.source;
  function read(){
    const c=readContext();
    if(!c?.projectId || !c.documentId || typeof c.revision!=='string' || typeof c.source!=='string')
      throw Error('선택한 문서의 원문과 판본을 확인할 수 없습니다.');
    // Source-only baseline/CAS needs the exact editor document, not a fabricated
    // project root. A supplied project must still bind that same document.
    if(c.projectSource != null && (!Array.isArray(c.projectSource.files)
      || !c.projectSource.files.some(f=>f.path===c.documentId && f.utf8_text===c.source)))
      throw Error('선택한 문서의 전체 프로젝트 원문을 확인할 수 없습니다.');
    return {projectId:c.projectId,documentId:c.documentId,revision:c.revision,source:c.source};
  }
  return {read, readCompletedRuns, selectCompletedRunPair,
    async validate(expected){
      const before=read(), context=readContext();
      const base={...expected,source:before.source};
      if(!same(base,before))return {ok:false,source:expected.source};
      if(!context.projectSource || !Array.isArray(context.projectSource.files)
        || !context.projectSource.files.some(f=>f.path===context.documentId && f.utf8_text===context.source))
        throw Error('후보 검사는 선택한 문서가 결속된 전체 프로젝트 원문이 필요합니다.');
      // A private copy only: preserve every other file, root, entry and lock.
      const candidate=structuredClone(context.projectSource);
      candidate.files=candidate.files.map(f=>f.path===expected.documentId?{...f,utf8_text:expected.source}:f);
      const prepared=await validateProject(candidate);
      if(!same(before,read()))return {ok:false,source:expected.source};
      if(prepared?.source_profile_identity!==context.profile || !prepared.prepare_link_sha256)
        throw Error('공통 프로젝트 검사의 판본 결속을 확인하지 못했습니다.');
      return {ok:true,source:expected.source,validationId:prepared.prepare_link_sha256};
    },
    compareAndSet(expected,source){
      assertCanEdit();
      if(typeof source!=='string'||!same(expected,read()))return {ok:false};
      // This call is synchronous. No Native save, VM creation, or reset here.
      replaceSource(source);
      return {ok:true};
    }
  };
}
export function fixedSpeedSuggestion({source}){
  const before='속도:수 := 1.',start=source.indexOf(before);
  if(start<0) throw Object.assign(Error('이 원문은 고정 로봇 속도 후보의 지원 대상이 아닙니다.'),{code:'E_CO_CREATION_UNSUPPORTED_TARGET'});
  if(source.indexOf(before,start+before.length)>=0)
    throw Object.assign(Error('고정 제안 대상이 한 곳으로 정해지지 않습니다.'),{code:'E_CO_CREATION_AMBIGUOUS_TARGET'});
  return {explanation:'같은 8칸을 더 천천히 갑니다. 속도 1에서는 8회, 1/2에서는 16회입니다. 실제 결과는 직접 실행해 비교하세요.',
    edit:{start,end:start+before.length,before,after:'속도:수 := 1 / 2.'}};
}
