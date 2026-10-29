import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs/promises';

// Final archive verification supplies the fresh extraction root/server explicitly.
// Do not fall back to a developer server when an archive URL was supplied.
const root = path.resolve(process.env.DDN_TEST_ROOT || process.cwd());
const baseUrl = process.env.DDN_TEST_BASE_URL || 'http://127.0.0.1:18933/';
const server = process.env.DDN_TEST_BASE_URL ? null : spawn('python',
  [path.join(root, 'solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py'), '--port', '18933'],
  { cwd: root, windowsHide: true, env: { ...process.env, PYTHONDONTWRITEBYTECODE: '1' }, stdio: 'ignore' });
const results = [];
async function fallingPiecePixels(page) {
  return page.locator('#canvas-bogae').evaluate(canvas => {
    const { data } = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height);
    let count = 0, totalY = 0;
    for (let index = 0; index < data.length; index += 4) {
      if (data[index] > 220 && data[index + 1] > 160 && data[index + 1] < 230 && data[index + 2] < 40) {
        count += 1;
        totalY += Math.floor(index / 4 / canvas.width);
      }
    }
    return { count, y: count ? totalY / count : null };
  });
}
let browser;
try {
  for (let i = 0; i < 60; i++) {
    try { if ((await fetch(baseUrl)).ok) break; } catch {}
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  browser = await chromium.launch({headless: true, executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  for (const title of ['포물선 운동', '진자 보기 코어', '일차함수 기울기 작업', '새 물리 프로젝트', '새 한 파일', '새 빈 프로젝트']) {
    const context = await browser.newContext();
    const network = [];
    const failedRequests = [];
    const pageErrors = [];
    // Context events include requests from module Workers, not just the page.
    context.on('response', response => network.push({
      url: response.url(), status: response.status(),
      resourceType: response.request().resourceType(),
    }));
    context.on('requestfailed', request => failedRequests.push({
      url: request.url(), error: request.failure()?.errorText || 'request failed',
      resourceType: request.resourceType(),
    }));
    context.on('page', opened => opened.on('pageerror', error => pageErrors.push(error.message)));
    const page = await context.newPage();
    await page.goto(baseUrl);
    await page.waitForFunction(() => document.querySelector('#source-editor')?.value.length > 0);
    await page.getByTestId('project-rail-toggle').click();
    if (title.startsWith('새 ')) {
      await page.getByTestId('new-ddn-project').click();
      await page.getByTestId('project-create-name').fill('새 예제 검사');
      if (title === '새 한 파일') await page.getByRole('radio', {name: '한 파일 시작본'}).check();
      if (title === '새 빈 프로젝트') await page.getByRole('radio', {name: '빈 DDN 프로젝트', exact: true}).check();
      await page.getByTestId('project-create-submit').click();
      await page.waitForFunction(() => {
        const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
        return snapshot.projectId.startsWith('native_') && snapshot.projectControls?.status === 'ready';
      });
    } else {
      await page.getByRole('option', {name: new RegExp(title)}).click();
      await page.waitForFunction(expected => {
        const snapshot = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
        return snapshot.project?.title === expected && snapshot.projectControls?.status === 'ready';
      }, title);
    }
    const source = await page.locator('#source-editor').inputValue();
    if (process.env.DDN_DT_ARTIFACT_DIR) {
      const artifact = await page.evaluate(async () => window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact());
      await fs.mkdir(process.env.DDN_DT_ARTIFACT_DIR, {recursive: true});
      await fs.writeFile(path.join(process.env.DDN_DT_ARTIFACT_DIR, `${title}.json`), artifact.file.json_text, 'utf8');
    }
    if (title.startsWith('새 ')) {
      await page.getByTestId('project-save').click();
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().persistence?.phase === 'saved');
      await page.locator('#source-editor').fill('임시 변경');
      await page.getByTestId('project-reopen').click();
      await page.waitForFunction(expected => document.querySelector('#source-editor')?.value === expected, source).catch(async error => {
        console.error('REOPEN_DIAGNOSTIC', JSON.stringify(await page.evaluate(() => ({
          source: document.querySelector('#source-editor')?.value,
          snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
        }))));
        throw error;
      });
      assert.equal(await page.locator('#source-editor').inputValue(), source, `${title}: 저장·재열기 원문 변경`);
    }
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(() => {
      const c = document.querySelector('#canvas-bogae');
      return Boolean(c?.dataset.committedMadi) || document.querySelector('[data-testid="run-status"]')?.textContent.includes('실행 실패');
    });
    if (title === '일차함수 기울기 작업') {
      assert.doesNotMatch(source, /매마디/, 'static linear example must execute once');
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle?.status === 'done');
    }
    if (title === '포물선 운동') {
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi >= 3);
      await page.getByTestId('lifecycle-run').click();
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle?.status === 'paused');
      const hash = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash);
      const picture = await page.locator('#canvas-bogae').evaluate(canvas => canvas.toDataURL());
      await page.getByTestId('view-role-overlay').click();
      assert.ok(await page.locator('#canvas-bogae').evaluate(canvas => canvas.toDataURL()) !== picture, JSON.stringify(await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().semanticRenderer)));
      assert.match(await page.getByTestId('spatial-trail').getAttribute('title'), /화면이 기록한/);
      await page.getByTestId('view-role-side').click();
      assert.equal(await page.locator(".side-view-instance[data-view-instance-role='side'] canvas").getAttribute('data-side-graph-panels'), '2');
      assert.equal(await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash), hash, 'projectile views changed world');
    }
    if (title === '진자 보기 코어') {
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi >= 3);
      await page.getByTestId('lifecycle-run').click();
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle?.status === 'paused');
      const worldBefore = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash);
      await page.getByTestId('view-role-side').click();
      await page.getByTestId('view-role-side').click();
      assert.equal(await page.locator(".side-view-instance[data-view-instance-role='side']").count(), 1, 'same side view duplicated');
      const sideCanvas = page.locator(".side-view-instance[data-view-instance-role='side'] canvas");
      assert.equal(await sideCanvas.getAttribute('data-side-graph-panels'), '3', 'pendulum side must show angle, velocity and energy panels');
      assert.ok(Number(await sideCanvas.getAttribute('data-side-graph-samples')) >= 3, 'side graph must use actual committed history');
      const id = await page.locator(".side-view-instance[data-view-instance-role='side']").getAttribute('data-view-instance-id');
      await page.locator(`[data-view-instance-close='${id}']`).click();
      assert.equal(await page.locator(".side-view-instance[data-view-instance-role='side']").count(), 0, 'side not hidden');
      await page.getByTestId('view-role-side').click();
      assert.equal(await page.locator(".side-view-instance[data-view-instance-role='side']").getAttribute('data-view-instance-id'), id, 'reopen replaced instance');
      const worldAfter = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash);
      assert.equal(worldAfter, worldBefore, 'view operations changed world state');
      const imageBefore = await page.locator('#canvas-bogae').evaluate(canvas => canvas.toDataURL());
      await page.getByTestId('view-role-overlay').click();
      const imageAfter = await page.locator('#canvas-bogae').evaluate(canvas => canvas.toDataURL());
      assert.notEqual(imageAfter, imageBefore, 'angle overlay must actually draw');
      assert.equal(await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash), worldBefore);
    }
    if (title === '격자 상태와 낙하 조각') {
      assert.match(source, /\(10, 20, 빈\)/, 'falling board must have 10 columns and 20 rows');
      const initial = await page.locator('#canvas-bogae').evaluate(canvas => canvas.toDataURL());
      const firstPiece = await fallingPiecePixels(page);
      await page.waitForFunction(() => {
        const s = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
        return s.activeFrame?.madi >= 4 || s.executionLifecycle.status === 'failed';
      });
      assert.notEqual(await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status), 'failed', JSON.stringify(await page.evaluate(() => { const s = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(); return {diagnostics:s.diagnostics, frame:s.activeFrame}; })));
      await page.getByTestId('lifecycle-run').click();
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle?.status === 'paused');
      const fallenPiece = await fallingPiecePixels(page);
      assert.ok(firstPiece.count > 0 && fallenPiece.count > 0, 'actual yellow piece must be visible');
      assert.ok(fallenPiece.y > firstPiece.y, 'falling row must move DOWN in actual canvas pixels');
      assert.notEqual(await page.locator('#canvas-bogae').evaluate(canvas => canvas.toDataURL()), initial, 'grid must fall rather than reinitialize');
      const madi = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.madi);
      const x = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().semanticRenderer.position?.x);
      assert.equal(typeof x, 'number', 'grid position must come from committed WASM result');
      await page.locator('#canvas-bogae').press('ArrowLeft');
      assert.equal(await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().nextInput?.lastKey), 'ArrowLeft');
      await page.getByTestId('lifecycle-step').click();
      await page.waitForFunction(previous => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi === previous + 1, madi);
      assert.equal(await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().semanticRenderer.position?.x), x - 1, 'actual key must move WASM grid piece');
      await page.getByTestId('view-role-side').click();
      await page.getByTestId('view-role-side').click();
      assert.equal(await page.locator(".side-view-instance[data-view-instance-role='side']").count(), 1);
      if (process.env.DDN_GRID_SCREENSHOT_DIR) {
        await fs.mkdir(process.env.DDN_GRID_SCREENSHOT_DIR, { recursive: true });
        await page.screenshot({ path: path.join(process.env.DDN_GRID_SCREENSHOT_DIR, 'grid-10x20-down.png'), fullPage: true });
      }
    }
    const result = await page.evaluate(() => ({
      status: document.querySelector('[data-testid="run-status"]')?.textContent ?? '',
      family: document.querySelector('#canvas-bogae')?.dataset.rendererFamily,
      madi: document.querySelector('#canvas-bogae')?.dataset.committedMadi,
      console: document.querySelector('#canvas-bogae')?.dataset.consoleOutput,
      error: document.querySelector('[data-testid="project-verification-drawer"]')?.textContent,
    }));
    if (title.startsWith('새 ')) {
      const before = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1).latest_state_hash);
      const downloaded = page.waitForEvent('download');
      await page.getByTestId('portable-project-export').click();
      const file = await downloaded;
      await page.getByTestId('portable-project-import-input').setInputFiles(await file.path());
      await page.waitForFunction(() => document.querySelector('[data-testid="portable-project-import-preview"]')?.dataset.state === 'ready');
      await page.getByTestId('portable-project-open').click();
      await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId.startsWith('portable_'));
      await page.getByTestId('lifecycle-run').click();
      await page.waitForFunction(() => Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1)?.latest_state_hash));
      const after = await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1).latest_state_hash);
      assert.equal(after, before, `${title}: 실제 내보내기·가져오기 뒤 세계 해시`);
    }
    await page.waitForLoadState('networkidle');
    const httpErrors = network.filter(response => response.status >= 400);
    results.push({title, legacy: /매마디\s*:\s*움직씨/.test(source), ...result,
      network: {responses: network, notFoundCount: network.filter(r => r.status === 404).length,
        httpErrors, failedRequests, pageErrors}});
    console.log(JSON.stringify(results.at(-1)));
    await context.close();
  }
  for (const result of results) {
    assert.equal(result.legacy, false, `${result.title}: 옛 훅 표면`);
    assert.equal(result.network.notFoundCount, 0, `${result.title}: 404 요청`);
    assert.deepEqual(result.network.httpErrors, [], `${result.title}: HTTP 오류`);
    assert.deepEqual(result.network.failedRequests, [], `${result.title}: 요청 실패`);
    assert.deepEqual(result.network.pageErrors, [], `${result.title}: 화면 오류`);
    assert.ok(result.madi, `${result.title}: ${result.error}`);
    assert.doesNotMatch(result.status, /실패/);
    if (!result.title.startsWith('새 ')) assert.equal(result.family, 'space2d', `${result.title}: 실제 도형 출력`);
  }
  console.log('WORKBENCH_CATALOG_PASS examples=3 new_project=3 save_reopen=3 export_import=3 legacy_hooks=0 network_404=0 request_failed=0');
} finally {
  if (browser) await browser.close();
  if (server) server.kill();
}
