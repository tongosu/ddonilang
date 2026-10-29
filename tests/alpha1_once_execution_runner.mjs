import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { chromium } from "playwright";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wasmDir = process.env.DDN_WASM_DIR ? path.resolve(process.env.DDN_WASM_DIR)
  : path.join(root, "solutions/seamgrim_ui_mvp/ui/wasm");
const wasm = await import(pathToFileURL(path.join(wasmDir, "ddonirang_tool.js")).href);
await wasm.default({ module_or_path: fs.readFileSync(path.join(wasmDir, "ddonirang_tool_bg.wasm")) });

const programs = [
  ["01_요금_계산기.ddn", ["2800", "2100"]],
  ["02_조건과_반복.ddn", ["작은 제곱", "작은 제곱", "큰 제곱", "큰 제곱", "30"]],
  ["03_자동_함수표.ddn", ["0", "0", "2", "1", "1", "4", "2", "4", "10", "3", "9", "20"]],
  ["04_시간단계_시뮬레이션.ddn", ["1", "2", "3", "2", "5", "4", "3", "9", "5", "4", "14", "6"]],
];

for (const [name, expected] of programs) {
  const sourcePath = `solutions/seamgrim_ui_mvp/alpha1/${name}`;
  const source = fs.readFileSync(path.join(root, sourcePath), "utf8");
  assert.doesNotMatch(source, /\(매마디\)마다|매마디\s*:\s*움직씨/u, `${name}: 계산 예제에 반복 훅을 넣지 않습니다`);
  const result = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, sourcePath, 1, "v1-core-v25"));
  assert.deepEqual(result.output_log_texts, expected, `${name}: 실제 WASM 한 번 실행 결과`);
}

// Summary ticks explicitly request repeated execution. Once/live routing is
// a Workbench workflow contract, so verify it in the real browser instead.
const browser = await chromium.launch({ headless: true, executablePath: process.env.CHROME_EXECUTABLE || "C:/Program Files/Google/Chrome/Application/chrome.exe" });
try {
  const page = await browser.newPage();
  for (const [name, expected] of programs) {
    await page.goto(process.env.DDN_TEST_URL || "http://127.0.0.1:8790/solutions/seamgrim_ui_mvp/ui/workbench/index.html");
    await page.waitForFunction(() => document.querySelector("#source-editor")?.value.includes("매마디"));
    await page.locator("#source-editor").fill(fs.readFileSync(path.join(root, "solutions/seamgrim_ui_mvp/alpha1", name), "utf8"));
    await page.getByTestId("lifecycle-run").click();
    await page.waitForFunction(() => document.querySelector('[data-testid="approved-workbench-root"]')?.dataset.executionState === "done");
    assert.equal(await page.getByTestId("approved-workbench-root").getAttribute("data-execution-mode"), "oneshot", name);
    const output = await page.locator("#canvas-bogae").getAttribute("data-console-output");
    assert.deepEqual(output.trim().split("\n"), expected, name);
    await page.waitForTimeout(350);
    assert.equal(await page.locator("#canvas-bogae").getAttribute("data-console-output"), output, `${name}: 완료 뒤 출력이 반복되지 않습니다`);
    assert.equal(await page.getByTestId("approved-workbench-root").getAttribute("data-execution-state"), "done", name);
  }
} finally {
  await browser.close();
}
console.log("ALPHA1_ONCE_EXECUTION_PASS programs=4 actual_wasm=1 browser_oneshot=4 repeated_output=0");
