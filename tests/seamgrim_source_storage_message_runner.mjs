import assert from 'node:assert/strict';
import {sourceStorageErrorMessage} from '../solutions/seamgrim_ui_mvp/ui/runtime/native_source_transport.js';
for(const message of ["Failed to execute 'send' on 'XMLHttpRequest': Failed to load",'TypeError: Failed to fetch','NetworkError: A network error occurred.']) {
  const shown=sourceStorageErrorMessage(new Error(message));
  assert(shown.includes('저장 완료를 확인하지 못했습니다'));
  assert(!shown.includes('XMLHttpRequest'));assert(shown.includes('성공으로 처리하거나'));
}
for(const message of ['E_SOURCE_WORLD_CURRENT: RecoveryRequired','E_PROJECT_RUNTIME_SOURCE_CHANGED: 원문이 바뀌었습니다.','E_SOURCE_SUCCESSOR_STARTED','E_SOURCE_PARSE: 문자열 NetworkError','E_SOURCE_PARSE: Failed to fetch'])
  assert.equal(sourceStorageErrorMessage(new Error(message)),message);
console.log('PASS: network display only; custody/source diagnostics preserved');
const conflict=sourceStorageErrorMessage(new Error('E_NATIVE_SOURCE_SOURCE_CHANGED'));
assert(conflict.startsWith('E_NATIVE_SOURCE_SOURCE_CHANGED'));
for(const text of ['기존 저장본은 보존','마지막 저장 자료','파일을 다시 가져온 뒤','새 ID로 복제'])assert(conflict.includes(text));
assert.equal(sourceStorageErrorMessage(new Error('E_SOURCE_PARSE: E_NATIVE_SOURCE_SOURCE_CHANGED')),'E_SOURCE_PARSE: E_NATIVE_SOURCE_SOURCE_CHANGED');
console.log('PASS: exact source conflict supplies only existing reopen/explicit clone actions; embedded program text is not relabelled');
const selection='E_NATIVE_SOURCE_AUTHORING_SELECTION_REQUIRED: 원문을 열 때 받은 기준 판으로 수정 저장하세요';
assert(sourceStorageErrorMessage(new Error(selection)).startsWith(selection));
assert(sourceStorageErrorMessage(new Error(selection)).includes('새 ID로 복제'));
assert.equal(sourceStorageErrorMessage(new Error('E_SOURCE_PARSE: '+selection)),'E_SOURCE_PARSE: '+selection);
console.log('PASS: exact authoring-selection guard keeps its cause and adds existing next actions');
for(const message of ['E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED',
 'E_REPLAY_COMPLETED_RECEIPT_REQUIRED: 완료 영수증을 확인하지 못했습니다 (CustodyConflict)',
 'Error: E_NEW_RECORD_TERMINAL_REQUIRED: CustodyConflict']) {
 const shown=sourceStorageErrorMessage(new Error(message));
 assert(shown.startsWith(message));
 for(const text of ['확인하지 못했습니다','지우거나 덮어쓰지 마세요','새 ID로 복제','원문만 새 작품','복구하는 것은 아닙니다','자동 복구는 아직 지원하지 않습니다'])assert(shown.includes(text));
 assert(!shown.includes('RecoveryRequired'));assert(!shown.includes('복구 완료'));
}
for(const message of ['E_SOURCE_PARSE: E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED',
 'E_SOURCE_PARSE: E_NEW_RECORD_TERMINAL_REQUIRED: CustodyConflict','CustodyConflict',
 'E_NEW_RECORD_TERMINAL_REQUIRED: OtherCause','E_NATIVE_SOURCE_LOGICAL_PREFIX_CHANGED_EXTRA'])
 assert.equal(sourceStorageErrorMessage(new Error(message)),message);
console.log('PASS: exact finish refusal adds conditional notice only; no pending classification, recovery or source-text relabelling');
