// Focused dispatch/guard test, not Native/browser closure.
import assert from 'node:assert/strict';
import {installAuthoredProjectPanel} from '../solutions/seamgrim_ui_mvp/ui/workbench/authored_project_panel.js';
globalThis.fetch=async()=>({ok:true,headers:{get:name=>name==='X-DDN-Native-Source'?'v1':null}});
globalThis.localStorage={getItem:()=>null,setItem:()=>{}};
for(const kind of ['logical','ordinary','edited','restore_failure']){
 const selectors=['[data-authored-status]','[data-authored-identity]','[data-authored-contents]','input[type=file]',
   ...['create','save','reopen','clone','export','import','register','module','source','composition','runtime-save','runtime-reopen','runtime-export'].map(n=>`[data-authored-${n}]`)];
 const elements=Object.fromEntries(selectors.map(k=>[k,{textContent:'',value:'',disabled:false,files:[],listeners:{},
   addEventListener(event,listener){this.listeners[event]=listener;}}]));
 const root={ownerDocument:{activeElement:null},contains:()=>false,querySelector:k=>elements[k],querySelectorAll:()=>Object.values(elements)};
 const snapshot={entry_path:'main.ddn',files:[{path:'main.ddn',text:'1 보여주기.'}]};
 const artifact={source_profile:'v1-core-v25',source_snapshot:snapshot,authored_identity:{logical_dependency_id:'fixed-project'}};
 const current={selected_project:'fixed-project',source_profile:'v1-core-v25',source_snapshot:structuredClone(snapshot)};
 const calls=[];
 installAuthoredProjectPanel({root,host:{},capture:()=>current,apply:async()=>current,
   bridge:{authoredProjectOperation:async()=>JSON.stringify(artifact)},ownsLogicalSource:()=>kind!=='ordinary',
   reopenProject:async()=>{calls.push('source_only');return {message:'원문만 열었습니다'};},
   saveProject:async()=>{calls.push('top_save');return {message:'저장'};},
   reopenRuntime:async text=>{assert.equal(text,JSON.stringify(artifact));calls.push('logical_restore');
     if(kind==='restore_failure')throw Error('E_NATIVE_SOURCE_LOGICAL_REPLAY_MISMATCH');return 'Native 저장본과 실제 입력을 검증했습니다';}});
 await elements['[data-authored-create]'].onclick();
 if(kind==='edited')current.source_snapshot.files[0].text='2 보여주기.';
 await elements['[data-authored-runtime-reopen]'].listeners.click();
 if(kind==='ordinary'){assert.deepEqual(calls,['source_only']);}
 else if(kind==='edited'){assert.deepEqual(calls,[]);assert.match(elements['[data-authored-status]'].textContent,/원문을 갱신 저장/);}
 else{assert.deepEqual(calls,['logical_restore']);assert.match(elements['[data-authored-status]'].textContent,
   kind==='restore_failure'?/E_NATIVE_SOURCE_LOGICAL_REPLAY_MISMATCH/:/Native 저장본과 실제 입력/);}
 if(kind==='logical'){
   calls.length=0;await elements['[data-authored-runtime-save]'].listeners.click();assert.deepEqual(calls,['top_save']);
 }
}
console.log('PASS Native logical resume dispatch, ordinary source-only preserved, source-change/restore-failure guards, save routing; browser NOT_CLAIMED');
