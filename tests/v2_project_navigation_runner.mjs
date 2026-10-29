import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';

const modulePath='solutions/seamgrim_ui_mvp/ui/workbench/project_navigation.js';
const baseline=process.argv.includes('--baseline');
const fixed=process.argv.find(value=>value.startsWith('--fixed='))?.slice(8);
if(fixed&&!/^[a-f0-9]{40}$/.test(fixed))throw new Error('고정 커밋은 전체 SHA로 지정하세요.');
const sourceRef=baseline?'25bc4477d66e6ef1a855c1a9f0ee148c4f3d4aa4':fixed;
const source=sourceRef
  ? execFileSync('git',['show',`${sourceRef}:${modulePath}`],{encoding:'utf8'})
  : readFileSync(new URL(`../${modulePath}`,import.meta.url),'utf8');
const { createProjectNavigation }=await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
const project={id:'two-file-work',title:'두 파일 작품',files:[
  {path:'app/main.ddn'},{path:'lib/helper.ddn'},{path:'lib/두번째.ddn'},{path:'lib/네번째.ddn'},
]};
const original=JSON.stringify(project);
const loaded=new Map(project.files.map(file=>[file.path,'1 보여주기.']));
const state={loadedFileTexts:loaded,fileTexts:new Map(loaded),activeFilePath:'app/main.ddn'};
const classes=new Set();
const tabs={children:[],classList:{add:key=>classes.add(key),remove:key=>classes.delete(key)},
  set innerHTML(_){this.children=[];},appendChild(child){this.children.push(child);}};
const priorDocument=globalThis.document;
globalThis.document={createElement:()=>({dataset:{},listeners:{},
  setAttribute(key,value){this[key]=value;},addEventListener(key,fn){this.listeners[key]=fn;}})};
let selected=null;
const navigation=createProjectNavigation({state,els:{fileTabs:tabs},
  localProjectEntryPath:()=>project.files[0].path,selectFile:path=>{selected=path;}});
const results=[];
function check(name,body){try{body();results.push({case:name,status:'PASS'});}
  catch(error){results.push({case:name,status:'FAIL',reason:error.message});}}
try {
  check('원문이 같으면 수정 표시 없음',()=>assert.equal(navigation.displayedProjectTitle(project),'두 파일 작품'));
  state.fileTexts.set('lib/helper.ddn','2 보여주기.');
  check('보조 파일 수정 표시',()=>assert.equal(navigation.displayedProjectTitle(project),'두 파일 작품 · 수정한 원문'));
  state.fileTexts.set('lib/helper.ddn',loaded.get('lib/helper.ddn'));
  check('보조 파일 원상 복원',()=>assert.equal(navigation.displayedProjectTitle(project),'두 파일 작품'));
  state.fileTexts.set('app/main.ddn','3 보여주기.');
  check('시작 파일 수정 표시 유지',()=>assert.equal(navigation.displayedProjectTitle(project),'두 파일 작품 · 수정한 원문'));
  state.fileTexts.set('app/main.ddn',loaded.get('app/main.ddn'));
  state.loadedFileTexts.delete('lib/네번째.ddn');
  state.fileTexts.set('lib/네번째.ddn','아직 읽지 않은 원문');
  check('읽지 않은 파일은 수정으로 추측하지 않음',()=>assert.equal(navigation.displayedProjectTitle(project),'두 파일 작품'));
  state.activeFilePath='lib/네번째.ddn';navigation.renderFileTabs(project);
  check('네번째 파일도 활성 탭 표시',()=>assert(tabs.children.some(tab=>tab.dataset.filePath===state.activeFilePath)));
  check('탭 세 개 제한 유지',()=>assert.equal(tabs.children.length,3));
  check('활성 탭 누르기',()=>{tabs.children.find(tab=>tab.dataset.filePath===state.activeFilePath).listeners.click();assert.equal(selected,state.activeFilePath);});
  state.activeFilePath='app/main.ddn';navigation.renderFileTabs(project);
  check('앞쪽 파일은 기존 순서 유지',()=>assert.deepEqual(tabs.children.map(tab=>tab.dataset.filePath),project.files.slice(0,3).map(file=>file.path)));
  navigation.renderFileTabs({...project,files:project.files.slice(0,1)});
  check('단일 파일 탭 숨김 유지',()=>assert(classes.has('hidden')));
  check('프로젝트와 읽은 원문을 변경하지 않음',()=>{assert.equal(JSON.stringify(project),original);assert.equal(loaded.get('lib/helper.ddn'),'1 보여주기.');});
} finally {globalThis.document=priorDocument;}
console.log(JSON.stringify({target:sourceRef??'working-module',results},null,2));
if(results.some(result=>result.status==='FAIL'))process.exitCode=1;
