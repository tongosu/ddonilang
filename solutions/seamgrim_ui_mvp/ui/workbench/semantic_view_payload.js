export const SEMANTIC_VIEW_PAYLOAD_SCHEMA = "seamgrim.workbench.semantic_view_payload.v1";
export const CANONICAL_VIEW_PAYLOAD_SCHEMA = "seamgrim.semantic_view_payload.v1_candidate";
export const SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA = "seamgrim.scene_summary_semantic_projection.v1_candidate";
export const CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA = "seamgrim.curriculum_semantic_scene_projection.v1_candidate";

const WORKBENCH_RESULT_SET_SCHEMA = "seamgrim.workbench.authoritative_result_set.v1_candidate";
const CANONICAL_VIEW_STATUS_DOMAIN = Object.freeze([
  "available",
  "no_view",
  "unavailable",
  "invalid",
  "unsupported",
  "ambiguous",
]);
const CANONICAL_VIEW_STATUS_SET = new Set(CANONICAL_VIEW_STATUS_DOMAIN);
const WORKBENCH_RESULT_SCHEMAS = Object.freeze({
  line: "seamgrim.workbench.line_result.v1_candidate",
  pendulum: "seamgrim.workbench.pendulum_result.v1_candidate",
  projectile: "seamgrim.workbench.projectile_result.v1_candidate",
  grid: "seamgrim.workbench.grid_result.v1_candidate",
});

function canonicalProjectionStatus(value, fallback = "unavailable") {
  const raw = String(value ?? fallback);
  const accepted = CANONICAL_VIEW_STATUS_SET.has(raw);
  return {
    raw,
    accepted,
    status: accepted ? raw : "invalid",
  };
}

function unknownProjectionStatusDiagnostic(code, statusInfo) {
  return {
    code,
    reason: "canonical_projection_status_invalid",
    actual_status: statusInfo.raw,
  };
}

function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value).sort().map((key) => [key, canonicalValue(value[key])]),
    );
  }
  return value;
}

export function stableSemanticJson(value) {
  return JSON.stringify(canonicalValue(value));
}

