// Run against the extracted candidate, never the shared target/debug directory.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
const root=process.env.DDN_ARCHIVE_ROOT;
const evidence=process.env.DDN_LIFECYCLE_EVIDENCE;
if(!root||!evidence)throw Error('DDN_ARCHIVE_ROOT and DDN_LIFECYCLE_EVIDENCE required');
fs.mkdirSync(evidence,{recursive:false});
const delay=ms=>new Promise(r=>setTimeout(r,ms));
const children=[],rows=[];
async function start(name,port=0) {
 const exe=path.join(root,'bin',name+'.exe');
 const child=spawn(exe,['workbench','--no-open','--port',String(port)],{cwd:root,windowsHide:true,stdio:['pipe','pipe','pipe']});
 children.push(child);let stdout='',stderr='';
 child.stdout.on('data',b=>stdout+=b.toString());child.stderr.on('data',b=>stderr+=b.toString());
 const closed=new Promise((resolve,reject)=>{child.once('error',reject);child.once('close',(code,signal)=>resolve({code,signal}));});
 const until=Date.now()+15000;
 while(!stdout.includes('\n')&&Date.now()<until&&child.exitCode===null)await delay(25);
 assert(stdout.includes('\n'),stderr||'startup record missing');
 const record=JSON.parse(stdout.split('\n')[0]);assert(record.ok);assert.notEqual(record.port,8790);
 assert.equal((await fetch(record.startup_url+'?token='+record.automation_token)).status,200);
 return {name,exe,child,record,closed,logs:()=>({stdout,stderr})};
}
async function enter(host, line='\n') {
 host.child.stdin.write(line);
 const exit=await Promise.race([host.closed,delay(5000).then(()=>{throw Error('Enter normal exit timed out');})]);
 assert.equal(exit.code,0);let stopped=false;
 try {await fetch(host.record.startup_url,{signal:AbortSignal.timeout(2000)});} catch {stopped=true;}
 assert(stopped,'listener remained after exit');
 assert(host.logs().stderr.includes('Enter'));
 return {name:host.name,pid:host.child.pid,port:host.record.port,exit,listener_closed:true,...host.logs()};
}
try {
 for(const name of ['ddonilang','ddonirang','또니랑']) {
  const first=await start(name);first.child.stdin.write('not-a-command\n');await delay(100);assert.equal(first.child.exitCode,null);
  rows.push({case:'nonempty_then_enter',...await enter(first)});
  const next=await start(name,first.record.port);rows.push({case:'same_port_restart_crlf',...await enter(next,'\r\n')});
  const whitespace=await start(name);rows.push({case:'whitespace_lf_enter',...await enter(whitespace,' \t\n')});
  for(const partial of ['', 'not-a-command', ' ', '\t', '\r', 'not-a-command\n \t']) {
   const eof=await start(name);eof.child.stdin.end(partial);await delay(250);assert.equal(eof.child.exitCode,null);
   assert.equal((await fetch(eof.record.startup_url+'?token='+eof.record.automation_token)).status,200);
   // EOF permanently closes the stdin channel. Only this exact test child is
   // cleaned up; this is not evidence of a product normal-stop operation.
   eof.child.kill();const cleaned=await eof.closed;
   rows.push({case:'eof_preserves_host',stdin_hex:Buffer.from(partial).toString('hex'),name,pid:eof.child.pid,port:eof.record.port,alive_before_cleanup:true,cleanup:'OWN_CHILD_ONLY_NOT_NORMAL_EXIT_EVIDENCE',exit:cleaned,...eof.logs()});
  }
 }
 const sha=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
 const result={result:'PASS',scope:'EXTRACTED_NATIVE_CLI3_ENTER_LIFECYCLE',root,rows,native_sha256:Object.fromEntries(['ddonilang','ddonirang','또니랑'].map(n=>[n,sha(path.join(root,'bin',n+'.exe'))])),auto_browser:'NOT_RUN',human:'NOT_RUN',foreign_processes_touched:0};
 fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(result,null,2)+'\n',{flag:'wx'});
 console.log(JSON.stringify({result:result.result,evidence,cases:rows.length}));
} finally {for(const child of children)if(child.exitCode===null&&child.signalCode===null)child.kill();}
