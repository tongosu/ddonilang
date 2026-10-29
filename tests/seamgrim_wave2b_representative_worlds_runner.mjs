#!/usr/bin/env node

import fs from "node:fs/promises";
import http from "node:http";
import path from "node:path";
import { chromium } from "playwright";

const OK = "seamgrim_wave2b_representative_worlds: ok";
const PROJECT_URL = "/solutions/seamgrim_ui_mvp/ui/index.html";

const WORLDS = [
  {
    firstId: "pendulum",
    lessonId: "rep_phys_pendulum_wave1_v1",
    sourceToken: "theta <- theta + omega * dt",
    roles: ["angle_measure", "length_measure", "vector_arrow", "trajectory_trail"],
    requiredShapeRoles: ["length_measure_line", "vector_arrow", "trajectory_trail"],
    primaryView: "space2d",
    camera: "follow",
    observations: ["theta", "omega", "bob_x", "bob_y"],
    lessonPlan: {
      scene_ref: "scene.pendulum.measure",
      measurement_components: ["angle", "length", "vector", "trail"],
      reaction_rule_refs: ["edge.theta_extremum"],
    },
  },
  {
    firstId: "projectile",
    lessonId: "rep_phys_projectile_xy_v1",
    sourceToken: "x <- x + vx * dt",
    roles: ["trajectory_trail", "vector_arrow", "fit_camera"],
    requiredShapeRoles: ["trajectory_trail", "vector_arrow"],
    minTrajectorySegments: 1,
    primaryView: "space2d",
    camera: "fit",
    observations: ["x", "y", "vx", "vy"],
    lessonPlan: {
      scene_ref: "scene.projectile.flight",
      measurement_components: ["vector", "trail"],
      reaction_rule_refs: ["level.projectile.flight"],
    },
  },
  {
    firstId: "linear-function",
    lessonId: "rep_math_function_line_v1",
    sourceToken: "y <- 기울기 * x + 절편",
    roles: ["graph_series", "axis_ticks", "graph_point", "slope_measure"],
    requiredShapeRoles: ["axis_ticks", "graph_point", "slope_measure"],
    minGraphSegments: 1,
    primaryView: "graph",
    camera: "fit",
    observations: ["x", "y"],
    lessonPlan: {
      scene_ref: "scene.linear.graph",
      measurement_components: ["axes", "graph_point", "slope"],
      reaction_rule_refs: ["level.linear.point"],
    },
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
      && /\/(?:graph|table|space2d|maegim_control|text)\.(?:json|detjson|md)$/i.test(pathname)
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

function base64UrlUtf8(text) {
  return Buffer.from(String(text ?? ""), "utf8")
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/g, "");
}

async function readCanvasEvidence(page) {
  return page.evaluate(() => {
    const canvas = document.querySelector("[data-testid='lesson-space2d-canvas']");
    if (!(canvas instanceof HTMLCanvasElement)) return null;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx || canvas.width <= 0 || canvas.height <= 0) return { width: rect.width, height: rect.height, colors: 0 };
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    const colors = new Set();
    const stride = Math.max(4, Math.floor((canvas.width * canvas.height) / 4000) * 4);
    for (let i = 0; i < data.length; i += stride) {
      colors.add(`${data[i]}:${data[i + 1]}:${data[i + 2]}:${data[i + 3]}`);
      if (colors.size > 12) break;
    }
    return { width: rect.width, height: rect.height, colors: colors.size };
  });
}

async function waitWorldReady(page, world) {
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 30_000 });
  await page.waitForFunction((expected) => {
    const root = document.querySelector("#screen-run");
    const roles = String(root?.dataset?.mainViewRoles ?? "").split(",").filter(Boolean);
    const hash = String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "");
    const trajectoryCount = Number(root?.dataset?.mainTrajectorySegmentCount ?? 0);
    const graphSegmentCount = Number(root?.dataset?.mainGraphSeriesSegmentCount ?? 0);
    const graphSeriesCount = Number(root?.dataset?.mainGraphSeriesCount ?? 0);
    const graphPointCount = Number(root?.dataset?.mainGraphPointCount ?? 0);
    const roleOk = expected.primaryView === "graph"
      ? graphSeriesCount >= 1 && graphPointCount >= 3
      : expected.roles.every((role) => roles.includes(role));
    const graphOk = expected.primaryView === "graph"
      ? graphSeriesCount >= 1 && graphPointCount >= 3
      : graphSegmentCount >= Number(expected.minGraphSegments ?? 0);
    return roleOk
      && trajectoryCount >= Number(expected.minTrajectorySegments ?? 0)
      && graphOk
      && String(document.querySelector("#run-ddn-preview")?.value ?? "").includes(expected.sourceToken)
      && hash.includes("blake3:");
  }, {
    roles: world.roles,
    primaryView: world.primaryView,
    sourceToken: world.sourceToken,
    minTrajectorySegments: world.minTrajectorySegments ?? 0,
    minGraphSegments: world.minGraphSegments ?? 0,
  }, { timeout: 35_000 });
}

