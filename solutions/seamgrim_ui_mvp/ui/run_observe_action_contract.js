export const OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT = "open-ddn-observation-output";
const OBSERVE_ACTION_KNOWN_SET = new Set([
  OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT,
]);

export function normalizeObserveAction(raw) {
  const code = String(raw ?? "").trim().toLowerCase();
  if (!code) return "";
  return OBSERVE_ACTION_KNOWN_SET.has(code) ? code : "";
}

export function isObserveActionOpenDdnObservationOutput(raw) {
  return normalizeObserveAction(raw) === OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT;
}

export function isObserveActionSupported(raw) {
  return Boolean(normalizeObserveAction(raw));
}

export function buildObserveActionPlan(action, payload = {}) {
  const normalized = normalizeObserveAction(action);
  if (!normalized) return null;
  if (normalized === OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT) {
    return {
      kind: "open-ddn-token",
      token: String(payload?.observeToken ?? "").trim(),
      source: "observation_output_rows",
    };
  }
  return null;
}
