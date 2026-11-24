import {renderAuthoredBogae,createAuthoredCompositionHost} from '../authored_bogae_renderer.js';

// Each selected observation owns only its local camera, side views and picks.
function mountAuthoredFrame(panel,item) {
  const document=panel.ownerDocument,toolbar=document.createElement('div'),stage=document.createElement('div'),list=document.createElement('div');
  const canvas=document.createElement('canvas');canvas.width=640;canvas.height=360;
  canvas.style.maxWidth='100%';canvas.setAttribute('role','img');
  canvas.setAttribute('aria-label','선택 마디의 실제 원문 지정 관측');
  stage.append(canvas);panel.append(toolbar,stage,list);
  let disposed=false;
  const host=createAuthoredCompositionHost(()=>draw());
  function draw(){
    if(disposed)return null;
    host.render({projectId:item.project.id,wasmState:item.frame.wasm_state,list,stage,toolbar,canvas});
    const ctx=canvas.getContext('2d');ctx.clearRect(0,0,canvas.width,canvas.height);
    return renderAuthoredBogae(ctx,canvas.width,canvas.height,item.frame.wasm_state,host);
  }
  try{
    const model=draw();
    if(model?.status!=='ready' || model.family!=='scene'){
      const note=document.createElement('p');note.textContent=model?.diagnostic?.message ?? model?.diagnostic?.code ?? '공급된 원문 지정 관측의 장면 표시를 지원하지 않습니다.';panel.append(note);
    }
    return {displayed:model?.status==='ready' && model.family==='scene'?'authored_scene':'unavailable',
      dispose(){if(disposed)return;disposed=true;host.reset();}};
  }catch(error){disposed=true;host.reset();throw error;}
}

// Read-only receiver for the fixed completed-run supplier. Dependencies are
// the existing canonical builder, committed selector and Canvas2D renderer.
// This function evaluates no model and defines no renderer or persistence.
function showCompletedRunPair({root,pair,buildCommittedFrameCanonicalViewPayloads,
  selectCommittedOutputView,renderSpace2dCanvas2d}) {
  if(!root || typeof buildCommittedFrameCanonicalViewPayloads!=='function'
    || typeof selectCommittedOutputView!=='function' || typeof renderSpace2dCanvas2d!=='function') {
    throw new Error('기존 관측·표시 연결이 필요합니다.');
  }
  function read(slot){
    const item=pair?.[slot],run=item?.run,frame=item?.frame;
    if(run?.schema!=='seamgrim.run_record.v1' || run.final_status!=='completed'
      || item.project?.id!==run.project_id || !Array.isArray(run.frames)
      || !Number.isSafeInteger(item.frameIndex) || item.frameIndex<0
      || run.frames[item.frameIndex]!==frame || !frame?.wasm_state) {
      throw new Error(`${slot}의 완료 실행·작품·선택 마디 결속이 없습니다.`);
    }
    const payloads=buildCommittedFrameCanonicalViewPayloads(run,frame);
    const authored=frame.wasm_state.bogae_projection;
    if(authored && (!frame.state_hash || frame.wasm_state.state_hash!==frame.state_hash)) {
      throw new Error(`${slot}의 실제 장면과 선택 마디 상태가 일치하지 않습니다.`);
    }
    return {item,authored,output:selectCommittedOutputView(payloads,frame,run.source_hash)};
  }
  // Admit both before altering the display. Never mix one valid new side with
  // the previous pair's other side after a failed incoming selection.
  const entries=['A','B'].map(slot=>({slot,...read(slot)}));
  const document=root.ownerDocument,group=document.createElement('div');
  group.dataset.completedPairView='true';
  const text=(parent,tag,value)=>{const node=document.createElement(tag);node.textContent=value;parent.append(node);return node;};
  const results=[],views=[];
  try{
  for(const {slot,item,authored,output} of entries){
    const panel=document.createElement('section');panel.dataset.completedExperiment=slot;
    text(panel,'h3',`실행 ${slot}`);
    text(panel,'p',`${item.project.name ? item.project.name+' · ' : ''}마디 ${item.frame.madi}`);
    const provenance=document.createElement('details');text(provenance,'summary','원문과 실행 출처');
    text(provenance,'p',`작품 ${item.project.id} · 실행 ${item.run.run_id}`);
    text(provenance,'p',`원문 연결 ${item.frame.source_hash || item.run.source_hash || '미공급'} · 상태 ${item.frame.state_hash || '미공급'}`);
    text(provenance,'p','마디는 실행 순서입니다. 표시된 원문 연결값을 검사·증명 결과로 해석하지 않습니다.');panel.append(provenance);
    let displayed='unavailable';
    if(authored){
      const view=mountAuthoredFrame(panel,item);views.push(view);displayed=view.displayed;
    } else if(output?.family==='space2d'){
      const canvas=document.createElement('canvas');canvas.width=640;canvas.height=360;
      canvas.style.maxWidth='100%';canvas.setAttribute('role','img');
      canvas.setAttribute('aria-label',`실행 ${slot}의 실제 2D 관측`);panel.append(canvas);
      if(renderSpace2dCanvas2d({canvas,space2d:output.space2d}))displayed='space2d';
      else text(panel,'p','공급된 2D 관측을 현재 표시 연결로 그리지 못했습니다.');
    }
    if(output?.lines?.length){text(panel,'pre',output.lines.join('\n'));if(displayed==='unavailable')displayed='console';}
    if(!authored && output?.family!=='space2d')text(panel,'p','이 마디에는 표시 가능한 실제 2D 장면이 공급되지 않았습니다.');
    if(!output && !authored)text(panel,'p','선택 마디에 결속된 표시 관측을 확인하지 못했습니다.');
    if(!item.overlayReceiptAvailable)text(panel,'p','의미 겹침에 필요한 실제 실행 증표가 공급되지 않았습니다.');
    group.append(panel);results.push({slot,displayed});
  }
  text(group,'p',pair.time?.note || '선택한 두 마디의 물리적 경과 시간 대응은 공급되지 않았습니다.');
  text(group,'p',`물리적 경과 시간의 같음: ${pair.time?.physicalElapsedTimeEquality || 'NOT_PRODUCED'}`);
  root.replaceChildren(group);
  return {results,dispose(){for(const view of views)view.dispose();}};
  }catch(error){for(const view of views)view.dispose();throw error;}
}

