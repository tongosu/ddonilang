#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import crypto from "node:crypto";
import { chromium } from "playwright";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  return "application/octet-stream";
}

async function createServer(root) {
  const base = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      const file = path.resolve(base, decodeURIComponent(url.pathname).replace(/^\/+/, ""));
      if (!file.startsWith(`${base}${path.sep}`)) throw new Error("forbidden");
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404);
      res.end("not found");
    }
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  return { server, baseUrl: `http://127.0.0.1:${address.port}` };
}

async function stepToMadi(page, target) {
  for (;;) {
    const snapshot = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    const current = Number(snapshot.activeFrame?.madi ?? -1);
    if (current >= target) return snapshot;
    await page.getByTestId("lifecycle-step").click();
    await page.waitForFunction(
      (previous) => Number(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi ?? -1) > previous,
      current,
    );
  }
}

async function assertNativeViewport(browser, baseUrl, viewport) {
  const context = await browser.newContext({ viewport });
  const page = await context.newPage();
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  const beforeCount = Number(await page.locator("#project-count").textContent());
  await page.getByTestId("new-ddn-project").focus();
  await page.keyboard.press("Enter");
  await page.getByTestId("project-create-cancel").focus();
  await page.keyboard.press("Enter");
  assert(Number(await page.locator("#project-count").textContent()) === beforeCount, `${viewport.width}: create cancel changed projects`);
  await page.getByTestId("new-ddn-project").focus();
  await page.keyboard.press("Enter");
  await page.getByTestId("project-create-name").fill(`키보드 물리 ${viewport.width}`);
  await page.getByTestId("project-create-submit").focus();
  await page.keyboard.press("Enter");
  await page.locator("#project-title").filter({ hasText: `키보드 물리 ${viewport.width}` }).waitFor();
  const editorBox = await page.getByTestId("source-editor").boundingBox();
  const canvasBox = await page.getByTestId("bogae-canvas").boundingBox();
  assert(editorBox?.width >= 180 && editorBox?.height >= 120, `${viewport.width}: editor unusable ${JSON.stringify(editorBox)}`);
  assert(canvasBox?.width >= 180 && canvasBox?.height >= 120, `${viewport.width}: primary view crushed ${JSON.stringify(canvasBox)}`);
  const rootButton = page.locator("[data-testid='file-list'] .file-row", { hasText: "뿌리.ddn" });
  await rootButton.focus();
  await page.keyboard.press("Enter");
  assert((await page.locator("#source-panel-title").textContent()) === "뿌리.ddn", `${viewport.width}: keyboard file select failed`);
  const overflow = await page.evaluate(() => Math.max(document.documentElement.scrollWidth, document.body.scrollWidth) - document.documentElement.clientWidth);
  assert(overflow <= 2, `${viewport.width}: horizontal overflow ${overflow}`);
  const unlabeled = await page.locator(".file-row-actions button:not([title]), .file-row-actions button:not([aria-label])").count();
  assert(unlabeled === 0, `${viewport.width}: unlabeled file icon`);
  await context.close();
}

