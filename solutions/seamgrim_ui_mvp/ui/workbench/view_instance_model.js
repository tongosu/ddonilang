export const VIEW_WORKSPACE_SCHEMA = "seamgrim.workbench.view_workspace.v1";
export const VIEW_INSTANCE_SCHEMA = "seamgrim.workbench.view_instance.v1";
export const VIEW_WORKSPACE_MIGRATION_SCHEMA = "seamgrim.workbench.view_workspace_migration.v1";

const OVERLAY_FAMILIES = new Set(["pendulum", "projectile", "grid"]);

function familyFromProject(project = {}) {
  return String(project.primaryView ?? "line");
}

function primaryInstance(project = {}) {
  return {
    schema: VIEW_INSTANCE_SCHEMA,
    view_instance_id: "view_primary_1",
    role: "primary",
    family: familyFromProject(project),
    target_view_instance_id: "",
    target: null,
    temporary: false,
    render_surface: "primary_canvas",
  };
}

function workspaceCopy(workspace) {
  return {
    ...workspace,
    instances: (workspace?.instances ?? []).map((instance) => ({
      ...instance,
      target: instance.target ? { ...instance.target } : null,
    })),
    selected_target: workspace?.selected_target ? { ...workspace.selected_target } : null,
  };
}

export function createViewWorkspace(project = {}) {
  return {
    schema: VIEW_WORKSPACE_SCHEMA,
    sequence: 1,
    primary_view_instance_id: "view_primary_1",
    active_view_instance_id: "view_primary_1",
    selected_target: null,
    instances: [primaryInstance(project)],
  };
}

export function viewRoleCapabilities(project = {}, semanticPayload = null, renderModel = null) {
  const family = familyFromProject(project);
  const policy = project.viewPolicy ?? {};
  const runtimeReady = Boolean(semanticPayload?.committed_frame && renderModel?.status === "ready");
  const observationCount = Number(semanticPayload?.ordered_observations?.length ?? 0);
  return {
    primary: { supported: true, enabled: true, reason: "" },
    side: {
      supported: runtimeReady && observationCount > 0 && Boolean(String(policy.side ?? "").trim()),
      enabled: runtimeReady && observationCount > 0,
      reason: runtimeReady ? "곁보기에 표시할 관찰 채널이 없습니다." : "실행 후 committed 관찰값이 생기면 열 수 있습니다.",
    },
    overlay: {
      supported: runtimeReady && OVERLAY_FAMILIES.has(family) && Boolean(String(policy.overlay ?? "").trim()),
      enabled: runtimeReady && OVERLAY_FAMILIES.has(family),
      reason: runtimeReady ? "현재 보기 family는 target 좌표 overlay를 제공하지 않습니다." : "실행 후 committed target 좌표가 생기면 켤 수 있습니다.",
    },
    magnifier: {
      supported: runtimeReady && Boolean(String(policy.magnifier ?? "").trim()),
      enabled: runtimeReady,
      reason: runtimeReady ? "으뜸보기에서 대상을 먼저 선택하세요." : "실행 후 committed target을 선택하면 열 수 있습니다.",
    },
  };
}

function nextId(workspace, role) {
  const sequence = Number(workspace?.sequence ?? 1) + 1;
  return { sequence, id: `view_${role}_${sequence}` };
}

function appendInstance(workspace, role, project, target = null, temporary = false) {
  const next = workspaceCopy(workspace);
  const { sequence, id } = nextId(next, role);
  next.sequence = sequence;
  next.active_view_instance_id = id;
  next.instances.push({
    schema: VIEW_INSTANCE_SCHEMA,
    view_instance_id: id,
    role,
    family: familyFromProject(project),
    target_view_instance_id: role === "primary" ? "" : next.primary_view_instance_id,
    target: target ? { ...target } : null,
    temporary,
    render_surface: role === "overlay" ? "target_overlay" : "side_surface",
  });
  return next;
}

