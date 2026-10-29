import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve('.'),wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const fixtures=path.resolve(process.env.DDN_PREFIX_FIXTURES),output=path.resolve(process.env.DDN_PREFIX_BROWSER_OUTPUT);
assert(!fs.existsSync(output),'use a new evidence directory');fs.mkdirSync(output,{recursive:true});
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const server=http.createServer((req,res)=>{
  const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  if(relative==='prefix-test.html'){res.writeHead(200,{'content-type':'text/html; charset=utf-8'}).end('<h1>중단된 입력 기록 검증</h1>');return;}
  if(relative==='favicon.ico'){res.writeHead(204).end();return;}
  const file=path.resolve(root,relative);
  if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  if(relative==='solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.manifest.detjson'){
    res.writeHead(200,{'content-type':'application/json'}).end(JSON.stringify({schema:'ddn.wasm_artifact_manifest.v1',
      artifact_id:'wo22.prefix.fixed-source-e41fba5c',
      js_binding_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool.js'))),
      wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),
      js_binding_hash_mode:'raw_bytes_sha256',wasm_hash_mode:'raw_bytes_sha256',js_wasm_reference:'ddonirang_tool_bg.wasm'}));return;
  }
  const actual=relative.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')?path.join(wasmDir,path.basename(file)):file;
  try{const bytes=fs.readFileSync(actual);res.writeHead(200,{'content-type':actual.endsWith('.wasm')?'application/wasm':actual.endsWith('.js')?'text/javascript':actual.endsWith('.css')?'text/css':'application/json'}).end(bytes);}
  catch{res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const origin=`http://127.0.0.1:${server.address().port}`;
assert.notEqual(server.address().port,8790);
let browser;const pageErrors=[],failedRequests=[],badResponses=[];
try{
  browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
  const page=await browser.newPage({viewport:{width:1280,height:900}});
  await page.route('**/*',route=>route.request().url().startsWith(origin+'/')?route.continue():route.abort());
  page.on('pageerror',e=>pageErrors.push(String(e)));
  page.on('requestfailed',r=>failedRequests.push(r.url()));
  page.on('response',r=>{if(r.status()>=400)badResponses.push({url:r.url(),status:r.status()});});
  await page.goto(origin+'/prefix-test.html');
  const source=fs.readFileSync(path.join(fixtures,'logical.ddn'),'utf8');
  const binding=fs.readFileSync(path.join(fixtures,'logical.binding.json'),'utf8');
  await page.evaluate(async({source,binding})=>{
    const {installLogicalInputPanel}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/logical_input_panel.js');
    window.prefixSource=source;
    window.prefixPanel=installLogicalInputPanel({parent:document.body,readSource:()=>window.prefixSource});
    await window.prefixPanel.logicalExecution.selectContract({bindingJson:binding,
      actionsJson:JSON.stringify({schema:'ddn.input_action_map.v1',actions:[{action_id:'왼쪽',input_snapshot:{last_key:'ArrowLeft'}}]}),capacity:32});
  },{source,binding});
  const panel=page.locator('#logical-input-panel');const feedback=panel.locator('[data-field="feedback"]');
  const file=panel.locator('[data-field="replay"]');const checkbox=panel.locator('[data-field="partial-replay"]');
  assert.equal(await checkbox.isChecked(),false);
  await panel.locator('[data-action="start"]').click();
  await page.waitForFunction(()=>document.querySelector('[data-field="result"]').textContent.includes('madi_count'));
  await panel.locator('[data-action="step"]').click();
  await page.waitForFunction(()=>document.querySelector('[data-field="result"]').textContent.includes('"madi": 0'));
  assert.deepEqual(await page.evaluate(()=>window.prefixPanel.logicalExecution.status()),{running:false,busy:false,started:true,failed:false});
  await file.setInputFiles(path.join(fixtures,'logical.tape'));
  await page.waitForFunction(()=>document.querySelector('[data-field="feedback"]').textContent.includes('미완'));
  assert.equal(await checkbox.isChecked(),false);
  assert.deepEqual(await page.evaluate(()=>window.prefixPanel.logicalExecution.status()),{running:false,busy:false,started:true,failed:false},'rejected file must preserve the current VM');
  // The existing VM must really remain usable after refusal, not merely keep
  // a stale scene. Stop its actual three-step prefix using the UI button.
  for(const madi of [1,2]) {
    await panel.locator('[data-action="step"]').click();
    await page.waitForFunction(madi=>document.querySelector('[data-field="result"]').textContent.includes(`"madi": ${madi}`),madi);
  }
  const downloadPromise=page.waitForEvent('download');
  // Exact R1.5 gives seal a genuine normal-finish issuer. Exercise the
  // explicit stop action instead; retain the original exact Incomplete
  // bytes, default refusal and Aborted refusal expectations below.
  const advanced=panel.locator('[data-field="advanced"]');
  if(!await advanced.evaluate(node=>node.open))await advanced.locator(':scope > summary').click();
  assert.deepEqual(await page.evaluate(()=>window.prefixPanel.logicalExecution.status()),
    {running:false,busy:false,started:true,failed:false},'opening recording controls is not input loss');
  await panel.locator('[data-action="stop-recording"]').click();
  const download=await downloadPromise;const recorded=path.join(output,'실제단추_중단기록.tape');await download.saveAs(recorded);
  assert.deepEqual(fs.readFileSync(recorded),fs.readFileSync(path.join(fixtures,'logical.tape')),'mouse stop and WASM producer must preserve the same exact prefix bytes');
  await checkbox.check();await file.setInputFiles(path.join(fixtures,'logical.tape'));
  await page.waitForFunction(()=>document.querySelector('[data-field="feedback"]').textContent==='중단된 기록, 3마디까지');
  const summary=JSON.parse(await panel.locator('[data-field="result"]').textContent());
  assert.deepEqual(summary,JSON.parse(fs.readFileSync(path.join(fixtures,'logical.summary.json'),'utf8')));
  await page.screenshot({path:path.join(output,'중단기록_명시부분재생.png'),fullPage:true});
  await file.setInputFiles(path.join(fixtures,'logical.aborted.tape'));
  await page.waitForFunction(()=>document.querySelector('[data-field="feedback"]').textContent.includes('실패 기록'));
  assert.equal(await checkbox.isChecked(),true);
  const rejection=await feedback.textContent();
  await page.evaluate(()=>window.prefixPanel.logicalExecution.resetForProject());
  assert.equal(await checkbox.isChecked(),false);
  assert.equal(await checkbox.isDisabled(),false);
  assert.equal(await feedback.textContent(),'');
  assert.deepEqual(pageErrors,[]);assert.deepEqual(failedRequests,[]);assert.deepEqual(badResponses,[]);
  fs.writeFileSync(path.join(output,'RESULT.json'),JSON.stringify({claim:'PREFIX_PANEL_CHROME_NOT_NATIVE_CUSTODY_OR_BETA2_PASS',
    wasm_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'))),summary,rejection,
    default_incomplete_rejected:true,rejected_file_preserves_usable_vm:true,mouse_stop_bytes_equal_wasm:true,
    explicit_prefix:true,aborted_rejected:true,project_change_reset:true,
    pageErrors,failedRequests,badResponses},null,2));
  console.log('PASS: Chrome panel default reject → explicit prefix → Aborted reject → project reset');
}finally{await browser?.close();await new Promise(resolve=>server.close(resolve));}