export function mountCompletedSceneView(root,{readPair,canonicalForFrame,semanticForFrame,selectOutput,renderSpace2d}) {
  if(!root || typeof readPair!=='function' || typeof canonicalForFrame!=='function'
    || typeof selectOutput!=='function' || typeof renderSpace2d!=='function')throw new Error('완료 실행 표시 연결이 필요합니다.');
  const document=root.ownerDocument,button=document.createElement('button'),status=document.createElement('p'),content=document.createElement('div');
  const style=document.createElement('link');style.rel='stylesheet';style.href=new URL('./completed_scene_view.css',import.meta.url).href;
  button.type='button';button.textContent='선택한 두 실행 표시';
  status.setAttribute('role','status');status.setAttribute('aria-live','polite');
  root.classList.add('completed-scene-view');root.replaceChildren(style,button,status,content);
  let disposed=false,disposePair=()=>{};
  function refresh(){
    if(disposed)return null;
    try{
      const pair=readPair();
      if(!pair){disposePair();disposePair=()=>{};content.replaceChildren();status.textContent='완료 실행 두 건과 마디를 먼저 선택하세요.';return null;}
      const next=showCompletedRunPair({root:content,pair,
        buildCommittedFrameCanonicalViewPayloads:(run,frame)=>{
          const snapshot=[pair.A,pair.B].find(item=>item.run===run && item.frame===frame);
          return canonicalForFrame(snapshot);
        },selectCommittedOutputView:selectOutput,renderSpace2dCanvas2d:renderSpace2d});
      disposePair();disposePair=next.dispose;
      const results=next.results;
      for(const [i,slot] of ['A','B'].entries()){
        const note=document.createElement('p');
        try{
          const semantic=typeof semanticForFrame==='function'?semanticForFrame(pair[slot]):null;
          note.textContent=`의미 관측 상태: ${typeof semantic?.status==='string'?semantic.status:'미공급'}`;
        }catch{note.textContent='의미 관측 연결 미지원 · 기본 관측 표시는 유지합니다.';}
        content.querySelectorAll('[data-completed-experiment]')[i].append(note);
      }
      status.textContent='선택한 완료 실행의 관측을 읽었습니다. 새 실행이나 저장은 하지 않았습니다.';
      return results;
    }catch(error){status.textContent=`표시를 갱신하지 못했습니다. 이전 표시가 있다면 그대로 유지합니다. ${error.message}`;return null;}
  }
  button.addEventListener('click',refresh);refresh();
  return {refresh,dispose(){if(disposed)return;disposed=true;disposePair();button.removeEventListener('click',refresh);root.replaceChildren();root.classList.remove('completed-scene-view');}};
}
