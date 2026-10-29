// A source inspection report, never a proof handle or persisted execution value.
export function installProofSourceInspection({root = document, runtimeBridge, readSource}) {
  const button = root.querySelector('[data-testid="proof-source-check"]');
  const panel = root.querySelector('[data-testid="proof-source-result"]');
  const status = root.querySelector('[data-testid="proof-source-status"]');
  const detail = root.querySelector('[data-testid="proof-source-detail"]');
  if (!button || !panel || !status || !detail) throw new Error('보임 검사 표시 영역이 없습니다.');
  let generation = 0;
  let observed = null;
  const same = (a, b) => a !== null && b !== null && a.projectId === b.projectId
    && a.path === b.path && a.sourceText === b.sourceText;

  function invalidate() {
    generation += 1;
    observed = null;
    button.disabled = false;
    panel.hidden = true;
    panel.dataset.proofStatus = 'not-checked';
    status.textContent = '';
    detail.textContent = '';
  }

  function observe() {
    if (observed && !same(observed, readSource())) invalidate();
  }

  async function check() {
    const snapshot = {...readSource()};
    const ticket = ++generation;
    observed = snapshot;
    button.disabled = true;
    panel.hidden = false;
    panel.dataset.proofStatus = 'checking';
    status.textContent = '현재 파일의 보임 원문을 검사하는 중입니다.';
    detail.textContent = '';
    try {
      const result = await runtimeBridge.verifyAssumptionSource(snapshot.sourceText, snapshot.path);
      if (ticket !== generation || !same(snapshot, readSource())) return;
      // This is the existing checker's report, not a certificate to consume.
      if (result?.assumption_dependent !== true || result?.unconditional_proof_issued !== false
          || !Number.isSafeInteger(result?.conditional_definition_count)
          || result.conditional_definition_count < 0 || typeof result?.message !== 'string') {
        throw new Error('보임 검사 결과 형식을 확인할 수 없습니다. 성공으로 표시하지 않습니다.');
      }
      if (result.conditional_definition_count === 0) {
        panel.dataset.proofStatus = 'no-definitions';
        status.textContent = '검사할 조건부 정의가 없습니다. 보임을 증명한 것으로 표시하지 않습니다.';
      } else {
        panel.dataset.proofStatus = 'conditional-accepted';
        status.textContent = `가정에 의존하는 정의 ${result.conditional_definition_count}개를 검사했습니다. 무가정 증명 발급이 아닙니다.`;
      }
      detail.textContent = result.message;
    } catch (error) {
      if (ticket !== generation || !same(snapshot, readSource())) return;
      panel.dataset.proofStatus = 'rejected';
      status.textContent = '현재 파일의 보임 검사를 거부했습니다. 원문과 가정을 확인하세요.';
      detail.textContent = String(error?.message ?? error);
    } finally {
      if (ticket === generation) button.disabled = false;
    }
  }

  button.addEventListener('click', () => { void check(); });
  root.querySelector('[data-testid="source-editor"]').addEventListener('input', invalidate, {capture: true});
  invalidate();
  return Object.freeze({observe, invalidate});
}
