import {diagnosticWordsText,diagnosticWordsErrorText,diagnosticRecoveryText} from '../diagnostic_words_ko.mjs';
import {createRuntimeBridgeV1} from '../../runtime/runtime_bridge.js';
import {renderAuthoredBogae,createAuthoredCompositionHost,observationCellDisplay} from '../authored_bogae_renderer.js';
import {mountMathProductObservation} from '../model_observation_view.mjs';
import {mountSelectedTableView} from '../bogae/selected_table_view.mjs';
import {committedObservationRows} from '../committed_observation_display.mjs';
import {mountSpace3PrimaryView,mountSpace3SideViews,space3PrimaryProjection,hasSpace3SideView} from '../bogae/xy/space3_primary_view.mjs';

// Editor-local evidence only: never a Native source/world permit or receipt.
export function createTemporaryRunModel({readContext, createBridge = createRuntimeBridgeV1, changed = () => {}}) {
  let bridge=null, epoch=0, serial=0, record=null, disposed=false;
  const snapshot=()=>record ? structuredClone(record) : null;
  const notify=()=>changed(snapshot());
  const scope=c=>JSON.stringify([c?.projectId,c?.documentId]);
  function current(token, captured) {try{return !disposed && token===epoch && scope(readContext())===scope(captured);}catch{return false;}}
  function cancel(reason='사용자가 임시 실행을 취소했습니다.') {
    epoch++; bridge?.cancelExecution(reason); bridge=null;
    if(record?.status==='running'){record.status='cancelled';record.notice=reason;notify();}
  }
  function retireForContextChange(reason='작품 또는 문서가 바뀌어 임시 실행을 폐기했습니다.') {
    if(!record||disposed)return;
    record=null;
    cancel(reason);
    notify();
  }
  function contextChanged() {
    if(!record||disposed)return;
    let c;try{c=readContext();}catch{retireForContextChange('현재 문서 문맥을 확인할 수 없어 임시 실행을 폐기했습니다.');return;}
    if(scope(c)!==scope(record.context))retireForContextChange();
    else {const different=c.revision!==record.context.revision;
      if(record.currentEditorDifferent!==different){record.currentEditorDifferent=different;notify();}}
  }
  async function start(count=3, {bogaeEntry}={}) {
    if(disposed)throw Error('닫힌 임시 실행입니다.');
    if(!Number.isSafeInteger(count)||count<1||count>64)throw Error('임시 실행은 1~64마디를 명시하세요.');
    const c=structuredClone(readContext());
    if(bogaeEntry!==undefined){if(typeof bogaeEntry!=='string')throw Error('보기 이름을 확인하세요.');c.bogaeEntry=bogaeEntry.trim();}
    if(!c?.projectId||!c.documentId||!c.source?.trim()||!c.profile)throw Error('현재 원문과 명시 글판이 필요합니다.');
    if(c.running)throw Error('일반 실행을 중지한 뒤 별도 편집판 실행을 시작하세요.');
    cancel('새 임시 실행으로 이전 요청을 폐기했습니다.');const token=epoch;
    const own=createBridge();bridge=own;
    record={id:`temporary_${++serial}`,status:'running',context:c,requestedMadi:count,
      completedMadi:0,frames:[],initialState:null,preparation:null,currentEditorDifferent:false,
      notice:'고정한 편집판의 임시 실행입니다. 저장본·이어하기·Native 저장 영수증을 갱신하지 않습니다.'};notify();
    let vm;
    try {
      let text=c.source,entry='',projectRuntimeArtifactText='';
      if(c.nominalSourceArtifactText){
        record.preparation={kind:'native-nominal-source',source_profile_identity:c.profile,source_path:c.documentId};
      }else if(c.projectArtifactText||c.portableJ1Tree){
        if(!c.projectSource)throw Error('명시 작품 원문이 없습니다. 다시 여세요.');
        const source_snapshot={root_path:c.projectSource.root_path,entry_path:c.projectSource.entry_path,
          files:c.projectSource.files.map(f=>({path:f.path,text:f.utf8_text}))};
        const operation=c.projectArtifactText?'update':'create';
        const input=c.projectArtifactText?{original_json:c.projectArtifactText,source_snapshot}
          :{source_profile:c.profile,module_name:'',source_snapshot};
        projectRuntimeArtifactText=await own.authoredProjectOperation({operation,text:JSON.stringify(input)});
        if(!current(token,c))return null;
        const bound=JSON.parse(projectRuntimeArtifactText);
        if(bound.source_profile!==c.profile)throw Error('편집판의 원문 글판이 다릅니다. 다시 여세요.');
        record.preparation={kind:'authored-project',source_profile_identity:c.profile,artifact_json_utf8:projectRuntimeArtifactText};
      }else if(c.projectSource){
        const prepared=await own.prepareProjectLink(c.projectSource,'',true,true);
        if(!current(token,c))return null;
        if(prepared.source_profile_identity!==c.profile||!prepared.linked_execution_source||!prepared.target||!prepared.prepare_link_sha256)
          throw Error('공통 linker의 원문·글판·시작 결속을 확인하지 못했습니다.');
        record.preparation=structuredClone(prepared);
        const requirement=prepared.diagnostics?.find(row=>row.code==='E_PROJECT_REGISTERED_NOMINAL_OWNER_REQUIRED');
        if(requirement)throw Error(requirement.code+': '+requirement.message);
        text=prepared.linked_execution_source;entry=prepared.target;
      }
      vm=await own.createSession({ddnSourceText:text,sourceProfileIdentity:c.profile,
        ...(!c.projectSource||c.nominalSourceArtifactText?{sourcePath:c.documentId}:{}),
        ...(c.nominalSourceArtifactText?{nominalSourceArtifactText:c.nominalSourceArtifactText}:{}),
        ...(projectRuntimeArtifactText?{projectRuntimeArtifactText}:{}),
        projectEntryName:entry,bogaeEntryName:c.bogaeEntry||'',cacheBust:'temporary-editor-v1'});
      if(!current(token,c))return null;
      await vm.updateLogic(text);if(!current(token,c))return null;
      await vm.reset({keepParams:false});
      if(c.binaryInput){
        const supplied=await vm.configureProjectAssetsBinary(JSON.stringify(c.binaryInput.projectSource),c.binaryInput.files);
        if(!current(token,c))return null;
        if(!(supplied instanceof Map)||[...supplied.values()].some(bytes=>!(bytes instanceof Uint8Array)))throw Error("실제 registry 바이트 반환이 없습니다.");
        record.verifiedAssetBytes=supplied;
      }
      for(const control of c.controls??[]){await vm.setParam({key:control.name,value:control.value});if(!current(token,c))return null;}
      record.initialState=await vm.getStateJson();if(!current(token,c))return null;
      const configured=vm.getConfiguredMadi();const limit=configured>0?Math.min(configured,count):count;
      for(let i=0;i<limit;i++){
        const frame=await vm.step({n:1,input:i===0?c.initialInput:null,sourceText:text});
        if(!current(token,c))return null;
        record.frames.push(structuredClone(frame));record.completedMadi++;contextChanged();notify();
        await new Promise(resolve=>setTimeout(resolve,0));
      }
      if(!current(token,c))return null;
      record.status='completed';contextChanged();notify();return snapshot();
    } catch(error) {
      if(!current(token,c))return null;
      record.status='failed';record.error=String(error?.message??error);
      const diagnostic=error?.diagnostic;
      const authoredParseFailure=!vm&&record.preparation?.kind==='authored-project'
        &&diagnostic?.phase==='lang_parse'&&diagnostic.code==='E_FIXED64_LITERAL_RANGE';
      if(c.portableJ1Tree&&diagnostic&&(!record.preparation||authoredParseFailure)){
        const {source_path,source_span}=error.diagnostic;
        if(source_path&&source_span)record.errorLocation={source_path,source_span:{...source_span,encoding:'utf-8-byte'}};
      }
      notify();return snapshot();
    } finally {
      if(bridge===own){own.cancelExecution('임시 실행 문맥을 종료했습니다.');bridge=null;}
    }
  }
  function hasReadOnlyFirstFrame(artifactText) {
    if (disposed || record?.status !== 'completed' || record.currentEditorDifferent
      || record.preparation?.kind !== 'authored-project'
      || record.preparation.artifact_json_utf8 !== artifactText) return false;
    let live;
    try { live = readContext(); } catch { return false; }
    const captured = record.context;
    if (live.revision !== captured.revision || live.profile !== captured.profile
      || live.bogaeEntry || captured.bogaeEntry || live.initialInput || captured.initialInput
      || live.binaryInput || captured.binaryInput || live.controls?.length || captured.controls?.length)
      return false;
    const frame = record.frames[0];
    const projection = frame?.bogae_projection;
    if (frame?.tick_id !== 0 || projection?.status !== 'ready'
      || projection.candidates?.length !== 1) return false;
    const artifact = projection.candidates[0].artifact;
    const isReadOnly = view => view?.schema === 'ddn.scene_space3_projection.v1';
    return isReadOnly(artifact?.view) || Boolean(artifact?.relations?.some(relation => isReadOnly(relation.view)));
  }
  return {snapshot,start,cancel,contextChanged,retireForContextChange,hasReadOnlyFirstFrame,dispose(){cancel('창을 닫아 임시 실행을 종료했습니다.');disposed=true;record=null;}};
}

