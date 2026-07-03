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

const PACKAGE_ROWS = [
  {
    id: "manifest",
    title: "Story manifest",
    artifact_kind: "rpg_story_manifest",
    local_uri: "seamgrim://rpg-story-package/local/manifest/village-door",
  },
  {
    id: "map_snapshot",
    title: "Map snapshot",
    artifact_kind: "rpg_map_snapshot",
    local_uri: "seamgrim://rpg-story-package/local/map/village-door",
  },
  {
    id: "script_bundle",
    title: "Script bundle",
    artifact_kind: "rpg_script_bundle",
    local_uri: "seamgrim://rpg-story-package/local/script/phrase-action",
  },
  {
    id: "playtest_transcript",
    title: "Playtest transcript",
    artifact_kind: "rpg_playtest_transcript",
    local_uri: "seamgrim://rpg-story-package/local/playtest/transcript",
  },
];

const DEFAULT_PACKAGE_FILES = [
  { name: "story.manifest.detjson", kind: "manifest", bytes: 734 },
  { name: "map.snapshot.detjson", kind: "map_snapshot", bytes: 428 },
  { name: "script.bundle.ddn", kind: "script_bundle", bytes: 512 },
  { name: "playtest.transcript.txt", kind: "playtest_transcript", bytes: 266 },
];

export const DEFAULT_RPG_STORY_PACKAGE_ROWS = PACKAGE_ROWS.map((row) => ({
  id: row.id,
  artifact_kind: row.artifact_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_RPG_STORY_PACKAGE_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return PACKAGE_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      artifact_kind: asText(source.artifact_kind, row.artifact_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_only: true,
      product_ui_behavior: true,
      registry_publish_claim: false,
      public_upload_claim: false,
      cloud_sync_claim: false,
      engine_adapter_claim: false,
    };
  });
}

