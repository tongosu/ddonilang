import {diagnosticPresentation, labels} from './messages.js';

import {textareaSelectionForUtf8Span} from './source_selection.js';



// 기존 화면/편집기/locale를 건드리지 않는 명시적 mount 접점.

export function mountMultilangPanel(root, controller, {technicalDetails = false, readOtherSource = () => null} = {}) {

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

    editor.setAttribute('aria-label', legend.textContent);editor.dataset.testid=`multilang-${language}-source`;

    const status = document.createElement('p'); status.setAttribute('role','status');

    const savedStatus = document.createElement('p');

    const output = document.createElement('pre');output.dataset.testid=`multilang-${language}-output`;

    const locations = document.createElement('div');
    const preview = document.createElement('div');

    const details = document.createElement('details');

    const summary = document.createElement('summary'); summary.textContent = '원진단·결과 자료 / 元の診断・結果';

    const raw = document.createElement('pre'); details.append(summary, raw);

    const buttons = {};

    field.append(legend, editor, status, savedStatus);

    for (const action of ['analyze','run','save','reopen']) {

      const button = document.createElement('button'); button.type = 'button';button.dataset.testid=`multilang-${language}-${action}`;

      button.addEventListener('click', () => { void controller[action](language); });

      buttons[action] = button; field.append(button);

    }

    editor.addEventListener('input', () => {

      const previous = controller.snapshot().slots[language].document;

      if (previous) controller.setDocument(language, {...previous, sourceText:editor.value});

    });

    field.append(output, locations, preview);

    if (technicalDetails) field.append(details);

    panel.append(field);

    controls[language] = {editor, status, savedStatus, output, raw, buttons, locations, preview};

  }

  root.append(panel);

  const unsubscribe = controller.subscribe(state => {

    const text = labels[state.locale]; title.textContent = text.title;

    locale.value = state.locale; comparison.textContent = state.comparison==='limited_callable'?text.limitedNotice:text.unverified;

    limitation.textContent = text.draftNotice;

    for (const language of ['ko','ja']) {

      const item = state.slots[language], control = controls[language];

      control.editor.disabled = !item.document;

      // textarea 표시만 CRLF를 LF로 본다. 저장/분석 원문은 item.document의 bytes를 유지한다.

      const displayedSource = (item.document?.sourceText ?? '').replace(/\r\n?/g,'\n');

      if (control.editor.value !== displayedSource) control.editor.value = item.document?.sourceText ?? '';

      control.status.textContent = item.status==='prepared'&&item.document?.limitedRole==='original'

        ?text.original_registered:(text[item.status]??item.status);

      control.savedStatus.textContent = item.savedReceipt

        ? (item.savedContentMatches ? text.matchesSaved : text.differsSaved) : text.noSaved;

      for (const [action, button] of Object.entries(control.buttons)) {

        button.textContent = action==='analyze'&&item.document?.limitedRole

          ?(item.document.limitedRole==='original'?text.registerOriginal:text.checkConsumer):text[action];

        button.disabled = item.busy || !item.document || (action !== 'reopen' && !item.document.sourceText)

          || (action === 'run' && (item.document?.projectPreparation===true ? !item.preparation : item.document?.limitedRole

            ?item.document.limitedRole==='original'||!item.preparation:item.analysis?.outcome !== 'ACCEPTED'))

          || (['save','reopen'].includes(action) && Boolean(item.document?.limitedRole))

          || (action === 'reopen' && item.receipt === null);

      }

      const lines = (item.analysis?.diagnostics ?? []).map(diagnostic => {

        const d = diagnosticPresentation(diagnostic, state.locale,{sourceText:item.document?.sourceText,sourcePath:item.document?.path});

        const position = d.span ? `${d.span.encoding} ${d.span.start}..${d.span.end}` : '';

        return d.humanized ? `${d.guidance}\n${d.path??''} ${position}\n진단 번호 · ${d.code}` : `${d.code} [${d.severity}] ${d.guidance}\n${d.original}\n${d.path??''} ${position}`;

      });

      const texts = (item.document?.projectPreparation===true || item.document?.limitedRole==='consumer') ? item.result?.output_log_texts

        :item.result?.engine_state?.output_display?.texts;

      if (Array.isArray(texts) && texts.every(value => typeof value === 'string')) lines.push(...texts);

      const execution = item.executionDiagnostic;
      if (execution) {
        const span = execution.source_span;
        const canCorrect = execution.phase === 'runtime_execution' && span && execution.source_path === item.document?.path
          && textareaSelectionForUtf8Span(item.document.sourceText, {encoding:'utf-8-byte', ...span}) !== null;
        const guidance = canCorrect ? text.runtimeSourceGuidance : diagnosticPresentation(execution, state.locale,{sourceText:item.document?.sourceText,sourcePath:item.document?.path}).guidance;
        lines.push(state.locale==='ko' && diagnosticPresentation(execution,state.locale).humanized ? `${diagnosticPresentation(execution,state.locale,{sourceText:item.document?.sourceText,sourcePath:item.document?.path}).guidance}\n${execution.source_path}`+(span?` UTF-8 바이트 ${span.start}..${span.end}`:'')+`\n진단 번호 · ${execution.code}` : `${execution.code} [${execution.phase}] ${execution.message}\n${execution.source_path}`+(span?` utf-8-byte ${span.start}..${span.end}`:'')+`\n${guidance}`);
      }
      if (item.error !== null) {

        if (!execution) {

          const code = 'E_MULTILANG_NATIVE_SELECTION_REQUIRED';

          lines.push(item.error.startsWith(code + ':') ? code + ': ' + text.nativeSelectionRequired : item.error);

        }

        if(item.document?.limitedRole==='consumer'&&item.error.includes('E_CALL_REQUIRED_PIN_MISSING')) lines.push(text.requiredPinHint);

      }

      control.output.textContent = lines.join('\n\n');

      control.locations.replaceChildren();
      control.preview.replaceChildren();

      const positions = [...(item.analysis?.diagnostics ?? [])];
      if (execution) positions.push({code:execution.code, path:execution.source_path,
        span:execution.source_span ? {encoding:'utf-8-byte', ...execution.source_span} : null});
      for (const diagnostic of positions) {

        const button = document.createElement('button'); button.type = 'button';

        const sameFile = diagnostic.path === item.document?.path;

        const source = sameFile ? item.document?.sourceText : readOtherSource(item.document, diagnostic.path);

        const selection = textareaSelectionForUtf8Span(source, diagnostic.span);

        const locationLabel = sameFile

          ? (state.locale === 'ja' ? 'ソース位置へ移動' : '원문 위치로 이동')

          : (state.locale === 'ja' ? '別ファイルの位置を表示' : '다른 원문의 위치 보기');

        button.textContent = `${diagnostic.code} · ${locationLabel}${sameFile ? '' : ` · ${diagnostic.path}`}`;

        button.disabled = selection === null;

        button.addEventListener('click', () => {

          const latest = controller.snapshot().slots[language];

          if (!selection || latest.generation !== item.generation || latest.document?.sourceText !== item.document.sourceText

              || latest.document.path !== item.document.path || latest.document.sourceProfileIdentity !== item.document.sourceProfileIdentity) return;

          if (sameFile) {

            control.editor.focus(); control.editor.setSelectionRange(selection.start, selection.end);

            return;

          }

          control.preview.replaceChildren();

          if (readOtherSource(latest.document, diagnostic.path) !== source) return;

          const label = document.createElement('label');

          label.textContent = `${diagnostic.path} · ${state.locale === 'ja' ? '読み取り専用' : '읽기 전용'}`;

          const original = document.createElement('textarea');

          original.readOnly = true;

          original.rows = 6;

          original.value = source;

          label.append(original);

          control.preview.append(label);

          original.focus(); original.setSelectionRange(selection.start, selection.end);

        });

        control.locations.append(button);

      }

      control.raw.textContent = JSON.stringify({analysis:item.analysis, result:item.result, executionDiagnostic:execution}, null, 2);

    }

  });

  return Object.freeze({dispose() { unsubscribe(); panel.remove(); }});

}

