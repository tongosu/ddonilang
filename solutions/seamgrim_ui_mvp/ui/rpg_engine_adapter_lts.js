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

const ADAPTER_ROWS = [
  {
    id: "godot_manifest",
    title: "Godot manifest",
    adapter_kind: "godot_project_manifest",
    local_uri: "seamgrim://rpg-engine-adapter/local/godot/manifest",
  },
  {
    id: "native_bridge",
    title: "Native bridge",
    adapter_kind: "native_bridge_contract",
    local_uri: "seamgrim://rpg-engine-adapter/local/native/bridge-contract",
  },
  {
    id: "asset_map",
    title: "Asset map",
    adapter_kind: "rpg_asset_map",
    local_uri: "seamgrim://rpg-engine-adapter/local/assets/story-package",
  },
  {
    id: "lts_gate",
    title: "LTS gate",
    adapter_kind: "adapter_lts_gate",
    local_uri: "seamgrim://rpg-engine-adapter/local/lts/gate",
  },
];

const DEFAULT_ADAPTER_FILES = [
  { name: "godot.project.manifest.detjson", kind: "godot_manifest", bytes: 842 },
  { name: "native.bridge.contract.detjson", kind: "native_bridge", bytes: 618 },
  { name: "rpg.asset.map.detjson", kind: "asset_map", bytes: 476 },
  { name: "adapter.lts.gate.detjson", kind: "lts_gate", bytes: 392 },
];

export const DEFAULT_RPG_ENGINE_ADAPTER_LTS_ROWS = ADAPTER_ROWS.map((row) => ({
  id: row.id,
  adapter_kind: row.adapter_kind,
  local_uri: row.local_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_RPG_ENGINE_ADAPTER_LTS_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return ADAPTER_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      adapter_kind: asText(source.adapter_kind, row.adapter_kind),
      local_uri: asText(source.local_uri, row.local_uri),
      ready: source.ready !== false,
      local_only: true,
      product_ui_behavior: true,
      runtime_execution_claim: false,
      native_binary_claim: false,
      godot_project_build_claim: false,
      cloud_sync_claim: false,
    };
  });
}

function normalizeAdapterFiles(files = DEFAULT_ADAPTER_FILES) {
  const rows = asArray(files).length > 0 ? asArray(files) : DEFAULT_ADAPTER_FILES;
  return rows.map((file, index) => {
    const source = asObject(file);
    return {
      name: asText(source.name, `adapter_${index + 1}.detjson`),
      kind: asText(source.kind, "adapter_artifact"),
      bytes: Math.max(0, Math.trunc(asNumber(source.bytes, 0))),
      ready: source.ready !== false,
    };
  });
}

function buildAdapterText(files) {
  return [
    "rpg_engine_adapter_lts:malhim_rpg_5_v1",
    "coordinate:차-5",
    "runtime_execution:false",
    ...files.map((file) => `${file.kind}\t${file.name}\t${file.bytes}`),
  ].join("\n");
}

