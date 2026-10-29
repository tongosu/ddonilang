import {
  buildDiagnosticFixitIntegration,
  formatDiagnosticFixitIntegrationText,
} from "./studio_diagnostic_fixit_integration.js";

function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const DEFAULT_SOURCE_TEXT = [
  "채비: {",
  "  y <- (1 + 2",
  "}.",
  "#이름: 낡은_진단_예제",
].join("\n");

export const DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_DIAGNOSTICS = [
  {
    technical_code: "E_BLOCK_HEADER_COLON_FORBIDDEN",
    message: "블록 헤더에는 ':'를 쓰지 않습니다.",
    span: { line: 1, column: 5 },
  },
  {
    technical_code: "E_PARSE_EXPECTED_RPAREN",
    message: "닫는 괄호가 필요합니다.",
    span: { line: 2, column: 14 },
  },
  {
    technical_code: "E_IMPORT_ALIAS_DUPLICATE",
    message: "import 별칭이 중복되었습니다.",
    span: { line: 3, column: 1 },
  },
];

export const DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS = [
  { id: "diagnostic_viewer", title: "Diagnostic viewer", ready: true },
  { id: "fixit_preview", title: "Fix-it preview", ready: true },
  { id: "lsp_contract", title: "LSP-lite contract", ready: true },
  { id: "boundary_guard", title: "Boundary guard", ready: true },
];

function normalizeRows(rows = DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      ready: source.ready !== false,
      local_only: true,
      preview_only: true,
    };
  });
}

function buildEvidenceText(rows, workflow) {
  return [
    "toolchain_diagnostic_ui_lsp:toolchain_pack_3_v1",
    "coordinate:타-3",
    "full_lsp_server:false",
    "lsp_protocol_change:false",
    "file_write:false",
    `diagnostic_count:${workflow.diagnostic_count ?? 0}`,
    `fixit_count:${workflow.fixit_count ?? 0}`,
    ...rows.map((row) => `${row.id}\t${row.ready === true ? "ready" : "missing"}`),
  ].join("\n");
}

export function buildToolchainDiagnosticUiLsp({
  rows = DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS,
  sourceText = DEFAULT_SOURCE_TEXT,
  diagnostics = DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_DIAGNOSTICS,
  activeRowId = "diagnostic_viewer",
} = {}) {
  const normalizedRows = normalizeRows(rows);
  const workflow = buildDiagnosticFixitIntegration({
    sourceText,
    diagnostics,
    boundary: {
      auto_apply: false,
      file_write: false,
      lsp_protocol_change: false,
      parser_frontdoor_change: false,
      runtime_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
    },
  });
  const readyRowCount = normalizedRows.filter((row) => row.ready).length;
  const ready = readyRowCount === normalizedRows.length && workflow.status === "diagnostic_fixit_ready";
  const active = normalizedRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : normalizedRows[0]?.id ?? "";
  return {
    __종류: "toolchain_diagnostic_ui_lsp",
    schema: "ddn.toolchain.diagnostic_ui_lsp.v1",
    work_item: "TA3_DIAGNOSTIC_UI_LSP_V1",
    primary_coordinate: "타-3",
    depends_on_coordinate: ["타-2"],
    pack: "toolchain_pack_3_v1",
    status: ready ? "toolchain_diagnostic_ui_lsp_ready" : "toolchain_diagnostic_ui_lsp_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    diagnostic_viewer_claim: workflow.diagnostic_count > 0,
    fixit_preview_claim: workflow.fixit_count > 0,
    lsp_lite_contract_claim: true,
    full_lsp_server_claim: false,
    lsp_protocol_change: false,
    auto_apply_claim: false,
    file_write_claim: false,
    active_allowlist_mutation: false,
    rows: normalizedRows,
    active_row_id: active,
    source_text: String(sourceText ?? ""),
    workflow,
    formatted_workflow_text: formatDiagnosticFixitIntegrationText(workflow),
    evidence_text: buildEvidenceText(normalizedRows, workflow),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 22,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 24,
      roadmap_v2_pack_evidence_reference_closed: 42,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 47,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "TA4_REGISTRY_VERIFICATION_V1",
  };
}

