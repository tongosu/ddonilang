import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {chromium} from 'playwright';
import {ALPHA1_LESSON_PROJECTS, DEFERRED_LESSONS} from '../solutions/seamgrim_ui_mvp/ui/workbench/alpha_lesson_catalog.js';

const base = process.env.DDN_TEST_BASE_URL;
assert(base, 'isolated product-server URL is required; never use the Owner server implicitly');
const requestedIds = process.env.DDN_LESSON_IDS?.split(',').filter(Boolean);
if (requestedIds) assert(requestedIds.length > 0, 'focused lesson selection must not be empty');
if (requestedIds) for (const id of requestedIds) assert(ALPHA1_LESSON_PROJECTS.some(row=>row.id===id), `unknown product lesson ${id}`);
const selectedLessons = requestedIds ? ALPHA1_LESSON_PROJECTS.filter(row=>requestedIds.includes(row.id)) : ALPHA1_LESSON_PROJECTS;
const browser = await chromium.launch({headless:true, executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const context = await browser.newContext();
const errors = [];
const requestFailures = [];
const pageErrors = [];
context.on('response', response => {if(response.status()>=400) errors.push({url:response.url(),status:response.status()});});
context.on('requestfailed', request => requestFailures.push(request.url()));
const page = await context.newPage();
page.on('pageerror', error => pageErrors.push(String(error)));
const results = [];
// These authored lessons plot a complete static domain within each madi.
// Their meta goals require the graph, not artificial motion between identical
// graphs. Replace the motion-only oracle with an exact full-interval oracle.
const fullIntervalOracles = {
  math_line: {pairs:41, start:[-4,-7], end:[4,9]},
  ssot_edu_phys_p004_02_voltage_divider_Vin_Vout: {pairs:41, start:[0,0], end:[10,20/3]},
  ssot_edu_phys_p004_03_parallel_resistors_IV: {pairs:41, start:[0,0], end:[10,1.5]},
};
// Complete static curves repeat per madi: verify every actual point against
// its two original observations instead of inventing moving-marker behavior.
const fullCurveOracles = {
  ssot_edu_phys_p001_04_free_fall_y_t: {pairs:64, first:[0,50]},
  ssot_edu_phys_p001_05_projectile_xy: {pairs:58, first:[0,0]},
};
const growingPairCurves = {
  ssot_edu_phys_p013_03_adiabatic_pv_v_p: {maxPairs:190,reference:100},
  ssot_edu_phys_p018_03_debroglie_lambda_of_p: {maxPairs:98,reference:1},
  ssot_edu_phys_p018_04_wien_lambda_max_of_T: {maxPairs:90,reference:2.9},
};
try {
  await page.goto(base);
  await page.waitForFunction(() => window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().projectControls?.status === 'ready');
  await page.getByTestId('global-official-collection').click();
  await page.getByTestId('project-rail-toggle').click();
  for (const row of DEFERRED_LESSONS) {
    const card = page.getByTestId(`deferred-lesson-${row.id}`);
    assert((await card.textContent()).includes('이월 · 알파2'));
    assert.equal(await card.getAttribute('role'), null, 'deferred history must not be a runnable option');
  }
  for (const lesson of selectedLessons) {
    if (!(await page.getByTestId(`project-card-${lesson.id}`).isVisible())) await page.getByTestId('project-rail-toggle').click();
    await page.getByTestId(`project-card-${lesson.id}`).click();
    await page.waitForFunction(id => {
      const state = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return state.projectId === id && state.projectControls?.project_id === id && state.projectControls?.status === 'ready';
    }, lesson.id, {timeout:60000});
    const source = fs.readFileSync(path.join('solutions/seamgrim_ui_mvp/lessons_v25_alpha1',lesson.id,'lesson.ddn'),'utf8');
    const intentPath = path.join('solutions/seamgrim_ui_mvp/lessons_v25_alpha1',lesson.id,'intent.json');
    const intent = JSON.parse(fs.readFileSync(fs.existsSync(intentPath) ? intentPath : intentPath.replace('intent.json','intention.json'),'utf8'));
    assert.equal(await page.locator('#source-editor').inputValue(), source, `${lesson.id}: fetch source bytes`);
    const expectedControls = [...source.matchAll(/^\s*([^\s:]+):\s*수\s*:=.*매김/gmu)].length;
    await page.waitForFunction(count => document.querySelectorAll('[data-testid="setup-control-list"] input').length === count, expectedControls);
    const controls = await page.locator('[data-testid="setup-control-list"] input').evaluateAll(nodes => nodes.map(node => ({name:node.dataset.controlName,value:node.value,min:node.min,max:node.max,step:node.step})));
    const pinNames = [...source.matchAll(/^\s*([^\s:]+):\s*수\s*:=.*매김/gmu)].map(match=>match[1]);
    assert.deepEqual(controls.map(row=>row.name).sort(), pinNames.sort(), `${lesson.id}: all original controls visible`);
    const teachingFile = lesson.files.find(file=>file.path==='text.md');
    if (teachingFile) {
      await page.getByTestId('file-tab-text-md').click();
      const teachingText = fs.readFileSync(path.join('solutions/seamgrim_ui_mvp/lessons_v25_alpha1',lesson.id,'text.md'),'utf8');
      assert.equal(await page.locator('#source-editor').inputValue(),teachingText,`${lesson.id}: original teaching text visible`);
      // Keep the text tab selected: execution must still use the declared lesson entry.
    }
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(() => {
      const state = window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
      return state.activeFrame || state.executionLifecycle.status === 'failed';
    },null,{timeout:60000});
    let snapshot = await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert.notEqual(snapshot.executionLifecycle.status,'failed',`${lesson.id}: ${JSON.stringify(snapshot.diagnostics)}`);
    if (snapshot.executionLifecycle.status === 'running') {
      await page.getByTestId('lifecycle-run').click();
      await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='paused');
    }
    const manualSteps = lesson.id === 'rep_science_phase_change_timeline_v1' ? 55 : 3;
    for(let index=0;index<manualSteps;index++) await page.evaluate(async()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.stepForward());
    snapshot = await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
    assert(snapshot.runs.at(-1).frame_count >= 4,`${lesson.id}: actual history`);
    // Numeric-only lessons require a graph, not a fabricated table protocol.
    // Keep the table assertion for every explicitly required or emitted table.
    if (lesson.id !== 'rep_science_phase_change_timeline_v1' &&
        ((intent.required_views || []).includes('table') || source.includes('"table.row" 보여주기.'))) {
      assert(await page.getByTestId('lesson-output-table').isVisible(), `${lesson.id}: committed output table visible`);
      assert((await page.getByTestId('lesson-output-table').locator('tbody tr').count()) > 0);
    }
    const renderer = await page.locator('#canvas-bogae').getAttribute('data-renderer-family');
    assert.equal(renderer,lesson.primaryView==='graph'?'space2d':'console',`${lesson.id}: primary renderer`);
    let trailSegments = 0;
    if(lesson.primaryView==='graph') {
      assert.equal(await page.getByTestId('spatial-trail').getAttribute('aria-pressed'),'true',`${lesson.id}: trend trail`);
      trailSegments = Number(await page.locator('#canvas-bogae').getAttribute('data-trail-segments'));
      const interval = fullIntervalOracles[lesson.id];
      const curve = fullCurveOracles[lesson.id];
      if (curve) {
        const state = snapshot.activeFrame.wasm_state;
        const drawlist = state.view_meta?.space2d?.drawlist;
        assert.equal(drawlist?.length,1,`${lesson.id}: one complete original curve`);
        assert.equal(drawlist[0].kind,'polyline');
        const points = drawlist[0].points;
        assert.equal(points.length,curve.pairs,`${lesson.id}: all original curve samples`);
        const outputs = state.resources?.value_json?.output_log ?? [];
        assert.equal(outputs.length,snapshot.runs.at(-1).frame_count*curve.pairs*2,`${lesson.id}: original pairs in every actual madi`);
        const batch = outputs.slice(-curve.pairs*2);
        for(let index=0;index<points.length;index++)
          for(const [axis,offset] of [['x',0],['y',1]])
            assert(Math.abs(Number(points[index][axis])-Number(batch[index*2+offset]?.text))<=1e-6,
              `${lesson.id}: original output/curve ${index}/${axis}`);
        for(const [axis,offset] of [['x',0],['y',1]])
          assert(Math.abs(Number(points[0][axis])-curve.first[offset])<=1e-6,
            `${lesson.id}: original curve start ${axis}`);
      } else if (growingPairCurves[lesson.id]) {
        const oracle = growingPairCurves[lesson.id];
        const state = snapshot.activeFrame.wasm_state;
        const curves = state.view_meta?.space2d?.drawlist;
        assert.equal(curves?.length,2,`${lesson.id}: both required original curves`);
        const outputs = state.resources?.value_json?.output_log ?? [];
        const count = Math.min(snapshot.runs.at(-1).frame_count,oracle.maxPairs);
        assert.equal(outputs.length,count*2,`${lesson.id}: original observations only`);
        for (const line of curves) {
          assert.equal(line.kind,'polyline');
          assert.equal(line.points.length,count,`${lesson.id}: complete growing sample history`);
        }
        for(let index=0;index<count;index++) {
          const volume = Number(outputs[index*2].text);
          const pressure = Number(outputs[index*2+1].text);
          assert(Math.abs(Number(curves[0].points[index].x)-volume)<=1e-6);
          assert(Math.abs(Number(curves[0].points[index].y)-pressure)<=1e-6);
          assert(Math.abs(Number(curves[1].points[index].x)-volume)<=1e-6);
          assert(Math.abs(Number(curves[1].points[index].y)-oracle.reference/volume)<=1e-6);
        }
      } else if (!interval) {
        assert(trailSegments > 0, `${lesson.id}: actual committed motion history, not merely enabled toggle`);
      } else {
        const state = snapshot.activeFrame.wasm_state;
        const drawlist = state.view_meta?.space2d?.drawlist;
        assert.equal(drawlist?.length,4,`${lesson.id}: real full-domain graph shapes`);
        const line = drawlist[2];
        assert.equal(line.kind,'line');
        for (const [key,value] of Object.entries({x1:interval.start[0],y1:interval.start[1],x2:interval.end[0],y2:interval.end[1]}))
          assert(Math.abs(Number(line[key])-value)<=1e-6,`${lesson.id}: full-domain ${key}`);
        const outputs = state.resources?.value_json?.output_log ?? [];
        assert.equal(outputs.length,snapshot.runs.at(-1).frame_count*interval.pairs*2,`${lesson.id}: every original sample pair in every actual madi`);
        const lastBatch=outputs.slice(-interval.pairs*2);
        for (const [offset,value] of [[0,interval.start[0]],[1,interval.start[1]],[lastBatch.length-2,interval.end[0]],[lastBatch.length-1,interval.end[1]]])
          assert(Math.abs(Number(lastBatch[offset]?.text)-value)<=1e-6,`${lesson.id}: actual endpoint observation ${offset}`);
      }
    }
    let override = null;
    if(controls.length) {
      const pin = controls[0];
      const originalValue = Number(pin.value);
      const newValue = originalValue + Number(pin.step || 1) <= Number(pin.max) ? originalValue + Number(pin.step || 1) : originalValue - Number(pin.step || 1);
      await page.locator(`[data-testid="setup-control-list"] input[data-control-name="${pin.name}"]`).fill(String(newValue));
      await page.getByTestId('setup-apply').click();
      await page.getByTestId('lifecycle-stop').click();
      await page.getByTestId('lifecycle-run').click();
      const expectedRaw = String(BigInt(Math.round(newValue*4294967296)));
      await page.waitForFunction(({name,raw}) => window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeFrame?.wasm_state?.param_overrides?.[name]?.raw_i64 === raw, {name:pin.name,raw:expectedRaw});
      const changed = await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      const raw = changed.activeFrame.wasm_state.param_overrides[pin.name]?.raw_i64;
      assert.equal(raw,String(BigInt(Math.round(newValue*4294967296))),`${lesson.id}: actual WASM changed control`);
      override = {name:pin.name,value:newValue,raw};
    }
    const sourceEdits = [];
    for (const edit of intent.source_edit_checks || []) {
      assert.equal(source.split(edit.replace_from).length,2,`${lesson.id}: authored edit must match once`);
      if (await page.getByTestId('lifecycle-stop').isEnabled()) await page.getByTestId('lifecycle-stop').click();
      if (teachingFile) await page.getByTestId('file-tab-lesson-ddn').click();
      const previousRun = await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeRunId);
      await page.locator('#source-editor').fill(source.replace(edit.replace_from,edit.replace_to));
      await page.locator('#source-editor').press('Escape');
      await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadiness.ready);
      assert(Number.isInteger(edit.ticks) && edit.ticks>0,'authored edit requires exact positive madi count');
      // The existing step button resumes a paused run; it does not start one.
      await page.getByTestId('lifecycle-run').click();
      await page.waitForFunction(id=>{
        const state=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();
        return state.activeRunId!==id && (state.activeFrame || state.executionLifecycle.status==='failed');
      },previousRun,{timeout:60000});
      let edited=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      assert.notEqual(edited.executionLifecycle.status,'failed',JSON.stringify(edited.diagnostics));
      if(edited.executionLifecycle.status==='running') {
        await page.getByTestId('lifecycle-run').click();
        await page.waitForFunction(()=>['paused','done','failed'].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status));
        edited=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
        assert.equal(edited.executionLifecycle.status,'paused',
          `${lesson.id}: pause must commit, not host stop or failure: ${JSON.stringify(edited.executionLifecycle)}`);
      }
      while(edited.runs.at(-1).frame_count<edit.ticks) {
        const count=edited.runs.at(-1).frame_count;
        await page.getByTestId('lifecycle-step').click();
        await page.waitForFunction(count=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().runs.at(-1).frame_count>count,count);
        edited=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      }
      while(edited.cursorIndex>edit.ticks-1) {
        const index=edited.cursorIndex;
        await page.getByTestId('history-back').click();
        await page.waitForFunction(index=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().cursorIndex===index-1,index);
        edited=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      }
      assert.equal(edited.cursorIndex,edit.ticks-1,'inspect the actual recorded authored-edit madi');
      const value=Number(edited.activeFrame.wasm_state.resources.fixed64[edit.resource_field]);
      assert(Number.isFinite(value));
      assert(Math.abs(value-edit.expected_resource_value)<=1e-6,`${lesson.id}: editor change must reach actual Worker/WASM`);
      const checkedValues=[];
      for(const check of edit.checks || []) {
        const actual=Number(edited.activeFrame.wasm_state.resources.fixed64[check.field]);
        assert(Number.isFinite(actual),`${lesson.id}: edited world field ${check.field} is missing`);
        // Browser resources are the existing six-place presentation surface.
        assert(Math.abs(actual-check.expected)<=Math.max(check.tolerance || 0,1e-6),
          `${lesson.id}: edited madi ${edit.ticks} ${check.field}=${actual}, expected ${check.expected}`);
        checkedValues.push({field:check.field,expected:check.expected,actual});
      }
      sourceEdits.push({ticks:edit.ticks,field:edit.resource_field,expected:edit.expected_resource_value,actual:value,
        checkedValues,state_hash:edited.activeFrame.state_hash});
      // Recorded history is intentionally read-only. Use the real branch action
      // to return to an editable source, never remove the readonly attribute.
      if(edited.sourceReadOnly) {
        await page.getByTestId('history-branch').click();
        await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadOnly);
      }
      if(await page.getByTestId('lifecycle-stop').isEnabled()) await page.getByTestId('lifecycle-stop').click();
      // Stopping can commit the in-flight last frame. Re-read the actual UI
      // after that transition rather than trusting the earlier history cursor.
      await page.waitForFunction(()=>!['running','preparing'].includes(window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status));
      const stopped = await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
      if(stopped.sourceReadOnly) {
        await page.getByTestId('history-branch').click();
        await page.waitForFunction(()=>!window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().sourceReadOnly);
      }
      await page.locator('#source-editor').fill(source);
    }
    results.push({id:lesson.id,controls:controls.length,frames:snapshot.runs.at(-1).frame_count,renderer,trailSegments,override,teachingTextVisible:Boolean(teachingFile),sourceEdits});
    console.log(`BROWSER_LESSON_PASS ${lesson.id}`);
  }
  assert.deepEqual(errors,[],'HTTP errors including Worker/WASM');
  assert.deepEqual(requestFailures,[],'failed requests');
  assert.deepEqual(pageErrors,[],'browser errors');
  const report={lessons:results.length,scope:requestedIds?'focused':'full_catalog',deferred:4,errors,requestFailures,pageErrors,results};
  if(process.env.DDN_LESSON_BROWSER_REPORT){assert(!fs.existsSync(process.env.DDN_LESSON_BROWSER_REPORT));fs.writeFileSync(process.env.DDN_LESSON_BROWSER_REPORT,JSON.stringify(report,null,2));}
  console.log(JSON.stringify({...report,results:undefined}));
} catch (error) {
  const failure={status:'FAIL',message:String(error),completed:results,
    errors,requestFailures,pageErrors,hostStopNotice:await page.getByTestId('host-stop-notice').textContent().catch(()=>null),editor:await page.locator('#source-editor').evaluate(node=>({
      readOnly:node.readOnly,disabled:node.disabled,display:getComputedStyle(node).display,
      visibility:getComputedStyle(node).visibility,rect:node.getBoundingClientRect().toJSON(),
      mode:document.querySelector('[data-testid="workbench-layout"]')?.dataset.mode,
      activeFile:document.querySelector('[role="tab"][aria-selected="true"]')?.textContent,
    })).catch(()=>null),snapshot:await page.evaluate(()=>{
      const state=window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot();
      return state ? {projectId:state.projectId,activeRunId:state.activeRunId,
        executionLifecycle:state.executionLifecycle,diagnostics:state.diagnostics,
        runs:state.runs,cursorIndex:state.cursorIndex} : null;
    }).catch(()=>null)};
  if(process.env.DDN_LESSON_BROWSER_REPORT){assert(!fs.existsSync(process.env.DDN_LESSON_BROWSER_REPORT));fs.writeFileSync(process.env.DDN_LESSON_BROWSER_REPORT,JSON.stringify(failure,null,2));}
  throw error;
} finally {await context.close();await browser.close();}
