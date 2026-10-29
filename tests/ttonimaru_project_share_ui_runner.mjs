#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "ttonimaru_project_share_ui: ok";

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
    "ttonimaru_publication_read_api.js",
    "ttonimaru_project_share_ui.js",
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
    await page.waitForSelector("[data-ttonimaru-project-share-ui][data-ttonimaru-project-share-ui-status='ttonimaru_project_share_ui_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TTONIMARU_PROJECT_SHARE_ROWS,
        buildTtonimaruProjectShareUi,
        formatTtonimaruProjectShareUiText,
      } = await import("./ttonimaru_project_share_ui.js");
      const projectShare = buildTtonimaruProjectShareUi({ rows: DEFAULT_TTONIMARU_PROJECT_SHARE_ROWS });
      return {
        projectShare,
        text: formatTtonimaruProjectShareUiText(projectShare),
      };
    });
    const share = moduleResult.projectShare;
    assert(share.schema === "ddn.ttonimaru.project_share_ui.v1", "schema mismatch");
    assert(share.work_item === "KA3_PROJECT_SHARE_UI_V1", "work item mismatch");
    assert(share.primary_coordinate === "카-3", "coordinate mismatch");
    assert(share.depends_on_coordinate.join(",") === "카-2", "dependency mismatch");
    assert(share.status === "ttonimaru_project_share_ui_ready", "status mismatch");
    assert(share.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(share.project_share_ui_claim === true, "project share UI claim mismatch");
    assert(share.project_snapshot_claim === true, "project snapshot claim mismatch");
    assert(share.revision_pin_claim === true, "revision pin claim mismatch");
    assert(share.share_link_claim === true, "share link claim mismatch");
    assert(share.remix_handoff_claim === true, "remix handoff claim mismatch");
    assert(share.product_ui_change === true, "product UI change mismatch");
    assert(share.runtime_claim === false, "runtime claim must stay false");
    assert(share.public_registry_seed_claim === false, "public registry seed must stay false");
    assert(share.public_registry_final_claim === false, "public registry final must stay false");
    assert(share.registry_publish_claim === false, "registry publish must stay false");
    assert(share.install_update_remove_claim === false, "install/update/remove must stay false");
    assert(share.trust_signing_claim === false, "trust signing must stay false");
    assert(share.team_membership_claim === false, "team membership must stay false");
    assert(share.account_permission_claim === false, "account permission must stay false");
    assert(share.cloud_sync_claim === false, "cloud sync must stay false");
    assert(share.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(share.progress.current_stage_total === 5, "stage total mismatch");
    assert(share.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(share.progress.roadmap_v2_matrix_behavior_closed === 19, "roadmap closed mismatch");
    assert(share.progress.roadmap_v2_matrix_behavior_percent === 21, "roadmap percent mismatch");
    assert(share.progress.roadmap_v2_pack_evidence_reference_closed === 39, "pack ref mismatch");
    assert(share.progress.roadmap_v2_pack_evidence_reference_percent === 43, "pack ref percent mismatch");
    assert(share.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(share.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(share.share_rows.map((row) => row.id).join(",") === "project_snapshot,revision_pin,share_link,remix_handoff", "share row order mismatch");
    assert(share.artifacts.map((file) => file.kind).join(",") === "project_snapshot,revision_pin,share_link,remix_handoff", "artifact order mismatch");
    assert(String(share.share_text).includes("public_registry_seed:false"), "share text missing seed boundary");
    assert(String(moduleResult.text).includes("project_share_ui_claim\ttrue"), "text missing share UI claim");
    assert(String(moduleResult.text).includes("public_registry_seed_claim\tfalse"), "text missing seed boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t19/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ttonimaru-project-share-ui]");
      const buttons = Array.from(document.querySelectorAll(".ttonimaru-share-btn[data-ttonimaru-share]"));
      buttons.find((button) => button.getAttribute("data-ttonimaru-share") === "share_link")?.click();
      document.querySelector("[data-ttonimaru-share-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-ttonimaru-project-share-ui-status") || "",
        copied: root?.getAttribute("data-ttonimaru-project-share-ui-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-ttonimaru-share-artifact]").length,
        progress: document.querySelector("[data-ttonimaru-share-progress]")?.textContent || "",
        summary: document.querySelector("[data-ttonimaru-share-summary]")?.textContent || "",
        title: document.querySelector("[data-ttonimaru-share-active-title]")?.textContent || "",
        link: document.querySelector("[data-ttonimaru-share-active-link]")?.textContent || "",
        preview: document.querySelector("[data-ttonimaru-share-preview]")?.textContent || "",
        globalSchema: window.__TTONIMARU_PROJECT_SHARE_UI__?.schema || "",
        globalText: window.__TTONIMARU_PROJECT_SHARE_UI_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "ttonimaru_project_share_ui_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `share row count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("19/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Project snapshot") && domResult.summary.includes("public registry seed") && domResult.summary.includes("cloud sync"), "summary missing scope boundary");
    assert(domResult.title === "Share link", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("ttonimaru://project/share/local/link"), "share link URI mismatch");
    assert(domResult.preview.includes("project.snapshot.detjson") && domResult.preview.includes("public_registry_seed:false"), "share preview mismatch");
    assert(domResult.globalSchema === "ddn.ttonimaru.project_share_ui.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t39/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("account_permission_claim\tfalse"), "global text missing account permission boundary");

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
