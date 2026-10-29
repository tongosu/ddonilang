#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function normalizeNewlines(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function stripMetaHeader(text) {
  const lines = normalizeNewlines(text).split("\n");
  let idx = 0;
  while (idx < lines.length) {
    const trimmed = lines[idx].replace(/^[ \t\uFEFF]+/, "");
    if (!trimmed) {
      idx += 1;
      continue;
    }
    if (trimmed.startsWith("#") && trimmed.includes(":")) {
      idx += 1;
      continue;
    }
    break;
  }
  return lines.slice(idx).join("\n");
}

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

function indexRow(columns, row) {
  const out = {};
  const colList = Array.isArray(columns) ? columns : [];
  const rowList = Array.isArray(row) ? row : [];
  colList.forEach((col, idx) => {
    const key = String(col?.key ?? "");
    if (!key) return;
    out[key] = rowList[idx];
  });
  return out;
}

function hasKey(columns, key) {
  const target = String(key ?? "").trim();
  if (!target) return false;
  const list = Array.isArray(columns) ? columns : [];
  return list.some((entry) => String(entry?.key ?? "").trim() === target);
}

async function initRuntime(rootDir) {
  const uiDir = path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModuleUrl = pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href;
  const wrapperUrl = pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href;
  const uiCommonUrl = pathToFileURL(path.join(uiDir, "wasm_page_common.js")).href;
  const runtimeStateUrl = pathToFileURL(path.join(uiDir, "seamgrim_runtime_state.js")).href;
  const wasmPath = path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm");

  const wasmModule = await import(wasmModuleUrl);
  const wrapper = await import(wrapperUrl);
  const uiCommon = await import(uiCommonUrl);
  const runtimeState = await import(runtimeStateUrl);
  const wasmBytes = await fs.readFile(wasmPath);

  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }
  if (typeof wasmModule.DdnWasmVm !== "function") {
    throw new Error("DdnWasmVm export 누락");
  }
  if (typeof wrapper.DdnWasmVmClient !== "function") {
    throw new Error("DdnWasmVmClient export 누락");
  }
  return { wasmModule, wrapper, uiCommon, runtimeState };
}

async function initCanonRuntime(rootDir) {
  const canonRuntimeUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "wasm_canon_runtime.js"),
  ).href;
  const runtimeMod = await import(canonRuntimeUrl);
  if (typeof runtimeMod.createWasmCanon !== "function") {
    throw new Error("createWasmCanon export 누락");
  }
  return runtimeMod;
}

async function makeClient(runtime, sourceText) {
  const source = stripMetaHeader(sourceText).trim();
  const vm = new runtime.wasmModule.DdnWasmVm(source);
  const client = new runtime.wrapper.DdnWasmVmClient(vm);
  return { vm, client };
}

function uniqueStrings(values) {
  return Array.from(new Set((Array.isArray(values) ? values : []).map((v) => String(v)))).sort((a, b) =>
    a.localeCompare(b),
  );
}

function parsePragmaRefsByNode(sourceText, nodeNames) {
  const refsByNode = new Map();
  (Array.isArray(nodeNames) ? nodeNames : []).forEach((name) => refsByNode.set(String(name), []));

  const lines = String(sourceText ?? "").split(/\r?\n/);
  const pragmaKinds = [
    ["#그래프", "graph"],
    ["#점", "point"],
    ["#공간", "space2d"],
    ["#표", "table"],
    ["#조종", "control"],
    ["#추적", "trace"],
    ["#진단", "diagnostic"],
  ];
  for (const rawLine of lines) {
    const line = String(rawLine ?? "").trim();
    if (!line.includes("#")) continue;
    const pragma = pragmaKinds.find(([prefix]) => line.includes(prefix));
    if (!pragma) continue;
    const kind = pragma[1];
    const tokens = line.match(/[0-9A-Za-z_가-힣.]+/g) ?? [];
    for (const [nodeName, refs] of refsByNode.entries()) {
      if (tokens.includes(nodeName)) {
        refs.push(kind);
      }
    }
  }

  const out = {};
  for (const [nodeName, refs] of refsByNode.entries()) {
    out[nodeName] = uniqueStrings(refs);
  }
  return out;
}

function buildManifestFromState(statePayload, sourceText) {
  const rawChannels = Array.isArray(statePayload?.channels)
    ? statePayload.channels
    : Array.isArray(statePayload?.state?.channels)
      ? statePayload.state.channels
      : [];
  const channels = rawChannels
    .map((entry) => {
      const key = String(entry?.key ?? entry?.name ?? "").trim();
      if (!key) return null;
      return {
        key,
        dtype: String(entry?.dtype ?? "unknown"),
        role: String(entry?.role ?? "state"),
        unit: typeof entry?.unit === "string" ? entry.unit : null,
      };
    })
    .filter((entry) => entry !== null);
  const nodeNames = channels.map((entry) => entry.key);
  const refsByNode = parsePragmaRefsByNode(sourceText, nodeNames);

  const xChannel = nodeNames.includes("프레임수")
    ? "프레임수"
    : nodeNames.includes("t")
      ? "t"
      : nodeNames[0] ?? "";
  const yChannels = nodeNames.filter((name) => name && name !== xChannel).slice(0, 2);

  const nodes = channels.map((entry) => ({
    name: entry.key,
    dtype: entry.dtype,
    role: entry.role || "state",
    pragma_refs: refsByNode[entry.key] ?? [],
    unit: entry.unit,
    group: "default",
  }));

  return {
    schema: "observation_protocol.v0",
    nodes,
    params: [],
    traces: [],
    diagnostics: [],
    default_pivot: {
      x_channel: xChannel,
      y_channels: yChannels,
    },
  };
}

function normalizeManifestNode(entry) {
  const source = entry && typeof entry === "object" ? entry : null;
  if (!source) return null;
  const name = String(source.name ?? source.key ?? "").trim();
  if (!name) return null;
  return {
    name,
    dtype: String(source.dtype ?? "unknown"),
    role: String(source.role ?? "state"),
    pragma_refs: Array.isArray(source.pragma_refs) ? uniqueStrings(source.pragma_refs) : [],
    unit: typeof source.unit === "string" ? source.unit : null,
    group: typeof source.group === "string" ? source.group : "default",
  };
}

function resolveObservationManifestFromState(statePayload, sourceText) {
  const native =
    statePayload?.observation_manifest && typeof statePayload.observation_manifest === "object"
      ? statePayload.observation_manifest
      : statePayload?.state?.observation_manifest &&
          typeof statePayload.state.observation_manifest === "object"
        ? statePayload.state.observation_manifest
        : null;
  if (!native || !String(native.schema ?? "").trim()) {
    return { manifest: buildManifestFromState(statePayload, sourceText), source: "synthesized" };
  }

  const nodes = Array.isArray(native.nodes) ? native.nodes.map(normalizeManifestNode).filter(Boolean) : [];
  const nodeNames = nodes.map((node) => String(node.name));
  const pivot = native.default_pivot && typeof native.default_pivot === "object" ? native.default_pivot : {};
  const xChannelRaw = String(pivot.x_channel ?? "").trim();
  const xChannel = xChannelRaw || (nodeNames.includes("프레임수") ? "프레임수" : nodeNames[0] ?? "");
  const yChannels = Array.isArray(pivot.y_channels)
    ? pivot.y_channels.map((item) => String(item)).filter(Boolean)
    : nodeNames.filter((name) => name && name !== xChannel).slice(0, 2);

  return {
    source: "native",
    manifest: {
      schema: String(native.schema),
      version: String(native.version ?? ""),
      nodes,
      params: Array.isArray(native.params) ? native.params : [],
      traces: Array.isArray(native.traces) ? native.traces : [],
      diagnostics: Array.isArray(native.diagnostics) ? native.diagnostics : [],
      default_pivot: {
        x_channel: xChannel,
        y_channels: yChannels,
      },
    },
  };
}

function metadataFromEngineResponse(rawState) {
  const tick = Number(rawState?.tick_id ?? rawState?.state?.tick_id ?? 0);
  return {
    contract: "D-STRICT",
    detmath_seal_hash: "",
    nuri_lock_hash: "",
    tick: Number.isFinite(tick) ? tick : 0,
  };
}

