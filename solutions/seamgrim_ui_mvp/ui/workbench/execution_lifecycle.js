export const EXECUTION_LIFECYCLE_SCHEMA = "seamgrim.workbench.execution_lifecycle.v1";

// A2-D2: this is local host pacing, never a source of world dt or madi numbers.
export function alpha2HostPacing(inputState) {
  const madiHz = inputState?.alpha2_host_madi_hz;
  if (!Number.isInteger(madiHz) || madiHz !== 60 || inputState?.alpha_host_dt !== "1") {
    throw new Error("E_ALPHA2_HOST_PACING_POLICY: 실행기의 60마디/초·dt=1 계약이 없습니다");
  }
  return Object.freeze({ madiHz, intervalMs: 1000 / madiHz });
}

export function hostMadiDelay(pacing, previousStartMs, nowMs) {
  if (!Number.isFinite(pacing?.intervalMs) || pacing.intervalMs <= 0
      || !Number.isFinite(nowMs) || (previousStartMs !== null && !Number.isFinite(previousStartMs))) {
    throw new Error("E_ALPHA2_HOST_PACING_CLOCK: 화면 재생 시각이 올바르지 않습니다");
  }
  // Slow work delays the next real step. Do not skip or fabricate world steps
  // to catch up with wall time; the serial pump admits at most one next step.
  return previousStartMs === null ? pacing.intervalMs
    : Math.max(0, pacing.intervalMs - Math.max(0, nowMs - previousStartMs));
}

export const EXECUTION_STATES = Object.freeze({
  IDLE: "idle",
  PREPARING: "preparing",
  RUNNING: "running",
  PAUSING: "pausing",
  PAUSED: "paused",
  STOPPING: "stopping",
  DONE: "done",
  FAILED: "failed",
  HISTORY_READONLY: "history_readonly",
});

export const EXECUTION_EVENTS = Object.freeze({
  START: "start",
  PREPARED: "prepared",
  PREPARE_FAILED: "prepare_failed",
  PAUSE_REQUESTED: "pause_requested",
  PAUSE_COMMITTED: "pause_committed",
  RESUME: "resume",
  STEP_REQUESTED: "step_requested",
  STEP_COMMITTED: "step_committed",
  STOP_REQUESTED: "stop_requested",
  STOP_COMMITTED: "stop_committed",
  RUNTIME_DONE: "runtime_done",
  RUNTIME_FAILED: "runtime_failed",
  HISTORY_OPENED: "history_opened",
  PROJECT_SWITCHED: "project_switched",
  REOPENED: "reopened",
  RESTORED: "restored",
});

const STATUS_LABELS = Object.freeze({
  idle: "실행 전",
  preparing: "실행 준비 중",
  running: "실행 중",
  pausing: "마디 경계에서 멈추는 중",
  paused: "멈춤",
  stopping: "실행을 종료하는 중",
  done: "실행 완료",
  failed: "실행 실패",
  history_readonly: "과거 마디 읽기 전용",
});

const ACTIVE_STATES = new Set([
  EXECUTION_STATES.PREPARING,
  EXECUTION_STATES.RUNNING,
  EXECUTION_STATES.PAUSING,
  EXECUTION_STATES.PAUSED,
  EXECUTION_STATES.STOPPING,
]);

function normalizedMode(value = "unknown") {
  return value === "live" || value === "oneshot" ? value : "unknown";
}

export function createExecutionLifecycle(overrides = {}) {
  return {
    schema: EXECUTION_LIFECYCLE_SCHEMA,
    status: EXECUTION_STATES.IDLE,
    mode: "unknown",
    run_id: "",
    generation: 0,
    transition_sequence: 0,
    last_event: "initialized",
    reason: "initial",
    final_status: "",
    history_origin_status: "",
    ...overrides,
  };
}

function accept(current, event, patch = {}) {
  return {
    accepted: true,
    code: "",
    lifecycle: {
      ...current,
      ...patch,
      schema: EXECUTION_LIFECYCLE_SCHEMA,
      transition_sequence: Number(current.transition_sequence ?? 0) + 1,
      last_event: event,
    },
  };
}

function reject(current, event, code) {
  return {
    accepted: false,
    code,
    event,
    lifecycle: current,
  };
}

