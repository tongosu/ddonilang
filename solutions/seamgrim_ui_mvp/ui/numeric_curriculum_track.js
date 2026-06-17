export const NUMERIC_TRACK_ID = "studio_numeric_curriculum_track_v1";

export const NUMERIC_TRACK_LESSON_IDS = Object.freeze([
  "rep_math_function_line_v1",
  "rep_phys_projectile_xy_v1",
  "rep_econ_supply_demand_tax_v1",
]);

export const NUMERIC_TRACK_MODULES = Object.freeze([
  "simulation_time_step",
  "root_finding",
  "exact_polynomial",
  "linear_inequality_interval",
  "post_solve_range_reporting",
]);

export const NUMERIC_TRACK_MODULE_LABELS = Object.freeze({
  simulation_time_step: "시간 전개",
  root_finding: "근 찾기",
  exact_polynomial: "다항식 해",
  linear_inequality_interval: "부등식 구간",
  post_solve_range_reporting: "풀이 후 범위 보고",
});

const NUMERIC_TRACK_LESSON_PREVIEWS = Object.freeze({
  rep_math_function_line_v1: Object.freeze({
    module_ids: Object.freeze([
      "root_finding",
      "exact_polynomial",
      "linear_inequality_interval",
    ]),
    evidence_packs: Object.freeze([
      "numeric_root_finding_bisection_v1",
      "polynomial_solve_minimum_v1",
      "linear_inequality_solve_minimum_v1",
    ]),
    summary: "함수의 근, 다항식 해, 일차 부등식 구간을 한 lesson에서 비교합니다.",
  }),
  rep_phys_projectile_xy_v1: Object.freeze({
    module_ids: Object.freeze([
      "simulation_time_step",
    ]),
    evidence_packs: Object.freeze([
      "ode_tick_loop_lesson_baseline_v1",
      "ode_method_comparison_v1",
    ]),
    summary: "틱 기반 시간 전개와 적분 방법 비교를 물리 lesson 흐름으로 연결합니다.",
  }),
  rep_econ_supply_demand_tax_v1: Object.freeze({
    module_ids: Object.freeze([
      "post_solve_range_reporting",
      "linear_inequality_interval",
    ]),
    evidence_packs: Object.freeze([
      "connect_flow_v1v_closure_v1",
      "linear_inequality_solve_minimum_v1",
    ]),
    summary: "수요·공급 결과를 범위/판정 보고와 부등식 구간으로 점검합니다.",
  }),
});

export function isNumericTrackLesson(lessonOrId) {
  const lessonId = typeof lessonOrId === "string"
    ? lessonOrId
    : String(lessonOrId?.id ?? "").trim();
  return NUMERIC_TRACK_LESSON_IDS.includes(lessonId);
}

export function buildNumericTrackLessonPreview(lessonOrId) {
  const lessonId = typeof lessonOrId === "string"
    ? lessonOrId
    : String(lessonOrId?.id ?? "").trim();
  if (!isNumericTrackLesson(lessonId)) return null;
  const spec = NUMERIC_TRACK_LESSON_PREVIEWS[lessonId];
  if (!spec) return null;
  const lesson = lessonOrId && typeof lessonOrId === "object" ? lessonOrId : {};
  const modules = [...spec.module_ids];
  return {
    schema: "seamgrim.numeric_track_lesson_preview.v1",
    track_id: NUMERIC_TRACK_ID,
    lesson_id: lessonId,
    title: String(lesson.title ?? lessonId),
    subject: String(lesson.subject ?? ""),
    modules,
    module_labels: modules.map((moduleId) => NUMERIC_TRACK_MODULE_LABELS[moduleId] ?? moduleId),
    evidence_packs: [...spec.evidence_packs],
    summary: spec.summary,
  };
}

export function buildNumericTrackRunPreset(lessonOrId) {
  const preview = buildNumericTrackLessonPreview(lessonOrId);
  if (!preview) return null;
  const recommendedViews = (() => {
    if (preview.lesson_id === "rep_phys_projectile_xy_v1") return ["space2d", "graph"];
    if (preview.lesson_id === "rep_econ_supply_demand_tax_v1") return ["graph", "table"];
    return ["graph", "table"];
  })();
  const focus = (() => {
    if (preview.lesson_id === "rep_phys_projectile_xy_v1") return "시간 전개";
    if (preview.lesson_id === "rep_econ_supply_demand_tax_v1") return "범위 보고";
    return "근/구간";
  })();
  return {
    schema: "seamgrim.numeric_track_run_preset.v1",
    track_id: NUMERIC_TRACK_ID,
    lesson_id: preview.lesson_id,
    focus,
    label: `수업보기: ${focus}`,
    module_ids: [...preview.modules],
    module_labels: [...preview.module_labels],
    recommended_views: recommendedViews,
    evidence_packs: [...preview.evidence_packs],
  };
}

export function buildNumericTrackIndexSnapshot(lessons = []) {
  const rows = Array.isArray(lessons) ? lessons : [];
  const byId = new Map();
  rows.forEach((lesson) => {
    const lessonId = String(lesson?.id ?? "").trim();
    if (!lessonId || byId.has(lessonId)) return;
    byId.set(lessonId, lesson);
  });
  const activeLessons = NUMERIC_TRACK_LESSON_IDS
    .map((lessonId) => byId.get(lessonId))
    .filter(Boolean)
    .map((lesson) => ({
      id: String(lesson.id ?? ""),
      title: String(lesson.title ?? lesson.id ?? ""),
      subject: String(lesson.subject ?? ""),
      grade: String(lesson.grade ?? ""),
      required_views: Array.isArray(lesson.requiredViews) ? lesson.requiredViews.map((item) => String(item)) : [],
    }));
  const activeIds = new Set(activeLessons.map((lesson) => lesson.id));
  return {
    schema: "seamgrim.numeric_track_index.v1",
    track_id: NUMERIC_TRACK_ID,
    module_count: NUMERIC_TRACK_MODULES.length,
    modules: [...NUMERIC_TRACK_MODULES],
    lesson_ids: [...NUMERIC_TRACK_LESSON_IDS],
    active_lesson_count: activeLessons.length,
    active_lessons: activeLessons,
    missing_lesson_ids: NUMERIC_TRACK_LESSON_IDS.filter((lessonId) => !activeIds.has(lessonId)),
    ui_filter: "numeric_track",
  };
}

export function buildNumericTrackReportExport(lessons = []) {
  const rows = Array.isArray(lessons) ? lessons : [];
  const byId = new Map();
  rows.forEach((lesson) => {
    const lessonId = String(lesson?.id ?? "").trim();
    if (!lessonId || byId.has(lessonId)) return;
    byId.set(lessonId, lesson);
  });
  const previews = NUMERIC_TRACK_LESSON_IDS
    .map((lessonId) => buildNumericTrackLessonPreview(byId.get(lessonId) ?? lessonId))
    .filter(Boolean);
  const moduleRows = NUMERIC_TRACK_MODULES.map((moduleId) => {
    const lessonIds = previews
      .filter((preview) => Array.isArray(preview.modules) && preview.modules.includes(moduleId))
      .map((preview) => preview.lesson_id);
    return {
      module_id: moduleId,
      label: NUMERIC_TRACK_MODULE_LABELS[moduleId] ?? moduleId,
      lesson_count: lessonIds.length,
      lesson_ids: lessonIds,
    };
  });
  const evidencePacks = [];
  const seenEvidence = new Set();
  previews.forEach((preview) => {
    (Array.isArray(preview.evidence_packs) ? preview.evidence_packs : []).forEach((packId) => {
      const row = String(packId ?? "").trim();
      if (!row || seenEvidence.has(row)) return;
      seenEvidence.add(row);
      evidencePacks.push(row);
    });
  });
  return {
    schema: "seamgrim.numeric_track_report_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_lesson_preview.v1",
    lesson_count: previews.length,
    lessons: previews,
    module_count: moduleRows.length,
    modules: moduleRows,
    evidence_pack_count: evidencePacks.length,
    evidence_packs: evidencePacks,
  };
}

export function formatNumericTrackIndexText(snapshot = null) {
  const row = snapshot && typeof snapshot === "object" ? snapshot : {};
  const modules = Array.isArray(row.modules) ? row.modules : [];
  const lessonIds = Array.isArray(row.lesson_ids) ? row.lesson_ids : [];
  const missing = Array.isArray(row.missing_lesson_ids) ? row.missing_lesson_ids : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `module_count\t${modules.length}`,
    `modules\t${modules.join("|")}`,
    `lesson_ids\t${lessonIds.join("|")}`,
    `active_lesson_count\t${row.active_lesson_count ?? 0}`,
    `missing_lesson_ids\t${missing.join("|")}`,
  ].join("\n");
}

export function formatNumericTrackLessonPreviewText(preview = null) {
  const row = preview && typeof preview === "object" ? preview : {};
  const modules = Array.isArray(row.modules) ? row.modules : [];
  const moduleLabels = Array.isArray(row.module_labels) ? row.module_labels : [];
  const evidence = Array.isArray(row.evidence_packs) ? row.evidence_packs : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `lesson_id\t${row.lesson_id ?? ""}`,
    `modules\t${modules.join("|")}`,
    `module_labels\t${moduleLabels.join("|")}`,
    `evidence_packs\t${evidence.join("|")}`,
    `summary\t${row.summary ?? ""}`,
  ].join("\n");
}

