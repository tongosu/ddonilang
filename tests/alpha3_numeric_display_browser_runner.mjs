// A3-P1: verify the real Worker/WASM and rendered output, not JS rounding.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {verifyTensorPhysicalCase} from './alpha3_tensor_physical_oracle.mjs';
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE && process.env.DDN_WASM_DIR);
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve(process.env.DDN_BROWSER_ROOT??'.'),wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const sourceFile=process.env.DDN_DISPLAY_SOURCE_FILE??'solutions/seamgrim_ui_mvp/samples/alpha3_numeric_display.ddn';
const originalSourceBytes=fs.readFileSync(sourceFile);
const sourceBytes=process.env.DDN_FILE_CRLF==='1'
  ?Buffer.from(originalSourceBytes.toString('utf8').replace(/\r\n?|\n/g,'\r\n')):originalSourceBytes;
const expectedDisplay=process.env.DDN_EXPECTED_DISPLAY?JSON.parse(process.env.DDN_EXPECTED_DISPLAY):['19.02','-19.02','19.019999999','차림[19.02, -19.02]'];
const decision=process.env.DDN_DISPLAY_DECISION??'A3-P1 PROVISIONAL';
const expectedError=process.env.DDN_EXPECTED_ERROR_CODE??'';
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-display-browser-'));
const errors=[],wasmResponses=[],servedWasm=[];
const server=http.createServer((req,res)=>{
  const rel=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const local=path.resolve(root,rel);
  if(!local.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  const file=rel.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')?path.join(wasmDir,path.basename(local)):local;
  try{
    const type=file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.css')?'text/css':file.endsWith('.html')?'text/html':'application/json';
    const bytes=fs.readFileSync(file);
    if(file.endsWith('ddonirang_tool_bg.wasm'))servedWasm.push(createHash('sha256').update(bytes).digest('hex'));
    res.writeHead(200,{'content-type':type}).end(bytes);
  }catch{res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
let page;
try{
  page=await browser.newPage({viewport:{width:1500,height:1000}});
  if(process.env.DDN_THREE_MADI_FLOW==='1')await page.addInitScript(()=>{
    // Slow only host scheduling so actual Pause/Step controls can be used.
    // No world input, dt, source, Worker result, or WASM state is replaced.
    const original=window.setTimeout.bind(window);
    window.setTimeout=(fn,delay,...args)=>original(fn,Number(delay)<20?1000:delay,...args);
  });
  page.on('pageerror',e=>errors.push(String(e)));
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text());});
  page.on('requestfailed',r=>errors.push(r.url()));
  page.on('response',r=>{if(r.status()>=400)errors.push(r.url());if(r.url().includes('ddonirang_tool_bg.wasm'))wasmResponses.push(r);});
  await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  if(process.env.DDN_OPEN_SOURCE_FILE==='1'){
    await page.locator('[data-testid="source-file-input"]').setInputFiles({name:path.basename(sourceFile),mimeType:'text/plain',buffer:sourceBytes});
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId==='opened_v25_source'
      && document.querySelector('[data-testid="source-file-input"]').value==='');
    assert.equal(await page.locator('#source-editor').inputValue(),sourceBytes.toString('utf8').replace(/\r\n?/g,'\n'));
    const opened=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert.equal(opened.activeRunId,'','file open never executes');
    assert.equal(opened.activeFrame,null,'file open never fabricates a frame');
  }else await page.locator('#source-editor').fill(fs.readFileSync(sourceFile,'utf8'));
  await page.keyboard.press('Escape');
  let result;
  if(expectedError){
    await page.waitForFunction(()=>{
      const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return !document.querySelector('[data-testid="lifecycle-run"]').disabled || state.diagnostics.length>0;
    });
    const before=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    if(!await page.locator('[data-testid="lifecycle-run"]').isDisabled())await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(code=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics.some(d=>d.code===code),expectedError);
    const after=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert.deepEqual(after.activeFrame,before.activeFrame,'failed source publishes no frame/world');
    assert.equal(after.activeRunId,before.activeRunId,'failed source publishes no run');
    result={status:'EXPLICIT_REJECT',code:expectedError,frame_preserved:true,diagnostics:after.diagnostics};
  }else{
    await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
    await page.locator('[data-testid="lifecycle-run"]').click();
    if(process.env.DDN_THREE_MADI_FLOW==='1'){
      await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.output_display?.texts?.length===2);
      await page.locator('[data-testid="lifecycle-run"]').click();
      await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-step"]').disabled);
      assert.equal((await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot())).activeFrame.wasm_state.output_display.texts.length,2);
      for(const length of [4,6]){
        await page.locator('[data-testid="lifecycle-step"]').click();
        await page.waitForFunction(n=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.output_display?.texts?.length===n,length);
        await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-step"]').disabled);
      }
    }
    if(process.env.DDN_TENSOR_PHYSICAL_CASE)
      await page.waitForFunction(()=>Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.resources?.value_json));
    else await page.waitForFunction(n=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.output_display?.texts?.length===n,expectedDisplay.length);
    result=await page.evaluate(()=>{
    const frame=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame;
    const snapshot=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
    return {state_hash:frame.state_hash,display:frame.wasm_state.output_display,
      admitted_source_sha256:snapshot.runs.find(run=>run.run_id===snapshot.activeRunId)?.shared_runtime_result?.source_identity,
      resources:frame.wasm_state.resources,
      raw:frame.wasm_state.resources.value_json.output_log,
      canvas:document.querySelector('#canvas-bogae').dataset.consoleOutput};
  });
  if(!process.env.DDN_TENSOR_PHYSICAL_CASE)assert.deepEqual(result.display.texts,expectedDisplay);
  for(const text of expectedDisplay)assert.ok(String(result.canvas).includes(text),'main canvas receives Rust output '+text);
  if(!process.env.DDN_DISPLAY_SOURCE_FILE)assert.equal(result.raw[0].text,'19.019999999');
  if(process.env.DDN_TENSOR_PHYSICAL_CASE){
    result.physical_oracle=verifyTensorPhysicalCase(process.env.DDN_TENSOR_PHYSICAL_CASE,result.resources);
  }else assert.equal(await page.locator('.bogae-panel .view-roles').isVisible(),false,
    'text output needs no meaningless spatial/overlay/source toolbar');
  assert.equal(result.admitted_source_sha256,createHash('sha256').update(sourceBytes).digest('hex'),
    'common Rust execution envelope must bind actual file bytes, not textarea-normalized display');
  }
  assert.equal(errors.length,0,JSON.stringify(errors));
  const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
  const expectedWasm=sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm')));
  for(const hash of servedWasm)assert.equal(hash,expectedWasm);
  assert.ok(servedWasm.length && wasmResponses.length);
  await page.locator('.bogae-panel').screenshot({path:path.join(dir,'typed-display.png')});
  // Return to a real authored drawing; text-only hiding is not sticky.
  while(await page.locator('dialog[open]').count())await page.keyboard.press('Escape');
  await page.locator('[data-testid="project-rail-toggle"]').click();
  await page.locator('#project-list [data-project-id="local_linear_slope_v1"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId==='local_linear_slope_v1'
    && window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().readiness.source.ready);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae').dataset.authoredBogaeStatus==='ready');
  assert.equal(await page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]').isVisible(),true);
  await page.locator('.bogae-panel').screenshot({path:path.join(dir,'returned-to-drawing.png')});
  const transport=[];
  if(process.env.DDN_VERIFY_FILE_ATOMICITY==='1'){
    for(const [name,bytes,code] of [
      ['BOM.ddn',Buffer.from([0xef,0xbb,0xbf,65]),'E_SOURCE_UTF8_BOM_FORBIDDEN'],
      ['invalid.ddn',Buffer.from([0xff]),'E_SOURCE_UTF8_INVALID'],
    ]){
      const before=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      const text=await page.locator('#source-editor').inputValue();
      await page.locator('[data-testid="source-file-input"]').setInputFiles({name,mimeType:'text/plain',buffer:bytes});
      await page.waitForFunction(code=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics.some(d=>d.code===code),code);
      const after=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      assert.equal(after.projectId,before.projectId);
      assert.equal(after.activeRunId,before.activeRunId);
      assert.deepEqual(after.activeFrame,before.activeFrame);
      assert.equal(await page.locator('#source-editor').inputValue(),text);
      transport.push({code,active_state_preserved:true});
      while(await page.locator('dialog[open]').count())await page.keyboard.press('Escape');
    }
    // Delay only File I/O, not parsing, execution, or product-generated results.
    await page.evaluate(()=>{
      const original=File.prototype.arrayBuffer;
      File.prototype.arrayBuffer=function(){
        if(this.name!=='delayed.ddn')return original.call(this);
        return new Promise(resolve=>{window.__releaseSourceRead=async()=>resolve(await original.call(this));});
      };
    });
    await page.locator('[data-testid="source-file-input"]').setInputFiles({name:'delayed.ddn',mimeType:'text/plain',buffer:Buffer.from('1 보여주기.')});
    await page.waitForFunction(()=>typeof window.__releaseSourceRead==='function');
    const newer='2 보여주기.';
    await page.locator('#source-editor').fill(newer);
    await page.evaluate(()=>window.__releaseSourceRead());
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics.some(d=>d.code==='E_SOURCE_FILE_STALE'));
    assert.equal(await page.locator('#source-editor').inputValue(),newer);
    assert.equal((await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot())).projectId,'local_linear_slope_v1');
    transport.push({code:'E_SOURCE_FILE_STALE',newer_source_preserved:true});
  }
  fs.writeFileSync(path.join(dir,'report.json'),JSON.stringify({status:'PASS',decision,source_file:sourceFile,browser_root:root,opened_file:process.env.DDN_OPEN_SOURCE_FILE==='1',source_bytes_sha256:sha(sourceBytes),crlf_variant:process.env.DDN_FILE_CRLF==='1',wasm_sha256:expectedWasm,result,transport,errors},null,2));
  console.log(JSON.stringify({status:'PASS',evidence_directory:dir,wasm_sha256:expectedWasm,result}));
}catch(error){
  const snapshot=await page?.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot()).catch(()=>null);
  fs.writeFileSync(path.join(dir,'failure.json'),JSON.stringify({error:String(error),snapshot,errors},null,2));
  await page?.screenshot({path:path.join(dir,'failure.png')});
  console.error('FAILURE_EVIDENCE='+dir);throw error;
}finally{await browser.close();await new Promise(resolve=>server.close(resolve));}
