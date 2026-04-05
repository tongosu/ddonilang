import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/play_source_contract.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    buildPlayLessonCandidates,
    resolvePlayExampleRequest,
    resolvePlayLaunchRequest,
    resolvePlayLessonRequest,
  } = mod;

  assert(typeof buildPlayLessonCandidates === "function", "play source contract: buildPlayLessonCandidates export");
  assert(typeof resolvePlayExampleRequest === "function", "play source contract: resolvePlayExampleRequest export");
  assert(typeof resolvePlayLaunchRequest === "function", "play source contract: resolvePlayLaunchRequest export");
  assert(typeof resolvePlayLessonRequest === "function", "play source contract: resolvePlayLessonRequest export");

  const directCandidates = buildPlayLessonCandidates("lessons/foo/lesson.ddn");
  assert(
    JSON.stringify(directCandidates) === JSON.stringify([
      "/lessons/foo/lesson.ddn",
      "/solutions/seamgrim_ui_mvp/lessons/foo/lesson.ddn",
    ]),
    "play source contract: default lesson candidates",
  );

  const traversalCandidates = buildPlayLessonCandidates("../secrets/lesson.ddn");
  assert(
    traversalCandidates.length === 0,
    "play source contract: traversal lesson candidates blocked",
  );

  const remoteCandidates = buildPlayLessonCandidates("https://example.com/lesson.ddn");
  assert(
    JSON.stringify(remoteCandidates) === JSON.stringify(["https://example.com/lesson.ddn"]),
    "play source contract: remote lesson candidate passthrough",
  );

  const prefixedCandidates = buildPlayLessonCandidates("lessons/foo/lesson.ddn", { projectPrefixedHost: true });
  assert(
    JSON.stringify(prefixedCandidates) === JSON.stringify([
      "/solutions/seamgrim_ui_mvp/lessons/foo/lesson.ddn",
      "/lessons/foo/lesson.ddn",
    ]),
    "play source contract: prefixed host candidate order",
  );

  const lessonPathRequest = resolvePlayLessonRequest({
    pathname: "/solutions/seamgrim_ui_mvp/ui/play.html",
    search: "?lesson=lessons/abc/lesson.ddn",
  });
  assert(lessonPathRequest.requested === true, "play source contract: lesson path requested");
  assert(lessonPathRequest.lessonPath === "lessons/abc/lesson.ddn", "play source contract: lesson path normalized");
  assert(
    lessonPathRequest.candidates[0] === "/solutions/seamgrim_ui_mvp/lessons/abc/lesson.ddn",
    "play source contract: lesson path candidate first on prefixed host",
  );

  const lessonIdRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?lessonId=demo_lesson",
  });
  assert(lessonIdRequest.requested === true, "play source contract: lesson id requested");
  assert(lessonIdRequest.lessonPath === "lessons/demo_lesson/lesson.ddn", "play source contract: lesson id path");
  assert(lessonIdRequest.sourceLabel === "lesson:demo_lesson", "play source contract: lesson id label");

  const emptyRequest = resolvePlayLessonRequest({ pathname: "/ui/play.html", search: "" });
  assert(emptyRequest.requested === false, "play source contract: empty request");

  const hashLessonRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "",
    hash: "#lesson=lessons/hash_case/lesson.ddn",
  });
  assert(hashLessonRequest.requested === true, "play source contract: hash lesson requested");
  assert(hashLessonRequest.lessonPath === "lessons/hash_case/lesson.ddn", "play source contract: hash lesson path");
  assert(hashLessonRequest.sourceLabel === "lesson:lessons/hash_case/lesson.ddn", "play source contract: hash lesson label");

  const hashLessonIdRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "",
    hash: "#lessonId=hash_demo",
  });
  assert(hashLessonIdRequest.requested === true, "play source contract: hash lesson id requested");
  assert(hashLessonIdRequest.lessonPath === "lessons/hash_demo/lesson.ddn", "play source contract: hash lesson id path");
  assert(hashLessonIdRequest.sourceLabel === "lesson:hash_demo", "play source contract: hash lesson id label");

  const lessonPrecedenceRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?lesson=lessons/query_case/lesson.ddn",
    hash: "#lesson=lessons/hash_case/lesson.ddn",
  });
  assert(lessonPrecedenceRequest.requested === true, "play source contract: lesson precedence requested");
  assert(
    lessonPrecedenceRequest.lessonPath === "lessons/query_case/lesson.ddn",
    "play source contract: query lesson takes precedence over hash",
  );

  const sourceLessonPathRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson:lessons/from_source/lesson.ddn",
  });
  assert(sourceLessonPathRequest.requested === true, "play source contract: source lesson path requested");
  assert(
    sourceLessonPathRequest.lessonPath === "lessons/from_source/lesson.ddn",
    "play source contract: source lesson path normalized",
  );
  assert(
    sourceLessonPathRequest.sourceLabel === "lesson:lessons/from_source/lesson.ddn",
    "play source contract: source lesson path label",
  );

  const sourceLessonIdRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson:from_source_id",
  });
  assert(sourceLessonIdRequest.requested === true, "play source contract: source lesson id requested");
  assert(
    sourceLessonIdRequest.lessonPath === "lessons/from_source_id/lesson.ddn",
    "play source contract: source lesson id path",
  );
  assert(
    sourceLessonIdRequest.sourceLabel === "lesson:from_source_id",
    "play source contract: source lesson id label",
  );

  const sourceLessonIdExplicitRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lessonId:explicit_source_id",
  });
  assert(sourceLessonIdExplicitRequest.requested === true, "play source contract: source explicit lesson id requested");
  assert(
    sourceLessonIdExplicitRequest.lessonPath === "lessons/explicit_source_id/lesson.ddn",
    "play source contract: source explicit lesson id path",
  );

  const sourceLessonPathExplicitRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lessonPath:lessons/explicit_path/lesson.ddn",
  });
  assert(
    sourceLessonPathExplicitRequest.requested === true,
    "play source contract: source explicit lesson path requested",
  );
  assert(
    sourceLessonPathExplicitRequest.lessonPath === "lessons/explicit_path/lesson.ddn",
    "play source contract: source explicit lesson path",
  );

  const sourceLessonPathSnakeRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson_path:lessons/explicit_path_snake/lesson.ddn",
  });
  assert(
    sourceLessonPathSnakeRequest.requested === true,
    "play source contract: source explicit lesson_path requested",
  );
  assert(
    sourceLessonPathSnakeRequest.lessonPath === "lessons/explicit_path_snake/lesson.ddn",
    "play source contract: source explicit lesson_path",
  );

  const hashSourceLessonRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "",
    hash: "#source=lesson:lessons/from_hash/lesson.ddn",
  });
  assert(hashSourceLessonRequest.requested === true, "play source contract: hash source lesson requested");
  assert(
    hashSourceLessonRequest.lessonPath === "lessons/from_hash/lesson.ddn",
    "play source contract: hash source lesson path",
  );

  const sourceLessonPrecedenceRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?lesson=lessons/query_explicit/lesson.ddn&source=lesson:lessons/query_source/lesson.ddn",
  });
  assert(sourceLessonPrecedenceRequest.requested === true, "play source contract: source lesson precedence requested");
  assert(
    sourceLessonPrecedenceRequest.lessonPath === "lessons/query_explicit/lesson.ddn",
    "play source contract: explicit lesson query takes precedence over source lesson",
  );

  const remoteLessonRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?lesson=https://example.com/remote_lesson.ddn",
  });
  assert(remoteLessonRequest.requested === true, "play source contract: remote lesson requested");
  assert(
    remoteLessonRequest.lessonPath === "https://example.com/remote_lesson.ddn",
    "play source contract: remote lesson path preserved",
  );
  assert(
    remoteLessonRequest.sourceLabel === "lesson:https://example.com/remote_lesson.ddn",
    "play source contract: remote lesson source label preserved",
  );

  const remoteSourceLessonRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson:https://example.com/source_remote_lesson.ddn",
  });
  assert(remoteSourceLessonRequest.requested === true, "play source contract: remote source lesson requested");
  assert(
    remoteSourceLessonRequest.lessonPath === "https://example.com/source_remote_lesson.ddn",
    "play source contract: remote source lesson path preserved",
  );
  assert(
    remoteSourceLessonRequest.sourceLabel === "lesson:https://example.com/source_remote_lesson.ddn",
    "play source contract: remote source lesson source label preserved",
  );

  const traversalLessonRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?lesson=../secrets/lesson.ddn",
  });
  assert(
    traversalLessonRequest.requested === false,
    "play source contract: traversal lesson request blocked",
  );

  const traversalSourceLessonRequest = resolvePlayLessonRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson:../secrets/lesson.ddn",
  });
  assert(
    traversalSourceLessonRequest.requested === false,
    "play source contract: traversal source lesson blocked",
  );

  const sourceExampleRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?source=example:fibonacci",
  });
  assert(sourceExampleRequest.requested === true, "play source contract: source example requested");
  assert(sourceExampleRequest.exampleKey === "fibonacci", "play source contract: source example key");
  assert(sourceExampleRequest.sourceLabel === "example:fibonacci", "play source contract: source example label");

  const sourceExampleCaseRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?source=Example:%20Counter",
  });
  assert(sourceExampleCaseRequest.requested === true, "play source contract: source example case-insensitive requested");
  assert(
    sourceExampleCaseRequest.exampleKey === "counter",
    "play source contract: source example case-insensitive key",
  );

  const sourceExampleIdRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?source=exampleId:pendulum",
  });
  assert(
    sourceExampleIdRequest.requested === true,
    "play source contract: source exampleId requested",
  );
  assert(
    sourceExampleIdRequest.exampleKey === "pendulum",
    "play source contract: source exampleId key",
  );

  const queryExampleRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?example=Counter",
  });
  assert(queryExampleRequest.requested === true, "play source contract: query example requested");
  assert(queryExampleRequest.exampleKey === "counter", "play source contract: query example normalized");
  assert(queryExampleRequest.sourceLabel === "example:counter", "play source contract: query example label");

  const hashExampleRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "",
    hash: "#source=example:pendulum",
  });
  assert(hashExampleRequest.requested === true, "play source contract: hash example requested");
  assert(hashExampleRequest.exampleKey === "pendulum", "play source contract: hash example key");
  assert(hashExampleRequest.sourceLabel === "example:pendulum", "play source contract: hash example label");

  const invalidExampleRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?source=example:!@#$",
  });
  assert(invalidExampleRequest.requested === false, "play source contract: invalid example ignored");

  const lessonSourceNotExampleRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson:demo",
  });
  assert(
    lessonSourceNotExampleRequest.requested === false,
    "play source contract: lesson source must not be parsed as example",
  );

  const hashFallbackWhenQuerySourceIsLessonRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?source=lesson:demo",
    hash: "#source=example:fibonacci",
  });
  assert(
    hashFallbackWhenQuerySourceIsLessonRequest.requested === true,
    "play source contract: hash example fallback when query source is lesson",
  );
  assert(
    hashFallbackWhenQuerySourceIsLessonRequest.exampleKey === "fibonacci",
    "play source contract: hash example key when query source is lesson",
  );
  assert(
    hashFallbackWhenQuerySourceIsLessonRequest.sourceScope === "hash",
    "play source contract: hash scope when query source is lesson",
  );

  const examplePrecedenceRequest = resolvePlayExampleRequest({
    pathname: "/ui/play.html",
    search: "?example=counter",
    hash: "#source=example:fibonacci",
  });
  assert(examplePrecedenceRequest.requested === true, "play source contract: example precedence requested");
  assert(
    examplePrecedenceRequest.exampleKey === "counter",
    "play source contract: query example takes precedence over hash",
  );

  const launchLessonRequest = resolvePlayLaunchRequest({
    pathname: "/ui/play.html",
    search: "?lesson=lessons/launch_lesson/lesson.ddn&example=fibonacci",
  });
  assert(launchLessonRequest.kind === "lesson", "play source contract: launch prefers lesson");
  assert(
    launchLessonRequest.lesson.lessonPath === "lessons/launch_lesson/lesson.ddn",
    "play source contract: launch lesson path",
  );

  const launchExampleRequest = resolvePlayLaunchRequest({
    pathname: "/ui/play.html",
    search: "?source=example:fibonacci",
  });
  assert(launchExampleRequest.kind === "example", "play source contract: launch picks example");
  assert(
    launchExampleRequest.example.exampleKey === "fibonacci",
    "play source contract: launch example key",
  );

  const launchQueryExampleVsHashLessonRequest = resolvePlayLaunchRequest({
    pathname: "/ui/play.html",
    search: "?source=example:counter",
    hash: "#lesson=lessons/hash_wins_if_bug/lesson.ddn",
  });
  assert(
    launchQueryExampleVsHashLessonRequest.kind === "example",
    "play source contract: launch keeps query example over hash lesson",
  );
  assert(
    launchQueryExampleVsHashLessonRequest.example.exampleKey === "counter",
    "play source contract: launch query example key over hash lesson",
  );

  const launchQueryLessonVsHashExampleRequest = resolvePlayLaunchRequest({
    pathname: "/ui/play.html",
    search: "?lesson=lessons/query_wins/lesson.ddn",
    hash: "#source=example:fibonacci",
  });
  assert(
    launchQueryLessonVsHashExampleRequest.kind === "lesson",
    "play source contract: launch keeps query lesson over hash example",
  );
  assert(
    launchQueryLessonVsHashExampleRequest.lesson.lessonPath === "lessons/query_wins/lesson.ddn",
    "play source contract: launch query lesson path over hash example",
  );

  const launchNoneRequest = resolvePlayLaunchRequest({
    pathname: "/ui/play.html",
    search: "",
    hash: "",
  });
  assert(launchNoneRequest.kind === "none", "play source contract: launch none");

  console.log("seamgrim play source contract ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
