import {diagnosticPresentation, labels} from './messages.js';
import {textareaSelectionForUtf8Span} from './source_selection.js';

// 기존 화면/편집기/locale를 건드리지 않는 명시적 mount 접점.
export function mountMultilangPanel(root, controller, {technicalDetails = false} = {}) {
  const document = root.ownerDocument;
  const panel = document.createElement('section');
  const title = document.createElement('h2');
  const locale = document.createElement('select');
  locale.setAttribute('aria-label', '표시 언어 / 表示言語');
  for (const [value, text] of [['ko','한국어'],['ja','日本語']]) {
    const option = document.createElement('option'); option.value = value; option.textContent = text;
    locale.append(option);
  }
  locale.addEventListener('change', () => controller.setLocale(locale.value));
  const comparison = document.createElement('p');
  const limitation = document.createElement('p');
  panel.append(title, locale, comparison, limitation);
  const controls = {};
  for (const language of ['ko','ja']) {
    const field = document.createElement('fieldset');
    const legend = document.createElement('legend'); legend.textContent = language === 'ko' ? '한국어 원문' : '日本語ソース';
    const editor = document.createElement('textarea'); editor.rows = 6;
    editor.setAttribute('aria-label', legend.textContent);
    const status = document.createElement('p'); status.setAttribute('role','status');
    const savedStatus = document.createElement('p');
    const output = document.createElement('pre');
    const locations = document.createElement('div');
    const details = document.createElement('details');
    const summary = document.createElement('summary'); summary.textContent = '원진단·결과 자료 / 元の診断・結果';
    const raw = document.createElement('pre'); details.append(summary, raw);
    const buttons = {};
    field.append(legend, editor, status, savedStatus);
    for (const action of ['analyze','run','save','reopen']) {
      const button = document.createElement('button'); button.type = 'button';
      button.addEventListener('click', () => { void controller[action](language); });
      buttons[action] = button; field.append(button);
    }
    editor.addEventListener('input', () => {
      const previous = controller.snapshot().slots[language].document;
      if (previous) controller.setDocument(language, {...previous, sourceText:editor.value});
    });
    field.append(output, locations);
    if (technicalDetails) field.append(details);
    panel.append(field);
    controls[language] = {editor, status, savedStatus, output, raw, buttons, locations};
  }
  root.append(panel);
  const unsubscribe = controller.subscribe(state => {
    const text = labels[state.locale]; title.textContent = text.title;
    locale.value = state.locale; comparison.textContent = text.unverified;
    limitation.textContent = text.draftNotice;
    for (const language of ['ko','ja']) {
      const item = state.slots[language], control = controls[language];
      control.editor.disabled = !item.document;
      // textarea 표시만 CRLF를 LF로 본다. 저장/분석 원문은 item.document의 bytes를 유지한다.
      const displayedSource = (item.document?.sourceText ?? '').replace(/\r\n?/g,'\n');
      if (control.editor.value !== displayedSource) control.editor.value = item.document?.sourceText ?? '';
      control.status.textContent = text[item.status] ?? item.status;
      control.savedStatus.textContent = item.savedReceipt
        ? (item.savedContentMatches ? text.matchesSaved : text.differsSaved) : text.noSaved;
      for (const [action, button] of Object.entries(control.buttons)) {
        button.textContent = text[action];
        button.disabled = item.busy || !item.document || (action !== 'reopen' && !item.document.sourceText)
          || (action === 'run' && item.analysis?.outcome !== 'ACCEPTED')
          || (action === 'reopen' && item.receipt === null);
      }
      const lines = (item.analysis?.diagnostics ?? []).map(diagnostic => {
        const d = diagnosticPresentation(diagnostic, state.locale);
        const position = d.span ? `${d.span.encoding} ${d.span.start}..${d.span.end}` : '';
        return `${d.code} [${d.severity}] ${d.guidance}\n${d.original}\n${d.path ?? ''} ${position}`;
      });
      const texts = item.result?.engine_state?.output_display?.texts;
      if (Array.isArray(texts) && texts.every(value => typeof value === 'string')) lines.push(...texts);
      if (item.error !== null) lines.push(item.error);
      control.output.textContent = lines.join('\n\n');
      control.locations.replaceChildren();
      for (const diagnostic of item.analysis?.diagnostics ?? []) {
        const button = document.createElement('button'); button.type = 'button';
        button.textContent = `${diagnostic.code} · ${state.locale === 'ja' ? 'ソース位置へ移動' : '원문 위치로 이동'}`;
        const selection = diagnostic.path === item.document?.path
          ? textareaSelectionForUtf8Span(item.document.sourceText, diagnostic.span) : null;
        button.disabled = selection === null;
        button.addEventListener('click', () => {
          const latest = controller.snapshot().slots[language];
          if (!selection || latest.generation !== item.generation || latest.document?.sourceText !== item.document.sourceText
              || latest.document.path !== item.document.path || latest.document.sourceProfileIdentity !== item.document.sourceProfileIdentity) return;
          control.editor.focus(); control.editor.setSelectionRange(selection.start, selection.end);
        });
        control.locations.append(button);
      }
      control.raw.textContent = JSON.stringify({analysis:item.analysis, result:item.result}, null, 2);
    }
  });
  return Object.freeze({dispose() { unsubscribe(); panel.remove(); }});
}
