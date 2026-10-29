function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function asCount(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) return 0;
  return Math.max(0, Math.trunc(number));
}

function asBool(value) {
  return value === true || value === "true" || value === "참";
}

function asTextArray(value) {
  return asArray(value).map((item) => String(item ?? "").trim()).filter(Boolean);
}

function normalizeAssignment(assignment, index) {
  const row = asObject(assignment);
  const status = asText(row.status ?? row["상태"], "open");
  const resultViews = asTextArray(row.result_views ?? row.required_views ?? row.requiredViews ?? row["결과확인"]);
  return {
    assignment_id: asText(row.assignment_id ?? row.id ?? row["과제ID"], `assignment_${String(index + 1).padStart(3, "0")}`),
    title: asText(row.title ?? row.name ?? row["제목"], "untitled assignment"),
    lesson_id: asText(row.lesson_id ?? row.lesson ?? row["lesson"] ?? row["수업"], ""),
    goals: asTextArray(row.goals ?? row["수업목표"]),
    missions: asTextArray(row.missions ?? row["수업활동"]),
    result_views: resultViews,
    result_views_label: asText(row.result_views_label ?? row.required_views_label ?? row["결과확인라벨"], resultViews.join(", ")),
    due_label: asText(row.due_label ?? row.due ?? row["마감"], ""),
    status,
    is_open: status !== "closed" && status !== "닫힘",
  };
}

function normalizeNameList(value) {
  return asArray(value).map((item) => String(item ?? "")).filter((item) => item.length > 0);
}

function normalizeSuiteCheck(suiteCheck) {
  const payload = asObject(suiteCheck);
  const kind = asText(payload.__이음관계종류 ?? payload.kind);
  const judgement = asText(payload["판정"] ?? payload.judgement, "미실행");
  const failedCases = normalizeNameList(payload["실패케이스들"] ?? payload.failed_cases);
  const expectedFailPassed = normalizeNameList(payload["기대실패통과케이스들"] ?? payload.expected_fail_passed_cases);
  const expectedPassFailed = normalizeNameList(payload["기대통과실패케이스들"] ?? payload.expected_pass_failed_cases);
  return {
    __종류: "studio_classroom_suite_check_view",
    suite_check_kind: kind,
    judgement,
    overall_pass: asBool(payload["전체통과"] ?? payload.overall_pass) || judgement === "통과",
    case_count: asCount(payload["개수"] ?? payload.case_count),
    pass_count: asCount(payload["통과개수"] ?? payload.pass_count),
    fail_count: asCount(payload["실패개수"] ?? payload.fail_count ?? failedCases.length),
    failed_cases: failedCases,
    expected_fail_passed_cases: expectedFailPassed,
    expected_pass_failed_cases: expectedPassFailed,
    mismatch_count: expectedFailPassed.length + expectedPassFailed.length,
  };
}

function normalizeRunResult(runResult) {
  const row = asObject(runResult);
  const stdout = asArray(row.stdout).map((item) => String(item ?? ""));
  const stderr = asArray(row.stderr).map((item) => String(item ?? ""));
  const exitCode = Number(row.exit_code ?? row.exitCode ?? 0);
  return {
    exit_code: Number.isFinite(exitCode) ? Math.trunc(exitCode) : 0,
    stdout,
    stderr,
    stdout_count: stdout.length,
    stderr_count: stderr.length,
  };
}

function resolveRecordByAssignmentId(records, assignmentId) {
  if (!records) return null;
  if (Array.isArray(records)) {
    return records.find((item) => {
      const row = asObject(item);
      return String(row.assignment_id ?? row.id ?? row["과제ID"] ?? "") === assignmentId;
    }) ?? null;
  }
  const map = asObject(records);
  return map[assignmentId] ?? null;
}

function summarizeRunStatus(runResult, suiteView) {
  if (suiteView && suiteView.judgement === "실패") return "실패";
  if (suiteView && suiteView.judgement === "통과") return "통과";
  if (!runResult) return "미실행";
  return runResult.exit_code === 0 ? "통과" : "실패";
}

export function buildClassroomAssignmentList(assignments = []) {
  const rows = asArray(assignments).map((item, index) => normalizeAssignment(item, index));
  const openCount = rows.filter((item) => item.is_open).length;
  return {
    __종류: "studio_classroom_assignment_list",
    assignment_count: rows.length,
    open_count: openCount,
    closed_count: rows.length - openCount,
    account_required: false,
    cloud_sync: false,
    assignments: rows,
  };
}