async function runWasmV0Smoke(packDir, runtime) {
  const eventsPath = path.join(packDir, "fixtures", "param_events.detjson");
  const eventsDoc = await readJson(eventsPath);
  const lessonRel = String(eventsDoc.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");

  const { vm, client } = await makeClient(runtime, lesson);
  const rows = [];
  let madi = 0;
  try {
    for (const event of eventsDoc.events ?? []) {
      const type = String(event?.type ?? "").trim().toLowerCase();
      if (type === "reset") {
        const keep = Boolean(event.keep_params ?? event.keepParams ?? false);
        client.resetParsed(keep);
        continue;
      }
      if (type === "set_param") {
        const key = String(event.name ?? event.key ?? "").trim();
        if (!key) throw new Error("set_param 이벤트에 key/name이 없습니다");
        client.setParamParsed(key, event.value);
        continue;
      }
      if (type === "columns") {
        client.columnsParsed();
        continue;
      }
      if (type === "step") {
        const nRaw = Number(event.n ?? 1);
        const n = Number.isFinite(nRaw) && nRaw > 0 ? Math.floor(nRaw) : 1;
        let last = null;
        for (let i = 0; i < n; i += 1) {
          last = client.stepOneParsed();
        }
        if (last?.state_hash) {
          rows.push({ madi, state_hash: last.state_hash });
          madi += 1;
        }
        continue;
      }
      throw new Error(`지원하지 않는 이벤트 타입: ${type}`);
    }
  } finally {
    if (typeof vm.free === "function") vm.free();
  }

  return {
    "expected/state_hash_trace.detjson": {
      algo: "blake3",
      rows,
    },
  };
}

async function runBridgeContractV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "contract.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    const stepCountRaw = Number(fixture.step_before_columns ?? 1);
    const stepCount = Number.isFinite(stepCountRaw) && stepCountRaw > 0 ? Math.floor(stepCountRaw) : 1;
    for (let i = 0; i < stepCount; i += 1) {
      client.stepOneParsed();
    }

    const paramKey = String(fixture?.set_param?.key ?? fixture?.set_param?.name ?? "").trim();
    if (!paramKey) {
      throw new Error("contract.detjson: set_param.key/name 누락");
    }
    const paramValue = fixture?.set_param?.value;
    const setParamResult = client.setParamParsed(paramKey, paramValue);
    const stateParsed = client.getStateParsed();
    const rawStatePayload = JSON.parse(vm.get_state_json());
    const columnsAfterSetParam = client.columnsParsed();
    const colList = Array.isArray(columnsAfterSetParam?.columns) ? columnsAfterSetParam.columns : [];
    const colKeys = new Set(colList.map((item) => String(item?.key ?? "")).filter(Boolean));
    const colRow = Array.isArray(columnsAfterSetParam?.row) ? columnsAfterSetParam.row : [];

    const resetKeep = client.resetParsed(true);
    const keepCols = Array.isArray(resetKeep?.columns) ? resetKeep.columns : [];
    const keepRowList = Array.isArray(resetKeep?.row) ? resetKeep.row : [];
    const keepRow = indexRow(resetKeep?.columns, resetKeep?.row);
    const keepValue = Object.prototype.hasOwnProperty.call(keepRow, paramKey) ? keepRow[paramKey] : null;

    const resetDrop = client.resetParsed(false);
    const dropCols = Array.isArray(resetDrop?.columns) ? resetDrop.columns : [];
    const dropRowList = Array.isArray(resetDrop?.row) ? resetDrop.row : [];
    const dropRow = indexRow(resetDrop?.columns, resetDrop?.row);
    const hasDropValue = Object.prototype.hasOwnProperty.call(dropRow, paramKey);
    const rawTopChannels = Array.isArray(rawStatePayload?.channels) ? rawStatePayload.channels : [];
    const rawTopRow = Array.isArray(rawStatePayload?.row) ? rawStatePayload.row : [];
    const rawStateChannels = Array.isArray(rawStatePayload?.state?.channels) ? rawStatePayload.state.channels : [];
    const rawStateRow = Array.isArray(rawStatePayload?.state?.row) ? rawStatePayload.state.row : [];
    const normalizedChannels = Array.isArray(stateParsed?.channels) ? stateParsed.channels : [];
    const normalizedRow = Array.isArray(stateParsed?.row) ? stateParsed.row : [];

    const report = {
      schema: "seamgrim.wasm.bridge_contract.v1",
      param_key: paramKey,
      raw_schema: String(rawStatePayload?.schema ?? ""),
      normalized_schema: String(stateParsed?.schema ?? ""),
      normalized_engine_schema: String(stateParsed?.engine_schema ?? ""),
      has_state_channel: Boolean(rawStatePayload?.state && typeof rawStatePayload.state === "object"),
      has_view_meta_channel: Boolean(rawStatePayload?.view_meta && typeof rawStatePayload.view_meta === "object"),
      has_view_hash: typeof rawStatePayload?.view_hash === "string" && rawStatePayload.view_hash.length > 0,
      state_has_resources: Boolean(rawStatePayload?.state?.resources && typeof rawStatePayload.state.resources === "object"),
      raw_top_channels_count: rawTopChannels.length,
      raw_top_row_count: rawTopRow.length,
      raw_state_channels_count: rawStateChannels.length,
      raw_state_row_count: rawStateRow.length,
      normalized_channels_count: normalizedChannels.length,
      normalized_row_count: normalizedRow.length,
      raw_top_row_matches_channels: rawTopRow.length === rawTopChannels.length,
      raw_state_row_matches_channels: rawStateRow.length === rawStateChannels.length,
      normalized_row_matches_channels: normalizedRow.length === normalizedChannels.length,
      columns_count_after_set_param: colList.length,
      columns_row_count_after_set_param: colRow.length,
      columns_row_matches_channels: colRow.length === colList.length,
      columns_has_param_key: colKeys.has(paramKey),
      set_param_ok: Boolean(setParamResult?.ok),
      set_param_state_hash: String(setParamResult?.state_hash ?? ""),
      reset_keep_params_has_key: keepValue !== null,
      reset_keep_params_value: keepValue,
      reset_keep_columns_count: keepCols.length,
      reset_keep_row_count: keepRowList.length,
      reset_keep_row_matches_columns: keepRowList.length === keepCols.length,
      reset_keep_columns_has_param_key: hasKey(keepCols, paramKey),
      reset_keep_params_state_hash: String(resetKeep?.state_hash ?? ""),
      reset_drop_params_has_key: hasDropValue,
      reset_drop_columns_count: dropCols.length,
      reset_drop_row_count: dropRowList.length,
      reset_drop_row_matches_columns: dropRowList.length === dropCols.length,
      reset_drop_columns_has_param_key: hasKey(dropCols, paramKey),
      reset_drop_params_state_hash: String(resetDrop?.state_hash ?? ""),
    };

    return {
      "expected/bridge_contract.detjson": report,
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runViewmetaStatehashV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "events.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");

  const { vm, client } = await makeClient(runtime, lesson);
  const observedStateHashes = {};
  const observedViewHashes = {};
  const observedHasViewHash = {};
  try {
    for (const event of fixture.events ?? []) {
      const type = String(event?.type ?? "").trim().toLowerCase();
      if (type === "reset") {
        const keep = Boolean(event.keep_params ?? event.keepParams ?? false);
        client.resetParsed(keep);
        continue;
      }
      if (type === "set_param") {
        const key = String(event.name ?? event.key ?? "").trim();
        if (!key) throw new Error("events.detjson: set_param key/name 누락");
        client.setParamParsed(key, event.value);
        continue;
      }
      if (type === "hash") {
        const label = String(event.label ?? "").trim();
        if (!label) throw new Error("events.detjson: hash label 누락");
        const state = client.getStateParsed();
        observedStateHashes[label] = String(state?.state_hash ?? client.getStateHash() ?? "");
        observedViewHashes[label] = String(state?.view_hash ?? "");
        observedHasViewHash[label] = observedViewHashes[label].length > 0;
        continue;
      }
      if (type === "step") {
        const nRaw = Number(event.n ?? 1);
        const n = Number.isFinite(nRaw) && nRaw > 0 ? Math.floor(nRaw) : 1;
        for (let i = 0; i < n; i += 1) {
          client.stepOneParsed();
        }
        continue;
      }
      throw new Error(`지원하지 않는 이벤트 타입: ${type}`);
    }

    const aLabel = String(fixture.compare?.a ?? "a");
    const bLabel = String(fixture.compare?.b ?? "b");
    const cLabel = String(fixture.compare?.c ?? "c");
    const aHash = String(observedStateHashes[aLabel] ?? "");
    const bHash = String(observedStateHashes[bLabel] ?? "");
    const cHash = String(observedStateHashes[cLabel] ?? "");
    const aViewHash = String(observedViewHashes[aLabel] ?? "");
    const bViewHash = String(observedViewHashes[bLabel] ?? "");
    const cViewHash = String(observedViewHashes[cLabel] ?? "");

    const report = {
      schema: "seamgrim.wasm.viewmeta_statehash.v1",
      labels: { a: aLabel, b: bLabel, c: cLabel },
      hashes: { [aLabel]: aHash, [bLabel]: bHash, [cLabel]: cHash },
      view_hashes: { [aLabel]: aViewHash, [bLabel]: bViewHash, [cLabel]: cViewHash },
      has_view_hash: {
        [aLabel]: Boolean(observedHasViewHash[aLabel]),
        [bLabel]: Boolean(observedHasViewHash[bLabel]),
        [cLabel]: Boolean(observedHasViewHash[cLabel]),
      },
      equal_ab: aHash !== "" && aHash === bHash,
      equal_bc: bHash !== "" && bHash === cHash,
      view_equal_ab: aViewHash !== "" && aViewHash === bViewHash,
      view_equal_bc: bViewHash !== "" && bViewHash === cViewHash,
      note: "a/b는 같은 상태 + 다른 view 전용 키, c는 상태 키 변경 포함",
    };

    return {
      "expected/viewmeta_statehash.detjson": report,
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runRestoreStateV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "restore.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    client.resetParsed(false);

    for (const op of fixture.prepare ?? []) {
      const type = String(op?.type ?? "").trim().toLowerCase();
      if (type === "set_param") {
        const key = String(op.key ?? op.name ?? "").trim();
        if (!key) throw new Error("restore.detjson: prepare set_param key 누락");
        client.setParamParsed(key, op.value);
        continue;
      }
      if (type === "step") {
        const n = Math.max(1, Math.floor(Number(op.n ?? 1)));
        for (let i = 0; i < n; i += 1) client.stepOneParsed();
        continue;
      }
      throw new Error(`restore.detjson: unsupported prepare op: ${type}`);
    }

    const snapshotRaw = vm.get_state_json();
    const snapshotObj = JSON.parse(snapshotRaw);
    const snapshotHash = String(snapshotObj?.state_hash ?? "");
    const snapshotTick = Number(snapshotObj?.tick_id ?? 0);
    const snapshotSchema = String(snapshotObj?.schema ?? "");

    for (const op of fixture.mutate ?? []) {
      const type = String(op?.type ?? "").trim().toLowerCase();
      if (type === "set_param") {
        const key = String(op.key ?? op.name ?? "").trim();
        if (!key) throw new Error("restore.detjson: mutate set_param key 누락");
        client.setParamParsed(key, op.value);
        continue;
      }
      if (type === "step") {
        const n = Math.max(1, Math.floor(Number(op.n ?? 1)));
        for (let i = 0; i < n; i += 1) client.stepOneParsed();
        continue;
      }
      throw new Error(`restore.detjson: unsupported mutate op: ${type}`);
    }

    const hashBeforeRestore = String(client.getStateHash() ?? "");
    const restoreResult = JSON.parse(vm.restore_state(snapshotRaw));
    const hashAfterRestore = String(client.getStateHash() ?? "");
    const stateAfter = client.getStateParsed();

    const report = {
      schema: "seamgrim.wasm.restore_state.v1",
      snapshot_schema: snapshotSchema,
      snapshot_tick: snapshotTick,
      snapshot_state_hash: snapshotHash,
      hash_before_restore: hashBeforeRestore,
      restore_ok: Boolean(restoreResult?.ok),
      restore_tick: Number(restoreResult?.tick ?? -1),
      hash_after_restore: hashAfterRestore,
      restored_equals_snapshot: hashAfterRestore !== "" && hashAfterRestore === snapshotHash,
      changed_before_restore: hashBeforeRestore !== "" && hashBeforeRestore !== snapshotHash,
      post_restore_schema: String(stateAfter?.engine_schema ?? ""),
      post_restore_tick: Number(stateAfter?.tick_id ?? -1),
      post_restore_state_hash: String(stateAfter?.state_hash ?? ""),
    };

    return {
      "expected/restore_state.detjson": report,
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

function channelKeys(channels) {
  const list = Array.isArray(channels) ? channels : [];
  return list
    .map((entry) => String(entry?.key ?? entry?.name ?? "").trim())
    .filter(Boolean);
}

function streamKeys(streams) {
  if (!streams || typeof streams !== "object") return [];
  return Object.keys(streams)
    .map((key) => String(key))
    .filter(Boolean)
    .sort((a, b) => a.localeCompare(b));
}

function summarizeStream(stream) {
  const buffer = Array.isArray(stream?.buffer) ? stream.buffer : [];
  const toNumberOrNull = (value) => {
    const n = Number(value);
    return Number.isFinite(n) ? n : null;
  };
  return {
    capacity: Number(stream?.capacity ?? 0),
    head: Number(stream?.head ?? 0),
    len: Number(stream?.len ?? 0),
    buffer_len: buffer.length,
    buffer_prefix: buffer.slice(0, 6).map((value) => {
      if (value === null) return null;
      if (typeof value === "number") return value;
      return toNumberOrNull(value);
    }),
  };
}

async function runStreamsSerializationV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "streams.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");
  const stepCount = Math.max(0, Math.floor(Number(fixture.step_before_state ?? 1)));
  const expectedKeys = Array.isArray(fixture.expected_keys)
    ? fixture.expected_keys.map((key) => String(key))
    : [];
  const expectedStreams =
    fixture.expected_streams && typeof fixture.expected_streams === "object"
      ? fixture.expected_streams
      : {};
  const restorePayload =
    fixture.restore_payload && typeof fixture.restore_payload === "object"
      ? fixture.restore_payload
      : null;

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    client.resetParsed(false);
    if (restorePayload) {
      JSON.parse(vm.restore_state(JSON.stringify(restorePayload)));
    }
    for (let i = 0; i < stepCount; i += 1) {
      client.stepOneParsed();
    }

    const rawState = JSON.parse(vm.get_state_json());
    const normalizedState = client.getStateParsed();
    const topStreams =
      rawState?.streams && typeof rawState.streams === "object" ? rawState.streams : {};
    const nestedStreams =
      rawState?.state?.streams && typeof rawState.state.streams === "object"
        ? rawState.state.streams
        : {};
    const normalizedStreams =
      normalizedState?.streams && typeof normalizedState.streams === "object"
        ? normalizedState.streams
        : {};
    const topKeys = streamKeys(topStreams);
    const nestedKeys = streamKeys(nestedStreams);
    const normalizedKeys = streamKeys(normalizedStreams);
    const details = {};
    const checks = {};

    expectedKeys.forEach((key) => {
      details[key] = {
        top: summarizeStream(topStreams?.[key]),
        nested: summarizeStream(nestedStreams?.[key]),
        normalized: summarizeStream(normalizedStreams?.[key]),
      };
    });

    Object.entries(expectedStreams).forEach(([key, expected]) => {
      const summary = details[key]?.top ?? summarizeStream(topStreams?.[key]);
      const exp = expected && typeof expected === "object" ? expected : {};
      checks[key] = {
        capacity: exp.capacity === undefined ? true : summary.capacity === Number(exp.capacity),
        head: exp.head === undefined ? true : summary.head === Number(exp.head),
        len: exp.len === undefined ? true : summary.len === Number(exp.len),
        buffer_prefix: Array.isArray(exp.buffer_prefix)
          ? JSON.stringify(summary.buffer_prefix.slice(0, exp.buffer_prefix.length)) ===
            JSON.stringify(exp.buffer_prefix)
          : true,
      };
    });

    const report = {
      schema: "seamgrim.wasm.streams_serialization.v1",
      raw_schema: String(rawState?.schema ?? ""),
      normalized_schema: String(normalizedState?.schema ?? ""),
      step_count: stepCount,
      expected_keys: expectedKeys,
      top_keys: topKeys,
      nested_keys: nestedKeys,
      normalized_keys: normalizedKeys,
      top_has_expected_keys: expectedKeys.every((key) => topKeys.includes(key)),
      nested_has_expected_keys: expectedKeys.every((key) => nestedKeys.includes(key)),
      normalized_has_expected_keys: expectedKeys.every((key) => normalizedKeys.includes(key)),
      top_nested_keys_equal: JSON.stringify(topKeys) === JSON.stringify(nestedKeys),
      top_normalized_keys_equal: JSON.stringify(topKeys) === JSON.stringify(normalizedKeys),
      top_nested_streams_equal_for_expected: expectedKeys.every(
        (key) => JSON.stringify(topStreams?.[key] ?? null) === JSON.stringify(nestedStreams?.[key] ?? null),
      ),
      stream_details: details,
      expected_checks: checks,
      expected_checks_all_passed: Object.values(checks).every((entry) =>
        Object.values(entry ?? {}).every(Boolean),
      ),
      state_value_keys: streamKeys(rawState?.state?.resources?.value ?? {}),
      state_fixed64_keys: streamKeys(rawState?.state?.resources?.fixed64 ?? {}),
    };

    return {
      "expected/streams_serialization.detjson": report,
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runObservationChannelsV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "observation.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    client.resetParsed(false);
    const paramKey = String(fixture?.set_param?.key ?? fixture?.set_param?.name ?? "").trim();
    if (!paramKey) throw new Error("observation.detjson: set_param.key/name 누락");
    const paramValue = fixture?.set_param?.value;
    client.setParamParsed(paramKey, paramValue);

    const rawState = JSON.parse(vm.get_state_json());
    const columnsPayload = client.columnsParsed();
    const stepRaw = JSON.parse(vm.step_one());

    const columnsKeys = channelKeys(columnsPayload?.columns);
    const stateKeys = channelKeys(rawState?.channels);
    const stateChannelKeys = channelKeys(rawState?.state?.channels);
    const stepKeys = channelKeys(stepRaw?.channels);
    const stepStateKeys = channelKeys(stepRaw?.state?.channels);
    const row = Array.isArray(rawState?.row) ? rawState.row : [];
    const stateRow = Array.isArray(rawState?.state?.row) ? rawState.state.row : [];
    const stepRow = Array.isArray(stepRaw?.row) ? stepRaw.row : [];
    const stepStateRow = Array.isArray(stepRaw?.state?.row) ? stepRaw.state.row : [];

    const report = {
      schema: "seamgrim.wasm.observation_channels.v1",
      raw_schema_state: String(rawState?.schema ?? ""),
      raw_schema_step: String(stepRaw?.schema ?? ""),
      columns_count: columnsKeys.length,
      state_channels_count: stateKeys.length,
      state_state_channels_count: stateChannelKeys.length,
      step_channels_count: stepKeys.length,
      step_state_channels_count: stepStateKeys.length,
      columns_vs_state_equal: JSON.stringify(columnsKeys) === JSON.stringify(stateKeys),
      state_vs_nested_state_equal: JSON.stringify(stateKeys) === JSON.stringify(stateChannelKeys),
      columns_vs_step_equal: JSON.stringify(columnsKeys) === JSON.stringify(stepKeys),
      step_vs_nested_step_equal: JSON.stringify(stepKeys) === JSON.stringify(stepStateKeys),
      state_row_matches_channels: row.length === stateKeys.length,
      nested_state_row_matches_channels: stateRow.length === stateChannelKeys.length,
      step_row_matches_channels: stepRow.length === stepKeys.length,
      nested_step_row_matches_channels: stepStateRow.length === stepStateKeys.length,
      contains_param_key: stateKeys.includes(paramKey) && columnsKeys.includes(paramKey),
      contains_tick_key: stateKeys.includes("프레임수"),
    };

    return {
      "expected/observation_channels.detjson": report,
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runTickLoopV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "tick_loop.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");
  const blocks = Array.isArray(fixture.step_blocks) ? fixture.step_blocks : [1, 2, 3];
  const normalizedBlocks = blocks
    .map((n) => Math.max(1, Math.floor(Number(n ?? 1))))
    .filter((n) => Number.isFinite(n) && n > 0);

  const runOnce = async () => {
    const { vm, client } = await makeClient(runtime, lesson);
    try {
      client.resetParsed(false);
      const rows = [];
      for (const n of normalizedBlocks) {
        let last = null;
        for (let i = 0; i < n; i += 1) {
          last = client.stepOneParsed();
        }
        rows.push({
          tick: Number(last?.tick_id ?? -1),
          state_hash: String(last?.state_hash ?? ""),
        });
      }
      return rows;
    } finally {
      if (typeof vm.free === "function") vm.free();
    }
  };

  const rowsA = await runOnce();
  const rowsB = await runOnce();
  const monotonicTick = rowsA.every((row, idx) => idx === 0 || row.tick > rowsA[idx - 1].tick);
  const deterministicEqual = JSON.stringify(rowsA) === JSON.stringify(rowsB);

  return {
    "expected/tick_loop.detjson": {
      schema: "seamgrim.wasm.tick_loop.v1",
      step_blocks: normalizedBlocks,
      rows_a: rowsA,
      rows_b: rowsB,
      monotonic_tick: monotonicTick,
      deterministic_equal: deterministicEqual,
    },
  };
}

async function runResetV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "reset.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");
  const paramKey = String(fixture?.set_param?.key ?? fixture?.set_param?.name ?? "g").trim();
  const paramValue = fixture?.set_param?.value ?? 12.0;
  const warmupSteps = Math.max(1, Math.floor(Number(fixture?.warmup_steps ?? 2)));

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    client.resetParsed(false);
    const initialHash = String(client.getStateHash() ?? "");

    for (let i = 0; i < warmupSteps; i += 1) client.stepOneParsed();
    const steppedHash = String(client.getStateHash() ?? "");
    const setResult = client.setParamParsed(paramKey, paramValue);
    const afterParamHash = String(client.getStateHash() ?? "");

    const resetKeep = client.resetParsed(true);
    const keepCols = Array.isArray(resetKeep?.columns) ? resetKeep.columns : [];
    const keepRow = Array.isArray(resetKeep?.row) ? resetKeep.row : [];
    const keepMap = indexRow(keepCols, keepRow);
    const keepHasKey = Object.prototype.hasOwnProperty.call(keepMap, paramKey);

    const resetDrop = client.resetParsed(false);
    const dropCols = Array.isArray(resetDrop?.columns) ? resetDrop.columns : [];
    const dropRow = Array.isArray(resetDrop?.row) ? resetDrop.row : [];
    const dropMap = indexRow(dropCols, dropRow);
    const dropHasKey = Object.prototype.hasOwnProperty.call(dropMap, paramKey);
    const dropHash = String(resetDrop?.state_hash ?? "");

    return {
      "expected/reset.detjson": {
        schema: "seamgrim.wasm.reset.v1",
        param_key: paramKey,
        initial_hash: initialHash,
        stepped_hash: steppedHash,
        set_param_ok: Boolean(setResult?.ok),
        after_param_hash: afterParamHash,
        reset_keep_has_key: keepHasKey,
        reset_keep_value: keepMap[paramKey] ?? null,
        reset_drop_has_key: dropHasKey,
        reset_drop_hash: dropHash,
        initial_equals_drop: initialHash !== "" && initialHash === dropHash,
      },
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runStateApplyV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "state_apply.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    client.resetParsed(false);
    const fullRaw = JSON.parse(vm.get_state_json());
    const patchRaw = JSON.parse(vm.step_one());

    const normalizedFull = runtime.runtimeState.normalizeWasmStatePayload(fullRaw);
    const normalizedPatch = runtime.runtimeState.normalizeWasmStatePayload(patchRaw);
    const fullObs = runtime.runtimeState.extractObservationChannelsFromState(fullRaw);
    const patchObs = runtime.runtimeState.extractObservationChannelsFromState(patchRaw);

    let modePatch = "";
    let modeFull = "";
    runtime.uiCommon.dispatchWasmStateApply({
      stateJson: patchRaw,
      patchMode: true,
      onPatch: () => {
        modePatch = "patch";
      },
      onFull: () => {
        modePatch = "full";
      },
    });
    runtime.uiCommon.dispatchWasmStateApply({
      stateJson: patchRaw,
      patchMode: false,
      onPatch: () => {
        modeFull = "patch";
      },
      onFull: () => {
        modeFull = "full";
      },
    });

    return {
      "expected/state_apply.detjson": {
        schema: "seamgrim.wasm.state_apply.v1",
        patch_count: Array.isArray(patchRaw?.patch) ? patchRaw.patch.length : 0,
        patch_dispatch_mode: modePatch,
        full_dispatch_mode: modeFull,
        normalized_full_schema: String(normalizedFull?.schema ?? ""),
        normalized_patch_schema: String(normalizedPatch?.schema ?? ""),
        full_channels_count: fullObs.channels.length,
        patch_channels_count: patchObs.channels.length,
        full_row_matches_channels: fullObs.row.length === fullObs.channels.length,
        patch_row_matches_channels: patchObs.row.length === patchObs.channels.length,
        state_hash_changes_after_step:
          String(fullRaw?.state_hash ?? "") !== "" &&
          String(patchRaw?.state_hash ?? "") !== "" &&
          String(fullRaw?.state_hash ?? "") !== String(patchRaw?.state_hash ?? ""),
      },
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runHashVerifyV1(packDir, runtime) {
  const fixturePath = path.join(packDir, "fixtures", "hash_verify.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");
  const steps = Math.max(1, Math.floor(Number(fixture.steps ?? 3)));
  const paramKey = String(fixture?.set_param?.key ?? fixture?.set_param?.name ?? "g").trim();
  const paramValue = fixture?.set_param?.value ?? 21.0;

  const runHashSequence = async (setParam = false) => {
    const { vm, client } = await makeClient(runtime, lesson);
    try {
      client.resetParsed(false);
      if (setParam) client.setParamParsed(paramKey, paramValue);
      const rows = [];
      for (let i = 0; i < steps; i += 1) {
        const parsed = client.stepOneParsed();
        rows.push(String(parsed?.state_hash ?? ""));
      }
      return rows;
    } finally {
      if (typeof vm.free === "function") vm.free();
    }
  };

  const baselineA = await runHashSequence(false);
  const baselineB = await runHashSequence(false);
  const changed = await runHashSequence(true);
  const baselineEqual = JSON.stringify(baselineA) === JSON.stringify(baselineB);
  const baselineVsChangedDifferent = JSON.stringify(baselineA) !== JSON.stringify(changed);

  return {
    "expected/hash_verify.detjson": {
      schema: "seamgrim.wasm.hash_verify.v1",
      steps,
      baseline_a: baselineA,
      baseline_b: baselineB,
      changed,
      baseline_equal: baselineEqual,
      baseline_vs_changed_different: baselineVsChangedDifferent,
    },
  };
}

async function runWasmCanonContractV1(packDir, rootDir) {
  const runtimeMod = await initCanonRuntime(rootDir);
  const wasmBytes = await fs.readFile(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"),
  );
  const canon = await runtimeMod.createWasmCanon({
    cacheBust: 0,
    initInput: wasmBytes,
  });

  const fixturePath = path.join(packDir, "fixtures", "contract.detjson");
  const fixture = await readJson(fixturePath);
  const flatRel = String(fixture.flat_fixture ?? "fixtures/guseong_input.ddn");
  const maegimRel = String(fixture.maegim_fixture ?? "fixtures/maegim_input.ddn");
  const alrimRel = String(fixture.alrim_fixture ?? "fixtures/alrim_input.ddn");
  const flatSource = await fs.readFile(path.join(packDir, flatRel), "utf8");
  const maegimSource = await fs.readFile(path.join(packDir, maegimRel), "utf8");
  const alrimSource = await fs.readFile(path.join(packDir, alrimRel), "utf8");

  const flat = await canon.canonFlatJson(flatSource);
  const maegim = await canon.canonMaegimPlan(maegimSource);
  const alrim = await canon.canonAlrimPlan(alrimSource);
  const flatInstances = Array.isArray(flat?.instances) ? flat.instances : [];
  const flatLinks = Array.isArray(flat?.links) ? flat.links : [];
  const flatTopo = Array.isArray(flat?.topo_order) ? flat.topo_order.map((item) => String(item)) : [];
  const controls = Array.isArray(maegim?.controls) ? maegim.controls : [];
  const handlers = Array.isArray(alrim?.handlers) ? alrim.handlers : [];

  return {
    "expected/canon_contract.detjson": {
      schema: "seamgrim.wasm.canon_contract.v1",
      build_info: String(canon.getBuildInfo?.() ?? ""),
      flat_schema: String(flat?.schema ?? ""),
      flat_instance_count: flatInstances.length,
      flat_link_count: flatLinks.length,
      flat_topo_order: flatTopo,
      flat_instance_names: flatInstances.map((item) => String(item?.name ?? "")).filter(Boolean),
      maegim_schema: String(maegim?.schema ?? ""),
      maegim_control_count: controls.length,
      maegim_control_names: controls.map((item) => String(item?.name ?? "")).filter(Boolean),
      maegim_decl_kinds: controls.map((item) => String(item?.decl_kind ?? "")).filter(Boolean),
      maegim_has_split_count: controls.some(
        (item) => typeof item?.split_count_expr_canon === "string" && item.split_count_expr_canon.length > 0,
      ),
      alrim_schema: String(alrim?.schema ?? ""),
      alrim_handler_count: handlers.length,
      alrim_handler_kinds: handlers.map((item) => String(item?.kind ?? "")).filter(Boolean),
      alrim_handler_scopes: handlers.map((item) => String(item?.scope ?? "")).filter(Boolean),
      alrim_handler_orders: handlers.map((item) => Number(item?.order ?? -1)),
    },
  };
}

function summarizeSliderSpecs(specs) {
  const normalizeNumber = (value) => {
    const numeric = Number(value);
    if (!Number.isFinite(numeric)) return null;
    return Number(numeric.toFixed(12));
  };
  return (Array.isArray(specs) ? specs : []).map((spec) => ({
    name: String(spec?.name ?? ""),
    type: String(spec?.type ?? ""),
    value: normalizeNumber(spec?.value),
    min: normalizeNumber(spec?.min),
    max: normalizeNumber(spec?.max),
    step: normalizeNumber(spec?.step),
    split_count:
      spec?.splitCount === null || spec?.splitCount === undefined ? null : normalizeNumber(spec?.splitCount),
    decl_kind: String(spec?.declKind ?? ""),
    source: String(spec?.source ?? ""),
  }));
}

function extractControlLines(text) {
  return normalizeNewlines(text)
    .split("\n")
    .map((line) => String(line ?? "").trim())
    .filter((line) => line.startsWith("g:수") || line.startsWith("theta0:수") || line.startsWith("tick:변수") || line.startsWith("각도:변수"));
}

function parseCharymTokens(rawValue) {
  const text = String(rawValue ?? "").trim();
  if (!text.startsWith("차림[")) return [];
  try {
    const parsed = JSON.parse(text.replace(/^차림/u, ""));
    return Array.isArray(parsed) ? parsed.map((item) => String(item ?? "")) : [];
  } catch (_) {
    return [];
  }
}

function coerceScalarText(rawValue) {
  const text = String(rawValue ?? "").trim();
  if (/^[+-]?(?:\d+(?:\.\d+)?|\.\d+)$/u.test(text)) {
    const numeric = Number(text);
    if (Number.isFinite(numeric)) return numeric;
  }
  return text;
}

function rowsFromTableRowTokens(tokens) {
  const list = Array.isArray(tokens) ? tokens.map((item) => String(item ?? "")) : [];
  const rows = [];
  let idx = 0;
  while (idx < list.length) {
    if (list[idx] !== "table.row") {
      idx += 1;
      continue;
    }
    idx += 1;
    const row = {};
    while (idx < list.length && list[idx] !== "table.row") {
      const key = String(list[idx] ?? "").trim();
      const value = idx + 1 < list.length ? list[idx + 1] : "";
      if (key) {
        row[key] = coerceScalarText(value);
      }
      idx += 2;
    }
    if (Object.keys(row).length > 0) {
      rows.push(row);
    }
  }
  return rows;
}

function createFakeCanvas2dForSmoke() {
  const calls = {
    arc: 0,
    lineTo: 0,
    fill: 0,
    stroke: 0,
    fillText: [],
  };
  const ctx = {
    fillStyle: "",
    strokeStyle: "",
    lineWidth: 1,
    globalAlpha: 1,
    font: "",
    clearRect() {},
    fillRect() {},
    beginPath() {},
    closePath() {},
    moveTo() {},
    lineTo() {
      calls.lineTo += 1;
    },
    stroke() {
      calls.stroke += 1;
    },
    fill() {
      calls.fill += 1;
    },
    setLineDash() {},
    arc() {
      calls.arc += 1;
    },
    fillText(text) {
      calls.fillText.push(String(text ?? ""));
    },
    strokeRect() {},
  };
  return {
    width: 320,
    height: 180,
    getContext(type) {
      if (type !== "2d") return null;
      return ctx;
    },
    calls,
    ctx,
  };
}

function ensureDomInputElementStubs() {
  const previous = {
    Element: globalThis.Element,
    HTMLInputElement: globalThis.HTMLInputElement,
    HTMLTextAreaElement: globalThis.HTMLTextAreaElement,
    HTMLSelectElement: globalThis.HTMLSelectElement,
  };
  if (typeof globalThis.Element !== "function") {
    globalThis.Element = class {};
  }
  if (typeof globalThis.HTMLInputElement !== "function") {
    globalThis.HTMLInputElement = class extends globalThis.Element {};
  }
  if (typeof globalThis.HTMLTextAreaElement !== "function") {
    globalThis.HTMLTextAreaElement = class extends globalThis.Element {};
  }
  if (typeof globalThis.HTMLSelectElement !== "function") {
    globalThis.HTMLSelectElement = class extends globalThis.Element {};
  }
  return () => {
    globalThis.Element = previous.Element;
    globalThis.HTMLInputElement = previous.HTMLInputElement;
    globalThis.HTMLTextAreaElement = previous.HTMLTextAreaElement;
    globalThis.HTMLSelectElement = previous.HTMLSelectElement;
  };
}

function toStepKeyList(rawValue) {
  if (Array.isArray(rawValue)) {
    return rawValue.map((item) => String(item ?? "").trim()).filter(Boolean);
  }
  const text = String(rawValue ?? "").trim();
  return text ? [text] : [];
}

function createRuntimeKeyEvent(key) {
  return {
    key,
    code: key,
    target: {},
    prevented: false,
    altKey: false,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    preventDefault() {
      this.prevented = true;
    },
  };
}

function summarizeSpace2dPrimaryShape(space2d) {
  const drawShape = Array.isArray(space2d?.drawlist) ? space2d.drawlist[0] ?? null : null;
  const shape = drawShape ?? (Array.isArray(space2d?.shapes) ? space2d.shapes[0] ?? null : null);
  if (!shape || typeof shape !== "object") return null;
  const numericKeys = ["x", "y", "r", "width"];
  const out = {
    kind: String(shape?.kind ?? ""),
    fill: String(shape?.fill ?? ""),
    stroke: String(shape?.stroke ?? ""),
  };
  numericKeys.forEach((key) => {
    const num = Number(shape?.[key]);
    out[key] = Number.isFinite(num) ? Number(num.toFixed(6)) : null;
  });
  return out;
}

async function runMaegimSliderSmokeV1(packDir, rootDir) {
  const runtimeUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "index.js"),
  ).href;
  const controlParserUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "components", "control_parser.js"),
  ).href;
  const sliderPanelUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "components", "slider_panel.js"),
  ).href;
  const runtimeMod = await import(runtimeUrl);
  const controlParser = await import(controlParserUrl);
  const sliderPanelMod = await import(sliderPanelUrl);
  if (typeof runtimeMod.createLessonCanonHydrator !== "function") {
    throw new Error("createLessonCanonHydrator export 누락");
  }
  if (typeof runtimeMod.createWasmCanon !== "function") {
    throw new Error("createWasmCanon export 누락");
  }
  if (typeof controlParser.applyControlValuesToDdnText !== "function") {
    throw new Error("applyControlValuesToDdnText export 누락");
  }
  if (typeof sliderPanelMod.SliderPanel !== "function") {
    throw new Error("SliderPanel export 누락");
  }

  const fixturePath = path.join(packDir, "fixtures", "contract.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const legacyRel = String(fixture.legacy_fixture ?? "fixtures/legacy_lesson.ddn");
  const lessonText = await fs.readFile(path.join(packDir, lessonRel), "utf8");
  const legacyText = await fs.readFile(path.join(packDir, legacyRel), "utf8");
  const applyValues = fixture?.apply_values && typeof fixture.apply_values === "object" ? fixture.apply_values : {};
  const preservedControlJson = String(fixture?.preserve_maegim_control_json ?? '{"schema":"keep.me.v1"}');

  const wasmBytes = await fs.readFile(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"),
  );
  const hydrator = runtimeMod.createLessonCanonHydrator({
    cacheBust: 0,
    initInput: wasmBytes,
  });
  const canon = await runtimeMod.createWasmCanon({
    cacheBust: 0,
    initInput: wasmBytes,
  });

  const hydrated = await hydrator.hydrateLessonCanon({
    id: "maegim-slider-smoke",
    title: "Maegim Slider Smoke",
    ddnText: lessonText,
    maegimControlJson: "",
  });
  const preserved = await hydrator.hydrateLessonCanon({
    id: "maegim-slider-preserve",
    title: "Maegim Slider Preserve",
    ddnText: lessonText,
    maegimControlJson: preservedControlJson,
  });
  const maegimPlan = JSON.parse(String(hydrated?.maegimControlJson ?? "{}"));

  const statusEl = { textContent: "" };
  const sliderPanel = new sliderPanelMod.SliderPanel({ container: null, statusEl });
  const sliderResult = sliderPanel.parseFromDdn(hydrated.ddnText ?? lessonText, {
    maegimControlJson: hydrated?.maegimControlJson ?? "",
  });
  sliderPanel.setValues(applyValues);
  const preservedReparse = sliderPanel.parseFromDdn(hydrated.ddnText ?? lessonText, {
    preserveValues: true,
    maegimControlJson: hydrated?.maegimControlJson ?? "",
  });
  const appliedText = controlParser.applyControlValuesToDdnText(hydrated.ddnText ?? lessonText, applyValues);

  const legacyStatusEl = { textContent: "" };
  const legacyPanel = new sliderPanelMod.SliderPanel({ container: null, statusEl: legacyStatusEl });
  const legacyResult = legacyPanel.parseFromDdn(legacyText, {
    maegimControlJson: "{broken",
  });

  return {
    "expected/maegim_slider.detjson": {
      schema: "seamgrim.web.maegim_slider_smoke.v1",
      build_info: String(canon.getBuildInfo?.() ?? ""),
      wasm_plan_schema: String(maegimPlan?.schema ?? ""),
      wasm_control_names: Array.isArray(maegimPlan?.controls)
        ? maegimPlan.controls.map((item) => String(item?.name ?? "")).filter(Boolean)
        : [],
      wasm_warning_codes: Array.isArray(maegimPlan?.warnings)
        ? maegimPlan.warnings.map((item) => String(item?.code ?? "")).filter(Boolean)
        : [],
      hydrated_control_json_present: Boolean(String(hydrated?.maegimControlJson ?? "").trim()),
      hydrated_preserves_existing_json: String(preserved?.maegimControlJson ?? "") === preservedControlJson,
      slider_source: String(sliderResult?.source ?? ""),
      slider_status: String(statusEl.textContent ?? ""),
      slider_axis_keys: Array.isArray(sliderResult?.axisKeys) ? sliderResult.axisKeys.map((item) => String(item)) : [],
      slider_default_axis_key: String(sliderResult?.defaultAxisKey ?? ""),
      slider_default_x_axis_key: String(sliderResult?.defaultXAxisKey ?? ""),
      slider_specs: summarizeSliderSpecs(sliderResult?.specs),
      preserved_values_after_reparse: preservedReparse?.values && typeof preservedReparse.values === "object"
        ? preservedReparse.values
        : {},
      applied_control_lines: extractControlLines(appliedText),
      legacy_fallback_source: String(legacyResult?.source ?? ""),
      legacy_status: String(legacyStatusEl.textContent ?? ""),
      legacy_warning_codes: Array.isArray(legacyResult?.warnings)
        ? legacyResult.warnings.map((item) => String(item?.code ?? "")).filter(Boolean)
        : [],
      legacy_specs: summarizeSliderSpecs(legacyResult?.specs),
    },
  };
}

