import { semanticViewPayloadHash } from "./semantic_view_payload.js";

export const SEMANTIC_RENDER_MODEL_SCHEMA = "seamgrim.workbench.semantic_render_model.v1";

function finiteNumber(value) {
  if (value === null || value === undefined || value === "") return null;
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function label(key, value, unit = "") {
  if (value === null || value === undefined) return null;
  return { key, value, unit };
}

function canonicalWorkbenchResult(payload, family) {
  const canonicalPayload = payload?.canonical_views?.[`workbench_${family}`];
  if (!canonicalPayload || typeof canonicalPayload !== "object") {
    return { status: "unavailable", result: null, diagnostic: null };
  }
  const committed = payload?.committed_frame;
  const identity = canonicalPayload.identity;
  if (
    !committed
    || !identity
    || String(identity.state_hash ?? "") !== String(committed.state_hash ?? "")
    || String(identity.source_hash ?? "") !== String(committed.source_hash ?? "")
    || Number(identity.tick_id) !== Number(committed.madi)
  ) {
    return {
      status: "unsupported",
      result: null,
      diagnostic: {
        code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
        reason: "canonical_result_identity_mismatch",
        actual_schema: String(canonicalPayload.schema ?? ""),
      },
    };
  }
  if (
    String(canonicalPayload?.content?.provider_identity ?? "") !== "ddonirang_tool.wasm_runtime"
    || String(canonicalPayload?.content?.provider_provenance ?? "") !== "native_authoritative_structured_result"
  ) {
    return {
      status: "unsupported",
      result: null,
      diagnostic: {
        code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
        reason: "canonical_result_provider_identity_mismatch",
        actual_schema: String(canonicalPayload.schema ?? ""),
      },
    };
  }
  return {
    status: String(canonicalPayload.status ?? "unavailable"),
    result: canonicalPayload?.content?.result ?? null,
    diagnostic: canonicalPayload.diagnostic ?? null,
  };
}

function canonicalMissingModel(model, authority) {
  return {
    ...model,
    status: authority.status === "no_view" ? "no_canonical_view" : `canonical_${authority.status}`,
    data_source: "canonical_workbench_result",
    canonical_diagnostic: authority.diagnostic,
  };
}

function unsupportedRendererModel(payload, family) {
  return canonicalMissingModel(baseModel(payload), {
    status: "unsupported",
    diagnostic: {
      // Reuse the existing typed canonical-projection failure taxonomy.  A
      // renderer token is capability selection, not semantic input, so it
      // must never select a different successful renderer.
      code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
      reason: "unsupported_renderer_family",
      actual_schema: family,
    },
  });
}

function projectionCamera(camera) {
  const minX = finiteNumber(camera?.x_min);
  const maxX = finiteNumber(camera?.x_max);
  const minY = finiteNumber(camera?.y_min);
  const maxY = finiteNumber(camera?.y_max);
  if (minX === null || maxX === null || minY === null || maxY === null || minX >= maxX || minY >= maxY) return null;
  return { minX, maxX, minY, maxY };
}

function missingProjectionCameraModel(payload, family) {
  return canonicalMissingModel(baseModel(payload), {
    status: "unsupported",
    diagnostic: {
      code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
      reason: "projection_camera_missing_or_invalid",
      actual_schema: family,
    },
  });
}

function unsupportedDrawItemModel(payload, family) {
  return canonicalMissingModel(baseModel(payload), {
    status: "unsupported",
    diagnostic: {
      code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
      reason: "unsupported_bogae_kind",
      actual_schema: family,
    },
  });
}

function staleSemanticPayloadModel(payload) {
  return canonicalMissingModel(baseModel(payload), {
    status: "unsupported",
    diagnostic: {
      code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
      reason: "semantic_payload_hash_mismatch",
      actual_schema: String(payload?.schema ?? ""),
    },
  });
}

function invalidCommittedFrameModel(payload) {
  return canonicalMissingModel(baseModel(payload), {
    status: "unsupported",
    diagnostic: {
      code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
      reason: "committed_frame_identity_missing_or_invalid",
      actual_schema: String(payload?.schema ?? ""),
    },
  });
}

function invalidViewBindingModel(payload) {
  return canonicalMissingModel(baseModel(payload), {
    status: "unsupported",
    diagnostic: {
      code: "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE",
      reason: "view_binding_identity_missing_or_mismatch",
      actual_schema: String(payload?.view_binding?.schema ?? ""),
    },
  });
}

function semanticPayloadHashMatches(payload) {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) return false;
  const { semantic_payload_hash: declaredHash, ...hashInput } = payload;
  return typeof declaredHash === "string" && declaredHash === semanticViewPayloadHash(hashInput);
}