const root = path.resolve(".");
const { server, baseUrl } = await createServer(root);
const browser = await chromium.launch({ headless: true });
try {
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await context.newPage();
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`, { waitUntil: "networkidle" });
  await page.getByTestId("new-ddn-project").click();
  await page.getByTestId("project-create-name").fill("낙하 운동 실험");
  await page.getByTestId("project-create-submit").click();
  await page.locator("#project-title").filter({ hasText: "낙하 운동 실험" }).waitFor();
  const fileRows = page.locator("[data-testid='file-list'] .file-row");
  assert(await fileRows.count() === 3, "physics template must create three actual files");
  const labels = await fileRows.allTextContents();
  assert(labels.some((text) => text.includes("뿌리.ddn")), "missing 뿌리.ddn");
  assert(labels.some((text) => text.includes("app/main.ddn")), "missing app/main.ddn");
  assert(labels.some((text) => text.includes("lib/힘.ddn")), "missing lib/힘.ddn");
  assert((await page.getByTestId("source-editor").inputValue()).includes('(힘="./lib/힘") 쓰임.'), "current typed import is not active");
  await page.getByTestId("project-graph-drawer").click();
  await page.waitForTimeout(1000);
  const graphSummary = await page.getByTestId("project-graph-summary").textContent();
  assert(graphSummary.includes("entry app/main.ddn"), `graph summary failed: ${graphSummary} ${(await page.getByTestId("project-graph-diagnostics").textContent())}`);
  assert((await page.getByTestId("project-graph-list").textContent()).includes("app/main.ddn —힘→ lib/힘.ddn"), "import graph edge missing");

  page.once("dialog", (dialog) => dialog.accept("lib/충돌.ddn"));
  await page.getByRole("button", { name: "lib/힘.ddn 이름 변경" }).click();
  await page.getByTestId("project-edit-dialog").waitFor({ state: "visible" });
  await page.evaluate(() => {
    const editor = document.querySelector("[data-testid='source-editor']");
    editor.value += "\n// concurrent edit";
    editor.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await page.getByTestId("project-edit-apply").click();
  await page.getByText(/미리보기 뒤 원문이 바뀌어 적용을 취소/u).waitFor();
  assert(await page.getByRole("button", { name: "lib/힘.ddn 이름 변경" }).count() === 1, "hash conflict partially renamed tree");
  await page.getByTestId("project-edit-cancel").click();
  await page.getByTestId("source-editor").fill((await page.getByTestId("source-editor").inputValue()).replace("\n// concurrent edit", ""));

  const mainWithControls = `${await page.getByTestId("source-editor").inputValue()}\n설명:글 := "./lib/힘".\n// ./lib/힘 주석`;
  await page.getByTestId("source-editor").fill(mainWithControls);

  await page.getByTestId("lifecycle-run").click();
  await page.waitForTimeout(1200);
  const started = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert(started.runtimeStatus === "running", `physics run failed: ${JSON.stringify(started.diagnostics)}`);
  const beforeRenameRun = await stepToMadi(page, 4);
  assert(beforeRenameRun.activeFrame?.madi >= 4, `physics run did not reach four madi: ${beforeRenameRun.activeFrame?.madi}`);
  const beforeRenameHash = beforeRenameRun.activeFrame.state_hash;
  const observationText = await page.getByTestId("semantic-observations").textContent();
  assert(observationText.includes("위치") && observationText.includes("m/s") && observationText.includes("m/s²"), "named/unit observations missing");
  const beforeRenameCanvas = crypto.createHash("sha256").update(await page.getByTestId("bogae-canvas").evaluate((canvas) => canvas.toDataURL())).digest("hex");
  const beforeRenameTrace = await page.getByTestId("bogae-canvas").getAttribute("data-project-run-trace-sha256");

  page.once("dialog", (dialog) => dialog.accept("lib/중력.ddn"));
  await page.getByRole("button", { name: "lib/힘.ddn 이름 변경" }).click();
  await page.waitForTimeout(500);
  if (!(await page.getByTestId("project-edit-dialog").evaluate((node) => node.open))) {
    throw new Error(`rename plan did not open: ${JSON.stringify((await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot())).diagnostics)}`);
  }
  await page.getByTestId("project-edit-dialog").waitFor({ state: "visible" });
  assert((await page.getByTestId("project-edit-preview").textContent()).includes("structured_project_reference"), "rename plan preview missing");
  await page.getByTestId("project-edit-apply").click();
  await page.getByRole("button", { name: "lib/중력.ddn 이름 변경" }).waitFor();
  const renamedMain = await page.getByTestId("source-editor").inputValue();
  assert(renamedMain.includes('(힘="./lib/중력") 쓰임.'), "typed import not renamed");
  assert(renamedMain.includes('설명:글 := "./lib/힘".'), "ordinary string was incorrectly renamed");
  assert(renamedMain.includes("// ./lib/힘 주석"), "comment was incorrectly renamed");

  await page.getByTestId("lifecycle-run").click();
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus === "running");
  const afterRenameRun = await stepToMadi(page, 4);
  assert(
    afterRenameRun.activeFrame.state_hash === beforeRenameHash,
    `rename changed runtime state hash: before=${beforeRenameHash}@${beforeRenameRun.activeFrame?.madi} after=${afterRenameRun.activeFrame.state_hash}@${afterRenameRun.activeFrame?.madi}`,
  );
  const afterRenameCanvas = crypto.createHash("sha256").update(await page.getByTestId("bogae-canvas").evaluate((canvas) => canvas.toDataURL())).digest("hex");
  assert(afterRenameCanvas === beforeRenameCanvas, "rename changed canvas pixels");

  await page.getByRole("button", { name: "lib/중력.ddn 삭제" }).click();
  await page.getByText(/참조 중인 모듈은 삭제할 수 없습니다/u).waitFor();
  assert(await page.getByRole("button", { name: "lib/중력.ddn 이름 변경" }).count() === 1, "referenced delete partially changed tree");

  await page.getByTestId("project-edit-undo").click();
  await page.getByRole("button", { name: "lib/힘.ddn 이름 변경" }).waitFor();

  await page.locator("[data-testid='file-list'] .file-row", { hasText: "lib/힘.ddn" }).click();
  const forceSource = await page.getByTestId("source-editor").inputValue();
  await page.getByTestId("source-editor").fill(forceSource.replace("9.8", "4.9"));
  await page.getByTestId("lifecycle-run").click();
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus === "running");
  const changedRun = await stepToMadi(page, 4);
  const changedTrace = await page.getByTestId("bogae-canvas").getAttribute("data-project-run-trace-sha256");
  const changedCanvas = crypto.createHash("sha256").update(await page.getByTestId("bogae-canvas").evaluate((canvas) => canvas.toDataURL())).digest("hex");
  assert(changedRun.activeFrame.state_hash !== beforeRenameHash, "physics value change did not change state hash");
  assert(changedTrace && changedTrace !== beforeRenameTrace, "physics value change did not change trace");
  assert(changedCanvas !== beforeRenameCanvas, "physics value change did not change canvas pixels");

  page.once("dialog", (dialog) => dialog.accept("app/보조.ddn"));
  await page.getByTestId("file-add").click();
  await page.getByTestId("project-edit-apply").click();
  assert(await fileRows.count() === 4, "file add did not atomically update tree");

  page.once("dialog", (dialog) => dialog.accept("app/보조_복사.ddn"));
  await page.getByRole("button", { name: "app/보조.ddn 복제" }).click();
  await page.getByTestId("project-edit-apply").click();
  assert(await fileRows.count() === 5, "file duplicate did not update tree");

  await page.getByRole("button", { name: "app/보조.ddn 진입 파일 지정" }).click();
  await page.getByTestId("project-edit-apply").click();
  await page.locator("[data-testid='file-list'] .file-row", { hasText: "뿌리.ddn" }).click();
  const rootAfterEntry = await page.getByTestId("source-editor").inputValue();
  assert(rootAfterEntry.includes("첫글: \"./app/보조.ddn\""), "root first entry was not updated");
  assert(rootAfterEntry.includes("앱: \"./app/보조.ddn\""), "root target was not updated");

  await page.reload({ waitUntil: "networkidle" });
  await page.getByTestId("global-my-work").click();
  await page.locator("[data-testid='my-work-item']").filter({ hasText: "낙하 운동 실험" }).click();
  assert(await fileRows.count() === 5, "local recovery lost project files");

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  assert(overflow <= 2, `horizontal overflow: ${overflow}`);
  for (const viewport of [
    { width: 1440, height: 900 },
    { width: 1024, height: 768 },
    { width: 778, height: 912 },
    { width: 390, height: 844 },
  ]) {
    await assertNativeViewport(browser, baseUrl, viewport);
  }
  console.log("seamgrim_native_project_authoring: ok");
} finally {
  await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
