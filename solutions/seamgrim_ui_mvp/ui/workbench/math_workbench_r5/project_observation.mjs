import {diagnosticSelection} from './workbench.mjs';

// 원래 작성 자료를 그대로 공통 검사기에 전달한다. 작품 생성/링크/저장은 하지 않는다.
export function createProjectObservation({observe,update,changed=()=>{}}) {
  let epoch=0,disposed=false,artifact=null,editBase=null;
  let state={status:'empty',name:'',report:null,error:null,files:[]};
  const snapshot=()=>structuredClone(state);
  const publish=next=>{state=next;changed(snapshot());};
  return {
    snapshot,
    checkedArtifact(){return !disposed&&state.status==='returned'?artifact:null;},
    canEditFile(path,hash){return !disposed&&state.status!=='running'&&typeof update==='function'&&!!editBase
      &&(hash===undefined||hash===editBase.hash)&&editBase.files.some(file=>file.path===path);},
    load(text,name='작품 원문') {
      if(disposed)return;epoch++;artifact=null;editBase=null;
      publish({status:'empty',name,report:null,error:null,files:[]});
      if(typeof text!=='string'||!text.length)throw Error('작품 원문 파일이 비어 있습니다.');
      // 파일 열기는 검사·복원 성공이 아니다. 무결성/의존 확인은 공통 Rust 입구가 맡는다.
      artifact=text;publish({...state,status:'loaded'});
    },
    async check(previous=null){
      if(disposed||artifact===null)return;const ticket=++epoch,raw=artifact;
      publish({...state,status:'running',report:null,error:null,files:[],comparison:null});
      try {
        const report=structuredClone(await observe(raw));
        if(disposed||ticket!==epoch)return;
        // artifact_hash 없는 분석 거부도 원 진단 그대로 표시하되 원문 탐색은 열지 않는다.
        if(report?.status==='rejected' && report.phase==='project_analysis' && !report.artifact_sha256){
          publish({...state,status:'rejected',report,error:null,files:[]});return;
        }
        const bytes=new TextEncoder().encode(raw);
        const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
        if(disposed||ticket!==epoch)return;
        if(report?.artifact_sha256!==hash || report.observation_only!==true || report.unconditional_proof_issued!==false
          || report.execution_checked!==false || report.coordinate_space!=='linked_source_with_original_source_mapping'
          || !Array.isArray(report.definitions))throw Error('작품 원문과 관찰 응답의 결속을 확인할 수 없습니다.');
        const files=JSON.parse(raw)?.source_snapshot?.files;
        if(!Array.isArray(files)||files.some(f=>typeof f.path!=='string'||typeof f.text!=='string')
          ||new Set(files.map(f=>f.path)).size!==files.length)throw Error('원래 파일 목록을 유일하게 확인할 수 없습니다.');
        editBase={artifact:raw,hash,files:structuredClone(files),report:structuredClone(report)};
        publish({...state,status:'returned',report,files:structuredClone(files),comparison:previous?structuredClone(previous):null});
      }catch(error){if(!disposed&&ticket===epoch)publish({...state,status:'error',report:null,files:[],error:String(error.message??error)});}
    },
    async editAndCheck(path,text){
      if(!this.canEditFile(path))return;
      if(typeof text!=='string')throw Error('검사한 작품의 파일만 교정할 수 있습니다.');
      // 분석 실패 초안은 기준으로 승격하지 않는다. 마지막 결속 확인 자료에서 같은 파일만 재교정한다.
      const original=editBase.artifact,ticket=++epoch;
      const previous={editedPath:path,artifact_sha256:editBase.hash,definitions:editBase.report.definitions.map(d=>({name:d.name,status:d.status,original_source:structuredClone(d.original_source)}))};
      const snapshot=structuredClone(JSON.parse(original).source_snapshot);
      const file=snapshot.files.find(file=>file.path===path);file.text=text;
      const expectedFiles=new Map(snapshot.files.map(file=>[file.path,file.text]));
      publish({...state,status:'running',report:null,error:null,files:[],comparison:null});
      try{
        const updated=await update({original_json:original,source_snapshot:snapshot});
        if(disposed||ticket!==epoch)return;
        if(typeof updated!=='string'||!updated.length)throw Error('공통 작품 수정 결과가 공급되지 않았습니다.');
        // 수정 응답도 요청한 원문 묶음과 대조한다. 공통 생성기는 판본·등록을
        // 재구성하지만 선택하지 않은 파일이나 사용자의 초안을 바꿀 수 없다.
        const returnedFiles=JSON.parse(updated)?.source_snapshot?.files;
        if(!Array.isArray(returnedFiles)||returnedFiles.length!==expectedFiles.size
          ||new Set(returnedFiles.map(file=>file?.path)).size!==expectedFiles.size
          ||returnedFiles.some(file=>!file||!expectedFiles.has(file.path)||file.text!==expectedFiles.get(file.path)))
          throw Error('작품 수정 응답의 파일 원문이 요청과 다릅니다. 기존 검사 기준과 교정 초안을 유지했습니다.');
        artifact=updated;
        await this.check(previous);
      }catch(error){if(!disposed&&ticket===epoch)publish({...state,status:'error',report:null,files:[],error:String(error.message??error)});}
    },
    clear(){if(disposed)return;epoch++;artifact=null;editBase=null;publish({status:'empty',name:'',report:null,error:null,files:[]});},
    dispose(){epoch++;disposed=true;artifact=null;editBase=null;state={status:'disposed',name:'',report:null,error:null,files:[]};}
  };
}

