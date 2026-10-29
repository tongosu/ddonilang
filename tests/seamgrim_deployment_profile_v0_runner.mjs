#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_deployment_profile_v0: ok";

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
    const pathname = url.pathname.replace(/^\/solutions\/seamgrim_ui_mvp/u, "");
    if (pathname === "/api/lessons/inventory" || pathname === "/api/lesson-inventory") return true;
    if ((pathname.startsWith("/lessons/") || pathname.startsWith("/seed_lessons_v1/")) && /\/(?:graph|table|space2d|text|maegim_control)\.(?:json|md)$/i.test(pathname)) return true;
  } catch (_) {
    return false;
  }
  return false;
}

function installFailureCapture(page, failures) {
  page.on("console", (msg) => {
    const text = String(msg.text() ?? "");
    if (msg.type() === "error" && !text.includes("Failed to load resource")) {
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
}

async function openPublish(page, baseUrl) {
  await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-browse:not(.hidden) .lesson-card[data-lesson-id]", { state: "visible", timeout: 20_000 });
  await page.locator("#screen-browse .main-shell-tab[data-shell-space='publish']").click();
  await page.waitForSelector("#screen-publish:not(.hidden) [data-deployment-profile-editor]", { state: "visible", timeout: 10_000 });
}

async function editDeploymentProfile(page) {
  await page.locator("#deployment-profile-preset").selectOption("focused_practice");
  await page.locator("#deployment-profile-default-view").selectOption("graph");
  await page.locator("#deployment-profile-answer-policy").selectOption("hidden");
  await page.locator("#deployment-profile-explanation-policy").selectOption("after_run");
  await page.locator("#deployment-profile-control-name").fill("theta0");
  await page.locator("#deployment-profile-control-min").fill("-0.8");
  await page.locator("#deployment-profile-control-max").fill("0.8");
  await page.locator("#deployment-profile-control-step").fill("0.1");
  await page.locator("[data-deployment-profile-view='table']").uncheck();
  await page.locator("[data-deployment-profile-view='text']").uncheck();
  await page.waitForFunction(() => {
    const hash = document.querySelector("#screen-publish")?.dataset?.deploymentProfileHash ?? "";
    return hash.startsWith("candidate:fnv1a32:deployment_profile:");
  });
}

async function copyPackage(page) {
  await page.locator("#screen-publish #btn-copy-course-package").click();
  await page.waitForFunction(() => window.__STUDIO_COURSE_PACKAGE_EXPORT_ACTION__?.copied === true, null, { timeout: 30_000 });
  const payloadText = await page.evaluate(() => window.__SEAMGRIM_TEST_CLIPBOARD__ ?? "");
  assert(payloadText.includes("studio_local_package_payload"), "clipboard package payload missing");
  const payload = JSON.parse(payloadText);
  const action = await page.evaluate(() => window.__STUDIO_COURSE_PACKAGE_EXPORT_ACTION__ ?? null);
  return { payload, action };
}

async function pastePackage(page, payloadText) {
  await page.locator("#screen-publish #btn-paste-local-package").click();
  await page.locator("#local-package-paste-text").fill(payloadText);
  await page.locator("#btn-local-package-paste-open").click();
}

async function main() {
  const root = path.resolve(".");
  const { server, baseUrl } = await createServer(root);
  const browser = await chromium.launch({ headless: true });
  const failures = [];
  try {
    const context = await browser.newContext({ viewport: { width: 1360, height: 860 }, locale: "ko-KR" });
    await context.addInitScript(() => {
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          async writeText(value) {
            window.__SEAMGRIM_TEST_CLIPBOARD__ = String(value ?? "");
          },
          async readText() {
            return window.__SEAMGRIM_TEST_CLIPBOARD__ ?? "";
          },
        },
      });
    });
    const page = await context.newPage();
    installFailureCapture(page, failures);

    await openPublish(page, baseUrl);
    await editDeploymentProfile(page);
    const profileUiHash = await page.locator("#screen-publish output[data-deployment-profile-hash]").textContent();
    assert(String(profileUiHash).includes("candidate:fnv1a32:deployment_profile:"), `profile hash preview missing: ${profileUiHash}`);

    const { payload, action } = await copyPackage(page);
    const profile = payload.manifest.deployment_profile;
    assert(profile.schema === "seamgrim.deployment_profile.v0", `profile schema mismatch: ${profile.schema}`);
    assert(profile.preset_ref === "focused_practice", `preset mismatch: ${profile.preset_ref}`);
    assert(profile.default_view === "graph", `default view mismatch: ${profile.default_view}`);
    assert(profile.answer_display_policy === "hidden", `answer policy mismatch: ${profile.answer_display_policy}`);
    assert(profile.explanation_display_policy === "after_run", `explanation policy mismatch: ${profile.explanation_display_policy}`);
    assert(profile.allowed_views.join(",") === "space2d,graph", `allowed views mismatch: ${profile.allowed_views.join(",")}`);
    assert(profile.allowed_controls[0].name === "theta0", "control name mismatch");
    assert(profile.allowed_controls[0].min === -0.8 && profile.allowed_controls[0].max === 0.8 && profile.allowed_controls[0].step === 0.1, `control range mismatch: ${JSON.stringify(profile.allowed_controls[0])}`);
    assert(profile.readonly_in_run === true && profile.state_hash_participation === false && profile.signature_claim === false, "profile boundary mismatch");
    assert(payload.manifest.deployment_profile_hash === profile.profile_hash, "manifest profile hash mismatch");
    assert(payload.deployment_profile_hash === profile.profile_hash, "payload profile hash mismatch");
    assert(String(payload.manifest.package_hash ?? "").startsWith("candidate:fnv1a32:studio_local_package:"), "manifest package hash missing");
    assert(String(payload.package_hash ?? "") === String(payload.manifest.package_hash ?? ""), "payload package hash mismatch");
    assert(Array.isArray(payload.manifest.package_hash_inputs) && payload.manifest.package_hash_inputs.includes("deployment_profile"), "deployment profile missing from package hash inputs");
    assert(action.manifest.deployment_profile.profile_hash === profile.profile_hash, "copy action profile hash mismatch");

    await pastePackage(page, JSON.stringify(payload, null, 2));
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === true, null, { timeout: 30_000 });
    await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 10_000 });
    const runProfile = await page.evaluate(() => ({
      readonly: document.querySelector("#screen-run")?.dataset?.deploymentProfileReadonly ?? "",
      profileHash: document.querySelector("#screen-run")?.dataset?.deploymentProfileHash ?? "",
      packageHash: document.querySelector("#screen-run")?.dataset?.localPackageHash ?? "",
      profileText: document.querySelector("[data-run-deployment-profile-readonly]")?.textContent ?? "",
      editorCount: document.querySelectorAll("#screen-run [data-deployment-profile-field]").length,
      switchCount: document.querySelectorAll("#screen-run [data-classroom-mode-switch], #screen-run [data-classroom-mode]").length,
      importAction: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
    }));
    assert(runProfile.readonly === "1", `run profile readonly missing: ${JSON.stringify(runProfile)}`);
    assert(runProfile.profileHash === profile.profile_hash, `run profile hash mismatch: ${JSON.stringify(runProfile)}`);
    assert(runProfile.packageHash === payload.manifest.package_hash, `run package hash mismatch: ${JSON.stringify(runProfile)}`);
    assert(runProfile.profileText.includes("focused_practice") && runProfile.profileText.includes("theta0") && runProfile.profileText.includes(profile.profile_hash), `run profile text mismatch: ${runProfile.profileText}`);
    assert(runProfile.editorCount === 0, "run screen must not expose deployment profile editor fields");
    assert(runProfile.switchCount === 0, "runtime role switch must stay absent");
    assert(runProfile.importAction.deployment_profile_hash === profile.profile_hash, "import action profile hash mismatch");

    await page.goto(`${baseUrl}/solutions/seamgrim_ui_mvp/ui/index.html?tab=publish`, { waitUntil: "domcontentloaded" });
    await page.waitForSelector("#screen-publish:not(.hidden)", { state: "visible", timeout: 10_000 });
    const tampered = JSON.parse(JSON.stringify(payload));
    tampered.manifest.deployment_profile.preset_ref = "tampered_profile";
    tampered.deployment_profile = tampered.manifest.deployment_profile;
    await pastePackage(page, JSON.stringify(tampered, null, 2));
    await page.waitForFunction(() => window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__?.imported === false, null, { timeout: 10_000 });
    const tamper = await page.evaluate(() => ({
      action: window.__STUDIO_LOCAL_PACKAGE_IMPORT_ACTION__ ?? null,
      status: document.querySelector("[data-local-package-import-status]")?.textContent ?? "",
      state: document.querySelector("[data-local-package-import-status]")?.dataset?.state ?? "",
    }));
    assert(tamper.action.error === "studio_local_package_deployment_profile_hash_mismatch", `tamper error mismatch: ${JSON.stringify(tamper)}`);
    assert(tamper.state === "error" && tamper.status.includes("배포 프로필 해시가 맞지 않습니다"), `tamper Korean diagnostic missing: ${JSON.stringify(tamper)}`);

    assert(failures.length === 0, failures.join("\n"));
    await page.close();
    await context.close();
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
