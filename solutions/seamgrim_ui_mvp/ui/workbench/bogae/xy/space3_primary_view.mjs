import {mountSpace3PolylineView} from './space3_polyline_view.mjs';
import {prepareSpace3R1Frame} from './space3_r1_camera.mjs';

function authoredCandidate(state) {
  const projection = state?.bogae_projection;
  if (!['ddn.bogae_projection.v1', 'ddn.bogae_projection.v2'].includes(projection?.schema)
    || projection.status !== 'ready'
    || !Array.isArray(projection.candidates) || projection.candidates.length !== 1) return null;
  const candidate = projection.candidates[0];
  return candidate?.artifact?.schema === 'ddn.authored_bogae_projection.v1' ? candidate : null;
}

export function space3PrimaryProjection(state) {
  const candidate = authoredCandidate(state);
  return candidate?.artifact.view?.spatial_profile === 'space3' ? candidate : null;
}

// The host supplies its current, ownership-checked runtime snapshot and
// invalidates it on source, run or project changes. No other frame is searched.
export function mountSpace3PrimaryView(root, {readState, subscribeState} = {}) {
  if (typeof readState !== 'function') throw new TypeError('현재 공간 보기 읽기가 필요합니다.');
  return mountSpace3FaceView(root, {
    async readFrame() {
      return space3PrimaryProjection(await readState())?.artifact.view ?? null;
    },
    subscribeFrame: subscribeState,
  });
}

function mountSpace3FaceView(root, {readFrame, subscribeFrame}) {
  return mountSpace3PolylineView(root, {
    readFrame,
    subscribeFrame,
    prepareFrame(view, viewport) {
      if (!Array.isArray(view.elements) || view.elements.some(element => element?.kind !== 'face')) {
        throw new TypeError('이 공간 보기에서 공급한 면 요소를 표시할 수 없습니다.');
      }
      return prepareSpace3R1Frame(view, view.elements.map(element => element.vertices), viewport,
        {geometryMode: 'solid_faces'});
    },
  });
}

// Side panels remain owned by the composition host. Bind only the ready
// relation attached to each live panel, never the primary view or a prior frame.
function space3SideRelations(state) {
  const supplied = authoredCandidate(state)?.artifact?.relations;
  return (Array.isArray(supplied) ? supplied : []).filter(relation =>
    relation?.kind === 'side' && relation.view?.schema === 'ddn.scene_space3_projection.v1'
      && relation.view.spatial_profile === 'space3');
}

export function hasSpace3SideView(state) {
  return space3SideRelations(state).length > 0;
}

function showSourceLocation(list, label, source) {
  const line = list.ownerDocument.createElement('li');
  const start = source?.start_line;
  const end = source?.end_line;
  const range = Number.isSafeInteger(start) && start > 0
    ? (Number.isSafeInteger(end) && end > start ? start + '–' + end + '줄' : start + '줄')
    : '줄 미공급';
  const path = typeof source?.path === 'string' && source.path ? source.path : '원문 경로 미공급';
  line.textContent = label + ' · ' + path + ' · ' + range;
  list.append(line);
}

export function mountSpace3SideViews(stage) {
  const views = new Map();
  function drop(panel, view) {
    view.display.dispose();
    view.time.remove();
    view.root.remove();
    view.details.remove();
    views.delete(panel);
  }
  function render(state) {
    const relations = new Map(space3SideRelations(state)
      .map(relation => [relation.relation_id, relation]));
    const active = new Set();
    for (const panel of stage.querySelectorAll('[data-authored-relation-id]')) {
      if (panel.hidden) continue;
      const relation = relations.get(panel.dataset.authoredRelationId);
      const original = panel.querySelector(':scope > canvas');
      if (!relation || !original) continue;
      active.add(panel);
      original.hidden = true;
      let view = views.get(panel);
      if (!view) {
        const root = stage.ownerDocument.createElement('section');
        root.dataset.space3SideView = 'true';
        root.style.height = 'clamp(160px, 28vh, 320px)';
        const time = stage.ownerDocument.createElement('p');
        const details = stage.ownerDocument.createElement('details');
        const summary = stage.ownerDocument.createElement('summary');
        summary.textContent = '공급된 3D 곁 출처';
        const provenance = stage.ownerDocument.createElement('pre');
        provenance.style.whiteSpace = 'pre-wrap';
        provenance.style.overflowWrap = 'anywhere';
        const locations = stage.ownerDocument.createElement('ul');
        locations.setAttribute('aria-label', '3D 원문 위치');
        locations.style.overflowWrap = 'anywhere';
        const rawDetails = stage.ownerDocument.createElement('details');
        const rawSummary = stage.ownerDocument.createElement('summary');
        rawSummary.textContent = '공급 원문 자세히';
        rawDetails.append(rawSummary, provenance);
        details.append(summary, locations, rawDetails);
        panel.append(time, root, details);
        view = {root, time, details, locations, provenance, relation: null, source: null, display: null};
        view.display = mountSpace3FaceView(root, {readFrame: () => view.source});
        views.set(panel, view);
      }
      view.time.textContent = Number.isSafeInteger(relation.resolved_madi) ? '3D 보기 마디 ' + relation.resolved_madi : '';
      if (view.relation !== relation) {
        view.relation = relation;
        const faceSources = Array.isArray(relation.view.elements)
          ? relation.view.elements.filter(element => element?.kind === 'face').map(element => element.source)
          : undefined;
        view.locations.replaceChildren();
        faceSources?.forEach((source, index) => showSourceLocation(view.locations, '면 ' + (index + 1), source));
        showSourceLocation(view.locations, '카메라', relation.view.camera?.source);
        showSourceLocation(view.locations, '장면', relation.view.source);
        view.provenance.textContent = JSON.stringify({
          relation_id: relation.relation_id,
          identity: relation.identity,
          basis_location: relation.basis_location,
          resolved_madi: relation.resolved_madi,
          time_policy: relation.time_policy,
          source: relation.view.source,
          camera_source: relation.view.camera?.source,
          face_sources: faceSources,
        }, null, 2);
      }
      if (view.source !== relation.view) {
        view.source = relation.view;
        void view.display.refresh();
      }
    }
    for (const [panel, view] of views) if (!active.has(panel)) drop(panel, view);
  }
  return {render, dispose() { for (const [panel, view] of views) drop(panel, view); }};
}