function committedFrameIdentityValid(payload) {
  const frame = payload?.committed_frame;
  return Boolean(
    frame &&
    typeof frame === "object" &&
    String(frame.run_id ?? "").trim() &&
    String(frame.workspace_identity ?? "").trim() &&
    Number.isInteger(frame.frame_id) && frame.frame_id >= 0 &&
    Number.isInteger(frame.madi) && frame.madi >= 0 &&
    String(frame.state_hash ?? "").trim() &&
    String(frame.source_hash ?? "").trim(),
  );
}

function executionReceiptBindingValid(payload) {
  const binding = payload?.execution_receipt;
  const frame = payload?.committed_frame;
  const lineage = binding?.committed_lineage;
  const last = lineage?.at(-1);
  const lineageValid = Array.isArray(lineage) && lineage.length > 0
    && binding.run_id === frame?.run_id
    && (lineage[0].state_hash === binding.state_hash || last.state_hash === binding.state_hash)
    && Number.isInteger(lineage[0].frame_id) && lineage[0].frame_id >= 0
    && lineage.every((row, index) => row.frame_id === lineage[0].frame_id + index
      && Number.isInteger(row.madi) && row.madi >= 0
      && (index === 0 || row.madi > lineage[index - 1].madi)
      && String(row.state_hash).trim() && row.source_hash === frame?.source_hash)
    && last.frame_id === frame?.frame_id && last.madi === frame?.madi
    && last.state_hash === frame?.state_hash;
  return Boolean(
    binding
    && typeof binding === "object"
    && String(binding.summary_schema ?? "").trim()
    && String(binding.source_profile_identity ?? "").trim()
    && String(binding.source_identity ?? "").trim()
    && (Object.hasOwn(binding, "committed_lineage") || Object.hasOwn(binding, "run_id")
      ? lineageValid
      : String(binding.state_hash ?? "") === String(frame?.state_hash ?? ""))
    && binding.work_cost_receipt
    && typeof binding.work_cost_receipt === "object"
    && String(binding.work_cost_receipt.schema ?? "").trim()
  );
}

function viewBindingIdentityValid(payload) {
  const primaryView = String(payload?.view_config?.primary_view ?? "").trim();
  const binding = payload?.view_binding;
  const canonicalKey = `workbench_${primaryView}`;
  const canonicalStatus = String(payload?.canonical_views?.[canonicalKey]?.status ?? "unavailable");
  return Boolean(
    primaryView
    &&
    binding
    && typeof binding === "object"
    && binding.schema === "seamgrim.workbench.semantic_view_binding.v1"
    && String(binding.view_family ?? "").trim() === primaryView
    && String(binding.canonical_payload_key ?? "").trim() === canonicalKey
    && String(binding.canonical_payload_status ?? "") === canonicalStatus
  );
}

function baseModel(payload) {
  const family = String(payload?.view_config?.primary_view ?? payload?.view_binding?.view_family ?? "");
  return {
    schema: SEMANTIC_RENDER_MODEL_SCHEMA,
    family,
    status: payload?.committed_frame ? "missing_semantic_values" : "awaiting_committed_frame",
    data_source: payload?.committed_frame ? "committed_runtime_frame" : "none",
    committed_frame: payload?.committed_frame ?? null,
    semantic_payload_hash: String(payload?.semantic_payload_hash ?? ""),
    role: String(payload?.view_config?.role ?? "primary"),
    overlay_enabled: Boolean(payload?.view_config?.overlay_enabled),
    focus_target: payload?.view_config?.focus_target ?? null,
    labels: [],
  };
}