async function runTempLessonSmokeV1(packDir, rootDir) {
  const outputLinesKey = "보개_출력_줄들";
  const runtime = await initRuntime(rootDir);
  const preprocessUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "ddn_preprocess.js"),
  ).href;
  const runScreenUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "screens", "run.js"),
  ).href;
  const preprocessMod = await import(preprocessUrl);
  const runScreenMod = await import(runScreenUrl);
  if (typeof preprocessMod.preprocessDdnText !== "function") {
    throw new Error("preprocessDdnText export 누락");
  }
  if (typeof runScreenMod.normalizeRuntimeTableView !== "function") {
    throw new Error("normalizeRuntimeTableView export 누락");
  }
  if (typeof runScreenMod.renderRuntimeTable !== "function") {
    throw new Error("renderRuntimeTable export 누락");
  }
  if (typeof runScreenMod.summarizeRuntimeTableView !== "function") {
    throw new Error("summarizeRuntimeTableView export 누락");
  }

  const fixturePath = path.join(packDir, "fixtures", "contract.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonText = await fs.readFile(path.join(packDir, lessonRel), "utf8");
  const pre = preprocessMod.preprocessDdnText(lessonText);
  const preparedBody = stripMetaHeader(pre?.bodyText ?? "").trim();

  const { vm, client } = await makeClient(runtime, preparedBody);
  try {
    client.resetParsed(false);
    const stepped = client.stepOneParsed();
    const obs = runtime.runtimeState.extractObservationChannelsFromState(stepped);
    const rawOutputLines = String(
      stepped?.resources?.value?.[outputLinesKey] ??
        stepped?.resources?.value?.보개_출력_줄들 ??
        "",
    ).trim();
    const outputTokens = parseCharymTokens(rawOutputLines);
    const rows = rowsFromTableRowTokens(outputTokens);
    const columns = [
      { key: "t", type: "number" },
      { key: "celsius", type: "string" },
      { key: "fahrenheit", type: "string" },
    ];
    const table = {
      schema: "seamgrim.table.v0",
      meta: { source: "temperature-smoke" },
      columns,
      rows,
    };
    const normalizedTable = runScreenMod.normalizeRuntimeTableView(table);
    const tableSummary = runScreenMod.summarizeRuntimeTableView(normalizedTable);
    const renderContainer = { innerHTML: "", clientWidth: 480 };
    const rendered = runScreenMod.renderRuntimeTable(renderContainer, table);

    return {
      "expected/temp_lesson.detjson": {
        schema: "seamgrim.web.temp_lesson_smoke.v1",
        prepared_body_has_temp_formats:
          preparedBody.includes("@.1C") && preparedBody.includes("@.1F"),
        parse_diag_codes: Array.isArray(pre?.diags)
          ? pre.diags.map((item) => String(item?.code ?? "")).filter(Boolean)
          : [],
        observation_keys: Object.keys(obs?.all_values ?? {}).map((key) => String(key)).sort((a, b) => a.localeCompare(b)),
        observation_temperature_strings: {
          celsius: String(obs?.all_values?.섭씨안내 ?? ""),
          fahrenheit: String(obs?.all_values?.화씨안내 ?? ""),
        },
        output_token_count: outputTokens.length,
        row_count: rows.length,
        first_row: rows[0] ?? null,
        last_row: rows[rows.length - 1] ?? null,
        table_summary: tableSummary,
        rendered_table: Boolean(rendered),
        rendered_contains_celsius: renderContainer.innerHTML.includes("@C"),
        rendered_contains_fahrenheit: renderContainer.innerHTML.includes("@F"),
        rendered_html_excerpt: String(renderContainer.innerHTML ?? "").slice(0, 240),
      },
    };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runMoyangRenderSmokeV1(packDir, rootDir) {
  const runtime = await initRuntime(rootDir);
  const preprocessUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "ddn_preprocess.js"),
  ).href;
  const pageCommonUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm_page_common.js"),
  ).href;
  const preprocessMod = await import(preprocessUrl);
  const pageCommonMod = await import(pageCommonUrl);
  if (typeof preprocessMod.preprocessDdnText !== "function") {
    throw new Error("preprocessDdnText export 누락");
  }
  if (typeof pageCommonMod.renderSpace2dCanvas2d !== "function") {
    throw new Error("renderSpace2dCanvas2d export 누락");
  }

  const fixturePath = path.join(packDir, "fixtures", "contract.detjson");
  const fixture = await readJson(fixturePath);
  const cases = Array.isArray(fixture?.cases) ? fixture.cases : [];
  const tickCountRaw = Number(fixture?.tick_count ?? 3);
  const tickCount = Number.isFinite(tickCountRaw) && tickCountRaw > 0 ? Math.floor(tickCountRaw) : 3;

  const results = [];
  for (const row of cases) {
    const id = String(row?.id ?? "").trim();
    const inputRel = String(row?.ddn_fixture ?? "").trim();
    if (!id || !inputRel) continue;
    const lessonText = await fs.readFile(path.join(packDir, inputRel), "utf8");
    const pre = preprocessMod.preprocessDdnText(lessonText);
    const preparedBody = stripMetaHeader(pre?.bodyText ?? "").trim();
    const { vm, client } = await makeClient(runtime, preparedBody);
    try {
      client.resetParsed(false);
      let stepped = null;
      for (let i = 0; i < tickCount; i += 1) {
        stepped = client.stepOneParsed();
      }
      const views = runtime.runtimeState.extractStructuredViewsFromState(stepped, { preferPatch: false });
      const shape = Array.isArray(views?.space2d?.shapes) ? views.space2d.shapes[0] ?? null : null;
      const canvas = createFakeCanvas2dForSmoke();
      const rendered = pageCommonMod.renderSpace2dCanvas2d({
        canvas,
        space2d: views?.space2d ?? null,
        primitiveSource: "shapes",
        viewState: { autoFit: true, zoom: 1, panPx: 0, panPy: 0 },
        showGrid: false,
        showAxis: false,
      });
      results.push({
        id,
        state_hash: String(stepped?.state_hash ?? ""),
        view_hash: String(stepped?.view_hash ?? ""),
        space2d_source: String(views?.space2d?.meta?.source ?? ""),
        shape: shape
          ? {
              kind: String(shape?.kind ?? ""),
              x: Number(Number(shape?.x ?? 0).toFixed(6)),
              y: Number(Number(shape?.y ?? 0).toFixed(6)),
              r: Number(Number(shape?.r ?? 0).toFixed(6)),
              fill: String(shape?.fill ?? ""),
              stroke: String(shape?.stroke ?? ""),
              width: Number(Number(shape?.width ?? 0).toFixed(6)),
            }
          : null,
        rendered,
        render_calls: {
          arc: canvas.calls.arc,
          fill: canvas.calls.fill,
          stroke: canvas.calls.stroke,
          line_to: canvas.calls.lineTo,
          fill_text_count: canvas.calls.fillText.length,
        },
      });
    } finally {
      if (typeof vm.free === "function") vm.free();
    }
  }

  const byId = new Map(results.map((row) => [row.id, row]));
  const a = byId.get("a") ?? null;
  const b = byId.get("b") ?? null;
  const c = byId.get("c") ?? null;

  return {
    "expected/moyang_render.detjson": {
      schema: "seamgrim.web.moyang_render_smoke.v1",
      tick_count: tickCount,
      cases: results,
      state_hash_equal_ab: Boolean(a && b && a.state_hash === b.state_hash),
      state_hash_equal_ac: Boolean(a && c && a.state_hash === c.state_hash),
      radius_diff_ab: Boolean(a && b && Number(a?.shape?.r) !== Number(b?.shape?.r)),
      fill_diff_ac: Boolean(a && c && String(a?.shape?.fill ?? "") !== String(c?.shape?.fill ?? "")),
      rendered_all: results.length > 0 && results.every((row) => row.rendered === true),
      arcs_all: results.map((row) => Number(row?.render_calls?.arc ?? 0)),
    },
  };
}

