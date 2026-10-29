#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { assert, closeServer, collectPageFailures, createServer, requireFile } from "./ganada_browser_runner_lib.mjs";

const OK = "ddn_local_project_workbench_multifile: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";

function base64UrlUtf8(text) {
  return Buffer.from(String(text ?? ""), "utf8").toString("base64");
}

async function clickProjectFile(page, filePath) {
  await page.locator(`[data-project-file-path="${filePath}"]`).click();
}

async function readProjectRunSummary(page) {
  const raw = await page.locator("[data-testid='project-run-result']").textContent();
  const parsed = JSON.parse(String(raw ?? "{}"));
  assert(parsed.schema === "ddn.local_project_run_summary.v1", `project run schema mismatch: ${parsed.schema}`);
  return parsed;
}

async function verifyWorkbench(page, baseUrl) {
  const ddnText = `설정 {
  제목: G9_로컬프로젝트.
}.

1 보여주기.
`;
  const encoded = base64UrlUtf8(ddnText);
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&ddn=${encodeURIComponent(encoded)}`, {
    waitUntil: "domcontentloaded",
  });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.locator("#screen-run [data-shell-space='create']").click();
  await page.waitForSelector("#screen-editor:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForSelector("#ddn-textarea", { state: "visible", timeout: 10_000 });

  const oneFile = await page.evaluate(() => ({
    mode: document.querySelector("[data-project-workbench-panel]")?.dataset?.projectMode ?? "",
    textareaVisible: Boolean(document.querySelector("#ddn-textarea")) && !document.querySelector("#ddn-textarea")?.classList.contains("hidden"),
    runVisible: Boolean(document.querySelector("#btn-run-from-editor")),
    status: document.querySelector("[data-testid='project-dependency-status']")?.textContent?.trim() ?? "",
  }));
  assert(oneFile.mode === "0", `project mode should be off initially: ${JSON.stringify(oneFile)}`);
  assert(oneFile.textareaVisible, "single-file textarea hidden before project mode");
  assert(oneFile.runVisible, "single-file run button missing");
  assert(oneFile.status.includes("단일 파일"), `single-file status mismatch: ${oneFile.status}`);

  await page.locator("[data-testid='project-enable']").click();
  await page.waitForFunction(() => document.querySelector("[data-project-workbench-panel]")?.dataset?.projectMode === "1");
  await page.waitForSelector("[data-project-file-path='app/main.ddn']", { state: "visible", timeout: 10_000 });
  const seeded = await page.evaluate(() => ({
    current: document.querySelector("[data-testid='project-current-file']")?.textContent?.trim() ?? "",
    entry: document.querySelector("[data-testid='project-entry-file']")?.textContent?.trim() ?? "",
    files: [...document.querySelectorAll("[data-project-file-path]")].map((node) => ({
      path: node.getAttribute("data-project-file-path"),
      role: node.getAttribute("data-project-role"),
      readonly: node.getAttribute("data-readonly"),
    })),
  }));
  assert(seeded.current.includes("app/main.ddn"), `current file mismatch: ${JSON.stringify(seeded)}`);
  assert(seeded.entry.includes("app/main.ddn"), `entry indicator mismatch: ${JSON.stringify(seeded)}`);
  for (const required of ["뿌리.ddn", "app/main.ddn", "lib/helper.ddn", "ddn.lock", "vendor/edu.physics/gaji.toml"]) {
    assert(seeded.files.some((row) => row.path === required), `seeded file missing: ${required}`);
  }

  for (const protectedPath of ["뿌리.ddn", "ddn.lock", "vendor/edu.physics/gaji.toml"]) {
    await clickProjectFile(page, protectedPath);
    const guard = await page.evaluate(() => ({
      current: document.querySelector("[data-project-workbench-panel]")?.dataset?.currentPath ?? "",
      readonly: document.querySelector("#ddn-textarea")?.readOnly ?? false,
      renameDisabled: document.querySelector("[data-testid='project-file-rename']")?.disabled ?? false,
      deleteDisabled: document.querySelector("[data-testid='project-file-delete']")?.disabled ?? false,
    }));
    assert(guard.current === protectedPath, `protected current mismatch: ${JSON.stringify(guard)}`);
    assert(guard.readonly, `protected file should be read-only: ${protectedPath}`);
    assert(guard.renameDisabled && guard.deleteDisabled, `protected controls enabled: ${JSON.stringify(guard)}`);
  }

  await clickProjectFile(page, "app/main.ddn");
  const entryGuard = await page.evaluate(() => ({
    readonly: document.querySelector("#ddn-textarea")?.readOnly ?? true,
    renameDisabled: document.querySelector("[data-testid='project-file-rename']")?.disabled ?? false,
    deleteDisabled: document.querySelector("[data-testid='project-file-delete']")?.disabled ?? false,
  }));
  assert(!entryGuard.readonly, `entry should be editable: ${JSON.stringify(entryGuard)}`);
  assert(entryGuard.renameDisabled && entryGuard.deleteDisabled, `entry guard mismatch: ${JSON.stringify(entryGuard)}`);

  await page.locator("[data-testid='project-file-create']").click();
  await page.waitForSelector("[data-project-file-path='lib/helper_extra.ddn']", { state: "visible", timeout: 10_000 });
  await page.locator("[data-testid='project-file-delete']").click();
  await page.waitForFunction(() => !document.querySelector("[data-project-file-path='lib/helper_extra.ddn']"));

  await clickProjectFile(page, "lib/helper.ddn");
  await page.locator("[data-testid='project-file-rename']").click();
  await page.waitForSelector("[data-project-file-path='lib/helper_renamed.ddn']", { state: "visible", timeout: 10_000 });
  const renameState = await page.evaluate(() => ({
    hasOld: Boolean(document.querySelector("[data-project-file-path='lib/helper.ddn']")),
    current: document.querySelector("[data-project-workbench-panel]")?.dataset?.currentPath ?? "",
  }));
  assert(!renameState.hasOld, `renamed file left old path: ${JSON.stringify(renameState)}`);
  assert(renameState.current === "lib/helper_renamed.ddn", `renamed current mismatch: ${JSON.stringify(renameState)}`);

  await clickProjectFile(page, "app/main.ddn");
  await page.fill("#ddn-textarea", `쓰임 {
  도움: "./lib/helper_renamed".
}
도움.값 보여주기.
`);
  await page.locator("[data-testid='project-run']").click();
  await page.waitForFunction(() => {
    const text = document.querySelector("[data-testid='project-run-result']")?.textContent ?? "";
    return text.includes("ddn.local_project_run_summary.v1") && text.includes("\"11\"");
  }, null, { timeout: 30_000 });
  const firstSummary = await readProjectRunSummary(page);
  assert(JSON.stringify(firstSummary.output_rows) === JSON.stringify(["11"]), `first output mismatch: ${JSON.stringify(firstSummary.output_rows)}`);
  assert(firstSummary.graph_inspected === true, "graph inspection not recorded");
  assert(firstSummary.lock_inspected === true, "lock inspection not recorded");
  assert(String(firstSummary.bounded_state_sha256 ?? "").length >= 32, "bounded state hash missing");
  assert(firstSummary.project_observation_trace?.schema === "ddn.local_project.observation_trace.v1", "first observation trace missing");
  assert(JSON.stringify(firstSummary.project_observation_trace.rows.map((row) => row.value_text)) === JSON.stringify(["11"]), "first observation trace values mismatch");
  assert(firstSummary.project_observation_trace.rows.every((row) => row.state_hash_participation === false), "first observation trace claims state hash participation");
  const firstLifecycle = await page.evaluate(() => ({
    status: document.querySelector("[data-testid='project-run-lifecycle']")?.dataset?.runtimeStatus ?? "",
    activeRunId: document.querySelector("[data-testid='project-run-lifecycle']")?.dataset?.activeRunId ?? "",
    archiveCount: document.querySelector("[data-testid='project-run-lifecycle']")?.dataset?.archiveCount ?? "",
    traceHash: document.querySelector("[data-testid='project-observation-trace']")?.dataset?.traceSha256 ?? "",
    traceRowCount: document.querySelector("[data-testid='project-observation-trace']")?.dataset?.rowCount ?? "",
    observations: [...document.querySelectorAll("[data-testid='project-observation-rows'] [data-project-observation-id]")].map((node) => ({
      id: node.getAttribute("data-project-observation-id"),
      name: node.getAttribute("data-project-observation-name"),
      value: node.getAttribute("data-project-observation-value"),
      stateHashParticipation: node.getAttribute("data-state-hash-participation"),
    })),
    rows: [...document.querySelectorAll("[data-testid='project-run-history'] [data-project-run-id]")].map((node) => ({
      runId: node.getAttribute("data-project-run-id"),
      status: node.getAttribute("data-project-run-status"),
      sourceHash: node.getAttribute("data-source-candidate-hash"),
      summaryHash: node.getAttribute("data-summary-sha256"),
      text: node.textContent,
    })),
  }));
  assert(firstLifecycle.status === "completed", `first lifecycle status mismatch: ${JSON.stringify(firstLifecycle)}`);
  assert(firstLifecycle.activeRunId === "project_run_001", `first active run mismatch: ${JSON.stringify(firstLifecycle)}`);
  assert(firstLifecycle.archiveCount === "1", `first archive count mismatch: ${JSON.stringify(firstLifecycle)}`);
  assert(firstLifecycle.rows.length === 1 && firstLifecycle.rows[0].status === "completed", `first history mismatch: ${JSON.stringify(firstLifecycle)}`);
  assert(String(firstLifecycle.rows[0].summaryHash ?? "") === firstSummary.summary_sha256, "first lifecycle summary hash mismatch");
  assert(String(firstLifecycle.rows[0].sourceHash ?? "").startsWith("candidate:fnv1a32:project_source:"), "first source candidate hash missing");
  assert(firstLifecycle.traceHash === firstSummary.project_observation_trace.trace_sha256, "first trace hash DOM mismatch");
  assert(firstLifecycle.traceRowCount === "1", `first trace row count mismatch: ${JSON.stringify(firstLifecycle)}`);
  assert(firstLifecycle.observations.length === 1 && firstLifecycle.observations[0].value === "11", `first observation DOM mismatch: ${JSON.stringify(firstLifecycle)}`);
  assert(firstLifecycle.observations[0].stateHashParticipation === "0", "first observation state hash boundary mismatch");

  const firstStatus = await page.locator("[data-testid='project-dependency-status']").textContent();
  assert(String(firstStatus ?? "").includes(String(firstSummary.project_graph_sha256).slice(0, 12)), "dependency status missing graph hash");

  await clickProjectFile(page, "lib/helper_renamed.ddn");
  await page.fill("#ddn-textarea", `드러냄 {
  값.
}
값 <- 12.
`);
  await page.locator("[data-testid='project-run']").click();
  await page.waitForFunction(() => {
    const text = document.querySelector("[data-testid='project-run-result']")?.textContent ?? "";
    return text.includes("ddn.local_project_run_summary.v1") && text.includes("\"12\"");
  }, null, { timeout: 30_000 });
  const secondSummary = await readProjectRunSummary(page);
  assert(JSON.stringify(secondSummary.output_rows) === JSON.stringify(["12"]), `second output mismatch: ${JSON.stringify(secondSummary.output_rows)}`);
  assert(secondSummary.project_graph_sha256 !== firstSummary.project_graph_sha256, "graph hash did not change after helper edit");
  assert(JSON.stringify(secondSummary.project_observation_trace.rows.map((row) => row.value_text)) === JSON.stringify(["12"]), "second observation trace values mismatch");
  assert(secondSummary.project_observation_trace.trace_sha256 !== firstSummary.project_observation_trace.trace_sha256, "trace hash did not change after helper edit");
  const secondLifecycle = await page.evaluate(() => ({
    status: document.querySelector("[data-testid='project-run-lifecycle']")?.dataset?.runtimeStatus ?? "",
    activeRunId: document.querySelector("[data-testid='project-run-lifecycle']")?.dataset?.activeRunId ?? "",
    archiveCount: document.querySelector("[data-testid='project-run-lifecycle']")?.dataset?.archiveCount ?? "",
    traceHash: document.querySelector("[data-testid='project-observation-trace']")?.dataset?.traceSha256 ?? "",
    traceRowCount: document.querySelector("[data-testid='project-observation-trace']")?.dataset?.rowCount ?? "",
    observations: [...document.querySelectorAll("[data-testid='project-observation-rows'] [data-project-observation-id]")].map((node) => ({
      id: node.getAttribute("data-project-observation-id"),
      name: node.getAttribute("data-project-observation-name"),
      value: node.getAttribute("data-project-observation-value"),
      stateHashParticipation: node.getAttribute("data-state-hash-participation"),
    })),
    rows: [...document.querySelectorAll("[data-testid='project-run-history'] [data-project-run-id]")].map((node) => ({
      runId: node.getAttribute("data-project-run-id"),
      status: node.getAttribute("data-project-run-status"),
      sourceHash: node.getAttribute("data-source-candidate-hash"),
      summaryHash: node.getAttribute("data-summary-sha256"),
      text: node.textContent,
    })),
  }));
  assert(secondLifecycle.status === "completed", `second lifecycle status mismatch: ${JSON.stringify(secondLifecycle)}`);
  assert(secondLifecycle.activeRunId === "project_run_002", `second active run mismatch: ${JSON.stringify(secondLifecycle)}`);
  assert(secondLifecycle.archiveCount === "2", `second archive count mismatch: ${JSON.stringify(secondLifecycle)}`);
  assert(secondLifecycle.rows.length === 2, `second history count mismatch: ${JSON.stringify(secondLifecycle)}`);
  assert(secondLifecycle.rows[0].runId === "project_run_002" && secondLifecycle.rows[1].runId === "project_run_001", `history order mismatch: ${JSON.stringify(secondLifecycle)}`);
  assert(secondLifecycle.rows[0].summaryHash === secondSummary.summary_sha256, "second lifecycle summary hash mismatch");
  assert(secondLifecycle.rows[0].sourceHash !== firstLifecycle.rows[0].sourceHash, "source candidate hash did not change after edit");
  assert(secondLifecycle.traceHash === secondSummary.project_observation_trace.trace_sha256, "second trace hash DOM mismatch");
  assert(secondLifecycle.traceRowCount === "1", `second trace row count mismatch: ${JSON.stringify(secondLifecycle)}`);
  assert(secondLifecycle.observations.length === 1 && secondLifecycle.observations[0].value === "12", `second observation DOM mismatch: ${JSON.stringify(secondLifecycle)}`);
}

async function main() {
  const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  await requireFile(path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "index.html"));
  await requireFile(path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool.js"));
  await requireFile(path.join(rootDir, "solutions", "seamgrim_ui_mvp", "ui", "wasm", "ddonirang_tool_bg.wasm"));
  const { server, baseUrl } = await createServer(rootDir);
  const browser = await chromium.launch({ headless: true });
  let failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await context.newPage();
    failures = collectPageFailures(page);
    await verifyWorkbench(page, baseUrl);
    await context.close();
  } finally {
    await browser.close();
    await closeServer(server);
  }
  const actionableFailures = failures.filter((line) => {
    if (/response 404: .*\/solutions\/seamgrim_ui_mvp\/lessons\/.*\/(?:graph\.json|table\.json|space2d\.json|text\.md|maegim_control\.json)$/u.test(line)) {
      return false;
    }
    return true;
  });
  assert(actionableFailures.length === 0, `browser failures: ${actionableFailures.join("; ")}`);
  process.stdout.write(`${OK}\n`);
}

main().catch((err) => {
  process.stderr.write(`${err?.stack ?? err}\n`);
  process.exit(1);
});
