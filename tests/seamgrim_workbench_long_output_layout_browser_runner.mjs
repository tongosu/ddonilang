import assert from 'node:assert/strict';
import fs from 'node:fs';
import {chromium} from 'playwright';

const url=process.env.DDN_TEST_BASE_URL;
const reportPath=process.env.DDN_LAYOUT_REPORT;
assert(url && reportPath,'isolated server and external immutable report required');
assert(!fs.existsSync(reportPath));
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const context=await browser.newContext({viewport:{width:1366,height:768}});
const page=await context.newPage();
const errors=[];
page.on('pageerror',error=>errors.push(String(error)));
context.on('response',response=>{if(response.status()>=400)errors.push(`${response.status()} ${response.url()}`);});
const report={status:'NOT_RUN',errors};
try {
  await page.goto(url);
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().projectControls?.status==='ready');
  await page.getByTestId('global-official-collection').click();
  await page.getByTestId('project-rail-toggle').click();
  await page.getByTestId('project-card-ssot_edu_phys_p013_03_adiabatic_pv_v_p').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectId==='ssot_edu_phys_p013_03_adiabatic_pv_v_p');
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>{
    const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
    return state.activeFrame && state.executionLifecycle.status==='running';
  });
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='paused');
  while(await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1).frame_count)<40)
    await page.evaluate(async()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.stepForward());
  report.layout=await page.evaluate(()=>{
    const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
    const observations=document.querySelector('[data-testid="semantic-observations"]');
    return {frames:state.runs.at(-1).frame_count,state_hash:state.activeFrame.state_hash,
      output_count:state.activeFrame.wasm_state.resources.value_json.output_log.length,
      table_rows:document.querySelectorAll('[data-testid="lesson-output-table"] tbody tr').length,
      canvas:document.querySelector('#canvas-bogae').getBoundingClientRect().toJSON(),
      observations:{height:observations.clientHeight,scrollHeight:observations.scrollHeight,overflowY:getComputedStyle(observations).overflowY},
      sourceReadOnly:state.sourceReadOnly};
  });
  await page.screenshot({path:reportPath.replace(/\.json$/u,'.png'),fullPage:true});
  assert.equal(report.layout.frames,40);
  assert.equal(report.layout.output_count,80,'actual WASM observations must remain intact');
  assert(report.layout.table_rows>0,'all committed rows remain in the DOM');
  assert(report.layout.canvas.height<=768,'output rows must not stretch the graph beyond the viewport');
  assert(report.layout.observations.height<=384,'long output has a bounded presentation');
  assert.equal(report.layout.observations.overflowY,'auto');
  // 실제 다중행 입력과 복원은 같은 값의 여러 trusted input을 만들 수
  // 있다. 원문 변경은 처리하되 게시된 세계/마디를 바꾸면 안 된다.
  const editor=page.locator('#source-editor');
  const source=await editor.inputValue();
  await page.getByTestId('lifecycle-stop').click();
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.reason==='stopped');
  const before=await page.evaluate(()=>{
    const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
    return {frame:JSON.stringify(state.activeFrame),count:state.runs.at(-1).frame_count};
  });
  await editor.fill(source.replace('k <- 100.','k <- 120.'));
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectControls.status==='ready');
  const started=performance.now();
  await editor.fill(source);
  report.source_restore_ms=performance.now()-started;
  await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().projectControls.status==='ready');
  assert.equal(await editor.inputValue(),source);
  const after=await page.evaluate(()=>{
    const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
    return {frame:JSON.stringify(state.activeFrame),count:state.runs.at(-1).frame_count};
  });
  assert.deepEqual(after,before,'editing must preserve the committed frame and madi count');
  assert.deepEqual(errors,[]);
  report.status='PASS';
} catch(error) {report.status='FAIL';report.message=String(error);throw error;}
finally {
  fs.writeFileSync(reportPath,JSON.stringify(report,null,2),{flag:'wx'});
  await context.close(); await browser.close();
}
console.log(JSON.stringify(report));
