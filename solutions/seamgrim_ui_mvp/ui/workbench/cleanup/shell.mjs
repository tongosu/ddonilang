import {diagnosticWordsKo} from '../diagnostic_words_ko.mjs';
import {WORKBENCH_UI,chooseSideDock,diagnosticLine,temporaryOutputVisible} from './policy.mjs';
const by=id=>document.querySelector(`[data-testid="${id}"]`);
function node(tag,text='',className='') {const n=document.createElement(tag);n.textContent=text;n.className=className;return n;}
function button(text,action,id) {const n=node('button',text);n.type='button';if(id)n.dataset.testid=id;n.addEventListener('click',action);return n;}
export function installWorkbenchCleanup({read,projects,lanes,openProject,newProject,restoreSaved,readSavedLocator,openDdnWork,openLessonOnStartup=false}) {
  const app=by('approved-workbench-root'),work=by('project-workbench'),layout=by('workbench-layout');
  app.dataset.uiCleanup='r1';app.style.setProperty('--overlay-color',WORKBENCH_UI.overlayColor);app.style.setProperty('--side-color',WORKBENCH_UI.sideColor);
  const toolbar=document.querySelector('.workbench-toolbar');
  const more=by('project-more-menu'),menu=more.querySelector('.project-more-actions');
  const fileMenu=by('cleanup-title-menu'),title=by('cleanup-title');
  const fileActions=fileMenu.querySelector('.cleanup-menu');
  by('cleanup-new').addEventListener('click',async()=>{await newProject();showWork();});
  by('cleanup-open-work').addEventListener('click',()=>openDdnWork());
  by('cleanup-browse').addEventListener('click',()=>showBrowse());
  fileActions.addEventListener('click',event=>{if(event.target.closest('button'))fileMenu.open=false;});
  by('source-file-input').addEventListener('change',()=>{fileMenu.open=false;more.open=false;});
  const save=by('project-save'),saveStatus=by('project-save-status');
  menu.addEventListener('click',event=>{if(event.target.closest('button'))more.open=false;});
  const history=by('cleanup-history-group');
  const co=by('co-creation');if(co)by('cleanup-developer-tools').append(co);
  const temporary=by('temporary-editor-run');
  if(temporary){
    by('bogae-stage').before(temporary);temporary.hidden=true;
    temporary.querySelector(':scope>summary').textContent='실행 결과';
    const settings=node('details'),settingsTitle=node('summary','실행 세부 설정');settingsTitle.dataset.testid='cleanup-temp-settings';settings.append(settingsTitle);
    for(const child of [...temporary.children])if(child.tagName==='LABEL'||child.matches('button[data-temp]')||child.matches('p:not([data-temp])'))settings.append(child);
    temporary.append(settings);
  }
  const source=by('source-panel'),editor=by('source-editor'),editorShell=editor.parentElement;
  const gutter=by('cleanup-source-lines');
  const notice=by('cleanup-source-notice'),noticeSummary=by('cleanup-notice-summary');
  const noticeDetails=by('cleanup-notice-details'),noticeList=by('cleanup-notice-list');
  const noticeToggle=noticeDetails.querySelector('summary');
  const isReadOnlySpace3=d=>d?.code==='E_PROJECT_RUNTIME_SPACE3_DURABILITY_UNSUPPORTED'
    ||d?.code==='E_SPACE3_PREVIEW_READONLY';
  const runWithoutSaving=button('저장 없이 실행',()=>{
    const action=by('editor-run'),current=read();
    if(!notice.hidden&&!runWithoutSaving.hidden&&!current.temporaryRunning
      &&current.diagnostics?.some(isReadOnlySpace3)&&!action?.disabled)action?.click();
  },'cleanup-space3-run-without-save');
  runWithoutSaving.hidden=true;
  noticeSummary.after(runWithoutSaving);
  let noticeContext='';
  const clearNotice=()=>{notice.hidden=true;noticeDetails.open=false;markedLines=new Set();syncLines();};
  editor.addEventListener('input',clearNotice);
  const browse=by('cleanup-browser'),heading=by('cleanup-browser-heading');
  const tabs=by('cleanup-browser-tabs'),cards=by('cleanup-cards'),search=by('cleanup-search'),hint=by('cleanup-startup-notice');
  by('cleanup-return').addEventListener('click',showWork);
  by('cleanup-open-file').addEventListener('click',()=>{showWork();by('source-file-open').click();});
  by('cleanup-browse-work').addEventListener('click',()=>{showWork();openDdnWork();});
  const myWorkPanel=by('my-work-panel');
  const names=['처음','또니랑 배우기','교과','대표 예시','내 작업','모두의 작품'];let tab=0,lastList='';
  function paintBrowse(force=false) {
    const all=projects(),key=JSON.stringify([tab,search.value,all.map(p=>[p.id,p.title,p.summary])]);if(!force&&key===lastList)return;lastList=key;cards.replaceChildren();
    const ids=lanes();const rows=all.filter(p=>tab===0 || (tab===1?ids.learning.includes(p.id):tab===2?Boolean(p.course?.subject||p.portableMetadata?.subject):tab===3?ids.examples.includes(p.id):false));
    if(tab===4){const my=myWorkPanel;my.classList.remove('hidden');cards.append(my);by('global-my-work').click();return;}
    for(const p of rows.filter(p=>`${p.title} ${p.summary??''} ${(p.tags??[]).join(' ')}`.toLocaleLowerCase().includes(search.value.toLocaleLowerCase()))) {
      const b=button('',async()=>{await openProject(p.id);showWork();});b.className='cleanup-card';b.append(node('strong',p.title??p.id),node('span',p.summary??''));cards.append(b);
    }
    if(!cards.children.length)cards.append(node('p','이 분류에 공급된 작품이 없습니다. 다른 분류나 DDN 원문 열기를 이용하세요.'));
  }
  [...tabs.querySelectorAll('button')].forEach((b,index)=>b.addEventListener('click',()=>{tab=index;tabs.querySelectorAll('button').forEach((n,i)=>n.setAttribute('aria-pressed',String(i===tab)));paintBrowse(true);}));
  search.addEventListener('input',()=>paintBrowse(true));
  function showBrowse(){browse.hidden=false;work.hidden=true;app.dataset.browsing='true';paintBrowse(true);heading.tabIndex=-1;heading.focus();}
  function showWork(){fileMenu.open=false;more.open=false;browse.hidden=true;work.hidden=false;app.dataset.browsing='false';refresh();}
  cards.addEventListener('click',e=>{if(e.target.closest('[data-testid="my-work-item"]'))showWork();});
  let markedLines=new Set(),numberedSource=null,numberedMarks='';
  const syncLines=()=>{const markKey=[...markedLines].join(',');if(numberedSource===editor.value&&numberedMarks===markKey){gutter.scrollTop=editor.scrollTop;return;}numberedSource=editor.value;numberedMarks=markKey;const count=editor.value.split('\n').length;gutter.textContent=Array.from({length:count},(_,i)=>`${markedLines.has(i+1)?'!':' '} ${i+1}`).join('\n');gutter.scrollTop=editor.scrollTop;};
  editor.addEventListener('scroll',()=>{gutter.scrollTop=editor.scrollTop;});editor.addEventListener('input',()=>{editor.dataset.completionNotice='';syncLines();});
  let lastNotice='',savedGeneration=0,dock='right',disposed=false,lastPresentedRun='',lastPresentedTemporary='';
  function refresh(){
    if(disposed)return;const c=read();title.textContent=c.title||'새 작품';title.title=c.path||'';syncLines();
    const diagnostics=[...(c.readiness?.diagnostics??[]),...(c.diagnostics??[])];
    const errors=diagnostics.filter(d=>d.severity==='error'&&!isReadOnlySpace3(d));
    const space3Notice=c.diagnostics?.find(isReadOnlySpace3);
    runWithoutSaving.hidden=!space3Notice||Boolean(errors.length)||Boolean(c.temporaryRunning)
      ||!by('editor-run')||by('editor-run').disabled;
    markedLines=new Set(errors.filter(d=>!d.path||d.path===c.path).map(d=>diagnosticLine(editor.value,d.span??d.source_span)).filter(n=>n!==null));syncLines();
    if(temporary){const has=temporaryOutputVisible(temporary.dataset.status,c.temporaryRunning,c.committedSourceCurrent);temporary.hidden=!has;if(has)temporary.open=true;app.dataset.temporaryShown=String(Boolean(has));by('bogae-stage').classList.toggle('cleanup-temporary-covered',Boolean(has));}
    if(temporary&&!temporary.hidden){by('run-status').textContent=`고정 편집판 ${{running:'실행 중',completed:'완료',failed:'실패',cancelled:'취소'}[temporary.dataset.status]??''}`;}
    const text=JSON.stringify([c.path,editor.value,diagnostics,editor.dataset.completionNotice||'']);
    if(lastNotice!==text){
      const context=JSON.stringify([c.path,editor.value]);
      if(context!==noticeContext){noticeDetails.open=false;noticeContext=context;}
      notice.hidden=!diagnostics.length&&!editor.dataset.completionNotice;
      notice.dataset.state=errors.length?'error':'ready';
      const first=errors[0]??space3Notice??diagnostics[0];
      noticeSummary.textContent=first===space3Notice&&space3Notice
        ?'3D 보기 안내 · 이 입체 보기는 읽기 전용입니다. 저장 없이 실행해 볼 수 있습니다.'
        :first?`${errors.length?'원문 오류 '+errors.length+'개':'원문 안내 '+diagnostics.length+'개'} · ${diagnosticWordsKo(first,first?.presentationSource)?.what??first.message??first.code??''}`.replace(/\s+/g,' '):editor.dataset.completionNotice||'';
      noticeSummary.title=noticeSummary.textContent;
      noticeDetails.hidden=!diagnostics.length;
      noticeToggle.textContent=`자세히 (${diagnostics.length})`;
      noticeList.replaceChildren();
      for(const d of diagnostics){
        const item=node('div','','cleanup-notice-item');
        const words=diagnosticWordsKo(d,d.presentationSource);
        if(words){item.append(node('p',words.category?`${words.category} · ${words.what}`:words.what));if(words.why)item.append(node('p',`이유 · ${words.why}`));for(const [key,value] of words.facts)item.append(node('p',`${key} · ${value}`));if(words.how)item.append(node('p',`고침 제안 · ${words.how}`));if(words.fragment)item.append(node('pre',`이 자리\n${words.fragment}\n^`));item.append(node('small',`진단 번호 · ${d.code}`));}else item.append(node('p',`${d.code??''}: ${d.message??''}`));
        const path=d.path??d.source_path??c.path,span=d.span??d.source_span;
        if(path)item.append(node('small',path));
        const line=path===c.path?diagnosticLine(editor.value,span):null;
        if(line!==null){
          const sourceAtNotice=editor.value,pathAtNotice=c.path;
          item.append(button(`${line}줄로 이동`,()=>{
            if(editor.value!==sourceAtNotice||read().path!==pathAtNotice)return;
            // Select the producer's exact UTF-8 range, never infer a call from its line.
            const bytes=new TextEncoder().encode(sourceAtNotice);
            let start,end;
            try {
              const decoder=new TextDecoder('utf-8',{fatal:true});
              start=decoder.decode(bytes.slice(0,span.start)).length;
              end=decoder.decode(bytes.slice(0,span.end)).length;
            } catch { return; }
            editor.focus();editor.setSelectionRange(start,end);
            editor.scrollTop=Math.max(0,(line-3)*parseFloat(getComputedStyle(editor).lineHeight||'22.4'));syncLines();
          }));
        }
        noticeList.append(item);
      }
      lastNotice=text;
    }
    history.style.visibility=c.hasFrames?'visible':'hidden';history.setAttribute('aria-hidden',String(!c.hasFrames));
    // Read only the existing completed save, and persist a locator, never source or permission material.
    if(c.save?.phase==='saved' && c.save.completedGeneration>savedGeneration){
      savedGeneration=c.save.completedGeneration;const locator=readSavedLocator();if(locator)try{localStorage.setItem('seamgrim.ui.last-saved.r1',JSON.stringify(locator));}catch{}
    }
    const temporaryRun=temporary?.dataset.presentationRun;
    const newTemporary=temporaryRun&&temporaryRun!==lastPresentedTemporary;
    if(newTemporary){
      lastPresentedTemporary=temporaryRun;
      if(browse.hidden&&!temporary.hidden&&matchMedia('(max-width:720px)').matches)by('mode-bogae-main').click();
    }
    if(c.hasFrames&&c.runId&&lastPresentedRun!==c.runId){lastPresentedRun=c.runId;if(matchMedia('(max-width:720px)').matches)by('mode-bogae-main').click();}
    if(!browse.hidden)paintBrowse();layout.dataset.mobileSource=String(read().mode==='ddn-main');
    const stage=by('bogae-stage'),rect=stage.getBoundingClientRect(),wrap=by('bogae-canvas-wrap');
    const ratio=Number(getComputedStyle(wrap).getPropertyValue('--primary-aspect'))||4/3;
    const next=chooseSideDock(rect.width,rect.height,ratio,dock);if(next!==dock)dock=next;
    stage.dataset.cleanupDock=dock;
  }
  // Closing the multilang drawer returns focus to its opener, or to the More menu that holds it when that menu is closed.
  const multilangDialog=by('multilang-dialog'),multilangOpener=by('multilang-open');
  const returnMultilangFocus=()=>{const active=document.activeElement;if(active&&active!==document.body&&!multilangDialog.contains(active))return;
    const openerShown=multilangOpener&&!multilangOpener.closest('details:not([open])')&&multilangOpener.getClientRects().length>0;
    (openerShown?multilangOpener:more.querySelector(':scope>summary'))?.focus();};
  multilangDialog?.addEventListener('close',returnMultilangFocus);
  const temporaryObserver=new MutationObserver(()=>refresh());if(temporary)temporaryObserver.observe(temporary,{attributes:true,attributeFilter:['data-status','data-presentation-run']});
  document.addEventListener('ddn-completion-notice',refresh);
  const savedObserver=new MutationObserver(()=>refresh());savedObserver.observe(saveStatus,{childList:true,subtree:true,characterData:true});
  const closeMenus=event=>{if(event.key==='Escape'){const target=fileMenu.open?fileMenu:more.open?more:null;if(target){target.open=false;target.querySelector('summary').focus();}}};
  document.addEventListener('keydown',closeMenus);
  const observer=new ResizeObserver(refresh);observer.observe(layout);
  const diagnosticsObserver=new MutationObserver(()=>refresh());diagnosticsObserver.observe(by('diagnostics')??document.querySelector('[data-testid="project-diagnostics"]')??notice,{childList:true,subtree:true});
  if(openLessonOnStartup)showWork();else showBrowse();
  // Startup is read-only: restore only a saved locator through the existing owner.
  let startup=null;try{startup=JSON.parse(localStorage.getItem('seamgrim.ui.last-saved.r1')??'null');}catch{}
  if(!openLessonOnStartup&&startup)Promise.resolve(restoreSaved(startup)).then(ok=>{if(ok)showWork();else hint.textContent='마지막 저장 작품을 열지 못했습니다. 내 작업 또는 DDN 원문 열기에서 복구하세요. 자동 실행하지 않았습니다.';}).catch(()=>{hint.textContent='마지막 저장 작품을 확인하지 못했습니다. 내 작업 또는 DDN 원문 열기를 이용하세요.';});
  refresh();
  return {refresh,showWork,showBrowse,dispose(){disposed=true;document.removeEventListener('keydown',closeMenus);document.removeEventListener('ddn-completion-notice',refresh);observer.disconnect();savedObserver.disconnect();multilangDialog?.removeEventListener('close',returnMultilangFocus);diagnosticsObserver.disconnect();temporaryObserver.disconnect();editor.removeEventListener('input',syncLines);editor.removeEventListener('input',clearNotice);}};
}
