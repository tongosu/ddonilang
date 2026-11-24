import {mountSelectedTableView} from './bogae/selected_table_view.mjs';
import {observationCellDisplay} from './authored_bogae_renderer.js';

// Presentation of actual Rust-owned samples only: no integration, resampling
// or row-number matching. Integer differences precede screen approximation.
function mountTimeGraph(parent,model,element) {
  const meta=model.time_execution,samples=model.committed_state?.temporal_samples;
  const box=element('details'),summary=element('summary','시간 그래프');box.append(summary);parent.append(box);
  const status=element('p');box.append(status);
  if(meta?.numeric_profile_identity!=='DDN-SOLVER-FIXED64-V1'||!Array.isArray(samples)||!samples.length){status.textContent='이 실행의 시간 표본 원자료가 없습니다.';return;}
  const raw=value=>{if(typeof value!=='string'||! /^-?\d{1,20}$/u.test(value))return null;const n=BigInt(value);return n>=-9223372036854775808n&&n<=9223372036854775807n?n:null;};
  const fields=samples[0].raw_fields?.filter(f=>typeof f.declaration_owner==='string'&&f.declaration_owner!==meta.axis_owner&&typeof f.name==='string'&&typeof f.unit==='string')??[];
  if(!fields.length){status.textContent='선택 가능한 값 원자료가 없습니다.';return;}
  const label=element('label','세로 값 '),select=element('select');select.setAttribute('aria-label','시간 그래프의 세로 값');
  for(const field of fields){const option=element('option',field.name+(field.unit?' @'+field.unit:''));option.value=field.declaration_owner;select.append(option);}select.value=fields[0].declaration_owner;label.append(select);box.append(label);
  const drawing=element('div');drawing.style.maxWidth='100%';box.append(drawing);
  const svgNode=(tag,attributes={})=>{const n=document.createElementNS('http://www.w3.org/2000/svg',tag);for(const [key,value]of Object.entries(attributes))n.setAttribute(key,String(value));return n;};
  function render(){
    drawing.replaceChildren();const field=fields.find(f=>f.declaration_owner===select.value);if(!field)return;
    const points=[];let previous=null;
    for(const sample of samples){
      const cell=sample.raw_fields?.find(f=>f.declaration_owner===field.declaration_owner);
      const x=raw(sample.time_raw),y=raw(cell?.q32_raw_decimal);
      if(x===null||y===null||previous!==null&&x<=previous||sample.source_hash!==meta.source_hash
        ||sample.definition_identity!==meta.definition_identity||sample.request_identity!==meta.request_identity
        ||sample.axis_owner!==meta.axis_owner||cell.unit!==field.unit){
        status.textContent='시간·값·단위·출처가 일치하는 실제 표본을 확인할 수 없습니다. 원문을 다시 명시 실행하세요.';return;
      }
      points.push({x,y,sample,cell});previous=x;
    }
    const xs=points.map(p=>p.x),ys=points.map(p=>p.y),min=a=>a.reduce((x,y)=>x<y?x:y),max=a=>a.reduce((x,y)=>x>y?x:y);
    const left=min(xs),right=max(xs),bottom=min(ys),top=max(ys);
    const screen=(value,low,high,length)=>high===low?length/2:Number((value-low)*BigInt(length*65536)/(high-low))/65536;
    const svg=svgNode('svg',{viewBox:'0 0 640 280',role:'img','aria-label':`실제 시간 표본 · ${field.name}${field.unit?' @'+field.unit:''}`});svg.dataset.modelTimeGraph='true';svg.style.width='100%';svg.style.maxWidth='100%';svg.style.height='auto';
    svg.append(svgNode('path',{d:'M48 16V240H624',fill:'none',stroke:'currentColor'}));
    const xy=points.map(p=>[48+screen(p.x,left,right,576),240-screen(p.y,bottom,top,224)]);
    svg.append(svgNode('polyline',{points:xy.map(p=>p.join(',')).join(' '),fill:'none',stroke:'#2563eb','stroke-width':2}));
    points.forEach((point,index)=>{const [cx,cy]=xy[index],circle=svgNode('circle',{cx,cy,r:3,fill:'#2563eb'});circle.dataset.timeRaw=point.sample.time_raw;circle.dataset.valueRaw=point.cell.q32_raw_decimal;circle.dataset.modelStateHash=point.sample.model_state_hash;
      const title=svgNode('title');title.textContent=`${point.sample.time_text??'실제 시각'} · ${point.cell.value_text??field.name}`;circle.append(title);svg.append(circle);});
    const axis=svgNode('text',{x:48,y:265,fill:'currentColor'});axis.textContent='시간 (s)';svg.append(axis);drawing.append(svg);
    status.textContent=`${field.name}${field.unit?' @'+field.unit:''} · 실제 ${points.length}개 표본. 선은 표본을 잇는 표시이며 사이값을 계산하지 않습니다.`;
  }
  select.onchange=render;render();
}