function fnv1a32(text) {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

export function semanticViewPayloadHash(payload) {
  return `candidate:fnv1a32:semantic_view:${fnv1a32(stableSemanticJson(payload))}`;
}

function canonicalViewPayloadHash(payload) {
  return `candidate:fnv1a32:canonical_view_payload:${fnv1a32(stableSemanticJson(payload))}`;
}

function normalizeExistingViewIdentity(identity = {}) {
  const tick = Number(identity?.tick_id);
  return {
    state_hash: String(identity?.state_hash ?? "").trim(),
    source_hash: String(identity?.source_hash ?? "").trim(),
    result_identity: String(identity?.result_identity ?? "").trim(),
    observation_projection_identity: String(identity?.observation_projection_identity ?? "").trim(),
    view_hash: String(identity?.view_hash ?? "").trim(),
    tick_id: Number.isFinite(tick) ? Math.trunc(tick) : null,
  };
}

function canonicalOutcome(family, status, identity, projection, content = null, diagnostic = null) {
  const provenance = {
    input_authority: String(projection?.provenance ?? "native_authoritative_structured_input"),
    source: String(projection?.source ?? "unavailable"),
  };
  const classification = String(projection?.classification ?? "").trim();
  if (classification) provenance.classification = classification;
  const payload = {
    schema: CANONICAL_VIEW_PAYLOAD_SCHEMA,
    contract_status: "product_local_candidate",
    family,
    status,
    identity: normalizeExistingViewIdentity(identity),
    provenance,
    content: content === null ? null : canonicalValue(content),
    diagnostic: diagnostic ? canonicalValue(diagnostic) : null,
  };
  return {
    ...payload,
    candidate_payload_hash: canonicalViewPayloadHash(payload),
  };
}

function requiredIdentityAvailable(identity = {}) {
  const normalized = normalizeExistingViewIdentity(identity);
  return Boolean(normalized.state_hash);
}

export function buildCanonicalConsoleViewPayload({ identity = {}, projection = null } = {}) {
  const statusInfo = canonicalProjectionStatus(projection?.status);
  const status = statusInfo.status;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("console", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (status !== "available") {
    const diagnostic = statusInfo.accepted
      ? projection?.diagnostic ?? null
      : unknownProjectionStatusDiagnostic("E_CANONICAL_CONSOLE_INVALID_CANDIDATE", statusInfo);
    return canonicalOutcome("console", status, identity, projection, null, diagnostic);
  }
  const entries = Array.isArray(projection?.entries) ? projection.entries : [];
  const normalizedEntries = entries.map((entry, index) => ({
    tick: Number.isFinite(Number(entry?.tick)) ? Math.trunc(Number(entry.tick)) : null,
    line_no: Number.isFinite(Number(entry?.line_no)) ? Math.trunc(Number(entry.line_no)) : index + 1,
    kind: String(entry?.kind ?? "output"),
    text: String(entry?.text ?? ""),
    output_identity: String(entry?.output_identity ?? ""),
    provenance: String(entry?.provenance ?? ""),
  }));
  return canonicalOutcome("console", "available", identity, projection, {
    ordered_entries: normalizedEntries,
  });
}

export function buildCanonicalGraphViewPayload({ identity = {}, projection = null } = {}) {
  const statusInfo = canonicalProjectionStatus(projection?.status);
  const status = statusInfo.status;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("graph", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (status !== "available") {
    if (!statusInfo.accepted) {
      return canonicalOutcome("graph", "invalid", identity, projection, null,
        unknownProjectionStatusDiagnostic("E_CANONICAL_GRAPH_INVALID_CANDIDATE", statusInfo));
    }
    const reason = String(projection?.reason ?? "").trim();
    const diagnostic = reason
      ? {
          code: status === "unsupported"
            ? "E_CANONICAL_GRAPH_UNSUPPORTED_CANDIDATE"
            : "E_CANONICAL_GRAPH_INVALID_CANDIDATE",
          reason,
          actual_schema: String(projection?.actual_schema ?? ""),
        }
      : null;
    return canonicalOutcome("graph", status, identity, projection, null, diagnostic);
  }
  return canonicalOutcome("graph", "available", identity, projection, {
    graph: projection.graph,
    series_identities: Array.isArray(projection?.series_identities)
      ? projection.series_identities.map(String)
      : [],
  });
}

function space2dEntityRows(space2d) {
  const collections = [
    ["points", Array.isArray(space2d?.points) ? space2d.points : []],
    ["shapes", Array.isArray(space2d?.shapes) ? space2d.shapes : []],
    ["drawlist", Array.isArray(space2d?.drawlist) ? space2d.drawlist : []],
  ];
  return collections.flatMap(([collection, rows]) => rows.map((row) => {
    const primitiveType = collection === "points" ? "point" : String(row?.kind ?? "").trim().toLowerCase();
    const sourceIdentity = String(row?.id ?? row?.group_id ?? "").trim();
    const identityRow = {
      entity_id: sourceIdentity,
      collection,
      primitive_type: primitiveType,
      identity_source: "source",
    };
    const semanticRole = String(row?.role ?? "").trim();
    if (semanticRole) identityRow.semantic_role = semanticRole;
    return identityRow;
  }));
}

export function buildCanonicalSpace2dViewPayload({ identity = {}, projection = null } = {}) {
  const statusInfo = canonicalProjectionStatus(projection?.status);
  const status = statusInfo.status;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("space2d", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (status !== "available") {
    if (!statusInfo.accepted) {
      return canonicalOutcome("space2d", "invalid", identity, projection, null,
        unknownProjectionStatusDiagnostic("E_CANONICAL_SPACE2D_INVALID_CANDIDATE", statusInfo));
    }
    const reason = String(projection?.reason ?? "").trim();
    const diagnostic = reason
      ? {
          code: status === "unsupported"
            ? "E_CANONICAL_SPACE2D_UNSUPPORTED_CANDIDATE"
            : "E_CANONICAL_SPACE2D_INVALID_CANDIDATE",
          reason,
          actual_schema: String(projection?.actual_schema ?? ""),
        }
      : null;
    return canonicalOutcome("space2d", status, identity, projection, null, diagnostic);
  }

  const entities = space2dEntityRows(projection?.space2d);
  const entityIds = entities.map((row) => row.entity_id);
  if (entityIds.some((entityId) => !entityId) || new Set(entityIds).size !== entityIds.length) {
    return canonicalOutcome("space2d", "invalid", identity, projection, null, {
      code: "E_CANONICAL_SPACE2D_INVALID_CANDIDATE",
      reason: "space2d_entity_identity_duplicate",
      actual_schema: String(projection?.actual_schema ?? ""),
    });
  }
  return canonicalOutcome("space2d", "available", identity, projection, {
    family_contract_version: "space2d.v1_candidate",
    space2d: projection.space2d,
    entity_identities: entities,
  });
}

function canonicalFamilyDiagnostic(family, status, projection) {
  const reason = String(projection?.reason ?? "").trim();
  if (!reason) return projection?.diagnostic ?? null;
  return {
    code: status === "unsupported"
      ? `E_CANONICAL_${family.toUpperCase()}_UNSUPPORTED_CANDIDATE`
      : `E_CANONICAL_${family.toUpperCase()}_INVALID_CANDIDATE`,
    reason,
    actual_schema: String(projection?.actual_schema ?? ""),
  };
}

export function buildCanonicalTableViewPayload({ identity = {}, projection = null } = {}) {
  const statusInfo = canonicalProjectionStatus(projection?.status);
  const status = statusInfo.status;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("table", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (status !== "available") {
    const diagnostic = statusInfo.accepted
      ? canonicalFamilyDiagnostic("table", status, projection)
      : unknownProjectionStatusDiagnostic("E_CANONICAL_TABLE_INVALID_CANDIDATE", statusInfo);
    return canonicalOutcome(
      "table",
      status,
      identity,
      projection,
      null,
      diagnostic,
    );
  }
  return canonicalOutcome("table", "available", identity, projection, {
    family_contract_version: "table.v1_candidate",
    table: projection.table,
    column_identities: projection.table.columns.map((column) => String(column.key)),
  });
}

export function buildCanonicalTextViewPayload({ identity = {}, projection = null } = {}) {
  const statusInfo = canonicalProjectionStatus(projection?.status);
  const status = statusInfo.status;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("text", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (status !== "available") {
    const diagnostic = statusInfo.accepted
      ? canonicalFamilyDiagnostic("text", status, projection)
      : unknownProjectionStatusDiagnostic("E_CANONICAL_TEXT_INVALID_CANDIDATE", statusInfo);
    return canonicalOutcome(
      "text",
      status,
      identity,
      projection,
      null,
      diagnostic,
    );
  }
  return canonicalOutcome("text", "available", identity, projection, {
    family_contract_version: "text.v1_candidate",
    text: projection.text,
  });
}

export function buildCanonicalStructureViewPayload({ identity = {}, projection = null } = {}) {
  const statusInfo = canonicalProjectionStatus(projection?.status);
  const status = statusInfo.status;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("structure", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (status !== "available") {
    const diagnostic = statusInfo.accepted
      ? canonicalFamilyDiagnostic("structure", status, projection)
      : unknownProjectionStatusDiagnostic("E_CANONICAL_STRUCTURE_INVALID_CANDIDATE", statusInfo);
    return canonicalOutcome(
      "structure",
      status,
      identity,
      projection,
      null,
      diagnostic,
    );
  }
  return canonicalOutcome("structure", "available", identity, projection, {
    family_contract_version: "structure.v1_candidate",
    structure: projection.structure,
    node_identities: projection.structure.nodes.map((node) => String(node.id)),
  });
}

function canonicalConsoleGridProjection(consolePayload) {
  return {
    provenance: String(consolePayload?.provenance?.input_authority ?? "native_authoritative_structured_input"),
    source: "canonical_console_view_payload",
    classification: "AUTHORITATIVE_STRUCTURED",
  };
}

export function buildCanonicalConsoleGridViewPayload({ identity = {}, consolePayload = null } = {}) {
  const projection = canonicalConsoleGridProjection(consolePayload);
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("console-grid", "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (
    consolePayload?.schema !== CANONICAL_VIEW_PAYLOAD_SCHEMA
    || consolePayload?.family !== "console"
  ) {
    return canonicalOutcome("console-grid", "invalid", identity, projection, null, {
      code: "E_CANONICAL_CONSOLE_GRID_INVALID_CANDIDATE",
      reason: "canonical_console_payload_required",
    });
  }
  const consoleStatusInfo = canonicalProjectionStatus(consolePayload.status);
  if (consoleStatusInfo.status !== "available") {
    const diagnostic = consoleStatusInfo.accepted
      ? consolePayload.diagnostic ?? null
      : unknownProjectionStatusDiagnostic("E_CANONICAL_CONSOLE_GRID_INVALID_CANDIDATE", consoleStatusInfo);
    return canonicalOutcome(
      "console-grid",
      consoleStatusInfo.status,
      identity,
      projection,
      null,
      diagnostic,
    );
  }
  const entries = Array.isArray(consolePayload?.content?.ordered_entries)
    ? consolePayload.content.ordered_entries
    : null;
  if (!entries) {
    return canonicalOutcome("console-grid", "invalid", identity, projection, null, {
      code: "E_CANONICAL_CONSOLE_GRID_INVALID_CANDIDATE",
      reason: "ordered_console_entries_missing",
    });
  }
  const orderedRows = entries.map((entry, row) => {
    const text = String(entry?.text ?? "");
    return {
      row,
      line_no: Number.isFinite(Number(entry?.line_no)) ? Math.trunc(Number(entry.line_no)) : row + 1,
      tick: Number.isFinite(Number(entry?.tick)) ? Math.trunc(Number(entry.tick)) : null,
      output_identity: String(entry?.output_identity ?? ""),
      text,
    };
  });
  const cols = orderedRows.reduce((max, row) => Math.max(max, [...row.text].length), 0);
  const cells = orderedRows.flatMap((row) => [...row.text].map((text, col) => ({
    cell_id: `console:${row.row}:${col}`,
    row: row.row,
    col,
    text,
    source_line_no: row.line_no,
    source_output_identity: row.output_identity,
  })));
  return canonicalOutcome("console-grid", "available", identity, projection, {
    family_contract_version: "console_grid.v1_candidate",
    grid: {
      rows: orderedRows.length,
      cols,
      ordered_rows: orderedRows,
      cells,
    },
  });
}

function validGrid2dProjection(projection) {
  const grid = projection?.grid2d;
  if (!grid || typeof grid !== "object" || Array.isArray(grid)) {
    return { status: "invalid", reason: "grid2d_object_missing" };
  }
  const schema = String(grid.schema ?? "").trim();
  if (schema !== "seamgrim.nurimaker.grid_state.v1") {
    return {
      status: schema ? "unsupported" : "invalid",
      reason: schema ? "unsupported_grid2d_schema" : "grid2d_schema_missing",
      actual_schema: schema,
    };
  }
  const rows = Number(grid.gridH ?? grid.grid_h);
  const cols = Number(grid.gridW ?? grid.grid_w);
  if (!Number.isInteger(rows) || rows < 1 || !Number.isInteger(cols) || cols < 1) {
    return { status: "invalid", reason: "grid2d_dimensions_invalid", actual_schema: schema };
  }
  if (
    !Array.isArray(grid.tiles)
    || grid.tiles.length !== rows
    || grid.tiles.some((row) => !Array.isArray(row)
      || row.length !== cols
      || row.some((value) => !Number.isInteger(Number(value)) || Number(value) < 0))
  ) {
    return { status: "invalid", reason: "grid2d_tiles_invalid", actual_schema: schema };
  }
  const actors = Array.isArray(projection?.actors) ? projection.actors : [];
  const actorIds = actors.map((actor) => String(actor?.id ?? "").trim());
  const actorInvalid = actors.some((actor, index) => {
    const row = Number(actor?.row);
    const col = Number(actor?.col);
    return !actorIds[index]
      || !Number.isInteger(row)
      || !Number.isInteger(col)
      || row < 0
      || col < 0
      || row >= rows
      || col >= cols;
  });
  if (actorInvalid || new Set(actorIds).size !== actorIds.length) {
    return { status: "invalid", reason: "grid2d_actor_identity_or_position_invalid", actual_schema: schema };
  }
  return { status: "available", grid, actors, rows, cols, actual_schema: schema };
}

export function buildCanonicalGrid2dViewPayload({ identity = {}, projection = null } = {}) {
  const sourceProjection = {
    provenance: String(projection?.provenance ?? "native_authoritative_structured_input"),
    source: "canonical_grid2d_authoritative_state",
    classification: "AUTHORITATIVE_STRUCTURED",
  };
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("grid2d", "unavailable", identity, sourceProjection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  const declaredStatusInfo = canonicalProjectionStatus(
    projection?.status,
    projection ? "available" : "no_view",
  );
  const declaredStatus = declaredStatusInfo.status;
  if (declaredStatus !== "available") {
    if (!declaredStatusInfo.accepted) {
      return canonicalOutcome("grid2d", "invalid", identity, sourceProjection, null,
        unknownProjectionStatusDiagnostic("E_CANONICAL_GRID2D_INVALID_CANDIDATE", declaredStatusInfo));
    }
    const reason = String(projection?.reason ?? "").trim();
    const diagnostic = projection?.diagnostic ?? (
      reason && (declaredStatus === "unsupported" || declaredStatus === "invalid")
        ? {
            code: declaredStatus === "unsupported"
              ? "E_CANONICAL_GRID2D_UNSUPPORTED_CANDIDATE"
              : "E_CANONICAL_GRID2D_INVALID_CANDIDATE",
            reason,
            actual_schema: String(projection?.actual_schema ?? ""),
          }
        : null
    );
    return canonicalOutcome("grid2d", declaredStatus, identity, sourceProjection, null, diagnostic);
  }
  const inspected = validGrid2dProjection(projection);
  if (inspected.status !== "available") {
    return canonicalOutcome("grid2d", inspected.status, identity, sourceProjection, null, {
      code: inspected.status === "unsupported"
        ? "E_CANONICAL_GRID2D_UNSUPPORTED_CANDIDATE"
        : "E_CANONICAL_GRID2D_INVALID_CANDIDATE",
      reason: inspected.reason,
      actual_schema: inspected.actual_schema ?? "",
    });
  }
  return canonicalOutcome("grid2d", "available", identity, sourceProjection, {
    family_contract_version: "grid2d.v1_candidate",
    grid: {
      rows: inspected.rows,
      cols: inspected.cols,
      tiles: inspected.grid.tiles.map((row) => row.map((value) => Math.trunc(Number(value)))),
      actors: inspected.actors.map((actor) => ({
        actor_id: String(actor.id),
        row: Math.trunc(Number(actor.row)),
        col: Math.trunc(Number(actor.col)),
        symbol: String(actor.label ?? String(actor.id).slice(0, 1).toUpperCase()),
      })),
    },
  });
}

function finiteNumberField(value) {
  return typeof value === "number" && Number.isFinite(value);
}

function finitePoint(value) {
  return Boolean(value
    && typeof value === "object"
    && !Array.isArray(value)
    && finiteNumberField(value.x)
    && finiteNumberField(value.y));
}

function optionalFiniteNumber(value) {
  return value === null || finiteNumberField(value);
}

function inspectWorkbenchResultContent(family, content) {
  if (!content || typeof content !== "object" || Array.isArray(content)) {
    return { ok: false, reason: "content_object_missing" };
  }
  if (family === "line") {
    return finitePoint(content.current)
      && content.equation
      && finiteNumberField(content.equation.a)
      && finiteNumberField(content.equation.b)
      ? { ok: true }
      : { ok: false, reason: "line_current_or_equation_invalid" };
  }
  if (family === "pendulum") {
    const current = content.current;
    return current
      && finiteNumberField(current.theta)
      && finiteNumberField(current.omega)
      && finiteNumberField(current.length)
      && current.length > 0
      && optionalFiniteNumber(current.energy)
      && optionalFiniteNumber(current.kinetic_energy)
      && optionalFiniteNumber(current.potential_energy)
      && finitePoint(current.physical_bob)
      && finitePoint(current.normalized_bob)
      ? { ok: true }
      : { ok: false, reason: "pendulum_current_invalid" };
  }
  if (family === "projectile") {
    const current = content.current;
    return current
      && [current.t, current.x, current.y, current.vx, current.vy].every(finiteNumberField)
      ? { ok: true }
      : { ok: false, reason: "projectile_current_invalid" };
  }
  if (family === "grid") {
    const space2d = content.space2d;
    const dimensions = content.dimensions;
    const dimensionsValid = dimensions === null || (dimensions
      && finiteNumberField(dimensions.columns)
      && dimensions.columns > 0
      && finiteNumberField(dimensions.rows)
      && dimensions.rows > 0);
    return finitePoint(content.position)
      && dimensionsValid
      && space2d
      && space2d.schema === "seamgrim.space2d.v0"
      && Array.isArray(space2d.drawlist)
      && space2d.drawlist.length > 0
      ? { ok: true }
      : { ok: false, reason: "grid_position_or_space2d_invalid" };
  }
  return { ok: false, reason: "unknown_workbench_family" };
}

export function buildCanonicalWorkbenchViewPayload({
  identity = {},
  projection = null,
  family,
} = {}) {
  const canonicalFamily = String(family ?? "");
  const payloadFamily = `workbench-${canonicalFamily}`;
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome(payloadFamily, "unavailable", identity, projection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  const declaredStatusInfo = canonicalProjectionStatus(projection?.status);
  const declaredStatus = declaredStatusInfo.status;
  if (declaredStatus !== "available") {
    const diagnostic = declaredStatusInfo.accepted
      ? projection?.diagnostic ?? null
      : unknownProjectionStatusDiagnostic("E_CANONICAL_WORKBENCH_RESULT_INVALID_CANDIDATE", declaredStatusInfo);
    return canonicalOutcome(payloadFamily, declaredStatus, identity, projection, null, diagnostic);
  }
  const resultSet = projection?.result_set;
  const actualSetSchema = String(resultSet?.schema ?? "");
  if (actualSetSchema !== WORKBENCH_RESULT_SET_SCHEMA) {
    const status = actualSetSchema ? "unsupported" : "invalid";
    return canonicalOutcome(payloadFamily, status, identity, projection, null, {
      code: status === "unsupported"
        ? "E_CANONICAL_WORKBENCH_RESULT_SET_UNSUPPORTED_CANDIDATE"
        : "E_CANONICAL_WORKBENCH_RESULT_SET_INVALID_CANDIDATE",
      reason: actualSetSchema ? "unsupported_result_set_schema" : "result_set_schema_missing",
      actual_schema: actualSetSchema,
    });
  }
  if (
    resultSet.producer !== "ddonirang_tool.wasm_runtime"
    || resultSet.provenance !== "native_authoritative_structured_result"
    || !resultSet.results
    || typeof resultSet.results !== "object"
    || Array.isArray(resultSet.results)
  ) {
    return canonicalOutcome(payloadFamily, "invalid", identity, projection, null, {
      code: "E_CANONICAL_WORKBENCH_RESULT_SET_INVALID_CANDIDATE",
      reason: "result_set_producer_provenance_or_results_invalid",
    });
  }
  const result = resultSet.results[canonicalFamily];
  if (result === undefined || result === null) {
    return canonicalOutcome(payloadFamily, "no_view", identity, projection, null, null);
  }
  const actualSchema = String(result?.schema ?? "");
  const expectedSchema = WORKBENCH_RESULT_SCHEMAS[canonicalFamily] ?? "";
  if (actualSchema !== expectedSchema) {
    const status = actualSchema ? "unsupported" : "invalid";
    return canonicalOutcome(payloadFamily, status, identity, projection, null, {
      code: status === "unsupported"
        ? "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE"
        : "E_CANONICAL_WORKBENCH_RESULT_INVALID_CANDIDATE",
      reason: actualSchema ? "unsupported_family_result_schema" : "family_result_schema_missing",
      actual_schema: actualSchema,
    });
  }
  if (
    result.family !== canonicalFamily
    || result.status !== "available"
    || result.producer !== "ddonirang_tool.wasm_runtime"
    || result.provenance !== "native_authoritative_structured_result"
  ) {
    return canonicalOutcome(payloadFamily, "invalid", identity, projection, null, {
      code: "E_CANONICAL_WORKBENCH_RESULT_INVALID_CANDIDATE",
      reason: "family_status_producer_or_provenance_invalid",
    });
  }
  const inspected = inspectWorkbenchResultContent(canonicalFamily, result.content);
  if (!inspected.ok) {
    return canonicalOutcome(payloadFamily, "invalid", identity, projection, null, {
      code: "E_CANONICAL_WORKBENCH_RESULT_INVALID_CANDIDATE",
      reason: inspected.reason,
    });
  }
  return canonicalOutcome(payloadFamily, "available", identity, projection, {
    family_contract_version: `${canonicalFamily}.v1_candidate`,
    // Keep the already-validated typed producer identity with the canonical
    // result so the renderer can reject a replay under another provider.
    provider_identity: result.producer,
    provider_provenance: result.provenance,
    result: result.content,
  });
}

function canonicalProjectionDiagnostic({ family, status, reason, actualSchema = "" }) {
  if (!reason || !["invalid", "unsupported", "ambiguous"].includes(status)) return null;
  const prefix = family === "scene-summary"
    ? "E_CANONICAL_SCENE_SUMMARY"
    : "E_CANONICAL_CURRICULUM_SCENE";
  return {
    code: status === "unsupported"
      ? `${prefix}_UNSUPPORTED_CANDIDATE`
      : status === "ambiguous"
        ? `${prefix}_AMBIGUOUS_CANDIDATE`
        : `${prefix}_INVALID_CANDIDATE`,
    reason,
    actual_schema: actualSchema,
  };
}

function canonicalPayloadIdentityMatches(identity, payload) {
  return stableSemanticJson(normalizeExistingViewIdentity(identity))
    === stableSemanticJson(normalizeExistingViewIdentity(payload?.identity));
}

function inspectNestedCanonicalPayload(payload, expectedFamily) {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    return { ok: false, status: "invalid", reason: "canonical_primary_payload_missing" };
  }
  if (payload.schema !== CANONICAL_VIEW_PAYLOAD_SCHEMA) {
    return {
      ok: false,
      status: payload.schema ? "unsupported" : "invalid",
      reason: payload.schema ? "canonical_primary_payload_schema_unsupported" : "canonical_primary_payload_schema_missing",
      actual_schema: String(payload.schema ?? ""),
    };
  }
  if (payload.family !== expectedFamily) {
    return { ok: false, status: "invalid", reason: "canonical_primary_payload_family_mismatch" };
  }
  if (payload.status !== "available" || payload.content === null) {
    return { ok: false, status: String(payload.status ?? "invalid"), reason: "canonical_primary_payload_not_available" };
  }
  if (!String(payload.candidate_payload_hash ?? "").trim()) {
    return { ok: false, status: "invalid", reason: "canonical_primary_payload_hash_missing" };
  }
  return { ok: true };
}

export function buildCanonicalSceneSummaryViewPayload({ identity = {}, projection = null } = {}) {
  const sourceProjection = {
    provenance: String(projection?.provenance ?? "canonical_view_payload_projection"),
    source: String(projection?.source ?? "scene_summary_semantic_projection"),
    classification: "AUTHORITATIVE_STRUCTURED",
  };
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("scene-summary", "unavailable", identity, sourceProjection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (!projection || typeof projection !== "object" || Array.isArray(projection)) {
    return canonicalOutcome("scene-summary", "no_view", identity, sourceProjection, null, null);
  }
  const actualSchema = String(projection.schema ?? "");
  if (actualSchema !== SCENE_SUMMARY_SEMANTIC_PROJECTION_SCHEMA) {
    const status = actualSchema ? "unsupported" : "invalid";
    return canonicalOutcome("scene-summary", status, identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "scene-summary",
      status,
      reason: actualSchema ? "scene_projection_schema_unsupported" : "scene_projection_schema_missing",
      actualSchema,
    }));
  }
  const statusInfo = canonicalProjectionStatus(projection.status, "invalid");
  const status = statusInfo.status;
  if (status !== "available") {
    return canonicalOutcome("scene-summary", status, identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "scene-summary",
      status,
      reason: statusInfo.accepted
        ? String(projection.reason ?? "")
        : "scene_projection_status_invalid",
      actualSchema,
    }));
  }
  const primaryFamily = String(projection.primary_family ?? "").trim();
  const requiredFamilies = Array.isArray(projection.required_families)
    ? projection.required_families.map((family) => String(family ?? "").trim()).filter(Boolean)
    : null;
  if (!primaryFamily || !requiredFamilies || new Set(requiredFamilies).size !== requiredFamilies.length) {
    return canonicalOutcome("scene-summary", "invalid", identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "scene-summary",
      status: "invalid",
      reason: "scene_primary_or_required_family_invalid",
      actualSchema,
    }));
  }
  const inspected = inspectNestedCanonicalPayload(projection.primary_payload, primaryFamily);
  if (!inspected.ok) {
    const nestedStatus = ["no_view", "unavailable", "invalid", "unsupported", "ambiguous"].includes(inspected.status)
      ? inspected.status
      : "invalid";
    return canonicalOutcome("scene-summary", nestedStatus, identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "scene-summary",
      status: nestedStatus,
      reason: inspected.reason,
      actualSchema: inspected.actual_schema ?? actualSchema,
    }));
  }
  if (!canonicalPayloadIdentityMatches(identity, projection.primary_payload)) {
    return canonicalOutcome("scene-summary", "invalid", identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "scene-summary",
      status: "invalid",
      reason: "scene_primary_payload_identity_mismatch",
      actualSchema,
    }));
  }
  return canonicalOutcome("scene-summary", "available", identity, sourceProjection, {
    family_contract_version: "scene_summary.v1_candidate",
    scene_summary: {
      primary_family: primaryFamily,
      required_families: requiredFamilies,
      primary_payload: projection.primary_payload,
    },
  });
}

