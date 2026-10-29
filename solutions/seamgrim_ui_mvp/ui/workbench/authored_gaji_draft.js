// A common-Rust-checked draft is still an edit, never a Native save receipt.
// Keep the old authored artifact until explicit source save acknowledges CAS.
export async function prepareAuthoredGajiDraft({artifact,source,bridge,assertCurrent}) {
  const original=JSON.stringify(artifact), proposed=JSON.parse(JSON.stringify(source));
  if(!artifact?.source_snapshot?.root_path || proposed.root_path!==artifact.source_snapshot.root_path
      || proposed.entry_path!==artifact.source_snapshot.entry_path) {
    throw new Error('E_PROJECT_EDIT_FAILED: 작성 작품의 뿌리·실행 경로 변경은 명시 변경 절차를 사용하세요.');
  }
  const text=await bridge.authoredProjectOperation({operation:'update',text:JSON.stringify({
    original_json:original,source_snapshot:{root_path:proposed.root_path,entry_path:proposed.entry_path,
      files:proposed.files.map(file=>({path:file.path,text:file.utf8_text}))},
  })});
  assertCurrent();
  const candidate=JSON.parse(text);
  if(candidate.authored_identity?.logical_dependency_id!==artifact.authored_identity.logical_dependency_id
      || candidate.source_profile!==artifact.source_profile) {
    throw new Error('E_PROJECT_EDIT_FAILED: 가지 갱신이 다른 작품이나 글판을 공급했습니다.');
  }
  const projected=JSON.parse(await bridge.authoredProjectOperation({operation:'source_project',text}));
  assertCurrent();
  const files=new Map(proposed.files.map(file=>[file.path,file.utf8_text]));
  if(projected.schema!=='ddn.virtual_project_source.v1'||projected.root_path!==proposed.root_path
      || projected.entry_path!==proposed.entry_path || !Array.isArray(projected.files)
      || files.size!==proposed.files.length || projected.files.length!==files.size
      || new Set(projected.files.map(file=>file.path)).size!==files.size
      || !projected.files.every(file=>files.has(file.path)&&file.utf8_text===files.get(file.path)
        &&typeof file.role==='string'&&file.role.length>0)) {
    throw new Error('E_PROJECT_RUNTIME_SOURCE_SUPPLY: 원자료와 파일 역할 공급이 다릅니다.');
  }
  return projected;
}