export function buildClassroomSuiteCheckView(suiteCheck = {}) {
  return normalizeSuiteCheck(suiteCheck);
}

export function buildClassroomRunResultSummary({
  assignment = {},
  runResult = null,
  suiteCheck = null,
} = {}) {
  const normalizedAssignment = normalizeAssignment(assignment, 0);
  const normalizedRun = runResult ? normalizeRunResult(runResult) : null;
  const suiteView = suiteCheck ? normalizeSuiteCheck(suiteCheck) : null;
  const runStatus = summarizeRunStatus(normalizedRun, suiteView);
  const failedCases = suiteView ? suiteView.failed_cases : [];
  return {
    __종류: "studio_classroom_run_result_summary",
    assignment_id: normalizedAssignment.assignment_id,
    title: normalizedAssignment.title,
    lesson_id: normalizedAssignment.lesson_id,
    run_status: runStatus,
    stdout_count: normalizedRun ? normalizedRun.stdout_count : 0,
    stderr_count: normalizedRun ? normalizedRun.stderr_count : 0,
    suite_judgement: suiteView ? suiteView.judgement : "미실행",
    suite_check_kind: suiteView ? suiteView.suite_check_kind : "",
    failed_case_count: failedCases.length,
    failure_case_names: failedCases,
    mismatch_case_count: suiteView ? suiteView.mismatch_count : 0,
    expected_fail_passed_cases: suiteView ? suiteView.expected_fail_passed_cases : [],
    expected_pass_failed_cases: suiteView ? suiteView.expected_pass_failed_cases : [],
  };
}

export function buildClassroomExportReport({
  assignmentList = null,
  resultSummaries = [],
} = {}) {
  const list = assignmentList && assignmentList.__종류 === "studio_classroom_assignment_list"
    ? assignmentList
    : buildClassroomAssignmentList([]);
  const summaries = asArray(resultSummaries).map((item) => asObject(item));
  const failed = summaries.filter((item) => String(item.run_status ?? "") === "실패");
  return {
    __종류: "studio_classroom_export_report",
    generated_locally: true,
    account_required: false,
    cloud_sync: false,
    assignment_count: list.assignment_count,
    summary_count: summaries.length,
    pass_count: summaries.length - failed.length,
    fail_count: failed.length,
    assignments: list.assignments,
    summaries,
  };
}

export function formatClassroomExportReportText(report = {}) {
  const payload = asObject(report);
  if (payload.__종류 !== "studio_classroom_export_report") {
    throw new Error("studio_classroom_expected_export_report");
  }
  const lines = [
    "수업 코드\t수업 제목\t수업 목표\t오늘 활동\t결과 확인\t배포 상태\t실행 결과\t확인 필요\t비고",
  ];
  const summariesById = new Map(asArray(payload.summaries).map((summary) => [String(summary.assignment_id ?? ""), summary]));
  asArray(payload.assignments).forEach((assignment) => {
    const id = String(assignment.assignment_id ?? "");
    const summary = summariesById.get(id) || {};
    const failed = normalizeNameList(summary.failure_case_names).join("|");
    const mismatches = [
      ...normalizeNameList(summary.expected_fail_passed_cases),
      ...normalizeNameList(summary.expected_pass_failed_cases),
    ].join("|");
    lines.push([
      id,
      String(assignment.title ?? ""),
      normalizeNameList(assignment.goals).join("|"),
      normalizeNameList(assignment.missions).join("|"),
      asText(assignment.result_views_label, normalizeNameList(assignment.result_views).join(", ")),
      assignment.is_open ? "열림" : "닫힘",
      String(summary.run_status ?? "미실행"),
      failed,
      mismatches,
    ].join("\t"));
  });
  return lines.join("\n");
}