function pendulumModel(payload) {
  const model = baseModel(payload);
  const authority = canonicalWorkbenchResult(payload, "pendulum");
  if (authority.status !== "available") return canonicalMissingModel(model, authority);
  const current = authority.result.current;
  return {
    ...model,
    status: "ready",
    data_source: "canonical_workbench_result",
    current,
    overlay: { theta: current.theta, normalized_bob: current.normalized_bob },
    labels: [
      label("theta", current.theta, "rad"),
      label("omega", current.omega, "rad/s"),
      label("energy", current.energy),
      label("kinetic_energy", current.kinetic_energy),
      label("potential_energy", current.potential_energy),
      label("L", current.length, "m"),
    ].filter(Boolean),
  };
}

function projectilePoint(values = {}, identity = {}) {
  const x = finiteNumber(values.x);
  const y = finiteNumber(values.y);
  if (x === null || y === null) return null;
  return {
    frame_id: Number(identity?.frame_id ?? -1),
    madi: Number(identity?.madi ?? 0),
    state_hash: String(identity?.state_hash ?? ""),
    x,
    y,
    t: finiteNumber(values.t),
    vx: finiteNumber(values.vx),
    vy: finiteNumber(values.vy),
  };
}

function projectileModel(payload, requireCamera = true) {
  const model = baseModel(payload);
  const authority = canonicalWorkbenchResult(payload, "projectile");
  if (authority.status !== "available") return canonicalMissingModel(model, authority);
  const camera = projectionCamera(authority.result.camera);
  // A spatial projection needs a camera. A side time graph or decoration
  // on an already committed Space2D picture uses no independent camera.
  if (!camera && requireCamera && model.role !== "side") return missingProjectionCameraModel(payload, "projectile");
  const current = projectilePoint(authority.result.current, payload?.committed_frame);
  const trajectory = (payload?.committed_trajectory ?? [])
    .map((row) => projectilePoint(row.values, row))
    .filter(Boolean);
  return {
    ...model,
    status: "ready",
    data_source: "canonical_workbench_result",
    current,
    trajectory,
    camera,
    overlay: current.vx === null || current.vy === null
      ? null
      : { origin: { x: current.x, y: current.y }, velocity: { x: current.vx, y: current.vy } },
    labels: [
      label("t", current.t, "s"),
      label("x", current.x, "m"),
      label("y", current.y, "m"),
      label("vx", current.vx, "m/s"),
      label("vy", current.vy, "m/s"),
    ].filter(Boolean),
  };
}

function gridModel(payload) {
  const model = baseModel(payload);
  const authority = canonicalWorkbenchResult(payload, "grid");
  if (authority.status !== "available") return canonicalMissingModel(model, authority);
  const result = authority.result;
  const camera = projectionCamera(result.space2d?.camera);
  if (!camera) return missingProjectionCameraModel(payload, "grid");
  if (!Array.isArray(result.space2d?.drawlist)
    || result.space2d.drawlist.some((item) => !isExactBogaeKind(item?.kind))) {
    return unsupportedDrawItemModel(payload, "grid");
  }
  const tick = finiteNumber(payload?.committed_frame?.madi);
  return {
    ...model,
    status: "ready",
    data_source: "canonical_workbench_result",
    tick,
    position: result.position,
    space2d: result.space2d,
    camera,
    dimensions: result.dimensions,
    draw_list: result.space2d.drawlist,
    geometry_source: "canonical_workbench_grid_result",
    labels: [label("tick", tick, "step"), label("position", `${result.position.x},${result.position.y}`, "cell")],
  };
}

