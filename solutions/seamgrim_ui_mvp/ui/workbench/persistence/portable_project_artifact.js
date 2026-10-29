import {
  assertProjectSnapshotArtifactV0,
  buildProjectSnapshotArtifactV0,
  PROJECT_SNAPSHOT_SCHEMA,
  PROJECT_SNAPSHOT_WRAPPER_SCHEMA,
  stableProjectSnapshotJson,
} from "../../project_snapshot.js";

export const PORTABLE_PROJECT_ARTIFACT_CONTRACT = "seamgrim.portable_project_artifact_contract.v1";
export const MAX_PORTABLE_PROJECT_BYTES = 2_000_000;
export const PORTABLE_PROJECT_EXTENSION = ".seamgrim-project.json";

const WRAPPER_KEYS = new Set([
  "schema",
  "project_snapshot",
  "canonical_json",
  "content_sha256",
  "canonical_byte_length",
]);

export const CURRENT_SNAPSHOT_PRESERVATION_FIELDS = Object.freeze([
  "lesson_plan_text",
  "controls_snapshot",
  "local_package_payload",
  "curriculum_inquiry_artifact",
  "curriculum_artifact_bundle",
  "curriculum_model_comparison",
  "curriculum_semantic_scene",
  "lineage",
  "metadata",
  "boundaries",
]);

const KNOWN_SNAPSHOT_FIELDS = new Set([
  "schema",
  "project_name",
  "source_lesson_id",
  "ddn_text",
  "lesson_plan_text",
  "controls_snapshot",
  "local_package_payload",
  "package_hash_compat",
  "curriculum_inquiry_artifact",
  "curriculum_artifact_bundle",
  "curriculum_model_comparison",
  "curriculum_semantic_scene",
  "lineage",
  "metadata",
  "boundaries",
]);

const FORBIDDEN_KEY_PATTERNS = [
  /capability/i,
  /authorization/i,
  /access[_-]?token/i,
  /refresh[_-]?token/i,
  /^token$/i,
  /^secret$/i,
  /edit[_-]?link/i,
  /view[_-]?link/i,
  /server[_-]?path/i,
  /local[_-]?storage/i,
  /browser[_-]?history/i,
  /checkpoint[_-]?archive/i,
  /run[_-]?archive/i,
];

