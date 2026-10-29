import assert from "node:assert/strict";
import { requireSafeWorkbenchResult } from "../solutions/seamgrim_ui_mvp/ui/runtime/execution_safety.js";
import { formatNumber } from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_renderer.js";

const fault = [{ op: "emit_signal", signal: "산술고장", targets: [] }];
for (const payload of [{ patch: fault }, { engine_state: { patch: fault } }, { state: { patch: fault } }]) {
  for (const input of [payload, JSON.stringify(payload)]) {
    assert.throws(() => requireSafeWorkbenchResult(input), { code: "E_WORKBENCH_ARITHMETIC_FAULT" });
  }
}
for (const payload of [{ output: ["산술고장"] }, { patch: [{ op: "emit_signal", signal: "다른사건" }] }, { patch: [] }]) {
  assert.equal(requireSafeWorkbenchResult(payload), payload);
}
assert.equal(formatNumber(0.10000000009313226), "0.1");
console.log("EXECUTION_SAFETY_PASS signal_shapes=6 ordinary_output_preserved=3 numeric_display=1");
