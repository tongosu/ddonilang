#!/usr/bin/env node

import crypto from "node:crypto";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const snapshotUrl = pathToFileURL(path.resolve(ROOT, "solutions/seamgrim_ui_mvp/ui/project_snapshot.js")).href;
const packageUrl = pathToFileURL(path.resolve(ROOT, "solutions/seamgrim_ui_mvp/ui/studio_local_share_package.js")).href;

const {
  PROJECT_SNAPSHOT_SCHEMA,
  PROJECT_SNAPSHOT_WRAPPER_SCHEMA,
  assertProjectSnapshotArtifactV0,
  buildProjectSnapshotArtifactV0,
  buildProjectSnapshotInputFromEditorState,
  stableProjectSnapshotJson,
} = await import(snapshotUrl);
const {
  buildStudioLocalPackagePayload,
  importStudioLocalPackagePayload,
  validateStudioLocalPackagePayload,
} = await import(packageUrl);

const editedDdn = `설정 {
  제목: 나의_진자_실험.
}.

1 보여주기.
`;
const lessonPlanText = `[lesson]
id = "rep_phys_pendulum_wave1_v1"
title = "나의 진자 실험"

[[activities]]
id = "a1"
title = "각도 관찰"
primary_view = "space2d"
`;

const localPackagePayload = buildStudioLocalPackagePayload({
  lessons: [{
    lesson_id: "rep_phys_pendulum_wave1_v1",
    title: "나의 진자 실험",
    required_views: ["space2d", "graph"],
    source_text: editedDdn,
    path: "lessons/rep_phys_pendulum_wave1_v1.ddn",
  }],
  reports: [{
    report_id: "lesson_plan_v0",
    title: "lesson_plan.v0",
    path: "reports/lesson_plan.v0.toml",
    mime: "text/plain; charset=utf-8",
    text: lessonPlanText,
  }],
});

const validation = validateStudioLocalPackagePayload(localPackagePayload);
assert(validation.valid === true, "local package should validate");
const imported = importStudioLocalPackagePayload(localPackagePayload);
assert(imported.package_hash === localPackagePayload.package_hash, "local package hash roundtrip");
assert(imported.lessons[0].source_text === editedDdn, "local package lesson text roundtrip");

const snapshotInput = {
  projectName: "나의 진자 실험",
  ddnText: editedDdn,
  lessonPlanText,
  sourceLessonId: "rep_phys_pendulum_wave1_v1",
  controlsSnapshot: {
    view_authoring: { camera_policy: "follow", component: "angle" },
    controls: [{ name: "theta0", value: 0.4, min: -1.2, max: 1.2, step: 0.05 }],
    selected_view: "space2d",
  },
  localPackagePayload,
  lineage: {
    source_lesson_id: "rep_phys_pendulum_wave1_v1",
    source_package_hash: localPackagePayload.package_hash,
  },
};

const artifactA = await buildProjectSnapshotArtifactV0(snapshotInput);
const artifactB = await buildProjectSnapshotArtifactV0({
  ...snapshotInput,
  controlsSnapshot: {
    selected_view: "space2d",
    controls: [{ step: 0.05, max: 1.2, min: -1.2, value: 0.4, name: "theta0" }],
    view_authoring: { component: "angle", camera_policy: "follow" },
  },
});

assert(artifactA.schema === PROJECT_SNAPSHOT_WRAPPER_SCHEMA, "wrapper schema");
assert(artifactA.project_snapshot.schema === PROJECT_SNAPSHOT_SCHEMA, "snapshot schema");
assert(artifactA.canonical_json === artifactB.canonical_json, "canonical bytes stable across object key order");
assert(artifactA.content_sha256 === artifactB.content_sha256, "sha stable across object key order");
assert(/^[0-9a-f]{64}$/.test(artifactA.content_sha256), "sha256 hex shape");
assert(
  artifactA.content_sha256 === crypto.createHash("sha256").update(artifactA.canonical_json, "utf8").digest("hex"),
  "sha256 over canonical utf8 json",
);
await assertProjectSnapshotArtifactV0(artifactA);

const changed = await buildProjectSnapshotArtifactV0({
  ...snapshotInput,
  ddnText: `${editedDdn}\n2 보여주기.\n`,
});
assert(changed.content_sha256 !== artifactA.content_sha256, "ddn changes hash");

const editorInput = buildProjectSnapshotInputFromEditorState({
  editorScreen: {
    getDdn: () => editedDdn,
    getLessonPlanText: () => lessonPlanText,
  },
  appState: {
    currentLesson: {
      id: "rep_phys_pendulum_wave1_v1",
      title: "오래된 제목",
      ddnText: "0 보여주기.",
      lessonPlanText: "old",
      localPackageHash: localPackagePayload.package_hash,
    },
  },
  projectName: "현재 편집기 값",
  localPackagePayload,
});
const editorArtifact = await buildProjectSnapshotArtifactV0(editorInput);
assert(editorArtifact.project_snapshot.ddn_text === editedDdn, "current editor DDN captured");
assert(editorArtifact.project_snapshot.lesson_plan_text === lessonPlanText, "current lesson plan captured");
assert(
  editorArtifact.project_snapshot.ddn_text !== "0 보여주기.",
  "stale currentLesson DDN must not win",
);
assert(editorArtifact.project_snapshot.local_package_payload.package_hash === localPackagePayload.package_hash, "package embedded");
assert(stableProjectSnapshotJson(editorArtifact.project_snapshot) === editorArtifact.canonical_json, "stable json helper");

console.log(JSON.stringify({
  schema: "seamgrim.project_snapshot.runner_result.v1",
  content_sha256: artifactA.content_sha256,
  canonical_byte_length: artifactA.canonical_byte_length,
  package_hash: localPackagePayload.package_hash,
  edited_value_beats_stale_currentLesson: true,
}));
