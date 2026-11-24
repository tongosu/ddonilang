// 공통 checker가 공급한 관찰을 표시한다. 수학적 관계나 판정을 추론하지 않는다.
export function conditionalAlgebraHtml(report, source, ordinaryHtml) {
  if (!report?.definitions?.some(item => item.conditional_definedness === true)) return ordinaryHtml(report, source);
  const escape = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
  const term = item => typeof item?.display_text === 'string' ? item.display_text : '수식 표시 미공급';
  const diagnostic = item => item ? `<pre>${escape(item.formatted || item.message || item.code)}</pre>` : '';
  const list = values => `<ul>${values.map(value => `<li>${escape(value)}</li>`).join('')}</ul>`;
  return `<p>조건부 약분 관찰 · ${report.status === 'accepted' ? '이번 명시 가정 아래 수용' : '거부'} · 무가정 증명 발급 아님</p>
    <p>가정이 참임을 증명하거나 제외점에서 값을 계산한 결과가 아닙니다. 거부는 등식이 거짓이라는 판정과 구별합니다.</p>
    ${diagnostic(report.diagnostic)}${report.definitions.map(definition => {
      if (definition.conditional_definedness !== true) return ordinaryHtml({...report, definitions:[definition]}, source);
      const assumptions = Array.isArray(definition.assumptions) ? definition.assumptions : [];
      const used = Array.isArray(definition.used_assumptions) ? definition.used_assumptions : [];
      const states = {accepted:'가정 아래 수용', rejected:'거부', not_checked:'미검사'};
      return `<section class="conditional-algebra"><h4>${escape(definition.name)} · ${escape(states[definition.status] || definition.status)}</h4>
        <p>작성한 목표: ${escape(term(definition.goal?.left))} = ${escape(term(definition.goal?.right))}</p>
        <h5>명시 가정</h5>${list(assumptions.map((item,index) => `가정 ${index+1}: ` + (item.relation === 'nonzero' ? `${term(item.expression)} ≠ 0` : item.relation === 'equality' ? `${term(item.left)} = ${term(item.right)}` : '지원하지 않는 관계 · 원래 응답을 확인하세요')))}
        ${diagnostic(definition.diagnostic)}
        ${definition.normalized_result ? `<p>공급된 정규형: ${escape(definition.normalized_result.left)} = ${escape(definition.normalized_result.right)}</p>` : '<p>검사된 정규형 미공급</p>'}
        <h5>약분 뒤에도 남는 조건</h5>${Array.isArray(definition.retained_conditions) ? list(definition.retained_conditions.map(item => `${item} ≠ 0`)) : '<p>조건 자료 미공급</p>'}
        <p>위 기호와 이 정의의 변수 대응</p>${list((definition.parameters || []).map(item => `${item.kernel_symbol}: ${item.name}`))}
        <p>사용 가정: ${escape(used.length ? used.map(index => Number.isSafeInteger(index) && index >= 0 && index < assumptions.length ? `가정 ${index+1}` : '가정 대응 미공급').join(', ') : '공급된 사용 가정 없음')} · 최소 가정 집합이라는 뜻은 아닙니다.</p>
        <ol>${(definition.steps || []).map(step => `<li>${escape(step.operation)} · ${escape(({checked:'수용',failed:'거부',not_checked:'미검사'})[step.status] || step.status)}${step.status === 'checked' ? `<p>${escape(term(step.equality?.left))} = ${escape(term(step.equality?.right))}</p>` : ''}</li>`).join('')}</ol>
        ${!definition.steps?.length ? '<p>검사된 단계 미공급 · 성공 단계를 보충하지 않습니다.</p>' : ''}</section>`;
    }).join('')}`;
}
export function goalText(goal) {
  if (goal?.kind === 'declared_proposition') return String(goal.label);
  if (goal?.kind === 'conjunction' && Array.isArray(goal.parts)) return '(' + goal.parts.map(goalText).join(' 이고 ') + ')';
  return '목표 상세 미공급';
}
export function evidenceLines(item) {
  const lines = [];
  const assumptions = Array.isArray(item.assumptions) ? item.assumptions : [];
  function walk(node, depth = 0) {
    if (depth > 64) { lines.push('  '.repeat(depth) + '더 깊은 근거는 원래 결과에서 확인하세요.'); return; }
    let label = '근거 상세 미공급';
    if (node?.kind === 'assumption') {
      const matches = assumptions.filter(a => a.assumption_slot === node.assumption_slot);
      label = matches.length === 1 ? `가정 사용: ${matches[0].local_name} — ${goalText(matches[0].goal)}` : '가정 참조 대응 미공급';
    } else if (node?.kind === 'conjunction') label = '근거 결합 (공급 순서)';
    else if (node?.kind === 'projection') label = `결합 근거의 인덱스 ${node.index} 선택 (0부터 시작)`;
    lines.push('  '.repeat(depth) + label);
    if (node?.kind === 'conjunction' && Array.isArray(node.parts)) for (const part of node.parts) walk(part, depth + 1);
    if (node?.kind === 'projection') walk(node.evidence, depth + 1);
  }
  if (item.evidence) walk(item.evidence);
  else lines.push('검사된 근거가 공급되지 않았습니다.');
  if (item.application) {
    lines.push(`호출 정의: ${item.application.definition_name}`);
    for (const s of item.application.substitution ?? []) lines.push(`호출 대입: ${goalText(s.formal)} → ${goalText(s.actual)}`);
  }
  return lines;
}
// 판본 결속을 마친 요청 원문의 실제 byte 범위만 읽는다. 목표 타입을 해석하지 않는다.
export function sourceExcerpt(source, span, expectedBytes) {
  if (typeof source !== 'string') return null;
  const bytes = new TextEncoder().encode(source);
  if (bytes.length !== expectedBytes || !Number.isSafeInteger(span?.start)
      || !Number.isSafeInteger(span?.end) || span.start < 0 || span.end <= span.start || span.end > bytes.length) return null;
  try { return new TextDecoder('utf-8', {fatal:true, ignoreBOM:true}).decode(bytes.slice(span.start, span.end)); }
  catch { return null; }
}
export function renderGoalEvidence(root, result, onLocate, source) {
  root.replaceChildren();
  const doc = root.ownerDocument;
  const add = (parent, tag, text) => { const el = doc.createElement(tag); el.textContent = text; parent.append(el); return el; };
  if (result?.observation_only !== true || result?.unconditional_proof_issued !== false || !Array.isArray(result.definitions)) {
    add(root, 'p', '읽기 전용 목표별 관찰이 공급되지 않았습니다. 원래 결과를 확인하세요.'); return;
  }
  add(root, 'p', '가정에 의존하는 검사 관찰입니다. 무가정 증명을 발급하지 않습니다.');
  add(root, 'p', '사용 가정 칸은 실제 입력 참조이며 최소 의미 의존성의 증명은 아닙니다.');
  const statusLabels = {accepted:'수용', rejected:'거부', not_checked:'미검사'};
  add(root, 'p', '원문 전체 관찰: ' + (statusLabels[result.status] ?? '알 수 없는 상태')
    + ' · ' + ({analysis:'원문 해소 단계', checker:'근거 검사 단계'}[result.phase] ?? '단계 상세 미공급'));
  if (result.definitions.length === 0) add(root, 'p', '정의별 관찰이 공급되지 않았습니다. 전체 원문 진단을 확인하세요.');
  const counts = {accepted:0, rejected:0, not_checked:0, unknown:0};
  for (const item of result.definitions) counts[Object.hasOwn(statusLabels, item.status) ? item.status : 'unknown']++;
  add(root, 'p', `전체 ${result.definitions.length}개 정의 · 수용 ${counts.accepted} · 거부 ${counts.rejected} · 미검사 ${counts.not_checked} · 알 수 없는 상태 ${counts.unknown}`);
  const sections = [];
  let selector;
  const show = value => {
    sections.forEach((section, index) => { section.hidden = value !== 'all' && value !== String(index); });
  };
  const targetOf = item => {
    const span = item.application?.definition_span;
    if (!Number.isSafeInteger(span?.start) || !Number.isSafeInteger(span?.end) || span.start < 0 || span.end <= span.start) return -1;
    const matches = result.definitions.flatMap((candidate, index) =>
      candidate.span?.start === span.start && candidate.span?.end === span.end
        && candidate.name === item.application.definition_name ? [index] : []);
    return matches.length === 1 ? matches[0] : -1;
  };
  const callTargets = result.definitions.map(targetOf);
  const navigate = (parent, title, index) => {
    const button = add(parent, 'button', title); button.type = 'button';
    button.onclick = () => {
      if (selector) selector.value = String(index);
      show(String(index)); sections[index].querySelector('h3').focus();
    };
  };
  if (result.definitions.length > 1) {
    const label = add(root, 'label', '표시할 정의');
    selector = add(label, 'select', '');
    selector.setAttribute('aria-label', '표시할 정의');
    add(selector, 'option', '전체 정의 (공급 순서)').value = 'all';
    result.definitions.forEach((item, index) => {
      add(selector, 'option', `${index + 1}. ${item.name} — ${statusLabels[item.status] ?? '알 수 없는 상태'}`).value = String(index);
    });
    selector.onchange = () => show(selector.value);
    add(root, 'p', '표시 범위만 바꿉니다. 전체 원문의 검사 결과는 그대로이며, 숨긴 정의의 거부·미검사도 유지됩니다.');
    const firstRejected = result.definitions.findIndex(item => item.status === 'rejected');
    if (firstRejected >= 0) {
      const button = add(root, 'button', '공급 순서의 첫 거부 보기'); button.type = 'button';
      button.onclick = () => {
        selector.value = String(firstRejected); show(selector.value);
        sections[firstRejected].querySelector('h3').focus();
      };
    }
  }
  const locate = (parent, title, span) => {
    if (!span || typeof onLocate !== 'function') return;
    const button = add(parent, 'button', title); button.type = 'button';
    button.onclick = () => onLocate(span);
  };
  for (const item of result.definitions) {
    const section = add(root, 'section', '');
    sections.push(section);
    const status = statusLabels[item.status] ?? '알 수 없는 상태';
    add(section, 'h3', `${item.name} — ${status}`).tabIndex = -1;
    if (item.status !== 'accepted') add(section, 'p', '아래는 공급된 원문 근거 구조입니다. 이 정의의 근거가 수용됐다는 뜻이 아닙니다.');
    add(section, 'p', '목표: ' + goalText(item.goal));
    if (item.goal?.kind === 'opaque_goal') {
      add(section, 'p', '이 목표의 상세 구조는 공급되지 않았습니다. 아래 선언 원문과 가정 참조를 읽되, 수용을 법칙 자체의 무가정 증명이나 전달 계산 실행으로 해석하지 마세요.');
      const excerpt = sourceExcerpt(source, item.span, result.source_bytes);
      if (excerpt !== null) {
        const declaration = add(section, 'details', '');
        add(declaration, 'summary', '해석을 보충하지 않은 정의 원문');
        add(declaration, 'pre', excerpt);
      }
      if (!item.application) add(section, 'p', '이 응답에는 호출 대상 연결이 공급되지 않았습니다. 원문 이름만으로 다른 정의에 연결하지 않습니다.');
    }
    locate(section, '정의 원문', item.span);
    locate(section, '결과 근거 원문', item.result_span);
    const assumptions = Array.isArray(item.assumptions) ? item.assumptions : [];
    const references = item.used_assumption_slots;
    const knownReferences = Array.isArray(references)
      && references.every(slot => Number.isSafeInteger(slot) && slot >= 0
        && assumptions.filter(a => a.assumption_slot === slot).length === 1);
    for (const a of assumptions) {
      const uniqueSlot = Number.isSafeInteger(a.assumption_slot) && a.assumption_slot >= 0
        && assumptions.filter(other => other.assumption_slot === a.assumption_slot).length === 1;
      const reference = !knownReferences || !uniqueSlot ? '참조 정보 미공급 또는 대응 불명확'
        : references.includes(a.assumption_slot) ? '공급된 원문 근거에서 참조됨'
          : '공급된 참조 목록에 없음 — 불필요하다는 증명은 아님';
      const row = add(section, 'p', `가정: ${a.public_pin} / 본문 이름: ${a.local_name} — ${goalText(a.goal)} · ${reference}`);
      locate(row, '가정 원문', a.span);
      if (a.goal?.kind === 'opaque_goal') {
        const excerpt = sourceExcerpt(source, a.span, result.source_bytes);
        if (excerpt !== null) add(section, 'pre', '실제 가정 선언 원문\n' + excerpt);
      }
    }
    add(section, 'pre', evidenceLines(item).join('\n'));
    if (item.application) {
      locate(section, '호출 정의 원문', item.application.definition_span);
      const target = callTargets[sections.length - 1];
      if (target >= 0) navigate(section, `호출 대상의 검사 결과 보기: ${result.definitions[target].name}`, target);
      else add(section, 'p', '이 응답 안에서 호출 대상의 검사 결과를 유일하게 대응할 수 없습니다. 원문과 원래 결과를 확인하세요.');
    }
    const consumers = callTargets.flatMap((target, index) => target === sections.length - 1 ? [index] : []);
    if (consumers.length) {
      add(section, 'p', '이 응답에 공급된 직접 호출입니다. 호출 관계만으로 소비 정의의 수용이나 전체 의존 관계를 판단하지 않습니다.');
      for (const index of consumers) navigate(section, `소비 정의의 검사 결과 보기: ${result.definitions[index].name}`, index);
    }
    if (item.diagnostic) {
      add(section, 'p', item.diagnostic.formatted ?? item.diagnostic.message ?? JSON.stringify(item.diagnostic));
      if (item.status === 'rejected') locate(section, '거부된 근거 원문', item.diagnostic.span);
    }
  }
  if (result.diagnostic) {
    add(root, 'p', result.diagnostic.formatted ?? result.diagnostic.message ?? JSON.stringify(result.diagnostic));
    if (result.status === 'rejected') locate(root, '원문 전체 거부 위치', result.diagnostic.span);
  }
}


