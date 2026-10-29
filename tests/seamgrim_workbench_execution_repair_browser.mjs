import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import path from 'node:path';
const root = process.cwd();
const server = spawn('python', [path.join(root, 'solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py'), '--port', '18931'],
  { cwd: root, windowsHide: true, env: { ...process.env, PYTHONDONTWRITEBYTECODE: '1' }, stdio: 'ignore' });
let browser;
try {
  for (let i = 0; i < 60; i++) {
    try { if ((await fetch('http://127.0.0.1:18931/')).ok) break; } catch {}
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  browser = await chromium.launch({headless: true, executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  const page = await browser.newPage({viewport: {width: 1366, height: 768}});
  await page.goto('http://127.0.0.1:18931/');
  await page.waitForFunction(() => document.querySelector('#source-editor')?.value.includes('매마디'));
  const original = await page.locator('#source-editor').inputValue();
  const withText = original.replace(/(\(매마디\)마다\s*\{)/, '$1\n "그림과 함께" 보여주기.');
  assert.notEqual(withText, original, 'Fixture must add a real output statement to the executing hook');
  await page.locator('#source-editor').fill(withText);
  const primary = page.getByTestId('lifecycle-run');
  assert.equal(await page.getByTestId('lifecycle-pause').count(), 0);
  assert.equal(await page.getByTestId('lifecycle-resume').count(), 0);
  await primary.click();
  await page.waitForFunction(() => document.querySelector('#canvas-bogae')?.dataset.rendererFamily === 'space2d');
  assert.equal(await page.getByTestId('bogae-text-output').isVisible(), true);
  await page.getByTestId('bogae-text-output').locator('summary').click();
  assert.match(await page.getByTestId('bogae-text-output-lines').innerText(), /그림과 함께/);
  await primary.click();
  await page.waitForFunction(() => document.querySelector('[data-testid="lifecycle-run"]')?.dataset.action === 'resume');
  for (let i = 0; i < 6; i++) await page.getByTestId('lifecycle-step').click();
  const canvas = page.locator('#canvas-bogae');
  assert.ok(Number(await canvas.getAttribute('data-trail-segments')) > 0, 'Actual moving positions must form a trail');
  const savedRange = await canvas.getAttribute('data-view-range');
  await page.getByTestId('spatial-trail').click();
  assert.equal(await canvas.getAttribute('data-trail-segments'), '0');
  assert.equal(await canvas.getAttribute('data-view-range'), savedRange, 'Trail visibility must not change the coordinate system');
  await page.getByTestId('spatial-trail').click();
  await page.getByTestId('spatial-fit').click();
  assert.ok(JSON.parse(await canvas.getAttribute('data-view-range')).xMax > 0);
  await page.getByTestId('spatial-reset').click();
  const divider = page.getByTestId('panel-divider');
  const beforeWidth = (await page.locator('.source-panel').boundingBox()).width;
  await divider.focus();
  await page.keyboard.press('ArrowRight');
  assert.ok((await page.locator('.source-panel').boundingBox()).width > beforeWidth + 10);
  const grip = await divider.boundingBox();
  const dragY = Math.min(600, Math.max(120, grip.y + grip.height / 2));
  assert.equal(await page.evaluate(({x,y}) => document.elementFromPoint(x,y)?.dataset.testid,
    {x:grip.x+grip.width/2,y:dragY}), 'panel-divider');
  await page.mouse.move(grip.x + grip.width / 2, dragY);
  await page.mouse.down();
  await page.mouse.move(grip.x - 100, dragY, {steps: 4});
  await page.mouse.up();
  assert.ok((await page.locator('.source-panel').boundingBox()).width < beforeWidth,
    JSON.stringify({after: await page.locator('.source-panel').boundingBox(), value:await divider.getAttribute('aria-valuenow')}));
  await primary.click();
  await page.waitForFunction(() => document.querySelector('[data-testid="lifecycle-run"]')?.dataset.action === 'pause');
  await primary.click();
  await page.waitForFunction(() => document.querySelector('[data-testid="lifecycle-run"]')?.dataset.action === 'resume');
  await page.locator('#source-editor').fill('값 := .');
  await page.waitForFunction(() => document.querySelector('[data-testid="lifecycle-run"]')?.getAttribute('aria-label') === '수정본 실행');
  await primary.click();
  await page.getByTestId('project-verification-drawer').waitFor({state: 'visible'});
  assert.match(await page.getByTestId('run-status').innerText(), /실행 실패/);
  assert.equal(await page.locator('#canvas-bogae').getAttribute('data-committed-madi'), '');
  assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'), '');
  assert.equal(await canvas.getAttribute('data-view-range'), '');
  assert.equal(await page.getByTestId('spatial-fit').isDisabled(), true);
  await page.getByTestId('project-verification-close').click();
  await page.locator('#source-editor').fill('값 := 100. 값 보여주기.');
  await primary.click();
  await page.waitForFunction(() => document.querySelector('#canvas-bogae')?.dataset.consoleOutput === '100');
  for (const invalid of [
    '만약 1 이라면 { 100 보여주기. }.',
    '값:수 := 1. 값 <- "가". 값 보여주기.',
  ]) {
    await page.locator('#source-editor').fill(invalid);
    await primary.click();
    await page.getByTestId('project-verification-drawer').waitFor({state: 'visible'});
    assert.match(await page.getByTestId('run-status').innerText(), /실행 실패/);
    assert.equal(await canvas.getAttribute('data-console-output'), '');
    assert.equal(await canvas.getAttribute('data-committed-madi'), '');
    await page.getByTestId('project-verification-close').click();
    await page.locator('#source-editor').fill('값:수 := 1. 값 <- 100. 값 보여주기.');
    await primary.click();
    await page.waitForFunction(() => document.querySelector('#canvas-bogae')?.dataset.consoleOutput === '100');
  }
  await page.route('**/lessons/rep_grid_game_state_drop_v1/lesson.ddn', route => route.fulfill({status: 404, body: 'missing'}));
  await page.getByRole('button', {name: '프로젝트', exact: true}).click();
  await page.getByTestId('project-card-rep_grid_game_state_drop_v1').click();
  await page.getByTestId('project-verification-drawer').waitFor({state: 'visible'});
  assert.match(await page.getByTestId('project-verification-drawer').innerText(), /E_PROJECT_FILE_LOAD/);
  assert.equal(await primary.isDisabled(), true, 'A missing example must not run fallback source');
  assert.equal(await page.locator('#source-editor').inputValue(), '');
  await page.getByTestId('project-verification-close').click();
  await page.unroute('**/lessons/rep_grid_game_state_drop_v1/lesson.ddn');
  if (!await page.getByTestId('project-card-rep_grid_game_state_drop_v1').isVisible()) {
    await page.getByRole('button', {name: '프로젝트', exact: true}).click();
  }
  await page.getByTestId('project-card-rep_grid_game_state_drop_v1').click();
  await page.waitForFunction(() => document.querySelector('#source-editor')?.value.length > 100);
  assert.equal(await primary.isDisabled(), false);
  await primary.click();
  try {
    await page.waitForFunction(() => document.querySelector('#canvas-bogae')?.dataset.rendererFamily === 'space2d'
      && document.querySelector('#canvas-bogae')?.dataset.committedStateHash);
  } catch (error) {
    console.error('GRID_FAILURE', await page.getByTestId('run-status').innerText(),
      await page.getByTestId('project-verification-drawer').innerText(),
      await page.evaluate(() => JSON.stringify({family: document.querySelector('#canvas-bogae').dataset.rendererFamily,
        space: globalThis.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__?.space2d,
        grid: {
          status: globalThis.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__?.workbench_grid?.status,
          count: globalThis.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__?.workbench_grid?.content?.result?.space2d?.drawlist?.length,
          first: globalThis.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__?.workbench_grid?.content?.result?.space2d?.drawlist?.[0],
        }})));
    throw error;
  }
  // The grid runtime emits an explicit camera. Exercise that actual payload,
  // without injecting a drawing or changing the program's runtime state.
  if (await primary.getAttribute('data-action') === 'pause') {
    await primary.click();
    await page.waitForFunction(() => document.querySelector('[data-testid="lifecycle-run"]')?.dataset.action === 'resume');
  }
  const explicitCamera = await page.evaluate(() => {
    const payloads = globalThis.__SEAMGRIM_WORKBENCH_CANONICAL_VIEW_PAYLOADS__;
    return payloads?.space2d?.content?.space2d?.camera
      ?? payloads?.workbench_grid?.content?.result?.space2d?.camera;
  });
  assert.ok(explicitCamera, 'Actual grid output must contain an explicit camera');
  const expectedCameraRange = {xMin: explicitCamera.x_min, xMax: explicitCamera.x_max,
    yMin: explicitCamera.y_min, yMax: explicitCamera.y_max};
  assert.deepEqual(JSON.parse(await canvas.getAttribute('data-view-range')), expectedCameraRange);
  const stateBeforeView = await canvas.getAttribute('data-committed-state-hash');
  assert.ok(stateBeforeView);
  await page.getByTestId('spatial-fit').click();
  assert.equal(await canvas.getAttribute('data-committed-state-hash'), stateBeforeView);
  await page.getByTestId('spatial-reset').click();
  assert.deepEqual(JSON.parse(await canvas.getAttribute('data-view-range')), expectedCameraRange);
  assert.equal(await canvas.getAttribute('data-committed-state-hash'), stateBeforeView);
  await divider.focus();
  await page.keyboard.press('ArrowLeft');
  assert.equal(await canvas.getAttribute('data-committed-state-hash'), stateBeforeView);
  await page.setViewportSize({width:390,height:844});
  assert.equal(await divider.isVisible(), false);
  assert.equal(await canvas.isVisible(), true);
  assert.equal(await canvas.getAttribute('data-committed-state-hash'), stateBeforeView);
  console.log('EXECUTION_REPAIR_PASS primary_pause_resume=1 paused_edit=1 failed_output_cleared=1 repair=1 graphics_and_text=1 trail=1 viewport=1 divider=1 missing_example_blocked=1 grid=1 mobile=1 explicit_camera=1 view_state_unchanged=1 approved_condition_binding=2');
} finally {
  if (browser) await browser.close();
  server.kill();
}
