import {mountMathWorkbench} from './math_workbench_r5/workbench.mjs';
import {mountProjectObservation} from './math_workbench_r5/project_observation.mjs';
import {conditionalAlgebraHtml} from './math_workbench_r5/goal_evidence.mjs';
import {createEquationAuthoringModel} from './notebook/model.js';
import {createRuntimeBridgeV1} from '../runtime/runtime_bridge.js';
import {equationObservationHtml} from './notebook/view.js';
import {mountMathProductObservation} from './model_observation_view.mjs';
const same=(a,b)=>Boolean(a&&b&&['projectId','path','generation','sourceText','profile'].every(k=>a[k]===b[k]));
export function createMathWorkbenchProductAdapter({readContext,readProjectSource,bridge=createRuntimeBridgeV1()}) {
  let epoch=0,disposed=false;
  const current=(ticket,c)=>!disposed&&ticket===epoch&&same(c,readContext());
  function check(ticket,c){if(!current(ticket,c))throw Error('E_MATH_HOST_STALE: 작품·의존·원문이 바뀌어 응답을 폐기했습니다.');}
  function capture(source,path,profile){const c=structuredClone(readContext());if(path!==c.path||(profile&&profile!==c.profile))throw Error('E_MATH_HOST_CONTEXT: 현재 파일·명시 글판이 다릅니다.');if(c.running)throw Error('일반 실행을 중지한 뒤 검사하세요.');return {c,ticket:epoch,source};}
  return {
    async execute(request){
      const {c,ticket}=capture(request.sourceText,request.sourcePath,request.sourceProfileIdentity);
      const project=structuredClone(readProjectSource?.()??null);
      if(!project){const value=await bridge.runSummaryWithSourceProfile(structuredClone(request));check(ticket,c);return value;}
      if(!project.files.some(f=>f.path===request.sourcePath))throw Error('E_MATH_HOST_SOURCE: 실제 작품에 대상 파일이 없습니다.');
      project.files=project.files.map(f=>f.path===request.sourcePath?{...f,utf8_text:request.sourceText}:f);
      // A validated rootless one-file work uses the same standalone Rust
      // frontdoor as its original document. No invented Start or linked text.
      // This is fresh analysis/execution, never Native world continuation.
      if(!project.root_path&&project.entry_path===request.sourcePath&&project.files.length===1){
        const value=await bridge.runSummaryWithSourceProfile(structuredClone(request));check(ticket,c);return value;
      }

      const prepared=await bridge.prepareProjectLink(project,'',true,true);check(ticket,c);
      if(prepared.source_profile_identity!==c.profile||!prepared.target||!prepared.linked_execution_source||!prepared.prepare_link_sha256)throw Error('공통 linker의 원문·시작·글판 결속이 없습니다.');
      const vm=await bridge.createSession({ddnSourceText:prepared.linked_execution_source,sourceProfileIdentity:c.profile,projectEntryName:prepared.target,cacheBust:'math-r5-host'});check(ticket,c);
      try {await vm.updateLogic(prepared.linked_execution_source);check(ticket,c);await vm.reset({keepParams:false});check(ticket,c);
        const n=request.requestedTicks??1;if(!Number.isSafeInteger(n)||n<1||n>64)throw Error('1~64마디의 명시 실행이 필요합니다.');
        const configured=vm.getConfiguredMadi();const count=configured>0?Math.min(configured,n):n;let frame;
        for(let i=0;i<count;i++){frame=await vm.step({n:1,input:null,sourceText:prepared.linked_execution_source});check(ticket,c);}
        // Existing display slot carries the actual ordinary-VM frame. This is
        // a host-local projection, not a Rust summary or Native receipt issuer.
        return {engine_state:frame};
      }finally{await vm.close().catch(()=>{});}
    },
    async proof(source,path,options){
      const {c,ticket}=capture(source,path);const result=await bridge.verifyAssumptionSource(source,path,options);check(ticket,c);
      // Display-local diagnostic shape for the existing component's byte-range
      // navigator. Preserve the Rust span and report; no string error parsing
      // or change to the bridge/WASM response contract.
      if(options?.includeSteps===true && result?.diagnostic?.span && result.source_path===path)
        return {...result,diagnostic:{...result.diagnostic,source_path:path,source_span:result.diagnostic.span}};
      return result;
    },
    async projectObservation(sourceProjectText){
      const c=structuredClone(readContext()),ticket=epoch;
      if(c.running)throw Error('일반 실행을 중지한 뒤 작품 원문을 검사하세요.');
      const result=await bridge.observeAlgebraProject({sourceProjectText});check(ticket,c);return result;
    },
    async projectUpdate(request){
      const c=structuredClone(readContext()),ticket=epoch;
      if(c.running)throw Error('일반 실행을 중지한 뒤 검사 초안을 교정하세요.');
      const result=await bridge.authoredProjectOperation({operation:'update',text:JSON.stringify(request)});check(ticket,c);return result;
    },
    invalidate(){epoch++;bridge.cancelExecution('수학 작업대 원문 문맥 변경');},
    dispose(){disposed=true;epoch++;bridge.cancelExecution('수학 작업대 종료');},
  };
}
export function installMathWorkbenchProductHost({toolbar,readContext,readProjectSource,applySource,saveSource,document=toolbar.ownerDocument}) {
  const opener=document.createElement('button');opener.type='button';opener.dataset.testid='math-workbench-r5-open';opener.textContent='수학 작업대';toolbar.append(opener);
  const dialog=document.createElement('dialog');dialog.dataset.testid='math-workbench-r5-dialog';dialog.style.cssText='width:min(1000px,94vw);max-height:88vh;overflow:auto;box-sizing:border-box';
  const style=document.createElement('style');style.textContent='[data-testid="math-workbench-r5-dialog"] textarea{display:block;box-sizing:border-box;width:100%;max-height:32vh}[data-testid="math-workbench-r5-dialog"] pre{white-space:pre-wrap;overflow-wrap:anywhere}';
  const title=document.createElement('h2');title.textContent='수학 작업대';
  const close=document.createElement('button');close.type='button';close.textContent='닫기';close.dataset.testid='math-workbench-r5-close';
  const save=document.createElement('button');save.type='button';save.textContent='편집기에 적용한 원문 저장';save.dataset.testid='math-workbench-r5-save';
  const notice=document.createElement('p');notice.setAttribute('role','status');notice.dataset.testid='math-workbench-r5-host-status';
  const help=document.createElement('p');help.textContent='아래 작업대에서는 편집 중인 원문을 관찰합니다. 위 근거 버튼과 저장은 본체 편집기에 적용된 원문을 대상으로 합니다. 결과를 서로 대신하지 않습니다. 명시 비영 조건과 약분 근거는 아래 ‘대수 단계 관찰’로 확인합니다.';
  const root=document.createElement('div');root.dataset.testid='math-workbench-r5-root';
  const projectRoot=document.createElement('section');projectRoot.dataset.testid='math-project-observation';
  const evidenceButton=document.createElement('button');evidenceButton.type='button';evidenceButton.textContent='편집기에 적용된 원문의 목표별 근거 보기';evidenceButton.dataset.testid='math-goal-evidence-read';
  const evidenceRoot=document.createElement('section');evidenceRoot.dataset.testid='math-goal-evidence';evidenceRoot.setAttribute('aria-label','목표별 조건부 근거');
  dialog.append(style,title,close,save,notice,help,evidenceButton,evidenceRoot,root,projectRoot);document.body.append(dialog);
  const adapter=createMathWorkbenchProductAdapter({readContext,readProjectSource});
  let mounted=null,projectMounted=null,bound=null,profile=null,applying=false,saving=false,disposed=false,evidenceEpoch=0;
  function ensureProjectObservation(){if(!disposed&&!projectMounted)projectMounted=mountProjectObservation(projectRoot,{observe:raw=>adapter.projectObservation(raw),update:request=>adapter.projectUpdate(request),render:(report,source)=>conditionalAlgebraHtml(report,source,equationObservationHtml)});}
  function clearEvidence(){evidenceEpoch++;evidenceButton.disabled=false;evidenceRoot.replaceChildren();}
  evidenceButton.onclick=async()=>{
    const ticket=++evidenceEpoch;const c=structuredClone(readContext());evidenceButton.disabled=true;evidenceRoot.textContent='현재 원문을 검사하고 있습니다.';
    try {
      const result=await adapter.proof(c.sourceText,c.path,{mode:'assumption-evidence'});
      if(disposed||ticket!==evidenceEpoch||!same(c,readContext()))return;
      evidenceRoot.replaceChildren();
      const p=document.createElement('p');p.textContent=`조건부 검사: ${{accepted:'수용',rejected:'거부',not_checked:'미검사'}[result.status]??result.status} · 무가정 증명 발급 아님`;evidenceRoot.append(p);
      const source=document.createElement('p');source.textContent=`원문: ${result.source_path} · ${result.source_sha256}`;evidenceRoot.append(source);
      for(const item of result.definitions??[]){
        const section=document.createElement('section');const h=document.createElement('h3');h.textContent=`${item.name} · ${{accepted:'수용',rejected:'거부',not_checked:'미검사'}[item.status]??item.status}`;
        const g=document.createElement('p');g.textContent=`목표: ${item.goal?.label??(item.goal?.kind==='conjunction'?'결합 목표':'상세 미공급 목표')} · 원문 ${item.span.start}~${item.span.end}`;
        const a=document.createElement('p');a.textContent=`가정: ${(item.assumptions??[]).map(x=>x.local_name).join(', ')||'없음'} · 사용된 가정 칸: ${(item.used_assumption_slots??[]).join(', ')||'없음'}`;
        const details=document.createElement('details'),summary=document.createElement('summary'),raw=document.createElement('pre');summary.textContent='공급된 근거 트리와 원문 위치';raw.textContent=JSON.stringify(item,null,2);details.append(summary,raw);section.append(h,g,a,details);evidenceRoot.append(section);
      }
      if(result.diagnostic){const pre=document.createElement('pre');pre.textContent=result.diagnostic.formatted??result.diagnostic.message;evidenceRoot.append(pre);}
    }catch(error){if(!disposed&&ticket===evidenceEpoch)evidenceRoot.textContent=String(error.message??error);}
    finally{if(!disposed&&ticket===evidenceEpoch)evidenceButton.disabled=false;}
  };
  function invalidate(message='작품·의존·원문이 바뀌었습니다. 현재 작품 원문을 다시 읽으세요.') {if(applying)return;clearEvidence();adapter.invalidate();mounted?.invalidate();projectMounted?.dispose();projectMounted=null;projectRoot.replaceChildren();notice.textContent=message;}
  function observe(){if(disposed||applying||saving||!mounted)return;let c;try{c=readContext();}catch{invalidate();return;}if(!same(bound,c)){bound=structuredClone(c);invalidate();}if(dialog.open)ensureProjectObservation();}
  function remount(){const c=readContext();mounted?.dispose();adapter.invalidate();profile=c.profile;bound=structuredClone(c);
    mounted=mountMathWorkbench(root,{initialRequest:{sourceText:c.sourceText,sourcePath:c.path,sourceProfileIdentity:c.profile,requestedTicks:1},readContext,
      runSummaryWithSourceProfile:r=>adapter.execute(r),verifyAssumptionSource:(s,p)=>adapter.proof(s,p),
      observeAssumptionSource:(s,p)=>adapter.proof(s,p,{mode:'assumption-evidence'}),
      observeAlgebraSource:(s,p)=>adapter.proof(s,p,{includeSteps:true}),
      renderAlgebraObservation:equationObservationHtml,
      linearProblemSupport:{createModel:createEquationAuthoringModel,render:equationObservationHtml,inspect:(source,path,options)=>adapter.proof(source,path,options)},
      mountObservationView:mountMathProductObservation,
      applySource:(expected,text)=>{applying=true;try{clearEvidence();adapter.invalidate();projectMounted?.dispose();projectMounted=null;projectRoot.replaceChildren();const applied=applySource(expected,text);bound=structuredClone(readContext());return applied;}finally{applying=false;}}});
  }
  opener.onclick=()=>{try{const c=readContext();if(!mounted||profile!==c.profile)remount();else observe();if(!dialog.open)dialog.showModal();ensureProjectObservation();notice.textContent='현재 작품 원문 읽기 후 명시 실행 또는 보임 검사를 선택하세요.';}catch(e){notice.textContent=String(e);}};
  close.onclick=()=>dialog.close();dialog.addEventListener('close',()=>invalidate('작업대를 닫아 이전 결과를 폐기했습니다.'));
  save.onclick=async()=>{if(saving||disposed)return;saving=true;save.disabled=true;try{const expected=structuredClone(readContext());const result=await saveSource(expected);if(!disposed){bound=structuredClone(readContext());invalidate(`본체 원문 저장 완료: ${result.outcome}. 이전 검사 결과는 저장하지 않았습니다. 다시 읽고 명시 검사하세요.`);}}catch(e){if(!disposed)notice.textContent=String(e);}finally{saving=false;save.disabled=false;}};
  return {observe,invalidate,dispose(){if(disposed)return;disposed=true;clearEvidence();mounted?.dispose();projectMounted?.dispose();adapter.dispose();opener.remove();dialog.remove();}};
}
