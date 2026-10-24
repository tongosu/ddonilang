// Wall time is a Workbench host control only, never a VM result or error.
export const HOST_STEP_STOP_MS = 10_000;
export const NATIVE_HOST_STEP_STOP_MS = 180_000;
export const HOST_STEP_STOP_NOTICE = "10초가 지나도 끝나지 않아 멈췄습니다. 반복이 끝나는 조건이 있는지 확인해 보세요.";
export const NATIVE_HOST_STEP_STOP_NOTICE = "실행 프로그램의 응답을 3분 동안 확인하지 못해 중지를 요청했습니다. 저장 성공으로 처리하지 않았습니다. 기존 저장본과 현재 편집을 보존하고 실행 프로그램을 확인하세요.";

export async function runHostStep(step, stop, { schedule = setTimeout, clear = clearTimeout, nativeSourceBound = false } = {}) {
  // A protected Native VM step includes actual custody/readback I/O. Its
  // bounded transport wait is not the pure-WASM loop budget. This flag only
  // selects a local host timer; it issues no VM/storage/replay permission.
  const native = nativeSourceBound === true;
  const timer = schedule(() => stop(native ? NATIVE_HOST_STEP_STOP_NOTICE : HOST_STEP_STOP_NOTICE),
    native ? NATIVE_HOST_STEP_STOP_MS : HOST_STEP_STOP_MS);
  try { return await step(); }
  finally { clear(timer); }
}
