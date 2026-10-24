// Existing committed-view canvas presentation; world state, payloads and drawing bodies unchanged.
export function createCanvasPresentation({ activeFrame, activeRun, authoredCompositionHost, buildCanonicalViewPayloadSet, buildCommittedSemanticViewPayload, buildCommittedSpace2dOverlay, committedLogicalSpace2d, currentLogicalPresentation, currentProject, displayedProjectTitle, els, EXECUTION_STATES, extractCanonicalViewAuthorityInputsFromState, localProjectRunEvidenceFromProject, overlayViewInstances, primaryViewInstance, reconcileViewWorkspace, renderAuthoredBogae, renderCommittedSemanticView, renderSpace2dCanvas2d, selectCommittedOutputView, spatialView, stableCanonicalJson, stableHash, stableSemanticJson, state, updateSpatialView, workbenchHostExecutionState, workbenchSpatialPresentationState }) {
function resizeCanvasToDisplaySize(canvas) {
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(320, Math.floor(rect.width || 640));
  const height = Math.max(240, Math.floor(rect.height || 360));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  return { width, height };
}

function setPrimaryAspect(wasmState) {
  const values = wasmState?.resources?.fixed64;
  const width = Number(values?.보개_그림판_가로), height = Number(values?.보개_그림판_세로);
  const explicit = Number.isFinite(width) && Number.isFinite(height) && width > 0 && height > 0;
  const ratio = explicit ? width / height : 4 / 3, wrap = els.canvas.parentElement;
  wrap.style.setProperty('--primary-aspect', String(ratio));
  wrap.dataset.aspectSource = explicit ? 'authored-dimensions' : 'product-default-4:3';
  const available = Math.max(120, Math.min(innerHeight, wrap.closest('.bogae-panel').getBoundingClientRect().bottom) - wrap.getBoundingClientRect().top - 12);
  wrap.style.maxWidth = `${Math.floor(available * ratio)}px`;
}

function drawProjectRunEvidence(ctx, width, height, evidence, semanticPayload) {
  const margin = Math.max(28, Math.min(width, height) * 0.09);
  ctx.fillStyle = "#eef5ed";
  ctx.strokeStyle = "#64748b";
  ctx.lineWidth = 1.5;
  ctx.fillRect(margin, margin, width - margin * 2, height - margin * 2);
  ctx.strokeRect(margin, margin, width - margin * 2, height - margin * 2);
  ctx.fillStyle = "#31533e";
  ctx.font = "700 15px Segoe UI, sans-serif";
  ctx.fillText("보관된 실행 증거", margin + 18, margin + 30);
  ctx.font = "600 13px Segoe UI, sans-serif";
  ctx.fillText(`state ${evidence.bounded_state_sha256.slice(0, 16) || "-"}`, margin + 18, margin + 58);
  ctx.fillText(`trace ${evidence.trace_sha256.slice(0, 16) || "-"}`, margin + 18, margin + 80);
  ctx.fillText(`outputs ${evidence.output_rows.length}개 · 의미 채널이 없으면 그래프로 추정하지 않음`, margin + 18, margin + 102);
  return {
    schema: "seamgrim.approved_workbench.bogae_view_artifact.v1",
    source: "local_project_run_evidence",
    trace_sha256: evidence.trace_sha256,
    bounded_state_sha256: evidence.bounded_state_sha256,
    run_lineage_hash: evidence.run_lineage?.lineage_hash ?? "",
    output_rows: evidence.output_rows,
    committed_frame: semanticPayload.committed_frame,
    semantic_payload_hash: semanticPayload.semantic_payload_hash,
    view_binding: semanticPayload.view_binding,
    view_config: semanticPayload.view_config,
  };
}

function buildCommittedFrameCanonicalViewPayloads(run, frame) {
  if (!frame) return null;
  const authorityInputs = extractCanonicalViewAuthorityInputsFromState(frame.wasm_state);
  authorityInputs.identity = {
    ...authorityInputs.identity,
    state_hash: String(frame.state_hash ?? authorityInputs.identity?.state_hash ?? ""),
    source_hash: String(frame.source_hash ?? run?.source_hash ?? authorityInputs.identity?.source_hash ?? ""),
    tick_id: Number.isFinite(Number(frame.madi))
      ? Math.trunc(Number(frame.madi))
      : authorityInputs.identity?.tick_id ?? null,
  };
  return buildCanonicalViewPayloadSet(authorityInputs);
}

function buildCommittedCanonicalViewHistory(run, activeFrame, family) {
  if (!run || !Array.isArray(run.frames) || !activeFrame) return [];
  const activeFrameId = Number(activeFrame.frame_id ?? Number.POSITIVE_INFINITY);
  const canonicalKey = `workbench_${String(family ?? "line")}`;
  return run.frames
    .filter((frame) => Number(frame?.frame_id ?? -1) <= activeFrameId)
    .map((frame) => {
      const payloads = buildCommittedFrameCanonicalViewPayloads(run, frame);
      return {
        frame_id: Number(frame.frame_id ?? -1),
        madi: Number(frame.madi ?? 0),
        state_hash: String(frame.state_hash ?? ""),
        canonical_views: { [canonicalKey]: payloads?.[canonicalKey] ?? null },
      };
    });
}

function drawPreview() {
  const logical = currentLogicalPresentation();
  setPrimaryAspect(logical?.value ?? activeFrame()?.wasm_state);
  if (logical?.value) {
    // Display the committed Rust response, not a synthetic ordinary run/frame.
    const canvas = els.canvas, ctx = canvas.getContext('2d', { willReadFrequently:true });
    const {width,height} = resizeCanvasToDisplaySize(canvas);
    ctx.clearRect(0,0,width,height);
    ctx.fillStyle='#f8fbf7'; ctx.fillRect(0,0,width,height);
    const explicitSpace=committedLogicalSpace2d(logical.value);
    const model = logical.value.bogae_projection
      ? renderAuthoredBogae(ctx,width,height,logical.value,authoredCompositionHost)
      : explicitSpace && renderSpace2dCanvas2d({canvas,space2d:explicitSpace,yAxisDown:true,pad:0,showViewMetrics:false})
        ? {data_source:'committed_logical_drawlist',status:'ready',draw_list:explicitSpace.drawlist} : null;
    canvas.dataset.renderSource = model?.data_source ?? '';
    canvas.dataset.committedStateHash = logical.value.state_hash ?? logical.value.frame?.state_hash ?? '';
    canvas.dataset.authoredBogaeDigest = model?.authored_digest ?? '';
    canvas.dataset.authoredBogaeStatus = model?.status ?? '';
    state.semanticRenderModel = model;
    for (const name of ['spatial-fit','spatial-reset','spatial-trail']) document.querySelector(`[data-testid='${name}']`).hidden=true;
    document.querySelector('[data-testid="bogae-text-output"]').hidden=true;
    return;
  }
  if ((workbenchHostExecutionState.pendingBogaeDisplay ?? workbenchHostExecutionState.rejectedBogaeDisplay)?.projectId !== state.projectId) {
    workbenchHostExecutionState.pendingBogaeDisplay = null;
    workbenchHostExecutionState.rejectedBogaeDisplay = null;
  }
  const frozenDisplay = workbenchHostExecutionState.pendingBogaeDisplay ?? workbenchHostExecutionState.rejectedBogaeDisplay;
  if (frozenDisplay) {
    // Even an empty initial picture is retained. Diagnostics live outside the
    // canvas; an invalid activation never paints a replacement error picture.
    const canvas = els.canvas;
    canvas.width = frozenDisplay.width;
    canvas.height = frozenDisplay.height;
    canvas.getContext("2d").putImageData(frozenDisplay.pixels, 0, 0);
    Object.assign(canvas.dataset, frozenDisplay.dataset);
    state.semanticRenderModel = frozenDisplay.renderModel;
    state.semanticViewPayload = frozenDisplay.semanticPayload;
    return;
  }
  const project = currentProject();
  const run = activeRun();
  const frame = activeFrame();
  state.viewWorkspace = reconcileViewWorkspace(state.viewWorkspace, project, frame ? { ...frame, run_id: run?.run_id } : null);
  const primaryInstance = primaryViewInstance(state.viewWorkspace);
  const overlayInstances = overlayViewInstances(state.viewWorkspace);
  const canonicalViewPayloads = buildCommittedFrameCanonicalViewPayloads(run, frame);
  const semanticPayload = buildCommittedSemanticViewPayload({
    project,
    run,
    frame,
    viewConfig: {
      primary_view: project.primaryView,
      role: "primary",
      mode: state.mode,
      view_instance_id: primaryInstance?.view_instance_id ?? "view_primary_1",
      overlay_instance_ids: overlayInstances.map((instance) => instance.view_instance_id),
      overlay_enabled: overlayInstances.length > 0,
    },
    canonicalViewPayloads,
    canonicalViewHistory: buildCommittedCanonicalViewHistory(run, frame, project.primaryView),
  });
  state.semanticViewPayload = semanticPayload;
  const canvas = els.canvas;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  const { width, height } = resizeCanvasToDisplaySize(canvas);
  ctx.clearRect(0, 0, width, height);
  ctx.fillStyle = "#f8fbf7";
  ctx.fillRect(0, 0, width, height);
  const runEvidence = localProjectRunEvidenceFromProject(project);
  let viewArtifact = null;
  let renderModel = null;
  const selectedOutput = selectCommittedOutputView(canonicalViewPayloads, frame, run?.source_hash);
  const display = frame?.wasm_state?.output_display;
  // A3-P1: presentation replaces no canonical payload, saved evidence, or hash.
  // The typed formatting comes from Rust, never from interpreting user strings.
  const committedOutput = selectedOutput && display?.schema === 'ddn.output-display.v1'
    && Array.isArray(display.texts) && display.texts.length === selectedOutput.lines.length
    ? {...selectedOutput, lines:display.texts.map(String)} : selectedOutput;
  canvas.dataset.viewRange = "";
  canvas.dataset.trailSegments = "0";
  for (const name of ["spatial-fit", "spatial-reset", "spatial-trail"]) {
    document.querySelector(`[data-testid='${name}']`).disabled = committedOutput?.family !== "space2d";
  }
  canvas.dataset.consoleOutput = "";
  const textOutput = document.querySelector("[data-testid='bogae-text-output']");
  const textLines = committedOutput?.lines ?? [];
  textOutput.hidden = committedOutput?.family !== "space2d" || textLines.length === 0;
  textOutput.querySelector("pre").textContent = textLines.join("\n");
  canvas.dataset.consoleOutput = textLines.join("\n");
  const authoredRenderModel = renderAuthoredBogae(ctx, width, height, frame?.wasm_state, authoredCompositionHost);
  canvas.dataset.authoredBogaeDigest = authoredRenderModel?.authored_digest ?? "";
  canvas.dataset.authoredBogaeStatus = authoredRenderModel?.status ?? "";
  if (authoredRenderModel) {
    for (const name of ["spatial-fit", "spatial-reset", "spatial-trail"]) document.querySelector(`[data-testid='${name}']`).hidden=true;
    renderModel = authoredRenderModel;
  } else if (committedOutput) {
    for (const name of ["spatial-fit", "spatial-reset"]) document.querySelector(`[data-testid='${name}']`).hidden=false;
    renderModel = {
      schema: "seamgrim.workbench.semantic_render_model.v1",
      family: committedOutput.family, status: "ready",
      data_source: "canonical_committed_output",
      committed_frame: semanticPayload.committed_frame,
      semantic_payload_hash: semanticPayload.semantic_payload_hash,
      canonical_payload_hash: committedOutput.payload_hash,
    };
    if (committedOutput.family === "space2d") {
      const display = updateSpatialView(spatialView, {runId: run.run_id, sourceHash: run.source_hash,
        frame, space2d: committedOutput.space2d});
      const overlay = buildCommittedSpace2dOverlay(semanticPayload, committedOutput.space2d);
      renderModel.overlay_enabled = overlay.model.overlay_enabled;
      renderModel.current = overlay.model.current;
      renderModel.position = overlay.model.position ?? null;
      renderModel.overlay_world_target = overlay.worldTarget ?? null;
      renderModel.overlay_drawlist = overlay.drawlist;
      renderModel.overlay_status = overlay.model.status;
      renderModel.overlay_diagnostic = overlay.model.canonical_diagnostic ?? null;
      const viewSpace = {...committedOutput.space2d, drawlist: [
        ...(workbenchSpatialPresentationState.showSpatialTrail && project.primaryView === "projectile" ? display.overlayDrawlist : []), ...committedOutput.space2d.drawlist,
        ...overlay.drawlist,
      ]};
      canvas.dataset.viewRange = JSON.stringify(display.viewState.range);
      canvas.dataset.trailSegments = String(workbenchSpatialPresentationState.showSpatialTrail && project.primaryView === "projectile" ? display.overlayDrawlist.length : 0);
      renderSpace2dCanvas2d({canvas, space2d: viewSpace, viewState: display.viewState,
        projectPoint: renderModel.current?.physical_bob ?? renderModel.current ?? renderModel.position,
        onProjectedPoint: (point) => { renderModel.canvas_projection = { canvas_current: point }; },
        yAxisDown: project.primaryView === "grid",
        primitiveSource: "drawlist", showAxis: true, showGrid: true,
        showXAxisTicks: true, showYAxisTicks: true, showViewMetrics: false,
        emptyText: "실행하면 그림이 여기에 나타납니다.", noPointsText: "이번 실행에는 그림 출력이 없습니다."});
    } else {
      ctx.fillStyle = "#132018";
      ctx.font = "16px Consolas, monospace";
      committedOutput.lines.forEach((line, index) => {
        ctx.fillText(line, 18, 54 + index * 22);
      });
      canvas.dataset.consoleOutput = committedOutput.lines.join("\n");
    }
  } else if (semanticPayload.committed_frame) {
    renderModel = renderCommittedSemanticView(ctx, width, height, semanticPayload);
  } else if (runEvidence) {
    viewArtifact = drawProjectRunEvidence(ctx, width, height, runEvidence, semanticPayload);
    renderModel = {
      schema: "seamgrim.workbench.semantic_render_model.v1",
      family: project.primaryView,
      status: "archived_evidence_only",
      data_source: "local_project_run_evidence",
      committed_frame: null,
      semantic_payload_hash: semanticPayload.semantic_payload_hash,
      role: "primary",
      labels: [],
    };
  } else if (state.executionLifecycle.status === EXECUTION_STATES.FAILED) {
    const failure = state.diagnostics.find((item) => item.severity === "error");
    ctx.fillStyle = "#9f2424";
    ctx.font = "600 15px sans-serif";
    ctx.fillText("실행을 멈췄습니다.", 18, 64);
    ctx.font = "14px sans-serif";
    ctx.fillText(String(failure?.message ?? "오류 내용을 확인해 주세요.").slice(0, 100), 18, 90, width - 36);
    renderModel = { family: "error", status: "failed", data_source: "execution_diagnostic" };
  } else {
    renderModel = renderCommittedSemanticView(ctx, width, height, semanticPayload);
  }
  state.semanticRenderModel = renderModel;
  ctx.fillStyle = "#31533e";
  ctx.font = "600 13px Segoe UI, sans-serif";
  ctx.fillText(displayedProjectTitle(project), 18, 24);
  if (!viewArtifact) {
    viewArtifact = {
      schema: "seamgrim.approved_workbench.bogae_view_artifact.v1",
      source: semanticPayload.committed_frame ? "committed_semantic_payload" : "static_project_preview",
      project_id: project.id,
      committed_frame: semanticPayload.committed_frame,
      semantic_payload_hash: semanticPayload.semantic_payload_hash,
      view_binding: semanticPayload.view_binding,
      view_config: semanticPayload.view_config,
      render_model: renderModel,
      view_instances: state.viewWorkspace.instances.map((instance) => ({
        view_instance_id: instance.view_instance_id,
        role: instance.role,
        target_view_instance_id: instance.target_view_instance_id,
      })),
    };
  }
  const viewArtifactHash = `candidate:fnv1a32:bogae_view:${stableHash(stableCanonicalJson(viewArtifact))}`;
  canvas.dataset.projectRunBound = runEvidence ? "true" : "false";
  canvas.dataset.projectRunTraceSha256 = runEvidence?.trace_sha256 ?? "";
  canvas.dataset.projectRunLineageHash = runEvidence?.run_lineage?.lineage_hash ?? "";
  canvas.dataset.committedFrameBound = semanticPayload.committed_frame ? "true" : "false";
  canvas.dataset.committedRunId = semanticPayload.committed_frame?.run_id ?? "";
  canvas.dataset.committedFrameId = String(semanticPayload.committed_frame?.frame_id ?? "");
  canvas.dataset.committedMadi = String(semanticPayload.committed_frame?.madi ?? "");
  canvas.dataset.committedStateHash = semanticPayload.committed_frame?.state_hash ?? "";
  canvas.dataset.semanticPayloadHash = semanticPayload.semantic_payload_hash;
  canvas.dataset.canonicalConsolePayloadStatus = canonicalViewPayloads?.console?.status ?? "";
  canvas.dataset.canonicalConsolePayloadHash = canonicalViewPayloads?.console?.candidate_payload_hash ?? "";
  canvas.dataset.canonicalGraphPayloadStatus = canonicalViewPayloads?.graph?.status ?? "";
  canvas.dataset.canonicalGraphPayloadHash = canonicalViewPayloads?.graph?.candidate_payload_hash ?? "";
  canvas.dataset.canonicalConsolePayloadJson = canonicalViewPayloads?.console
    ? stableSemanticJson(canonicalViewPayloads.console)
    : "";
  canvas.dataset.canonicalGraphPayloadJson = canonicalViewPayloads?.graph
    ? stableSemanticJson(canonicalViewPayloads.graph)
    : "";
  canvas.dataset.viewBindingResolution = semanticPayload.view_binding.resolution;
  canvas.dataset.rendererFamily = renderModel?.family ?? "";
  canvas.dataset.rendererStatus = renderModel?.status ?? "";
  canvas.dataset.rendererDataSource = renderModel?.data_source ?? "";
  canvas.dataset.rendererTrajectoryLength = String(renderModel?.trajectory?.length ?? renderModel?.points?.length ?? 0);
  canvas.dataset.rendererCurrentTheta = String(renderModel?.current?.theta ?? "");
  canvas.dataset.rendererCurrentX = String(renderModel?.current?.x ?? "");
  canvas.dataset.rendererCurrentY = String(renderModel?.current?.y ?? "");
  canvas.dataset.rendererTick = String(renderModel?.tick ?? "");
  canvas.dataset.canvasSignature = viewArtifactHash;
  canvas.dataset.viewArtifactHash = viewArtifactHash;
  globalThis.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__ = canonicalViewPayloads;
}

  return { resizeCanvasToDisplaySize, setPrimaryAspect, drawProjectRunEvidence, buildCommittedFrameCanonicalViewPayloads, buildCommittedCanonicalViewHistory, drawPreview };
}
