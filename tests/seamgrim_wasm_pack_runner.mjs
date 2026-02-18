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
  const observed = {};
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
        observed[label] = String(client.getStateHash() ?? "");
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
    const aHash = String(observed[aLabel] ?? "");
    const bHash = String(observed[bLabel] ?? "");
    const cHash = String(observed[cLabel] ?? "");

    const report = {
      schema: "seamgrim.wasm.viewmeta_statehash.v1",
      labels: { a: aLabel, b: bLabel, c: cLabel },
      hashes: { [aLabel]: aHash, [bLabel]: bHash, [cLabel]: cHash },
      equal_ab: aHash !== "" && aHash === bHash,
      equal_bc: bHash !== "" && bHash === cHash,
      note: "a/b는 보개_* 변경만 포함, c는 상태 키 변경 포함",
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
  if (packId === "seamgrim_wasm_viewmeta_statehash_v1") {
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
