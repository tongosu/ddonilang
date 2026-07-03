#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "free_lab_research_workflow: ok";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) throw new Error(`missing file: ${file}`);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  return "text/plain; charset=utf-8";
}

function createServer(root) {
  const resolvedRoot = path.resolve(root);
  const server = http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url || "/", "http://127.0.0.1");
      if (url.pathname === "/favicon.ico") {
        res.writeHead(204, { "cache-control": "no-store" });
        res.end();
        return;
      }
      const rawPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, { "content-type": mimeType(file), "cache-control": "no-store" });
      res.end(bytes);
    } catch (_) {
      res.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
      res.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") reject(new Error("failed to bind static server"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    return (
      url.pathname === "/api/lessons/inventory" ||
      url.pathname === "/api/lesson-inventory" ||
      url.pathname.startsWith("/lessons/") ||
      url.pathname.startsWith("/seed_lessons_v1/")
    );
  } catch (_) {
    return false;
  }
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  for (const rel of [
    "index.html",
    "app.js",
    "styles.css",
    "free_lab_share_pack.js",
    "free_lab_research_workflow.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, locale: "ko-KR" });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error" && !String(msg.text() ?? "").includes("Failed to load resource")) {
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !res.url().endsWith("/favicon.ico") && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("[data-free-lab-research-workflow][data-free-lab-research-workflow-status='free_lab_research_workflow_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_FREE_LAB_RESEARCH_WORKFLOW_ROWS,
        buildFreeLabResearchWorkflow,
        formatFreeLabResearchWorkflowText,
      } = await import("./free_lab_research_workflow.js");
      const workflow = buildFreeLabResearchWorkflow({ rows: DEFAULT_FREE_LAB_RESEARCH_WORKFLOW_ROWS });
      return {
        workflow,
        text: formatFreeLabResearchWorkflowText(workflow),
      };
    });
    const workflow = moduleResult.workflow;
    assert(workflow.schema === "ddn.seamgrim.free_lab.research_workflow.v1", "schema mismatch");
    assert(workflow.work_item === "BA5_FREE_LAB_RESEARCH_WORKFLOW_CLOSURE_V1", "work item mismatch");
    assert(workflow.primary_coordinate === "바-5", "coordinate mismatch");
    assert(workflow.status === "free_lab_research_workflow_ready", "status mismatch");
    assert(workflow.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(workflow.research_mode_claim === true, "research mode claim mismatch");
    assert(workflow.batch_queue_claim === true, "batch queue claim mismatch");
    assert(workflow.csv_export_claim === true, "csv export claim mismatch");
    assert(workflow.notebook_handoff_claim === true, "notebook handoff claim mismatch");
    assert(workflow.product_ui_change === true, "product UI change mismatch");
    assert(workflow.runtime_claim === false, "runtime claim must stay false");
    assert(workflow.public_upload_claim === false, "public upload claim must stay false");
    assert(workflow.registry_publish_claim === false, "registry publish claim must stay false");
    assert(workflow.cloud_sync_claim === false, "cloud sync claim must stay false");
    assert(workflow.external_notebook_server_claim === false, "external notebook server claim must stay false");
    assert(workflow.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(workflow.progress.current_stage_total === 5, "stage total mismatch");
    assert(workflow.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(workflow.progress.roadmap_v2_matrix_behavior_closed === 12, "roadmap closed mismatch");
    assert(workflow.progress.roadmap_v2_matrix_behavior_percent === 13, "roadmap percent mismatch");
    assert(workflow.progress.roadmap_v2_pack_evidence_reference_closed === 31, "pack ref mismatch");
    assert(workflow.progress.roadmap_v2_pack_evidence_reference_percent === 34, "pack ref percent mismatch");
    assert(workflow.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(workflow.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(workflow.workflows.map((row) => row.id).join(",") === "batch,csv,notebook", "workflow order mismatch");
    assert(workflow.batch_runs.map((run) => run.id).join(",") === "baseline,low_lever,high_lever", "batch run order mismatch");
    assert(String(workflow.csv_text).includes("run_id,coefficient,start_value,frame_3_value"), "csv header missing");
    assert(String(moduleResult.text).includes("research_mode_claim\ttrue"), "text missing research claim");
    assert(String(moduleResult.text).includes("notebook_handoff_claim\ttrue"), "text missing notebook claim");
    assert(String(moduleResult.text).includes("public_upload_claim\tfalse"), "text missing public upload boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t12/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-free-lab-research-workflow]");
      const buttons = Array.from(document.querySelectorAll("[data-free-lab-research]"));
      buttons.find((button) => button.getAttribute("data-free-lab-research") === "notebook")?.click();
      document.querySelector("[data-free-lab-research-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-free-lab-research-workflow-status") || "",
        copied: root?.getAttribute("data-free-lab-research-workflow-copied") || "",
        buttonCount: buttons.length,
        progress: document.querySelector("[data-free-lab-research-progress]")?.textContent || "",
        summary: document.querySelector("[data-free-lab-research-summary]")?.textContent || "",
        title: document.querySelector("[data-free-lab-research-active-title]")?.textContent || "",
        link: document.querySelector("[data-free-lab-research-active-link]")?.textContent || "",
        csv: document.querySelector("[data-free-lab-research-csv]")?.textContent || "",
        globalSchema: window.__SEAMGRIM_FREE_LAB_RESEARCH_WORKFLOW__?.schema || "",
        globalText: window.__SEAMGRIM_FREE_LAB_RESEARCH_WORKFLOW_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "free_lab_research_workflow_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 3, `workflow count mismatch: ${domResult.buttonCount}`);
    assert(domResult.progress.includes("12/90 ROADMAP") && domResult.progress.includes("5/5 stage"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("batch queue") && domResult.summary.includes("CSV export") && domResult.summary.includes("notebook handoff"), "summary missing research workflow");
    assert(domResult.title === "Notebook handoff", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("seamgrim-notebook://local/"), "notebook handoff link mismatch");
    assert(domResult.csv.includes("baseline,2,1,7") && domResult.csv.includes("high_lever,3,1,10"), "csv preview mismatch");
    assert(domResult.globalSchema === "ddn.seamgrim.free_lab.research_workflow.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t31/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("cloud_sync_claim\tfalse"), "global text missing cloud boundary");

    if (failures.length > 0) throw new Error(failures.join("\n"));
    await context.close();
  } finally {
    if (browser) await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
