// 표시 전용. 원진단의 코드·심각도·원문 위치와 실행 의미는 변경하지 않는다.
const hints = Object.freeze({
  E_PARSE: Object.freeze({ko: '괄호·연산자·문장 끝을 원진단 위치에서 확인하세요.',
    ja: '元の診断の位置で、括弧・演算子・文末を確認してください。'}),
  E_SOURCE_PROFILE_REQUIRED: Object.freeze({ko: '원문 프로필 공급이 필요합니다. 표시 언어와 별개입니다.',
    ja: 'ソースのプロファイルが必要です。表示言語とは別です。'}),
  E_SOURCE_PROFILE_UNSUPPORTED: Object.freeze({ko: '이 원문 프로필은 지원되지 않습니다.',
    ja: 'このソースのプロファイルには対応していません。'}),
});

export function diagnosticPresentation(diagnostic, locale = 'ko') {
  if (!['ko', 'ja'].includes(locale)) throw new TypeError('표시 언어는 ko/ja만 지원합니다.');
  return Object.freeze({
    code: diagnostic.code, severity: diagnostic.severity,
    original: diagnostic.message, path: diagnostic.path,
    span: diagnostic.span == null ? null : Object.freeze({...diagnostic.span}),
    guidance: hints[diagnostic.code]?.[locale] ?? (locale === 'ja'
      ? '[ko] この診断の説明は未翻訳です。元の診断を確認してください。'
      : '이 코드의 보조 설명은 없습니다. 원진단을 확인하세요.'),
    translated: Boolean(hints[diagnostic.code]),
  });
}

export const labels = Object.freeze({
  ko: Object.freeze({title:'다국어 소비 패널', analyze:'검사', run:'한 번 계산',
    draftNotice:'원문 확인용 소비 패널입니다. DDL 작성 도구와 별개이며 일본어 문구는 원어민 검토 전입니다.',
    matchesSaved:'현재 원문은 마지막 저장본과 같습니다. 분석 유효성은 별도입니다.',
    differsSaved:'저장본 이후 수정되었습니다. 저장본으로 돌아갈 수 있습니다.', noSaved:'재열기 가능한 저장본이 없습니다.',
    save:'원문 저장', reopen:'저장본 열고 재검사', empty:'원문을 입력하세요.',
    supply_missing:'공통 기능 공급 대기', edited:'수정됨 · 재검사 필요',
    unsupported:'현재 원문 문맥은 분석 미지원',
    recheck_required:'공급 자료 변경 · 재검사 필요',
    checking:'검사 중', accepted:'분석 수용 · 실행 전', rejected:'분석 거부',
    running:'계산 중', completed:'계산 결과 수신', saving:'원문 저장 중', saved:'원문 저장 응답 수신',
    reopening:'저장본 여는 중', stale:'이전 원문의 결과 폐기', failed:'작업 실패',
    idle:'원문 대기', unverified:'공통 의미 비교는 아직 연결되지 않았습니다.'}),
  ja: Object.freeze({title:'多言語ソース確認パネル', analyze:'検査', run:'一回計算',
    draftNotice:'ソース確認用パネルです。DDL作成ツールとは別です。日本語の文言は母語話者による確認前です。',
    matchesSaved:'現在のソースは最後の保存内容と同じです。解析の有効性は別途確認します。',
    differsSaved:'保存後に変更されています。保存したソースに戻せます。', noSaved:'開き直せる保存内容はありません。',
    save:'ソースを保存', reopen:'保存したソースを開いて再検査', empty:'ソースを入力してください。',
    supply_missing:'共通機能の供給待ち', edited:'変更済み・再検査が必要',
    unsupported:'このソース文脈の解析は未対応です',
    recheck_required:'参照資料の変更・再検査が必要',
    checking:'検査中', accepted:'解析受理・実行前', rejected:'解析拒否',
    running:'計算中', completed:'計算結果を受信', saving:'ソースを保存中', saved:'保存の応答を受信',
    reopening:'保存したソースを開いています', stale:'古いソースの結果を破棄', failed:'操作失敗',
    idle:'ソース待ち', unverified:'共通の意味を比較する接続はまだありません。'}),
});
