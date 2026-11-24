// 공통 편집기 바깥에 장착하는 화면. 기존 키 입력·자동완성 이벤트를 가로채지 않는다.
export function mountCoCreationPanel(root, flow) {
  const make = (tag, text) => { const e = document.createElement(tag); e.textContent = text; return e; };
  const heading = make("h2", "작은 수정 후보 검토 · AI 미연결");
  const label = make("label", "어떤 점을 바꾸고 싶나요?");
  const intent = make("textarea", ""); intent.maxLength = 1000; label.append(intent);
  const disclosure = make("p", "기기 안의 고정 후보를 사용합니다. 현재 공급 후보는 배달 로봇의 속도 1을 1/2로 바꾸는 예제에 한정됩니다. 다른 작품의 후보를 자동으로 만들거나 입력한 의도를 AI가 해석하지 않습니다.");
  const boundary = make("p", "후보는 제안입니다. 검사 통과는 의도 충족이나 모든 안전을 보증하지 않습니다. 적용 전 변경 내용을 확인하세요. 적용·원문 복원은 실행·저장·외부 전송을 하지 않으며 실행 세계를 되감지 않습니다.");
  const sent = make("pre", ""), explanation = make("p", ""), diff = make("pre", "");
  const baselineInfo = make("p", ""), baselineDiff = make("pre", "");
  const status = make("p", ""); status.setAttribute("role", "status");
  const buttons = make("div", ""); let approved = null;
  const button = (text, action) => {
    const b = make("button", text); b.type = "button";
    b.onclick = async () => { try { await action(); } catch { status.textContent = "현재 문서와 변경 의도를 확인하고 다시 시도하세요."; } };
    buttons.append(b); return b;
  };
  const preview = button("원문·의도 확인", () => { approved = flow.preview(intent.value); sent.textContent = `변경 의도: ${approved.intent}\n\n원문:\n${approved.base.source}`; send.disabled = false; });
  const send = button("확인한 원문의 고정 제안 보기", () => { const p = approved; approved = null; send.disabled = true; return flow.request(p); }); send.disabled = true;
  const cancel = button("요청 취소", flow.cancel);
  const check = button("공통 제품 검사", flow.check);
  const apply = button("이 후보 적용", flow.apply);
  const reject = button("후보 거부", flow.reject);
  const restore = button("적용 전 원문 복원", flow.restore);
  button("현재 원문을 기준으로 보관", flow.captureBaseline);
  const baselinePreview = button("기준 원문으로 되돌릴 내용 확인", flow.previewBaselineRestore);
  const baselineRestore = button("확인한 기준 원문으로 복원", flow.restoreBaseline);
  intent.addEventListener("input", () => {
    approved = null; send.disabled = true; sent.textContent = "";
    const s = flow.view();
    // 이전 의도로 받은 후보나 진행 중 응답을 새 의도의 제안처럼 보이지 않게 한다.
    if (s.candidate || s.status === "requesting" || s.status === "checking") flow.cancel();
  });
  root.replaceChildren(heading, label, disclosure, boundary, sent, explanation, diff, baselineInfo, baselineDiff, buttons, status);
  const unsubscribe = flow.subscribe(s => {
    approved = null; send.disabled = true;
    if (["stale", "cancelled", "rejected", "applied", "restored", "failed", "invalid", "unavailable"].includes(s.status)) sent.textContent = "";
    baselineInfo.textContent = s.baselineDocument ? `기준 문서: ${s.baselineDocument} · 작업 중에만 보관, 저장본 아님` : "기준 원문을 보관하면 직접 편집한 뒤에도 내용을 확인하고 되돌릴 수 있습니다.";
    baselineDiff.textContent = s.restorePreview ? `현재 원문:\n${s.restorePreview.current.source}\n\n복원할 기준 원문:\n${s.restorePreview.source}` : "";
    baselinePreview.disabled = !s.baselineDocument;
    baselineRestore.disabled = !s.restorePreview;
    status.textContent = s.message;
    explanation.textContent = s.candidate?.explanation ?? "";
    diff.textContent = s.candidate ? `${s.candidate.checked ? "검사 통과 · 적용 전" : "미검사 제안"}\n이전: ${s.candidate.edit.before}\n후보: ${s.candidate.edit.after}` : "";
    check.disabled = s.status !== "candidate"; apply.disabled = s.status !== "checked";
    cancel.disabled = s.status !== "requesting"; reject.disabled = !s.candidate;
    restore.disabled = !s.canRestore;
    preview.disabled = s.status === "requesting" || s.status === "checking";
  });
  // This reads finished run records only. Rendering remains the common Bogae consumer's job.
  const completed = make("details", ""); completed.setAttribute('data-testid','completed-run-selection');
  completed.append(make('summary','완료 실행 원자료 선택'));
  const completedStatus=make('p','명시적으로 완료한 실행 두 건의 원문·마디·입력 자료를 읽습니다. 실행하거나 저장하지 않습니다.');
  completedStatus.setAttribute('role','status');
  let finished=[];
  const choices=['A','B'].map(side=>{
    const label=make('label',`실행 ${side} `), run=make('select',''), frame=make('select','');
    run.setAttribute('aria-label',`완료 실행 ${side}`); frame.setAttribute('aria-label',`완료 마디 ${side}`);
    const fillFrames=()=>{
      frame.replaceChildren(); const selected=finished[Number(run.value)];
      if(run.value===''||!selected)return;
      for(const f of selected.frames){const option=make('option',`마디 ${f.madi}`);option.value=String(f.index);frame.append(option);}
      if(selected.frames.length)frame.value=String(selected.frames.at(-1).index);
    };
    run.addEventListener('change',fillFrames);label.append(run,frame);completed.append(label);
    return {run,frame,fillFrames};
  });
  const reload=make('button','완료 실행 목록 읽기');reload.type='button';
  reload.onclick=()=>{
    try{
      finished=flow.readCompletedRuns();
      for(const c of choices){c.run.replaceChildren();c.frame.replaceChildren();
        const empty=make('option','완료 실행 선택');empty.value='';c.run.append(empty);c.run.value='';
        for(const item of finished){const option=make('option',`${item.runId} · ${item.projectId}${item.contextAvailable?'':' · 당시 설정 없음'}`);option.value=String(item.index);c.run.append(option);}
      }
      completedStatus.textContent=`완료 실행 ${finished.length}건. 당시 설정이 없는 과거 실행은 새로 실행해 완료해야 합니다.`;
    }catch(error){completedStatus.textContent=String(error.message??error);}
  };
  const readPair=make('button','선택한 두 실행 원자료 읽기');readPair.type='button';
  const readout=make('pre','');readout.setAttribute('style','max-height:12rem;overflow:auto;white-space:pre-wrap');
  readPair.onclick=()=>{
    try{
      const [a,b]=choices.map(c=>{
        if(c.run.value===''||c.frame.value==='')throw Error('각 완료 실행과 마디를 선택하세요.');
        return {run:Number(c.run.value),frame:Number(c.frame.value),revision:finished[Number(c.run.value)]?.revision};
      });
      const pair=flow.selectCompletedRunPair(a,b);
      readout.textContent=['A','B'].map(side=>{
        const value=pair[side];return `${side}: ${value.run.run_id} · 작품 ${value.project.id} · 보기 ${value.viewConfig.primary_view||'미지정'}
마디 ${value.frame.madi} · 상태 ${value.frame.state_hash}
원문 ${value.run.project_source_hash||value.run.source_hash}
기본2D: 기존 표시 수신부 확인 대기 / 의미 overlay: ${value.overlayReceiptAvailable?'실행 증표 공급됨':'실행 증표 미공급'}`;
      }).join('\n\n')+`

${pair.time.note}
입력 dt열 일치: ${pair.time.sameDeliveredDtSequence?'예':'아니오/미공급'} · 총 작품시간 동등 판정: 미생산`;
      completedStatus.textContent='두 완료 실행의 원자료 사본을 읽었습니다. 새 실행·저장·세계 되감기는 하지 않았습니다.';
    }catch(error){readout.textContent='';completedStatus.textContent=String(error.message??error);}
  };
  completed.append(reload,readPair,readout,completedStatus);root.append(completed);
  return () => { unsubscribe(); root.replaceChildren(); };
}
