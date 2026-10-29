#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "ttonimaru_publication_read_api: ok";

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
    await page.waitForSelector("[data-ttonimaru-publication-read-api][data-ttonimaru-publication-read-api-status='ttonimaru_publication_read_api_ready']");

    const moduleResult = await page.evaluate(async () => {
      const {
        DEFAULT_TTONIMARU_PUBLICATION_READ_API_ROWS,
        buildTtonimaruPublicationReadApi,
        formatTtonimaruPublicationReadApiText,
      } = await import("./ttonimaru_publication_read_api.js");
      const publicationReadApi = buildTtonimaruPublicationReadApi({ rows: DEFAULT_TTONIMARU_PUBLICATION_READ_API_ROWS });
      return {
        publicationReadApi,
        text: formatTtonimaruPublicationReadApiText(publicationReadApi),
      };
    });
    const api = moduleResult.publicationReadApi;
    assert(api.schema === "ddn.ttonimaru.publication_read_api.v1", "schema mismatch");
    assert(api.work_item === "KA2_PUBLICATION_READ_API_CLOSURE_V1", "work item mismatch");
    assert(api.primary_coordinate === "카-2", "coordinate mismatch");
    assert(api.depends_on_coordinate.join(",") === "카-1", "dependency mismatch");
    assert(api.status === "ttonimaru_publication_read_api_ready", "status mismatch");
    assert(api.matrix_closure_tier === "닫힘-동작", "closure tier mismatch");
    assert(api.publication_read_api_claim === true, "publication read API claim mismatch");
    assert(api.immutable_publication_claim === true, "immutable publication claim mismatch");
    assert(api.read_only_api_claim === true, "read-only API claim mismatch");
    assert(api.revision_pin_claim === true, "revision pin claim mismatch");
    assert(api.package_metadata_read_claim === true, "metadata read claim mismatch");
    assert(api.product_ui_change === true, "product UI change mismatch");
    assert(api.runtime_claim === false, "runtime claim must stay false");
    assert(api.public_registry_final_claim === false, "public registry final must stay false");
    assert(api.mutation_api_claim === false, "mutation API must stay false");
    assert(api.registry_publish_claim === false, "registry publish must stay false");
    assert(api.install_update_remove_claim === false, "install/update/remove must stay false");
    assert(api.trust_signing_claim === false, "trust signing must stay false");
    assert(api.team_membership_claim === false, "team membership must stay false");
    assert(api.progress.current_stage_closed === 5, "stage closed mismatch");
    assert(api.progress.current_stage_total === 5, "stage total mismatch");
    assert(api.progress.current_stage_percent === 100, "stage percent mismatch");
    assert(api.progress.roadmap_v2_matrix_behavior_closed === 18, "roadmap closed mismatch");
    assert(api.progress.roadmap_v2_matrix_behavior_percent === 20, "roadmap percent mismatch");
    assert(api.progress.roadmap_v2_pack_evidence_reference_closed === 38, "pack ref mismatch");
    assert(api.progress.roadmap_v2_pack_evidence_reference_percent === 42, "pack ref percent mismatch");
    assert(api.progress.studio_local_super_long_closed === 9, "studio-local closed mismatch");
    assert(api.progress.studio_local_super_long_percent === 50, "studio-local percent mismatch");
    assert(api.endpoints.map((row) => row.id).join(",") === "publication_read,manifest_read,package_metadata,alias_redirect", "endpoint order mismatch");
    assert(api.artifacts.map((file) => file.kind).join(",") === "api_contract,read_fixture,manifest_v1,metadata_read", "artifact order mismatch");
    assert(String(api.api_text).includes("public_registry_final:false"), "API text missing final boundary");
    assert(String(moduleResult.text).includes("publication_read_api_claim\ttrue"), "text missing read API claim");
    assert(String(moduleResult.text).includes("public_registry_final_claim\tfalse"), "text missing final boundary");
    assert(String(moduleResult.text).includes("roadmap_matrix\t18/90"), "text missing roadmap progress");

    const domResult = await page.evaluate(async () => {
      const root = document.querySelector("[data-ttonimaru-publication-read-api]");
      const buttons = Array.from(document.querySelectorAll(".ttonimaru-publication-btn[data-ttonimaru-publication-endpoint]"));
      buttons.find((button) => button.getAttribute("data-ttonimaru-publication-endpoint") === "package_metadata")?.click();
      document.querySelector("[data-ttonimaru-publication-copy]")?.click();
      await new Promise((resolve) => setTimeout(resolve, 0));
      return {
        rootStatus: root?.getAttribute("data-ttonimaru-publication-read-api-status") || "",
        copied: root?.getAttribute("data-ttonimaru-publication-read-api-copied") || "",
        buttonCount: buttons.length,
        artifactCount: document.querySelectorAll("[data-ttonimaru-publication-artifact]").length,
        progress: document.querySelector("[data-ttonimaru-publication-progress]")?.textContent || "",
        summary: document.querySelector("[data-ttonimaru-publication-summary]")?.textContent || "",
        title: document.querySelector("[data-ttonimaru-publication-active-title]")?.textContent || "",
        endpoint: document.querySelector("[data-ttonimaru-publication-active-endpoint]")?.textContent || "",
        preview: document.querySelector("[data-ttonimaru-publication-preview]")?.textContent || "",
        globalSchema: window.__TTONIMARU_PUBLICATION_READ_API__?.schema || "",
        globalText: window.__TTONIMARU_PUBLICATION_READ_API_TEXT__ || "",
      };
    });
    assert(domResult.rootStatus === "ttonimaru_publication_read_api_ready", `DOM status mismatch: ${domResult.rootStatus}`);
    assert(domResult.copied === "true", "copy state missing");
    assert(domResult.buttonCount === 4, `endpoint count mismatch: ${domResult.buttonCount}`);
    assert(domResult.artifactCount === 4, `artifact count mismatch: ${domResult.artifactCount}`);
    assert(domResult.progress.includes("18/90 ROADMAP") && domResult.progress.includes("5/5 stage") && domResult.progress.includes("100%"), `progress mismatch: ${domResult.progress}`);
    assert(domResult.summary.includes("Immutable publication") && domResult.summary.includes("trust signing") && domResult.summary.includes("install/update/remove"), "summary missing scope boundary");
    assert(domResult.title === "Package metadata", `active title mismatch: ${domResult.title}`);
    assert(domResult.endpoint.includes("/api/v1/registry/packages/"), "metadata endpoint mismatch");
    assert(domResult.preview.includes("publication.api.contract.detjson") && domResult.preview.includes("public_registry_final:false"), "API preview mismatch");
    assert(domResult.globalSchema === "ddn.ttonimaru.publication_read_api.v1", "global schema mismatch");
    assert(domResult.globalText.includes("pack_evidence_reference\t38/90"), "global text missing pack reference");
    assert(domResult.globalText.includes("trust_signing_claim\tfalse"), "global text missing trust boundary");

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
