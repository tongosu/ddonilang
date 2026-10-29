// Existing project metadata and source loading helpers; original defaults, text and lookup rules retained.
export function createProjectMetadata({ currentProject, ONE_FILE_DDN_PROJECT_ID, state, workbenchSourceProfile }) {
function inferContentKinds(project = currentProject()) {
  const text = [project.kind, project.primaryView, ...(project.tags ?? [])].join(" ").toLowerCase();
  if (text.includes("게임") || text.includes("game") || text.includes("grid")) return ["game"];
  if (text.includes("manim")) return ["korean_manim"];
  if (text.includes("ai")) return ["ai_gym"];
  if (text.includes("증명") || text.includes("시험")) return ["test_or_proof"];
  if (text.includes("수업") || text.includes("교과")) return ["lesson"];
  return ["simulation"];
}

function inferSubject(project = currentProject()) {
  const text = [project.kind, project.title, project.summary, ...(project.tags ?? [])].join(" ").toLowerCase();
  if (text.includes("물리") || text.includes("phys") || text.includes("진자") || text.includes("포물선")) return "physics";
  if (text.includes("수학") || text.includes("math") || text.includes("일차함수")) return "math";
  if (text.includes("게임") || text.includes("grid")) return "game";
  return "general";
}

function defaultPortableMetadata(project = currentProject()) {
  return {
    source_profile_identity: workbenchSourceProfile(project),
    title: project.title,
    summary: project.summary,
    content_kinds: inferContentKinds(project),
    tags: [...(project.tags ?? [])],
    subject: inferSubject(project),
    level: "local",
    required_views: Object.keys(project.viewPolicy ?? {}),
    required_gaji: [],
    determinism_grade: project.viewGrade || "candidate",
    trust_badge: "unverified_local_file",
  };
}

function trimPortableText(value, limit = 120) {
  return String(value ?? "").replace(/\s+/g, " ").trim().slice(0, limit);
}

function portableListFromText(value, limit = 8) {
  return String(value ?? "")
    .split(/[,\n]/u)
    .map((item) => trimPortableText(item, 40))
    .filter(Boolean)
    .slice(0, limit);
}

function normalizePortableMetadataDraft(value = {}, project = currentProject()) {
  const base = defaultPortableMetadata(project);
  const row = value && typeof value === "object" && !Array.isArray(value) ? value : {};
  const kind = String(row.content_kind ?? row.content_kinds?.[0] ?? base.content_kinds[0] ?? "simulation");
  const allowedKind = ["simulation", "game", "lesson", "korean_manim", "ai_gym", "test_or_proof"].includes(kind)
    ? kind
    : "simulation";
  return {
    source_profile_identity: workbenchSourceProfile(project),
    title: trimPortableText(row.title ?? base.title, 80),
    summary: trimPortableText(row.summary ?? base.summary, 180),
    content_kinds: [allowedKind],
    tags: Array.isArray(row.tags) ? row.tags.map((item) => trimPortableText(item, 40)).filter(Boolean).slice(0, 8) : portableListFromText(base.tags.join(", "), 8),
    subject: trimPortableText(row.subject ?? base.subject, 40),
    level: trimPortableText(row.level ?? base.level, 40),
    required_views: Array.isArray(row.required_views) ? row.required_views.map((item) => trimPortableText(item, 40)).filter(Boolean).slice(0, 8) : [...base.required_views],
    required_gaji: Array.isArray(row.required_gaji) ? row.required_gaji.map((item) => trimPortableText(item, 40)).filter(Boolean).slice(0, 8) : [...base.required_gaji],
    determinism_grade: trimPortableText(row.determinism_grade ?? base.determinism_grade, 40),
    trust_badge: "unverified_local_file",
  };
}

function metadataDraftFromProject(project = currentProject()) {
  return normalizePortableMetadataDraft(project.portableMetadata ?? defaultPortableMetadata(project), project);
}

function currentPortableMetadata(project = currentProject()) {
  return normalizePortableMetadataDraft(state.portableMetadataDraft ?? metadataDraftFromProject(project), project);
}

function currentPortableLineage(project = currentProject()) {
  if (project.portableArtifact?.content_sha256) {
    const snapshot = project.portableSnapshot ?? {};
    return {
      source_lesson_id: String(snapshot.source_lesson_id || "portable"),
      source_project_id: "",
      source_revision_id: "",
      source_package_hash: "",
      origin_content_sha256: String(project.portableArtifact.content_sha256 || ""),
      origin_kind: project.id.includes("_clone_") ? "portable_clone" : "portable_import",
    };
  }
  const sourceLessonId = project.id === ONE_FILE_DDN_PROJECT_ID ? "custom" : project.id;
  const originKind = state.serverProject.projectId
    ? "server_project"
    : project.id === ONE_FILE_DDN_PROJECT_ID
      ? "one_file_ddn"
      : "official_lesson";
  return {
    source_lesson_id: sourceLessonId,
    source_project_id: state.serverProject.projectId || "",
    source_revision_id: state.serverProject.headRevisionId || "",
    source_package_hash: state.serverProject.packageHash || "",
    origin_content_sha256: "",
    origin_kind: originKind,
  };
}

async function loadText(file) {
  if (file.url) {
    try {
      const response = await fetch(new URL(file.url, import.meta.url), { cache: "no-store" });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
        throw new Error("E_SOURCE_UTF8_BOM_FORBIDDEN: 원문은 BOM 없는 UTF-8이어야 합니다");
      }
      return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
    } catch (error) {
      throw new Error(`예제 파일을 불러오지 못했습니다: ${file.path} (${error.message})`);
    }
  }
  return String(file.fallback ?? "");
}

async function loadProjectFiles(project) {
  const texts = new Map();
  for (const file of project.files) {
    texts.set(file.path, await loadText(file));
  }
  state.fileTexts = texts;
  state.loadedFileTexts = new Map(texts);
}

  return { inferContentKinds, inferSubject, defaultPortableMetadata, trimPortableText, portableListFromText, normalizePortableMetadataDraft, metadataDraftFromProject, currentPortableMetadata, currentPortableLineage, loadText, loadProjectFiles };
}
