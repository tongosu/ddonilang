import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
const { chromium } = process.env.DDN_PLAYWRIGHT_MODULE
  ? await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href)
  : await import('playwright');
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer } from 'node:net';
import fs from 'node:fs/promises';
import path from 'node:path';

// This runner uses the shipped course and real UI/Worker. An explicitly supplied
// extraction root/server never falls back to an existing development server.
const root = path.resolve(process.env.DDN_TEST_ROOT || process.cwd());
const courseDir = path.join(root, 'solutions/seamgrim_ui_mvp/first_steps');
const manifest = JSON.parse(await fs.readFile(path.join(courseDir, '검증목록.json'), 'utf8'));
assert.equal(manifest.cases.length, 40);
assert.equal(manifest.corrections.length, 5);
assert.equal(manifest.modifications.length, 10);
const rows = [...manifest.cases, ...manifest.corrections, ...manifest.modifications];
assert.equal(new Set(rows.map(row => row.id)).size, 55);
assert.equal(rows.filter(row => !Array.isArray(row.expected)).length, 5);
const sources = new Map(await Promise.all(rows.map(async row => [row.id,
  (await fs.readFile(path.join(courseDir, row.file), 'utf8')).replace(/\r\n/g, '\n')])));
const baseUrl = process.env.DDN_TEST_BASE_URL || 'http://127.0.0.1:18935/';
const responses = [], failedRequests = [], pageErrors = [], workerUrls = [], results = [];
let server, browser, context;

async function startOwnServer() {
  // A busy port is a blocker; do not probe or reuse its foreign HTTP service.
  await new Promise((resolve, reject) => {
    const reservation = createServer();
    reservation.once('error', reject);
    reservation.listen({host: '127.0.0.1', port: 18935, exclusive: true}, () => reservation.close(resolve));
  });
  server = spawn(process.env.DDN_TEST_PYTHON || 'python',
    [path.join(root, 'solutions/seamgrim_ui_mvp/tools/ddn_exec_server.py'), '--port', '18935'],
    {cwd: root, windowsHide: true, env: {...process.env, PYTHONDONTWRITEBYTECODE: '1'}, stdio: ['ignore', 'pipe', 'pipe']});
  let serverError = '';
  server.stderr.on('data', data => { serverError += data.toString(); });
  await new Promise((resolve, reject) => {
    let readyText = '';
    const timer = setTimeout(() => reject(new Error(`첫걸음 검사 서버 준비 timeout: ${serverError}`)), 20000);
    server.once('error', error => { clearTimeout(timer); reject(error); });
    server.once('exit', code => { clearTimeout(timer); reject(new Error(`첫걸음 검사 서버 종료 ${code}: ${serverError}`)); });
    server.stdout.on('data', data => {
      readyText += data.toString();
      if (readyText.includes('seamgrim.ddn_exec_server.ready.v1')) { clearTimeout(timer); resolve(); }
    });
  });
}

