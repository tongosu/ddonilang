// Workbench host policy: never display an arithmetic-fault frame as a
// successful simulation. The language's signal and CLI contracts are unchanged.
export function requireSafeWorkbenchResult(result) {
  const payload = typeof result === "string" ? JSON.parse(result) : result;
  const patches = [payload?.patch, payload?.engine_state?.patch, payload?.state?.patch];
  if (patches.some(patch => Array.isArray(patch) && patch.some(op =>
    op?.op === "emit_signal" && op.signal === "산술고장"))) {
    const error = new Error("산술고장이 발생해 실행을 멈췄습니다. 계산식을 확인해 주세요.");
    error.code = "E_WORKBENCH_ARITHMETIC_FAULT";
    throw error;
  }
  return result;
}