// 기존 표시기에만 전달하는 사본. 판정/항/근거는 그대로 두고 실제 공급된
// original_source를 해당 파일의 excerpt 좌표로 사용한다. 원 응답은 변경하지 않는다.
export function projectDefinitionDisplay(report,definition,files){
  const mapping=definition?.original_source;
  const file=files.find(f=>f.path===mapping?.path);
  const valid=m=>file&&m?.path===file.path&&diagnosticSelection({source_path:file.path,source_span:m},{sourcePath:file.path,sourceText:file.text});
  if(!valid(mapping)||!Array.isArray(definition.steps)||definition.steps.some(step=>!valid(step.original_source)))return null;
  const displayed=structuredClone(definition);
  displayed.span={start:mapping.start,end:mapping.end};
  for(const step of displayed.steps)step.span={start:step.original_source.start,end:step.original_source.end};
  return {source:file.text,report:{...report,diagnostic:null,definitions:[displayed]}};
}

// 호출의 해소된 범위는 이 응답 안에서만 사용한다. 이름으로 대상을 찾지 않는다.
export function projectCallTarget(report,step,files){
  const call=step?.operand,span=call?.definition_span;
  if(call?.kind!=='definition_call'||!Number.isSafeInteger(span?.start)||!Number.isSafeInteger(span?.end)||span.start>=span.end)return null;
  const matches=(report?.definitions??[]).filter(d=>d.span?.start===span.start&&d.span?.end===span.end);
  if(matches.length!==1||matches[0].name!==call.definition)return null;
  const target=matches[0],original=target.original_source,file=files.find(f=>f.path===original?.path);
  if(!file||!diagnosticSelection({source_path:file.path,source_span:original},{sourcePath:file.path,sourceText:file.text}))return null;
  return target;
}

