// A3-P11: actual authored root import -> UI Worker -> source-bound Rust VM.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
assert.ok(process.env.DDN_WASM_DIR && process.env.DDN_PLAYWRIGHT_MODULE);
const wasmDir=path.resolve(process.env.DDN_WASM_DIR),root=process.cwd();
const bytes=fs.readFileSync(path.join(wasmDir,'ddonirang_tool_bg.wasm'));
const expectedHash=createHash('sha256').update(bytes).digest('hex');
const wasm=await import(pathToFileURL(path.join(wasmDir,'ddonirang_tool.js')).href);
await wasm.default({module_or_path:bytes});
const editModules=process.env.DDN_SAVED_EDIT_MODULES==='1';
const createFromUi=process.env.DDN_SAVED_CREATE_FROM_UI==='1';
const typedTuple=process.env.DDN_SAVED_TYPED_TUPLE==='1';
const declaredCallTuple=process.env.DDN_SAVED_DECLARED_CALL_TUPLE==='1';
const literalTuple=process.env.DDN_SAVED_LITERAL_TUPLE==='1';
assert(!literalTuple || typedTuple,'literal workflow requires typed Tuple coverage');
assert(!declaredCallTuple || typedTuple,'declared result workflow uses typed Tuple coverage');
const staticTuple=process.env.DDN_SAVED_TUPLE_STATIC_EXPR==='1';
const localSum=process.env.DDN_SAVED_LOCAL_SUM==='1';
assert(!staticTuple || typedTuple,'constant selection workflow requires the explicit typed Tuple sample');
assert(!createFromUi || editModules, 'UI creation workflow uses the real three-file physics template');
const expectedCheck=editModules?7:4;
const sourceRoot=`(글판="v25", 모듬=차림[(이름=앱, 머리="app/main.ddn", 원문="app/main.ddn")${editModules?', (이름=힘, 머리="lib/힘.ddn", 원문="lib/힘.ddn")':''}], 시작=(모듈=앱, 정의=실행)) 얽음.\n`;
const liveTest=process.env.DDN_SAVED_LIVE==='1';
const recordSchema='(가로:셈수, 세로:셈수) 좌표:묶음씨 := {}.\n';
const tupleSelection=staticTuple?'(2 / 2)':'1';
const tupleWriteIndex=staticTuple?'(1 - 1)':'0';
const callDeclarations=declaredCallTuple?'(원값:바른수) 정수전달:바른수:셈씨 := { 정수전달 <- 원값. }. (원값:셈수) 실수전달:셈수:셈씨 := { 실수전달 <- 원값. }.\n':'';
const tupleInteger=literalTuple?'7':declaredCallTuple?'((원값=정수) 정수전달)':'정수';
const tupleReal=literalTuple?'7.0':declaredCallTuple?'((원값=실수) 실수전달)':'실수';
const tupleDeclarations=typedTuple?callDeclarations+`정수:바른수 := 7. 실수:셈수 := 7. 글값:글 := "가". 논값:참거짓 := 참. 정수묶음 := (${tupleInteger},글값,논값). 소수묶음 := (${tupleReal},글값,논값). 빈묶음 := (). 중첩묶음 := (정수묶음,소수묶음). 범위묶음 := 정수묶음[0..2]. 선택묶음 := (정수묶음[${tupleSelection}],논값).\n`:'';
const tupleWrites=typedTuple?`정수묶음[${tupleWriteIndex}] <- 정수. `:'';
// Raw module definitions enter the same project frontdoor as the student's
// source. Only the numeric result escapes this local sum; no sum codec fixture.
const sumDeclarations=localSum?'선택:고름씨 := { $수(내용:수). }. 고름계산:수:셈씨 := { 자료 := 선택.$수(내용=4). 자료에 따라 { 선택.$수(내용)이면 { 고름계산 <- 내용. }. }. }.\n':'';
const calculation=(localSum?'고름검산 <- () 고름계산. ':'')+'판 <- (차림=차림[차림[1,2],차림[3,4]],원소씨=셈수,모양=(2,2)) 텐서로. 검산 <- 판[1,1]. 점 <- (가로=검산,세로=2) 좌표. 바이트자료 <- (글="가") UTF8바이트열로.';
const views=`(값:수) 진행그림:보기값:셈씨 := {
  점 := (중심=차림[값,0],반지름=0.2) 표준장면.원.
  장면 := (요소들=차림[점],가로범위=차림[-1,10],세로범위=차림[-1,1]) 표준장면.평면.
  진행그림 <- (장면=장면) 표준보기.장면.
}.
으뜸보기 := (보기=((값=검산) 진행그림)) 표준보개.으뜸.
곁보기 := (보기=((값=검산+1) 진행그림),자리=#오른쪽) 표준보개.곁.
실험짜임 := (관계들=차림[으뜸보기,곁보기]) 표준보개.짜임.
`;
const source=(editModules?'(힘=힘) 쓰임.\n':'')+recordSchema+sumDeclarations+tupleDeclarations+(liveTest?`채비 { 진행:수 := 0. }. 실행:움직씨 := { ${tupleWrites}(매마디)마다 { ${calculation} 진행 <- 진행 + 1. 진행 보여주기. (시간: 진행 * 1@s, 값: 검산) 보여주기. }. }.\r\n`:`실행:움직씨 := { ${tupleWrites}${calculation} 검산 보여주기. (값: 검산) 보여주기. }.\r\n`)+views;
const originalSource=editModules?source.replace('차림[3,4]','차림[3,힘.값]'):source;
let artifact=createFromUi?'':wasm.authored_project_operation('create',JSON.stringify({source_profile:'v1-core-v25',module_name:'',source_snapshot:{root_path:'root.ddn',entry_path:'app/main.ddn',files:[{path:'root.ddn',text:sourceRoot},{path:'app/main.ddn',text:originalSource},...(editModules?[{path:'lib/힘.ddn',text:'(값=값) 드러냄.\n값:수 := 4.\n'}]:[])]}}));
const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-alpha3-saved-browser-'));
const artifactFile=path.join(output,'authored.json');fs.writeFileSync(artifactFile,artifact);
const served=[],errors=[];
const server=http.createServer((req,res)=>{
  const relative=decodeURIComponent(new URL(req.url,'http://localhost').pathname).replace(/^\/+/, '');
  const local=path.resolve(root,relative);
  if(!local.startsWith(root+path.sep)){res.writeHead(403).end();return;}
  const file=relative.startsWith('solutions/seamgrim_ui_mvp/ui/wasm/')?path.join(wasmDir,path.basename(local)):local;
  try{let content=fs.readFileSync(file);if(file.endsWith('_bg.wasm'))served.push(createHash('sha256').update(content).digest('hex'));
    // Diagnostic-only transport probe, never release PASS evidence.
    if(process.env.DDN_TRACE_WASM_TRAPS==='1'&&file.endsWith('runtime_worker.js'))
      content=Buffer.from(content.toString('utf8').replace('catch (error) {','catch (error) { console.error("WASM_TRAP_STACK",error?.stack);'));
    res.writeHead(200,{'content-type':file.endsWith('.wasm')?'application/wasm':file.endsWith('.js')?'text/javascript':file.endsWith('.html')?'text/html':file.endsWith('.css')?'text/css':'application/json'}).end(content);
  }catch{res.writeHead(404).end();}
});
// This host uses a dynamic range starting at 1024. Never let an automatic
// assignment bind the owner's protected 8790 or a Chrome-restricted low port.
// The private transport port is not an input to the authored world.
let privateListening=false;
for(let attempt=0;attempt<64&&!privateListening;attempt++) {
  privateListening=await new Promise((resolve,reject)=>{
    const ready=()=>{server.off('error',failed);resolve(true);};
    const failed=error=>{
      server.off('listening',ready);
      if(error.code==='EADDRINUSE'||error.code==='EACCES')resolve(false);else reject(error);
    };
    server.once('error',failed);server.once('listening',ready);
    server.listen(20000+Math.floor(Math.random()*40000),'127.0.0.1');
  });
}
assert(privateListening,'private browser test port unavailable; owner servers were not changed');
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
// Private profile only: terminate the actual browser process, then reopen its
// disk-backed IndexedDB. Page reload alone cannot prove close/reopen persistence.
const browserOptions={headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe',
  viewport:{width:1500,height:1000}};
