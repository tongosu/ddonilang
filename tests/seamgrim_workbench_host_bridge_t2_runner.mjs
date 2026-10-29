#!/usr/bin/env node

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const BASELINE = "0249aa0ded9ec58832c4c85e6ef169c6356a8fdc";
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const WORKBENCH_PATH = "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js";

const DIRECT_HOST_PATTERNS = Object.freeze({
  user_file_text_read: /\bfile\.text\(\)/gu,
  user_file_byte_read: /\bfile\.arrayBuffer\(\)/gu,
  browser_download_assignment: /\blink\.download\s*=/gu,
  browser_clipboard_write: /globalThis\.navigator\?\.clipboard\?\.writeText\?/gu,
});

function countMatches(text, pattern) {
  return [...String(text).matchAll(pattern)].length;
}

function directHostInventory(text) {
  const rows = Object.entries(DIRECT_HOST_PATTERNS).map(([family, pattern]) => ({
    family,
    call_sites: countMatches(text, pattern),
  }));
  return {
    families: rows.filter((row) => row.call_sites > 0).length,
    call_sites: rows.reduce((sum, row) => sum + row.call_sites, 0),
    rows,
  };
}

function gitShow(commit, relativePath) {
  return execFileSync("git", ["show", `${commit}:${relativePath}`], {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
}

async function read(relativePath) {
  return fs.readFile(path.join(ROOT, relativePath), "utf8");
}

function lfProjection(text) {
  return String(text).replace(/^\uFEFF/u, "").replace(/\r\n?/gu, "\n");
}

async function assertUnchanged(relativePath) {
  assert.equal(
    lfProjection(await read(relativePath)),
    lfProjection(gitShow(BASELINE, relativePath)),
    `${relativePath} drifted from T2 baseline`,
  );
}

async function main() {
  const baselineWorkbench = gitShow(BASELINE, WORKBENCH_PATH);
  const currentWorkbench = await read(WORKBENCH_PATH);
  const before = directHostInventory(baselineWorkbench);
  const after = directHostInventory(currentWorkbench);

  assert.equal(before.families, 4, `baseline selected host families drifted: ${JSON.stringify(before)}`);
  assert.equal(before.call_sites, 4, `baseline selected host call sites drifted: ${JSON.stringify(before)}`);
  assert.equal(after.families, 0, `direct host bypass families remain: ${JSON.stringify(after)}`);
  assert.equal(after.call_sites, 0, `direct host bypass call sites remain: ${JSON.stringify(after)}`);

  for (const token of [
    'import { createHostBridgeV1 } from "../host/host_bridge.js";',
    "const hostBridge = createHostBridgeV1();",
    "hostBridge.readTextFile(file)",
    "hostBridge.readFileBytes(file)",
    "hostBridge.downloadTextFile({",
    "hostBridge.writeClipboardText(text)",
  ]) {
    assert.ok(currentWorkbench.includes(token), `Workbench HostBridge routing token missing: ${token}`);
  }

  const storagePattern = /globalThis\.(?:localStorage|sessionStorage)/gu;
  assert.equal(
    countMatches(currentWorkbench, storagePattern),
    countMatches(baselineWorkbench, storagePattern),
    "Workbench storage access count changed",
  );

  for (const relativePath of [
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/local_fallback.js",
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/local_gotgan_store.js",
    "solutions/seamgrim_ui_mvp/ui/workbench/persistence/capability_session.js",
  ]) {
    await assertUnchanged(relativePath);
  }

  for (const relativePath of [
    "solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js",
    "solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_renderer.js",
    "solutions/seamgrim_ui_mvp/ui/view_plan_runtime.js",
  ]) {
    const source = await read(relativePath);
    assert.deepEqual(
      directHostInventory(source),
      {
        families: 0,
        call_sites: 0,
        rows: Object.keys(DIRECT_HOST_PATTERNS).map((family) => ({ family, call_sites: 0 })),
      },
      `${relativePath} must remain outside HostBridge responsibility`,
    );
    assert.equal(countMatches(source, storagePattern), 0, `${relativePath} must not own browser storage`);
  }

  const runtimeBridgeSource = await read("solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js");
  assert.ok(runtimeBridgeSource.includes('"sourceAnalysis"'), "T4 SourceAnalysis route missing");
  assert.ok(runtimeBridgeSource.includes("wasm_source_analysis_snapshot_json"), "T4 WASM query route missing");
  assert.ok(!runtimeBridgeSource.includes("HostBridge"), "RuntimeBridge must not own host capabilities");

  const hostBridgeSource = await read("solutions/seamgrim_ui_mvp/ui/host/host_bridge.js");
  const browserAdapterSource = await read("solutions/seamgrim_ui_mvp/ui/host/browser_host_adapter.js");
  for (const forbidden of ["wasm", "RuntimeBridge", "localStorage", "sessionStorage", "indexedDB", "fetch("]) {
    assert.ok(!hostBridgeSource.includes(forbidden), `HostBridge owns forbidden concern: ${forbidden}`);
    assert.ok(!browserAdapterSource.includes(forbidden), `BrowserHostAdapter owns forbidden concern: ${forbidden}`);
  }

  const hostModule = await import(pathToFileURL(path.join(ROOT, "solutions/seamgrim_ui_mvp/ui/host/host_bridge.js")));
  const browserModule = await import(pathToFileURL(path.join(ROOT, "solutions/seamgrim_ui_mvp/ui/host/browser_host_adapter.js")));
  assert.equal(hostModule.HOST_BRIDGE_V1_SCHEMA, "seamgrim.host_bridge.v1");
  assert.deepEqual(hostModule.HOST_BRIDGE_V1_OPERATIONS, [
    "readTextFile",
    "readFileBytes",
    "downloadTextFile",
    "writeClipboardText",
  ]);

  const textFile = { kind: "text-file" };
  const byteFile = { kind: "byte-file" };
  const downloadRequest = { text: "payload", filename: "artifact.json", mimeType: "application/json" };
  const textResult = { kind: "text-result" };
  const byteResult = { kind: "byte-result" };
  const downloadResult = { kind: "download-result" };
  const clipboardResult = { kind: "clipboard-result" };
  const calls = [];
  const fakeAdapter = {
    readTextFile(file) { calls.push(["readTextFile", file]); return textResult; },
    readFileBytes(file) { calls.push(["readFileBytes", file]); return byteResult; },
    downloadTextFile(request) { calls.push(["downloadTextFile", request]); return downloadResult; },
    writeClipboardText(text) { calls.push(["writeClipboardText", text]); return clipboardResult; },
  };
  const bridge = new hostModule.HostBridgeV1({ adapter: fakeAdapter });
  assert.equal(bridge.readTextFile(textFile), textResult);
  assert.equal(bridge.readFileBytes(byteFile), byteResult);
  assert.equal(bridge.downloadTextFile(downloadRequest), downloadResult);
  assert.equal(bridge.writeClipboardText("clip"), clipboardResult);
  assert.deepEqual(calls, [
    ["readTextFile", textFile],
    ["readFileBytes", byteFile],
    ["downloadTextFile", downloadRequest],
    ["writeClipboardText", "clip"],
  ]);

  const failure = new Error("host failure sentinel");
  const failingBridge = new hostModule.HostBridgeV1({
    adapter: {
      writeClipboardText() { throw failure; },
    },
  });
  assert.throws(() => failingBridge.writeClipboardText("x"), (error) => error === failure);

  const browserCalls = [];
  const anchor = {
    style: {},
    click() { browserCalls.push("click"); },
    remove() { browserCalls.push("remove"); },
  };
  const documentRef = {
    createElement(tag) { browserCalls.push(["createElement", tag]); return anchor; },
    body: {
      appendChild(node) { browserCalls.push(["appendChild", node]); },
    },
  };
  const browserTextFile = {
    text() { browserCalls.push(["text", this]); return textResult; },
  };
  const browserByteFile = {
    arrayBuffer() { browserCalls.push(["arrayBuffer", this]); return byteResult; },
  };
  const navigatorRef = {
    clipboard: {
      writeText(text) { browserCalls.push(["writeText", text]); return clipboardResult; },
    },
  };
  const browserAdapter = new browserModule.BrowserHostAdapter({ documentRef, navigatorRef });
  assert.equal(browserAdapter.readTextFile(browserTextFile), textResult);
  assert.equal(browserAdapter.readFileBytes(browserByteFile), byteResult);
  assert.equal(browserAdapter.writeClipboardText("클립"), clipboardResult);
  assert.equal(browserAdapter.downloadTextFile({
    text: "한글 payload",
    filename: "artifact.json",
    mimeType: "application/json;charset=utf-8",
  }), undefined);
  assert.equal(anchor.href, `data:application/json;charset=utf-8,${encodeURIComponent("한글 payload")}`);
  assert.equal(anchor.download, "artifact.json");
  assert.equal(anchor.rel, "noopener");
  assert.equal(anchor.style.display, "none");
  assert.deepEqual(browserCalls, [
    ["text", browserTextFile],
    ["arrayBuffer", browserByteFile],
    ["writeText", "클립"],
    ["createElement", "a"],
    ["appendChild", anchor],
    "click",
    "remove",
  ]);

  const unsupported = new browserModule.BrowserHostAdapter({ documentRef: null, navigatorRef: null });
  assert.throws(() => unsupported.readTextFile({}), /browser text file read is unavailable/u);
  assert.throws(() => unsupported.readFileBytes({}), /browser binary file read is unavailable/u);
  assert.throws(() => unsupported.downloadTextFile({}), /browser file download is unavailable/u);
  assert.throws(() => unsupported.writeClipboardText("x"), /browser clipboard write is unavailable/u);

  console.log(JSON.stringify({
    schema: "seamgrim.workbench.host_bridge_t2.runner_result.v1",
    baseline_commit: BASELINE,
    bridge_schema: hostModule.HOST_BRIDGE_V1_SCHEMA,
    direct_host_families_before: before.families,
    direct_host_call_sites_before: before.call_sites,
    direct_host_families_after: after.families,
    direct_host_call_sites_after: after.call_sites,
    host_operations: hostModule.HOST_BRIDGE_V1_OPERATIONS,
    runtime_bridge_host_boundary_preserved: true,
    runtime_bridge_change_scope: "T4_SOURCE_ANALYSIS_ROUTE_ONLY",
    storage_files_unchanged: true,
    view_files_unchanged_from_t2: false,
    view_files_change_scope: "T5F_CANONICAL_WORKBENCH_RESULT_MIGRATION",
    view_files_host_boundary_preserved: true,
    input_output_identity_preserved: true,
    error_identity_preserved: true,
    unsupported_fail_closed: true,
    result: "PASS",
  }));
}

await main();
