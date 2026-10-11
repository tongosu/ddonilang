// Device events stay in the host. Only the admitted logical channel identities,
// ordered Press/Repeat/Release and explicit settled held enter the world.
export class LogicalActionInput {
  constructor({ session, channels, initialHeld, actionMap, maxEvents }) {
    if (!session || typeof session.step !== "function" || typeof session.abort !== "function") {
      throw new Error("E_LOGICAL_INPUT_SESSION: 논리 실행이 필요합니다.");
    }
    if (!Number.isSafeInteger(maxEvents) || maxEvents < 1) {
      throw new Error("E_LOGICAL_INPUT_CAPACITY: 사건 용량을 명시해야 합니다.");
    }
    const names = Object.keys(channels ?? {}).sort();
    if (!names.length || names.some(name => channels[name] !== "Control")
        || JSON.stringify(Object.keys(initialHeld ?? {}).sort()) !== JSON.stringify(names)
        || names.some(name => typeof initialHeld[name] !== "boolean")) {
      throw new Error("E_LOGICAL_INPUT_SCHEMA: 조작 채널과 초기 눌린 상태가 필요합니다.");
    }
    if (actionMap?.schema !== "ddn.input_action_map.v1" || !Array.isArray(actionMap.actions)) {
      throw new Error("E_INPUT_ACTION_MAP_INVALID: 명시 행동 사상이 필요합니다.");
    }
    this.keyToChannel = new Map();
    const mapped = new Set();
    for (const action of actionMap.actions) {
      const channel = action.action_id;
      const key = action.input_snapshot?.last_key;
      if (!names.includes(channel) || typeof key !== "string" || !key
          || this.keyToChannel.has(key) || mapped.has(channel)) {
        throw new Error("E_INPUT_ACTION_MAP_INVALID: 채널·키 사상이 모호하거나 결속과 다릅니다.");
      }
      this.keyToChannel.set(key, channel);
      mapped.add(channel);
    }
    if (mapped.size !== names.length) {
      throw new Error("E_INPUT_ACTION_MAP_INVALID: 모든 논리 채널의 사상이 필요합니다.");
    }
    this.session = session;
    this.maxEvents = maxEvents;
    this.held = { ...initialHeld };
    this.events = [];
    this.madi = 0;
    this.inFlight = false;
    this.failure = null;
    this.abortPromise = null;
    this.detach = null;
  }

  loseInput(message) {
    if (this.failure) return this.abortPromise;
    this.failure = new Error(`E_LOGICAL_INPUT_LOST: ${message}`);
    this.abortPromise = Promise.resolve().then(() => this.session.abort(message));
    // Keep the failure for callers, but avoid an unobserved browser rejection.
    this.abortPromise.catch(() => {});
    return this.abortPromise;
  }

  receive(event, released = false) {
    if (this.failure) throw this.failure;
    const channel = this.keyToChannel.get(event.key);
    if (!channel) return false;
    if (this.events.length >= this.maxEvents) {
      this.loseInput("사건 용량을 넘었습니다.");
      throw this.failure;
    }
    const repeat = !released && event.repeat === true;
    if ((released || repeat) && !this.held[channel]
        || (!released && !repeat && this.held[channel])) {
      this.loseInput("누름·반복·뗌 순서가 끊겼습니다.");
      throw this.failure;
    }
    const kind = released ? "Release" : repeat ? "Repeat" : "Press";
    this.events.push({ ordinal:this.events.length, channel, event:{kind} });
    if (!repeat) this.held[channel] = !released;
    return true;
  }

  attach(target, { controls = [] } = {}) {
    if (!target || typeof target.addEventListener !== "function" || this.detach) {
      throw new Error("E_LOGICAL_INPUT_TARGET: 입력 받을 화면을 한 번만 연결해야 합니다.");
    }
    const onEvent = released => event => {
      try { if (this.receive(event, released)) event.preventDefault(); }
      catch { /* failure is visible through step/seal; no synthetic release */ }
    };
    const down = onEvent(false), up = onEvent(true);
    const lost = () => { this.loseInput("입력 화면의 초점을 잃었습니다. 다시 시작하세요."); };
    // One explicit input region may include the main playback controls.
    // Editors, project menus and arbitrary document nodes remain outside it.
    const candidates = [...new Set([target, ...controls.filter(Boolean)])];
    // Listen once at the outermost admitted node; bubbling must not duplicate
    // a Press when an explicit view stage contains the primary canvas.
    const regions = candidates.filter(node => !candidates.some(other => other !== node && other.contains?.(node)));
    const focusOut = event => {
      // Moving among controls in this input region is not lost input.
      if (!event.relatedTarget || !regions.some(region => region === event.relatedTarget || region.contains?.(event.relatedTarget))) lost();
    };
    const window = target.ownerDocument?.defaultView;
    const document = target.ownerDocument;
    const visibility = () => { if (document.visibilityState === "hidden") lost(); };
    for (const region of regions) {
      region.addEventListener("keydown", down);
      region.addEventListener("keyup", up);
      region.addEventListener("focusout", focusOut);
    }
    window?.addEventListener("blur", lost);
    window?.addEventListener("pagehide", lost);
    document?.addEventListener("visibilitychange", visibility);
    this.detach = () => {
      for (const region of regions) {
        region.removeEventListener("keydown", down);
        region.removeEventListener("keyup", up);
        region.removeEventListener("focusout", focusOut);
      }
      window?.removeEventListener("blur", lost);
      window?.removeEventListener("pagehide", lost);
      document?.removeEventListener("visibilitychange", visibility);
      this.detach = null;
    };
    return this.detach;
  }

  async step() {
    if (this.failure) { await this.abortPromise; throw this.failure; }
    if (this.inFlight) throw new Error("E_LOGICAL_INPUT_STEP_BUSY: 한 마디가 아직 계산 중입니다.");
    this.inFlight = true;
    const input = { madi:this.madi, events:this.events, held:{...this.held} };
    this.events = [];
    try {
      const result = await this.session.step(input);
      if (this.failure) { await this.abortPromise; throw this.failure; }
      if (result?.frame?.input?.madi !== this.madi) {
        this.loseInput("실행기가 다른 마디를 반환했습니다.");
        await this.abortPromise;
        throw this.failure;
      }
      this.madi++;
      return result;
    } catch (error) {
      if (!this.failure) {
        this.loseInput(String(error?.message ?? error));
        await this.abortPromise;
      }
      throw error;
    } finally { this.inFlight = false; }
  }

  async seal() {
    if (this.failure) { await this.abortPromise; throw this.failure; }
    if (this.inFlight || this.events.length) {
      throw new Error("E_LOGICAL_INPUT_PENDING: 아직 세계에 전달하지 않은 사건이나 계산이 있습니다.");
    }
    const record = await this.session.seal();
    // A Worker seal reply does not cover input/loss that arrived while awaiting it.
    if (this.failure) { await this.abortPromise; throw this.failure; }
    if (this.inFlight || this.events.length) {
      throw new Error("E_LOGICAL_INPUT_PENDING: 아직 세계에 전달하지 않은 사건이나 계산이 있습니다.");
    }
    return record;
  }
}
