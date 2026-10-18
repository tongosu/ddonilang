// Presentation only. Identity, digest and candidate admission come from Rust.
// No world/resource fallback and no selection by label or candidate order.
function targetText(target) {
  if (target?.kind === "fixed64") {
    const raw=BigInt(target.raw), unit=4294967296n;
    return raw%unit===0n ? String(raw/unit) : `${raw}/4294967296`;
  }
  if (target?.kind === "list") return `[${target.values.map(targetText).join(", ")}]`;
  if (target?.kind === "pack") return `{${Object.entries(target.fields).map(([key,value])=>`${key}: ${targetText(value)}`).join(", ")}}`;
  if (target?.kind === "none") return "없음";
  if (target?.kind === "bool") return target.value ? "참" : "거짓";
  if (target?.kind === "text") return JSON.stringify(target.value);
  if (target?.kind === "scene_element" && target.element.kind === "circle") {
    // Readable, explicitly approximate UI text. The exact raw typed value is
    // preserved in the expandable record; never use this text for calculation.
    const text=raw=>String(Number((Number(BigInt(raw))/4294967296).toFixed(6)));
    return `중심 (x ≈ ${text(target.element.center[0])}, y ≈ ${text(target.element.center[1])}), 반지름 ≈ ${text(target.element.radius)}`;
  }
  return "대상 정보 없음";
}
// Local display hit testing of Rust-resolved, explicitly authored targets only.
// A six-pixel affordance makes small sample circles selectable; nearest wins,
// but an exact tie is ambiguous and never auto-selects by declaration order.
export function pickAuthoredTarget(view, relations, width, height, px, py, camera = null) {
  if (view?.kind!=="scene" || px<18 || py<18 || px>width-18 || py>height-18) return null;
  const zoom=camera?.zoom ?? 1;
  px=(px-width/2-(camera?.x ?? 0))/zoom+width/2;
  py=(py-height/2-(camera?.y ?? 0))/zoom+height/2;
  const [xmin,xmax]=view.horizontal.map(BigInt),[ymin,ymax]=view.vertical.map(BigInt);
  const sx=Number(xmax-xmin),sy=Number(ymax-ymin),scale=Math.min((width-36)/sx,(height-36)/sy);
  let best=null, distance=Infinity, tied=false;
  for (const relation of relations ?? []) {
    if (relation.kind!=="magnifier" || relation.selection?.kind!=="primary_scene_element.v1") continue;
    for (const index of relation.selection.element_indices) {
      const element=view.elements[index];
      if (element?.kind!=="circle") continue;
      const x=(width-sx*scale)/2+Number(BigInt(element.center[0])-xmin)*scale;
      const y=(height+sy*scale)/2-Number(BigInt(element.center[1])-ymin)*scale;
      const d=Math.hypot(px-x,py-y),radius=Math.max(6/zoom,Number(element.radius)*scale);
      if (d>radius) continue;
      if (d<distance-1e-9) {best=relation;distance=d;tied=false;}
      else if (Math.abs(d-distance)<1e-9 && best?.relation_id!==relation.relation_id) tied=true;
    }
  }
  return tied ? null : best;
}
function renderPlane(ctx,width,height,view,style = null) {
  // Conversion is display-only. Rust supplies exact geometry and authored bounds.
  const [xmin,xmax] = view.horizontal.map(BigInt), [ymin,ymax] = view.vertical.map(BigInt);
  const xspan=Number(xmax-xmin), yspan=Number(ymax-ymin);
  // Equal raw lengths have equal pixel lengths. Letterboxing preserves the
  // authored bounds; it is not fitting bounds to observed elements.
  const zoom=style?.camera?.zoom ?? 1;
  const scale=Math.min((width-36)/xspan,(height-36)/yspan)*zoom;
  const x = value => (width-xspan*scale)/2 + Number(BigInt(value)-xmin)*scale+(style?.camera?.x ?? 0);
  const y = value => (height+yspan*scale)/2-Number(BigInt(value)-ymin)*scale+(style?.camera?.y ?? 0);
  ctx.save(); ctx.beginPath(); ctx.rect(18,18,width-36,height-36); ctx.clip();
  ctx.strokeStyle=style?.color ?? "#215b8b"; ctx.fillStyle=style?.color ?? "#132018";
  ctx.lineWidth=style?.color ? 2 : 1.5;
  for (const element of view.elements) {
    if (element.kind === "circle") {
      ctx.beginPath(); ctx.ellipse(x(element.center[0]),y(element.center[1]),
        Number(element.radius)*scale,Number(element.radius)*scale,0,0,2*Math.PI);
      ctx.stroke();
    } else if (element.kind === "polygon") {
      ctx.beginPath();
      element.vertices.forEach((point,index) => {
        if (index===0) ctx.moveTo(x(point[0]),y(point[1]));
        else ctx.lineTo(x(point[0]),y(point[1]));
      });
      ctx.closePath(); ctx.stroke();
    } else if (element.kind === "text") {
      ctx.fillText(element.content,x(element.position[0]),y(element.position[1]));
    }
  }
  ctx.restore();
}