export function formatNumericTrackReportExportText(report = null) {
  const row = report && typeof report === "object" ? report : {};
  const lessons = Array.isArray(row.lessons) ? row.lessons : [];
  const modules = Array.isArray(row.modules) ? row.modules : [];
  const evidence = Array.isArray(row.evidence_packs) ? row.evidence_packs : [];
  const lessonLines = lessons.map((lesson) => {
    const moduleLabels = Array.isArray(lesson.module_labels) ? lesson.module_labels.join("|") : "";
    const packs = Array.isArray(lesson.evidence_packs) ? lesson.evidence_packs.join("|") : "";
    return `${lesson.lesson_id ?? ""}\t${lesson.title ?? ""}\t${moduleLabels}\t${packs}`;
  });
  const moduleLines = modules.map((module) => {
    const lessonIds = Array.isArray(module.lesson_ids) ? module.lesson_ids.join("|") : "";
    return `${module.module_id ?? ""}\t${module.label ?? ""}\t${module.lesson_count ?? 0}\t${lessonIds}`;
  });
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `lesson_count\t${lessons.length}`,
    `module_count\t${modules.length}`,
    `evidence_pack_count\t${evidence.length}`,
    `evidence_packs\t${evidence.join("|")}`,
    "",
    "lesson_id\ttitle\tmodule_labels\tevidence_packs",
    ...lessonLines,
    "",
    "module_id\tlabel\tlesson_count\tlesson_ids",
    ...moduleLines,
  ].join("\n");
}

export function formatNumericTrackRunPresetText(preset = null) {
  const row = preset && typeof preset === "object" ? preset : {};
  const modules = Array.isArray(row.module_ids) ? row.module_ids : [];
  const moduleLabels = Array.isArray(row.module_labels) ? row.module_labels : [];
  const views = Array.isArray(row.recommended_views) ? row.recommended_views : [];
  const evidence = Array.isArray(row.evidence_packs) ? row.evidence_packs : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `lesson_id\t${row.lesson_id ?? ""}`,
    `focus\t${row.focus ?? ""}`,
    `label\t${row.label ?? ""}`,
    `module_ids\t${modules.join("|")}`,
    `module_labels\t${moduleLabels.join("|")}`,
    `recommended_views\t${views.join("|")}`,
    `evidence_packs\t${evidence.join("|")}`,
  ].join("\n");
}

export function buildNumericTrackRunResultLink({
  lesson = null,
  preset = null,
  runKind = "",
  channels = 0,
  stateHash = "",
  launchKind = "",
  recordedAt = "",
} = {}) {
  const resolvedPreset = preset && typeof preset === "object"
    ? preset
    : buildNumericTrackRunPreset(lesson);
  if (!resolvedPreset) return null;
  const hash = String(stateHash ?? "").trim();
  return {
    schema: "seamgrim.numeric_track_run_result_link.v1",
    track_id: NUMERIC_TRACK_ID,
    lesson_id: resolvedPreset.lesson_id,
    preset_schema: resolvedPreset.schema,
    preset_focus: resolvedPreset.focus,
    preset_label: resolvedPreset.label,
    run_kind: String(runKind ?? "").trim(),
    channels: Math.max(0, Number.isFinite(Number(channels)) ? Math.trunc(Number(channels)) : 0),
    state_hash: hash && hash !== "-" ? hash : "",
    launch_kind: String(launchKind ?? "").trim(),
    recorded_at: String(recordedAt ?? "").trim(),
    evidence_packs: Array.isArray(resolvedPreset.evidence_packs) ? [...resolvedPreset.evidence_packs] : [],
  };
}

export function normalizeNumericTrackRunResultLink(link = null) {
  if (!link || typeof link !== "object") return null;
  if (link.schema !== "seamgrim.numeric_track_run_result_link.v1") return null;
  const lessonId = String(link.lesson_id ?? "").trim();
  if (!lessonId || !isNumericTrackLesson(lessonId)) return null;
  return {
    schema: "seamgrim.numeric_track_run_result_link.v1",
    track_id: String(link.track_id ?? NUMERIC_TRACK_ID),
    lesson_id: lessonId,
    preset_schema: String(link.preset_schema ?? ""),
    preset_focus: String(link.preset_focus ?? ""),
    preset_label: String(link.preset_label ?? ""),
    run_kind: String(link.run_kind ?? "").trim(),
    channels: Math.max(0, Number.isFinite(Number(link.channels)) ? Math.trunc(Number(link.channels)) : 0),
    state_hash: String(link.state_hash ?? "").trim(),
    launch_kind: String(link.launch_kind ?? "").trim(),
    recorded_at: String(link.recorded_at ?? "").trim(),
    evidence_packs: Array.isArray(link.evidence_packs)
      ? link.evidence_packs.map((item) => String(item ?? "").trim()).filter(Boolean)
      : [],
  };
}

export function formatNumericTrackRunResultLinkText(link = null) {
  const row = link && typeof link === "object" ? link : {};
  const evidence = Array.isArray(row.evidence_packs) ? row.evidence_packs : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `lesson_id\t${row.lesson_id ?? ""}`,
    `preset_focus\t${row.preset_focus ?? ""}`,
    `preset_label\t${row.preset_label ?? ""}`,
    `run_kind\t${row.run_kind ?? ""}`,
    `channels\t${row.channels ?? 0}`,
    `state_hash\t${row.state_hash ?? ""}`,
    `launch_kind\t${row.launch_kind ?? ""}`,
    `recorded_at\t${row.recorded_at ?? ""}`,
    `evidence_packs\t${evidence.join("|")}`,
  ].join("\n");
}

export function buildNumericTrackResultHistorySnapshot({
  lessons = [],
  runPrefs = {},
} = {}) {
  const rows = Array.isArray(lessons) ? lessons : [];
  const prefs = runPrefs?.lessons && typeof runPrefs.lessons === "object" ? runPrefs.lessons : {};
  const historyRows = [];
  rows.forEach((lesson) => {
    const lessonId = String(lesson?.id ?? "").trim();
    if (!lessonId) return;
    const link = normalizeNumericTrackRunResultLink(prefs?.[lessonId]?.numericTrackRunResultLink);
    if (!link) return;
    historyRows.push({
      lesson_id: lessonId,
      title: String(lesson?.title ?? lessonId),
      subject: String(lesson?.subject ?? ""),
      preset_focus: link.preset_focus,
      run_kind: link.run_kind,
      channels: link.channels,
      state_hash: link.state_hash,
      state_hash_short: link.state_hash ? link.state_hash.slice(0, 12) : "",
      recorded_at: link.recorded_at,
      evidence_packs: [...link.evidence_packs],
    });
  });
  return {
    schema: "seamgrim.numeric_track_result_history_filter.v1",
    track_id: NUMERIC_TRACK_ID,
    result_count: historyRows.length,
    result_lesson_ids: historyRows.map((row) => row.lesson_id),
    rows: historyRows,
  };
}

export function formatNumericTrackResultHistoryText(snapshot = null) {
  const row = snapshot && typeof snapshot === "object" ? snapshot : {};
  const rows = Array.isArray(row.rows) ? row.rows : [];
  const resultIds = Array.isArray(row.result_lesson_ids) ? row.result_lesson_ids : [];
  const body = rows.map((item) => {
    const evidence = Array.isArray(item.evidence_packs) ? item.evidence_packs.join("|") : "";
    return [
      item.lesson_id ?? "",
      item.title ?? "",
      item.preset_focus ?? "",
      item.run_kind ?? "",
      item.channels ?? 0,
      item.state_hash_short ?? "",
      item.recorded_at ?? "",
      evidence,
    ].join("\t");
  });
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `result_count\t${rows.length}`,
    `result_lesson_ids\t${resultIds.join("|")}`,
    "",
    "lesson_id\ttitle\tpreset_focus\trun_kind\tchannels\tstate_hash_short\trecorded_at\tevidence_packs",
    ...body,
  ].join("\n");
}

function pushGroupedRow(map, key, lessonId) {
  const groupKey = String(key ?? "").trim() || "-";
  if (!map.has(groupKey)) {
    map.set(groupKey, {
      key: groupKey,
      result_count: 0,
      lesson_ids: [],
    });
  }
  const row = map.get(groupKey);
  row.result_count += 1;
  row.lesson_ids.push(lessonId);
}

