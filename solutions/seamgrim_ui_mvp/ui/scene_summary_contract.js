import { createInputRegistryState, getSelectedInputRegistryItem } from "./input_registry.js";
import { resolveGraphUpdateTick } from "./update_tick_contract.js";
import { normalizeViewFamilyList } from "./view_family_contract.js";
import {
  CANONICAL_VIEW_PAYLOAD_SCHEMA,
  SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
  buildCanonicalSceneSummaryViewPayload,
  stableSemanticJson,
} from "./workbench/semantic_view_payload.js";

export const SCENE_SUMMARY_SCHEMA = "seamgrim.scene.v0";

function toFiniteNumber(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function toInteger(raw, fallback = 0) {
  const num = Number(raw);
  if (!Number.isFinite(num)) return fallback;
  return Math.trunc(num);
}

function toText(raw) {
  return String(raw ?? "").trim();
}

function normalizeTimestamp(raw) {
  const text = toText(raw);
  if (!text) return new Date().toISOString();
  const parsed = Date.parse(text);
  if (!Number.isFinite(parsed)) return new Date().toISOString();
  return new Date(parsed).toISOString();
}

function readObservationTimeNow(observation) {
  const row = observation && typeof observation === "object" ? observation : {};
  const values = row.all_values && typeof row.all_values === "object"
    ? row.all_values
    : row.values && typeof row.values === "object"
      ? row.values
      : {};
  const keys = ["t", "time", "tick", "프레임수", "시간"];
  for (const key of keys) {
    const hit = toFiniteNumber(values[key]);
    if (hit !== null) return hit;
  }
  return null;
}

function readGraphSeries(graph) {
  const row = graph && typeof graph === "object" ? graph : {};
  return Array.isArray(row.series) ? row.series : [];
}

function readPrimarySeries(graph) {
  const series = readGraphSeries(graph);
  if (!series.length) return null;
  return (
    series.find((entry) => Array.isArray(entry?.points) && entry.points.length > 0) ??
    series[0] ??
    null
  );
}

function readSeriesId(series) {
  return toText(series?.id ?? series?.name ?? series?.label);
}

function readSeriesPoints(series) {
  const points = Array.isArray(series?.points) ? series.points : [];
  return points
    .map((row) => ({ x: toFiniteNumber(row?.x), y: toFiniteNumber(row?.y) }))
    .filter((row) => row.x !== null && row.y !== null);
}

function inferTimelineFromRuntime(runtimeDerived) {
  const graph = runtimeDerived?.views?.graph ?? null;
  const primarySeries = readPrimarySeries(graph);
  const points = readSeriesPoints(primarySeries);
  if (!points.length) return null;

  const xValues = points.map((row) => Number(row.x));
  const tMin = Math.min(...xValues);
  const tMax = Math.max(...xValues);

  let step = null;
  for (let i = 1; i < xValues.length; i += 1) {
    const diff = Math.abs(xValues[i] - xValues[i - 1]);
    if (diff > 0) {
      step = diff;
      break;
    }
  }

  const now = readObservationTimeNow(runtimeDerived?.observation);
  return {
    t_min: tMin,
    t_max: tMax,
    step,
    now: now === null ? tMax : now,
    playing: false,
    frame_count: xValues.length,
    frame_sample: xValues.slice(0, 5),
  };
}

function normalizeTimeSummary(raw) {
  if (!raw || typeof raw !== "object") return null;
  const tMin = toFiniteNumber(raw.t_min);
  const tMax = toFiniteNumber(raw.t_max);
  if (tMin === null || tMax === null) return null;
  return {
    t_min: tMin,
    t_max: tMax,
    step: toFiniteNumber(raw.step),
    now: toFiniteNumber(raw.now),
    playing: Boolean(raw.playing),
    frame_count: Math.max(0, toInteger(raw.frame_count, 0)),
    frame_sample: Array.isArray(raw.frame_sample)
      ? raw.frame_sample.map((value) => toFiniteNumber(value)).filter((value) => value !== null)
      : [],
  };
}

function summarizeInputRegistry(inputRegistryState) {
  const state = createInputRegistryState(inputRegistryState);
  const selected = getSelectedInputRegistryItem(state);
  const registrySummary = state.registry
    .map((entry) => ({
      id: toText(entry?.id),
      type: toText(entry?.type),
      label: toText(entry?.label),
    }))
    .filter((entry) => entry.id)
    .sort((a, b) => a.id.localeCompare(b.id, "ko"));

  const payload = selected?.payload && typeof selected.payload === "object" ? selected.payload : {};
  return {
    selected_id: toText(state.selectedId),
    registry_summary: registrySummary,
    kind: toText(selected?.type),
    label: toText(selected?.label),
    input_hash: toText(payload.input_hash ?? payload.source_input_hash),
    result_hash: toText(payload.result_hash),
  };
}

function summarizeLayer(raw, index = 0) {
  const row = raw && typeof raw === "object" ? raw : {};
  const graph = row.graph && typeof row.graph === "object" ? row.graph : {};
  const updateTick = resolveGraphUpdateTick(graph);
  const primarySeries = readPrimarySeries(graph);
  const points = readSeriesPoints(primarySeries);
  return {
    id: toText(row.id) || `run-${index + 1}`,
    label: toText(row.label) || toText(row.id) || `run-${index + 1}`,
    visible: row.visible !== false,
    order: toInteger(row.order ?? row.layer_index ?? row.layerIndex, index),
    opacity: toFiniteNumber(row.opacity) ?? 1,
    update: updateTick.update,
    tick: updateTick.tick,
    ticks: updateTick.ticks,
    series_id: readSeriesId(primarySeries) || toText(row.series_id),
    points: points.length,
  };
}

function summarizeFallbackLayer(runtimeDerived, { lessonId = "", lessonTitle = "" } = {}) {
  const graph = runtimeDerived?.views?.graph ?? null;
  const primarySeries = readPrimarySeries(graph);
  if (!primarySeries) return [];
  const updateTick = resolveGraphUpdateTick(graph);
  const points = readSeriesPoints(primarySeries);
  const id = lessonId ? `run:${lessonId}` : "run:runtime";
  const label = toText(lessonTitle) || toText(lessonId) || "runtime";
  return [
    {
      id,
      label,
      visible: true,
      order: 0,
      opacity: 1,
      update: updateTick.update,
      tick: updateTick.tick,
      ticks: updateTick.ticks,
      series_id: readSeriesId(primarySeries),
      points: points.length,
    },
  ];
}

function summarizeLayers(overlayRuns, runtimeDerived, lessonInfo) {
  const source = Array.isArray(overlayRuns) ? overlayRuns : [];
  const rows = source.map((run, index) => summarizeLayer(run, index)).filter((row) => row.id);
  const picked = rows.length ? rows : summarizeFallbackLayer(runtimeDerived, lessonInfo);
  return picked.sort((a, b) => {
    if (a.order !== b.order) return a.order - b.order;
    return a.id.localeCompare(b.id, "ko");
  });
}

function presentationViewKind(family) {
  const value = String(family ?? "").trim();
  if (value === "graph") return "view-graph";
  if (value === "space2d" || value === "grid2d") return "view-2d";
  if (value === "table") return "view-table";
  if (value === "text" || value === "structure") return "view-text";
  return "";
}

const CANONICAL_VIEW_KEY_BY_FAMILY = Object.freeze({
  console: "console",
  "console-grid": "console_grid",
  graph: "graph",
  space2d: "space2d",
  table: "table",
  text: "text",
  structure: "structure",
  grid2d: "grid2d",
  "workbench-line": "workbench_line",
  "workbench-pendulum": "workbench_pendulum",
  "workbench-projectile": "workbench_projectile",
  "workbench-grid": "workbench_grid",
});

function canonicalPayloadForFamily(canonicalViewPayloads, family) {
  const key = CANONICAL_VIEW_KEY_BY_FAMILY[String(family ?? "").trim()] ?? "";
  if (!key) return null;
  const payload = canonicalViewPayloads?.[key];
  return payload && typeof payload === "object" && !Array.isArray(payload) ? payload : null;
}

function canonicalIdentityFromPayloads(payloads) {
  const rows = (Array.isArray(payloads) ? payloads : [])
    .filter((payload) => payload?.schema === CANONICAL_VIEW_PAYLOAD_SCHEMA && payload?.identity)
    .map((payload) => payload.identity);
  if (!rows.length) return {};
  const first = stableSemanticJson(rows[0]);
  return rows.every((row) => stableSemanticJson(row) === first) ? rows[0] : {};
}

function buildSceneSummarySemanticProjection({ requiredViews, primaryView, canonicalViewPayloads }) {
  const requiredFamilies = normalizeViewFamilyList(requiredViews);
  const explicitPrimary = String(primaryView ?? "").trim().toLowerCase();
  const payloadRows = requiredFamilies
    .map((family) => ({ family, payload: canonicalPayloadForFamily(canonicalViewPayloads, family) }))
    .filter((row) => row.payload);
  const allCanonicalPayloads = Object.values(
    canonicalViewPayloads && typeof canonicalViewPayloads === "object" ? canonicalViewPayloads : {},
  ).filter((payload) => payload?.schema === CANONICAL_VIEW_PAYLOAD_SCHEMA
    && !["scene-summary", "curriculum-semantic-scene"].includes(String(payload?.family ?? "")));
  const identity = canonicalIdentityFromPayloads(payloadRows.map((row) => row.payload).concat(allCanonicalPayloads));

  if (explicitPrimary) {
    const primaryPayload = canonicalPayloadForFamily(canonicalViewPayloads, explicitPrimary);
    if (!primaryPayload) {
      return {
        identity,
        projection: {
          schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
          status: "no_view",
          primary_family: explicitPrimary,
          required_families: requiredFamilies,
          reason: "explicit_primary_canonical_payload_absent",
          source: "explicit_primary_view_contract",
          provenance: "canonical_view_payload_projection",
        },
      };
    }
    return {
      identity: primaryPayload.identity ?? identity,
      projection: {
        schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
        status: String(primaryPayload.status ?? "invalid"),
        primary_family: explicitPrimary,
        required_families: requiredFamilies,
        primary_payload: primaryPayload.status === "available" ? primaryPayload : null,
        reason: primaryPayload.status === "available" ? "" : "explicit_primary_canonical_payload_not_available",
        source: "explicit_primary_view_contract",
        provenance: "canonical_view_payload_projection",
      },
    };
  }

  const available = payloadRows.filter((row) => row.payload.status === "available");
  if (available.length > 1) {
    return {
      identity,
      projection: {
        schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
        status: "ambiguous",
        required_families: requiredFamilies,
        reason: "multiple_equal_required_canonical_view_candidates",
        source: "required_view_contract",
        provenance: "canonical_view_payload_projection",
      },
    };
  }
  if (available.length === 1) {
    return {
      identity: available[0].payload.identity ?? identity,
      projection: {
        schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
        status: "available",
        primary_family: available[0].family,
        required_families: requiredFamilies,
        primary_payload: available[0].payload,
        reason: "",
        source: "single_required_canonical_view_candidate",
        provenance: "canonical_view_payload_projection",
      },
    };
  }
  const failure = payloadRows.find((row) => ["unsupported", "invalid", "ambiguous", "unavailable"].includes(row.payload.status));
  return {
    identity,
    projection: {
      schema: SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA,
      status: String(failure?.payload?.status ?? "no_view"),
      primary_family: failure?.family ?? "",
      required_families: requiredFamilies,
      reason: failure ? "required_canonical_view_candidate_not_available" : "no_required_canonical_view_candidate",
      source: "required_view_contract",
      provenance: "canonical_view_payload_projection",
    },
  };
}

function buildViewConfig(runtimeDerived) {
  const graphAxis = runtimeDerived?.views?.graph?.axis ?? {};
  return {
    range: {
      x_min: toFiniteNumber(graphAxis.x_min),
      x_max: toFiniteNumber(graphAxis.x_max),
      y_min: toFiniteNumber(graphAxis.y_min),
      y_max: toFiniteNumber(graphAxis.y_max),
    },
    pan_x: 0,
    pan_y: 0,
    zoom: 1,
    grid: true,
    axis: true,
  };
}

function resolveHashes({ runtimeDerived, runtimeHash, inputSummary }) {
  const graphMeta = runtimeDerived?.views?.graph?.meta ?? {};
  const inputHash = toText(
    graphMeta.source_input_hash ??
      inputSummary.input_hash,
  );
  const resultHash = toText(
    graphMeta.result_hash ??
      runtimeHash ??
      inputSummary.result_hash,
  );
  return {
    input_hash: inputHash,
    result_hash: resultHash,
  };
}

function normalizeBogaeScene(raw) {
  if (raw === null || raw === undefined) return null;
  if (!Array.isArray(raw)) return null;
  return raw.map((row) => (row && typeof row === "object" ? { ...row } : row));
}

export function buildSceneSummarySnapshot({
  timestamp = "",
  lessonId = "",
  lessonTitle = "",
  requiredViews = [],
  inputRegistryState = null,
  overlayRuns = [],
  runtimeDerived = null,
  runtimeHash = "",
  primaryView = "",
  canonicalViewPayloads = null,
  view = null,
  time = undefined,
  bogaeScene = null,
} = {}) {
  const normalizedRequiredViews = normalizeViewFamilyList(requiredViews);
  const inputSummary = summarizeInputRegistry(inputRegistryState);
  const hashes = resolveHashes({
    runtimeDerived,
    runtimeHash,
    inputSummary,
  });
  const layers = summarizeLayers(overlayRuns, runtimeDerived, { lessonId, lessonTitle });
  const canonicalPayloadSet = canonicalViewPayloads && typeof canonicalViewPayloads === "object"
    ? canonicalViewPayloads
    : runtimeDerived?.canonicalViewPayloads && typeof runtimeDerived.canonicalViewPayloads === "object"
      ? runtimeDerived.canonicalViewPayloads
      : {};
  const sceneProjection = buildSceneSummarySemanticProjection({
    requiredViews: normalizedRequiredViews,
    primaryView,
    canonicalViewPayloads: canonicalPayloadSet,
  });
  const canonicalViewPayload = buildCanonicalSceneSummaryViewPayload(sceneProjection);
  const resolvedPrimaryFamily = canonicalViewPayload.status === "available"
    ? String(canonicalViewPayload?.content?.scene_summary?.primary_family ?? "")
    : String(primaryView ?? "").trim();
  const viewSummary = view && typeof view === "object"
    ? {
      kind: toText(view.kind) || presentationViewKind(resolvedPrimaryFamily),
      config: view.config && typeof view.config === "object" ? { ...view.config } : buildViewConfig(runtimeDerived),
    }
    : {
      kind: presentationViewKind(resolvedPrimaryFamily),
      config: buildViewConfig(runtimeDerived),
    };
  const timeSummary = time === undefined
    ? inferTimelineFromRuntime(runtimeDerived)
    : normalizeTimeSummary(time);

  const presentationSession = {
    ts: normalizeTimestamp(timestamp),
    view: viewSummary,
    layers,
    time: timeSummary,
    bogae_scene: normalizeBogaeScene(bogaeScene),
  };
  return {
    schema: SCENE_SUMMARY_SCHEMA,
    canonical_view_payload: canonicalViewPayload,
    presentation_session: presentationSession,
    ts: presentationSession.ts,
    view: presentationSession.view,
    inputs: {
      selected_id: inputSummary.selected_id,
      registry_summary: inputSummary.registry_summary,
      kind: inputSummary.kind,
      label: inputSummary.label,
      input_hash: hashes.input_hash,
      result_hash: hashes.result_hash,
    },
    required_views: normalizedRequiredViews,
    layers: presentationSession.layers,
    time: presentationSession.time,
    hashes,
    bogae_scene: presentationSession.bogae_scene,
    boundaries: {
      canonical_owner: "semantic_view_payload",
      top_level_scene_fields_are_session_compatibility: true,
      canonical_identity_excludes: [
        "timestamp",
        "local_storage_custody",
        "view_config",
        "timeline",
        "layers",
        "bogae_scene",
      ],
    },
  };
}

export function serializeSceneSummarySession(summary) {
  return {
    schema: SCENE_SUMMARY_SCHEMA,
    scene: summary && typeof summary === "object"
      ? JSON.parse(JSON.stringify(summary))
      : null,
  };
}

export function restoreSceneSummarySession(payload) {
  const row = payload && typeof payload === "object" ? payload : {};
  const scene = row.scene && typeof row.scene === "object" ? row.scene : row;
  if (toText(scene.schema) !== SCENE_SUMMARY_SCHEMA) return null;
  if (scene.canonical_view_payload !== undefined) {
    if (
      scene.canonical_view_payload?.schema !== CANONICAL_VIEW_PAYLOAD_SCHEMA
      || scene.canonical_view_payload?.family !== "scene-summary"
    ) return null;
  }
  return JSON.parse(JSON.stringify(scene));
}
