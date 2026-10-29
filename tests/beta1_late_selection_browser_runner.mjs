import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const [root,fixture]=process.argv.slice(2);assert(root&&fixture&&process.env.DDN_PLAYWRIGHT_MODULE);
const hash=b=>createHash('sha256').update(b).digest('hex');
const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-late-selection-browser-'));
const wasmDir=path.join(root,'solutions/seamgrim_ui_mvp/ui/wasm'),wasmBytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);await wasm.default({module_or_path:wasmBytes});
const authoredFile=path.join(fixture,'authored-project.json'),authored=fs.readFileSync(authoredFile,'utf8');
const binding=fs.readFileSync(path.join(fixture,'binding.json'),'utf8'),actions=fs.readFileSync(path.join(fixture,'actions.json'),'utf8');
const projectId=JSON.parse(authored).authored_identity.logical_dependency_id;
const errors=[],servedWasm=[],servedPanel=[];
const panelRelative='solutions/seamgrim_ui_mvp/ui/workbench/logical_input_panel.js';
const override=process.env.DDN_LATE_UI_OVERRIDE;
const checkGenericGuide=process.env.DDN_GENERIC_GUIDE_CHECK==='1';
const server=http.createServer((req,res)=>{
 const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
 const file=path.resolve(root,relative);
 if(!file.startsWith(path.resolve(root)+path.sep)){res.writeHead(403).end();return;}
 try{const actual=override&&relative===panelRelative?override:file;const bytes=fs.readFileSync(actual);
  if(relative===panelRelative)servedPanel.push(hash(bytes));if(file.endsWith('_bg.wasm'))servedWasm.push(hash(bytes));
  res.writeHead(200,{'content-type':file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript; charset=utf-8':file.endsWith('.html')?'text/html; charset=utf-8':file.endsWith('.css')?'text/css; charset=utf-8':'application/json; charset=utf-8'}).end(bytes);
 }catch{res.writeHead(404).end();}
});
// Local test transport only: never bind the owner's protected 8790 or a
// Chrome-restricted low port. A busy candidate must not disturb its owner.
let listening=false;
for(let attempt=0;attempt<64&&!listening;attempt++) {
 listening=await new Promise((resolve,reject)=>{
  const ready=()=>{server.off('error',failed);resolve(true);};
  const failed=error=>{server.off('listening',ready);
   if(error.code==='EADDRINUSE'||error.code==='EACCES')resolve(false);else reject(error);};
  server.once('error',failed);server.once('listening',ready);
  server.listen(20000+Math.floor(Math.random()*40000),'127.0.0.1');
 });
}
assert(listening,'private test port unavailable; existing listeners preserved');
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const options={headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',viewport:{width:1500,height:1000}};
const profile=path.join(output,'chrome-profile');let browser=await chromium.launchPersistentContext(profile,options);
const url=`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`;
let page,report;
async function openPage(){page=browser.pages()[0]??await browser.newPage();page.on('pageerror',e=>errors.push(String(e)));page.on('requestfailed',r=>errors.push(r.url()));page.on('response',r=>{if(r.status()>=400)errors.push(r.url());});await page.goto(url);await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');}
async function menu(){const node=page.getByTestId('project-more-menu');if(!await node.evaluate(n=>n.open))await node.locator('summary').click();return node;}
async function authoredPanel(){await menu();await page.getByTestId('authored-project-open').click();}
async function closeAuthored(){await page.locator('[data-authored-close]').click();}
async function bindCurrent(){
 await menu();await page.getByTestId('logical-record-open').click();const panel=page.locator('#logical-input-panel');
 await panel.locator('[data-field="advanced"]').evaluate(n=>n.open=true);
 await panel.locator('[data-field="contract-editor"]').evaluate(n=>n.open=true);
 await panel.locator('[data-field="binding"]').fill(binding);await panel.locator('[data-field="actions"]').fill(actions);await panel.locator('[data-field="capacity"]').fill('128');
 await panel.locator('[data-action="bind-source"]').click();
 await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('아직 세계 마디'));
 await page.getByTestId('logical-record-close').click();
}
async function record(){return page.evaluate(async id=>{const {loadRuntimeProject}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js');const value=await loadRuntimeProject(id);return value?{...value,checkpoint_bytes:Array.from(value.checkpoint_bytes??[])}:null;},projectId);}
async function presentation(){return page.evaluate(()=>{const canvas=document.querySelector('[data-testid="bogae-canvas"]');return{hash:canvas.dataset.committedStateHash??'',madi:document.querySelector('[data-testid="current-madi"]').textContent,pixels:canvas.toDataURL(),renderer:canvas.dataset.renderSource??''};});}
async function result(){return JSON.parse(await page.locator('#logical-input-panel [data-field="result"]').textContent());}
try{
 await openPage();
 if(checkGenericGuide){
  await page.getByTestId('project-rail-toggle').click();
  await page.locator('#project-list [data-project-id="local_linear_slope_v1"]').click();
  const linearSource=fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/samples/alpha2_linear_composition.ddn'),'utf8');
  await page.waitForFunction(source=>document.querySelector('#source-editor').value===source,linearSource);
  if(await page.getByTestId('project-rail-toggle').getAttribute('aria-expanded')==='true')await page.getByTestId('project-rail-toggle').click();
  assert.match(await page.getByTestId('example-guide').textContent(),/정적인 직선/,'the official linear lesson keeps its own pre-run guide');
 }
 await authoredPanel();await page.locator('[data-testid="authored-project-drawer"] input[type=file]').setInputFiles(authoredFile);
 await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('파일을 열었습니다'));
 await page.locator('[data-authored-save]').click();await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
 const uiAuthored=await page.locator('[data-authored-contents]').inputValue();assert.equal(JSON.parse(uiAuthored).authored_identity.logical_dependency_id,projectId);
 await closeAuthored();
 if(checkGenericGuide){
  await page.screenshot({path:path.join(output,'generic-import-before-run.png')});
  assert.equal(await page.getByTestId('example-guide').textContent(),'','a generic imported project must not inherit the linear lesson guide');
 }
 await bindCurrent();
 await page.getByTestId('lifecycle-run').click();
 await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame?.input?.madi>=0;}catch{return false;}});
 await page.getByTestId('lifecycle-run').click();await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]').dataset.action==='resume');
 let value=await result();while(value.frame.input.madi<2){const previous=value.frame.input.madi;await page.getByTestId('lifecycle-step').click();await page.waitForFunction(old=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame?.input?.madi===old+1;}catch{return false;}},previous);value=await result();}
 const count=value.frame.input.madi+1;assert.equal(value.resources.fixed64.진행,String(count));
 await authoredPanel();await page.locator('[data-authored-runtime-save]').click();
 await page.waitForFunction(()=>{const text=document.querySelector('[data-authored-status]').textContent;return text.includes('이력과 실행 상태를 함께 저장')||text.startsWith('저장 자료 오류');});
 assert.ok((await page.locator('[data-authored-status]').textContent()).includes('이력과 실행 상태를 함께 저장'));
 const saved=await record();assert(saved?.checkpoint_bytes?.length);await closeAuthored();
 const before=await presentation();assert.match(before.hash,/^blake3:/);await page.screenshot({path:path.join(output,'before-runtime-abort.png')});
 await page.getByTestId('bogae-canvas').focus();await page.keyboard.down('ArrowLeft');
 await page.getByTestId('lifecycle-step').click();
 await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('FATAL:CHARIM_INDEX_OUT_OF_RANGE'));
 const after=await presentation();await page.screenshot({path:path.join(output,'after-runtime-abort.png')});
 const framePreserved=before.hash===after.hash&&before.madi===after.madi&&before.pixels===after.pixels;
 assert.deepEqual(await record(),saved,'runtime abort leaves the old normal IndexedDB head and all saved bytes unchanged');
 await authoredPanel();await page.locator('[data-authored-runtime-save]').click();
 await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.startsWith('저장 자료 오류'));
 assert.deepEqual(await record(),saved,'saving an aborted session must not replace the normal save');await closeAuthored();
 await browser.close();browser=await chromium.launchPersistentContext(profile,options);await openPage();await authoredPanel();await page.locator('[data-authored-reopen]').click();
 await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));await closeAuthored();
 if(checkGenericGuide)assert.equal(await page.getByTestId('example-guide').textContent(),'','a generic source reopened in a new Chrome process must not inherit the linear guide');
 await bindCurrent();await authoredPanel();
 await page.locator('[data-authored-runtime-reopen]').click();
 await page.waitForFunction(()=>{const text=document.querySelector('[data-authored-status]').textContent;return text.includes('복원')||text.startsWith('저장 자료 오류');});
 assert.ok(!(await page.locator('[data-authored-status]').textContent()).startsWith('저장 자료 오류'));
 await closeAuthored();assert.equal((await presentation()).hash,before.hash,'cold reopening restores the normal pre-failure world');
 await page.getByTestId('lifecycle-step').click();
 await page.waitForFunction(n=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame?.input?.madi===n;}catch{return false;}},count);
 const continued=await result();assert.equal(continued.resources.fixed64.진행,String(count+1));
 const vm=wasm.DdnLogicalInputVm.from_authored_project(uiAuthored,wasm.DdnLogicalInputVm.bind_authored_project_json(uiAuthored,binding));
 const held=JSON.parse(binding).initial_held;for(let madi=0;madi<count+1;madi++)vm.step_json(JSON.stringify({madi,events:[],held}));
 assert.equal(continued.state_hash,JSON.parse(vm.summary_json()).state_hash,'actual UI cold continuation equals uninterrupted product execution');vm.free();
 await page.screenshot({path:path.join(output,'cold-normal-continuation.png')});
 await page.getByTestId('lifecycle-stop').click();
 await page.waitForFunction(()=>document.querySelector('[data-testid="bogae-canvas"]').dataset.committedStateHash==='');
 assert.deepEqual(await record(),saved,'explicit close clears the live picture, not the saved normal head');
 assert.equal(errors.length,0,errors.join('\n'));assert(servedWasm.length&&servedWasm.every(h=>h===hash(wasmBytes)));
 report={status:framePreserved?'PASS_LATE_RUNTIME_WORKBENCH_FAILURE_PRESERVATION':'FAIL_LAST_COMMITTED_PRESENTATION_CLEARED',root,fixture,output,target_kind:override?'UNCOMMITTED_SINGLE_UI_OVERLAY_NOT_EXACT_ZIP':'EXACT_EXTRACTED_PACKAGE',panel_sha256:servedPanel[0],wasm_sha256:hash(wasmBytes),prefix_madi_count:count,generic_guide_check:checkGenericGuide?'IMPORT_AND_COLD_REOPEN_EMPTY_OFFICIAL_LINEAR_PRESERVED':'NOT_RUN',frame_preserved:framePreserved,before:{...before,pixels_sha256:hash(Buffer.from(before.pixels)),pixels:undefined},after:{...after,pixels_sha256:hash(Buffer.from(after.pixels)),pixels:undefined},old_storage_preserved:true,aborted_save_rejected:true,cold_normal_equals_continuous:true,explicit_close_clears_live_projection:true,errors,Human:'NOT_RUN',independent:'NOT_RUN'};
 if(!framePreserved)process.exitCode=1;
}catch(e){report={status:'FAILED',error:String(e.stack??e),root,fixture,output,errors};process.exitCode=1;try{await page?.screenshot({path:path.join(output,'unexpected-failure.png')});}catch{}}
finally{await browser.close();await new Promise(resolve=>server.close(resolve));if(report)fs.writeFileSync(path.join(output,'REPORT.json'),JSON.stringify(report,null,2));}
console.log(JSON.stringify(report));
