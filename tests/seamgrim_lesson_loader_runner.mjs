import fs from "fs/promises";
import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function readJson(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  return JSON.parse(text);
}

async function main() {
  const root = process.cwd();
  const contractPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js");
  const contractMod = await import(pathToFileURL(contractPath).href);
  const {
    parseLessonDdnMetaHeader,
    resolveLessonDisplayMeta,
    buildLessonSelectionSnapshot,
  } = contractMod;

  assert(typeof parseLessonDdnMetaHeader === "function", "lesson loader export: parseLessonDdnMetaHeader");
  assert(typeof resolveLessonDisplayMeta === "function", "lesson loader export: resolveLessonDisplayMeta");
  assert(typeof buildLessonSelectionSnapshot === "function", "lesson loader export: buildLessonSelectionSnapshot");

  const casePath = path.resolve(root, "pack/seamgrim_lesson_loader_basics/c01_load/case.detjson");
  const caseDoc = await readJson(casePath);
  assert(
    String(caseDoc?.schema ?? "") === "ddn.seamgrim.lesson_loader_case.v1",
    "lesson loader pack: schema",
  );
  const rows = Array.isArray(caseDoc?.cases) ? caseDoc.cases : [];
  assert(rows.length >= 2, "lesson loader pack: at least 2 cases");

  for (let i = 0; i < rows.length; i += 1) {
    const row = rows[i] ?? {};
    const ddnPath = path.resolve(root, String(row.ddn_path ?? ""));
    const expectedPath = path.resolve(root, String(row.expected_path ?? ""));
    const ddnText = await fs.readFile(ddnPath, "utf8");
    const expected = await readJson(expectedPath);
    const ddnMeta = parseLessonDdnMetaHeader(ddnText);
    const displayMeta = resolveLessonDisplayMeta({
      baseTitle: row.base_title ?? row.lesson_id,
      baseDescription: "",
      tomlMeta: {},
      ddnMetaHeader: ddnMeta,
    });
    const actual = buildLessonSelectionSnapshot({
      id: row.lesson_id,
      title: displayMeta.title,
      requiredViews: row.required_views,
      ddnMetaHeader: ddnMeta,
    });
    const lhs = JSON.stringify(actual);
    const rhs = JSON.stringify(expected);
    assert(lhs === rhs, `lesson loader pack mismatch: case#${i + 1}`);
  }

  const noHeaderMeta = parseLessonDdnMetaHeader("(매마디)마다 { 보임 { t: 1. }. }.");
  assert(noHeaderMeta.hasAny === false, "lesson loader parse: no header");
  const headerWithRequiredViews = parseLessonDdnMetaHeader(
    "#이름: 테스트\n#필수보기: 2d, graph, text, 2d\n(매마디)마다 { 보임 { t: 1. }. }.",
  );
  assert(
    JSON.stringify(headerWithRequiredViews.required_views) === JSON.stringify(["space2d", "graph", "text"]),
    "lesson loader parse: ddn header required_views normalize",
  );
  const normalized = buildLessonSelectionSnapshot({
    id: "demo",
    title: "demo",
    requiredViews: ["2d", "graph", "2d"],
    ddnMetaHeader: { name: "n", desc: "d" },
  });
  assert(
    JSON.stringify(normalized.required_views) === JSON.stringify(["space2d", "graph"]),
    "lesson loader snapshot: required_views normalize",
  );
  const normalizedFromHeader = buildLessonSelectionSnapshot({
    id: "demo2",
    title: "demo2",
    ddnMetaHeader: { name: "n", desc: "d", required_views: ["3d", "graph", "3d"] },
  });
  assert(
    JSON.stringify(normalizedFromHeader.required_views) === JSON.stringify(["space3d", "graph"]),
    "lesson loader snapshot: required_views fallback from ddn header",
  );

  console.log("seamgrim lesson loader runner ok");
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