function normalizePackageFiles(files = DEFAULT_PACKAGE_FILES) {
  const rows = asArray(files).length > 0 ? asArray(files) : DEFAULT_PACKAGE_FILES;
  return rows.map((file, index) => {
    const source = asObject(file);
    return {
      name: asText(source.name, `artifact_${index + 1}.detjson`),
      kind: asText(source.kind, "artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildPackageText(files) {
  return [
    "rpg_story_package:malhim_rpg_4_v1",
    "coordinate:차-4",
    ...files.map((file) => `${file.kind}\t${file.name}\t${file.bytes}`),
  ].join("\n");
}

export function buildRpgStoryPackage({
  rows = DEFAULT_RPG_STORY_PACKAGE_ROWS,
  packageFiles = DEFAULT_PACKAGE_FILES,
  activeArtifactId = "manifest",
} = {}) {
  const artifacts = normalizeRows(rows);
  const files = normalizePackageFiles(packageFiles);
  const active = artifacts.some((artifact) => artifact.id === activeArtifactId)
    ? activeArtifactId
    : artifacts[0]?.id ?? "";
  const readyArtifactCount = artifacts.filter((artifact) => artifact.ready).length;
  const readyFileCount = files.filter((file) => file.ready).length;
  const fileKinds = new Set(files.map((file) => file.kind));
  const hasManifest = fileKinds.has("manifest");
  const hasMap = fileKinds.has("map_snapshot");
  const hasScript = fileKinds.has("script_bundle");
  const hasTranscript = fileKinds.has("playtest_transcript");
  const packageReady = (
    readyArtifactCount === artifacts.length &&
    readyFileCount === files.length &&
    hasManifest &&
    hasMap &&
    hasScript &&
    hasTranscript
  );
  return {
    __종류: "rpg_story_package",
    schema: "ddn.seamgrim.rpg_story.package.v1",
    work_item: "CHA4_RPG_STORY_PACKAGE_V1",
    primary_coordinate: "차-4",
    depends_on_coordinate: ["차-3"],
    pack: "malhim_rpg_4_v1",
    status: packageReady ? "rpg_story_package_ready" : "rpg_story_package_incomplete",
    matrix_closure_tier: packageReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    story_package_claim: packageReady,
    manifest_claim: hasManifest,
    map_snapshot_claim: hasMap,
    script_bundle_claim: hasScript,
    playtest_transcript_claim: hasTranscript,
    registry_publish_claim: false,
    public_upload_claim: false,
    cloud_sync_claim: false,
    engine_adapter_claim: false,
    artifact_count: artifacts.length,
    package_file_count: files.length,
    package_size_bytes: files.reduce((sum, file) => sum + file.bytes, 0),
    current_stage_closed: packageReady ? 5 : Math.min(4, readyArtifactCount),
    current_stage_total: 5,
    current_stage_percent: packageReady ? 100 : Math.round((Math.min(4, readyArtifactCount) / 5) * 100),
    progress: {
      current_stage_closed: packageReady ? 5 : Math.min(4, readyArtifactCount),
      current_stage_total: 5,
      current_stage_percent: packageReady ? 100 : Math.round((Math.min(4, readyArtifactCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 16,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 18,
      roadmap_v2_pack_evidence_reference_closed: 36,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 40,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    artifacts,
    package_files: files,
    package_text: buildPackageText(files),
    active_artifact_id: active,
    next_item: "CHA5_RPG_ENGINE_ADAPTER_LTS_V1",
  };
}

export function formatRpgStoryPackageText(storyPackage = {}) {
  const payload = asObject(storyPackage);
  if (payload.schema !== "ddn.seamgrim.rpg_story.package.v1") {
    throw new Error("seamgrim_expected_rpg_story_package");
  }
  const artifacts = asArray(payload.artifacts);
  const files = asArray(payload.package_files);
  return [
    `schema\t${payload.schema}`,
    `work_item\t${payload.work_item ?? ""}`,
    `primary_coordinate\t${payload.primary_coordinate ?? ""}`,
    `status\t${payload.status ?? ""}`,
    `matrix_closure_tier\t${payload.matrix_closure_tier ?? ""}`,
    `current_stage\t${payload.progress?.current_stage_closed ?? 0}/${payload.progress?.current_stage_total ?? 0}`,
    `current_stage_percent\t${payload.progress?.current_stage_percent ?? 0}`,
    `roadmap_matrix\t${payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0}/${payload.progress?.roadmap_v2_matrix_behavior_total ?? 0}`,
    `roadmap_matrix_percent\t${payload.progress?.roadmap_v2_matrix_behavior_percent ?? 0}`,
    `pack_evidence_reference\t${payload.progress?.roadmap_v2_pack_evidence_reference_closed ?? 0}/${payload.progress?.roadmap_v2_pack_evidence_reference_total ?? 0}`,
    `pack_evidence_reference_percent\t${payload.progress?.roadmap_v2_pack_evidence_reference_percent ?? 0}`,
    `studio_local_super_long\t${payload.progress?.studio_local_super_long_closed ?? 0}/${payload.progress?.studio_local_super_long_total ?? 0}`,
    `studio_local_super_long_percent\t${payload.progress?.studio_local_super_long_percent ?? 0}`,
    `story_package_claim\t${payload.story_package_claim === true ? "true" : "false"}`,
    `manifest_claim\t${payload.manifest_claim === true ? "true" : "false"}`,
    `map_snapshot_claim\t${payload.map_snapshot_claim === true ? "true" : "false"}`,
    `script_bundle_claim\t${payload.script_bundle_claim === true ? "true" : "false"}`,
    `playtest_transcript_claim\t${payload.playtest_transcript_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `public_upload_claim\t${payload.public_upload_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    `engine_adapter_claim\t${payload.engine_adapter_claim === true ? "true" : "false"}`,
    "",
    "artifact_id\tartifact_kind\tlocal_only\tlocal_uri",
    ...artifacts.map((artifact) => [
      artifact.id,
      artifact.artifact_kind,
      artifact.local_only === true ? "true" : "false",
      artifact.local_uri,
    ].join("\t")),
    "",
    "file_name\tkind\tbytes",
    ...files.map((file) => [
      file.name,
      file.kind,
      file.bytes,
    ].join("\t")),
  ].join("\n");
}

export function renderRpgStoryPackage(root, storyPackage = {}) {
  if (!root) return null;
  const payload = asObject(storyPackage);
  const artifacts = asArray(payload.artifacts);
  const activeId = asText(payload.active_artifact_id, artifacts[0]?.id ?? "");
  const active = artifacts.find((row) => row.id === activeId) || artifacts[0] || {};
  const files = asArray(payload.package_files);
  root.dataset.rpgStoryPackageStatus = asText(
    payload.status,
    "rpg_story_package_incomplete",
  );
  root.innerHTML = `
    <div class="rpg-story-package-head">
      <div>
        <div class="rpg-story-package-kicker">Story package</div>
        <h2>RPG story package</h2>
      </div>
      <div class="rpg-story-package-progress" data-rpg-story-package-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="rpg-story-package-summary" data-rpg-story-package-summary>
      manifest, map snapshot, script bundle, playtest transcript를 로컬 story package로 묶고 공개 업로드와 engine adapter는 후속으로 둡니다.
    </div>
    <div class="rpg-story-package-body">
      <div class="rpg-story-package-list" data-rpg-story-package-list>
        ${artifacts.map((row) => `
          <button
            type="button"
            class="rpg-story-package-btn${row.id === activeId ? " active" : ""}"
            data-rpg-story-package="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.artifact_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="rpg-story-package-detail" data-rpg-story-package-detail>
        <div class="rpg-story-package-title" data-rpg-story-package-active-title>${escapeHtml(active.title)}</div>
        <p data-rpg-story-package-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.package_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="rpg-story-package-files" data-rpg-story-package-files>
          ${files.map((file) => `
            <span data-rpg-story-package-file="${escapeHtml(file.name)}">
              ${escapeHtml(file.name)} · ${escapeHtml(file.kind)} · ${escapeHtml(String(file.bytes))}b
            </span>
          `).join("")}
        </div>
        <pre class="rpg-story-package-preview" data-rpg-story-package-preview>${escapeHtml(payload.package_text || "")}</pre>
        <button type="button" class="ghost" data-rpg-story-package-copy>story package 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (artifactId) => {
    renderRpgStoryPackage(root, {
      ...payload,
      active_artifact_id: artifactId,
    });
  };
  root.querySelectorAll("[data-rpg-story-package]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-rpg-story-package") || "");
    });
  });
  root.querySelector("[data-rpg-story-package-copy]")?.addEventListener("click", async () => {
    const text = formatRpgStoryPackageText(payload);
    root.dataset.rpgStoryPackageCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
