import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {chromium} from 'playwright';
const root=path.resolve(process.env.DDN_TEST_ROOT || '.');
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROME_EXECUTABLE || 'C:/Program Files/Google/Chrome/Application/chrome.exe'});
try {
  const page=await browser.newPage({viewport:{width:1360,height:860}});
  const url=process.env.DDN_TEST_URL || 'http://127.0.0.1:18840/';
  async function open() {
    await page.goto(url);
    await page.waitForFunction(()=>document.querySelector('#source-editor')?.value.includes('매마디'));
  }
  await open();
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.rendererFamily==='space2d');
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>document.querySelector('[data-testid="lifecycle-run"]')?.dataset.action==='resume');
  const first=await page.locator('#canvas-bogae').getAttribute('data-committed-madi');
  await page.getByTestId('lifecycle-step').click();
  await page.waitForFunction(n=>Number(document.querySelector('#canvas-bogae')?.dataset.committedMadi)>Number(n),first);
  const colors=await page.locator('#canvas-bogae').evaluate(c=>{
    const d=c.getContext('2d').getImageData(0,0,c.width,c.height).data;
    const colors=new Set();for(let i=0;i<d.length;i+=4)colors.add(`${d[i]},${d[i+1]},${d[i+2]}`);return colors.size;
  });
  assert.ok(colors>8,'Actual drawing absent');
  // Every advertised physics project must execute its actual shipped source.
  // Nonblank projectile pixels do not prove the pendulum example works.
  await open();
  await page.getByRole('button',{name:'프로젝트',exact:true}).click();
  await page.getByRole('option',{name:/진자 보기 코어/}).click();
  assert.doesNotMatch(await page.locator('#source-editor').inputValue(),/#기본관찰/);
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.rendererFamily==='space2d');
  await page.getByTestId('lifecycle-run').click();
  const observations=()=>page.getByTestId('semantic-observations').innerText();
  const bobX=text=>Number(text.match(/bob_x\s+(-?[\d.]+)/)?.[1]);
  const beforeBob=bobX(await observations());
  for(let i=0;i<12;i++) await page.getByTestId('lifecycle-step').click();
  const afterBob=bobX(await observations());
  assert.ok(Number.isFinite(beforeBob)&&Number.isFinite(afterBob)&&Math.abs(afterBob-beforeBob)>0.001,'Pendulum must advance, not just display a static canvas');
  const visibleBob=await page.locator('#canvas-bogae').evaluate(c=>{
    const d=c.getContext('2d').getImageData(0,0,c.width,c.height).data;
    let count=0;for(let i=0;i<d.length;i+=4)if(d[i]===56&&d[i+1]===189&&d[i+2]===248)count++;
    return count;
  });
  assert.ok(visibleBob>100,'Pendulum bob must be visibly sized');
  const canvasBox=await page.locator('#canvas-bogae').boundingBox();
  const controlsBox=await page.getByTestId('setup-controls').boundingBox();
  assert.ok(controlsBox.y>=canvasBox.y+canvasBox.height,'Initial controls belong below Bogae');
  const fixtures=[['값 := 100.\n값 보여주기.','100']];
  for(const [name,last] of [['01_요금_계산기.ddn','2100'],['02_조건과_반복.ddn','30'],['03_자동_함수표.ddn','20'],['04_시간단계_시뮬레이션.ddn','6']]) {
    fixtures.push([await fs.readFile(path.join(root,'solutions/seamgrim_ui_mvp/alpha1',name),'utf8'),last]);
  }
  for(const [source,last] of fixtures) {
    await open();
    await page.locator('#source-editor').fill(source);
    await page.getByTestId('lifecycle-run').click();
    await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.rendererFamily==='console');
    const output=await page.locator('#canvas-bogae').getAttribute('data-console-output');
    assert.equal(output.trim().split('\n').at(-1),last);
  }
  await open();
  await page.locator('#source-editor').fill('값 := .');
  await page.getByTestId('lifecycle-run').click();
  await page.getByTestId('project-verification-drawer').waitFor({state:'visible'});
  assert.match(await page.getByTestId('project-verification-drawer').innerText(),/E_PARSE|E_SOURCE/);
  await page.getByTestId('project-verification-close').click();
  assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'),'');
  await page.locator('#source-editor').fill('값 := 100.\n값 보여주기.');
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.consoleOutput==='100');
  // A rejected edit must clear visible results and repeated Run must not
  // silently resume the previously valid program retained inside the VM.
  await page.locator('#source-editor').fill('값 := 1. 값 <- "가". 값 보여주기.');
  for(let attempt=0;attempt<2;attempt++) {
    await page.getByTestId('lifecycle-run').click();
    await page.getByTestId('project-verification-drawer').waitFor({state:'visible'});
    assert.match(await page.getByTestId('project-verification-drawer').innerText(),/E_CHECK_TYPE_MISMATCH|E_STATIC_VALIDATION/);
    assert.equal(await page.locator('#canvas-bogae').getAttribute('data-console-output'),'');
    await page.getByTestId('project-verification-close').click();
  }
  await page.locator('#source-editor').fill('값 := 100.\n값 보여주기.');
  await page.getByTestId('lifecycle-run').click();
  await page.waitForFunction(()=>document.querySelector('#canvas-bogae')?.dataset.consoleOutput==='100');
  await page.setViewportSize({width:390,height:844});
  assert.ok(await page.locator('#source-editor').isVisible());
  assert.equal(await page.locator('.runtime-console').count(),0,'No new independent UI panel');
  console.log('WORKBENCH_ALPHA_OUTPUT_PASS actual_server=1 drawlist=1 pause_step=1 console=5 error_repair=1 mobile=1');
} finally {await browser.close();}
