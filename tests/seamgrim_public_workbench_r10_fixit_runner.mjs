#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const SERVER = path.join(ROOT, "solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py");
const LESSON = path.join(ROOT, "solutions/seamgrim_ui_mvp/lessons/rep_phys_projectile_xy_v1/lesson.ddn");
const WASM_BINDING = path.join(ROOT, "solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js");
const AUTHORITY_LF_SHA256 = "ae4013f36d77e2536dfa587d9743df3dee342c3cf3a425921edfbbdc4ac71b31";
const BAD_LF_SHA256 = "823d0687306ad1c13782dd5feb7f8a8f1d57291f76f29d1e9d870fbd891e42d2";
const WASM_BINDING_SHA256 = "177e9d7f7cfb9de48741f2b0aa20463330643baa49403a91f28420fd80016946";
const LEGACY_PATHS = ["/ui/app.js", "/ui/screens/editor.js", "/ui/screens/run.js"];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function sha256(text) {
  return crypto.createHash("sha256").update(text, "utf8").digest("hex");
}

function lfProjection(text) {
  return String(text).replace(/^\uFEFF/u, "").replace(/\r\n?/gu, "\n");
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = address && typeof address !== "string" ? address.port : 0;
      server.close(() => resolve(port));
    });
  });
}

async function startServer() {
  const dataDir = await fs.mkdtemp(path.join(os.tmpdir(), "seamgrim-r10-fixit-"));
  const port = await freePort();
  let stderr = "";
  const child = spawn(process.env.PYTHON || "python", [SERVER, "--host", "127.0.0.1", "--port", String(port)], {
    cwd: ROOT,
    env: { ...process.env, SEAMGRIM_PROJECT_DATA_DIR: dataDir },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", () => {});
  child.stderr.on("data", (chunk) => { stderr += String(chunk); });
  const baseUrl = `http://127.0.0.1:${port}`;
  for (let attempt = 0; attempt < 120; attempt += 1) {
    if (child.exitCode !== null) throw new Error(`server exited ${child.exitCode}: ${stderr}`);
    try {
      const response = await fetch(`${baseUrl}/api/health`);
      if (response.ok) return { child, baseUrl, dataDir };
    } catch (_) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error(`server health timeout: ${stderr}`);
}

async function stopServer(child) {
  if (!child || child.exitCode !== null) return;
  child.kill();
  await new Promise((resolve) => child.once("exit", resolve));
}

function capabilityFromLink(link) {
  const url = new URL(link);
  const params = new URLSearchParams(url.hash.replace(/^#/u, ""));
  return {
    projectId: String(params.get("sg_project") ?? ""),
    role: String(params.get("sg_role") ?? ""),
    capability: String(params.get("sg_cap") ?? ""),
  };
}

async function readServerProject(baseUrl, session) {
  const response = await fetch(`${baseUrl}/api/v1/projects/${encodeURIComponent(session.projectId)}`, {
    headers: { Authorization: `Bearer ${session.capability}` },
  });
  assert(response.ok, `server project read failed: ${response.status}`);
  return response.json();
}

function objectCarriesExactSource(value, source) {
  if (typeof value === "string") return lfProjection(value) === source;
  if (Array.isArray(value)) return value.some((item) => objectCarriesExactSource(item, source));
  if (value && typeof value === "object") return Object.values(value).some((item) => objectCarriesExactSource(item, source));
  return false;
}

function attachEvidenceListeners(page, label, evidence) {
  page.on("console", (message) => {
    if (message.type() === "error") evidence.consoleErrors.push(`${label}: ${message.text()}`);
  });
  page.on("pageerror", (error) => evidence.pageErrors.push(`${label}: ${error.message}`));
  page.on("request", (request) => {
    const pathname = new URL(request.url()).pathname;
    if (LEGACY_PATHS.some((legacyPath) => pathname.endsWith(legacyPath))) {
      evidence.legacyAccesses.push(pathname);
    }
  });
}

async function installManifestSealedWasmBinding(context) {
  const raw = await fs.readFile(WASM_BINDING, "utf8");
  const sealedLfBytes = lfProjection(raw);
  assert(sha256(sealedLfBytes) === WASM_BINDING_SHA256, "manifest-sealed WASM binding LF hash mismatch");
  await context.route("**/solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.js*", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "text/javascript; charset=utf-8",
      body: sealedLfBytes,
    });
  });
  return sha256(sealedLfBytes);
}

async function waitForWorkbench(page) {
  await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()), null, { timeout: 30_000 });
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id='rep_phys_projectile_xy_v1']", { timeout: 30_000 });
}