// Generated XY plane scenes carry their own ordered, typed table.
function xyAxisText(view) {
  const projection=view?.table_projection;
  const columns=view?.kind==='scene'&&view.spatial_profile==='plane2'
    &&projection?.schema==='ddn.public_observation_table_projection.v2'
    &&projection.tables?.length===1?projection.tables[0].columns:null;
  if(!Array.isArray(columns)||columns.length!==2)return '';
  const labels=columns.map(column=>{
    const name=column.label||column.identity?.name;
    return typeof name==='string'&&name.trim()?name+(column.unit?' ['+column.unit+']':''):'';
  });
  return labels.every(Boolean)?'가로: '+labels[0]+' · 세로: '+labels[1]:'';
}

export function mountTemporaryEditorRun(root,{readContext,onChanged=()=>{},onDiagnosticSource}={}) {
  root.dataset.testid='temporary-editor-run';
  root.innerHTML='<summary>편집판 실행 결과</summary><p>현재 DDN 편집판을 별도로 실행합니다. 저장본·이어하기는 바꾸지 않습니다. 한 파일의 최상위 문장과 지원 훅을 사용하며, 실행이라는 이름의 정의는 필수가 아닙니다. 모둠 작품은 뿌리가 선택한 시작 정의를 사용합니다. 닫은 뒤 미보관 편집판의 복구는 보장하지 않습니다.</p><label>실행 마디 <input data-temp="count" type="number" min="1" max="64" value="1"></label> <label>표시할 보기 이름 (선택) <input data-temp="view" placeholder="원문에 작성한 보기 이름"></label> <button type="button" data-temp="start">편집판만 임시 실행</button> <button type="button" data-temp="cancel">임시 실행 취소</button><p data-temp="status" role="status"></p><pre data-temp="output" style="white-space:pre-wrap;overflow-wrap:anywhere"></pre><canvas data-temp="canvas" width="640" height="260" style="max-width:100%;background:#f8fbf8"></canvas><section data-temp="observation" aria-label="편집판의 모형 결과표"></section><details><summary>고정 편집판·초기 입력·실행 근거</summary><pre data-temp="evidence" style="white-space:pre-wrap;overflow-wrap:anywhere"></pre></details>';
  const status=root.querySelector('[data-temp="status"]'),canvas=root.querySelector('canvas'),evidence=root.querySelector('[data-temp="evidence"]');
  const output=root.querySelector('[data-temp="output"]');
  const jump=root.ownerDocument.createElement('button');
  jump.type='button';
  jump.textContent='원문 위치로 이동';
  jump.hidden=true;
  status.after(jump);
  jump.onclick=()=>{
    const run=model.snapshot();
    if(!run?.errorLocation||run.currentEditorDifferent)return;
    let current;
    try { current=readContext(); } catch { return; }
    if(current.revision===run.context.revision)onDiagnosticSource?.(run.errorLocation,run.context);
  };
  // Reuse the same authored relation host as the main stage. Its scope belongs
  // to this frozen editor run, never the saved project's world or next edit.
  const element=(tag,text)=>{const node=document.createElement(tag);if(text!==undefined)node.textContent=text;return node;};
  const stage=element('div'),wrap=element('div'),toolbar=element('div'),sides=element('div');
  stage.className='bogae-stage';stage.style.cssText='position:relative;min-height:260px;min-width:0;max-width:100%';
  wrap.className='canvas-wrap';sides.className='side-view-panel';toolbar.className='authored-view-toolbar';
  toolbar.setAttribute('aria-label','편집판 무대 조작');
  const sideChip=element('button');sideChip.type='button';sideChip.dataset.authoredSideChip='true';sideChip.hidden=true;
  toolbar.append(sideChip);canvas.before(toolbar,stage);wrap.append(canvas);stage.append(wrap,sides);
  canvas.tabIndex=0;canvas.setAttribute('aria-label','명시 실행한 편집판 장면');
  const axisCaption=element('p');
  axisCaption.dataset.testid='temporary-xy-axes';
  axisCaption.dataset.temp='axes';
  axisCaption.hidden=true;
  stage.before(axisCaption);
  const publicObservations=element('section');publicObservations.dataset.tempPublicObservations='true';
  publicObservations.setAttribute('aria-label','명시 실행한 편집판의 관찰');stage.after(publicObservations);
  let model;
  const space3Root=element('section');
  space3Root.dataset.testid='temporary-space3-view';
  space3Root.style.cssText='height:clamp(280px,45vh,520px);min-width:0';
  space3Root.hidden=true;
  wrap.append(space3Root);
  const space3Madi = element('p');
  space3Madi.dataset.testid = 'temporary-space3-view-madi';
  space3Madi.hidden = true;
  space3Root.before(space3Madi);
  let space3State=null, space3Run=null, space3Retired=false;
  const space3View=mountSpace3PrimaryView(space3Root,{readState:()=>space3State});
  const sideSpace3Views=mountSpace3SideViews(stage);
  const composition=createAuthoredCompositionHost(()=>renderRecord(model.snapshot()));
  sideChip.onclick=()=>composition.toggleSides();
  const errorDisplay=(value,sourceText,sourcePath)=>{
    let diagnostic;
    try { diagnostic=JSON.parse(value); } catch { return diagnosticWordsErrorText(value,{sourceText,sourcePath})??value; }
    const words=diagnosticWordsText(diagnostic,{sourceText,sourcePath});
    if(words)return words+`\n진단 번호 · ${diagnostic.code}`;
    const message=diagnostic?.message;
    if (diagnostic?.code==='E_CHECK_TABLE_SELECTION' && diagnostic.phase==='execution_parse'
      && typeof message==='string'
      && message.startsWith('E_CHECK_TABLE_SELECTION: E_TYPE_MISMATCH: 수 종류의 정적 공급을 확정할 수 없습니다: E_SPACE3_SOURCE_PROFILE: 첫 공간 R1은 명시 @m 성분3개의 직접 차림 원문을 받습니다 (')) {
      return 'E_SPACE3_SOURCE_PROFILE: 현재 3D 면과 카메라의 길이 좌표는 성분마다 @m를 붙인 세 값으로 직접 적어야 합니다. 좌표를 변수나 계산식으로 공급하는 기능은 아직 지원하지 않습니다. 원래 검사 진단은 실행 근거에서 확인할 수 있습니다.';
    }
    if (diagnostic?.code==='E_FIXED64_LITERAL_RANGE' && typeof message==='string'
      && message.startsWith(diagnostic.code+':')) return message;
    return `${diagnostic?.code??''} ${message??value}`.trim();
  };
  let observationState=null;
  let tableFrame=null, tableRun=null, tableRetired=false;
  const tableRoot=element('section');
  tableRoot.dataset.testid='temporary-table-inspector';
  tableRoot.style.minWidth='0';
  tableRoot.hidden=true;
  publicObservations.before(tableRoot);
  const tableView=mountSelectedTableView(tableRoot,{readState:()=>tableFrame});
  const observationView=mountMathProductObservation(root.querySelector('[data-temp="observation"]'),{readState:()=>observationState});
  const history = element('section');
  const historyLabel = element('label', '편집판 보기 마디 ');
  const historyMadi = element('select');
  historyMadi.dataset.testid = 'temporary-space3-history';
  historyMadi.style.minHeight = '44px';
  historyLabel.append(historyMadi);
  history.append(historyLabel);
  history.hidden = true;
  status.after(history);
  historyMadi.onchange = () => renderRecord(model.snapshot());
  const isSpace3 = frame => Boolean(space3PrimaryProjection(frame));
  const hasPlaneImages = view => view?.spatial_profile === 'plane2'
    && view.elements?.some(element => element.kind === 'image');
  const isPlane2 = frame => {
    const projection = frame.bogae_projection;
    const artifact = projection?.candidates?.[0]?.artifact;
    return ['ddn.bogae_projection.v1', 'ddn.bogae_projection.v2'].includes(projection?.schema)
      && projection.status === 'ready' && projection.candidates?.length === 1
      && artifact?.schema === 'ddn.authored_bogae_projection.v1'
      && artifact.view?.kind === 'scene' && artifact.view.spatial_profile === 'plane2';
  };

  function renderRecord(r) {
    if (r?.id !== space3Run) {
      space3Run = r?.id;
      space3Retired = false;
      historyMadi.replaceChildren();
    }
    if (r?.currentEditorDifferent) space3Retired = true;
    const spatialHistory = r?.status === 'completed' && r.frames.length > 1
      && r.frames.some(frame => isSpace3(frame) || hasSpace3SideView(frame))
      && r.frames.every((frame, i, frames) => (isSpace3(frame) || isPlane2(frame))
        && Number.isSafeInteger(frame.tick_id) && frame.tick_id >= 0
        && (i === 0 || frame.tick_id > frames[i - 1].tick_id));
    const canSeek = spatialHistory && !space3Retired;
    history.hidden = !canSeek;
    if (canSeek && historyMadi.options.length === 0) {
      r.frames.forEach((frame, i) => {
        const option = element('option', String(frame.tick_id));
        option.value = String(i);
        historyMadi.append(option);
      });
      historyMadi.selectedIndex = r.frames.length - 1;
    }
    const selected = canSeek ? historyMadi.selectedIndex : -1;
    const supplied = selected >= 0 ? r.frames[selected] : r?.frames.at(-1);
    observationState=r?.status==='completed'&&!r.currentEditorDifferent&&supplied?.model_execution?.schema==='ddn.model_execution.candidate.v1'?supplied:null;
    observationView.refresh();
    root.dataset.presentationRun=r?.id??'';
    root.dataset.status=r?.status??'not_produced';const ctx=canvas.getContext('2d');ctx.clearRect(0,0,canvas.width,canvas.height);
    const location=!r?.currentEditorDifferent&&r?.errorLocation;
    jump.hidden=!location||!onDiagnosticSource;
    const errorLocation=location?` · ${location.source_path} · UTF-8 ${location.source_span.start}–${location.source_span.end}`:'';
    status.textContent=r?.currentEditorDifferent?'원문이 변경됐습니다. 현재 실행 결과가 없습니다. 명시 실행해 주세요.':r?`편집판 ${{running:'실행 중',completed:'완료',failed:'실행 실패',cancelled:'취소'}[r.status]??r.status} · ${r.completedMadi}마디 · ${r.currentEditorDifferent?'현재 편집과 다른 고정 편집판입니다. ':''}${r.error&&!r.currentEditorDifferent?errorDisplay(r.error,r.context?.source,r.context?.documentId)+errorLocation+' '+diagnosticRecoveryText(r.error):r.notice}`:'임시 실행 결과 없음';
    evidence.textContent=r?JSON.stringify(r,null,2):'';
    const frame=supplied,current=r?.status==='completed'&&!r.currentEditorDifferent&&!(spatialHistory&&space3Retired);
    if (r?.id!==tableRun) {tableRun=r?.id;tableRetired=false;}
    if (r?.currentEditorDifferent) tableRetired=true;
    const tableArtifact=current&&!tableRetired?frame?.bogae_projection?.candidates?.[0]?.artifact:null;
    const nextTable=[tableArtifact?.view,...(tableArtifact?.relations??[]).map(relation=>relation.view)]
      .some(view=>view?.table_projection)?frame:null;
    const tableChanged=Boolean(nextTable||tableFrame);
    tableFrame=nextTable;
    tableRoot.hidden=!tableFrame;
    if (tableChanged) void tableView.refresh();
    const artifact=frame?.bogae_projection?.candidates?.[0]?.artifact;
    const primaryHasImages=hasPlaneImages(artifact?.view);
    const primarySpace3=isSpace3(frame);
    const nextSpace3=current&&!space3Retired&&primarySpace3?frame:null;
    const refreshSpace3=Boolean(space3State||nextSpace3);
    space3State=nextSpace3;
    space3Root.hidden=!space3State;
    const resolvedMadi = space3State
      ? artifact.resolved_madi ?? (artifact.time_policy?.kind === 'common' ? frame.tick_id : null)
      : null;
    const showMadi = Number.isSafeInteger(resolvedMadi) && resolvedMadi >= 0;
    space3Madi.hidden = !showMadi;
    space3Madi.textContent = showMadi ? '3D 보기 마디 ' + resolvedMadi : '';
    if(refreshSpace3)void space3View.refresh();
    const sideSpace3State = current && !space3Retired && hasSpace3SideView(frame) ? frame : null;
    canvas.hidden=!current||!frame?.bogae_projection||primaryHasImages||primarySpace3;
    stage.hidden=canvas.hidden&&!space3State&&!sideSpace3State;
    const axisText=canvas.hidden||!isPlane2(frame)?'':xyAxisText(artifact.view);
    axisCaption.hidden=!axisText;
    axisCaption.textContent=axisText;
    canvas.setAttribute('aria-label',axisText||'명시 실행한 편집판 장면');
    // Keep the active spatial result ahead of the plain output without
    // reparenting the stage or its live canvas and controls.
    if (nextSpace3 || sideSpace3State) {
      if (publicObservations.previousElementSibling !== output) publicObservations.before(output);
    } else if (axisCaption.previousElementSibling !== output) {
      axisCaption.before(output);
    }
    output.textContent=current&&frame?(Array.isArray(frame.output_display?.texts)?frame.output_display.texts.join('\n'):JSON.stringify(frame.output_display??frame.row??{},null,2)):'';
    const authored=composition.render({projectId:`${r?.context.projectId??''}:${r?.id??''}`,wasmState:stage.hidden?null:frame,list:sides,stage,toolbar,canvas:canvas.hidden?null:canvas,
      showSide:relation=>!hasPlaneImages(relation.view)});
    sideChip.hidden=!authored?.sideTotal;sideChip.textContent=`곁 ${authored?.sideTotal??0}`;sideChip.setAttribute('aria-expanded',String(Boolean(authored?.sideCount)));
    sideSpace3Views.render(sideSpace3State);
    const sideAxes=new Map((current&&Array.isArray(artifact?.relations)?artifact.relations:[])
      .filter(relation=>relation.kind==='side').map(relation=>[relation.relation_id,xyAxisText(relation.view)]));
    for(const panel of stage.querySelectorAll('[data-authored-relation-id]')){
      const sideCanvas=panel.querySelector(':scope > canvas');
      let caption=panel.querySelector(':scope > [data-temp="side-axes"]');
      const text=panel.hidden||!sideCanvas||sideCanvas.hidden?'':sideAxes.get(panel.dataset.authoredRelationId);
      if(!text){
        if(caption&&sideCanvas?.getAttribute('aria-label')===caption.textContent)sideCanvas.removeAttribute('aria-label');
        caption?.remove();
        continue;
      }
      if(!caption){
        caption=element('p');
        caption.dataset.temp='side-axes';
        caption.dataset.testid='temporary-side-xy-axes';
        sideCanvas.before(caption);
      }
      caption.textContent=text;
      sideCanvas.setAttribute('aria-label',text);
    }
    if(!canvas.hidden)renderAuthoredBogae(ctx,canvas.width,canvas.height,frame,{hidden:composition.hidden,camera:composition.camera,selectedRelationIds:composition.selectedRelationIds});
    publicObservations.replaceChildren();
    if(current&&!observationState){
      for(const supplied of selected >= 0 ? r.frames.slice(0, selected + 1) : r.frames){
        const rows=committedObservationRows({wasm_state:supplied,state_hash:supplied.state_hash,madi:supplied.tick_id});
        for(const row of rows){
          const group=element('div');group.dataset.committedMadi=String(row.committed_madi);group.dataset.stateHash=row.state_hash;group.dataset.sourceTraceHash=row.source_trace_hash;
          group.append(element('strong',`마디 ${row.committed_madi}`));
          row.cells.forEach((cell,index)=>{const column=row.columns[index],label=column.label??column.identity?.name??'',display=observationCellDisplay(cell,column);
            const unitAlreadyShown = cell.status === 'present' && column.dtype === 'unit'
              && typeof display === 'string' && display.endsWith('@' + column.unit);
            const unitSuffix = column.unit && !unitAlreadyShown ? ' ' + column.unit : '';
            const value=element('p',`${label}: ${typeof display==='string'?display:JSON.stringify(display)}${unitSuffix}`);value.dataset.publicObservationSourceRef=column.identity?.source_ref??'';value.dataset.observationStatus=cell.status;group.append(value);});
          publicObservations.append(group);
        }
      }
    }
    publicObservations.hidden=!publicObservations.children.length;
  }
  model=createTemporaryRunModel({readContext,changed:r=>{onChanged(r);renderRecord(r);}});
  root.querySelector('[data-temp="start"]').onclick=()=>model.start(Number(root.querySelector('[data-temp="count"]').value),{bogaeEntry:root.querySelector('[data-temp="view"]').value}).catch(e=>{status.textContent=e.message;});
  root.querySelector('[data-temp="cancel"]').onclick=()=>model.cancel();
  return {...model,dispose(){sideSpace3Views.dispose();composition.reset();observationView.dispose();space3View.dispose();space3Root.remove();space3Madi.remove();tableView.dispose();tableRoot.remove();model.dispose();history.remove();}};
}