function inspectCurriculumAnchors(anchors) {
  if (!Array.isArray(anchors)) return { ok: false, reason: "curriculum_anchors_missing" };
  const ids = anchors.map((anchor) => String(anchor?.anchor_id ?? "").trim());
  if (ids.some((id) => !id) || new Set(ids).size !== ids.length) {
    return { ok: false, reason: "curriculum_anchor_identity_invalid" };
  }
  const valid = anchors.every((anchor) => {
    const kind = String(anchor?.kind ?? "");
    const name = String(anchor?.name ?? "").trim();
    const source = String(anchor?.binding?.source ?? "");
    return ["control", "observation"].includes(kind)
      && Boolean(name)
      && ["slider_panel", "committed_observation_payload"].includes(source)
      && typeof anchor?.binding?.resolved === "boolean"
      && !Object.prototype.hasOwnProperty.call(anchor, "target_selector")
      && !Object.prototype.hasOwnProperty.call(anchor, "a11y_label");
  });
  return valid ? { ok: true } : { ok: false, reason: "curriculum_anchor_contract_invalid" };
}

export function buildCanonicalCurriculumSemanticSceneViewPayload({ identity = {}, projection = null } = {}) {
  const sourceProjection = {
    provenance: String(projection?.provenance ?? "validated_curriculum_card_and_committed_bindings"),
    source: String(projection?.source ?? "curriculum_semantic_scene_projection"),
    classification: "AUTHORITATIVE_STRUCTURED",
  };
  if (!requiredIdentityAvailable(identity)) {
    return canonicalOutcome("curriculum-semantic-scene", "unavailable", identity, sourceProjection, null, {
      code: "E_CANONICAL_VIEW_REQUIRED_IDENTITY_MISSING_CANDIDATE",
      reason: "state_or_result_identity_missing",
    });
  }
  if (!projection || typeof projection !== "object" || Array.isArray(projection)) {
    return canonicalOutcome("curriculum-semantic-scene", "no_view", identity, sourceProjection, null, null);
  }
  const actualSchema = String(projection.schema ?? "");
  if (actualSchema !== CURRICULUM_SEMANTIC_SCENE_PROJECTION_SCHEMA) {
    const status = actualSchema ? "unsupported" : "invalid";
    return canonicalOutcome("curriculum-semantic-scene", status, identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "curriculum-semantic-scene",
      status,
      reason: actualSchema ? "curriculum_projection_schema_unsupported" : "curriculum_projection_schema_missing",
      actualSchema,
    }));
  }
  const statusInfo = canonicalProjectionStatus(projection.status, "invalid");
  const status = statusInfo.status;
  if (status !== "available") {
    return canonicalOutcome("curriculum-semantic-scene", status, identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "curriculum-semantic-scene",
      status,
      reason: statusInfo.accepted
        ? String(projection.reason ?? "")
        : "curriculum_projection_status_invalid",
      actualSchema,
    }));
  }
  const sceneId = String(projection.scene_id ?? "").trim();
  const lessonId = String(projection.lesson_id ?? "").trim();
  const worldId = String(projection.world_id ?? "").trim();
  const primaryView = String(projection.primary_view ?? "").trim();
  const primaryFamily = String(projection.primary_view_family ?? "").trim();
  if (!sceneId || !lessonId || !worldId || !primaryView || !primaryFamily) {
    return canonicalOutcome("curriculum-semantic-scene", "invalid", identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "curriculum-semantic-scene",
      status: "invalid",
      reason: "curriculum_scene_or_primary_identity_missing",
      actualSchema,
    }));
  }
  const inspectedPrimary = inspectNestedCanonicalPayload(projection.primary_view_payload, primaryFamily);
  if (!inspectedPrimary.ok) {
    const nestedStatus = ["no_view", "unavailable", "invalid", "unsupported", "ambiguous"].includes(inspectedPrimary.status)
      ? inspectedPrimary.status
      : "invalid";
    return canonicalOutcome("curriculum-semantic-scene", nestedStatus, identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "curriculum-semantic-scene",
      status: nestedStatus,
      reason: inspectedPrimary.reason,
      actualSchema: inspectedPrimary.actual_schema ?? actualSchema,
    }));
  }
  if (!canonicalPayloadIdentityMatches(identity, projection.primary_view_payload)) {
    return canonicalOutcome("curriculum-semantic-scene", "invalid", identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "curriculum-semantic-scene",
      status: "invalid",
      reason: "curriculum_primary_payload_identity_mismatch",
      actualSchema,
    }));
  }
  const inspectedAnchors = inspectCurriculumAnchors(projection.anchors);
  if (!inspectedAnchors.ok) {
    return canonicalOutcome("curriculum-semantic-scene", "invalid", identity, sourceProjection, null, canonicalProjectionDiagnostic({
      family: "curriculum-semantic-scene",
      status: "invalid",
      reason: inspectedAnchors.reason,
      actualSchema,
    }));
  }
  const anchors = projection.anchors.map((anchor) => ({
    anchor_id: String(anchor.anchor_id),
    kind: String(anchor.kind),
    name: String(anchor.name),
    binding: canonicalValue(anchor.binding),
  }));
  const resolvedCount = anchors.filter((anchor) => anchor.binding.resolved === true).length;
  const conceptLabels = Array.isArray(projection.concept_labels)
    ? projection.concept_labels.map((label) => String(label ?? "").trim()).filter(Boolean)
    : [];
  return canonicalOutcome("curriculum-semantic-scene", "available", identity, sourceProjection, {
    family_contract_version: "curriculum_semantic_scene.v1_candidate",
    curriculum_semantic_scene: {
      scene_id: sceneId,
      lesson_id: lessonId,
      world_id: worldId,
      primary_view: primaryView,
      primary_view_family: primaryFamily,
      primary_view_payload: projection.primary_view_payload,
      concept_labels: conceptLabels,
      anchors,
      runtime_binding: {
        expected_count: anchors.length,
        resolved_count: resolvedCount,
        complete: anchors.length > 0 && resolvedCount === anchors.length,
        observation_source: "seamgrim.observation_payload.v1",
        control_source: "slider_panel",
      },
    },
  });
}

