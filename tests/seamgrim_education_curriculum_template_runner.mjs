import fs from "fs/promises";
import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function fakeElement() {
  const classes = new Set(["hidden"]);
  return {
    textContent: "",
    innerHTML: "",
    classList: {
      add(name) {
        classes.add(name);
      },
      remove(name) {
        classes.delete(name);
      },
      contains(name) {
        return classes.has(name);
      },
    },
  };
}

async function readJson(filePath) {
  const text = await fs.readFile(filePath, "utf8");
  return JSON.parse(text);
}

async function main() {
  const root = process.cwd();
  const loaderPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/lesson_loader_contract.js");
  const browsePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/browse.js");
  const loader = await import(pathToFileURL(loaderPath).href);
  const browse = await import(pathToFileURL(browsePath).href);

  assert(typeof loader.parseTomlMeta === "function", "parseTomlMeta export");
  assert(typeof browse.BrowseScreen === "function", "BrowseScreen export");

  const seedMetaPath = path.resolve(root, "solutions/seamgrim_ui_mvp/seed_lessons_v1/roguelike_grid_pathfind_v1/meta.toml");
  const seedMeta = loader.parseTomlMeta(await fs.readFile(seedMetaPath, "utf8"));
  assert(seedMeta.schema === "CurriculumMetaV1", "seed meta schema");
  assert(seedMeta.lesson_id === "roguelike_grid_pathfind_v1", "seed meta lesson_id");
  assert(seedMeta.subject === "game", "seed meta subject");
  assert(seedMeta.grade === "all", "seed meta grade alias");
  assert(seedMeta.required_views.includes("console_grid"), "seed meta console_grid view");
  assert(seedMeta.required_views.includes("grid2d"), "seed meta grid2d view");
  assert(seedMeta.learning_goals.length >= 2, "seed meta learning goals");

  const koreanMeta = loader.parseTomlMeta(`
schema = "CurriculumMetaV1"
lesson_id = "ko_alias_lesson"
title = "한국어 키 차시"
"과목" = "경제"
"학년군" = "중등"
"단원" = "수요와 공급"
"차시" = "1차시"
"난이도" = "기본"
"학습목표" = ["목표 A"]
"핵심개념" = ["개념 A"]
"선수개념" = []
"오개념" = []
"허용조작" = []
"필수계기판" = ["graph", "table"]
evidence = ["checker"]
defaults = { "최대마디" = "24" }
`);
  assert(koreanMeta.subject === "경제", "korean alias subject");
  assert(koreanMeta.grade === "중등", "korean alias grade");
  assert(koreanMeta.learning_goals[0] === "목표 A", "korean alias learning_goals");
  assert(JSON.stringify(koreanMeta.required_views) === JSON.stringify(["graph", "table"]), "korean alias required_views");
  assert(koreanMeta.defaults?.["최대마디"] === "24", "inline table parse");

  const manifestPath = path.resolve(root, "solutions/seamgrim_ui_mvp/seed_lessons_v1/seed_manifest.detjson");
  const manifest = await readJson(manifestPath);
  const roguelikeSeed = manifest.seeds.find((row) => row?.seed_id === "roguelike_grid_pathfind_v1");
  assert(roguelikeSeed, "roguelike seed manifest row");
  assert(String(roguelikeSeed.meta_toml ?? "").endsWith("/meta.toml"), "roguelike seed meta_toml");
  assert(manifest.featured_seed_ids.includes("roguelike_grid_pathfind_v1"), "roguelike featured seed still exposed");

  const screen = new browse.BrowseScreen({ root: { querySelector: () => null } });
  screen.detailPanelEl = fakeElement();
  screen.detailSubjectBadgeEl = fakeElement();
  screen.detailTitleEl = fakeElement();
  screen.detailDescEl = fakeElement();
  screen.detailKeywordsEl = fakeElement();
  screen.detailCurriculumEl = fakeElement();
  screen.showLessonDetail({
    id: "roguelike_grid_pathfind_v1",
    title: seedMeta.title,
    description: "meta detail",
    subject: seedMeta.subject,
    grade: seedMeta.grade,
    curriculumMeta: {
      unit: seedMeta.unit,
      lesson: seedMeta.lesson,
      difficulty: seedMeta.difficulty,
      learningGoals: seedMeta.learning_goals,
      coreConcepts: seedMeta.core_concepts,
      requiredViews: seedMeta.required_views,
      teacherNotesRef: seedMeta.teacher_notes_ref,
      studentSheetRef: seedMeta.student_sheet_ref,
    },
  });
  assert(!screen.detailPanelEl.classList.contains("hidden"), "detail panel visible");
  assert(screen.detailCurriculumEl.innerHTML.includes("학습목표"), "detail learning goals title");
  assert(screen.detailCurriculumEl.innerHTML.includes("벽이 있는 격자"), "detail learning goal content");
  assert(screen.detailCurriculumEl.innerHTML.includes("핵심개념"), "detail core concepts title");
  assert(screen.detailCurriculumEl.innerHTML.includes("std_grid"), "detail core concept content");
  assert(screen.detailCurriculumEl.innerHTML.includes("필수보기: console_grid, grid2d"), "detail required views");

  console.log("seamgrim education curriculum template runner ok");
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