async function readWorldState(page) {
  return page.evaluate(() => {
    const root = document.querySelector("#screen-run");
    let payload = [];
    try {
      payload = JSON.parse(document.querySelector("#run-observe-output-body")?.dataset?.observationPayloadJson ?? "[]");
    } catch (_) {
      payload = [];
    }
    const space = window.__SEAMGRIM_MAIN_VISUAL__?.space2d ?? window.__SEAMGRIM_LAST_VIEWS__?.space2d ?? null;
    return {
      routeLesson: new URL(location.href).searchParams.get("lesson"),
      root: { ...(root?.dataset ?? {}) },
      source: document.querySelector("#run-ddn-preview")?.value ?? "",
      hash: document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "",
      payload,
      shapeRoles: Array.from(new Set((Array.isArray(space?.shapes) ? space.shapes : []).map((shape) => String(shape?.role ?? "")).filter(Boolean))),
      cameraPolicy: root?.dataset?.viewPlanCameraPolicy || space?.meta?.camera_policy || "",
      lessonPlanVisible: document.querySelector("[data-lesson-plan-panel]")?.classList.contains("hidden") === false,
    };
  });
}

async function verifyWorld(page, baseUrl, world) {
  const route = world.primaryView === "graph"
    ? `${baseUrl}${PROJECT_URL}?tab=studio&lesson=${world.lessonId}`
    : `${baseUrl}${PROJECT_URL}?first_delight=${world.firstId}`;
  await page.goto(route, { waitUntil: "domcontentloaded" });
  try {
    await waitWorldReady(page, world);
  } catch (err) {
    throw new Error(`${world.lessonId}: ${String(err?.message ?? err)}`);
  }
  if (world.camera === "follow") {
    await page.locator("#select-view-plan-camera-policy").selectOption("follow");
    await page.waitForFunction(() => document.querySelector("#screen-run")?.dataset?.viewPlanCameraPolicy === "follow", null, { timeout: 10_000 });
  }
  const state = await readWorldState(page);
  if (world.firstId === "pendulum") {
    await page.locator("#btn-pause").click();
    await page.waitForFunction(() => ["paused", "done"].includes(document.querySelector("#screen-run")?.dataset?.engineStatus ?? ""), null, { timeout: 10_000 });
  }
  assert(state.routeLesson === world.lessonId, `${world.lessonId}: route lesson mismatch ${state.routeLesson}`);
  assert(state.source.includes(world.sourceToken), `${world.lessonId}: DDN source token missing`);
  assert(state.hash.includes("blake3:"), `${world.lessonId}: state hash missing`);
  if (world.primaryView === "graph") {
    assert(Number(state.root.mainGraphSeriesCount ?? 0) >= 1, `${world.lessonId}: graph series evidence missing`);
    assert(Number(state.root.mainGraphPointCount ?? 0) >= 3, `${world.lessonId}: graph point evidence missing`);
  } else {
    const visibleRoles = new Set([
      ...String(state.root.mainViewRoles ?? "").split(",").filter(Boolean),
      ...state.shapeRoles,
    ]);
    for (const role of world.roles) {
      assert(visibleRoles.has(role), `${world.lessonId}: role missing ${role}; root=${state.root.mainViewRoles ?? ""}; shapes=${state.shapeRoles.join(",")}`);
    }
  }
  if (world.primaryView !== "graph") {
    for (const role of world.requiredShapeRoles) {
      assert(state.shapeRoles.includes(role), `${world.lessonId}: shape role missing ${role}; roles=${state.shapeRoles.join(",")}`);
    }
  }
  if (world.primaryView !== "graph") {
    assert(state.cameraPolicy === world.camera || String(state.root.mainViewRoles ?? "").includes(world.camera), `${world.lessonId}: camera policy missing`);
  }
  const payloadText = JSON.stringify(state.payload);
  for (const key of world.observations) {
    assert(payloadText.includes(key) || state.source.includes(key), `${world.lessonId}: observation key missing ${key}`);
  }
  if (world.primaryView === "graph") {
    assert(Number(state.root.mainGraphSeriesCount ?? 0) >= 1 && Number(state.root.mainGraphPointCount ?? 0) >= 3, `${world.lessonId}: graph primary evidence invalid`);
  } else {
    const canvas = await readCanvasEvidence(page);
    assert(canvas && canvas.width >= 300 && canvas.height >= 180 && canvas.colors > 4, `${world.lessonId}: canvas invalid ${JSON.stringify(canvas)}`);
  }

  const reopenedUrl = `${baseUrl}${PROJECT_URL}?tab=studio&ddn=${base64UrlUtf8(state.source)}`;
  await page.goto(reopenedUrl, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#screen-run:not(.hidden)", { state: "visible", timeout: 20_000 });
  await page.waitForFunction((token) => String(document.querySelector("#run-ddn-preview")?.value ?? "").includes(token), world.sourceToken, { timeout: 20_000 });
  const reopenedHash = await page.locator("#run-mirror-hash").getAttribute("title");
  if (!String(reopenedHash ?? "").includes("blake3:")) {
    await page.locator("#btn-run").click();
  }
  await page.waitForFunction(() => String(document.querySelector("#run-mirror-hash")?.getAttribute("title") ?? "").includes("blake3:"), null, { timeout: 35_000 });
}

async function verifyLessonComposition(page, baseUrl) {
  const toml = [
    'schema = "seamgrim.lesson_plan.v0"',
    'lesson_id = "wave2b_representative_worlds"',
    'title = "Wave2B 대표 세계"',
    "",
    ...WORLDS.flatMap((world, index) => [
      index ? "" : "",
      "[[activities]]",
      `activity_id = "${world.lessonId}"`,
      'kind = "observe"',
      `title = "${world.lessonId}"`,
      `prompt = "${world.lessonId} 실행 흐름을 관찰하세요."`,
      `scene_ref = "${world.lessonPlan.scene_ref}"`,
      `primary_view = "${world.primaryView}"`,
      `camera_policy = "${world.camera}"`,
      `measurement_components = [${world.lessonPlan.measurement_components.map((item) => `"${item}"`).join(", ")}]`,
      `reaction_rule_refs = [${world.lessonPlan.reaction_rule_refs.map((item) => `"${item}"`).join(", ")}]`,
      `view_authoring_hash = "candidate:fnv1a32:view_reaction_authoring:${world.lessonId}"`,
      'view_plan_ref = "view_plan.v1"',
    ]),
  ].join("\n");
  await page.goto(`${baseUrl}${PROJECT_URL}`, { waitUntil: "domcontentloaded" });
  const result = await page.evaluate(async (source) => {
    const { parseLessonPlanToml, serializeLessonPlanToml, roundtripLessonPlanToml } = await import("./lesson_plan_v0.js");
    const parsed = parseLessonPlanToml(source);
    const canonical = serializeLessonPlanToml(parsed);
    const roundtrip = roundtripLessonPlanToml(canonical);
    return { parsed, canonical, roundtrip };
  }, toml);
  assert(result.parsed.activities.length === 3, `composition activity count mismatch: ${result.parsed.activities.length}`);
  assert(result.roundtrip.byte_stable === true, "composition canonical bytes are not stable");
  assert(result.roundtrip.semantic_equal === true, "composition semantic roundtrip failed");
  for (const world of WORLDS) {
    const activity = result.parsed.activities.find((row) => row.activity_id === world.lessonId);
    assert(activity, `composition missing ${world.lessonId}`);
    assert(activity.primary_view === world.primaryView, `${world.lessonId}: primary view not preserved`);
    assert(activity.camera_policy === world.camera, `${world.lessonId}: camera policy not preserved`);
    assert(activity.measurement_components.join(",") === world.lessonPlan.measurement_components.join(","), `${world.lessonId}: measures not preserved`);
    assert(activity.reaction_rule_refs.join(",") === world.lessonPlan.reaction_rule_refs.join(","), `${world.lessonId}: reactions not preserved`);
    assert(activity.view_plan_ref === "view_plan.v1", `${world.lessonId}: view plan ref not preserved`);
  }
}

async function main() {
  const { server, baseUrl } = await createServer(path.resolve("."));
  const browser = await chromium.launch();
  const failures = [];
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 760 }, locale: "ko-KR" });
    installFailureCapture(page, failures);
    for (const world of WORLDS) {
      await verifyWorld(page, baseUrl, world);
    }
    await verifyLessonComposition(page, baseUrl);
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
