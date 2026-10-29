#!/usr/bin/env node

import assert from "node:assert/strict";
import {
  auditPortableSnapshotFieldPreservation,
  buildPortableProjectArtifact,
  MAX_PORTABLE_PROJECT_BYTES,
  parsePortableProjectArtifactBytes,
  portableProjectDiagnostic,
  safePortableProjectFilename,
  serializePortableProjectArtifact,
  validatePortableProjectArtifact,
} from "../solutions/seamgrim_ui_mvp/ui/workbench/persistence/portable_project_artifact.js";

function sampleInput(extra = {}) {
  return {
    projectName: "포물선 <운동> / 테스트",
    ddnText: "설정 { 제목: \"포물선\". }.\n\"ok\" 보여주기.\n",
    lessonPlanText: "활동: 관찰\n",
    sourceLessonId: "rep_phys_projectile_xy_v1",
    controlsSnapshot: {
      selected_view: "primary",
      view_workspace: {
        schema: "seamgrim.workbench.view_workspace.v1",
        sequence: 1,
        primary_view_instance_id: "view_primary_1",
        active_view_instance_id: "view_primary_1",
        selected_target: null,
        instances: [
          {
            schema: "seamgrim.workbench.view_instance.v1",
            view_instance_id: "view_primary_1",
            role: "primary",
            family: "projectile",
            target_view_instance_id: "",
            target: null,
            temporary: false,
            render_surface: "primary_canvas",
          },
        ],
      },
      run_mode: "ddn-main",
      controls: [{ name: "g", value: 9.8 }],
      view_authoring: { camera: "fit" },
    },
    localPackagePayload: {
      manifest: { package_hash: "candidate:package:001" },
      rows: [{ path: "lesson.ddn" }],
    },
    curriculumInquiryArtifact: { prompt: "observe" },
    curriculumArtifactBundle: { cards: ["source"] },
    curriculumModelComparison: { models: ["imperative", "seum"] },
    curriculumSemanticScene: { view: "projectile" },
    lineage: {
      source_lesson_id: "rep_phys_projectile_xy_v1",
      origin_kind: "official_lesson",
      origin_content_sha256: "origin-001",
    },
    metadata: {
      title: "포물선 운동",
      summary: "가져오기 검증용",
      tags: ["물리", "portable"],
      content_kinds: ["simulation"],
    },
    ...extra,
  };
}

async function expectCode(label, fn, code) {
  try {
    await fn();
  } catch (err) {
    assert.equal(err.code, code, `${label}: expected ${code}, got ${err.code || err.message}`);
    const diag = portableProjectDiagnostic(err);
    assert.equal(diag.code, code);
    assert.equal(diag.severity, "error");
    return;
  }
  throw new Error(`${label}: expected ${code}`);
}

