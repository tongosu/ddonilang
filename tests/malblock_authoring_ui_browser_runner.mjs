#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const OK = "malblock_authoring_ui_browser: ok";
const UNSAFE_BROWSER_PORTS = new Set([
  1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95,
  101, 102, 103, 104, 109, 110, 111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161,
  179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532, 540, 548, 554, 556, 563,
  587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 5060, 5061,
  6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697, 10080,
]);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function requireFile(file) {
  const stat = await fs.stat(file).catch(() => null);
  if (!stat || !stat.isFile()) {
    throw new Error(`missing file: ${file}`);
  }
}

function mimeType(file) {
  if (file.endsWith(".html")) return "text/html; charset=utf-8";
  if (file.endsWith(".js")) return "application/javascript; charset=utf-8";
  if (file.endsWith(".css")) return "text/css; charset=utf-8";
  if (file.endsWith(".json") || file.endsWith(".detjson")) return "application/json; charset=utf-8";
  if (file.endsWith(".wasm")) return "application/wasm";
  if (file.endsWith(".ddn")) return "text/plain; charset=utf-8";
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
      const decodedPath = decodeURIComponent(url.pathname === "/" ? "/solutions/seamgrim_ui_mvp/ui/index.html" : url.pathname);
      const rawPath = decodedPath.startsWith("/lessons/")
        ? `/solutions/seamgrim_ui_mvp${decodedPath}`
        : decodedPath;
      const rel = rawPath.replace(/^\/+/, "");
      const file = path.resolve(resolvedRoot, rel);
      if (file !== resolvedRoot && !file.startsWith(resolvedRoot + path.sep)) {
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
      if (UNSAFE_BROWSER_PORTS.has(address.port)) {
        server.close(() => {
          createServer(root).then(resolve, reject);
        });
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
    const pathname = url.pathname;
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") {
      return true;
    }
  } catch (_) {
    return false;
  }
  return false;
}

