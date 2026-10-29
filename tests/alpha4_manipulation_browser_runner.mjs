import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE && process.env.DDN_NATIVE_DIR);
const checkCompactExperiment=process.env.DDN_EXPERIMENT_COMPACT_CHECK==='1';
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
  const root=path.resolve(process.env.DDN_TEST_ROOT || '.'),output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha4-manipulation-'));
const errors=[],sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const server=http.createServer((req,res)=>{
  const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const file=path.resolve(root,rel);
  if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  try{res.writeHead(200,{'content-type':file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':'application/json'}).end(fs.readFileSync(file));}
  catch{res.writeHead(404).end();}
});
// A provided launcher already owns the tested HTTP surface. Do not start an
// unused second server (the host's low ephemeral range includes owner 8790).
if(!process.env.DDN_TEST_ORIGIN) {
  let listening=false;
  for(let attempt=0;attempt<64&&!listening;attempt++) {
    listening=await new Promise((resolve,reject)=>{
      const ready=()=>{server.off('error',failed);resolve(true);};
      const failed=error=>{server.off('listening',ready);if(['EADDRINUSE','EACCES'].includes(error.code))resolve(false);else reject(error);};
      server.once('error',failed);server.once('listening',ready);
      server.listen(20000+Math.floor(Math.random()*40000),'127.0.0.1');
    });
  }
  assert.ok(listening,'private test port unavailable; owner servers were not changed');
}
const origin=process.env.DDN_TEST_ORIGIN || `http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const page=await browser.newPage({viewport:{width:1600,height:1100},acceptDownloads:true});
page.on('pageerror',error=>errors.push(String(error)));page.on('requestfailed',request=>errors.push(request.url()));
page.on('response',response=>{if(response.status()>=400)errors.push(response.url());});
const panel=page.locator('#logical-input-panel');
const controls=page.locator('[data-testid="logical-manipulation-controls"]');
const state=()=>panel.locator('[data-field="result"]').textContent().then(JSON.parse);
try{
  await page.goto(process.env.DDN_TEST_START_URL ?? (process.env.DDN_TEST_ORIGIN ? `${origin}/` : `${origin}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`));
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await page.locator('[data-testid="global-official-collection"]').click();
  if(!(await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').isVisible()))await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  assert.ok(await controls.locator('input[data-manipulation-channel="반지름"]').isVisible());
  // Switching before the first run must also discard the previous contract.
  await page.locator('[data-testid="global-official-collection"]').click();
  await page.locator('[data-testid="project-card-alpha4_lrl_lab"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  assert.equal(await controls.locator('input,canvas').count(),0);
  assert.equal(await panel.locator('[data-field="binding"]').inputValue(),'');
  await page.locator('[data-testid="global-official-collection"]').click();
  await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi>=1;}catch{return false;}});
  assert.equal(await panel.evaluate(node=>node.open),false,'live controls do not require the record drawer');
  for(const viewport of [{width:1366,height:768},{width:1920,height:1080}]) {
    await page.setViewportSize(viewport);
    await page.waitForTimeout(100);
    const bounds=await page.locator('[data-testid="bogae-canvas-wrap"]').boundingBox();
    assert.ok(Math.abs(bounds.width/bounds.height-4/3)<0.02,'default primary is 4:3 at both desktop sizes');
    assert.equal(await page.locator('[data-testid="logical-record-drawer"]').isVisible(),false);
    assert.ok(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1),'desktop has no document scroll');
  }
  await page.setViewportSize({width:1600,height:1100});
  const slider=controls.locator('input[data-manipulation-channel="반지름"]');
  const sceneBox=await page.locator('#canvas-bogae').boundingBox();
  const controlsBox=await controls.boundingBox();
  assert.ok(controlsBox.y+controlsBox.height<=sceneBox.y+2,'controls are directly above the main scene');
  await page.screenshot({path:path.join(output,'controls-next-to-scene.png'),fullPage:true});
  const sliderBox=await slider.boundingBox();await page.mouse.click(sliderBox.x+sliderBox.width*0.65,sliderBox.y+sliderBox.height/2);
  const pad=controls.locator('canvas[data-manipulation-channel="위치"]');
  const box=await pad.boundingBox();await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();
  await page.mouse.move(box.x+box.width*0.7,box.y+box.height*0.3,{steps:6});await page.mouse.up();
  // Raw conversion is a Worker/Rust operation, so wait for its explicit intake
  // before advancing the next madi. No fixture mutates the world directly.
  await page.waitForTimeout(300);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().livePump.in_flight);
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>{try{return Number(JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).resources.fixed64.반지름)>1;}catch{return false;}});
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="logical-record-open"]').click();
  const download=page.waitForEvent('download');await panel.locator('[data-action="seal"]').click();
  const tape=path.join(output,'actual-mouse-input.tape');await(await download).saveAs(tape);
  await page.locator('[data-testid="logical-record-close"]').click();
  let expected=await state();const source=await page.locator('#source-editor').inputValue(),binding=await panel.locator('[data-field="binding"]').inputValue();
  const sourceFile=path.join(output,'source.ddn'),bindingFile=path.join(output,'binding.json');fs.writeFileSync(sourceFile,source);fs.writeFileSync(bindingFile,binding);
  assert.equal(fs.readFileSync(tape).subarray(0,18).toString(),'DDN_INPUT_TAPE_V3\n','the default download uses the approved unified transport, not the legacy V2 carrier');
  for(const name of ['ddonirang','ddonilang','또니랑']){
    const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['조작입력재생',sourceFile,tape,bindingFile],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
    assert.notEqual(result.status,0,'a live stop cannot invent a finite completion proof');
    assert.match(result.stderr,/E_INPUT_TAPE_V3/);assert.equal(result.stdout.trim(),'','incomplete replay publishes no success');
  }
  await page.locator('[data-testid="lifecycle-stop"]').click();
  // Reopen the real page: the downloaded file, not a retained live VM,
  // must supply the replay. Select the authored input contract explicitly.
  await page.reload();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await page.locator('[data-testid="global-official-collection"]').click();
  if(!(await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').isVisible()))await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  assert.equal(await page.locator('#source-editor').inputValue(),source);
  assert.deepEqual(JSON.parse(await panel.locator('[data-field="binding"]').inputValue()),JSON.parse(binding));
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="logical-record-open"]').click();
  await panel.locator('[data-field="advanced"] > summary').click();
  // This is a separately declared finite request, not a relabelled live
  // prefix. The original request bytes enter the shared Rust recorder.
  const plan={binding:JSON.parse(binding),snapshots:[
    {madi:0,held:{},events:[{ordinal:0,channel:'반지름',event:{kind:'Number',payload:{raw:'8589934592',phase:'Set'}}},{ordinal:1,channel:'위치',event:{kind:'Coordinate',payload:{raw:['0','0'],phase:'Begin'}}}]},
    {madi:1,held:{},events:[{ordinal:0,channel:'위치',event:{kind:'Coordinate',payload:{raw:['4294967296','0'],phase:'Move'}}}]},
    {madi:2,held:{},events:[{ordinal:0,channel:'위치',event:{kind:'Coordinate',payload:{raw:['4294967296','0'],phase:'End'}}}]},
  ]};
  const malformed=structuredClone(plan);malformed.binding.source_revision='sha256:'+'0'.repeat(64);
  const beforeRejected=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  let downloads=0;page.on('download',()=>downloads++);
  await panel.locator('[data-field="finite-request"]').setInputFiles({name:'wrong-binding.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(malformed))});
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="feedback"]').textContent.includes('E_'));
  assert.equal(downloads,0);assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),beforeRejected);
  const completeDownload=page.waitForEvent('download');
  await panel.locator('[data-field="finite-request"]').setInputFiles({name:'finite-request.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(plan))});
  const completeTape=path.join(output,'finite-request-v3.tape');await(await completeDownload).saveAs(completeTape);
  assert.equal(fs.readFileSync(completeTape).subarray(0,18).toString(),'DDN_INPUT_TAPE_V3\n');
  for(const name of ['ddonirang','ddonilang','또니랑']){
    const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,name+'.exe'),['조작입력재생',sourceFile,completeTape,bindingFile],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
    assert.equal(result.status,0,result.stderr||String(result.error));const value=JSON.parse(result.stdout);
    if(name==='ddonirang')expected=value;else assert.deepEqual(value,expected);
  }
  await page.locator('[data-testid="logical-record-close"]').click();
  await page.reload();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await page.locator('[data-testid="global-official-collection"]').click();
  if(!(await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').isVisible()))await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator('[data-testid="project-card-alpha4_manipulation_point"]').click();
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="logical-record-open"]').click();
  await panel.locator('[data-field="replay"]').setInputFiles(completeTape);
  await page.waitForFunction(count=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count===count;}catch{return false;}},expected.madi_count);
  assert.deepEqual(await state(),expected);assert.deepEqual(errors,[]);
  await page.locator('[data-testid="logical-record-close"]').click();
  await page.screenshot({path:path.join(output,'actual-mouse-input.png'),fullPage:true});
  await page.locator('[data-testid="global-official-collection"]').click();
  if(!(await page.locator('[data-testid="project-card-alpha4_lrl_lab"]').isVisible()))await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator('[data-testid="project-card-alpha4_lrl_lab"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  assert.equal(await controls.locator('input,canvas').count(),0,'running/replayed project controls are discarded');
  assert.equal(await panel.locator('[data-field="binding"]').inputValue(),'');
  assert.match(await page.locator('#source-editor').inputValue(), /\(그림=공용그림\) 쓰임/);
  assert.equal(await page.locator('[data-exploration-parameter="이심률"]').inputValue(),'0.5');
  assert.equal(await page.locator('[data-exploration-parameter="간격"]').inputValue(),'0.01');
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi>=10);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().livePump.in_flight);
  assert.equal(await page.locator('[data-authored-relation-id]').count(),2,'hodograph and measurements are separate sides');
  assert.ok(await page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]').isVisible());
  const beforeLaunch=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  if(!(await page.locator('[data-testid="project-more-menu"]').evaluate(node=>node.open))) await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="project-files-open"]').click();
  assert.ok(await page.locator('[data-testid="file-list"]').isVisible());
  assert.equal(await page.locator('[data-testid="file-list"] > *').count(),3,'multi-module files remain reachable');
  await page.locator('[data-testid="project-files-close"]').click();
  await page.locator('[data-testid="mode-bogae-full"]').click();
  assert.equal(await page.locator('[data-testid="project-rail"]').isVisible(),false,'experiment mode hides catalog');
  const edge=page.locator('[data-authored-side-edge="right"]');
  assert.equal(await edge.isVisible(),false,'open sides are not covered by a duplicate edge handle');
  const rightClose=page.locator('[data-authored-dock="right"] [data-authored-side-close]:visible');
  while(await rightClose.count())await rightClose.first().click();
  assert.ok(await edge.isVisible(),'closing all authored sides exposes their reopening handle');
  assert.equal(await edge.getAttribute('aria-expanded'),'false');
  assert.equal(await page.locator('[data-authored-dock="right"]').isVisible(),false);
  await edge.click();
  assert.equal(await edge.getAttribute('aria-expanded'),'true');
  assert.equal(await edge.isVisible(),false,'reopened sides hide the duplicate edge handle');
  assert.ok(await page.locator('[data-authored-dock="right"]').isVisible());
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),beforeLaunch,'edge operations never advance or alter the world');
  assert.ok(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1));
  for(const viewport of [{width:1366,height:768},{width:1920,height:1080}]) {
    await page.setViewportSize(viewport);await page.waitForTimeout(150);
    const primary=await page.locator('#canvas-bogae').boundingBox();
    assert.ok(primary.y+primary.height<=viewport.height,'whole primary scene fits experiment viewport');
    if(checkCompactExperiment){
      assert.equal(await page.getByTestId('file-tabs').isVisible(),false,'experiment mode hides editing-only file tabs');
      assert.ok(primary.height>=viewport.height*0.6,`experiment scene should use at least 60% of viewport height: ${primary.height}/${viewport.height}`);
      assert.ok(await page.getByTestId('project-more-menu').isVisible(),'storage and record tools remain reachable in the compact menu');
    }
    assert.ok(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1));
    await page.screenshot({path:path.join(output,`experiment-${viewport.width}.png`),fullPage:true});
  }
  await page.setViewportSize({width:1600,height:1100});
  await page.screenshot({path:path.join(output,'experiment-mode.png'),fullPage:true});
  await page.locator('[data-testid="mode-ddn-main"]').click();
  const sourceBeforeCamera=await page.locator('#source-editor').inputValue();
  const mainCanvas=page.locator('#canvas-bogae'),cameraBox=await mainCanvas.boundingBox();
  const beforePixels=await mainCanvas.evaluate(node=>node.toDataURL());
  await page.mouse.move(cameraBox.x+cameraBox.width/2,cameraBox.y+cameraBox.height/2);
  await page.mouse.down({button:'middle'});
  await page.mouse.move(cameraBox.x+cameraBox.width/2+35,cameraBox.y+cameraBox.height/2+15,{steps:4});
  await page.mouse.up({button:'middle'});await page.mouse.wheel(0,-180);
  await page.waitForTimeout(100);
  assert.notEqual(await mainCanvas.evaluate(node=>node.toDataURL()),beforePixels,'middle drag and wheel alter the rendered view');
  assert.equal(await page.locator('#source-editor').inputValue(),sourceBeforeCamera,'camera cannot change initial conditions');
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),beforeLaunch,'camera cannot advance or change world state');
  await page.screenshot({path:path.join(output,'middle-pan-wheel-zoom.png'),fullPage:true});
  await page.locator('[data-authored-camera-controls] > summary').click();
  await page.getByRole('button',{name:'원문 범위로 돌아가기',exact:true}).click();
  await page.locator('[data-authored-camera-controls] > summary').click();
  const launch=await page.evaluate(()=>{
    const canvas=document.querySelector('#canvas-bogae'),rect=canvas.getBoundingClientRect();
    const view=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.wasm_state.bogae_projection.candidates[0].artifact.view;
    const raw=x=>Number(BigInt(x))/2**32,[xmin,xmax]=view.horizontal.map(raw),[ymin,ymax]=view.vertical.map(raw);
    const center=view.elements[0].center.map(raw),scale=Math.min((canvas.width-36)/(xmax-xmin),(canvas.height-36)/(ymax-ymin));
    const screen=p=>({x:rect.left+((canvas.width-(xmax-xmin)*scale)/2+(p[0]-xmin)*scale)*rect.width/canvas.width,
      y:rect.top+((canvas.height+(ymax-ymin)*scale)/2-(p[1]-ymin)*scale)*rect.height/canvas.height});
    return {start:screen(center),end:screen([center[0]+0.2,center[1]+1.2])};
  });
  await page.mouse.move(launch.start.x,launch.start.y);await page.mouse.down();
  await page.mouse.move(launch.end.x,launch.end.y,{steps:6});await page.mouse.up();
  assert.match(await page.locator('#source-editor').inputValue(),/발사설정 := 1\./);
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),beforeLaunch,'launch drag only edits source until explicit rerun');
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(hash=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash!==hash,beforeLaunch.state_hash);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().livePump.in_flight);
  await page.screenshot({path:path.join(output,'lrl-lab.png'),fullPage:true});
  await page.locator('[data-testid="global-official-collection"]').click();
  await page.locator('[data-testid="project-card-alpha3_double_pendulum_lab"]').click();
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi>=48);
  assert.equal(await page.locator('#completion-popup').isVisible(),false,'completion from another project cannot cover this source');
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().livePump.in_flight);
  const pendulumFrame=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  assert.ok(await page.locator('[data-authored-relation-id]').count()>=2,'double pendulum exercises multiple side panels');
  for(const mode of ['ddn-main','bogae-full']) {
    await page.locator(`[data-testid="mode-${mode}"]`).click();
    for(const viewport of [{width:1366,height:768},{width:1920,height:1080}]) {
      await page.setViewportSize(viewport);await page.waitForTimeout(150);
      assert.ok(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1),'multiple pendulum sides cannot lengthen the page');
      const box=await page.locator('#canvas-bogae').boundingBox();
      assert.ok(box.y+box.height<=viewport.height,'pendulum primary remains fully in the viewport');
      if(checkCompactExperiment&&mode==='bogae-full'){
        assert.equal(await page.getByTestId('file-tabs').isVisible(),false);
        assert.ok(box.height>=viewport.height*0.6,`pendulum scene uses experiment space: ${box.height}/${viewport.height}`);
      }
      await page.screenshot({path:path.join(output,`pendulum-${mode}-${viewport.width}.png`),fullPage:true});
    }
  }
  const closedSideHandles=page.locator('[data-authored-side-edge]:visible');
  while(await closedSideHandles.count()) {
    const handle=closedSideHandles.first();
    assert.equal(await handle.getAttribute('aria-expanded'),'false');
    await handle.click();assert.equal(await handle.getAttribute('aria-expanded'),'true');
    assert.equal(await handle.isVisible(),false,'an open side uses its own Close control');
  }
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),pendulumFrame,'pendulum view operations preserve the world');
  if(checkCompactExperiment){
    await page.setViewportSize({width:390,height:844});await page.waitForTimeout(150);
    assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'narrow experiment must not overflow horizontally');
    assert.ok(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1),'narrow experiment remains one page');
    assert.equal(await page.getByTestId('file-tabs').isVisible(),false);
    assert.equal(await page.getByTestId('source-panel').isVisible(),false,'narrow full-view mode must not reveal the editor');
    await page.screenshot({path:path.join(output,'experiment-narrow-390.png'),fullPage:true});
  }
  await page.locator('[data-testid="mode-ddn-main"]').click();
  await page.setViewportSize({width:1600,height:1100});
  if(checkCompactExperiment)assert.ok(await page.getByTestId('file-tabs').isVisible(),'returning to editing restores file tabs');
  const liveSide=page.locator('.side-view-instance[data-authored-relation-id]').first();
  const sideId=await liveSide.getAttribute('data-authored-relation-id');
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi>=81);
  await liveSide.locator('[data-authored-side-close]').click();
  const closedAt=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.madi);
  await page.waitForFunction(madi=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi>=madi+3,closedAt);
  assert.equal(await liveSide.isVisible(),false,'side close persists across live madis');
  await page.locator(`[data-authored-relation-toggle="${sideId}"]`).click();
  assert.ok(await liveSide.isVisible(),'same side can reopen during play');
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().livePump.in_flight);
  await page.screenshot({path:path.join(output,'pendulum-live-close-reopen.png'),fullPage:true});
  await page.locator('#source-editor').focus();
  await page.keyboard.press('Control+End');await page.keyboard.type('\n ');
  await page.waitForFunction(()=>!document.querySelector('#completion-popup').hidden,{},{timeout:5000});
  assert.ok(await page.locator('#completion-popup button').count()>0,'focused typing still offers actual Rust completions');
  await page.locator('[data-testid="global-first-steps"]').click();
  assert.equal(await page.locator('#completion-popup').isVisible(),false,'leaving the editor closes completion');
  if(!(await page.locator('[data-testid="project-card-first_steps_alpha4_29A"]').isVisible()))await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator('[data-testid="project-card-first_steps_alpha4_29A"]').click();
  await page.locator('[data-testid="first-steps-open"]').click();
  assert.ok((await page.locator('[data-testid="first-steps-title"]').textContent()).includes('29A'));
  for(const pdf of ['학생용_첫걸음_29-30차시.pdf','교사용_첫걸음_29-30차시.pdf']){
    const response=await page.request.get(`${origin}/solutions/seamgrim_ui_mvp/first_steps_alpha4/${encodeURIComponent(pdf)}`);assert.equal(response.status(),200);
  }
  assert.deepEqual(errors,[]);
  const report={result:'PASS_PROVISIONAL_ALPHA4_MOUSE_RECORD',evidence:output,tape_sha256:sha(fs.readFileSync(tape)),wasm_sha256:sha(fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool_bg.wasm'))),madi_count:expected.madi_count,native_aliases:3,lrl_browser:'PASS',pendulum_multi_side_desktop2:'PASS',course29_30_pdf:'PASS',errors};
  fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
}catch(error){console.error('ALPHA4_EVIDENCE',output);console.error(await panel.locator('[data-field="feedback"]').textContent());
  const layout=await page.evaluate(()=>({url:location.href,viewport:{width:innerWidth,height:innerHeight},
    sheets:Array.from(document.styleSheets,sheet=>{let rules;try{rules=sheet.cssRules.length;}catch(error){rules=String(error);}return {href:sheet.href,disabled:sheet.disabled,rules};}),
    links:Array.from(document.querySelectorAll('link[rel="stylesheet"]'),link=>({href:link.href,loaded:Boolean(link.sheet)})),
    body_style:{display:getComputedStyle(document.body).display,font:getComputedStyle(document.body).fontFamily},
    primary:document.querySelector('[data-testid="bogae-canvas-wrap"]')?.getBoundingClientRect().toJSON()}));
  fs.writeFileSync(path.join(output,'failure.json'),JSON.stringify({error:String(error.stack??error),errors,layout},null,2));
  await page.screenshot({path:path.join(output,'failure.png'),fullPage:true});throw error;}
finally{await browser.close();if(server.listening)await new Promise(resolve=>server.close(resolve));}
