import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {displayMessage,installDisplayLocale} from '../solutions/seamgrim_ui_mvp/ui/workbench/display_locale.js';

class Node {
  constructor(dataset={},text='',attributes={}) {this.dataset={...dataset};this.textContent=text;this.attributes={...attributes};this.value='';this.listeners={};this.hidden=false;}
  getAttribute(key){return this.attributes[key]??null;}
  setAttribute(key,value){this.attributes[key]=String(value);}
  removeAttribute(key){delete this.attributes[key];}
  addEventListener(key,fn){this.listeners[key]=fn;}
  closest(){return {open:false};}
}
const caption=new Node({displayKey:'save'},'저장',{title:'프로젝트 저장','aria-label':'프로젝트 저장'});
const completion=new Node({displayKey:'completion',displayTitleKey:'completionHelp'},'추천',
  {title:'추천 열기: Ctrl+Space. 선택: 클릭 또는 Ctrl+Enter. 닫기: Esc.','aria-label':'추천 열기'});
const icon=new Node({displayTitleKey:'step',displayLabelKey:'step'},'↦',{title:'한 마디 앞으로','aria-label':'한 마디 앞으로'});
icon.children=[{sentinel:'unmodified SVG/child'}];icon.disabled=true;icon.dataset.disabledReason='paused_canStep';
const input=new Node({displayPlaceholderKey:'searchPlaceholder'},'',{placeholder:'DDN 이름, 태그'});input.value='한국어 입력 · x+5=9';
const unknown=new Node({displayTitleKey:'futureUnknown'},'◆',{title:'원래 안내 <원문>'});
const select=new Node();const notice=new Node();const events=[];const saved=[];
const doc={documentElement:{dataset:{},lang:'ko'},defaultView:{localStorage:{getItem:()=>null,setItem:(k,v)=>saved.push([k,v])},
 CustomEvent:class {constructor(type,data){this.type=type;this.detail=data.detail;}}},
 querySelector:selector=>selector.includes('display-language-notice')?notice:select,
 querySelectorAll:selector=>({'[data-display-key]':[caption,completion],
 '[data-display-title-key]':[completion,icon,unknown],
 '[data-display-label-key]':[icon],'[data-display-placeholder-key]':[input]})[selector]??[],
 dispatchEvent:event=>events.push(event)};
const controlBefore=JSON.stringify({disabled:icon.disabled,reason:icon.dataset.disabledReason,children:icon.children,input:input.value});
const preference=installDisplayLocale(doc);assert.equal(preference.locale,'ko');
assert.equal(preference.change('ja'),true);
assert.equal(caption.textContent,'保存');assert.equal(icon.textContent,'↦');
assert.equal(icon.attributes.title,'1マディ進める');assert.equal(icon.attributes['aria-label'],'1マディ進める');
assert.ok(completion.attributes.title.includes('Ctrl+Space'));assert.ok(completion.attributes.title.includes('Ctrl+Enter'));
assert.match(input.attributes.placeholder,/DDNの名前/);
assert.equal(JSON.stringify({disabled:icon.disabled,reason:icon.dataset.disabledReason,children:icon.children,input:input.value}),controlBefore);
console.log('PASS metadata/caption translation preserves icon, children, edit value and disabled reason');

assert.equal(unknown.attributes.title,'[ko] 원래 안내 <원문>');
assert.equal(unknown.dataset.displayMetadataFallback,'ko');
assert.equal(preference.change('ko'),true);
assert.equal(caption.textContent,'저장');assert.equal(caption.attributes.title,'프로젝트 저장');
assert.equal(icon.attributes.title,'한 마디 앞으로');assert.equal(input.attributes.placeholder,'DDN 이름, 태그');
assert.equal(completion.attributes.title,'추천 열기: Ctrl+Space. 선택: 클릭 또는 Ctrl+Enter. 닫기: Esc.');
assert.equal(unknown.attributes.title,'원래 안내 <원문>');
const beforeInvalid=JSON.stringify({saved,events:events.map(e=>e.detail),caption:caption.textContent,input:input.value});
assert.equal(preference.change('en'),false);
assert.equal(JSON.stringify({saved,events:events.slice(0,-1).map(e=>e.detail),caption:caption.textContent,input:input.value}),beforeInvalid);
assert.equal(events.at(-1).detail.locale,'ko');assert.equal(notice.hidden,false);
console.log('PASS exact Korean roundtrip, explicit metadata fallback and invalid locale preserves preference');

for(const key of ['run','pause','resume','runChanged','logicalRun','step','stop','back','branch',
 'disabled','commandPending','stepUnavailable','stopUnavailable','backUnavailable','branchUnavailable',
 'logicalFailed','logicalRunning','logicalPaused','logicalReady']) {
 assert.equal(displayMessage(key,'ko').fallback,false,key);assert.equal(displayMessage(key,'ja').fallback,false,key);
}
const html=readFileSync(new URL('../solutions/seamgrim_ui_mvp/ui/workbench/index.html',import.meta.url),'utf8');
assert.match(html,/data-testid="project-save" data-display-key="save"/);
assert.match(html,/data-testid="completion-request" data-display-key="completion" data-display-title-key="completionHelp"/);
assert.ok(html.includes('aria-label="실행">▶</button>'));
assert.ok(html.includes('aria-label="한 마디 앞으로">↦</button>'));
console.log('PASS source/control resources cover normal, disabled and logical execution presentation without changing icons');
