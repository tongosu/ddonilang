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

const AUTHORING_ROWS = [
  {
    id: "map_editor",
    title: "Map editor",
    artifact_kind: "rpg_grid_map",
    handoff_uri: "seamgrim://rpg-box/local/map-editor/village-door",
  },
  {
    id: "script_blocks",
    title: "Script blocks",
    artifact_kind: "rpg_phrase_action_script",
    handoff_uri: "seamgrim://rpg-box/local/script/phrase-action",
  },
  {
    id: "playtest",
    title: "Playtest handoff",
    artifact_kind: "rpg_story_playtest",
    handoff_uri: "seamgrim://studio/run/rpg-box/story-playtest",
  },
];

const DEFAULT_MAP_CELLS = [
  { id: "start", label: "시작", row: 1, col: 1, tile: "spawn" },
  { id: "door", label: "문앞", row: 1, col: 2, tile: "door" },
  { id: "npc", label: "NPC", row: 2, col: 2, tile: "npc" },
];

const DEFAULT_SCRIPT_LINES = [
  "phrase:문을 연다 -> action:open_door",
  "actor:용사 -> state:position=문앞",
  "dialogue:NPC -> 안녕 용사 방문=1",
];

export const DEFAULT_RPG_BOX_AUTHORING_ROWS = AUTHORING_ROWS.map((row) => ({
  id: row.id,
  artifact_kind: row.artifact_kind,
  handoff_uri: row.handoff_uri,
  ready: true,
}));

function normalizeRows(rows = DEFAULT_RPG_BOX_AUTHORING_ROWS) {
  const byId = new Map(asArray(rows).map((row) => {
    const payload = asObject(row);
    return [asText(payload.id), payload];
  }));
  return AUTHORING_ROWS.map((row) => {
    const source = byId.get(row.id) || {};
    return {
      id: row.id,
      title: row.title,
      artifact_kind: asText(source.artifact_kind, row.artifact_kind),
      handoff_uri: asText(source.handoff_uri, row.handoff_uri),
      ready: source.ready !== false,
      local_only: true,
      product_ui_behavior: true,
      cloud_sync_claim: false,
      registry_publish_claim: false,
      engine_adapter_claim: false,
      public_story_package_claim: false,
    };
  });
}

function normalizeMapCells(cells = DEFAULT_MAP_CELLS) {
  const rows = asArray(cells).length > 0 ? asArray(cells) : DEFAULT_MAP_CELLS;
  return rows.map((cell, index) => {
    const source = asObject(cell);
    return {
      id: asText(source.id, `cell_${index + 1}`),
      label: asText(source.label, `칸 ${index + 1}`),
      row: asNumber(source.row, 0),
      col: asNumber(source.col, 0),
      tile: asText(source.tile, "floor"),
      ready: source.ready !== false,
    };
  });
}

function normalizeScriptLines(lines = DEFAULT_SCRIPT_LINES) {
  const rows = asArray(lines).length > 0 ? asArray(lines) : DEFAULT_SCRIPT_LINES;
  return rows.map((line) => asText(line)).filter(Boolean);
}

function buildPlaytestDdn(scriptLines) {
  return [
    "\"CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1\" 보여주기.",
    "\"rpg box authoring ui sealed\" 보여주기.",
    "\"stage: 5/5 = 100%\" 보여주기.",
    "\"roadmap matrix behavior: 15/90 = 17%\" 보여주기.",
    "\"pack evidence reference: 35/90 = 39%\" 보여주기.",
    "\"studio local super-long: 9/18 = 50%\" 보여주기.",
    ...scriptLines.map((line) => `"${line}" 보여주기.`),
  ].join("\n");
}

