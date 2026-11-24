// A source inspection report, never a proof handle or persisted execution value.
export function installProofSourceInspection({root = document, runtimeBridge, readSource}) {
  const button = root.querySelector('[data-testid="proof-source-check"]');
  const panel = root.querySelector('[data-testid="proof-source-result"]');
  const status = root.querySelector('[data-testid="proof-source-status"]');
  const detail = root.querySelector('[data-testid="proof-source-detail"]');
  if (!button || !panel || !status || !detail) throw new Error('보임 검사 표시 영역이 없습니다.');
  let generation = 0;
  let observed = null;
  let presenceSource = null;
  const same = (a, b) => a !== null && b !== null && a.projectId === b.projectId
    && a.path === b.path && a.sourceText === b.sourceText && a.profile === b.profile;

  function invalidate() {
    generation += 1;
    observed = null;
    presenceSource = null;
    button.hidden = true;
    button.dataset.presence = 'unknown';
    button.disabled = false;
    panel.hidden = true;
    panel.dataset.proofStatus = 'not-checked';
    status.textContent = '';
    detail.textContent = '';
  }

  async function observe() {
    const snapshot = {...readSource()};
    if (observed && !same(observed, snapshot)) invalidate();
    if (same(presenceSource, snapshot)) return;
    invalidate();
    presenceSource = snapshot;
    const ticket = generation;
    // Ordinary shared parsing only: never auto-run the mathematical checker.
    try {
      const result = await runtimeBridge.sourceAnalysis({source_text:snapshot.sourceText,
        path:snapshot.path, source_profile_identity:snapshot.profile, analysis_generation:ticket});
      if (ticket !== generation || !same(snapshot, readSource())) return;
      const valid = result?.schema === 'ddn.source_analysis_snapshot.candidate.v1'
        && result.analysis_generation === ticket && result.source_profile_identity === snapshot.profile
        && typeof result.source_revision_identity === 'string' && result.source_revision_identity.length > 0
        && typeof result.proof_source_present === 'boolean';
      button.hidden = !valid || !result.proof_source_present;
      button.dataset.presence = valid ? (result.proof_source_present ? 'present' : 'absent') : 'unknown';
    } catch {
      if (ticket !== generation || !same(snapshot, readSource())) return;
      button.hidden = true;
      button.dataset.presence = 'unknown';
      // Existing source analysis owns diagnostics; failed parsing is not proof absence.
    }
  }

  async function check() {
    if (button.hidden) return;
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
