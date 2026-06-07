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

function asBool(value) {
  return value === true || value === "true" || value === "참";
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const ARTIFACT_DEFS = [
  {
    id: "studio-static-bundle",
    planned_path: "build/studio_release/studio-static-bundle.zip",
    artifact_kind: "static_bundle",
    title: "static bundle",
    summary: "Studio 정적 번들은 계획만 기록하며 archive 파일을 생성하지 않습니다.",
  },
  {
    id: "studio-local-package-sample",
    planned_path: "build/studio_release/studio-local-package-sample.detjson",
    artifact_kind: "local_package_sample",
    title: "local package sample",
    summary: "로컬 package sample은 dry-run row로만 남고 publication snapshot을 emit하지 않습니다.",
  },
  {
    id: "studio-rc-matrix",
    planned_path: "build/studio_release/studio-rc-matrix.detjson",
    artifact_kind: "release_candidate_matrix",
    title: "RC matrix",
    summary: "release candidate matrix는 계획 좌표만 고정하고 benchmark나 LTS 인증을 실행하지 않습니다.",
  },
  {
    id: "studio-checksum-manifest",
    planned_path: "build/studio_release/SHA256SUMS.txt",
    artifact_kind: "checksum_manifest",
    title: "checksum manifest",
    summary: "SHA256 manifest는 planned-only이며 signing은 explicit approval 이후로 잠급니다.",
  },
];

const CHECKSUM_POLICY = {
  algorithm: "sha256",
  manifest_path: "build/studio_release/SHA256SUMS.txt",
  ordering: "path_lexicographic",
  scope: "local_build_artifacts_only",
  signing: "excluded_v1_approval_gated",
  generated_now: false,
};

export const DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS = ARTIFACT_DEFS.map((row) => ({
  id: row.id,
  planned_path: row.planned_path,
  kind: row.artifact_kind,
  generated_now: false,
  product_ui_change: true,
  artifact_generation_claim: false,
  archive_generation_claim: false,
  publication_checksum_generation_claim: false,
  artifact_signing_claim: false,
}));

function normalizeRows(rows = DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS) {
  const rowById = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return ARTIFACT_DEFS.map((def) => {
    const source = rowById.get(def.id) || {};
    return {
      id: def.id,
      title: def.title,
      summary: def.summary,
      planned_path: asText(source.planned_path, def.planned_path),
      kind: asText(source.kind, def.artifact_kind),
      artifact_surface: "local_studio_publication_artifact_dry_run",
      generated_now: asBool(source.generated_now),
      artifact_generation_claim: false,
      archive_generation_claim: false,
      publication_checksum_generation_claim: false,
      artifact_signing_claim: false,
      release_approval_claim: false,
      release_execution_claim: false,
      public_release_claim: false,
      public_upload_claim: false,
      registry_publish_claim: false,
      github_release_claim: false,
      install_enablement_claim: false,
      publication_snapshot_emit_claim: false,
      cloud_sync_claim: false,
      runtime_claim: false,
      replay_claim: false,
      lesson_schema_change: false,
      active_allowlist_mutation: false,
      parser_frontdoor_change: false,
      benchmark_execution_claim: false,
      lts_certification_claim: false,
      product_ui_change: true,
    };
  });
}

export function buildPublicationArtifactDryRun({
  artifactRows = DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS,
  activeArtifactId = "studio-checksum-manifest",
} = {}) {
  const rows = normalizeRows(artifactRows);
  const active = rows.some((row) => row.id === activeArtifactId)
    ? activeArtifactId
    : rows[0]?.id ?? "";
  const dryRunOnly = rows.every((row) => (
    row.generated_now === false &&
    row.artifact_generation_claim === false &&
    row.archive_generation_claim === false &&
    row.publication_checksum_generation_claim === false &&
    row.artifact_signing_claim === false &&
    row.release_execution_claim === false &&
    row.public_upload_claim === false &&
    row.runtime_claim === false
  ));
  const stages = [
    ["artifact_row_alignment", rows.length === ARTIFACT_DEFS.length],
    ["static_bundle_planned", rows.some((row) => row.id === "studio-static-bundle")],
    ["local_package_sample_planned", rows.some((row) => row.id === "studio-local-package-sample")],
    ["rc_matrix_planned", rows.some((row) => row.id === "studio-rc-matrix")],
    ["checksum_manifest_planned", rows.some((row) => row.id === "studio-checksum-manifest")],
    ["generation_boundary_blocked", dryRunOnly],
  ].map(([stage_id, ready]) => ({ stage_id, ready: ready === true }));
  const ready_stage_count = stages.filter((stage) => stage.ready).length;
  return {
    __종류: "studio_publication_artifact_dry_run",
    schema: "ddn.studio.publication_artifact_dry_run.v1",
    work_item: "STUDIO_PUBLICATION_ARTIFACT_DRY_RUN_V1",
    based_on: "STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1",
    workflow_claim: "publication_artifact_dry_run",
    primary_coordinate: "타-3",
    support_coordinate: "마-3",
    next_state: "AWAIT_EXPLICIT_RELEASE_APPROVAL",
    product_ui_change: true,
    product_code_change: true,
    publication_artifact_dry_run_claim: true,
    publication_artifact_dry_run_only: true,
    runtime_claim: false,
    replay_claim: false,
    artifact_generation_claim: false,
    archive_generation_claim: false,
    publication_checksum_generation_claim: false,
    artifact_signing_claim: false,
    release_approval_claim: false,
    release_execution_claim: false,
    public_release_claim: false,
    public_upload_claim: false,
    registry_publish_claim: false,
    github_release_claim: false,
    install_enablement_claim: false,
    publication_snapshot_emit_claim: false,
    cloud_sync_claim: false,
    benchmark_execution_claim: false,
    lts_certification_claim: false,
    lesson_schema_change: false,
    active_allowlist_mutation: false,
    parser_frontdoor_change: false,
    checksum_policy: { ...CHECKSUM_POLICY },
    planned_artifact_count: rows.length,
    all_planned_artifacts_generated_now: false,
    status: ready_stage_count === stages.length ? "publication_artifact_dry_run_ready" : "publication_artifact_dry_run_incomplete",
    tone: ready_stage_count === stages.length ? "success" : "warning",
    artifact_row_count: rows.length,
    stage_count: stages.length,
    ready_stage_count,
    missing_stage_count: stages.length - ready_stage_count,
    active_artifact_id: active,
    planned_artifacts: rows,
    stages,
    progress: {
      super_long_behavior_closed: 18,
      super_long_total: 18,
      super_long_percent: 100,
      current_stage_closed: 4,
      current_stage_total: 8,
      current_stage_percent: 50,
      roadmap_v2_behavior_closed: 90,
      roadmap_v2_total: 90,
      roadmap_v2_percent: 100,
    },
    next_item: "STUDIO_TEACHER_FEEDBACK_LOOP_SEED_V1",
  };
}

export function formatPublicationArtifactDryRunText(dryRun = {}) {
  const payload = asObject(dryRun);
  if (payload.schema !== "ddn.studio.publication_artifact_dry_run.v1") {
    throw new Error("seamgrim_expected_publication_artifact_dry_run");
  }
  const rows = asArray(payload.planned_artifacts);
  return [
    `schema\t${payload.schema}`,
    `workflow_claim\t${payload.workflow_claim ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `next_state\t${payload.next_state ?? ""}`,
    `planned_artifact_count\t${payload.planned_artifact_count ?? rows.length}`,
    `all_planned_artifacts_generated_now\t${payload.all_planned_artifacts_generated_now === true ? "true" : "false"}`,
    `checksum_manifest_path\t${payload.checksum_policy?.manifest_path ?? ""}`,
    `artifact_generation_claim\t${payload.artifact_generation_claim === true ? "true" : "false"}`,
    `artifact_signing_claim\t${payload.artifact_signing_claim === true ? "true" : "false"}`,
    `followup\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `followup_percent\t${payload.progress?.current_stage_percent ?? ""}`,
    "",
    "artifact_id\tkind\tgenerated_now\tplanned_path",
    ...rows.map((row) => [
      row.id,
      row.kind,
      row.generated_now === true ? "true" : "false",
      row.planned_path,
    ].join("\t")),
  ].join("\n");
}

export function renderPublicationArtifactDryRun(root, dryRun = {}) {
  if (!root) return null;
  const payload = asObject(dryRun);
  const rows = asArray(payload.planned_artifacts);
  const activeId = asText(payload.active_artifact_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.publicationArtifactDryRunStatus = asText(payload.status, "publication_artifact_dry_run_incomplete");
  root.innerHTML = `
    <div class="artifact-dry-run-head">
      <div>
        <div class="artifact-dry-run-kicker">Publication artifacts</div>
        <h2>Artifact dry-run set</h2>
      </div>
      <div class="artifact-dry-run-progress" data-artifact-dry-run-progress>
        <span>${escapeHtml(String(payload.progress?.super_long_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.super_long_total ?? 0))} overall</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} follow-up</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
        <span>dry-run only</span>
      </div>
    </div>
    <div class="artifact-dry-run-summary" data-artifact-dry-run-summary>
      ${escapeHtml(String(payload.planned_artifact_count ?? 0))} artifacts · generated_now=false · signing approval-gated
    </div>
    <div class="artifact-dry-run-body">
      <div class="artifact-dry-run-list" data-artifact-dry-run-list>
        ${rows.map((row) => `
          <button
            type="button"
            class="artifact-dry-run-btn${row.id === activeId ? " active" : ""}"
            data-artifact-dry-run="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="artifact-dry-run-detail" data-artifact-dry-run-detail>
        <div class="artifact-dry-run-title" data-artifact-dry-run-active-title>${escapeHtml(active.title)}</div>
        <p data-artifact-dry-run-active-summary>${escapeHtml(active.summary)}</p>
        <dl>
          <div><dt>kind</dt><dd data-artifact-dry-run-active-kind>${escapeHtml(active.kind)}</dd></div>
          <div><dt>path</dt><dd data-artifact-dry-run-active-path>${escapeHtml(active.planned_path)}</dd></div>
          <div><dt>boundary</dt><dd>generated_now=false</dd></div>
        </dl>
        <button type="button" class="ghost" data-artifact-dry-run-copy>artifact dry-run 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => {
    renderPublicationArtifactDryRun(root, {
      ...payload,
      active_artifact_id: rowId,
    });
  };
  root.querySelectorAll("[data-artifact-dry-run]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-artifact-dry-run") || "");
    });
  });
  root.querySelector("[data-artifact-dry-run-copy]")?.addEventListener("click", async () => {
    const text = formatPublicationArtifactDryRunText(payload);
    root.dataset.publicationArtifactDryRunCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