export function buildCanonicalViewPayloadSet({
  identity = {},
  console = null,
  graph = null,
  space2d = null,
  table = null,
  text = null,
  structure = null,
  grid2d = null,
  workbench = null,
  sceneSummary = null,
  curriculumSemanticScene = null,
} = {}) {
  const consolePayload = buildCanonicalConsoleViewPayload({ identity, projection: console });
  return {
    console: consolePayload,
    console_grid: buildCanonicalConsoleGridViewPayload({ identity, consolePayload }),
    graph: buildCanonicalGraphViewPayload({ identity, projection: graph }),
    space2d: buildCanonicalSpace2dViewPayload({ identity, projection: space2d }),
    table: buildCanonicalTableViewPayload({ identity, projection: table }),
    text: buildCanonicalTextViewPayload({ identity, projection: text }),
    structure: buildCanonicalStructureViewPayload({ identity, projection: structure }),
    grid2d: buildCanonicalGrid2dViewPayload({ identity, projection: grid2d }),
    workbench_line: buildCanonicalWorkbenchViewPayload({ identity, projection: workbench, family: "line" }),
    workbench_pendulum: buildCanonicalWorkbenchViewPayload({ identity, projection: workbench, family: "pendulum" }),
    workbench_projectile: buildCanonicalWorkbenchViewPayload({ identity, projection: workbench, family: "projectile" }),
    workbench_grid: buildCanonicalWorkbenchViewPayload({ identity, projection: workbench, family: "grid" }),
    scene_summary: buildCanonicalSceneSummaryViewPayload({ identity, projection: sceneSummary }),
    curriculum_semantic_scene: buildCanonicalCurriculumSemanticSceneViewPayload({ identity, projection: curriculumSemanticScene }),
  };
}