export function formatToolchainDiagnosticUiLspText(diagnosticUi = {}) {
  const payload = asObject(diagnosticUi);
  if (payload.schema !== "ddn.toolchain.diagnostic_ui_lsp.v1") {
    throw new Error("toolchain_expected_diagnostic_ui_lsp");
  }
  const progress = asObject(payload.progress);
  const workflow = asObject(payload.workflow);
  const rows = asArray(payload.rows);
  return [
    `schema\t${payload.schema ?? ""}`,
    `work_item\t${payload.work_item ?? ""}`,
    `primary_coordinate\t${payload.primary_coordinate ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `matrix_closure_tier\t${payload.matrix_closure_tier ?? ""}`,
    `current_stage\t${progress.current_stage_closed ?? 0}/${progress.current_stage_total ?? 0}`,
    `current_stage_percent\t${progress.current_stage_percent ?? 0}`,
    `roadmap_matrix\t${progress.roadmap_v2_matrix_behavior_closed ?? 0}/${progress.roadmap_v2_matrix_behavior_total ?? 0}`,
    `roadmap_matrix_percent\t${progress.roadmap_v2_matrix_behavior_percent ?? 0}`,
    `pack_evidence_reference\t${progress.roadmap_v2_pack_evidence_reference_closed ?? 0}/${progress.roadmap_v2_pack_evidence_reference_total ?? 0}`,
    `pack_evidence_reference_percent\t${progress.roadmap_v2_pack_evidence_reference_percent ?? 0}`,
    `studio_local_super_long\t${progress.studio_local_super_long_closed ?? 0}/${progress.studio_local_super_long_total ?? 0}`,
    `studio_local_super_long_percent\t${progress.studio_local_super_long_percent ?? 0}`,
    `diagnostic_viewer_claim\t${payload.diagnostic_viewer_claim === true ? "true" : "false"}`,
    `fixit_preview_claim\t${payload.fixit_preview_claim === true ? "true" : "false"}`,
    `lsp_lite_contract_claim\t${payload.lsp_lite_contract_claim === true ? "true" : "false"}`,
    `full_lsp_server_claim\t${payload.full_lsp_server_claim === true ? "true" : "false"}`,
    `lsp_protocol_change\t${payload.lsp_protocol_change === true ? "true" : "false"}`,
    `auto_apply_claim\t${payload.auto_apply_claim === true ? "true" : "false"}`,
    `file_write_claim\t${payload.file_write_claim === true ? "true" : "false"}`,
    `diagnostic_count\t${workflow.diagnostic_count ?? 0}`,
    `fixit_count\t${workflow.fixit_count ?? 0}`,
    "",
    "row_id\tready",
    ...rows.map((row) => `${row.id}\t${row.ready === true ? "true" : "false"}`),
    "",
    String(payload.formatted_workflow_text ?? ""),
  ].join("\n");
}

export function renderToolchainDiagnosticUiLsp(root, diagnosticUi = {}) {
  if (!root) return null;
  const payload = asObject(diagnosticUi);
  const progress = asObject(payload.progress);
  const workflow = asObject(payload.workflow);
  const preview = asObject(workflow.preview);
  const rows = asArray(payload.rows);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.toolchainDiagnosticUiLspStatus = asText(payload.status, "toolchain_diagnostic_ui_lsp_incomplete");
  root.innerHTML = `
    <div class="toolchain-diagnostic-head">
      <div>
        <div class="toolchain-diagnostic-kicker">Toolchain diagnostics</div>
        <h2>Diagnostic UI / LSP-lite</h2>
      </div>
      <div class="toolchain-diagnostic-progress" data-toolchain-diagnostic-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="toolchain-diagnostic-summary" data-toolchain-diagnostic-summary>
      기존 Studio 진단/fix-it preview를 제품 UI에 연결하고, full LSP server, protocol change, file write, auto-apply는 후속으로 둡니다.
    </div>
    <div class="toolchain-diagnostic-body">
      <div class="toolchain-diagnostic-list" data-toolchain-diagnostic-list>
        ${rows.map((row) => `
          <button type="button" class="toolchain-diagnostic-btn${row.id === activeId ? " active" : ""}" data-toolchain-diagnostic-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${row.ready === true ? "ready" : "missing"}</small>
          </button>
        `).join("")}
      </div>
      <div class="toolchain-diagnostic-detail">
        <div class="toolchain-diagnostic-title" data-toolchain-diagnostic-active-title>${escapeHtml(active.title)}</div>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>diagnostics</dt><dd>${escapeHtml(String(workflow.diagnostic_count ?? 0))}</dd></div>
          <div><dt>fix-it</dt><dd>${escapeHtml(String(workflow.fixit_count ?? 0))}</dd></div>
        </dl>
        <pre class="toolchain-diagnostic-source" data-toolchain-diagnostic-source>${escapeHtml(payload.source_text ?? "")}</pre>
        <pre class="toolchain-diagnostic-diff" data-toolchain-diagnostic-diff>${escapeHtml(preview.diff_text ?? "")}</pre>
        <pre class="toolchain-diagnostic-preview" data-toolchain-diagnostic-preview>${escapeHtml(preview.preview_text ?? "")}</pre>
        <button type="button" class="ghost" data-toolchain-diagnostic-copy>diagnostic 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderToolchainDiagnosticUiLsp(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-toolchain-diagnostic-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-toolchain-diagnostic-row") || ""));
  });
  root.querySelector("[data-toolchain-diagnostic-copy]")?.addEventListener("click", async () => {
    root.dataset.toolchainDiagnosticUiLspCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatToolchainDiagnosticUiLspText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
