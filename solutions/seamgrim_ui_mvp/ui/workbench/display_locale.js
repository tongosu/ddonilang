import {createExecutionLifecycle, executionStatusLabel} from './execution_lifecycle.js';

// UI preference only. Never pass locale or translated text to the compiler,
// execution, project source, storage receipt, or replay APIs.
export const DISPLAY_LOCALES = Object.freeze(['ko', 'ja']);
export const DISPLAY_STORAGE_KEY = 'ddn.workbench.display-locale.v1';

export const DISPLAY_MESSAGES = Object.freeze({
  projects: ['프로젝트', 'プロジェクト'],
  search: ['DDN 찾기', 'DDNを探す'],
  lessons: ['첫걸음', 'はじめの一歩'],
  collection: ['공식 모음', '公式の作品'],
  myWork: ['내 작업', '自分の作品'],
  newProject: ['새 프로젝트', '新しいプロジェクト'],
  lessonGuide: ['학습 안내', '学習ガイド'],
  previous: ['이전 활동', '前の課題'],
  next: ['다음 활동', '次の課題'],
  save: ['저장', '保存'],
  savePending: ['저장 중…', '保存中…'],
  savePendingNotice: ['저장 중… 완료될 때까지 창을 닫지 마세요.', '保存中です。完了するまでこの画面を閉じないでください。'],
  savePendingHint: ['저장소 확인에 시간이 걸릴 수 있습니다. 완료 또는 실패 안내가 나올 때까지 기다려 주세요.', '保存先の確認に時間がかかる場合があります。完了または問題の案内が出るまでお待ちください。'],
  saveSourceDone: ['원문 저장 완료', 'ソースの保存処理が完了しました。実行状態の保存とは別です。'],
  saveWorldDone: ['실행 상태 저장 완료', '実行状態の保存処理が完了しました。'],
  saveServerDone: ['서버 저장 완료', 'サーバーへの保存処理が完了しました。'],
  saveLocalOnly: ['이 기기 임시 저장', 'この端末の一時保存です。Nativeの保護保存とは異なります。'],
  saveProblem: ['저장 확인 문제', '保存の確認で問題が発生しました。元の案内を確認してください。'],
  saveRejected: ['저장 요청 거부', '保存の要求が拒否されました。元の案内を確認してください。'],
  saveConflict: ['저장 충돌', '保存が競合しました。元の案内を確認してください。'],
  sourceReopened: ['저장 원문 다시 열기', '保存されたソースを開き直しました。実行は開始していません。'],
  reopen: ['다시 열기', '開き直す'],
  diagnostics: ['오류·진단', 'エラー・診断'],
  output: ['출력', '出力'],
  sourceOpen: ['v25 원문 열기', 'v25のソースを開く'],
  revisions: ['저장·리비전', '保存・リビジョン'],
  inputRecord: ['입력 기록', '入力の記録'],
  files: ['프로젝트 파일', 'プロジェクトのファイル'],
  identity: ['작성 정체 자료', '定義の識別情報'],
  history: ['기록', '履歴'],
  transferSettings: ['가져오기·내보내기 설정', '入出力の設定'],
  packages: ['가지 관리', 'パッケージ管理'],
  clone: ['내 버전 만들기', '自分の版を作る'],
  preserve: ['로컬 변경 보존', 'ローカルの変更を保持'],
  close: ['닫기', '閉じる'],
  exportProject: ['⇩ 내보내기', '⇩ 書き出す'],
  importProject: ['⇧ 가져오기', '⇧ 読み込む'],
  moreTools: ['세부', '詳細'],
  proofCheck: ['보임 검사', '条件付き証明の検査'],
  completion: ['추천', '候補'],
  completionHelp: ['추천 열기: Ctrl+Space. 선택: 클릭 또는 Ctrl+Enter. 닫기: Esc.', '候補: Ctrl+Space。選択: クリックまたはCtrl+Enter。閉じる: Esc。'],
  source: ['원문', 'ソース'],
  searchPlaceholder: ['DDN 이름, 태그', 'DDNの名前、タグ'],
  run: ['실행', '実行'],
  pause: ['일시정지', '一時停止'],
  resume: ['재개', '再開'],
  runChanged: ['수정본 실행', '編集したソースを実行'],
  logicalRun: ['논리 입력 실행', '論理入力を実行'],
  step: ['한 마디 앞으로', '1マディ進める'],
  stop: ['중지', '停止'],
  back: ['한 마디 뒤로 보기', '1マディ前を見る'],
  branch: ['현재 마디에서 갈라 이어가기', '現在のマディから分岐して続ける'],
  disabled: ['사용 불가', '使用不可'],
  commandPending: ['이전 실행 명령이 완료되는 중입니다.', '前の実行操作が完了するまで待ってください。'],
  stepUnavailable: ['최신 마디에서 멈춘 연속 실행만 한 마디 진행할 수 있습니다.', '最新マディで一時停止した連続実行だけを1マディ進められます。'],
  stopUnavailable: ['준비 중이거나 활성 상태인 실행이 없습니다.', '準備中または動作中の実行がありません。'],
  backUnavailable: ['읽을 수 있는 이전 마디가 없습니다.', '表示できる前のマディがありません。'],
  branchUnavailable: ['갈라 이어갈 기록 마디가 없습니다.', '分岐して続ける記録マディがありません。'],
  logicalFailed: ['논리 입력 중단', '論理入力が中断されました'],
  logicalRunning: ['논리 입력 실행 중', '論理入力を実行中'],
  logicalPaused: ['논리 입력 일시정지 · 기록 결과', '論理入力を一時停止・記録された結果'],
  logicalReady: ['논리 입력 준비', '論理入力の準備'],
  runIdle: ['실행 전', '実行前'],
  runPreparing: ['실행 준비 중', '実行の準備中'],
  runRunning: ['실행 중', '実行中'],
  runPausing: ['마디 경계에서 멈추는 중', 'マディの境界で一時停止中'],
  runPaused: ['멈춤', '一時停止中'],
  runStopping: ['실행을 종료하는 중', '実行を終了中'],
  runDone: ['실행 완료', '実行完了'],
  runStopped: ['실행 중지', '実行停止'],
  runFailed: ['실행 실패', '実行失敗'],
  runHistory: ['과거 마디 읽기 전용', '過去のマディを表示・読み取り専用'],
  runModeLive: ['연속 실행', '連続実行'],
  runModeOnce: ['한 번 실행', '1回実行'],
  runFixSource: ['실행 실패 · 원문을 고친 뒤 다시 실행해 주세요.', '実行失敗・ソースを修正してから再実行してください。'],
  runPrevious: ['이전 성공 기록 보존', '前回の成功記録を保持'],
  runImported: ['불러온 기록입니다. 원문을 실행해 주세요.', '読み込んだ記録です。ソースを実行してください。'],
  runDirty: ['수정한 원문은 새 실행에 적용됩니다.', '編集したソースは新しい実行に適用されます。'],
  language: ['표시 언어', '表示言語'],
  unsupported: ['지원하지 않는 표시 언어입니다. 한국어 또는 일본어를 선택하세요.', 'この表示言語には対応していません。韓国語または日本語を選んでください。'],
  notSaved: ['표시 언어를 이 창에서만 적용했습니다. 설정을 저장하지 못했습니다.', '表示言語はこの画面だけに適用されました。設定を保存できませんでした。'],
  boundary: ['원문·실행 결과·오류 원문은 번역하지 않습니다.', 'ソース・実行結果・エラー原文は翻訳しません。'],
});