export function transitionExecutionLifecycle(currentInput, event, payload = {}) {
  const current = createExecutionLifecycle(currentInput ?? {});
  const status = current.status;
  const mode = normalizedMode(current.mode);

  // A3-P12: invoked only after the separate Rust candidate is verified.
  // Restoration is not a fabricated START/STEP and does not execute a madi.
  if (event === EXECUTION_EVENTS.RESTORED) {
    if (![EXECUTION_STATES.IDLE,EXECUTION_STATES.PAUSED,EXECUTION_STATES.DONE,EXECUTION_STATES.FAILED].includes(status))
      return reject(current,event,'E_EXECUTION_RESTORE_NOT_ALLOWED');
    const restoredMode=normalizedMode(payload.mode);
    if(restoredMode==='unknown' || !payload.runId)return reject(current,event,'E_EXECUTION_RESTORE_MODE_REQUIRED');
    const done=restoredMode==='oneshot' || payload.completed===true;
    return accept(current,event,{status:done?EXECUTION_STATES.DONE:EXECUTION_STATES.PAUSED,
      mode:restoredMode,run_id:String(payload.runId),generation:Number(current.generation??0)+1,
      reason:'verified_runtime_restore',final_status:done?'completed':'',history_origin_status:''});
  }

  if (event === EXECUTION_EVENTS.PROJECT_SWITCHED || event === EXECUTION_EVENTS.REOPENED) {
    return accept(current, event, {
      status: EXECUTION_STATES.IDLE,
      mode: "unknown",
      run_id: "",
      generation: Number(current.generation ?? 0) + 1,
      reason: event === EXECUTION_EVENTS.REOPENED ? "reopened" : "project_switched",
      final_status: "",
      history_origin_status: "",
    });
  }

  if (event === EXECUTION_EVENTS.START) {
    const branchStart = Boolean(payload.branch);
    const allowed = [EXECUTION_STATES.IDLE, EXECUTION_STATES.DONE, EXECUTION_STATES.FAILED].includes(status)
      || (branchStart && status === EXECUTION_STATES.HISTORY_READONLY);
    if (!allowed) return reject(current, event, "E_EXECUTION_START_NOT_ALLOWED");
    return accept(current, event, {
      status: EXECUTION_STATES.PREPARING,
      mode: "unknown",
      run_id: "",
      generation: Number(current.generation ?? 0) + 1,
      reason: branchStart ? "branch_start" : "run_start",
      final_status: "",
      history_origin_status: "",
    });
  }

  if (event === EXECUTION_EVENTS.PREPARED) {
    if (status !== EXECUTION_STATES.PREPARING) {
      return reject(current, event, "E_EXECUTION_PREPARED_OUT_OF_ORDER");
    }
    const preparedMode = normalizedMode(payload.mode);
    if (preparedMode === "unknown") {
      return reject(current, event, "E_EXECUTION_MODE_REQUIRED");
    }
    return accept(current, event, {
      status: preparedMode === "oneshot" ? EXECUTION_STATES.DONE : EXECUTION_STATES.RUNNING,
      mode: preparedMode,
      run_id: String(payload.runId ?? ""),
      reason: preparedMode === "oneshot" ? "oneshot_committed" : "live_ready",
      final_status: preparedMode === "oneshot" ? "completed" : "",
    });
  }

  if (event === EXECUTION_EVENTS.PREPARE_FAILED || event === EXECUTION_EVENTS.RUNTIME_FAILED) {
    if (status === EXECUTION_STATES.IDLE || status === EXECUTION_STATES.DONE || status === EXECUTION_STATES.FAILED) {
      return reject(current, event, "E_EXECUTION_FAILURE_OUT_OF_ORDER");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.FAILED,
      reason: String(payload.reason ?? "runtime_failed"),
      final_status: "failed",
    });
  }

  if (event === EXECUTION_EVENTS.PAUSE_REQUESTED) {
    if (status !== EXECUTION_STATES.RUNNING || mode !== "live") {
      return reject(current, event, "E_EXECUTION_PAUSE_NOT_ALLOWED");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.PAUSING,
      reason: "pause_requested",
    });
  }

  if (event === EXECUTION_EVENTS.PAUSE_COMMITTED) {
    if (status !== EXECUTION_STATES.PAUSING || mode !== "live") {
      return reject(current, event, "E_EXECUTION_PAUSE_COMMIT_OUT_OF_ORDER");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.PAUSED,
      reason: "paused_at_committed_boundary",
    });
  }

  if (event === EXECUTION_EVENTS.RESUME) {
    if (status !== EXECUTION_STATES.PAUSED || mode !== "live" || payload.atLatestCursor === false) {
      return reject(current, event, "E_EXECUTION_RESUME_NOT_ALLOWED");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.RUNNING,
      reason: "resumed_same_run",
    });
  }

  if (event === EXECUTION_EVENTS.STEP_REQUESTED || event === EXECUTION_EVENTS.STEP_COMMITTED) {
    if (status !== EXECUTION_STATES.PAUSED || mode !== "live" || payload.atLatestCursor === false) {
      return reject(current, event, "E_EXECUTION_STEP_NOT_ALLOWED");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.PAUSED,
      reason: event === EXECUTION_EVENTS.STEP_REQUESTED ? "manual_step_requested" : "manual_step_committed",
    });
  }

  if (event === EXECUTION_EVENTS.STOP_REQUESTED) {
    if (!ACTIVE_STATES.has(status) || status === EXECUTION_STATES.STOPPING) {
      return reject(current, event, "E_EXECUTION_STOP_NOT_ALLOWED");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.STOPPING,
      generation: Number(current.generation ?? 0) + 1,
      reason: "stop_requested",
    });
  }

  if (event === EXECUTION_EVENTS.STOP_COMMITTED) {
    if (status !== EXECUTION_STATES.STOPPING) {
      return reject(current, event, "E_EXECUTION_STOP_COMMIT_OUT_OF_ORDER");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.DONE,
      reason: "stopped",
      final_status: String(payload.finalStatus ?? "stopped"),
    });
  }

  if (event === EXECUTION_EVENTS.RUNTIME_DONE) {
    if (![EXECUTION_STATES.RUNNING, EXECUTION_STATES.PAUSING, EXECUTION_STATES.PAUSED].includes(status)) {
      return reject(current, event, "E_EXECUTION_DONE_OUT_OF_ORDER");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.DONE,
      reason: String(payload.reason ?? "runtime_done"),
      final_status: "completed",
    });
  }

  if (event === EXECUTION_EVENTS.HISTORY_OPENED) {
    if (![EXECUTION_STATES.RUNNING, EXECUTION_STATES.PAUSED, EXECUTION_STATES.DONE, EXECUTION_STATES.FAILED, EXECUTION_STATES.HISTORY_READONLY].includes(status)) {
      return reject(current, event, "E_EXECUTION_HISTORY_NOT_ALLOWED");
    }
    return accept(current, event, {
      status: EXECUTION_STATES.HISTORY_READONLY,
      generation: Number(current.generation ?? 0) + 1,
      reason: "history_cursor",
      history_origin_status: status === EXECUTION_STATES.HISTORY_READONLY
        ? current.history_origin_status
        : status,
    });
  }

  return reject(current, event, "E_EXECUTION_EVENT_UNKNOWN");
}

