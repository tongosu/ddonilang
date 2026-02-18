#!/usr/bin/env node

import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function enginePayload({ tickId, stateHash, channels, row }) {
  return JSON.stringify({
    schema: "seamgrim.engine_response.v0",
    tick_id: tickId,
    state_hash: stateHash,
    input: {},
    resources: {
      json: {},
      fixed64: {},
      handle: {},
      value: {},
    },
    channels,
    row,
    state: {
      tick_id: tickId,
      channels,
      row,
      resources: {
        json: {},
        fixed64: {},
        handle: {},
        value: {},
      },
      patch: [],
      streams: {},
    },
    view_meta: {},
    view_hash: "blake3:viewhash",
  });
}

async function main() {
  const root = process.cwd();
  const wrapperPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/wasm_ddn_wrapper.js");
  const wrapper = await import(pathToFileURL(wrapperPath).href);

  const { DdnWasmVmClient, parseStateJson } = wrapper;
  assert(typeof DdnWasmVmClient === "function", "wrapper export: DdnWasmVmClient");
  assert(typeof parseStateJson === "function", "wrapper export: parseStateJson");

  const parsed = parseStateJson(
    enginePayload({
      tickId: 3,
      stateHash: "blake3:abc",
      channels: [{ key: "g", dtype: "fixed64", role: "state" }],
      row: [9.8],
    }),
  );
  assert(parsed.schema === "seamgrim.state.v0", "parseStateJson: normalized schema");
  assert(parsed.engine_schema === "seamgrim.engine_response.v0", "parseStateJson: engine schema");
  assert(Array.isArray(parsed.channels) && parsed.channels.length === 1, "parseStateJson: channels");
  assert(Array.isArray(parsed.row) && parsed.row.length === 1, "parseStateJson: row");

  let compatUpdated = "";
  const vmCompatOnly = {
    update_logic(source) {
      compatUpdated = String(source ?? "");
    },
    set_rng_seed() {},
    set_input() {},
    step_one() {
      return enginePayload({
        tickId: 0,
        stateHash: "blake3:s0",
        channels: [{ key: "x", dtype: "fixed64", role: "state" }],
        row: [1],
      });
    },
    step_one_with_input() {
      return enginePayload({
        tickId: 1,
        stateHash: "blake3:s1",
        channels: [{ key: "x", dtype: "fixed64", role: "state" }],
        row: [2],
      });
    },
    get_state_hash() {
      return "blake3:state";
    },
    get_state_json() {
      return enginePayload({
        tickId: 2,
        stateHash: "blake3:s2",
        channels: [{ key: "x", dtype: "fixed64", role: "state" }],
        row: [3],
      });
    },
  };
  const compatClient = new DdnWasmVmClient(vmCompatOnly);
  compatClient.updateLogicWithMode("매틱:움직씨 = {}.", "strict");
  assert(compatUpdated.includes("매틱"), "updateLogicWithMode fallback -> update_logic");
  const compatCols = compatClient.columnsParsed();
  assert(
    Array.isArray(compatCols.columns) &&
      compatCols.columns.length === 0 &&
      Array.isArray(compatCols.row) &&
      compatCols.row.length === 0,
    "columnsParsed fallback: columns+row",
  );
  let caught = false;
  try {
    compatClient.setParamParsed("g", 1);
  } catch (err) {
    caught = String(err?.message ?? err).includes("set_param");
  }
  assert(caught, "setParamParsed: missing function error");
  caught = false;
  try {
    compatClient.setParamFixed64Parsed("g", 4294967296);
  } catch (err) {
    caught = String(err?.message ?? err).includes("set_param_fixed64");
  }
  assert(caught, "setParamFixed64Parsed: missing function error");
  caught = false;
  try {
    compatClient.setParamFixed64StringParsed("g", "4294967296");
  } catch (err) {
    caught = String(err?.message ?? err).includes("set_param_fixed64_str");
  }
  assert(caught, "setParamFixed64StringParsed: missing function error");
  caught = false;
  try {
    compatClient.resetParsed(false);
  } catch (err) {
    caught = String(err?.message ?? err).includes("reset");
  }
  assert(caught, "resetParsed: missing function error");
  caught = false;
  try {
    compatClient.restoreStateParsed("{}");
  } catch (err) {
    caught = String(err?.message ?? err).includes("restore_state");
  }
  assert(caught, "restoreStateParsed: missing function error");

  let stepSeq = 0;
  let lastMode = "";
  let lastSetParam = null;
  let lastSetParamFixed = null;
  let lastSetParamFixedStr = null;
  let lastResetKeep = null;
  let lastRestoreRaw = "";
  const viewPrefixes = [];
  const aiActions = [];
  const vmFull = {
    update_logic() {},
    update_logic_with_mode(_source, mode) {
      lastMode = String(mode ?? "");
    },
    set_rng_seed() {},
    set_input() {},
    columns() {
      return JSON.stringify({
        columns: [{ key: "g", dtype: "fixed64", role: "state" }],
        row: [15],
      });
    },
    set_param(key, value) {
      lastSetParam = { key, value };
      return JSON.stringify({ ok: true, state_hash: "blake3:setparam" });
    },
    set_param_fixed64(key, raw_i64) {
      lastSetParamFixed = { key, raw_i64 };
      return JSON.stringify({ ok: true, state_hash: "blake3:setparamfixed" });
    },
    set_param_fixed64_str(key, raw_i64) {
      lastSetParamFixedStr = { key, raw_i64 };
      return JSON.stringify({ ok: true, state_hash: "blake3:setparamfixedstr" });
    },
    add_view_prefix(prefix) {
      viewPrefixes.push(String(prefix ?? ""));
    },
    clear_view_prefixes() {
      viewPrefixes.length = 0;
    },
    inject_ai_action(key, value_json) {
      aiActions.push({ key: String(key ?? ""), value_json: String(value_json ?? "") });
    },
    clear_ai_injections() {
      aiActions.length = 0;
    },
    reset(keep) {
      lastResetKeep = Boolean(keep);
      return JSON.stringify({
        ok: true,
        keep_params: Boolean(keep),
        columns: [{ key: "g" }],
        row: [Boolean(keep) ? 15 : 0],
      });
    },
    restore_state(raw) {
      lastRestoreRaw = String(raw ?? "");
      return JSON.stringify({ ok: true, tick: 2 });
    },
    step_one() {
      const idx = stepSeq++;
      return enginePayload({
        tickId: idx,
        stateHash: `blake3:step${idx}`,
        channels: [{ key: "x", dtype: "fixed64", role: "state" }],
        row: [idx + 1],
      });
    },
    step_one_with_input() {
      const idx = stepSeq++;
      return enginePayload({
        tickId: idx,
        stateHash: `blake3:input${idx}`,
        channels: [{ key: "x", dtype: "fixed64", role: "state" }],
        row: [idx + 10],
      });
    },
    get_state_hash() {
      return "blake3:full";
    },
    get_state_json() {
      return enginePayload({
        tickId: 77,
        stateHash: "blake3:state77",
        channels: [{ key: "x", dtype: "fixed64", role: "state" }],
        row: [77],
      });
    },
  };

  const client = new DdnWasmVmClient(vmFull);
  client.updateLogicWithMode("x<-1.", "strict");
  assert(lastMode === "strict", "updateLogicWithMode: uses wasm mode API");

  const cols = client.columnsParsed();
  assert(Array.isArray(cols.columns) && cols.columns.length === 1, "columnsParsed: columns");
  assert(Array.isArray(cols.row) && cols.row.length === 1, "columnsParsed: row");

  const setResult = client.setParamParsed("g", 15);
  assert(setResult.ok === true, "setParamParsed: ok");
  assert(lastSetParam?.key === "g" && lastSetParam?.value === 15, "setParamParsed: forwarded");
  const setFixedResult = client.setParamFixed64Parsed("g", 4294967296);
  assert(setFixedResult.ok === true, "setParamFixed64Parsed: ok");
  assert(
    lastSetParamFixed?.key === "g" && lastSetParamFixed?.raw_i64 === 4294967296,
    "setParamFixed64Parsed: forwarded",
  );
  const setFixedStrResult = client.setParamFixed64StringParsed("g", "9223372036854775807");
  assert(setFixedStrResult.ok === true, "setParamFixed64StringParsed: ok");
  assert(
    lastSetParamFixedStr?.key === "g" && lastSetParamFixedStr?.raw_i64 === "9223372036854775807",
    "setParamFixed64StringParsed: forwarded",
  );
  assert(client.addViewPrefix("__view_") === true, "addViewPrefix: available");
  assert(viewPrefixes.length === 1 && viewPrefixes[0] === "__view_", "addViewPrefix: forwarded");
  assert(client.clearViewPrefixes() === true, "clearViewPrefixes: available");
  assert(viewPrefixes.length === 0, "clearViewPrefixes: forwarded");
  assert(
    client.injectAiAction("agent_action", "{\"dir\":\"left\"}") === true,
    "injectAiAction: available",
  );
  assert(aiActions.length === 1, "injectAiAction: forwarded");
  assert(client.clearAiInjections() === true, "clearAiInjections: available");
  assert(aiActions.length === 0, "clearAiInjections: forwarded");

  const s0 = client.stepOneParsed();
  const s1 = client.stepOneParsed();
  assert(s0.frame_id === 0, "stepOneParsed: first frame_id");
  assert(s1.frame_id === 1, "stepOneParsed: second frame_id");

  const g0 = client.getStateParsed();
  assert(g0.frame_id === 2, "getStateParsed: uses current frame_id");

  const r0 = client.resetParsed(true);
  assert(r0.ok === true && lastResetKeep === true, "resetParsed: forwarded");
  const s2 = client.stepOneParsed();
  assert(s2.frame_id === 0, "resetParsed: frame_id reset");

  const restoreResult = client.restoreStateParsed("{\"schema\":\"seamgrim.engine_response.v0\"}");
  assert(restoreResult.ok === true, "restoreStateParsed: ok");
  assert(lastRestoreRaw.includes("seamgrim.engine_response.v0"), "restoreStateParsed: forwarded");
  const s3 = client.stepOneWithInputParsed(0, "", 0, 0, 0.016);
  assert(s3.frame_id === 0, "restoreStateParsed: frame_id reset");

  console.log("seamgrim wasm wrapper ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
