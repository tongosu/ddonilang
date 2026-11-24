import {readDdnSourceFile} from './source_file.js';

// File transport only. Root syntax, roles, ownership and linking stay in Rust.
export async function readDdnWorkFiles(selected) {
  const files=Array.from(selected??[]);
  if(!files.length)return [];
  if(files.length>64)throw Error('한 번에 DDN 원문 64개까지 열 수 있습니다.');
  const folder=files.every(f=>f.webkitRelativePath);
  const seen=new Set(),paths=[];
  for(const file of files){
    const raw=folder?file.webkitRelativePath.split('/').slice(1).join('/'):file.name;
    if(!raw||raw!==raw.normalize('NFC')||raw.split('/').some(p=>!p||p==='.'||p==='..')||/[\\:]/u.test(raw))
      throw Error(`원문 경로를 확인하세요: ${raw}. 경로를 임의로 고치지 않았습니다.`);
    const key=raw.toLowerCase();
    if(seen.has(key))throw Error(`동명 또는 대소문자 충돌 파일: ${raw}. 폴더 선택으로 원래 하위 경로를 유지하세요.`);
    seen.add(key);paths.push(raw);
  }
  const rows=await Promise.all(files.map(async(file,i)=>{
    const opened=await readDdnSourceFile(file);
    return {path:paths[i],text:opened.source};
  }));
  if(rows.reduce((n,r)=>n+new TextEncoder().encode(r.text).length,0)>8_000_000)
    throw Error('선택한 원문 전체가 8,000,000바이트를 넘습니다.');
  return rows;
}

export async function prepareDdnWork(files,rootPath,entryPath,bridge,assertCurrent=()=>{},profile='v1-core-v25') {
  if(!['v1-core-v25','v1-core-v25-ja-base-r1'].includes(profile))throw Error('지원되는 글판을 명시해서 선택하세요.');
  if(!rootPath||!entryPath||rootPath===entryPath||![rootPath,entryPath].every(p=>files.some(f=>f.path===p)))
    throw Error('작품의 얽음 뿌리와 실행할 모듈의 원문을 각각 고르세요. 이름으로 추정하지 않습니다.');
  // Explicit transport roles only; the actual graph parser checks declarations
  // and supplies exact missing paths before the string-only artifact codec.
  if(profile==='v1-core-v25')await bridge.projectGraph({schema:'ddn.virtual_project_source.v1',root_path:rootPath,entry_path:entryPath,
    files:files.map(f=>({path:f.path,role:f.path===rootPath?'root':'ddn_source',utf8_text:f.text}))});
  assertCurrent();
  const sourceSnapshot={root_path:rootPath,entry_path:entryPath,files};
  const text=await bridge.authoredProjectOperation({operation:'create',text:JSON.stringify({
    source_profile:profile,source_snapshot:sourceSnapshot,module_name:''})});
  assertCurrent();
  const projected=JSON.parse(await bridge.authoredProjectOperation({operation:'source_project',text}));
  assertCurrent();
  if(projected.schema!=='ddn.virtual_project_source.v1'||projected.root_path!==rootPath||projected.entry_path!==entryPath
    ||projected.files.length!==files.length||!files.every(f=>projected.files.some(p=>p.path===f.path&&p.utf8_text===f.text)))
    throw Error('공통 검사기가 돌려준 원문과 선택 파일이 다릅니다. 기존 작품은 유지했습니다.');
  // J1 keeps the existing checked artifact owner through execution. The generic
  // virtual-tree linker has no profile slot; do not infer syntax or add a wire field.
  if(profile==='v1-core-v25-ja-base-r1')return {projected,prepared:null,profile,artifactText:text};
  // The actual Rust probe distinguishes registered source custody from a
  // flat executable. Retain its existing authored artifact; opening neither
  // registers a replacement identity nor invokes the Native execution owner.
  const registered=await bridge.authoredProjectOperation({operation:'source_probe',text:JSON.stringify({artifact_json_utf8:text})});
  assertCurrent();
  if(registered==='true')return {projected,prepared:null,profile,artifactText:text};
  if(registered!=='false')throw Error('공통 등록 원문 판정을 확인하지 못했습니다. 기존 작품은 유지했습니다.');
  const prepared=await bridge.prepareProjectLink(projected,'',true,false);
  assertCurrent();
  if(prepared.source_profile_identity!=='v1-core-v25'||!prepared.linked_execution_source||!prepared.prepare_link_sha256)
    throw Error('공통 작품 검사 결과의 원문·글판 결속을 확인하지 못했습니다.');
  return {projected,prepared,profile};
}

export function workOpenError(error) {
  const text=String(error?.message??error),start=text.indexOf('{');
  if(start>=0)try{
    const d=JSON.parse(text.slice(start));
    if(d.schema==='ddn.local_project.error.v1'&&typeof d.code==='string'&&typeof d.message==='string')
      return `${d.code}: ${d.message}${typeof d.path==='string'?` · ${d.path}`:''}`;
  }catch{} // Preserve other diagnostics as supplied, never reinterpret them as success.
  return text;
}

