#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function initRuntime(rootDir) {
  const uiDir = path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModuleUrl = pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href;
  const wrapperUrl = pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href;
  const preprocessUrl = pathToFileURL(path.join(uiDir, "runtime", "ddn_preprocess.js")).href;
  const wasmPath = path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm");

  const wasmModule = await import(wasmModuleUrl);
  const wrapper = await import(wrapperUrl);
  const preprocess = await import(preprocessUrl);
  const wasmBytes = await fs.readFile(wasmPath);

  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }

  return { wasmModule, wrapper, preprocess };
}

function readConsoleGridLines(state) {
  const valueJson = state?.resources?.value_json;
  const lines = valueJson?.보개_출력_줄들;
  return Array.isArray(lines) ? lines.map((line) => String(line ?? "")) : [];
}

async function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const runtime = await initRuntime(rootDir);
  const sourceText = `설정 {
  제목: 새_교과.
  설명: "채비를 조절하면서 결과를 확인하세요.".
}.

채비 {
  계수:수 <- 1.
  프레임수:수 <- 0.
  t:수 <- 0.
  y:수 <- 0.
}.

(매마디)마다 {
  t <- 프레임수.
  y <- (계수 * t).
  t 보여주기.
  y 보여주기.
  프레임수 <- (프레임수 + 1).
}.`;

  const bodyText = runtime.preprocess.preprocessDdnText(sourceText).bodyText;
  assert(!bodyText.includes("#이름:") && !bodyText.includes("#설명:"), "draft preprocess: legacy hash meta absent");
  assert(bodyText.includes("{ __wasm_start_once <= 0 }인것 일때 {"), "draft preprocess: start-once guard injected");
  assert(bodyText.includes("프레임수 <- 0."), "draft preprocess: frame counter init injected");
  assert(bodyText.includes("t <- 0."), "draft preprocess: t init injected");
  assert(bodyText.includes("y <- 0."), "draft preprocess: y init injected");
  assert(!bodyText.includes("    계수 <- 1."), "draft preprocess: plain control/default stays outside start-once init");

  const vm = new runtime.wasmModule.DdnWasmVm(bodyText);
  const client = new runtime.wrapper.DdnWasmVmClient(vm);

  const step0 = client.stepOneParsed();
  const step1 = client.stepOneParsed();
  const step2 = client.stepOneParsed();

  assert(JSON.stringify(readConsoleGridLines(step0)) === JSON.stringify(["0", "0"]), "draft runtime: first tick shows 0/0");
  assert(JSON.stringify(readConsoleGridLines(step1)) === JSON.stringify(["1", "1"]), "draft runtime: second tick shows 1/1");
  assert(JSON.stringify(readConsoleGridLines(step2)) === JSON.stringify(["2", "2"]), "draft runtime: third tick shows 2/2");

  console.log("seamgrim studio draft runtime ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