function valueText(value) {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return stableSemanticJson(value);
}

export function orderedObservationsFromWasmState(wasmState = {}) {
  const channels = Array.isArray(wasmState?.channels) ? wasmState.channels : [];
  const row = Array.isArray(wasmState?.row) ? wasmState.row : [];
  const resourceValues = wasmState?.resources?.value_json && typeof wasmState.resources.value_json === "object"
    ? wasmState.resources.value_json
    : {};
  return channels.map((channel, index) => {
    const name = String(channel?.key ?? channel?.name ?? `channel_${index + 1}`);
    const value = index < row.length ? row[index] : resourceValues[name];
    return {
      index,
      observation_id: `runtime.${name}`,
      name,
      role: String(channel?.role ?? "state"),
      dtype: String(channel?.dtype ?? typeof value),
      value,
      value_text: valueText(value),
      unit: String(channel?.unit ?? ""),
      state_hash_participation: true,
    };
  });
}

export function committedResourcesFromWasmState(wasmState = {}) {
  const values = wasmState?.resources?.value_json;
  return values && typeof values === "object" && !Array.isArray(values)
    ? canonicalValue(values)
    : {};
}

function resolveViewBinding(primaryView, canonicalViewPayloads) {
  const canonicalKey = `workbench_${primaryView}`;
  const canonicalPayload = canonicalViewPayloads?.[canonicalKey];
  return {
    schema: "seamgrim.workbench.semantic_view_binding.v1",
    view_family: primaryView,
    requested_channels: [],
    resolved_channels: [],
    resolved_resources: [],
    missing_channels: [],
    canonical_payload_key: canonicalKey,
    canonical_payload_status: String(canonicalPayload?.status ?? "unavailable"),
    resolution: canonicalPayload?.status === "available"
      ? "canonical_workbench_result"
      : "canonical_workbench_result_unavailable",
  };
}