// Current observation is a local presentation filter. Keep every authored row
// from the latest committed madi (including multiple events in that madi).
// Tables without time metadata are not guessed to be time series.
export function observationRowsForDisplay(table, mode = "current") {
  const rows=table.rows ?? [];
  if(mode==="history" || !rows.length || rows.some(row=>!Number.isSafeInteger(row.committed_madi))) return rows;
  const latest=rows.reduce((madi,row)=>Math.max(madi,row.committed_madi),-1);
  return rows.filter(row=>row.committed_madi===latest);
}

// A3-P1: format only a typed Fixed64's exact canonical decimal. BigInt keeps
// ties-to-even exact; strings, rationals and integers are never reinterpreted.
export function observationCellDisplay(cell, column) {
  if (cell.status !== "present") return cell.reason ?? cell.status;
  if (column?.dtype !== "fixed64") return cell.value_text;
  const match=/^(-?)(\d+)(?:\.(\d+))?$/.exec(cell.value_canon ?? "");
  if (!match) return cell.value_text;
  const fraction=match[3] ?? "";
  const denominator=10n**BigInt(fraction.length);
  const exact=BigInt(match[2])*denominator+BigInt(fraction || "0");
  const scaled=exact*1000000n;
  let rounded=scaled/denominator;
  const remainder=scaled%denominator;
  if (remainder*2n>denominator || (remainder*2n===denominator && rounded%2n===1n)) rounded++;
  const digits=(rounded%1000000n).toString().padStart(6,"0").replace(/0+$/,"");
  return `${match[1] && rounded ? "-" : ""}${rounded/1000000n}${digits ? `.${digits}` : ""}`;
}