function lineModel(payload) {
  const model = baseModel(payload);
  const authority = canonicalWorkbenchResult(payload, "line");
  if (authority.status !== "available") return canonicalMissingModel(model, authority);
  const camera = projectionCamera(authority.result.camera);
  if (!camera) return missingProjectionCameraModel(payload, "line");
  const current = projectilePoint(authority.result.current, payload?.committed_frame);
  const points = (payload?.committed_trajectory ?? [])
    .map((row) => projectilePoint(row.values, row))
    .filter(Boolean);
  if (current && !points.some((point) => point.frame_id === current.frame_id)) points.push(current);
  const { a, b } = authority.result.equation;
  return {
    ...model,
    status: "ready",
    data_source: "canonical_workbench_result",
    current,
    points,
    equation: a === null || b === null ? null : { a, b },
    camera,
    labels: [
      label("x", current?.x),
      label("y", current?.y),
      label("a", a),
      label("b", b),
    ].filter(Boolean),
  };
}

export function buildSemanticRenderModel(payload) {
  if (!payload?.committed_frame) return baseModel(payload);
  if (!semanticPayloadHashMatches(payload)) return staleSemanticPayloadModel(payload);
  if (!committedFrameIdentityValid(payload)) return invalidCommittedFrameModel(payload);
  if (!executionReceiptBindingValid(payload)) return invalidCommittedFrameModel(payload);
  if (!viewBindingIdentityValid(payload)) return invalidViewBindingModel(payload);
  const family = String(payload?.view_config?.primary_view ?? payload?.view_binding?.view_family ?? "");
  if (family === "pendulum") return pendulumModel(payload);
  if (family === "projectile") return projectileModel(payload);
  if (family === "grid") return gridModel(payload);
  if (family === "line") return lineModel(payload);
  return unsupportedRendererModel(payload, family);
}

// Decorations use the same world coordinates and transform as the main drawlist.
export function buildCommittedSpace2dOverlay(payload, space2d) {
  let model = buildSemanticRenderModel(payload);
  if (model.family === "projectile"
      && model.canonical_diagnostic?.reason === "projection_camera_missing_or_invalid") {
    model = projectileModel(payload, false);
  }
  const drawlist = [];
  if (model.status !== "ready" || !(model.overlay_enabled || model.role === "overlay")) return { model, drawlist };
  const current = model.current;
  let x, y;
  if (model.family === "pendulum") {
    x = current.physical_bob?.x;
    y = current.physical_bob?.y;
  } else if (model.family === "projectile" && model.overlay) {
    x = current.x; y = current.y;
  } else return { model, drawlist };
  const matches = (a, b) => Number.isFinite(a) && Math.abs(a - b) <= 1e-6 * Math.max(1, Math.abs(b));
  if (!space2d?.drawlist?.some((item) => item.kind === "circle" && matches(item.x, x) && matches(item.y, y))) {
    return { model, drawlist };
  }
  if (model.family === "projectile") {
    drawlist.push({ kind: "arrow", x1: x, y1: y,
      x2: x + model.overlay.velocity.x * 0.2, y2: y + model.overlay.velocity.y * 0.2,
      color: "#1f7a4d", width: 3 });
  } else {
    drawlist.push({ kind: "line", x1: 0, y1: 0, x2: 0, y2: -current.length,
      style: "dashed", color: "#a16207", width: 3 });
    const radius = current.length * 0.22;
    for (let index = 0; index < 24; index += 1) {
      const a = current.theta * index / 24;
      const b = current.theta * (index + 1) / 24;
      drawlist.push({ kind: "line", x1: radius * Math.sin(a), y1: -radius * Math.cos(a),
        x2: radius * Math.sin(b), y2: -radius * Math.cos(b), color: "#a16207", width: 3 });
    }
  }
  return { model, drawlist, worldTarget: { x, y } };
}

