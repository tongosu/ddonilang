function parseJsonText(text, label) {
  try {
    return JSON.parse(String(text ?? ""));
  } catch (err) {
    throw new Error(`${label} JSON 파싱 실패: ${String(err?.message ?? err)}`);
  }
}

function buildStateDiag(code, message, detail = "") {
  return {
    code,
    message,
    detail: String(detail ?? ""),
  };
}

function requireTarget(target) {
  if (!target || typeof target !== "object") {
    throw new Error("wasm state observer target이 없습니다.");
  }
  return target;
}

function readStateHash(target) {
  const view = requireTarget(target);
  if (typeof view.getStateHash === "function") {
    return String(view.getStateHash() ?? "");
  }
  if (typeof view.get_state_hash === "function") {
    return String(view.get_state_hash() ?? "");
  }
  throw new Error("wasm state hash API가 없습니다.");
}

function readStateParsed(target) {
  const view = requireTarget(target);
  if (typeof view.getStateParsed === "function") {
    const state = view.getStateParsed();
    return state && typeof state === "object" ? state : parseJsonText(state, "state");
  }
  if (typeof view.get_state_json === "function") {
    return parseJsonText(view.get_state_json(), "state");
  }
  throw new Error("wasm state JSON API가 없습니다.");
}

export function getStateHash(target) {
  return readStateHash(target);
}

export function getStateParsed(target) {
  return readStateParsed(target);
}

export function createWasmStateObserverClient(target) {
  let lastStateDiag = null;

  function readWithDiag(kind, fn) {
    try {
      const result = fn();
      lastStateDiag = null;
      return result;
    } catch (err) {
      lastStateDiag = buildStateDiag(
        "E_WASM_STATE_OBSERVER_READ_FAILED",
        `wasm state ${kind} 읽기에 실패했습니다.`,
        err?.message ?? String(err ?? ""),
      );
      throw err;
    }
  }

  return {
    getStateHash() {
      return readWithDiag("hash", () => readStateHash(target));
    },
    getStateParsed() {
      return readWithDiag("json", () => readStateParsed(target));
    },
    getLastStateDiag() {
      return lastStateDiag ? { ...lastStateDiag } : null;
    },
  };
}
