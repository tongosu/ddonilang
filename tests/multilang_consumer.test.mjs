import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createMultilangController} from '../solutions/seamgrim_ui_mvp/ui/workbench/multilang/controller.js';
import {diagnosticPresentation} from '../solutions/seamgrim_ui_mvp/ui/workbench/multilang/messages.js';
import {textareaSelectionForUtf8Span} from '../solutions/seamgrim_ui_mvp/ui/workbench/multilang/source_selection.js';

// 콜백 double은 소비자 경계 검사에만 사용한다. 일본어 제품 실행 근거가 아니다.
const doc = (sourceText='합 := 2 + 3.') => ({sourceText,path:'학습.ddn',sourceProfileIdentity:'profile:test'});
const accepted = r => ({schema:'ddn.source_analysis_snapshot.candidate.v1',
  source_revision_identity:'producer:opaque',analysis_generation:r.analysis_generation,
  source_profile_identity:r.source_profile_identity,outcome:'ACCEPTED',diagnostics:[]});
const deferred = () => { let resolve; const promise = new Promise(r => {resolve=r;}); return {promise,resolve}; };

test('원문은 그대로 공통 API로 전달하고 ACCEPTED만 실행한다', async () => {
  const calls=[];
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>{calls.push(r);return accepted(r);},
    runSummaryWithSourceProfile:async r=>{calls.push(r);return {typed_failure:'원본그대로'};}}});
  c.setDocument('ko',doc()); assert.equal((await c.run('ko')).status,'analysis_required');
  await c.analyze('ko'); await c.run('ko');
  assert.equal(calls[0].source_text,doc().sourceText); assert.equal(calls[1].sourceText,doc().sourceText);
  assert.deepEqual(c.snapshot().slots.ko.result,{typed_failure:'원본그대로'});
  assert.equal(c.snapshot().comparison,'supplier_required');
});
test('미공급 일본어 profile은 파싱·추측·fallback 없이 대기한다',async()=>{
  let calls=0; const c=createMultilangController({runtimeBridge:{sourceAnalysis:()=>{calls++;}}});
  c.setDocument('ja',{...doc('#!日本語\n'),sourceProfileIdentity:null});
  assert.equal((await c.analyze('ja')).status,'supply_missing'); assert.equal(calls,0);
});
test('거부된 원문은 실행하지 않는다',async()=>{
  let runs=0; const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>({...accepted(r),outcome:'REJECTED'}),
    runSummaryWithSourceProfile:()=>{runs++;}}});
  c.setDocument('ko',doc()); await c.analyze('ko'); assert.equal((await c.run('ko')).status,'analysis_required'); assert.equal(runs,0);
});
test('비동기 검사 중 수정과 A→B→A는 옛 결과를 폐기한다',async()=>{
  const wait=deferred(); let request;
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:r=>{request=r;return wait.promise;}}});
  c.setDocument('ko',doc('A')); const pending=c.analyze('ko');
  c.setDocument('ko',doc('B')); c.setDocument('ko',doc('A')); wait.resolve(accepted(request));
  assert.equal((await pending).status,'stale'); assert.equal(c.snapshot().slots.ko.analysis,null);
});
test('잘못된 generation/profile 응답과 재검사 실패는 이전 ACCEPTED를 재사용하지 않는다',async()=>{
  let bad=false; const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>
    bad?{...accepted(r),analysis_generation:999}:accepted(r)}});
  c.setDocument('ko',doc()); await c.analyze('ko'); bad=true;
  assert.equal((await c.analyze('ko')).status,'failed');
  assert.equal((await c.run('ko')).status,'analysis_required');
});
test('수정 중인 실행 결과를 현재 원문에 붙이지 않는다',async()=>{
  const wait=deferred(); const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>accepted(r),
    runSummaryWithSourceProfile:()=>wait.promise}});
  c.setDocument('ko',doc()); await c.analyze('ko'); const pending=c.run('ko');
  c.setDocument('ko',doc('다른 원문')); wait.resolve({output:'옛 결과'});
  assert.equal((await pending).status,'stale'); assert.equal(c.snapshot().slots.ko.result,null);
});
test('표시 전환은 원문·분석·저장·실행을 호출하지 않는다',async()=>{
  let calls=0; const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>{calls++;return accepted(r);}}});
  c.setDocument('ko',doc()); await c.analyze('ko'); const before=c.snapshot().slots;
  c.setLocale('ja'); c.setLocale('ko'); assert.deepEqual(c.snapshot().slots,before); assert.equal(calls,1);
});
test('저장본은 공통 open 결과를 재검사하고 자동 실행하지 않는다',async()=>{
  let checks=0,runs=0; const receipt={native:'opaque'};
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>{checks++;return accepted(r);},
    runSummaryWithSourceProfile:()=>{runs++;}},persistence:{saveDocument:async()=>receipt,openDocument:async()=>doc('저장본')}});
  c.setDocument('ko',doc()); await c.analyze('ko'); await c.save('ko'); await c.reopen('ko');
  assert.equal(c.snapshot().slots.ko.document.sourceText,'저장본'); assert.equal(checks,2); assert.equal(runs,0);
});
test('저장 대기 중 수정해도 이중 저장을 막고 옛 receipt는 이전 원문 저장본으로 보존한다',async()=>{
  const wait=deferred(); let saves=0;
  const c=createMultilangController({persistence:{saveDocument:()=>{saves++;return wait.promise;}}});
  c.setDocument('ko',doc()); const pending=c.save('ko'); c.setDocument('ko',doc('수정'));
  assert.equal((await c.save('ko')).status,'busy'); wait.resolve({saved:'이전원문'});
  assert.equal((await pending).status,'saved_previous');
  const state=c.snapshot().slots.ko;
  assert.deepEqual(state.receipt,{saved:'이전원문'}); assert.equal(state.savedContentMatches,false);
  assert.equal(state.savedReceipt.document.sourceText,doc().sourceText);
  assert.equal(state.document.sourceText,'수정'); assert.equal(state.analysis,null); assert.equal(saves,1);
});
test('원진단 severity·Unicode byte span·message는 번역과 분리한다',()=>{
  const original={code:'E_PARSE',severity:'error',message:'合計の原文',path:'日本語.ddn',span:{encoding:'utf-8-byte',start:13,end:14}};
  const shown=diagnosticPresentation(original,'ja'); assert.equal(shown.severity,'error');
  assert.equal(shown.original,original.message); assert.deepEqual(shown.span,original.span);
  const unknown=diagnosticPresentation({...original,code:'UNKNOWN'},'ja'); assert.match(unknown.guidance,/\[ko\]/);
  assert.equal(unknown.translated,false);
});
test('공급 없는 저장·실행은 지원완료로 표시하지 않는다',async()=>{
  const c=createMultilangController(); c.setDocument('ko',doc());
  assert.equal((await c.save('ko')).status,'supply_missing'); assert.equal((await c.reopen('ko')).status,'save_required');
  assert.equal((await c.analyze('ko')).status,'supply_missing');
});
test('외부 snapshot 수정과 다른 말씨 검사가 내부 상태를 덮지 않는다',async()=>{
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>accepted(r)}});
  c.setDocument('ko',doc()); c.setDocument('ja',doc('別ソース')); await c.analyze('ko');
  const leaked=c.snapshot(); leaked.slots.ko.document.sourceText='오염'; await c.analyze('ja');
  assert.equal(c.snapshot().slots.ko.document.sourceText,doc().sourceText);
  assert.equal(c.snapshot().slots.ja.document.sourceText,'別ソース');
});

