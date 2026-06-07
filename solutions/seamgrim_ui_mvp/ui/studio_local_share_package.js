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

function byteLengthUtf8(value) {
  return new TextEncoder().encode(String(value ?? "")).length;
}

function normalizePath(value, fallback) {
  const text = asText(value, fallback).replace(/\\/g, "/").replace(/^\/+/, "");
  return text || fallback;
}

function normalizeLesson(lesson, index) {
  const row = asObject(lesson);
  const lessonId = asText(row.lesson_id ?? row.id ?? row["수업"], `lesson_${String(index + 1).padStart(3, "0")}`);
  const source = String(row.source_text ?? row.source ?? row.ddn ?? row["본문"] ?? "");
  return {
    lesson_id: lessonId,
    title: asText(row.title ?? row.name ?? row["제목"], lessonId),
    path: normalizePath(row.path, `lessons/${lessonId}.ddn`),
    mime: "text/plain; charset=utf-8",
    source_text: source,
    byte_size: byteLengthUtf8(source),
  };
}

function normalizeReport(report, index) {
  const row = asObject(report);
  const reportId = asText(row.report_id ?? row.id ?? row["보고서"], `report_${String(index + 1).padStart(3, "0")}`);
  const text = String(row.text ?? row.report_text ?? row["본문"] ?? "");
  return {
    report_id: reportId,
    title: asText(row.title ?? row.name ?? row["제목"], reportId),
    path: normalizePath(row.path, `reports/${reportId}.txt`),
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
  return {
    __종류: "studio_local_package_manifest",
    package_id: asText(packageId, "local.studio.package"),
    title: asText(title, "Studio local package"),
    version: asText(version, "0.1.0"),
    generated_locally: true,
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
  return {
    __종류: "studio_local_package_payload",
    manifest: packageManifest,
    lessons: normalizedLessons,
    reports: normalizedReports,
    assets: normalizedAssets,
    import_export_format: "studio_local_package_payload_v1",
    generated_locally: true,
    account_required: false,
    cloud_sync: false,
    public_registry: false,
  };
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
  const lines = [
    "구분\t경로\t제목\t크기",
    `package\t${String(manifest.package_id ?? "")}\t${String(manifest.title ?? "")}\t${Number(manifest.file_count ?? 0)}`,
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
