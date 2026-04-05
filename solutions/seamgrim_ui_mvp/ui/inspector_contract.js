import { normalizeViewFamilyList } from "./view_family_contract.js";

const EXPECTED_SCHEMA = Object.freeze({
  graph: "seamgrim.graph.v0",
  scene: "seamgrim.scene.v0",
  snapshot: "seamgrim.snapshot.v0",
  session: "seamgrim.session.v0",
});

function toText(raw) {
  return String(raw ?? "").trim();
}

function toFinite(raw) {
  const num = Number(raw);
  return Number.isFinite(num) ? num : null;
}

function normalizeSchemaCheck({ name = "", expected = "", actual = "", required = true } = {}) {
  const expectedText = toText(expected);
  const actualText = toText(actual);
  const ok = expectedText && actualText && expectedText === actualText;
  if (!required && !actualText) {
    return {
      name: toText(name),
      expected: expectedText,
      actual: actualText,
      status: "SKIP",
      ok: true,
    };
  }
  return {
    name: toText(name),
    expected: expectedText,
    actual: actualText,
    status: ok ? "OK" : "FAIL",
    ok,
  };
}

function buildRunList(sceneSummary, uiPrefsLessons) {
  const layers = Array.isArray(sceneSummary?.layers) ? sceneSummary.layers : [];
  if (layers.length > 0) {
    return layers
      .map((row, index) => ({
        id: toText(row?.id) || `run-${index + 1}`,
        label: toText(row?.label),
        update: toText(row?.update),
        tick: toFinite(row?.tick),
        points: Math.max(0, Number(row?.points) || 0),
      }))
      .slice(0, 8);
  }

  const entries = Object.entries(uiPrefsLessons && typeof uiPrefsLessons === "object" ? uiPrefsLessons : {})
    .map(([lessonId, pref]) => {
      const row = pref && typeof pref === "object" ? pref : {};
      return {
        id: toText(lessonId),
        label: "",
        update: "",
        tick: null,
        points: Math.max(0, Number(row?.lastRunChannels) || 0),
        lastRunAt: toText(row?.lastRunAt),
      };
    })
    .filter((row) => row.id)
    .sort((a, b) => {
      const ta = Date.parse(a.lastRunAt);
      const tb = Date.parse(b.lastRunAt);
      const na = Number.isFinite(ta) ? ta : 0;
      const nb = Number.isFinite(tb) ? tb : 0;
      return nb - na;
    });
  return entries.slice(0, 8);
}

function buildLogs({
  parseWarnings = [],
  lastExecPathHint = "",
  bridgeCheck = null,
  schemaChecks = [],
} = {}) {
  const out = [];
  const warnings = Array.isArray(parseWarnings) ? parseWarnings : [];
  warnings.forEach((warning) => {
    out.push({
      level: "WARN",
      code: toText(warning?.code) || "W_PARSE",
      message: toText(warning?.message) || "문법 경고",
    });
  });

  const execHint = toText(lastExecPathHint);
  if (execHint && (execHint.includes("실패") || execHint.toLowerCase().includes("fail"))) {
    out.push({
      level: "ERROR",
      code: "E_RUNTIME_PATH",
      message: execHint,
    });
  }

  const failedSchemaNames = schemaChecks.filter((row) => !row.ok).map((row) => row.name).filter(Boolean);
  if (failedSchemaNames.length > 0) {
    out.push({
      level: "ERROR",
      code: "E_SCHEMA_MISMATCH",
      message: `schema mismatch: ${failedSchemaNames.join(", ")}`,
    });
  }

  if (bridgeCheck && bridgeCheck.ok === false) {
    out.push({
      level: "ERROR",
      code: "E_BRIDGE_CHECK",
      message: toText(bridgeCheck.reason) || "bridge check failed",
    });
  }
  return out;
}