const profileDirectory=path.join(output,'browser-profile');
let browser=await chromium.launchPersistentContext(profileDirectory,browserOptions);
const attachPageEvidence=page=>{
  page.on('pageerror',error=>errors.push(String(error)));
  page.on('console',message=>{if(message.type()==='error')errors.push(message.text());
    if(message.text().startsWith('WASM_TRAP_STACK'))console.error(message.text());
    if(message.text().startsWith('[storage-check]'))console.log(message.text());});
  page.on('requestfailed',request=>errors.push(request.url()));
  page.on('response',response=>{if(response.status()>=400)errors.push(response.url());});
};
let page;
let logicalStorageOwner=null;
try{
  page=browser.pages()[0]??await browser.newPage();
  attachPageEvidence(page);
  await page.goto(`http://127.0.0.1:${server.address().port}/solutions/seamgrim_ui_mvp/ui/workbench/index.html`);
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  let menu=page.locator('[data-testid="project-more-menu"]');
  if(createFromUi){
    const rail=page.locator('[data-testid="project-rail-toggle"]');
    if(await rail.getAttribute('aria-expanded')!=='true')await rail.click();
    await page.getByTestId('new-ddn-project').click();
    await page.getByTestId('project-create-name').fill('직접 만든 세 파일 작품');
    await page.locator('input[name="project-template"][value="physics"]').check();
    await page.getByTestId('project-create-submit').click();
    await page.getByTestId('project-create-dialog').waitFor({state:'hidden'});
    for(const [file,text] of [['뿌리.ddn',sourceRoot],['app/main.ddn',originalSource],
      ['lib/힘.ddn','(값=값) 드러냄.\n값:수 := 4.\n']]){
      await page.locator(`[data-testid="file-tabs"] button[data-file-path="${file}"]`).click();
      await page.getByTestId('source-editor').fill(text);
    }
    await menu.locator('summary').click();
    await page.getByTestId('authored-project-open').click();
    await page.locator('[data-authored-create]').click();
    await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('명시 작품 정체를 만들었습니다'));
    await page.locator('[data-authored-save]').click();
    await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
    artifact=await page.locator('[data-authored-contents]').inputValue();
    fs.writeFileSync(artifactFile,artifact);
    assert.equal(JSON.parse(artifact).source_snapshot.root_path,'뿌리.ddn');
  }else{
  await menu.locator('summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
  await page.locator('[data-testid="authored-project-drawer"] input[type=file]').setInputFiles(artifactFile);
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('파일을 열었습니다'));
  }
  await page.locator('[data-authored-close]').click();
  if(editModules){
    const original=JSON.parse(artifact);
    await page.locator('[data-testid="file-tabs"] button[data-file-path="lib/힘.ddn"]').click();
    await page.locator('[data-testid="source-editor"]').fill('(값=값) 드러냄.\n값:수 := 7.\n');
    await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
    await page.locator('[data-authored-save]').click();
    await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
    artifact=await page.locator('[data-authored-contents]').inputValue();
    const edited=JSON.parse(artifact);
    assert.deepEqual(edited.authored_identity,original.authored_identity,'editing a module preserves explicit project/module identities');
    assert.notEqual(edited.content_sha256,original.content_sha256);
    assert.equal(edited.source_snapshot.files.find(file=>file.path==='lib/힘.ddn').text,'(값=값) 드러냄.\n값:수 := 7.\n');
    await page.locator('[data-authored-close]').click();
    await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
    await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.pauseRun());
    assert.equal(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.wasm_state.resources.value_json.검산),7,'saving must update execution immediately, not only after a cold reopening');
    await page.reload();
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
    await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
    await page.locator('[data-authored-reopen]').click();
    await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));
    assert.equal(await page.locator('[data-authored-contents]').inputValue(),artifact);
    await page.locator('[data-authored-close]').click();
  }
  await page.waitForFunction(()=>!document.querySelector('[data-testid="lifecycle-run"]').disabled);
  await page.locator('[data-testid="lifecycle-run"]').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
  await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.pauseRun());
  await page.waitForFunction(()=>['paused','done'].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status));
  const snapshot=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  const frame=snapshot.activeFrame;
  assert.equal(frame.wasm_state.resources.value_json.검산,expectedCheck);
  assert.ok(frame.wasm_state.resources.value_json.판,'the UI receives the actual Tensor result');
  assert.equal(frame.wasm_state.resources.value_json.점.type,'record','actual UI save uses typed Record, not a Map');
  assert.equal(frame.wasm_state.resources.value_json.점.version,1);
  assert.equal(frame.wasm_state.resources.value_json.바이트자료.type,'bytes_v1');
  assert.equal(frame.wasm_state.resources.value_json.바이트자료.payload_byte_length,6,'encoding preserves decomposed scalar bytes');
  await menu.locator('summary').click();
  await page.locator('[data-testid="authored-project-open"]').click();
  await page.locator('[data-authored-runtime-save]').click();
  await page.waitForFunction(()=>{const text=document.querySelector('[data-authored-status]').textContent;return text.includes('실행 상태를 저장하고') || text.startsWith('저장 자료 오류');});
  assert.ok((await page.locator('[data-authored-status]').textContent()).includes('실행 상태를 저장하고'),await page.locator('[data-authored-status]').textContent());
  const beforeExport=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash);
  const downloadPromise=page.waitForEvent('download');
  await page.locator('[data-authored-runtime-export]').click();
  const downloaded=await downloadPromise;
  const downloadedFile=path.join(output,downloaded.suggestedFilename());await downloaded.saveAs(downloadedFile);
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('CLI 이어 실행:'));
  const exportMessage=await page.locator('[data-authored-status]').textContent();
  const lineage=exportMessage.match(/--부모 (처음|[0-9a-f]+) --판 ([0-9a-f]+) --마디수 1/);
  assert.ok(lineage,'export supplies explicit storage lineage, not a filename identity');
  assert.equal(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame.state_hash),beforeExport);
  const downloadedBytes=fs.readFileSync(downloadedFile,'utf8');
  const downloadedVm=wasm.DdnWasmVm.from_saved_project_artifact_json(downloadedBytes,lineage[1]==='처음'?'':lineage[1],lineage[2]);
  assert.equal(downloadedVm.get_state_hash(),beforeExport);downloadedVm.step_one();
  const downloadedNextHash=downloadedVm.get_state_hash();downloadedVm.free();
  for(const bin of JSON.parse(process.env.DDN_CLI_BINS||'[]')){
    const next=spawnSync(bin,['작품자료','이어실행',downloadedFile,'--부모',lineage[1],'--판',lineage[2],'--마디수','1'],{encoding:'utf8'});
    assert.equal(next.status,0,next.stderr);assert.equal(JSON.parse(next.stdout).state_hash,downloadedNextHash,'actual browser download -> Native continuation');
  }
  await page.locator('[data-authored-runtime-reopen]').click();
  await page.waitForFunction(()=>{const text=document.querySelector('[data-authored-status]').textContent;return text.includes('실행 상태를 복원') || text.startsWith('저장 자료 오류');});
  assert.ok((await page.locator('[data-authored-status]').textContent()).includes('실행 상태를 복원'),await page.locator('[data-authored-status]').textContent());
  const reopened=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert.equal(reopened.activeFrame.state_hash,frame.state_hash);
  assert.equal(reopened.activeFrame.madi,frame.madi,'restored observation identifies the last committed madi, not the next executable tick');
  assert.equal(reopened.activeFrame.wasm_state.resources.value_json.검산,expectedCheck);
  if(process.env.DDN_LOGICAL_STORAGE_OWNER==='1') {
    const projectId=JSON.parse(artifact).authored_identity.logical_dependency_id;
    const readStored=()=>page.evaluate(async id=>{
      const store=await import('./persistence/runtime_project_store.js');return store.loadRuntimeProject(id);
    },projectId);
    const prior=await readStored();
    const empty=new wasm.DdnWasmVm(''),initialWorld=empty.get_state_hash();empty.free();
    const binding={source_profile:'v1-core-v25-logical-alpha2',input_schema:'beta1.storage-owner@1',
      channel_registry:'beta1.storage-owner.channels@1',source_revision:'pending-explicit-binding',initial_world:initialWorld,
      initial_rng:7,recording_policy:'beta1.storage-owner.recording@1',integrity_policy:'ddn.logical_input_tape.v1',
      madi_hz:60,dt_raw:4294967296,channels:{확인:'Control'},initial_held:{확인:false}};
    const actions={schema:'ddn.input_action_map.v1',actions:[{action_id:'확인',input_snapshot:{last_key:'KeyZ'}}]};
    await page.locator('[data-authored-close]').click();
    await menu.locator('summary').click();await page.locator('[data-testid="logical-record-open"]').click();
    const panel=page.locator('#logical-input-panel');
    await panel.locator('[data-field="advanced"] > summary').click();
    await panel.locator('[data-field="contract-editor"] > summary').click();
    await panel.locator('[data-field="binding"]').fill(JSON.stringify(binding));
    await panel.locator('[data-field="actions"]').fill(JSON.stringify(actions));
    await panel.locator('[data-field="capacity"]').fill('128');
    await panel.locator('[data-action="bind-source"]').click();
    await page.waitForFunction(()=>document.querySelector('#logical-input-panel [data-field="result"]').textContent.includes('아직 세계 마디'));
    await page.locator('[data-testid="logical-record-close"]').click();
    await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(()=>{try{return JSON.parse(document.querySelector('#logical-input-panel [data-field="result"]').textContent).frame?.input?.madi>=0;}catch{return false;}});
    await page.locator('[data-testid="lifecycle-run"]').click();
    await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]').dataset.action==='resume');
    const logicalResult=await panel.locator('[data-field="result"]').textContent();
    if(!await menu.evaluate(node=>node.open))await menu.locator('summary').click();
    await page.locator('[data-testid="authored-project-open"]').click();
    // Logical save/reopen now have their own source-bound checkpoint path;
    // the separate logical browser runner proves those actual UI workflows.
    // Carrier-only export must still reject instead of exporting the stale VM.
    for(const selector of ['[data-authored-runtime-export]']) {
      await page.locator(selector).click();
      await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('E_PROJECT_RUNTIME_LOGICAL_CHECKPOINT_REQUIRED'));
      assert.deepEqual(await readStored(),prior,'logical storage must not overwrite the paused ordinary VM checkpoint');
      assert.equal(await panel.locator('[data-field="result"]').textContent(),logicalResult,'storage controls preserve the paused logical response');
      assert.equal(await page.locator('[data-testid="lifecycle-run"]').getAttribute('data-action'),'resume','storage drawer is an admitted host control, not input loss');
    }
    await page.screenshot({path:path.join(output,'logical-storage-owner-rejected.png')});
    logicalStorageOwner={ordinary_checkpoint_preserved:true,logical_response_preserved:true,carrier_only_export_rejected:true,
      logical_save_reopen_evidence:'alpha2_logical_worker_browser_runner.mjs'};
    await page.locator('[data-authored-close]').click();await page.locator('[data-testid="lifecycle-stop"]').click();
    await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
  }
  await page.locator('[data-authored-save]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('이 기기에 저장했습니다'));
  const reopenUrl=page.url();
  await browser.close();
  browser=await chromium.launchPersistentContext(profileDirectory,browserOptions);
  page=browser.pages()[0]??await browser.newPage();
  attachPageEvidence(page);
  await page.goto(reopenUrl);
  menu=page.locator('[data-testid="project-more-menu"]');
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
  await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
  await page.locator('[data-authored-reopen]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.includes('저장 자료를 다시 열었습니다'));
  const coldBefore=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert.equal(coldBefore.activeFrame,null,'cold reopening must not secretly execute a new world');
  await page.locator('[data-authored-runtime-reopen]').click();
  await page.waitForFunction(()=>{const text=document.querySelector('[data-authored-status]').textContent;return text.includes('실행 상태를 복원') || text.startsWith('저장 자료 오류');});
  assert.ok((await page.locator('[data-authored-status]').textContent()).includes('실행 상태를 복원'),await page.locator('[data-authored-status]').textContent());
  let coldRestored=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert.equal(coldRestored.activeFrame.state_hash,frame.state_hash);
  assert.equal(coldRestored.activeFrame.wasm_state.tick_id,reopened.activeFrame.wasm_state.tick_id,'cold restore must not run an extra initialization tick');
  if(liveTest){
    assert.equal(coldRestored.executionLifecycle.status,'paused');
    const restoredTick=coldRestored.activeFrame.wasm_state.tick_id;
    const restoredMadi=coldRestored.activeFrame.madi;
    await page.locator('[data-authored-close]').click();
    await page.locator('[data-testid="lifecycle-step"]').click();
    await page.waitForFunction(madi=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.madi===madi+1,restoredMadi);
    const continued=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert.equal(continued.activeFrame.wasm_state.tick_id,restoredTick,'the resumed step commits the saved next tick');
    const expectedVm=wasm.DdnWasmVm.from_authored_project_json(artifact);
    for(let tick=0;tick<restoredTick+1;tick++)expectedVm.step_one();
    assert.equal(continued.activeFrame.state_hash,expectedVm.get_state_hash(),'cold reopened UI continues the actual saved world, not a fresh initialization');expectedVm.free();
    coldRestored=continued;
    await menu.locator('summary').click();await page.locator('[data-testid="authored-project-open"]').click();
  }
  // Corrupt only this isolated browser's test storage. Both host rows still
  // agree, so the common Rust artifact gate (not a JS checksum) must reject.
  await page.evaluate(async projectId=>{
    const db=await new Promise((resolve,reject)=>{const request=indexedDB.open('ddn.runtime_project.local.v1',1);request.onsuccess=()=>resolve(request.result);request.onerror=()=>reject(request.error);});
    try{await new Promise((resolve,reject)=>{
      const tx=db.transaction(['heads','revisions'],'readwrite',{durability:'strict'});
      const request=tx.objectStore('heads').get(projectId);
      request.onsuccess=()=>{const record=request.result;record.artifact_text='{}';tx.objectStore('heads').put(record);tx.objectStore('revisions').put(record);};
      tx.oncomplete=resolve;tx.onabort=()=>reject(tx.error);tx.onerror=()=>reject(tx.error);
    });}finally{db.close();}
  },JSON.parse(artifact).authored_identity.logical_dependency_id);
  await page.locator('[data-authored-runtime-reopen]').click();
  await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.startsWith('저장 자료 오류'));
  assert.ok((await page.locator('[data-authored-status]').textContent()).includes('E_PROJECT_RUNTIME_ARTIFACT_FIELD_COUNT'));
  const rejected=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert.deepEqual(rejected.activeFrame,coldRestored.activeFrame,'failed restore publishes no frame/world/tick changes');
  await page.locator('[data-authored-close]').click();
  await menu.locator('summary').click();
  await page.locator('[data-testid="project-history-open"]').click();
  await page.locator('[data-testid="input-tape-export"]').click();
  await page.waitForFunction(()=>document.querySelector('[data-testid="input-tape-status"]').textContent.includes('E_RESTORED_INPUT_HISTORY_UNAVAILABLE'));
  const afterTapeRefusal=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
  assert.deepEqual(afterTapeRefusal.activeFrame,coldRestored.activeFrame,'a missing full input history must not be invented or change the restored world');
  await page.locator('[data-testid="project-history-close"]').click();
  const actualVm=wasm.DdnWasmVm.from_authored_project_json(artifact);actualVm.step_one();
  // Beta1: the actual WASM restore frontdoor must reject a missing channel,
  // a wrong channel type and a late malformed row, without changing the VM.
  const completeState=JSON.parse(actualVm.get_state_json());
  if(localSum)assert.equal(Number(completeState.resources.value_json.고름검산),4,'local sum calculation survives the real source-bound cold restore');
  if(typedTuple){
    const values=completeState.state.snapshot_v2.resources.value_det;
    for(const name of ['정수묶음','소수묶음','빈묶음','중첩묶음','범위묶음','선택묶음'])assert.equal(values[name].type,'tuple_v1',name);
    assert.notEqual(values.정수묶음.canonical_sha256,values.소수묶음.canonical_sha256,'declared 바른수/셈수 survive equal raw values');
  }
  const atomicRestoreCases=[];
  for(const kind of ['missing-channel','wrong-channel-type','late-json-row','contradictory-entity-cursor','missing-observation-frames','invalid-next-tick','contradictory-next-tick','invalid-display-policy','record-version','record-field-type','bytes-version','bytes-frame-hash','bytes-raw-order','missing-input-keys','missing-input-last-key','missing-input-pointer-x','missing-input-pointer-y','missing-input-base-seed','missing-input-seed',...(typedTuple?['tuple-source-type','tuple-resource-tag']:[])]){
    const damaged=structuredClone(completeState),resources=damaged.state.snapshot_v2.resources;
    if(kind==='missing-channel')delete resources.fixed64;
    else if(kind==='wrong-channel-type')resources.value_det=null;
    else if(kind==='late-json-row')resources.json={'a-valid':'preserve as text','z-invalid':17};
    else if(kind==='contradictory-entity-cursor'){resources.next_entity=0;resources.components=[{entity:0,tag:'위치',json:'{}'}];}
    else if(kind==='missing-observation-frames')delete damaged.state.snapshot_v2.public_observation_frames;
    else if(kind==='invalid-next-tick')damaged.state.snapshot_v2.next_tick_id=-1;
    else if(kind==='contradictory-next-tick')damaged.state.snapshot_v2.next_tick_id+=1;
    else if(kind==='record-version')resources.value_det.점.version=2;
    else if(kind==='record-field-type')resources.value_det.점.type_identity.가로.name='글';
    else if(kind==='bytes-version')resources.value_det.바이트자료.version=2;
    else if(kind==='bytes-frame-hash')resources.value_det.바이트자료.frame_sha256='00'.repeat(32);
    else if(kind==='bytes-raw-order')resources.value_det.바이트자료=Object.fromEntries(Object.entries(resources.value_det.바이트자료).reverse());
    else if(kind==='tuple-source-type')resources.value_det.정수묶음=structuredClone(resources.value_det.소수묶음);
    else if(kind==='tuple-resource-tag'){resources.value_det.모르는묶음=structuredClone(resources.value_det.정수묶음);delete resources.value_det.정수묶음;}
    else if(kind==='missing-input-keys')delete damaged.state.input.keys_pressed;
    else if(kind==='missing-input-last-key')delete damaged.state.input.last_key_name;
    else if(kind==='missing-input-pointer-x')delete damaged.state.input.pointer_x_i32;
    else if(kind==='missing-input-pointer-y')delete damaged.state.input.pointer_y_i32;
    else if(kind==='missing-input-base-seed')delete damaged.state.input.rng_base_seed;
    else if(kind==='missing-input-seed'){delete damaged.state.input.rng_base_seed;delete damaged.state.input.rng_seed;}
    else damaged.output_display.fraction_digits=5;
    const before=actualVm.get_state_json();
    const expectedError=kind==='missing-observation-frames'?/E_WASM_STATE_OBSERVATION_FRAMES_INVALID/:
      ['invalid-next-tick','contradictory-next-tick'].includes(kind)?/E_WASM_STATE_TICK_INVALID/:
      kind==='record-version'?/E_PRODUCT_RECORD_VERSION/:
      kind==='record-field-type'?/E_PRODUCT_RECORD_WIRE/:
      kind==='bytes-version'?/E_BYTES_STORAGE_VERSION/:
      kind==='bytes-frame-hash'?/E_BYTES_STORAGE_HASH/:
      kind==='bytes-raw-order'?/E_BYTES_SNAPSHOT/:
      kind.startsWith('tuple-')?/E_TUPLE_PERSISTENCE_UNBOUND/:
      kind==='invalid-display-policy'?/E_OUTPUT_DISPLAY_SNAPSHOT/:
      kind.startsWith('missing-input-')?/E_WASM_STATE_INPUT_INVALID/:/E_RESOURCE_SNAPSHOT_INVALID/;
    assert.throws(()=>actualVm.restore_state(JSON.stringify(damaged)),expectedError);
    assert.equal(actualVm.get_state_json(),before,`${kind}: no partial world, tick, input or display publication`);
    atomicRestoreCases.push({kind,rejected:true,pre_state_unchanged:true});
  }
  const runtimeArtifact=actualVm.export_saved_project_artifact_json('','01');
  const runtimeFile=path.join(output,'runtime.json');fs.writeFileSync(runtimeFile,runtimeArtifact);
  // Adversarial transport fixtures, not a product writer or a new format.
  // Keep the source/envelope/PESI and resource snapshot untouched. Recompute
  // existing attachment byte hashes so rejection reaches shared input admission
  // rather than merely detecting an obsolete outer checksum.
  const missingInputFiles=[];
  for(const field of ['keys_pressed','last_key_name','pointer_x_i32','pointer_y_i32','rng_base_seed','seed','next_tick_id']){
    const damaged=JSON.parse(runtimeArtifact),attachment=damaged.runtime_attachment;
    const state=JSON.parse(attachment.runtime_state_json_utf8);
    if(field==='next_tick_id')state.state.snapshot_v2.next_tick_id+=1;
    else if(field==='seed'){delete state.state.input.rng_base_seed;delete state.state.input.rng_seed;}
    else delete state.state.input[field];
    attachment.runtime_state_json_utf8=JSON.stringify(state);
    attachment.runtime_state_byte_length=Buffer.byteLength(attachment.runtime_state_json_utf8,'utf8');
    attachment.runtime_state_sha256=createHash('sha256').update(attachment.runtime_state_json_utf8).digest('hex');
    damaged.canonical_json=JSON.stringify({project_snapshot:damaged.project_snapshot,runtime_attachment:attachment});
    damaged.canonical_byte_length=Buffer.byteLength(damaged.canonical_json,'utf8');
    damaged.content_sha256=createHash('sha256').update(damaged.canonical_json).digest('hex');
    const bytes=JSON.stringify(damaged),file=path.join(output,`missing-input-${field}.json`);
    fs.writeFileSync(file,bytes);
    const error=field==='next_tick_id'?/E_WASM_STATE_TICK_INVALID/:/E_WASM_STATE_INPUT_INVALID/;
    assert.throws(()=>wasm.DdnWasmVm.from_saved_project_artifact_json(bytes,'','01'),error);
    missingInputFiles.push({field,file,error});
  }
  const nativeResume=[];
  if(process.env.DDN_CLI_BINS){
    const bins=JSON.parse(process.env.DDN_CLI_BINS);
    const continuedVm=wasm.DdnWasmVm.from_saved_project_artifact_json(runtimeArtifact,'','01');
    continuedVm.step_one();continuedVm.step_one();
    const expected=continuedVm.get_state_hash();
    const expectedState=JSON.parse(continuedVm.get_state_json());continuedVm.free();
    assert.ok(expectedState.public_observation_trace.samples.length,'comparison must contain actual observation samples, not only empty arrays');
    assert.ok(expectedState.output_display.rows.length,'comparison must contain actual output rows');
    assert.ok(expectedState.bogae_projection,'comparison must contain the authored main/side composition');
    const comparePresentation=(native,state)=>{
      for(const field of ['public_observation_trace','public_observation_trace_hash','public_observation_table_projection','output_display','bogae_projection']){
        assert.deepEqual(native[field],state[field],`Native/WASM exact restored ${field}`);
      }
      assert.deepEqual(native.public_observation_frames,state.state.snapshot_v2.public_observation_frames,'Native/WASM committed observation frames');
    };
    for(const bin of bins){
      const invoke=(revision,madi)=>spawnSync(bin,['작품자료','이어실행',runtimeFile,'--부모','처음','--판',revision,'--마디수',String(madi)],{encoding:'utf8'});
      const read=invoke('01',0);assert.equal(read.status,0,read.stderr);
      const readResult=JSON.parse(read.stdout);
      assert.equal(readResult.state_hash,actualVm.get_state_hash(),'Native restore does not initialize or step');
      comparePresentation(readResult,completeState);
      const run=invoke('01',2);assert.equal(run.status,0,run.stderr);
      const result=JSON.parse(run.stdout);assert.equal(result.state_hash,expected,'same exact stored artifact -> Native/WASM continuation');
      comparePresentation(result,expectedState);
      const stale=invoke('02',1);assert.notEqual(stale.status,0,'wrong selected revision is rejected');
      for(const {field,file,error} of missingInputFiles){
        const rejected=spawnSync(bin,['작품자료','이어실행',file,'--부모','처음','--판','01','--마디수','1'],{encoding:'utf8'});
        assert.notEqual(rejected.status,0,`${field}: Native must reject before continuation`);
        assert.match(rejected.stderr,error);
        assert.equal(rejected.stdout.trim(),'','failure must not publish a success result/hash');
      }
      nativeResume.push({bin,executable_sha256:createHash('sha256').update(fs.readFileSync(bin)).digest('hex'),
        restore:'PASS',continuation:'PASS_TWO_MADI_EXACT_WASM_HASH',presentation:'PASS_TRACE_TABLE_DISPLAY_FRAMES_BOGAE',stale_revision:'REJECT',missing_input_rejection:'PASS_SIX_CASES_NO_SUCCESS_OUTPUT',contradictory_madi_rejection:'PASS_NO_SUCCESS_OUTPUT',state_hash:result.state_hash});
    }
  }
  const projectId='storage-mechanics-'+JSON.parse(artifact).authored_identity.logical_dependency_id;
  const storageFailures=await page.evaluate(async ({projectId,runtimeArtifact})=>{
    console.log('[storage-check] start');
    const store=await import('./persistence/runtime_project_store.js');
    const record=store.runtimeStorageRecord(projectId,'','01',runtimeArtifact);
    const saved=await store.commitRuntimeProject(record);
    let staleRejected=false;
    try{await store.commitRuntimeProject(store.runtimeStorageRecord(projectId,'','02',runtimeArtifact));}
    catch(error){staleRejected=String(error).includes('E_RUNTIME_STORAGE_STALE');}
    if(!staleRejected)throw new Error('stale writer must be rejected');
    const reread=await store.loadRuntimeProject(projectId);
    if(JSON.stringify(saved)!==JSON.stringify(reread))throw new Error('stale writer changed the stored head');
    // Entry-time bytes must be frozen before opening the asynchronous database.
    // These are storage-mechanics records; Rust admission/receipt is separate.
    const mutableId=projectId+'-mutable',mutable=store.runtimeStorageRecord(mutableId,'','01',runtimeArtifact);
    const expected=JSON.stringify(mutable),pending=store.commitRuntimeProject(mutable);
    mutable.revision_hex='02';mutable.artifact_text='caller changed after save started';
    const captured=await pending;
    if(JSON.stringify(captured)!==expected)throw new Error('save admitted caller-mutated bytes after entry');
    const rawTransaction=operation=>new Promise((resolve,reject)=>{
      const open=indexedDB.open('ddn.runtime_project.local.v1',1);
      open.onerror=()=>reject(open.error);
      open.onsuccess=()=>{
        const db=open.result,tx=db.transaction(['heads','revisions'],'readwrite');
        let result;
        tx.oncomplete=()=>{db.close();resolve(result);};
        tx.onabort=()=>{db.close();reject(tx.error??new Error('fixture transaction aborted'));};
        tx.onerror=()=>{};
        operation(tx,value=>{result=value;});
      };
    });
    const readPair=()=>rawTransaction((tx,set)=>{
      const result={},head=tx.objectStore('heads').get(projectId),revision=tx.objectStore('revisions').get([projectId,'01']);
      head.onsuccess=()=>{result.head=head.result;};revision.onsuccess=()=>{result.revision=revision.result;};set(result);
    });
    const rejectSuccessor=async (label,expectedError)=>{
      const before=JSON.stringify(await readPair());let rejected=false;
      try{await store.commitRuntimeProject(store.runtimeStorageRecord(projectId,'01','02',runtimeArtifact));}
      catch(error){rejected=String(error).includes(expectedError);}
      if(!rejected)throw new Error(label+': corrupt parent must reject before writing');
      if(JSON.stringify(await readPair())!==before)throw new Error(label+': failed save changed previous rows');
    };
    // Only this isolated test profile is mutated, never the user's work.
    await rawTransaction(tx=>tx.objectStore('revisions').put({...saved,artifact_text:'damaged parent'}));
    await rejectSuccessor('parent-revision-mismatch','E_RUNTIME_STORAGE_CHAIN');
    await rawTransaction(tx=>{
      tx.objectStore('revisions').put(saved);
      tx.objectStore('heads').put({...saved,schema:'damaged'});
    });
    await rejectSuccessor('invalid-parent-head','E_RUNTIME_STORAGE_RECORD');
    await rawTransaction(tx=>{
      tx.objectStore('heads').put(saved);
      tx.objectStore('revisions').put(store.runtimeStorageRecord(projectId,'01','02','occupied immutable revision'));
    });
    await rejectSuccessor('duplicate-new-revision','ConstraintError');
    await rawTransaction(tx=>tx.objectStore('revisions').delete([projectId,'02']));
    const encodingCases=[];
    const shared={label:'shared acyclic child'},array=[shared];array.ignored=array;
    const acyclic=store.runtimeStorageRecord(projectId+'-acyclic','','01',runtimeArtifact);
    acyclic.extra={left:shared,right:shared,array};
    const acyclicText=JSON.stringify(acyclic);
    if(JSON.stringify(await store.commitRuntimeProject(acyclic))!==acyclicText)throw new Error('shared children or ignored array property changed comparison');
    for(const kind of ['bigint','cycle']){
      console.log('[storage-check] '+kind+' preflight');
      const successor=store.runtimeStorageRecord(projectId,'01','02',runtimeArtifact);
      successor.extra=kind==='bigint'?1n:successor;
      const before=JSON.stringify(await readPair());let rejected=false;
      console.log('[storage-check] '+kind+' entry snapshot');
      try{await store.commitRuntimeProject(successor);}
      catch(error){rejected=String(error).includes('E_RUNTIME_STORAGE_RECORD_ENCODING');}
      console.log('[storage-check] '+kind+' entry rejected='+rejected);
      if(!rejected||JSON.stringify(await readPair())!==before)throw new Error(kind+': encoding failure changed normal rows');
      const damaged={...saved};damaged.extra=kind==='bigint'?1n:damaged;
      console.log('[storage-check] '+kind+' damaged read');
      await rawTransaction(tx=>tx.objectStore('revisions').put(damaged));
      let readRejected=false;
      try{await store.loadRuntimeProject(projectId);}
      catch(error){readRejected=String(error).includes('E_RUNTIME_STORAGE_RECORD_ENCODING');}
      if(!readRejected)throw new Error(kind+': invalid revision returned as successful load');
      let writeRejected=false;
      console.log('[storage-check] '+kind+' damaged parent');
      try{await store.commitRuntimeProject(store.runtimeStorageRecord(projectId,'01','02',runtimeArtifact));}
      catch(error){writeRejected=String(error).includes('E_RUNTIME_STORAGE_RECORD_ENCODING');}
      if(!writeRejected)throw new Error(kind+': invalid parent admitted successor');
      const afterFailure=await rawTransaction((tx,set)=>{
        const result={};set(result);
        const head=tx.objectStore('heads').get(projectId),next=tx.objectStore('revisions').get([projectId,'02']);
        head.onsuccess=()=>{result.head=head.result;};
        next.onsuccess=()=>{result.next=next.result;};
      });
      if(JSON.stringify(afterFailure.head)!==JSON.stringify(saved)||afterFailure.next)throw new Error(kind+': failed save changed persistent rows');
      await rawTransaction(tx=>tx.objectStore('revisions').put(saved));
      console.log('[storage-check] '+kind+' restored');
      encodingCases.push({kind,preflight:'REJECT_NO_ROW_CHANGE',read:'REJECT',parent:'REJECT_NO_SUCCESSOR'});
    }
    const preserved=await store.loadRuntimeProject(projectId);
    if(JSON.stringify(preserved)!==JSON.stringify(saved))throw new Error('failed writes lost the last normal saved record');
    return {stored:preserved,caller_mutation:'PASS_ENTRY_BYTES_PRESERVED',
      corrupt_parent:'PASS_TWO_REJECTIONS_NO_ROW_CHANGE',duplicate_revision:'PASS_ABORT_PRESERVES_NORMAL_HEAD',
      encoding_exception_atomicity:encodingCases,
      shared_acyclic_metadata:'PASS_EXACT_JSON_COMPARISON',
      scope:'BROWSER_STORAGE_MECHANICS_NOT_RD669_RECOVERY'};
  },{projectId,runtimeArtifact});
  const stored=storageFailures.stored;
  assert.equal(stored.artifact_text,runtimeArtifact);
  const restored=wasm.DdnWasmVm.from_saved_project_artifact_json(stored.artifact_text,stored.parent_hex,stored.revision_hex);
  assert.equal(restored.get_state_hash(),actualVm.get_state_hash());
  restored.free();actualVm.free();
  let identityLifecycle='NOT_RUN';
  if(process.env.DDN_SAVED_IDENTITY_LIFECYCLE==='1') {
    const openPanel=async()=>{
      menu=page.getByTestId('project-more-menu');
      if(!await menu.evaluate(node=>node.open))await menu.locator('summary').click();
      await page.getByTestId('authored-project-open').click();
    };
    const statusContains=async text=>{
      // Prior negative checks leave an error message visible. Wait for this
      // operation's shared busy guard to settle before reading its response.
      await page.waitForFunction(()=>!document.querySelector('[data-authored-register]').disabled);
      await page.waitForFunction(text=>{
        const value=document.querySelector('[data-authored-status]').textContent;
        return value.includes(text)||value.startsWith('저장 자료 오류');
      },text);
      assert.ok((await page.locator('[data-authored-status]').textContent()).includes(text),await page.locator('[data-authored-status]').textContent());
    };
    const getText=()=>page.locator('[data-authored-contents]').inputValue();
    const state=()=>page.evaluate(()=>({
      frame:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame,
      source:document.querySelector('#source-editor').value,
      artifact:document.querySelector('[data-authored-contents]').value,
      local:localStorage.getItem('ddn.authored_project.last.v1'),
    }));
    await openPanel();
    await page.locator('[data-authored-module]').fill('앱');
    await page.locator('[data-authored-source]').fill(JSON.parse(await getText()).source_snapshot.entry_path);
    await page.locator('[data-authored-composition]').fill('실험짜임');
    fs.writeFileSync(path.join(output,'identity-before-register.json'),await getText());
    fs.writeFileSync(path.join(output,'identity-before-register-state.json'),JSON.stringify(await state(),null,2));
    await page.locator('[data-authored-register]').click();await statusContains('보개 시작을 명시 등록');
    const registeredText=await getText(),registered=JSON.parse(registeredText);
    const owner=registered.authored_identity.modules.find(module=>module.compositions.some(value=>value.definition==='실험짜임'));
    assert.ok(owner);const composition=owner.compositions.find(value=>value.definition==='실험짜임');
    const target={owner_module_id:owner.id,target_kind:'composition',target_id:composition.id};
    await page.locator('[data-authored-close]').click();
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
    await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.pauseRun());
    await page.waitForFunction(()=>['paused','done'].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status));
    await openPanel();
    await page.locator('[data-authored-rename-target]').locator('..').locator('..').evaluate(node=>node.open=true);
    await page.locator('[data-authored-rename-target]').selectOption(JSON.stringify(target));
    // A valid new name without the student's source edit must be refused.
    const beforeFailure=await state();
    const storedBefore=await page.evaluate(async id=>JSON.stringify(await (await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js')).loadRuntimeProject(id)),projectId);
    await page.locator('[data-authored-new-name]').fill('새실험짜임');
    await page.locator('[data-authored-rename]').click();
    await page.waitForFunction(()=>!document.querySelector('[data-authored-rename]').disabled);
    await page.waitForFunction(()=>document.querySelector('[data-authored-status]').textContent.startsWith('저장 자료 오류'));
    assert.deepEqual(await state(),beforeFailure,'rejected rename preserves project source, VM frame, authored bytes and saved source');
    assert.equal(await page.evaluate(async id=>JSON.stringify(await (await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js')).loadRuntimeProject(id)),projectId),storedBefore,'rejected rename preserves runtime record');
    await page.screenshot({path:path.join(output,'identity-rename-rejected-preserved.png'),fullPage:true});
    await page.locator('[data-authored-close]').click();
    const entry=registered.source_snapshot.entry_path;
    await page.getByRole('button',{name:entry,exact:true}).click();
    await page.locator('#source-editor').fill(registered.source_snapshot.files.find(file=>file.path===entry).text.replaceAll('실험짜임','새실험짜임'));
    await openPanel();
    await page.locator('[data-authored-rename]').click();await statusContains('명시 변경을 검증');
    const renamedText=await getText(),renamed=JSON.parse(renamedText);
    assert.equal(renamed.authored_identity.logical_dependency_id,registered.authored_identity.logical_dependency_id);
    assert.deepEqual(renamed.authored_identity.modules.map(module=>module.id),registered.authored_identity.modules.map(module=>module.id));
    assert.equal(renamed.authored_identity.modules.find(module=>module.id===owner.id).compositions.find(value=>value.id===composition.id).definition,'새실험짜임');
    const renameRequest={original_json:registeredText,source_snapshot:renamed.source_snapshot,...target,new_name:'새실험짜임'};
    const invalidRename={...renameRequest,source_snapshot:registered.source_snapshot};
    const oldPath=renamed.source_snapshot.entry_path,newPath='app/옮긴원문.ddn';
    await page.locator('[data-authored-close]').click();
    await page.getByRole('button',{name:renamed.source_snapshot.root_path,exact:true}).click();
    await page.locator('#source-editor').fill(renamed.source_snapshot.files.find(file=>file.path===renamed.source_snapshot.root_path).text.replaceAll(oldPath,newPath));
    await openPanel();
    await page.locator('[data-authored-old-path]').fill(oldPath);await page.locator('[data-authored-new-path]').fill(newPath);
    await page.locator('[data-authored-relocate]').click();await statusContains('명시 변경을 검증');
    const movedText=await getText(),moved=JSON.parse(movedText);
    assert.equal(moved.authored_identity.logical_dependency_id,registered.authored_identity.logical_dependency_id);
    assert.equal(moved.source_snapshot.entry_path,newPath);
    assert(!moved.source_snapshot.files.some(file=>file.path===oldPath));
    assert(moved.source_snapshot.files.some(file=>file.path===newPath));
    assert.deepEqual(moved.authored_identity.modules.map(module=>module.id),registered.authored_identity.modules.map(module=>module.id));
    const moveRequest={original_json:renamedText,source_snapshot:moved.source_snapshot,old_path:oldPath,new_path:newPath};
    await page.locator('[data-authored-clone]').click();await statusContains('새 ID로 복제');
    const clonedText=await getText(),cloned=JSON.parse(clonedText);
    assert.notEqual(cloned.authored_identity.logical_dependency_id,moved.authored_identity.logical_dependency_id);
    assert.equal(cloned.authored_identity.clone_origin,moved.authored_identity.logical_dependency_id);
    assert.deepEqual(cloned.source_snapshot,moved.source_snapshot);
    for(const module of cloned.authored_identity.modules)assert(!moved.authored_identity.modules.some(old=>old.id===module.id));
    await page.locator('[data-authored-save]').click();await statusContains('이 기기에 저장했습니다');
    await page.screenshot({path:path.join(output,'identity-renamed-moved-cloned.png'),fullPage:true});
    fs.writeFileSync(path.join(output,'identity-rename-request.json'),JSON.stringify(renameRequest));
    fs.writeFileSync(path.join(output,'identity-rename-invalid.json'),JSON.stringify(invalidRename));
    fs.writeFileSync(path.join(output,'identity-move-request.json'),JSON.stringify(moveRequest));
    fs.writeFileSync(path.join(output,'identity-moved.json'),movedText);fs.writeFileSync(path.join(output,'identity-cloned.json'),clonedText);
    const cloneVm=wasm.DdnWasmVm.from_authored_project_json(clonedText);
    cloneVm.step_one();cloneVm.step_one();const cloneHash2=cloneVm.get_state_hash();
    cloneVm.step_one();cloneVm.step_one();const cloneHash4=cloneVm.get_state_hash();cloneVm.free();
    const native=[];
    for(const bin of JSON.parse(process.env.DDN_CLI_BINS||'[]')){
      const invoke=(operation,file)=>spawnSync(bin,['작품자료',operation,path.join(output,file)],{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
      for(const [operation,file,expected] of [['이름변경','identity-rename-request.json',renamed],['경로변경','identity-move-request.json',moved]]){
        const run=invoke(operation,file);assert.ifError(run.error);assert.equal(run.status,0,run.stderr);assert.deepEqual(JSON.parse(run.stdout),expected);
      }
      const bad=invoke('이름변경','identity-rename-invalid.json');assert.ifError(bad.error);assert.notEqual(bad.status,0);assert.equal(bad.stdout,'');
      const clone=invoke('복제','identity-moved.json');assert.ifError(clone.error);assert.equal(clone.status,0,clone.stderr);
      const nativeClone=JSON.parse(clone.stdout);assert.equal(nativeClone.authored_identity.clone_origin,moved.authored_identity.logical_dependency_id);assert.notEqual(nativeClone.authored_identity.logical_dependency_id,cloned.authored_identity.logical_dependency_id);assert.deepEqual(nativeClone.source_snapshot,moved.source_snapshot);
      const store=fs.mkdtempSync(path.join(output,'identity-native-store-'));
      const run=spawnSync(bin,['작품자료','실행',path.join(output,'identity-cloned.json'),'--마디수','2','--저장소',store,'--부모','처음','--판','0d','--저장소-초기화'],{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
      assert.ifError(run.error);assert.equal(run.status,0,run.stderr);const savedClone=JSON.parse(run.stdout);
      assert.equal(savedClone.summary.state_hash,cloneHash2);
      const savedFile=path.join(store,'saved.json');fs.writeFileSync(savedFile,savedClone.saved_artifact_json_utf8);
      const continued=spawnSync(bin,['작품자료','이어실행',savedFile,'--부모','처음','--판','0d','--마디수','2','--저장소',store,'--거래',savedClone.transaction_digest_sha256,'--일반'],{encoding:'utf8',timeout:30000,maxBuffer:16*1024*1024});
      assert.ifError(continued.error);assert.equal(continued.status,0,continued.stderr);assert.equal(JSON.parse(continued.stdout).state_hash,cloneHash4);
      native.push({bin,rename_move_equal:true,clone_fresh_ids:true,rejected_no_stdout:true,run_state_hash:cloneHash2,cold2plus2_equals4:cloneHash4});
    }
    const url=page.url();await browser.close();browser=await chromium.launchPersistentContext(profileDirectory,browserOptions);
    page=browser.pages()[0]??await browser.newPage();attachPageEvidence(page);await page.goto(url);
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
    await openPanel();await page.locator('[data-authored-reopen]').click();await statusContains('저장 자료를 다시 열었습니다');
    assert.deepEqual(JSON.parse(await getText()),cloned,'real browser process restart preserves renamed/moved clone');
    assert.equal(await page.evaluate(async id=>JSON.stringify(await (await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js')).loadRuntimeProject(id)),projectId),storedBefore,'clone/source operations preserve the original runtime save');
    await page.locator('[data-authored-close]').click();
    // Idle documents start with the real run control; stepping is only for
    // an already started, paused execution, not a hidden initialization API.
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash);
    await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.pauseRun());
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='paused');
    const cloneLive=(await state()).frame;
    const committedCount=Number(cloneLive.wasm_state.resources.value_json.진행);
    assert(Number.isSafeInteger(committedCount)&&committedCount>0,'authored counter advances once per committed madi');
    const expectedClone=wasm.DdnWasmVm.from_authored_project_json(clonedText);
    for(let tick=0;tick<committedCount;tick++)expectedClone.step_one();
    assert.equal(cloneLive.state_hash,expectedClone.get_state_hash());
    await openPanel();await page.locator('[data-authored-runtime-save]').click();await statusContains('실행 상태를 저장하고 다시 읽어 검산');
    await browser.close();browser=await chromium.launchPersistentContext(profileDirectory,browserOptions);
    page=browser.pages()[0]??await browser.newPage();attachPageEvidence(page);await page.goto(url);
    await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready');
    await openPanel();await page.locator('[data-authored-reopen]').click();await statusContains('저장 자료를 다시 열었습니다');
    await page.locator('[data-authored-runtime-reopen]').click();await statusContains('실행 상태를 복원');
    assert.equal((await state()).frame.state_hash,cloneLive.state_hash,'cold reopen restores the clone, not its original project');
    await page.locator('[data-authored-close]').click();await page.getByTestId('lifecycle-step').click();expectedClone.step_one();
    await page.waitForFunction(hash=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.state_hash===hash,expectedClone.get_state_hash());expectedClone.free();
    assert.equal(await page.evaluate(async id=>JSON.stringify(await (await import('/solutions/seamgrim_ui_mvp/ui/workbench/persistence/runtime_project_store.js')).loadRuntimeProject(id)),projectId),storedBefore,'saving and cold-resuming the clone does not replace the original runtime head');
    identityLifecycle={status:'PASS_RENAME_RELOCATE_CLONE_NATIVE3_UI_COLD_REOPEN',native,original_runtime_preserved:true,failed_rename_preserves_vm_source_artifact_record:true,clone_runtime_cold_resume:true};
  }
  assert.ok(served.length);for(const hash of served)assert.equal(hash,expectedHash);
  assert.deepEqual(errors,[]);
  await page.screenshot({path:path.join(output,'root-tensor-execution.png'),fullPage:true});
  const result={status:'PASS_ROOT_UI',wasm_sha256:expectedHash,state_hash:frame.state_hash,errors,output,
    indexeddb_strict_cas_reread:'PASS',rust_restoration_of_exact_stored_bytes:'PASS',
    runtime_save_reopen:'PASS_SAME_OPEN_AND_BROWSER_PROCESS_RESTART',rd669_receipt:'NOT_IMPLEMENTED'};
  result.driver_sha256=createHash('sha256').update(fs.readFileSync(new URL(import.meta.url))).digest('hex');
  const releasePath=path.join(root,'RELEASE_MANIFEST.json');
  if(fs.existsSync(releasePath)){
    const releaseBytes=fs.readFileSync(releasePath),release=JSON.parse(releaseBytes);
    assert.ok(/^[0-9a-f]{40}$/.test(release.source?.commit));
    assert.ok(/^[0-9a-f]{40}$/.test(release.source?.tree));
    result.source_commit=release.source.commit;result.source_tree=release.source.tree;
    result.release_manifest_sha256=createHash('sha256').update(releaseBytes).digest('hex');
    result.target_kind='EXTRACTED_PACKAGE';
  }else{
    result.source_commit=spawnSync('git',['--no-optional-locks','rev-parse','HEAD'],{encoding:'utf8'}).stdout.trim();
    result.target_kind='SOURCE_CHECKOUT';
  }
  result.restored_tape_export='EXPLICIT_REJECT_MISSING_FULL_HISTORY';
  result.resource_restore_atomicity=atomicRestoreCases;
  result.storage_failure_preservation={...storageFailures,stored:undefined};
  result.native_resume=nativeResume.length?nativeResume:'NOT_RUN';
  result.runtime_artifact_path=runtimeFile;
  result.browser_runtime_export={file:downloadedFile,restore:'PASS_EXACT_UI_HASH',native_continuation:process.env.DDN_CLI_BINS?'PASS_THREE_NAMES':'NOT_RUN'};
  result.continuation=liveTest?'PASS_ONE_MADI_EXACT_RUST_HASH':'NOT_APPLICABLE_ONESHOT';
  result.module_edit_save_cold_reopen=editModules?'PASS_THREE_FILES_IMMEDIATE_AND_COLD_DEPENDENT_LIBRARY_VALUE_4_TO_7':'NOT_RUN';
  result.project_creation=createFromUi?'PASS_REAL_UI_TEMPLATE_FILES_EXPLICIT_IDENTITY_AND_SAVE':'COMMON_RUST_CREATE_THEN_UI_IMPORT';
  result.typed_tuple=typedTuple?'SOURCE_BOUND_COLD_RESTORE_NATIVE3_AND_TWO_ATOMIC_NEGATIVES':'NOT_RUN';
  result.typed_tuple_declared_calls=declaredCallTuple?'EXPLICIT_PINS_AND_DECLARED_RESULTS_THROUGH_ROOT_TUPLE_COLD_RESTORE':'NOT_RUN';
  result.literal_tuple_defaults=literalTuple?'RAW_INTEGER_DECIMAL_DISTINCT_CHILD_TYPES_COLD_RESTORE':'NOT_RUN';
  result.local_sum_source=localSum?'RAW_MODULE_SCHEMA_AND_LOCAL_CALCULATION_COLD_RESTORE_NATIVE3; NO_SUM_VALUE_STORAGE':'NOT_RUN';
  result.typed_tuple_static_expression=staticTuple?'RAW_AUTHORED_SOURCE_CONSTANT_SELECTION_REPLACEMENT_COLD_RESTORE_NATIVE3':'NOT_RUN';
  result.logical_storage_owner=logicalStorageOwner??'NOT_RUN';
  result.identity_lifecycle=identityLifecycle;
  fs.writeFileSync(path.join(output,'report.json'),JSON.stringify(result,null,2));console.log(JSON.stringify(result));
}catch(error){
  if(page){const diagnostic=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot());
    fs.writeFileSync(path.join(output,'failure.json'),JSON.stringify(diagnostic,null,2));
    await page.screenshot({path:path.join(output,'failure.png')});
    console.error(JSON.stringify({output,diagnostics:diagnostic?.diagnostics,errors}));}
  throw error;
}finally{await browser.close();await new Promise(resolve=>server.close(resolve));}
