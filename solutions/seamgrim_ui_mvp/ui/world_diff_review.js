function asObject(value) {
  return value && typeof value === "object" && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function asText(value, fallback = "") {
  const text = String(value ?? "").trim();
  return text || fallback;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function buildReviewEvent(payload, change, decision, appliedToSession) {
  return {
    schema: "ddn.world_diff.review_event.v1",
    artifact_schema: "ddn.world_diff.v1",
    world_id: asText(payload.world_id),
    candidate_hash: asText(payload?.candidate?.canonical_source_hash),
    change_id: asText(change.id),
    decision,
    human_confirmation_event: true,
    applied_to_session: Boolean(appliedToSession),
    file_write: false,
    authority_after_decision: "local_session_only",
  };
}

export function renderWorldDiffReview(root, rawArtifact, {
  onApprove = null,
  onReject = null,
  onDecision = null,
} = {}) {
  if (!root) return null;
  const payload = asObject(rawArtifact);
  const changes = asArray(payload.changes);
  const change = asObject(changes[0]);
  const review = asObject(payload.review);
  const valid = payload.schema === "ddn.world_diff.v1"
    && payload.authority === "none"
    && review.requires_human_confirmation === true
    && review.auto_apply === false
    && changes.length > 0;
  root.classList.toggle("hidden", !valid);
  if (!valid) {
    root.replaceChildren();
    delete root.dataset.worldDiffStatus;
    delete root.dataset.worldDiffDecision;
    return null;
  }

  const before = asText(change?.before?.canon, "-");
  const after = asText(change?.after?.canon, "-");
  const execution = asObject(payload.execution_preview);
  const runSummary = execution.madi && execution.state_hash_equal === false
    ? `같은 ${Number(execution.madi)}마디에서 실행 결과가 달라집니다.`
    : "실행 결과 차이는 아직 계산하지 않았습니다.";
  root.dataset.worldDiffStatus = asText(payload.status, "candidate_preview");
  root.dataset.worldDiffAuthority = "none";
  root.dataset.worldDiffDecision = "pending";
  root.innerHTML = `
    <div class="run-world-diff-head">
      <div>
        <span class="run-world-diff-kicker">슬기 제안 검토</span>
        <strong>뜻으로 바뀐 점</strong>
      </div>
      <span class="run-world-diff-authority">후보 · 권한 없음</span>
    </div>
    <p class="run-world-diff-summary">${escapeHtml(change.korean ?? "")}</p>
    <div class="run-world-diff-change" data-testid="world-diff-change">
      <div>
        <span>현재</span>
        <strong data-world-diff-before>${escapeHtml(before)}</strong>
      </div>
      <span class="run-world-diff-arrow" aria-hidden="true">→</span>
      <div>
        <span>후보</span>
        <strong data-world-diff-after>${escapeHtml(after)}</strong>
      </div>
      <span class="run-world-diff-subject">${escapeHtml(change.subject ?? "")}</span>
    </div>
    <p class="run-world-diff-execution">${escapeHtml(runSummary)}</p>
    <p class="run-world-diff-boundary" data-testid="world-diff-boundary">아직 세계에 적용되지 않았습니다. 적용과 버리기는 사람이 결정합니다.</p>
    <div class="run-world-diff-actions">
      <button type="button" class="btn-primary" data-testid="world-diff-approve" title="이 후보를 현재 실행에 적용">✓ 적용</button>
      <button type="button" class="ghost" data-testid="world-diff-reject" title="이 후보를 버림">× 버리기</button>
      <span data-testid="world-diff-decision">결정 대기</span>
    </div>
  `;

  const approveButton = root.querySelector("[data-testid='world-diff-approve']");
  const rejectButton = root.querySelector("[data-testid='world-diff-reject']");
  const decisionLabel = root.querySelector("[data-testid='world-diff-decision']");
  const finish = (decision, applied) => {
    const event = buildReviewEvent(payload, change, decision, applied);
    root.dataset.worldDiffDecision = decision;
    if (decisionLabel) decisionLabel.textContent = decision === "approved" ? "사람이 적용함" : "사람이 버림";
    if (approveButton) approveButton.disabled = true;
    if (rejectButton) rejectButton.disabled = true;
    try {
      window.__SEAMGRIM_WORLD_DIFF_REVIEW_EVENT__ = event;
      root.dispatchEvent(new CustomEvent("seamgrim:world-diff-review", { bubbles: true, detail: event }));
    } catch (_) {
      // Browser instrumentation is optional.
    }
    if (typeof onDecision === "function") onDecision(event);
    return event;
  };

  approveButton?.addEventListener("click", async () => {
    approveButton.disabled = true;
    const result = typeof onApprove === "function" ? await onApprove(change, payload) : false;
    const applied = result === true || result?.applied === true;
    if (!applied) {
      approveButton.disabled = false;
      if (decisionLabel) decisionLabel.textContent = "적용하지 못함";
      return;
    }
    finish("approved", true);
  });
  rejectButton?.addEventListener("click", async () => {
    rejectButton.disabled = true;
    if (typeof onReject === "function") await onReject(change, payload);
    finish("rejected", false);
  });
  try {
    window.__SEAMGRIM_WORLD_DIFF__ = payload;
  } catch (_) {
    // Browser instrumentation is optional.
  }
  return payload;
}

