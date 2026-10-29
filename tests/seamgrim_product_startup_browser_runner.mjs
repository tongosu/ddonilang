// Already-running provided ZIP launcher; never substitutes a static server.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const [root,origin]=process.argv.slice(2);
assert.ok(root && origin && process.env.DDN_PLAYWRIGHT_MODULE);
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-product-startup-'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const base='solutions/seamgrim_ui_mvp/';
const expectedWasm=sha(fs.readFileSync(path.join(root,base+'ui/wasm/ddonirang_tool_bg.wasm')));
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const errors=[],menus=[],assets=[],wasm=[];
try {
  const page=await browser.newPage({viewport:{width:1500,height:1000}});
  page.on('pageerror',e=>errors.push(String(e)));
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  page.on('requestfailed',r=>errors.push(r.url()));
  page.on('response',r=>{if(r.status()>=400)errors.push(`HTTP ${r.status()} ${r.url()}`);});
  await page.goto(origin+'/');
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  const courses=['first_steps','first_steps_alpha2','first_steps_alpha3'];
  for(const course of courses) {
    const response=await page.request.get(origin+'/'+course+'/'+encodeURIComponent('검증목록.json'));
    assert.equal(response.status(),200);
    assert.equal(sha(await response.body()),sha(fs.readFileSync(path.join(root,base+course+'/검증목록.json'))));
    assets.push(course);
  }
  const alpha4Dir=path.join(root,base+'first_steps_alpha4');
  if(fs.existsSync(alpha4Dir)){
    const alpha4Files=fs.readdirSync(alpha4Dir).filter(name=>/\.(md|pdf)$/.test(name));
    assert.equal(alpha4Files.filter(name=>name.endsWith('.pdf')).length,2);
    for(const name of alpha4Files){
      const response=await page.request.get(origin+'/'+base+'first_steps_alpha4/'+encodeURIComponent(name));
      assert.equal(response.status(),200);
      assert.equal(sha(await response.body()),sha(fs.readFileSync(path.join(alpha4Dir,name))));
    }
    assets.push('first_steps_alpha4');
  }
  const response=await page.request.get(origin+'/'+base+'ui/wasm/ddonirang_tool_bg.wasm');
  assert.equal(response.status(),200);wasm.push(sha(await response.body()));
  assert.equal(wasm[0],expectedWasm);
  for(const [id,name] of [['rep_phys_projectile_xy_v1','parabola'],['rep_phys_pendulum_wave1_v1','pendulum'],['local_linear_slope_v1','linear']]) {
    await page.locator('[data-testid="project-rail-toggle"]').click();
    await page.locator(`#project-list [data-project-id="${id}"]`).click();
    const source=fs.readFileSync(path.join(root,base+`samples/alpha2_${name}_composition.ddn`),'utf8');
    await page.waitForFunction(s=>document.querySelector('#source-editor').value===s,source);
    if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')==='true')
      await page.locator('[data-testid="project-rail-toggle"]').click();
    await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false);
    const ok=await page.evaluate(async()=>{const a=window.__SEAMGRIM_APPROVED_WORKBENCH__;const ok=await a.startRun();a.pauseRun();return ok;});
    assert.equal(ok,true,JSON.stringify(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics)));
    const hash=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
    assert.ok(hash);menus.push({name,hash});
  }
  assert.deepEqual(errors,[]);
  const activities=[];
  if(process.env.DDN_TEST_EDUCATION==='1'){
    const manifest=JSON.parse(fs.readFileSync(path.join(root,base+'first_steps_alpha3/검증목록.json'),'utf8'));
    for(const row of manifest.cases.filter(row=>row.lesson>=21)){
      await page.locator('[data-testid="project-rail-toggle"]').click();
      await page.locator(`#project-list [data-project-id="first_steps_alpha3_${row.id}"]`).click();
      const source=fs.readFileSync(path.resolve(root,base+'first_steps_alpha3',row.file),'utf8');
      try{await page.waitForFunction(s=>document.querySelector('#source-editor').value===s,source);}
      catch(error){console.error(JSON.stringify({id:row.id,editor:await page.locator('#source-editor').inputValue(),snapshot:await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot())}));throw error;}
      if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')==='true')await page.locator('[data-testid="project-rail-toggle"]').click();
      for(const item of row.project_files??[]){
        const response=await page.request.get(origin+'/first_steps_alpha3/'+item.file);
        assert.equal(response.status(),200);assert.equal(sha(await response.body()),sha(fs.readFileSync(path.resolve(root,base+'first_steps_alpha3',item.file))));
      }
      if(row.id==='27A'){
        const menu=page.locator('[data-testid="project-more-menu"]');
        await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
        await page.locator('[data-authored-create]').click();
        await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('명시 작품 정체를 만들었습니다'));
        await page.locator('[data-authored-close]').click();
        const ok=await page.evaluate(async()=>{const a=window.__SEAMGRIM_APPROVED_WORKBENCH__;const ok=await a.startRun();a.pauseRun();return ok;});
        assert.equal(ok,true);
        const before=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
        assert.ok(before?.state_hash);
        await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
        await page.locator('[data-authored-runtime-save]').click();
        await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('실행 상태를 저장하고'));
        await page.locator('[data-authored-runtime-reopen]').click();
        await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('실행 상태를 복원'));
        assert.equal(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash),before.state_hash);
        await page.locator('[data-authored-close]').click();
      }
      activities.push({id:row.id,opened:true,explicit_project_save_restore:row.id==='27A'});
    }
    for(const name of ['학생용_첫걸음_21-28차시.pdf','교사용_첫걸음_21-28차시.pdf']){
      const response=await page.request.get(origin+'/first_steps_alpha3/'+encodeURIComponent(name));
      assert.equal(response.status(),200);assert.equal(sha(await response.body()),sha(fs.readFileSync(path.join(root,base+'first_steps_alpha3',name))));
    }
    assert.deepEqual(errors,[]);
  }
  await page.screenshot({path:path.join(evidence,'ready.png'),fullPage:true});
  const report={status:'PASS_PRODUCT_LAUNCHER_STARTUP',root,origin,evidence,assets,menus,activities,wasm_sha256:expectedWasm,errors,independent_review:'NOT_CLAIMED',human:'NOT_RUN'};
  fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(report,null,2));
  console.log(JSON.stringify(report));
} finally {await browser.close();}