export function installDdnWorkOpen({bridge,readContext,apply}) {
  const dialog=document.createElement('dialog');dialog.dataset.testid='ddn-work-open';
  dialog.setAttribute('aria-label','여러 DDN 원문으로 작품 열기');
  dialog.style.cssText='width:min(620px,calc(100vw - 32px));max-height:85vh;overflow:auto;box-sizing:border-box';
  dialog.innerHTML='<h2>DDN 작품 열기</h2><p><label>원문 글판 <select data-work="profile" aria-label="원문 글판"><option value="v1-core-v25">한국어 v25</option><option value="v1-core-v25-ja-base-r1">일본어 J1 (지원 범위)</option></select></label></p><p>v25 얽음 뿌리와 필요한 DDN 원문을 함께 고르세요. 하위 폴더가 있으면 폴더 선택으로 경로를 유지하세요. 이 첫 범위는 DDN 원문만 지원하며 외부 가지·DDL·별도 자원은 여기서 대신 만들지 않습니다.</p><button type="button" data-work="files">원문 파일들 선택</button> <button type="button" data-work="folder">원문 폴더 선택</button><input type="file" multiple accept=".ddn,.또니" data-work="input" hidden><input type="file" webkitdirectory multiple data-work="directory" hidden><p data-work="selected"></p><label>얽음 뿌리 <select data-work="root" aria-label="얽음 뿌리"></select></label><p><label>실행 모듈의 원문 <select data-work="entry" aria-label="실행 모듈의 원문"></select></label></p><p>열기는 실행·저장이 아닙니다. 검사에 실패하면 빠진 원문과 경로를 확인해 다시 선택하세요. 현재 편집은 바꾸지 않습니다.</p><p data-work="status" role="status"></p><button type="button" data-work="open">검사 후 작품 열기</button> <button type="button" data-work="cancel">취소</button>';
  document.body.append(dialog);
  const by=k=>dialog.querySelector(`[data-work="${k}"]`),status=by('status');
  let epoch=0,rows=[],captured='',busy=false;
  const context=()=>JSON.stringify(readContext());
  function clear(){by('profile').value='v1-core-v25';rows=[];by('selected').textContent='선택한 원문 없음';for(const k of ['root','entry']){
    const o=document.createElement('option');o.value='';o.textContent='명시 선택';by(k).replaceChildren(o);
  }by('input').value='';by('directory').value='';}
  function invalidate(){epoch++;busy=false;by('open').disabled=false;}
  function close(){invalidate();dialog.close();clear();}
  function assertCurrent(token){if(token!==epoch||!dialog.open||context()!==captured)
    throw Error('파일을 읽거나 검사하는 동안 작품·원문이 바뀌었습니다. 현재 편집을 유지했습니다. 다시 여세요.');}
  by('profile').onchange=()=>{invalidate();status.textContent='선택한 글판으로 다시 검사 후 여세요. 실행·저장은 하지 않았습니다.';};
  by('files').onclick=()=>by('input').click();by('folder').onclick=()=>by('directory').click();
  async function select(event){
    if(!event.target.files?.length)return; // Native chooser cancellation preserves both current work and pending selection.
    const token=++epoch;busy=false;by('open').disabled=false;
    status.textContent='원문 읽는 중…';
    try{
      const next=await readDdnWorkFiles(event.target.files);assertCurrent(token);rows=next;
      by('selected').textContent=rows.map(r=>r.path).join(' · ');
      for(const k of ['root','entry']){by(k).replaceChildren();const empty=document.createElement('option');empty.value='';empty.textContent='명시 선택';by(k).append(empty);
        for(const row of rows){const o=document.createElement('option');o.value=row.path;o.textContent=row.path;by(k).append(o);}}
      status.textContent='뿌리와 실행 모듈 원문을 선택한 뒤 검사 후 여세요. 아직 실행·저장하지 않았습니다.';
    }catch(error){if(token===epoch){clear();status.textContent=`읽기 실패: ${error.message}. 현재 편집은 유지했습니다.`;}}
  }
  by('input').onchange=select;by('directory').onchange=select;
  by('open').onclick=async()=>{
    if(busy)return;busy=true;by('open').disabled=true;const token=epoch;
    try{
      assertCurrent(token);status.textContent='공통 작품 검사 중…';
      const result=await prepareDdnWork(structuredClone(rows),by('root').value,by('entry').value,bridge,()=>assertCurrent(token),by('profile').value);
      assertCurrent(token);await apply(result.projected,captured,result);close();
    }catch(error){if(token===epoch)status.textContent=`열지 않았습니다: ${workOpenError(error)}. 필요한 원문과 선언 경로를 확인하세요. 현재 편집은 유지했습니다.`;}
    finally{if(token===epoch){busy=false;by('open').disabled=false;}}
  };
  by('cancel').onclick=close;dialog.addEventListener('cancel',e=>{e.preventDefault();close();});
  function open(){invalidate();clear();captured=context();status.textContent='기존 미저장 편집은 검사 완료 전까지 유지됩니다.';dialog.showModal();}
  return {open,
    async openFiles(files,{rootPath,entryPath}) {
      open();
      await select({target:{files}});
      if(!dialog.open||![rootPath,entryPath].every(path=>rows.some(row=>row.path===path)))return false;
      // The example supplies its exact declared paths. Admission remains the
      // same explicit button action as a user-selected DDN work.
      by('root').value=rootPath;by('entry').value=entryPath;
      return true;
    },dispose(){invalidate();dialog.remove();}};
}
