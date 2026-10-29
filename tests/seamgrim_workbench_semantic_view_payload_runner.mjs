#!/usr/bin/env node

import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalConsoleViewPayload,
  buildCanonicalSpace2dViewPayload,
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  orderedObservationsFromWasmState,
  stableSemanticJson,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const resultOnlyIdentity = buildCanonicalConsoleViewPayload({
  identity: { result_identity: "result:without-committed-state" },
  projection: {
    status: "available",
    entries: [{ tick: 0, line_no: 1, kind: "output", text: "must not publish" }],
  },
});
assert(
  resultOnlyIdentity.status === "unavailable" && resultOnlyIdentity.content === null,
  "result identity without committed state identity was accepted for projection",
);
const whitespaceStateIdentity = buildCanonicalConsoleViewPayload({
  identity: { state_hash: "  ", result_identity: "result:blank-state" },
  projection: {
    status: "available",
    entries: [{ tick: 0, line_no: 1, kind: "output", text: "must not publish" }],
  },
});
assert(
  whitespaceStateIdentity.status === "unavailable" && whitespaceStateIdentity.content === null,
  "blank state identity was accepted for projection",
);
const sourceLessSpace2dIdentity = buildCanonicalSpace2dViewPayload({
  identity: { state_hash: "blake3:committed-state" },
  projection: {
    status: "available",
    space2d: { drawlist: [{ kind: "circle", x: 1, y: 2, r: 0.5 }] },
  },
});
assert(
  sourceLessSpace2dIdentity.status !== "available" && sourceLessSpace2dIdentity.content === null,
  "source-less space2d entity received a derived identity",
);

function wasmState(tick, theta, omega, energy) {
  const length = 2;
  const bobX = length * Math.sin(theta);
  const bobY = -length * Math.cos(theta);
  return {
    tick_id: tick,
    state_hash: `blake3:state_${tick}`,
    channels: [
      { key: "theta", dtype: "num", role: "state" },
      { key: "omega", dtype: "num", role: "state" },
      { key: "energy", dtype: "num", role: "state" },
    ],
    row: [theta, omega, energy],
    view_meta: {
      workbench: {
        schema: "seamgrim.workbench.authoritative_result_set.v1_candidate",
        producer: "ddonirang_tool.wasm_runtime",
        provenance: "native_authoritative_structured_result",
        results: {
          pendulum: {
            schema: "seamgrim.workbench.pendulum_result.v1_candidate",
            family: "pendulum",
            status: "available",
            producer: "ddonirang_tool.wasm_runtime",
            provenance: "native_authoritative_structured_result",
            content: {
              current: {
                theta,
                omega,
                energy,
                kinetic_energy: null,
                potential_energy: null,
                length,
                physical_bob: { x: bobX, y: bobY },
                normalized_bob: { x: bobX / length, y: -bobY / length },
              },
            },
          },
        },
      },
    },
  };
}

const run = {
  run_id: "run_semantic",
  source_hash: "sha256:source",
  shared_runtime_result: {
    schema: "ddn.runtime.run_summary.v1_candidate",
    source_profile_identity: "v1-core-v25",
    source_identity: "sha256:source",
    state_hash: "blake3:state_1",
  },
  work_cost_receipt: { schema: "ddonirang.work_cost.v1-candidate" },
  frames: [
    {
      frame_id: 0,
      madi: 0,
      state_hash: "blake3:state_0",
      source_hash: "sha256:source",
      wasm_state: wasmState(0, 0.5, 0, 1.2),
    },
    {
      frame_id: 1,
      madi: 1,
      state_hash: "blake3:state_1",
      source_hash: "sha256:source",
      wasm_state: wasmState(1, 0.48, -0.1, 1.2),
    },
  ],
};

const project = { id: "pendulum", primaryView: "pendulum" };
const primaryConfig = { primary_view: "pendulum", role: "primary", mode: "ddn-main" };
const canonicalViewHistory = run.frames.map((frame) => {
  const inputs = extractCanonicalViewAuthorityInputsFromState(frame.wasm_state);
  inputs.identity = {
    ...inputs.identity,
    state_hash: frame.state_hash,
    source_hash: frame.source_hash,
    tick_id: frame.madi,
  };
  return {
    frame_id: frame.frame_id,
    madi: frame.madi,
    state_hash: frame.state_hash,
    canonical_views: buildCanonicalViewPayloadSet(inputs),
  };
});
const canonicalViewPayloads = canonicalViewHistory.at(-1).canonical_views;
const payloadArgs = {
  project,
  run,
  frame: run.frames[1],
  viewConfig: primaryConfig,
  canonicalViewPayloads,
  canonicalViewHistory,
};
const first = buildCommittedSemanticViewPayload(payloadArgs);
const repeated = buildCommittedSemanticViewPayload(payloadArgs);

assert(stableSemanticJson(first) === stableSemanticJson(repeated), "same frame and view config must produce byte-equivalent semantic payloads");
assert(first.semantic_payload_hash === repeated.semantic_payload_hash, "same frame and view config hash mismatch");
assert(first.committed_frame.run_id === "run_semantic", "committed run identity missing");
assert(first.committed_frame.frame_id === 1 && first.committed_frame.madi === 1, "committed frame identity mismatch");
assert(first.committed_frame.state_hash === "blake3:state_1", "committed state hash mismatch");
assert(first.execution_receipt.state_hash === first.committed_frame.state_hash, "shared execution receipt state binding mismatch");
assert(first.ordered_observations.map((row) => row.name).join(",") === "theta,omega,energy", "observation order changed");
assert(first.view_binding.resolved_channels.length === 0, "raw channel binding remained active");
assert(first.view_binding.resolution === "canonical_workbench_result", "pendulum canonical binding resolution mismatch");
assert(first.committed_trajectory.length === 2, "committed trajectory did not include both frames");

const overlay = buildCommittedSemanticViewPayload({
  project,
  run,
  frame: run.frames[1],
  viewConfig: { ...primaryConfig, role: "overlay" },
  canonicalViewPayloads,
  canonicalViewHistory,
});
assert(overlay.semantic_payload_hash !== first.semantic_payload_hash, "view config change did not change semantic payload hash");
assert(overlay.committed_frame.state_hash === first.committed_frame.state_hash, "view config changed runtime state hash");
assert(overlay.view_config.participates_in_state_hash === false, "view config must remain outside runtime state hash");

const observations = orderedObservationsFromWasmState(wasmState(2, 0.4, -0.2, 1.19));
assert(observations.every((row) => row.state_hash_participation === true), "runtime observations must be marked state-derived");

console.log("seamgrim_workbench_semantic_view_payload: ok");