// 경로·판본 검증을 마친 한 응답 안에서만 공급된 정의 span을 연결한다.
export function algebraSourceLinks(report) {
  if (report?.observation_only !== true || report?.unconditional_proof_issued !== false || !Array.isArray(report.definitions)) return [];
  const valid = span => Number.isSafeInteger(span?.start) && Number.isSafeInteger(span?.end) && span.start >= 0 && span.end > span.start;
  const groups = report.definitions.map(def => ({name:def.name, links:[]}));
  const add = (index, label, span) => { if (valid(span)) groups[index].links.push({label,span:{start:span.start,end:span.end}}); };
  report.definitions.forEach((def,index) => {
    add(index,'정의 전체',def.span);
    (def.assumptions ?? []).forEach((a,i) => {add(index,`가정 ${i+1} 왼쪽`,a.left_span);add(index,`가정 ${i+1} 오른쪽`,a.right_span);});
    (def.steps ?? []).forEach((step,i) => {
      const status = {checked:'검사됨',failed:'첫 거부',not_checked:'미검사'}[step.status] ?? '상태 미확인';
      add(index,`단계 ${i+1} · ${status}`,step.span);
      const call = step.operand;
      if (call?.kind !== 'definition_call' || !valid(call.definition_span)) return;
      const matches = report.definitions.flatMap((candidate,j) => candidate.name === call.definition
        && candidate.span?.start === call.definition_span.start && candidate.span?.end === call.definition_span.end ? [j] : []);
      if (matches.length !== 1) return;
      add(index,`단계 ${i+1}의 제공 정의: ${call.definition}`,call.definition_span);
      add(matches[0],`소비 호출: ${def.name} · 단계 ${i+1}`,step.span);
    });
  });
  return groups;
}
export function renderAlgebraSourceNavigation(root, report, onLocate) {
  if (typeof onLocate !== 'function') return;
  const doc=root.ownerDocument, box=doc.createElement('details'), title=doc.createElement('summary');
  title.textContent='제공 정의·소비 호출·가정·단계의 원문 탐색';box.append(title);
  const note=doc.createElement('p');note.textContent='이번 응답이 공급한 호출과 위치입니다. 연결 자체가 호출 수용이나 최소 의존성의 증명은 아닙니다.';box.append(note);
  for (const group of algebraSourceLinks(report)) {
    const section=doc.createElement('section'), heading=doc.createElement('h4');heading.textContent=group.name;section.append(heading);
    for (const link of group.links) {
      const button=doc.createElement('button');button.type='button';button.textContent=link.label;
      button.addEventListener('click',()=>onLocate(link.span));section.append(button);
    }
    box.append(section);
  }
  root.prepend(box);
}
