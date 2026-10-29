#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave3b_foundation12_curriculum_cohort: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const TARGET_LESSONS = Object.freeze([
  "elem_math_number_line",
  "elem_math_fraction_line",
  "elem_math_rectangle_area",
  "elem_science_daily_temperature",
  "elem_econ_allowance_budget",
  "elem_econ_store_stock",
  "rep_math_function_line_v1",
  "rep_phys_pendulum_wave1_v1",
  "rep_phys_projectile_xy_v1",
  "rep_econ_supply_demand_tax_v1",
  "rep_phys_damped_oscillation_v1",
  "high_math_derivative",
]);

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

async function waitForVisibleCard(page, lessonId) {
  const selector = `.lesson-card[data-lesson-id='${lessonId}']`;
  await page.waitForFunction(
    (sel) => {
      const node = document.querySelector(sel);
      if (!node) return false;
      const style = window.getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    },
    selector,
    { timeout: 20_000 },
  );
}

async function selectFilter(page, selector, value) {
  await page.selectOption(selector, value);
  await page.waitForTimeout(80);
}

async function visibleTargetIds(page) {
  return page.evaluate((targetIds) => {
    return targetIds.filter((id) => {
      const node = document.querySelector(`.lesson-card[data-lesson-id='${id}']`);
      if (!node) return false;
      const style = window.getComputedStyle(node);
      const rect = node.getBoundingClientRect();
      return style.display !== "none" && style.visibility !== "hidden" && rect.width > 0 && rect.height > 0;
    });
  }, TARGET_LESSONS);
}

async function openLessonCard(page, lessonId) {
  const selector = `.lesson-card[data-lesson-id='${lessonId}']`;
  await waitForVisibleCard(page, lessonId);
  await page.locator(selector).scrollIntoViewIfNeeded();
  await page.click(selector);
  await page.waitForSelector(`[data-curriculum-card][data-curriculum-card-lesson-id='${lessonId}']`, {
    state: "visible",
    timeout: 10_000,
  });
  const detail = await page.evaluate((id) => {
    const section = document.querySelector(`[data-curriculum-card][data-curriculum-card-lesson-id='${id}']`);
    const panel = document.querySelector("#catalog-detail-panel");
    const text = section?.textContent?.trim() ?? "";
    return {
      text,
      panelText: panel?.textContent?.trim() ?? "",
      internalLeak: ["status_axes", "candidate", "implementation", "evidence", "behavior_refs", "audit"].filter((token) =>
        text.includes(token),
      ),
      validation: window.__SEAMGRIM_CURRICULUM_CARD_VALIDATION__ ?? null,
    };
  }, lessonId);
  assert(detail.validation?.ok === true, `${lessonId}: curriculum validation failed: ${JSON.stringify(detail.validation)}`);
  assert(detail.validation?.card_count === 12, `${lessonId}: card count mismatch: ${detail.validation?.card_count}`);
  assert(detail.text.includes("학년군:"), `${lessonId}: grade band missing in student detail`);
  assert(detail.text.includes("허용 조절값:"), `${lessonId}: allowed controls missing in student detail`);
  assert(detail.text.includes("관찰값:"), `${lessonId}: observations missing in student detail`);
  assert(detail.text.includes("예상:"), `${lessonId}: prediction prompt missing in student detail`);
  assert(detail.text.includes("실험:"), `${lessonId}: inquiry prompt missing in student detail`);
  assert(detail.internalLeak.length === 0, `${lessonId}: internal card fields leaked: ${detail.internalLeak.join(",")}`);
}

async function openLessonInStudio(page, lessonId) {
  await page.click("#btn-open-in-studio");
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction(
    (id) => {
      const hash = document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "";
      const body = document.querySelector("[data-testid='lesson-observe-output-body']") ?? document.querySelector("#run-observe-output-body");
      return hash.includes("blake3:") && Number(body?.dataset?.observationPayloadCount ?? 0) > 0 && document.body.textContent.includes(id);
    },
    lessonId,
    { timeout: 20_000 },
  );
  const runState = await page.evaluate(() => {
    const body = document.querySelector("[data-testid='lesson-observe-output-body']") ?? document.querySelector("#run-observe-output-body");
    const errorPanel = document.querySelector("[data-testid='lesson-error-panel']");
    return {
      url: location.href,
      hashText: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      outputCount: Number(body?.dataset?.observationPayloadCount ?? 0),
      outputJson: String(body?.dataset?.observationPayloadJson ?? ""),
      errorHidden: errorPanel?.classList.contains("hidden") ?? false,
      errorText: errorPanel?.textContent?.trim() ?? "",
      roleToggleCount: document.querySelectorAll("[data-role-toggle], #role-toggle, .role-toggle").length,
    };
  });
  assert(runState.url.includes(`lesson=${encodeURIComponent(lessonId)}`), `${lessonId}: route did not preserve lesson id: ${runState.url}`);
  assert(runState.hashText.includes("blake3:"), `${lessonId}: missing blake3 state hash: ${runState.hashText}`);
  assert(runState.errorHidden, `${lessonId}: run error panel visible: ${runState.errorText}`);
  assert(runState.outputCount > 0, `${lessonId}: observation output missing`);
  assert(runState.outputJson.includes("{"), `${lessonId}: observation payload JSON missing`);
  assert(runState.roleToggleCount === 0, `${lessonId}: runtime role toggle appeared`);
}

async function main() {
  const root = path.resolve(".");
  const { server, baseUrl } = await createServer(root);
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("console", (msg) => {
      const text = String(msg.text() ?? "");
      if (msg.type() === "error" && !text.includes("Failed to load resource")) failures.push(`console error: ${text}`);
    });
    await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
    for (const lessonId of TARGET_LESSONS) {
      await waitForVisibleCard(page, lessonId);
    }
    const allVisible = await visibleTargetIds(page);
    assert(allVisible.length === 12, `default catalog did not expose all 12 cards: ${allVisible.join(",")}`);

    await selectFilter(page, "#filter-grade", "elementary");
    const elementaryVisible = await visibleTargetIds(page);
    assert(elementaryVisible.length === 6, `elementary filter mismatch: ${elementaryVisible.join(",")}`);
    assert(elementaryVisible.every((id) => id.startsWith("elem_")), `elementary filter leaked non-elementary cards: ${elementaryVisible.join(",")}`);

    await selectFilter(page, "#filter-grade", "");
    await selectFilter(page, "#filter-subject", "math");
    const mathVisible = await visibleTargetIds(page);
    assert(mathVisible.includes("high_math_derivative"), `math filter omitted high derivative: ${mathVisible.join(",")}`);
    assert(mathVisible.includes("rep_math_function_line_v1"), `math filter omitted line function: ${mathVisible.join(",")}`);

    await selectFilter(page, "#filter-subject", "");
    for (const lessonId of TARGET_LESSONS) {
      await openLessonCard(page, lessonId);
      await openLessonInStudio(page, lessonId);
      await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
    }
    if (failures.length) throw new Error(failures.join("\n"));
  } finally {
    await browser.close();
    await closeServer(server);
  }
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err?.message || String(err));
  process.exit(1);
});