export function buildClassroomReportWorkflow({
  assignments = [],
  runResults = {},
  suiteChecks = {},
} = {}) {
  const assignmentList = buildClassroomAssignmentList(assignments);
  const suiteCheckViews = [];
  const resultSummaries = assignmentList.assignments.map((assignment) => {
    const assignmentId = String(assignment.assignment_id ?? "");
    const suiteCheck = resolveRecordByAssignmentId(suiteChecks, assignmentId);
    const suiteView = suiteCheck ? buildClassroomSuiteCheckView(suiteCheck) : null;
    if (suiteView) {
      suiteCheckViews.push({
        assignment_id: assignmentId,
        ...suiteView,
      });
    }
    return buildClassroomRunResultSummary({
      assignment,
      runResult: resolveRecordByAssignmentId(runResults, assignmentId),
      suiteCheck,
    });
  });
  const exportReport = buildClassroomExportReport({
    assignmentList,
    resultSummaries,
  });
  const exportText = formatClassroomExportReportText(exportReport);
  const failed = resultSummaries.filter((summary) => summary.run_status === "실패");
  const mismatches = resultSummaries.reduce((total, summary) => total + asCount(summary.mismatch_case_count), 0);
  const stages = [
    ["assignment_list", assignmentList.__종류 === "studio_classroom_assignment_list" && assignmentList.assignment_count > 0],
    ["run_result_summaries", resultSummaries.length === assignmentList.assignment_count],
    ["suite_check_views", suiteCheckViews.length > 0],
    ["export_report", exportReport.__종류 === "studio_classroom_export_report"],
    ["export_report_text", exportText.length > 0 && !exportText.endsWith("\n")],
    ["local_only_boundary", exportReport.account_required === false && exportReport.cloud_sync === false],
  ].map(([stageId, ready]) => ({
    stage_id: stageId,
    ready: ready === true,
  }));
  const readyStageCount = stages.filter((stage) => stage.ready).length;
  const ready = readyStageCount === stages.length;
  return {
    __종류: "studio_classroom_report_workflow",
    schema: "seamgrim.classroom_report_workflow.v1",
    primary_coordinate: "마-3",
    support_coordinate: "하-3",
    workflow_claim: "classroom_report_workflow",
    generated_locally: true,
    account_required: false,
    cloud_sync: false,
    permission_system: false,
    replay_claim: false,
    status: ready ? "classroom_report_ready" : "classroom_report_incomplete",
    tone: ready ? "success" : "warning",
    stage_count: stages.length,
    ready_stage_count: readyStageCount,
    missing_stage_count: stages.length - readyStageCount,
    assignment_count: assignmentList.assignment_count,
    open_count: assignmentList.open_count,
    closed_count: assignmentList.closed_count,
    summary_count: resultSummaries.length,
    pass_count: exportReport.pass_count,
    fail_count: failed.length,
    suite_check_count: suiteCheckViews.length,
    mismatch_case_count: mismatches,
    export_text_line_count: exportText ? exportText.split("\n").length : 0,
    stages,
    assignment_list: assignmentList,
    suite_check_views: suiteCheckViews,
    result_summaries: resultSummaries,
    export_report: exportReport,
    export_text: exportText,
  };
}

export function formatClassroomReportWorkflowText(workflow = {}) {
  const payload = asObject(workflow);
  if (payload.__종류 !== "studio_classroom_report_workflow") {
    throw new Error("studio_classroom_expected_report_workflow");
  }
  const stages = asArray(payload.stages);
  const stageLines = stages.map((stage) => [
    String(stage.stage_id ?? ""),
    stage.ready === true ? "true" : "false",
  ].join("\t"));
  return [
    `schema\t${payload.schema ?? ""}`,
    `primary_coordinate\t${payload.primary_coordinate ?? ""}`,
    `support_coordinate\t${payload.support_coordinate ?? ""}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `generated_locally\t${payload.generated_locally === true ? "true" : "false"}`,
    `account_required\t${payload.account_required === true ? "true" : "false"}`,
    `cloud_sync\t${payload.cloud_sync === true ? "true" : "false"}`,
    `permission_system\t${payload.permission_system === true ? "true" : "false"}`,
    `replay_claim\t${payload.replay_claim === true ? "true" : "false"}`,
    `status\t${payload.status ?? ""}`,
    `stage_count\t${payload.stage_count ?? stages.length}`,
    `ready_stage_count\t${payload.ready_stage_count ?? 0}`,
    `missing_stage_count\t${payload.missing_stage_count ?? 0}`,
    `assignment_count\t${payload.assignment_count ?? 0}`,
    `summary_count\t${payload.summary_count ?? 0}`,
    `pass_count\t${payload.pass_count ?? 0}`,
    `fail_count\t${payload.fail_count ?? 0}`,
    `suite_check_count\t${payload.suite_check_count ?? 0}`,
    `mismatch_case_count\t${payload.mismatch_case_count ?? 0}`,
    `export_text_line_count\t${payload.export_text_line_count ?? 0}`,
    "",
    "stage_id\tready",
    ...stageLines,
    "",
    "export_text",
    String(payload.export_text ?? ""),
  ].join("\n");
}
