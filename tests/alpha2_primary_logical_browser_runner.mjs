// WO14-A2-UI: actual top-level playback controls, real Worker/WASM and records.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE);
assert.ok(process.env.DDN_NATIVE_DIR);
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve('.'), output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-primary-logical-'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const errors=[],requests=[];
const server=http.createServer((req,res)=>{
  const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const file=path.resolve(root,relative);
  if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  try {
    const bytes=fs.readFileSync(file),type=file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':'application/json';
    res.writeHead(200,{'content-type':type}).end(bytes);
  } catch {res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const page=await browser.newPage({viewport:{width:1400,height:1050}});
page.on('pageerror',error=>errors.push(String(error)));
page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
page.on('requestfailed',request=>requests.push(request.url()));
page.on('response',response=>{if(response.status()>=400)requests.push(`${response.status()} ${response.url()}`);});
const state=()=>page.evaluate(()=>JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent));
const waitMadi=n=>page.waitForFunction(n=>{try {const v=JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent);return (v.frame?.input?.madi ?? v.madi_count)>=n;}catch{return false;}},n);
try {
  await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  const panel=page.locator('#logical-input-panel'), primary=page.locator('[data-testid="lifecycle-run"]');
  const rail=page.locator('[data-testid="project-rail-toggle"]');
  if(await rail.getAttribute('aria-expanded')!=='true')await rail.click();
  await page.locator('[data-global-entry="first-steps"]').click();
  assert.equal(await page.locator('[data-testid^="project-card-first_steps_alpha2_"]').count(),18,'six lessons with authored and modified examples');
  await page.locator('[data-testid="project-card-first_steps_alpha2_11A"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  await primary.click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.bogae_projection?.status==='ready');
  await page.locator('[data-testid="first-steps-open"]').click();
  const studentPdf=await page.locator('.first-steps-documents a').first().getAttribute('href');
  assert.ok(studentPdf.includes('first_steps_alpha2/'));
  assert.equal((await page.request.get(new URL(studentPdf,page.url()).href)).status(),200);
  await page.locator('[data-testid="project-card-first_steps_alpha2_15A"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  if(!(await page.locator('[data-testid="first-steps-guide"]').evaluate(node=>node.open)))await page.locator('[data-testid="first-steps-open"]').click();
  await page.locator('[data-testid="first-steps-input-contract"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('현재 원문에 결속했습니다'));
  await primary.click();await waitMadi(1);await primary.click();
  await page.waitForFunction(()=>{const button=document.querySelector('[data-testid="lifecycle-run"]');return button.dataset.action==='resume' && !button.disabled;});
  assert.ok((await state()).bogae_projection,'education source uses actual logical Rust session');
  await page.locator('[data-testid="lifecycle-stop"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('실행을 닫았습니다'));
  await page.locator('[data-global-entry="official-collection"]').click();
  await page.locator('[data-testid="project-card-alpha2_logical_tetris"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="binding"]').value.includes('source_revision') && !document.querySelector('[data-testid="lifecycle-run"]').disabled);
  assert.equal(await panel.evaluate(node=>node.open),false,'official entry requires no lower contract panel');
  assert.equal(await page.locator('[data-testid="project-verification-open"]').textContent(),'검증','diagnostics from previous project must not leak');
  await page.locator('[data-testid="project-verification-open"]').click();
  assert.ok((await page.locator('[data-testid="diagnostic-list"]').textContent()).includes('문제가 발견되지 않았습니다.'));
  await page.keyboard.press('Escape');
  assert.equal(await panel.locator('[data-action="start"]').isVisible(),false);
  assert.equal(await panel.locator('[data-action="play"]').isVisible(),false);
  await primary.click(); await waitMadi(1);
  assert.equal(await primary.getAttribute('data-action'),'pause');
  await primary.click();
  await page.waitForFunction(()=>{const button=document.querySelector('[data-testid="lifecycle-run"]');return button.dataset.action==='resume' && !button.disabled;});
  const before=await state();
  assert.equal(await page.locator('[data-testid="current-madi"]').textContent(),`마디: ${before.frame.input.madi}`);
  assert.equal(await page.locator('[data-testid="project-verification-open"]').textContent(),'검증');
  await page.locator('.bogae-panel').screenshot({path:path.join(output,'official-tetris.png')});
  await page.waitForTimeout(90);assert.deepEqual(await state(),before,'top-level pause stops committed madis');
  const mainCanvas=page.locator('[data-testid="bogae-canvas"]');
  assert.equal(await mainCanvas.getAttribute('data-committed-state-hash'),before.state_hash ?? before.frame.state_hash,'main primary presents the actual committed world');
  assert.equal(await panel.locator('[data-field="canvas"]').isVisible(),false,'no duplicate lower world canvas');
  await mainCanvas.focus();
  await page.keyboard.press('ArrowLeft');
  await page.locator('[data-testid="lifecycle-step"]').click();
  await waitMadi(before.frame.input.madi+1);
  const moved=await state();
  assert.equal(Number(moved.resources.fixed64.가로),Number(before.resources.fixed64.가로)-1,'key input changes the actual Rust world');
  await primary.click();await waitMadi(moved.frame.input.madi+1);await primary.click();
  await page.waitForFunction(()=>{const button=document.querySelector('[data-testid="lifecycle-run"]');return button.dataset.action==='resume' && !button.disabled;});
  await panel.locator('summary').first().click();
  const download=page.waitForEvent('download');await panel.locator('[data-action="seal"]').click();
  const tape=path.join(output,'actual-input.tape');await (await download).saveAs(tape);
  const expected=await state(),source=await page.locator('#source-editor').inputValue(),binding=await panel.locator('[data-field="binding"]').inputValue();
  const sourceFile=path.join(output,'source.ddn'),bindingFile=path.join(output,'binding.json');
  fs.writeFileSync(sourceFile,source);fs.writeFileSync(bindingFile,binding);
  for(const name of ['ddonirang','ddonilang','또니랑']) {
    const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['논리입력재생',sourceFile,tape,bindingFile],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
    assert.equal(result.status,0,result.stderr||String(result.error));assert.deepEqual(JSON.parse(result.stdout),expected);
  }
  await page.locator('[data-testid="lifecycle-stop"]').click();
  await panel.locator('[data-field="replay"]').setInputFiles(tape);
  await page.waitForFunction(count=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count===count;}catch{return false;}},expected.madi_count);
  assert.deepEqual(await state(),expected);
  await page.screenshot({path:path.join(output,'primary-playback.png'),fullPage:true});
  // An editor remains outside the explicitly admitted controls.
  await primary.click();
  await page.waitForFunction(()=>{const button=document.querySelector('[data-testid="lifecycle-run"]');return button.dataset.action==='pause' && !button.disabled;});
  await waitMadi(1);await page.locator('#source-editor').focus();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_LOST'));
  assert.deepEqual(errors,[]);assert.deepEqual(requests,[]);
  const report={result:'PASS_PRIMARY_LOGICAL_BROWSER',evidence:output,wasm_sha256:sha(fs.readFileSync('solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm')),tape_sha256:sha(fs.readFileSync(tape)),madi_count:expected.madi_count,native_aliases:3,errors,requests};
  fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
} catch(error) {
  console.error('PRIMARY_LOGICAL_EVIDENCE',output);
  console.error('PRIMARY_LOGICAL_STATUS',await page.locator('#logical-input-panel [data-field="feedback"]').textContent());
  await page.screenshot({path:path.join(output,'failure.png'),fullPage:true});throw error;
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