export function displayMessage(key, locale, original = '') {
  if (!DISPLAY_LOCALES.includes(locale)) throw new RangeError('Unsupported display locale');
  const entry = DISPLAY_MESSAGES[key];
  const text = entry?.[locale === 'ja' ? 1 : 0];
  if (typeof text === 'string' && text.length) return {text, fallback: false};
  return {text: `[ko] ${entry?.[0] ?? original}`, fallback: true};
}

// A presentation snapshot only. It cannot normalize or mutate the live lifecycle.
export function runStatusPresentation(presentation, locale) {
  if (!DISPLAY_LOCALES.includes(locale)) throw new RangeError('Unsupported display locale');
  const message = key => displayMessage(key, locale).text;
  const lifecycle = createExecutionLifecycle(presentation.lifecycle ?? {});
  const original = presentation.stepInFlight && lifecycle.status === 'preparing'
    ? '실행 중' : executionStatusLabel(lifecycle);
  const keys = {idle:'runIdle',preparing:'runPreparing',running:'runRunning',pausing:'runPausing',
    paused:'runPaused',stopping:'runStopping',done:'runDone',failed:'runFailed',history_readonly:'runHistory'};
  let lifecycleText = original, fallback = false;
  if (locale === 'ja') {
    const key = presentation.stepInFlight && lifecycle.status === 'preparing' ? 'runRunning'
      : lifecycle.status === 'done' && lifecycle.final_status === 'stopped' ? 'runStopped'
      : Object.hasOwn(keys,lifecycle.status) ? keys[lifecycle.status] : null;
    if (!key) {lifecycleText = '[ko] ' + original;fallback = true;}
    else {
      lifecycleText = message(key);
      if (!(presentation.stepInFlight && lifecycle.status === 'preparing')
          && (lifecycle.mode === 'live' || lifecycle.mode === 'oneshot')) {
        lifecycleText += ' · ' + message(lifecycle.mode === 'live' ? 'runModeLive' : 'runModeOnce');
      }
    }
  }
  switch (presentation.kind) {
    case 'readiness_blocked': return {text:message('runFixSource'),fallback:false};
    case 'failure': return {text:message('runFailed') + ' · ' + (presentation.code ?? 'E_PROJECT_RUN_FAILED'),fallback:false};
    case 'previous_success': return {text:message('runPrevious') + ' · ' + lifecycleText,fallback};
    case 'imported_record': return {text:message('runImported'),fallback:false};
    case 'dirty_source': return {text:lifecycleText + ' · ' + message('runDirty'),fallback};
    case 'lifecycle': return {text:lifecycleText,fallback};
    default: return {text:'[ko] ' + (presentation.original ?? ''),fallback:true};
  }
}

