import {mountPngBytePreview} from './png_byte_preview.mjs';

const object = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const hex = value => typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);
const refused = new Set(['rejected', 'invalid', 'failed', 'error']);
const sourceText = value => JSON.stringify(value, null, 2);

// Local host connection only. readState supplies an already current-source/run
// fenced snapshot (or null), as it does for the selected table. The byte reader
// must use that exact execution's checked registry. This module never selects
// another frame, loads assets from a URL, issues identity or grants admission.
export function mountPngSceneView(root, {readState, readVerifiedBytes, subscribeState,
  decode, maxViews = 256, maxImages = 256, maxBytes = 16 * 1024 * 1024} = {}) {
  if (!root?.ownerDocument || typeof readState !== 'function' || typeof readVerifiedBytes !== 'function'
    || (subscribeState != null && typeof subscribeState !== 'function')
    || ![maxViews,maxImages,maxBytes].every(n => Number.isSafeInteger(n) && n > 0))
    throw new TypeError('현재 보기와 검증된 그림 바이트를 공급할 연결이 필요합니다.');
  const document = root.ownerDocument, owner = document.createElement('section');
  owner.dataset.pngSceneView = 'true';owner.style.minWidth = '0';root.append(owner);
  const notice = document.createElement('p');notice.setAttribute('role','status');
  notice.setAttribute('aria-live','polite');owner.append(notice);
  const body = document.createElement('div');owner.append(body);
  let generation = 0, disposed = false, controller = null, previews = [], unsubscribe = null;
  const node = (parent,tag,text) => {
    const element = document.createElement(tag);if (text !== undefined) element.textContent = text;
    parent.append(element);return element;
  };
  function outcome(kind,message) {owner.dataset.pngSceneState = kind;notice.textContent = message;return kind;}
  function retire() {
    generation++;controller?.abort();controller = null;
    previews.splice(0).forEach(preview => preview.dispose());body.replaceChildren();
  }
  function viewsFrom(state) {
    const projection = state?.bogae_projection;
    if (!projection) return {kind:'not_produced',message:'현재 원문의 그림 공급 대기 · NOT_PRODUCED'};
    if (projection.status !== 'ready' || projection.diagnostic != null)
      return {kind:projection.diagnostic != null || refused.has(projection.status) ? 'rejected' : 'unsupported',
        message:`공급 상태: ${projection.status ?? '미공급'}`,diagnostic:projection.diagnostic};
    if (!Array.isArray(projection.candidates) || projection.candidates.length !== 1)
      return {kind:'unsupported',message:'하나의 실제 보기 후보가 필요합니다.'};
    const candidate = projection.candidates[0], artifact = candidate?.artifact;
    if (!object(artifact)) return {kind:'not_produced',message:'현재 보기 자료가 없습니다. · NOT_PRODUCED'};
    if (artifact.relations != null && !Array.isArray(artifact.relations))
      return {kind:'unsupported',message:'관계 보기 자료 구조를 지원하지 않습니다.'};
    const views = [{view:artifact.view,relation:null,label:'기본 보기'}];
    for (const relation of artifact.relations ?? []) views.push({view:relation?.view,relation,
      label:`관계 보기: ${relation?.authored_name ?? relation?.kind ?? '이름 미공급'}`});
    if (views.length > maxViews) return {kind:'unsupported',message:'한 화면의 보기 개수 한도를 넘었습니다.'};
    return {candidate,views};
  }
  async function refresh() {
    if (disposed) return null;
    retire();const request = generation;controller = new AbortController();
    const signal = controller.signal, current = () => !disposed && request === generation && !signal.aborted;
    outcome('loading','현재 원문의 그림을 읽는 중입니다. 이전 그림은 폐기했습니다.');
    let state;
    try {state = await readState();}
    catch (error) {if (!current()) return null;return outcome('rejected',`현재 보기 읽기 실패: ${error?.message ?? String(error)}`);}
    if (!current()) return null;
    const result = viewsFrom(state);
    if (!result.views) {
      if (result.diagnostic != null) {
        const raw = node(body,'pre',sourceText(result.diagnostic));raw.style.whiteSpace='pre-wrap';raw.style.overflowWrap='anywhere';
      }
      return outcome(result.kind,result.message);
    }
    const results = [];let totalBytes = 0, images = 0;
    for (const entry of result.views) {
      if (!current()) return null;
      const section = node(body,'section');node(section,'h3',entry.label);
      const status = node(section,'p'), view = entry.view;
      const finish = (kind,text) => {section.dataset.pngSceneState=kind;status.textContent=text;results.push(kind);};
      if (view == null) {finish('not_produced','이 보기의 그림 자료는 미공급입니다. · NOT_PRODUCED');continue;}
      if (view.kind !== 'scene' || view.spatial_profile !== 'plane2' || !Array.isArray(view.elements)) {
        finish('unsupported','이 보기의 그림 공간 표시는 지원하지 않습니다.');continue;
      }
      const elements = view.elements.filter(element => element?.kind === 'image');
      if (!elements.length) {finish('not_produced','이 보기에는 그림 요소가 없습니다. · NOT_PRODUCED');continue;}
      images += elements.length;
      if (images > maxImages) {finish('unsupported','한 화면의 그림 개수 한도를 넘었습니다.');continue;}
      status.textContent='이 보기의 검증된 그림 바이트를 읽는 중입니다.';
      try {
        const record = sourceText({candidate:result.candidate,relation:entry.relation,view});
        const displayView=JSON.parse(sourceText(view));
        const displayElements=displayView.elements.filter(element=>element?.kind==='image');
        const layers = [];
        for (const [index,element] of displayElements.entries()) {
          const asset = element.asset;
          if (!object(asset) || !hex(asset.owner) || !hex(asset.identity) || !hex(asset.byte_sha256)
            || !Number.isSafeInteger(asset.version) || asset.version < 1
            || !Number.isSafeInteger(asset.byte_size) || asset.byte_size < 1 || asset.mime !== 'image/png'
            || typeof asset.resource_handle !== 'string' || !/^[0-9a-f]{16}$/.test(asset.resource_handle))
            throw new Error('그림의 실제 자산·판본·바이트 표시 자료가 불완전합니다.');
          if (asset.byte_size > maxBytes-totalBytes) throw new Error('현재 화면의 전체 그림 바이트 한도를 넘었습니다.');
          const supplied = await readVerifiedBytes({state,candidate:result.candidate,view,
            relation:entry.relation,element:elements[index],signal});
          if (!current()) return null;
          if (!(supplied instanceof Uint8Array) || supplied.length !== asset.byte_size)
            throw new Error('이 보기의 공급 바이트 크기가 실제 자산 자료와 다릅니다.');
          const bytes = new Uint8Array(supplied);totalBytes += bytes.length;
          // Transport consistency with supplied bytes only, never an asset
          // identity/seal/rights grant. Rust admission and owner checks remain.
          if (!globalThis.crypto?.subtle) throw new Error('이 환경에서 공급 바이트 일치를 확인할 수 없습니다.');
          const digest = await globalThis.crypto.subtle.digest('SHA-256',bytes);
          if (!current()) return null;
          if (Array.from(new Uint8Array(digest),n=>n.toString(16).padStart(2,'0')).join('') !== asset.byte_sha256)
            throw new Error('이 보기의 공급 바이트가 실제 자산 해시와 다릅니다.');
          layers.push({element,bytes,horizontal:element.horizontal,vertical:element.vertical});
        }
        if (!current()) return null;
        const preview = mountPngBytePreview(section,{decode,maxBytes,maxImages});previews.push(preview);
        const shown = await preview.show(layers,{plane:displayView,scene:true,signal});
        if (!current()) return null;
        const details=node(section,'details');node(details,'summary','이 보기의 공급 원자료');
        const raw=node(details,'pre',record);raw.style.whiteSpace='pre-wrap';raw.style.overflowWrap='anywhere';
        finish(shown ?? 'not_produced',shown === 'ready' ? '이 보기의 그림을 공급 순서대로 표시했습니다.' : '이 보기의 그림 표시 상태를 확인하세요.');
      } catch (error) {
        if (!current()) return null;
        finish('rejected',`그림 바이트 표시 거부: ${error?.message ?? String(error)}`);
      }
    }
    if (!current()) return null;
    const kinds=new Set(results),kind=kinds.size===1 ? results[0] : 'mixed';
    return outcome(kind ?? 'not_produced','보기별 그림 공급·표시 상태를 확인하세요.');
  }
  function dispose() {
    if (disposed) return;retire();disposed=true;unsubscribe?.();unsubscribe=null;owner.remove();
  }
  try {
    if (subscribeState) {
      unsubscribe=subscribeState(()=>{void refresh();});
      if (unsubscribe != null && typeof unsubscribe !== 'function') throw new TypeError('보기 구독 해제 함수가 필요합니다.');
    }
  } catch (error) {retire();disposed=true;owner.remove();throw error;}
  void refresh();
  return {refresh,dispose};
}