test('UTF-8 위치를 한글·가나·emoji의 UTF-16 선택으로 표시한다',()=>{
  const source='あ😀한)';
  assert.deepEqual(textareaSelectionForUtf8Span(source,{encoding:'utf-8-byte',start:10,end:11}),{start:4,end:5});
  assert.deepEqual(textareaSelectionForUtf8Span(source,{encoding:'utf-8-byte',start:3,end:7}),{start:1,end:3});
  assert.equal(source,'あ😀한)');
});
test('CRLF는 원문을 유지하고 textarea 표시 위치에만 반영한다',()=>{
  const source='あ\r\n😀X';
  assert.deepEqual(textareaSelectionForUtf8Span(source,{encoding:'utf-8-byte',start:9,end:10}),{start:4,end:5});
  assert.equal(textareaSelectionForUtf8Span(source,{encoding:'utf-8-byte',start:4,end:5}),null);
  assert.equal(source,'あ\r\n😀X');
});
test('문자 중간·역전·범위 밖·다른 encoding은 위치를 추측하지 않는다',()=>{
  for(const span of [{start:1,end:3},{start:3,end:2},{start:-1,end:0},{start:0,end:99},{start:NaN,end:2}]) {
    assert.equal(textareaSelectionForUtf8Span('あ😀',{encoding:'utf-8-byte',...span}),null);
  }
  assert.equal(textareaSelectionForUtf8Span('あ',{encoding:'utf-16',start:0,end:1}),null);
  assert.deepEqual(textareaSelectionForUtf8Span('あ',{encoding:'utf-8-byte',start:3,end:3}),{start:1,end:1});
});