export function executionControlPolicy(lifecycleInput, context = {}) {
  const lifecycle = createExecutionLifecycle(lifecycleInput ?? {});
  const status = lifecycle.status;
  const mode = normalizedMode(lifecycle.mode);
  const hasFrame = Boolean(context.hasFrame);
  const frameCount = Math.max(0, Number(context.frameCount ?? (hasFrame ? 1 : 0)));
  const cursorIndex = Number(context.cursorIndex ?? -1);
  const atLatestCursor = hasFrame && cursorIndex === frameCount - 1;
  const hasEarlierFrame = hasFrame && cursorIndex > 0;

  return {
    canStart: [EXECUTION_STATES.IDLE, EXECUTION_STATES.DONE, EXECUTION_STATES.FAILED].includes(status),
    canPause: status === EXECUTION_STATES.RUNNING && mode === "live",
    canResume: status === EXECUTION_STATES.PAUSED && mode === "live" && atLatestCursor,
    canStep: status === EXECUTION_STATES.PAUSED && mode === "live" && atLatestCursor,
    canBack: hasEarlierFrame && [
      EXECUTION_STATES.RUNNING,
      EXECUTION_STATES.PAUSED,
      EXECUTION_STATES.DONE,
      EXECUTION_STATES.FAILED,
      EXECUTION_STATES.HISTORY_READONLY,
    ].includes(status),
    canBranch: hasFrame && (
      status === EXECUTION_STATES.HISTORY_READONLY
      || status === EXECUTION_STATES.DONE
      || status === EXECUTION_STATES.FAILED
      || (status === EXECUTION_STATES.PAUSED && !atLatestCursor)
    ),
    canStop: [
      EXECUTION_STATES.PREPARING,
      EXECUTION_STATES.RUNNING,
      EXECUTION_STATES.PAUSING,
      EXECUTION_STATES.PAUSED,
    ].includes(status),
    atLatestCursor,
    status,
    mode,
  };
}

export function executionStatusLabel(lifecycleInput) {
  const lifecycle = createExecutionLifecycle(lifecycleInput ?? {});
  const base = lifecycle.status === EXECUTION_STATES.DONE && lifecycle.final_status === "stopped"
    ? "실행 중지" : STATUS_LABELS[lifecycle.status] ?? lifecycle.status;
  const mode = normalizedMode(lifecycle.mode);
  if (mode === "unknown") return base;
  return `${base} · ${mode === "live" ? "연속 실행" : "한 번 실행"}`;
}

export function claimRunFinalization(run, finalStatus = "stopped") {
  if (!run || typeof run !== "object") return { claimed: false, reason: "run_missing" };
  if (run.server_finalized) return { claimed: false, reason: "already_finalized" };
  if (run.server_finalization_started) return { claimed: false, reason: "already_claimed" };
  run.server_finalization_started = true;
  run.server_finalize_attempt_count = Number(run.server_finalize_attempt_count ?? 0) + 1;
  run.final_status = String(finalStatus || "stopped");
  return { claimed: true, reason: "claimed" };
}

export function legacyRuntimeStatus(lifecycleInput) {
  const lifecycle = createExecutionLifecycle(lifecycleInput ?? {});
  if (lifecycle.status === EXECUTION_STATES.HISTORY_READONLY) return "history-readonly";
  if (lifecycle.status === EXECUTION_STATES.DONE && lifecycle.final_status === "stopped") return "stopped";
  if (lifecycle.status === EXECUTION_STATES.FAILED) {
    if (lifecycle.reason === "source_empty") return "source-error";
    if (lifecycle.reason === "stale_source") return "stale-source";
    return "error";
  }
  if (lifecycle.status === EXECUTION_STATES.IDLE && lifecycle.reason === "reopened") return "reopened";
  return lifecycle.status;
}
