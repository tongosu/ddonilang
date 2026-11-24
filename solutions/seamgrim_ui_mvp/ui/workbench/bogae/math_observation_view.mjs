// UI-local adapter only. No parser, calculation, checker, storage or world writes.
import {equationSourceSlice} from '../notebook/model.js';
// readObservation() supplies {source,path,report}; the other callbacks are the
// existing validateEquationObservation/equationObservationHtml. Source slicing
// and claim/assumption/step formatting belong to that existing renderer.
// Optional UI-local subscribeObservation(refresh) returns an unsubscribe function.
// Notify on selection/source/report changes, including clearing an observation.
export function mountMathObservationView(root,{readObservation,validateObservation,renderSteps,subscribeObservation}) {
  if(!root || [readObservation,validateObservation,renderSteps].some(fn=>typeof fn!=='function')) {
    throw new TypeError('실제 관측과 원문 표시 연결이 필요합니다.');
  }
  if(subscribeObservation!==undefined&&typeof subscribeObservation!=='function') {
    throw new TypeError('관측 변경 연결은 함수여야 합니다.');
  }
  const doc=root.ownerDocument,style=doc.createElement('link'),button=doc.createElement('button');
  const status=doc.createElement('p'),content=doc.createElement('div');
  style.rel='stylesheet';style.href=new URL('./math_observation_view.css',import.meta.url).href;
  button.type='button';button.textContent='관측 표시 갱신';
  status.setAttribute('role','status');status.setAttribute('aria-live','polite');
  root.classList.add('bogae-math-observation');root.replaceChildren(style,button,status,content);
  let epoch=0,disposed=false;
  const same=(a,b)=>Boolean(a&&b&&a.source===b.source&&a.path===b.path&&JSON.stringify(a.report)===JSON.stringify(b.report));
  async function refresh(){
    if(disposed)return null;
    const ticket=++epoch;content.replaceChildren();
    try{
      const incoming=readObservation();
      if(!incoming?.report){status.textContent='NOT_PRODUCED · 표시할 실제 관측이 없습니다.';return null;}
      const captured=structuredClone(incoming);
      status.textContent='원문과 관측의 결속을 확인하고 있습니다.';
      await validateObservation(captured.report,captured.source,captured.path);
      if(disposed||ticket!==epoch)return null;
      if(!same(captured,readObservation())){status.textContent='원문이나 관측이 바뀌었습니다. 새 관측을 선택하세요.';return null;}
      const definitions=captured.report.definitions;
      const rendered=doc.createElement('section');rendered.dataset.mathObservation='true';
      rendered.innerHTML=renderSteps(captured.report,captured.source);
      content.replaceChildren(rendered);
      status.textContent='원문에 결속된 실제 관측입니다. 계산·검사 결과는 생산자가 공급합니다.';
      return {shown:true,definitions:definitions.length};
    }catch(error){
      if(!disposed&&ticket===epoch){content.replaceChildren();status.textContent=`관측을 표시하지 못했습니다. ${error.message}`;}
      return null;
    }
  }
  function dispose(){
    if(disposed)return;
    disposed=true;epoch++;button.removeEventListener('click',refresh);
    try{unsubscribe?.();}finally{root.replaceChildren();root.classList.remove('bogae-math-observation');}
  }
  let unsubscribe;
  button.addEventListener('click',refresh);refresh();
  try{
    const disconnect=subscribeObservation?.(refresh);
    if(disconnect!==undefined&&typeof disconnect!=='function')throw new TypeError('관측 변경 연결은 해제 함수를 반환해야 합니다.');
    unsubscribe=disconnect;
  }catch(error){dispose();throw error;}
  return {refresh,dispose};
}