export function formatNumber(value) {
  // Display only: preserve the original value in the committed payload and
  // receipts. Numeric observation text uses the same existing four places.
  if (typeof value === "string" && /^-?(?:0|[1-9]\d*)\.\d+$/u.test(value)) {
    const number = Number(value);
    if (Number.isFinite(number)) return formatNumber(number);
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    if (Math.abs(value) >= 1000 || (Math.abs(value) > 0 && Math.abs(value) < 0.001)) return value.toExponential(3);
    return Number(value.toFixed(4)).toString();
  }
  return String(value ?? "-");
}

function drawLabels(ctx, labels, x, y, lineHeight = 18) {
  ctx.fillStyle = "#164e63";
  ctx.font = "600 13px Segoe UI, sans-serif";
  labels.slice(0, 6).forEach((row, index) => {
    ctx.fillText(`${row.key} ${formatNumber(row.value)}${row.unit ? ` ${row.unit}` : ""}`, x, y + index * lineHeight);
  });
}

function drawAwaiting(ctx, width, height, model) {
  ctx.fillStyle = "#31533e";
  ctx.font = "700 16px Segoe UI, sans-serif";
  const message = model.status === "awaiting_committed_frame"
    ? "실행하면 결과가 나타납니다."
    : "이번 실행에는 이 보기에 표시할 결과가 없습니다.";
  ctx.fillText(message, Math.max(22, width * 0.08), height * 0.5);
}