export function buildNumericTrackResultSummaryExport(historySnapshot = null) {
  const history = historySnapshot && typeof historySnapshot === "object" ? historySnapshot : {};
  const rows = Array.isArray(history.rows) ? history.rows : [];
  const focusMap = new Map();
  const runKindMap = new Map();
  const evidence = [];
  const seenEvidence = new Set();
  let latestRecordedAt = "";

  const resultRows = rows.map((row) => {
    const lessonId = String(row?.lesson_id ?? "").trim();
    const recordedAt = String(row?.recorded_at ?? "").trim();
    if (recordedAt && (!latestRecordedAt || Date.parse(recordedAt) > Date.parse(latestRecordedAt))) {
      latestRecordedAt = recordedAt;
    }
    pushGroupedRow(focusMap, row?.preset_focus, lessonId);
    pushGroupedRow(runKindMap, row?.run_kind, lessonId);
    (Array.isArray(row?.evidence_packs) ? row.evidence_packs : []).forEach((packId) => {
      const item = String(packId ?? "").trim();
      if (!item || seenEvidence.has(item)) return;
      seenEvidence.add(item);
      evidence.push(item);
    });
    return {
      lesson_id: lessonId,
      title: String(row?.title ?? lessonId),
      preset_focus: String(row?.preset_focus ?? ""),
      run_kind: String(row?.run_kind ?? ""),
      channels: Math.max(0, Number.isFinite(Number(row?.channels)) ? Math.trunc(Number(row.channels)) : 0),
      state_hash_short: String(row?.state_hash_short ?? ""),
      recorded_at: recordedAt,
    };
  });

  const focusRows = Array.from(focusMap.values()).map((row) => ({
    preset_focus: row.key,
    result_count: row.result_count,
    lesson_ids: [...row.lesson_ids],
  }));
  const runKindRows = Array.from(runKindMap.values()).map((row) => ({
    run_kind: row.key,
    result_count: row.result_count,
    lesson_ids: [...row.lesson_ids],
  }));

  return {
    schema: "seamgrim.numeric_track_result_summary_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_history_filter.v1",
    result_count: resultRows.length,
    result_lesson_ids: resultRows.map((row) => row.lesson_id),
    latest_recorded_at: latestRecordedAt,
    focus_count: focusRows.length,
    focuses: focusRows,
    run_kind_count: runKindRows.length,
    run_kinds: runKindRows,
    evidence_pack_count: evidence.length,
    evidence_packs: evidence,
    results: resultRows,
  };
}