function frameIdentity(run, frame, project) {
  if (!run || !frame) return null;
  return {
    run_id: String(run.run_id ?? ""),
    // `project.id` is the existing Local Workspace identity.  Carry it with
    // the committed frame so a projection cannot be replayed into another
    // workspace that happens to have the same source/state hashes.
    workspace_identity: String(project?.id ?? "").trim(),
    frame_id: Number(frame.frame_id ?? -1),
    madi: Number(frame.madi ?? 0),
    state_hash: String(frame.state_hash ?? ""),
    source_hash: String(frame.source_hash ?? run.source_hash ?? ""),
  };
}

function executionReceiptBinding(run, frame) {
  const summary = run?.shared_runtime_result;
  const receipt = run?.work_cost_receipt;
  if (!summary || typeof summary !== "object" || !receipt || typeof receipt !== "object") return null;
  const frameIndex = run.frames?.indexOf(frame) ?? -1;
  const lineage = frameIndex >= 0 ? run.frames.slice(0, frameIndex + 1).map((row) => ({
    frame_id: Number(row.frame_id), madi: Number(row.madi),
    state_hash: String(row.state_hash ?? ""), source_hash: String(row.source_hash ?? run.source_hash ?? ""),
  })) : [];
  return {
    summary_schema: String(summary.schema ?? ""),
    source_profile_identity: String(summary.source_profile_identity ?? ""),
    source_identity: String(summary.source_identity ?? ""),
    state_hash: String(summary.state_hash ?? ""),
    work_cost_receipt: canonicalValue(receipt),
    run_id: String(run.run_id ?? ""),
    committed_lineage: lineage,
  };
}

