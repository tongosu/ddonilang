const PLAY_PROJECT_PREFIX = "solutions/seamgrim_ui_mvp/";

function normalizePlayLessonPath(raw) {
  const normalized = String(raw ?? "")
    .replace(/\\/g, "/")
    .replace(/^\.\//, "")
    .replace(/^\//, "")
    .trim();
  if (!normalized) return "";
  const segments = normalized.split("/");
  const safeSegments = [];
  for (const segmentRaw of segments) {
    const segment = String(segmentRaw ?? "").trim();
    if (!segment || segment === ".") continue;
    if (segment === "..") return "";
    safeSegments.push(segment);
  }
  return safeSegments.join("/");
}

function buildLessonPathFromId(rawId) {
  const lessonId = String(rawId ?? "").trim();
  if (!lessonId) return "";
  return `lessons/${lessonId}/lesson.ddn`;
}

function formatLessonSourceLabelFromPath(rawPath) {
  const pathRaw = String(rawPath ?? "").trim();
  if (!pathRaw) return "lesson:";
  if (/^https?:\/\//i.test(pathRaw)) {
    return `lesson:${pathRaw}`;
  }
  return `lesson:${normalizePlayLessonPath(pathRaw)}`;
}

function parseLessonSourceDescriptor(rawSource) {
  const source = String(rawSource ?? "").trim();
  if (!source) {
    return { lessonPathRaw: "", lessonIdRaw: "" };
  }
  const match = source.match(/^lesson(_path|path|_id|id)?:\s*(.+)$/i);
  if (!match) {
    return { lessonPathRaw: "", lessonIdRaw: "" };
  }
  const kind = String(match[1] ?? "").toLowerCase();
  const value = String(match[2] ?? "").trim();
  if (!value) {
    return { lessonPathRaw: "", lessonIdRaw: "" };
  }
  if (kind === "_id" || kind === "id") {
    return { lessonPathRaw: "", lessonIdRaw: value };
  }
  if (kind === "_path" || kind === "path") {
    return { lessonPathRaw: value, lessonIdRaw: "" };
  }
  // lesson:<value> 는 path 우선 추정, 나머지는 lesson id로 처리한다.
  if (value.includes("/") || value.endsWith(".ddn")) {
    return { lessonPathRaw: value, lessonIdRaw: "" };
  }
  return { lessonPathRaw: "", lessonIdRaw: value };
}

function isProjectPrefixedHost(pathname, projectPrefix = PLAY_PROJECT_PREFIX) {
  const normalizedPath = String(pathname ?? "").trim();
  if (!normalizedPath) return false;
  return normalizedPath.includes(`/${projectPrefix}`) || normalizedPath.startsWith(`/${projectPrefix.slice(0, -1)}`);
}

function resolvePlayLessonRequestFromParams(paramsLike, pathname, {
  projectPrefix = PLAY_PROJECT_PREFIX,
  sourceScope = "",
} = {}) {
  const params = paramsLike instanceof URLSearchParams
    ? paramsLike
    : new URLSearchParams(String(paramsLike ?? "").replace(/^\?/, ""));
  const sourceRaw = String(params.get("source") ?? params.get("src") ?? "").trim();
  const lessonPathRaw = String(
    params.get("lesson")
    ?? params.get("lesson_path")
    ?? params.get("lessonPath")
    ?? "",
  ).trim();
  const lessonIdRaw = String(
    params.get("lesson_id")
    ?? params.get("lessonId")
    ?? "",
  ).trim();
  const sourceDescriptor = parseLessonSourceDescriptor(sourceRaw);

  let lessonPath = "";
  let sourceLabel = "";
  if (lessonPathRaw) {
    lessonPath = lessonPathRaw;
    sourceLabel = formatLessonSourceLabelFromPath(lessonPathRaw);
  } else if (lessonIdRaw) {
    lessonPath = buildLessonPathFromId(lessonIdRaw);
    sourceLabel = `lesson:${lessonIdRaw}`;
  } else if (sourceDescriptor.lessonPathRaw) {
    lessonPath = sourceDescriptor.lessonPathRaw;
    sourceLabel = formatLessonSourceLabelFromPath(sourceDescriptor.lessonPathRaw);
  } else if (sourceDescriptor.lessonIdRaw) {
    lessonPath = buildLessonPathFromId(sourceDescriptor.lessonIdRaw);
    sourceLabel = `lesson:${sourceDescriptor.lessonIdRaw}`;
  }
  if (!lessonPath) {
    return {
      requested: false,
      lessonPath: "",
      sourceLabel: "",
      candidates: [],
      sourceScope,
    };
  }

  const projectPrefixedHost = isProjectPrefixedHost(pathname, projectPrefix);
  const candidates = buildPlayLessonCandidates(lessonPath, { projectPrefix, projectPrefixedHost });
  if (candidates.length === 0) {
    return {
      requested: false,
      lessonPath: "",
      sourceLabel: "",
      candidates: [],
      sourceScope,
    };
  }
  const normalizedLessonPath = /^https?:\/\//i.test(String(lessonPath ?? "").trim())
    ? String(lessonPath ?? "").trim()
    : normalizePlayLessonPath(lessonPath);
  return {
    requested: true,
    lessonPath: normalizedLessonPath,
    sourceLabel,
    candidates,
    sourceScope,
  };
}

export function buildPlayLessonCandidates(rawPath, {
  projectPrefix = PLAY_PROJECT_PREFIX,
  projectPrefixedHost = false,
} = {}) {
  const raw = String(rawPath ?? "").trim();
  if (!raw) return [];
  if (/^https?:\/\//i.test(raw)) return [raw];

  const normalized = normalizePlayLessonPath(rawPath);
  if (!normalized) return [];

  const stripped = normalized.startsWith(projectPrefix)
    ? normalized.slice(projectPrefix.length)
    : normalized;
  const prefixed = normalized.startsWith(projectPrefix)
    ? normalized
    : `${projectPrefix}${normalized}`;

  const primary = `/${stripped}`;
  const secondary = `/${prefixed}`;
  if (primary === secondary) return [primary];
  return projectPrefixedHost ? [secondary, primary] : [primary, secondary];
}

export function resolvePlayLessonRequest(locationLike = null, {
  projectPrefix = PLAY_PROJECT_PREFIX,
} = {}) {
  const search = String(locationLike?.search ?? "");
  const hash = String(locationLike?.hash ?? "");
  const pathname = String(locationLike?.pathname ?? "");
  const queryLesson = resolvePlayLessonRequestFromParams(
    new URLSearchParams(search.replace(/^\?/, "")),
    pathname,
    { projectPrefix, sourceScope: "query" },
  );
  if (queryLesson.requested) return queryLesson;
  return resolvePlayLessonRequestFromParams(
    new URLSearchParams(hash.replace(/^#/, "")),
    pathname,
    { projectPrefix, sourceScope: "hash" },
  );
}

function normalizePlayExampleKey(raw) {
  return String(raw ?? "")
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9_]/g, "");
}

function parseExampleSourceDescriptor(rawSource) {
  const source = String(rawSource ?? "").trim();
  if (!source) return "";
  const match = source.match(/^example(_id|id)?:\s*(.+)$/i);
  if (!match) return "";
  return String(match[2] ?? "").trim();
}

export function resolvePlayExampleRequest(locationLike = null) {
  const search = String(locationLike?.search ?? "");
  const hash = String(locationLike?.hash ?? "");
  const queryParams = new URLSearchParams(search.replace(/^\?/, ""));
  const hashParams = new URLSearchParams(hash.replace(/^#/, ""));
  const querySourceRaw = String(queryParams.get("source") ?? queryParams.get("src") ?? "").trim();
  const queryExampleRaw = String(
    queryParams.get("example")
    ?? queryParams.get("example_id")
    ?? queryParams.get("exampleId")
    ?? "",
  ).trim();
  const querySourceExampleKey = parseExampleSourceDescriptor(querySourceRaw);
  const useQuery = Boolean(queryExampleRaw || querySourceExampleKey);
  const params = useQuery ? queryParams : hashParams;
  const sourceRaw = String(params.get("source") ?? params.get("src") ?? "").trim();
  const exampleRaw = String(
    params.get("example")
    ?? params.get("example_id")
    ?? params.get("exampleId")
    ?? "",
  ).trim();

  const sourceExampleKey = parseExampleSourceDescriptor(sourceRaw);
  const exampleKey = sourceExampleKey || exampleRaw;
  const normalized = normalizePlayExampleKey(exampleKey);
  if (!normalized) {
    return {
      requested: false,
      exampleKey: "",
      sourceLabel: "",
      sourceScope: useQuery ? "query" : "hash",
    };
  }
  return {
    requested: true,
    exampleKey: normalized,
    sourceLabel: `example:${normalized}`,
    sourceScope: useQuery ? "query" : "hash",
  };
}

export function resolvePlayLaunchRequest(locationLike = null, options = {}) {
  const search = String(locationLike?.search ?? "");
  const hash = String(locationLike?.hash ?? "");
  const pathname = String(locationLike?.pathname ?? "");
  const projectPrefix = options?.projectPrefix ?? PLAY_PROJECT_PREFIX;

  const queryLesson = resolvePlayLessonRequestFromParams(
    new URLSearchParams(search.replace(/^\?/, "")),
    pathname,
    { projectPrefix, sourceScope: "query" },
  );
  if (queryLesson.requested) {
    return {
      kind: "lesson",
      lesson: queryLesson,
      example: { requested: false, exampleKey: "", sourceLabel: "" },
    };
  }

  const queryExample = resolvePlayExampleRequest({ search, hash: "", pathname });
  if (queryExample.requested) {
    return {
      kind: "example",
      lesson: queryLesson,
      example: queryExample,
    };
  }

  const hashLesson = resolvePlayLessonRequestFromParams(
    new URLSearchParams(hash.replace(/^#/, "")),
    pathname,
    { projectPrefix, sourceScope: "hash" },
  );
  if (hashLesson.requested) {
    return {
      kind: "lesson",
      lesson: hashLesson,
      example: { requested: false, exampleKey: "", sourceLabel: "" },
    };
  }

  const hashExample = resolvePlayExampleRequest({ search: "", hash, pathname });
  if (hashExample.requested) {
    return {
      kind: "example",
      lesson: hashLesson,
      example: hashExample,
    };
  }

  const lesson = resolvePlayLessonRequest(locationLike, options);
  const example = resolvePlayExampleRequest(locationLike);

  return {
    kind: "none",
    lesson,
    example,
  };
}