function drawPendulum(ctx, width, height, model) {
  const cx = width * 0.5;
  const cy = height * 0.2;
  const viewLength = Math.min(width, height) * 0.34;
  const bx = cx + model.current.normalized_bob.x * viewLength;
  const by = cy + model.current.normalized_bob.y * viewLength;
  ctx.strokeStyle = "#d6dfd2";
  ctx.lineWidth = 1;
  for (let index = 0; index < 5; index += 1) {
    const y = height * (0.18 + index * 0.15);
    ctx.beginPath();
    ctx.moveTo(width * 0.12, y);
    ctx.lineTo(width * 0.88, y);
    ctx.stroke();
  }
  ctx.strokeStyle = "#475569";
  ctx.lineWidth = Math.max(3, width * 0.004);
  ctx.beginPath();
  ctx.moveTo(cx, cy);
  ctx.lineTo(bx, by);
  ctx.stroke();
  ctx.fillStyle = "#f59e0b";
  ctx.beginPath();
  ctx.arc(cx, cy, 8, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = "#38bdf8";
  ctx.strokeStyle = "#0e7490";
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.arc(bx, by, Math.max(16, Math.min(width, height) * 0.045), 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();
  if (model.overlay_enabled || model.role === "overlay") {
    ctx.strokeStyle = "#a16207";
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(cx, cy, Math.max(30, viewLength * 0.22), Math.PI / 2, Math.PI / 2 - model.current.theta, model.current.theta > 0);
    ctx.stroke();
    ctx.setLineDash([6, 5]);
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(cx, cy + viewLength);
    ctx.stroke();
    ctx.setLineDash([]);
  }
  drawLabels(ctx, model.labels, 20, height - Math.min(100, 18 * model.labels.length));
  return { canvas_bob: { x: bx, y: by }, overlay_origin: { x: cx, y: cy } };
}

function drawAxes(ctx, width, height, bounds, margin = 46, yAxisDown = false, squareUnits = false) {
  const plotWidth = Math.max(1, width - margin * 2);
  const plotHeight = Math.max(1, height - margin * 2);
  const spanX = Math.max(1e-9, bounds.maxX - bounds.minX);
  const spanY = Math.max(1e-9, bounds.maxY - bounds.minY);
  const unitScale = Math.min(plotWidth / spanX, plotHeight / spanY);
  const scaleX = squareUnits ? unitScale : plotWidth / spanX;
  const scaleY = squareUnits ? unitScale : plotHeight / spanY;
  const offsetX = (plotWidth - spanX * scaleX) / 2;
  const offsetY = (plotHeight - spanY * scaleY) / 2;
  const sx = (x) => margin + offsetX + (x - bounds.minX) * scaleX;
  const sy = (y) => yAxisDown
    ? margin + offsetY + (y - bounds.minY) * scaleY
    : height - margin - offsetY - (y - bounds.minY) * scaleY;
  ctx.strokeStyle = "#64748b";
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(margin, height - margin);
  ctx.lineTo(width - margin, height - margin);
  ctx.moveTo(margin, height - margin);
  ctx.lineTo(margin, margin);
  ctx.stroke();
  return { sx, sy, margin };
}

function drawProjectile(ctx, width, height, model) {
  const points = model.trajectory.length ? model.trajectory : [model.current];
  const { sx, sy, margin } = drawAxes(ctx, width, height, model.camera);
  ctx.fillStyle = "#475569";
  ctx.font = "600 13px Segoe UI, sans-serif";
  ctx.fillText("x (m)", width - margin - 34, height - margin + 28);
  ctx.fillText("y (m)", margin - 10, margin - 14);
  ctx.strokeStyle = "#0e7490";
  ctx.lineWidth = 4;
  ctx.beginPath();
  points.forEach((point, index) => {
    if (index === 0) ctx.moveTo(sx(point.x), sy(point.y));
    else ctx.lineTo(sx(point.x), sy(point.y));
  });
  ctx.stroke();
  const currentCanvas = { x: sx(model.current.x), y: sy(model.current.y) };
  ctx.fillStyle = "#b45309";
  ctx.beginPath();
  ctx.arc(currentCanvas.x, currentCanvas.y, 8, 0, Math.PI * 2);
  ctx.fill();
  if ((model.overlay_enabled || model.role === "overlay") && model.overlay) {
    const scale = 0.2;
    ctx.strokeStyle = "#1f7a4d";
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.moveTo(currentCanvas.x, currentCanvas.y);
    ctx.lineTo(sx(model.current.x + model.overlay.velocity.x * scale), sy(model.current.y + model.overlay.velocity.y * scale));
    ctx.stroke();
  }
  drawLabels(ctx, model.labels, 18, 22);
  return { canvas_current: currentCanvas, trajectory_point_count: points.length };
}

function normalizeKind(item) {
  return String(item?.kind ?? "");
}

function isExactBogaeKind(kind) {
  return new Set(["line", "rect", "circle", "point", "polygon", "polyline", "text", "arrow"])
    .has(String(kind ?? ""));
}

function drawSpace2d(ctx, width, height, model) {
  const drawList = model.draw_list;
  const { sx, sy } = drawAxes(ctx, width, height, model.camera, 38, model.family === "grid", model.family === "grid");
  drawList.forEach((item) => {
    const kind = normalizeKind(item);
    ctx.strokeStyle = String(item.stroke ?? item.선색 ?? "#0e7490");
    ctx.fillStyle = String(item.fill ?? item.색 ?? item.채움색 ?? "#22c55e");
    ctx.lineWidth = Math.max(1, finiteNumber(item.line_width ?? item.굵기) ?? 2);
    if (kind === "line") {
      const x1 = finiteNumber(item.x1);
      const y1 = finiteNumber(item.y1);
      const x2 = finiteNumber(item.x2);
      const y2 = finiteNumber(item.y2);
      if ([x1, y1, x2, y2].every((value) => value !== null)) {
        ctx.beginPath();
        ctx.moveTo(sx(x1), sy(y1));
        ctx.lineTo(sx(x2), sy(y2));
        ctx.stroke();
      }
    } else if (kind === "rect") {
      const x = finiteNumber(item.x);
      const y = finiteNumber(item.y);
      const itemWidth = finiteNumber(item.w ?? item.width ?? item.가로);
      const itemHeight = finiteNumber(item.h ?? item.height ?? item.세로);
      if ([x, y, itemWidth, itemHeight].every((value) => value !== null)) {
        const left = Math.min(sx(x), sx(x + itemWidth));
        const top = Math.min(sy(y), sy(y + itemHeight));
        const canvasWidth = Math.abs(sx(x + itemWidth) - sx(x));
        const canvasHeight = Math.abs(sy(y + itemHeight) - sy(y));
        ctx.fillRect(left, top, canvasWidth, canvasHeight);
        ctx.strokeRect(left, top, canvasWidth, canvasHeight);
      }
    } else if (kind === "circle" || kind === "point") {
      const x = finiteNumber(item.x);
      const y = finiteNumber(item.y);
      if (x !== null && y !== null) {
        const radius = Math.max(3, (finiteNumber(item.r ?? item.radius ?? item.크기) ?? 0.2) * 8);
        ctx.beginPath();
        ctx.arc(sx(x), sy(y), radius, 0, Math.PI * 2);
        ctx.fill();
        ctx.stroke();
      }
    }
  });
}

function drawBoardCells(ctx, width, height, model) {
  const board = model.board;
  const rows = board.length;
  const columns = Math.max(1, ...board.map((row) => row.length));
  const size = Math.min(width * 0.72, height * 0.72);
  const cell = Math.min(size / columns, size / rows);
  const x0 = (width - columns * cell) / 2;
  const y0 = (height - rows * cell) / 2;
  board.forEach((row, y) => row.forEach((value, x) => {
    ctx.fillStyle = value === null || value === undefined || value === "." || value === 0 ? "#111827" : "#22c55e";
    ctx.fillRect(x0 + x * cell, y0 + y * cell, cell, cell);
    ctx.strokeStyle = "#314155";
    ctx.lineWidth = 1;
    ctx.strokeRect(x0 + x * cell, y0 + y * cell, cell, cell);
  }));
  return { x0, y0, cell, rows, columns };
}

function drawGrid(ctx, width, height, model) {
  let boardTransform = null;
  if (model.draw_list.length) drawSpace2d(ctx, width, height, model);
  else if (Array.isArray(model.board) && model.board.every(Array.isArray)) boardTransform = drawBoardCells(ctx, width, height, model);
  else if (model.position && model.dimensions) {
    const rows = Math.max(1, Number(model.dimensions.rows));
    const columns = Math.max(1, Number(model.dimensions.columns));
    const blank = Array.from({ length: Math.ceil(rows) }, () => Array.from({ length: Math.ceil(columns) }, () => null));
    boardTransform = drawBoardCells(ctx, width, height, { ...model, board: blank });
  }
  if (model.position && boardTransform) {
    const { x0, y0, cell } = boardTransform;
    ctx.strokeStyle = model.overlay_enabled || model.role === "overlay" ? "#f59e0b" : "#22c55e";
    ctx.lineWidth = model.overlay_enabled || model.role === "overlay" ? 4 : 3;
    ctx.strokeRect(x0 + model.position.x * cell + 2, y0 + model.position.y * cell + 2, Math.max(4, cell - 4), Math.max(4, cell - 4));
  }
  drawLabels(ctx, model.labels, 18, height - Math.max(26, model.labels.length * 18));
  return { geometry_source: model.geometry_source, position: model.position };
}

function drawLine(ctx, width, height, model) {
  let points = model.points;
  if (!points.length && model.equation) {
    points = [-2, 2].map((x) => ({ x, y: model.equation.a * x + model.equation.b }));
  }
  const { sx, sy } = drawAxes(ctx, width, height, model.camera);
  ctx.strokeStyle = "#0e7490";
  ctx.lineWidth = 4;
  ctx.beginPath();
  points.forEach((point, index) => {
    if (index === 0) ctx.moveTo(sx(point.x), sy(point.y));
    else ctx.lineTo(sx(point.x), sy(point.y));
  });
  if (points.length > 1) ctx.stroke();
  const current = model.current ?? points.at(-1);
  if (current) {
    const canvasCurrent = { x: sx(current.x), y: sy(current.y) };
    ctx.fillStyle = "#a16207";
    ctx.beginPath();
    ctx.arc(canvasCurrent.x, canvasCurrent.y, 7, 0, Math.PI * 2);
    ctx.fill();
    drawLabels(ctx, model.labels, 18, 22);
    return { point_count: points.length, current: { x: current.x, y: current.y }, canvas_current: canvasCurrent };
  }
  drawLabels(ctx, model.labels, 18, 22);
  return { point_count: points.length, current: null, canvas_current: null };
}

export function renderCommittedSemanticView(ctx, width, height, payload) {
  const model = buildSemanticRenderModel(payload);
  if (model.status !== "ready") {
    drawAwaiting(ctx, width, height, model);
    return model;
  }
  let canvasProjection = null;
  if (model.role === "side" && ["pendulum", "projectile"].includes(model.family)) {
    return drawCommittedSideGraphs(ctx, width, height, model, payload);
  }
  if (model.family === "pendulum") canvasProjection = drawPendulum(ctx, width, height, model);
  else if (model.family === "projectile") canvasProjection = drawProjectile(ctx, width, height, model);
  else if (model.family === "grid") canvasProjection = drawGrid(ctx, width, height, model);
  else canvasProjection = drawLine(ctx, width, height, model);
  return { ...model, canvas_projection: canvasProjection };
}

// Only plot accepted WASM observations; never integrate or simulate in JS.
function drawCommittedSideGraphs(ctx, width, height, model, payload) {
  const groups = model.family === "pendulum"
    ? [["각도 / rad", ["theta"]], ["각속도 / rad/s", ["omega"]], ["에너지 (단위 질량)", ["energy", "kinetic_energy", "potential_energy"]]]
    : [["위치 / m", ["x", "y"]], ["속도 / m/s", ["vx", "vy"]]];
  const history = payload.committed_trajectory ?? [];
  const colors = ["#0284c7", "#d97706", "#16a34a"];
  let sampleCount = 0;
  groups.forEach(([title, keys], panel) => {
    const top = panel * height / groups.length;
    const bottom = (panel + 1) * height / groups.length - 18;
    const rows = history.map(row => ({...row.values, madi: row.madi})).filter(row => Number.isFinite(Number(row.madi)));
    const values = rows.flatMap(row => keys.map(key => finiteNumber(row[key])).filter(value => value !== null));
    ctx.fillStyle = "#31533e"; ctx.font = "12px sans-serif";
    ctx.fillText(title, 8, top + 14);
    if (!values.length) { ctx.fillText("아직 기록된 관찰값이 없습니다.", 8, top + 34); return; }
    sampleCount = Math.max(sampleCount, rows.length);
    const minT = Math.min(...rows.map(row => Number(row.madi)));
    const maxT = Math.max(...rows.map(row => Number(row.madi)));
    const minY = Math.min(0, ...values), maxY = Math.max(0, ...values);
    const sx = t => 42 + (Number(t) - minT) / Math.max(1e-9, maxT - minT) * (width - 60);
    const sy = y => bottom - (y - minY) / Math.max(1e-9, maxY - minY) * Math.max(1, bottom - top - 38);
    ctx.strokeStyle = "#94a3b8"; ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(42, top + 28); ctx.lineTo(42, bottom); ctx.lineTo(width - 18, bottom); ctx.stroke();
    ctx.fillStyle = "#64748b";
    ctx.fillText(formatNumber(maxY), 2, top + 30); ctx.fillText(formatNumber(minY), 2, bottom);
    ctx.fillText(`${formatNumber(minT)} → ${formatNumber(maxT)} 마디`, 42, bottom + 14);
    keys.forEach((key, index) => {
      ctx.strokeStyle = colors[index]; ctx.fillStyle = colors[index]; ctx.lineWidth = 2;
      ctx.fillText(key, 100 + index * 135, top + 14);
      ctx.beginPath(); let started = false;
      rows.forEach(row => { const y = finiteNumber(row[key]); if (y === null) { started = false; return; }
        if (started) ctx.lineTo(sx(row.madi), sy(y)); else ctx.moveTo(sx(row.madi), sy(y)); started = true; });
      ctx.stroke();
    });
  });
  return { ...model, side_graph_samples: sampleCount, side_graph_panels: groups.length };
}
