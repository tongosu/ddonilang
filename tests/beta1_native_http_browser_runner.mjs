// Exact extracted Native host: fresh browser contexts must load every module.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const root=path.resolve(process.argv[2]);
assert.ok(process.env.DDN_PLAYWRIGHT_MODULE);
const output=fs.mkdtempSync(path.join(os.tmpdir(),'ddn-native-http-browser-'));
const sha=b=>createHash('sha256').update(b).digest('hex');
const {chromium}=await import(pathToFileURL(process.env.DDN_PLAYWRIGHT_MODULE).href);
const binary=path.join(root,'bin/ddonilang.exe');
const expectedCss=JSON.parse(fs.readFileSync(path.join(root,'bin/workbench/asset_manifest.detjson'),'utf8'))
  .files.find(row=>row.path==='ui/workbench/workbench.css');
assert.ok(expectedCss);
const child=spawn(binary,['workbench','--no-open','--port','0'],{cwd:root,stdio:['ignore','pipe','pipe']});
let browser,startup,failure;
const rows=[];
try{
 startup=await new Promise((resolve,reject)=>{
  let out='';const timer=setTimeout(()=>reject(Error('Native startup timeout')),30000);
  child.once('exit',code=>reject(Error('Native early exit '+code)));
  child.stdout.on('data',b=>{out+=b;if(out.includes('\n')){clearTimeout(timer);resolve(JSON.parse(out.split('\n')[0]));}});
 });
 const url=new URL(startup.startup_url);url.searchParams.set('token',startup.automation_token);
 browser=await chromium.launch({headless:true,executablePath:'C:/Program Files/Google/Chrome/Application/chrome.exe'});
 for(let index=0;index<8;index++){
  const context=await browser.newContext({viewport:{width:1366,height:768}}),page=await context.newPage();
  const row={index,errors:[],css:[]},pending=[];rows.push(row);
  page.on('requestfailed',r=>row.errors.push({url:r.url(),failure:r.failure()}));
  page.on('pageerror',e=>row.errors.push({page_error:String(e)}));
  page.on('console',m=>{if(m.type()==='error')row.errors.push({console_error:m.text()});});
  page.on('response',r=>{
   if(r.status()>=400)row.errors.push({url:r.url(),status:r.status()});
   if(r.url().endsWith('.css'))pending.push((async()=>{const bytes=await r.body();row.css.push({url:r.url(),status:r.status(),bytes:bytes.length,sha256:sha(bytes)});})());
  });
  try{
   await page.goto(url.href);
   await page.waitForFunction(()=>window.__SEAMGRIM_APPROVED_WORKBENCH__?.getSnapshot().readiness.initialization==='ready',null,{timeout:10000});
   await Promise.all(pending);
   row.layout=await page.evaluate(()=>({readiness:window.__SEAMGRIM_APPROVED_WORKBENCH__.getSnapshot().readiness,
    links:Array.from(document.querySelectorAll('link[rel="stylesheet"]'),l=>({loaded:Boolean(l.sheet)})),
    rules:Array.from(document.styleSheets,s=>s.cssRules.length),font:getComputedStyle(document.body).fontFamily}));
   assert.ok(row.layout.links.length&&row.layout.links.every(l=>l.loaded),'stylesheet must be applied');
   assert.ok(row.css.some(css=>css.status===200&&css.bytes===expectedCss.bytes&&css.sha256===expectedCss.sha256),'exact packaged CSS bytes required');
   assert.ok(row.layout.rules.every(n=>n>0));assert.deepEqual(row.errors,[]);
   if(index===0||index===7)await page.screenshot({path:path.join(output,'initial-'+index+'.png'),fullPage:true});
  }catch(error){await page.screenshot({path:path.join(output,'failure-'+index+'.png'),fullPage:true});throw error;}
  finally{await context.close();}
 }
}catch(error){failure=String(error.stack??error);throw error;}
finally{
 await browser?.close();if(!child.killed)child.kill();
 const report={result:failure?'FAIL':'PASS_NATIVE_HTTP_FRESH_CONTEXTS',scope:'NATIVE_HTTP_FIRST_LOAD_ONLY_NOT_DURABILITY_OR_HUMAN',root,
  driver_sha256:sha(fs.readFileSync(new URL(import.meta.url))),binary_sha256:sha(fs.readFileSync(binary)),
  release_manifest_sha256:sha(fs.readFileSync(path.join(root,'RELEASE_MANIFEST.json'))),
  asset_manifest_identity:startup?.asset_manifest_identity,rows,failure};
 fs.writeFileSync(path.join(output,'REPORT.json'),JSON.stringify(report,null,2));
 console.log(JSON.stringify({output,result:report.result,contexts:rows.length,error_contexts:rows.filter(r=>r.errors.length).length}));
}