export function buildRpgEngineAdapterLts({
  rows = DEFAULT_RPG_ENGINE_ADAPTER_LTS_ROWS,
  adapterFiles = DEFAULT_ADAPTER_FILES,
  activeAdapterId = "godot_manifest",
} = {}) {
  const adapters = normalizeRows(rows);
  const files = normalizeAdapterFiles(adapterFiles);
  const active = adapters.some((adapter) => adapter.id === activeAdapterId)
    ? activeAdapterId
    : adapters[0]?.id ?? "";
  const readyAdapterCount = adapters.filter((adapter) => adapter.ready).length;
  const readyFileCount = files.filter((file) => file.ready).length;
  const fileKinds = new Set(files.map((file) => file.kind));
  const hasGodot = fileKinds.has("godot_manifest");
  const hasNative = fileKinds.has("native_bridge");
  const hasAssetMap = fileKinds.has("asset_map");
  const hasLtsGate = fileKinds.has("lts_gate");
  const adapterReady = (
    readyAdapterCount === adapters.length &&
    readyFileCount === files.length &&
    hasGodot &&
    hasNative &&
    hasAssetMap &&
    hasLtsGate
  );
  return {
    __종류: "rpg_engine_adapter_lts",
    schema: "ddn.seamgrim.rpg_engine.adapter_lts.v1",
    work_item: "CHA5_RPG_ENGINE_ADAPTER_LTS_V1",
    primary_coordinate: "차-5",
    depends_on_coordinate: ["차-4"],
    pack: "malhim_rpg_5_v1",
    status: adapterReady ? "rpg_engine_adapter_lts_ready" : "rpg_engine_adapter_lts_incomplete",
    matrix_closure_tier: adapterReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    adapter_lts_claim: adapterReady,
    godot_manifest_claim: hasGodot,
    native_bridge_contract_claim: hasNative,
    asset_map_claim: hasAssetMap,
    lts_gate_claim: hasLtsGate,
    native_runtime_execution_claim: false,
    godot_project_build_claim: false,
    native_binary_claim: false,
    cloud_sync_claim: false,
    adapter_count: adapters.length,
    adapter_file_count: files.length,
    adapter_size_bytes: files.reduce((sum, file) => sum + file.bytes, 0),
    current_stage_closed: adapterReady ? 5 : Math.min(4, readyAdapterCount),
    current_stage_total: 5,
    current_stage_percent: adapterReady ? 100 : Math.round((Math.min(4, readyAdapterCount) / 5) * 100),
    progress: {
      current_stage_closed: adapterReady ? 5 : Math.min(4, readyAdapterCount),
      current_stage_total: 5,
      current_stage_percent: adapterReady ? 100 : Math.round((Math.min(4, readyAdapterCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 17,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 19,
      roadmap_v2_pack_evidence_reference_closed: 37,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 41,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    adapters,
    adapter_files: files,
    adapter_text: buildAdapterText(files),
    active_adapter_id: active,
    next_item: "KA2_PUBLICATION_READ_API_CLOSURE_V1",
  };
}

export function formatRpgEngineAdapterLtsText(adapterLts = {}) {
  const payload = asObject(adapterLts);
  if (payload.schema !== "ddn.seamgrim.rpg_engine.adapter_lts.v1") {
    throw new Error("seamgrim_expected_rpg_engine_adapter_lts");
  }
  const adapters = asArray(payload.adapters);
  const files = asArray(payload.adapter_files);
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
    `adapter_lts_claim\t${payload.adapter_lts_claim === true ? "true" : "false"}`,
    `godot_manifest_claim\t${payload.godot_manifest_claim === true ? "true" : "false"}`,
    `native_bridge_contract_claim\t${payload.native_bridge_contract_claim === true ? "true" : "false"}`,
    `asset_map_claim\t${payload.asset_map_claim === true ? "true" : "false"}`,
    `lts_gate_claim\t${payload.lts_gate_claim === true ? "true" : "false"}`,
    `native_runtime_execution_claim\t${payload.native_runtime_execution_claim === true ? "true" : "false"}`,
    `godot_project_build_claim\t${payload.godot_project_build_claim === true ? "true" : "false"}`,
    `native_binary_claim\t${payload.native_binary_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    "",
    "adapter_id\tadapter_kind\tlocal_only\tlocal_uri",
    ...adapters.map((adapter) => [
      adapter.id,
      adapter.adapter_kind,
      adapter.local_only === true ? "true" : "false",
      adapter.local_uri,
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

export function renderRpgEngineAdapterLts(root, adapterLts = {}) {
  if (!root) return null;
  const payload = asObject(adapterLts);
  const adapters = asArray(payload.adapters);
  const activeId = asText(payload.active_adapter_id, adapters[0]?.id ?? "");
  const active = adapters.find((row) => row.id === activeId) || adapters[0] || {};
  const files = asArray(payload.adapter_files);
  root.dataset.rpgEngineAdapterLtsStatus = asText(
    payload.status,
    "rpg_engine_adapter_lts_incomplete",
  );
  root.innerHTML = `
    <div class="rpg-engine-adapter-head">
      <div>
        <div class="rpg-engine-adapter-kicker">Engine adapter LTS</div>
        <h2>RPG engine adapter</h2>
      </div>
      <div class="rpg-engine-adapter-progress" data-rpg-engine-adapter-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="rpg-engine-adapter-summary" data-rpg-engine-adapter-summary>
      Godot manifest, native bridge contract, asset map, LTS gate를 local adapter handoff로 묶고 실제 native execution/build는 후속으로 둡니다.
    </div>
    <div class="rpg-engine-adapter-body">
      <div class="rpg-engine-adapter-list" data-rpg-engine-adapter-list>
        ${adapters.map((row) => `
          <button
            type="button"
            class="rpg-engine-adapter-btn${row.id === activeId ? " active" : ""}"
            data-rpg-engine-adapter="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.adapter_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="rpg-engine-adapter-detail" data-rpg-engine-adapter-detail>
        <div class="rpg-engine-adapter-title" data-rpg-engine-adapter-active-title>${escapeHtml(active.title)}</div>
        <p data-rpg-engine-adapter-active-link>${escapeHtml(active.local_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>bytes</dt><dd>${escapeHtml(String(payload.adapter_size_bytes ?? 0))}</dd></div>
        </dl>
        <div class="rpg-engine-adapter-files" data-rpg-engine-adapter-files>
          ${files.map((file) => `
            <span data-rpg-engine-adapter-file="${escapeHtml(file.name)}">
              ${escapeHtml(file.name)} · ${escapeHtml(file.kind)} · ${escapeHtml(String(file.bytes))}b
            </span>
          `).join("")}
        </div>
        <pre class="rpg-engine-adapter-preview" data-rpg-engine-adapter-preview>${escapeHtml(payload.adapter_text || "")}</pre>
        <button type="button" class="ghost" data-rpg-engine-adapter-copy>adapter LTS 텍스트 복사</button>
      </div>
    </div>
  `;
  const rerender = (adapterId) => {
    renderRpgEngineAdapterLts(root, {
      ...payload,
      active_adapter_id: adapterId,
    });
  };
  root.querySelectorAll("[data-rpg-engine-adapter]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-rpg-engine-adapter") || "");
    });
  });
  root.querySelector("[data-rpg-engine-adapter-copy]")?.addEventListener("click", async () => {
    const text = formatRpgEngineAdapterLtsText(payload);
    root.dataset.rpgEngineAdapterLtsCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  return payload;
}