function canonicalHistoryRows(primaryView, canonicalViewHistory) {
  const key = `workbench_${primaryView}`;
  if (!Array.isArray(canonicalViewHistory)) return [];
  return canonicalViewHistory.flatMap((row) => {
    const payload = row?.canonical_views?.[key];
    if (payload?.status !== "available") return [];
    const result = payload?.content?.result;
    const current = result?.current;
    if (!current || typeof current !== "object" || Array.isArray(current)) return [];
    const values = primaryView === "line" && result?.equation
      ? { ...current, ...result.equation }
      : { ...current };
    return [{
      frame_id: Number(row?.frame_id ?? -1),
      madi: Number(row?.madi ?? 0),
      state_hash: String(row?.state_hash ?? ""),
      values: canonicalValue(values),
      authority: key,
    }];
  });
}

function explicitPrimaryView(viewConfig, project) {
  const configured = String(viewConfig?.primary_view ?? "").trim();
  if (configured) return configured;
  return String(project?.primaryView ?? "").trim();
}

export function buildCommittedSemanticViewPayload({
  project = {},
  run = null,
  frame = null,
  viewConfig = {},
  canonicalViewPayloads = null,
  canonicalViewHistory = null,
} = {}) {
  const primaryView = explicitPrimaryView(viewConfig, project);
  const observations = frame ? orderedObservationsFromWasmState(frame.wasm_state) : [];
  const committedResources = frame ? committedResourcesFromWasmState(frame.wasm_state) : {};
  const binding = resolveViewBinding(primaryView, canonicalViewPayloads);
  const payload = {
    schema: SEMANTIC_VIEW_PAYLOAD_SCHEMA,
    committed_frame: frameIdentity(run, frame, project),
    execution_receipt: executionReceiptBinding(run, frame),
    ordered_observations: observations,
    committed_resources: committedResources,
    committed_view_meta: frame?.wasm_state?.view_meta && typeof frame.wasm_state.view_meta === "object"
      ? canonicalValue(frame.wasm_state.view_meta)
      : {},
    view_binding: binding,
    committed_trajectory: canonicalHistoryRows(primaryView, canonicalViewHistory),
    canonical_views: canonicalViewPayloads && typeof canonicalViewPayloads === "object"
      ? canonicalValue(canonicalViewPayloads)
      : null,
    canonical_view_history: Array.isArray(canonicalViewHistory)
      ? canonicalValue(canonicalViewHistory)
      : [],
    view_config: {
      primary_view: primaryView,
      role: String(viewConfig.role ?? "primary"),
      mode: String(viewConfig.mode ?? "ddn-main"),
      view_instance_id: String(viewConfig.view_instance_id ?? ""),
      target_view_instance_id: String(viewConfig.target_view_instance_id ?? ""),
      overlay_instance_ids: Array.isArray(viewConfig.overlay_instance_ids)
        ? [...viewConfig.overlay_instance_ids].map(String).sort()
        : [],
      overlay_enabled: Boolean(viewConfig.overlay_enabled),
      focus_target: viewConfig.focus_target && typeof viewConfig.focus_target === "object"
        ? canonicalValue(viewConfig.focus_target)
        : null,
      participates_in_state_hash: false,
    },
  };
  return {
    ...payload,
    semantic_payload_hash: semanticViewPayloadHash(payload),
  };
}