// A project report's linked offsets must never index an individual source file.
// These copies are display projections only; the supplied report stays intact.
function projectDisplayGroups(report,sourceProjectText){
  const artifact=JSON.parse(sourceProjectText),files=artifact?.source_snapshot?.files;
  if(artifact?.schema!=='ddn.authored_project_artifact.v1'||!Array.isArray(files))throw new Error('전체 작품 원문이 필요합니다.');
  const sources=new Map();
  for(const file of files){
    if(typeof file.path!=='string'||typeof file.text!=='string'||sources.has(file.path))throw new Error('작품 원문 경로가 올바르지 않습니다.');
    sources.set(file.path,file.text);
  }
  const original=(item,path)=>{
    const span=item?.original_source;
    if(!span||!sources.has(span.path)||(path!==undefined&&span.path!==path))throw new Error('해당 파일의 실제 원문 위치가 공급되지 않았습니다.');
    equationSourceSlice(sources.get(span.path),span);
    return {start:span.start,end:span.end};
  };
  if(report.diagnostic?.original_source)original(report.diagnostic);
  const groups=[];let stopped=false;
  for(const definition of report.definitions){
    if(typeof definition.name!=='string'||!['accepted','rejected','not_checked'].includes(definition.status)||!Array.isArray(definition.steps))throw new Error('정의별 실제 검사 상태가 필요합니다.');
    const span=original(definition),path=definition.original_source.path;
    if(definition.diagnostic?.original_source)original(definition.diagnostic);
    const steps=definition.steps.map(step=>{
      if(!Number.isSafeInteger(step.index)||typeof step.operation!=='string'||!['checked','failed','not_checked'].includes(step.status))throw new Error('실제 단계 상태가 필요합니다.');
      if(stopped&&step.status!=='not_checked')throw new Error('실패 뒤 검사 완료를 표시할 수 없습니다.');
      if(step.status==='checked'){
        if(!step.equality)throw new Error('검사된 양변이 공급되지 않았습니다.');
      }else{
        if(step.equality!==null||step.reciprocal!==null)throw new Error('미수용 단계에 검사값을 표시할 수 없습니다.');
        stopped=true;
      }
      return {...step,span:original(step,path)};
    });
    if(definition.status==='accepted'&&steps.some(step=>step.status!=='checked'))throw new Error('정의와 단계의 검사 상태가 일치하지 않습니다.');
    if(definition.status!=='accepted')stopped=true;
    if(groups.at(-1)?.path!==path)groups.push({path,definitions:[]});
    groups.at(-1).definitions.push({...definition,span,steps});
  }
  return groups.map(({path,definitions})=>({path,source:sources.get(path),report:{...report,definitions}}));
}

export function mountProjectMathObservationView(root,{readObservation,renderSteps,subscribeObservation}){
  if(typeof readObservation!=='function'||typeof renderSteps!=='function')throw new Error('전체 작품 관측과 기존 표시 연결이 필요합니다.');
  let activeGroups=[];
  const api=mountMathObservationView(root,{
    readObservation(){const item=readObservation();return item?{source:item.sourceProjectText,path:'<whole-project>',report:item.report}:null;},
    async validateObservation(report,source){
      if(typeof source!=='string'||report?.coordinate_space!=='linked_source_with_original_source_mapping'
        ||report.observation_only!==true||report.unconditional_proof_issued!==false
        ||!['accepted','rejected'].includes(report.status)||!['analysis','checker'].includes(report.phase)
        ||!Array.isArray(report.definitions))throw new Error('전체 작품의 실제 관측이 필요합니다.');
      const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source));
      const hash=Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join('');
      if(hash!==report.artifact_sha256)throw new Error('전체 작품 원문 결속이 일치하지 않습니다.');
      if(report.phase==='analysis'&&report.definitions.length)throw new Error('분석 실패에 검사 단계를 표시할 수 없습니다.');
      if(report.status==='accepted'&&(!report.definitions.length||report.definitions.some(item=>item.status!=='accepted')))throw new Error('전체 수용 상태가 일치하지 않습니다.');
      projectDisplayGroups(report,source);
    },
    renderSteps(report,source){
      const escape=value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'})[c]);
      const groups=projectDisplayGroups(report,source);activeGroups=groups;
      const sourceButton=(span,label)=>`<button type="button" data-math-source-start="${span.start}" data-math-source-end="${span.end}">${escape(label)}</button>`;
      // The producer's overall verdict belongs to the entire project, not
      // to each source file. Definition/step verdicts remain renderer-owned.
      const summary=`<section data-math-project-summary aria-label="전체 작품 검사 상태"><h3>전체 작품 검사 상태</h3>
        <p>${report.status==='accepted'?'이번 가정 아래 수용':'거부'} · 무가정 증명 발급 아님</p>
        ${report.diagnostic?`<pre>${escape(report.diagnostic.formatted||report.diagnostic.message||report.diagnostic.code)}</pre>`:''}</section>`;
      return groups.length?summary+groups.map((group,index)=>`<section data-math-source-file="${escape(group.path)}" data-math-source-group="${index}"><h3>원문 ${escape(group.path)}</h3>
        <nav aria-label="${escape(group.path)} 원문 위치" class="math-source-navigation">${group.report.definitions.map(definition=>
          sourceButton(definition.span,`${definition.name} 정의 원문`)+definition.steps.map(step=>sourceButton(step.span,`${definition.name} · ${step.index+1}단계 ${step.operation}`)).join('')).join('')}</nav>
        <details data-math-source-document><summary>파일 원문과 선택 위치</summary><p data-math-source-location role="status">정의나 단계를 선택하면 해당 원문 위치를 보여 줍니다.</p>
          <textarea data-math-source-text readonly spellcheck="false" rows="8" aria-label="${escape(group.path)} 읽기 전용 원문"></textarea></details>
        ${renderSteps(group.report,group.source)}</section>`).join(''):`<section data-math-project-summary aria-label="전체 작품 검사 상태"><h3>전체 작품 검사 상태</h3>${renderSteps(report,'')}</section>`;
    },subscribeObservation
  });
  function selectSource(event){
    const button=event.target.closest?.('[data-math-source-start]');
    if(!button||!root.contains(button))return;
    const section=button.closest('[data-math-source-group]'),group=activeGroups[Number(section.dataset.mathSourceGroup)];
    if(!group)return;
    const start=Number(button.dataset.mathSourceStart),end=Number(button.dataset.mathSourceEnd);
    const source=section.querySelector('[data-math-source-text]');
    // Textarea values normalize line endings; map the producer's UTF-8 offsets
    // into those display-only UTF-16 positions, retaining an authored BOM.
    const bytes=new TextEncoder().encode(group.source),decoder=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true});
    const position=offset=>decoder.decode(bytes.slice(0,offset)).replace(/\r\n?/g,'\n').length;
    source.value=group.source;section.querySelector('[data-math-source-document]').open=true;
    source.focus();source.setSelectionRange(position(start),position(end));
    section.querySelector('[data-math-source-location]').textContent=`${group.path} · 원문 바이트 ${start}–${end} · 읽기 전용`;
  }
  root.addEventListener('click',selectSource);
  return {refresh:api.refresh,dispose(){root.removeEventListener('click',selectSource);activeGroups=[];api.dispose();}};
}

