// A3-P4: real keyboard, Worker/Rust reactions and actual downloaded record.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE && process.env.DDN_NATIVE_DIR);
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve('.'),out=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-reaction-browser-'));
const wasmDir=path.resolve(process.env.DDN_WASM_DIR || 'solutions/seamgrim_ui_mvp/ui/wasm');
const sha=x=>createHash('sha256').update(x).digest('hex');
const errors=[],served=[];
const server=http.createServer((req,res)=>{
  const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const local=path.resolve(root,relative);
  if(!local.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  // Bind the actual Worker import to the explicitly selected build. Never
  // report a fresh external WASM hash while serving the older UI bundle.
  const file=relative.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')?
    path.join(wasmDir,path.basename(local)):local;
  try{const bytes=fs.readFileSync(file);if(file.endsWith('.wasm'))served.push(sha(bytes));
    res.writeHead(200,{'content-type':file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':'application/json'}).end(bytes);
  }catch{res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const page=await browser.newPage({viewport:{width:1450,height:1100}});
await page.addInitScript(()=>{
  // Observe actual Canvas draw calls; never replace computation or rendering.
  window.__A3_REACTION_CANVAS_TEXTS=[];
  const clear=CanvasRenderingContext2D.prototype.clearRect,fill=CanvasRenderingContext2D.prototype.fillText;
  CanvasRenderingContext2D.prototype.clearRect=function(...args){
    if(this.canvas.id==='canvas-bogae')window.__A3_REACTION_CANVAS_TEXTS=[];
    return clear.apply(this,args);
  };
  CanvasRenderingContext2D.prototype.fillText=function(text,...args){
    if(this.canvas.id==='canvas-bogae')window.__A3_REACTION_CANVAS_TEXTS.push(String(text));
    return fill.call(this,text,...args);
  };
});
page.on('pageerror',e=>errors.push(String(e)));page.on('requestfailed',r=>errors.push(r.url()));
page.on('response',r=>{if(r.status()>=400)errors.push(`${r.status()} ${r.url()}`);});
page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
const state=()=>page.evaluate(()=>JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent));
const paused=()=>page.waitForFunction(()=>{const button=document.querySelector('[data-testid="lifecycle-run"]');return button.dataset.action==='resume'&&!button.disabled;});
try{
  await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  const source=fs.readFileSync('samples/alpha3_logical_reaction.ddn','utf8');
  const binding=fs.readFileSync('solutions/seamgrim_ui_mvp/first_steps_alpha3/입력/20A.binding.json','utf8');
  const actions=JSON.stringify({schema:'ddn.input_action_map.v1',actions:[
    {action_id:'더함',input_snapshot:{last_key:'ArrowLeft'}},
    {action_id:'곱함',input_snapshot:{last_key:'ArrowRight'}},
  ]});
  await page.locator('[data-testid="global-first-steps"]').click();
  if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')!=='true')await page.locator('[data-testid="project-rail-toggle"]').click();
  for(const id of ['17A','17B','18A','18B','19A','MOD17','20A']){
    await page.locator(`#project-list [data-project-id="first_steps_alpha3_${id}"]`).click();
    await page.waitForFunction(id=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId===`first_steps_alpha3_${id}`,id);
    const catalog=JSON.parse(fs.readFileSync('solutions/seamgrim_ui_mvp/first_steps_alpha3/검증목록.json','utf8'));
    const row=[...catalog.cases,...catalog.modifications].find(row=>row.id===id);
    const exact=fs.readFileSync(path.resolve('solutions/seamgrim_ui_mvp/first_steps_alpha3',row.file),'utf8');
    await page.waitForFunction(exact=>document.querySelector('#source-editor').value===exact,exact);
  }
  assert.equal(await page.locator('#source-editor').inputValue(),source);
  const pdfs=await page.locator('.first-steps-documents a').evaluateAll(links=>links.map(link=>link.href));
  for(const url of pdfs){const response=await page.request.get(url);assert.equal(response.status(),200);assert.equal((await response.body()).subarray(0,5).toString(),'%PDF-');}
  if(await page.locator('[data-testid="project-rail-toggle"]').getAttribute('aria-expanded')==='true')await page.locator('[data-testid="project-rail-toggle"]').click();
  const panel=page.locator('#logical-input-panel'),primary=page.locator('[data-testid="lifecycle-run"]');
  await page.locator('[data-testid="first-steps-guide"]').evaluate(guide=>guide.open=true);
  await page.locator('[data-testid="first-steps-input-contract"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel').logicalExecution.ownsCurrentSource());
  await primary.click();
  await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame?.input?.madi>=0;}catch{return false;}});
  await primary.click();await paused();
  const before=await state();assert.equal(before.resources.fixed64.계산값,'2');
  await panel.locator('[data-field="region"]').focus();
  await page.keyboard.down('ArrowLeft');await page.keyboard.down('ArrowRight');
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).resources?.fixed64?.계산값==='6';}catch{return false;}});
  const step=await state();assert.equal(step.resources.fixed64.마디결과,'6');
  assert.deepEqual(step.frame.input.events.map(e=>e.channel),['더함','곱함']);
  assert.equal(step.reaction_support,true);
  const download=page.waitForEvent('download');await panel.locator('[data-action="seal"]').click();
  const tape=path.join(out,'keyboard-reaction.tape');await (await download).saveAs(tape);
  const expected=await state(),sourceFile=path.join(out,'reaction.ddn'),bindingFile=path.join(out,'binding.json');
  fs.writeFileSync(sourceFile,source);fs.writeFileSync(bindingFile,await panel.locator('[data-field="binding"]').inputValue());
  for(const alias of ['ddonilang','ddonirang','또니랑']){
    const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,alias+'.exe'),['논리입력재생',sourceFile,tape,bindingFile],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
    assert.equal(result.status,0,result.stderr||String(result.error));assert.deepEqual(JSON.parse(result.stdout),expected);
  }
  await page.locator('[data-testid="lifecycle-stop"]').click();
  await panel.locator('[data-field="replay"]').setInputFiles(tape);
  await page.waitForFunction(n=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count===n;}catch{return false;}},expected.madi_count);
  assert.deepEqual(await state(),expected);
  await page.waitForFunction(()=>window.__A3_REACTION_CANVAS_TEXTS.includes('반응결과 | 매마디결과')&&window.__A3_REACTION_CANVAS_TEXTS.includes('6 | 6'));
  const rendered=await page.evaluate(()=>window.__A3_REACTION_CANVAS_TEXTS);
  await page.screenshot({path:path.join(out,'keyboard-reaction.png'),fullPage:true});
  const wasmHash=sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm')));
  assert.ok(served.length);assert.ok(served.every(hash=>hash===wasmHash));assert.deepEqual(errors,[]);
  const report={status:'PASS',decision:'A3-P4 PROVISIONAL',actual_chrome_worker_wasm:true,wasm_sha256:wasmHash,native_aliases:3,state_hash:expected.state_hash,tape_sha256:sha(fs.readFileSync(tape)),madi_count:expected.madi_count,rendered_canvas_texts:rendered,errors};
  fs.writeFileSync(path.join(out,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,evidence_directory:out}));
}catch(error){
  console.error('REACTION_BROWSER_EVIDENCE',out);
  fs.writeFileSync(path.join(out,'failure.json'),JSON.stringify({error:String(error),snapshot:await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot()).catch(()=>null),errors},null,2));
  await page.screenshot({path:path.join(out,'failure.png'),fullPage:true});throw error;
}finally{await browser.close();await new Promise(resolve=>server.close(resolve));}
