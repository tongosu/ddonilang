import assert from "node:assert/strict";
import { runHostStep, HOST_STEP_STOP_MS, NATIVE_HOST_STEP_STOP_MS, NATIVE_HOST_STEP_STOP_NOTICE } from "../solutions/seamgrim_ui_mvp/ui/runtime/host_step_watchdog.js";
let now = 0, next = 0, stopped = 0;
const timers = new Map();
const clock = {
  schedule(fn, ms) { assert.equal(ms, 10000); const id = ++next; timers.set(id, { fn, due: now + ms }); return id; },
  clear(id) { timers.delete(id); },
};
function advance(ms) { now += ms; for (const [id, item] of timers) if (item.due <= now) { timers.delete(id); item.fn(); } }
for (let i = 0; i < 4; i++) {
  const frame = { state_hash: `frame${i}` };
  assert.equal(await runHostStep(async () => { advance(9000); return frame; }, () => stopped++, clock), frame);
}
assert.equal(stopped, 0, "total simulation duration must not trigger host stop");
assert.equal(timers.size, 0);
let cancel;
const pending = new Promise((_, reject) => { cancel = reject; });
const cancellation = new Error("user stop");
const blocked = runHostStep(() => pending, () => { stopped++; cancel(cancellation); }, clock);
const rejected = assert.rejects(blocked, error => error === cancellation);
advance(HOST_STEP_STOP_MS - 1); assert.equal(stopped, 0);
advance(1); await rejected; assert.equal(stopped, 1);
assert.equal(timers.size, 0);
console.log("HOST_STEP_WATCHDOG_PASS per_step=4 total_36s_allowed=1 user_stop_identity=1 timers_cleared=1");
let nativeNotice;
const nativeClock={...clock,schedule(fn,ms){assert.equal(ms,NATIVE_HOST_STEP_STOP_MS);const id=++next;timers.set(id,{fn,due:now+ms});return id;}};
assert.equal(await runHostStep(async()=>{advance(37000);return 'protected Native frame';},()=>{throw Error('Native I/O must not acquire the 10s loop timeout');},
  {...nativeClock,nativeSourceBound:true}),'protected Native frame');
assert.equal(timers.size,0);
let rejectNative;
const nativeBlocked=runHostStep(()=>new Promise((_,reject)=>{rejectNative=reject;}),notice=>{nativeNotice=notice;rejectNative(cancellation);},
  {...nativeClock,nativeSourceBound:true});
const nativeRejected=assert.rejects(nativeBlocked,error=>error===cancellation);
advance(NATIVE_HOST_STEP_STOP_MS-1);assert.equal(nativeNotice,undefined);
advance(1);await nativeRejected;assert.equal(nativeNotice,NATIVE_HOST_STEP_STOP_NOTICE);assert.equal(timers.size,0);
console.log('HOST_NATIVE_STEP_BOUNDED_PASS actual_native_profile=180s pure_wasm_profile=10s notices_distinct=1');