// An immutable UI snapshot only. No storage reads, writes, retries or receipt
// admission. Keep original details/codes visible, including uncertain recovery.
export function persistencePresentation(presentation, locale) {
  if (!DISPLAY_LOCALES.includes(locale)) throw new RangeError('Unsupported display locale');
  const message = key => displayMessage(key, locale).text;
  const saving = presentation.phase === 'saving';
  const common = {buttonText:message(saving?'savePending':'save'),title:saving?message('savePendingHint'):''};
  if (saving) return {...common,text:message('savePendingNotice'),fallback:false};
  const original = presentation.original ?? '';
  if (locale === 'ko') return {...common,text:original,fallback:false};
  const outcomes = {source_only:'saveSourceDone',native_world:'saveWorldDone',server_project:'saveServerDone',local_fallback:'saveLocalOnly'};
  const phases = {error:'saveProblem',rejected:'saveRejected',conflict:'saveConflict'};
  const key = presentation.phase === 'source_reopened' && presentation.statusMode === 'source_only' ? 'sourceReopened'
    : presentation.phase === 'saved' && presentation.statusMode === presentation.outcome
    ? Object.hasOwn(outcomes,presentation.outcome)?outcomes[presentation.outcome]:null
    : Object.hasOwn(phases,presentation.phase)?phases[presentation.phase]:null;
  const detail = original ? '[ko] '+original : '';
  return {...common,text:key?message(key)+(detail?' · '+detail:''):detail||'[ko] ',fallback:!key||Boolean(detail)};
}

export function installDisplayLocale(root = document) {
  const select = root.querySelector('[data-testid="display-language"]');
  const notice = root.querySelector('[data-testid="display-language-notice"]');
  if (!select || !notice) return null;
  const nodes = Array.from(root.querySelectorAll('[data-display-key]'), node => ({
    node, key: node.dataset.displayKey, original: node.textContent,
    title: node.getAttribute('title'), label: node.getAttribute('aria-label'),
  }));
  // Metadata-only resources must not replace icon/text children or input values.
  const metadata = [];
  for (const [selector, dataKey, attribute] of [
    ['[data-display-title-key]', 'displayTitleKey', 'title'],
    ['[data-display-label-key]', 'displayLabelKey', 'aria-label'],
    ['[data-display-placeholder-key]', 'displayPlaceholderKey', 'placeholder'],
  ]) {
    for (const node of root.querySelectorAll(selector)) {
      metadata.push({node, key: node.dataset[dataKey], attribute, original: node.getAttribute(attribute)});
    }
  }
  let locale = 'ko';
  let storage;
  let startupProblem = '';
  try {
    storage = root.defaultView.localStorage;
    const saved = storage.getItem(DISPLAY_STORAGE_KEY);
    if (saved !== null && !DISPLAY_LOCALES.includes(saved)) startupProblem = 'unsupported';
    else if (saved) locale = saved;
  } catch { startupProblem = 'notSaved'; }

  function paint(problem = '') {
    root.documentElement.lang = locale;
    root.documentElement.dataset.displayLocale = locale;
    select.value = locale;
    for (const {node, key, original, title, label} of nodes) {
      const message = displayMessage(key, locale, original.trim());
      node.textContent = locale === 'ko' && !message.fallback ? original : message.text;
      if (title !== null) node.setAttribute('title', locale === 'ko' ? title : message.text);
      if (label !== null) node.setAttribute('aria-label', locale === 'ko' ? label : message.text);
      if (message.fallback) node.dataset.displayFallback = 'ko';
      else delete node.dataset.displayFallback;
    }
    for (const {node, key, attribute, original} of metadata) {
      const message = displayMessage(key, locale, original ?? '');
      if (locale === 'ko' && original === null) node.removeAttribute(attribute);
      else node.setAttribute(attribute, locale === 'ko' ? original : message.text);
      if (message.fallback) node.dataset.displayMetadataFallback = 'ko';
      else delete node.dataset.displayMetadataFallback;
    }
    notice.textContent = problem ? displayMessage(problem, locale).text : '';
    notice.hidden = !problem;
    if (problem) select.closest('details').open = true;
    // Repaint dynamic presentation from its unchanged original diagnostics.
    // This event carries only a UI preference, never a compiler/runtime input.
    if (typeof root.dispatchEvent === 'function' && typeof root.defaultView?.CustomEvent === 'function') {
      root.dispatchEvent(new root.defaultView.CustomEvent('ddn-display-locale-change', {detail: {locale}}));
    }
  }
  function change(next) {
    if (!DISPLAY_LOCALES.includes(next)) { paint('unsupported'); return false; }
    locale = next;
    let problem = '';
    try {
      if (!storage) throw new Error('UI storage unavailable');
      storage.setItem(DISPLAY_STORAGE_KEY, next);
    } catch { problem = 'notSaved'; }
    paint(problem);
    return true;
  }
  select.addEventListener('change', () => change(select.value));
  paint(startupProblem);
  return Object.freeze({get locale() { return locale; }, change});
}
