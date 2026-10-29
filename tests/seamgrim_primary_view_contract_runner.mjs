#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_primary_view_contract: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const WORLDS = [
  {
    lessonId: "rep_math_function_line_v1",
    primaryView: "graph",
    mode: "graph",
    axes: "x,y",
    minGraphPoints: 3,
  },
  {
    lessonId: "rep_phys_projectile_xy_v1",
    primaryView: "space2d",
    mode: "space2d",
    axes: "x,y",
    minShapes: 3,
  },
  {
    lessonId: "rep_econ_supply_demand_tax_v1",
    primaryView: "graph",
    mode: "graph",
    axes: "t,가격",
    minGraphPoints: 3,
  },
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".toml") || file.endsWith(".ddn")) return "text/plain; charset=utf-8";
  if (file.endsWith(".md")) return "text/markdown; charset=utf-8";
  return "application/octet-stream";
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
      const rawPath = decodeURIComponent(url.pathname === "/" ? PROJECT_URL : url.pathname);
      const file = path.resolve(resolvedRoot, rawPath.replace(/^\/+/, ""));
      if (file !== resolvedRoot && !file.startsWith(`${resolvedRoot}${path.sep}`)) {
        res.writeHead(403);
        res.end("forbidden");
        return;
      }
      const bytes = await fs.readFile(file);
      res.writeHead(200, {
        "content-type": mimeType(file),
        "cache-control": "no-store",
        "access-control-allow-origin": "*",
      });
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
      if (!address || typeof address === "string") {
        reject(new Error("failed to bind static server"));
        return;
      }
      resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

async function closeServer(server) {
  await new Promise((resolve) => server.close(resolve));
}

function isAllowedFallback404(urlText) {
  try {
    const url = new URL(urlText);
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/(?:graph|table|space2d|maegim_control)\.(?:json|detjson)$/i.test(pathname)
    ) return true;
    if (
      (pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/"))
      && /\/text\.md$/i.test(pathname)
    ) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    if (msg.type() !== "error") return;
    if (String(msg.text() ?? "").includes("Failed to load resource")) return;
    failures.push(`console error: ${msg.text()}`);
  });
  page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
  page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
  page.on("response", (res) => {
    if (res.status() < 400) return;
    if (res.status() === 404 && isAllowedFallback404(res.url())) return;
    failures.push(`response ${res.status()}: ${res.url()}`);
  });
}

