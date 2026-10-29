// Device events stay in the host. Only the admitted logical channel identities,
// ordered Press/Repeat/Release and explicit settled held enter the world.
export class LogicalActionInput {
  constructor({ session, channels, initialHeld, actionMap, maxEvents, valueChannels = null, nextMadi = 0, finishBarrier = null }) {
    if (!session || typeof session.step !== "function" || typeof session.abort !== "function") {
      throw new Error("E_LOGICAL_INPUT_SESSION: 논리 실행이 필요합니다.");
    }
    if (!Number.isSafeInteger(maxEvents) || maxEvents < 1) {
      throw new Error("E_LOGICAL_INPUT_CAPACITY: 사건 용량을 명시해야 합니다.");
    }
    if(!Number.isSafeInteger(nextMadi)||nextMadi<0)
      throw new Error('E_LOGICAL_INPUT_POSITION: 확인된 다음 마디가 필요합니다.');
    const names = Object.keys(channels ?? {}).sort();
    if ((!names.length && !Object.keys(valueChannels ?? {}).length) || names.some(name => channels[name] !== "Control")
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
    this.finishBarrier = finishBarrier;
    this.valueChannels = valueChannels;
    this.maxEvents = maxEvents;
    this.held = { ...initialHeld };
    this.events = [];
    this.madi = nextMadi;
    this.inFlight = false;
    this.failure = null;
    this.abortPromise = null;
    this.detach = null;
    this.stopped = false;
    this.stopping = false;
    this.producedEvents = 0n;
    this.processedEvents = 0n;
    this.inFlightDone = null;
    this.finishedRecord = null;
    this.finishPromise = null;
    this.finishNotice = null;
    this.finishedPacket = null;
    this.finishUncertain = false;
    this.nativeStopStored = false;
    this.nativeStopFailure = null;
    this.failedInput = null;
    // The actual host queue registers before its first submitted step. An
    // old restored VM may still run, but missing replay progress cannot finish.
    this.producerReady = null;
    if (typeof session.registerInputProducer === 'function') {
      try { this.producerReady = Promise.resolve(session.registerInputProducer()).then(registration => {
        if (typeof registration?.accepted_event_count !== 'string'
            || !/^(0|[1-9][0-9]*)$/.test(registration.accepted_event_count))
          throw new Error('E_MISSING_PENDING_SUPPLY: 처리된 원 입력의 위치가 필요합니다.');
        const original = BigInt(registration.accepted_event_count);
        this.processedEvents += original; this.producedEvents += original;
        return registration;
      }); }
      catch (error) { this.producerReady = Promise.reject(error); }
    }
    this.producerReady?.catch(() => {});
  }

  loseInput(message) {
    if (this.stopped || this.stopping || this.finishUncertain) return this.abortPromise;
    if (this.failure) return this.abortPromise;
    this.failure = new Error(`E_LOGICAL_INPUT_LOST: ${message}`);
    this.abortPromise = Promise.resolve().then(() => this.session.abort(message));
    // Keep the failure for callers, but avoid an unobserved browser rejection.
    this.abortPromise.catch(() => {});
    return this.abortPromise;
  }

  receive(event, released = false) {
    if (this.stopped || this.stopping || this.finishUncertain) throw new Error('E_INPUT_RECORDING_STOPPED: 기록이 끝났거나 종료 확인이 필요합니다.');
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
    this.producedEvents++;
    if (!repeat) this.held[channel] = !released;
    return true;
  }

  // Provisional typed host intake. Exact raw strings are produced/validated by
  // Rust. Never hide a coordinate in an action name or write world resources.
  receiveValue(channel, payload) {
    if (this.stopped || this.stopping || this.finishUncertain) throw new Error('E_INPUT_RECORDING_STOPPED: 기록이 끝났거나 종료 확인이 필요합니다.');
    if (this.failure) throw this.failure;
    const contract=this.valueChannels?.[channel];
    if (!contract || !['Number','Coordinate'].includes(contract.kind)
        || !['Set','Begin','Move','End'].includes(payload?.phase)) {
      this.loseInput('E_MANIPULATION_TYPE: 선언된 조작 채널과 단계가 필요합니다.');throw this.failure;
    }
    const raw=contract.kind==='Number'?[payload.raw]:payload.raw;
    if(!Array.isArray(raw)||raw.length!==(contract.kind==='Number'?1:2)
        ||raw.some(value=>typeof value!=='string'||! /^(?:0|-?[1-9][0-9]*)$/.test(value))) {
      this.loseInput('E_MANIPULATION_VALUE: 손실 없는 raw 수 표기가 필요합니다.');throw this.failure;
    }
    if(this.events.length>=this.maxEvents){this.loseInput('사건 용량을 넘었습니다.');throw this.failure;}
    this.events.push({ordinal:this.events.length,channel,event:{kind:contract.kind,payload:{raw:contract.kind==='Number'?raw[0]:[...raw],phase:payload.phase}}});
    this.producedEvents++;
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
    if (this.stopped || this.stopping || this.finishUncertain) throw new Error('E_INPUT_RECORDING_STOPPED: 기록이 끝났거나 종료 확인이 필요합니다.');
    if (this.failure) { await this.abortPromise; throw this.failure; }
    if (this.inFlight) throw new Error("E_LOGICAL_INPUT_STEP_BUSY: 한 마디가 아직 계산 중입니다.");
    this.inFlight = true;
    let completeStep;
    this.inFlightDone = new Promise(resolve => { completeStep = resolve; });
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
      this.processedEvents += BigInt(input.events.length);
      return result;
    } catch (error) {
      if (!this.failure) {
        // Retain only the actual submitted attempt. Focus/input loss or a
        // host-supplied outcome cannot manufacture a Native VM failure.
        this.failedInput = structuredClone(input);
        this.loseInput(String(error?.message ?? error));
        await this.abortPromise;
      }
      throw error;
    } finally { this.inFlight = false; completeStep(); this.inFlightDone = null; }
  }