const FORBIDDEN_STRING_PATTERNS = [
  /\bBearer\s+[A-Za-z0-9._~+/=-]+/i,
  /\bAuthorization\s*:/i,
  /\bsg_cap_[A-Za-z0-9._-]+/i,
  /https?:\/\/[^\s"'<>]+/i,
  /\b(?:edit|view)[_-]?capability\b/i,
];

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function byteLengthUtf8(value) {
  return new TextEncoder().encode(String(value ?? "")).length;
}

function makeError(code, message, details = {}) {
  const err = new Error(message || code);
  err.code = code;
  err.details = details;
  return err;
}

function normalizeError(err) {
  if (err?.code) return err;
  const message = String(err?.message ?? err ?? "");
  if (message.includes("artifact_schema")) return makeError("E_PORTABLE_WRAPPER_SCHEMA", "portable wrapper schema mismatch");
  if (message.includes("snapshot_schema")) return makeError("E_PORTABLE_SNAPSHOT_SCHEMA", "portable snapshot schema mismatch");
  if (message.includes("canonical_json")) return makeError("E_PORTABLE_CANONICAL_MISMATCH", "portable canonical JSON mismatch");
  if (message.includes("sha256")) return makeError("E_PORTABLE_SHA_MISMATCH", "portable SHA-256 mismatch");
  return makeError("E_PORTABLE_VALIDATION_FAILED", message || "portable project validation failed");
}

function bytesFromInput(input) {
  if (input instanceof Uint8Array) return input;
  if (input instanceof ArrayBuffer) return new Uint8Array(input);
  if (typeof input === "string") return new TextEncoder().encode(input);
  if (ArrayBuffer.isView(input)) {
    return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  }
  throw makeError("E_PORTABLE_BYTES_UNSUPPORTED", "portable project bytes must be text, ArrayBuffer, or Uint8Array");
}

function decodeUtf8(bytes) {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch (_) {
    throw makeError("E_PORTABLE_FILE_UTF8", "portable project file must be valid UTF-8 JSON");
  }
}

function parseJson(text) {
  try {
    return JSON.parse(text);
  } catch (err) {
    throw makeError("E_PORTABLE_FILE_JSON", "portable project file must be JSON", { message: String(err?.message ?? err) });
  }
}

function assertWrapperShape(artifact) {
  const row = asObject(artifact);
  if (row.schema !== PROJECT_SNAPSHOT_WRAPPER_SCHEMA) {
    throw makeError("E_PORTABLE_WRAPPER_SCHEMA", "portable wrapper schema mismatch");
  }
  for (const key of Object.keys(row)) {
    if (!WRAPPER_KEYS.has(key)) {
      throw makeError("E_PORTABLE_WRAPPER_FIELD", `portable wrapper has unsupported field: ${key}`, { key });
    }
  }
  const snapshot = asObject(row.project_snapshot);
  if (snapshot.schema !== PROJECT_SNAPSHOT_SCHEMA) {
    throw makeError("E_PORTABLE_SNAPSHOT_SCHEMA", "portable snapshot schema mismatch");
  }
  return row;
}

function assertNoPrototypeKey(key, path) {
  if (key === "__proto__" || key === "constructor.prototype" || path.endsWith(".constructor.prototype")) {
    throw makeError("E_PORTABLE_PROTOTYPE_KEY", `prototype pollution key rejected at ${path}`, { path });
  }
}

function assertNoForbiddenKey(key, path) {
  if (FORBIDDEN_KEY_PATTERNS.some((pattern) => pattern.test(key))) {
    throw makeError("E_PORTABLE_FORBIDDEN_FIELD", `forbidden portable project field rejected at ${path}`, { path, key });
  }
}

function assertNoForbiddenString(value, path) {
  const text = String(value ?? "");
  const pattern = FORBIDDEN_STRING_PATTERNS.find((item) => item.test(text));
  if (pattern) {
    throw makeError("E_PORTABLE_FORBIDDEN_FIELD", `forbidden portable project string rejected at ${path}`, { path });
  }
}

export function scanPortableProjectArtifactBoundary(value, path = "$") {
  if (Array.isArray(value)) {
    value.forEach((item, index) => scanPortableProjectArtifactBoundary(item, `${path}[${index}]`));
    return true;
  }
  if (value && typeof value === "object") {
    for (const key of Object.keys(value)) {
      const childPath = `${path}.${key}`;
      assertNoPrototypeKey(key, childPath);
      assertNoForbiddenKey(key, childPath);
      scanPortableProjectArtifactBoundary(value[key], childPath);
    }
    return true;
  }
  if (typeof value === "string") {
    assertNoForbiddenString(value, path);
  }
  return true;
}

export function auditPortableSnapshotFieldPreservation(snapshot = {}) {
  const row = asObject(snapshot);
  const present_current_fields = CURRENT_SNAPSHOT_PRESERVATION_FIELDS.filter((field) =>
    Object.prototype.hasOwnProperty.call(row, field),
  );
  const missing_current_fields = CURRENT_SNAPSHOT_PRESERVATION_FIELDS.filter((field) =>
    !Object.prototype.hasOwnProperty.call(row, field),
  );
  const unknown_snapshot_fields = Object.keys(row)
    .filter((field) => !KNOWN_SNAPSHOT_FIELDS.has(field))
    .sort();
  return {
    schema: "seamgrim.portable_project_field_preservation_audit.v1",
    present_current_fields,
    missing_current_fields,
    unknown_snapshot_fields,
    unknown_fields_preserved_by_validation: true,
  };
}

export function safePortableProjectFilename(projectName = "seamgrim-project") {
  const token = String(projectName || "seamgrim-project")
    .normalize("NFKC")
    .replace(/[\u0000-\u001f<>:"/\\|?*]+/g, "-")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80);
  return `${token || "seamgrim-project"}${PORTABLE_PROJECT_EXTENSION}`;
}

export function serializePortableProjectArtifact(artifact = {}) {
  const jsonText = stableProjectSnapshotJson(artifact);
  return {
    schema: "seamgrim.portable_project_file_bytes.v1",
    json_text: jsonText,
    bytes: new TextEncoder().encode(jsonText),
    byte_length: byteLengthUtf8(jsonText),
  };
}

export async function validatePortableProjectArtifact(artifact = {}) {
  let row;
  try {
    scanPortableProjectArtifactBoundary(artifact);
    row = assertWrapperShape(artifact);
    await assertProjectSnapshotArtifactV0(row);
  } catch (err) {
    throw normalizeError(err);
  }
  const canonicalByteLength = byteLengthUtf8(row.canonical_json);
  if (Number(row.canonical_byte_length) !== canonicalByteLength) {
    throw makeError("E_PORTABLE_CANONICAL_BYTE_LENGTH", "portable canonical byte length mismatch");
  }
  return {
    schema: "seamgrim.portable_project_validation_result.v1",
    ok: true,
    wrapper_schema: row.schema,
    snapshot_schema: row.project_snapshot.schema,
    content_sha256: row.content_sha256,
    canonical_byte_length: canonicalByteLength,
    field_audit: auditPortableSnapshotFieldPreservation(row.project_snapshot),
  };
}

export async function parsePortableProjectArtifactBytes(input, { maxBytes = MAX_PORTABLE_PROJECT_BYTES } = {}) {
  const bytes = bytesFromInput(input);
  if (bytes.byteLength > maxBytes) {
    throw makeError("E_PORTABLE_FILE_TOO_LARGE", "portable project file is larger than the local 2 MiB limit", {
      byte_length: bytes.byteLength,
      max_bytes: maxBytes,
    });
  }
  const text = decodeUtf8(bytes);
  const artifact = parseJson(text);
  const validation = await validatePortableProjectArtifact(artifact);
  return {
    schema: "seamgrim.portable_project_parse_result.v1",
    artifact,
    validation,
    byte_length: bytes.byteLength,
  };
}

export async function buildPortableProjectArtifact(input = {}) {
  const artifact = await buildProjectSnapshotArtifactV0(input);
  const validation = await validatePortableProjectArtifact(artifact);
  const serialized = serializePortableProjectArtifact(artifact);
  if (serialized.byte_length > MAX_PORTABLE_PROJECT_BYTES) {
    throw makeError("E_PORTABLE_FILE_TOO_LARGE", "portable project export exceeds the local 2 MiB limit", {
      byte_length: serialized.byte_length,
      max_bytes: MAX_PORTABLE_PROJECT_BYTES,
    });
  }
  const filename = safePortableProjectFilename(
    input.fileNameBase ?? input.projectName ?? artifact.project_snapshot?.metadata?.title ?? artifact.project_snapshot?.project_name,
  );
  return {
    schema: PORTABLE_PROJECT_ARTIFACT_CONTRACT,
    artifact,
    validation,
    file: {
      schema: "seamgrim.portable_project_file.v1",
      filename,
      json_text: serialized.json_text,
      byte_length: serialized.byte_length,
      content_sha256: artifact.content_sha256,
    },
  };
}

export function portableProjectDiagnostic(err) {
  const normalized = normalizeError(err);
  const labels = {
    E_PORTABLE_FILE_TOO_LARGE: "파일이 2MiB 제한보다 큽니다.",
    E_PORTABLE_FILE_UTF8: "UTF-8 JSON 파일만 가져올 수 있습니다.",
    E_PORTABLE_FILE_JSON: "JSON 형식이 아닙니다.",
    E_PORTABLE_WRAPPER_SCHEMA: "셈그림 프로젝트 파일 형식이 아닙니다.",
    E_PORTABLE_WRAPPER_FIELD: "지원하지 않는 wrapper 필드가 있습니다.",
    E_PORTABLE_SNAPSHOT_SCHEMA: "프로젝트 snapshot 형식이 맞지 않습니다.",
    E_PORTABLE_CANONICAL_MISMATCH: "canonical JSON이 파일 내용과 맞지 않습니다.",
    E_PORTABLE_SHA_MISMATCH: "content SHA-256이 canonical JSON과 맞지 않습니다.",
    E_PORTABLE_CANONICAL_BYTE_LENGTH: "canonical byte 길이가 맞지 않습니다.",
    E_PORTABLE_FORBIDDEN_FIELD: "권한·링크·토큰 계열 정보가 포함되어 거부했습니다.",
    E_PORTABLE_PROTOTYPE_KEY: "prototype pollution 위험 키가 포함되어 거부했습니다.",
  };
  return {
    schema: "seamgrim.portable_project_diagnostic.v1",
    code: normalized.code,
    severity: "error",
    message: labels[normalized.code] || "프로젝트 파일 검증에 실패했습니다.",
    details: normalized.details ?? {},
  };
}