async function main() {
  const built = await buildPortableProjectArtifact(sampleInput());
  assert.equal(built.schema, "seamgrim.portable_project_artifact_contract.v1");
  assert.equal(built.artifact.schema, "seamgrim.project_snapshot_artifact.v0");
  assert.match(built.file.filename, /^포물선-운동-테스트\.seamgrim-project\.json$/u);
  assert.ok(built.file.byte_length > 100);
  assert.ok(built.file.byte_length <= MAX_PORTABLE_PROJECT_BYTES);

  const rebuilt = await buildPortableProjectArtifact(sampleInput());
  assert.equal(rebuilt.file.json_text, built.file.json_text, "same input must export byte-identical JSON text");

  const parsed = await parsePortableProjectArtifactBytes(built.file.json_text);
  assert.equal(parsed.validation.content_sha256, built.artifact.content_sha256);
  assert.deepEqual(parsed.artifact.project_snapshot.curriculum_artifact_bundle, { cards: ["source"] });
  assert.deepEqual(parsed.artifact.project_snapshot.metadata.tags, ["물리", "portable"]);
  assert.equal(
    parsed.artifact.project_snapshot.controls_snapshot.view_workspace.schema,
    "seamgrim.workbench.view_workspace.v1",
  );

  const unknown = structuredClone(built.artifact);
  unknown.project_snapshot.experimental_future_field = { keep: true };
  unknown.canonical_json = JSON.stringify(JSON.parse(serializePortableProjectArtifact({
    schema: unknown.project_snapshot.schema,
  }).json_text));
  unknown.canonical_json = JSON.stringify(Object.keys(unknown.project_snapshot).sort().reduce((out, key) => {
    out[key] = unknown.project_snapshot[key];
    return out;
  }, {}));
  // Use the product serializer instead of the ad hoc line above to preserve deep ordering.
  unknown.canonical_json = (await import("../solutions/seamgrim_ui_mvp/ui/project_snapshot.js"))
    .stableProjectSnapshotJson(unknown.project_snapshot);
  const cryptoDigest = await globalThis.crypto.subtle.digest("SHA-256", new TextEncoder().encode(unknown.canonical_json));
  unknown.content_sha256 = Array.from(new Uint8Array(cryptoDigest)).map((byte) => byte.toString(16).padStart(2, "0")).join("");
  unknown.canonical_byte_length = new TextEncoder().encode(unknown.canonical_json).length;
  const unknownValidation = await validatePortableProjectArtifact(unknown);
  assert.deepEqual(unknownValidation.field_audit.unknown_snapshot_fields, ["experimental_future_field"]);

  const audit = auditPortableSnapshotFieldPreservation(built.artifact.project_snapshot);
  assert.equal(audit.missing_current_fields.length, 0);
  assert.equal(audit.unknown_fields_preserved_by_validation, true);

  assert.equal(safePortableProjectFilename(" a/b:c*d?e "), "a-b-c-d-e.seamgrim-project.json");

  const wrongWrapper = structuredClone(built.artifact);
  wrongWrapper.schema = "wrong";
  await expectCode("wrong wrapper", () => validatePortableProjectArtifact(wrongWrapper), "E_PORTABLE_WRAPPER_SCHEMA");

  const wrongSnapshot = structuredClone(built.artifact);
  wrongSnapshot.project_snapshot.schema = "wrong";
  wrongSnapshot.canonical_json = "{}";
  await expectCode("wrong snapshot", () => validatePortableProjectArtifact(wrongSnapshot), "E_PORTABLE_SNAPSHOT_SCHEMA");

  const canonicalMismatch = structuredClone(built.artifact);
  canonicalMismatch.canonical_json = "{}";
  await expectCode("canonical mismatch", () => validatePortableProjectArtifact(canonicalMismatch), "E_PORTABLE_CANONICAL_MISMATCH");

  const shaMismatch = structuredClone(built.artifact);
  shaMismatch.content_sha256 = "0".repeat(64);
  await expectCode("sha mismatch", () => validatePortableProjectArtifact(shaMismatch), "E_PORTABLE_SHA_MISMATCH");

  const extraWrapper = structuredClone(built.artifact);
  extraWrapper.unsupported_extra = true;
  await expectCode("extra wrapper field", () => validatePortableProjectArtifact(extraWrapper), "E_PORTABLE_WRAPPER_FIELD");

  await expectCode("too large", () => parsePortableProjectArtifactBytes(new Uint8Array(MAX_PORTABLE_PROJECT_BYTES + 1)), "E_PORTABLE_FILE_TOO_LARGE");
  await expectCode("invalid utf8", () => parsePortableProjectArtifactBytes(new Uint8Array([0xc3, 0x28])), "E_PORTABLE_FILE_UTF8");
  await expectCode("invalid json", () => parsePortableProjectArtifactBytes("{"), "E_PORTABLE_FILE_JSON");

  const forbidden = await buildPortableProjectArtifact(sampleInput({ metadata: { capability: "sg_cap_secret" } })).catch((err) => err);
  assert.equal(forbidden.code, "E_PORTABLE_FORBIDDEN_FIELD");

  const protoJson = `{"schema":"seamgrim.project_snapshot_artifact.v0","__proto__":{},"project_snapshot":{},"canonical_json":"{}","content_sha256":"","canonical_byte_length":2}`;
  await expectCode("prototype key", () => parsePortableProjectArtifactBytes(protoJson), "E_PORTABLE_PROTOTYPE_KEY");

  console.log("seamgrim_portable_project_artifact: ok");
}

main().catch((err) => {
  console.error(err?.stack || err);
  process.exit(1);
});