  async seal() {
    if (this.finishedRecord !== null) {
      // Revalidate the actual worker generation, rather than turning a stale
      // host cache into a successful owner response after worker replacement.
      if(this.finishBarrier)await this.finishBarrier(structuredClone(this.finishedPacket),structuredClone(this.finishedRecord),await this.session.resultIdentity());
      const record = await this.session.finishPending(structuredClone(this.finishedPacket));
      const notice = await this.session.finishNotice();
      if (notice?.excluded_input_count !== this.finishNotice.excluded_input_count)
        throw new Error('E_INPUT_FINISH_NOTICE: 원래 종료 확인과 다릅니다.');
      return record;
    }
    if (this.finishPromise !== null) return this.finishPromise;
    if (this.producerReady !== null) {
      if (this.stopped || this.stopping) throw new Error('E_INPUT_RECORDING_STOPPED: 중단 기록을 완료 봉인으로 바꿀 수 없습니다.');
      this.stopping = true; // fence new intake before awaiting an older step
      this.finishPromise = (async () => {
        let finishInvoked=false,gotComplete=false;
        try {
          if (this.inFlightDone) await this.inFlightDone;
          if (this.failure) { await this.abortPromise; throw this.failure; }
          const registration = await this.producerReady;
          if (typeof registration?.producer_generation !== 'string'
              || typeof this.session.finishPending !== 'function'
              || typeof this.session.finishNotice !== 'function') {
            throw new Error('E_MISSING_PENDING_SUPPLY: 실제 입력 생산자 연결이 필요합니다.');
          }
          const events = structuredClone(this.events);
          if (this.processedEvents + BigInt(events.length) !== this.producedEvents)
            throw new Error('E_MISSING_PENDING_SUPPLY: 처리 입력과 queue 순서가 다릅니다.');
          const packet = {
            producer_generation: registration.producer_generation,
            first_unprocessed: this.processedEvents.toString(),
            observed_until: this.producedEvents.toString(), events,
          };
          finishInvoked=true;this.finishUncertain=true;
          if(this.finishBarrier){
            if(typeof this.session.previewFinishPending!=='function')
              throw new Error('E_INPUT_FINISH_NOT_READY: 저장 확정 전 기록 후보 확인이 필요합니다.');
            if(typeof this.session.resultIdentity!=='function')
              throw new Error('E_INPUT_FINISH_NOT_READY: 원 값·형식의 결과 정체 확인이 필요합니다.');
            const preview=await this.session.previewFinishPending(structuredClone(packet));
            const resultIdentity=await this.session.resultIdentity();
            const confirmed=await this.finishBarrier(structuredClone(packet),structuredClone(preview),resultIdentity);
            if(JSON.stringify(confirmed)!==JSON.stringify(preview))
              throw new Error('E_NATIVE_FINISH_RESULT_MISMATCH: Native와 화면의 완료 기록이 다릅니다.');
          }
          const record = await this.session.finishPending(packet);
          gotComplete=true;
          const notice = await this.session.finishNotice();
          if (notice?.excluded_input_count !== String(events.length))
            throw new Error('E_INPUT_FINISH_NOTICE: 제외 입력의 확인 응답이 다릅니다.');
          // Do not discard the real queue until the common owner confirms
          // success. No Release/End, step, or receipt is synthesized.
          this.finishedRecord = structuredClone(record);
          this.finishNotice = structuredClone(notice);
          this.finishedPacket = structuredClone(packet);
          this.events = [];
          this.stopped = true;
          this.finishUncertain = false;
          this.detach?.();
          return structuredClone(this.finishedRecord);
        } catch(error) {
          // An encoder/typed-end rejection precedes publication; transport
          // loss or a missing post-finish acknowledgement does not prove that.
          const reason=String(error?.message??error);
          if(!finishInvoked || (!gotComplete && /^(?:E_INPUT_TAPE_V3:|E_INPUT_FINISH_NOT_READY:|E_NATIVE_FINISH_BEFORE_PENDING:)/.test(reason)))
            this.finishUncertain=false;
          throw error;
        } finally {
          this.stopping = false;
          this.finishPromise = null;
        }
      })();
      return this.finishPromise;
    }
    if (this.stopped || this.stopping) throw new Error('E_INPUT_RECORDING_STOPPED: 중단 기록을 완료 봉인으로 바꿀 수 없습니다.');
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

  async stopRecording() {
    if (this.stopped || this.stopping || this.finishUncertain) throw new Error('E_INPUT_RECORDING_STOPPED: 기록이 끝났거나 종료 확인이 필요합니다.');
    if (this.inFlight || (!this.failure && this.events.length)) {
      throw new Error('E_LOGICAL_INPUT_PENDING: 미전달 사건을 먼저 한 마디 처리한 뒤 기록을 중단하세요.');
    }
    if (typeof this.session.stopRecording !== 'function') {
      throw new Error('E_INPUT_V3_UNSUPPORTED: 통합 기록 실행기가 필요합니다.');
    }
    if (this.failure) await this.abortPromise;
    this.stopping = true;
    this.detach?.(); // Explicit stop ends intake; no synthetic release/End.
    try {
      const summary=this.finishBarrier?.stop?await this.session.summary():null;
      let nativeStopped=null;
      if(this.finishBarrier?.stop){
        try{
          const result=summary?.outcome==='ABORTED'&&!this.failedInput?null:await this.session.resultIdentity();
          nativeStopped=await this.finishBarrier.stop(summary,result,this.failedInput);
        }catch(error){
          if(summary?.outcome!=='ABORTED')throw error;
          // Preserve the actual failed file even when Native verification is
          // unavailable. This is explicitly not Native persistence success.
          this.nativeStopFailure=String(error?.message??error);
          try{await this.finishBarrier.close?.();}catch(closeError){this.nativeStopFailure+=' / '+String(closeError?.message??closeError);}
        }
      }
      const record = await this.session.stopRecording();
      if (!['INCOMPLETE','ABORTED'].includes(record?.outcome)) {
        throw new Error('E_INPUT_RECORDING_PROOF: live 중단을 완료 기록으로 게시할 수 없습니다.');
      }
      if(nativeStopped!==null&&JSON.stringify(nativeStopped)!==JSON.stringify(record))
        throw new Error('E_NATIVE_NO_END_RESULT: Native와 화면의 중단 기록이 다릅니다.');
      this.nativeStopStored=nativeStopped!==null;
      return record;
    } finally {
      // An uncertain Worker reply is not permission to continue recording.
      this.stopped = true; this.stopping = false;
    }
  }
}