export function renderAuthoredBogae(ctx, width, height, wasmState, local = null) {
  const projection = wasmState?.bogae_projection;
  if (!projection) return null;
  const model = {family: "result_table", role: "primary",
    data_source: "rust_authored_bogae_projection", status: projection.status};
  ctx.font = "14px sans-serif";
  ctx.fillStyle = "#132018";
  if (projection.status !== "ready" || projection.candidates?.length !== 1) {
    model.diagnostic = projection.diagnostic ?? {code: "E_BOGAE_PROJECTION_INVALID"};
    ctx.fillStyle = "#9f2424";
    ctx.fillText(model.diagnostic.message ?? model.diagnostic.code, 18, 58, width - 36);
    return model;
  }
  const candidate = projection.candidates[0];
  const view = candidate.artifact?.view;
  const isPlane = view?.kind === "scene" && view.spatial_profile === "plane2";
  if (view?.kind !== "result_table" && !isPlane) {
    model.status = "unsupported";
    model.diagnostic = {code: "E_BOGAE_VIEW_UNSUPPORTED"};
    ctx.fillText("이 보기의 표시를 지원하지 않습니다.", 18, 58, width - 36);
    return model;
  }
  model.authored_digest = candidate.authored_digest;
  model.identity = candidate.artifact.identity;
  model.relations = candidate.artifact.relations;
  if (isPlane) {
    model.family = "scene"; model.spatial_profile="plane2";
    model.element_count=view.elements.length;
    renderPlane(ctx,width,height,view,{...local?.overlayStyle,camera:local?.camera});
    const overlays=(model.relations ?? []).filter(r=>r.kind==="overlay"&&!local?.hidden?.has(r.relation_id)).sort((a,b)=>a.order-b.order);
    model.active_overlay_ids=overlays.map(r=>r.relation_id);
    for (const overlay of overlays) {
      const overlayState={...wasmState,bogae_projection:{...projection,candidates:[{...candidate,
        artifact:{...candidate.artifact,view:overlay.view,relations:[]}}]}};
      renderAuthoredBogae(ctx,width,height,overlayState,{...local,overlayStyle:{color:"#b45309"}});
    }
    // Selection is local presentation state, never a new world element.
    const selected=(model.relations ?? []).filter(r=>local?.selectedRelationIds?.has(r.relation_id));
    model.selected_object_ids=selected.map(r=>r.object_identity?.id ?? r.target_digest);
    const [xmin,xmax]=view.horizontal.map(BigInt),[ymin,ymax]=view.vertical.map(BigInt);
    const sx=Number(xmax-xmin),sy=Number(ymax-ymin),s=Math.min((width-36)/sx,(height-36)/sy)*(local?.camera?.zoom ?? 1);
    ctx.save();ctx.strokeStyle="#e6a600";ctx.lineWidth=3;
    for (const relation of selected) for (const index of relation.selection?.element_indices ?? []) {
      const circle=view.elements[index];if (circle?.kind!=="circle") continue;
      ctx.beginPath();ctx.ellipse((width-sx*s)/2+Number(BigInt(circle.center[0])-xmin)*s+(local?.camera?.x ?? 0),
        (height+sy*s)/2-Number(BigInt(circle.center[1])-ymin)*s+(local?.camera?.y ?? 0),
        Math.max(6,Number(circle.radius)*s)+4,Math.max(6,Number(circle.radius)*s)+4,0,0,2*Math.PI);ctx.stroke();
    }
    ctx.restore();
    return model;
  }
  const tableProjection = view.table_projection ?? (candidate.artifact?.resolved_madi === undefined
    ? wasmState.public_observation_table_projection : null);
  if (!tableProjection && candidate.artifact?.resolved_madi !== undefined) {
    model.status = "invalid";
    model.diagnostic = {code:"E_BOGAE_TIME_OUT_OF_RANGE"};
    ctx.fillText("지정 마디의 결과표가 없습니다.",18,58,width-36);
    return model;
  }
  const tables = tableProjection?.tables ?? [];
  model.table_projection_hash = tableProjection?.projection_hash ?? null;
  model.row_count = tables.reduce((sum, table) => sum + table.rows.length, 0);
  let y = 58;
  for (const table of tables) {
    const columns = table.columns.map(column => column.label ?? column.name ?? "");
    ctx.font = "600 14px sans-serif";
    ctx.fillText(columns.join(" | "), 18, y, width - 36);
    y += 24;
    ctx.font = "14px sans-serif";
    for (const row of table.rows) {
      if (y > height - 24) break;
      const cells = row.cells.map((cell,index) => observationCellDisplay(cell,table.columns[index]));
      ctx.fillText(cells.join(" | "), 18, y, width - 36);
      y += 22;
    }
  }
  if (!model.row_count) ctx.fillText("이 실행에는 공개 관찰 결과가 없습니다.", 18, y, width - 36);
  const overlays = (model.relations ?? []).filter(relation => relation.kind === "overlay")
    .filter(relation => local?.hidden?.has(relation.relation_id) !== true)
    .sort((a,b) => a.order - b.order);
  model.active_overlay_ids = overlays.map(relation => relation.relation_id);
  for (const relation of overlays) {
    if (relation.view?.kind === "scene") {
      const overlayState={...wasmState,bogae_projection:{...projection,candidates:[{...candidate,
        artifact:{...candidate.artifact,view:relation.view,relations:[]}}]}};
      renderAuthoredBogae(ctx,width,height,overlayState);
      continue;
    }
    // Result-table overlays are tables, not invented world-space vectors.
    // Their placement/colour is a local display-plane choice.
    ctx.fillStyle = "#215b8b";
    const overlayTables = relation.view?.table_projection?.tables ??
      (relation.resolved_madi === undefined ? tables : []);
    const rows = overlayTables.flatMap(table => table.rows.map(row => row.cells.map((cell,index) => observationCellDisplay(cell,table.columns[index])).join(" | ")));
    ctx.fillText(`겹 · ${relation.authored_name ?? "추가 표시"}: ${rows.join(" / ") || "공개 관찰 없음"}`, 18, Math.min(height - 18, y + 22), width - 36);
    y += 22;
  }
  return model;
}

