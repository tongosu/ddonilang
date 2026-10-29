#!/usr/bin/env node
import assert from "node:assert/strict";
import { chromium } from "playwright";

// 별도 제품 서버에서 버전 표시만 검사한다. 실행/교재 합격을 주장하지 않는다.
const baseUrl = process.env.SEAMGRIM_VERSION_BASE_URL || "http://127.0.0.1:18954";
const url = new URL("/ui/workbench/index.html", baseUrl);
assert(["127.0.0.1", "localhost"].includes(url.hostname), "별도 로컬 서버만 허용");
assert.notEqual(url.port, "8790", "사용자 서버에는 접근하지 않음");
const expectedTitle = "셈그림 결정적 작업대 · 1.0.0-alpha.1 · 알파 후보";
const expectedLabel = "1.0.0-alpha.1 · 알파 후보";
const browser = await chromium.launch({ headless: true,
  ...(process.env.SEAMGRIM_BROWSER_CHANNEL ? { channel: process.env.SEAMGRIM_BROWSER_CHANNEL } : {}) });
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  await page.goto(url.href, { waitUntil: "domcontentloaded" });
  assert.equal(await page.title(), expectedTitle, "초기 창 제목에 버전과 알파 후보 표시");
  const label = page.getByTestId("product-version");
  assert.equal(await label.textContent(), expectedLabel);
  assert(await label.isVisible(), "상단 버전 표시가 실제 보임");
  await page.waitForSelector("[data-testid='approved-workbench-root'][data-project-id]", { timeout: 30000 });
  assert.equal(await page.title(), expectedTitle, "초기 프로젝트 로드 후 제목 유지");
  assert.equal(await label.textContent(), expectedLabel, "초기 프로젝트 로드 후 버전 유지");
  await page.setViewportSize({ width: 390, height: 844 });
  assert(await label.isVisible(), "좁은 화면에서도 상단 버전 표시가 보임");
  const bounds = await label.boundingBox();
  assert(bounds && bounds.x >= 0 && bounds.y >= 0 && bounds.x + bounds.width <= 390,
    "좁은 화면에서 버전 표시가 화면 안에 있음");
  console.log(JSON.stringify({ status: "PASS_VERSION_DISPLAY_ONLY", url: url.href,
    title: await page.title(), label: await label.textContent(), desktop_visible: true,
    project_load_preserves_title: true, narrow_visible: true, narrow_bounds: bounds }));
} finally {
  await browser.close();
}