// Explicit inspection controller. The supplied product frontdoor owns all
// calculation/checking; source edits invalidate display and never auto-inspect.
export function mountProjectMathInspector(root,{readSourceProject,inspectObservation,renderSteps,subscribeSource}){
  if(!root||[readSourceProject,inspectObservation,renderSteps].some(fn=>typeof fn!=='function')
    ||(subscribeSource!==undefined&&typeof subscribeSource!=='function'))throw new Error('현재 작품과 실제 검사 연결이 필요합니다.');
  const doc=root.ownerDocument,button=doc.createElement('button'),status=doc.createElement('p'),display=doc.createElement('section');
  button.type='button';button.textContent='현재 작품 검사';status.setAttribute('role','status');status.setAttribute('aria-live','polite');
  root.classList.add('bogae-math-inspector');
  root.replaceChildren(button,status,display);
  let observation=null,epoch=0,disposed=false,unsubscribe;
  const view=mountProjectMathObservationView(display,{readObservation:()=>observation,renderSteps});
  function invalidate(){
    if(disposed)return;
    epoch++;observation=null;button.disabled=false;view.refresh();
    status.textContent='원문이 바뀌었습니다. 현재 작품 검사를 눌러 새 관측을 받으세요.';
  }
  async function inspect(){
    if(disposed)return null;
    const ticket=++epoch;observation=null;view.refresh();button.disabled=true;
    status.textContent='현재 작품의 실제 관측을 요청하고 있습니다.';
    try{
      const source=readSourceProject();
      if(typeof source!=='string'||!source.length)throw new Error('전체 작품 원문이 필요합니다.');
      const report=await inspectObservation({sourceProjectText:source});
      if(disposed||ticket!==epoch)return null;
      if(readSourceProject()!==source){invalidate();return null;}
      if(report?.phase==='project_analysis'&&report.status==='rejected'
        &&report.observation_only===true&&report.unconditional_proof_issued===false
        &&Array.isArray(report.definitions)&&report.definitions.length===0&&report.diagnostic){
        throw new Error(report.diagnostic.formatted||report.diagnostic.message||report.diagnostic.code||'작품 분석이 거부됐습니다.');
      }
      observation={sourceProjectText:source,report};
      const shown=await view.refresh();
      if(disposed||ticket!==epoch)return null;
      if(readSourceProject()!==source){invalidate();return null;}
      status.textContent=shown?.shown?'현재 원문에 결속된 실제 검사 관측을 표시했습니다.':'새 관측을 현재 원문에 연결하지 못했습니다.';
      return shown;
    }catch(error){
      if(!disposed&&ticket===epoch){observation=null;view.refresh();status.textContent=`새 관측을 받지 못했습니다. ${error.message}`;}
      return null;
    }finally{if(!disposed&&ticket===epoch)button.disabled=false;}
  }
  function dispose(){
    if(disposed)return;
    disposed=true;epoch++;observation=null;button.removeEventListener('click',inspect);
    try{unsubscribe?.();}finally{view.dispose();root.replaceChildren();root.classList.remove('bogae-math-inspector');}
  }
  button.addEventListener('click',inspect);status.textContent='현재 작품 검사를 눌러 실제 관측을 받으세요.';
  try{
    const disconnect=subscribeSource?.(invalidate);
    if(disconnect!==undefined&&typeof disconnect!=='function')throw new Error('원문 변경 연결은 해제 함수를 반환해야 합니다.');
    unsubscribe=disconnect;
  }catch(error){dispose();throw error;}
  return {inspect,invalidate,dispose};
}
