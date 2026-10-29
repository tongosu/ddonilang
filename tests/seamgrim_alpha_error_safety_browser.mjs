import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const browser = await chromium.launch({headless:true, executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
try {
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.goto(process.env.DDN_TEST_URL || 'http://127.0.0.1:8790/solutions/seamgrim_ui_mvp/ui/workbench/index.html');
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.source);
  assert.equal(await page.locator('a[aria-label="셈그림 기존 화면으로 돌아가기"]').count(), 0);
  assert.equal(await page.getByTestId('project-verification-open').isVisible(), true);
  await page.locator('#source-editor').fill('값 := (1 / 0). 값 보여주기.');
  await page.getByTestId('lifecycle-run').click();
  await page.getByTestId('project-verification-drawer').waitFor({state:'visible'});
  assert.match(await page.getByTestId('project-verification-drawer').innerText(), /0으로 나눌 수 없습니다/);
  assert.doesNotMatch(await page.getByTestId('project-verification-drawer').innerText(), /E_RUNTIME_WORKER/);
  assert.equal(await page.evaluate(() => {
    const list = document.querySelector('[data-testid="diagnostic-list"]');
    const metadata = document.querySelector('[data-testid="view-controls"]');
    return Boolean(list.compareDocumentPosition(metadata) & Node.DOCUMENT_POSITION_FOLLOWING);
  }), true);
  assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'), '');
  await page.getByTestId('project-verification-close').click();
  await page.locator('#source-editor').fill('(1 / 0) 보여주기.');
  await page.getByTestId('lifecycle-run').click();
  await page.getByTestId('project-verification-drawer').waitFor({state:'visible'});
  assert.match(await page.getByTestId('project-verification-drawer').innerText(), /산술고장/);
  await page.getByTestId('project-verification-close').click();
  await page.locator('#source-editor').fill('(매마디)마다 { 값 <- (1 / 0). }.');
  await page.getByTestId('lifecycle-run').click();
  await page.getByTestId('project-verification-drawer').waitFor({state:'visible'});
  assert.match(await page.getByTestId('project-verification-drawer').innerText(), /산술고장/);
  assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'), '');
  await page.getByTestId('project-verification-close').click();
  await page.locator('#source-editor').fill('값 := 0. { 참 }인 동안 { 값 <- 값 + 1. }.');
  await page.getByTestId('lifecycle-run').click();
  await page.getByTestId('lifecycle-stop').waitFor({state:'visible'});
  await page.waitForFunction(() => !document.querySelector('[data-testid="lifecycle-stop"]').disabled);
  // A round-trip on the page must remain responsive while the WASM loop runs.
  assert.equal(await page.evaluate(() => 2 + 2), 4);
  await page.getByTestId('lifecycle-stop').click({timeout:3000});
  await page.waitForFunction(() => !document.querySelector('[data-testid="lifecycle-run"]').disabled);
  await page.locator('#source-editor').fill('값 := 100. 값 보여주기.');
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(() => document.querySelector('#canvas-bogae').dataset.consoleOutput === '100');
  await page.getByTestId('project-rail-toggle').click();
  await page.getByTestId('new-ddn-project').click();
  const payload = '<img src=x onerror="window.__xss=1">';
  await page.getByTestId('project-create-name').fill(payload);
  await page.getByTestId('project-create-submit').click();
  await page.waitForFunction(p => document.querySelector('#project-title')?.textContent === p, payload);
  assert.equal(await page.evaluate(() => window.__xss), undefined);
  assert.equal(await page.locator('#project-list img').count(), 0);
  assert.ok((await page.locator('#project-list').textContent()).includes(payload));
  const exported = await page.evaluate(async () =>
    (await window.__SEAMGRIM_APPROVED_WORKBENCH__.buildCurrentPortableProjectArtifact()).file.json_text);
  await page.getByTestId('portable-project-import-input').setInputFiles({
    name: 'literal-title.seamgrim-project.json', mimeType: 'application/json',
    buffer: Buffer.from(exported, 'utf8'),
  });
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().pendingPortableImport?.has_artifact);
  await page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.openPendingPortableProject());
  assert.equal(await page.evaluate(() => window.__xss), undefined);
  assert.equal(await page.locator('#project-list img').count(), 0);
  assert.ok((await page.locator('#project-list').textContent()).includes(payload));
  assert.deepEqual(errors, []);
  console.log('ALPHA_ERROR_SAFETY_PASS runtime_reason=1 arithmetic_signal=2 infinite_stop_recover=1 literal_project_title=1 portable_import=1 legacy_link_removed=1');
} finally { await browser.close(); }