// The model's explicit 내보임 owns this table. It is not a fallback for a
// different selected Bogae view and does not manufacture a time series.
export function mountMathProductObservation(root,{readState,subscribeState}={}) {
  let disposed=false,kind='',selected=null,unsubscribe=null;
  root.style.minWidth='0';root.style.maxWidth='100%';
  const element=(tag,text)=>{const n=document.createElement(tag);if(text!==undefined)n.textContent=String(text);return n;};
  function refresh(){
    if(disposed)return;
    const state=readState();
    const next=!state?'none':(state.runtime_schema==='ddn.model_execution.candidate.v1'||state.model_execution?.schema==='ddn.model_execution.candidate.v1')?'model':'selected';
    if(next!==kind){selected?.dispose();selected=null;root.replaceChildren();kind=next;
      if(kind==='selected')selected=mountSelectedTableView(root,{readState});}
    if(kind==='none')return;
    if(kind==='selected'){selected.refresh();return;}
    root.replaceChildren();
    const model=state.model_execution??state.engine_state?.model_execution;
    const projection=state.public_observation_table_projection;
    const timed=model?.time_execution?.numeric_profile_identity==='DDN-SOLVER-FIXED64-V1';
    const notice=element('p',timed?'모형의 명시 시간 계산 결과표 · 공급된 실제 시각의 표본입니다.':'모형의 명시 결과표 · 정적 관계 계산입니다. 시간 궤적이 아닙니다.');root.append(notice);
    if(model?.status!=='completed'||projection?.schema!=='ddn.public_observation_table_projection.v2'
      ||projection.run?.status!=='completed'||!Array.isArray(projection.tables)){
      root.append(element('p','이 실행의 완료된 모형 결과표가 공급되지 않았습니다.'));return;
    }
    if(projection.presentation_state!=='ready'){
      root.append(element('p',`결과표 공급 상태: ${projection.presentation_state??'미공급'}`));return;
    }
    if(timed)mountTimeGraph(root,model,element);
    for(const data of projection.tables){
      if(!Array.isArray(data.columns)||!Array.isArray(data.rows)||data.rows.some(row=>!Array.isArray(row.cells)||row.cells.length!==data.columns.length)){
        root.append(element('p','공급된 표의 열·셀 대응을 확인할 수 없습니다.'));continue;
      }
      const table=element('table'),caption=element('caption',state.source_path??'모형 원문 경로 미공급');table.append(caption);
      const head=element('thead'),tr=element('tr');
      for(const column of data.columns){
        const th=element('th',`${column.label??column.identity?.name??'이름 미공급'} · ${column.dtype??'타입 미공급'}${column.unit?' @'+column.unit:''}`);
        th.scope='col';th.title=column.identity?.source_ref??'';tr.append(th);
      }
      head.append(tr);table.append(head);const body=element('tbody');
      for(const row of data.rows){const line=element('tr');line.dataset.stateHash=row.state_hash??'';
        for(const [index,cell] of row.cells.entries()){
          const td=element('td',observationCellDisplay(cell,data.columns[index]));td.dataset.sampleStatus=cell.status??'미공급';line.append(td);
        }body.append(line);
      }table.append(body);
      const viewport=element('div');viewport.style.maxWidth='100%';viewport.style.overflowX='auto';
      viewport.tabIndex=0;viewport.setAttribute('role','region');viewport.setAttribute('aria-label','모형 관계 계산 결과표');
      viewport.append(table);root.append(viewport);
    }
    const details=element('details'),summary=element('summary','이 실행의 출처와 원래 표 자료'),raw=element('pre');raw.textContent=JSON.stringify({source_path:state.source_path,projection,...(timed?{time_execution:model.time_execution}:{})},null,2);details.append(summary,raw);root.append(details);
  }
  if(subscribeState)unsubscribe=subscribeState(refresh);
  refresh();
  return {refresh,dispose(){if(disposed)return;disposed=true;unsubscribe?.();selected?.dispose();selected=null;root.replaceChildren();}};
}
