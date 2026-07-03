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

function asNumber(value, fallback = 0) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

const PUBLICATION_ROWS = [
  { id: "micro_course", title: "5-minute course", publication_kind: "five_minute_course_outline", local_uri: "education://publication/5-minute-course" },
  { id: "workshop_course", title: "1-hour workshop", publication_kind: "one_hour_workshop_outline", local_uri: "education://publication/1-hour-workshop" },
  { id: "four_week_course", title: "4-week course", publication_kind: "four_week_course_outline", local_uri: "education://publication/4-week-course" },
  { id: "publication_bundle", title: "Publication bundle", publication_kind: "local_publication_bundle", local_uri: "education://publication/bundle" },
  { id: "share_handoff", title: "Share handoff", publication_kind: "local_share_handoff", local_uri: "education://publication/share-handoff" },
];

const DEFAULT_ARTIFACTS = [
  { name: "education.publication.micro_course.detjson", kind: "micro_course", bytes: 778 },
  { name: "education.publication.workshop_course.detjson", kind: "workshop_course", bytes: 842 },
  { name: "education.publication.four_week_course.detjson", kind: "four_week_course", bytes: 884 },
  { name: "education.publication.bundle.detjson", kind: "publication_bundle", bytes: 736 },
  { name: "education.publication.share_handoff.detjson", kind: "share_handoff", bytes: 708 },
];

export const DEFAULT_EDUCATION_PUBLICATION_ROWS = PUBLICATION_ROWS.map((row) => ({
  id: row.id,
  publication_kind: row.publication_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_EDUCATION_PUBLICATION_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return PUBLICATION_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      publication_kind: asText(source.publication_kind, row.publication_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_publication_claim: true,
      public_upload_claim: false,
    };
  });
}

