#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";
const OUT = path.resolve(ROOT, "pack/seamgrim_wave2e_durable_project_failures_v1/baseline.detjson");

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

function createStaticServer(root) {
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
        res.writeHead(403, { "cache-control": "no-store" });
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
      res.writeHead(404, { "content-type": "text/plain; charset=utf-8", "cache-control": "no-store" });
      res.end("not found");
    }
  });
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") reject(new Error("static server bind failed"));
      else resolve({ server, baseUrl: `http://127.0.0.1:${address.port}` });
    });
  });
}

function closeServer(server) {
  return new Promise((resolve) => server.close(resolve));
}

async function git(args) {
  const { spawn } = await import("node:child_process");
  return new Promise((resolve, reject) => {
    const child = spawn("git", args, { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => { stdout += chunk.toString("utf8"); });
    child.stderr.on("data", (chunk) => { stderr += chunk.toString("utf8"); });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) reject(new Error(`git ${args.join(" ")} failed: ${stderr || stdout}`));
      else resolve(stdout.trim());
    });
  });
}

async function main() {
  const head = await git(["rev-parse", "--short", "HEAD"]);
  const docsSsotStatus = await git(["status", "--short", "--", "docs/ssot"]);
  const { server, baseUrl } = await createStaticServer(ROOT);
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
    await page.goto(`${baseUrl}${PROJECT_URL}?first_delight=pendulum&devSurfaces=1`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 25_000 });
    await page.waitForSelector("#btn-save-server", { state: "attached", timeout: 10_000 });
    const browserCapture = await page.evaluate(() => {
      const readButton = (selector) => {
        const node = document.querySelector(selector);
        if (!(node instanceof HTMLButtonElement)) return null;
        return {
          selector,
          text: String(node.textContent ?? "").trim(),
          disabled: node.disabled,
          title: String(node.getAttribute("title") ?? ""),
          aria_label: String(node.getAttribute("aria-label") ?? ""),
        };
      };
      return {
        url_query: location.search,
        durable_project_buttons: [
          readButton("#btn-save-server"),
          readButton("#btn-share-link"),
          readButton("#btn-revision-history"),
          readButton("#btn-share-clone"),
        ],
        has_runtime_role_switch: Boolean(
          document.querySelector("[data-runtime-role-switch], #btn-run-role-student, #btn-run-role-teacher"),
        ),
      };
    });
    const buttons = browserCapture.durable_project_buttons.filter(Boolean);
    assert(buttons.length === 4, "expected four durable project buttons");
    for (const button of buttons) {
      assert(button.disabled === true, `expected disabled baseline button: ${button.selector}`);
      assert(button.text.includes("준비 중"), `expected not-ready label: ${button.selector}`);
    }
    assert(browserCapture.has_runtime_role_switch === false, "runtime role switch must remain absent");
    const payload = {
      schema: "seamgrim.wave2e.durable_project_failures.baseline.v1",
      pack: "seamgrim_wave2e_durable_project_failures_v1",
      captured_from_head: head,
      docs_ssot_dirty_at_capture: docsSsotStatus.length > 0,
      baseline_checks: {
        wave2d_first_view_authoring: "PASS",
        platform_server_adapter_contract: "FAIL_existing_appState_studio_harness",
        platform_server_action_rail: "PASS",
        studio_local_package_export_action_v1: "PASS",
      },
      browser_capture: browserCapture,
      failure_vectors: {
        server_save_share_not_ready: true,
        capability_leak_guard_missing: true,
        stale_editor_save_guard_missing: true,
        conflict_overwrite_guard_missing: true,
        view_escalation_guard_missing: true,
        restart_persistence_missing: true,
        backup_restore_missing: true,
      },
      boundaries: {
        docs_ssot_change: false,
        new_ddn_syntax: false,
        state_hash_formula_change: false,
        login_cloud_claim: false,
      },
    };
    await fs.writeFile(OUT, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
    console.log(`seamgrim wave2e durable project failure capture ok: ${path.relative(ROOT, OUT)}`);
  } finally {
    await browser.close();
    await closeServer(server);
  }
}

await main();