async function runInteractiveEventSmokeV1(packDir, rootDir) {
  const runtime = await initRuntime(rootDir);
  const preprocessUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "runtime", "ddn_preprocess.js"),
  ).href;
  const runScreenUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "screens", "run.js"),
  ).href;
  const pageCommonUrl = pathToFileURL(
    path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm_page_common.js"),
  ).href;
  const preprocessMod = await import(preprocessUrl);
  const runScreenMod = await import(runScreenUrl);
  const pageCommonMod = await import(pageCommonUrl);
  if (typeof preprocessMod.preprocessDdnText !== "function") {
    throw new Error("preprocessDdnText export 누락");
  }
  if (typeof runScreenMod.RunScreen !== "function") {
    throw new Error("RunScreen export 누락");
  }
  if (typeof pageCommonMod.renderSpace2dCanvas2d !== "function") {
    throw new Error("renderSpace2dCanvas2d export 누락");
  }

  const fixturePath = path.join(packDir, "fixtures", "contract.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonText = await fs.readFile(path.join(packDir, lessonRel), "utf8");
  const pre = preprocessMod.preprocessDdnText(lessonText);
  const preparedBody = stripMetaHeader(pre?.bodyText ?? "").trim();
  const steps = Array.isArray(fixture?.steps) ? fixture.steps : [];

  const restoreDomStubs = ensureDomInputElementStubs();
  const rootStub = {
    classList: {
      contains() {
        return false;
      },
    },
  };
  const runScreen = new runScreenMod.RunScreen({
    root: rootStub,
    wasmState: {
      fpsLimit: Number(fixture?.fps_limit ?? 30),
      dtMax: Number(fixture?.dt_max ?? 0.1),
      inputEnabled: true,
    },
  });
  runScreen.screenVisible = true;

  const { vm, client } = await makeClient(runtime, preparedBody);
  try {
    const results = [];
    for (const row of steps) {
      const id = String(row?.id ?? "").trim();
      if (!id) continue;
      if (typeof row?.input_enabled === "boolean") {
        runScreen.wasmState.inputEnabled = row.input_enabled;
      }
      toStepKeyList(row?.keyup).forEach((key) => {
        runScreen.handleRuntimeInputKeyUp(createRuntimeKeyEvent(key));
      });
      toStepKeyList(row?.keydown).forEach((key) => {
        runScreen.handleRuntimeInputKeyDown(createRuntimeKeyEvent(key));
      });

      const stepped = runScreen.stepClientOne(client);
      const resolvedState = runScreen.resolveSteppedState(client, stepped.state);
      const observation = runtime.runtimeState.extractObservationChannelsFromState(resolvedState);
      const indexed = indexRow(observation?.channels, observation?.row);
      const views = runtime.runtimeState.extractStructuredViewsFromState(resolvedState, { preferPatch: false });
      const space2d = views?.space2d ?? null;
      const canvas = createFakeCanvas2dForSmoke();
      const primitiveSource = Array.isArray(space2d?.drawlist) && space2d.drawlist.length > 0 ? "drawlist" : "shapes";
      const rendered = pageCommonMod.renderSpace2dCanvas2d({
        canvas,
        space2d,
        primitiveSource,
        viewState: { autoFit: true, zoom: 1, panPx: 0, panPy: 0 },
        showGrid: false,
        showAxis: false,
      });
      results.push({
        id,
        input_enabled: Boolean(runScreen.wasmState?.inputEnabled ?? true),
        input: stepped.input,
        prevented: {
          keydown: toStepKeyList(row?.keydown).length > 0,
          keyup: toStepKeyList(row?.keyup).length > 0,
        },
        state_hash: String(resolvedState?.state_hash ?? ""),
        view_hash: String(resolvedState?.view_hash ?? ""),
        flags: {
          left: Boolean(indexed.left ?? false),
          right: Boolean(indexed.right ?? false),
          spin: Boolean(indexed.spin ?? false),
        },
        shape: summarizeSpace2dPrimaryShape(space2d),
        rendered,
        render_calls: {
          arc: canvas.calls.arc,
          fill: canvas.calls.fill,
          stroke: canvas.calls.stroke,
          line_to: canvas.calls.lineTo,
        },
      });
    }

    const byId = new Map(results.map((row) => [row.id, row]));
    const baseline = byId.get("baseline") ?? null;
    const rightDown = byId.get("right_down") ?? null;
    const rightHold = byId.get("right_hold") ?? null;
    const rightUp = byId.get("right_up") ?? null;
    const leftDown = byId.get("left_down") ?? null;
    const upDown = byId.get("up_down") ?? null;
    const upHold = byId.get("up_hold") ?? null;
    const disabledRight = byId.get("disabled_right") ?? null;

    return {
      "expected/interactive_event.detjson": {
        schema: "seamgrim.web.interactive_event_smoke.v1",
        parse_diag_codes: Array.isArray(pre?.diags)
          ? pre.diags.map((item) => String(item?.code ?? "")).filter(Boolean)
          : [],
        prepared_body_uses_drawlist: preparedBody.includes("보개_그림판_목록"),
        steps: results,
        right_moves_view: Boolean(rightDown && Number(rightDown?.shape?.x) === 1),
        hold_keeps_right_pressed: Boolean(rightHold && rightHold.flags.right === true && rightHold.input.lastKey === ""),
        release_clears_right: Boolean(rightUp && rightUp.flags.right === false && Number(rightUp?.shape?.x) === 0),
        left_moves_view: Boolean(leftDown && Number(leftDown?.shape?.x) === -1),
        spin_requires_pulse: Boolean(
          upDown &&
            upHold &&
            Number(upDown?.shape?.r) > Number(upHold?.shape?.r) &&
            upDown.flags.spin === true &&
            upHold.flags.spin === false,
        ),
        disabled_input_blocks_event: Boolean(
          baseline &&
            disabledRight &&
            disabledRight.input_enabled === false &&
            disabledRight.input.keys === 0 &&
            Number(disabledRight?.shape?.x) === Number(baseline?.shape?.x),
        ),
        rendered_all: results.length > 0 && results.every((row) => row.rendered === true),
      },
    };
  } finally {
    restoreDomStubs();
    if (typeof vm.free === "function") vm.free();
  }
}

