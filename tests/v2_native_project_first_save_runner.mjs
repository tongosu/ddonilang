import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
const path='solutions/seamgrim_ui_mvp/ui/workbench/project_registry.js';
const fixed=process.argv.find(v=>v.startsWith('--fixed='))?.slice(8);
if(fixed&&!/^[a-f0-9]{40}$/.test(fixed))throw Error('전체 고정 커밋을 지정하세요.');
const raw=fixed?execFileSync('git',['show',`${fixed}:${path}`],{encoding:'utf8'}):readFileSync(new URL(`../${path}`,import.meta.url),'utf8');
const {createWorkbenchProjectRegistry}=await import(`data:text/javascript;base64,${Buffer.from(raw).toString('base64')}`);
async function create(template,host){
  const state={},registry={nativeProjects:[]};let writes=0,probes=0;
  const api=createWorkbenchProjectRegistry({state,workbenchProjectRegistryState:registry,
    nativeSourceAvailable:host===undefined?undefined:async()=>{probes++;if(host==='error')throw Error('host unavailable');return host;},
    SOURCE_ANALYSIS_PROFILE_IDENTITY:'v1-core-v25',
    localProjectEntryPath:p=>p.portableSnapshot.local_package_payload?.entry_path??p.files[0].path,
    writeNativeProjectCatalog:()=>writes++,selectProject:async()=>{},rememberMyWorkProject:()=>{},
    pushDiagnostic:()=>{},renderAll:()=>{}});
  let error=null;try{await api.createNativeProject('두 파일 작품',template);}catch(e){error=e.message;}
  return {project:registry.nativeProjects[0],writes,probes,error};
}
const rows=[];
for(const [template,host,expected] of [['physics',true,true],['empty',true,true],['single',true,false],['physics',false,false],['physics',undefined,false]]){
  const actual=await create(template,host);
  assert.equal(Boolean(actual.project.nativeSaveOnFirstUse),expected);
  assert.equal(actual.writes,1);
  if(expected)assert.equal(actual.project.portableSnapshot.local_package_payload.root_path,'뿌리.ddn');
  rows.push({template,host:host??'not-supplied',native_first_save:expected,status:'PASS'});
}
const failed=await create('physics','error');assert.equal(failed.error,'host unavailable');assert.equal(failed.project,undefined);assert.equal(failed.writes,0);
rows.push({case:'호스트 조회 실패 시 프로젝트 부분 게시 없음',status:'PASS'});
console.log(JSON.stringify({target:fixed??'working-module',results:rows},null,2));
