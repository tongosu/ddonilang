const STATUS_LABEL_FALLBACK = Object.freeze({
  landed: "지금 배울 수 있음",
  mixed: "기반 있음 · 일부 열림",
  gap_open: "설계 열림",
});

const SOURCE_GROUPS = Object.freeze([
  { title: "이어진 교재·가이드", kinds: new Set(["textbook", "guide", "curriculum_index"]) },
  { title: "구현 근거", kinds: new Set(["implementation_report"]) },
  { title: "설계 제안 · 승인 전", kinds: new Set(["proposal"]) },
]);

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function text(value) {
  return String(value ?? "").trim();
}

function make(tagName, className = "", content = "") {
  const node = document.createElement(tagName);
  if (className) node.className = className;
  if (content) node.textContent = content;
  return node;
}

function appendTextList(parent, rows, className) {
  const list = make("ul", className);
  rows.forEach((row) => {
    const item = make("li", "", text(row));
    list.append(item);
  });
  parent.append(list);
  return list;
}

function renderReferenceSection(title, rows, pathKey = "path") {
  const section = make("section", "learning-spine-reference-section");
  section.append(make("h4", "", title));
  const list = make("ul", "learning-spine-reference-list");
  rows.forEach((row) => {
    const item = make("li", "learning-spine-reference-row");
    item.dataset.sourceKind = text(row?.source_kind);
    const name = make("span", "learning-spine-reference-name", text(row?.title || row?.role || row?.[pathKey]));
    const evidenceLabel = row?.evidence_status === "verified_by_stage_gate"
      ? "검증 관문 연결"
      : row?.evidence_status === "reference_only"
        ? "참고 자료"
        : "";
    const roleText = [text(row?.role), evidenceLabel].filter(Boolean).join(" · ");
    const role = make("span", "learning-spine-reference-role", roleText);
    item.append(name);
    if (role.textContent && role.textContent !== name.textContent) item.append(role);
    list.append(item);
  });
  section.append(list);
  return section;
}

function renderWorldSection(stage, onOpenLesson) {
  const section = make("section", "learning-spine-world-section");
  section.append(make("h4", "", "이어진 셈그림 세계"));
  const list = make("div", "learning-spine-world-list");
  asArray(stage?.worlds).forEach((world) => {
    const row = make("div", "learning-spine-world-row");
    row.dataset.learningWorldId = text(world?.lesson_id);
    const copy = make("div", "learning-spine-world-copy");
    copy.append(make("strong", "", text(world?.title || world?.lesson_id)));
    copy.append(make("span", "", text(world?.role)));
    const button = make("button", "btn-primary learning-spine-world-open", "세계 열기");
    button.type = "button";
    button.dataset.learningWorldOpen = text(world?.lesson_id);
    button.addEventListener("click", () => {
      const lessonId = text(world?.lesson_id);
      if (lessonId) onOpenLesson?.(lessonId, stage, world);
    });
    row.append(copy, button);
    list.append(row);
  });
  section.append(list);
  return section;
}

function renderCorrespondence(rows) {
  const section = make("section", "learning-spine-correspondence");
  section.append(make("h4", "", "대응 교과 현재 자리"));
  const list = make("div", "learning-spine-correspondence-list");
  rows.forEach((row) => {
    const item = make("div", "learning-spine-correspondence-row");
    item.append(make("strong", "", text(row?.language)));
    const status = make("span", "learning-spine-correspondence-status", text(row?.status));
    status.dataset.status = text(row?.status);
    item.append(status, make("p", "", text(row?.relation)));
    list.append(item);
  });
  section.append(list);
  return section;
}

