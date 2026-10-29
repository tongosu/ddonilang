// Focused host UI race test only. Codec/Worker/CLI proof is in the actual
// browser runner; these fake elements do not claim product browser closure.
import assert from 'node:assert/strict';
import {installAuthoredProjectPanel} from '../solutions/seamgrim_ui_mvp/ui/workbench/authored_project_panel.js';
// This non-browser test has no Native provider; make that environment explicit.
globalThis.fetch=async()=>({ok:true,headers:{get:()=>null}});
for(const operation of ['reopen','clone']) {
  const keys=['[data-authored-status]','[data-authored-identity]','[data-authored-contents]','input[type=file]',
    ...['create','save','reopen','clone','export','import','register','module','source','composition'].map(name=>`[data-authored-${name}]`)];
  const elements=Object.fromEntries(keys.map(key=>[key,{textContent:'',value:'',disabled:false,files:[]}]));
  // This harness has no focused browser node. Keep the real Element/Document
  // interface available without pretending that it proves focus behavior.
  const root={ownerDocument:{activeElement:null},contains:()=>false,
    querySelector:key=>elements[key],querySelectorAll:()=>Object.values(elements)};
  const original={source_profile:'v1-core-v25',source_snapshot:{entry_path:'main.ddn',files:[{path:'main.ddn',text:'1 보여주기.'}]},authored_identity:{logical_dependency_id:'original'}};
  let current={selected_project:'before',source_profile:original.source_profile,source_snapshot:original.source_snapshot};
  let stored='', race=false;
  globalThis.localStorage={getItem:()=>stored,setItem:(_,value)=>{stored=value;}};
  const panel=installAuthoredProjectPanel({root,host:{},capture:()=>current,
    bridge:{authoredProjectOperation:async({operation})=>JSON.stringify(operation==='clone'?{...original,authored_identity:{logical_dependency_id:'clone'}}:original)},
    apply:async artifact=>{
      const expected={selected_project:`authored_${artifact.authored_identity.logical_dependency_id}`,source_profile:artifact.source_profile,source_snapshot:artifact.source_snapshot};
      current=structuredClone(expected);
      await Promise.resolve();
      if(race) current.source_snapshot.files[0].text='2 보여주기.';
      return expected;
    }});
  await elements['[data-authored-create]'].onclick();
  await elements['[data-authored-save]'].onclick();
  const originalBytes=panel.getArtifactText();
  race=true;
  await elements[`[data-authored-${operation}]`].onclick();
  assert.match(elements['[data-authored-status]'].textContent,/바뀌었습니다/);
  assert.equal(panel.getArtifactText(),originalBytes,'stale apply must not publish artifact');
  await elements['[data-authored-save]'].onclick();
  assert.match(elements['[data-authored-status]'].textContent,/다른 작품이나 프로필/);
  assert.equal(stored,originalBytes,'stale apply must not bless edited editor as baseline');
}
console.log('AUTHORED_PANEL_APPLY_RACE_PASS: reopen/clone; focused host test, actual browser NOT_CLAIMED');
