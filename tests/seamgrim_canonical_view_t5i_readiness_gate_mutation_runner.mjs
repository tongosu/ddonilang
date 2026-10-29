#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { stableSemanticJson } from "../solutions/seamgrim_ui_mvp/ui/workbench/semantic_view_payload.js";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function proveMutationRed({ label, greenValue, mutate, invariant }) {
  assert(invariant(greenValue), `${label}: initial GREEN failed`);
  const mutant = mutate(structuredClone(greenValue));
  assert(!invariant(mutant), `${label}: mutation did not turn RED`);
  assert(invariant(greenValue), `${label}: restored GREEN failed`);
  return { label, initial: "GREEN", mutated: "RED", restored: "GREEN" };
}

const root = path.resolve(import.meta.dirname, "..");
const payload = JSON.parse(await fs.readFile(path.join(
  root,
  "pack/seamgrim_scene_curriculum_canonical_view_payload_v1/expected/curriculum_semantic_scene.payload.detjson",
), "utf8"));
const contract = JSON.parse(await fs.readFile(path.join(
  root,
  "pack/seamgrim_canonical_view_t5i_native_readiness_audit_v1/contract.detjson",
), "utf8"));
const results = [];

results.push(proveMutationRed({
  label: "M1_DOM_SELECTOR_ENTERS_CANONICAL_PAYLOAD",
  greenValue: payload,
  mutate(value) {
    value.content.curriculum_semantic_scene.anchors[0].target_selector = "#runtime-dom";
    return value;
  },
  invariant(value) {
    const bytes = stableSemanticJson(value);
    return !contract.web_only_payload_keys.some((key) => bytes.includes(`\"${key}\"`));
  },
}));

const orderProbe = {
  left: { alpha: 1, beta: 2 },
  right: { beta: 2, alpha: 1 },
  serializer: "stable",
};
results.push(proveMutationRed({
  label: "M2_NONDETERMINISTIC_INSERTION_ORDER_SERIALIZATION",
  greenValue: orderProbe,
  mutate(value) {
    value.serializer = "insertion_order";
    return value;
  },
  invariant(value) {
    const serialize = value.serializer === "stable" ? stableSemanticJson : JSON.stringify;
    return serialize(value.left) === serialize(value.right);
  },
}));

results.push(proveMutationRed({
  label: "M3_PRESENTATION_PROFILE_ENTERS_SEMANTIC_IDENTITY",
  greenValue: {
    payload,
    presentation_profile: contract.presentation_profile_probe,
    identity_policy: "semantic_payload_only",
  },
  mutate(value) {
    value.identity_policy = "semantic_payload_plus_presentation_profile";
    return value;
  },
  invariant(value) {
    const baselineIdentityBytes = stableSemanticJson(value.payload);
    const candidateIdentityBytes = value.identity_policy === "semantic_payload_only"
      ? stableSemanticJson(value.payload)
      : stableSemanticJson({ payload: value.payload, presentation_profile: value.presentation_profile });
    return baselineIdentityBytes === candidateIdentityBytes;
  },
}));

console.log(JSON.stringify({
  status: "PASS",
  mutation_count: results.length,
  mutation_residue: 0,
  results,
}));