export function buildRpgBoxAuthoringUi({
  rows = DEFAULT_RPG_BOX_AUTHORING_ROWS,
  mapCells = DEFAULT_MAP_CELLS,
  scriptLines = DEFAULT_SCRIPT_LINES,
  activePanelId = "map_editor",
} = {}) {
  const panels = normalizeRows(rows);
  const normalizedMapCells = normalizeMapCells(mapCells);
  const normalizedScriptLines = normalizeScriptLines(scriptLines);
  const active = panels.some((panel) => panel.id === activePanelId)
    ? activePanelId
    : panels[0]?.id ?? "";
  const readyPanelCount = panels.filter((panel) => panel.ready).length;
  const readyMapCount = normalizedMapCells.filter((cell) => cell.ready).length;
  const hasMap = normalizedMapCells.length >= 3 && readyMapCount === normalizedMapCells.length;
  const hasScript = normalizedScriptLines.length >= 3 && normalizedScriptLines.some((line) => line.includes("open_door"));
  const hasPlaytest = panels.some((panel) => (
    panel.id === "playtest" &&
    panel.handoff_uri.startsWith("seamgrim://studio/run/")
  ));
  const authoringReady = readyPanelCount === panels.length && hasMap && hasScript && hasPlaytest;
  const playtestDdn = buildPlaytestDdn(normalizedScriptLines);
  return {
    __종류: "rpg_box_authoring_ui",
    schema: "ddn.seamgrim.rpg_box.authoring_ui.v1",
    work_item: "CHA3_RPG_BOX_NURIMAKER_AUTHORING_UI_V1",
    primary_coordinate: "차-3",
    depends_on_coordinate: ["차-2"],
    pack: "malhim_rpg_3_v1",
    status: authoringReady ? "rpg_box_authoring_ui_ready" : "rpg_box_authoring_ui_incomplete",
    matrix_closure_tier: authoringReady ? "닫힘-동작" : "진행",
    product_ui_change: true,
    product_code_change: true,
    runtime_claim: false,
    new_parser_claim: false,
    grammar_claim: false,
    authoring_ui_claim: authoringReady,
    map_editor_claim: hasMap,
    script_block_claim: hasScript,
    playtest_handoff_claim: hasPlaytest,
    story_package_claim: false,
    engine_adapter_claim: false,
    registry_publish_claim: false,
    cloud_sync_claim: false,
    panel_count: panels.length,
    map_cell_count: normalizedMapCells.length,
    script_line_count: normalizedScriptLines.length,
    current_stage_closed: authoringReady ? 5 : Math.min(4, readyPanelCount),
    current_stage_total: 5,
    current_stage_percent: authoringReady ? 100 : Math.round((Math.min(4, readyPanelCount) / 5) * 100),
    progress: {
      current_stage_closed: authoringReady ? 5 : Math.min(4, readyPanelCount),
      current_stage_total: 5,
      current_stage_percent: authoringReady ? 100 : Math.round((Math.min(4, readyPanelCount) / 5) * 100),
      roadmap_v2_matrix_behavior_closed: 15,
      roadmap_v2_matrix_behavior_total: 90,
      roadmap_v2_matrix_behavior_percent: 17,
      roadmap_v2_pack_evidence_reference_closed: 35,
      roadmap_v2_pack_evidence_reference_total: 90,
      roadmap_v2_pack_evidence_reference_percent: 39,
      studio_local_super_long_closed: 9,
      studio_local_super_long_total: 18,
      studio_local_super_long_percent: 50,
    },
    panels,
    map_cells: normalizedMapCells,
    script_lines: normalizedScriptLines,
    playtest_ddn: playtestDdn,
    active_panel_id: active,
    next_item: "CHA4_RPG_STORY_PACKAGE_V1",
  };
}

export function formatRpgBoxAuthoringUiText(authoring = {}) {
  const payload = asObject(authoring);
  if (payload.schema !== "ddn.seamgrim.rpg_box.authoring_ui.v1") {
    throw new Error("seamgrim_expected_rpg_box_authoring_ui");
  }
  const panels = asArray(payload.panels);
  const mapCells = asArray(payload.map_cells);
  const scriptLines = asArray(payload.script_lines);
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
    `authoring_ui_claim\t${payload.authoring_ui_claim === true ? "true" : "false"}`,
    `map_editor_claim\t${payload.map_editor_claim === true ? "true" : "false"}`,
    `script_block_claim\t${payload.script_block_claim === true ? "true" : "false"}`,
    `playtest_handoff_claim\t${payload.playtest_handoff_claim === true ? "true" : "false"}`,
    `story_package_claim\t${payload.story_package_claim === true ? "true" : "false"}`,
    `engine_adapter_claim\t${payload.engine_adapter_claim === true ? "true" : "false"}`,
    `registry_publish_claim\t${payload.registry_publish_claim === true ? "true" : "false"}`,
    `cloud_sync_claim\t${payload.cloud_sync_claim === true ? "true" : "false"}`,
    "",
    "panel_id\tartifact_kind\tlocal_only\thandoff_uri",
    ...panels.map((panel) => [
      panel.id,
      panel.artifact_kind,
      panel.local_only === true ? "true" : "false",
      panel.handoff_uri,
    ].join("\t")),
    "",
    "cell_id\tlabel\trow\tcol\ttile",
    ...mapCells.map((cell) => [
      cell.id,
      cell.label,
      cell.row,
      cell.col,
      cell.tile,
    ].join("\t")),
    "",
    "script",
    ...scriptLines,
  ].join("\n");
}

