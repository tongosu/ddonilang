import { sourceExcerpt } from './goal_evidence.mjs';
// 기존 B model/checker/renderer를 주입한다. 대상 해소·수학 판정은 구현하지 않는다.
export function mountLinearProblemPanel(root, {createModel, inspect, readContext, render, locateDiagnostic, retainResult}) {
  const doc=root.ownerDocument;
  const add=(tag,text)=>{const el=doc.createElement(tag);el.textContent=text;root.append(el);return el;};
  add('h3','선택 등식의 일차 문제군 검사');
  add('p','현재 편집 원문에서 초기 등식을 읽고 직접 선택하세요. 답·검산은 그 등식에만 해당하며, 정의의 조건부 단계 검사와 가정 집합의 일관성은 구별됩니다. 작품 적용·저장은 수행하지 않습니다.');
  const load=add('button','편집 원문의 가정 대상 읽기');load.type='button';
  const label=add('label','검사할 초기 등식');const select=doc.createElement('select');label.append(select);
  const selectionSource=add('section','');selectionSource.setAttribute('aria-label','선택한 초기 등식의 원문');
  const predictionLabel=add('label','선택 등식의 검사 전 예상 — 미검증 학습자 메모');
  const prediction=doc.createElement('textarea');prediction.rows=3;prediction.maxLength=4000;predictionLabel.append(prediction);
  add('p','선택한 등식의 답과 이유를 예상하세요. 검사기에 보내거나 채점하지 않습니다. 원문·선택 변경 시 비우며, 비교 기록에는 검사 버튼을 누른 당시 메모만 남깁니다.');
  let submittedPrediction='';
  function clearPrediction(){prediction.value='';submittedPrediction='';}
  const check=add('button','선택 등식 명시 검사');check.type='button';
  const status=add('p','NOT_PRODUCED');status.setAttribute('role','status');
  const locate=add('button','선택 검사 진단의 원문 위치');locate.type='button';locate.hidden=true;
  const result=add('section','');result.setAttribute('aria-label','선택 등식의 계산과 단계');
  const keep=add('button','선택 검사 결과를 비교용으로 남기기');keep.type='button';keep.hidden=typeof retainResult!=='function';
  const details=add('details',''),summary=doc.createElement('summary'),raw=doc.createElement('pre');summary.textContent='선택 요청과 원래 B 응답';details.append(summary,raw);
  let disposed=false, epoch=0;
  function changed(state) {
    if(disposed)return;
    keep.disabled=state.busy||!state.report||state.selectedTargetIndex===null;
    prediction.disabled=state.busy||state.selectedTargetIndex===null;
    load.disabled=state.busy||!readContext();check.disabled=state.busy||state.selectedTargetIndex===null;
    select.replaceChildren();const empty=doc.createElement('option');empty.value='';empty.textContent='초기 등식을 명시적으로 선택하세요';select.append(empty);
    state.targets.forEach((target,index)=>{const option=doc.createElement('option');option.value=String(index);option.textContent=target.label;select.append(option);});
    select.value=state.selectedTargetIndex===null?'':String(state.selectedTargetIndex);select.disabled=state.busy||!state.targets.length;
    selectionSource.replaceChildren();
    const selectedTarget=state.targets[state.selectedTargetIndex];
    if(selectedTarget && state.context) {
      const source=state.context.sourceText, bytes=new TextEncoder().encode(source).length;
      const left=sourceExcerpt(source,selectedTarget.request?.left_span,bytes);
      const right=sourceExcerpt(source,selectedTarget.request?.right_span,bytes);
      const heading=doc.createElement('p');heading.textContent=selectedTarget.label;selectionSource.append(heading);
      const excerpt=doc.createElement('pre');excerpt.textContent=left!==null&&right!==null
        ? `좌변 원문: ${left}\n우변 원문: ${right}` : '선택 등식의 정확한 원문 범위를 표시할 수 없습니다. 원래 대상 자료를 확인하세요.';
      selectionSource.append(excerpt);
      const explanation=doc.createElement('p');explanation.textContent='선택한 원문입니다. 검사 결과가 아닙니다. 답·검산은 이 등식에 한정되며 다른 가정과 연립하거나 전체 가정의 일관성을 판정하지 않습니다.';selectionSource.append(explanation);
    }
    result.replaceChildren();raw.textContent='';locate.hidden=true;locate.onclick=null;
    status.textContent=state.error|| (state.busy?'요청 중':state.report?'선택한 등식의 응답 도착':state.targets.length?'대상 목록입니다. 검사 성공이 아닙니다.':'NOT_PRODUCED · 대상을 다시 읽고 선택하세요.');
    const diagnosticText=state.error || (state.report?.status==='rejected' ? state.report.diagnostic : null);
    if (typeof locateDiagnostic==='function' && diagnosticText && state.context && locateDiagnostic(diagnosticText,state.context,false)) {
      locate.hidden=false;locate.onclick=()=>{if(!disposed)locateDiagnostic(diagnosticText,state.context,true);};
    }
    if(state.report) {
      const selected=state.targets[state.selectedTargetIndex];
      const title=doc.createElement('p');title.textContent='이번 선택: '+(selected?.label??'선택 정보 없음');result.append(title);
      const predicted=doc.createElement('p');predicted.textContent=submittedPrediction ? '검사 당시 예상 · 미검증: '+submittedPrediction : '검사 전 예상은 작성하지 않았습니다.';result.append(predicted);
      const content=doc.createElement('div');
      try{content.innerHTML=render(state.report,state.context.sourceText);}catch(error){content.textContent='상세 표시 실패: '+String(error);}
      result.append(content);raw.textContent=JSON.stringify({target:selected?.request,report:state.report},null,2);
    }
  }
  const model=createModel({readContext,inspect,changed});
  async function run(action){const ticket=++epoch;try{await action();}catch(error){if(!disposed && ticket===epoch)status.textContent=String(error.message??error);}}
  load.onclick=()=>run(async()=>{clearPrediction();model.load();await model.readTargets();});
  select.onchange=()=>run(()=>{clearPrediction();model.selectTarget(select.value===''?null:Number(select.value));});
  check.onclick=()=>run(()=>{submittedPrediction=prediction.value;return model.check('linear');});
  keep.onclick=()=>run(()=>{
    const state=model.snapshot();
    if (!state.report || state.busy || state.selectedTargetIndex===null || typeof retainResult!=='function') return;
    retainResult(state,submittedPrediction);
    status.textContent='선택 등식·당시 원문·응답을 비교 기록에 남겼습니다. 작품 저장은 아닙니다.';
  });
  changed(model.snapshot());
  return {invalidate(){epoch++;clearPrediction();model.invalidate();},dispose(){disposed=true;epoch++;clearPrediction();model.dispose();root.replaceChildren();}};
}
