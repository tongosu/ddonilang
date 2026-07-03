#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "education_publication_pack: ok";

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
    "education_classroom_ui_pack.js",
    "education_publication_pack.js",
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
    await page.waitForSelector("[data-education-publication-pack][data-education-publication-pack-status='education_publication_pack_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_EDUCATION_PUBLICATION_ROWS,
        buildEducationPublicationPack,
        formatEducationPublicationPackText,
      } = await import("./education_publication_pack.js");
      const publication = buildEducationPublicationPack({ rows: DEFAULT_EDUCATION_PUBLICATION_ROWS });
      return { publication, text: formatEducationPublicationPackText(publication) };
    });
    const publication = moduleResult.publication;
    assert(publication.schema === "ddn.education.publication_pack.v1", "schema mismatch");
    assert(publication.work_item === "HA4_PUBLIC_COURSE_PUBLICATION_PACK_V1", "work item mismatch");
    assert(publication.primary_coordinate === "하-4", "coordinate mismatch");
    assert(publication.depends_on_coordinate.join(",") === "하-3,마-4,카-4", "dependency mismatch");
    assert(publication.pack === "education_curriculum_4_v1", "pack mismatch");
    assert(publication.status === "education_publication_pack_ready", "status mismatch");
    assert(publication.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(publication.education_publication_pack_claim === true, "publication pack claim mismatch");
    assert(publication.micro_course_claim === true, "micro course mismatch");
    assert(publication.workshop_course_claim === true, "workshop course mismatch");
    assert(publication.four_week_course_claim === true, "four week course mismatch");
    assert(publication.publication_bundle_claim === true, "publication bundle mismatch");
    assert(publication.share_handoff_claim === true, "share handoff mismatch");
    assert(publication.public_upload_claim === false, "public upload must stay false");
    assert(publication.public_link_creation_claim === false, "public link must stay false");
    assert(publication.registry_publish_claim === false, "registry publish must stay false");
    assert(publication.release_execution_claim === false, "release execution must stay false");
    assert(publication.artifact_signing_claim === false, "artifact signing must stay false");
    assert(publication.account_permission_change_claim === false, "account permission must stay false");
    assert(publication.state_hash_participation_claim === false, "state hash participation must stay false");
    assert(publication.runtime_claim === false, "runtime claim must stay false");
    assert(publication.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(publication.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(publication.progress.roadmap_v2_matrix_behavior_closed === 31, "roadmap closed mismatch");
    assert(publication.progress.roadmap_v2_matrix_behavior_percent === 34, "roadmap percent mismatch");
    assert(publication.progress.roadmap_v2_pack_evidence_reference_closed === 51, "pack ref mismatch");
    assert(publication.progress.roadmap_v2_pack_evidence_reference_percent === 57, "pack ref percent mismatch");
    assert(publication.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(publication.rows.map((row) => row.id).join(",") === "micro_course,workshop_course,four_week_course,publication_bundle,share_handoff", "row order mismatch");
    assert(String(publication.publication_text).includes("public_upload:false"), "publication text missing upload boundary");
    assert(String(publication.publication_text).includes("registry_publish:false"), "publication text missing registry boundary");
    assert(String(moduleResult.text).includes("pack_evidence_reference\t51/90"), "text missing pack reference");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-education-publication-pack]");
      const buttons = Array.from(document.querySelectorAll(".education-publication-btn[data-education-publication-row]"));
      buttons.find((button) => button.getAttribute("data-education-publication-row") === "four_week_course")?.click();
      document.querySelector("[data-education-publication-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-education-publication-pack-status") || "",
        copied: root?.getAttribute("data-education-publication-pack-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-education-publication-artifact]").length,
        progress: document.querySelector("[data-education-publication-progress]")?.textContent || "",
        summary: document.querySelector("[data-education-publication-summary]")?.textContent || "",
        title: document.querySelector("[data-education-publication-active-title]")?.textContent || "",
        link: document.querySelector("[data-education-publication-active-link]")?.textContent || "",
        preview: document.querySelector("[data-education-publication-preview]")?.textContent || "",
        globalSchema: window.__EDUCATION_PUBLICATION_PACK__?.schema || "",
        globalText: window.__EDUCATION_PUBLICATION_PACK_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "education_publication_pack_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 5, `button count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 5, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("31/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("public upload") && domResult.summary.includes("public link creation") && domResult.summary.includes("release execution"), "summary missing boundary");
    assert(domResult.title === "4-week course", `active title mismatch: ${domResult.title}`);
    assert(domResult.link.includes("education://publication/4-week-course"), "four week URI mismatch");
    assert(domResult.preview.includes("public_upload:false") && domResult.preview.includes("registry_publish:false"), "preview mismatch");
    assert(domResult.globalSchema === "ddn.education.publication_pack.v1", "global schema mismatch");
    assert(domResult.globalText.includes("roadmap_matrix\t31/90"), "global text missing roadmap matrix");
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