export function openSideView(workspace, project, capabilities) {
  if (!capabilities?.side?.supported || !capabilities.side.enabled) return workspace;
  const existing = (workspace?.instances ?? []).find((instance) => (
    instance.role === "side" && instance.family === familyFromProject(project)
  ));
  if (existing) {
    const next = workspaceCopy(workspace);
    next.instances.find((instance) => instance.view_instance_id === existing.view_instance_id).hidden = false;
    next.active_view_instance_id = existing.view_instance_id;
    return next;
  }
  return appendInstance(workspace, "side", project);
}

export function toggleOverlayView(workspace, project, capabilities) {
  const existing = (workspace?.instances ?? []).find((instance) => instance.role === "overlay");
  if (existing) return closeViewInstance(workspace, existing.view_instance_id);
  if (!capabilities?.overlay?.supported || !capabilities.overlay.enabled) return workspace;
  return appendInstance(workspace, "overlay", project, {
    target_id: workspace.primary_view_instance_id,
    kind: "primary_coordinate_space",
  });
}

export function selectViewTarget(workspace, target) {
  const next = workspaceCopy(workspace);
  const current = next.selected_target;
  const sameTarget = current && target && ["target_id", "kind", "run_id", "state_hash", "frame_id", "madi"]
    .every((key) => current[key] === target[key]);
  // A new committed target terminates the previous local magnifier. Re-selecting
  // the same target only changes focus; it must not allocate another instance.
  if (!sameTarget) next.instances = next.instances.filter((instance) => instance.role !== "magnifier");
  next.selected_target = target ? { ...target } : null;
  next.active_view_instance_id = next.primary_view_instance_id;
  return next;
}

export function openMagnifierView(workspace, project, capabilities) {
  if (!capabilities?.magnifier?.supported || !capabilities.magnifier.enabled || !workspace?.selected_target) return workspace;
  const existing = workspace.instances.find((instance) => instance.role === "magnifier");
  if (existing) {
    const next = workspaceCopy(workspace);
    next.active_view_instance_id = existing.view_instance_id;
    return next;
  }
  return appendInstance(workspace, "magnifier", project, workspace.selected_target, true);
}

export function closeViewInstance(workspace, instanceId) {
  if (!workspace || instanceId === workspace.primary_view_instance_id) return workspace;
  const next = workspaceCopy(workspace);
  const closing = next.instances.find((instance) => instance.view_instance_id === instanceId);
  if (closing?.role === "side") {
    closing.hidden = true;
    next.active_view_instance_id = next.primary_view_instance_id;
    return next;
  }
  next.instances = next.instances.filter((instance) => (
    instance.view_instance_id !== instanceId && instance.target_view_instance_id !== instanceId
  ));
  if (!next.instances.some((instance) => instance.view_instance_id === next.active_view_instance_id)) {
    next.active_view_instance_id = next.primary_view_instance_id;
  }
  return next;
}

export function closeAllSideViews(workspace) {
  const next = workspaceCopy(workspace);
  next.instances = next.instances.filter((instance) => instance.role !== "magnifier");
  for (const instance of next.instances) if (instance.role === "side") instance.hidden = true;
  next.active_view_instance_id = next.primary_view_instance_id;
  return next;
}

export function focusPrimaryView(workspace) {
  const next = workspaceCopy(workspace);
  next.active_view_instance_id = next.primary_view_instance_id;
  return next;
}