function renderStageDetail(root, index, stage, onOpenLesson) {
  root.replaceChildren();
  root.dataset.learningStageActive = text(stage?.id);

  const heading = make("header", "learning-spine-stage-heading");
  const titleWrap = make("div", "learning-spine-stage-title-wrap");
  titleWrap.append(
    make("span", "learning-spine-stage-number", `${Number(stage?.order) || 0}단계`),
    make("h3", "", text(stage?.title)),
  );
  const status = make(
    "span",
    "learning-spine-stage-status",
    text(index?.status_labels?.[stage?.status]) || STATUS_LABEL_FALLBACK[stage?.status] || text(stage?.status),
  );
  status.dataset.status = text(stage?.status);
  heading.append(titleWrap, status);

  const outcome = make("p", "learning-spine-stage-outcome", text(stage?.outcome));
  const meta = make("div", "learning-spine-stage-meta");
  meta.append(
    make("span", "", `노출 ${asArray(stage?.exposure_levels).join(" · ")}`),
    make("span", "", `가나다 ${asArray(stage?.ganada_cells).join(" · ")}`),
  );

  const axes = stage?.state_axes ?? {};
  const authority = make("div", "learning-spine-authority-line");
  authority.dataset.sourceStatus = text(axes?.source);
  authority.dataset.reflectionStatus = text(axes?.reflection);
  authority.dataset.implementationStatus = text(axes?.implementation);
  authority.dataset.ownerDecisionStatus = text(axes?.owner_decision);
  authority.append(
    make("span", "", "근거 연결됨"),
    make("span", "", "연결 색인에만 반영"),
    make("span", "", "승격 결정 대기"),
  );

  const grid = make("div", "learning-spine-detail-grid");
  const sourceRefs = asArray(stage?.source_refs);
  SOURCE_GROUPS.forEach((group) => {
    const rows = sourceRefs.filter((row) => group.kinds.has(text(row?.source_kind)));
    if (rows.length) grid.append(renderReferenceSection(group.title, rows));
  });
  grid.append(renderReferenceSection("팩 근거", asArray(stage?.pack_refs)));

  root.append(heading, outcome, meta, authority, renderWorldSection(stage, onOpenLesson), grid);

  const correspondence = asArray(stage?.correspondence);
  if (correspondence.length) root.append(renderCorrespondence(correspondence));

  const openItems = asArray(stage?.open_items).map(text).filter(Boolean);
  if (openItems.length) {
    const openSection = make("section", "learning-spine-open-section");
    openSection.append(make("h4", "", "아직 열려 있는 것"));
    appendTextList(openSection, openItems, "learning-spine-open-list");
    root.append(openSection);
  }
}

export function mountLearningSpineIndex({ root, index, opener, onOpenLesson, onOpenChange } = {}) {
  if (!root) return null;
  const stages = asArray(index?.stages).slice().sort((a, b) => Number(a?.order) - Number(b?.order));
  if (!stages.length) {
    root.textContent = "배움길 자료를 찾지 못했습니다.";
    return null;
  }

  root.replaceChildren();
  root.dataset.schema = text(index?.schema);
  const header = make("header", "learning-spine-header");
  const headerCopy = make("div", "learning-spine-header-copy");
  headerCopy.append(make("span", "learning-spine-kicker", "기존 교재와 세계 연결"), make("h2", "", text(index?.title)));
  const closeButton = make("button", "ghost learning-spine-close", "×");
  closeButton.type = "button";
  closeButton.title = "배움길 닫기";
  closeButton.setAttribute("aria-label", "배움길 닫기");
  header.append(headerCopy, closeButton);

  const body = make("div", "learning-spine-body");
  const nav = make("nav", "learning-spine-stage-nav");
  nav.setAttribute("aria-label", "배움 단계");
  const detail = make("article", "learning-spine-stage-detail");
  body.append(nav, detail);
  root.append(header, body);

  let activeStageId = text(stages[0]?.id);
  const buttons = new Map();
  const selectStage = (stageId) => {
    const stage = stages.find((row) => text(row?.id) === text(stageId)) || stages[0];
    activeStageId = text(stage?.id);
    buttons.forEach((button, id) => {
      const active = id === activeStageId;
      button.classList.toggle("active", active);
      button.setAttribute("aria-current", active ? "step" : "false");
    });
    renderStageDetail(detail, index, stage, onOpenLesson);
    try {
      window.__SEAMGRIM_LEARNING_SPINE_SELECTION__ = {
        schema: "seamgrim.learning_spine.selection.v1",
        stage_id: activeStageId,
        status: text(stage?.status),
        authority: "none",
      };
    } catch (_) {
      // Browser instrumentation is optional.
    }
    return stage;
  };

  stages.forEach((stage) => {
    const button = make("button", "learning-spine-stage-button");
    button.type = "button";
    button.dataset.learningStageId = text(stage?.id);
    button.append(
      make("span", "learning-spine-stage-button-number", String(Number(stage?.order) || "")),
      make("span", "learning-spine-stage-button-title", text(stage?.title)),
      make("span", "learning-spine-stage-button-short-title", text(stage?.nav_title || stage?.title)),
    );
    button.addEventListener("click", () => selectStage(stage?.id));
    buttons.set(text(stage?.id), button);
    nav.append(button);
  });

  const setOpen = (open) => {
    const next = Boolean(open);
    root.classList.toggle("hidden", !next);
    root.setAttribute("aria-hidden", next ? "false" : "true");
    onOpenChange?.(next);
    if (next) {
      closeButton.focus({ preventScroll: true });
    } else {
      opener?.focus?.({ preventScroll: true });
    }
    return next;
  };
  closeButton.addEventListener("click", () => setOpen(false));
  selectStage(activeStageId);

  return {
    open() {
      return setOpen(true);
    },
    close() {
      return setOpen(false);
    },
    toggle() {
      return setOpen(root.classList.contains("hidden"));
    },
    selectStage,
    getState() {
      return { open: !root.classList.contains("hidden"), activeStageId, stageCount: stages.length };
    },
  };
}
