// Presentation only. Never rewrite the compiler/runtime diagnostic or infer a
// different error code from message text. Unknown codes keep the raw original.
const JAPANESE_GUIDANCE = Object.freeze({
  E_PROJECT_RUN_FAILED: '作品を実行できませんでした。原文のエラーコードと位置を確認してください。',
  E_WASM_STEP_FAILED: '実行を続けられませんでした。原文のエラーコードと位置を確認してください。',
  E_SOURCE_FILE_OPEN: 'ソースファイルを開けませんでした。原文に示された理由を確認してください。',
  E_PROJECT_EDIT_FAILED: '作品の編集を適用できませんでした。原文に示された理由を確認してください。',
  E_PROJECT_RUNTIME_SOURCE_CHANGED: 'ソースが変更されています。作品のソースを更新して保存してから実行してください。',
  E_LOGICAL_INPUT_PROFILE_REQUIRED: 'この作品には論理入力の設定が必要です。入力の記録パネルを確認してください。',
  E_NATIVE_SOURCE_SOURCE_CHANGED: '保存されたソースと現在のソースが異なります。保存済みの版を確認してください。',
  E_NATIVE_SOURCE_SAVE_WORLD_MISMATCH: '実行結果が保存候補と一致しません。以前の保存内容を確認してください。',
  E_NATIVE_SOURCE_SAVE_RESULT_MISMATCH: '結果の識別情報が一致しません。以前の保存内容を確認してください。',
});

export function diagnosticPresentation(code, original, locale = 'ko') {
  if (locale !== 'ko' && locale !== 'ja') throw new RangeError('Unsupported display locale');
  const guidance = locale === 'ja' && Object.hasOwn(JAPANESE_GUIDANCE, code)
    ? JAPANESE_GUIDANCE[code] : '';
  return Object.freeze({original, guidance, fallback: locale === 'ja' && !guidance});
}

export function appendDiagnosticPresentation(node, diagnostic, original, locale = 'ko') {
  const view = diagnosticPresentation(diagnostic.code, original, locale);
  const doc = node.ownerDocument;
  node.dataset.diagnosticCode = diagnostic.code;
  if (view.fallback) node.dataset.displayFallback = 'ko';
  else delete node.dataset.displayFallback;
  if (view.guidance) {
    const guidance = doc.createElement('span');
    guidance.dataset.testid = 'diagnostic-guidance';
    guidance.lang = 'ja';
    guidance.style.display = 'block';
    guidance.textContent = view.guidance;
    node.append(guidance);
  }
  if (view.fallback) {
    const marker = doc.createElement('span');
    marker.textContent = '[ko] ';
    node.append(marker);
  }
  const raw = doc.createElement('span');
  raw.dataset.testid = 'diagnostic-original';
  raw.textContent = original;
  node.append(raw);
}

export function diagnosticSummary(locale, count, errors) {
  if (locale !== 'ko' && locale !== 'ja') throw new RangeError('Unsupported display locale');
  if (locale === 'ja') return {
    label: count ? `検証 (${count})` : '検証',
    aria: count ? `エラーと診断を開く、診断 ${count} 件${errors ? `、エラー ${errors} 件` : ''}`
      : 'エラーと診断を開く、診断なし',
    empty: '問題は見つかりませんでした。',
  };
  return {
    label: count ? `검증 (${count})` : '검증',
    aria: count ? `오류와 진단 열기, 진단 ${count}개${errors ? `, 오류 ${errors}개` : ''}`
      : '오류와 진단 열기, 진단 없음',
    empty: '문제가 발견되지 않았습니다.',
  };
}