async function main() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(scriptDir, "..");
  const uiRoot = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");

  for (const rel of [
    "index.html",
    "screens/block_editor.js",
    "block_editor/ddn_block_engine.js",
    "block_editor/ddn_block_codec.js",
    "block_editor/seamgrim_palette.js",
    "block_editor/rpgbox_palette.js",
  ]) {
    await requireFile(path.join(uiRoot, rel));
  }

  const { server, baseUrl } = await createServer(root);
  let browser = null;
  const failures = [];
  try {
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({
      viewport: { width: 1280, height: 860 },
      locale: "ko-KR",
    });
    const page = await context.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        if (String(msg.text() ?? "").includes("Failed to load resource")) return;
        failures.push(`console error: ${msg.text()}`);
      }
    });
    page.on("pageerror", (err) => failures.push(`pageerror: ${err.message}`));
    page.on("requestfailed", (req) => failures.push(`request failed: ${req.url()} ${req.failure()?.errorText || ""}`));
    page.on("response", (res) => {
      if (res.status() >= 400 && !(res.status() === 404 && isAllowedFallback404(res.url()))) {
        failures.push(`response ${res.status()}: ${res.url()}`);
      }
    });

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
    const result = await page.evaluate(async () => {
      const { BlockEditorScreen } = await import("./screens/block_editor.js");
      const host = document.createElement("section");
      host.id = "malblock-authoring-smoke-host";
      host.innerHTML = `
        <span id="block-editor-title"></span>
        <select id="block-editor-mode">
          <option value="seamgrim">seamgrim</option>
          <option value="rpg">rpg</option>
        </select>
        <div id="block-editor-result"></div>
        <div id="block-editor-summary"></div>
        <textarea id="block-ddn-preview"></textarea>
        <div id="block-palette"></div>
        <div id="block-canvas"></div>
        <button id="btn-back-from-block-editor" type="button"></button>
        <button id="btn-block-to-text" type="button"></button>
        <button id="btn-block-run" type="button"></button>
        <button id="btn-advanced-block-editor" type="button"></button>
      `;
      document.body.appendChild(host);

      const textModeCalls = [];
      const runCalls = [];
      const fakeCanon = {
        async canonBlockEditorPlan(source) {
          if (String(source ?? "").includes("BROKEN")) {
            throw new Error("malblock_authoring_decode_error");
          }
          return {
            schema: "ddn.block_editor_plan.v1",
            blocks: [
              {
                kind: "charim_item_var",
                fields: { name: "t", type_name: "수", value: "0" },
                exprs: {},
                inputs: {},
              },
              {
                kind: "show",
                fields: { expr: "t" },
                exprs: {},
                inputs: {},
              },
            ],
          };
        },
      };

      const screen = new BlockEditorScreen({
        root: host,
        canon: fakeCanon,
        onTextMode(ddnText, meta) {
          textModeCalls.push({ ddnText: String(ddnText ?? ""), mode: String(meta?.mode ?? "") });
        },
        onRun(ddnText, meta) {
          runCalls.push({ ddnText: String(ddnText ?? ""), mode: String(meta?.mode ?? "") });
        },
      });
      screen.init();
      await screen.loadSource("채비 { t:수 <- 0. }.\nt 보여주기.", {
        title: "Malblock Authoring Smoke",
        mode: "seamgrim",
      });
      const validTitle = host.querySelector("#block-editor-title")?.textContent ?? "";

      const paletteButtons = Array.from(host.querySelectorAll("#block-palette .block-category__item"));
      const showButton = paletteButtons.find((button) => button.dataset.kind === "show");
      if (!showButton) {
        throw new Error("show palette button missing");
      }
      showButton.click();
      const insertedKinds = screen.engine.lastCanvasSummary.block_kinds.slice();
      const insertedDdn = screen.getDdn();

      const firstId = screen.currentBlocks[0]?.id;
      const lastId = screen.currentBlocks[screen.currentBlocks.length - 1]?.id;
      const moveOk = screen.engine.moveBlock(lastId, 0);
      const movedKinds = screen.engine.lastCanvasSummary.block_kinds.slice();
      const movedDdn = screen.getDdn();

      const removeOk = screen.engine.removeBlock(firstId);
      const removedKinds = screen.engine.lastCanvasSummary.block_kinds.slice();
      const removedDdn = screen.getDdn();

      screen.root.querySelector("#btn-block-to-text").click();
      screen.root.querySelector("#btn-block-run").click();

      await screen.loadSource("BROKEN", {
        title: "Broken Malblock Smoke",
        mode: "seamgrim",
      });
      const errorState = {
        resultText: host.querySelector("#block-editor-result")?.textContent ?? "",
        summaryText: host.querySelector("#block-editor-summary")?.textContent ?? "",
        blockKinds: screen.engine.lastCanvasSummary.block_kinds.slice(),
        rawCount: screen.engine.lastCanvasSummary.block_kinds.filter((kind) => kind === "raw").length,
      };

      return {
        title: host.querySelector("#block-editor-title")?.textContent ?? "",
        validTitle,
        mode: host.querySelector("#block-editor-mode")?.value ?? "",
        paletteSummary: screen.engine.lastPaletteSummary,
        initialCategoryLabels: Array.from(host.querySelectorAll("#block-palette .block-category__label"))
          .map((node) => node.textContent),
        initialPaletteButtonCount: paletteButtons.length,
        insertedKinds,
        insertedDdn,
        moveOk,
        movedKinds,
        movedDdn,
        removeOk,
        removedKinds,
        removedDdn,
        textModeCalls,
        runCalls,
        errorState,
      };
    });

    assert(result.validTitle === "Malblock Authoring Smoke", "block editor title mismatch");
    assert(result.mode === "seamgrim", "block editor mode mismatch");
    assert(result.paletteSummary.category_count >= 4, "palette category count too small");
    assert(result.paletteSummary.category_ids.includes("charim"), "palette charim category missing");
    assert(result.paletteSummary.category_ids.includes("logic"), "palette logic category missing");
    assert(result.initialCategoryLabels.includes("채비"), "palette label 채비 missing");
    assert(result.initialPaletteButtonCount >= 20, "palette button count too small");
    assert(result.insertedKinds.includes("show"), "palette click did not insert show block");
    assert(result.insertedDdn.includes("보여주기"), "inserted DDN should include show statement");
    assert(result.moveOk === true, "moveBlock should return true");
    assert(result.movedKinds[0] === "show", "moveBlock should move last show block to front");
    assert(result.movedDdn.trimStart().includes("보여주기"), "moved DDN should retain show statement");
    assert(result.removeOk === true, "removeBlock should return true");
    assert(result.removedKinds.length === result.movedKinds.length - 1, "removeBlock should remove one top-level block");
    assert(!result.removedDdn.startsWith("채비"), "removed DDN should no longer start with original first block");
    assert(result.textModeCalls.length === 1, "text mode callback count mismatch");
    assert(result.runCalls.length === 1, "run callback count mismatch");
    assert(result.textModeCalls[0].ddnText === result.removedDdn, "text mode callback DDN mismatch");
    assert(result.runCalls[0].ddnText === result.removedDdn, "run callback DDN mismatch");
    assert(result.errorState.summaryText.includes("errors=1"), "decode error summary missing");
    assert(result.errorState.resultText.includes("malblock_authoring_decode_error"), "decode error text missing");
    assert(result.errorState.rawCount === 1, "decode error should produce raw fallback block");

    if (failures.length > 0) {
      throw new Error(failures.join("\n"));
    }
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
