// Draft only. Inputs must be admitted notebook projections from one fixed
// connected producer. This is not a public wire or a scene/world comparator.
import { comparisonRanges, notebookViewport, notebookExperimentHtml } from './view.js';
import { selectedSample, sampleSelectionHtml } from './sample_selection.mjs';
import { buildGraphPreviewHtml } from '../../components/graph_preview.js';

const same = (a,b) => JSON.stringify(a) === JSON.stringify(b);
const escape = value => String(value).replace(/[&<>"']/g,c=>
  ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'})[c]);

export function comparisonSupport(views) {
  if (!Array.isArray(views) || views.length !== 2 || views.some(v=>!v?.comparisonKey)) {
    return {supported:false,reason:'실제 생산자가 확인한 두 관측이 필요합니다.'};
  }
  const [a,b]=views,ka=a.comparisonKey,kb=b.comparisonKey;
  for (const field of ['model','profile','entryPath','rootSource','domain']) {
    if (typeof ka[field]!=='string' || !ka[field] || !same(ka[field],kb[field])) {
      return {supported:false,reason:'모형·원문 프로필·의존 문맥 또는 정의역이 달라 비교를 연결하지 않았습니다.'};
    }
  }
  if (!Array.isArray(ka.times) || !ka.times.length || !same(ka.times,kb.times)
      || a.rows?.length!==ka.times.length || b.rows?.length!==kb.times.length) {
    return {supported:false,reason:'정확 표본 시각과 순서가 달라 연결하지 않았습니다. 보간하지 않습니다.'};
  }
  if (!Array.isArray(a.columns) || !Array.isArray(b.columns)
      || !same(a.columns.map(c=>[c.label,c.unit]),b.columns.map(c=>[c.label,c.unit]))) {
    return {supported:false,reason:'표시 단위와 열 대응이 달라 같은 축으로 묶지 않았습니다.'};
  }
  // initial/target/comparedInput are experiment conditions, not a blanket
  // equality gate. Supplied summaries keep each experiment's actual values.
  for (const view of views) selectedSample(view,null);
  return {supported:true,changedInputs:!same(ka.initial,kb.initial)
    || !same(ka.target,kb.target) || !same(a.comparedInput,b.comparedInput)};
}

export function notebookComparisonHtml(views,{layout='side',mode='graph',index=null,zoom=1,pan=0,selectionControls=true}={}) {
  if (!['side','overlay'].includes(layout)) throw new Error('지원하지 않는 비교 배치입니다.');
  const support=comparisonSupport(views);
  if (!support.supported) return `<p role="status">${escape(support.reason)}</p>`;
  for (const view of views) selectedSample(view,index);
  const ranges=comparisonRanges(views);
  const controls=views.map((view,i)=>`<section data-ab-experiment="${i}"><h3>실험 ${i?'B':'A'}</h3>${selectionControls?sampleSelectionHtml(view,index):''}${notebookExperimentHtml(view,layout==='overlay'?'table':mode,{...ranges,zoom,pan,clipId:`ab-side-${i}`})}</section>`).join('');
  if (layout==='side') return `<div class="notebook-ab-pair">${controls}</div>`;
  if (ranges.incompatible || views.some(v=>!v.graph || v.graph.series?.length!==1
      || v.graph.series[0].points.length!==v.rows.length || !v.graph.series[0].points.every(p=>
        typeof p.x==='number' && Number.isFinite(p.x) && typeof p.y==='number' && Number.isFinite(p.y)))) {
    return `<p role="status">겹쳐 그릴 실제 좌표가 없거나 축 표시가 다릅니다. 각 실험의 표를 읽으세요.</p><div class="notebook-ab-pair">${controls}</div>`;
  }
  const axis=notebookViewport(ranges.axis,{zoom,pan});
  if (!axis) throw new Error('표시 범위를 확인할 수 없습니다.');
  const graph={axis,series:views.map((view,i)=>({...view.graph.series[0],id:`${i?'B':'A'} · ${view.graph.series[0].id}`}))};
  return `<figure class="notebook-ab-overlay"><style>.notebook-ab-overlay .runtime-graph-line--b{stroke:#8a3b16;stroke-width:3;stroke-dasharray:7 4}.notebook-ab-overlay circle[data-ab-selected]{stroke:currentColor;stroke-width:3;fill:white;r:6px}</style>
    <figcaption>A 실선 · B 점선 · 점은 실제 표본, 좌표 위치는 화면 근사</figcaption>
    ${buildGraphPreviewHtml(graph,{width:480,height:300,axisLabels:views[0].axisLabels,clipId:'notebook-ab-overlay'})}</figure>
    <div class="notebook-ab-pair">${controls}</div>`;
}

export function markComparisonSelection(root,views,index) {
  const support=comparisonSupport(views);
  if (!support.supported) throw new Error(support.reason);
  for(const view of views) selectedSample(view,index);
  const points=[...root.querySelectorAll('.notebook-ab-overlay circle.runtime-graph-point')];
  const count=views[0].rows.length;
  if(points.length!==count*2)throw new Error('겹친 그래프와 두 관측의 표본 대응이 다릅니다.');
  points.forEach((point,i)=>{
    if(index!==null && (i===index || i===count+index))point.setAttribute('data-ab-selected','true');
    else point.removeAttribute('data-ab-selected');
  });
}