async function main() {
  const authorityRaw = await fs.readFile(LESSON, "utf8");
  const authority = lfProjection(authorityRaw);
  const badSource = authority.replace("채비 {", "채비: {");
  assert(authority !== badSource, "fixture did not contain the authority setup header");
  assert(sha256(authority) === AUTHORITY_LF_SHA256, `authority LF hash mismatch: ${sha256(authority)}`);
  assert(sha256(badSource) === BAD_LF_SHA256, `bad LF hash mismatch: ${sha256(badSource)}`);
  assert(Buffer.byteLength(badSource, "utf8") === 853, "bad fixture byte length mismatch");

  const server = await startServer();
  const browser = await chromium.launch({ headless: true });
  const evidence = { consoleErrors: [], pageErrors: [], legacyAccesses: [] };
  try {
    const mainContext = await browser.newContext({ viewport: { width: 1440, height: 900 }, locale: "ko-KR" });
    const servedWasmBindingHash = await installManifestSealedWasmBinding(mainContext);
    const page = await mainContext.newPage();
    attachEvidenceListeners(page, "main", evidence);
    await page.goto(`${server.baseUrl}/`, { waitUntil: "networkidle" });
    await waitForWorkbench(page);
    assert(new URL(page.url()).pathname === "/solutions/seamgrim_ui_mvp/ui/workbench/index.html", `default route mismatch: ${page.url()}`);

    await page.fill("[data-testid='source-editor']", badSource);
    const beforeWarningRun = await page.evaluate(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return {
        runGeneration: snapshot.sourceReadiness.run_generation,
        runCount: snapshot.runs.length,
        vmEntries: snapshot.sourceReadiness.wasm_vm_entry_count,
        activeRunId: snapshot.activeRunId,
        latestStateHash: snapshot.runs.at(-1)?.latest_state_hash ?? "",
        applyOutcome: snapshot.sourceReadiness.last_apply_outcome.status,
        runOutcome: snapshot.sourceReadiness.last_run_outcome.status,
        localDiagnosticCode: snapshot.sourceReadiness.code,
      };
    });
    assert(beforeWarningRun.localDiagnosticCode === "", "JS created a 채비 language verdict before authoritative analysis");
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction((before) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return snapshot.runtimeStatus === "running"
        && snapshot.runs.length > before
        && snapshot.sourceReadiness?.code === "W_BLOCK_HEADER_COLON_DEPRECATED";
    }, beforeWarningRun.runCount, { timeout: 30_000 });
    await page.evaluate(() => document.querySelector("[data-testid='project-verification-open']")?.click());
    await page.waitForFunction(() => document.querySelector("[data-testid='project-verification-drawer']")?.open === true);
    const diagnosticEvidence = await page.evaluate(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      const button = document.querySelector("[data-testid='source-readiness-fixit']");
      const list = document.querySelector("[data-testid='diagnostic-list']");
      return {
        readiness: snapshot.sourceReadiness,
        buttonTag: button?.tagName ?? "",
        buttonText: button?.textContent?.trim() ?? "",
        buttonDisabled: Boolean(button?.disabled),
        ariaLive: list?.getAttribute("aria-live") ?? "",
        uiText: list?.textContent ?? "",
      };
    });
    const diagnostic = diagnosticEvidence.readiness;
    assert(diagnostic.schema === "seamgrim.workbench.source_readiness.v1", "readiness schema missing");
    assert(diagnostic.policy_layer === "workbench_source_readiness_projection", "readiness projection layer missing");
    assert(diagnostic.authority_status === "shared_frontend", "warning authority missing");
    assert(diagnostic.code === "W_BLOCK_HEADER_COLON_DEPRECATED", "authoritative warning code missing");
    assert(diagnostic.ready && !diagnostic.blocked, "authoritative warning blocked execution");
    assert(diagnostic.analysis_outcome === "ACCEPTED", "authoritative accepted outcome missing");
    assert(diagnostic.fix_it_id === "FIX_SOURCE_REMOVE_SETUP_COLON", "fix-it id missing");
    assert(diagnostic.span?.encoding === "utf-8-byte", "authoritative UTF-8 span encoding missing");
    assert(Number.isInteger(diagnostic.span?.start) && diagnostic.span?.end > diagnostic.span?.start, "authoritative warning span missing");
    assert(diagnosticEvidence.buttonTag === "BUTTON" && !diagnosticEvidence.buttonDisabled, "enabled native fix button missing");
    assert(diagnosticEvidence.buttonText.includes("콜론"), "fix button is not text-labelled");
    assert(diagnosticEvidence.ariaLive === "polite", "diagnostic list aria-live contract missing");
    assert(diagnosticEvidence.uiText.includes("W_BLOCK_HEADER_COLON_DEPRECATED"), "warning is not presented to the user");

    const afterWarningRun = await page.evaluate(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return {
        runtimeStatus: snapshot.runtimeStatus,
        runGeneration: snapshot.sourceReadiness.run_generation,
        runCount: snapshot.runs.length,
        vmEntries: snapshot.sourceReadiness.wasm_vm_entry_count,
        activeRunId: snapshot.activeRunId,
        latestStateHash: snapshot.runs.at(-1)?.latest_state_hash ?? "",
        applyOutcome: snapshot.sourceReadiness.last_apply_outcome.status,
        runOutcome: snapshot.sourceReadiness.last_run_outcome.status,
        runBlocked: snapshot.sourceReadiness.run_blocked,
        candidate: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSourceReadinessCandidate(),
      };
    });
    assert(afterWarningRun.runtimeStatus === "running", "warning source did not run");
    assert(afterWarningRun.runCount > beforeWarningRun.runCount, "warning source did not create a run record");
    assert(afterWarningRun.vmEntries > beforeWarningRun.vmEntries, "warning source did not enter runtime VM");
    assert(!afterWarningRun.runBlocked && afterWarningRun.runOutcome === "run_started", "warning source was reported as blocked");
    assert(beforeWarningRun.applyOutcome === "none" && beforeWarningRun.runOutcome === "none", "initial outcomes are not cleanly separated");
    assert(afterWarningRun.applyOutcome === beforeWarningRun.applyOutcome, "warning run overwrote apply outcome");
    assert(afterWarningRun.candidate?.trigger_authority === "shared_frontend_diagnostic", "fix trigger is not authoritative");
    assert(afterWarningRun.candidate?.replacement_authority === "rust_shared_frontend_repair_proposal", "Rust repair replacement authority is missing");
    assert(afterWarningRun.candidate?.replacement_offset_source === "authoritative_original_source_span", "authoritative original-source span is missing");
    assert(afterWarningRun.candidate?.diagnostic_identity, "diagnostic identity is missing from the repair candidate");
    assert(afterWarningRun.candidate?.repair_proposal_identity, "proposal identity is missing from the repair candidate");

    await page.evaluate(() => document.querySelector("[data-testid='project-verification-close']")?.click());
    await page.click("[data-testid='lifecycle-stop']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus === "stopped");
    const staleSource = `${badSource}// stale mutation\n`;
    await page.fill("[data-testid='source-editor']", staleSource);
    const staleResult = await page.evaluate((candidate) => ({
      outcome: window.__SEAMGRIM_APPROVED_WORKBENCH__.applySourceReadinessFixIt(candidate),
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      readiness: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness,
    }), afterWarningRun.candidate);
    assert(staleResult.outcome.status === "stale_rejected", `stale candidate was not rejected: ${JSON.stringify(staleResult.outcome)}`);
    assert(staleResult.source === staleSource, "stale rejection mutated source");
    assert(staleResult.readiness.last_apply_outcome.status === "stale_rejected", "stale result missing from apply outcome");
    assert(staleResult.readiness.last_run_outcome.status === "run_started", "stale apply overwrote run outcome");

    await page.fill("[data-testid='source-editor']", badSource);
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return snapshot.runtimeStatus === "running"
        && snapshot.sourceReadiness?.code === "W_BLOCK_HEADER_COLON_DEPRECATED";
    }, null, { timeout: 30_000 });
    await page.click("[data-testid='lifecycle-stop']");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus === "stopped");
    await page.evaluate(() => document.querySelector("[data-testid='project-verification-open']")?.click());
    await page.waitForFunction(() => document.querySelector("[data-testid='project-verification-drawer']")?.open === true);
    await page.locator("[data-testid='source-readiness-fixit']").focus();
    assert(await page.evaluate(() => document.activeElement?.dataset?.testid === "source-readiness-fixit"), "fix button keyboard focus missing");
    await page.keyboard.press("Enter");
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness?.last_apply_outcome?.status === "applied");
    const applied = await page.evaluate(() => ({
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      readiness: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness,
      focusRestored: document.activeElement?.dataset?.testid === "source-editor",
      liveText: document.querySelector("[data-testid='diagnostic-list']")?.textContent ?? "",
    }));
    assert(applied.source === authority, "fix did not restore the exact authority LF projection");
    assert(sha256(applied.source) === AUTHORITY_LF_SHA256, "applied source hash mismatch");
    assert(applied.readiness.ready === true && applied.readiness.last_apply_outcome.changed_count === 1, "applied readiness outcome mismatch");
    assert(applied.readiness.last_apply_outcome.status === "applied", "applied result missing from apply outcome");
    assert(applied.readiness.last_run_outcome.status === "run_started", "applied fix overwrote run outcome");
    assert(applied.focusRestored, "focus was not restored to source editor");
    assert(applied.liveText.includes("고침을 적용했습니다"), "aria-live completion message missing");

    const secondApply = await page.evaluate(() => {
      const before = document.querySelector("[data-testid='source-editor']")?.value ?? "";
      const outcome = window.__SEAMGRIM_APPROVED_WORKBENCH__.applySourceReadinessFixIt();
      const after = document.querySelector("[data-testid='source-editor']")?.value ?? "";
      return {
        outcome,
        unchanged: before === after,
        readiness: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness,
      };
    });
    assert(secondApply.outcome.status === "no_op" && secondApply.unchanged, "second fix application was not idempotent");
    assert(secondApply.readiness.last_apply_outcome.status === "no_op", "no-op missing from apply outcome");
    assert(secondApply.readiness.last_run_outcome.status === "run_started", "no-op overwrote run outcome");

    const runsBeforeActual = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.length);
    await page.click("[data-testid='lifecycle-run']");
    try {
      await page.waitForFunction((before) => {
        const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
        return snapshot.runtimeStatus === "running" && snapshot.runs.length > before && Boolean(snapshot.runs.at(-1)?.latest_state_hash);
      }, runsBeforeActual, { timeout: 30_000 });
    } catch (error) {
      const debug = await page.evaluate(() => ({
        snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
        source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      }));
      throw new Error(`actual WASM run timeout: ${JSON.stringify({ runtimeStatus: debug.snapshot.runtimeStatus, runs: debug.snapshot.runs, diagnostics: debug.snapshot.diagnostics, readiness: debug.snapshot.sourceReadiness, source: debug.source })}`, { cause: error });
    }
    const actualRun = await page.evaluate(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return {
        runtimeStatus: snapshot.runtimeStatus,
        stateHashPresent: Boolean(snapshot.runs.at(-1)?.latest_state_hash),
        vmEntries: snapshot.sourceReadiness.wasm_vm_entry_count,
        runCount: snapshot.runs.length,
        applyOutcome: snapshot.sourceReadiness.last_apply_outcome.status,
        runOutcome: snapshot.sourceReadiness.last_run_outcome.status,
      };
    });
    assert(actualRun.vmEntries > afterWarningRun.vmEntries, "fixed source did not enter the actual runtime VM");
    assert(actualRun.applyOutcome === "no_op", "successful run overwrote apply outcome");
    assert(actualRun.runOutcome === "run_started", "successful run outcome missing");

    const negativeSource = authority
      .replace("설명: \"", "설명: \"문자열 채비: { / ")
      .replace("채비 {", "// 줄주석 채비: {\n채비 {");
    await page.fill("[data-testid='source-editor']", negativeSource);
    await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness?.ready === true);
    const falsePositive = await page.evaluate(() => {
      const api = window.__SEAMGRIM_APPROVED_WORKBENCH__;
      return {
        sourceReady: api.getSnapshot().sourceReadiness.ready,
        stringReady: api.analyzeSourceReadiness('설정 { 설명: "채비: {". }.').ready,
        commentReady: api.analyzeSourceReadiness('// 채비: {\n설정 { 제목: "ok". }.').ready,
        templateReady: api.analyzeSourceReadiness('글무늬 {\n  채비: {\n}\n설정 { 제목: "ok". }.').ready,
        typeColonReady: api.analyzeSourceReadiness('설정 { 제목: "ok". }.\n값: 수 <- 1.').ready,
      };
    });
    assert(Object.values(falsePositive).every(Boolean), `false-positive readiness failure: ${JSON.stringify(falsePositive)}`);
    const negativeRunsBefore = actualRun.runCount;
    await page.click("[data-testid='lifecycle-run']");
    await page.waitForFunction((before) => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return snapshot.runtimeStatus === "running" && snapshot.runs.length > before;
    }, negativeRunsBefore, { timeout: 30_000 });
    const negativeRun = await page.evaluate(() => ({
      runtimeStatus: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runtimeStatus,
      runBlocked: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.run_blocked,
    }));
    assert(!negativeRun.runBlocked, "negative source was blocked");

    const creatorContext = await browser.newContext({ viewport: { width: 1280, height: 800 }, locale: "ko-KR" });
    await installManifestSealedWasmBinding(creatorContext);
    const creator = await creatorContext.newPage();
    attachEvidenceListeners(creator, "creator", evidence);
    await creator.goto(`${server.baseUrl}/`, { waitUntil: "networkidle" });
    await waitForWorkbench(creator);
    await creator.fill("[data-testid='source-editor']", badSource);
    await creator.click("[data-testid='project-save']");
    await creator.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject?.head_revision_id), null, { timeout: 30_000 });
    await creator.waitForFunction(() => String(document.querySelector("[data-testid='project-view-link']")?.href ?? "").includes("sg_cap="), null, { timeout: 30_000 });
    const saved = await creator.evaluate(() => ({
      viewHref: document.querySelector("[data-testid='project-view-link']")?.href ?? "",
      snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    }));
    const viewSession = capabilityFromLink(saved.viewHref);
    assert(viewSession.projectId && viewSession.role === "view" && viewSession.capability, "view capability link missing");
    const serverBefore = await readServerProject(server.baseUrl, viewSession);
    assert(objectCarriesExactSource(serverBefore, badSource), "server revision did not preserve bad source fixture");

    const viewContext = await browser.newContext({ viewport: { width: 1280, height: 800 }, locale: "ko-KR" });
    await installManifestSealedWasmBinding(viewContext);
    const viewer = await viewContext.newPage();
    attachEvidenceListeners(viewer, "viewer", evidence);
    await viewer.goto(saved.viewHref, { waitUntil: "domcontentloaded" });
    await viewer.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.()?.capabilitySession?.role === "view", null, { timeout: 30_000 });
    await viewer.click("[data-testid='lifecycle-run']");
    await viewer.waitForFunction(() => {
      const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.();
      return snapshot?.capabilitySession?.role === "view" && snapshot?.sourceReadiness?.code === "W_BLOCK_HEADER_COLON_DEPRECATED";
    }, null, { timeout: 30_000 });
    const viewBefore = await viewer.evaluate(() => ({
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      head: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.head_revision_id,
      revisionCount: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.revision_count,
      fixDisabled: Boolean(document.querySelector("[data-testid='source-readiness-fixit']")?.disabled),
      applyAvailable: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.apply_available,
      applyOutcome: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.last_apply_outcome.status,
      runOutcome: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.last_run_outcome.status,
    }));
    const viewOutcome = await viewer.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.applySourceReadinessFixIt());
    const viewAfter = await viewer.evaluate(() => ({
      source: document.querySelector("[data-testid='source-editor']")?.value ?? "",
      head: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.head_revision_id,
      revisionCount: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().serverProject.revision_count,
      applyOutcome: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.last_apply_outcome.status,
      runOutcome: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.last_run_outcome.status,
    }));
    const serverAfter = await readServerProject(server.baseUrl, viewSession);
    assert(viewBefore.fixDisabled && !viewBefore.applyAvailable, "view-only fix control was not disabled");
    assert(viewOutcome.status === "view_only_rejected", "view-only API application was not rejected");
    assert(viewBefore.applyOutcome === "none" && viewBefore.runOutcome === "none", "view-only initial outcomes are not clean");
    assert(viewAfter.applyOutcome === "view_only_rejected", "view-only rejection missing from apply outcome");
    assert(viewAfter.runOutcome === "none", "view-only apply overwrote run outcome");
    assert(viewBefore.source === badSource && viewAfter.source === viewBefore.source, "view-only rejection changed source");
    assert(viewAfter.head === viewBefore.head && viewAfter.revisionCount === viewBefore.revisionCount, "view-only rejection changed revision state");
    assert(JSON.stringify(serverAfter) === JSON.stringify(serverBefore), "view-only rejection changed server source or revision");

    await viewContext.close();
    await creatorContext.close();
    await mainContext.close();
    assert(evidence.consoleErrors.length === 0, `console errors: ${evidence.consoleErrors.join("\n")}`);
    assert(evidence.pageErrors.length === 0, `page errors: ${evidence.pageErrors.join("\n")}`);
    assert(evidence.legacyAccesses.length === 0, `legacy paths accessed: ${evidence.legacyAccesses.join(", ")}`);

    const report = {
      schema: "seamgrim.public_workbench_r10_fixit.runner_result.v1",
      fixture: {
        authority_lf_sha256: sha256(authority),
        authority_lf_bytes: Buffer.byteLength(authority, "utf8"),
        bad_lf_sha256: sha256(badSource),
        bad_lf_bytes: Buffer.byteLength(badSource, "utf8"),
      },
      diagnostic: {
        schema: diagnostic.schema,
        code: diagnostic.code,
        upstream_cause_code: diagnostic.upstream_cause_code,
        fix_it_id: diagnostic.fix_it_id,
        span: diagnostic.span,
        authority_status: diagnostic.authority_status,
        analysis_outcome: diagnostic.analysis_outcome,
        native_button: diagnosticEvidence.buttonTag === "BUTTON",
        aria_live: diagnosticEvidence.ariaLive,
      },
      warning_run: {
        status: afterWarningRun.runtimeStatus,
        run_generation_advanced: afterWarningRun.runGeneration > beforeWarningRun.runGeneration,
        run_record_created: afterWarningRun.runCount > beforeWarningRun.runCount,
        vm_entered: afterWarningRun.vmEntries > beforeWarningRun.vmEntries,
        run_blocked: afterWarningRun.runBlocked,
      },
      outcome_separation: {
        before_warning_run: { apply: beforeWarningRun.applyOutcome, run: beforeWarningRun.runOutcome },
        after_warning_run: { apply: afterWarningRun.applyOutcome, run: afterWarningRun.runOutcome },
        after_stale: {
          apply: staleResult.readiness.last_apply_outcome.status,
          run: staleResult.readiness.last_run_outcome.status,
        },
        after_apply: {
          apply: applied.readiness.last_apply_outcome.status,
          run: applied.readiness.last_run_outcome.status,
        },
        after_no_op: {
          apply: secondApply.readiness.last_apply_outcome.status,
          run: secondApply.readiness.last_run_outcome.status,
        },
        after_run: { apply: actualRun.applyOutcome, run: actualRun.runOutcome },
        view_only_after_apply: { apply: viewAfter.applyOutcome, run: viewAfter.runOutcome },
      },
      stale_fix: { status: staleResult.outcome.status, source_unchanged: staleResult.source === staleSource },
      applied_fix: {
        status: "applied",
        changed_count: applied.readiness.last_apply_outcome.changed_count,
        authority_hash_restored: sha256(applied.source) === AUTHORITY_LF_SHA256,
        focus_restored: applied.focusRestored,
        aria_live_completion: applied.liveText.includes("고침을 적용했습니다"),
      },
      idempotency: { status: secondApply.outcome.status, source_unchanged: secondApply.unchanged },
      actual_wasm_run: actualRun,
      wasm_binding_transport: {
        mode: "manifest_sealed_lf_projection",
        sha256: servedWasmBindingHash,
        product_frontdoor: "createWasmVm",
        frontdoor_bypassed: false,
      },
      false_positive: { ...falsePositive, run_not_blocked: !negativeRun.runBlocked },
      view_only: {
        button_disabled: viewBefore.fixDisabled,
        apply_available: viewBefore.applyAvailable,
        outcome: viewOutcome.status,
        source_unchanged: viewAfter.source === viewBefore.source,
        revision_unchanged: viewAfter.head === viewBefore.head && viewAfter.revisionCount === viewBefore.revisionCount,
        server_payload_unchanged: JSON.stringify(serverAfter) === JSON.stringify(serverBefore),
      },
      browser: { console_error_count: 0, page_error_count: 0, legacy_access_count: 0 },
    };
    console.log(JSON.stringify(report));
  } finally {
    await browser.close();
    await stopServer(server.child);
  }
}

await main();
