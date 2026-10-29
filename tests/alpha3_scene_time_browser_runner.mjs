// A3-TIME-P1 actual Chrome -> Worker -> WASM -> authored canvas.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE && process.env.DDN_WASM_DIR);
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve('.'),wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-time-browser-'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const wasmHash=sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm')));
const errors=[],served=[];
const server=http.createServer((req,res)=>{
  const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const local=path.resolve(root,rel);
  if(!local.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  const file=rel.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')?path.join(wasmDir,path.basename(local)):local;
  try {
    const bytes=fs.readFileSync(file);if(file.endsWith('ddonirang_tool_bg.wasm'))served.push(sha(bytes));
    res.writeHead(200,{'content-type':file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':'application/json'}).end(bytes);
  }catch{res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
let page;const results=[];
try {
  for(const [name,file,entry] of [['scene','alpha3_scene_time.ddn','비교짜임'],['table','alpha3_table_time.ddn','짜임'],['relative','alpha3_scene_relative.ddn','비교짜임']]) {
    page=await browser.newPage({viewport:{width:1500,height:1000}});
    page.on('pageerror',e=>errors.push(String(e)));page.on('requestfailed',r=>errors.push(r.url()));
    page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});page.on('response',r=>{if(r.status()>=400)errors.push(r.url());});
    await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
    await page.locator('#source-editor').fill(fs.readFileSync('solutions/seamgrim_ui_mvp/samples/'+file,'utf8'));await page.keyboard.press('Escape');
    await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
    if(await page.locator('[data-testid="bogae-entry"]').isVisible())await page.locator('[data-testid="bogae-entry"]').fill(entry);
    await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.bogae_projection?.scene_cursor===2);
    const frame=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
    const relations=frame.wasm_state.bogae_projection.candidates[0].artifact.relations;
    assert.equal(relations[1].resolved_madi,name==='relative'?1:0);
    if(name!=='table')assert.equal(relations[1].view.elements[0].center[0],name==='relative'?'8589934592':'4294967296');
    else assert.equal(relations[1].view.table_projection.tables[0].rows[0].cells[0].value_text,'1');
    const side=page.locator(`[data-authored-relation-id="${relations[1].relation_id}"]`);
    assert.equal(await side.isVisible(),true);
    await side.locator('[data-authored-side-close]').click();assert.equal(await side.isVisible(),false);
    await page.locator(`[data-authored-relation-toggle="${relations[1].relation_id}"]`).click();
    assert.equal(await side.isVisible(),true);
    assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),frame,'view controls cannot mutate committed world/history');
    await page.locator('.bogae-panel').screenshot({path:path.join(dir,name+'-time.png')});
    results.push({name,state_hash:frame.state_hash,scene_cursor:2,selected_madi:relations[1].resolved_madi});await page.close();
  }
  assert.equal(errors.length,0,JSON.stringify(errors));assert.ok(served.length);for(const hash of served)assert.equal(hash,wasmHash);
  const report={status:'PASS',decision:'A3-TIME-P1 PROVISIONAL',actual_chrome_worker_wasm:true,wasm_sha256:wasmHash,results,errors};
  fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,evidence_directory:dir}));
}catch(error){
  const snapshot=await page?.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot()).catch(()=>null);
  fs.writeFileSync(path.join(dir,'failure.json'),JSON.stringify({error:String(error),snapshot,errors},null,2));
  await page?.screenshot({path:path.join(dir,'failure.png')}).catch(()=>{});console.error('FAILURE_EVIDENCE='+dir);throw error;
}finally{await browser.close();await new Promise(resolve=>server.close(resolve));}