export function buildInspectorReport({
  lesson = null,
  lastRuntimeHash = "",
  parseWarnings = [],
  runtimeTickCounter = 0,
  runtimeTimeValue = null,
  playbackPaused = false,
  playbackSpeed = 1,
  lastExecPathHint = "",
  lastRuntimeDerived = null,
  sceneSummary = null,
  snapshotV0 = null,
  sessionV0 = null,
  uiPrefsLessons = null,
} = {}) {
  const graphSchema = toText(
    lastRuntimeDerived?.views?.graph?.schema ??
      sceneSummary?.view?.config?.schema ??
      snapshotV0?.run?.graph?.schema,
  );
  const sceneSchema = toText(sceneSummary?.schema);
  const snapshotSchema = toText(snapshotV0?.schema);
  const sessionSchema = toText(sessionV0?.schema);

  const schemaChecks = [
    normalizeSchemaCheck({ name: "graph", expected: EXPECTED_SCHEMA.graph, actual: graphSchema, required: false }),
    normalizeSchemaCheck({ name: "scene", expected: EXPECTED_SCHEMA.scene, actual: sceneSchema }),
    normalizeSchemaCheck({ name: "snapshot", expected: EXPECTED_SCHEMA.snapshot, actual: snapshotSchema }),
    normalizeSchemaCheck({ name: "session", expected: EXPECTED_SCHEMA.session, actual: sessionSchema }),
  ];

  const inputHash = toText(
    sceneSummary?.hashes?.input_hash ??
      sceneSummary?.inputs?.input_hash ??
      snapshotV0?.run?.hash?.input,
  );
  const resultHash = toText(
    sceneSummary?.hashes?.result_hash ??
      sceneSummary?.inputs?.result_hash ??
      snapshotV0?.run?.hash?.result ??
      lastRuntimeHash,
  );
  const stateHash = toText(lastRuntimeHash);

  const missingRequired = schemaChecks.filter((row) => row.name !== "graph" && !row.ok);
  const bridgeOk = missingRequired.length === 0 && Boolean(resultHash);
  const bridgeCheck = bridgeOk
    ? { ok: true, reason: "" }
    : {
      ok: false,
      reason: missingRequired.length
        ? `missing/invalid schema: ${missingRequired.map((row) => row.name).join(", ")}`
        : "result_hash missing",
    };

  const ddnMeta = lesson?.ddnMetaHeader && typeof lesson.ddnMetaHeader === "object" ? lesson.ddnMetaHeader : {};
  const requiredViews = normalizeViewFamilyList(
    lesson?.requiredViews ?? lesson?.required_views ?? ddnMeta?.requiredViews ?? ddnMeta?.required_views ?? [],
  );
  const runs = buildRunList(sceneSummary, uiPrefsLessons);
  const logs = buildLogs({
    parseWarnings,
    lastExecPathHint,
    bridgeCheck,
    schemaChecks,
  });

  return {
    lesson: {
      id: toText(lesson?.id),
      title: toText(lesson?.title),
      description: toText(lesson?.description),
      ddn_meta: {
        name: toText(ddnMeta?.name),
        desc: toText(ddnMeta?.desc),
        required_views: requiredViews,
      },
    },
    hash: {
      state: stateHash,
      input: inputHash,
      result: resultHash,
    },
    runtime: {
      t: toFinite(runtimeTimeValue),
      tick: Math.max(0, Number(runtimeTickCounter) || 0),
      playing: !playbackPaused,
      speed: Number.isFinite(Number(playbackSpeed)) ? Number(playbackSpeed) : 1,
    },
    schema: schemaChecks,
    bridge_check: bridgeCheck,
    runs,
    logs,
  };
}

export function formatInspectorReportText(report) {
  const row = report && typeof report === "object" ? report : {};
  const lesson = row.lesson && typeof row.lesson === "object" ? row.lesson : {};
  const hash = row.hash && typeof row.hash === "object" ? row.hash : {};
  const runtime = row.runtime && typeof row.runtime === "object" ? row.runtime : {};
  const schema = Array.isArray(row.schema) ? row.schema : [];
  const runs = Array.isArray(row.runs) ? row.runs : [];
  const logs = Array.isArray(row.logs) ? row.logs : [];
  const bridge = row.bridge_check && typeof row.bridge_check === "object" ? row.bridge_check : { ok: false, reason: "" };

  const tText = runtime.t === null || runtime.t === undefined ? "-" : String(runtime.t);
  const speedText = Number.isFinite(Number(runtime.speed)) ? Number(runtime.speed).toFixed(2) : "1.00";
  const lines = [
    `lesson.id: ${toText(lesson.id) || "-"}`,
    `lesson.title: ${toText(lesson.title) || "-"}`,
    `lesson.desc: ${toText(lesson.description) || "-"}`,
    `ddn_meta.name: ${toText(lesson?.ddn_meta?.name) || "-"}`,
    `ddn_meta.desc: ${toText(lesson?.ddn_meta?.desc) || "-"}`,
    `ddn_meta.required_views: ${Array.isArray(lesson?.ddn_meta?.required_views) && lesson.ddn_meta.required_views.length > 0 ? lesson.ddn_meta.required_views.join(", ") : "-"}`,
    `hash.state: ${toText(hash.state) || "-"}`,
    `hash.input: ${toText(hash.input) || "-"}`,
    `hash.result: ${toText(hash.result) || "-"}`,
    `runtime: t=${tText} tick=${Math.max(0, Number(runtime.tick) || 0)} playing=${runtime.playing ? "1" : "0"} speed=${speedText}x`,
    `bridge_check: ${bridge.ok ? "OK" : "FAIL"}${toText(bridge.reason) ? ` (${toText(bridge.reason)})` : ""}`,
    `schema:`,
  ];
  schema.forEach((item) => {
    const name = toText(item?.name) || "-";
    const status = toText(item?.status) || "-";
    const expected = toText(item?.expected) || "-";
    const actual = toText(item?.actual) || "-";
    lines.push(`  ${name}: ${status} (expected=${expected}, actual=${actual})`);
  });
  lines.push(`runs: ${runs.length}`);
  runs.forEach((item) => {
    const id = toText(item?.id) || "-";
    const label = toText(item?.label);
    const update = toText(item?.update) || "-";
    const tick = item?.tick === null || item?.tick === undefined ? "-" : String(item.tick);
    const points = Math.max(0, Number(item?.points) || 0);
    lines.push(`  ${id}${label ? `(${label})` : ""} update=${update} tick=${tick} points=${points}`);
  });
  lines.push(`logs: ${logs.length}`);
  logs.forEach((item) => {
    lines.push(`  [${toText(item?.level) || "INFO"}] ${toText(item?.code) || "-"} ${toText(item?.message) || "-"}`);
  });
  return lines.join("\n");
}
