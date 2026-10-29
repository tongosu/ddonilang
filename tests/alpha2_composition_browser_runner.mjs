import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

assert.ok(process.env.DDN_PLAYWRIGHT_MODULE,'cached Playwright module required');
assert.ok(process.env.DDN_WASM_DIR,'source-built WASM directory required');
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve('.'),wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const servedWasm=[];
const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha2-composition-browser-'));
const server=http.createServer((req,res)=>{
  const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const local=path.resolve(root,relative);
  if(!local.startsWith(root+path.sep)) {res.writeHead(403).end();return;}
  const file=relative.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')?path.join(wasmDir,path.basename(local)):local;
  try {
    const bytes=fs.readFileSync(file);
    if(file.endsWith('ddonirang_tool_bg.wasm'))servedWasm.push({url:req.url,sha256:sha(bytes)});
    const type=file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':file.endsWith('.ddn')?'text/plain':'application/json';
    res.writeHead(200,{'content-type':type}).end(bytes);
  } catch {res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const page=await browser.newPage({viewport:{width:1600,height:1100}});
const pageerrors=[],consoleErrors=[],requests=[],httpFailures=[],loadedWasm=[],sampleRequests=[];
page.on('pageerror',error=>pageerrors.push(String(error)));
page.on('console',message=>{if(message.type()==='error')consoleErrors.push(message.text());});
page.on('requestfailed',request=>requests.push({url:request.url(),error:request.failure()?.errorText}));
page.on('response',response=>{
  if(response.status()>=400)httpFailures.push({url:response.url(),status:response.status()});
  if(new URL(response.url()).pathname.endsWith('/ddonirang_tool_bg.wasm'))loadedWasm.push(response);
  if(/\/samples\/alpha2_(parabola|pendulum|linear)_composition\.ddn$/.test(response.url()))sampleRequests.push({url:response.url(),status:response.status()});
});
const frame=()=>page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
const settle=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
const results=[];
try {
  await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await page.evaluate(()=>{
    localStorage.setItem('seamgrim.workbench.my_work_index.v1',JSON.stringify([
      {project_id:'old-parabola-test',title:'포물선운동2',origin_kind:'local_only',head_revision_id:'preserve-me'},
      {project_id:'other-work-test',title:'다른 작업',origin_kind:'local_only'}
    ]));
  });
  await page.reload();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await page.locator('[data-testid="global-my-work"]').click();
  if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')!=='true')await page.locator('[data-testid="project-rail-toggle"]').click();
  const oldWork=page.locator('[data-testid="my-work-item"][data-project-id="old-parabola-test"]');
  assert.equal(await oldWork.count(),1);
  await page.locator('[data-my-work-visibility="index"][data-project-id="old-parabola-test"]').click();
  assert.equal(await oldWork.count(),0);
  assert.equal(await page.locator('[data-testid="my-work-item"][data-project-id="other-work-test"]').count(),1);
  const saved=await page.evaluate(()=>JSON.parse(localStorage.getItem('seamgrim.workbench.my_work_index.v1')));
  assert.equal(saved.find(row=>row.project_id==='old-parabola-test').head_revision_id,'preserve-me');
  assert.equal(saved.length,2,'hiding must not delete any stored record');
  await page.reload();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await page.locator('[data-testid="global-my-work"]').click();
  if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')!=='true')await page.locator('[data-testid="project-rail-toggle"]').click();
  assert.equal(await oldWork.count(),0,'hidden state persists across reopening');
  await page.locator('[data-my-work-visibility="toggle"]').click();
  assert.equal(await oldWork.count(),1);
  await page.locator('[data-my-work-visibility="index"][data-project-id="old-parabola-test"]').click();
  assert.equal(await oldWork.count(),1,'explicit restore retains the work');
  await page.evaluate(()=>localStorage.removeItem('seamgrim.workbench.my_work_index.v1'));
  for(const name of ['parabola','pendulum','linear']) {
    await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
    const source=fs.readFileSync(`solutions/seamgrim_ui_mvp/samples/alpha2_${name}_composition.ddn`,'utf8');
    const menu={parabola:['rep_phys_projectile_xy_v1','포물선 운동'],pendulum:['rep_phys_pendulum_wave1_v1','진자 보개짜임'],linear:['local_linear_slope_v1','일차함수 기울기 작업']}[name];
    await page.locator('[data-testid="project-rail-toggle"]').click();
    await page.locator(`#project-list [data-project-id="${menu[0]}"]`).click();
    await page.waitForFunction(({id,text})=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId===id
      && document.querySelector('#source-editor').value===text,{id:menu[0],text:source});
    assert.equal(await page.locator('#project-title').textContent(),menu[1]);
    assert.ok(sampleRequests.some(row=>row.url.endsWith(`/samples/alpha2_${name}_composition.ddn`) && row.status===200));
    if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')==='true')await page.locator('[data-testid="project-rail-toggle"]').click();
    // Keep the direct exact-source editor ingress as well as menu loading.
    await page.locator('#source-editor').fill(source);
    await page.keyboard.press('Escape');
    await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false
      && window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().readiness.source.ready===true);
    await page.locator('[data-testid="lifecycle-run"]').focus();
    // Same production commands as the Run/Pause controls. Await preparation,
    // then pause before its auto-pump timer: exactly one real Rust/WASM madi.
    await page.evaluate(async()=>{
      const workbench=window.__SEAMGRIM_APPROVED_WORKBENCH__;
      await workbench.startRun();workbench.pauseRun();
    });
    await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.authoredBogaeStatus==='ready');
    const snapshot=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    const run=snapshot.runs.find(row=>row.run_id===snapshot.activeRunId);
    assert.equal(run.committed_step_count,1,'one actual madi, no fabricated JS frame');
    assert.equal(run.execution_mode,name==='linear'?'oneshot':'live');
    assert.equal(await page.locator('#source-editor').inputValue(),source,'source bytes remain exact');
    assert.equal(snapshot.semanticRenderer.data_source,'rust_authored_bogae_projection');
    const committed=snapshot.activeFrame;
    const projection=committed.wasm_state.bogae_projection;
    assert.equal(projection.status,'ready');assert.equal(projection.candidates.length,1);
    const relations=projection.candidates[0].artifact.relations;
    const expected=name==='linear'?['primary','overlay','side',...Array(21).fill('magnifier')]:name==='pendulum'?['primary','overlay','side','side','magnifier','magnifier']:['primary','overlay','side','side','magnifier'];
    assert.deepEqual(relations.map(row=>row.kind),expected);
    const sides=relations.filter(row=>row.kind==='side');
    for (const relation of relations.filter(row=>['side','overlay'].includes(row.kind))) {
      assert.ok(relation.authored_name,'Rust supplies the exact authored binding label');
      assert.ok((await page.locator(`[data-authored-relation-toggle="${relation.relation_id}"]`).textContent()).includes(relation.authored_name));
    }
    for (const mode of ['ddn-main','bogae-main','bogae-full']) {
      await page.locator(`[data-testid="mode-${mode}"]`).click();
      await settle();
      const toolbar=await page.locator('.bogae-panel > .panel-header .authored-view-toolbar').boundingBox();
      assert.ok(toolbar.height<75,`${mode}: controls must not consume the expanding canvas row`);
      const header=await page.locator('.bogae-panel > .panel-header').boundingBox();
      assert.ok(toolbar.y+toolbar.height<=header.y+header.height+1,'controls stay in the compact title row');
      await page.locator('.bogae-panel').screenshot({path:path.join(output,`${name}-${mode}-compact.png`)});
      const side=sides[0],panel=page.locator(`[data-authored-relation-id="${side.relation_id}"]`);
      await panel.locator('[data-authored-side-close]').click({delay:120});
      assert.equal(await panel.isVisible(),false,`${mode}: individual Close hides the retained side`);
      await page.locator(`[data-authored-relation-toggle="${side.relation_id}"]`).click();
      assert.equal(await panel.isVisible(),true);
    }
    await page.locator('[data-testid="mode-ddn-main"]').click();
    assert.equal(await page.locator('[data-authored-relation-id]').count(),sides.length);
    assert.equal(await page.locator('[data-authored-dock="right"] [data-authored-relation-id]').count(),sides.length);
    const primary=page.locator('#canvas-bogae');
    const canvasMetrics=()=>primary.evaluate(canvas=>{const rect=canvas.getBoundingClientRect();return {
      intrinsic_width:canvas.width,intrinsic_height:canvas.height,display_width:rect.width,display_height:rect.height,
      authored_digest:canvas.dataset.authoredBogaeDigest};});
    await settle();
    const initialPixels=await primary.evaluate(canvas=>canvas.toDataURL());
    const layout={before:await canvasMetrics()};
    await primary.screenshot({path:path.join(output,`${name}-overlay-before.png`)});
    const overlay=page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]');
    await overlay.click();
    await settle();
    layout.off=await canvasMetrics();
    assert.equal(await overlay.getAttribute('aria-pressed'),'false');
    assert.notEqual(await primary.evaluate(canvas=>canvas.toDataURL()),initialPixels,'overlay must change real canvas pixels');
    assert.deepEqual(await frame(),committed);
    await primary.screenshot({path:path.join(output,`${name}-overlay-off.png`)});
    await overlay.click();
    await settle();
    layout.on=await canvasMetrics();
    assert.equal(await overlay.getAttribute('aria-pressed'),'true');
    await primary.screenshot({path:path.join(output,`${name}-overlay-on.png`)});
    fs.writeFileSync(path.join(output,`${name}-overlay-layout.json`),JSON.stringify(layout,null,2));
    assert.equal(sha(await primary.evaluate(canvas=>canvas.toDataURL())),sha(initialPixels),'overlay reopening must restore actual pixels after layout settles');
    for(const side of sides) {
      const id=side.relation_id,panel=page.locator(`[data-authored-relation-id="${id}"]`);
      assert.equal(await panel.isVisible(),true);
      await page.evaluate(id=>{window.retainedSide=document.querySelector(`[data-authored-relation-id="${id}"]`);},id);
      const pixels=await panel.locator('canvas').evaluate(canvas=>canvas.toDataURL());
      await panel.locator('[data-authored-side-close]').click();
      assert.equal(await panel.isVisible(),false);
      for(const other of sides.filter(row=>row!==side))assert.equal(await page.locator(`[data-authored-relation-id="${other.relation_id}"]`).isVisible(),true);
      assert.deepEqual(await frame(),committed);
      await page.locator(`[data-authored-relation-toggle="${id}"]`).click();
      assert.equal(await panel.isVisible(),true);
      assert.equal(await page.evaluate(id=>window.retainedSide===document.querySelector(`[data-authored-relation-id="${id}"]`),id),true);
      assert.equal(sha(await panel.locator('canvas').evaluate(canvas=>canvas.toDataURL())),sha(pixels),'side reopening must restore actual pixels');
      assert.deepEqual(await frame(),committed);
    }
    await page.locator('[data-testid="authored-close-sides"]').click();
    for(const side of sides) assert.equal(await page.locator(`[data-authored-relation-id="${side.relation_id}"]`).isVisible(),false);
    assert.deepEqual(await frame(),committed,'all sides close is local only');
    for(const side of sides) await page.locator(`[data-authored-relation-toggle="${side.relation_id}"]`).click();
    assert.equal(await page.locator('[data-testid="view-role-primary"]').isVisible(),false);
    assert.equal(await page.locator('[data-testid="bogae-entry"]').isVisible(),false,'one view needs no selector or explanatory banner');
    assert.equal(await page.locator('[data-testid="bogae-source-access"]').isVisible(),false);
    assert.equal(await page.locator('[data-testid="spatial-fit"]').isVisible(),false);
    const magnifiers=relations.filter(row=>row.kind==='magnifier');
    const magnifier=magnifiers[0];
    assert.equal(magnifier.recipe.kind,'sealed_named_callable');
    assert.equal(await page.locator('[data-authored-magnifier-id]').count(),0);
    const pick=async relation=>{
      // Mouse event in real Chrome. Geometry comes from the committed Rust
      // primary, not a synthetic replacement frame or JS recipe evaluation.
      const position=await primary.evaluate((canvas,relation)=>{
        const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.wasm_state;
        const view=state.bogae_projection.candidates[0].artifact.view;
        const circle=view.elements[relation.selection.element_indices[0]];
        const [xmin,xmax]=view.horizontal.map(BigInt),[ymin,ymax]=view.vertical.map(BigInt);
        const sx=Number(xmax-xmin),sy=Number(ymax-ymin),s=Math.min((canvas.width-36)/sx,(canvas.height-36)/sy);
        const rect=canvas.getBoundingClientRect();
        return {x:rect.left+((canvas.width-sx*s)/2+Number(BigInt(circle.center[0])-xmin)*s)*rect.width/canvas.width,
          y:rect.top+((canvas.height+sy*s)/2-Number(BigInt(circle.center[1])-ymin)*s)*rect.height/canvas.height};
      },relation);
      await page.mouse.click(position.x,position.y);await settle();
    };
    await pick(magnifier);
    const instance=page.locator('[data-authored-magnifier-id]');
    await instance.locator('canvas').waitFor();
    assert.equal(await instance.count(),1);
    assert.equal(await instance.evaluate(node=>node.parentElement.classList.contains('canvas-wrap')),true);
    assert.equal(await page.locator('[data-authored-dock] [data-authored-magnifier-id]').count(),0,'magnifier must not create a side panel');
    assert.deepEqual(JSON.parse(await instance.locator('[data-authored-magnifier-target]').textContent()),magnifier.target);
    await page.evaluate(()=>{window.firstMagnifier=document.querySelector('[data-authored-magnifier-id]');});
    assert.deepEqual(await frame(),committed);
    await instance.screenshot({path:path.join(output,`${name}-magnifier.png`)});
    if(magnifiers.length>1) {
      await pick(magnifiers.at(-1));
      assert.equal(await instance.count(),1,'switching target closes old instance before opening new');
      assert.equal(await page.evaluate(()=>window.firstMagnifier.isConnected),false);
      assert.deepEqual(JSON.parse(await instance.locator('[data-authored-magnifier-target]').textContent()),magnifiers.at(-1).target);
      assert.deepEqual(await frame(),committed,'picking must not advance madi or mutate world/input/receipt');
    }
    await instance.locator('[data-authored-magnifier-close]').click();
    assert.equal(await page.locator('[data-authored-magnifier-id]').count(),0);
    assert.deepEqual(await frame(),committed);
    await pick(magnifier);
    assert.equal(await page.evaluate(()=>window.firstMagnifier===document.querySelector('[data-authored-magnifier-id]')),false);
    assert.deepEqual(await frame(),committed);
    await page.locator('[data-testid="bogae-stage"]').screenshot({path:path.join(output,`${name}-composition.png`)});
    if(name==='pendulum') {
      await page.evaluate(()=>{window.liveMagnifier=document.querySelector('[data-authored-magnifier-id]');});
      const starting=await frame();
      await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.resumeRun());
      await page.waitForFunction(hash=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash!==hash,starting.state_hash);
      assert.equal(await page.evaluate(()=>window.liveMagnifier===document.querySelector('[data-authored-magnifier-id]')),true,
        'moving bob retains the same magnifier instance');
      assert.notDeepEqual(JSON.parse(await instance.locator('[data-authored-magnifier-target]').textContent()),magnifier.target);
      assert.equal(await instance.evaluate(node=>node.parentElement.classList.contains('canvas-wrap')),true,'detail is on the primary, not a side dock');
      const liveSide=page.locator('[data-authored-side-close]').first();
      const sideId=await liveSide.getAttribute('data-authored-side-close');
      await liveSide.click({delay:220});
      const advanced=await frame();await page.waitForTimeout(200);
      assert.notEqual((await frame()).state_hash,advanced.state_hash,'close does not pause playback');
      assert.equal(await page.locator(`[data-authored-relation-id="${sideId}"]`).isVisible(),false,'closed side stays hidden during playback');
      await page.locator(`[data-authored-relation-toggle="${sideId}"]`).click();
      assert.equal(await page.locator(`[data-authored-relation-id="${sideId}"]`).isVisible(),true);
      // The fixed pivot remains hit-testable while the bob moves; selecting it replaces the bob.
      await pick(magnifiers.at(-1));
      await instance.waitFor();
      assert.equal(await page.evaluate(()=>window.liveMagnifier.isConnected),false);
      const running=await frame();await page.waitForTimeout(200);
      assert.notEqual((await frame()).state_hash,running.state_hash,'live target selection never auto-pauses');
      assert.equal(await instance.count(),1,'picked information remains visible during playback');
      await instance.locator('[data-authored-magnifier-close]').click();
      await page.waitForTimeout(100);assert.equal(await instance.count(),0,'close terminates while playback continues');
      await pick(magnifier);
      assert.equal(await instance.getAttribute('data-authored-magnifier-id'),magnifier.relation_id,
        'the moving bob must also be clickable during playback, not only the fixed pivot');
      const liveObject=await instance.getAttribute('data-object-identity');
      const liveBefore=await frame();await page.waitForTimeout(150);
      assert.notEqual((await frame()).state_hash,liveBefore.state_hash);
      assert.equal(await instance.getAttribute('data-object-identity'),liveObject);
      await page.locator('[data-testid="bogae-stage"]').screenshot({path:path.join(output,'pendulum-live-magnifier.png')});
      await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.pauseRun());
    }
    assert.deepEqual((await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics)).filter(row=>row.severity==='error'),[]);
    results.push({name,source_sha256:sha(source),state_hash:committed.state_hash,relation_kinds:expected,committed_step_count:run.committed_step_count,execution_mode:run.execution_mode,menu_project_id:menu[0],menu_title:menu[1],menu_exact_source:true,direct_editor_exact_source:true});
  }
  assert.ok(loadedWasm.length>=1,'browser must request the actual WASM bytes');
  assert.ok(servedWasm.length>=1,'owned server must serve the requested binary');
  for(const served of servedWasm)assert.equal(served.sha256,sha(wasmBytes));
  assert.deepEqual(pageerrors,[]);assert.deepEqual(consoleErrors,[]);assert.deepEqual(requests,[]);assert.deepEqual(httpFailures,[]);
  const report={result:'ALPHA2_COMPOSITION_BROWSER_PASS',actual_chrome_worker_wasm:true,
    wasm_sha256:sha(wasmBytes),served_wasm:servedWasm,evidence_directory:output,results,sample_requests:sampleRequests,pageerrors:0,console_errors:0,request_failures:0,http_failures:0,
    pendulum_model:'small-angle approximation; no alpha1 nonlinear-state equivalence claim',
    scope:'Actual primary clicks, object-following detail on primary, live pose updates and target replacement, live side close/reopen, no automatic pause; independent review/Human/whole Alpha2 closure not claimed'};
  fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(report,null,2));
  console.log(JSON.stringify(report));
} catch(error) {
  console.error('COMPOSITION_BROWSER_ERROR',error);
  console.error('COMPOSITION_BROWSER_DIAGNOSTICS',JSON.stringify(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot()?.diagnostics)));
  await page.screenshot({path:path.join(output,'failure.png'),fullPage:true});
  console.error('COMPOSITION_BROWSER_EVIDENCE',output);throw error;
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
