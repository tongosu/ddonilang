#!/usr/bin/env node

import assert from "node:assert/strict";
import {
  buildPortableProjectArtifact,
  parsePortableProjectArtifactBytes,
  serializePortableProjectArtifact,
  validatePortableProjectArtifact,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/persistence/portable_project_artifact.js";
import { stableProjectSnapshotJson } from "../solutions/seamgrim_ui_mvp/ui/project_snapshot.js";

const OK = "seamgrim_local_content_exchange_quality: ok";

async function sha256Hex(text) {
  const digest = await globalThis.crypto.subtle.digest("SHA-256", new TextEncoder().encode(String(text ?? "")));
  return Array.from(new Uint8Array(digest)).map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function cases() {
  return [
    {
      id: "one_file",
      projectName: "새 한 파일 DDN",
      sourceLessonId: "custom",
      ddnText: "설정 { 제목: \"새 한 파일 DDN\". }.\n\"one\" 보여주기.\n",
      metadata: { title: "새 한 파일 DDN", content_kinds: ["simulation"], tags: ["local"] },
    },
    {
      id: "projectile",
      projectName: "포물선 운동",
      sourceLessonId: "rep_phys_projectile_xy_v1",
      ddnText: "설정 { 제목: \"포물선\". 마디수: 12. }.\n\"projectile\" 보여주기.\n",
      metadata: { title: "포물선 운동", subject: "physics", tags: ["포물선"] },
    },
    {
      id: "pendulum",
      projectName: "진자 보기 코어",
      sourceLessonId: "rep_phys_pendulum_wave1_v1",
      ddnText: "설정 { 제목: \"진자\". 마디수: 12. }.\n\"pendulum\" 보여주기.\n",
      controlsSnapshot: { selected_view: "overlay", run_mode: "bogae-main" },
      metadata: { title: "진자 보기 코어", required_views: ["primary", "overlay"] },
    },
    {
      id: "grid",
      projectName: "격자 상태와 낙하 조각",
      sourceLessonId: "rep_grid_game_state_drop_v1",
      ddnText: "설정 { 제목: \"격자\". 마디수: 4. }.\n\"grid\" 보여주기.\n",
      localPackagePayload: { manifest: { package_hash: "candidate:grid" }, files: ["space2d.json", "table.json"] },
      metadata: { title: "격자 상태와 낙하 조각", content_kinds: ["game"], required_gaji: ["std_grid"] },
    },
    {
      id: "curriculum_aux",
      projectName: "수업계획 포함 프로젝트",
      sourceLessonId: "custom",
      ddnText: "설정 { 제목: \"수업계획\". }.\n\"lesson\" 보여주기.\n",
      lessonPlanText: "활동 1: 관찰\n활동 2: 설명\n",
      curriculumInquiryArtifact: { question: "왜 같은가?" },
      curriculumArtifactBundle: { cards: [{ id: "source", title: "원문" }] },
      curriculumModelComparison: { pair: ["imperative", "seum"] },
      curriculumSemanticScene: { scene: "line" },
      metadata: { title: "수업계획 포함 프로젝트", content_kinds: ["lesson"], tags: ["curriculum"] },
    },
  ];
}

async function assertRoundtrip(input) {
  const built = await buildPortableProjectArtifact(input);
  const parsed = await parsePortableProjectArtifactBytes(built.file.json_text);
  assert.equal(parsed.artifact.content_sha256, built.artifact.content_sha256, `${input.id}: parse sha mismatch`);
  assert.equal(parsed.artifact.canonical_json, built.artifact.canonical_json, `${input.id}: canonical mismatch`);
  const serialized = serializePortableProjectArtifact(parsed.artifact);
  assert.equal(serialized.json_text, built.file.json_text, `${input.id}: wrapper serialization changed`);
  const reparsed = await parsePortableProjectArtifactBytes(serialized.bytes);
  assert.deepEqual(reparsed.artifact.project_snapshot.metadata, built.artifact.project_snapshot.metadata, `${input.id}: metadata loss`);
  for (const field of [
    "lesson_plan_text",
    "controls_snapshot",
    "local_package_payload",
    "curriculum_inquiry_artifact",
    "curriculum_artifact_bundle",
    "curriculum_model_comparison",
    "curriculum_semantic_scene",
    "lineage",
    "metadata",
    "boundaries",
  ]) {
    assert.deepEqual(reparsed.artifact.project_snapshot[field], built.artifact.project_snapshot[field], `${input.id}: field loss ${field}`);
  }
  const edited = await buildPortableProjectArtifact({
    ...input,
    ddnText: `${input.ddnText}\n\"edited\" 보여주기.\n`,
  });
  assert.notEqual(edited.file.content_sha256, built.file.content_sha256, `${input.id}: edit did not change hash`);
  return built.file.content_sha256;
}

async function assertUnknownFieldPreserved() {
  const built = await buildPortableProjectArtifact(cases()[0]);
  const artifact = structuredClone(built.artifact);
  artifact.project_snapshot.future_snapshot_field = { keep: true, nested: ["yes"] };
  artifact.canonical_json = stableProjectSnapshotJson(artifact.project_snapshot);
  artifact.content_sha256 = await sha256Hex(artifact.canonical_json);
  artifact.canonical_byte_length = new TextEncoder().encode(artifact.canonical_json).length;
  const validation = await validatePortableProjectArtifact(artifact);
  assert.deepEqual(validation.field_audit.unknown_snapshot_fields, ["future_snapshot_field"]);
  const parsed = await parsePortableProjectArtifactBytes(serializePortableProjectArtifact(artifact).bytes);
  assert.deepEqual(parsed.artifact.project_snapshot.future_snapshot_field, { keep: true, nested: ["yes"] });
}

async function main() {
  const hashes = [];
  for (const input of cases()) {
    hashes.push(await assertRoundtrip(input));
  }
  assert.equal(new Set(hashes).size, hashes.length, "representative hashes should be distinct");
  await assertUnknownFieldPreserved();
  console.log(JSON.stringify({
    schema: "seamgrim.local_content_exchange_quality.runner_result.v1",
    cases: cases().map((item) => item.id),
    distinct_hashes: hashes.length,
    unknown_field_preserved: true,
  }));
  console.log(OK);
}

main().catch((err) => {
  console.error(err?.stack || err);
  process.exit(1);
});

