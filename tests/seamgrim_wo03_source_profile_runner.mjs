import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { buildWorkbenchProjectArtifact, workbenchSourceProfile, selectUnversionedDocumentProfile } from "../solutions/seamgrim_ui_mvp/ui/workbench/persistence/project_adapter.js";

const profile = "v1-core-v25";
const artifact = await buildWorkbenchProjectArtifact({
  projectId: "explicit-test", projectName: "판본 검증", sourceProfileIdentity: profile,
  files: [{ path: "lesson.ddn", text: "값 <- 1. 값 보여주기." }],
});
const received = { portableSnapshot: JSON.parse(JSON.stringify(artifact.project_snapshot)) };
assert.equal(workbenchSourceProfile(received), profile, "export/import preserves explicit profile");
assert.equal(workbenchSourceProfile({ portableSnapshot: { ddn_text: "값 := 1." } }), "", "tokens never select a profile");
assert.equal(workbenchSourceProfile({ sourceProfileIdentity: profile, portableMetadata: { source_profile_identity: "legacy" } }), "", "conflicting identities fail closed");
assert.equal(workbenchSourceProfile({ resolvedRootSourceProfileIdentity: profile }), profile, "validated root supplies authority");
assert.equal(workbenchSourceProfile({ resolvedRootSourceProfileIdentity: profile, portableMetadata: { source_profile_identity: "legacy" } }), "", "metadata cannot override root");
await assert.rejects(buildWorkbenchProjectArtifact({ sourceProfileIdentity: profile, metadata: { source_profile_identity: "legacy" } }), /판본/);
const unbound = await buildWorkbenchProjectArtifact({ files: [{ path: "lesson.ddn", text: "값 := 1." }] });
assert.equal(workbenchSourceProfile({ portableSnapshot: unbound.project_snapshot }), "");
const ui = await readFile(new URL("../solutions/seamgrim_ui_mvp/ui/workbench/workbench.js", import.meta.url), "utf8");
assert.match(ui, /글판="v25"/);
assert.doesNotMatch(ui.slice(ui.indexOf("function nativeProjectFiles"), ui.indexOf("function buildNativeProject")), /프로젝트 \{|판본:|첫글:|목표 \{|모듈길/);
assert.match(ui, /!projectSource && sourceProfileIdentity !== SOURCE_ANALYSIS_PROFILE_IDENTITY/);
assert.match(ui, /const rootProfile = String\(prepared\?\.source_profile_identity/);
assert.match(ui, /sourceProfileIdentity: workbenchSourceProfile\(project\)/);
assert.match(ui, /실행 대상: \$\{String\(payload.root_path/);
assert.match(ui, /globalThis.confirm\("이 파일에는 문법 판본이 없습니다/);
const original = { portableSnapshot: { metadata: {}, ddn_text: "값 := 1." } };
const snapshotBytes = JSON.stringify(original.portableSnapshot);
selectUnversionedDocumentProfile(original, profile);
assert.equal(workbenchSourceProfile(original), profile);
assert.equal(JSON.stringify(original.portableSnapshot), snapshotBytes, "explicit choice preserves received bytes");
assert.throws(() => selectUnversionedDocumentProfile({ portableMetadata: { source_profile_identity: "legacy" } }, profile), /판본/);
const factorySource = ui.slice(ui.indexOf("function nativeProjectFiles"), ui.indexOf("function buildNativeProject"));
const nativeFiles = new Function("ddnQuotedText", "defaultOneFileDdnSource", `${factorySource}; return nativeProjectFiles;`)(String, () => "값 := 0.");
const empty = nativeFiles("빈 작업", "empty");
const physics = nativeFiles("물리", "physics");
assert.equal(empty.length, 2);
assert.equal(physics.length, 3);
assert.equal(nativeFiles("한 파일", "single").length, 1);
assert.match(empty[0].utf8_text, /시작=\(모듈=앱, 정의=실행\)/);
assert.match(empty[1].utf8_text, /실행:움직씨 :=/);
assert.match(physics[0].utf8_text, /이름=힘, 머리="lib\/힘.ddn", 원문="lib\/힘.ddn"/);
assert.match(physics[1].utf8_text, /실행:움직씨 :=/);
assert.match(physics[1].utf8_text, /\(힘=힘\) 쓰임\./, "import binds the declared local module, not a guessed path");
console.log("WO03_SOURCE_PROFILE_PASS roundtrip=1 missing=1 conflict=1 root_authority=1 no_content_inference=1");