export function mountProjectObservation(root,{observe,render,update}){
  const doc=root.ownerDocument;
  const add=(tag,text)=>{const e=doc.createElement(tag);e.textContent=text;root.append(e);return e;};
  add('h2','저장된 작품 원문의 보임 재검사');
  add('p','기존 제품에서 내보낸 작품 원문 파일을 선택하세요. 파일을 열기만 하면 검사하지 않습니다. 검사용 교정은 메모리에만 적용하며 원본 파일·Native 저장·이어 실행을 변경하지 않습니다.');
  const label=add('label','작품 원문 파일');const input=doc.createElement('input');input.type='file';input.accept='.json';label.append(input);
  const check=add('button','작품 원문 명시 재검사'),clear=add('button','작품 관찰 닫기');check.type=clear.type='button';
  const download=add('button','검사한 작품 원문 내려받기');download.type='button';download.disabled=true;
  add('p','내려받기는 마지막으로 검사한 작품 원문 자료만 보존합니다. 검사 결과·미적용 초안·실행 상태는 저장하지 않습니다. Native 저장 완료를 뜻하지 않으며, 다시 열어도 명시 재검사가 필요합니다.');
  const status=add('p','NOT_PRODUCED');status.setAttribute('role','status');
  const list=add('section','');list.setAttribute('aria-label','작품의 제공·소비 정의');
  const comparison=add('section','');comparison.setAttribute('aria-label','작품 교정 전후의 실제 판정');
  const sourceLabel=add('label','선택한 원래 파일 — 읽기 전용');const source=doc.createElement('textarea');source.readOnly=true;source.rows=12;sourceLabel.append(source);
  const location=add('p','');
  const edit=add('button','선택한 파일을 검사 초안으로 가져오기');edit.type='button';edit.hidden=typeof update!=='function';edit.disabled=true;
  const draftLabel=add('label','작품 파일의 검사 초안 — 아직 미적용');const draft=doc.createElement('textarea');draft.rows=12;draftLabel.append(draft);draftLabel.hidden=typeof update!=='function';
  const apply=add('button','이 초안으로 작품을 명시 재검사');apply.type='button';apply.hidden=typeof update!=='function';apply.disabled=true;
  const draftStatus=add('p','검사 초안은 파일에 저장되지 않습니다.');draftStatus.hidden=typeof update!=='function';
  const details=add('details',''),summary=doc.createElement('summary'),raw=doc.createElement('pre');summary.textContent='공급된 작품 관찰 원문';details.append(summary,raw);
  let disposed=false,fileEpoch=0,interactionEpoch=0,selectedFile=null,draftPath=null,draftHash=null,draftBase='';
  const dirtyDraft=()=>!!draftPath&&draft.value!==draftBase;
  function mayReplaceDraft(){
    if(!dirtyDraft())return true;
    const accepted=doc.defaultView?.confirm('아직 검사에 적용하지 않은 교정 초안이 있습니다. 이 초안을 버리고 계속할까요?')===true;
    if(!accepted)draftStatus.textContent='취소했습니다. 교정 초안과 기존 작품을 유지합니다.';
    return accepted;
  }
  function changed(state){
    check.disabled=state.status==='empty'||state.status==='running';
    download.disabled=state.status!=='returned'||(!!draftPath&&state.files.find(file=>file.path===draftPath)?.text!==draft.value);
    draft.disabled=state.status==='running';
    status.textContent=state.error??(state.status==='loaded'?state.name+' · 아직 검사하지 않았습니다.':state.status==='running'?'작품 검사 중':state.status==='returned'?state.name+' · 조건부 대수 정의 관찰. 실행·가정의 참·무가정 증명은 검사하지 않았습니다.':state.status==='rejected'?state.report?.diagnostic?.message??'작품 분석 거부':'NOT_PRODUCED');
    list.replaceChildren();comparison.replaceChildren();source.value='';location.textContent='';raw.textContent=state.report?JSON.stringify(state.report,null,2):'';
    selectedFile=null;edit.disabled=true;apply.disabled=!draftPath||!model.canEditFile(draftPath,draftHash);
    if(state.status==='empty'||state.status==='loaded'){draft.value='';draftBase='';draftPath=null;draftHash=null;draftStatus.textContent='검사 초안은 파일에 저장되지 않습니다.';}
    if(state.status==='returned'&&draftPath&&state.files.find(file=>file.path===draftPath)?.text===draft.value){
      draftBase=draft.value;draftHash=state.report.artifact_sha256;
      apply.disabled=!model.canEditFile(draftPath,draftHash);
    }
    if(state.status!=='returned')return;
    if(state.comparison){
      const title=doc.createElement('h3');title.textContent='교정 전후 검사 읽기';comparison.append(title);
      const note=doc.createElement('p');note.textContent=state.comparison.editedPath+' 파일을 교정해 명시 검사한 두 결과입니다. 각 판본의 목록을 따로 보여 줍니다. 이름으로 정의를 자동 대응시키거나 모든 의존의 원인을 판정하지 않습니다. 이전 결과는 현재 원문의 판정이 아닙니다.';comparison.append(note);
      for(const [caption,hash,definitions] of [['교정 기준의 이전 검사',state.comparison.artifact_sha256,state.comparison.definitions],['교정 후 현재 검사',state.report.artifact_sha256,state.report.definitions]]){
        const block=doc.createElement('section'),heading=doc.createElement('h4'),items=doc.createElement('ul');heading.textContent=caption;block.append(heading,items);
        for(const definition of definitions){const item=doc.createElement('li');item.textContent=(definition.original_source?.path??'원래 파일 미공급')+' · '+definition.name+' · '+definition.status;items.append(item);}
        const provenance=doc.createElement('details'),summary=doc.createElement('summary'),value=doc.createElement('code');summary.textContent='검사 원문 판본';value.textContent=hash;provenance.append(summary,value);block.append(provenance);comparison.append(block);
      }
    }
    function link(parent,title,mapping){
      const file=state.files.find(f=>f.path===mapping?.path);
      const selection=file&&diagnosticSelection({source_path:file.path,source_span:mapping},{sourcePath:file.path,sourceText:file.text});
      if(!selection){const p=doc.createElement('p');p.textContent=title+' · 원래 위치 미공급';parent.append(p);return;}
      const b=doc.createElement('button');b.type='button';b.textContent=title+' · '+file.path;
      b.onclick=()=>{if(disposed||model.snapshot().report?.artifact_sha256!==state.report.artifact_sha256)return;source.value=file.text;source.focus();source.setSelectionRange(selection.start,selection.end);location.textContent=file.path+' · 원래 UTF-8 '+mapping.start+'..'+mapping.end;selectedFile=file;edit.disabled=false;};parent.append(b);
    }
    for(const definition of state.report.definitions){
      const part=doc.createElement('section');list.append(part);const title=doc.createElement('h3');title.textContent=definition.name+' · '+definition.status;part.append(title);
      if(typeof render==='function'){
        const display=projectDefinitionDisplay(state.report,definition,state.files);
        const body=doc.createElement('details'),heading=doc.createElement('summary'),content=doc.createElement('div');
        heading.textContent='가정·주장·단계 근거 읽기';body.append(heading,content);part.append(body);
        if(display){try{content.innerHTML=render(display.report,display.source);}catch(error){content.textContent='상세 표시 실패: '+String(error);}}
        else content.textContent='상세 원문의 파일 위치 결속이 부족합니다. 원 응답과 공급된 위치만 확인하세요.';
      }
      link(part,'정의 원문',definition.original_source);
      for(const [index,assumption] of (definition.assumptions??[]).entries()){
        if(assumption.relation==='nonzero')link(part,`명시 비영 가정 ${index+1}의 식`,assumption.expression?.original_source);
        else if(assumption.relation==='equality'){
          link(part,`가정 ${index+1} 왼쪽`,assumption.left?.original_source);
          link(part,`가정 ${index+1} 오른쪽`,assumption.right?.original_source);
        }
      }
      link(part,'목표 왼쪽 원문',definition.goal?.left?.original_source);
      link(part,'목표 오른쪽 원문',definition.goal?.right?.original_source);
      for(const step of definition.steps??[]){
        link(part,String(step.index+1)+'. '+step.operation+' · '+step.status,step.original_source);
        if(step.operand?.kind==='definition_call'){
          const target=projectCallTarget(state.report,step,state.files);
          if(target){
            const explanation=doc.createElement('p');explanation.textContent='이 단계가 참조한 제공 정의: '+target.name+' · 현재 정의 판정: '+target.status+'. 호출 연결은 이 응답의 범위이며 무가정 증명이나 모든 의존의 영향 판정이 아닙니다.';part.append(explanation);
            link(part,'호출한 제공 정의 원문 · '+target.name,target.original_source);
          }else{const missing=doc.createElement('p');missing.textContent='호출 대상의 정확한 원문 결속이 부족해 연결하지 않았습니다.';part.append(missing);}
        }
      }
      if(definition.diagnostic){const p=doc.createElement('pre');p.textContent=definition.diagnostic.formatted??definition.diagnostic.message;part.append(p);link(part,'거부 원문 위치',definition.diagnostic.original_source);}
    }
  }
  const model=createProjectObservation({observe,update,changed});
  edit.onclick=()=>{if(disposed||!selectedFile||model.snapshot().status!=='returned'||!mayReplaceDraft())return;interactionEpoch++;draft.value=selectedFile.text;draftBase=draft.value;draftPath=selectedFile.path;draftHash=model.snapshot().report.artifact_sha256;apply.disabled=false;draftStatus.textContent=draftPath+' · 검사한 파일을 복사한 초안입니다. 아직 적용하지 않았습니다.';};
  draft.oninput=()=>{interactionEpoch++;download.disabled=true;apply.disabled=!draftPath||!model.canEditFile(draftPath,draftHash);draftStatus.textContent='편집 중 · 위 결과는 이 초안의 검사 결과가 아닙니다. 마지막 원문 결속을 확인한 작품에서 선택 파일만 교정합니다. 내려받기 전에 초안을 명시 검사하세요.';};
  apply.onclick=async()=>{if(disposed||!draftPath||!model.canEditFile(draftPath,draftHash))return;interactionEpoch++;const ticket=fileEpoch;draftStatus.textContent='공통 작품 수정·명시 검사 중 · 원본 파일 저장 아님';await model.editAndCheck(draftPath,draft.value);if(!disposed&&ticket===fileEpoch)draftStatus.textContent=model.canEditFile(draftPath,draftHash)?'초안을 유지했습니다. 오류를 교정하고 다시 명시 검사할 수 있습니다. 원본 파일은 저장되지 않았습니다.':'검사 요청을 마쳤습니다. 위 실제 결과를 확인하세요. 추가 교정은 현재 파일을 다시 가져오세요. 원본 파일은 저장되지 않았습니다.';};
  input.onchange=async()=>{
    const ticket=++fileEpoch,interaction=interactionEpoch,file=input.files?.[0];if(!file)return;
    try{
      const text=await file.text();
      if(disposed||ticket!==fileEpoch)return;
      if(interaction!==interactionEpoch){input.value='';status.textContent='파일을 읽는 동안 작업이 바뀌어 열기를 취소했습니다. 현재 작품과 초안을 유지합니다.';return;}
      if(!text.length)throw Error('작품 원문 파일이 비어 있습니다.');
      if(!mayReplaceDraft()){input.value='';return;}
      interactionEpoch++;model.load(text,file.name);
    }catch(e){if(!disposed&&ticket===fileEpoch){input.value='';status.textContent='파일을 열지 못했습니다. 기존 작품과 초안을 유지합니다. '+String(e);}}
  };
  download.onclick=()=>{
    if(disposed||download.disabled)return;
    const text=model.checkedArtifact();if(text===null)return;
    const url=URL.createObjectURL(new Blob([text],{type:'application/json;charset=utf-8'}));
    const anchor=doc.createElement('a');anchor.href=url;anchor.download='수학작품_검사원문.json';
    root.append(anchor);anchor.click();anchor.remove();setTimeout(()=>URL.revokeObjectURL(url),0);
  };
  check.onclick=()=>{interactionEpoch++;return model.check();};clear.onclick=()=>{if(!mayReplaceDraft())return;fileEpoch++;interactionEpoch++;input.value='';model.clear();};changed(model.snapshot());
  return {dispose(){disposed=true;fileEpoch++;model.dispose();root.replaceChildren();}};
}