export function reconcileViewWorkspace(workspace, project, committedFrame = null) {
  const next = workspaceCopy(workspace?.schema === VIEW_WORKSPACE_SCHEMA ? workspace : createViewWorkspace(project));
  const family = familyFromProject(project);
  let primary = next.instances.find((instance) => instance.view_instance_id === next.primary_view_instance_id);
  if (!primary) {
    primary = primaryInstance(project);
    next.instances.unshift(primary);
    next.primary_view_instance_id = primary.view_instance_id;
  }
  primary.role = "primary";
  primary.family = family;
  primary.render_surface = "primary_canvas";
  next.instances = next.instances.map((instance) => ({ ...instance, family }));
  const selectedHash = String(next.selected_target?.state_hash ?? "");
  const committedHash = String(committedFrame?.state_hash ?? "");
  const changedScope = next.selected_target && ["run_id", "frame_id", "madi"]
    .some((key) => next.selected_target[key] !== undefined && next.selected_target[key] !== committedFrame?.[key]);
  if (next.selected_target && (!selectedHash || !committedHash || selectedHash !== committedHash || changedScope)) {
    next.selected_target = null;
    next.instances = next.instances.filter((instance) => !instance.temporary);
  }
  if (!next.instances.some((instance) => instance.view_instance_id === next.active_view_instance_id)) {
    next.active_view_instance_id = next.primary_view_instance_id;
  }
  return next;
}

export function primaryViewInstance(workspace) {
  return (workspace?.instances ?? []).find((instance) => instance.view_instance_id === workspace.primary_view_instance_id) ?? null;
}

export function sideViewInstances(workspace) {
  return (workspace?.instances ?? []).filter((instance) => !instance.hidden && ["side", "magnifier"].includes(instance.role));
}

export function overlayViewInstances(workspace) {
  return (workspace?.instances ?? []).filter((instance) => instance.role === "overlay");
}

export function legacySelectedView(workspace) {
  if ((workspace?.instances ?? []).some((instance) => instance.role === "overlay")) return "overlay";
  if ((workspace?.instances ?? []).some((instance) => instance.role === "magnifier")) return "magnifier";
  if ((workspace?.instances ?? []).some((instance) => instance.role === "side" && !instance.hidden)) return "side";
  return "primary";
}

export function serializeViewWorkspace(workspace) {
  const next = workspaceCopy(workspace);
  next.selected_target = null;
  next.instances = next.instances.filter((instance) => !instance.temporary).map((instance) => ({
    ...instance,
    target: instance.role === "overlay" ? instance.target : null,
  }));
  next.active_view_instance_id = next.instances.some((instance) => instance.view_instance_id === next.active_view_instance_id)
    ? next.active_view_instance_id
    : next.primary_view_instance_id;
  return next;
}

export function hydrateViewWorkspaceWithMigration(project, snapshot = null, legacySelection = "") {
  if (snapshot?.schema === VIEW_WORKSPACE_SCHEMA && Array.isArray(snapshot.instances)) {
    return {
      workspace: reconcileViewWorkspace(snapshot, project, null),
      migration: null,
    };
  }
  const legacyValue = String(legacySelection ?? "").trim();
  const legacyRole = ["primary", "side", "overlay", "magnifier"].includes(legacyValue)
    ? legacyValue
    : "primary";
  let workspace = createViewWorkspace(project);
  if (legacyRole === "side") workspace = appendInstance(workspace, "side", project);
  if (legacyRole === "overlay") workspace = appendInstance(workspace, "overlay", project, {
    target_id: workspace.primary_view_instance_id,
    kind: "primary_coordinate_space",
  });
  const resolvedRole = ["side", "overlay"].includes(legacyRole) ? legacyRole : "primary";
  return {
    workspace,
    migration: legacyValue
      ? {
          schema: VIEW_WORKSPACE_MIGRATION_SCHEMA,
          code: "W_LEGACY_VIEW_SELECTION_MIGRATED",
          legacy_selection: legacyValue,
          resolved_role: resolvedRole,
          preserved_world_state: true,
          message: `기존 보기 선택 '${legacyValue}'을(를) 독립 ${resolvedRole} 보기 구조로 이행했습니다.`,
        }
      : null,
  };
}

export function hydrateViewWorkspace(project, snapshot = null, legacySelection = "") {
  return hydrateViewWorkspaceWithMigration(project, snapshot, legacySelection).workspace;
}