export function formatNumericTrackResultSummaryExportText(summary = null) {
  const row = summary && typeof summary === "object" ? summary : {};
  const resultRows = Array.isArray(row.results) ? row.results : [];
  const focusRows = Array.isArray(row.focuses) ? row.focuses : [];
  const runKindRows = Array.isArray(row.run_kinds) ? row.run_kinds : [];
  const evidence = Array.isArray(row.evidence_packs) ? row.evidence_packs : [];
  const resultLines = resultRows.map((item) => [
    item.lesson_id ?? "",
    item.title ?? "",
    item.preset_focus ?? "",
    item.run_kind ?? "",
    item.channels ?? 0,
    item.state_hash_short ?? "",
    item.recorded_at ?? "",
  ].join("\t"));
  const focusLines = focusRows.map((item) => [
    item.preset_focus ?? "",
    item.result_count ?? 0,
    Array.isArray(item.lesson_ids) ? item.lesson_ids.join("|") : "",
  ].join("\t"));
  const runKindLines = runKindRows.map((item) => [
    item.run_kind ?? "",
    item.result_count ?? 0,
    Array.isArray(item.lesson_ids) ? item.lesson_ids.join("|") : "",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `result_count\t${resultRows.length}`,
    `latest_recorded_at\t${row.latest_recorded_at ?? ""}`,
    `focus_count\t${focusRows.length}`,
    `run_kind_count\t${runKindRows.length}`,
    `evidence_pack_count\t${evidence.length}`,
    `evidence_packs\t${evidence.join("|")}`,
    "",
    "lesson_id\ttitle\tpreset_focus\trun_kind\tchannels\tstate_hash_short\trecorded_at",
    ...resultLines,
    "",
    "preset_focus\tresult_count\tlesson_ids",
    ...focusLines,
    "",
    "run_kind\tresult_count\tlesson_ids",
    ...runKindLines,
  ].join("\n");
}

export function buildNumericTrackResultTimelineView(historySnapshot = null) {
  const history = historySnapshot && typeof historySnapshot === "object" ? historySnapshot : {};
  const rows = Array.isArray(history.rows) ? history.rows : [];
  const timeline = rows
    .map((row) => {
      const recordedAt = String(row?.recorded_at ?? "").trim();
      const recordedMs = Date.parse(recordedAt);
      return {
        lesson_id: String(row?.lesson_id ?? "").trim(),
        title: String(row?.title ?? row?.lesson_id ?? ""),
        preset_focus: String(row?.preset_focus ?? ""),
        run_kind: String(row?.run_kind ?? ""),
        channels: Math.max(0, Number.isFinite(Number(row?.channels)) ? Math.trunc(Number(row.channels)) : 0),
        state_hash_short: String(row?.state_hash_short ?? ""),
        recorded_at: recordedAt,
        recorded_ms: Number.isFinite(recordedMs) ? recordedMs : 0,
      };
    })
    .filter((row) => row.lesson_id)
    .sort((a, b) => {
      if (a.recorded_ms !== b.recorded_ms) return b.recorded_ms - a.recorded_ms;
      return a.lesson_id.localeCompare(b.lesson_id, "ko");
    })
    .map((row, index) => ({
      index,
      lesson_id: row.lesson_id,
      title: row.title,
      preset_focus: row.preset_focus,
      run_kind: row.run_kind,
      channels: row.channels,
      state_hash_short: row.state_hash_short,
      recorded_at: row.recorded_at,
    }));

  return {
    schema: "seamgrim.numeric_track_result_timeline_view.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_history_filter.v1",
    result_count: timeline.length,
    first_recorded_at: timeline.length > 0 ? timeline[timeline.length - 1].recorded_at : "",
    latest_recorded_at: timeline.length > 0 ? timeline[0].recorded_at : "",
    rows: timeline,
  };
}

export function formatNumericTrackResultTimelineViewText(timelineView = null) {
  const row = timelineView && typeof timelineView === "object" ? timelineView : {};
  const rows = Array.isArray(row.rows) ? row.rows : [];
  const lines = rows.map((item) => [
    item.index ?? 0,
    item.recorded_at ?? "",
    item.lesson_id ?? "",
    item.title ?? "",
    item.preset_focus ?? "",
    item.run_kind ?? "",
    item.channels ?? 0,
    item.state_hash_short ?? "",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `result_count\t${rows.length}`,
    `latest_recorded_at\t${row.latest_recorded_at ?? ""}`,
    `first_recorded_at\t${row.first_recorded_at ?? ""}`,
    "",
    "index\trecorded_at\tlesson_id\ttitle\tpreset_focus\trun_kind\tchannels\tstate_hash_short",
    ...lines,
  ].join("\n");
}

export function buildNumericTrackResultReopenTarget(timelineRow = null) {
  const row = timelineRow && typeof timelineRow === "object" ? timelineRow : {};
  const lessonId = String(row.lesson_id ?? "").trim();
  if (!lessonId) return null;
  return {
    schema: "seamgrim.numeric_track_result_reopen_target.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_timeline_view.v1",
    lesson_id: lessonId,
    title: String(row.title ?? lessonId),
    preset_focus: String(row.preset_focus ?? ""),
    run_kind: String(row.run_kind ?? ""),
    channels: Math.max(0, Number.isFinite(Number(row.channels)) ? Math.trunc(Number(row.channels)) : 0),
    state_hash_short: String(row.state_hash_short ?? ""),
    recorded_at: String(row.recorded_at ?? ""),
    reopen_action: "browse_detail",
    replay_claim: false,
  };
}

export function formatNumericTrackResultReopenTargetText(target = null) {
  const row = target && typeof target === "object" ? target : {};
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `lesson_id\t${row.lesson_id ?? ""}`,
    `title\t${row.title ?? ""}`,
    `preset_focus\t${row.preset_focus ?? ""}`,
    `run_kind\t${row.run_kind ?? ""}`,
    `channels\t${row.channels ?? 0}`,
    `state_hash_short\t${row.state_hash_short ?? ""}`,
    `recorded_at\t${row.recorded_at ?? ""}`,
    `reopen_action\t${row.reopen_action ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
  ].join("\n");
}

export function buildNumericTrackResultCompare(timelineView = null) {
  const view = timelineView && typeof timelineView === "object" ? timelineView : {};
  const rows = Array.isArray(view.rows) ? view.rows : [];
  if (rows.length < 2) return null;
  const latest = rows[0] && typeof rows[0] === "object" ? rows[0] : {};
  const previous = rows[1] && typeof rows[1] === "object" ? rows[1] : {};
  const latestChannels = Math.max(0, Number.isFinite(Number(latest.channels)) ? Math.trunc(Number(latest.channels)) : 0);
  const previousChannels = Math.max(0, Number.isFinite(Number(previous.channels)) ? Math.trunc(Number(previous.channels)) : 0);
  return {
    schema: "seamgrim.numeric_track_result_compare.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_timeline_view.v1",
    compare_claim: "metadata_only",
    replay_claim: false,
    compare_kind: "latest_vs_previous",
    latest: {
      lesson_id: String(latest.lesson_id ?? ""),
      title: String(latest.title ?? latest.lesson_id ?? ""),
      preset_focus: String(latest.preset_focus ?? ""),
      run_kind: String(latest.run_kind ?? ""),
      channels: latestChannels,
      state_hash_short: String(latest.state_hash_short ?? ""),
      recorded_at: String(latest.recorded_at ?? ""),
    },
    previous: {
      lesson_id: String(previous.lesson_id ?? ""),
      title: String(previous.title ?? previous.lesson_id ?? ""),
      preset_focus: String(previous.preset_focus ?? ""),
      run_kind: String(previous.run_kind ?? ""),
      channels: previousChannels,
      state_hash_short: String(previous.state_hash_short ?? ""),
      recorded_at: String(previous.recorded_at ?? ""),
    },
    same_lesson: String(latest.lesson_id ?? "") === String(previous.lesson_id ?? ""),
    same_focus: String(latest.preset_focus ?? "") === String(previous.preset_focus ?? ""),
    same_run_kind: String(latest.run_kind ?? "") === String(previous.run_kind ?? ""),
    channel_delta: latestChannels - previousChannels,
    state_hash_changed: String(latest.state_hash_short ?? "") !== String(previous.state_hash_short ?? ""),
  };
}

export function formatNumericTrackResultCompareText(compare = null) {
  const row = compare && typeof compare === "object" ? compare : {};
  const latest = row.latest && typeof row.latest === "object" ? row.latest : {};
  const previous = row.previous && typeof row.previous === "object" ? row.previous : {};
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `compare_kind\t${row.compare_kind ?? ""}`,
    `compare_claim\t${row.compare_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    "",
    "side\tlesson_id\ttitle\tpreset_focus\trun_kind\tchannels\tstate_hash_short\trecorded_at",
    [
      "latest",
      latest.lesson_id ?? "",
      latest.title ?? "",
      latest.preset_focus ?? "",
      latest.run_kind ?? "",
      latest.channels ?? 0,
      latest.state_hash_short ?? "",
      latest.recorded_at ?? "",
    ].join("\t"),
    [
      "previous",
      previous.lesson_id ?? "",
      previous.title ?? "",
      previous.preset_focus ?? "",
      previous.run_kind ?? "",
      previous.channels ?? 0,
      previous.state_hash_short ?? "",
      previous.recorded_at ?? "",
    ].join("\t"),
    "",
    `same_lesson\t${row.same_lesson === true ? "true" : "false"}`,
    `same_focus\t${row.same_focus === true ? "true" : "false"}`,
    `same_run_kind\t${row.same_run_kind === true ? "true" : "false"}`,
    `channel_delta\t${row.channel_delta ?? 0}`,
    `state_hash_changed\t${row.state_hash_changed === true ? "true" : "false"}`,
  ].join("\n");
}

export function buildNumericTrackResultCompareExport(compare = null) {
  const row = compare && typeof compare === "object" ? compare : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare.v1") return null;
  const latest = row.latest && typeof row.latest === "object" ? row.latest : {};
  const previous = row.previous && typeof row.previous === "object" ? row.previous : {};
  const compareText = formatNumericTrackResultCompareText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    compare_kind: String(row.compare_kind ?? ""),
    latest_lesson_id: String(latest.lesson_id ?? ""),
    previous_lesson_id: String(previous.lesson_id ?? ""),
    same_lesson: row.same_lesson === true,
    same_focus: row.same_focus === true,
    same_run_kind: row.same_run_kind === true,
    channel_delta: Number.isFinite(Number(row.channel_delta)) ? Number(row.channel_delta) : 0,
    state_hash_changed: row.state_hash_changed === true,
    compare: row,
    compare_text: compareText,
  };
}

export function formatNumericTrackResultCompareExportText(compareExport = null) {
  const row = compareExport && typeof compareExport === "object" ? compareExport : {};
  const compareText = String(row.compare_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `compare_kind\t${row.compare_kind ?? ""}`,
    `latest_lesson_id\t${row.latest_lesson_id ?? ""}`,
    `previous_lesson_id\t${row.previous_lesson_id ?? ""}`,
    `same_lesson\t${row.same_lesson === true ? "true" : "false"}`,
    `same_focus\t${row.same_focus === true ? "true" : "false"}`,
    `same_run_kind\t${row.same_run_kind === true ? "true" : "false"}`,
    `channel_delta\t${row.channel_delta ?? 0}`,
    `state_hash_changed\t${row.state_hash_changed === true ? "true" : "false"}`,
    "",
    "compare_text",
    compareText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistory(timelineView = null) {
  const view = timelineView && typeof timelineView === "object" ? timelineView : {};
  const rows = Array.isArray(view.rows) ? view.rows : [];
  const pairs = [];
  for (let index = 0; index + 1 < rows.length; index += 1) {
    const pairTimeline = {
      schema: "seamgrim.numeric_track_result_timeline_view.v1",
      track_id: NUMERIC_TRACK_ID,
      source_schema: view.source_schema ?? "",
      result_count: 2,
      rows: [rows[index], rows[index + 1]],
    };
    const compare = buildNumericTrackResultCompare(pairTimeline);
    if (!compare) continue;
    pairs.push({
      index,
      latest_lesson_id: compare.latest.lesson_id,
      previous_lesson_id: compare.previous.lesson_id,
      latest_recorded_at: compare.latest.recorded_at,
      previous_recorded_at: compare.previous.recorded_at,
      same_lesson: compare.same_lesson,
      same_focus: compare.same_focus,
      same_run_kind: compare.same_run_kind,
      channel_delta: compare.channel_delta,
      state_hash_changed: compare.state_hash_changed,
      compare,
    });
  }
  return {
    schema: "seamgrim.numeric_track_result_compare_history.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_timeline_view.v1",
    compare_claim: "metadata_only",
    replay_claim: false,
    pair_count: pairs.length,
    pairs,
  };
}

export function formatNumericTrackResultCompareHistoryText(history = null) {
  const row = history && typeof history === "object" ? history : {};
  const pairs = Array.isArray(row.pairs) ? row.pairs : [];
  const lines = pairs.map((item) => [
    item.index ?? 0,
    item.latest_lesson_id ?? "",
    item.previous_lesson_id ?? "",
    item.latest_recorded_at ?? "",
    item.previous_recorded_at ?? "",
    item.same_lesson === true ? "true" : "false",
    item.same_focus === true ? "true" : "false",
    item.same_run_kind === true ? "true" : "false",
    item.channel_delta ?? 0,
    item.state_hash_changed === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `compare_claim\t${row.compare_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${pairs.length}`,
    "",
    "index\tlatest_lesson_id\tprevious_lesson_id\tlatest_recorded_at\tprevious_recorded_at\tsame_lesson\tsame_focus\tsame_run_kind\tchannel_delta\tstate_hash_changed",
    ...lines,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryExport(history = null) {
  const row = history && typeof history === "object" ? history : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history.v1") return null;
  const pairs = Array.isArray(row.pairs) ? row.pairs : [];
  const historyText = formatNumericTrackResultCompareHistoryText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    pair_count: pairs.length,
    history: row,
    history_text: historyText,
  };
}

export function formatNumericTrackResultCompareHistoryExportText(historyExport = null) {
  const row = historyExport && typeof historyExport === "object" ? historyExport : {};
  const historyText = String(row.history_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? 0}`,
    "",
    "history_text",
    historyText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReport(history = null) {
  const row = history && typeof history === "object" ? history : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history.v1") return null;
  const pairs = Array.isArray(row.pairs) ? row.pairs : [];
  const rows = pairs.map((pair) => ({
    index: Number.isFinite(Number(pair.index)) ? Math.trunc(Number(pair.index)) : 0,
    latest_lesson_id: String(pair.latest_lesson_id ?? ""),
    previous_lesson_id: String(pair.previous_lesson_id ?? ""),
    same_lesson: pair.same_lesson === true,
    same_focus: pair.same_focus === true,
    same_run_kind: pair.same_run_kind === true,
    channel_delta: Number.isFinite(Number(pair.channel_delta)) ? Number(pair.channel_delta) : 0,
    state_hash_changed: pair.state_hash_changed === true,
  }));
  const stateHashChangedCount = rows.filter((item) => item.state_hash_changed).length;
  const sameLessonCount = rows.filter((item) => item.same_lesson).length;
  const sameFocusCount = rows.filter((item) => item.same_focus).length;
  const sameRunKindCount = rows.filter((item) => item.same_run_kind).length;
  const runKindChangedCount = rows.length - sameRunKindCount;
  const channelDeltaTotal = rows.reduce((sum, item) => sum + item.channel_delta, 0);
  const channelDeltaAbsTotal = rows.reduce((sum, item) => sum + Math.abs(item.channel_delta), 0);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history.v1",
    report_claim: "metadata_summary",
    replay_claim: false,
    pair_count: rows.length,
    state_hash_changed_count: stateHashChangedCount,
    state_hash_same_count: rows.length - stateHashChangedCount,
    same_lesson_count: sameLessonCount,
    same_focus_count: sameFocusCount,
    same_run_kind_count: sameRunKindCount,
    run_kind_changed_count: runKindChangedCount,
    channel_delta_total: channelDeltaTotal,
    channel_delta_abs_total: channelDeltaAbsTotal,
    rows,
  };
}

export function formatNumericTrackResultCompareHistoryReportText(report = null) {
  const row = report && typeof report === "object" ? report : {};
  const rows = Array.isArray(row.rows) ? row.rows : [];
  const lines = rows.map((item) => [
    item.index ?? 0,
    item.latest_lesson_id ?? "",
    item.previous_lesson_id ?? "",
    item.same_lesson === true ? "true" : "false",
    item.same_focus === true ? "true" : "false",
    item.same_run_kind === true ? "true" : "false",
    item.channel_delta ?? 0,
    item.state_hash_changed === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `report_claim\t${row.report_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? rows.length}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `state_hash_same_count\t${row.state_hash_same_count ?? 0}`,
    `same_lesson_count\t${row.same_lesson_count ?? 0}`,
    `same_focus_count\t${row.same_focus_count ?? 0}`,
    `same_run_kind_count\t${row.same_run_kind_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    "index\tlatest_lesson_id\tprevious_lesson_id\tsame_lesson\tsame_focus\tsame_run_kind\tchannel_delta\tstate_hash_changed",
    ...lines,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTable(report = null) {
  const row = report && typeof report === "object" ? report : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report.v1") return null;
  const sourceRows = Array.isArray(row.rows) ? row.rows : [];
  const columns = [
    "index",
    "latest_lesson_id",
    "previous_lesson_id",
    "same_lesson",
    "same_focus",
    "same_run_kind",
    "channel_delta",
    "state_hash_changed",
  ];
  const rows = sourceRows.map((item) => ({
    index: Number.isFinite(Number(item.index)) ? Math.trunc(Number(item.index)) : 0,
    latest_lesson_id: String(item.latest_lesson_id ?? ""),
    previous_lesson_id: String(item.previous_lesson_id ?? ""),
    same_lesson: item.same_lesson === true,
    same_focus: item.same_focus === true,
    same_run_kind: item.same_run_kind === true,
    channel_delta: Number.isFinite(Number(item.channel_delta)) ? Number(item.channel_delta) : 0,
    state_hash_changed: item.state_hash_changed === true,
  }));
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report.v1",
    table_claim: "metadata_table",
    replay_claim: false,
    pair_count: rows.length,
    column_count: columns.length,
    columns,
    rows,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_total: Number.isFinite(Number(row.channel_delta_total)) ? Number(row.channel_delta_total) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableText(table = null) {
  const row = table && typeof table === "object" ? table : {};
  const columns = Array.isArray(row.columns) ? row.columns : [];
  const rows = Array.isArray(row.rows) ? row.rows : [];
  const tableLines = rows.map((item) => [
    item.index ?? 0,
    item.latest_lesson_id ?? "",
    item.previous_lesson_id ?? "",
    item.same_lesson === true ? "true" : "false",
    item.same_focus === true ? "true" : "false",
    item.same_run_kind === true ? "true" : "false",
    item.channel_delta ?? 0,
    item.state_hash_changed === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `table_claim\t${row.table_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? rows.length}`,
    `column_count\t${row.column_count ?? columns.length}`,
    `columns\t${columns.join("|")}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    columns.length > 0 ? columns.join("\t") : "index\tlatest_lesson_id\tprevious_lesson_id\tsame_lesson\tsame_focus\tsame_run_kind\tchannel_delta\tstate_hash_changed",
    ...tableLines,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableExport(table = null) {
  const row = table && typeof table === "object" ? table : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table.v1") return null;
  const tableText = formatNumericTrackResultCompareHistoryReportTableText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    column_count: Number.isFinite(Number(row.column_count)) ? Math.trunc(Number(row.column_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_total: Number.isFinite(Number(row.channel_delta_total)) ? Number(row.channel_delta_total) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
    table: row,
    table_text: tableText,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableExportText(tableExport = null) {
  const row = tableExport && typeof tableExport === "object" ? tableExport : {};
  const tableText = String(row.table_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `column_count\t${row.column_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    "table_text",
    tableText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableSummary(table = null) {
  const row = table && typeof table === "object" ? table : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table.v1") return null;
  const rows = Array.isArray(row.rows) ? row.rows : [];
  const lessonIds = [];
  const seenLessonIds = new Set();
  rows.forEach((item) => {
    [item.latest_lesson_id, item.previous_lesson_id].forEach((lessonId) => {
      const value = String(lessonId ?? "").trim();
      if (!value || seenLessonIds.has(value)) return;
      seenLessonIds.add(value);
      lessonIds.push(value);
    });
  });
  const sameRunKindCount = rows.filter((item) => item.same_run_kind === true).length;
  const runKindChangedCount = rows.length - sameRunKindCount;
  const stateHashChangedCount = rows.filter((item) => item.state_hash_changed === true).length;
  const channelDeltaTotal = rows.reduce((sum, item) => (
    sum + (Number.isFinite(Number(item.channel_delta)) ? Number(item.channel_delta) : 0)
  ), 0);
  const channelDeltaAbsTotal = rows.reduce((sum, item) => (
    sum + Math.abs(Number.isFinite(Number(item.channel_delta)) ? Number(item.channel_delta) : 0)
  ), 0);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_summary.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table.v1",
    summary_claim: "metadata_summary",
    replay_claim: false,
    pair_count: rows.length,
    column_count: Number.isFinite(Number(row.column_count)) ? Math.trunc(Number(row.column_count)) : 0,
    lesson_count: lessonIds.length,
    lesson_ids: lessonIds,
    state_hash_changed_count: stateHashChangedCount,
    state_hash_same_count: rows.length - stateHashChangedCount,
    same_run_kind_count: sameRunKindCount,
    run_kind_changed_count: runKindChangedCount,
    channel_delta_total: channelDeltaTotal,
    channel_delta_abs_total: channelDeltaAbsTotal,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableSummaryText(summary = null) {
  const row = summary && typeof summary === "object" ? summary : {};
  const lessonIds = Array.isArray(row.lesson_ids) ? row.lesson_ids : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `summary_claim\t${row.summary_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `column_count\t${row.column_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? lessonIds.length}`,
    `lesson_ids\t${lessonIds.join("|")}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `state_hash_same_count\t${row.state_hash_same_count ?? 0}`,
    `same_run_kind_count\t${row.same_run_kind_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableSummaryExport(summary = null) {
  const row = summary && typeof summary === "object" ? summary : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_summary.v1") return null;
  const summaryText = formatNumericTrackResultCompareHistoryReportTableSummaryText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_summary_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_summary.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    column_count: Number.isFinite(Number(row.column_count)) ? Math.trunc(Number(row.column_count)) : 0,
    lesson_count: Number.isFinite(Number(row.lesson_count)) ? Math.trunc(Number(row.lesson_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_total: Number.isFinite(Number(row.channel_delta_total)) ? Number(row.channel_delta_total) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
    summary: row,
    summary_text: summaryText,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableSummaryExportText(summaryExport = null) {
  const row = summaryExport && typeof summaryExport === "object" ? summaryExport : {};
  const summaryText = String(row.summary_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `column_count\t${row.column_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    "summary_text",
    summaryText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatus(summary = null) {
  const row = summary && typeof summary === "object" ? summary : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_summary.v1") return null;
  const pairCount = Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0;
  const columnCount = Number.isFinite(Number(row.column_count)) ? Math.trunc(Number(row.column_count)) : 0;
  const lessonCount = Number.isFinite(Number(row.lesson_count)) ? Math.trunc(Number(row.lesson_count)) : 0;
  const stateHashChangedCount = Number.isFinite(Number(row.state_hash_changed_count))
    ? Math.trunc(Number(row.state_hash_changed_count))
    : 0;
  const runKindChangedCount = Number.isFinite(Number(row.run_kind_changed_count))
    ? Math.trunc(Number(row.run_kind_changed_count))
    : 0;
  const channelDeltaTotal = Number.isFinite(Number(row.channel_delta_total)) ? Number(row.channel_delta_total) : 0;
  const channelDeltaAbsTotal = Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0;
  const statusReasons = [];
  if (stateHashChangedCount > 0) statusReasons.push("state_hash_changed");
  if (runKindChangedCount > 0) statusReasons.push("run_kind_changed");
  if (channelDeltaAbsTotal > 0) statusReasons.push("channel_delta_nonzero");
  const hasChanges = statusReasons.length > 0;
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_summary.v1",
    status_claim: "metadata_status",
    replay_claim: false,
    pair_count: pairCount,
    column_count: columnCount,
    lesson_count: lessonCount,
    state_hash_changed_count: stateHashChangedCount,
    run_kind_changed_count: runKindChangedCount,
    channel_delta_total: channelDeltaTotal,
    channel_delta_abs_total: channelDeltaAbsTotal,
    has_changes: hasChanges,
    status: hasChanges ? "변화있음" : "변화없음",
    status_reasons: statusReasons,
    summary: row,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusText(status = null) {
  const row = status && typeof status === "object" ? status : {};
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `status_claim\t${row.status_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `has_changes\t${row.has_changes === true ? "true" : "false"}`,
    `status_reasons\t${reasons.join("|")}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `column_count\t${row.column_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusExport(status = null) {
  const row = status && typeof status === "object" ? status : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status.v1") return null;
  const statusText = formatNumericTrackResultCompareHistoryReportTableStatusText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    status_value: row.status ?? "",
    has_changes: row.has_changes === true,
    status_reasons: Array.isArray(row.status_reasons) ? [...row.status_reasons] : [],
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    column_count: Number.isFinite(Number(row.column_count)) ? Math.trunc(Number(row.column_count)) : 0,
    lesson_count: Number.isFinite(Number(row.lesson_count)) ? Math.trunc(Number(row.lesson_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_total: Number.isFinite(Number(row.channel_delta_total)) ? Number(row.channel_delta_total) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
    status: row,
    status_text: statusText,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusExportText(statusExport = null) {
  const row = statusExport && typeof statusExport === "object" ? statusExport : {};
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const statusText = String(row.status_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status_value ?? row.status?.status ?? ""}`,
    `has_changes\t${row.has_changes === true ? "true" : "false"}`,
    `status_reasons\t${reasons.join("|")}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `column_count\t${row.column_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    "status_text",
    statusText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadge(status = null) {
  const row = status && typeof status === "object" ? status : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status.v1") return null;
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const hasChanges = row.has_changes === true;
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status.v1",
    badge_claim: "metadata_badge",
    replay_claim: false,
    label: String(row.status ?? (hasChanges ? "변화있음" : "변화없음")),
    tone: hasChanges ? "warning" : "success",
    has_changes: hasChanges,
    reason_count: reasons.length,
    status_reasons: [...reasons],
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    lesson_count: Number.isFinite(Number(row.lesson_count)) ? Math.trunc(Number(row.lesson_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
    status: row,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeText(badge = null) {
  const row = badge && typeof badge === "object" ? badge : {};
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `badge_claim\t${row.badge_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `label\t${row.label ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `has_changes\t${row.has_changes === true ? "true" : "false"}`,
    `reason_count\t${row.reason_count ?? reasons.length}`,
    `status_reasons\t${reasons.join("|")}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadgeExport(badge = null) {
  const row = badge && typeof badge === "object" ? badge : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1") return null;
  const badgeText = formatNumericTrackResultCompareHistoryReportTableStatusBadgeText(row);
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    label: String(row.label ?? ""),
    tone: String(row.tone ?? ""),
    has_changes: row.has_changes === true,
    reason_count: Number.isFinite(Number(row.reason_count)) ? Math.trunc(Number(row.reason_count)) : reasons.length,
    status_reasons: [...reasons],
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    lesson_count: Number.isFinite(Number(row.lesson_count)) ? Math.trunc(Number(row.lesson_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
    badge: row,
    badge_text: badgeText,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeExportText(badgeExport = null) {
  const row = badgeExport && typeof badgeExport === "object" ? badgeExport : {};
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const badgeText = String(row.badge_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `label\t${row.label ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `has_changes\t${row.has_changes === true ? "true" : "false"}`,
    `reason_count\t${row.reason_count ?? reasons.length}`,
    `status_reasons\t${reasons.join("|")}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    "badge_text",
    badgeText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11y(badge = null) {
  const row = badge && typeof badge === "object" ? badge : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1") return null;
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const label = String(row.label ?? "");
  const tone = String(row.tone ?? "");
  const reasonCount = Number.isFinite(Number(row.reason_count)) ? Math.trunc(Number(row.reason_count)) : reasons.length;
  const ariaLabel = `numeric status badge: ${label}; tone ${tone}; reasons ${reasonCount}; replay false`;
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge.v1",
    a11y_claim: "non_color_status_badge",
    replay_claim: false,
    role: "status",
    label,
    tone,
    aria_label: ariaLabel,
    title: ariaLabel,
    has_changes: row.has_changes === true,
    reason_count: reasonCount,
    status_reasons: [...reasons],
    color_only_claim: false,
    badge: row,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yText(a11y = null) {
  const row = a11y && typeof a11y === "object" ? a11y : {};
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `a11y_claim\t${row.a11y_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `role\t${row.role ?? ""}`,
    `label\t${row.label ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `aria_label\t${row.aria_label ?? ""}`,
    `title\t${row.title ?? ""}`,
    `has_changes\t${row.has_changes === true ? "true" : "false"}`,
    `reason_count\t${row.reason_count ?? reasons.length}`,
    `status_reasons\t${reasons.join("|")}`,
    `color_only_claim\t${row.color_only_claim === true ? "true" : "false"}`,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(a11y = null) {
  const row = a11y && typeof a11y === "object" ? a11y : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y.v1") return null;
  const a11yText = formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yText(row);
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    a11y_claim: String(row.a11y_claim ?? ""),
    role: String(row.role ?? ""),
    label: String(row.label ?? ""),
    tone: String(row.tone ?? ""),
    aria_label: String(row.aria_label ?? ""),
    title: String(row.title ?? ""),
    has_changes: row.has_changes === true,
    reason_count: Number.isFinite(Number(row.reason_count)) ? Math.trunc(Number(row.reason_count)) : reasons.length,
    status_reasons: [...reasons],
    color_only_claim: row.color_only_claim === true,
    a11y: row,
    a11y_text: a11yText,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExportText(a11yExport = null) {
  const row = a11yExport && typeof a11yExport === "object" ? a11yExport : {};
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const a11yText = String(row.a11y_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `a11y_claim\t${row.a11y_claim ?? ""}`,
    `role\t${row.role ?? ""}`,
    `label\t${row.label ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `aria_label\t${row.aria_label ?? ""}`,
    `title\t${row.title ?? ""}`,
    `has_changes\t${row.has_changes === true ? "true" : "false"}`,
    `reason_count\t${row.reason_count ?? reasons.length}`,
    `status_reasons\t${reasons.join("|")}`,
    `color_only_claim\t${row.color_only_claim === true ? "true" : "false"}`,
    "",
    "a11y_text",
    a11yText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatus(a11yExport = null) {
  const row = a11yExport && typeof a11yExport === "object" ? a11yExport : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_export.v1") return null;
  const statusChecks = [
    ["export_present", row.export_claim === "metadata_text"],
    ["no_replay_claim", row.replay_claim === false],
    ["non_color_claim", row.a11y_claim === "non_color_status_badge"],
    ["role_status", row.role === "status"],
    ["aria_label_present", String(row.aria_label ?? "").trim().length > 0],
    ["title_present", String(row.title ?? "").trim().length > 0],
    ["color_only_false", row.color_only_claim === false],
  ];
  const missingChecks = statusChecks.filter(([, ok]) => !ok).map(([name]) => name);
  const ready = missingChecks.length === 0;
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_export.v1",
    status_claim: "metadata_status",
    replay_claim: false,
    status: ready ? "a11y_ready" : "a11y_incomplete",
    tone: ready ? "success" : "warning",
    ready,
    check_count: statusChecks.length,
    passing_check_count: statusChecks.length - missingChecks.length,
    missing_check_count: missingChecks.length,
    missing_checks: missingChecks,
    status_reasons: statusChecks.map(([name]) => name),
    a11y_export: row,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusText(status = null) {
  const row = status && typeof status === "object" ? status : {};
  const missing = Array.isArray(row.missing_checks) ? row.missing_checks : [];
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `status_claim\t${row.status_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `ready\t${row.ready === true ? "true" : "false"}`,
    `check_count\t${row.check_count ?? 0}`,
    `passing_check_count\t${row.passing_check_count ?? 0}`,
    `missing_check_count\t${row.missing_check_count ?? missing.length}`,
    `missing_checks\t${missing.join("|")}`,
    `status_reasons\t${reasons.join("|")}`,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(status = null) {
  const row = status && typeof status === "object" ? status : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status.v1") return null;
  const missing = Array.isArray(row.missing_checks) ? row.missing_checks : [];
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const statusText = formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status.v1",
    export_claim: "metadata_text",
    status_claim: String(row.status_claim ?? ""),
    replay_claim: false,
    status_value: String(row.status ?? ""),
    tone: String(row.tone ?? ""),
    ready: row.ready === true,
    check_count: Number.isFinite(Number(row.check_count)) ? Math.trunc(Number(row.check_count)) : reasons.length,
    passing_check_count: Number.isFinite(Number(row.passing_check_count)) ? Math.trunc(Number(row.passing_check_count)) : 0,
    missing_check_count: Number.isFinite(Number(row.missing_check_count)) ? Math.trunc(Number(row.missing_check_count)) : missing.length,
    missing_checks: [...missing],
    status_reasons: [...reasons],
    status: row,
    status_text: statusText,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportText(statusExport = null) {
  const row = statusExport && typeof statusExport === "object" ? statusExport : {};
  const missing = Array.isArray(row.missing_checks) ? row.missing_checks : [];
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const statusText = String(row.status_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `status_claim\t${row.status_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status_value ?? row.status?.status ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `ready\t${row.ready === true ? "true" : "false"}`,
    `check_count\t${row.check_count ?? reasons.length}`,
    `passing_check_count\t${row.passing_check_count ?? 0}`,
    `missing_check_count\t${row.missing_check_count ?? missing.length}`,
    `missing_checks\t${missing.join("|")}`,
    `status_reasons\t${reasons.join("|")}`,
    "",
    "status_text",
    statusText,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummary(statusExport = null) {
  const row = statusExport && typeof statusExport === "object" ? statusExport : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export.v1") return null;
  const missing = Array.isArray(row.missing_checks) ? row.missing_checks : [];
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  const statusText = String(row.status_text ?? "");
  const summaryChecks = [
    ["export_present", row.export_claim === "metadata_text"],
    ["status_claim_present", row.status_claim === "metadata_status"],
    ["no_replay_claim", row.replay_claim === false],
    ["ready_true", row.ready === true],
    ["status_text_present", statusText.trim().length > 0],
  ];
  const missingSummaryChecks = summaryChecks.filter(([, ok]) => !ok).map(([name]) => name);
  const ready = missingSummaryChecks.length === 0;
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export_summary.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report_table_status_badge_a11y_status_export.v1",
    summary_claim: "metadata_summary",
    export_claim: String(row.export_claim ?? ""),
    status_claim: String(row.status_claim ?? ""),
    replay_claim: false,
    status: ready ? "summary_ready" : "summary_incomplete",
    tone: ready ? "success" : "warning",
    ready,
    status_value: String(row.status_value ?? row.status?.status ?? ""),
    status_tone: String(row.tone ?? ""),
    source_ready: row.ready === true,
    check_count: Number.isFinite(Number(row.check_count)) ? Math.trunc(Number(row.check_count)) : reasons.length,
    missing_check_count: Number.isFinite(Number(row.missing_check_count)) ? Math.trunc(Number(row.missing_check_count)) : missing.length,
    status_text_line_count: statusText ? statusText.split("\n").length : 0,
    summary_check_count: summaryChecks.length,
    passing_summary_check_count: summaryChecks.length - missingSummaryChecks.length,
    missing_summary_check_count: missingSummaryChecks.length,
    missing_summary_checks: missingSummaryChecks,
    status_reasons: [...reasons],
    status_export: row,
  };
}

export function formatNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummaryText(summary = null) {
  const row = summary && typeof summary === "object" ? summary : {};
  const missing = Array.isArray(row.missing_summary_checks) ? row.missing_summary_checks : [];
  const reasons = Array.isArray(row.status_reasons) ? row.status_reasons : [];
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `summary_claim\t${row.summary_claim ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `status_claim\t${row.status_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `ready\t${row.ready === true ? "true" : "false"}`,
    `status_value\t${row.status_value ?? ""}`,
    `status_tone\t${row.status_tone ?? ""}`,
    `source_ready\t${row.source_ready === true ? "true" : "false"}`,
    `check_count\t${row.check_count ?? 0}`,
    `missing_check_count\t${row.missing_check_count ?? 0}`,
    `status_text_line_count\t${row.status_text_line_count ?? 0}`,
    `summary_check_count\t${row.summary_check_count ?? 0}`,
    `passing_summary_check_count\t${row.passing_summary_check_count ?? 0}`,
    `missing_summary_check_count\t${row.missing_summary_check_count ?? missing.length}`,
    `missing_summary_checks\t${missing.join("|")}`,
    `status_reasons\t${reasons.join("|")}`,
  ].join("\n");
}

export function buildNumericReportWorkflowConsolidation(history = null) {
  const row = history && typeof history === "object" ? history : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history.v1") return null;
  const historyExport = buildNumericTrackResultCompareHistoryExport(row);
  const report = buildNumericTrackResultCompareHistoryReport(row);
  const reportExport = buildNumericTrackResultCompareHistoryReportExport(report);
  const table = buildNumericTrackResultCompareHistoryReportTable(report);
  const tableExport = buildNumericTrackResultCompareHistoryReportTableExport(table);
  const tableSummary = buildNumericTrackResultCompareHistoryReportTableSummary(table);
  const tableSummaryExport = buildNumericTrackResultCompareHistoryReportTableSummaryExport(tableSummary);
  const tableStatus = buildNumericTrackResultCompareHistoryReportTableStatus(tableSummary);
  const tableStatusExport = buildNumericTrackResultCompareHistoryReportTableStatusExport(tableStatus);
  const badge = buildNumericTrackResultCompareHistoryReportTableStatusBadge(tableStatus);
  const badgeExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeExport(badge);
  const badgeA11y = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11y(badge);
  const badgeA11yExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yExport(badgeA11y);
  const badgeA11yStatus = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatus(badgeA11yExport);
  const badgeA11yStatusExport = buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExport(badgeA11yStatus);
  const badgeA11yStatusExportSummary =
    buildNumericTrackResultCompareHistoryReportTableStatusBadgeA11yStatusExportSummary(badgeA11yStatusExport);
  const stages = [
    ["compare_history", row, row.pair_count > 0],
    ["compare_history_export", historyExport, Boolean(historyExport)],
    ["history_report", report, Boolean(report)],
    ["history_report_export", reportExport, Boolean(reportExport)],
    ["report_table", table, Boolean(table) && Array.isArray(table.rows)],
    ["report_table_export", tableExport, Boolean(tableExport)],
    ["report_table_summary", tableSummary, Boolean(tableSummary)],
    ["report_table_summary_export", tableSummaryExport, Boolean(tableSummaryExport)],
    ["report_table_status", tableStatus, Boolean(tableStatus)],
    ["report_table_status_export", tableStatusExport, Boolean(tableStatusExport)],
    ["status_badge", badge, Boolean(badge)],
    ["status_badge_export", badgeExport, Boolean(badgeExport)],
    ["status_badge_a11y", badgeA11y, Boolean(badgeA11y)],
    ["status_badge_a11y_export", badgeA11yExport, Boolean(badgeA11yExport)],
    ["status_badge_a11y_status", badgeA11yStatus, Boolean(badgeA11yStatus)],
    ["status_badge_a11y_status_export", badgeA11yStatusExport, Boolean(badgeA11yStatusExport)],
    ["status_badge_a11y_status_export_summary", badgeA11yStatusExportSummary, Boolean(badgeA11yStatusExportSummary?.ready)],
  ].map(([stageId, artifact, ready]) => ({
    stage_id: stageId,
    schema: String(artifact?.schema ?? ""),
    ready: ready === true,
  }));
  const readyStageCount = stages.filter((stage) => stage.ready).length;
  const ready = stages.length > 0 && readyStageCount === stages.length;
  const sourceSchemas = [];
  const seenSchemas = new Set();
  stages.forEach((stage) => {
    if (!stage.schema || seenSchemas.has(stage.schema)) return;
    seenSchemas.add(stage.schema);
    sourceSchemas.push(stage.schema);
  });
  return {
    schema: "seamgrim.numeric_report_workflow_consolidation.v1",
    track_id: NUMERIC_TRACK_ID,
    primary_coordinate: "마-3",
    workflow_claim: "product_workflow_consolidation",
    replay_claim: false,
    source_schemas: sourceSchemas,
    stage_count: stages.length,
    ready_stage_count: readyStageCount,
    missing_stage_count: stages.length - readyStageCount,
    status: ready ? "workflow_ready" : "workflow_incomplete",
    tone: ready ? "success" : "warning",
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    row_count: Array.isArray(table?.rows) ? table.rows.length : 0,
    lesson_count: Number.isFinite(Number(tableSummary?.lesson_count)) ? Math.trunc(Number(tableSummary.lesson_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(tableSummary?.state_hash_changed_count)) ? Math.trunc(Number(tableSummary.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(tableSummary?.run_kind_changed_count)) ? Math.trunc(Number(tableSummary.run_kind_changed_count)) : 0,
    channel_delta_abs_total: Number.isFinite(Number(tableSummary?.channel_delta_abs_total)) ? Number(tableSummary.channel_delta_abs_total) : 0,
    summary_status: String(badgeA11yStatusExportSummary?.status ?? ""),
    summary_ready: badgeA11yStatusExportSummary?.ready === true,
    stages,
    report_table_summary: tableSummary,
    a11y_status_export_summary: badgeA11yStatusExportSummary,
  };
}

export function formatNumericReportWorkflowConsolidationText(workflow = null) {
  const row = workflow && typeof workflow === "object" ? workflow : {};
  const schemas = Array.isArray(row.source_schemas) ? row.source_schemas : [];
  const stages = Array.isArray(row.stages) ? row.stages : [];
  const stageLines = stages.map((stage) => [
    stage.stage_id ?? "",
    stage.schema ?? "",
    stage.ready === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `primary_coordinate\t${row.primary_coordinate ?? ""}`,
    `workflow_claim\t${row.workflow_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `stage_count\t${row.stage_count ?? stages.length}`,
    `ready_stage_count\t${row.ready_stage_count ?? 0}`,
    `missing_stage_count\t${row.missing_stage_count ?? 0}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `row_count\t${row.row_count ?? 0}`,
    `lesson_count\t${row.lesson_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    `summary_status\t${row.summary_status ?? ""}`,
    `summary_ready\t${row.summary_ready === true ? "true" : "false"}`,
    `source_schemas\t${schemas.join("|")}`,
    "",
    "stage_id\tschema\tready",
    ...stageLines,
  ].join("\n");
}

export function buildNumericResultReportConsolidation({
  lessons = [],
  runPrefs = {},
} = {}) {
  const rows = Array.isArray(lessons) ? lessons : [];
  const prefs = runPrefs?.lessons && typeof runPrefs.lessons === "object" ? runPrefs.lessons : {};
  const resultLinks = rows
    .map((lesson) => {
      const lessonId = String(lesson?.id ?? "").trim();
      if (!lessonId) return null;
      return normalizeNumericTrackRunResultLink(prefs?.[lessonId]?.numericTrackRunResultLink);
    })
    .filter(Boolean);
  const history = buildNumericTrackResultHistorySnapshot({ lessons: rows, runPrefs });
  const summary = buildNumericTrackResultSummaryExport(history);
  const timeline = buildNumericTrackResultTimelineView(history);
  const compare = buildNumericTrackResultCompare(timeline);
  const compareHistory = buildNumericTrackResultCompareHistory(timeline);
  const reportWorkflow = buildNumericReportWorkflowConsolidation(compareHistory);
  const evidencePacks = [];
  const seenEvidence = new Set();
  resultLinks.forEach((link) => {
    (Array.isArray(link.evidence_packs) ? link.evidence_packs : []).forEach((packId) => {
      const item = String(packId ?? "").trim();
      if (!item || seenEvidence.has(item)) return;
      seenEvidence.add(item);
      evidencePacks.push(item);
    });
  });
  const stages = [
    ["run_result_links", "seamgrim.numeric_track_run_result_link.v1", resultLinks.length > 0],
    ["history_snapshot", history?.schema, Number(history?.result_count ?? 0) > 0],
    ["summary_export", summary?.schema, Number(summary?.result_count ?? 0) > 0],
    ["timeline_view", timeline?.schema, Number(timeline?.result_count ?? 0) > 0],
    ["latest_compare", compare?.schema, Boolean(compare)],
    ["compare_history", compareHistory?.schema, Number(compareHistory?.pair_count ?? 0) > 0],
    ["report_workflow", reportWorkflow?.schema, reportWorkflow?.status === "workflow_ready"],
    ["evidence_pack_rollup", "seamgrim.numeric_result_report_evidence_rollup.v1", evidencePacks.length > 0],
    ["no_replay_boundary", "seamgrim.numeric_result_report_boundary.v1", true],
    ["no_runtime_boundary", "seamgrim.numeric_result_report_boundary.v1", true],
  ].map(([stageId, schema, ready]) => ({
    stage_id: stageId,
    schema: String(schema ?? ""),
    ready: ready === true,
  }));
  const readyStageCount = stages.filter((stage) => stage.ready).length;
  const ready = stages.length > 0 && readyStageCount === stages.length;
  const sourceSchemas = [];
  const seenSchemas = new Set();
  [
    "seamgrim.numeric_track_run_result_link.v1",
    history?.schema,
    summary?.schema,
    timeline?.schema,
    compare?.schema,
    compareHistory?.schema,
    reportWorkflow?.schema,
  ].forEach((schema) => {
    const item = String(schema ?? "").trim();
    if (!item || seenSchemas.has(item)) return;
    seenSchemas.add(item);
    sourceSchemas.push(item);
  });
  return {
    schema: "seamgrim.numeric_result_report_consolidation.v1",
    track_id: NUMERIC_TRACK_ID,
    primary_coordinate: "마-3",
    support_coordinate: "다-2",
    workflow_claim: "numeric_result_report_consolidation",
    replay_claim: false,
    runtime_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    status: ready ? "numeric_result_report_ready" : "numeric_result_report_incomplete",
    tone: ready ? "success" : "warning",
    source_schemas: sourceSchemas,
    result_count: resultLinks.length,
    history_result_count: Number.isFinite(Number(history?.result_count)) ? Math.trunc(Number(history.result_count)) : 0,
    summary_result_count: Number.isFinite(Number(summary?.result_count)) ? Math.trunc(Number(summary.result_count)) : 0,
    timeline_result_count: Number.isFinite(Number(timeline?.result_count)) ? Math.trunc(Number(timeline.result_count)) : 0,
    pair_count: Number.isFinite(Number(compareHistory?.pair_count)) ? Math.trunc(Number(compareHistory.pair_count)) : 0,
    evidence_pack_count: evidencePacks.length,
    evidence_packs: evidencePacks,
    latest_recorded_at: String(timeline?.latest_recorded_at ?? summary?.latest_recorded_at ?? ""),
    report_workflow_status: String(reportWorkflow?.status ?? ""),
    report_workflow_stage_count: Number.isFinite(Number(reportWorkflow?.stage_count)) ? Math.trunc(Number(reportWorkflow.stage_count)) : 0,
    report_workflow_ready_stage_count: Number.isFinite(Number(reportWorkflow?.ready_stage_count)) ? Math.trunc(Number(reportWorkflow.ready_stage_count)) : 0,
    stage_count: stages.length,
    ready_stage_count: readyStageCount,
    missing_stage_count: stages.length - readyStageCount,
    stages,
    result_links: resultLinks,
    history,
    summary,
    timeline,
    compare,
    compare_history: compareHistory,
    report_workflow: reportWorkflow,
  };
}

export function formatNumericResultReportConsolidationText(consolidation = null) {
  const row = consolidation && typeof consolidation === "object" ? consolidation : {};
  const schemas = Array.isArray(row.source_schemas) ? row.source_schemas : [];
  const evidence = Array.isArray(row.evidence_packs) ? row.evidence_packs : [];
  const stages = Array.isArray(row.stages) ? row.stages : [];
  const stageLines = stages.map((stage) => [
    stage.stage_id ?? "",
    stage.schema ?? "",
    stage.ready === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `primary_coordinate\t${row.primary_coordinate ?? ""}`,
    `support_coordinate\t${row.support_coordinate ?? ""}`,
    `workflow_claim\t${row.workflow_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `runtime_claim\t${row.runtime_claim === true ? "true" : "false"}`,
    `lesson_schema_change\t${row.lesson_schema_change === true ? "true" : "false"}`,
    `active_allowlist_mutation\t${row.active_allowlist_mutation === true ? "true" : "false"}`,
    `status\t${row.status ?? ""}`,
    `tone\t${row.tone ?? ""}`,
    `result_count\t${row.result_count ?? 0}`,
    `history_result_count\t${row.history_result_count ?? 0}`,
    `summary_result_count\t${row.summary_result_count ?? 0}`,
    `timeline_result_count\t${row.timeline_result_count ?? 0}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `evidence_pack_count\t${row.evidence_pack_count ?? 0}`,
    `evidence_packs\t${evidence.join("|")}`,
    `latest_recorded_at\t${row.latest_recorded_at ?? ""}`,
    `report_workflow_status\t${row.report_workflow_status ?? ""}`,
    `report_workflow_stage_count\t${row.report_workflow_stage_count ?? 0}`,
    `report_workflow_ready_stage_count\t${row.report_workflow_ready_stage_count ?? 0}`,
    `stage_count\t${row.stage_count ?? stages.length}`,
    `ready_stage_count\t${row.ready_stage_count ?? 0}`,
    `missing_stage_count\t${row.missing_stage_count ?? 0}`,
    `source_schemas\t${schemas.join("|")}`,
    "",
    "stage_id\tschema\tready",
    ...stageLines,
  ].join("\n");
}

export function buildNumericTrackResultCompareHistoryReportExport(report = null) {
  const row = report && typeof report === "object" ? report : {};
  if (row.schema !== "seamgrim.numeric_track_result_compare_history_report.v1") return null;
  const reportText = formatNumericTrackResultCompareHistoryReportText(row);
  return {
    schema: "seamgrim.numeric_track_result_compare_history_report_export.v1",
    track_id: NUMERIC_TRACK_ID,
    source_schema: "seamgrim.numeric_track_result_compare_history_report.v1",
    export_claim: "metadata_text",
    replay_claim: false,
    pair_count: Number.isFinite(Number(row.pair_count)) ? Math.trunc(Number(row.pair_count)) : 0,
    state_hash_changed_count: Number.isFinite(Number(row.state_hash_changed_count)) ? Math.trunc(Number(row.state_hash_changed_count)) : 0,
    run_kind_changed_count: Number.isFinite(Number(row.run_kind_changed_count)) ? Math.trunc(Number(row.run_kind_changed_count)) : 0,
    channel_delta_total: Number.isFinite(Number(row.channel_delta_total)) ? Number(row.channel_delta_total) : 0,
    channel_delta_abs_total: Number.isFinite(Number(row.channel_delta_abs_total)) ? Number(row.channel_delta_abs_total) : 0,
    report: row,
    report_text: reportText,
  };
}

export function formatNumericTrackResultCompareHistoryReportExportText(reportExport = null) {
  const row = reportExport && typeof reportExport === "object" ? reportExport : {};
  const reportText = String(row.report_text ?? "");
  return [
    `schema\t${row.schema ?? ""}`,
    `track_id\t${row.track_id ?? ""}`,
    `source_schema\t${row.source_schema ?? ""}`,
    `export_claim\t${row.export_claim ?? ""}`,
    `replay_claim\t${row.replay_claim === true ? "true" : "false"}`,
    `pair_count\t${row.pair_count ?? 0}`,
    `state_hash_changed_count\t${row.state_hash_changed_count ?? 0}`,
    `run_kind_changed_count\t${row.run_kind_changed_count ?? 0}`,
    `channel_delta_total\t${row.channel_delta_total ?? 0}`,
    `channel_delta_abs_total\t${row.channel_delta_abs_total ?? 0}`,
    "",
    "report_text",
    reportText,
  ].join("\n");
}
