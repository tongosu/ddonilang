#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { chromium } from "playwright";

const OK = "seamgrim_curriculum_cards: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";

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

async function readJson(file) {
  return JSON.parse(await fs.readFile(file, "utf-8"));
}

async function main() {
  const root = path.resolve(".");
  const cardsPath = path.join(root, "solutions/seamgrim_ui_mvp/lessons/lesson_cards.v0.detjson");
  const crosswalkPath = path.join(root, "solutions/seamgrim_ui_mvp/lessons/curriculum_crosswalk.v0.detjson");
  const cardsBundle = await readJson(cardsPath);
  const crosswalk = await readJson(crosswalkPath);
  const module = await import(pathToFileURL(path.join(root, "solutions/seamgrim_ui_mvp/ui/curriculum_cards.js")).href);
  const validation = module.validateCurriculumCardBundle({ cardsBundle, crosswalk });
  assert(validation.ok === true, `valid card bundle rejected: ${JSON.stringify(validation.diagnostics)}`);
  assert(validation.card_count === 12, `card count mismatch: ${validation.card_count}`);
  const invalidBundle = JSON.parse(JSON.stringify(cardsBundle));
  invalidBundle.cards[0].curriculum_refs = ["missing.curriculum.ref"];
  const invalidValidation = module.validateCurriculumCardBundle({ cardsBundle: invalidBundle, crosswalk });
  assert(invalidValidation.ok === false, "unknown ref should be rejected");
  assert(
    invalidValidation.diagnostics.some((diag) => diag.code === "E_CURRICULUM_CARD_UNKNOWN_REF" && String(diag.message).includes("알 수 없는")),
    `unknown ref diagnostic must be Korean and stable: ${JSON.stringify(invalidValidation.diagnostics)}`,
  );

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
    await page.waitForSelector(".lesson-card[data-lesson-id='rep_math_function_line_v1']", { state: "visible", timeout: 20_000 });
    await page.click(".lesson-card[data-lesson-id='rep_math_function_line_v1']");
    await page.waitForSelector("[data-curriculum-card][data-curriculum-card-lesson-id='rep_math_function_line_v1']", { state: "visible", timeout: 10_000 });
    const detailState = await page.evaluate(() => {
      const section = document.querySelector("[data-curriculum-card]");
      const validationState = window.__SEAMGRIM_CURRICULUM_CARD_VALIDATION__ ?? null;
      return {
        validationState,
        sectionText: section?.textContent?.trim() ?? "",
        keywords: Array.from(document.querySelectorAll("#detail-keywords .detail-keyword")).map((node) => node.textContent?.trim() ?? ""),
        internalLeak: ["status_axes", "candidate", "evidence", "behavior_refs", "audit"].filter((token) => section?.textContent?.includes(token)),
      };
    });
    assert(detailState.validationState?.ok === true, `browser validation failed: ${JSON.stringify(detailState.validationState)}`);
    assert(detailState.validationState?.card_count === 12, `browser card count mismatch: ${detailState.validationState?.card_count}`);
    assert(detailState.sectionText.includes("학년군: 중학교"), `grade band missing: ${detailState.sectionText}`);
    assert(detailState.sectionText.includes("핵심 개념:"), `core concept missing: ${detailState.sectionText}`);
    assert(detailState.sectionText.includes("탐구 행동:"), `inquiry action missing: ${detailState.sectionText}`);
    assert(detailState.sectionText.includes("예상:"), `prediction prompt missing: ${detailState.sectionText}`);
    assert(detailState.sectionText.includes("실험:"), `inquiry prompt missing: ${detailState.sectionText}`);
    assert(detailState.keywords.some((text) => text.includes("기울기")), `curriculum concepts not merged into keywords: ${detailState.keywords.join("|")}`);
    assert(detailState.internalLeak.length === 0, `student detail leaked internal fields: ${detailState.internalLeak.join(",")}`);
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