async function verifyWorld(page, baseUrl, world) {
  await page.goto(`${baseUrl}${PROJECT_URL}?tab=studio&lesson=${world.lessonId}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction(({ mode, minGraphPoints, minShapes }) => {
    const root = document.querySelector("#screen-run");
    const observeBody = document.querySelector("#run-observe-output-body");
    if (!root || root.dataset.primaryViewContract !== "1") return false;
    if (root.dataset.primaryViewSchemaId !== "seamgrim.primary_view_rules") return false;
    if (root.dataset.primaryViewSchemaVersion !== "v1") return false;
    if (root.dataset.primaryViewHeuristicSelection !== "0") return false;
    if (!String(root.dataset.primaryViewRulesetHash ?? "").startsWith("candidate:fnv1a32:primary_view_rules:")) return false;
    if (!String(root.dataset.lessonDeclarationHash ?? "").startsWith("candidate:fnv1a32:lesson_declaration:")) return false;
    if (observeBody?.dataset?.observationPayloadSchema !== "seamgrim.observation_payload.v1") return false;
    if (Number(observeBody?.dataset?.observationPayloadCount ?? 0) < 3) return false;
    if (root.dataset.mainVisualMode !== mode) return false;
    if (mode === "graph") {
      return Number(root.dataset.mainGraphPointCount ?? 0) >= minGraphPoints;
    }
    return Number(root.dataset.mainShapeCount ?? 0) >= minShapes;
  }, world, { timeout: 20_000 });

  const evidence = await page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    const graphHost = document.querySelector("#run-main-graph-host");
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    const ddnText = document.querySelector("#run-ddn-preview")?.value ?? "";
    const observeText = document.querySelector("#run-observe-output-body")?.innerText ?? "";
    const observeBody = document.querySelector("#run-observe-output-body");
    const observationPayloadJson = observeBody?.dataset?.observationPayloadJson ?? "[]";
    let observationPayload = [];
    try {
      observationPayload = JSON.parse(observationPayloadJson);
    } catch (_) {
      observationPayload = [];
    }
    const bodyText = document.body.innerText;
    return {
      urlLesson: new URL(window.location.href).searchParams.get("lesson"),
      primaryViewContract: root?.dataset?.primaryViewContract ?? "",
      mainVisualMode: root?.dataset?.mainVisualMode ?? "",
      mainViewSource: root?.dataset?.mainViewSource ?? "",
      mainViewAxes: root?.dataset?.mainViewAxes ?? "",
      mainGraphSeriesCount: Number(root?.dataset?.mainGraphSeriesCount ?? 0),
      mainGraphPointCount: Number(root?.dataset?.mainGraphPointCount ?? 0),
      mainShapeCount: Number(root?.dataset?.mainShapeCount ?? 0),
      graphVisible: graphHost ? getComputedStyle(graphHost).display !== "none" : false,
      canvasVisible: canvas ? getComputedStyle(canvas).display !== "none" : false,
      ddnText,
      observeText,
      observationPayloadSchema: observeBody?.dataset?.observationPayloadSchema ?? "",
      observationPayloadCount: Number(observeBody?.dataset?.observationPayloadCount ?? 0),
      observationPayload,
      bodyText,
    };
  });

  assert(evidence.urlLesson === world.lessonId, `${world.lessonId}: route lesson mismatch`);
  assert(evidence.mainVisualMode === world.mode, `${world.lessonId}: primary mode mismatch ${JSON.stringify(evidence)}`);
  if (world.mode === "graph") {
    assert(evidence.mainViewSource === "observation_series", `${world.lessonId}: unexpected graph source ${evidence.mainViewSource}`);
    assert(evidence.graphVisible, `${world.lessonId}: graph host not visible`);
    assert(!evidence.canvasVisible, `${world.lessonId}: canvas should be hidden for graph primary`);
    assert(evidence.mainViewAxes === world.axes, `${world.lessonId}: axes mismatch ${evidence.mainViewAxes}`);
    assert(evidence.mainGraphSeriesCount >= 1, `${world.lessonId}: graph series missing`);
    assert(evidence.mainGraphPointCount >= world.minGraphPoints, `${world.lessonId}: graph points missing`);
  } else {
    assert(["observation", "observation_output_rows", "view_plan"].includes(evidence.mainViewSource), `${world.lessonId}: unexpected space2d source ${evidence.mainViewSource}`);
    assert(evidence.canvasVisible, `${world.lessonId}: canvas should be visible for space2d primary`);
    assert(evidence.mainShapeCount >= world.minShapes, `${world.lessonId}: space2d shapes missing`);
    assert(evidence.mainViewAxes === world.axes, `${world.lessonId}: axes mismatch ${evidence.mainViewAxes}`);
  }
  assert(evidence.ddnText.length > 80, `${world.lessonId}: DDN source not present`);
  assert(evidence.observationPayloadSchema === "seamgrim.observation_payload.v1", `${world.lessonId}: observation payload schema missing`);
  assert(evidence.observationPayloadCount >= 3, `${world.lessonId}: observation payload too small`);
  assert(Array.isArray(evidence.observationPayload) && evidence.observationPayload.length === evidence.observationPayloadCount, `${world.lessonId}: observation payload json mismatch`);
  for (const row of evidence.observationPayload.slice(0, 6)) {
    for (const key of ["id", "label", "value", "unit", "order", "delta", "role", "source_ref"]) {
      assert(Object.prototype.hasOwnProperty.call(row, key), `${world.lessonId}: observation payload missing ${key}`);
    }
    assert(String(row.id ?? "").trim(), `${world.lessonId}: observation id empty`);
    assert(String(row.label ?? "").trim(), `${world.lessonId}: observation label empty`);
    assert(Number.isFinite(Number(row.order)) && Number(row.order) >= 1, `${world.lessonId}: observation order invalid`);
    assert(String(row.source_ref ?? "").startsWith("observation."), `${world.lessonId}: observation source_ref invalid ${row.source_ref}`);
  }
  assert(evidence.observeText.includes("관찰값"), `${world.lessonId}: human observation label missing`);
  assert(!evidence.bodyText.includes("학생으로 실행"), `${world.lessonId}: old student role entry visible`);
  assert(!evidence.bodyText.includes("교사용 배포 준비"), `${world.lessonId}: old teacher role entry visible`);
}

async function main() {
  const root = path.resolve(".");
  const { server, baseUrl } = await createServer(root);
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    installFailureCapture(page, failures);
    for (const world of WORLDS) {
      await verifyWorld(page, baseUrl, world);
    }
    await page.close();
    assert(failures.length === 0, failures.join("\n"));
    process.stdout.write(`${OK}\n`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