test('같은 원문의 공급변경은 재열기 위치만 보존하고 분석과 실행을 무효화한다',async()=>{
  let checks=0,runs=0,saves=0;
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>{checks++;return accepted(r);},
    runSummaryWithSourceProfile:async()=>{runs++;return {output:5};}},
    persistence:{saveDocument:async()=>{saves++;return {opaque:'실제공급자가줄위치'};}}});
  c.setDocument('ko',doc()); await c.analyze('ko'); await c.run('ko'); await c.save('ko');
  const before=c.snapshot().slots.ko;
  c.setDocument('ko',doc(),{recheckOnly:true}); const after=c.snapshot().slots.ko;
  assert.equal(after.document.sourceText,before.document.sourceText);
  assert.deepEqual(after.receipt,before.receipt); assert.equal(after.analysis,null); assert.equal(after.result,null);
  assert.equal(after.status,'recheck_required'); assert.equal(after.generation,before.generation+1);
  assert.deepEqual([checks,runs,saves],[1,1,1]);
  assert.equal((await c.run('ko')).status,'analysis_required');
});
test('재검사 옵션으로 다른 원문·경로·프로필의 저장 위치를 가져오지 못한다',()=>{
  const c=createMultilangController(); c.setDocument('ko',doc());
  for(const changed of [{...doc(),sourceText:'새원문'},{...doc(),path:'다른.ddn'},
    {...doc(),sourceProfileIdentity:'다른판본'}]) {
    assert.throws(()=>c.setDocument('ko',changed,{recheckOnly:true}),/같은 원문/);
  }
  assert.equal(c.snapshot().slots.ko.generation,1);
  assert.throws(()=>c.setDocument('ko',doc(),{recheckOnly:'true'}),/참\/거짓/);
});
test('같은 원문의 외부공급 변경 중 도착한 이전 분석은 폐기한다',async()=>{
  const wait=deferred(); let request;
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:r=>{request=r;return wait.promise;}}});
  c.setDocument('ko',doc()); const pending=c.analyze('ko');
  c.setDocument('ko',doc(),{recheckOnly:true}); wait.resolve(accepted(request));
  assert.equal((await pending).status,'stale'); assert.equal(c.snapshot().slots.ko.analysis,null);
  assert.equal(c.snapshot().slots.ko.status,'recheck_required');
});

