export const OBSERVATION_MANIFEST_SCHEMA = "ddn.observation_manifest.v0";
export const OBSERVATION_MANIFEST_VERSION = "20.6.33";

const OBSERVATION_MANIFEST_FAILURE = Object.freeze({
  missing: {
    code: "E_WASM_OBSERVATION_MANIFEST_MISSING",
    message: "이 실행 결과에는 필요한 observation manifest가 없습니다.",
  },
  invalid: {
    code: "E_WASM_OBSERVATION_MANIFEST_INVALID",
    message: "이 실행 결과의 observation manifest 형식이 올바르지 않습니다.",
  },
  unsupported: {
    code: "E_WASM_OBSERVATION_MANIFEST_UNSUPPORTED",
    message: "이 실행 결과의 observation manifest schema 또는 version을 지원하지 않습니다.",
  },
});

export function buildObservationManifestContractDiagnostic(
  kind,
  reason,
  manifest = null,
  operation = "state",
) {
  const contract = OBSERVATION_MANIFEST_FAILURE[kind] ?? OBSERVATION_MANIFEST_FAILURE.invalid;
  const actualSchema = manifest && typeof manifest === "object" && !Array.isArray(manifest)
    ? String(manifest.schema ?? "")
    : "";
  const actualVersion = manifest && typeof manifest === "object" && !Array.isArray(manifest)
    ? String(manifest.version ?? "")
    : "";
  return {
    code: contract.code,
    message: contract.message,
    phase: "observation_manifest",
    outcome: String(kind ?? "invalid").toUpperCase(),
    detail: JSON.stringify({
      phase: "observation_manifest",
      operation: String(operation ?? "state"),
      outcome: String(kind ?? "invalid").toUpperCase(),
      reason: String(reason ?? "invalid_manifest"),
      expected_schema: OBSERVATION_MANIFEST_SCHEMA,
      expected_version: OBSERVATION_MANIFEST_VERSION,
      actual_schema: actualSchema,
      actual_version: actualVersion,
    }),
  };
}

function invalidInspection(reason, manifest, operation) {
  return {
    status: "invalid",
    manifest: null,
    diagnostic: buildObservationManifestContractDiagnostic("invalid", reason, manifest, operation),
  };
}

export function inspectNativeObservationManifest(payload, operation = "state") {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    return invalidInspection("payload_not_object", null, operation);
  }
  if (
    !("observation_manifest" in payload)
    || payload.observation_manifest === null
    || payload.observation_manifest === undefined
  ) {
    return {
      status: "missing",
      manifest: null,
      diagnostic: buildObservationManifestContractDiagnostic(
        "missing",
        "manifest_absent",
        null,
        operation,
      ),
    };
  }
  const manifest = payload.observation_manifest;
  if (!manifest || typeof manifest !== "object" || Array.isArray(manifest)) {
    return invalidInspection("manifest_not_object", manifest, operation);
  }
  const schema = String(manifest.schema ?? "").trim();
  const version = String(manifest.version ?? "").trim();
  if (!schema || !version) {
    return invalidInspection("schema_or_version_missing", manifest, operation);
  }
  if (schema !== OBSERVATION_MANIFEST_SCHEMA || version !== OBSERVATION_MANIFEST_VERSION) {
    return {
      status: "unsupported",
      manifest: null,
      diagnostic: buildObservationManifestContractDiagnostic(
        "unsupported",
        "schema_or_version_unsupported",
        manifest,
        operation,
      ),
    };
  }
  if (
    !Array.isArray(manifest.nodes)
    || !Array.isArray(manifest.params)
    || !Array.isArray(manifest.traces)
    || !Array.isArray(manifest.diagnostics)
    || !manifest.default_pivot
    || typeof manifest.default_pivot !== "object"
    || Array.isArray(manifest.default_pivot)
    || typeof manifest.default_pivot.x_channel !== "string"
    || !Array.isArray(manifest.default_pivot.y_channels)
    || manifest.default_pivot.y_channels.some((name) => typeof name !== "string")
    || manifest.nodes.some((node) => (
      !node
      || typeof node !== "object"
      || Array.isArray(node)
      || !String(node.name ?? "").trim()
      || !String(node.dtype ?? "").trim()
      || !String(node.role ?? "").trim()
    ))
  ) {
    return invalidInspection("manifest_shape_invalid", manifest, operation);
  }
  const nodeNames = manifest.nodes.map((node) => String(node.name).trim());
  if (new Set(nodeNames).size !== nodeNames.length) {
    return invalidInspection("manifest_node_identity_duplicate", manifest, operation);
  }
  return {
    status: "available",
    manifest,
    diagnostic: null,
  };
}
