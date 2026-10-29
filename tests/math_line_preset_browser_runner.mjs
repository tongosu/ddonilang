import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {chromium} from 'playwright';
import {ALPHA1_LESSON_PROJECTS} from '../solutions/seamgrim_ui_mvp/ui/workbench/alpha_lesson_catalog.js';
const url=process.env.DDN_TEST_BASE_URL;
assert(url,'explicit isolated product URL required');
const target=new URL(url);
assert(['127.0.0.1','localhost'].includes(target.hostname)&&target.port!=='8790','local isolated URL required; Owner 8790 forbidden');
const path='solutions/seamgrim_ui_mvp/lessons_v25_alpha1/math_line/';
const preset=fs.readFileSync(path+'inputs/preset_1.ddn','utf8');
const source=fs.readFileSync(path+'lesson.ddn','utf8');
const intent=JSON.parse(fs.readFileSync(path+'intent.json','utf8'));
const workbench=fs.readFileSync('solutions/seamgrim_ui_mvp/ui/workbench/workbench.js','utf8');
const body=workbench.slice(workbench.indexOf('function currentProjectDdnSources('),workbench.indexOf('\nfunction declaredActionMapFromProject('));
const sources=new Function('project','state',body+'\nreturn currentProjectDdnSources(project);');
const files=['root','module','main','meta'].map(role=>({path:role+'.ddn',role,fallback:'원문'}));
assert.deepEqual(sources({files},{fileTexts:new Map()}).map(row=>row.source_path),['root.ddn','module.ddn','main.ddn']);
const lesson=ALPHA1_LESSON_PROJECTS.find(row=>row.id==='math_line');
assert.deepEqual(lesson.files.map(file=>[file.path,file.role]),[['lesson.ddn','main'],['text.md','meta'],['inputs/preset_1.ddn','meta'],['inputs/README.md','meta']]);
const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
const context=await browser.newContext(),page=await context.newPage(),errors=[];
context.on('response',response=>{if(response.status()>=400)errors.push(response.url()+':'+response.status());});
page.on('pageerror',error=>errors.push(String(error)));
const results=[];
async function pins(){
 await page.waitForFunction(()=>document.querySelectorAll('[data-testid="setup-control-list"] input').length===5);
 const values=await page.locator('[data-testid="setup-control-list"] input').evaluateAll(nodes=>nodes.map(node=>({name:node.dataset.controlName,value:node.value,min:node.min,max:node.max,step:node.step})));
 assert.deepEqual(values.map(v=>v.name).sort(),['a','b','dx','x_end','x_start']);
 for(const pin of values){const expected=intent.controls[pin.name];assert.equal(Number(pin.min),expected.min);assert.equal(Number(pin.max),expected.max);assert.equal(Number(pin.step),expected.step);}
 return values;
}
async function run(expected,label){
 const old=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().activeRunId);
 await page.getByTestId('lifecycle-run').click();
 await page.waitForFunction(old=>{const s=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();return s.activeRunId!==old&&(s.activeFrame||s.executionLifecycle.status==='failed');},old,{timeout:30000});
 let s=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
 assert.notEqual(s.executionLifecycle.status,'failed',JSON.stringify(s.diagnostics));
 if(s.executionLifecycle.status==='running'){await page.getByTestId('lifecycle-run').click();await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().executionLifecycle.status==='paused');}
 s=await page.evaluate(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot());
 const state=s.activeFrame.wasm_state,channels=state.channels??[],row=state.row??[];
 const values=Object.fromEntries(channels.map((c,i)=>[typeof c==='string'?c:c.key??c.name,row[i]]));
 for(const [field,value] of Object.entries(expected))assert(Math.abs(Number(values[field])-value)<1e-6,label+':'+field+' '+JSON.stringify(values));
 assert.equal(await page.locator('#canvas-bogae').getAttribute('data-renderer-family'),'space2d');
 const drawlist=state.view_meta?.space2d?.drawlist;
 // 원 모양의 동일4개 ID는 표본마다 갱신된다. 전체 표본은 출력 로그로 따로 확인한다.
 assert.equal(drawlist?.length,4,label+': original final 4 shape identities');
 const outputs=state.resources?.value_json?.output_log??[];
 const sampleCount=expected.그림개수;
 assert(outputs.length>=sampleCount*2 && outputs.length%(sampleCount*2)===0,label+': complete x/y sample outputs');
 const lastBatch=outputs.slice(-sampleCount*2);
 const xStart=label==='default'?-4:-6,a=label==='default'||label==='preset a=2'?2:1.2,b=label==='default'?1:0.5;
 const line=drawlist[2],xLast=xStart+(sampleCount-1)*0.2;
 for(const [field,value] of Object.entries({x1:xStart,y1:a*xStart+b,x2:xLast,y2:a*xLast+b}))
  assert(Math.abs(Number(line[field])-value)<1e-6,label+': full interval line '+field+' '+JSON.stringify(line));
 for(let index=0;index<sampleCount;index++){
  const x=xStart+index*0.2,y=a*x+b;
  assert(Math.abs(Number(lastBatch[index*2].text)-x)<1e-6,label+': x sample '+index);
  assert(Math.abs(Number(lastBatch[index*2+1].text)-y)<1e-6,label+': y sample '+index);
 }
 results.push({label,values:Object.fromEntries(Object.keys(expected).map(key=>[key,values[key]])),state_hash:state.state_hash,renderer:'space2d',drawlist_count:drawlist.length,full_interval_line:line,last_batch_numeric_outputs:lastBatch.length});
 return s;
}
try{
 await page.goto(url);
 await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().projectControls?.status==='ready');
 await page.getByTestId('global-official-collection').click();
 await page.getByTestId('project-rail-toggle').click();
 await page.getByTestId('project-card-math_line').click();
 await page.waitForFunction(()=>{const s=window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot();return s.projectId==='math_line'&&s.projectControls.status==='ready';});
 assert.equal(await page.locator('#source-editor').inputValue(),source);
 const defaultPins=await pins();for(const pin of defaultPins)assert.equal(Number(pin.value),intent.controls[pin.name].default);
 await run({그림개수:41,그림시작y:-7,y:9},'default');
 for(const [tab,file] of [['file-tab-text-md','text.md'],['file-row-inputs-README-md','inputs/README.md'],['file-tab-inputs-preset-1-ddn','inputs/preset_1.ddn']]){
  await page.getByTestId(tab).click();assert.equal(await page.locator('#source-editor').inputValue(),fs.readFileSync(path+file,'utf8'));await pins();
 }
 await page.getByTestId('lifecycle-stop').click();
 if(!await page.getByTestId('new-ddn-project').isVisible())await page.getByTestId('project-rail-toggle').click();
 await page.getByTestId('new-ddn-project').click();
 await page.getByTestId('project-create-name').fill('math_line preset 독립검증');
 await page.locator('input[name="project-template"][value="single"]').check();
 await page.getByTestId('project-create-submit').click();
 await page.waitForFunction(()=>document.querySelector('#project-title')?.textContent==='math_line preset 독립검증');
 await page.locator('#source-editor').fill(preset);
 await page.locator('#source-editor').press('Escape');
 const presetPins=await pins();
 for(const [name,value] of Object.entries({a:1.2,b:0.5,x_start:-6,x_end:6,dx:0.2}))assert.equal(Number(presetPins.find(p=>p.name===name).value),value);
 await run({그림개수:61,그림시작y:-6.7,y:7.7},'preset');
 await page.locator('[data-testid="setup-control-list"] input[data-control-name="a"]').fill('2');
 await page.getByTestId('setup-apply').click();
 await page.getByTestId('lifecycle-stop').click();
 const changed=await run({그림개수:61,그림시작y:-11.5,y:12.5},'preset a=2');
 assert.equal(changed.activeFrame.wasm_state.param_overrides.a.raw_i64,'8589934592');
 const response=await context.request.get(new URL('../wasm/ddonirang_tool_bg.wasm',url).href);
 const wasmHash=createHash('sha256').update(await response.body()).digest('hex');
 const manifest=JSON.parse(fs.readFileSync('solutions/seamgrim_ui_mvp/ui/wasm/ddonirang_tool.manifest.detjson'));
 assert.equal(wasmHash,manifest.wasm_sha256);assert.deepEqual(errors,[]);
 if(process.env.DDN_TEST_SCREENSHOT_PATH)await page.locator('#canvas-bogae').screenshot({path:process.env.DDN_TEST_SCREENSHOT_PATH});
 const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
 const report={status:'MATH_LINE_PRESET_FLOW_PASS',role_filter:'MAIN_ROOT_MODULE_PRESERVED_META_EXCLUDED',
  source_sha256:sha(source),preset_sha256:sha(preset),readme_sha256:sha(fs.readFileSync(path+'inputs/README.md')),
  wasm_sha256:wasmHash,results,network_errors:errors,zip:'NOT_RUN_PARENT_FIXED_ARCHIVE_REQUIRED'};
 if(process.env.DDN_TEST_REPORT_PATH)fs.writeFileSync(process.env.DDN_TEST_REPORT_PATH,JSON.stringify(report,null,2)+'\n',{flag:'wx'});
 console.log(JSON.stringify(report));
}finally{await context.close();await browser.close();}