// Local lifecycle only. All authored relation identities are supplied by Rust.
// Hiding a side retains the same DOM/canvas and relation-local state.
export function createAuthoredCompositionHost(onChange) {
  let scope = null;
  let controls = null;
  let hidden = new Set();
  let sidePanels = new Map();
  let magnifierPanels = new Map();
  let docks = new Map();
  let layoutStage = null;
  const edgeHandles = new Map();
  let pickCanvas=null, pickHandler=null;
  let cameraCleanup=null;
  let camera={zoom:1,x:0,y:0}, cameraControls=null;
  function reset() {
    cameraCleanup?.(); cameraCleanup=null;
    if (pickCanvas && pickHandler) pickCanvas.removeEventListener("click",pickHandler);
    pickCanvas=null;pickHandler=null;
    controls?.remove(); controls = null;
    cameraControls?.remove(); cameraControls=null; camera={zoom:1,x:0,y:0};
    for (const panel of sidePanels.values()) panel.remove();
    for (const {panel} of magnifierPanels.values()) panel.remove();
    magnifierPanels = new Map();
    for (const dock of docks.values()) dock.remove();
    for (const handle of edgeHandles.values()) handle.remove();
    edgeHandles.clear();
    docks = new Map();
    if (layoutStage) {
      delete layoutStage.dataset.authoredLayout;
      delete layoutStage.dataset.authoredPrimaryKind;
      layoutStage.style.removeProperty("--authored-left");
      layoutStage.style.removeProperty("--authored-right");
      layoutStage = null;
    }
    sidePanels = new Map(); hidden = new Set(); scope = null;
  }
  return {
    reset,
    closeAllSides() {
      if (!scope) return false;
      for (const [id,panel] of sidePanels) { hidden.add(id); panel.hidden=true; }
      onChange();
      return true;
    },
    get hidden() { return hidden; },
    get camera() { return camera; },
    get selectedRelationIds() { return new Set(magnifierPanels.keys()); },
    render({projectId, wasmState, list, stage, toolbar, canvas:primaryCanvas}) {
      const projection = wasmState?.bogae_projection;
      if (projection?.status !== "ready" || projection.candidates?.length !== 1) { reset(); return null; }
      const candidate = projection.candidates[0];
      // Geometry may change at every committed madi; that does not replace an
      // authored relation instance. Only Rust-authored identity scopes local state.
      const nextScope = `${projectId}:${JSON.stringify(candidate.artifact.identity)}`;
      if (scope !== nextScope) {
        reset(); scope = nextScope;
        layoutStage = stage;
        stage.dataset.authoredLayout = "true";
        controls = document.createElement("div");
        controls.dataset.authoredRelationControls = "true";
        controls.tabIndex=-1;
        // Reopen controls must remain reachable when the side pane is hidden.
        toolbar.append(controls);
        cameraControls=document.createElement('details');
        cameraControls.dataset.authoredCameraControls='true';
        const cameraSummary=document.createElement('summary');cameraSummary.textContent='화면';cameraControls.append(cameraSummary);
        const cameraActions=document.createElement('div');cameraActions.className='authored-camera-actions';cameraControls.append(cameraActions);
        const operations=[['＋','확대',()=>camera.zoom=Math.min(8,camera.zoom*1.25)],['−','축소',()=>camera.zoom=Math.max(0.25,camera.zoom/1.25)],
          ['전체','원문 범위로 돌아가기',()=>camera={zoom:1,x:0,y:0}],
          ['←','왼쪽으로 보기 이동',()=>camera.x-=40],['→','오른쪽으로 보기 이동',()=>camera.x+=40],
          ['↑','위로 보기 이동',()=>camera.y-=40],['↓','아래로 보기 이동',()=>camera.y+=40]];
        for(const [label,title,change] of operations){const button=document.createElement('button');button.type='button';button.textContent=label;
          button.title=title;button.setAttribute('aria-label',title);button.addEventListener('click',()=>{change();onChange();});cameraActions.append(button);}
        toolbar.append(cameraControls);
        list.replaceChildren();
      }
      // Reconcile Rust-supplied relation identities on every committed frame.
      stage.dataset.authoredPrimaryKind=candidate.artifact?.view?.kind ?? "";
      cameraControls.hidden=candidate.artifact?.view?.kind!=='scene';
      // Preserve local state only for identities that still exist in this scope.
      const relations = candidate.artifact.relations ?? [];
      const relationById = new Map(relations.map(relation => [relation.relation_id, relation]));
      if (primaryCanvas && pickCanvas!==primaryCanvas) {
        if (pickCanvas && pickHandler) pickCanvas.removeEventListener("click",pickHandler);
        cameraCleanup?.();
        pickCanvas=primaryCanvas;
        const canvas=primaryCanvas;
        let drag=null;
        const down=event=>{
          if(event.button!==1 || layoutStage?.dataset.authoredPrimaryKind!=='scene')return;
          event.preventDefault();drag={pointer:event.pointerId,x:event.clientX,y:event.clientY};
          canvas.setPointerCapture(event.pointerId);
        };
        const move=event=>{
          if(drag?.pointer!==event.pointerId)return;
          const rect=canvas.getBoundingClientRect();
          camera.x+=(event.clientX-drag.x)*canvas.width/rect.width;
          camera.y+=(event.clientY-drag.y)*canvas.height/rect.height;
          drag.x=event.clientX;drag.y=event.clientY;onChange();
        };
        const end=event=>{if(drag?.pointer===event.pointerId){drag=null;if(canvas.hasPointerCapture(event.pointerId))canvas.releasePointerCapture(event.pointerId);}};
        const wheel=event=>{
          if(layoutStage?.dataset.authoredPrimaryKind!=='scene')return;
          event.preventDefault();camera.zoom=Math.max(0.25,Math.min(8,camera.zoom*Math.exp(-event.deltaY*0.001)));onChange();
        };
        const aux=event=>{if(event.button===1)event.preventDefault();};
        canvas.addEventListener('pointerdown',down);canvas.addEventListener('pointermove',move);
        canvas.addEventListener('pointerup',end);canvas.addEventListener('pointercancel',end);
        canvas.addEventListener('wheel',wheel,{passive:false});canvas.addEventListener('auxclick',aux);
        cameraCleanup=()=>{
          if(drag && canvas.hasPointerCapture(drag.pointer))canvas.releasePointerCapture(drag.pointer);
          drag=null;
          canvas.removeEventListener('pointerdown',down);canvas.removeEventListener('pointermove',move);
          canvas.removeEventListener('pointerup',end);canvas.removeEventListener('pointercancel',end);
          canvas.removeEventListener('wheel',wheel);canvas.removeEventListener('auxclick',aux);
        };
        pickHandler=event=>pickCanvas.selectAuthoredTarget(event);
        pickCanvas.addEventListener("click",pickHandler);
      }
      if (pickCanvas) {
        pickCanvas.selectAuthoredTarget=event=>{
          const rect=pickCanvas.getBoundingClientRect();
          const picked=pickAuthoredTarget(candidate.artifact.view,relations,pickCanvas.width,pickCanvas.height,
            (event.clientX-rect.left)*pickCanvas.width/rect.width,(event.clientY-rect.top)*pickCanvas.height/rect.height,camera);
          if (!picked) return;
          const openPicked=()=>{
            const button=[...controls.querySelectorAll("button")].find(button=>button.dataset.authoredRelationToggle===picked.relation_id);
            if (!button) return;
            if (magnifierPanels.has(picked.relation_id)) return;
            for (const {panel} of magnifierPanels.values()) panel.remove();
            magnifierPanels.clear();
            button.openMagnifier(); onChange();
          };
          openPicked();
        };
      }
      // Same authored object, changing current properties: update the existing
      // instance. Different object/recipe/removal: terminate, never retarget.
      for (const [id, instance] of magnifierPanels) {
        const relation=relationById.get(id);
        if (relation?.kind!=="magnifier" || instance.target!==(relation.object_identity?.id ?? relation.target_digest)
            || instance.recipe!==relation.recipe?.seal) {
          if (instance.panel.contains?.(document.activeElement)) controls.focus();
          instance.panel.remove(); magnifierPanels.delete(id);
        }
      }
      for (const [id, panel] of sidePanels) {
        if (relationById.get(id)?.kind !== "side") { panel.remove(); sidePanels.delete(id); }
      }
      for (const button of controls.querySelectorAll("button")) {
        const relation = relationById.get(button.dataset.authoredRelationToggle);
        if (!relation || relation.kind !== button.dataset.relationKind) button.remove();
      }
      for (const id of hidden) if (!relationById.has(id)) hidden.delete(id);
      for (const relation of relations) {
          if (relation.kind === "magnifier") {
            let button=[...controls.querySelectorAll("button")].find(node=>node.dataset.authoredRelationToggle===relation.relation_id);
            if (!button) {
              button=document.createElement("button"); button.type="button";
              button.dataset.authoredRelationToggle=relation.relation_id; button.dataset.relationKind="magnifier";
              button.textContent="돋 · 대상 보기";
              button.addEventListener("click",()=>{
                const old=magnifierPanels.get(relation.relation_id);
                if (old) {old.panel.remove(); magnifierPanels.delete(relation.relation_id);}
                else {
                  // The handler is rebound below each frame so it sees the
                  // currently committed target, never its creation-time value.
                  button.openMagnifier();
                }
                onChange();
              });
              controls.append(button);
            }
            // Spatial targets are opened by choosing the primary element, not
            // a button that silently chooses a preassigned target.
            button.hidden=relation.selection?.kind==="primary_scene_element.v1";
            button.openMagnifier=()=>{
              const panel=document.createElement("section"); panel.className="side-view-instance";
              panel.dataset.authoredMagnifierId=relation.relation_id;
              panel.dataset.targetDigest=relation.target_digest;
              const title=document.createElement("strong"); title.textContent="돋보기 · 선택한 대상";
              const close=document.createElement("button"); close.type="button"; close.textContent="닫기";
              close.dataset.authoredMagnifierClose=relation.relation_id;
              close.addEventListener("click",()=>{
                // Removing a focused close button must not move focus outside
                // the logical-input region and falsely report lost input.
                (relation.selection?.kind==="primary_scene_element.v1" ? pickCanvas : button)?.focus?.();
                panel.remove(); magnifierPanels.delete(relation.relation_id);onChange();
              });
              const canvas=document.createElement("canvas"); canvas.width=220;canvas.height=120;
              const target=document.createElement("pre"); target.dataset.authoredMagnifierTarget="true";
              // Display only the exact typed Rust target; never read world data
              // again or invoke the recipe in JavaScript.
              target.textContent=JSON.stringify(relation.target,null,2);
              const value=document.createElement("p"); value.dataset.authoredMagnifierValue="true";
              value.textContent=`대상 값: ${targetText(relation.target)}`;
              const details=document.createElement("details"), summary=document.createElement("summary");
              summary.textContent="정확한 대상 기록"; details.append(summary,target);
              panel.append(title,close,canvas,value,details);
              const spatial=relation.selection?.kind==="primary_scene_element.v1";
              if (spatial) {
                panel.className="authored-magnifier-popover";
                (pickCanvas?.parentElement ?? pickCanvas?.parent ?? stage).append(panel);
              } else {
                let dock=docks.get("right");
                if (!dock) {dock=document.createElement("div");dock.className="authored-side-dock";
                  dock.dataset.authoredDock="right";stage.append(dock);docks.set("right",dock);}
                dock.append(panel);
              }
              magnifierPanels.set(relation.relation_id,{panel,target:relation.object_identity?.id ?? relation.target_digest,recipe:relation.recipe.seal});
            };
            continue;
          }
          if (!["side","overlay"].includes(relation.kind)) continue;
          let button = [...controls.querySelectorAll("button")].find(node =>
            node.dataset.authoredRelationToggle === relation.relation_id);
          if (!button) {
          button = document.createElement("button");
          button.type = "button"; button.dataset.authoredRelationToggle = relation.relation_id;
          button.dataset.relationKind = relation.kind;
          button.textContent = `${relation.kind === "side" ? "곁" : "겹"} · ${relation.authored_name ?? (relation.placement ?? relation.order)}`;
          button.addEventListener("click", () => {
            if (hidden.has(relation.relation_id)) hidden.delete(relation.relation_id);
            else hidden.add(relation.relation_id);
            onChange();
          });
          controls.append(button);
          }
          button.textContent = `${relation.kind === "side" ? "곁" : "겹"} · ${relation.authored_name ?? (relation.placement ?? relation.order)}`;
          if (relation.kind !== "side") continue;
          let panel = sidePanels.get(relation.relation_id);
          if (!panel) {
          panel = document.createElement("section");
          panel.className = "side-view-instance"; panel.dataset.authoredRelationId = relation.relation_id;
          panel.dataset.placement = relation.placement;
          const title = document.createElement("strong"); title.textContent = `곁보기 · ${relation.placement}`;
          const close = document.createElement("button"); close.type="button"; close.textContent="닫기";
          close.dataset.authoredSideClose=relation.relation_id;
          close.addEventListener("click",()=>{
            hidden.add(relation.relation_id);
            panel.hidden=true;
            button.focus();
            onChange();
          });
          const canvas = document.createElement("canvas"); canvas.width=360; canvas.height=220;
          const header=document.createElement("div"); header.className="side-view-instance-header";
          const history=document.createElement("button");history.type="button";history.hidden=true;
          history.dataset.authoredTableMode="current";
          panel.authoredTableModeButton=history;
          history.addEventListener("click",()=>{
            history.dataset.authoredTableMode=history.dataset.authoredTableMode==="current"?"history":"current";
            onChange();
          });
          const table=document.createElement("div");table.className="authored-side-table";
          table.dataset.testid="authored-side-table";table.hidden=true;
          panel.authoredTableElement=table;
          header.append(title,history,close); panel.append(header,canvas,table); sidePanels.set(relation.relation_id,panel);
          }
          panel.dataset.placement = relation.placement;
          panel.querySelector("strong").textContent = `곁 · ${relation.authored_name ?? relation.placement}`;
          // RD-134 owns authored position hints; pixel sizes and the Auto choice
          // are local layout. Auto currently uses the right dock, not a new role.
          const dockPosition = {"왼쪽":"left","오른쪽":"right","위":"top","아래":"bottom","자동":"right"}[relation.placement];
          let dock = docks.get(dockPosition);
          if (!dock) {
            dock = document.createElement("div");
            dock.className = "authored-side-dock";
            dock.dataset.authoredDock = dockPosition;
            stage.append(dock); docks.set(dockPosition,dock);
          }
          // Re-inserting a live panel between pointer-down and pointer-up can
          // cancel its Close click. Keep the same node attached across madis.
          if ((panel.parentElement ?? panel.parent) !== dock) dock.append(panel);
      }
      let sideCount = 0;
      for (const [id,panel] of sidePanels) {
        panel.hidden = hidden.has(id);
        if (panel.hidden) continue;
        sideCount++;
        const canvas = panel.querySelector("canvas"), ctx = canvas.getContext("2d");
        ctx.clearRect(0,0,canvas.width,canvas.height);
        ctx.fillStyle="#f8fbf7"; ctx.fillRect(0,0,canvas.width,canvas.height);
        const sideRelation=relationById.get(id);
        const tableElement=panel.authoredTableElement;
        const isTable=sideRelation.view?.kind === "result_table";
        const history=panel.authoredTableModeButton;
        history.hidden=!isTable;
        panel.dataset.authoredViewKind=sideRelation.view.kind;
        canvas.hidden=isTable;tableElement.hidden=!isTable;
        if(isTable) {
          // A narrow side is a readable field/value list, not a nine-column
          // bitmap compressed to fit. Use only Rust's authored projection.
          // Typed exact display rounding only; no resource fallback or edits to evidence.
          tableElement.replaceChildren();
          const tableProjection=sideRelation.view.table_projection ??
            (sideRelation.resolved_madi===undefined ? wasmState.public_observation_table_projection : null);
          const mode=history.dataset.authoredTableMode;
          const total=(tableProjection?.tables ?? []).reduce((count,table)=>count+(table.rows?.length??0),0);
          history.textContent=mode==="current"?`기록 보기 (${total})`:"현재 값";
          history.setAttribute("aria-pressed",String(mode==="history"));
          tableElement.dataset.displayMode=mode;
          tableElement.dataset.totalRows=String(total);
          if(!tableProjection) {
            tableElement.textContent="이 시점의 결과표가 없습니다.";
          } else {
            for(const table of tableProjection.tables ?? []) {
              for(const row of observationRowsForDisplay(table,mode)) {
                const group=document.createElement("dl");
                if(row.committed_madi!==undefined) group.dataset.committedMadi=String(row.committed_madi);
                for(let index=0;index<row.cells.length;index++) {
                  const label=document.createElement("dt"),value=document.createElement("dd");
                  const column=table.columns[index],cell=row.cells[index];
                  label.textContent=column?.label ?? column?.name ?? String(index+1);
                  value.textContent=observationCellDisplay(cell,column);
                  group.append(label,value);
                }
                tableElement.append(group);
              }
            }
          }
          tableElement.dataset.committedStateHash=wasmState.state_hash ?? "";
          canvas.dataset.committedStateHash=wasmState.state_hash ?? "";
          continue;
        }
        const sideState = {...wasmState,bogae_projection:{...projection,candidates:[{...candidate,
          artifact:{...candidate.artifact,view:sideRelation.view,relations:[]}}]}};
        renderAuthoredBogae(ctx,canvas.width,canvas.height,sideState);
        canvas.dataset.committedStateHash = wasmState.state_hash ?? "";
      }
      for (const button of controls.querySelectorAll("button")) {
        button.setAttribute("aria-pressed",String(button.dataset.relationKind==="magnifier"
          ? magnifierPanels.has(button.dataset.authoredRelationToggle) : !hidden.has(button.dataset.authoredRelationToggle)));
      }
      for (const [id,{panel}] of magnifierPanels) {
        const relation=relationById.get(id);
        panel.dataset.targetDigest=relation.target_digest;
        panel.dataset.objectIdentity=relation.object_identity?.id ?? "";
        panel.querySelector("strong").textContent=relation.object_identity ? `돋 · ${relation.object_identity.label}` : "돋보기 · 선택한 대상";
        panel.querySelector("pre").textContent=JSON.stringify(relation.target,null,2);
        panel.querySelector("p").textContent=`현재 값: ${targetText(relation.target)}`;
        const canvas=panel.querySelector("canvas"),ctx=canvas.getContext("2d");
        ctx.clearRect(0,0,canvas.width,canvas.height);ctx.fillStyle="#f8fbf7";ctx.fillRect(0,0,canvas.width,canvas.height);
        renderAuthoredBogae(ctx,canvas.width,canvas.height,{...wasmState,bogae_projection:{...projection,
          candidates:[{...candidate,artifact:{...candidate.artifact,view:relation.view,relations:[]}}]}});
        canvas.dataset.committedStateHash=wasmState.state_hash??"";
        if (pickCanvas && relation.selection?.element_indices?.length) {
          const view=candidate.artifact.view, circle=view.elements[relation.selection.element_indices[0]];
          const [xmin,xmax]=view.horizontal.map(BigInt),[ymin,ymax]=view.vertical.map(BigInt);
          const sx=Number(xmax-xmin),sy=Number(ymax-ymin),s=Math.min((pickCanvas.width-36)/sx,(pickCanvas.height-36)/sy)*camera.zoom;
          const rect=pickCanvas.getBoundingClientRect(),parent=pickCanvas.parentElement?.getBoundingClientRect?.() ?? rect;
          const x=(pickCanvas.width-sx*s)/2+Number(BigInt(circle.center[0])-xmin)*s+camera.x;
          const y=(pickCanvas.height+sy*s)/2-Number(BigInt(circle.center[1])-ymin)*s+camera.y;
          panel.style.left=`${rect.left-parent.left+Math.max(4,Math.min(rect.width-240,x*rect.width/pickCanvas.width+12))}px`;
          panel.style.top=`${rect.top-parent.top+Math.max(4,Math.min(rect.height-210,y*rect.height/pickCanvas.height-60))}px`;
        }
      }
      for (const [position,dock] of docks) {
        dock.hidden = [...dock.children].every(panel => panel.hidden);
        const sides=[...dock.children].filter(panel=>panel.dataset.authoredRelationId);
        let handle=edgeHandles.get(position);
        if(sides.length && !handle) {
          handle=document.createElement('button');handle.type='button';
          handle.dataset.authoredSideEdge=position;
          handle.addEventListener('click',()=>{
            const panels=[...dock.children].filter(panel=>panel.dataset.authoredRelationId);
            const open=panels.every(panel=>hidden.has(panel.dataset.authoredRelationId));
            for(const panel of panels) {
              if(open)hidden.delete(panel.dataset.authoredRelationId);
              else hidden.add(panel.dataset.authoredRelationId);
            }
            onChange();
          });
          stage.append(handle);edgeHandles.set(position,handle);
        }
        if(handle) {
          // The edge handle opens a hidden dock. An open dock already has
          // per-panel Close controls; do not cover its content with a duplicate.
          handle.hidden=!sides.length || !dock.hidden;
          const names=sides.map(panel=>relationById.get(panel.dataset.authoredRelationId)?.authored_name ?? '곁').join(' · ');
          const arrow={left:'›',right:'‹',top:'⌄',bottom:'⌃'}[position];
          handle.textContent=`${arrow} ${names}`;
          handle.setAttribute('aria-expanded',String(!dock.hidden));
          handle.setAttribute('aria-label',`${names} ${dock.hidden?'열기':'닫기'}`);
        }
        if (position === "left" || position === "right") {
          stage.style.setProperty(`--authored-${position}`,dock.hidden ? "0px" : "minmax(140px, 0.5fr)");
        }
      }
      stage.dataset.sideOpen = sideCount ? "true" : "false";
      stage.dataset.sideInstanceCount = String(sideCount);
      stage.dataset.overlayInstanceCount = String((candidate.artifact.relations ?? []).filter(r=>r.kind==="overlay"&&!hidden.has(r.relation_id)).length);
      stage.dataset.magnifierInstanceCount=String(magnifierPanels.size);
      return {sideCount, relationCount:(candidate.artifact.relations ?? []).length};
    },
  };
}
