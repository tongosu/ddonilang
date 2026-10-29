import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import crypto from 'node:crypto';
import { chromium } from 'playwright';

// Compare only sample-resource membership; run the same real source/WASM/UI.
const root = path.resolve(process.env.DDN_TEST_ROOT || '.');
const uiPath = '/solutions/seamgrim_ui_mvp/ui/workbench/index.html';
const jsPath = '/solutions/seamgrim_ui_mvp/ui/workbench/workbench.js';
const current = await fs.readFile(path.join(root, jsPath), 'utf8');
const anchor = '        path: "meta.toml",';
assert.equal(current.split(anchor).length, 2);
assert.ok(!current.includes('path: "space2d.json"'));
const baseline = current.replace('      {\n' + anchor, [
  '      { path: "space2d.json", role: "view", url: "../../lessons/rep_grid_game_state_drop_v1/space2d.json" },',
  '      { path: "table.json", role: "observation", url: "../../lessons/rep_grid_game_state_drop_v1/table.json" },',
  '      {\n' + anchor,
].join('\n'));
assert.notEqual(current, baseline);
const server = http.createServer(async (req, res) => {
  try {
    const file = path.resolve(root, '.' + new URL(req.url, 'http://localhost').pathname);
    if (!file.startsWith(root + path.sep)) { res.writeHead(403); res.end(); return; }
    const bytes = await fs.readFile(file);
    const ext = path.extname(file);
    res.setHeader('Content-Type', ({'.js':'text/javascript', '.html':'text/html', '.wasm':'application/wasm', '.json':'application/json', '.css':'text/css'})[ext] || 'text/plain');
    res.end(bytes);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const url = `http://127.0.0.1:${server.address().port}${uiPath}`;
const browser = await chromium.launch({headless:true, executablePath: process.env.CHROME_EXECUTABLE || 'C:/Program Files/Google/Chrome/Application/chrome.exe'});
try {
  const results = [];
  for (const code of [baseline, current]) {
    const context = await browser.newContext({viewport:{width:1360,height:860}});
    await context.route('**/workbench.js*', route => route.fulfill({status:200, contentType:'text/javascript', body:code}));
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    await page.goto(url);
    await page.waitForSelector('#source-editor');
    await page.getByRole('button',{name:'프로젝트',exact:true}).click();
    await page.getByRole('option',{name:/격자 상태와 낙하 조각/}).click();
    await page.waitForSelector('button[data-action-id="left"]').catch(async error => {
      console.error('grid selection diagnostic', await page.evaluate(() => ({
        snapshot: window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot?.(),
        actions: document.querySelector('[data-testid="action-controls"]')?.outerHTML,
      })));
      throw error;
    });
    await page.locator('#canvas-bogae').click();
    await page.keyboard.press('ArrowLeft');
    await page.waitForFunction(() => document.querySelector('[data-testid="action-controls"]')?.innerText.includes('다음 마디 입력: 왼쪽'), {timeout:3000});
    await page.evaluate(() => {
      window.gridFrames = {};
      const canvas = document.querySelector('#canvas-bogae');
      new MutationObserver(() => {
        const madi = Number(canvas.dataset.committedMadi);
        if (madi >= 1 && madi <= 3 && !window.gridFrames[madi]) {
          window.gridFrames[madi] = {
            state:canvas.dataset.committedStateHash,
            pixels:canvas.toDataURL(),
            output:canvas.dataset.consoleOutput || '',
            observations:document.querySelector('[data-testid="semantic-observations"]')?.innerText || '',
          };
        }
      }).observe(canvas,{attributes:true});
    });
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(() => Object.keys(window.gridFrames).length === 3).catch(async error => {
      console.error(await page.evaluate(() => ({frames:window.gridFrames,canvas:document.querySelector('#canvas-bogae')?.dataset,
        diagnostic:document.querySelector('[data-testid="project-verification-drawer"]')?.innerText,
        status:document.querySelector('[data-testid="lifecycle-run"]')?.outerHTML})));
      throw error;
    });
    await page.getByTestId('lifecycle-run').click();
    const result = await page.evaluate(() => ({
      frames:window.gridFrames,
      actions:[...document.querySelectorAll('button[data-action-id]')].map(b=>[b.dataset.actionId,b.textContent]),
      title:document.title,
    }));
    assert.deepEqual(errors, []);
    results.push(result);
    // The shipped grid does not read input; prove the input reaches actual
    // WASM separately, rather than letting JS synthesize a visible result.
    await page.getByTestId('lifecycle-stop').click();
    await page.locator('#source-editor').fill('(매마디)마다 { 입력키 보여주기. }.');
    await page.waitForFunction(() => document.querySelector('[data-testid="approved-workbench-root"]')?.dataset.projectControlStatus !== 'loading');
    await page.locator('#canvas-bogae').click();
    await page.keyboard.press('ArrowRight');
    await page.waitForFunction(() => document.querySelector('[data-testid="action-status"]')?.innerText.includes('오른쪽'));
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(() => document.querySelector('#canvas-bogae')?.dataset.consoleOutput?.includes('ArrowRight')).catch(async error => {
      console.error(await page.evaluate(() => ({canvas:document.querySelector('#canvas-bogae')?.dataset,
        diagnostics:document.querySelector('[data-testid="project-verification-drawer"]')?.innerText,
        actions:document.querySelector('[data-testid="action-status"]')?.innerText})));
      throw error;
    });
    await context.close();
  }
  assert.deepEqual(results[1], results[0], 'Removing sample resources changed actual frames/output/input');
  console.log(JSON.stringify({result:'PASS',frames:3,input:'left',before_after_equal:true,frame_sha256:crypto.createHash('sha256').update(JSON.stringify(results[1])).digest('hex')}));
} finally {
  await browser.close();
  await new Promise(resolve => server.close(resolve));
}