function normalizeArtifacts(artifacts = DEFAULT_ARTIFACTS) {
  const rows = asArray(artifacts).length > 0 ? asArray(artifacts) : DEFAULT_ARTIFACTS;
  return rows.map((artifact, index) => {
    const source = asObject(artifact);
    return {
      name: asText(source.name, `education_publication_${index + 1}.detjson`),
      kind: asText(source.kind, "education_publication_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildPublicationText(rows, artifacts) {
  return [
    "education_publication_pack:education_curriculum_4_v1",
    "coordinate:하-4",
    "public_upload:false",
    "public_link_creation:false",
    "registry_publish:false",
    "release_execution:false",
    "artifact_signing:false",
    "account_permission_change:false",
    "state_hash_participation:false",
    ...rows.map((row) => `${row.id}\t${row.publication_kind}\t${row.ready ? "ready" : "missing"}`),
    "",
    ...artifacts.map((artifact) => `${artifact.kind}\t${artifact.name}\t${artifact.bytes}`),
  ].join("\n");
}

export function buildEducationPublicationPack({
  rows = DEFAULT_EDUCATION_PUBLICATION_ROWS,
  artifacts = DEFAULT_ARTIFACTS,
  activeRowId = "micro_course",
} = {}) {
  const publicationRows = normalizeRows(rows);
  const publicationArtifacts = normalizeArtifacts(artifacts);
  const artifactKinds = new Set(publicationArtifacts.map((artifact) => artifact.kind));
  const readyRowCount = publicationRows.filter((row) => row.ready).length;
  const readyArtifactCount = publicationArtifacts.filter((artifact) => artifact.ready).length;
  const hasAllArtifacts = ["micro_course", "workshop_course", "four_week_course", "publication_bundle", "share_handoff"]
    .every((kind) => artifactKinds.has(kind));
  const ready = readyRowCount === publicationRows.length &&
    readyArtifactCount === publicationArtifacts.length &&
    hasAllArtifacts;
  const active = publicationRows.some((row) => row.id === activeRowId)
    ? activeRowId
    : publicationRows[0]?.id ?? "";
  return {
    __종류: "education_publication_pack",
    schema: "ddn.education.publication_pack.v1",
    work_item: "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1",
    primary_coordinate: "하-4",
    depends_on_coordinate: ["하-3", "마-4", "카-4"],
    pack: "education_curriculum_4_v1",
    status: ready ? "education_publication_pack_ready" : "education_publication_pack_incomplete",
    matrix_closure_tier: ready ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    education_publication_pack_claim: ready,
    micro_course_claim: artifactKinds.has("micro_course"),
    workshop_course_claim: artifactKinds.has("workshop_course"),
    four_week_course_claim: artifactKinds.has("four_week_course"),
    publication_bundle_claim: artifactKinds.has("publication_bundle"),
    share_handoff_claim: artifactKinds.has("share_handoff"),
    public_upload_claim: false,
    public_link_creation_claim: false,
    registry_publish_claim: false,
    release_execution_claim: false,
    artifact_signing_claim: false,
    account_permission_change_claim: false,
    state_hash_participation_claim: false,
    parser_frontdoor_change: false,
    grammar_claim: false,
    rows: publicationRows,
    artifacts: publicationArtifacts,
    active_row_id: active,
    publication_text: buildPublicationText(publicationRows, publicationArtifacts),
    artifact_size_bytes: publicationArtifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    progress: {
      current_stage_closed: ready ? 5 : Math.min(4, readyRowCount),
      current_stage_total: 5,
      current_stage_percent: ready ? 100 : Math.round((Math.min(4, readyRowCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 31,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 34,
      roadmap_v2_pack_evidence_reference_closed: 51,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 57,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    next_item: "HA5_EDUCATION_OPERATIONS_LTS_V1",
  };
}

export function formatEducationPublicationPackText(publication = {}) {
  const payload = asObject(publication);
  if (payload.schema !== "ddn.education.publication_pack.v1") {
    throw new Error("education_expected_publication_pack");
  }
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
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
    `education_publication_pack_claim\t${payload.education_publication_pack_claim === true ? "true" : "false"}`,
    `micro_course_claim\t${payload.micro_course_claim === true ? "true" : "false"}`,
    `workshop_course_claim\t${payload.workshop_course_claim === true ? "true" : "false"}`,
    `four_week_course_claim\t${payload.four_week_course_claim === true ? "true" : "false"}`,
    `publication_bundle_claim\t${payload.publication_bundle_claim === true ? "true" : "false"}`,
    `share_handoff_claim\t${payload.share_handoff_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `public_link_creation_claim\t${payload.public_link_creation_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `release_execution_claim\t${payload.release_execution_claim === true ? "true" : "false"}`,
    "",
    "row_id\tpublication_kind\tready",
    ...rows.map((row) => `${row.id}\t${row.publication_kind}\t${row.ready === true ? "true" : "false"}`),
    "",
    "artifact_name\tkind\tbytes",
    ...artifacts.map((artifact) => `${artifact.name}\t${artifact.kind}\t${artifact.bytes}`),
  ].join("\n");
}

export function renderEducationPublicationPack(root, publication = {}) {
  if (!root) return null;
  const payload = asObject(publication);
  const progress = asObject(payload.progress);
  const rows = asArray(payload.rows);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_row_id, rows[0]?.id ?? "");
  const active = rows.find((row) => row.id === activeId) || rows[0] || {};
  root.dataset.educationPublicationPackStatus = asText(payload.status, "education_publication_pack_incomplete");
  root.innerHTML = `
    <div class="education-publication-head">
      <div>
        <div class="education-publication-kicker">Education Publication</div>
        <h2>Education publication pack</h2>
      </div>
      <div class="education-publication-progress" data-education-publication-progress>
        <span>${escapeHtml(String(progress.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(progress.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(progress.current_stage_closed ?? 0))}/${escapeHtml(String(progress.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(progress.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="education-publication-summary" data-education-publication-summary>
      5-minute course, 1-hour workshop, 4-week course, publication bundle, share handoff를 로컬 publication pack으로 고정하고 public upload, public link creation, registry publish, release execution은 후속으로 둡니다.
    </div>
    <div class="education-publication-body">
      <div class="education-publication-list">
        ${rows.map((row) => `
          <button type="button" class="education-publication-btn${row.id === activeId ? " active" : ""}" data-education-publication-row="${escapeHtml(row.id)}">
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.publication_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="education-publication-detail">
        <div class="education-publication-title" data-education-publication-active-title>${escapeHtml(active.title)}</div>
        <p data-education-publication-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate ?? "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack ?? "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.artifact_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="education-publication-artifacts">
          ${artifacts.map((artifact) => `
            <span data-education-publication-artifact="${escapeHtml(artifact.name)}">${escapeHtml(artifact.name)} · ${escapeHtml(artifact.kind)} · ${escapeHtml(String(artifact.bytes))}b</span>
          `).join("")}
        </div>
        <pre class="education-publication-preview" data-education-publication-preview>${escapeHtml(payload.publication_text ?? "")}</pre>
        <button type="button" class="ghost" data-education-publication-copy>Publication pack 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (rowId) => renderEducationPublicationPack(root, { ...payload, active_row_id: rowId });
  root.querySelectorAll("[data-education-publication-row]").forEach((button) => {
    button.addEventListener("click", () => rerender(button.getAttribute("data-education-publication-row") || ""));
  });
  root.querySelector("[data-education-publication-copy]")?.addEventListener("click", async () => {
    root.dataset.educationPublicationPackCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(formatEducationPublicationPackText(payload));
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
