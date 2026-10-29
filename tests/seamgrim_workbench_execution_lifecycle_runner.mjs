#!/usr/bin/env node

import {
  EXECUTION_EVENTS,
  EXECUTION_STATES,
  claimRunFinalization,
  createExecutionLifecycle,
  executionControlPolicy,
  alpha2HostPacing,
  hostMadiDelay,
  transitionExecutionLifecycle,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/execution_lifecycle.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const pacing = alpha2HostPacing({ alpha2_host_madi_hz: 60, alpha_host_dt: "1" });
assert(pacing.madiHz === 60 && pacing.intervalMs === 1000 / 60, "Rust host policy must pace at 60Hz");
assert(hostMadiDelay(pacing, 100, 105) === 1000 / 60 - 5, "step work time must not be added twice");
assert(hostMadiDelay(pacing, 100, 140) === 0, "slow work must schedule one next step without fake catch-up");
assert(hostMadiDelay(pacing, null, 10000) === 1000 / 60, "resume must not catch up time spent paused");
assert(hostMadiDelay(pacing, 100, 90) === 1000 / 60, "backwards clock must not accelerate world steps");
for (const input of [{}, { alpha2_host_madi_hz: "60", alpha_host_dt: "1" },
  { alpha2_host_madi_hz: 60, alpha_host_dt: "0.015625" }, { alpha2_host_madi_hz: 64, alpha_host_dt: "1" }]) {
  let rejected = false;
  try { alpha2HostPacing(input); } catch (err) { rejected = err.message.startsWith("E_ALPHA2_HOST_PACING_POLICY:"); }
  assert(rejected, "missing or conflicting Rust host policy must not get a JS default");
}

function transition(lifecycle, event, payload = {}) {
  const result = transitionExecutionLifecycle(lifecycle, event, payload);
  assert(result.accepted, `${event} rejected: ${result.code}`);
  return result.lifecycle;
}

function rejected(lifecycle, event, code, payload = {}) {
  const result = transitionExecutionLifecycle(lifecycle, event, payload);
  assert(!result.accepted, `${event} unexpectedly accepted`);
  assert(result.code === code, `${event} rejection mismatch: ${result.code}`);
}

function policy(lifecycle, frameCount = 0, cursorIndex = frameCount - 1) {
  return executionControlPolicy(lifecycle, {
    hasFrame: frameCount > 0,
    frameCount,
    cursorIndex,
  });
}

let lifecycle = createExecutionLifecycle();
let controls = policy(lifecycle);
assert(lifecycle.status === EXECUTION_STATES.IDLE, "initial state must be idle");
assert(controls.canStart && !controls.canPause && !controls.canStep && !controls.canStop, "idle controls mismatch");

lifecycle = transition(lifecycle, EXECUTION_EVENTS.START);
const preparingGeneration = lifecycle.generation;
assert(lifecycle.status === EXECUTION_STATES.PREPARING, "start must enter preparing");
rejected(lifecycle, EXECUTION_EVENTS.START, "E_EXECUTION_START_NOT_ALLOWED");
controls = policy(lifecycle);
assert(!controls.canStart && controls.canStop, "preparing controls mismatch");

const switched = transition(lifecycle, EXECUTION_EVENTS.PROJECT_SWITCHED);
assert(switched.status === EXECUTION_STATES.IDLE, "project switch must return idle");
assert(switched.generation > preparingGeneration, "project switch must invalidate preparing generation");
rejected(switched, EXECUTION_EVENTS.PREPARED, "E_EXECUTION_PREPARED_OUT_OF_ORDER", { mode: "live", runId: "stale" });

lifecycle = transition(switched, EXECUTION_EVENTS.START);
lifecycle = transition(lifecycle, EXECUTION_EVENTS.PREPARED, { mode: "live", runId: "run_live" });
controls = policy(lifecycle, 2, 1);
assert(lifecycle.status === EXECUTION_STATES.RUNNING && lifecycle.mode === "live", "live preparation mismatch");
assert(controls.canPause && controls.canStop && !controls.canResume && !controls.canStep, "live/running controls mismatch");

lifecycle = transition(lifecycle, EXECUTION_EVENTS.PAUSE_REQUESTED);
assert(lifecycle.status === EXECUTION_STATES.PAUSING, "pause request must enter pausing");
lifecycle = transition(lifecycle, EXECUTION_EVENTS.PAUSE_COMMITTED);
controls = policy(lifecycle, 2, 1);
assert(lifecycle.status === EXECUTION_STATES.PAUSED, "pause boundary must enter paused");
assert(controls.canResume && controls.canStep && controls.canStop, "live/paused controls mismatch");

lifecycle = transition(lifecycle, EXECUTION_EVENTS.STEP_REQUESTED, { atLatestCursor: true });
lifecycle = transition(lifecycle, EXECUTION_EVENTS.STEP_COMMITTED, { atLatestCursor: true });
assert(lifecycle.status === EXECUTION_STATES.PAUSED, "manual step must remain paused");
lifecycle = transition(lifecycle, EXECUTION_EVENTS.RESUME, { atLatestCursor: true });
assert(lifecycle.status === EXECUTION_STATES.RUNNING && lifecycle.run_id === "run_live", "resume must keep same run");

const runningGeneration = lifecycle.generation;
lifecycle = transition(lifecycle, EXECUTION_EVENTS.HISTORY_OPENED);
controls = policy(lifecycle, 3, 1);
assert(lifecycle.status === EXECUTION_STATES.HISTORY_READONLY, "past cursor must be read-only");
assert(lifecycle.generation > runningGeneration, "history cursor must invalidate active callbacks");
assert(controls.canBranch && !controls.canStart && !controls.canResume && !controls.canStep, "history controls mismatch");
lifecycle = transition(lifecycle, EXECUTION_EVENTS.START, { branch: true });
assert(lifecycle.status === EXECUTION_STATES.PREPARING, "branch must create a new preparing run");

lifecycle = transition(lifecycle, EXECUTION_EVENTS.PREPARED, { mode: "oneshot", runId: "run_child" });
controls = policy(lifecycle, 1, 0);
assert(lifecycle.status === EXECUTION_STATES.DONE && lifecycle.final_status === "completed", "oneshot must finish immediately");
assert(controls.canStart && controls.canBranch, "oneshot/done restart and branch controls mismatch");
assert(!controls.canPause && !controls.canResume && !controls.canStep && !controls.canStop, "oneshot/done active controls must be disabled");

lifecycle = transition(lifecycle, EXECUTION_EVENTS.START);
lifecycle = transition(lifecycle, EXECUTION_EVENTS.PREPARE_FAILED, { reason: "test_failure" });
controls = policy(lifecycle);
assert(lifecycle.status === EXECUTION_STATES.FAILED && controls.canStart, "failed run must allow a fresh start");

lifecycle = transition(lifecycle, EXECUTION_EVENTS.START);
lifecycle = transition(lifecycle, EXECUTION_EVENTS.PREPARED, { mode: "live", runId: "run_stop" });
lifecycle = transition(lifecycle, EXECUTION_EVENTS.STOP_REQUESTED);
assert(lifecycle.status === EXECUTION_STATES.STOPPING, "stop request must enter stopping");
lifecycle = transition(lifecycle, EXECUTION_EVENTS.STOP_COMMITTED, { finalStatus: "stopped" });
assert(lifecycle.status === EXECUTION_STATES.DONE && lifecycle.final_status === "stopped", "stop commit mismatch");
rejected(lifecycle, EXECUTION_EVENTS.STOP_REQUESTED, "E_EXECUTION_STOP_NOT_ALLOWED");

const failedArchive = { server_run_id: "srv_failed" };
const firstFinalizeClaim = claimRunFinalization(failedArchive, "failed");
const duplicateFinalizeClaim = claimRunFinalization(failedArchive, "failed");
assert(firstFinalizeClaim.claimed, "failed archive finalization was not claimed");
assert(!duplicateFinalizeClaim.claimed, "failed archive finalization was claimed twice");
assert(failedArchive.server_finalize_attempt_count === 1, "failed archive finalization attempt count must be exactly one");

// A3-P12: verified restore admits no fictitious initialization/step.
const restoredLive=transition(createExecutionLifecycle(),EXECUTION_EVENTS.RESTORED,{mode:'live',runId:'saved-run'});
assert(restoredLive.status===EXECUTION_STATES.PAUSED && restoredLive.run_id==='saved-run','cold restore must publish paused actual run');
assert(restoredLive.generation===1,'restore invalidates earlier async execution replies');
const resumedRestore=transition(restoredLive,EXECUTION_EVENTS.RESUME,{atLatestCursor:true});
assert(resumedRestore.status===EXECUTION_STATES.RUNNING,'restored live run can resume');
rejected(resumedRestore,EXECUTION_EVENTS.RESTORED,'E_EXECUTION_RESTORE_NOT_ALLOWED');
const restoredDone=transition(createExecutionLifecycle(),EXECUTION_EVENTS.RESTORED,{mode:'oneshot',runId:'saved-once'});
assert(restoredDone.status===EXECUTION_STATES.DONE,'one-shot restore remains completed');
rejected(createExecutionLifecycle(),EXECUTION_EVENTS.RESTORED,'E_EXECUTION_RESTORE_MODE_REQUIRED');
console.log("seamgrim_workbench_execution_lifecycle: ok");
