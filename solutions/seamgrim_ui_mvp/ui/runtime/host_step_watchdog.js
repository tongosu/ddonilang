// Wall time is a Workbench host control only, never a VM result or error.
export const HOST_STEP_STOP_MS = 10_000;
export const HOST_STEP_STOP_NOTICE = "10초가 지나도 끝나지 않아 멈췄습니다. 반복이 끝나는 조건이 있는지 확인해 보세요.";

export async function runHostStep(step, stop, { schedule = setTimeout, clear = clearTimeout } = {}) {
  const timer = schedule(stop, HOST_STEP_STOP_MS);
  try { return await step(); }
  finally { clear(timer); }
}
