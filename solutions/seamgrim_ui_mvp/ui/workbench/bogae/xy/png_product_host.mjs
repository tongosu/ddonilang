import {mountPngSceneView} from './png_scene_view.mjs';
// Existing T1 immutable manifest, not a new identity issuer or an arbitrary PNG registry.
const registeredManifest = "{\"bundle_id\":\"binary-input-ui-fixture\",\"entries\":[{\"handle\":\"59a44d51684ff9e7\",\"hash\":\"1ac7e62cd920484827355bec278b72b7e2d4c6fc7489e71e8f502d06169f7bd9\",\"mime\":\"image/png\",\"path\":\"images/pixel.png\",\"size\":77}],\"hash_algo\":\"blake3\",\"version\":\"v0\"}";
const scope = c => JSON.stringify([c?.projectId,c?.documentId]);
const key = c => JSON.stringify([c?.projectId,c?.documentId,c?.revision,c?.source,c?.profile]);
export function createPngProductHost({readContext,readRun,changed=()=>{}}) {
  let generation=0,attachment=null,held=null,disposed=false,lastContext='',blockedRunId=null;
  const current=c=>!disposed&&scope(readContext())===scope(c);
  function invalidate(){held=null;changed();}
  async function chooseFile(file) {
    if(!file)return false;
    const context=structuredClone(readContext()),captured=key(context),request=++generation;
    attachment=null;invalidate();
    if(context.profile!=='v1-core-v25'||context.projectArtifactText||context.projectSource)
      throw Error('현재 PNG 연결은 한국어 단일 DDN 편집판만 지원합니다. 저장 작품의 자산 소유를 바꾸지 않았습니다.');
    if(!Number.isSafeInteger(file.size)||file.size<1||file.size>256*1024)throw Error('PNG 원자료는 1~262144바이트 범위로 선택하세요.');
    const bytes=new Uint8Array(await file.arrayBuffer());
    if(disposed||request!==generation||key(readContext())!==captured)throw Error('PNG를 읽는 동안 원문이나 작품이 바뀌었습니다. 현재 편집은 유지했습니다.');
    if(bytes.byteLength!==file.size)throw Error('선택한 PNG의 실제 크기가 다릅니다.');
    attachment={scope:scope(context),bytes:new Uint8Array(bytes),generation:request};invalidate();return true;
  }
  function captureFor(c) {
    if(!attachment||attachment.scope!==scope(c)||disposed)return null;
    if(c.profile!=='v1-core-v25'||c.projectArtifactText||c.projectSource)throw Error('이 작품의 PNG 저장·모듈 자산 소유 교체는 지원하지 않습니다.');
    return {selectionGeneration:attachment.generation,
      projectSource:{schema:'ddn.virtual_project_source.v1',root_path:c.documentId,entry_path:c.documentId,
        files:[{path:c.documentId,role:'ddn_source',utf8_text:c.source},
          {path:'ddn.asset.json',role:'asset_manifest',utf8_text:registeredManifest}]},
      files:new Map([['images/pixel.png',new Uint8Array(attachment.bytes)]])};
  }
  function sameRun(run,c) {
    return run&&run.id!==blockedRunId&&['running','completed'].includes(run.status)&&!run.currentEditorDifferent
      &&scope(run.context)===scope(c)&&run.context.source===c.source&&run.context.profile===c.profile
      &&run.context.binaryInput?.selectionGeneration===attachment?.generation
      &&run.verifiedAssetBytes instanceof Map;
  }
  function readState() {
    held=null;if(disposed)return null;
    const c=readContext(),run=readRun();if(!sameRun(run,c))return null;
    const frame=run.frames.at(-1);if(!frame)return null;
    held={state:frame,runId:run.id,frameText:JSON.stringify(frame),contextKey:key(c),bytes:run.verifiedAssetBytes};return frame;
  }
  async function readVerifiedBytes({state,candidate,view,relation,element,signal}) {
    if(signal?.aborted||!held||held.state!==state||key(readContext())!==held.contextKey)throw Error('현재 실행과 다른 그림 요청입니다.');
    const run=readRun();if(!sameRun(run,readContext())||run.id!==held.runId||JSON.stringify(run.frames.at(-1))!==held.frameText)
      throw Error('이 그림의 실제 실행·마디가 바뀌었습니다.');
    const artifact=candidate?.artifact;
    if(state.bogae_projection?.candidates?.[0]!==candidate||!artifact
      ||(relation===null?artifact.view!==view:!artifact.relations?.includes(relation)||relation.view!==view)
      ||!view?.elements?.includes(element))throw Error('이 실행의 해당 보기·요소가 아닙니다.');
    const bytes=held.bytes.get(element.asset?.resource_handle);
    if(!(bytes instanceof Uint8Array))throw Error('이 실행의 검증 registry 바이트가 없습니다.');
    return new Uint8Array(bytes);
  }
  return {chooseFile,captureFor,readState,readVerifiedBytes,
    retireRun(){blockedRunId=readRun()?.id??null;invalidate();},
    contextChanged(){const next=key(readContext());if(next===lastContext)return;lastContext=next;
      if(attachment&&attachment.scope!==scope(readContext())){generation++;attachment=null;}invalidate();},
    dispose(){disposed=true;generation++;attachment=null;held=null;},
    state(){return {hasInput:!!attachment,generation};}};
}
export function mountPngProductHost(root,{readContext,readRun,subscribeState,onInputChanged=()=>{}}) {
  const d=root.ownerDocument,label=d.createElement('label'),input=d.createElement('input'),status=d.createElement('p'),viewRoot=d.createElement('div');
  label.textContent='등록된 PNG 원자료 열기 ';input.type='file';input.accept='.png,image/png';input.dataset.testid='png-companion-open';label.append(input);
  status.setAttribute('role','status');status.textContent='현재 등록 범위는 제공된 images/pixel.png 한 판입니다. 열기만으로 실행·저장하지 않습니다.';
  root.append(label,status,viewRoot);
  const listeners=new Set(),notify=()=>{onInputChanged();for(const listener of listeners)listener();};
  const host=createPngProductHost({readContext,readRun,changed:notify});
  const view=mountPngSceneView(viewRoot,{readState:()=>root.open===false?null:host.readState(),readVerifiedBytes:host.readVerifiedBytes,
    subscribeState:changed=>{listeners.add(changed);return()=>listeners.delete(changed);}});
  const unsubscribe=subscribeState(()=>{void view.refresh();});
  const toggle=()=>{if(root.open===false)host.retireRun();void view.refresh();};
  root.addEventListener('toggle',toggle);
  input.onchange=async()=>{const file=input.files?.[0];if(!file)return;
    try{if(await host.chooseFile(file))status.textContent='PNG 원자료를 읽었습니다. 현재 원문의 다음 명시 실행에서 실제 바이트·소유·판본을 검사합니다. 저장하지 않았습니다.';}
    catch(error){status.textContent=`PNG를 연결하지 않았습니다: ${error.message}. 원문은 유지했습니다.`;}
    finally{input.value='';}};
  return {...host,dispose(){unsubscribe?.();root.removeEventListener('toggle',toggle);host.dispose();view.dispose();listeners.clear();}};
}
