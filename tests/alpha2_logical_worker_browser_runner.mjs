import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const root=path.resolve('.'), wasmDir=path.resolve(process.env.DDN_WASM_DIR);
const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-logical-browser-'));
const module=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
const binary=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await module.default({module_or_path:binary});
const empty=new module.DdnWasmVm(''); const initialWorld=empty.get_state_hash(); empty.free();
const source=`(매마디)마다 {
  계산값 <- 2. 사건들 <- () 입력사상.사건들.
  사건들마다 {
    { 사건들.종류 == "누름" }이면 {
      { 사건들.채널 == "더함" }이면 { 계산값 <- 계산값 + 1. }
      아니면 { 계산값 <- 계산값 * 2. }.
    }.
  }.
}.`;
const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'alpha2.browser-test@1',
  channel_registry:'alpha2.browser-channels@1',source_revision:`sha256:${createHash('sha256').update(source).digest('hex')}`,
  initial_world:initialWorld,initial_rng:7,recording_policy:'alpha2.browser-recording@1',
  integrity_policy:'ddn.logical_input_tape.v1',madi_hz:60,dt_raw:4294967296,
  channels:{곱함:'Control',더함:'Control'},initial_held:{곱함:false,더함:false}};
const actionMap={schema:'ddn.input_action_map.v1',actions:[
  {action_id:'더함',input_snapshot:{last_key:'ArrowLeft'}},
  {action_id:'곱함',input_snapshot:{last_key:'ArrowRight'}},
]};
const server=http.createServer((req,res)=>{
  const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  if(relative==='test.html') {
    res.writeHead(200,{'content-type':'text/html; charset=utf-8'}).end('<button id="input">논리 입력</button><pre id="result"></pre>'); return;
  }
  const local=path.resolve(root,relative);
  if(!local.startsWith(root+path.sep)) {res.writeHead(403).end();return;}
  const file=relative.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')
    ? path.join(wasmDir,path.basename(local)):local;
  if(relative==='solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.manifest.detjson') {
    // Build an honest ephemeral manifest from the actual isolated bytes; do not
    // reuse the old release identity or bypass the product hash verifier.
    res.writeHead(200,{'content-type':'application/json'}).end(JSON.stringify({
      artifact_id:'alpha2.logical-host.browser-candidate',
      schema:'ddn.wasm_artifact_manifest.v1',
      js_binding_sha256:sha(fs.readFileSync(path.join(wasmDir,'ddonirang_tool.js'))),
      wasm_sha256:sha(binary),js_binding_hash_mode:'raw_bytes_sha256',wasm_hash_mode:'raw_bytes_sha256',
      js_wasm_reference:'ddonirang_tool_bg.wasm',
    }));return;
  }
  try {const bytes=fs.readFileSync(file);res.writeHead(200,{'content-type':file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':'application/json'}).end(bytes);}
  catch {res.writeHead(404).end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browserOptions={headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'};
const browserProfile=path.join(output,'chrome-profile');
let browser=process.env.DDN_AUTHORED_LOGICAL_FIXTURE
  ? await chromium.launchPersistentContext(browserProfile,browserOptions)
  : await chromium.launch(browserOptions);
const page=await browser.newPage(); const errors=[],requests=[];
await page.addInitScript(() => {
  const post = Worker.prototype.postMessage;
  Worker.prototype.postMessage = function (message, ...args) {
    if (window.delayLogicalOperation === message?.operation) {
      window.delayLogicalOperation = null;
      const worker = this, receive = worker.onmessage;
      worker.onmessage = event => {
        if (event.data?.id !== message.id) { receive.call(worker, event); return; }
        window.logicalReplyHeld = true;
        window.releaseLogicalReply = () => {
          worker.onmessage = receive;
          receive.call(worker, event); // Deliver the unchanged actual Worker/WASM reply.
        };
      };
    }
    return post.call(this, message, ...args);
  };
});
page.on('pageerror',error=>errors.push(String(error)));
page.on('requestfailed',request=>requests.push(request.url()));
const badResponses=[];
page.on('response',response=>{if(response.status()>=400) badResponses.push(response.url());});
try {
  await page.goto(`http://127.0.0.1:${server.address().port}/test.html`);
  if(process.env.DDN_AUTHORED_LOGICAL_FIXTURE) {
    const fixture=path.resolve(process.env.DDN_AUTHORED_LOGICAL_FIXTURE);
    const authored=fs.readFileSync(path.join(fixture,'authored-project.json'),'utf8');
    const first=JSON.parse(fs.readFileSync(path.join(fixture,'initial-input.json'),'utf8'));
    const next=JSON.parse(fs.readFileSync(path.join(fixture,'next-input.json'),'utf8'));
    const expected=JSON.parse(fs.readFileSync(path.join(fixture,'continuous-expected.json'),'utf8'));
    // Preserve the admitted channels for interactive authored projects. The
    // older zero-channel counter needs only a host test channel, not new source.
    const panelBinding=Object.keys(first.binding.channels).length?first.binding:
      {...first.binding,channels:{확인:'Control'},initial_held:{확인:false}};
    const panelActions=process.env.DDN_AUTHORED_LOGICAL_ACTION_MAP
      ? JSON.parse(fs.readFileSync(process.env.DDN_AUTHORED_LOGICAL_ACTION_MAP,'utf8'))
      : {schema:'ddn.input_action_map.v1',actions:[{
        action_id:Object.keys(panelBinding.channels)[0],input_snapshot:{last_key:'ArrowLeft'}}]};
    const actual=await page.evaluate(async ({authored,first,next})=>{
      const {createRuntimeBridgeV1}=await import('/solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js');
      const {commitRuntimeProject,loadRuntimeProject,runtimeStorageRecord}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js');
      const bridge=createRuntimeBridgeV1();
      const session=await bridge.createLogicalSession({authoredProjectJson:authored,bindingJson:JSON.stringify(first.binding)});
      for(const snapshot of first.snapshots)await session.step(snapshot);
      const before=await session.summary();
      const artifact=await session.exportSavedProjectArtifact(new Uint8Array(),new Uint8Array([2]));
      if(artifact!==await session.exportSavedProjectArtifact(new Uint8Array(),new Uint8Array([2])))throw new Error('export changed');
      const projectId=JSON.parse(authored).authored_identity.logical_dependency_id;
      const checkpoint=await session.exportProjectCheckpoint(new Uint8Array(),new Uint8Array([2]));
      const record=runtimeStorageRecord(projectId,'','02',artifact,checkpoint);
      await commitRuntimeProject(record);
      const stored=await loadRuntimeProject(projectId);
      for(const snapshot of next.snapshots)await session.step(snapshot);
      const summary=await session.summary();
      await session.close();
      document.querySelector('#result').textContent=JSON.stringify({madi:summary.madi_count,stored:stored.artifact_text===artifact});
      return {before,summary,stored_exact:stored.artifact_text===artifact,projectId,artifact};
    },{authored,first,next});
    assert.deepEqual(actual.summary,expected);
    assert.equal(actual.before.madi_count,first.snapshots.length);assert.equal(actual.stored_exact,true);
    await page.evaluate(async ({authored,panelBinding,panelActions})=>{
      const {installLogicalInputPanel}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/logical_input_panel.js');
      window.authoredForPanel=authored;
      window.logicalProjectPanel=installLogicalInputPanel({parent:document.body,
        readSource:()=>JSON.parse(window.authoredForPanel).source_snapshot.files[1].text,
        readAuthoredProject:()=>window.authoredForPanel});
      // The keyboard host requires an explicit mapped channel; the Native
      // fixture's zero-channel request remains unchanged for parity above.
      await window.logicalProjectPanel.logicalExecution.selectContract({bindingJson:JSON.stringify(panelBinding),
        actionsJson:JSON.stringify(panelActions),capacity:32});
    },{authored,panelBinding,panelActions});
    const logicalProjectPanel=page.locator('#logical-input-panel');
    await logicalProjectPanel.locator('[data-action="start"]').click();
    await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('madi_count'));
    await logicalProjectPanel.locator('[data-action="step"]').click();
    await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 0'));
    const panelArtifact=await page.evaluate(()=>window.logicalProjectPanel.logicalExecution.exportSavedProjectArtifact(new Uint8Array(),new Uint8Array([2])));
    assert.equal(JSON.parse(panelArtifact).schema,'seamgrim.project_snapshot_artifact.v1');
    await logicalProjectPanel.locator('[data-action="step"]').click();
    await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 1'));
    await page.screenshot({path:path.join(output,'authored-project-panel-running.png')});
    await logicalProjectPanel.locator('[data-action="stop"]').click();
    await page.screenshot({path:path.join(output,'authored-worker-export.png')});
    await browser.close();
    browser=await chromium.launchPersistentContext(browserProfile,browserOptions);
    const reopened=await browser.newPage();
    reopened.on('pageerror',error=>errors.push(String(error)));
    reopened.on('requestfailed',request=>requests.push(request.url()));
    reopened.on('response',response=>{if(response.status()>=400)badResponses.push(response.url());});
    await reopened.goto(`http://127.0.0.1:${server.address().port}/test.html`);
    const reopenedRecord=await reopened.evaluate(async projectId=>{
      const {loadRuntimeProject}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js');
      return loadRuntimeProject(projectId);
    },actual.projectId);
    assert.equal(reopenedRecord.artifact_text,actual.artifact,'browser restart must preserve the exact stored carrier');
    const cold=await reopened.evaluate(async ({projectId,binding,next})=>{
      const {loadRuntimeProject,commitRuntimeProject,runtimeStorageRecord}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js');
      const {createRuntimeBridgeV1}=await import('/solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js');
      const record=await loadRuntimeProject(projectId),bridge=createRuntimeBridgeV1();
      const restore=async b=>bridge.createLogicalSession({sourceText:'',bindingJson:JSON.stringify(b),
        savedCheckpoint:{...record,parent_bytes:[],revision_bytes:[2]}});
      let rejected=false;
      try{const bad=await restore({...binding,initial_rng:binding.initial_rng^1});await bad.close();}
      catch{rejected=true;}
      if(!rejected)throw new Error('foreign seed accepted');
      const session=await restore(binding);
      for(const snapshot of next.snapshots)await session.step(snapshot);
      const summary=await session.summary();
      const artifact=await session.exportSavedProjectArtifact(new Uint8Array([2]),new Uint8Array([3]));
      const checkpoint=await session.exportProjectCheckpoint(new Uint8Array([2]),new Uint8Array([3]));
      await commitRuntimeProject(runtimeStorageRecord(projectId,'02','03',artifact,checkpoint));
      await session.close();
      const second=await bridge.createLogicalSession({sourceText:'',bindingJson:JSON.stringify(binding),
        savedCheckpoint:{artifact_text:artifact,checkpoint_bytes:checkpoint,parent_bytes:[2],revision_bytes:[3]}});
      const secondSummary=await second.summary();await second.close();
      return {summary,secondSummary,foreignSeedRejected:rejected};
    },{projectId:actual.projectId,binding:first.binding,next});
    assert.deepEqual(cold.summary,expected);
    assert.deepEqual(cold.secondSummary,expected);
    const panelSetup=async target=>target.evaluate(async ({authored,binding,actions})=>{
      const {installLogicalInputPanel}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/logical_input_panel.js');
      window.authoredForPanel=authored;
      window.logicalProjectPanel=installLogicalInputPanel({parent:document.body,
        readSource:()=>JSON.parse(window.authoredForPanel).source_snapshot.files[1].text,
        readAuthoredProject:()=>window.authoredForPanel});
      await window.logicalProjectPanel.logicalExecution.selectContract({
        bindingJson:JSON.stringify(binding),actionsJson:JSON.stringify(actions),capacity:32});
    },{authored,binding:panelBinding,actions:panelActions});
    await panelSetup(reopened);
    await reopened.locator('#logical-input-panel [data-action="start"]').click();
    await reopened.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('madi_count'));
    await reopened.locator('#logical-input-panel [data-action="step"]').click();
    await reopened.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 0'));
    await reopened.evaluate(async projectId=>{
      const context=await window.logicalProjectPanel.logicalExecution.captureStorageContext();
      const exported=await context.export(new Uint8Array([3]),new Uint8Array([4]));
      const {runtimeStorageRecord,commitRuntimeProject}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js');
      const record=runtimeStorageRecord(projectId,'03','04',exported.artifact,exported.checkpoint);
      const staged=await window.logicalProjectPanel.logicalExecution.prepareStoredCheckpoint(record);
      try{await commitRuntimeProject(record);context.unchanged();}finally{await staged.close();}
    },actual.projectId);
    await reopened.locator('#logical-input-panel [data-action="step"]').click();
    await reopened.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 1'));
    const panelExpected=await reopened.evaluate(async()=>{
      const context=await window.logicalProjectPanel.logicalExecution.captureStorageContext();return context.summary();
    });
    await browser.close();
    browser=await chromium.launchPersistentContext(browserProfile,browserOptions);
    const panelCold=await browser.newPage();
    panelCold.on('pageerror',error=>errors.push(String(error)));
    panelCold.on('requestfailed',request=>requests.push(request.url()));
    await panelCold.goto(`http://127.0.0.1:${server.address().port}/test.html`);
    await panelSetup(panelCold);
    await panelCold.evaluate(async projectId=>{
      const {loadRuntimeProject}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js');
      const record=await loadRuntimeProject(projectId);
      const staged=await window.logicalProjectPanel.logicalExecution.prepareStoredCheckpoint(record);
      await staged.publish();
    },actual.projectId);
    await panelCold.locator('#logical-input-panel [data-action="step"]').click();
    await panelCold.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 1'));
    const panelActual=await panelCold.evaluate(async()=>{
      const context=await window.logicalProjectPanel.logicalExecution.captureStorageContext();return context.summary();
    });
    assert.deepEqual(panelActual,panelExpected);
    await panelCold.screenshot({path:path.join(output,'cold-panel-restored-next-madi.png')});
    const openWorkbench=async target=>{
      await target.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
      await target.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
      const menu=target.locator('[data-testid="project-more-menu"]');
      await menu.locator('summary').click();await target.locator('[data-testid="authored-project-open"]').click();
      await target.locator('[data-testid="authored-project-drawer"] input[type=file]').setInputFiles({name:'authored.json',mimeType:'application/json',buffer:Buffer.from(authored)});
      await target.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('파일을 열었습니다'));
      await target.locator('[data-authored-close]').click();
      if(!await menu.evaluate(node=>node.open))await menu.locator('summary').click();
      await target.locator('[data-testid="logical-record-open"]').click();
      const panel=target.locator('#logical-input-panel');
      await panel.locator('[data-field="advanced"] > summary').click();
      await panel.locator('[data-field="contract-editor"] > summary').click();
      await panel.locator('[data-field="binding"]').fill(JSON.stringify(panelBinding));
      await panel.locator('[data-field="actions"]').fill(JSON.stringify(panelActions));
      await panel.locator('[data-field="capacity"]').fill('32');
      await panel.locator('[data-action="bind-source"]').click();
      await target.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('아직 세계 마디'));
      await target.locator('[data-testid="logical-record-close"]').click();
      if(!await menu.evaluate(node=>node.open))await menu.locator('summary').click();
      await target.locator('[data-testid="authored-project-open"]').click();
      await target.locator('[data-authored-runtime-reopen]').click();
      await target.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('논리 입력 실행을 복원')||document.querySelector('[data-authored-status]').textContent.includes('오류'));
      assert.match(await target.locator('[data-authored-status]').textContent(),/논리 입력 실행을 복원/);
      await target.locator('[data-authored-close]').click();
    };
    await openWorkbench(panelCold);
    await panelCold.locator('[data-testid="lifecycle-step"]').click();
    await panelCold.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 1'));
    const workbenchFrame=JSON.parse(await panelCold.locator('#logical-input-panel [data-field="result"]').textContent());
    assert.deepEqual(workbenchFrame.frame,panelExpected.frames.at(-1));
    const menu=panelCold.locator('[data-testid="project-more-menu"]');
    if(!await menu.evaluate(node=>node.open))await menu.locator('summary').click();
    await panelCold.locator('[data-testid="authored-project-open"]').click();
    await panelCold.locator('[data-authored-runtime-save]').click();
    await panelCold.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('함께 저장')||document.querySelector('[data-authored-status]').textContent.includes('오류'));
    assert.match(await panelCold.locator('[data-authored-status]').textContent(),/함께 저장/);
    await browser.close();browser=await chromium.launchPersistentContext(browserProfile,browserOptions);
    const workbenchCold=await browser.newPage();
    workbenchCold.on('pageerror',error=>errors.push(String(error)));
    workbenchCold.on('requestfailed',request=>requests.push(request.url()));
    await openWorkbench(workbenchCold);
    const restoredSummary=JSON.parse(await workbenchCold.locator('#logical-input-panel [data-field="result"]').textContent());
    const {view_meta:restoredView,...restoredCore}=restoredSummary;
    assert.deepEqual(restoredCore,panelExpected,'display-only metadata must not change the restored execution');
    if (restoredSummary.view_meta?.draw_list?.length && !restoredSummary.bogae_projection) {
      await workbenchCold.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.renderSource==='committed_logical_drawlist');
      assert.equal(await workbenchCold.locator('[data-testid="example-guide"]').textContent(),'');
      await workbenchCold.screenshot({path:path.join(output,'workbench-storage-cold-restored-board.png')});
    }
    await workbenchCold.locator('[data-testid="lifecycle-step"]').click();
    await workbenchCold.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 2'));
    if (restoredSummary.view_meta?.draw_list?.length && !restoredSummary.bogae_projection) {
      assert.equal(await workbenchCold.locator('#canvas-bogae').getAttribute('data-render-source'),'committed_logical_drawlist');
      assert.equal(await workbenchCold.locator('[data-testid="example-guide"]').textContent(),'');
      const colored=await workbenchCold.locator('#canvas-bogae').evaluate(canvas=>{
        const data=canvas.getContext('2d').getImageData(0,0,canvas.width,canvas.height).data;
        let cells=0;for(let i=0;i<data.length;i+=4)if(Math.max(data[i],data[i+1],data[i+2])-Math.min(data[i],data[i+1],data[i+2])>50 && Math.max(data[i],data[i+1],data[i+2])>100)cells++;
        return cells;
      });
      assert.ok(colored>100,'real restored piece colors must occupy the primary canvas');
    }
    await workbenchCold.screenshot({path:path.join(output,'workbench-storage-cold-resume.png')});
    fs.writeFileSync(path.join(output,'cold-view-diagnostic.json'),JSON.stringify(await workbenchCold.evaluate(()=>({canvas:{...document.querySelector('#canvas-bogae').dataset},guide:document.querySelector('[data-testid="example-guide"]').textContent,summary:JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent)})),null,2));
    assert.deepEqual(errors,[]);assert.deepEqual(requests,[]);assert.deepEqual(badResponses,[]);
    const result={scope:'AUTHORED_LOGICAL_WORKER_EXPORT_INDEXEDDB',source_status:'WIP_NOT_FIXED',output,
      worker_native_equal:true,indexeddb_exact_readback:true,browser_exit_reopen_exact_storage:true,project_panel_start_step_export:true,wasm_sha256:sha(binary),
      browser_cold_resume:'PASS_WORKER_INDEXEDDB_PREFIX_REPLAY',second_checkpoint_restore:true,foreign_seed_rejected:cold.foreignSeedRejected,
      shipped_logical_panel_cold_restore_step:'PASS',
      workbench_storage_buttons:'PASS_SAVE_COLD_REOPEN_STEP',rd669_durable_receipt:'NOT_CLAIMED',release_human:'NOT_CLAIMED'};
    const releasePath=path.join(root,'RELEASE_MANIFEST.json');
    if(fs.existsSync(releasePath)) {
      const releaseBytes=fs.readFileSync(releasePath),release=JSON.parse(releaseBytes);
      assert.equal(release.build_identity?.wasm_sha256,sha(binary));
      assert.match(release.source?.commit??'',/^[0-9a-f]{40}$/);
      result.source_status='FIXED_EXTRACTED_PACKAGE_NOT_RELEASE_APPROVED';
      result.source_commit=release.source.commit;
      result.source_tree=release.source.tree;
      result.release_manifest_sha256=sha(releaseBytes);
    }
    fs.writeFileSync(path.join(output,'result.json'),JSON.stringify(result,null,2));
    console.log(JSON.stringify(result));
  } else {
  await page.evaluate(async ({source,binding,actionMap})=>{
    const {createRuntimeBridgeV1}=await import('/solutions/seamgrim_ui_mvp/ui/runtime/runtime_bridge.js');
    const {LogicalActionInput}=await import('/solutions/seamgrim_ui_mvp/ui/runtime/logical_action_input.js');
    window.bridge=createRuntimeBridgeV1();
    window.session=await window.bridge.createLogicalSession({sourceText:source,bindingJson:JSON.stringify(binding)});
    window.host=new LogicalActionInput({session:window.session,channels:binding.channels,initialHeld:binding.initial_held,actionMap,maxEvents:32});
    window.host.attach(document.querySelector('#input'));
  },{source,binding,actionMap});
  await page.locator('#input').focus();
  await page.keyboard.down('ArrowLeft'); await page.keyboard.down('ArrowRight');
  const first=await page.evaluate(async ()=>{const result=await window.host.step();document.querySelector('#result').textContent=result.resources.fixed64.계산값;return result;});
  assert.equal(first.resources.fixed64.계산값,'6');
  assert.equal(await page.locator('#result').textContent(),'6');
  assert.deepEqual(first.frame.input.events.map(event=>event.channel),['더함','곱함']);
  // A second down is a real browser repeat, not fabricated by the mapper.
  await page.keyboard.down('ArrowLeft');
  const repeat=await page.evaluate(()=>window.host.step());
  assert.equal(repeat.frame.input.events[0].event.kind,'Repeat');
  assert.equal(repeat.resources.fixed64.계산값,'2');
  await page.keyboard.up('ArrowLeft'); await page.keyboard.up('ArrowRight');
  const released=await page.evaluate(()=>window.host.step());
  assert.deepEqual(released.frame.input.events.map(event=>event.event.kind),['Release','Release']);
  assert.deepEqual(released.frame.input.held,{곱함:false,더함:false});
  const record=await page.evaluate(()=>window.host.seal());
  const workerAuthored=await page.evaluate(async()=>{
    const text=JSON.stringify({source_profile:'v1-core-v25',module_name:'앱',source_snapshot:{entry_path:'main.ddn',files:[{path:'main.ddn',text:'값 := 1.'}]}});
    const created=await window.bridge.authoredProjectOperation({operation:'create',text});
    const reopened=await window.bridge.authoredProjectOperation({operation:'open',text:created});
    const cloned=await window.bridge.authoredProjectOperation({operation:'clone',text:created});
    let rejected=false;
    try {await window.bridge.authoredProjectOperation({operation:'open',text:created.replace('값 := 1.','값 := 2.')});} catch {rejected=true;}
    return {created,reopened,cloned,rejected};
  });
  assert.equal(workerAuthored.created,workerAuthored.reopened);
  assert.notEqual(JSON.parse(workerAuthored.created).authored_identity.logical_dependency_id,JSON.parse(workerAuthored.cloned).authored_identity.logical_dependency_id);
  assert.equal(workerAuthored.rejected,true);
  const replay=await page.evaluate(async({source,binding,tapeBytes})=>window.bridge.replayLogicalInput({sourceText:source,tapeBytes,bindingJson:JSON.stringify(binding)}),{source,binding,tapeBytes:record.tape_bytes});
  assert.deepEqual(replay,record.summary);
  fs.writeFileSync(path.join(output,'source.ddn'),source);
  fs.writeFileSync(path.join(output,'binding.json'),JSON.stringify(binding));
  fs.writeFileSync(path.join(output,'browser.tape'),Buffer.from(record.tape_bytes));
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${name}.exe`),['논리입력재생',path.join(output,'source.ddn'),path.join(output,'browser.tape'),path.join(output,'binding.json')],{encoding:'utf8',timeout:30000});
    assert.equal(run.status,0,run.stderr); assert.deepEqual(JSON.parse(run.stdout),record.summary);
  }
  await page.locator('#result').evaluate(node=>{node.tabIndex=0;node.focus();});
  const lost=await page.evaluate(async()=>{
    try {await window.host.seal();return 'unexpected success';}
    catch(error) {return String(error);}
  });
  assert.match(lost,/E_LOGICAL_INPUT_LOST/);
  const aborted=await page.evaluate(()=>window.session.summary());
  assert.equal(aborted.outcome,'ABORTED');
  assert.equal(aborted.madi_count,3);
  assert.deepEqual(aborted.frames,record.summary.frames,'focus loss must not invent release');
  // Exercise the shipped Workbench panel controls, not a replacement JS world.
  await page.evaluate(async ({source,binding,actionMap})=>{
    const {installLogicalInputPanel}=await import('/solutions/seamgrim_ui_mvp/ui/workbench/logical_input_panel.js');
    window.panelSource=source;
    const panel=installLogicalInputPanel({parent:document.body,readSource:()=>window.panelSource});
    panel.open=true;
    panel.querySelector('[data-field="binding"]').value=JSON.stringify(binding);
    panel.querySelector('[data-field="actions"]').value=JSON.stringify(actionMap);
    panel.querySelector('[data-field="capacity"]').value='32';
  },{source,binding,actionMap});
  const panel=page.locator('#logical-input-panel');
  await panel.locator('[data-action="start"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('madi_count'));
  await panel.locator('[data-field="region"]').focus();
  await page.keyboard.down('ArrowLeft'); await page.keyboard.down('ArrowRight');
  await panel.locator('[data-action="step"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"계산값": "6"'));
  await page.keyboard.up('ArrowLeft'); await page.keyboard.up('ArrowRight');
  await panel.locator('[data-action="step"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi": 1'));
  const downloadPromise=page.waitForEvent('download');
  await panel.locator('[data-action="seal"]').click();
  const download=await downloadPromise;
  const panelTape=path.join(output,'panel.tape'); await download.saveAs(panelTape);
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${name}.exe`),['논리입력재생',path.join(output,'source.ddn'),panelTape,path.join(output,'binding.json')],{encoding:'utf8',timeout:30000});
    assert.equal(run.status,0,run.stderr); assert.equal(JSON.parse(run.stdout).madi_count,2);
  }
  await panel.locator('[data-action="stop"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('실행을 닫았습니다'));
  await panel.locator('[data-field="replay"]').setInputFiles(panelTape);
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi_count": 2'));
  await panel.locator('[data-action="start"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"madi_count": 0'));
  await panel.locator('[data-action="play"]').click();
  await page.waitForFunction(()=>{
    try {return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi >= 2;}
    catch {return false;}
  });
  await panel.locator('[data-action="pause"]').click();
  await page.evaluate(()=>{window.panelSource+='\n';});
  await panel.locator('[data-action="step"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_BINDING_CHANGED'));
    await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
    await page.waitForFunction(()=>Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__));
  const authored=page.locator('[data-testid="authored-project-drawer"]');
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
  assert.equal(await authored.locator('[data-authored-identity]').textContent(),'아직 없음');
  const sourceBefore=await page.locator('#source-editor').inputValue();
  await authored.locator('[data-authored-create]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('명시 작품 정체'));
  const artifactText=await authored.locator('[data-authored-contents]').inputValue();
  const projectId=JSON.parse(artifactText).authored_identity.logical_dependency_id;
  await authored.locator('[data-authored-save]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
  await authored.locator('[data-authored-reopen]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));
  assert.equal(await authored.locator('[data-authored-identity]').textContent(),projectId);
  assert.equal(await page.locator('#source-editor').inputValue(),sourceBefore);
  assert.equal(await authored.locator('[data-authored-contents]').inputValue(),artifactText);
  const downloadEvent=page.waitForEvent('download');
  await authored.locator('[data-authored-export]').click();
  const exported=await downloadEvent;
  const exportedPath=path.join(output,'authored-project.json');await exported.saveAs(exportedPath);
  assert.equal(fs.readFileSync(exportedPath,'utf8'),artifactText);
  if(process.env.DDN_AUTHORED_NATIVE_DIR) for(const alias of ['ddonilang','ddonirang','또니랑']) {
    const result=spawnSync(path.join(process.env.DDN_AUTHORED_NATIVE_DIR,alias+'.exe'),['작품자료','열기',exportedPath],{encoding:'utf8',timeout:30000});
    assert.equal(result.status,0,result.stderr);assert.equal(result.stdout.trim(),artifactText);
  }
  await authored.locator('input[type=file]').setInputFiles({name:'project.json',mimeType:'application/json',buffer:Buffer.from(artifactText)});
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('파일을 열었습니다'));
  assert.equal(await authored.locator('[data-authored-identity]').textContent(),projectId);
  await authored.locator('[data-authored-clone]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('새 ID로 복제했습니다'));
  assert.notEqual(await authored.locator('[data-authored-identity]').textContent(),projectId);
  await authored.screenshot({path:path.join(output,'authored-project-storage.png')});
  assert.equal(await page.evaluate(()=>localStorage.getItem('ddn.authored_project.last.v1')),artifactText,'clone must not overwrite saved original');
  const cloneText=await authored.locator('[data-authored-contents]').inputValue();
  await authored.locator('input[type=file]').setInputFiles({name:'bom.json',mimeType:'application/json',buffer:Buffer.from('\ufeff'+artifactText)});
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료 오류'));
  assert.equal(await authored.locator('[data-authored-contents]').inputValue(),cloneText,'rejected import must not publish');
  await authored.locator('[data-authored-close]').click();
  await page.locator('#source-editor').fill(sourceBefore+'\n// 저장되지 않은 편집\n');
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
  await authored.locator('[data-authored-save]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
  const editedText=await authored.locator('[data-authored-contents]').inputValue();
  const edited=JSON.parse(editedText), cloned=JSON.parse(cloneText);
  assert.deepEqual(edited.authored_identity,cloned.authored_identity,'edit save must retain every ID');
  assert.equal(edited.source_snapshot.files[0].text,sourceBefore+'\n// 저장되지 않은 편집\n');
  assert.notEqual(edited.content_sha256,cloned.content_sha256);
  await authored.locator('[data-authored-reopen]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));
  assert.equal(await page.locator('#source-editor').inputValue(),edited.source_snapshot.files[0].text);
  assert.equal(await authored.locator('[data-authored-contents]').inputValue(),editedText);
  await page.evaluate(()=>{window.delayLogicalOperation='authoredProjectOperation';window.logicalReplyHeld=false;});
  await authored.locator('[data-authored-save]').click();
  await page.waitForFunction(()=>window.logicalReplyHeld===true);
  await page.evaluate(()=>{
    const editor=document.querySelector('#source-editor');
    editor.value+='\n// 갱신 응답 대기 중 편집\n';editor.dispatchEvent(new Event('input',{bubbles:true}));
    window.releaseLogicalReply();
  });
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('바뀌었습니다'));
  assert.equal(await page.evaluate(()=>localStorage.getItem('ddn.authored_project.last.v1')),editedText,'stale update must not overwrite saved source');
  assert.equal(await authored.locator('[data-authored-contents]').inputValue(),editedText);
  await authored.locator('[data-authored-reopen]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));
  assert.equal(await page.locator('#source-editor').inputValue(),edited.source_snapshot.files[0].text);
  await authored.locator('[data-authored-close]').click();
  assert.equal(await page.locator('#logical-input-panel').count(),1,'normal Workbench must install the product panel');
  let tetris=fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn'),'utf8')+'\n// 명시 계약 선택은 현재 편집 원문에 결속한다.\n';
  const tetrisChannels=Object.fromEntries(['왼쪽','오른쪽','아래','회전','떨어뜨리기','다시시작'].map(name=>[name,'Control']));
  const tetrisBinding={...binding,input_schema:'alpha2.tetris.controls@1',channel_registry:'alpha2.tetris.channels@1',
    recording_policy:'alpha2.tetris.recording@1',source_revision:`sha256:${sha(tetris)}`,channels:tetrisChannels,
    initial_held:Object.fromEntries(Object.keys(tetrisChannels).map(name=>[name,false]))};
  const keys=['ArrowLeft','ArrowRight','ArrowDown','ArrowUp',' ','r'];
  const tetrisActions={schema:'ddn.input_action_map.v1',actions:Object.keys(tetrisChannels).map((name,n)=>({action_id:name,input_snapshot:{last_key:keys[n]}}))};
  await page.locator('#source-editor').fill(tetris);
  await panel.locator(':scope > summary').click();
  await panel.locator('[data-action="tetris-contract"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('현재 원문에 결속했습니다'));
  const selectedBinding=JSON.parse(await panel.locator('[data-field="binding"]').inputValue());
  assert.deepEqual(selectedBinding,tetrisBinding,'Rust must bind only current source, preserving explicit preset policy/channels');
  assert.deepEqual(JSON.parse(await panel.locator('[data-field="actions"]').inputValue()),tetrisActions);
  assert.equal(await panel.locator('[data-field="capacity"]').inputValue(),'128');
  assert.equal(await panel.locator('[data-field="contract-editor"]').getAttribute('open'),null,'JSON editor stays collapsed');
  assert.equal(await page.locator('#source-editor').inputValue(),tetris,'preset must not replace authored source');
  // WO14-A2-UI: same product contract, single main playback frontdoor.
  const primary=page.locator('[data-testid="lifecycle-run"]');
  const mainCanvas=page.locator('[data-testid="bogae-canvas"]');
  const mainStage=page.locator('[data-testid="bogae-stage"]');
  await primary.click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"조각번호"'));
  await primary.click();
  await page.waitForFunction(()=>{const b=document.querySelector('[data-testid="lifecycle-run"]');return b.dataset.action==='resume' && !b.disabled;});
  const readTetris=()=>page.evaluate(()=>JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent));
  const firstTetris=await readTetris();
  assert.equal(firstTetris.resources.fixed64.가로,'3'); assert.equal(firstTetris.view_meta.space2d.drawlist.length,5);
  assert.equal(firstTetris.bogae_projection.status,'ready');
  assert.equal(await mainCanvas.getAttribute('data-render-source'),'rust_authored_bogae_projection');
  assert.equal(await mainStage.locator('[data-authored-relation-id]').count(),2,'two authored Side instances, not one per madi');
  const firstSideIds=await mainStage.locator('[data-authored-relation-id]').evaluateAll(nodes=>nodes.map(node=>node.dataset.authoredRelationId));
  // Approved Alpha2 polish removed the game's cell magnifier. Real object-pick
  // behavior remains covered by the composition browser examples, not omitted.
  assert.equal(firstTetris.bogae_projection.candidates[0].artifact.relations.filter(row=>row.kind==='magnifier').length,0);
  assert.equal(await page.locator('[data-authored-magnifier-id]').count(),0);
  await mainCanvas.focus();
  await page.keyboard.press('ArrowLeft');
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"가로": "2"'));
  assert.deepEqual(await mainStage.locator('[data-authored-relation-id]').evaluateAll(nodes=>nodes.map(node=>node.dataset.authoredRelationId)),firstSideIds);
  // Rotate the first T, then many real press/release pairs must stop at x=-1
  // (its min local x is1). All events remain in received order in one snapshot.
  await page.keyboard.press('ArrowUp');
  for(let n=0;n<20;n++) await page.keyboard.press('ArrowLeft');
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('"가로": "-1"'));
  await page.keyboard.press(' ');
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>{
    try {return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).view_meta.space2d.drawlist.length===9;} catch{return false;}
  });
  const dropped=await readTetris();
  assert.equal(dropped.resources.value_json.판.flat().filter(cell=>cell!==0).length,4);
  assert.equal(dropped.resources.fixed64.세로,'0');
  assert.ok(dropped.view_meta.space2d.drawlist.some(shape=>Number(shape.y)>=400),'actual fixed cells must appear at the bottom');
  await mainCanvas.screenshot({path:path.join(output,'tetris.png')});
  await page.keyboard.press('r');
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>{
    try {return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).view_meta.space2d.drawlist.length===5;} catch{return false;}
  });
  const reset=await readTetris(); assert.equal(reset.resources.fixed64.가로,'3'); assert.equal(reset.resources.fixed64.점수,'0');
  await panel.screenshot({path:path.join(output,'workbench-contract.png')});
  const tetrisDownloadPromise=page.waitForEvent('download');
  await panel.locator('[data-action="seal"]').click();
  const tetrisDownload=await tetrisDownloadPromise;
  const tetrisTape=path.join(output,'tetris-browser.tape');await tetrisDownload.saveAs(tetrisTape);
  const tetrisSummary=await readTetris();
  assert.equal(await mainCanvas.getAttribute('data-committed-state-hash'),tetrisSummary.state_hash,'successful seal retains authored committed picture');
  assert.deepEqual(await mainStage.locator('[data-authored-relation-id]').evaluateAll(nodes=>nodes.map(node=>node.dataset.authoredRelationId)),firstSideIds,'seal retains existing Side instances');
  fs.writeFileSync(path.join(output,'tetris.ddn'),tetris);
  fs.writeFileSync(path.join(output,'tetris-binding.json'),JSON.stringify(selectedBinding));
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const run=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${name}.exe`),['논리입력재생',path.join(output,'tetris.ddn'),tetrisTape,path.join(output,'tetris-binding.json')],{encoding:'utf8',timeout:30000,maxBuffer:8*1024*1024});
    assert.equal(run.status,0,run.stderr);assert.deepEqual(JSON.parse(run.stdout),tetrisSummary);
  }
  await page.locator('[data-testid="lifecycle-stop"]').click();
  // click dispatch does not await the actual Worker close reply. Selecting a
  // file during close is correctly rejected as busy; wait for completed close.
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('실행을 닫았습니다'));
  await panel.locator('[data-field="replay"]').setInputFiles(tetrisTape);
  await page.waitForFunction(count=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count===count;}catch{return false;}},tetrisSummary.madi_count);
  assert.deepEqual(await readTetris(),tetrisSummary);
  assert.equal(await mainCanvas.getAttribute('data-committed-state-hash'),tetrisSummary.state_hash,'successful file replay displays the last committed authored picture');
  assert.equal(await mainStage.locator('[data-authored-relation-id]').count(),2,'replay displays both authored Sides');
  // Delay actual browser file reading, not world execution, to expose editor races.
  await page.evaluate(() => {
    const read = File.prototype.arrayBuffer;
    File.prototype.arrayBuffer = async function () {
      File.prototype.arrayBuffer = read;
      window.replayFileWaiting = true;
      await new Promise(resolve => { window.releaseReplayFile = resolve; });
      return read.call(this);
    };
  });
  await panel.locator('[data-field="replay"]').setInputFiles(tetrisTape);
  await page.waitForFunction(()=>window.replayFileWaiting);
  // A busy rejection must also clear the input so this same file remains selectable.
  await panel.locator('[data-field="replay"]').setInputFiles([]);
  await panel.locator('[data-field="replay"]').setInputFiles(tetrisTape);
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_STEP_BUSY'));
  assert.equal(await panel.locator('[data-field="replay"]').inputValue(),'');
  await page.locator('#source-editor').fill(tetris+'\n');
  await page.evaluate(()=>window.releaseReplayFile());
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_BINDING_CHANGED'));
  await page.locator('#source-editor').fill(tetris);
  await panel.locator('[data-field="replay"]').setInputFiles(tetrisTape);
  await page.waitForFunction(count=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count===count;}catch{return false;}},tetrisSummary.madi_count);
  assert.deepEqual(await readTetris(),tetrisSummary);
  await page.locator('#source-editor').fill(tetris+'\n');
  await panel.locator('[data-field="replay"]').setInputFiles(tetrisTape);
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_BINDING_MISMATCH'));
  await panel.locator('[data-action="bind-source"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('현재 원문에 결속했습니다'));
  const rebound=JSON.parse(await panel.locator('[data-field="binding"]').inputValue());
  assert.deepEqual(rebound,{...selectedBinding,source_revision:`sha256:${sha(tetris+'\n')}`});
  async function startPausedLogical() {
    await primary.click();
    await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi>=0;}catch{return false;}});
    await primary.click();
    await page.waitForFunction(()=>{const b=document.querySelector('[data-testid="lifecycle-run"]');return b.dataset.action==='resume' && !b.disabled;});
  }
  await startPausedLogical();
  await page.evaluate(()=>{ window.logicalReplyHeld=false; window.delayLogicalOperation='logicalStep'; });
  await page.locator('[data-testid="lifecycle-step"]').click();
  await page.waitForFunction(()=>window.logicalReplyHeld);
  // Preserve input focus: an ordinary editor focus change already aborts input.
  // An application-driven edit must also invalidate the outstanding frame.
  await page.evaluate(text=>{
    const editor=document.querySelector('#source-editor'); editor.value=text;
    editor.dispatchEvent(new Event('input',{bubbles:true}));
  },tetris+'\n// 계산 대기 중 편집\n');
  await page.evaluate(()=>window.releaseLogicalReply());
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_BINDING_CHANGED'));
  assert.equal(await panel.locator('[data-field="canvas"]').isVisible(),false,'stale frame must not be shown for edited source');
  await panel.locator('[data-action="seal"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('E_LOGICAL_INPUT_BINDING_CHANGED'));
  await page.locator('[data-testid="lifecycle-stop"]').click();
  // Delay the unchanged actual seal response, then edit without moving focus.
  await page.locator('#source-editor').fill(tetris);
  await panel.locator('[data-action="bind-source"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('현재 원문에 결속했습니다'));
  await startPausedLogical();
  let staleDownloads=0;
  const countDownload=()=>staleDownloads++;
  page.on('download',countDownload);
  await page.evaluate(()=>{ window.logicalReplyHeld=false; window.delayLogicalOperation='logicalSeal'; });
  await panel.locator('[data-action="seal"]').click();
  await page.waitForFunction(()=>window.logicalReplyHeld);
  await page.evaluate(text=>{
    const editor=document.querySelector('#source-editor'); editor.value=text;
    editor.dispatchEvent(new Event('input',{bubbles:true}));
  },tetris+'\n// 저장 응답 대기 중 편집\n');
  await page.evaluate(()=>window.releaseLogicalReply());
  await page.waitForFunction(()=>{
    const text=document.querySelector('#logical-input-panel [data-field="result"]').textContent;
    return text.includes('E_LOGICAL_INPUT_BINDING_CHANGED') || text.includes('"madi_count": 1');
  });
  assert.ok((await panel.locator('[data-field="result"]').textContent()).includes('E_LOGICAL_INPUT_BINDING_CHANGED'),'edited source must reject an outstanding seal result');
  assert.equal(staleDownloads,0,'stale source record must not be downloaded');
  page.off('download',countDownload);
  await page.locator('[data-testid="lifecycle-stop"]').click();
  const multiSnapshot={entry_path:'app/main.ddn',files:[
    {path:'selected.ddn',text:'(글판="v25", 모듬=차림[(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn"),(이름=힘, 머리="lib/힘.ddn", 원문="lib/힘.ddn")], 시작=(모듈=앱, 정의=실행)) 얽음.'},
    {path:'app/main.ddn',text:'실행:움직씨 := { 1 보여주기. }.'},
    {path:'lib/힘.ddn',text:'힘 := 2.'},
  ],root_path:'selected.ddn'};
  const multiText=module.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:multiSnapshot}));
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
  await authored.locator('input[type=file]').setInputFiles({name:'multi.json',mimeType:'application/json',buffer:Buffer.from(multiText)});
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('파일을 열었습니다'));
  assert.deepEqual(JSON.parse(await authored.locator('[data-authored-contents]').inputValue()),JSON.parse(multiText));
  await authored.locator('[data-authored-close]').click();
  const multiTab=page.locator('[data-testid="file-tabs"] [data-file-path="lib/힘.ddn"]');
  assert.equal(await multiTab.count(),1);
  await multiTab.click();
  await page.locator('#source-editor').fill('힘 := 3.');
  await page.locator('[data-testid="project-more-menu"] > summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
  await authored.locator('[data-authored-save]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
  const multiSaved=await authored.locator('[data-authored-contents]').inputValue();
  assert.deepEqual(JSON.parse(multiSaved).authored_identity,JSON.parse(multiText).authored_identity);
  assert.equal(JSON.parse(multiSaved).source_snapshot.files[2].text,'힘 := 3.');
  await authored.locator('[data-authored-reopen]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));
  assert.equal(await authored.locator('[data-authored-contents]').inputValue(),multiSaved);
  await authored.screenshot({path:path.join(output,'authored-multi-storage.png')});
  await authored.locator('[data-authored-close]').click();
  await multiTab.click();
  assert.equal(await page.locator('#source-editor').inputValue(),'힘 := 3.');
  // Start a fresh one-file Workbench rather than inheriting the rooted fixture.
  await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
  await page.waitForFunction(()=>Boolean(window.__SEAMGRIM_APPROVED_WORKBENCH__));
  const authoredPrimary='짜임:보개짜임값 := (관계들=차림[(보기=(() 표준보기.결과표)) 표준보개.으뜸]) 표준보개.짜임. (측정: 3) 보임.';
  await page.locator('#source-editor').fill('값 <- 9. 첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표.');
  const beforeInitialRejection=await page.evaluate(()=>({snapshot:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    canvas:document.querySelector('#canvas-bogae').toDataURL()}));
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='failed');
  const afterInitialRejection=await page.evaluate(()=>({snapshot:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    canvas:document.querySelector('#canvas-bogae').toDataURL()}));
  assert.ok(afterInitialRejection.snapshot.diagnostics.some(item=>item.code==='E_BOGAE_SELECTION_REQUIRED'));
  assert.deepEqual(afterInitialRejection.snapshot.activeFrame,beforeInitialRejection.snapshot.activeFrame);
  assert.equal(afterInitialRejection.canvas,beforeInitialRejection.canvas,'initial empty canvas must also remain unchanged');
  await page.keyboard.press('Escape');
  await page.locator('[data-testid="bogae-entry"]').fill('둘째');
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.authoredBogaeStatus==='ready');
  const selectedProjection=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.wasm_state.bogae_projection);
  assert.equal(selectedProjection.candidates.length,1);
  assert.equal(selectedProjection.candidates[0].label,'둘째');
  assert.equal(selectedProjection.persistent_identity,false);
  // A completed one-shot can choose the next run; source/profile/identity are not rewritten.
  await page.locator('[data-testid="bogae-entry"]').fill('');
  await page.locator('#source-editor').fill(authoredPrimary);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.authoredBogaeStatus === 'ready'
    && window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state
      ?.public_observation_table_projection?.tables?.[0]?.rows?.[0]?.cells?.[0]?.value_text === '3');
  const authoredCanvas=page.locator('#canvas-bogae');
  assert.match(await authoredCanvas.getAttribute('data-authored-bogae-digest'),/^[a-f0-9]{64}$/);
  const projectedState=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert.equal(projectedState.semanticRenderer.data_source,'rust_authored_bogae_projection');
  assert.equal(projectedState.activeFrame.wasm_state.public_observation_table_projection.tables[0].rows[0].cells[0].value_text,'3');
  await authoredCanvas.screenshot({path:path.join(output,'authored-primary-result-table.png')});
  await page.locator('#source-editor').fill('값 <- 9. 첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표.');
  const beforeRejected=await page.evaluate(()=>({snapshot:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    canvas:document.querySelector('#canvas-bogae').toDataURL()}));
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics.some(item=>item.code==='E_BOGAE_SELECTION_REQUIRED'));
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='failed');
  const afterRejected=await page.evaluate(()=>({snapshot:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot(),
    canvas:document.querySelector('#canvas-bogae').toDataURL()}));
  assert.deepEqual(afterRejected.snapshot.activeFrame,beforeRejected.snapshot.activeFrame,'rejected request must preserve committed world/frame/madi/hash');
  assert.equal(afterRejected.canvas,beforeRejected.canvas,'rejected request must preserve actual canvas pixels');
  assert.equal(afterRejected.snapshot.activeRunId,beforeRejected.snapshot.activeRunId);
  await page.keyboard.press('Escape');
  const authoredRoles='기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 겹:겹보개관계값 := (보기=(() 표준보기.결과표), 기준=기본, 순서=1) 표준보개.겹. 곁:곁보개관계값 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁. 짜임 := (관계들=차림[기본,겹,곁]) 표준보개.짜임. (측정:3) 보임.';
  await page.locator('#source-editor').fill(authoredRoles);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>document.querySelectorAll('[data-authored-relation-id]').length===1);
  const beforeViewActions=await page.evaluate(()=>{
    window.retainedAuthoredSide=document.querySelector('[data-authored-relation-id]');
    return window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame;
  });
  const sidePanel=page.locator('[data-authored-relation-id]');
  assert.equal(await sidePanel.getAttribute('data-placement'),'오른쪽');
  await page.locator('[data-authored-side-close]').click();
  assert.equal(await sidePanel.isVisible(),false);
  await page.locator('[data-authored-relation-toggle][data-relation-kind="side"]').click();
  assert.equal(await sidePanel.isVisible(),true);
  assert.equal(await page.evaluate(()=>window.retainedAuthoredSide===document.querySelector('[data-authored-relation-id]')),true);
  const withOverlay=await authoredCanvas.evaluate(canvas=>canvas.toDataURL());
  await page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]').click();
  assert.equal(await page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]').getAttribute('aria-pressed'),'false');
  assert.notEqual(await authoredCanvas.evaluate(canvas=>canvas.toDataURL()),withOverlay,'overlay toggles only the rendered picture');
  const afterViewActions=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  assert.deepEqual(afterViewActions,beforeViewActions,'side/overlay actions must not change world/madi/hash');
  await authoredCanvas.screenshot({path:path.join(output,'authored-overlay-side.png')});
  // Hold an unchanged actual Worker reply while the next activation is pending.
  // Local controls stay usable; rejecting that activation must not replace a side.
  await page.locator('#source-editor').fill('값 <- 9. 첫째 := () 표준보기.결과표. 둘째 := () 표준보기.결과표.');
  const beforePendingRejection=await page.evaluate(()=>({frame:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame,
    canvas:document.querySelector('#canvas-bogae').toDataURL()}));
  await page.evaluate(()=>{window.logicalReplyHeld=false;window.delayLogicalOperation='createSession';});
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.logicalReplyHeld===true);
  await page.locator('[data-authored-side-close]').click();
  assert.equal(await page.evaluate(()=>window.retainedAuthoredSide===document.querySelector('[data-authored-relation-id]')),true,
    'pending activation must retain the committed side instance');
  assert.equal(await sidePanel.isVisible(),false);
  await page.evaluate(()=>window.releaseLogicalReply());
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='failed');
  assert.equal(await page.evaluate(()=>window.retainedAuthoredSide===document.querySelector('[data-authored-relation-id]')),true);
  assert.equal(await sidePanel.isVisible(),false,'rejection must retain the local hidden state');
  assert.equal(await page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]').getAttribute('aria-pressed'),'false');
  const afterPendingRejection=await page.evaluate(()=>({frame:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame,
    canvas:document.querySelector('#canvas-bogae').toDataURL(),
    diagnostics:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().diagnostics}));
  assert.ok(afterPendingRejection.diagnostics.some(item=>item.code==='E_BOGAE_SELECTION_REQUIRED'));
  assert.deepEqual(afterPendingRejection.frame,beforePendingRejection.frame);
  assert.equal(afterPendingRejection.canvas,beforePendingRejection.canvas);
  await page.keyboard.press('Escape');
  await page.locator('[data-authored-relation-toggle][data-relation-kind="side"]').click();
  assert.equal(await sidePanel.isVisible(),true);
  assert.equal(await page.evaluate(()=>window.retainedAuthoredSide===document.querySelector('[data-authored-relation-id]')),true);
  const placements=['왼쪽','오른쪽','위','아래','자동'];
  const positionedSides=placements.map((placement,index)=>`곁${index} := (보기=(() 표준보기.결과표), 자리=#${placement}) 표준보개.곁.`).join(' ');
  await page.locator('#source-editor').fill(`기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. ${positionedSides} 짜임 := (관계들=차림[기본,곁0,곁1,곁2,곁3,곁4]) 표준보개.짜임. (측정:3) 보임.`);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>document.querySelectorAll('[data-authored-relation-id]').length===5);
  // Compact split mode intentionally stacks local docks below 900px. Check
  // physical placement hints in the actual full-width view, not that fallback.
  await page.locator('[data-testid="mode-bogae-full"]').click();
  const positions=await page.evaluate(()=>{
    const bounds=element=>{const r=element.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom};};
    return {primary:bounds(document.querySelector('[data-testid="bogae-canvas-wrap"]')),
      docks:Object.fromEntries([...document.querySelectorAll('[data-authored-dock]')].map(d=>[d.dataset.authoredDock,bounds(d)]))};
  });
  assert.ok(positions.docks.left.right<=positions.primary.left,'left hint must place the side to the left');
  assert.ok(positions.docks.right.left>=positions.primary.right,'right hint must place the side to the right');
  assert.ok(positions.docks.top.bottom<=positions.primary.top,'top hint must place the side above');
  assert.ok(positions.docks.bottom.top>=positions.primary.bottom,'bottom hint must place the side below');
  assert.equal(await page.locator('[data-authored-dock="right"] [data-placement="자동"]').count(),1);
  const placementFrame=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  const rightSide=page.locator('[data-authored-dock="right"] [data-placement="오른쪽"]');
  const autoSide=page.locator('[data-authored-dock="right"] [data-placement="자동"]');
  const rightId=await rightSide.getAttribute('data-authored-relation-id');
  await page.evaluate(()=>{window.retainedRightSide=document.querySelector('[data-placement="오른쪽"]');});
  await rightSide.locator('[data-authored-side-close]').click();
  assert.equal(await rightSide.isVisible(),false,'closing one side must hide it even when its shared dock stays open');
  assert.equal(await autoSide.isVisible(),true,'the other side in the same dock must remain visible');
  await page.locator(`[data-authored-relation-toggle="${rightId}"]`).click();
  assert.equal(await rightSide.isVisible(),true);
  assert.equal(await page.evaluate(()=>window.retainedRightSide===document.querySelector('[data-placement="오른쪽"]')),true);
  await page.evaluate(()=>{window.retainedLeftSide=document.querySelector('[data-placement="왼쪽"]');});
  await page.locator('[data-placement="왼쪽"] [data-authored-side-close]').click();
  assert.equal(await page.locator('[data-authored-dock="left"]').isVisible(),false);
  const leftId=await page.evaluate(()=>window.retainedLeftSide.dataset.authoredRelationId);
  await page.locator(`[data-authored-relation-toggle="${leftId}"]`).click();
  assert.equal(await page.evaluate(()=>window.retainedLeftSide===document.querySelector('[data-placement="왼쪽"]')),true);
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),placementFrame);
  await page.locator('[data-testid="bogae-stage"]').screenshot({path:path.join(output,'authored-side-placements.png')});
  const planeSource='원 := (중심=차림[0,0], 반지름=2) 표준장면.원. 작은원 := (중심=차림[0,0], 반지름=1) 표준장면.원. 장면 := (요소들=차림[원], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 작은장면 := (요소들=차림[작은원], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 기본 := (보기=((장면=장면) 표준보기.장면)) 표준보개.으뜸. 겹 := (보기=((장면=작은장면) 표준보기.장면), 기준=기본, 순서=1) 표준보개.겹. 곁 := (보기=(() 표준보기.결과표), 자리=#오른쪽) 표준보개.곁. 짜임 := (관계들=차림[기본,겹,곁]) 표준보개.짜임. (측정:3) 보임.';
  await page.locator('#source-editor').fill(planeSource);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.bogae_projection?.candidates?.[0]?.artifact?.view?.kind==='scene');
  const planeFrame=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  assert.equal(planeFrame.wasm_state.bogae_projection.candidates[0].artifact.view.elements[0].radius,'8589934592');
  const planePixels=await authoredCanvas.evaluate(canvas=>canvas.toDataURL());
  await page.locator('[data-authored-relation-toggle][data-relation-kind="overlay"]').click();
  assert.notEqual(await authoredCanvas.evaluate(canvas=>canvas.toDataURL()),planePixels,'authored inner-circle overlay must actually change pixels');
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),planeFrame);
  assert.equal(await page.locator('[data-authored-relation-id]').isVisible(),true);
  await authoredCanvas.screenshot({path:path.join(output,'authored-circle-plane.png')});
  const polygonSource='네모 := (꼭짓점들=차림[차림[0,0],차림[2,0],차림[2,2],차림[0,2]]) 표준장면.다각형. 장면 := (요소들=차림[네모], 가로범위=차림[-1,3], 세로범위=차림[-1,3]) 표준장면.평면. 보기 := (장면=장면) 표준보기.장면.';
  await page.locator('#source-editor').fill(polygonSource);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.bogae_projection?.candidates?.[0]?.artifact?.view?.elements?.[0]?.kind==='polygon');
  const polygonFrame=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  assert.deepEqual(polygonFrame.wasm_state.bogae_projection.candidates[0].artifact.view.elements[0].vertices,[['0','0'],['8589934592','0'],['8589934592','8589934592'],['0','8589934592']]);
  assert.notEqual(await authoredCanvas.evaluate(canvas=>canvas.toDataURL()),planePixels);
  await authoredCanvas.screenshot({path:path.join(output,'authored-polygon-plane.png')});
  const magnifierSource='(대상:수) 확대:보기값:셈씨 := { 원 := (중심=차림[0,0], 반지름=대상) 표준장면.원. 장면 := (요소들=차림[원], 가로범위=차림[-5,5], 세로범위=차림[-5,5]) 표준장면.평면. 확대 <- (장면=장면) 표준보기.장면. }. 기본 := (보기=(() 표준보기.결과표)) 표준보개.으뜸. 자세히 := (대상=2, 보기법=확대) 표준보개.돋. 짜임 := (관계들=차림[기본,자세히]) 표준보개.짜임.';
  await page.locator('#source-editor').fill(magnifierSource);
  await page.locator('[data-testid="lifecycle-run"]').click();
  const magnifierToggle=page.locator('[data-authored-relation-toggle][data-relation-kind="magnifier"]');
  await magnifierToggle.waitFor();
  const magnifierFrame=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame);
  const authoredMagnifier=magnifierFrame.wasm_state.bogae_projection.candidates[0].artifact.relations[1];
  assert.equal(authoredMagnifier.recipe.kind,'sealed_named_callable');
  assert.equal(authoredMagnifier.target.raw,'8589934592');
  assert.equal(authoredMagnifier.view.elements[0].radius,'8589934592');
  assert.equal(await page.locator('[data-authored-magnifier-id]').count(),0);
  await magnifierToggle.click();
  await page.locator('[data-authored-magnifier-id] canvas').waitFor();
  assert.equal(await page.locator('[data-authored-magnifier-id]').count(),1);
  await page.evaluate(()=>{window.firstMagnifier=document.querySelector('[data-authored-magnifier-id]');});
  const magnifierCanvas=page.locator('[data-authored-magnifier-id] canvas');
  const canvasRatio=await magnifierCanvas.evaluate(canvas=>{
    const rect=canvas.getBoundingClientRect();
    return {display:(rect.width-2)/(rect.height-2),intrinsic:canvas.width/canvas.height};
  });
  assert.ok(Math.abs(canvasRatio.display-canvasRatio.intrinsic)<0.02,'CSS must not turn an authored circle into an ellipse');
  assert.equal(await magnifierCanvas.evaluate(canvas=>{
    const data=canvas.getContext('2d').getImageData(0,0,canvas.width,canvas.height).data;
    for(let i=0;i<data.length;i+=4) if(data[i]<80 && data[i+2]>100) return true;
    return false;
  }),true,'sealed Rust recipe must produce an actual circle, not only metadata');
  await magnifierCanvas.screenshot({path:path.join(output,'authored-sealed-magnifier.png')});
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),magnifierFrame);
  await page.locator('[data-authored-magnifier-close]').click();
  assert.equal(await page.locator('[data-authored-magnifier-id]').count(),0);
  await magnifierToggle.click();
  assert.equal(await page.evaluate(()=>window.firstMagnifier===document.querySelector('[data-authored-magnifier-id]')),false);
  assert.deepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame),magnifierFrame);
  // A pause followed immediately by saving must await the unchanged actual
  // Worker reply, not reject or silently omit its committed input frame.
  const oldProjectSource=await page.locator('#source-editor').inputValue();
  const oldProjectSnapshot=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  await panel.evaluate(element=>element.open=true);
  await panel.locator('[data-action="open-tetris"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('현재 원문에 결속했습니다'));
  tetris=fs.readFileSync(path.join(root,'solutions/seamgrim_ui_mvp/samples/alpha2_logical_tetris.ddn'),'utf8');
  fs.writeFileSync(path.join(output,'tetris.ddn'),tetris);
  fs.writeFileSync(path.join(output,'tetris-binding.json'),await panel.locator('[data-field="binding"]').inputValue());
  assert.equal(await page.locator('#source-editor').inputValue(),tetris);
  assert.notEqual(oldProjectSource,tetris);
  assert.notDeepEqual(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().project),oldProjectSnapshot.project);
  await page.evaluate(()=>{window.logicalReplyHeld=false;window.delayLogicalOperation='logicalStep';});
  await primary.click();
  await page.waitForFunction(()=>window.logicalReplyHeld);
  await primary.click();
  const pausedDownload=page.waitForEvent('download',{timeout:5000});
  await panel.locator('[data-action="seal"]').click();
  await page.evaluate(()=>window.releaseLogicalReply());
  const pausedFile=path.join(output,'paused-in-flight.tape');
  await (await pausedDownload).saveAs(pausedFile);
  const pausedSummary=await readTetris();
  assert.equal(pausedSummary.madi_count,1,'the actual pending madi must be sealed exactly once');
  assert.notEqual(pausedSummary.outcome,'ABORTED');
  // Continuous real play is observed, not asserted to meet an invented timing
  // SLA. The actual downloaded file must replay at Native3 regardless of pace.
  await page.locator('[data-testid="lifecycle-stop"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('실행을 닫았습니다'));
  const playStarted=performance.now();
  await primary.click();
  await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi>=30;}catch{return false;}});
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press(' ');
  await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame.input.madi>=90;}catch{return false;}});
  await primary.click();
  const continuousDownload=page.waitForEvent('download');
  await panel.locator('[data-action="seal"]').click();
  const continuousFile=path.join(output,'continuous-play.tape');
  await (await continuousDownload).saveAs(continuousFile);
  const continuousSummary=await readTetris();
  const continuousObservation={madis:continuousSummary.madi_count,elapsed_ms:performance.now()-playStarted};
  assert.ok(continuousSummary.madi_count>=91);
  for(const name of ['ddonilang','ddonirang','또니랑']) {
    const result=spawnSync(path.join(process.env.DDN_NATIVE_DIR,`${name}.exe`),['논리입력재생',path.join(output,'tetris.ddn'),continuousFile,path.join(output,'tetris-binding.json')],{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
    assert.equal(result.status,0,result.stderr);assert.deepEqual(JSON.parse(result.stdout),continuousSummary);
  }
  await page.locator('[data-testid="lifecycle-stop"]').click();
  await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('실행을 닫았습니다'));
  await panel.locator('[data-field="replay"]').setInputFiles(continuousFile);
  await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).madi_count>=91;}catch{return false;}});
  assert.deepEqual(await readTetris(),continuousSummary);
  assert.deepEqual(errors,[]); assert.deepEqual(requests,[]); assert.deepEqual(badResponses,[]);
  console.log(JSON.stringify({result:'ALPHA2_LOGICAL_WORKER_BROWSER_PASS',actual_key_events:5,
    actual_worker_wasm:true,native_aliases:3,madi_count:3,evidence_directory:output,
    authored_project_worker_create_reopen_clone_and_corruption:true,
    authored_project_ui_create_save_reopen_export_import_clone_and_bom:true,
    authored_project_ui_edited_source_save_and_reopen_preserve_ids:true,
    authored_project_ui_stale_update_reply_rejected:true,
    authored_project_ui_multi_module_import_edit_save_reopen_preserve_ids:true,
    authored_temporary_primary_actual_workbench_render:true,
    multiple_bogae_activation_preserves_committed_canvas_and_world:true,
    multiple_bogae_initial_rejection_preserves_empty_canvas:true,
    authored_overlay_side_hide_reopen_same_instance_world_unchanged:true,
    pending_rejected_activation_retains_side_instance_and_local_state:true,
    authored_side_placement_five_hints_actual_layout:true,
    authored_plane_circle_overlay_and_result_table_side_actual_wasm:true,
    authored_polygon_exact_vertices_actual_wasm:true,
    authored_sealed_magnifier_explicit_open_close_new_instance_world_unchanged:true,
    wasm_sha256:createHash('sha256').update(binary).digest('hex'),pageerrors:errors.length,
    request_failures:requests.length,http_failures:badResponses.length,
    workbench_panel_controls:true,
    actual_workbench_tetris_keys_and_canvas:true,
    explicit_contract_selection_and_rebinding:true,tetris_browser_file_native_replay:true,
    replay_file_edit_race_rejected:true,busy_file_selection_cleared:true,
    live_worker_edit_race_rejected:true,seal_worker_edit_race_rejected:true,pause_pending_frame_then_seal:true,continuous_play_observation:continuousObservation,
    explicit_new_tetris_workflow:true,
    scope:'Actual host/Worker, authored views, Tetris play and downloaded-record Native3 replay; full Alpha2 release not claimed'}));
  }
  } catch(error) {
    console.error('BROWSER_ORIGINAL_ERROR',error);
    const activePage=page.isClosed()?(browser.pages?.().at(-1)??browser.contexts?.().at(-1)?.pages().at(-1)):page;
    if(activePage&&!activePage.isClosed()) {
      console.error('BROWSER_DIAGNOSTICS',JSON.stringify(await activePage.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot()?.diagnostics)));
      fs.writeFileSync(path.join(output,'failure-workbench.json'),JSON.stringify(await activePage.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot() ?? null),null,2));
      console.error('BROWSER_FAILURE_STATE',(await activePage.locator('[data-field="result"]').allTextContents()).map(text=>text.slice(0,800)),errors,badResponses);
      console.error('BROWSER_FAILURE_STATUS',await activePage.locator('[data-authored-status]').allTextContents(),await activePage.locator('#logical-input-panel [data-field="feedback"]').allTextContents(),output);
      await activePage.screenshot({path:path.join(output,'failure.png')});
    }
  throw error;
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
