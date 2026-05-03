import { normalizeViewFamilyList } from "./view_family_contract.js";
import { formatDisplayLabel, formatSourceLabel } from "./display_label_contract.js";

const EXPECTED_SCHEMA = Object.freeze({
  graph: "seamgrim.graph.v0",
  scene: "seamgrim.scene.v0",
  snapshot: "seamgrim.snapshot.v0",
  session: "seamgrim.session.v0",
});
const VIEW_FAMILIES = Object.freeze(["graph", "space2d", "table", "text", "structure"]);
const NON_STRICT_VIEW_SOURCES = Object.freeze([
  "observation_output",
  "observation-output-lines",
  "observation-fallback",
]);

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
    const userMessage = toText(warning?.message) || "문법 경고";
    const technicalMessage = toText(warning?.technical_message);
    const mergedMessage =
      technicalMessage && technicalMessage !== userMessage
        ? `${userMessage} (기술: ${technicalMessage})`
        : userMessage;
    out.push({
      level: "WARN",
      code: toText(warning?.code) || "W_PARSE",
      message: mergedMessage,
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

function inferViewFamiliesFromPresence(views) {
  const out = [];
  VIEW_FAMILIES.forEach((family) => {
    if (views?.[family] && typeof views[family] === "object") {
      out.push(family);
    }
  });
  return normalizeViewFamilyList(out);
}

function isNonStrictViewSource(rawSource) {
  const source = toText(rawSource).toLowerCase();
  if (!source) return false;
  return NON_STRICT_VIEW_SOURCES.includes(source);
}

function buildViewContractSummary(lastRuntimeDerived) {
  const views = lastRuntimeDerived?.views && typeof lastRuntimeDerived.views === "object" ? lastRuntimeDerived.views : {};
  const viewContract =
    views.contract && typeof views.contract === "object"
      ? views.contract
      : lastRuntimeDerived?.view_contract && typeof lastRuntimeDerived.view_contract === "object"
        ? lastRuntimeDerived.view_contract
        : {};
  const source = toText(viewContract.source) || "runtime";
  const schema = toText(viewContract.schema) || "seamgrim.view_contract.v1";
  const contractFamilies = normalizeViewFamilyList(viewContract.families ?? []);
  const families = contractFamilies.length > 0 ? contractFamilies : inferViewFamiliesFromPresence(views);
  const byFamilyRaw = viewContract.by_family && typeof viewContract.by_family === "object" ? viewContract.by_family : {};
  const sourceMap = {};
  const nonStrictFamilies = [];
  VIEW_FAMILIES.forEach((family) => {
    const row = byFamilyRaw[family] && typeof byFamilyRaw[family] === "object" ? byFamilyRaw[family] : {};
    const runtimeView = views?.[family] && typeof views[family] === "object" ? views[family] : null;
    const fallbackGraphSource = family === "graph" ? toText(lastRuntimeDerived?.graphSource) : "";
    const familySource = toText(row.source) || toText(runtimeView?.meta?.source) || fallbackGraphSource || "-";
    const familySchema = toText(row.schema) || toText(runtimeView?.schema) || "-";
    const available = runtimeView ? true : Boolean(row.available);
    sourceMap[family] = {
      source: familySource,
      schema: familySchema,
      available,
    };
    if (available && isNonStrictViewSource(familySource)) {
      nonStrictFamilies.push(family);
    }
  });
  const uniqueNonStrictFamilies = [...new Set(nonStrictFamilies)];
  return {
    schema,
    source,
    families,
    source_map: sourceMap,
    strict: uniqueNonStrictFamilies.length === 0,
    non_strict_families: uniqueNonStrictFamilies,
  };
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
  const viewContract = buildViewContractSummary(lastRuntimeDerived);
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
    view_contract: viewContract,
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
  const viewContract = row.view_contract && typeof row.view_contract === "object" ? row.view_contract : {};
  const viewFamilies = Array.isArray(viewContract.families) ? viewContract.families : [];
  const viewSourceMap = viewContract.source_map && typeof viewContract.source_map === "object" ? viewContract.source_map : {};
  const viewStrict = Boolean(viewContract.strict);
  const nonStrictFamilies = Array.isArray(viewContract.non_strict_families) ? viewContract.non_strict_families : [];
  const runs = Array.isArray(row.runs) ? row.runs : [];
  const logs = Array.isArray(row.logs) ? row.logs : [];
  const bridge = row.bridge_check && typeof row.bridge_check === "object" ? row.bridge_check : { ok: false, reason: "" };

  const tText = runtime.t === null || runtime.t === undefined ? "-" : String(runtime.t);
  const speedText = Number.isFinite(Number(runtime.speed)) ? Number(runtime.speed).toFixed(2) : "1.00";
  const lines = [
    `교과 식별자: ${toText(lesson.id) || "-"}`,
    `교과 제목: ${toText(lesson.title) || "-"}`,
    `교과 설명: ${toText(lesson.description) || "-"}`,
    `DDN 메타 이름: ${toText(lesson?.ddn_meta?.name) || "-"}`,
    `DDN 메타 설명: ${toText(lesson?.ddn_meta?.desc) || "-"}`,
    `DDN 필수 보기: ${Array.isArray(lesson?.ddn_meta?.required_views) && lesson.ddn_meta.required_views.length > 0 ? lesson.ddn_meta.required_views.map((item) => formatDisplayLabel(item)).join(", ") : "-"}`,
    `보기 계약 꼴: ${toText(viewContract.schema) || "-"}`,
    `보기 계약 출처: ${toText(viewContract.source) ? formatSourceLabel(viewContract.source) : "-"}`,
    `보기 종류: ${viewFamilies.length > 0 ? viewFamilies.map((item) => formatDisplayLabel(item)).join(", ") : "-"}`,
    `보기 엄격성: ${viewStrict ? "정상" : "실패"}${nonStrictFamilies.length ? ` (${nonStrictFamilies.map((item) => formatDisplayLabel(item)).join(", ")})` : ""}`,
    `상태 해시: ${toText(hash.state) || "-"}`,
    `입력 해시: ${toText(hash.input) || "-"}`,
    `결과 해시: ${toText(hash.result) || "-"}`,
    `런타임: 시간=${tText} 마디=${Math.max(0, Number(runtime.tick) || 0)} 실행중=${runtime.playing ? "1" : "0"} 속도=${speedText}x`,
    `연결 확인: ${bridge.ok ? "정상" : "실패"}${toText(bridge.reason) ? ` (${toText(bridge.reason)})` : ""}`,
    `꼴 검사:`,
  ];
  schema.forEach((item) => {
    const name = formatDisplayLabel(toText(item?.name) || "-");
    const status = formatDisplayLabel(toText(item?.status) || "-");
    const expected = toText(item?.expected) || "-";
    const actual = toText(item?.actual) || "-";
    lines.push(`  ${name}: ${status} (기대값=${expected}, 실제값=${actual})`);
  });
  lines.push(`보기 계약 출처:`);
  VIEW_FAMILIES.forEach((family) => {
    const row0 = viewSourceMap?.[family] && typeof viewSourceMap[family] === "object" ? viewSourceMap[family] : {};
    const source = toText(row0.source) ? formatSourceLabel(row0.source) : "-";
    const schemaText = toText(row0.schema) || "-";
    const available = row0.available ? "1" : "0";
    lines.push(`  ${formatDisplayLabel(family)}: 출처=${source} 꼴=${schemaText} 사용가능=${available}`);
  });
  lines.push(`실행 기록: ${runs.length}`);
  runs.forEach((item) => {
    const id = toText(item?.id) || "-";
    const label = toText(item?.label);
    const update = toText(item?.update) || "-";
    const tick = item?.tick === null || item?.tick === undefined ? "-" : String(item.tick);
    const points = Math.max(0, Number(item?.points) || 0);
    lines.push(`  ${id}${label ? `(${label})` : ""} 갱신=${update} 마디=${tick} 점=${points}`);
  });
  lines.push(`기록: ${logs.length}`);
  logs.forEach((item) => {
    lines.push(`  [${toText(item?.level) || "INFO"}] ${toText(item?.code) || "-"} ${toText(item?.message) || "-"}`);
  });
  return lines.join("\n");
}