export function renderRpgBoxAuthoringUi(root, authoring = {}, { onOpenPlaytest = null } = {}) {
  if (!root) return null;
  const payload = asObject(authoring);
  const panels = asArray(payload.panels);
  const activeId = asText(payload.active_panel_id, panels[0]?.id ?? "");
  const active = panels.find((row) => row.id === activeId) || panels[0] || {};
  const mapCells = asArray(payload.map_cells);
  const scriptLines = asArray(payload.script_lines);
  root.dataset.rpgBoxAuthoringUiStatus = asText(
    payload.status,
    "rpg_box_authoring_ui_incomplete",
  );
  root.innerHTML = `
    <div class="rpg-box-authoring-head">
      <div>
        <div class="rpg-box-authoring-kicker">RPG Box authoring</div>
        <h2>RPG Box / 누리메이커</h2>
      </div>
      <div class="rpg-box-authoring-progress" data-rpg-box-authoring-progress>
        <span>${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_closed ?? 0))}/${escapeHtml(String(payload.progress?.roadmap_v2_matrix_behavior_total ?? 0))} ROADMAP</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_closed ?? 0))}/${escapeHtml(String(payload.progress?.current_stage_total ?? 0))} stage</span>
        <span>${escapeHtml(String(payload.progress?.current_stage_percent ?? 0))}%</span>
      </div>
    </div>
    <div class="rpg-box-authoring-summary" data-rpg-box-authoring-summary>
      map editor, script blocks, local playtest handoff를 한 authoring UI로 묶고 story package와 engine adapter는 후속으로 둡니다.
    </div>
    <div class="rpg-box-authoring-body">
      <div class="rpg-box-authoring-list" data-rpg-box-authoring-list>
        ${panels.map((row) => `
          <button
            type="button"
            class="rpg-box-authoring-btn${row.id === activeId ? " active" : ""}"
            data-rpg-box-authoring="${escapeHtml(row.id)}"
          >
            <span>${escapeHtml(row.title)}</span>
            <small>${escapeHtml(row.artifact_kind)}</small>
          </button>
        `).join("")}
      </div>
      <div class="rpg-box-authoring-detail" data-rpg-box-authoring-detail>
        <div class="rpg-box-authoring-title" data-rpg-box-authoring-active-title>${escapeHtml(active.title)}</div>
        <p data-rpg-box-authoring-active-link>${escapeHtml(active.handoff_uri)}</p>
        <dl>
          <div><dt>coordinate</dt><dd>${escapeHtml(payload.primary_coordinate || "")}</dd></div>
          <div><dt>pack</dt><dd>${escapeHtml(payload.pack || "")}</dd></div>
          <div><dt>map cells</dt><dd>${escapeHtml(String(payload.map_cell_count ?? 0))}</dd></div>
        </dl>
        <div class="rpg-box-authoring-grid" data-rpg-box-authoring-grid>
          ${mapCells.map((cell) => `
            <span data-rpg-box-authoring-cell="${escapeHtml(cell.id)}">
              ${escapeHtml(cell.label)} · ${escapeHtml(cell.tile)} (${escapeHtml(String(cell.row))},${escapeHtml(String(cell.col))})
            </span>
          `).join("")}
        </div>
        <pre class="rpg-box-authoring-script" data-rpg-box-authoring-script>${escapeHtml(scriptLines.join("\n"))}</pre>
        <div class="rpg-box-authoring-actions">
          <button type="button" class="btn-primary" data-rpg-box-authoring-open>작업실에서 playtest 열기</button>
          <button type="button" class="ghost" data-rpg-box-authoring-copy>authoring UI 텍스트 복사</button>
        </div>
      </div>
    </div>
  `;
  const rerender = (panelId) => {
    renderRpgBoxAuthoringUi(root, {
      ...payload,
      active_panel_id: panelId,
    }, { onOpenPlaytest });
  };
  root.querySelectorAll("[data-rpg-box-authoring]").forEach((button) => {
    button.addEventListener("click", () => {
      rerender(button.getAttribute("data-rpg-box-authoring") || "");
    });
  });
  root.querySelector("[data-rpg-box-authoring-copy]")?.addEventListener("click", async () => {
    const text = formatRpgBoxAuthoringUiText(payload);
    root.dataset.rpgBoxAuthoringUiCopied = "true";
    try {
      await navigator?.clipboard?.writeText?.(text);
    } catch (_) {
      // Clipboard is optional in local browser smokes.
    }
  });
  root.querySelector("[data-rpg-box-authoring-open]")?.addEventListener("click", () => {
    root.dataset.rpgBoxAuthoringUiOpened = "true";
    if (typeof onOpenPlaytest === "function") {
      onOpenPlaytest(payload);
    }
  });
  return payload;
}