test('저장 후 편집기를 비워도 저장본을 다시 열고 재검사할 수 있다',async()=>{
  let runs=0,opened=0;
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>accepted(r),runSummaryWithSourceProfile:()=>{runs++;}},
    persistence:{saveDocument:async()=>({opaque:'첫저장'}),openDocument:async receipt=>{
      assert.deepEqual(receipt,{opaque:'첫저장'});opened++;return doc();}}});
  c.setDocument('ko',doc()); await c.save('ko'); c.setDocument('ko',doc(''));
  assert.equal(c.snapshot().slots.ko.savedContentMatches,false);
  assert.equal((await c.reopen('ko')).status,'accepted');
  assert.equal(c.snapshot().slots.ko.document.sourceText,doc().sourceText);
  assert.equal(c.snapshot().slots.ko.savedContentMatches,true); assert.equal(opened,1); assert.equal(runs,0);
});
test('저장 완료 후 편집하고 재검사해도 이전 저장세대와 복귀 위치는 보존한다',async()=>{
  const c=createMultilangController({runtimeBridge:{sourceAnalysis:async r=>accepted(r)},
    persistence:{saveDocument:async()=>({id:'첫저장'})}});
  c.setDocument('ko',doc()); await c.save('ko'); const saved=c.snapshot().slots.ko.savedReceipt;
  c.setDocument('ko',doc('새 원문')); await c.analyze('ko');
  assert.deepEqual(c.snapshot().slots.ko.savedReceipt,saved);
  assert.equal(c.snapshot().slots.ko.savedContentMatches,false);
});
test('새 작품 교체는 같은 경로라도 이전 작품 저장 위치를 이어받지 않는다',async()=>{
  const wait=deferred();
  const c=createMultilangController({persistence:{saveDocument:()=>wait.promise}});
  c.setDocument('ko',doc()); const pending=c.save('ko');
  c.setDocument('ko',doc('다른 작품'),{replaceDocument:true}); wait.resolve({id:'다른작품에붙이면안됨'});
  assert.equal((await pending).status,'stale'); assert.equal(c.snapshot().slots.ko.receipt,null);
  assert.equal((await c.reopen('ko')).status,'save_required');
});
test('재열기 중 편집하면 열린 저장본으로 현재 편집을 덮지 않는다',async()=>{
  const wait=deferred();
  const c=createMultilangController({persistence:{saveDocument:async()=>({id:'저장본'}),openDocument:()=>wait.promise}});
  c.setDocument('ko',doc()); await c.save('ko'); c.setDocument('ko',doc(''));
  const pending=c.reopen('ko'); c.setDocument('ko',doc('사용자 최신편집')); wait.resolve(doc());
  assert.equal((await pending).status,'stale'); assert.equal(c.snapshot().slots.ko.document.sourceText,'사용자 최신편집');
  assert.notEqual(c.snapshot().slots.ko.savedReceipt,null);
});
test('다음 저장 실패는 이전 정상 저장본과 현재 편집을 보존한다',async()=>{
  let count=0;
  const c=createMultilangController({persistence:{saveDocument:async()=>{
    if(++count===1)return {id:'정상저장'};throw new Error('저장실패');}}});
  c.setDocument('ko',doc()); await c.save('ko'); c.setDocument('ko',doc('새 수정'));
  assert.equal((await c.save('ko')).status,'failed');
  assert.deepEqual(c.snapshot().slots.ko.receipt,{id:'정상저장'});
  assert.equal(c.snapshot().slots.ko.document.sourceText,'새 수정');
});
test('adapter의 미지원/미공급 표시는 실패와 구분하되 성공 상태는 발급하지 않는다',async()=>{
  let kind='unsupported';const c=createMultilangController({runtimeBridge:{sourceAnalysis:async()=>{
    const error=new Error('원인 보존');error.presentationStatus=kind;throw error;}}});
  c.setDocument('ko',doc());assert.equal((await c.analyze('ko')).status,'unsupported');
  kind='supply_missing';assert.equal((await c.analyze('ko')).status,'supply_missing');
  kind='accepted';assert.equal((await c.analyze('ko')).status,'failed');
  assert.equal(c.snapshot().slots.ko.analysis,null);assert.equal(c.snapshot().slots.ko.error,'원인 보존');
});
