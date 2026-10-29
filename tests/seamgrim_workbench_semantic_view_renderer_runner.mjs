#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { extractCanonicalViewAuthorityInputsFromState } from "../solutions/seamgrim_ui_mvp/ui/seamgrim_runtime_state.js";
import {
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  semanticViewPayloadHash,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";
import {
  buildSemanticRenderModel,
  buildCommittedSpace2dOverlay,
  renderCommittedSemanticView,
  formatNumber,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_renderer.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

assert(formatNumber("1.9020000016") === "1.902", "decimal observation text must use display precision");
assert(formatNumber("19.019999999") === "19.02", "display rounding must remove Fixed64 residue");
assert(formatNumber("001.9") === "001.9", "noncanonical text must remain text");
assert(formatNumber("문장 1.9020000016") === "문장 1.9020000016", "ordinary text must not be rewritten");

const resultSchemas = {
  line: "seamgrim.workbench.line_result.v1_candidate",
  pendulum: "seamgrim.workbench.pendulum_result.v1_candidate",
  projectile: "seamgrim.workbench.projectile_result.v1_candidate",
  grid: "seamgrim.workbench.grid_result.v1_candidate",
};

function authoritativeResultSet(family, content) {
  return {
    schema: "seamgrim.workbench.authoritative_result_set.v1_candidate",
    producer: "ddonirang_tool.wasm_runtime",
    provenance: "native_authoritative_structured_result",
    results: {
      [family]: {
        schema: resultSchemas[family],
        family,
        status: "available",
        producer: "ddonirang_tool.wasm_runtime",
        provenance: "native_authoritative_structured_result",
        content,
      },
    },
  };
}

function frame(frameId, values, { resources = {}, viewMeta = {}, workbench = null } = {}) {
  const entries = Object.entries(values);
  return {
    frame_id: frameId,
    madi: frameId + 1,
    state_hash: `blake3:frame_${frameId}`,
    source_hash: "sha256:semantic_renderer",
    wasm_state: {
      channels: entries.map(([key, value]) => ({ key, dtype: typeof value === "number" ? "num" : "str", role: "state" })),
      row: entries.map(([, value]) => value),
      resources: { value_json: { ...values, ...resources } },
      view_meta: workbench ? { ...viewMeta, workbench } : viewMeta,
    },
  };
}

function canonicalFor(run, row) {
  const inputs = extractCanonicalViewAuthorityInputsFromState(row.wasm_state);
  inputs.identity = {
    ...inputs.identity,
    state_hash: row.state_hash,
    source_hash: row.source_hash ?? run.source_hash,
    tick_id: row.madi,
  };
  return buildCanonicalViewPayloadSet(inputs);
}

function payloadFor(primaryView, frames, activeIndex = frames.length - 1, role = "primary") {
  const run = {
    run_id: `run_${primaryView}`,
    source_hash: "sha256:semantic_renderer",
    shared_runtime_result: {
      schema: "ddn.runtime.run_summary.v1_candidate",
      source_profile_identity: "v1-core-v25",
      source_identity: "sha256:semantic_renderer",
      state_hash: String(frames[activeIndex]?.state_hash ?? ""),
    },
    work_cost_receipt: { schema: "ddonirang.work_cost.v1-candidate" },
    frames,
  };
  const projectId = primaryView || "missing_renderer_identity";
  const canonicalViewHistory = frames.slice(0, activeIndex + 1).map((row) => ({
    frame_id: row.frame_id,
    madi: row.madi,
    state_hash: row.state_hash,
    canonical_views: canonicalFor(run, row),
  }));
  return buildCommittedSemanticViewPayload({
    project: { id: projectId, primaryView },
    run,
    frame: frames[activeIndex],
    viewConfig: { primary_view: primaryView, role, mode: "ddn-main" },
    canonicalViewPayloads: canonicalViewHistory.at(-1).canonical_views,
    canonicalViewHistory,
  });
}

function payloadWithoutRendererIdentity(frames, activeIndex = frames.length - 1) {
  const run = {
    run_id: "run_missing_renderer_identity",
    source_hash: "sha256:semantic_renderer",
    shared_runtime_result: {
      schema: "ddn.runtime.run_summary.v1_candidate",
      source_profile_identity: "v1-core-v25",
      source_identity: "sha256:semantic_renderer",
      state_hash: String(frames[activeIndex]?.state_hash ?? ""),
    },
    work_cost_receipt: { schema: "ddonirang.work_cost.v1-candidate" },
    frames,
  };
  const canonicalViewHistory = frames.slice(0, activeIndex + 1).map((row) => ({
    frame_id: row.frame_id,
    madi: row.madi,
    state_hash: row.state_hash,
    canonical_views: canonicalFor(run, row),
  }));
  return buildCommittedSemanticViewPayload({
    project: { id: "missing_renderer_identity" },
    run,
    frame: frames[activeIndex],
    viewConfig: { role: "primary", mode: "ddn-main" },
    canonicalViewPayloads: canonicalViewHistory.at(-1).canonical_views,
    canonicalViewHistory,
  });
}

const pendulumPrimary = buildSemanticRenderModel(payloadFor("pendulum", [
  frame(0, { theta: 99, omega: 99, energy: 99, L: 99 }, {
    workbench: authoritativeResultSet("pendulum", {
      current: {
        theta: 0.25,
        omega: -0.1,
        energy: 1.4,
        kinetic_energy: 0.4,
        potential_energy: 1,
        length: 2,
        physical_bob: { x: 0.4948079185090459, y: -1.9378248434212895 },
        normalized_bob: { x: 0.24740395925452294, y: 0.9689124217106447 },
      },
    }),
  }),
]));
const pendulumOverlay = buildSemanticRenderModel(payloadFor("pendulum", [
  frame(0, { theta: 99 }, {
    workbench: authoritativeResultSet("pendulum", {
      current: {
        theta: 0.25,
        omega: -0.1,
        energy: 1.4,
        kinetic_energy: 0.4,
        potential_energy: 1,
        length: 2,
        physical_bob: { x: 0.4948079185090459, y: -1.9378248434212895 },
        normalized_bob: { x: 0.24740395925452294, y: 0.9689124217106447 },
      },
    }),
  }),
], 0, "overlay"));
assert(pendulumPrimary.status === "ready", "pendulum renderer did not resolve committed theta");
assert(pendulumPrimary.current.theta === 0.25, "pendulum used a non-committed theta");
assert(Math.abs(pendulumPrimary.current.physical_bob.x - 2 * Math.sin(0.25)) < 1e-12, "pendulum bob x was not derived from committed theta/L");
assert(Math.abs(pendulumPrimary.current.physical_bob.y + 2 * Math.cos(0.25)) < 1e-12, "pendulum bob y was not derived from committed theta/L");
assert(JSON.stringify(pendulumOverlay.overlay.normalized_bob) === JSON.stringify(pendulumPrimary.current.normalized_bob), "pendulum overlay left the primary coordinate system");
assert(pendulumPrimary.labels.some((row) => row.key === "omega" && row.value === -0.1), "pendulum committed omega label missing");

const projectileFrames = [
  frame(0, { t: 99, x: 99, y: 99, vx: 99, vy: 99 }, {
    workbench: authoritativeResultSet("projectile", { current: { t: 0, x: 0, y: 0, vx: 8, vy: 15 }, camera: { x_min: -1, x_max: 3, y_min: -1, y_max: 4 } }),
  }),
  frame(1, { t: 99, x: 99, y: 99, vx: 99, vy: 99 }, {
    workbench: authoritativeResultSet("projectile", { current: { t: 1, x: 0.8, y: 1.5, vx: 8, vy: 14.02 }, camera: { x_min: -1, x_max: 3, y_min: -1, y_max: 4 } }),
  }),
  frame(2, { t: 99, x: 99, y: 99, vx: 99, vy: 99 }, {
    workbench: authoritativeResultSet("projectile", { current: { t: 2, x: 1.6, y: 2.902, vx: 8, vy: 13.04 }, camera: { x_min: -1, x_max: 3, y_min: -1, y_max: 4 } }),
  }),
];
const projectilePayload = payloadFor("projectile", projectileFrames, 1);
const projectile = buildSemanticRenderModel(projectilePayload);
assert(projectile.status === "ready", "projectile renderer did not resolve committed x/y");
assert(projectile.current.x === 0.8 && projectile.current.y === 1.5, "projectile current point was not the active committed frame");
assert(projectile.trajectory.length === 2, "projectile trajectory included a future committed frame beyond the active cursor");
assert(projectile.trajectory.at(-1).state_hash === "blake3:frame_1", "projectile trajectory did not end at the active committed frame");
assert(projectile.labels.some((row) => row.key === "vy" && row.value === 14.02), "projectile committed velocity label missing");

const overlayPayload = payloadFor("projectile", projectileFrames, 1, "overlay");
const overlaySpace = { drawlist: [{ kind: "circle", x: 0.8, y: 1.5, r: 0.35 }] };
const overlayBefore = JSON.stringify({ overlayPayload, overlaySpace });
// Live VM frames continue from a one-tick runSummary; the receipt itself stays unchanged.
const livePayload = payloadFor("projectile", projectileFrames, 2, "overlay");
livePayload.execution_receipt.state_hash = projectileFrames[0].state_hash;
function rehash(payload) {
  const { semantic_payload_hash, ...input } = payload;
  payload.semantic_payload_hash = semanticViewPayloadHash(input);
  return payload;
}
rehash(livePayload);
assert(buildSemanticRenderModel(livePayload).status === "ready", "live lineage rejected original one-tick receipt");
const equalHashWrongRun = structuredClone(overlayPayload);
equalHashWrongRun.execution_receipt.run_id = "other_run";
assert(buildSemanticRenderModel(rehash(equalHashWrongRun)).status !== "ready", "equal hash bypassed wrong-run lineage");
for (const mutate of [
  (p) => { p.execution_receipt.run_id = "other_run"; },
  (p) => { p.execution_receipt.committed_lineage[1].source_hash = "other_source"; },
  (p) => { p.execution_receipt.committed_lineage.pop(); },
  (p) => { p.execution_receipt.committed_lineage[0].state_hash = "unrelated_start"; },
]) {
  const invalid = structuredClone(livePayload);
  mutate(invalid);
  assert(buildSemanticRenderModel(rehash(invalid)).status !== "ready", "invalid live receipt lineage accepted");
}
const actualOverlay = buildCommittedSpace2dOverlay(overlayPayload, overlaySpace);
assert(actualOverlay.drawlist.length === 1, "canonical projectile decoration missing");
assert(actualOverlay.drawlist[0].x1 === 0.8 && actualOverlay.drawlist[0].y1 === 1.5, "velocity detached from committed point");
assert(Math.abs(actualOverlay.drawlist[0].x2 - 2.4) < 1e-12, "velocity world scale changed");
const noCameraDecoration = structuredClone(overlayPayload);
delete noCameraDecoration.canonical_views.workbench_projectile.content.result.camera;
rehash(noCameraDecoration);
assert(buildCommittedSpace2dOverlay(noCameraDecoration, overlaySpace).drawlist.length === 1,
  "committed Space2D decoration must not demand a separate projection camera");
const invalidNoCamera = structuredClone(noCameraDecoration);
invalidNoCamera.execution_receipt.run_id = "other_run";
assert(buildCommittedSpace2dOverlay(rehash(invalidNoCamera), overlaySpace).drawlist.length === 0,
  "camera-free decoration bypassed receipt identity");
assert(buildCommittedSpace2dOverlay(projectilePayload, overlaySpace).drawlist.length === 0, "disabled decoration still drawn");
assert(buildCommittedSpace2dOverlay(overlayPayload, { drawlist: [] }).drawlist.length === 0, "decoration fabricated absent main picture");
assert(buildCommittedSpace2dOverlay(overlayPayload, { drawlist: [{kind:"circle",x:99,y:99}] }).drawlist.length === 0, "decoration accepted mismatched coordinates");
const nextOverlay = buildCommittedSpace2dOverlay(payloadFor("projectile", projectileFrames, 2, "overlay"),
  { drawlist: [{kind:"circle",x:1.6,y:2.902,r:0.35}] });
assert(nextOverlay.worldTarget.x === 1.6 && nextOverlay.worldTarget.y === 2.902, "decoration target failed to follow committed step");
assert(JSON.stringify({ overlayPayload, overlaySpace }) === overlayBefore, "decoration mutated committed payload");
const pendulumDecoration = buildCommittedSpace2dOverlay(payloadFor("pendulum", [frame(0, {}, {
  workbench: authoritativeResultSet("pendulum", {current: pendulumPrimary.current}),
})], 0, "overlay"), {drawlist:[{kind:"circle", ...pendulumPrimary.current.physical_bob, r:0.08}]});
assert(pendulumDecoration.drawlist.length === 25, "canonical pendulum reference and angle arc missing");
assert(pendulumDecoration.drawlist[0].y2 === -2, "pendulum reference uses different coordinate system");

const board = [
  [".", ".", "."],
  [".", "O", "."],
  [".", ".", "."],
];
const gridPayload = payloadFor("grid", [
  frame(6, { tick: 7 }, {
    resources: { board, piece: { x: 1, y: 1 } },
    viewMeta: {
      space2d: {
        camera: { x_min: 0, x_max: 3, y_min: 0, y_max: 3 },
        drawlist: [{ kind: "circle", x: 1.5, y: 1.5, r: 0.25 }],
      },
    },
    workbench: authoritativeResultSet("grid", {
      position: { x: 1, y: 1 },
      dimensions: { columns: 3, rows: 3 },
      space2d: {
        schema: "seamgrim.space2d.v0",
        camera: { x_min: 0, x_max: 3, y_min: 0, y_max: 3 },
        drawlist: [{ kind: "circle", x: 1.5, y: 1.5, r: 0.25 }],
      },
    }),
  }),
]);
const grid = buildSemanticRenderModel(gridPayload);
assert(grid.status === "ready", "grid renderer ignored committed board/view geometry");
assert(grid.tick === 7, "grid renderer did not use committed tick");
assert(grid.position.x === 1 && grid.position.y === 1, "grid renderer did not use committed piece position");
assert(grid.geometry_source === "canonical_workbench_grid_result", "grid renderer geometry was not canonical runtime data");

const missingGridCameraPayload = payloadFor("grid", [frame(7, { tick: 8 }, {
  workbench: authoritativeResultSet("grid", {
    position: { x: 1, y: 1 },
    dimensions: { columns: 3, rows: 3 },
    space2d: {
      schema: "seamgrim.space2d.v0",
      drawlist: [{ kind: "circle", x: 1.5, y: 1.5, r: 0.25 }],
    },
  }),
})]);
const missingGridCamera = buildSemanticRenderModel(missingGridCameraPayload);
assert(missingGridCamera.status === "canonical_unsupported", "geometry must not synthesize a grid camera");
assert(missingGridCamera.canonical_diagnostic?.reason === "projection_camera_missing_or_invalid", "grid missing camera diagnostic drift");

const unknownBogaeKindPayload = payloadFor("grid", [frame(8, { tick: 9 }, {
  workbench: authoritativeResultSet("grid", {
    position: { x: 1, y: 1 },
    dimensions: { columns: 3, rows: 3 },
    space2d: {
      schema: "seamgrim.space2d.v0",
      camera: { x_min: 0, x_max: 3, y_min: 0, y_max: 3 },
      drawlist: [{ kind: "prefix-line", x1: 0, y1: 0, x2: 1, y2: 1 }],
    },
  }),
})]);
const unknownBogaeKindBefore = JSON.stringify(unknownBogaeKindPayload);
const unknownBogaeKind = buildSemanticRenderModel(unknownBogaeKindPayload);
assert(unknownBogaeKind.status === "canonical_unsupported", "unknown Bogae kind must fail closed");
assert(unknownBogaeKind.canonical_diagnostic?.reason === "unsupported_bogae_kind", "unknown Bogae kind diagnostic drift");
assert(!Object.hasOwn(unknownBogaeKind, "draw_list"), "unknown Bogae kind published a render model");
const unknownBogaeKindCanvas = failureCanvas();
const unknownBogaeKindRendered = renderCommittedSemanticView(unknownBogaeKindCanvas, 320, 180, unknownBogaeKindPayload);
assert(!Object.hasOwn(unknownBogaeKindRendered, "canvas_projection"), "unknown Bogae kind published canvas output");
assert(JSON.stringify(unknownBogaeKindPayload) === unknownBogaeKindBefore, "unknown Bogae kind mutated semantic payload");

const linePayload = payloadFor("line", [frame(0, { a: 99, b: 99, x: 99, y: 99 }, {
  workbench: authoritativeResultSet("line", {
    current: { x: 2, y: 5 },
    equation: { a: 2, b: 1 },
    camera: { x_min: -2, x_max: 4, y_min: -3, y_max: 9 },
  }),
})]);
const line = buildSemanticRenderModel(linePayload);
assert(line.status === "ready" && line.current.x === 2 && line.current.y === 5, "static graph did not use committed result point");
assert(line.labels.map((row) => row.key).join(",") === "x,y,a,b", "static graph semantic labels missing");
assert(JSON.stringify(line.camera) === JSON.stringify({ minX: -2, maxX: 4, minY: -3, maxY: 9 }), "line renderer did not retain explicit camera");
assert(
  linePayload.canonical_views.workbench_line.content.provider_identity === "ddonirang_tool.wasm_runtime",
  "canonical result did not preserve its existing provider identity",
);
assert(
  linePayload.canonical_views.workbench_line.content.provider_provenance === "native_authoritative_structured_result",
  "canonical result did not preserve its existing provider provenance",
);
assert(
  linePayload.execution_receipt?.state_hash === linePayload.committed_frame.state_hash,
  "committed semantic payload did not bind its existing shared execution receipt",
);
assert(
  linePayload.committed_frame.workspace_identity === "line",
  "committed frame did not bind the existing Local Workspace identity",
);

const missingWorkspaceIdentity = structuredClone(linePayload);
missingWorkspaceIdentity.committed_frame.workspace_identity = "";
const { semantic_payload_hash: missingWorkspaceHash, ...missingWorkspaceInput } = missingWorkspaceIdentity;
missingWorkspaceIdentity.semantic_payload_hash = semanticViewPayloadHash(missingWorkspaceInput);
const missingWorkspaceBefore = JSON.stringify(missingWorkspaceIdentity);
const missingWorkspaceModel = buildSemanticRenderModel(missingWorkspaceIdentity);
assert(missingWorkspaceModel.status === "canonical_unsupported", "missing Local Workspace identity must fail before projection");
assert(missingWorkspaceModel.canonical_diagnostic?.reason === "committed_frame_identity_missing_or_invalid", "workspace identity failure drift");
assert(!Object.hasOwn(missingWorkspaceModel, "current") && !Object.hasOwn(missingWorkspaceModel, "canvas_projection"), "missing workspace identity published a projection");
assert(JSON.stringify(missingWorkspaceIdentity) === missingWorkspaceBefore, "missing workspace identity mutated semantic payload");

const missingViewIdentity = structuredClone(linePayload);
missingViewIdentity.view_config.primary_view = "";
const { semantic_payload_hash: missingViewHash, ...missingViewInput } = missingViewIdentity;
missingViewIdentity.semantic_payload_hash = semanticViewPayloadHash(missingViewInput);
const missingViewBefore = JSON.stringify(missingViewIdentity);
const missingViewModel = buildSemanticRenderModel(missingViewIdentity);
assert(missingViewModel.status === "canonical_unsupported", "missing view identity must fail before projection");
assert(missingViewModel.canonical_diagnostic?.reason === "view_binding_identity_missing_or_mismatch", "view identity failure drift");
assert(!Object.hasOwn(missingViewModel, "current") && !Object.hasOwn(missingViewModel, "canvas_projection"), "missing view identity published a projection");
assert(JSON.stringify(missingViewIdentity) === missingViewBefore, "missing view identity mutated semantic payload");

const stalePayloadHash = structuredClone(linePayload);
stalePayloadHash.semantic_payload_hash = "candidate:fnv1a32:semantic_view:stale";
const stalePayloadHashBefore = JSON.stringify(stalePayloadHash);
const stalePayloadHashModel = buildSemanticRenderModel(stalePayloadHash);
assert(stalePayloadHashModel.status === "canonical_unsupported", "stale semantic payload hash must fail before projection");
assert(stalePayloadHashModel.canonical_diagnostic?.reason === "semantic_payload_hash_mismatch", "stale semantic payload hash diagnostic drift");
assert(!Object.hasOwn(stalePayloadHashModel, "current") && !Object.hasOwn(stalePayloadHashModel, "canvas_projection"), "stale semantic payload hash published a projection");
assert(JSON.stringify(stalePayloadHash) === stalePayloadHashBefore, "stale semantic payload hash mutated semantic payload");

const mismatchedViewBinding = structuredClone(linePayload);
mismatchedViewBinding.view_binding.view_family = "grid";
const { semantic_payload_hash: mismatchedViewBindingHash, ...mismatchedViewBindingInput } = mismatchedViewBinding;
mismatchedViewBinding.semantic_payload_hash = semanticViewPayloadHash(mismatchedViewBindingInput);
const mismatchedViewBindingBefore = JSON.stringify(mismatchedViewBinding);
const mismatchedViewBindingModel = buildSemanticRenderModel(mismatchedViewBinding);
assert(mismatchedViewBindingModel.status === "canonical_unsupported", "hash-valid mismatched view binding must fail before projection");
assert(mismatchedViewBindingModel.canonical_diagnostic?.reason === "view_binding_identity_missing_or_mismatch", "view binding mismatch diagnostic drift");
assert(!Object.hasOwn(mismatchedViewBindingModel, "current") && !Object.hasOwn(mismatchedViewBindingModel, "canvas_projection"), "mismatched view binding published a projection");
assert(JSON.stringify(mismatchedViewBinding) === mismatchedViewBindingBefore, "mismatched view binding mutated semantic payload");

const staleViewBindingStatus = structuredClone(linePayload);
staleViewBindingStatus.view_binding.canonical_payload_status = "unavailable";
const { semantic_payload_hash: staleViewBindingHash, ...staleViewBindingInput } = staleViewBindingStatus;
staleViewBindingStatus.semantic_payload_hash = semanticViewPayloadHash(staleViewBindingInput);
const staleViewBindingBefore = JSON.stringify(staleViewBindingStatus);
const staleViewBindingModel = buildSemanticRenderModel(staleViewBindingStatus);
assert(staleViewBindingModel.status === "canonical_unsupported", "stale canonical view-binding status must fail before projection");
assert(staleViewBindingModel.canonical_diagnostic?.reason === "view_binding_identity_missing_or_mismatch", "stale view-binding status failure drift");
assert(!Object.hasOwn(staleViewBindingModel, "current") && !Object.hasOwn(staleViewBindingModel, "canvas_projection"), "stale view-binding status published a projection");
assert(JSON.stringify(staleViewBindingStatus) === staleViewBindingBefore, "stale view-binding status mutated semantic payload");

const mismatchedCanonicalIdentity = structuredClone(linePayload);
mismatchedCanonicalIdentity.canonical_views.workbench_line.identity.state_hash = "blake3:stale-canonical-frame";
const { semantic_payload_hash: mismatchedCanonicalIdentityHash, ...mismatchedCanonicalIdentityInput } = mismatchedCanonicalIdentity;
mismatchedCanonicalIdentity.semantic_payload_hash = semanticViewPayloadHash(mismatchedCanonicalIdentityInput);
const mismatchedCanonicalIdentityBefore = JSON.stringify(mismatchedCanonicalIdentity);
const mismatchedCanonicalIdentityModel = buildSemanticRenderModel(mismatchedCanonicalIdentity);
assert(mismatchedCanonicalIdentityModel.status === "canonical_unsupported", "hash-valid canonical identity mismatch must fail before projection");
assert(mismatchedCanonicalIdentityModel.canonical_diagnostic?.reason === "canonical_result_identity_mismatch", "canonical identity mismatch diagnostic drift");
assert(!Object.hasOwn(mismatchedCanonicalIdentityModel, "current") && !Object.hasOwn(mismatchedCanonicalIdentityModel, "canvas_projection"), "canonical identity mismatch published a projection");
assert(JSON.stringify(mismatchedCanonicalIdentity) === mismatchedCanonicalIdentityBefore, "canonical identity mismatch mutated semantic payload");

const mismatchedProviderIdentity = structuredClone(linePayload);
mismatchedProviderIdentity.canonical_views.workbench_line.content.provider_identity = "other.provider";
const { semantic_payload_hash: mismatchedProviderHash, ...mismatchedProviderInput } = mismatchedProviderIdentity;
mismatchedProviderIdentity.semantic_payload_hash = semanticViewPayloadHash(mismatchedProviderInput);
const mismatchedProviderBefore = JSON.stringify(mismatchedProviderIdentity);
const mismatchedProviderModel = buildSemanticRenderModel(mismatchedProviderIdentity);
assert(mismatchedProviderModel.status === "canonical_unsupported", "hash-valid provider mismatch must fail before projection");
assert(mismatchedProviderModel.canonical_diagnostic?.reason === "canonical_result_provider_identity_mismatch", "provider identity failure drift");
assert(!Object.hasOwn(mismatchedProviderModel, "current") && !Object.hasOwn(mismatchedProviderModel, "canvas_projection"), "provider mismatch published a projection");
assert(JSON.stringify(mismatchedProviderIdentity) === mismatchedProviderBefore, "provider mismatch mutated semantic payload");

const staleExecutionReceipt = structuredClone(linePayload);
staleExecutionReceipt.execution_receipt.state_hash = "blake3:stale-receipt-state";
const { semantic_payload_hash: staleReceiptHash, ...staleReceiptInput } = staleExecutionReceipt;
staleExecutionReceipt.semantic_payload_hash = semanticViewPayloadHash(staleReceiptInput);
const staleExecutionReceiptBefore = JSON.stringify(staleExecutionReceipt);
const staleExecutionReceiptModel = buildSemanticRenderModel(staleExecutionReceipt);
assert(staleExecutionReceiptModel.status === "canonical_unsupported", "stale execution receipt state must fail before projection");
assert(staleExecutionReceiptModel.canonical_diagnostic?.reason === "committed_frame_identity_missing_or_invalid", "stale execution receipt failure drift");
assert(!Object.hasOwn(staleExecutionReceiptModel, "current") && !Object.hasOwn(staleExecutionReceiptModel, "canvas_projection"), "stale execution receipt published a projection");
assert(JSON.stringify(staleExecutionReceipt) === staleExecutionReceiptBefore, "stale execution receipt mutated semantic payload");

const missingCommittedFrameIdentity = structuredClone(linePayload);
missingCommittedFrameIdentity.committed_frame.run_id = "";
const { semantic_payload_hash: missingCommittedFrameHash, ...missingCommittedFrameInput } = missingCommittedFrameIdentity;
missingCommittedFrameIdentity.semantic_payload_hash = semanticViewPayloadHash(missingCommittedFrameInput);
const missingCommittedFrameBefore = JSON.stringify(missingCommittedFrameIdentity);
const missingCommittedFrameModel = buildSemanticRenderModel(missingCommittedFrameIdentity);
assert(missingCommittedFrameModel.status === "canonical_unsupported", "hash-valid payload without a run identity must fail before projection");
assert(missingCommittedFrameModel.canonical_diagnostic?.reason === "committed_frame_identity_missing_or_invalid", "committed frame identity diagnostic drift");
assert(!Object.hasOwn(missingCommittedFrameModel, "current") && !Object.hasOwn(missingCommittedFrameModel, "canvas_projection"), "invalid committed frame published a projection");
assert(JSON.stringify(missingCommittedFrameIdentity) === missingCommittedFrameBefore, "invalid committed frame mutated semantic payload");

const missingCommittedSourceIdentity = structuredClone(linePayload);
missingCommittedSourceIdentity.committed_frame.source_hash = "";
const { semantic_payload_hash: missingCommittedSourceHash, ...missingCommittedSourceInput } = missingCommittedSourceIdentity;
missingCommittedSourceIdentity.semantic_payload_hash = semanticViewPayloadHash(missingCommittedSourceInput);
const missingCommittedSourceModel = buildSemanticRenderModel(missingCommittedSourceIdentity);
assert(missingCommittedSourceModel.status === "canonical_unsupported", "hash-valid payload without a source identity must fail before projection");
assert(missingCommittedSourceModel.canonical_diagnostic?.reason === "committed_frame_identity_missing_or_invalid", "committed source identity diagnostic drift");
assert(!Object.hasOwn(missingCommittedSourceModel, "current"), "invalid committed source identity published a projection");

const missingCameraPayload = payloadFor("line", [frame(0, { x: 9, y: 12 }, {
  workbench: authoritativeResultSet("line", {
    current: { x: 2, y: 5 },
    equation: { a: 2, b: 1 },
  }),
})]);
const missingCameraBefore = JSON.stringify(missingCameraPayload);
const missingCamera = buildSemanticRenderModel(missingCameraPayload);
assert(missingCamera.status === "canonical_unsupported", "missing explicit camera must fail closed");
assert(missingCamera.canonical_diagnostic?.reason === "projection_camera_missing_or_invalid", "missing camera diagnostic drift");
assert(!Object.hasOwn(missingCamera, "camera") && !Object.hasOwn(missingCamera, "current"), "missing camera published a projection model");
assert(JSON.stringify(missingCameraPayload) === missingCameraBefore, "missing camera projection mutated semantic payload");
for (const model of [pendulumPrimary, pendulumOverlay, projectile, grid, line]) {
  assert(model.data_source === "canonical_workbench_result", `${model.family}: raw-state authority remains active`);
}

const awaiting = buildSemanticRenderModel(buildCommittedSemanticViewPayload({
  project: { id: "empty", primaryView: "pendulum" },
  viewConfig: { primary_view: "pendulum", role: "primary", mode: "ddn-main" },
}));
assert(awaiting.status === "awaiting_committed_frame", "pre-run preview fabricated a runtime frame");

const missingRendererIdentityPayload = payloadWithoutRendererIdentity([frame(0, { x: 9, y: 12, a: 4, b: 2 }, {
  workbench: authoritativeResultSet("line", {
    current: { x: 2, y: 5 },
    equation: { a: 2, b: 1 },
  }),
})]);
const missingRendererIdentityBefore = JSON.stringify(missingRendererIdentityPayload);
const missingRendererIdentity = buildSemanticRenderModel(missingRendererIdentityPayload);
assert(missingRendererIdentityPayload.view_config.primary_view === "", "missing renderer identity must not default to line");
assert(missingRendererIdentity.status === "canonical_unsupported", "numeric observations must not select line without renderer identity");
assert(missingRendererIdentity.canonical_diagnostic?.reason === "view_binding_identity_missing_or_mismatch", "missing renderer identity failure drift");
assert(!Object.hasOwn(missingRendererIdentity, "current"), "missing renderer identity created a line model");
assert(JSON.stringify(missingRendererIdentityPayload) === missingRendererIdentityBefore, "missing renderer identity mutated semantic payload");

function failureCanvas() {
  const calls = [];
  return {
    calls,
    fillText(...args) { calls.push(["fillText", ...args]); },
    set fillStyle(value) { calls.push(["fillStyle", value]); },
    set font(value) { calls.push(["font", value]); },
  };
}

function assertUnsupportedRendererFamily(family) {
  const payload = payloadFor(family, [frame(0, { x: 9, y: 12 }, {
    workbench: authoritativeResultSet("line", {
      current: { x: 2, y: 5 },
      equation: { a: 2, b: 1 },
    }),
  })]);
  const before = JSON.stringify(payload);
  const first = buildSemanticRenderModel(payload);
  const second = buildSemanticRenderModel(payload);
  const expectedReason = family === ""
    ? "view_binding_identity_missing_or_mismatch"
    : "unsupported_renderer_family";
  assert(first.status === "canonical_unsupported", `${JSON.stringify(family)} must fail as an unsupported projection`);
  assert(first.canonical_diagnostic?.code === "E_CANONICAL_WORKBENCH_RESULT_UNSUPPORTED_CANDIDATE", `${JSON.stringify(family)} diagnostic drift`);
  assert(first.canonical_diagnostic?.reason === expectedReason, `${JSON.stringify(family)} must preserve projection failure cause`);
  if (family !== "") assert(first.canonical_diagnostic?.actual_schema === family, `${JSON.stringify(family)} renderer identity drift`);
  assert(!Object.hasOwn(first, "current") && !Object.hasOwn(first, "points") && !Object.hasOwn(first, "trajectory"), `${JSON.stringify(family)} must not create a line/default model`);
  assert(JSON.stringify(first) === JSON.stringify(second), `${JSON.stringify(family)} failure must be deterministic`);
  assert(JSON.stringify(payload) === before, `${JSON.stringify(family)} renderer mutated the semantic payload`);
  const ctx = failureCanvas();
  const rendered = renderCommittedSemanticView(ctx, 320, 180, payload);
  assert(rendered.status === "canonical_unsupported", `${JSON.stringify(family)} renderer silently succeeded`);
  assert(!Object.hasOwn(rendered, "canvas_projection"), `${JSON.stringify(family)} renderer published a canvas success projection`);
  assert(ctx.calls.some(([name]) => name === "fillText"), `${JSON.stringify(family)} failure was swallowed instead of rendered as a failure`);
  assert(JSON.stringify(payload) === before, `${JSON.stringify(family)} renderer mutated payload during projection`);
}

for (const unsupportedFamily of [
  "unknown_renderer",
  "",
  "line-compatible",
  "LINE",
  "prefix-line",
  "line-suffix",
  "grid2d",
]) {
  assertUnsupportedRendererFamily(unsupportedFamily);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workbenchSource = await fs.readFile(path.join(root, "solutions/seamgrim_ui_mvp/ui/workbench/workbench.js"), "utf8");
for (const forbidden of ["theta = 0.52", "projectileParamsFromSource", "projectilePoints", "readDdnSetupNumber", "grid session tick 0"]) {
  assert(!workbenchSource.includes(forbidden), `forbidden renderer shortcut remains: ${forbidden}`);
}

console.log("seamgrim_workbench_semantic_view_renderer: ok");
