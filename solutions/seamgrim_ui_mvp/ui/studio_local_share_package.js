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

function asTextArray(value) {
  return asArray(value)
    .map((item) => String(item ?? "").trim())
    .filter(Boolean);
}

function byteLengthUtf8(value) {
  return new TextEncoder().encode(String(value ?? "")).length;
}

function stableStringify(value) {
  if (Array.isArray(value)) {
    return `[${value.map((item) => stableStringify(item)).join(",")}]`;
  }
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function fnv1a32(value) {
  const source = String(value ?? "");
  let hash = 0x811c9dc5;
  for (let index = 0; index < source.length; index += 1) {
    hash ^= source.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function candidateHash(label, value) {
  return `candidate:fnv1a32:${label}:${fnv1a32(stableStringify(value))}`;
}

const CLASSROOM_EXCHANGE_SCHEMA_V0 = "seamgrim.classroom_exchange.v0";
const MAX_CLASSROOM_EXCHANGE_PAYLOAD_BYTES = 2_000_000;
const MAX_STUDENT_SUBMISSION_BYTES = 1_000_000;

function normalizePath(value, fallback) {
  const text = asText(value, fallback).replace(/\\/g, "/").replace(/^\/+/, "");
  return text || fallback;
}

function safePackagePathToken(value, fallback) {
  const token = String(value ?? "")
    .trim()
    .replace(/[^a-zA-Z0-9._-]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return token || fallback;
}

function normalizeLesson(lesson, index) {
  const row = asObject(lesson);
  const lessonId = asText(row.lesson_id ?? row.id ?? row["수업"], `lesson_${String(index + 1).padStart(3, "0")}`);
  const pathToken = safePackagePathToken(lessonId, `lesson_${String(index + 1).padStart(3, "0")}`);
  const source = String(row.source_text ?? row.source ?? row.ddn ?? row["본문"] ?? "");
  return {
    lesson_id: lessonId,
    title: asText(row.title ?? row.name ?? row["제목"], lessonId),
    description: asText(row.description ?? row.desc ?? row["설명"], ""),
    grade: asText(row.grade ?? row["학년"], ""),
    subject: asText(row.subject ?? row["교과"], ""),
    required_views: asTextArray(row.required_views ?? row.requiredViews ?? row["보기"]),
    goals: asTextArray(row.goals ?? row.learning_goals ?? row["학습목표"]),
    missions: asTextArray(row.missions ?? row["수업활동"]),
    curriculum_card: asObject(row.curriculum_card ?? row.curriculumCard),
    path: normalizePath(row.path, `lessons/${pathToken}.ddn`),
    mime: "text/plain; charset=utf-8",
    source_text: source,
    byte_size: byteLengthUtf8(source),
  };
}

function normalizeAllowedControl(control, index) {
  const row = asObject(control);
  const name = asText(row.name ?? row.control ?? row.id, index === 0 ? "theta0" : `control_${index + 1}`);
  const min = Number(row.min ?? row.minimum ?? 0);
  const max = Number(row.max ?? row.maximum ?? 1);
  const step = Number(row.step ?? 0.1);
  return {
    name,
    label: asText(row.label ?? row.title, name),
    min: Number.isFinite(min) ? min : 0,
    max: Number.isFinite(max) ? max : 1,
    step: Number.isFinite(step) && step > 0 ? step : 0.1,
  };
}

function normalizeViewName(value) {
  const text = String(value ?? "").trim().toLowerCase();
  if (["space2d", "graph", "table", "text"].includes(text)) return text;
  return "";
}

export function buildDeploymentProfileV0(profile = {}) {
  const row = asObject(profile);
  const allowedViews = asTextArray(row.allowed_views ?? row.allowedViews)
    .map(normalizeViewName)
    .filter(Boolean);
  const uniqueViews = Array.from(new Set(allowedViews.length ? allowedViews : ["space2d", "graph", "table", "text"]));
  const requestedDefaultView = normalizeViewName(row.default_view ?? row.defaultView);
  const defaultView = uniqueViews.includes(requestedDefaultView) ? requestedDefaultView : uniqueViews[0];
  const allowedControls = asArray(row.allowed_controls ?? row.allowedControls)
    .map((item, index) => normalizeAllowedControl(item, index))
    .filter((item) => item.name);
  const normalized = {
    schema: "seamgrim.deployment_profile.v0",
    profile_version: "v0.0",
    preset_ref: asText(row.preset_ref ?? row.presetRef, "guided_student_default"),
    allowed_controls: allowedControls.length ? allowedControls : [normalizeAllowedControl({ name: "theta0", label: "초기 각도", min: -1.2, max: 1.2, step: 0.05 }, 0)],
    answer_display_policy: asText(row.answer_display_policy ?? row.answerDisplayPolicy, "after_run"),
    explanation_display_policy: asText(row.explanation_display_policy ?? row.explanationDisplayPolicy, "always"),
    default_view: defaultView,
    allowed_views: uniqueViews,
    run_policy: {
      allow_run: row.run_policy?.allow_run !== false && row.runPolicy?.allowRun !== false,
      allow_restart: row.run_policy?.allow_restart !== false && row.runPolicy?.allowRestart !== false,
    },
    readonly_in_run: true,
    state_hash_participation: false,
    signature_claim: false,
  };
  return {
    ...normalized,
    profile_hash: candidateHash("deployment_profile", normalized),
  };
}

function profileWithoutHash(profile = {}) {
  const out = { ...asObject(profile) };
  delete out.profile_hash;
  return out;
}

function assertDeploymentProfileHash(profile = {}) {
  const row = asObject(profile);
  if (!Object.keys(row).length) return null;
  if (row.schema !== "seamgrim.deployment_profile.v0") {
    throw new Error("studio_local_package_deployment_profile_schema_mismatch");
  }
  const expected = buildDeploymentProfileV0(profileWithoutHash(row));
  if (row.profile_hash !== expected.profile_hash) {
    throw new Error("studio_local_package_deployment_profile_hash_mismatch");
  }
  return row;
}

function buildPackageHash({ manifest = {}, lessons = [], reports = [], assets = [] } = {}) {
  const manifestInput = { ...asObject(manifest) };
  delete manifestInput.package_hash;
  return candidateHash("studio_local_package", {
    manifest: manifestInput,
    lessons: asArray(lessons),
    reports: asArray(reports),
    assets: asArray(assets),
  });
}

function buildClassroomExchangePayloadHashInput({
  exchange = {},
  lessons = [],
  reports = [],
  assets = [],
  deploymentProfile = null,
} = {}) {
  const row = { ...asObject(exchange) };
  delete row.payload_hash;
  return {
    exchange: row,
    deployment_profile: deploymentProfile ?? null,
    lessons: asArray(lessons),
    reports: asArray(reports),
    assets: asArray(assets),
  };
}

function buildClassroomExchangePayloadHash(input = {}) {
  return candidateHash("classroom_exchange_payload", buildClassroomExchangePayloadHashInput(input));
}

function buildClassroomExchangeOriginHash(lessons = []) {
  const firstLesson = asObject(asArray(lessons)[0]);
  return candidateHash("classroom_exchange_origin", {
    lesson_id: asText(firstLesson.lesson_id ?? firstLesson.id, "lesson"),
    source_text: String(firstLesson.source_text ?? ""),
  });
}

export function buildClassroomExchangeV0({
  kind = "lesson_bundle",
  manifest = {},
  lessons = [],
  reports = [],
  assets = [],
  projectSnapshot = null,
  curriculumArtifacts = null,
  runEvidence = null,
  lineage = null,
  localLabel = "",
  createdAtOrSequence = "local-sequence-0001",
} = {}) {
  const normalizedLessons = asArray(lessons);
  const normalizedReports = asArray(reports);
  const normalizedAssets = asArray(assets);
  const firstLesson = asObject(normalizedLessons[0]);
  const manifestRow = asObject(manifest);
  const deploymentProfile = asObject(manifestRow.deployment_profile);
  const lessonId = asText(firstLesson.lesson_id ?? firstLesson.id, "lesson");
  const packageId = asText(manifestRow.package_id, "local_classroom_exchange");
  const exchange = {
    schema: CLASSROOM_EXCHANGE_SCHEMA_V0,
    kind: kind === "student_submission" ? "student_submission" : "lesson_bundle",
    exchange_id: asText(manifestRow.package_id, `${packageId}.${lessonId}`),
    lesson_id: lessonId,
    origin_hash: buildClassroomExchangeOriginHash(normalizedLessons),
    deployment_profile_ref: asText(deploymentProfile.preset_ref, "guided_student_default"),
    deployment_profile_hash: asText(manifestRow.deployment_profile_hash ?? deploymentProfile.profile_hash, ""),
    project_snapshot: asObject(projectSnapshot ?? {
      package_id: packageId,
      title: asText(manifestRow.title, lessonId),
      lesson_count: normalizedLessons.length,
    }),
    curriculum_artifacts: asObject(curriculumArtifacts ?? {
      lesson_plan_report: normalizedReports.some((report) => asText(report.report_id, "") === "lesson_plan_v0"),
      view_refs: asTextArray(firstLesson.required_views),
    }),
    run_evidence: asArray(runEvidence ?? []),
    lineage: asObject(lineage ?? {
      source: "local_teacher_export",
      parent_exchange_id: "",
    }),
    local_label: asText(localLabel, asText(manifestRow.title, lessonId)),
    created_at_or_sequence: asText(createdAtOrSequence, "local-sequence-0001"),
    boundaries: {
      account_identity_claim: false,
      cloud_sync: false,
      public_registry: false,
      cryptographic_signature_claim: false,
      ai_grading_claim: false,
    },
  };
  return {
    ...exchange,
    payload_hash: buildClassroomExchangePayloadHash({
      exchange,
      lessons: normalizedLessons,
      reports: normalizedReports,
      assets: normalizedAssets,
      deploymentProfile,
    }),
  };
}

function assertClassroomExchangePayloadSize(payload = {}) {
  if (byteLengthUtf8(JSON.stringify(payload ?? {})) > MAX_CLASSROOM_EXCHANGE_PAYLOAD_BYTES) {
    throw new Error("studio_local_package_classroom_exchange_too_large");
  }
}

function assertClassroomExchangeV0({
  exchange = {},
  manifest = {},
  lessons = [],
  reports = [],
  assets = [],
} = {}) {
  const row = asObject(exchange);
  if (!Object.keys(row).length) {
    throw new Error("studio_local_package_classroom_exchange_missing");
  }
  if (row.schema !== CLASSROOM_EXCHANGE_SCHEMA_V0) {
    throw new Error("studio_local_package_classroom_exchange_schema_mismatch");
  }
  if (row.kind !== "lesson_bundle") {
    throw new Error("studio_local_package_classroom_exchange_kind_mismatch");
  }
  const expected = buildClassroomExchangeV0({
    kind: row.kind,
    manifest,
    lessons,
    reports,
    assets,
    projectSnapshot: row.project_snapshot,
    curriculumArtifacts: row.curriculum_artifacts,
    runEvidence: row.run_evidence,
    lineage: row.lineage,
    localLabel: row.local_label,
    createdAtOrSequence: row.created_at_or_sequence,
  });
  if (row.payload_hash !== expected.payload_hash) {
    throw new Error("studio_local_package_classroom_exchange_hash_mismatch");
  }
  return row;
}

function buildStudentSubmissionHashInput(exchange = {}) {
  const row = { ...asObject(exchange) };
  delete row.payload_hash;
  return row;
}

export function buildStudentSubmissionExchangeV0({
  lessonId = "",
  originHash = "",
  originPayloadHash = "",
  deploymentProfileHash = "",
  predictionText = "",
  explanationText = "",
  runEvidence = {},
  controlSnapshot = {},
  observationRefs = [],
  studentProjectSnapshot = {},
  lineage = {},
  localLabel = "",
  createdAtOrSequence = "local-student-sequence-0001",
} = {}) {
  const normalizedLessonId = asText(lessonId, "lesson");
  const projectSnapshot = asObject(studentProjectSnapshot);
  const response = {
    prediction_text: String(predictionText ?? ""),
    explanation_text: String(explanationText ?? ""),
  };
  const evidence = asObject(runEvidence);
  const controls = asObject(controlSnapshot);
  const exchange = {
    schema: CLASSROOM_EXCHANGE_SCHEMA_V0,
    kind: "student_submission",
    exchange_id: asText(projectSnapshot.exchange_id, `student_submission.${normalizedLessonId}.${fnv1a32(stableStringify({ response, evidence, controls })).slice(0, 8)}`),
    lesson_id: normalizedLessonId,
    origin_hash: asText(originHash, ""),
    origin_payload_hash: asText(originPayloadHash, ""),
    deployment_profile_ref: asText(projectSnapshot.deployment_profile_ref, ""),
    deployment_profile_hash: asText(deploymentProfileHash, ""),
    response,
    run_evidence: evidence,
    control_snapshot: controls,
    observation_refs: asArray(observationRefs).map((item) => String(item ?? "").trim()).filter(Boolean),
    student_project_snapshot: projectSnapshot,
    curriculum_artifacts: asObject(projectSnapshot.curriculum_artifacts),
    lineage: {
      source: "student_submission",
      origin_exchange_hash: asText(originPayloadHash, ""),
      ...asObject(lineage),
    },
    local_label: asText(localLabel, ""),
    created_at_or_sequence: asText(createdAtOrSequence, "local-student-sequence-0001"),
    evidence_binding_hash: candidateHash("student_submission_evidence_binding", {
      response,
      run_evidence: evidence,
      control_snapshot: controls,
    }),
    boundaries: {
      account_identity_claim: false,
      cloud_sync: false,
      public_registry: false,
      cryptographic_signature_claim: false,
      ai_grading_claim: false,
    },
  };
  return {
    ...exchange,
    payload_hash: candidateHash("student_submission_payload", buildStudentSubmissionHashInput(exchange)),
  };
}

export function validateStudentSubmissionExchangeV0(payload = {}) {
  try {
    const row = asObject(payload);
    if (byteLengthUtf8(JSON.stringify(row)) > MAX_STUDENT_SUBMISSION_BYTES) {
      throw new Error("student_submission_exchange_too_large");
    }
    if (row.schema !== CLASSROOM_EXCHANGE_SCHEMA_V0) {
      throw new Error("student_submission_exchange_schema_mismatch");
    }
    if (row.kind !== "student_submission") {
      throw new Error("student_submission_exchange_kind_mismatch");
    }
    const expected = buildStudentSubmissionExchangeV0({
      lessonId: row.lesson_id,
      originHash: row.origin_hash,
      originPayloadHash: row.origin_payload_hash,
      deploymentProfileHash: row.deployment_profile_hash,
      predictionText: row.response?.prediction_text,
      explanationText: row.response?.explanation_text,
      runEvidence: row.run_evidence,
      controlSnapshot: row.control_snapshot,
      observationRefs: row.observation_refs,
      studentProjectSnapshot: row.student_project_snapshot,
      lineage: row.lineage,
      localLabel: row.local_label,
      createdAtOrSequence: row.created_at_or_sequence,
    });
    if (row.payload_hash !== expected.payload_hash) {
      throw new Error("student_submission_exchange_hash_mismatch");
    }
    if (!asText(row.run_evidence?.runtime_hash, "")) {
      throw new Error("student_submission_exchange_missing_run_evidence");
    }
    return {
      valid: true,
      error: "",
      payload_hash: row.payload_hash,
      lesson_id: asText(row.lesson_id, ""),
      origin_payload_hash: asText(row.origin_payload_hash, ""),
    };
  } catch (error) {
    return {
      valid: false,
      error: String(error?.message ?? error),
    };
  }
}

export function formatStudentSubmissionExchangeError(error) {
  const code = String(error?.message ?? error ?? "").trim();
  const messages = {
    student_submission_exchange_too_large: "학생 제출이 허용 크기보다 큽니다.",
    student_submission_exchange_schema_mismatch: "학생 제출 형식이 맞지 않습니다.",
    student_submission_exchange_kind_mismatch: "학생 제출 종류가 맞지 않습니다.",
    student_submission_exchange_hash_mismatch: "학생 제출 해시가 맞지 않습니다. 제출 파일이 수정되었는지 확인하세요.",
    student_submission_exchange_missing_run_evidence: "학생 제출에 실행 근거가 없습니다.",
  };
  if (messages[code]) return messages[code];
  if (/[가-힣]/.test(code)) return code;
  return "학생 제출을 열 수 없습니다.";
}

function hasOwn(row, key) {
  return Object.prototype.hasOwnProperty.call(row, key);
}

function buildControlDiffRows(controlSnapshot = {}, baseline = {}) {
  const controls = asObject(controlSnapshot);
  const original = asObject(baseline);
  const names = Array.from(new Set([
    ...Object.keys(original),
    ...Object.keys(controls),
  ])).sort();
  return names.map((name) => {
    const hasBaseline = hasOwn(original, name);
    const hasSubmitted = hasOwn(controls, name);
    const originalValue = hasBaseline ? original[name] : null;
    const submittedValue = hasSubmitted ? controls[name] : null;
    return {
      name,
      original_value: originalValue,
      submitted_value: submittedValue,
      changed: stableStringify(originalValue) !== stableStringify(submittedValue),
      baseline_status: hasBaseline ? "known" : "missing_baseline",
    };
  });
}

export function buildStudentSubmissionReviewRecord(payload = {}, { existingPayloadHashes = [] } = {}) {
  const row = asObject(payload);
  const validation = validateStudentSubmissionExchangeV0(row);
  const projectSnapshot = asObject(row.student_project_snapshot);
  const response = asObject(row.response);
  const runEvidence = asObject(row.run_evidence);
  const controlSnapshot = asObject(row.control_snapshot);
  const originBaseline = asObject(projectSnapshot.origin_control_baseline);
  const payloadHash = asText(row.payload_hash, "");
  const duplicate = existingPayloadHashes.map((item) => String(item ?? "").trim()).includes(payloadHash);
  return {
    payload_hash: payloadHash,
    exchange_id: asText(row.exchange_id, ""),
    lesson_id: asText(row.lesson_id ?? projectSnapshot.lesson_id, ""),
    origin_hash: asText(row.origin_hash, ""),
    origin_payload_hash: asText(row.origin_payload_hash, ""),
    deployment_profile_hash: asText(row.deployment_profile_hash, ""),
    local_label: asText(row.local_label, ""),
    prediction_present: asText(response.prediction_text, "") !== "",
    explanation_present: asText(response.explanation_text, "") !== "",
    prediction_length: String(response.prediction_text ?? "").length,
    explanation_length: String(response.explanation_text ?? "").length,
    run_evidence: {
      runtime_hash: asText(runEvidence.runtime_hash, ""),
      engine_status: asText(runEvidence.engine_status, ""),
    },
    observation_refs: asTextArray(row.observation_refs),
    changed_controls: buildControlDiffRows(controlSnapshot, originBaseline),
    integrity_status: validation.valid ? "valid" : "invalid",
    integrity_error: validation.valid ? "" : validation.error,
    duplicate_status: duplicate ? "duplicate" : "unique",
    version_status: row.schema === CLASSROOM_EXCHANGE_SCHEMA_V0 && row.kind === "student_submission"
      ? "classroom_exchange.v0.student_submission"
      : "unsupported",
    reopen_available: asText(projectSnapshot.ddn_text, "") !== "",
    auto_score_claim: false,
    ai_grading_claim: false,
  };
}

export function buildStudentSubmissionArchiveExport({ rows = [] } = {}) {
  const records = asArray(rows).map((row, index) => {
    const record = asObject(row.record ?? row);
    return {
      index: index + 1,
      local_label: asText(record.local_label, ""),
      lesson_id: asText(record.lesson_id, ""),
      origin_payload_hash: asText(record.origin_payload_hash, ""),
      payload_hash: asText(record.payload_hash, ""),
      prediction_present: record.prediction_present === true,
      explanation_present: record.explanation_present === true,
      run_evidence_hash: asText(record.run_evidence?.runtime_hash, ""),
      observation_ref_count: asArray(record.observation_refs).length,
      changed_controls: asArray(record.changed_controls),
      integrity_status: asText(record.integrity_status, ""),
      duplicate_status: asText(record.duplicate_status, ""),
      version_status: asText(record.version_status, ""),
    };
  });
  const archive = {
    generated_by: "SEAMGRIM_WAVE3B_STUDENT_SUBMISSION_ARCHIVE_V1",
    report_kind: "student_submission_archive",
    lesson_id: asText(records[0]?.lesson_id, ""),
    origin_payload_hash: asText(records[0]?.origin_payload_hash, ""),
    review_count: records.length,
    auto_score_claim: false,
    rows: records,
  };
  return {
    ...archive,
    export_hash: candidateHash("student_submission_archive_export", archive),
  };
}

function assertClassroomExchangePayloadMirror(packagePayload = {}, classroomExchange = {}) {
  const row = asObject(packagePayload);
  const exchange = asObject(classroomExchange);
  const schema = asText(row.schema, "");
  if (schema && schema !== CLASSROOM_EXCHANGE_SCHEMA_V0) {
    throw new Error("studio_local_package_classroom_exchange_schema_mismatch");
  }
  const kind = asText(row.kind, "");
  if (kind && kind !== "lesson_bundle") {
    throw new Error("studio_local_package_classroom_exchange_kind_mismatch");
  }
  const payloadHash = asText(row.payload_hash, "");
  if (payloadHash && payloadHash !== asText(exchange.payload_hash, "")) {
    throw new Error("studio_local_package_classroom_exchange_hash_mismatch");
  }
  const exchangeId = asText(row.exchange_id, "");
  if (exchangeId && exchangeId !== asText(exchange.exchange_id, "")) {
    throw new Error("studio_local_package_classroom_exchange_hash_mismatch");
  }
  const profileHash = asText(row.deployment_profile_hash, "");
  if (profileHash && profileHash !== asText(exchange.deployment_profile_hash, "")) {
    throw new Error("studio_local_package_deployment_profile_hash_mismatch");
  }
}

function formatStudentRequiredViewLabel(value) {
  const key = String(value ?? "").trim().toLowerCase();
  if (key === "graph") return "그래프";
  if (key === "table") return "표";
  if (key === "space2d") return "그림";
  if (key === "text") return "설명";
  return String(value ?? "").trim();
}

function buildStudioLocalPackageStudentInstructions(lessons = [], { sessionLabel = "" } = {}) {
  const lessonRows = asArray(lessons);
  const firstLesson = lessonRows[0] ?? {};
  const hasMultipleLessons = lessonRows.length > 1;
  const lessonId = asText(firstLesson.lesson_id ?? firstLesson.id, "");
  const lessonTitle = asText(firstLesson.title, "");
  const session = asText(sessionLabel, "");
  const goalRows = asTextArray(firstLesson.goals ?? firstLesson.learning_goals);
  const missionRows = asTextArray(firstLesson.missions);
  const viewLabels = asTextArray(firstLesson.required_views ?? firstLesson.requiredViews)
    .map(formatStudentRequiredViewLabel)
    .filter(Boolean)
    .slice(0, 4);
  const resultInstruction = viewLabels.length
    ? `결과 확인: ${viewLabels.join(", ")}를 확인합니다.`
    : "결과 확인 화면을 확인합니다.";
  return [
    ...(session ? [`차시: ${session}`] : []),
    ...(hasMultipleLessons ? [`교과 수: ${lessonRows.length}`] : []),
    ...(lessonTitle ? [`${hasMultipleLessons ? "첫 수업" : "수업"}: ${lessonTitle}`] : []),
    ...(lessonId ? [`${hasMultipleLessons ? "첫 수업 코드" : "수업 코드"}: ${lessonId}`] : []),
    ...(goalRows.length ? [`목표: ${goalRows[0]}`] : []),
    ...(missionRows.length ? [`오늘 활동: ${missionRows[0]}`] : []),
    "셈그림 Studio에서 배포 열기 또는 붙여넣기 열기로 선생님이 보낸 JSON 배포 파일이나 JSON 텍스트를 엽니다.",
    ...(hasMultipleLessons ? ["배포 수업 선택에서 오늘 수업을 고릅니다."] : []),
    "받은 수업 실행을 눌러 교사가 보낸 수업을 시작합니다.",
    resultInstruction,
    "결과 확인 뒤 이름을 입력하고 결과 복사 또는 결과 저장으로 교사에게 제출합니다.",
  ];
}

function buildStudioLocalPackageMaterialsSummary({ lessons = [], reports = [], assets = [] } = {}) {
  const lessonTitles = asArray(lessons)
    .map((item) => asText(item.title ?? item.lesson_id, ""))
    .filter(Boolean);
  const reportTitles = asArray(reports)
    .map((item) => asText(item.title ?? item.report_id, ""))
    .filter(Boolean);
  const assetTitles = asArray(assets)
    .map((item) => asText(item.path ?? item.role, ""))
    .filter(Boolean);
  return [
    lessonTitles.length
      ? `교과 ${lessonTitles.length}개: ${lessonTitles.join(", ")}`
      : "교과 없음",
    reportTitles.length
      ? `교사용 자료 ${reportTitles.length}개: ${reportTitles.join(", ")}`
      : "교사용 자료 없음",
    ...(assetTitles.length ? [`추가 자료 ${assetTitles.length}개: ${assetTitles.join(", ")}`] : []),
  ];
}

function buildStudioLocalPackageStudentMaterialsSummary({ lessons = [], reports = [] } = {}) {
  const lessonTitles = asArray(lessons)
    .map((item) => asText(item.title ?? item.lesson_id, ""))
    .filter(Boolean);
  const studentReportTitles = asArray(reports)
    .map((item) => asText(item.title ?? item.report_id, ""))
    .filter((item) => item && item.includes("학생"));
  return [
    lessonTitles.length
      ? `교과 ${lessonTitles.length}개: ${lessonTitles.join(", ")}`
      : "교과 없음",
    ...(studentReportTitles.length
      ? [`학생 자료 ${studentReportTitles.length}개: ${studentReportTitles.join(", ")}`]
      : []),
  ];
}

function normalizeReport(report, index) {
  const row = asObject(report);
  const reportId = asText(row.report_id ?? row.id ?? row["보고서"], `report_${String(index + 1).padStart(3, "0")}`);
  const pathToken = safePackagePathToken(reportId, `report_${String(index + 1).padStart(3, "0")}`);
  const text = String(row.text ?? row.report_text ?? row["본문"] ?? "");
  return {
    report_id: reportId,
    title: asText(row.title ?? row.name ?? row["제목"], reportId),
    path: normalizePath(row.path, `reports/${pathToken}.txt`),
    mime: asText(row.mime, "text/plain; charset=utf-8"),
    text,
    byte_size: byteLengthUtf8(text),
  };
}

function normalizeAsset(asset, index) {
  const row = asObject(asset);
  const path = normalizePath(row.path, `assets/asset_${String(index + 1).padStart(3, "0")}`);
  return {
    path,
    role: asText(row.role ?? row.kind, "asset"),
    mime: asText(row.mime, "application/octet-stream"),
    byte_size: Math.max(0, Math.trunc(Number(row.byte_size ?? row.size ?? 0) || 0)),
  };
}

function fileEntry(path, type, byteSize, mime) {
  return {
    path,
    type,
    byte_size: Math.max(0, Math.trunc(Number(byteSize) || 0)),
    mime,
  };
}

function declaredCount(value) {
  const text = String(value ?? "").trim();
  if (!/^(0|[1-9]\d*)$/.test(text)) return null;
  const number = Number(text);
  return Number.isSafeInteger(number) ? number : null;
}

function assertDeclaredCount(name, declared, actual) {
  const expected = declaredCount(declared);
  if (expected === null || expected !== actual) {
    throw new Error(`studio_local_package_${name}_count_mismatch`);
  }
}

function assertNoDuplicatePaths(paths) {
  const seen = new Set();
  for (const path of paths) {
    const normalized = normalizePath(path, "");
    if (!normalized) continue;
    if (seen.has(normalized)) {
      throw new Error("studio_local_package_duplicate_path");
    }
    seen.add(normalized);
  }
}

function assertContentPathsDoNotOverlapStatic(staticPaths, contentPaths) {
  const staticPathSet = new Set(staticPaths.map((item) => normalizePath(item, "")).filter(Boolean));
  for (const path of contentPaths) {
    const normalized = normalizePath(path, "");
    if (normalized && staticPathSet.has(normalized)) {
      throw new Error("studio_local_package_duplicate_path");
    }
  }
}

function assertSafePackagePaths(paths) {
  for (const path of paths) {
    const raw = asText(path, "").replace(/\\/g, "/");
    if (raw.startsWith("/") || /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(raw)) {
      throw new Error("studio_local_package_unsafe_path");
    }
    const normalized = normalizePath(path, "");
    if (!normalized) {
      throw new Error("studio_local_package_unsafe_path");
    }
    const parts = normalized.split("/");
    if (parts.some((part) => !part || part === "." || part === "..")) {
      throw new Error("studio_local_package_unsafe_path");
    }
    if (/^[a-zA-Z]:/.test(normalized)) {
      throw new Error("studio_local_package_unsafe_path");
    }
  }
}

function assertSafeRawRowPaths(rows) {
  for (const row of rows) {
    const value = asObject(row).path;
    if (value !== undefined && value !== null) {
      assertSafePackagePaths([value]);
    }
  }
}

function assertRawAssetByteSizes(rows) {
  for (const row of rows) {
    const asset = asObject(row);
    const value = asset.byte_size ?? asset.size;
    if (value === undefined || value === null) continue;
    if (declaredCount(value) === null) {
      throw new Error("studio_local_package_asset_byte_size_mismatch");
    }
  }
}

function assertManifestContainsPaths(manifestFiles, paths, errorCode) {
  const available = new Set(manifestFiles.map((item) => normalizePath(item.path, "")));
  const missing = paths.map((item) => normalizePath(item, "")).filter((item) => item && !available.has(item));
  if (missing.length > 0) {
    throw new Error(errorCode);
  }
}

function assertManifestByteSizes(manifestFiles, rows, errorCode) {
  const filesByPath = new Map(manifestFiles.map((item) => [normalizePath(item.path, ""), item]));
  for (const row of rows) {
    const path = normalizePath(row.path, "");
    if (!path) continue;
    const entry = filesByPath.get(path);
    const declared = declaredCount(entry?.byte_size);
    if (declared === null || declared !== Math.max(0, Math.trunc(Number(row.byte_size) || 0))) {
      throw new Error(errorCode);
    }
  }
}

function assertManifestTypes(manifestFiles, rows, expectedType, errorCode) {
  const filesByPath = new Map(manifestFiles.map((item) => [normalizePath(item.path, ""), item]));
  for (const row of rows) {
    const path = normalizePath(row.path, "");
    if (!path) continue;
    const entry = filesByPath.get(path);
    if (asText(entry?.type, "") !== expectedType(row)) {
      throw new Error(errorCode);
    }
  }
}

export function formatStudioLocalPackageError(error) {
  const code = String(error?.message ?? error ?? "").trim();
  const messages = {
    studio_local_package_expected_payload: "셈그림 배포 묶음 파일이 아닙니다.",
    studio_local_package_expected_manifest: "배포 묶음 manifest가 없습니다.",
    studio_local_package_invalid_json: "배포 묶음 JSON을 읽을 수 없습니다.",
    studio_local_package_format_mismatch: "배포 묶음 형식 버전이 맞지 않습니다.",
    studio_local_package_remote_boundary_mismatch: "배포 묶음에 계정/클라우드/공개 저장소 의존성이 포함되어 있습니다.",
    studio_local_package_missing_lesson: "배포 묶음에 실행할 교과가 없습니다.",
    studio_local_package_empty_lesson_source: "배포 묶음의 교과 원문이 비어 있습니다.",
    studio_local_package_lesson_count_mismatch: "배포 묶음의 교과 개수가 manifest와 맞지 않습니다.",
    studio_local_package_report_count_mismatch: "배포 묶음의 리포트 개수가 manifest와 맞지 않습니다.",
    studio_local_package_asset_count_mismatch: "배포 묶음의 자료 개수가 manifest와 맞지 않습니다.",
    studio_local_package_file_count_mismatch: "배포 묶음의 파일 목록 개수가 manifest와 맞지 않습니다.",
    studio_local_package_missing_static_bundle: "배포 묶음에 필수 실행 파일 목록이 없습니다.",
    studio_local_package_entry_mismatch: "배포 묶음 시작 파일이 올바르지 않습니다.",
    studio_local_package_manifest_missing_static_file: "배포 묶음 manifest에 필수 실행 파일이 없습니다.",
    studio_local_package_manifest_missing_lesson_file: "배포 묶음 manifest에 교과 파일이 없습니다.",
    studio_local_package_manifest_missing_report_file: "배포 묶음 manifest에 리포트 파일이 없습니다.",
    studio_local_package_manifest_missing_asset_file: "배포 묶음 manifest에 자료 파일이 없습니다.",
    studio_local_package_lesson_byte_size_mismatch: "배포 묶음의 교과 파일 크기가 manifest와 맞지 않습니다.",
    studio_local_package_report_byte_size_mismatch: "배포 묶음의 리포트 파일 크기가 manifest와 맞지 않습니다.",
    studio_local_package_asset_byte_size_mismatch: "배포 묶음의 자료 파일 크기가 manifest와 맞지 않습니다.",
    studio_local_package_lesson_type_mismatch: "배포 묶음 manifest의 교과 파일 유형이 맞지 않습니다.",
    studio_local_package_report_type_mismatch: "배포 묶음 manifest의 리포트 파일 유형이 맞지 않습니다.",
    studio_local_package_asset_type_mismatch: "배포 묶음 manifest의 자료 파일 유형이 맞지 않습니다.",
    studio_local_package_duplicate_path: "배포 묶음에 중복된 파일 경로가 있습니다.",
    studio_local_package_unsafe_path: "배포 묶음에 허용되지 않는 파일 경로가 있습니다.",
    studio_local_package_deployment_profile_schema_mismatch: "배포 프로필 형식이 맞지 않습니다.",
    studio_local_package_deployment_profile_hash_mismatch: "배포 프로필 해시가 맞지 않습니다. 선생님이 만든 원본 배포 파일인지 확인하세요.",
    studio_local_package_hash_mismatch: "배포 묶음 해시가 맞지 않습니다. 배포 파일이 수정되었는지 확인하세요.",
    studio_local_package_classroom_exchange_missing: "수업 교환 봉투가 없습니다.",
    studio_local_package_classroom_exchange_schema_mismatch: "수업 교환 봉투 형식이 맞지 않습니다.",
    studio_local_package_classroom_exchange_kind_mismatch: "수업 교환 봉투 종류가 맞지 않습니다.",
    studio_local_package_classroom_exchange_hash_mismatch: "수업 교환 봉투 해시가 맞지 않습니다. 배포 파일이 수정되었는지 확인하세요.",
    studio_local_package_classroom_exchange_too_large: "배포 묶음이 허용 크기보다 큽니다.",
    studio_local_package_invalid_payload: "배포 묶음을 열 수 없습니다.",
  };
  if (messages[code]) return messages[code];
  if (/[가-힣]/.test(code)) return code;
  return "배포 묶음을 열 수 없습니다.";
}

export function buildStudioLocalPackageManifest({
  packageId = "",
  title = "",
  version = "0.1.0",
  sessionLabel = "",
  lessons = [],
  reports = [],
  assets = [],
  staticFiles = [],
  deploymentProfile = null,
} = {}) {
  const normalizedLessons = asArray(lessons).map((item, index) => normalizeLesson(item, index));
  const normalizedReports = asArray(reports).map((item, index) => normalizeReport(item, index));
  const normalizedAssets = asArray(assets).map((item, index) => normalizeAsset(item, index));
  const normalizedStaticFiles = asArray(staticFiles).map((item) => {
    const row = asObject(item);
    return fileEntry(
      normalizePath(row.path, "index.html"),
      "static",
      row.byte_size ?? row.size ?? 0,
      asText(row.mime, "application/octet-stream"),
    );
  });
  const requiredStatic = [
    fileEntry("index.html", "static", 0, "text/html; charset=utf-8"),
    fileEntry("app.js", "static", 0, "application/javascript; charset=utf-8"),
    fileEntry("styles.css", "static", 0, "text/css; charset=utf-8"),
  ];
  const seenStatic = new Set(normalizedStaticFiles.map((item) => item.path));
  const staticBundle = [
    ...requiredStatic.filter((item) => !seenStatic.has(item.path)),
    ...normalizedStaticFiles,
  ];
  const lessonFiles = normalizedLessons.map((item) => fileEntry(item.path, "lesson", item.byte_size, item.mime));
  const reportFiles = normalizedReports.map((item) => fileEntry(item.path, "report", item.byte_size, item.mime));
  const assetFiles = normalizedAssets.map((item) => fileEntry(item.path, item.role, item.byte_size, item.mime));
  const files = [...staticBundle, ...lessonFiles, ...reportFiles, ...assetFiles];
  const session = asText(sessionLabel, "");
  const packageIdText = asText(packageId, "local.studio.package");
  const studentInstructions = buildStudioLocalPackageStudentInstructions(normalizedLessons, { sessionLabel: session });
  const materialsSummary = buildStudioLocalPackageMaterialsSummary({
    lessons: normalizedLessons,
    reports: normalizedReports,
    assets: normalizedAssets,
  });
  const studentMaterialsSummary = buildStudioLocalPackageStudentMaterialsSummary({
    lessons: normalizedLessons,
    reports: normalizedReports,
  });
  const profile = buildDeploymentProfileV0(deploymentProfile ?? {});
  if (packageIdText) studentInstructions.push(`배포 코드: ${packageIdText}`);
  const manifest = {
    __종류: "studio_local_package_manifest",
    package_id: packageIdText,
    title: asText(title, "셈그림 교사용 배포 묶음"),
    version: asText(version, "0.1.0"),
    ...(session ? { session_label: session } : {}),
    generated_locally: true,
    delivery_mode: "studio_json_import",
    open_with: "seamgrim_studio_local_package_import",
    student_entry_label: "배포 열기",
    student_instructions: studentInstructions,
    materials_summary: materialsSummary,
    student_materials_summary: studentMaterialsSummary,
    deployment_profile: profile,
    deployment_profile_hash: profile.profile_hash,
    package_hash_inputs: ["manifest", "lessons", "reports", "assets", "deployment_profile"],
    account_required: false,
    cloud_sync: false,
    public_registry: false,
    entry: "index.html",
    lesson_count: normalizedLessons.length,
    report_count: normalizedReports.length,
    asset_count: normalizedAssets.length,
    file_count: files.length,
    required_static_files: requiredStatic.map((item) => item.path),
    files,
  };
  return {
    ...manifest,
    package_hash: buildPackageHash({
      manifest,
      lessons: normalizedLessons,
      reports: normalizedReports,
      assets: normalizedAssets,
    }),
  };
}

export function buildStudioLocalPackagePayload({
  manifest = null,
  lessons = [],
  reports = [],
  assets = [],
  classroomExchange = null,
} = {}) {
  const normalizedLessons = asArray(lessons).map((item, index) => normalizeLesson(item, index));
  const normalizedReports = asArray(reports).map((item, index) => normalizeReport(item, index));
  const normalizedAssets = asArray(assets).map((item, index) => normalizeAsset(item, index));
  const packageManifest = manifest && manifest.__종류 === "studio_local_package_manifest"
    ? manifest
    : buildStudioLocalPackageManifest({ lessons: normalizedLessons, reports: normalizedReports, assets: normalizedAssets });
  const exchange = classroomExchange && typeof classroomExchange === "object"
    ? classroomExchange
    : buildClassroomExchangeV0({
      kind: "lesson_bundle",
      manifest: packageManifest,
      lessons: normalizedLessons,
      reports: normalizedReports,
      assets: normalizedAssets,
    });
  const studentInstructions = asTextArray(packageManifest.student_instructions).length
    ? asTextArray(packageManifest.student_instructions)
    : buildStudioLocalPackageStudentInstructions(normalizedLessons);
  const studentMaterialsSummary = asTextArray(packageManifest.student_materials_summary).length
    ? asTextArray(packageManifest.student_materials_summary)
    : buildStudioLocalPackageStudentMaterialsSummary({ lessons: normalizedLessons, reports: normalizedReports });
  return {
    schema: CLASSROOM_EXCHANGE_SCHEMA_V0,
    kind: exchange.kind,
    exchange_id: exchange.exchange_id,
    lesson_id: exchange.lesson_id,
    origin_hash: exchange.origin_hash,
    payload_hash: exchange.payload_hash,
    deployment_profile_ref: exchange.deployment_profile_ref,
    classroom_exchange_hash: exchange.payload_hash,
    __종류: "studio_local_package_payload",
    manifest: packageManifest,
    classroom_exchange: exchange,
    deployment_profile: packageManifest.deployment_profile ?? null,
    lessons: normalizedLessons,
    reports: normalizedReports,
    assets: normalizedAssets,
    import_export_format: "studio_local_package_payload_v1",
    generated_locally: true,
    delivery_mode: "studio_json_import",
    open_with: "seamgrim_studio_local_package_import",
    student_entry_label: "배포 열기",
    student_instructions: studentInstructions,
    student_materials_summary: studentMaterialsSummary,
    deployment_profile_hash: packageManifest.deployment_profile_hash ?? "",
    package_hash: packageManifest.package_hash ?? "",
    account_required: false,
    cloud_sync: false,
    public_registry: false,
  };
}

export function validateStudioLocalPackagePayload(payload = {}) {
  try {
    const packagePayload = asObject(payload);
    assertClassroomExchangePayloadSize(packagePayload);
    if (packagePayload.__종류 !== "studio_local_package_payload") {
      throw new Error("studio_local_package_expected_payload");
    }
    const manifest = asObject(packagePayload.manifest);
    if (manifest.__종류 !== "studio_local_package_manifest") {
      throw new Error("studio_local_package_expected_manifest");
    }
    if (packagePayload.import_export_format !== "studio_local_package_payload_v1") {
      throw new Error("studio_local_package_format_mismatch");
    }
    if (
      packagePayload.account_required === true ||
      packagePayload.cloud_sync === true ||
      packagePayload.public_registry === true ||
      manifest.account_required === true ||
      manifest.cloud_sync === true ||
      manifest.public_registry === true
    ) {
      throw new Error("studio_local_package_remote_boundary_mismatch");
    }
    const deploymentProfile = assertDeploymentProfileHash(manifest.deployment_profile ?? packagePayload.deployment_profile ?? {});
    if (deploymentProfile && manifest.deployment_profile_hash !== deploymentProfile.profile_hash) {
      throw new Error("studio_local_package_deployment_profile_hash_mismatch");
    }
    const rawLessons = asArray(packagePayload.lessons);
    const rawReports = asArray(packagePayload.reports);
    const rawAssets = asArray(packagePayload.assets);
    assertSafeRawRowPaths(rawLessons);
    assertSafeRawRowPaths(rawReports);
    assertSafeRawRowPaths(rawAssets);
    assertRawAssetByteSizes(rawAssets);
    const lessons = rawLessons.map((item, index) => normalizeLesson(item, index));
    const reports = rawReports.map((item, index) => normalizeReport(item, index));
    const assets = rawAssets.map((item, index) => normalizeAsset(item, index));
    if (lessons.length < 1) {
      throw new Error("studio_local_package_missing_lesson");
    }
    if (lessons.some((lesson) => !String(lesson.source_text ?? "").trim())) {
      throw new Error("studio_local_package_empty_lesson_source");
    }
    assertDeclaredCount("lesson", manifest.lesson_count, lessons.length);
    assertDeclaredCount("report", manifest.report_count, reports.length);
    assertDeclaredCount("asset", manifest.asset_count, assets.length);
    const requiredStatic = asArray(manifest.required_static_files).map((item) => normalizePath(item, ""));
    const missingRequired = ["index.html", "app.js", "styles.css"].filter((item) => !requiredStatic.includes(item));
    if (missingRequired.length > 0) {
      throw new Error("studio_local_package_missing_static_bundle");
    }
    if (normalizePath(manifest.entry, "index.html") !== "index.html") {
      throw new Error("studio_local_package_entry_mismatch");
    }
    const manifestFiles = asArray(manifest.files).map((item) => asObject(item));
    assertDeclaredCount("file", manifest.file_count, manifestFiles.length);
    const declaredStaticPaths = manifestFiles
      .filter((item) => asText(item.type, "") === "static")
      .map((item) => item.path);
    assertSafePackagePaths([
      ...requiredStatic,
      ...manifestFiles.map((item) => item.path),
      ...lessons.map((item) => item.path),
      ...reports.map((item) => item.path),
      ...assets.map((item) => item.path),
    ]);
    assertContentPathsDoNotOverlapStatic([
      ...requiredStatic,
      ...declaredStaticPaths,
    ], [
      ...lessons.map((item) => item.path),
      ...reports.map((item) => item.path),
      ...assets.map((item) => item.path),
    ]);
    assertNoDuplicatePaths(manifestFiles.map((item) => item.path));
    assertManifestContainsPaths(manifestFiles, requiredStatic, "studio_local_package_manifest_missing_static_file");
    assertManifestContainsPaths(manifestFiles, lessons.map((item) => item.path), "studio_local_package_manifest_missing_lesson_file");
    assertManifestContainsPaths(manifestFiles, reports.map((item) => item.path), "studio_local_package_manifest_missing_report_file");
    assertManifestContainsPaths(manifestFiles, assets.map((item) => item.path), "studio_local_package_manifest_missing_asset_file");
    assertManifestByteSizes(manifestFiles, lessons, "studio_local_package_lesson_byte_size_mismatch");
    assertManifestByteSizes(manifestFiles, reports, "studio_local_package_report_byte_size_mismatch");
    assertManifestByteSizes(manifestFiles, assets, "studio_local_package_asset_byte_size_mismatch");
    assertManifestTypes(manifestFiles, lessons, () => "lesson", "studio_local_package_lesson_type_mismatch");
    assertManifestTypes(manifestFiles, reports, () => "report", "studio_local_package_report_type_mismatch");
    assertManifestTypes(manifestFiles, assets, (item) => asText(item.role, "asset"), "studio_local_package_asset_type_mismatch");
    const expectedPackageHash = buildPackageHash({
      manifest,
      lessons,
      reports,
      assets,
    });
    if (manifest.package_hash && manifest.package_hash !== expectedPackageHash) {
      throw new Error("studio_local_package_hash_mismatch");
    }
    if (packagePayload.package_hash && packagePayload.package_hash !== expectedPackageHash) {
      throw new Error("studio_local_package_hash_mismatch");
    }
    assertNoDuplicatePaths([
      ...lessons.map((item) => item.path),
      ...reports.map((item) => item.path),
      ...assets.map((item) => item.path),
    ]);
    const classroomExchange = assertClassroomExchangeV0({
      exchange: packagePayload.classroom_exchange,
      manifest,
      lessons,
      reports,
      assets,
    });
    assertClassroomExchangePayloadMirror(packagePayload, classroomExchange);
    return {
      __종류: "studio_local_package_validation",
      valid: true,
      lesson_count: lessons.length,
      report_count: reports.length,
      asset_count: assets.length,
      file_count: manifestFiles.length,
      deployment_profile_hash: deploymentProfile?.profile_hash ?? "",
      package_hash: expectedPackageHash,
      classroom_exchange_hash: classroomExchange.payload_hash,
      error: "",
    };
  } catch (error) {
    return {
      __종류: "studio_local_package_validation",
      valid: false,
      lesson_count: 0,
      report_count: 0,
      asset_count: 0,
      file_count: 0,
      error: String(error?.message ?? error),
    };
  }
}

export function importStudioLocalPackagePayload(payload = {}) {
  const packagePayload = asObject(payload);
  if (packagePayload.__종류 !== "studio_local_package_payload") {
    throw new Error("studio_local_package_expected_payload");
  }
  const manifest = asObject(packagePayload.manifest);
  if (manifest.__종류 !== "studio_local_package_manifest") {
    throw new Error("studio_local_package_expected_manifest");
  }
  const validation = validateStudioLocalPackagePayload(packagePayload);
  if (!validation.valid) {
    throw new Error(validation.error || "studio_local_package_invalid_payload");
  }
  return {
    __종류: "studio_local_package_import_result",
    manifest,
    classroom_exchange: asObject(packagePayload.classroom_exchange),
    deployment_profile: manifest.deployment_profile ?? packagePayload.deployment_profile ?? null,
    lessons: asArray(packagePayload.lessons).map((item, index) => normalizeLesson(item, index)),
    reports: asArray(packagePayload.reports).map((item, index) => normalizeReport(item, index)),
    assets: asArray(packagePayload.assets).map((item, index) => normalizeAsset(item, index)),
    lesson_count: asArray(packagePayload.lessons).length,
    report_count: asArray(packagePayload.reports).length,
    deployment_profile_hash: String(manifest.deployment_profile_hash ?? packagePayload.deployment_profile_hash ?? "").trim(),
    classroom_exchange_hash: String(packagePayload.classroom_exchange?.payload_hash ?? packagePayload.classroom_exchange_hash ?? "").trim(),
    package_hash: String(manifest.package_hash ?? packagePayload.package_hash ?? "").trim(),
    generated_locally: true,
    account_required: false,
    cloud_sync: false,
    public_registry: false,
  };
}

export function validateStudioStaticBundle({
  manifest = {},
  availableFiles = [],
} = {}) {
  const payload = asObject(manifest);
  if (payload.__종류 !== "studio_local_package_manifest") {
    throw new Error("studio_local_package_expected_manifest");
  }
  const available = new Set(asArray(availableFiles).map((item) => normalizePath(item, "")));
  const required = asArray(payload.required_static_files).map((item) => normalizePath(item, ""));
  const missing = required.filter((item) => item && !available.has(item));
  return {
    __종류: "studio_local_static_bundle_check",
    entry: asText(payload.entry, "index.html"),
    required_count: required.length,
    missing_count: missing.length,
    check_result: missing.length === 0 ? "통과" : "실패",
    missing_files: missing,
  };
}

export function formatStudioLocalPackageIndexText(payload = {}) {
  const packagePayload = asObject(payload);
  if (packagePayload.__종류 !== "studio_local_package_payload") {
    throw new Error("studio_local_package_expected_payload");
  }
  const manifest = asObject(packagePayload.manifest);
  const resultInstruction = asTextArray(packagePayload.student_instructions)
    .find((item) => item.includes("결과 확인")) || "결과 확인";
  const lines = [
    "구분\t경로\t제목\t크기",
    `package\t${String(manifest.package_id ?? "")}\t${String(manifest.title ?? "")}\t${Number(manifest.file_count ?? 0)}`,
    `guide\t학생 배포 안내문\t배포 열기/붙여넣기 열기 → 받은 수업 실행 · ${resultInstruction.replace(/[.。]\s*$/, "")}\t0`,
  ];
  asArray(packagePayload.lessons).forEach((lesson) => {
    lines.push(["lesson", String(lesson.path ?? ""), String(lesson.title ?? ""), String(lesson.byte_size ?? 0)].join("\t"));
  });
  asArray(packagePayload.reports).forEach((report) => {
    lines.push(["report", String(report.path ?? ""), String(report.title ?? ""), String(report.byte_size ?? 0)].join("\t"));
  });
  asArray(packagePayload.assets).forEach((asset) => {
    lines.push([String(asset.role ?? "asset"), String(asset.path ?? ""), "", String(asset.byte_size ?? 0)].join("\t"));
  });
  return lines.join("\n");
}
