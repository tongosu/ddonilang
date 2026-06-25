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
    path: normalizePath(row.path, `lessons/${pathToken}.ddn`),
    mime: "text/plain; charset=utf-8",
    source_text: source,
    byte_size: byteLengthUtf8(source),
  };
}

function formatStudentRequiredViewLabel(value) {
  const key = String(value ?? "").trim().toLowerCase();
  if (key === "graph") return "그래프";
  if (key === "table") return "표";
  if (key === "space2d") return "그림";
  if (key === "text") return "설명";
  return String(value ?? "").trim();
}

function buildStudioLocalPackageStudentInstructions(lessons = []) {
  const firstLesson = asArray(lessons)[0] ?? {};
  const viewLabels = asTextArray(firstLesson.required_views ?? firstLesson.requiredViews)
    .map(formatStudentRequiredViewLabel)
    .filter(Boolean)
    .slice(0, 4);
  const resultInstruction = viewLabels.length
    ? `결과 확인: ${viewLabels.join(", ")}를 확인합니다.`
    : "결과 확인 화면을 확인합니다.";
  return [
    "셈그림 Studio에서 배포 열기를 누른 뒤 선생님이 보낸 JSON 배포 파일을 선택합니다.",
    "받은 수업 실행을 눌러 교사가 보낸 수업을 시작합니다.",
    resultInstruction,
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
    mime: "text/plain; charset=utf-8",
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
  lessons = [],
  reports = [],
  assets = [],
  staticFiles = [],
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
  const studentInstructions = buildStudioLocalPackageStudentInstructions(normalizedLessons);
  return {
    __종류: "studio_local_package_manifest",
    package_id: asText(packageId, "local.studio.package"),
    title: asText(title, "셈그림 교사용 배포 묶음"),
    version: asText(version, "0.1.0"),
    generated_locally: true,
    delivery_mode: "studio_json_import",
    open_with: "seamgrim_studio_local_package_import",
    student_entry_label: "배포 열기",
    student_instructions: studentInstructions,
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
}

export function buildStudioLocalPackagePayload({
  manifest = null,
  lessons = [],
  reports = [],
  assets = [],
} = {}) {
  const normalizedLessons = asArray(lessons).map((item, index) => normalizeLesson(item, index));
  const normalizedReports = asArray(reports).map((item, index) => normalizeReport(item, index));
  const normalizedAssets = asArray(assets).map((item, index) => normalizeAsset(item, index));
  const packageManifest = manifest && manifest.__종류 === "studio_local_package_manifest"
    ? manifest
    : buildStudioLocalPackageManifest({ lessons: normalizedLessons, reports: normalizedReports, assets: normalizedAssets });
  const studentInstructions = asTextArray(packageManifest.student_instructions).length
    ? asTextArray(packageManifest.student_instructions)
    : buildStudioLocalPackageStudentInstructions(normalizedLessons);
  return {
    __종류: "studio_local_package_payload",
    manifest: packageManifest,
    lessons: normalizedLessons,
    reports: normalizedReports,
    assets: normalizedAssets,
    import_export_format: "studio_local_package_payload_v1",
    generated_locally: true,
    delivery_mode: "studio_json_import",
    open_with: "seamgrim_studio_local_package_import",
    student_entry_label: "배포 열기",
    student_instructions: studentInstructions,
    account_required: false,
    cloud_sync: false,
    public_registry: false,
  };
}

export function validateStudioLocalPackagePayload(payload = {}) {
  try {
    const packagePayload = asObject(payload);
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
    assertNoDuplicatePaths([
      ...lessons.map((item) => item.path),
      ...reports.map((item) => item.path),
      ...assets.map((item) => item.path),
    ]);
    return {
      __종류: "studio_local_package_validation",
      valid: true,
      lesson_count: lessons.length,
      report_count: reports.length,
      asset_count: assets.length,
      file_count: manifestFiles.length,
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
    lessons: asArray(packagePayload.lessons).map((item, index) => normalizeLesson(item, index)),
    reports: asArray(packagePayload.reports).map((item, index) => normalizeReport(item, index)),
    assets: asArray(packagePayload.assets).map((item, index) => normalizeAsset(item, index)),
    lesson_count: asArray(packagePayload.lessons).length,
    report_count: asArray(packagePayload.reports).length,
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
    `guide\tStudio 배포 열기\t받은 수업 실행 · ${resultInstruction.replace(/[.。]\s*$/, "")}\t0`,
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