async function snapshot(page) {
  return page.evaluate(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
}

async function closeDiagnostic(page) {
  const close = page.getByTestId('project-verification-close');
  if (await close.isVisible()) await close.click();
}

async function waitProject(page, row) {
  await page.waitForFunction(id => {
    const state = window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot();
    return state?.projectId === `first_steps_${id}` && ['ready', 'error'].includes(state.projectControls?.status);
  }, row.id).catch(async error => {
    console.error('FIRST_STEPS_PROJECT_DIAGNOSTIC', JSON.stringify({id: row.id, snapshot: await snapshot(page),
      source: await page.locator('#source-editor').inputValue(), httpErrors: responses.filter(response => response.status >= 400),
      failedRequests, pageErrors}));
    throw error;
  });
  const state = await snapshot(page);
  // The optional control authoring plan is not the execution gate: a valid
  // named-pin course can fail canonMaegimPlan while the shared frontend/VM runs.
  assert.ok(['ready', 'error'].includes(state.projectControls.status));
  assert.equal(state.controls.lifecycle_run.enabled, true, `${row.id}: ${JSON.stringify(state.diagnostics)}`);
  assert.equal(state.project.title, `첫걸음 ${row.id} · ${row.title}`);
  assert.equal(await page.locator('#source-editor').inputValue(), sources.get(row.id), `${row.id}: 수업 원문`);
  const guide = page.getByTestId('first-steps-guide');
  if (await guide.evaluate(node => node.open)) await page.getByTestId('first-steps-close').click();
  assert.equal(await page.getByTestId('first-steps-panel').isVisible(), false, `${row.id}: 안내 접기`);
  const beforeGuide = await page.locator('#source-editor').boundingBox();
  await page.getByTestId('first-steps-open').click();
  assert.equal(await page.getByTestId('first-steps-panel').isVisible(), true, `${row.id}: 수업 단계 패널`);
  const afterGuide = await page.locator('#source-editor').boundingBox();
  assert.equal(afterGuide.y, beforeGuide.y, `${row.id}: 안내가 작업 공간을 아래로 밀면 안 됨`);
  assert.equal(afterGuide.height, beforeGuide.height, `${row.id}: 편집 높이 보존`);
  const panelText = await page.getByTestId('first-steps-panel').innerText();
  assert.ok(panelText.includes(row.id) && panelText.includes(row.title), `${row.id}: 현재 수업 메타정보`);
  const expected = page.getByTestId('first-steps-expected');
  assert.equal(await expected.evaluate(node => node.tagName), 'DETAILS', `${row.id}: 기대결과 접기`);
  if (!(await expected.evaluate(node => node.open))) await expected.locator('summary').click();
  const expectedText = await expected.innerText();
  if (Array.isArray(row.expected)) {
    for (const line of row.expected) assert.ok(expectedText.includes(line), `${row.id}: 기대결과 ${line}`);
  } else assert.ok(expectedText.includes(row.expected.reject), `${row.id}: 거부 안내`);
}

async function selectProject(page, row) {
  await closeDiagnostic(page);
  const toggle = page.getByTestId('project-rail-toggle');
  if (await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
  const card = page.getByTestId(`project-card-first_steps_${row.id}`);
  assert.equal(await card.getAttribute('role'), 'option', `${row.id}: 프로젝트 선택 항목`);
  await card.click();
  await waitProject(page, row);
}

async function runRow(page, row, {source = sources.get(row.id), record = true} = {}) {
  const before = await snapshot(page);
  const runCount = before.runs.length;
  const generation = before.executionLifecycle.generation;
  const workerCount = workerUrls.length;
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(previousGeneration => {
    const state = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
    return state.executionLifecycle?.generation > previousGeneration && ['done', 'failed'].includes(state.executionLifecycle?.status);
  }, generation, {timeout: 30000});
  const state = await snapshot(page);
  const run = state.runs.at(-1);
  const diagnostics = JSON.stringify(state.diagnostics);
  assert.ok(workerUrls.length > workerCount, `${row.id}: 실행에 실제 Worker가 생성되지 않음`);
  if (Array.isArray(row.expected)) {
    assert.ok(state.runs.length > runCount, `${row.id}: 실제 run 없음`);
    assert.equal(state.executionLifecycle.status, 'done', `${row.id}: ${diagnostics}`);
    assert.equal(run.execution_mode, 'oneshot', `${row.id}: 수업은 한 번 실행`);
    assert.ok(state.activeFrame?.state_hash, `${row.id}: WASM committed frame 없음`);
    assert.equal(run.shared_runtime_result?.source_profile_identity, manifest.profile);
    assert.equal(run.shared_runtime_result?.source_identity, createHash('sha256').update(source).digest('hex'), `${row.id}: 실제 원문 결속`);
    assert.equal(run.shared_runtime_result?.state_hash, state.activeFrame.state_hash, `${row.id}: 영수증과 실제 세계`);
    assert.deepEqual(run.shared_runtime_result?.output_log_texts, row.expected, `${row.id}: 실제 WASM 출력`);
    const outputLog = state.activeFrame.wasm_state?.resources?.value_json?.output_log;
    assert.deepEqual(Array.isArray(outputLog) ? outputLog.map(line => String(line.text)) : [], row.expected, `${row.id}: 실제 frame 출력`);
    assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'), row.expected.join('\n'), `${row.id}: 화면 출력/잔류 제거`);
  } else {
    assert.equal(state.executionLifecycle.status, 'failed', `${row.id}: 오류 예제가 정상 실행됨`);
    assert.ok(diagnostics.includes(row.expected.reject), `${row.id}: 의도한 거부 ${diagnostics}`);
    assert.equal(state.activeFrame, null, `${row.id}: 실패에 결과 frame 게시`);
    if (state.runs.length === runCount) {
      assert.equal(state.sourceReadiness.authority_status, 'shared_frontend', `${row.id}: 제품 frontend 거부 근거 없음`);
      assert.equal(state.sourceReadiness.last_run_outcome.status, 'run_blocked', `${row.id}: 실행 시 명시거부 없음`);
    } else assert.equal(run.frame_count, 0, `${row.id}: 실패에 결과 frame 게시`);
    assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'), '', `${row.id}: 오류 뒤 이전 출력 잔류`);
  }
  const result = {id: row.id, source_sha256: createHash('sha256').update(source).digest('hex'),
    expected: row.expected, status: state.executionLifecycle.status,
    state_hash: state.activeFrame?.state_hash ?? null, worker_created: workerUrls.length - workerCount, result: 'PASS'};
  if (record) results.push(result);
  console.log(JSON.stringify(result));
}

try {
  if (!process.env.DDN_TEST_BASE_URL) await startOwnServer();
  browser = await chromium.launch({headless: true,
    executablePath: process.env.DDN_TEST_CHROME || 'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  context = await browser.newContext();
  context.on('response', response => responses.push({url: response.url(), status: response.status(), resourceType: response.request().resourceType()}));
  context.on('requestfailed', request => failedRequests.push({url: request.url(), error: request.failure()?.errorText, resourceType: request.resourceType()}));
  context.on('page', opened => {
    opened.on('pageerror', error => pageErrors.push(error.message));
    opened.on('worker', worker => workerUrls.push(worker.url()));
  });
  const page = await context.newPage();
  await page.goto(baseUrl);
  await page.waitForFunction(() => document.querySelector('#source-editor')?.value.length > 0);
  await page.getByTestId('global-first-steps').click();
  assert.equal(await page.getByTestId('first-steps-guide').evaluate(node => node.open), false, '안내 기본 접힘');
  for (const name of ['학생용_첫걸음_10차시.pdf', '교사용_안내와_해답.pdf']) {
    const response = await context.request.get(new URL('/first_steps/' + encodeURIComponent(name), baseUrl).href);
    assert.equal(response.status(), 200, name + ': PDF 링크');
    assert.ok(response.headers()['content-type'].includes('application/pdf'), name + ': PDF MIME');
    assert.equal((await response.body()).subarray(0, 5).toString(), '%PDF-', name + ': 실제 PDF');
  }
  let transitionChecked = false, recoveryChecked = false;
  for (const row of manifest.cases) {
    // 03D is opened through the actual next-step control from 03C.
    if (row.id !== '03D' || !transitionChecked) await selectProject(page, row);
    else await waitProject(page, row);
    await runRow(page, row);
    if (row.id === '03C') {
      await page.getByTestId('first-steps-next').click();
      await waitProject(page, manifest.cases.find(item => item.id === '03D'));
      await page.getByTestId('first-steps-previous').click();
      await waitProject(page, row);
      await page.getByTestId('first-steps-next').click();
      transitionChecked = true;
    }
    if (row.id === '07D') {
      const fix = manifest.corrections.find(item => item.id === 'FIX07D');
      await selectProject(page, fix);
      await runRow(page, fix);
      recoveryChecked = true;
    }
  }
  for (const row of [...manifest.corrections, ...manifest.modifications]) {
    if (row.id === 'FIX07D' && recoveryChecked) continue;
    await selectProject(page, row);
    await runRow(page, row);
  }
  assert.equal(results.length, 55);
  assert.equal(new Set(results.map(row => row.id)).size, 55);

  // Edit the authored 03C source into MOD03, save, damage the editor, reopen,
  // and execute the restored bytes. This catches course metadata loading the
  // original source over a student's saved revision.
  const original = manifest.cases.find(row => row.id === '03C');
  const modified = manifest.modifications.find(row => row.id === 'MOD03');
  await selectProject(page, original);
  await page.keyboard.press('Escape');
  assert.equal(await page.getByTestId('first-steps-panel').isVisible(), false, 'Escape로 안내 닫기');
  await page.locator('#source-editor').fill(sources.get(modified.id));
  await page.getByTestId('project-save').click();
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().persistence?.phase === 'saved');
  await page.locator('#source-editor').fill('임시 변경');
  await page.getByTestId('project-reopen').click();
  await page.waitForFunction(expected => document.querySelector('#source-editor')?.value === expected, sources.get(modified.id));
  assert.equal((await snapshot(page)).projectId, 'server_document_first_steps_03C', '수업에서 저장한 내 문서 재열기 identity');
  await runRow(page, modified, {record: false});
  await page.waitForLoadState('networkidle');
  const httpErrors = responses.filter(response => response.status >= 400);
  assert.deepEqual(httpErrors, [], '첫걸음 HTTP 오류 (Worker 포함)');
  assert.deepEqual(failedRequests, [], '첫걸음 요청 실패 (Worker 포함)');
  assert.deepEqual(pageErrors, [], '첫걸음 화면 오류');
  assert.ok(workerUrls.some(url => url.includes('/runtime/runtime_worker.js')), '제품 runtime Worker 없음');
  assert.ok(responses.some(response => new URL(response.url).pathname.endsWith('.wasm') && response.status === 200), '실제 WASM 로딩 없음');
  console.log(JSON.stringify({result: 'FIRST_STEPS_COURSE_BROWSER_PASS', activities: 40,
    normal: 35, rejected: 5, corrections: 5, modifications: 10, total: results.length,
    next_previous: transitionChecked, error_recovery: recoveryChecked, edit_save_reopen: true,
    worker_count: workerUrls.length, http_errors: 0, request_failed: 0, page_errors: 0}));
} finally {
  if (context) await context.close();
  if (browser) await browser.close();
  if (server && server.exitCode === null) server.kill();
}