async function runObservationManifestPack(packDir, runtime, kind) {
  const fixturePath = path.join(packDir, "fixtures", "manifest.detjson");
  const fixture = await readJson(fixturePath);
  const lessonRel = String(fixture.ddn_fixture ?? "fixtures/lesson.ddn");
  const lessonPath = path.join(packDir, lessonRel);
  const lesson = await fs.readFile(lessonPath, "utf8");
  const steps = Math.max(0, Math.floor(Number(fixture.step_before_state ?? 1)));

  const { vm, client } = await makeClient(runtime, lesson);
  try {
    client.resetParsed(false);
    for (let i = 0; i < steps; i += 1) client.stepOneParsed();
    const rawState = JSON.parse(vm.get_state_json());
    const resolvedManifest = resolveObservationManifestFromState(rawState, lesson);
    const manifest = resolvedManifest.manifest;
    const meta = metadataFromEngineResponse(rawState);

    const nodeNames = manifest.nodes.map((node) => String(node.name));
    const pivotX = String(manifest?.default_pivot?.x_channel ?? "");
    const pivotYs = Array.isArray(manifest?.default_pivot?.y_channels)
      ? manifest.default_pivot.y_channels.map((y) => String(y))
      : [];
    const pragmaRefNodes = manifest.nodes.filter(
      (node) => Array.isArray(node.pragma_refs) && node.pragma_refs.length > 0,
    );
    const expectedDtypes = fixture?.expected_dtypes && typeof fixture.expected_dtypes === "object"
      ? fixture.expected_dtypes
      : {};
    const dtypeMatches = Object.entries(expectedDtypes).every(([key, dtype]) => {
      const node = manifest.nodes.find((entry) => String(entry.name) === String(key));
      return node && String(node.dtype) === String(dtype);
    });

    const base = {
      schema: `seamgrim.wasm.${kind}.v1`,
      manifest_schema: String(manifest?.schema ?? ""),
      manifest_source: String(resolvedManifest.source ?? ""),
      native_manifest: resolvedManifest.source === "native",
      node_count: manifest.nodes.length,
      param_count: manifest.params.length,
      trace_count: manifest.traces.length,
      diagnostic_count: manifest.diagnostics.length,
      contract: String(meta.contract ?? ""),
      detmath_seal_hash: String(meta.detmath_seal_hash ?? ""),
      nuri_lock_hash: String(meta.nuri_lock_hash ?? ""),
      tick: Number(meta.tick ?? 0),
    };

    if (kind === "observation_manifest_smoke") {
      return {
        "expected/manifest_smoke.detjson": {
          ...base,
          nodes_non_empty: manifest.nodes.length > 0,
          has_default_pivot: pivotX.length > 0,
        },
      };
    }
    if (kind === "observation_manifest_role") {
      const roleSet = uniqueStrings(manifest.nodes.map((node) => node.role));
      const allStateRole = manifest.nodes.every((node) => String(node.role) === "state");
      return {
        "expected/manifest_role.detjson": {
          ...base,
          role_set: roleSet,
          all_state_role: allStateRole,
        },
      };
    }
    if (kind === "observation_manifest_pragma_refs") {
      return {
        "expected/manifest_pragma_refs.detjson": {
          ...base,
          pragma_ref_nodes: pragmaRefNodes.map((node) => ({
            name: String(node.name),
            pragma_refs: uniqueStrings(node.pragma_refs),
          })),
          pragma_ref_node_count: pragmaRefNodes.length,
        },
      };
    }
    if (kind === "observation_manifest_dtype") {
      return {
        "expected/manifest_dtype.detjson": {
          ...base,
          expected_dtype_count: Object.keys(expectedDtypes).length,
          dtype_matches: dtypeMatches,
          dtypes: manifest.nodes.map((node) => ({
            name: String(node.name),
            dtype: String(node.dtype),
          })),
        },
      };
    }
    if (kind === "observation_manifest_pivot") {
      const xExists = nodeNames.includes(pivotX);
      const yExistAll = pivotYs.every((name) => nodeNames.includes(name));
      return {
        "expected/manifest_pivot.detjson": {
          ...base,
          x_channel: pivotX,
          y_channels: pivotYs,
          x_exists_in_nodes: xExists,
          y_exists_in_nodes: yExistAll,
        },
      };
    }
    throw new Error(`지원하지 않는 observation manifest pack kind: ${kind}`);
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

async function runPack(packDir, runtime) {
  const packId = path.basename(packDir);
  if (packId === "seamgrim_wasm_v0_smoke") {
    return runWasmV0Smoke(packDir, runtime);
  }
  if (packId === "seamgrim_wasm_bridge_contract_v1") {
    return runBridgeContractV1(packDir, runtime);
  }
  if (packId === "seamgrim_wasm_viewmeta_statehash_v1" || packId === "patent_b_state_view_hash_isolation_v1") {
    return runViewmetaStatehashV1(packDir, runtime);
  }
  if (packId === "seamgrim_wasm_restore_state_v1") {
    return runRestoreStateV1(packDir, runtime);
  }
  if (packId === "seamgrim_wasm_observation_channels_v1") {
    return runObservationChannelsV1(packDir, runtime);
  }
  if (packId === "seamgrim_wasm_streams_serialization_v1") {
    return runStreamsSerializationV1(packDir, runtime);
  }
  if (packId === "seamgrim_tick_loop_v1") {
    return runTickLoopV1(packDir, runtime);
  }
  if (packId === "seamgrim_reset_v1") {
    return runResetV1(packDir, runtime);
  }
  if (packId === "seamgrim_state_apply_v1") {
    return runStateApplyV1(packDir, runtime);
  }
  if (packId === "seamgrim_hash_verify_v1") {
    return runHashVerifyV1(packDir, runtime);
  }
  if (packId === "observation_manifest_smoke_v1") {
    return runObservationManifestPack(packDir, runtime, "observation_manifest_smoke");
  }
  if (packId === "observation_manifest_role_v1") {
    return runObservationManifestPack(packDir, runtime, "observation_manifest_role");
  }
  if (packId === "observation_manifest_pragma_refs_v1") {
    return runObservationManifestPack(packDir, runtime, "observation_manifest_pragma_refs");
  }
  if (packId === "observation_manifest_dtype_v1") {
    return runObservationManifestPack(packDir, runtime, "observation_manifest_dtype");
  }
  if (packId === "observation_manifest_pivot_v1") {
    return runObservationManifestPack(packDir, runtime, "observation_manifest_pivot");
  }
  throw new Error(`지원하지 않는 pack: ${packId}`);
}

async function main() {
  const packArg = process.argv[2];
  if (!packArg) {
    throw new Error("사용법: node tests/seamgrim_wasm_pack_runner.mjs <pack_dir>");
  }
  const packDir = path.resolve(packArg);
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  if (path.basename(packDir) === "seamgrim_wasm_canon_contract_v1") {
    const outputs = await runWasmCanonContractV1(packDir, rootDir);
    process.stdout.write(`${JSON.stringify({ pack_id: path.basename(packDir), outputs }, null, 2)}\n`);
    return;
  }
  if (path.basename(packDir) === "seamgrim_maegim_slider_smoke_v1") {
    const outputs = await runMaegimSliderSmokeV1(packDir, rootDir);
    process.stdout.write(`${JSON.stringify({ pack_id: path.basename(packDir), outputs }, null, 2)}\n`);
    return;
  }
  if (path.basename(packDir) === "seamgrim_temp_lesson_smoke_v1") {
    const outputs = await runTempLessonSmokeV1(packDir, rootDir);
    process.stdout.write(`${JSON.stringify({ pack_id: path.basename(packDir), outputs }, null, 2)}\n`);
    return;
  }
  if (path.basename(packDir) === "seamgrim_moyang_render_smoke_v1") {
    const outputs = await runMoyangRenderSmokeV1(packDir, rootDir);
    process.stdout.write(`${JSON.stringify({ pack_id: path.basename(packDir), outputs }, null, 2)}\n`);
    return;
  }
  if (path.basename(packDir) === "seamgrim_interactive_event_smoke_v1") {
    const outputs = await runInteractiveEventSmokeV1(packDir, rootDir);
    process.stdout.write(`${JSON.stringify({ pack_id: path.basename(packDir), outputs }, null, 2)}\n`);
    return;
  }
  const runtime = await initRuntime(rootDir);
  const outputs = await runPack(packDir, runtime);
  const payload = {
    pack_id: path.basename(packDir),
    outputs,
  };
  process.stdout.write(`${JSON.stringify(payload, null, 2)}\n`);
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
