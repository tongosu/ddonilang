// Actual extracted ZIP only. No alternate WASM, synthetic world or fixture server.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL,fileURLToPath} from 'node:url';

assert.ok(process.argv[2], 'Usage: node alpha2_archive_browser_runner.mjs <fresh extracted package root>');
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE, 'Explicit cached DDN_PLAYWRIGHT_MODULE required');
const root=fs.realpathSync(process.argv[2]);
const version=process.env.DDN_ARCHIVE_VERSION || '1.0.0-alpha.2';
assert.ok(['1.0.0-alpha.2','1.0.0-alpha.3','1.0.0-alpha.4','1.0.0-beta.1'].includes(version),'explicit supported archive version');
const evidence=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha2-zip-browser-'));
const base='solutions/seamgrim_ui_mvp/', ui=base+'ui/workbench/', samples=base+'samples/';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasmHash=sha(fs.readFileSync(path.join(root,base+'ui/wasm/ddonirang_tool_bg.wasm')));
const requests=[], httpFailures=[], servedWasm=[], pageErrors=[], consoleErrors=[], failedRequests=[];
const server=http.createServer((req,res)=>{
  try {
    const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
    const local=fs.realpathSync(path.resolve(root,rel));
    assert.ok(local.startsWith(root+path.sep),'outside extracted ZIP');
    const bytes=fs.readFileSync(local);
    const type=local.endsWith('.wasm')?'application/wasm':local.endsWith('.js')?'text/javascript':
      local.endsWith('.html')?'text/html':local.endsWith('.css')?'text/css':
      local.endsWith('.pdf')?'application/pdf':local.endsWith('.json')?'application/json':'text/plain';
    if(local.endsWith('ddonirang_tool_bg.wasm'))servedWasm.push(sha(bytes));
    requests.push({path:rel,status:200});res.writeHead(200,{'content-type':type}).end(bytes);
  } catch(error) {
    httpFailures.push({url:req.url,error:String(error)});res.writeHead(404).end();
  }
});
// This host's ephemeral range includes the owner's protected 8790. Choose a
// private high port before binding, rather than briefly binding a low one.
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
const origin=`http://127.0.0.1:${server.address().port}`;
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
let browser, page;
const completed=[], pending=['menu3','templates3','tetris-file-native3-screen-replay','first_steps57-pdf-http','first_steps_alpha2_18','persistent-storage','diagnostics3-ui'];
const snapshot=()=>page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
const frame=async()=>(await snapshot()).activeFrame;
const settle=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
async function load() {
  await page.goto(origin+'/'+ui+'index.html');
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  assert.ok((await page.locator('[data-testid="product-version"]').textContent()).includes(version));
}
async function run() {
  await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false);
  const success=await page.evaluate(async()=>{
    const app=window.__SEAMGRIM_APPROVED_WORKBENCH__;
    const success=await app.startRun();app.pauseRun();return success;
  });
  assert.equal(success,true,JSON.stringify((await snapshot()).diagnostics));
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
  return frame();
}
async function menu(id,name) {
  await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator(`#project-list [data-project-id="${id}"]`).click();
  const source=fs.readFileSync(path.join(root,samples+`alpha2_${name}_composition.ddn`),'utf8');
  await page.waitForFunction(text=>document.querySelector('#source-editor').value===text,source);
  if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')==='true')
    await page.locator('[data-testid="project-rail-toggle"]').click();
}
async function authoredOpen() {
  const more=page.locator('[data-testid="project-more-menu"]');
  if(!(await more.evaluate(node=>node.open)))await more.locator('summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
}
async function authoredStatus(text) {
  await page.waitForFunction(text=>document.querySelector('[data-authored-status]').textContent.includes(text),text);
}
const mark=(name,data)=>{completed.push({name,...data});pending.splice(pending.indexOf(name),1);};
try {
  browser=await chromium.launch({headless:true,executablePath:process.env.DDN_CHROME_EXECUTABLE||'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  page=await browser.newPage({viewport:{width:1600,height:1100},acceptDownloads:true});
  page.setDefaultTimeout(45000);
  page.on('pageerror',error=>pageErrors.push(String(error)));
  page.on('console',message=>{if(message.type()==='error')consoleErrors.push(message.text());});
  page.on('requestfailed',request=>failedRequests.push({url:request.url(),error:request.failure()?.errorText}));
  const menus=[];
  for(const [id,name] of [['rep_phys_projectile_xy_v1','parabola'],['rep_phys_pendulum_wave1_v1','pendulum'],['local_linear_slope_v1','linear']]) {
    await load();await menu(id,name);const before=await run();
    await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.authoredBogaeStatus==='ready');
    const canvas=page.locator('#canvas-bogae');await settle();
    assert.equal(await page.locator('[data-testid="bogae-entry"]').getAttribute('aria-label'),'표시할 보개 이름');
    // Result-table sides retain a canvas node but hide it in favor of the
    // actual typed table. Measure the visible retained side container instead.
    const sideBox=await page.locator('[data-authored-relation-id]').first().boundingBox();
    assert.ok(sideBox && sideBox.width>=250,'side must be readable, not a thumbnail');
    const pixels=await canvas.evaluate(node=>node.toDataURL());
    const overlay=page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]');
    await overlay.click();await settle();assert.notEqual(await canvas.evaluate(node=>node.toDataURL()),pixels);
    assert.deepEqual(await frame(),before);await overlay.click();await settle();
    assert.equal(await canvas.evaluate(node=>node.toDataURL()),pixels);
    const side=page.locator('[data-authored-relation-id]').first();
    const relation=await side.getAttribute('data-authored-relation-id');
    await side.locator('[data-authored-side-close]').click();assert.equal(await side.isVisible(),false);
    await page.locator(`[data-authored-relation-toggle="${relation}"]`).click();assert.equal(await side.isVisible(),true);
    const sideIds=await page.locator('[data-authored-relation-id]').evaluateAll(nodes=>nodes.map(node=>node.dataset.authoredRelationId));
    await page.locator('[data-testid="authored-close-sides"]').click();
    for(const id of sideIds) assert.equal(await page.locator(`[data-authored-relation-id="${id}"]`).isVisible(),false);
    assert.deepEqual(await frame(),before);
    for(const id of sideIds) await page.locator(`[data-authored-relation-toggle="${id}"]`).click();
    assert.equal(await page.locator('[data-testid="view-role-primary"]').isVisible(),false);
    assert.equal(await page.locator('[data-testid="bogae-source-access"]').isVisible(),false);
    const magnifiers=before.wasm_state.bogae_projection.candidates[0].artifact.relations.filter(row=>row.kind==='magnifier');
    async function pick(target,sameFrame=true) {
      const point=await canvas.evaluate((node,target)=>{
        const view=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.wasm_state.bogae_projection.candidates[0].artifact.view;
        const shape=view.elements[target.selection.element_indices[0]];
        const [xmin,xmax]=view.horizontal.map(BigInt),[ymin,ymax]=view.vertical.map(BigInt);
        const sx=Number(xmax-xmin),sy=Number(ymax-ymin),s=Math.min((node.width-36)/sx,(node.height-36)/sy);
        const r=node.getBoundingClientRect();
        return {x:r.left+((node.width-sx*s)/2+Number(BigInt(shape.center[0])-xmin)*s)*r.width/node.width,
          y:r.top+((node.height+sy*s)/2-Number(BigInt(shape.center[1])-ymin)*s)*r.height/node.height};
      },target);
      await page.mouse.click(point.x,point.y);await settle();
      await page.locator('[data-authored-magnifier-id] canvas').waitFor();
      assert.equal(await page.locator('[data-authored-magnifier-id]').count(),1);
      if(sameFrame) {
        assert.deepEqual(JSON.parse(await page.locator('[data-authored-magnifier-target]').textContent()),target.target);
        assert.deepEqual(await frame(),before);
      } else {
        assert.equal(await page.locator('[data-authored-magnifier-id]').getAttribute('data-authored-magnifier-id'),target.relation_id);
        assert.equal(await page.evaluate(id=>{
          const current=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.wasm_state.bogae_projection.candidates[0].artifact.relations.find(row=>row.relation_id===id);
          return JSON.stringify(JSON.parse(document.querySelector('[data-authored-magnifier-target]').textContent))===JSON.stringify(current.target);
        },target.relation_id),true,'shown live properties come from the committed Rust frame');
      }
    }
    await pick(magnifiers[0]);
    if(magnifiers.length>1) {
      await page.evaluate(()=>{window.previousPickedMagnifier=document.querySelector('[data-authored-magnifier-id]');});
      await pick(magnifiers.at(-1));
      assert.equal(await page.evaluate(()=>window.previousPickedMagnifier.isConnected),false,'old target instance terminated');
    }
    await page.locator('[data-authored-magnifier-close]').click();assert.deepEqual(await frame(),before);
    if(name==='pendulum') {
      await pick(magnifiers[0]);
      await page.evaluate(()=>{window.followedObject=document.querySelector('[data-authored-magnifier-id]');});
      await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.resumeRun());
      await page.waitForFunction(hash=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash!==hash,before.state_hash);
      assert.equal(await page.evaluate(()=>window.followedObject===document.querySelector('[data-authored-magnifier-id]')),true);
      assert.notDeepEqual(JSON.parse(await page.locator('[data-authored-magnifier-target]').textContent()),magnifiers[0].target);
      const closeSide=page.locator('[data-authored-side-close]').first();
      const id=await closeSide.getAttribute('data-authored-side-close');await closeSide.click();
      const live=await frame();await page.waitForTimeout(200);
      assert.notEqual((await frame()).state_hash,live.state_hash);
      assert.equal(await page.locator(`[data-authored-relation-id="${id}"]`).isVisible(),false);
      await page.locator(`[data-authored-relation-toggle="${id}"]`).click();
      assert.equal(await page.locator(`[data-authored-relation-id="${id}"]`).isVisible(),true);
      await pick(magnifiers.at(-1),false);
      assert.equal(await page.evaluate(()=>window.followedObject.isConnected),false);
      const running=await frame();await page.waitForTimeout(200);
      assert.notEqual((await frame()).state_hash,running.state_hash,'actual ZIP live target click never pauses');
      assert.equal(await page.locator('[data-authored-magnifier-id]').count(),1,'actual ZIP information remains open during playback');
      await page.locator('[data-authored-magnifier-close]').click();await page.waitForTimeout(100);
      assert.equal(await page.locator('[data-authored-magnifier-id]').count(),0);
      await pick(magnifiers[0],false);
      const objectId=await page.locator('[data-authored-magnifier-id]').getAttribute('data-object-identity');
      const moving=await frame();await page.waitForTimeout(150);
      assert.notEqual((await frame()).state_hash,moving.state_hash);
      assert.equal(await page.locator('[data-authored-magnifier-id]').getAttribute('data-object-identity'),objectId);
      await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.pauseRun());
    }
    await page.screenshot({path:path.join(evidence,`${name}.png`),fullPage:true});
    menus.push({id,name,state_hash:before.state_hash,source:'actual ZIP menu',view_operations_world_unchanged:true});
  }
  mark('menu3',{menus});
  const templateResults=[];
  for(const template of ['physics','single','empty']) {
    await load();
    const rail=page.locator('[data-testid="project-rail-toggle"]');
    if(await rail.getAttribute('aria-expanded')!=='true')await rail.click();
    // Button is a real creation control; locator is resolved against shipped UI.
    await page.locator('[data-testid="new-ddn-project"]').click();
    await page.locator('[data-testid="project-create-name"]').fill(`ZIP 검증 ${template}`);
    await page.locator(`input[name="project-template"][value="${template}"]`).check();
    await page.locator('[data-testid="project-create-submit"]').click();
    await page.locator('[data-testid="project-create-dialog"]').waitFor({state:'hidden'});
    const result=await run();
    templateResults.push({template,state_hash:result.state_hash,source:await page.locator('#source-editor').inputValue()});
  }
  mark('templates3',{templates:templateResults});
  // Verify all dynamic education assets via actual HTTP without lowering them.
  const course=JSON.parse(fs.readFileSync(path.join(root,base+'first_steps/검증목록.json'),'utf8'));
  const coursePaths=[...course.cases,...course.corrections,...course.modifications].map(row=>base+'first_steps/'+row.file);
  coursePaths.push(base+'first_steps/보충/06_마지막에배우는축약.ddn',base+'first_steps/보충/09_축약대조.ddn');
  assert.equal(new Set(coursePaths).size,57);
  const pdfs=['학생용_첫걸음_10차시.pdf','교사용_안내와_해답.pdf'].map(name=>base+'first_steps/'+name);
  const pdfLinks=await page.locator('a[href$=".pdf"]').evaluateAll(nodes=>nodes.map(node=>decodeURIComponent(new URL(node.href).pathname).replace(/^\/+/,'')));
  for(const rel of pdfs)assert.ok(pdfLinks.includes(rel),'shipped UI must link actual PDF: '+rel);
  for(const rel of [...coursePaths,...pdfs]) {
    const response=await page.request.get(origin+'/'+rel);assert.equal(response.status(),200,rel);
    assert.equal(sha(await response.body()),sha(fs.readFileSync(path.join(root,rel))));
  }
  mark('first_steps57-pdf-http',{sources:57,pdfs:2,source_execution:'NOT_RUN_HERE_SEPARATE_NATIVE_WASM_GATE'});
  const extended=JSON.parse(fs.readFileSync(path.join(root,base+'first_steps_alpha2/검증목록.json'),'utf8'));
  const extendedRows=[...extended.cases,...extended.modifications];
  assert.equal(extendedRows.length,18);
  const extendedAssets=new Set(extendedRows.map(row=>row.file));
  for(const row of extendedRows)for(const key of ['binding_file','request_file'])if(row[key])extendedAssets.add(row[key]);
  for(const name of ['입력/action_map.json','학생용_첫걸음_11-16차시.pdf','교사용_안내와_해답.pdf'])extendedAssets.add(name);
  for(const rel of extendedAssets)assert.equal((await page.request.get(origin+'/'+base+'first_steps_alpha2/'+rel)).status(),200);
  const courseRail=page.locator('[data-testid="project-rail-toggle"]');
  if(await courseRail.getAttribute('aria-expanded')!=='true')await courseRail.click();
  await page.locator('[data-global-entry="first-steps"]').click();
  assert.equal(await page.locator('[data-testid^="project-card-first_steps_alpha2_"]').count(),18);
  await page.locator('[data-testid="project-card-first_steps_alpha2_11A"]').click();
  await run();
  assert.equal((await frame()).wasm_state.bogae_projection.status,'ready');
  await page.locator('[data-testid="first-steps-open"]').click();
  const extendedPdf=await page.locator('.first-steps-documents a').first().getAttribute('href');
  assert.ok(extendedPdf.includes('first_steps_alpha2/'));
  assert.equal((await page.request.get(new URL(extendedPdf,page.url()).href)).status(),200);
  mark('first_steps_alpha2_18',{sources:18,pdfs:2,http_assets:extendedAssets.size,actual_scene:true,
    source_execution:'ALL18_SEPARATE_NATIVE3_WASM_GATE'});
  await load();await menu('local_linear_slope_v1','linear');const temporary=await run();
  await authoredOpen();await page.locator('[data-authored-create]').click();
  await authoredStatus('명시 작품 정체');
  await page.locator('[data-authored-module]').fill('앱');
  await page.locator('[data-authored-composition]').fill('일차함수짜임');
  await page.locator('[data-authored-register]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-contents]').value.includes('ddn.authored_project_artifact.v2'));
  await page.locator('[data-authored-save]').click();await authoredStatus('이 기기에 저장했습니다');
  const saved=await page.locator('[data-authored-contents]').inputValue();
  await page.locator('[data-authored-close]').click();const persistent=await run();
  assert.equal(persistent.wasm_state.bogae_projection.persistent_identity,true);
  assert.equal(persistent.state_hash,temporary.state_hash);
  await authoredOpen();await page.locator('[data-authored-reopen]').click();await authoredStatus('저장 자료를 다시 열었습니다');
  assert.equal(await page.locator('[data-authored-contents]').inputValue(),saved);
  const projectDownloadPromise=page.waitForEvent('download');await page.locator('[data-authored-export]').click();
  const projectFile=path.join(evidence,'authored-project.json');await (await projectDownloadPromise).saveAs(projectFile);
  assert.equal(fs.readFileSync(projectFile,'utf8'),saved);
  await page.locator('[data-testid="authored-project-drawer"] input[type="file"]').setInputFiles(projectFile);
  await authoredStatus('파일을 열었습니다');assert.equal(await page.locator('[data-authored-contents]').inputValue(),saved);
  await page.locator('[data-authored-close]').click();const imported=await run();
  assert.deepEqual(imported.wasm_state.bogae_projection,persistent.wasm_state.bogae_projection);
  assert.equal(imported.state_hash,persistent.state_hash);
  mark('persistent-storage',{state_hash:persistent.state_hash,export_sha256:sha(fs.readFileSync(projectFile))});
  await load();const panel=page.locator('#logical-input-panel');
  await page.locator('[data-testid="global-official-collection"]').click();
  assert.equal(await page.locator('[data-testid="project-card-alpha2_logical_tetris"]').isVisible(),true,'official collection must open its list without a second button');
  assert.ok(!(await page.locator('[data-testid="project-card-alpha2_logical_tetris"]').textContent()).includes('알파1 실행'),'logical keyboard work must not carry an alpha1 label');
  await page.locator('[data-testid="project-card-alpha2_logical_tetris"]').click();
  assert.equal(await panel.locator('[data-action="tetris-contract"]').count(),0,'common UI has no hardcoded game contract');
  const source=fs.readFileSync(path.join(root,samples+'alpha2_logical_tetris.ddn'),'utf8');
  await page.waitForFunction(text=>document.querySelector('#source-editor').value===text,source);
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="binding"]').value.includes('source_revision'));
  const binding=await panel.locator('[data-field="binding"]').inputValue();
  const primary=page.locator('[data-testid="lifecycle-run"]');
  await primary.click();
  const logical=()=>page.evaluate(()=>JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent));
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"조각번호"'));
  await page.waitForFunction(()=>{
    try {const value=JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent);
      return document.querySelector('[data-testid="current-madi"]').textContent===`마디: ${value.frame.input.madi}`;}catch{return false;}
  });
  await page.screenshot({path:path.join(evidence,'actual-tetris-running-madi.png'),fullPage:true});
  await primary.click();
  await page.waitForFunction(()=>{const b=document.querySelector('[data-testid="lifecycle-run"]');return b.dataset.action==='resume' && !b.disabled;});
  const first=await logical();await page.locator('[data-testid="bogae-canvas"]').focus();await page.keyboard.press('ArrowLeft');
  const tetrisBounds=await page.locator('[data-testid="bogae-canvas-wrap"]').boundingBox();
  assert.ok(Math.abs(tetrisBounds.width/tetrisBounds.height-0.5)<0.02,'authored 240x480 overrides the common 4:3 default');
  for(const viewport of [{width:1366,height:768},{width:1920,height:1080}]) {
    await page.setViewportSize(viewport);await settle();await page.waitForTimeout(150);
    const board=await page.locator('[data-testid="bogae-canvas-wrap"]').boundingBox();
    assert.ok(board.y>=0 && board.y+board.height<=viewport.height,'whole Tetris board is inside the viewport');
    assert.ok(await page.evaluate(()=>document.documentElement.scrollHeight<=innerHeight+1),'Tetris does not lengthen the page');
    await page.screenshot({path:path.join(evidence,`tetris-whole-board-${viewport.width}.png`),fullPage:true});
  }
  await page.setViewportSize({width:1600,height:1100});await settle();
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(madi=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi>madi;}catch{return false;}},first.frame.input.madi);
  const moved=await logical();assert.equal(first.resources.fixed64.가로,'3');assert.equal(moved.resources.fixed64.가로,'2');
  assert.equal(await page.locator('[data-testid="current-madi"]').textContent(),`마디: ${moved.frame.input.madi}`,'keyboard single-step counter must reflect committed input');
  await primary.click();
  await page.waitForFunction(()=>{
    try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi>=3;}catch{return false;}
  });
  await primary.click();
  await page.waitForFunction(()=>{const b=document.querySelector('[data-testid="lifecycle-run"]');return b.dataset.action==='resume' && !b.disabled;});
  await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().livePump.in_flight);
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="logical-record-open"]').click();
  const download=page.waitForEvent('download').then(value=>({value}),error=>({error}));
  await panel.locator('[data-action="seal"]').click();
  const downloaded=await download;
  assert.ok(downloaded.value,`record save failed: ${await panel.locator('[data-field="result"]').textContent()} / ${await panel.locator('[data-field="feedback"]').textContent()}`);
  const tapeFile=path.join(evidence,'actual-browser-logical-input.tape');await downloaded.value.saveAs(tapeFile);
  await page.locator('[data-testid="logical-record-close"]').click();
  const expected=await logical(), sourceFile=path.join(evidence,'tetris.ddn'), bindingFile=path.join(evidence,'binding.json');
  fs.writeFileSync(sourceFile,source);fs.writeFileSync(bindingFile,binding);
  const nativeResults=[];
  for(const name of ['ddonirang','ddonilang','또니랑']) {
    const native=path.join(root,'bin',name+'.exe');
    const result=spawnSync(native,['논리입력재생',sourceFile,tapeFile,bindingFile],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
    assert.equal(result.status,0,result.stderr||String(result.error));assert.deepEqual(JSON.parse(result.stdout),expected);
    nativeResults.push({name,binary_sha256:sha(fs.readFileSync(native)),same_complete_summary:true});
  }
  await page.locator('[data-testid="lifecycle-stop"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('실행을 닫았습니다'));
  await panel.locator('[data-field="replay"]').setInputFiles(tapeFile);
  await page.waitForFunction(count=>{
    try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count===count;}catch{return false;}
  },expected.madi_count);
  assert.deepEqual(await logical(),expected);
  assert.equal(await panel.locator('[data-field="canvas"]').isVisible(),false);
  assert.equal(await page.locator('[data-testid="bogae-canvas"]').getAttribute('data-committed-state-hash'),expected.state_hash);
  await page.screenshot({path:path.join(evidence,'actual-tetris-file-replay.png'),fullPage:true});
  mark('tetris-file-native3-screen-replay',{madi_count:expected.madi_count,tape_sha256:sha(fs.readFileSync(tapeFile)),nativeResults});
  const diagnosticResults=[];
  for(const [id,body,code,line] of [
    ['primary','기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸.\n짜임 := (관계들=차림[기본,기본]) 표준보개.짜임.','E_BOGAE_PRIMARY_COUNT',4],
    ['radius','원 := (중심=차림[0,0], 반지름=0) 표준장면.원.','E_SCENE_RADIUS',3],
    ['recipe','(대상:수) 확대:수:셈씨 := { 확대 <- 대상. }.\n자세히 := (대상=2, 보기법=확대) 표준보개.돋.','E_BOGAE_RECIPE_TYPE',4],
  ]) {
    // These are standalone constructor errors, not a formal project's missing
    // start definition. Use a shipped one-file editor, preserving test source.
    await load();await menu('rep_phys_projectile_xy_v1','parabola');
    await page.locator('#source-editor').fill('\n\n'+body);
    // Follow the normal readiness protocol instead of invoking startRun while
    // source-edit analysis is still pending (which returns false without run).
    await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false);
    await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(code=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics
      .some(item=>String(item.message).includes(code)),code);
    const diagnostics=(await snapshot()).diagnostics;
    assert.ok(diagnostics.some(item=>String(item.message).includes(code)),JSON.stringify(diagnostics));
    const visible=await page.locator('body').innerText();
    assert.ok(visible.includes(code),`${id}: error must be visible`);
    assert.ok(visible.includes(`(${line}:`),`${id}: authored line missing on screen`);
    await page.screenshot({path:path.join(evidence,`diagnostic-${id}.png`),fullPage:true});
    diagnosticResults.push({id,code,authored_line:line,rejected:true,visible:true});
    await page.keyboard.press('Escape');
  }
  mark('diagnostics3-ui',{cases:diagnosticResults});
  if(version==='1.0.0-alpha.2') {
  for(const [id,call] of [['fixed','(마디=2) 표준시간.고정마디'],
    ['relative','(차이=-1) 표준시간.상대마디']]) {
    await load();await menu('rep_phys_projectile_xy_v1','parabola');
    await page.locator('#source-editor').fill('\n\n시간 := '+call+'.');
    await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false);
    await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics
      .some(item=>String(item.message).includes('E_BOGAE_TIME_POLICY_UNSUPPORTED')));
    const visible=await page.locator('body').innerText();
    assert.ok(visible.includes('E_BOGAE_TIME_POLICY_UNSUPPORTED'));
    assert.ok(visible.includes('알파3'));
    await page.screenshot({path:path.join(evidence,`time-unsupported-${id}.png`),fullPage:true});
  }
  mark('time-policy-unsupported2-ui');
  } else {
    // Alpha3 implements RD-136. Execute the existing positive Chrome/Worker
    // gate against this exact extracted archive, not the Alpha2 refusal oracle.
    const timeGate=spawnSync(process.execPath,[path.join(path.dirname(fileURLToPath(import.meta.url)),'alpha3_scene_time_browser_runner.mjs')],{
      cwd:root,env:{...process.env,DDN_WASM_DIR:path.join(root,base+'ui/wasm')},encoding:'utf8',timeout:240000,
    });
    assert.equal(timeGate.status,0,timeGate.stderr+'\n'+timeGate.stdout);
    const timeReport=JSON.parse(timeGate.stdout.trim().split(/\r?\n/).at(-1));
    assert.equal(timeReport.status,'PASS');assert.equal(timeReport.wasm_sha256,wasmHash);
    mark('time-policy-fixed-relative3-ui',{report:timeReport});
  }
  if(process.env.DDN_SELECTION_PARITY_REPORT){
    const input=JSON.parse(fs.readFileSync(process.env.DDN_SELECTION_PARITY_REPORT,'utf8'));
    assert.equal(input.result,'PASS');assert.equal(input.wasm_sha256,wasmHash);
    const rows=[];
    for(const item of input.rows){
      const source=fs.readFileSync(item.source,'utf8');assert.equal(sha(Buffer.from(source)),item.source_sha256);
      await load();await menu('rep_phys_projectile_xy_v1','parabola');
      await page.locator('#source-editor').fill(source);
      if(item.expected_rejection){
        const failure=JSON.parse(item.wasm_error);
        const code=item.code??failure.code;
        await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false);
        await page.getByTestId('lifecycle-run').click();
        await page.waitForFunction(code=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics.some(row=>String(row.message).includes(code)||row.code===code),code);
        assert.equal((await snapshot()).activeFrame,null,`${item.id}: no rejected frame publication`);
        assert.ok((await page.locator('body').innerText()).includes(code),`${item.id}: visible diagnostic`);
        rows.push({id:item.id,source_sha256:item.source_sha256,rejected:true,visible_code:code});
      }else{
        await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.disabled===false);
        await page.getByTestId('lifecycle-run').click();
        await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
        const result=await frame();
        assert.deepEqual(result.wasm_state.output_display.texts,item.expected_texts,`${item.id}: hand-computed output`);
        assert.equal(result.state_hash,item.state_hash,`${item.id}: same CLI3/WASM/Workbench world`);
        rows.push({id:item.id,source_sha256:item.source_sha256,state_hash:result.state_hash,execution_control:'actual-lifecycle-run-button'});
      }
      await page.screenshot({path:path.join(evidence,`selection-${item.id}.png`),fullPage:true});
      await page.keyboard.press('Escape');
    }
    mark('typed-selection-raw-frontdoors',{family:input.family,cases:rows});
  }
  assert.ok(servedWasm.length>0);for(const hash of servedWasm)assert.equal(hash,wasmHash);
  assert.deepEqual(pageErrors,[]);assert.deepEqual(consoleErrors,[]);assert.deepEqual(failedRequests,[]);assert.deepEqual(httpFailures,[]);
  const report={status:version==='1.0.0-beta.1'?'PASS_EXTRACTED_BETA1_BROWSER':version==='1.0.0-alpha.2'?'PASS_EXTRACTED_ALPHA2_BROWSER':'PASS_EXTRACTED_ALPHA3_BROWSER',version,root,evidence,wasm_sha256:wasmHash,servedWasm,
    completed,pending,request_count:requests.length,http_failures:httpFailures,page_errors:pageErrors,
    console_errors:consoleErrors,request_failures:failedRequests,independent_review:'NOT_CLAIMED',owner_human:'NOT_RUN'};
  fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
} catch(error) {
  if(page)await page.screenshot({path:path.join(evidence,'failure.png'),fullPage:true}).catch(()=>{});
  const report={status:'FAIL',error:String(error),root,evidence,completed,not_run_or_incomplete:pending,httpFailures,pageErrors,consoleErrors,failedRequests};
  fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(report,null,2));console.error(JSON.stringify(report));throw error;
} finally {
  if(browser)await browser.close();await new Promise(resolve=>server.close(resolve));
}
