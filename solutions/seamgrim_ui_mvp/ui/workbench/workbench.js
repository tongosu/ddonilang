import { displayMessage } from './display_locale.js';
import { installMultilangHost } from './multilang/host.js';
import { installProofSourceInspection } from "./proof_source_inspection.js";
import { installNotebookHost } from './notebook_host.js';
import { createUniformMotionProvider } from './notebook/uniform_motion.js';
let notebookHost = null;
import { createGajiController } from "./gaji_controller.js";
import { prepareAuthoredGajiDraft } from "./authored_gaji_draft.js";
import { createEditorPresentation } from "./editor_presentation.js";
import { createApprovedWorkbenchApi } from "./approved_workbench_api.js";
import { createProjectMetadata } from "./project_metadata.js";
import { createProjectControls } from "./project_controls.js";
import { createProjectPersistence } from "./project_persistence.js";
import { createRuntimePersistence } from "./runtime_persistence.js";
import { createExecutionController, createLogicalPresentationPanel } from "./execution_controller.js";
import { createCanvasPresentation } from "./canvas_presentation.js";
import { createPortableActions } from "./portable_actions.js";
import { createWorkbenchProjectRegistry } from "./project_registry.js";
import { createWorkbenchRunArchive } from "./run_archive.js";
import { createViewPresentation } from "./view_presentation.js";
import { createSourceReadinessModel } from "./source_readiness_model.js";
import { createProjectNavigation } from "./project_navigation.js";
import { createWorkbenchProjectCatalog } from "./project_catalog.js";
import { createRuntimeBridgeV1 } from "../runtime/runtime_bridge.js";
import {nativeSourceAvailable,nativeSourceRequest,sourceStorageErrorMessage} from '../runtime/native_source_transport.js';
import { installLogicalInputPanel, committedLogicalSpace2d } from "./logical_input_panel.js";
import { installAuthoredProjectPanel } from "./authored_project_panel.js";
import { createEditRecoveryGuard } from './persistence/edit_recovery_guard.js';
import { readDdnSourceFile } from './source_file.js';
import { loadRuntimeProject, commitRuntimeProject, newRuntimeRevisionHex, runtimeStorageRecord } from './persistence/runtime_project_store.js';
import { renderAuthoredBogae, createAuthoredCompositionHost } from "./authored_bogae_renderer.js";
import { applyExplorationPreset } from "./exploration_presets.js";
// Owner 2026-10-05: first steps ship; authored Bogae composition is deferred.
import { FIRST_STEPS_PROJECTS as FIRST_STEPS_ALPHA1_PROJECTS } from "./first_steps_catalog.js";
import { FIRST_STEPS_ALPHA2_PROJECTS } from './first_steps_alpha2_catalog.js';
import { FIRST_STEPS_ALPHA3_PROJECTS } from './first_steps_alpha3_catalog.js';
import { FIRST_STEPS_ALPHA4_PROJECTS } from './first_steps_alpha4_catalog.js';
const FIRST_STEPS_PROJECTS=[...FIRST_STEPS_ALPHA1_PROJECTS,...FIRST_STEPS_ALPHA2_PROJECTS,...FIRST_STEPS_ALPHA3_PROJECTS,...FIRST_STEPS_ALPHA4_PROJECTS];
const DEFERRED_LESSONS = [];
import { runHostStep } from "../runtime/host_step_watchdog.js";
const workbenchHostExecutionState = {
  hostStopNotice: "", // Presentation only; excluded from run/state/receipt/replay.
  hostStepInFlight: false,
  pendingBogaeDisplay: null, // Local committed presentation, never world state.
  rejectedBogaeDisplay: null,
};
const workbenchAuthoredPanelAccess = {
  get current() { return authoredProjectPanel; }
};
const authoredCompositionHost = createAuthoredCompositionHost(() => { renderAll(); drawPreview(); });
const workbenchLogicalPresentationState = {
  logicalPanel: null,
  logicalPresentation: null,
};
function currentLogicalPresentation() {
  return workbenchLogicalPresentationState.logicalPanel?.logicalExecution.ownsCurrentSource()
    && workbenchLogicalPresentationState.logicalPresentation?.projectId === state.projectId ? workbenchLogicalPresentationState.logicalPresentation : null;
}
function renderCurrentMadi(value, fallback = null) {
  const countText=String(value?.madi_count??'');
  const count=/^\d{1,20}$/u.test(countText)?BigInt(countText):null;
  const index=value?.frame?.input?.madi??value?.frames?.at(-1)?.input?.madi
    ??(count!==null&&count>0n?String(count-1n):fallback);
  els.currentMadi.textContent=`마디: ${index??'-'}`;
  els.currentMadi.title=count!==null
    ?`0부터 세는 마디 번호입니다. 지금까지 ${countText}마디를 처리했습니다.`
    :'마디 번호는 0부터 셉니다.';
}

async function stepWithHostStop(vm, options) {
  workbenchHostExecutionState.hostStepInFlight = true;
  renderAll();
  try { return await runHostStep(() => vm.step(options), (notice) => {
    workbenchHostExecutionState.hostStopNotice = notice;
    // Use the exact user-stop path; do not synthesize a runtime failure.
    void stopRun();
  }, {nativeSourceBound: vm.hasNativeSource?.() === true}); } finally { workbenchHostExecutionState.hostStepInFlight = false; }
}
import { createHostBridgeV1 } from "../host/host_bridge.js";
import { extractCanonicalViewAuthorityInputsFromState, extractObservationOutputRowsFromState } from "../seamgrim_runtime_state.js";
import { renderSpace2dCanvas2d } from "../wasm_page_common.js";
import { selectCommittedOutputView } from "./committed_output_view.js";
import { createSpatialViewState, updateSpatialView, fitSpatialHistory, resetSpatialRange } from "./spatial_view_state.js";
const spatialView = createSpatialViewState();
const workbenchSpatialPresentationState = {
  showSpatialTrail: true,
};
import {
  analyzeSourceReadiness,
  MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA,
  applySourceReadinessFix,
  projectAuthoritativeSourceAnalysis,
  projectSourceAnalysisUnavailable,
  sharedRunSummaryBindingValid,
  SOURCE_ANALYSIS_PROFILE_IDENTITY,
  SOURCE_READINESS_DIAGNOSTIC_CODE,
  SOURCE_READINESS_FIX_ID,
} from "./source_readiness.js";
import {
  describePersistenceStatus,
  localProjectStorageKey as projectStorageKey,
  loadLocalFallbackSnapshot,
  saveLocalFallbackSnapshot,
  stableCanonicalJson,
} from "./persistence/local_fallback.js";
import {
  appendServerRevision,
  buildWorkbenchProjectArtifact,
  workbenchSourceProfile,
  selectUnversionedDocumentProfile,
  cloneServerProject,
  createServerProject,
  listServerRevisions,
  openServerProject,
  restoreServerRevision,
} from "./persistence/project_adapter.js";
import {
  buildWorkbenchCapabilityLink,
  captureWorkbenchCapabilitySession,
  clearWorkbenchCapabilitySession,
  readWorkbenchCapabilitySession,
  writeWorkbenchCapabilitySession,
} from "./persistence/capability_session.js";
import {
  appendRunCheckpoint,
  buildCheckpointPayload,
  buildRunArchiveDraft,
  createRunArchive,
  finalizeRunArchive,
  getRunArchive,
  listRunArchives,
  summarizeRunArchiveBoundary,
} from "./persistence/run_archive_adapter.js";
import {
  buildPortableProjectArtifact,
  parsePortableProjectArtifactBytes,
  portableProjectDiagnostic,
} from "./persistence/portable_project_artifact.js";
import { exportLocalGotganArtifact, importLocalGotganArtifact, loadLocalGotganStore, publishToLocalGotgan, searchLocalGotgan } from "./persistence/local_gotgan_store.js";
import {
  EXECUTION_EVENTS,
  EXECUTION_STATES,
  claimRunFinalization,
  createExecutionLifecycle,
  executionControlPolicy,
  executionStatusLabel,
  legacyRuntimeStatus,
  alpha2HostPacing,
  hostMadiDelay,
  transitionExecutionLifecycle,
} from "./execution_lifecycle.js";
import {
  buildCanonicalViewPayloadSet,
  buildCommittedSemanticViewPayload,
  stableSemanticJson,
} from "./semantic_view_payload.js";
import {
  renderCommittedSemanticView,
  buildCommittedSpace2dOverlay,
  formatNumber,
} from "./semantic_view_renderer.js";
import {
  closeAllSideViews,
  closeViewInstance,
  createViewWorkspace,
  focusPrimaryView,
  hydrateViewWorkspaceWithMigration,
  legacySelectedView,
  openMagnifierView,
  openSideView,
  overlayViewInstances,
  primaryViewInstance,
  reconcileViewWorkspace,
  selectViewTarget,
  serializeViewWorkspace,
  sideViewInstances,
  toggleOverlayView,
  viewRoleCapabilities,
} from "./view_instance_model.js";
import {
  actionById,
  buildProjectControlModel,
  buildSetupRecipe,
  emptyProjectControlModel,
  normalizeActionMap,
} from "./project_control_model.js";

const runtimeBridge = createRuntimeBridgeV1();
const proofSourceInspection = installProofSourceInspection({
  runtimeBridge,
  readSource: () => ({projectId: currentProject().id, path: activeFile().path, sourceText: state.fileTexts.get(activeFile().path) ?? ""}),
});
const hostBridge = createHostBridgeV1();

const { PROJECTS, FALLBACK_PENDULUM_DDN, FALLBACK_PROJECTILE_DDN } = createWorkbenchProjectCatalog({ SOURCE_ANALYSIS_PROFILE_IDENTITY, FIRST_STEPS_PROJECTS, FIRST_STEPS_ALPHA3_PROJECTS, FIRST_STEPS_ALPHA4_PROJECTS });

const MY_WORK_INDEX_KEY = "seamgrim.workbench.my_work_index.v1";
const PORTABLE_PROJECT_CATALOG_KEY = "seamgrim.workbench.portable_project_catalog.v1";
const ONE_FILE_DDN_PROJECT_ID = "local_one_file_ddn_v1";
const ONE_FILE_DDN_DRAFT_KEY = "seamgrim.workbench.one_file_ddn.draft.v1";
const NATIVE_PROJECT_CATALOG_KEY = "seamgrim.workbench.native_project_catalog.v1";

const workbenchProjectRegistryState = {
  customOneFileProject: null,
  receivedPortableProjects: [],
  nativeProjects: [],
};

function writeNativeProjectCatalog() {
  try {
    globalThis.localStorage?.setItem?.(NATIVE_PROJECT_CATALOG_KEY, JSON.stringify(workbenchProjectRegistryState.nativeProjects.slice(0, 30)));
  } catch (_) {
    // Local recovery is best-effort; portable export remains available.
  }
}

function loadNativeProjectCatalog() {
  try {
    const raw = globalThis.localStorage?.getItem?.(NATIVE_PROJECT_CATALOG_KEY) ?? "";
    const rows = raw ? JSON.parse(raw) : [];
    workbenchProjectRegistryState.nativeProjects = Array.isArray(rows) ? rows.filter((row) => row?.id && Array.isArray(row?.files)) : [];
  } catch (_) {
    workbenchProjectRegistryState.nativeProjects = [];
  }
}

function syncNativeProjectFromState(project = currentProject()) {
  const index = workbenchProjectRegistryState.nativeProjects.findIndex((row) => row.id === project?.id);
  if (index < 0) return;
  const files = project.files.map((file) => ({
    ...file,
    fallback: String(state.fileTexts.get(file.path) ?? file.fallback ?? ""),
  }));
  const payload = project.portableSnapshot?.local_package_payload;
  const portableSnapshot = { ...project.portableSnapshot };
  if (localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {})) {
    portableSnapshot.local_package_payload = {
      ...payload,
      files: files.map((file) => ({ path: file.path, role: file.role, utf8_text: file.fallback })),
    };
  }
  workbenchProjectRegistryState.nativeProjects[index] = {
    ...project,
    files,
    portableSnapshot,
  };
  writeNativeProjectCatalog();
}

function initialProjectIdFromLocation() {
  try {
    const params = new URLSearchParams(globalThis.location?.search ?? "");
    for (const key of ["project", "project_id", "lesson", "lesson_id", "source_lesson_id"]) {
      const value = String(params.get(key) ?? "").trim();
      if (value && PROJECTS.some((project) => project.id === value)) return value;
    }
  } catch (_) {
    // Static file and restricted contexts can keep the default public project.
  }
  return PROJECTS[0].id;
}

const state = {
  globalEntry: "ddn-search",
  projectId: initialProjectIdFromLocation(),
  activeFilePath: "lesson.ddn",
  mode: "ddn-main",
  fileTexts: new Map(),
  loadedFileTexts: new Map(),
  fileFilter: "",
  projectFilter: "",
  myWorkRows: [],
  showHiddenMyWork: false,
  recipeVersion: 1,
  pendingRecipe: null,
  nextInput: null,
  actionSeq: 0,
  projectControlsGeneration: 0,
  projectControlsRefreshTimer: null,
  projectControls: emptyProjectControlModel(initialProjectIdFromLocation()),
  diagnostics: [],
  runSeq: 0,
  activeRunId: "",
  runs: [],
  runArchives: [],
  selectedRunArchive: null,
  cursorIndex: -1,
  runtimeStatus: "idle",
  runtimeBusy: false,
  projectLoadError: "",
  executionLifecycle: createExecutionLifecycle(),
  livePump: {
    pacing: null,
    previousStartMs: null,
    scheduledDelayMs: null,
    generation: 0,
    scheduled: false,
    timerId: null,
    inFlight: false,
    inFlightPromise: null,
    stepStartCount: 0,
    stepCommitCount: 0,
    maxInFlight: 0,
    currentInFlight: 0,
    lastOrigin: "",
    lastCancelReason: "",
  },
  semanticViewPayload: null,
  semanticRenderModel: null,
  wasm: null,
  wasmVmEntryCount: 0,
  sourceReadiness: null,
  sourceAnalysisGeneration: 0,
  sourceAnalysisSourceText: "",
  sourceAnalysisInFlight: false,
  completionDocumentVersion: 1,
  completionGeneration: 0,
  completionItems: [],
  completionSelectedIndex: 0,
  sourceReadinessRunBlocked: false,
  sourceReadinessLastApplyOutcome: {
    schema: "seamgrim.workbench.source_readiness_apply_outcome.v1",
    status: "none",
    code: "",
    fix_it_id: "",
  },
  sourceReadinessLastRunOutcome: {
    schema: "seamgrim.workbench.source_readiness_run_outcome.v1",
    status: "none",
    code: "",
    source_hash: "",
  },
  projectExecution: {
    importedRecord: null,
    current: null,
    previousSuccess: null,
    failure: null,
    lockManuallyEdited: false,
    lockTrust: {
      schema: "seamgrim.workbench.project_lock_trust.v1",
      verified_source_hash: "",
      verified_lock_sha256: "",
      local_non_lock_edit_after_verified: false,
    },
  },
  viewWorkspace: createViewWorkspace(PROJECTS[0]),
  persistenceStatus: null,
  persistenceOperation: {
    generation: 0,
    completedGeneration: 0,
    phase: "idle",
    outcome: "",
  },
  lastPortableExport: null,
  pendingPortableImport: null,
  projectEditUndo: [],
  projectEditGeneration: 0,
  pendingProjectEdit: null,
  projectGraph: null,
  portableMetadataDraft: null,
  capabilitySession: null,
  serverProject: {
    projectId: "",
    headRevisionId: "",
    contentSha256: "",
    packageHash: "",
    editCapability: "",
    viewCapability: "",
    role: "",
    revisions: [],
    conflict: null,
  },
};

function emptyServerProjectState() {
  return {
    projectId: "",
    headRevisionId: "",
    contentSha256: "",
    packageHash: "",
    editCapability: "",
    viewCapability: "",
    role: "",
    revisions: [],
    conflict: null,
  };
}

function readPortableProjectCatalog() {
  try {
    const parsed = JSON.parse(globalThis.localStorage?.getItem?.(PORTABLE_PROJECT_CATALOG_KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed : [];
  } catch (_) {
    return [];
  }
}

const els = {
  root: document.querySelector("[data-testid='approved-workbench-root']"),
  projectRailToggle: document.querySelector("[data-testid='project-rail-toggle']"),
  projectCount: document.querySelector("#project-count"),
  projectList: document.querySelector("#project-list"),
  projectSearch: document.querySelector("#project-search"),
  myWorkPanel: document.querySelector("[data-testid='my-work-panel']"),
  myWorkList: document.querySelector("[data-testid='my-work-list']"),
  myWorkCount: document.querySelector("[data-testid='my-work-count']"),
  title: document.querySelector("#project-title"),
  kind: document.querySelector("#project-kind"),
  summary: document.querySelector("#project-summary"),
  sourceTitle: document.querySelector("#source-panel-title"),
  sourceEditor: document.querySelector("#source-editor"),
  completionPopup: document.querySelector("#completion-popup"),
  fileTabs: document.querySelector("[data-testid='file-tabs']"),
  fileDrawer: document.querySelector("[data-testid='file-drawer']"),
  fileList: document.querySelector("#file-list"),
  fileSearch: document.querySelector("#file-search"),
  fileCount: document.querySelector("#file-count"),
  layout: document.querySelector("[data-testid='workbench-layout']"),
  canvas: document.querySelector("#canvas-bogae"),
  ratioLabel: document.querySelector("#mode-ratio-label"),
  viewGradeLabel: document.querySelector("#view-grade-label"),
  projectServerMeta: document.querySelector("[data-testid='project-server-meta']"),
  projectEditLink: document.querySelector("[data-testid='project-edit-link']"),
  projectViewLink: document.querySelector("[data-testid='project-view-link']"),
  projectRevisionList: document.querySelector("[data-testid='project-revision-list']"),
  projectClone: document.querySelector("[data-testid='project-clone']"),
  portableExport: document.querySelector("[data-testid='portable-project-export']"),
  portableExportStatus: document.querySelector("[data-testid='portable-project-export-status']"),
  portableImport: document.querySelector("[data-testid='portable-project-import']"),
  portableDialog: document.querySelector("[data-testid='portable-project-dialog']"),
  portableImportInput: document.querySelector("[data-testid='portable-project-import-input']"),
  portableImportPreview: document.querySelector("[data-testid='portable-project-import-preview']"),
  portableImportTitle: document.querySelector("[data-testid='portable-project-import-title']"),
  portableImportIntegrity: document.querySelector("[data-testid='portable-project-import-integrity']"),
  portableImportTrust: document.querySelector("[data-testid='portable-project-import-trust']"),
  portableImportDetails: document.querySelector("[data-testid='portable-project-import-details']"),
  portableOpen: document.querySelector("[data-testid='portable-project-open']"),
  portableCancel: document.querySelector("[data-testid='portable-project-cancel']"),
  portableMetadataStatus: document.querySelector("[data-testid='portable-metadata-status']"),
  portableMetaTitle: document.querySelector("[data-testid='portable-meta-title']"),
  portableMetaSummary: document.querySelector("[data-testid='portable-meta-summary']"),
  portableMetaContentKind: document.querySelector("[data-testid='portable-meta-content-kind']"),
  portableMetaTags: document.querySelector("[data-testid='portable-meta-tags']"),
  portableMetaSubject: document.querySelector("[data-testid='portable-meta-subject']"),
  portableMetaLevel: document.querySelector("[data-testid='portable-meta-level']"),
  portableMetaRequiredViews: document.querySelector("[data-testid='portable-meta-required-views']"),
  portableMetaRequiredGaji: document.querySelector("[data-testid='portable-meta-required-gaji']"),
  portableLineageOrigin: document.querySelector("[data-testid='portable-lineage-origin']"),
  portableLineageFile: document.querySelector("[data-testid='portable-lineage-file']"),
  portableLineageVersion: document.querySelector("[data-testid='portable-lineage-version']"),
  projectConflictPanel: document.querySelector("[data-testid='project-conflict-panel']"),
  projectConflictMessage: document.querySelector("[data-testid='project-conflict-message']"),
  projectConflictServer: document.querySelector("[data-testid='project-conflict-server']"),
  projectConflictLocal: document.querySelector("[data-testid='project-conflict-local']"),
  projectConflictLocalPackage: document.querySelector("[data-testid='project-conflict-local-package']"),
  projectStorageDrawer: document.querySelector("[data-testid='project-storage-drawer']"),
  projectHistoryDrawer: document.querySelector("[data-testid='project-history-drawer']"),
  projectVerificationDrawer: document.querySelector("[data-testid='project-verification-drawer']"),
  projectVerificationOpen: document.querySelector("[data-testid='project-verification-open']"),
  runStatus: document.querySelector("[data-testid='run-status']"),
  runHash: document.querySelector("[data-testid='run-hash']"),
  currentMadi: document.querySelector("[data-testid='current-madi']"),
  runLineage: document.querySelector("[data-testid='run-lineage']"),
  runArchiveStatus: document.querySelector("[data-testid='run-archive-status']"),
  runArchiveList: document.querySelector("[data-testid='run-archive-list']"),
  runArchiveDetail: document.querySelector("[data-testid='run-archive-detail']"),
  runArchiveDetailTitle: document.querySelector("[data-testid='run-archive-detail-title']"),
  runArchiveCheckpointList: document.querySelector("[data-testid='run-archive-checkpoint-list']"),
  saveStatus: document.querySelector("[data-testid='project-save-status']"),
  bogaeStage: document.querySelector("[data-testid='bogae-stage']"),
  sideViewTitle: document.querySelector("[data-testid='side-view-title']"),
  sideViewBody: document.querySelector("[data-testid='side-view-body']"),
  sideViewList: document.querySelector("[data-testid='side-view-list']"),
  viewPrimary: document.querySelector("[data-testid='view-role-primary']"),
  viewSide: document.querySelector("[data-testid='view-role-side']"),
  viewOverlay: document.querySelector("[data-testid='view-role-overlay']"),
  viewMagnifier: document.querySelector("[data-testid='view-role-magnifier']"),
  bogaeSourceAccess: document.querySelector("[data-testid='bogae-source-access']"),
  layoutPolicy: document.querySelector("[data-testid='layout-policy']"),
  viewRoleSummary: document.querySelector("[data-testid='view-role-summary']"),
  controlGrid: document.querySelector("[data-testid='control-grid']"),
  setupControls: document.querySelector("[data-testid='setup-controls']"),
  setupControlList: document.querySelector("[data-testid='setup-control-list']"),
  setupStatus: document.querySelector("[data-testid='setup-status']"),
  actionControls: document.querySelector("[data-testid='action-controls']"),
  actionButtonList: document.querySelector("[data-testid='action-button-list']"),
  actionStatus: document.querySelector("[data-testid='action-status']"),
  viewStatus: document.querySelector("[data-testid='view-status']"),
  diagnostics: document.querySelector("[data-testid='diagnostic-list']"),
  semanticObservations: document.querySelector("[data-testid='semantic-observations']"),
  projectCreateDialog: document.querySelector("[data-testid='project-create-dialog']"),
  projectCreateForm: document.querySelector("[data-testid='project-create-form']"),
  projectCreateName: document.querySelector("[data-testid='project-create-name']"),
  projectEditDialog: document.querySelector("[data-testid='project-edit-dialog']"),
  projectEditSummary: document.querySelector("[data-testid='project-edit-summary']"),
  projectEditPreview: document.querySelector("[data-testid='project-edit-preview']"),
  projectGraphSummary: document.querySelector("[data-testid='project-graph-summary']"),
  projectGraphList: document.querySelector("[data-testid='project-graph-list']"),
  projectGraphDiagnostics: document.querySelector("[data-testid='project-graph-diagnostics']"),
  gajiBundleDialog: document.querySelector("[data-testid='gaji-bundle-dialog']"),
  gajiBundleForm: document.querySelector("[data-testid='gaji-bundle-form']"),
  gajiBundleId: document.querySelector("[data-testid='gaji-bundle-id']"),
  gajiBundleVersion: document.querySelector("[data-testid='gaji-bundle-version']"),
  gajiBundleDescription: document.querySelector("[data-testid='gaji-bundle-description']"),
  gajiBundleTags: document.querySelector("[data-testid='gaji-bundle-tags']"),
  gajiBundleSelected: document.querySelector("[data-testid='gaji-bundle-selected']"),
  gajiBundlePreview: document.querySelector("[data-testid='gaji-bundle-preview']"),
  gajiBundleStatus: document.querySelector("[data-testid='gaji-bundle-status']"),
  gajiBundlePublish: document.querySelector("[data-testid='gaji-bundle-publish']"),
  gajiShelfDialog: document.querySelector("[data-testid='gaji-shelf-dialog']"),
  gajiShelfSearch: document.querySelector("[data-testid='gaji-shelf-search']"),
  gajiShelfResults: document.querySelector("[data-testid='gaji-shelf-results']"),
  gajiInstallAlias: document.querySelector("[data-testid='gaji-install-alias']"),
  gajiInstallPreview: document.querySelector("[data-testid='gaji-install-preview']"),
  gajiInstallStatus: document.querySelector("[data-testid='gaji-install-status']"),
  gajiInstallApply: document.querySelector("[data-testid='gaji-install-apply']"),
  gajiInstallRemove: document.querySelector("[data-testid='gaji-install-remove']"),
  gajiRemixStart: document.querySelector("[data-testid='gaji-remix-start']"),
  gajiRemixEditor: document.querySelector("[data-testid='gaji-remix-editor']"),
  gajiRemixId: document.querySelector("[data-testid='gaji-remix-id']"),
  gajiRemixVersion: document.querySelector("[data-testid='gaji-remix-version']"),
  gajiRemixSource: document.querySelector("[data-testid='gaji-remix-source']"),
  gajiRemixPublish: document.querySelector("[data-testid='gaji-remix-publish']"),
};

const workbenchGajiPresentationState = {
  pendingGajiBundle: null,
  pendingGajiInstall: null,
};

function currentProject() {
  return allProjects().find((project) => project.id === state.projectId) ?? PROJECTS[0];
}

function activeFile(project = currentProject()) {
  return project.files.find((file) => file.path === state.activeFilePath) ?? project.files[0];
}

const editRecoveryGuard=createEditRecoveryGuard({
  // Access storage lazily: a restricted browser must get a visible diagnostic
  // rather than aborting the Workbench module before it renders.
  storage:{getItem:key=>localStorage.getItem(key),setItem:(key,value)=>localStorage.setItem(key,value)},
  capture:()=>state.fileTexts.size?{project_id:state.projectId,profile:workbenchSourceProfile(currentProject()),
    files:currentProject().files.map(file=>({path:file.path,text:state.fileTexts.get(file.path)??''}))}:null,
  applyDraft:async draft=>{
    if(state.runtimeBusy||state.livePump.inFlight||state.runtimeStatus==='running')
      throw new Error('E_EDIT_RECOVERY_PAUSE_REQUIRED: 실행을 멈춘 뒤 편집을 복구하세요.');
    for(const file of draft.files)state.fileTexts.set(file.path,file.text);
    state.completionDocumentVersion+=1;state.completionGeneration+=1;closeCompletionPopup();
    markCurrentProjectExecutionStale('edit_recovered');state.lastPortableExport=null;
    syncNativeProjectFromState();renderAll();
  },
  present:notice=>{
    const root=document.querySelector('[data-testid="edit-recovery-notice"]');
    root.hidden=!notice.visible;
    root.querySelector('[data-testid="edit-recovery-message"]').textContent=notice.message;
    root.querySelector('[data-testid="edit-recovery-restore"]').hidden=!notice.canRestore;
    root.querySelector('[data-testid="edit-recovery-open"]').hidden=!notice.canOpen;
  },
});
document.querySelector('[data-testid="edit-recovery-restore"]').addEventListener('click',()=>editRecoveryGuard.restore());
document.querySelector('[data-testid="edit-recovery-dismiss"]').addEventListener('click',()=>editRecoveryGuard.dismiss());
document.querySelector('[data-testid="edit-recovery-open"]').addEventListener('click',()=>reopenProjectSnapshot().catch(()=>renderAll()));
// Capture runs before the normal input handler. A renderer/control exception
// cannot erase the just-typed text by preventing its model update.
els.sourceEditor.addEventListener('input',()=>editRecoveryGuard.saveEditor(state.activeFilePath,els.sourceEditor.value),{capture:true});
window.addEventListener('error',event=>editRecoveryGuard.fault(event.error??event.message));
window.addEventListener('unhandledrejection',event=>editRecoveryGuard.fault(event.reason));

function activeRun() {
  return state.runs.find((run) => run.run_id === state.activeRunId) ?? null;
}

function activeFrame() {
  const run = activeRun();
  if (!run || state.cursorIndex < 0) return null;
  return run.frames[state.cursorIndex] ?? null;
}

function executionLifecycleContext() {
  const run = activeRun();
  return {
    hasFrame: Boolean(activeFrame()),
    frameCount: run?.frames.length ?? 0,
    cursorIndex: state.cursorIndex,
  };
}

function syncExecutionLifecycleCompatibility() {
  state.runtimeStatus = legacyRuntimeStatus(state.executionLifecycle);
}

function applyExecutionEvent(event, payload = {}) {
  const result = transitionExecutionLifecycle(state.executionLifecycle, event, payload);
  if (result.accepted) {
    state.executionLifecycle = result.lifecycle;
    syncExecutionLifecycleCompatibility();
  }
  return result;
}

function executionGenerationIsCurrent(generation, projectId) {
  return Number(state.executionLifecycle.generation) === Number(generation)
    && state.projectId === projectId;
}

function assertExecutionGeneration(generation, projectId) {
  if (executionGenerationIsCurrent(generation, projectId)) return;
  const err = new Error("E_EXECUTION_GENERATION_SUPERSEDED: 오래된 실행 준비 결과를 폐기했습니다");
  err.code = "E_EXECUTION_GENERATION_SUPERSEDED";
  throw err;
}

function cancelLivePump(reason = "cancelled", { invalidateInFlight = true } = {}) {
  if (state.livePump.timerId !== null) {
    clearTimeout(state.livePump.timerId);
  }
  state.livePump.timerId = null;
  state.livePump.scheduled = false;
  state.livePump.previousStartMs = null;
  state.livePump.scheduledDelayMs = null;
  state.livePump.lastCancelReason = reason;
  if (invalidateInFlight) state.livePump.generation += 1;
}

function resetLivePumpMetricsForRun() {
  state.livePump.pacing = null;
  state.livePump.previousStartMs = null;
  state.livePump.stepStartCount = 0;
  state.livePump.stepCommitCount = 0;
  state.livePump.maxInFlight = 0;
  state.livePump.currentInFlight = 0;
  state.livePump.lastOrigin = "";
}

function livePumpToken() {
  return {
    executionGeneration: Number(state.executionLifecycle.generation),
    pumpGeneration: Number(state.livePump.generation),
    projectId: state.projectId,
    runId: state.activeRunId,
  };
}

function livePumpTokenIsCurrent(token, { allowPausing = false, allowPaused = false } = {}) {
  if (!token || Number(token.pumpGeneration) !== Number(state.livePump.generation)) return false;
  if (!executionGenerationIsCurrent(token.executionGeneration, token.projectId)) return false;
  if (state.activeRunId !== token.runId) return false;
  const status = state.executionLifecycle.status;
  return status === EXECUTION_STATES.RUNNING
    || (allowPausing && status === EXECUTION_STATES.PAUSING)
    || (allowPaused && status === EXECUTION_STATES.PAUSED);
}

async function beginCommittedLiveStep(origin = "auto") {
  if (state.livePump.inFlight) return false;
  const token = livePumpToken();
  const allowPaused = origin === "manual";
  if (!livePumpTokenIsCurrent(token, { allowPaused })) return false;
  state.livePump.inFlight = true;
  state.livePump.currentInFlight += 1;
  state.livePump.maxInFlight = Math.max(state.livePump.maxInFlight, state.livePump.currentInFlight);
  state.livePump.stepStartCount += 1;
  state.livePump.lastOrigin = origin;
  if (origin === "auto") state.livePump.previousStartMs = performance.now();
  renderAll();
  const operation = commitNextLiveStep({ origin, token });
  state.livePump.inFlightPromise = operation;
  try {
    return await operation;
  } finally {
    state.livePump.currentInFlight = Math.max(0, state.livePump.currentInFlight - 1);
    state.livePump.inFlight = false;
    if (state.livePump.inFlightPromise === operation) state.livePump.inFlightPromise = null;
    if (state.executionLifecycle.status === EXECUTION_STATES.PAUSING) {
      applyExecutionEvent(EXECUTION_EVENTS.PAUSE_COMMITTED);
    }
    renderAll();
  }
}

async function runLivePumpTurn(expectedPumpGeneration) {
  if (Number(expectedPumpGeneration) !== Number(state.livePump.generation)) return false;
  if (state.executionLifecycle.status !== EXECUTION_STATES.RUNNING) return false;
  const committed = await beginCommittedLiveStep("auto");
  if (committed && state.executionLifecycle.status === EXECUTION_STATES.RUNNING) {
    scheduleLivePump();
  }
  return committed;
}

function scheduleLivePump() {
  const lifecycle = state.executionLifecycle;
  if (lifecycle.status !== EXECUTION_STATES.RUNNING || lifecycle.mode !== "live") return false;
  if (state.livePump.scheduled || state.livePump.inFlight) return false;
  const expectedPumpGeneration = Number(state.livePump.generation);
  const delayMs = hostMadiDelay(state.livePump.pacing, state.livePump.previousStartMs, performance.now());
  state.livePump.scheduledDelayMs = delayMs;
  state.livePump.scheduled = true;
  state.livePump.timerId = setTimeout(() => {
    state.livePump.timerId = null;
    state.livePump.scheduled = false;
    runLivePumpTurn(expectedPumpGeneration).catch((err) => {
      pushDiagnostic("E_LIVE_PUMP_TURN_FAILED", String(err?.message ?? err), "error");
      renderAll();
    });
  }, delayMs);
  return true;
}

function includesFilter(project, filter) {
  const needle = String(filter ?? "").trim().toLowerCase();
  if (!needle) return true;
  const metadata = project.portableMetadata ?? {};
  const haystack = [
    project.id,
    project.title,
    project.kind,
    project.summary,
    metadata.summary,
    metadata.subject,
    metadata.level,
    metadata.origin_hash_prefix,
    ...(project.tags ?? []),
    ...(metadata.tags ?? []),
    ...(metadata.content_kinds ?? []),
    ...(metadata.required_views ?? []),
    ...(metadata.required_gaji ?? []),
  ]
    .join(" ")
    .toLowerCase();
  return haystack.includes(needle);
}

function projectsForGlobalEntry() {
  if (state.globalEntry === "first-steps") return FIRST_STEPS_PROJECTS;
  if (state.globalEntry === "official-collection") return PROJECTS;
  return allProjects();
}

const { inferContentKinds, inferSubject, defaultPortableMetadata, trimPortableText, portableListFromText, normalizePortableMetadataDraft, metadataDraftFromProject, currentPortableMetadata, currentPortableLineage, loadText, loadProjectFiles } = createProjectMetadata({ currentProject, ONE_FILE_DDN_PROJECT_ID, state, workbenchSourceProfile });

const { displayedProjectTitle, renderGlobalNav, setProjectRailOpen, readMyWorkIndex, writeMyWorkIndex, rememberMyWorkProject, refreshMyWorkIndex, renderMyWork, appendMyWorkRow, renderProjects, renderFileTabs } = createProjectNavigation({ state, els, MY_WORK_INDEX_KEY, localProjectEntryPath, currentProject, openServerProject, readPortableProjectCatalog, projectLink, escapeHtml, projectsForGlobalEntry, includesFilter, DEFERRED_LESSONS, selectProject, selectFile });

const { defaultOneFileDdnSource, readOneFileDraft, writeOneFileDraft, writePortableProjectCatalog, normalizePortableTextArray, localProjectTreePayloadFromSnapshot, projectRunLineageFromPayload, localProjectRunEvidenceFromProject, importedProjectRunEvidenceFromProject, gajiIdsFromLocalProjectPayload, filesFromLocalProjectPayload, portableProjectFromArtifact, loadPortableProjectCatalog, rememberPortableProject, buildOneFileProject, ddnQuotedText, nativeProjectFiles, buildNativeProject, createNativeProject, allProjects } = createWorkbenchProjectRegistry({ nativeSourceAvailable, currentProject, FIRST_STEPS_PROJECTS, localProjectEntryPath, ONE_FILE_DDN_DRAFT_KEY, ONE_FILE_DDN_PROJECT_ID, PORTABLE_PROJECT_CATALOG_KEY, PROJECTS, pushDiagnostic, rememberMyWorkProject, renderAll, selectProject, selectUnversionedDocumentProfile, SOURCE_ANALYSIS_PROFILE_IDENTITY, stableCanonicalJson, stableHash, state, workbenchProjectRegistryState, writeNativeProjectCatalog, readPortableProjectCatalog });

function escapeHtml(value = "") {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function projectLink({ projectId = "", role = "view", capability = "" } = {}) {
  return buildWorkbenchCapabilityLink({
    baseUrl: globalThis.location?.href ?? "",
    projectId,
    role,
    capability,
  });
}

function updateProjectLink(node, { href = "", label = "" } = {}) {
  if (!node) return;
  if (!href) {
    node.href = "#";
    node.textContent = label || "링크 없음";
    node.setAttribute("aria-disabled", "true");
    return;
  }
  node.href = href;
  node.textContent = label;
  node.setAttribute("aria-disabled", "false");
}

const EXECUTION_CONTROL_BINDINGS = Object.freeze([
  ["lifecycle-step", "canStep", "한 마디 앞으로", "최신 마디에서 멈춘 연속 실행만 한 마디 진행할 수 있습니다.", "step", "stepUnavailable"],
  ["lifecycle-stop", "canStop", "중지", "준비 중이거나 활성 상태인 실행이 없습니다.", "stop", "stopUnavailable"],
  ["history-back", "canBack", "한 마디 뒤로 보기", "읽을 수 있는 이전 마디가 없습니다.", "back", "backUnavailable"],
  ["history-branch", "canBranch", "현재 마디에서 갈라 이어가기", "갈라 이어갈 기록 마디가 없습니다.", "branch", "branchUnavailable"],
]);

function renderExecutionControls() {
  const policy = executionControlPolicy(state.executionLifecycle, executionLifecycleContext());
  const primary = document.querySelector("[data-testid='lifecycle-run']");
  const changed = activeRunSourceChanged();
  const action = policy.canPause ? "pause" : policy.canResume && !changed ? "resume" : "start";
  const locale = document.documentElement.dataset.displayLocale ?? 'ko';
  const labelKey = action === "pause" ? "pause" : action === "resume" ? "resume" : changed && policy.canResume ? "runChanged" : "run";
  const label = displayMessage(labelKey, locale).text;
  primary.textContent = action === "pause" ? "Ⅱ" : "▶";
  primary.dataset.action = action;
  primary.title = label;
  primary.setAttribute("aria-label", label);
  primary.disabled = Boolean(state.projectLoadError || state.runtimeBusy || state.sourceAnalysisInFlight)
    || (!policy.canPause && Boolean(state.livePump.inFlight))
    || !(policy.canPause || policy.canResume || policy.canStart);
  const blockConcurrentCommand = Boolean(state.runtimeBusy);
  const bogaeEntryInput=document.querySelector('[data-testid="bogae-entry"]');
  if(bogaeEntryInput) bogaeEntryInput.disabled=blockConcurrentCommand || policy.canPause || policy.canResume;
  const stepInFlight = Boolean(state.livePump.inFlight);
  for (const [testId, policyKey, label, unavailable, labelKey, reasonKey] of EXECUTION_CONTROL_BINDINGS) {
    const button = document.querySelector(`[data-testid='${testId}']`);
    if (!button) continue;
    const preparingStop = testId === "lifecycle-stop" && policy.status === EXECUTION_STATES.PREPARING;
    const pumpBoundaryCommand = testId === "lifecycle-pause" || testId === "lifecycle-stop";
    const concurrentBlocked = (blockConcurrentCommand && !preparingStop)
      || (stepInFlight && !pumpBoundaryCommand);
    const enabled = Boolean(policy[policyKey]) && !concurrentBlocked;
    const reasonCode = concurrentBlocked ? "operation_in_flight" : enabled ? "" : `${policy.status}_${policyKey}`;
    const reasonText = concurrentBlocked ? "이전 실행 명령이 완료되는 중입니다." : unavailable;
    button.disabled = !enabled;
    button.dataset.disabledReason = reasonCode;
    const shownLabel = locale === 'ko' ? label : displayMessage(labelKey, locale).text;
    const shownReason = locale === 'ko' ? reasonText : displayMessage(concurrentBlocked ? 'commandPending' : reasonKey, locale).text;
    button.title = enabled ? shownLabel : `${shownLabel}: ${shownReason}`;
    button.setAttribute("aria-label", enabled ? shownLabel : `${shownLabel}, ${displayMessage('disabled', locale).text}: ${shownReason}`);
  }
  els.root.dataset.executionState = policy.status;
  els.root.dataset.executionMode = policy.mode;
  els.root.dataset.executionGeneration = String(state.executionLifecycle.generation);
  if (workbenchLogicalPresentationState.logicalPanel && (workbenchLogicalPresentationState.logicalPanel.logicalExecution.ownsCurrentSource() || workbenchLogicalPresentationState.logicalPanel.logicalExecution.status().started)) {
    const logical = workbenchLogicalPresentationState.logicalPanel.logicalExecution.status();
    primary.textContent = logical.running ? "Ⅱ" : "▶";
    primary.dataset.action = logical.running ? "pause" : logical.started && !logical.failed ? "resume" : "start";
    primary.title = displayMessage(logical.running ? 'pause' : logical.started && !logical.failed ? 'resume' : 'logicalRun', locale).text;
    primary.setAttribute("aria-label", primary.title);
    primary.disabled = logical.busy || state.runtimeBusy || state.sourceAnalysisInFlight;
    for (const id of ["lifecycle-step","lifecycle-stop"]) {
      const node = document.querySelector(`[data-testid="${id}"]`);
      node.disabled = logical.busy || !logical.started || (id === "lifecycle-step" && logical.running);
      node.removeAttribute("data-disabled-reason");
    }
    for (const id of ["history-back","history-branch"]) document.querySelector(`[data-testid="${id}"]`).disabled = true;
    els.root.dataset.executionOwner = "logical_input";
    els.root.dataset.executionState = logical.running ? "running" : logical.started ? "paused" : "idle";
    const committed=currentLogicalPresentation()?.value;
    els.runStatus.textContent=displayMessage(logical.failed ? 'logicalFailed' : logical.running ? 'logicalRunning' : committed ? 'logicalPaused' : 'logicalReady', locale).text;
    renderCurrentMadi(committed);
    els.runHash.textContent=`state_hash: ${committed?.state_hash ?? committed?.frame?.state_hash ?? '-'}`;
  } else {
    els.root.dataset.executionOwner = "ordinary";
    if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().busy){
      primary.disabled=true;
      for(const id of ['lifecycle-step','lifecycle-stop'])document.querySelector(`[data-testid="${id}"]`).disabled=true;
    }
  }
}

document.addEventListener('ddn-display-locale-change', renderExecutionControls);

function activeRunSourceChanged() {
  const run = activeRun();
  const sourceHash = currentVirtualProjectSourceHash(currentProject())
    || `candidate:fnv1a32:${stableHash(activeRuntimeSourceText(currentProject()))}`;
  return Boolean(run && run.source_hash !== sourceHash);
}

async function primaryExecutionCommand() {
  if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().busy)return false;
  if(currentProject().nativeSaveOnFirstUse&&!workbenchAuthoredPanelAccess.current.getRunProjectArtifact()) {
    if(!window.confirm('이 공식 작품을 새 ID로 내 작업에 저장하고 실행할까요? 기존 작품과 저장본은 바꾸지 않습니다.'))return false;
    try{await saveProjectSnapshot();}catch{return false;}
  }
  if (workbenchLogicalPresentationState.logicalPanel && (workbenchLogicalPresentationState.logicalPanel.logicalExecution.ownsCurrentSource() || workbenchLogicalPresentationState.logicalPanel.logicalExecution.status().started)) return workbenchLogicalPresentationState.logicalPanel.logicalExecution.command('primary');
  const policy = executionControlPolicy(state.executionLifecycle, executionLifecycleContext());
  if (policy.canPause) return pauseRun();
  if (policy.canResume && !activeRunSourceChanged()) return resumeRun();
  if (policy.canResume && !await stopRun()) return false;
  return startRun();
}

function renderServerProject() {
  const server = state.serverProject;
  const project = currentProject();
  const projectId = server.projectId || state.capabilitySession?.project_id || "";
  const revisionId = server.headRevisionId || "";
  const role = state.capabilitySession?.role || server.role || "";
  const canEdit = role === "edit" || !projectId;
  const canPortableClone = Boolean(project.portableArtifact?.content_sha256);
  els.projectServerMeta.textContent = projectId
    ? `프로젝트: ${projectId} · 현재 저장: ${revisionId || "-"} · 권한: ${role === "edit" ? "편집" : role === "view" ? "보기" : "미지정"}`
    : "아직 서버에 저장하지 않았습니다.";
  const saveButton = document.querySelector("[data-testid='project-save']");
  saveButton.disabled = Boolean(projectId && !canEdit);
  renderPersistenceFeedback();
  renderExecutionControls();
  if (els.projectClone) {
    els.projectClone.disabled = !canPortableClone && (!projectId || !state.capabilitySession?.capability);
    els.projectClone.dataset.disabledReason = els.projectClone.disabled ? "clone_source_unavailable" : "";
    els.projectClone.textContent = canPortableClone || role === "view" ? "내 버전 만들기" : "복제본 만들기";
  }
  updateProjectLink(els.projectEditLink, {
    href: projectId && server.editCapability
      ? projectLink({ projectId, role: "edit", capability: server.editCapability })
      : "",
    label: "편집 링크",
  });
  updateProjectLink(els.projectViewLink, {
    href: projectId && server.viewCapability
      ? projectLink({ projectId, role: "view", capability: server.viewCapability })
      : "",
    label: "보기 링크",
  });
  const revisions = Array.isArray(server.revisions) ? server.revisions : [];
  els.projectRevisionList.innerHTML = revisions.length
    ? revisions
      .map((row) => {
        const rid = String(row.revision_id ?? "");
        const content = String(row.content_sha256 ?? "").slice(0, 12);
        const pack = String(row.package_hash ?? "").slice(0, 12);
        const isHead = rid === revisionId;
        const restoreButton = isHead || !canEdit
          ? ""
          : `<button type="button" data-testid="project-restore-revision" data-project-restore-revision="${escapeHtml(rid)}" title="이 리비전을 새 HEAD로 복원">복원</button>`;
        return `<span class="project-revision-item" data-testid="project-revision-item" data-project-revision-id="${escapeHtml(rid)}" data-head="${isHead ? "true" : "false"}"><span>rev ${escapeHtml(rid)} · content ${escapeHtml(content || "-")} · package ${escapeHtml(pack || "-")}</span>${restoreButton}</span>`;
      })
      .join("")
    : "<span>저장 이력 없음</span>";
  const conflict = server.conflict;
  els.projectConflictPanel.classList.toggle("hidden", !conflict);
  if (conflict) {
    els.projectConflictMessage.textContent = conflict.message || "다른 창에서 먼저 저장되었습니다.";
    els.projectConflictServer.textContent = `server: ${conflict.serverRevisionId || "-"} · ${String(conflict.serverContentSha256 || "").slice(0, 16) || "-"}`;
    els.projectConflictLocal.textContent = `local parent: ${conflict.localParentRevisionId || "-"}`;
    els.projectConflictLocalPackage.value = conflict.localPackageJson || "";
  } else {
    els.projectConflictLocalPackage.value = "";
  }
}

function renderPortableLineage(project = currentProject()) {
  if (!els.portableLineageOrigin) return;
  const lineage = project.portableSnapshot?.lineage ?? currentPortableLineage(project);
  const originLabel = project.portableArtifact
    ? String(lineage.source_lesson_id || project.portableSnapshot?.source_lesson_id || "portable")
    : project.id === ONE_FILE_DDN_PROJECT_ID
      ? "custom"
      : project.title;
  const fileHash = project.portableArtifact?.content_sha256
    ? String(project.portableArtifact.content_sha256).slice(0, 12)
    : "-";
  const version = state.serverProject.headRevisionId
    ? `revision ${state.serverProject.headRevisionId}`
    : project.id.includes("_clone_")
      ? project.id
      : "-";
  els.portableLineageOrigin.textContent = `원본: ${originLabel || "-"}`;
  els.portableLineageFile.textContent = `받은 파일: ${fileHash}`;
  els.portableLineageVersion.textContent = `내 버전: ${version}`;
}

function stableHash(text) {
  let hash = 0x811c9dc5;
  for (let i = 0; i < text.length; i += 1) {
    hash ^= text.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function activeSourceText() {
  return state.fileTexts.get("lesson.ddn") ?? state.fileTexts.get(activeFile().path) ?? "";
}

function localProjectEntryPath(project = currentProject()) {
  const payload = localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {});
  return String(payload?.entry_path ?? project.files?.[0]?.path ?? "lesson.ddn").trim() || "lesson.ddn";
}

function activeRuntimeSourceText(project = currentProject()) {
  const entryPath = localProjectEntryPath(project);
  return state.fileTexts.get(entryPath) ?? state.fileTexts.get("lesson.ddn") ?? state.fileTexts.get(activeFile(project).path) ?? "";
}

const { sourceReadinessViewOnly, sourceReadinessApplyOutcome, sourceReadinessRunOutcome, refreshPendingSourceReadiness, refreshSourceReadiness, moduleDirectiveRegistryForProject, refreshAuthoritativeSourceReadiness, getSourceReadinessCandidate, sameAuthoritativeRepairCandidate, sourceReadinessSnapshot, applySourceReadinessFixIt } = createSourceReadinessModel({ state, els, currentProject, activeRuntimeSourceText, localProjectEntryPath, analyzeSourceReadiness, workbenchSourceProfile, currentVirtualProjectSource, SOURCE_ANALYSIS_PROFILE_IDENTITY, runtimeBridge, projectEditDiagnostic, MODULE_DIRECTIVE_ANALYSIS_SNAPSHOT_SCHEMA, projectAuthoritativeSourceAnalysis, projectSourceAnalysisUnavailable, applySourceReadinessFix, SOURCE_READINESS_FIX_ID, pushDiagnostic, renderAll, syncNativeProjectFromState, markCurrentProjectExecutionStale, refreshProjectGraph, ONE_FILE_DDN_PROJECT_ID, writeOneFileDraft });

function currentVirtualProjectSource(project = currentProject()) {
  const payload = localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {});
  if (!payload) return null;
  const files = [];
  const seen = new Set();
  for (const file of Array.isArray(payload.files) ? payload.files : []) {
    const pathValue = String(file?.path ?? "").trim();
    if (!pathValue || seen.has(pathValue)) continue;
    seen.add(pathValue);
    files.push({
      path: pathValue,
      role: String(file?.role ?? "ddn_source").trim() || "ddn_source",
      utf8_text: String(state.fileTexts.get(pathValue) ?? file?.utf8_text ?? file?.text ?? ""),
    });
  }
  if (!files.length) return null;
  return {
    schema: "ddn.virtual_project_source.v1",
    kind: "seamgrim.local_project_tree.v0",
    root_path: String(payload.root_path ?? "").trim(),
    entry_path: String(payload.entry_path ?? "").trim(),
    files,
  };
}

const { currentProjectDdnSources, declaredActionMapFromProject, refreshProjectControls, scheduleProjectControlsRefresh, setupControlTestId, renderProjectControls } = createProjectControls({ buildProjectControlModel, currentProject, els, emptyProjectControlModel, normalizeActionMap, renderExplorationPresets, runtimeBridge, state });

function currentVirtualProjectSourceHash(project = currentProject()) {
  const source = currentVirtualProjectSource(project);
  return virtualProjectSourceHash(source);
}

function virtualProjectSourceHash(source) {
  if (!source) return "";
  return `candidate:fnv1a32:virtual_project_source:${stableHash(stableCanonicalJson(source))}`;
}

function emptyProjectLockTrust() {
  return {
    schema: "seamgrim.workbench.project_lock_trust.v1",
    verified_source_hash: "",
    verified_lock_sha256: "",
    local_non_lock_edit_after_verified: false,
  };
}

function resetProjectExecutionState({ keepImportedRecord = true } = {}) {
  state.projectExecution = {
    importedRecord: keepImportedRecord ? state.projectExecution?.importedRecord ?? null : null,
    current: null,
    previousSuccess: state.projectExecution?.previousSuccess ?? null,
    failure: null,
    lockManuallyEdited: state.projectExecution?.lockManuallyEdited ?? false,
    lockTrust: emptyProjectLockTrust(),
  };
}

function seedImportedProjectExecutionRecord(project = currentProject()) {
  const evidence = importedProjectRunEvidenceFromProject(project);
  state.projectExecution = {
    importedRecord: evidence
      ? {
          schema: "seamgrim.workbench.imported_project_execution_record.v1",
          state: "imported_record",
          evidence,
          source_hash: currentVirtualProjectSourceHash(project),
        }
      : null,
    current: null,
    previousSuccess: state.projectExecution?.previousSuccess ?? null,
    failure: null,
    lockManuallyEdited: false,
    lockTrust: emptyProjectLockTrust(),
  };
}

function markCurrentProjectExecutionStale(reason = "source_edit") {
  notebookHost?.invalidate();
  const current = state.projectExecution?.current;
  if (!current) return;
  state.projectExecution.previousSuccess = {
    ...current,
    state: "previous_success",
    stale_reason: reason,
  };
  state.projectExecution.current = null;
}

function outputRowsFromWasmState(wasmState = {}) {
  const outputLog = wasmState?.resources?.value_json?.output_log;
  if (Array.isArray(outputLog) && outputLog.length) {
    return outputLog.map((row)=>String(row?.text ?? row?.value ?? ""));
  }
  if (Array.isArray(wasmState?.row)) {
    return wasmState.row.map((value) => String(value ?? ""));
  }
  return [];
}

function observationRowsFromWasmState(wasmState = {}, outputRows = []) {
  const channels = Array.isArray(wasmState?.channels) ? wasmState.channels : [];
  return outputRows.map((value, index) => {
    const channel = channels[index] && typeof channels[index] === "object" ? channels[index] : {};
    return {
      index,
      observation_id: `project.obs.${index + 1}`,
      name: String(channel.key ?? `output_${index + 1}`),
      value_text: String(value ?? ""),
      unit: "",
      committed_madi: readMadi(wasmState, index === 0 ? 0 : index),
      target_path: "",
      runtime_state_hash: readStateHash(wasmState, ""),
      source: "workbench_current_file_tree_vm",
      state_hash_participation: false,
    };
  });
}

async function buildProjectObservationTraceFromVm({
  prepared,
  wasmState,
  outputRows,
  observationRows,
  vmLifecycle = "updateLogic+reset+runTicks(1)",
} = {}) {
  const trace = {
    schema: "ddn.local_project.observation_trace.v1",
    source: "workbench_current_file_tree_vm",
    row_count: observationRows.length,
    rows: observationRows.map((row) => ({
      ...row,
      target_path: String(prepared?.target_path ?? row.target_path ?? ""),
    })),
    determinism: {
      source: "workbench-current-file-tree",
      frontdoor: "wasm_project_prepare_link_json",
      vm_lifecycle: vmLifecycle,
      requested_ticks: 1,
      committed_madi: readMadi(wasmState, 0),
      runtime_state_hash: readStateHash(wasmState, ""),
      project_graph_sha256: String(prepared?.project_graph?.project_graph_sha256 ?? ""),
      linked_execution_source_sha256: String(prepared?.linked_execution_source_sha256 ?? ""),
      prepare_link_sha256: String(prepared?.prepare_link_sha256 ?? ""),
      output_rows: outputRows,
      state_hash_formula_change: false,
    },
  };
  const traceSha = await sha256HexUtf8(stableCanonicalJson(trace));
  return {
    ...trace,
    trace_sha256: traceSha ? `sha256:${traceSha}` : `candidate:fnv1a32:trace:${stableHash(stableCanonicalJson(trace))}`,
  };
}

function buildProjectRunLineageFromCurrentExecution({
  prepared,
  run,
  frame,
  boundedStateSha256,
  vmLifecycle = "updateLogic+reset+runTicks(1)",
} = {}) {
  const request = {
    frontdoor: "wasm_project_prepare_link_json",
    vm_lifecycle: vmLifecycle,
    requested_ticks: 1,
    target: String(prepared?.target ?? ""),
    target_path: String(prepared?.target_path ?? ""),
    run_id: String(run?.run_id ?? ""),
  };
  const runRequestHash = `candidate:fnv1a32:project_run_request:${stableHash(stableCanonicalJson(request))}`;
  const lineage = {
    schema: "seamgrim.local_project_run_lineage.v0",
    project_graph_hash: String(prepared?.project_graph?.project_graph_sha256 ?? ""),
    lock_hash: String(prepared?.verified_lock?.lock_sha256 ?? ""),
    linked_source_hash: String(prepared?.linked_execution_source_sha256 ?? ""),
    run_request_hash: runRequestHash,
    parent_run_id: String(run?.parent_run_id ?? ""),
    branch_madi: run?.parent_madi === null || run?.parent_madi === undefined ? null : Number(run.parent_madi),
    branch_state_hash: String(run?.parent_state_hash ?? ""),
    final_state_hash: String(frame?.state_hash ?? ""),
    view_artifact_hash: "",
    created_revision: boundedStateSha256,
  };
  return {
    ...lineage,
    lineage_hash: `candidate:fnv1a32:project_run_lineage:${stableHash(stableCanonicalJson(lineage))}`,
  };
}

function projectExecutionPlanFromPrepared(prepared = {}, projectSourceHash = "") {
  return {
    schema: "seamgrim.workbench.project_execution_plan.v1",
    source_hash: String(projectSourceHash ?? ""),
    frontdoor: "wasm_project_prepare_link_json",
    target: String(prepared?.target ?? ""),
    target_path: String(prepared?.target_path ?? ""),
    project_graph: {
      project_graph_sha256: String(prepared?.project_graph?.project_graph_sha256 ?? ""),
    },
    verified_lock: prepared?.verified_lock
      ? {
          lock_sha256: String(prepared.verified_lock.lock_sha256 ?? ""),
        }
      : null,
    linked_execution_source: String(prepared?.linked_execution_source ?? ""),
    linked_execution_source_sha256: String(prepared?.linked_execution_source_sha256 ?? ""),
    prepare_link_sha256: String(prepared?.prepare_link_sha256 ?? ""),
    execution_mode_decision: requireParsedExecutionModeDecision(
      prepared?.runtime_execution_envelope?.execution_mode_decision,
    ),
    state_hash_formula_change: false,
  };
}

function requireParsedExecutionModeDecision(decision = null) {
  const parsed = decision && typeof decision === "object" ? decision : null;
  const mode = String(parsed?.mode ?? "").trim();
  if (
    !parsed
    || String(parsed.authority ?? "") !== "parsed_program"
    || !["oneshot", "live"].includes(mode)
  ) {
    throw new Error("E_EXECUTION_MODE_DECISION_MISSING: Rust parsed_program 실행 모드 판정이 없습니다");
  }
  return { ...parsed, mode };
}

function recordVerifiedProjectLockTrust(projectSourceHash = "", projectExecutionPlan = null) {
  const lockSha = String(projectExecutionPlan?.verified_lock?.lock_sha256 ?? "").trim();
  if (!lockSha) {
    state.projectExecution.lockTrust = emptyProjectLockTrust();
    return;
  }
  state.projectExecution.lockTrust = {
    schema: "seamgrim.workbench.project_lock_trust.v1",
    verified_source_hash: String(projectSourceHash ?? ""),
    verified_lock_sha256: lockSha,
    local_non_lock_edit_after_verified: false,
  };
  state.projectExecution.lockManuallyEdited = false;
}

async function buildCurrentProjectRunEvidence({
  prepared,
  run,
  frame,
  wasmState,
  vmLifecycle = "updateLogic+reset+runTicks(1)",
} = {}) {
  const outputRows = outputRowsFromWasmState(wasmState);
  const observationRows = observationRowsFromWasmState(wasmState, outputRows);
  const trace = await buildProjectObservationTraceFromVm({
    prepared,
    wasmState,
    outputRows,
    observationRows,
    vmLifecycle,
  });
  const boundedInput = {
    schema: "ddn.local_project_run_summary.bounded_state.v1",
    state_hash: readStateHash(wasmState, ""),
    output_rows: outputRows,
  };
  const boundedSha = await sha256HexUtf8(stableCanonicalJson(boundedInput));
  const boundedStateSha256 = boundedSha
    ? `sha256:${boundedSha}`
    : `candidate:fnv1a32:bounded_state:${stableHash(stableCanonicalJson(boundedInput))}`;
  return {
    schema: "seamgrim.approved_workbench.local_project_run_evidence.v1",
    frontdoor: "wasm_project_prepare_link_json",
    runtime_lifecycle: "workbench_vm_update_reset_tick",
    bounded_state_sha256: boundedStateSha256,
    runtime_state_hash: readStateHash(wasmState, ""),
    summary_sha256: String(prepared?.prepare_link_sha256 ?? ""),
    trace_sha256: trace.trace_sha256,
    run_lineage: buildProjectRunLineageFromCurrentExecution({
      prepared,
      run,
      frame,
      boundedStateSha256,
      vmLifecycle,
    }),
    output_rows: outputRows,
    observation_rows: trace.rows,
    project_observation_trace: trace,
    execution_preparation: {
      schema: "ddn.local_project.execution_preparation.v1",
      source: "workbench_current_file_tree",
      frontdoor: "wasm_project_prepare_link_json",
      prepare_link_sha256: String(prepared?.prepare_link_sha256 ?? ""),
      project_graph_sha256: String(prepared?.project_graph?.project_graph_sha256 ?? ""),
      lock_sha256: String(prepared?.verified_lock?.lock_sha256 ?? ""),
      linked_execution_source_sha256: String(prepared?.linked_execution_source_sha256 ?? ""),
      target: String(prepared?.target ?? ""),
      target_path: String(prepared?.target_path ?? ""),
      state_hash_formula_change: false,
    },
  };
}

function projectDiagnosticFromError(err, fallbackCode = "E_PROJECT_RUN_FAILED") {
  const message = String(err?.message ?? err ?? "");
  try {
    const typed = JSON.parse(message);
    if (typeof typed?.code === "string" && typeof typed?.message === "string") {
      return { ...typed, severity: "error" };
    }
  } catch { /* Older/non-runtime transport errors keep the fallback below. */ }
  const match = message.match(/\b(E_[A-Z0-9_]+)\b/u);
  return {
    code: String(err?.code ?? match?.[1] ?? fallbackCode),
    message: message || fallbackCode,
    severity: "error",
  };
}

function recipeHash() {
  const project = currentProject();
  const sourceHash = currentVirtualProjectSourceHash(project) || `candidate:fnv1a32:${stableHash(activeRuntimeSourceText(project))}`;
  const payload = JSON.stringify({
    project_id: state.projectId,
    source_hash: sourceHash,
    recipe_version: state.recipeVersion,
    setup: state.pendingRecipe ?? {},
  });
  return `candidate:fnv1a32:${stableHash(payload)}`;
}

function buildRunId(prefix = "run") {
  state.runSeq += 1;
  return `${prefix}_${String(state.runSeq).padStart(3, "0")}`;
}

function readMadi(wasmState, fallback = 0) {
  const candidates = [
    wasmState?.tick_id,
    wasmState?.madi,
    wasmState?.tick,
    wasmState?.state?.tick_id,
  ];
  for (const candidate of candidates) {
    const value = Number(candidate);
    if (Number.isFinite(value)) return Math.max(0, Math.trunc(value));
  }
  return fallback;
}

function readStateHash(wasmState, fallback = "-") {
  const value = wasmState?.state_hash ?? wasmState?.stateHash ?? wasmState?.state?.state_hash;
  const text = String(value ?? "").trim();
  return text || fallback;
}

function logicalInputRequired(code, message) {
  return code === "E_LOGICAL_INPUT_PROFILE_REQUIRED" || String(message).includes("E_LOGICAL_INPUT_PROFILE_REQUIRED");
}
function pushDiagnostic(code, message, severity = "warning") {
  if (logicalInputRequired(code, message)) {
    message += "\n이 원문은 논리 행동 입력이 필요합니다. 아래 ‘알파2 논리 입력 실행’에서 입력 계약을 선택하고 ‘현재 원문으로 새로 시작’을 누르세요.";
    const panel = document.querySelector('#logical-input-panel');
    if (panel) {
      panel.open = true;
      const feedback=panel.querySelector('[data-field="feedback"]');
      if (feedback) feedback.textContent=message;
      panel.scrollIntoView({block: "center"});
    }
  }
  state.diagnostics = [
    {
      code,
      message,
      severity,
      at: new Date().toISOString(),
    },
    ...state.diagnostics,
  ].slice(0, 8);
  renderDiagnostics();
}

function clearDiagnostics() {
  state.diagnostics = [];
  renderDiagnostics();
}

const { resizeCanvasToDisplaySize, setPrimaryAspect, drawProjectRunEvidence, buildCommittedFrameCanonicalViewPayloads, buildCommittedCanonicalViewHistory, drawPreview } = createCanvasPresentation({ activeFrame, activeRun, authoredCompositionHost, buildCanonicalViewPayloadSet, buildCommittedSemanticViewPayload, buildCommittedSpace2dOverlay, committedLogicalSpace2d, currentLogicalPresentation, currentProject, displayedProjectTitle, els, EXECUTION_STATES, extractCanonicalViewAuthorityInputsFromState, localProjectRunEvidenceFromProject, overlayViewInstances, primaryViewInstance, reconcileViewWorkspace, renderAuthoredBogae, renderCommittedSemanticView, renderSpace2dCanvas2d, selectCommittedOutputView, spatialView, stableCanonicalJson, stableHash, stableSemanticJson, state, updateSpatialView, workbenchHostExecutionState, workbenchSpatialPresentationState });

const { renderMode, buildViewInstanceSemanticPayload, setViewActionState, renderSideViewInstances, initialDragPoint, renderViewPolicy, selectPrimaryCanvasTarget, renderSemanticObservations, renderRunMetadata, renderPersistenceFeedback, renderDiagnostics } = createViewPresentation({ activeFrame, activeRun, activeRuntimeSourceText, applyExplorationPreset, applySourceReadinessFixIt, authoredCompositionHost, buildCommittedCanonicalViewHistory, buildCommittedFrameCanonicalViewPayloads, buildCommittedSemanticViewPayload, currentLogicalPresentation, currentProject, currentVirtualProjectSourceHash, describePersistenceStatus, els, escapeHtml, EXECUTION_STATES, executionStatusLabel, extractObservationOutputRowsFromState, formatNumber, getSourceReadinessCandidate, loadLocalFallbackSnapshot, localProjectRunEvidenceFromProject, overlayViewInstances, primaryViewInstance, pushDiagnostic, reconcileViewWorkspace, renderAll, renderCommittedSemanticView, renderCurrentMadi, resizeCanvasToDisplaySize, selectCommittedOutputView, selectViewTarget, sideViewInstances, sourceReadinessViewOnly, stableHash, state, viewRoleCapabilities, workbenchHostExecutionState });

const { inputSnapshotRequestBinding, inputSnapshotRequestHash, queuedInputBindingValid, makeSnapshot, buildFrame, appendFrame, runtimeStateSignalsDone, runReachedDone, editableProjectSession, renderRunArchiveHistory, openRunArchiveDetail, refreshRunArchives, createServerRunArchive, persistRunCheckpoint, finalizeRunRecord, finalizeActiveRun, finalizeFailedRunArchive } = createWorkbenchRunArchive({ activeRun, activeRuntimeSourceText, appendRunCheckpoint, buildCheckpointPayload, buildRunArchiveDraft, claimRunFinalization, createRunArchive, currentProject, currentVirtualProjectSourceHash, els, finalizeRunArchive, getRunArchive, listRunArchives, localProjectEntryPath, primaryViewInstance, pushDiagnostic, readMadi, readStateHash, readWorkbenchCapabilitySession, stableCanonicalJson, stableHash, state, workbenchSourceProfile });

const { renderPortableExportStatus, renderPortableImportPreview, renderPortableMetadataEditor, triggerPortableDownload, tapeStatus, exportCurrentInputTape, checkInputTapeFile, buildCurrentPortableProjectArtifact, exportPortableProject, previewPortableProjectFile, openPendingPortableProject, updatePortableMetadataDraftFromInputs, updateServerStateFromPayload } = createPortableActions({ activeRun, activeRuntimeSourceText, buildPortableProjectArtifact, buildPortableProjectInput, currentLocalProjectTreePayload, currentPortableMetadata, currentProject, currentVirtualProjectSource, els, gajiIdsFromLocalProjectPayload, hostBridge, localProjectEntryPath, localProjectTreePayloadFromSnapshot, normalizePortableMetadataDraft, normalizePortableTextArray, parsePortableProjectArtifactBytes, portableListFromText, portableProjectDiagnostic, portableProjectFromArtifact, pushDiagnostic, readPortableProjectCatalog, rememberMyWorkProject, rememberPortableProject, renderAll, runtimeBridge, seedImportedProjectExecutionRecord, selectProject, selectUnversionedDocumentProfile, SOURCE_ANALYSIS_PROFILE_IDENTITY, state, workbenchProjectRegistryState, workbenchSourceProfile, writeWorkbenchCapabilitySession });

const { clearFailedExecutionOutput, startRun, commitNextLiveStep, stepForward, pauseRun, resumeRun, stopRun, backOneMadi, branchFromCursor } = createExecutionController({ activeFrame, activeRun, activeRuntimeSourceText, activeSourceText, alpha2HostPacing, appendFrame, applyExecutionEvent, applyRunRecipeToWasm, assertExecutionGeneration, beginCommittedLiveStep, buildCurrentProjectRunEvidence, buildFrame, buildRunId, cancelLivePump, clearDiagnostics, createServerRunArchive, currentProject, currentVirtualProjectSource, els, EXECUTION_EVENTS, EXECUTION_STATES, executionControlPolicy, executionGenerationIsCurrent, executionLifecycleContext, finalizeActiveRun, finalizeFailedRunArchive, finalizeRunRecord, inputSnapshotRequestBinding, livePumpToken, livePumpTokenIsCurrent, localProjectEntryPath, logicalInputRequired, makeSnapshot, markCurrentProjectExecutionStale, openProgressiveDialog, persistRunCheckpoint, prepareCurrentProjectLink, projectDiagnosticFromError, projectExecutionPlanFromPrepared, pushDiagnostic, queuedInputBindingValid, readStateHash, recipeHash, recordVerifiedProjectLockTrust, refreshAuthoritativeSourceReadiness, refreshGeneratedProjectLockForRun, renderAll, requireParsedExecutionModeDecision, resetLivePumpMetricsForRun, runReachedDone, runtimeBridge, scheduleLivePump, sha256HexUtf8, sharedRunSummaryBindingValid, SOURCE_READINESS_DIAGNOSTIC_CODE, sourceReadinessRunOutcome, stableHash, state, stepWithHostStop, virtualProjectSourceHash, workbenchAuthoredPanelAccess, workbenchHostExecutionState, workbenchSourceProfile });

const { captureRuntimeStorageContext, restoredRuntimeCandidate, saveCurrentRuntimeProject, reopenCurrentRuntimeProject, exportCurrentRuntimeProject } = createRuntimePersistence({ activeFrame, activeRun, activeRunSourceChanged, alpha2HostPacing, buildFrame, buildRunId, commitRuntimeProject, currentProject, currentVirtualProjectSource, drawPreview, EXECUTION_EVENTS, hostBridge, loadRuntimeProject, nativeSourceAvailable, nativeSourceRequest, newRuntimeRevisionHex, projectExecutionPlanFromPrepared, readStateHash, recipeHash, renderAll, runtimeBridge, runtimeStorageRecord, state, syncExecutionLifecycleCompatibility, transitionExecutionLifecycle, virtualProjectSourceHash, workbenchAuthoredPanelAccess, workbenchHostExecutionState, workbenchLogicalPresentationState, workbenchSourceProfile });

const { beginPersistenceOperation, completePersistenceOperation, saveProjectSnapshot, reopenProjectSnapshot } = createProjectPersistence({ activeRun, appendServerRevision, applyExecutionEvent, applySetupRecipe, authoredCompositionHost, backOneMadi, branchFromCursor, buildProjectSnapshot, buildServerProjectArtifact, cancelLivePump, closeAllSideViews, closeViewInstance, createServerProject, currentProject, describePersistenceStatus, editRecoveryGuard, els, EXECUTION_EVENTS, finalizeRunRecord, hydrateProductViewWorkspace, loadLocalFallbackSnapshot, openProjectFromServerSession, primaryExecutionCommand, pushDiagnostic, readWorkbenchCapabilitySession, recordServerConflict, refreshServerRevisions, rememberMyWorkProject, renderAll, renderPersistenceFeedback, renderServerProject, saveCurrentRuntimeProject, saveLocalFallbackSnapshot, selectProjectInputContract, setNextAction, sourceStorageErrorMessage, state, stepForward, stopRun, updateServerStateFromPayload, workbenchAuthoredPanelAccess, workbenchLogicalPresentationState });

const { renderFileDrawer, projectDiagnosticFix, renderProjectGraph, renderSource, completionPosition, completionOffset, closeCompletionPopup, renderCompletionPopup, selectCompletion, requestCompletion, renderProjectMeta, renderFirstSteps } = createEditorPresentation({ activeFile, currentProject, displayedProjectTitle, els, escapeHtml, FIRST_STEPS_PROJECTS, localProjectTreePayloadFromSnapshot, runtimeBridge, selectFile, SOURCE_ANALYSIS_PROFILE_IDENTITY, state, workbenchProjectRegistryState, workbenchSourceProfile });

const { closeGajiBundleDialog, openGajiBundleDialog, previewGajiBundle, publishGajiBundle, currentCapabilityIsViewOnly, selectedShelfProfiles, refreshGajiShelf, reportGajiShelfError, previewShelfInstall, applyShelfInstall, removeShelfInstall, startShelfRemix, publishShelfRemix, openGajiShelf, exportSelectedGajiArtifact, importGajiArtifactFile } = createGajiController({ activeFile, applyVirtualProjectSource, currentProject, currentVirtualProjectSource, ddnQuotedText, els, exportLocalGotganArtifact, hostBridge, importLocalGotganArtifact, loadLocalGotganStore, publishToLocalGotgan, pushDiagnostic, refreshProjectGraph, renderAll, runtimeBridge, searchLocalGotgan, state, triggerPortableDownload, workbenchGajiPresentationState, workbenchProjectRegistryState });

function renderAll() {
  const hostNotice = document.querySelector('[data-testid="host-stop-notice"]');
  hostNotice.textContent = workbenchHostExecutionState.hostStopNotice;
  hostNotice.hidden = !workbenchHostExecutionState.hostStopNotice;
  const project = currentProject();
  refreshSourceReadiness(project);
  renderGlobalNav();
  renderProjects();
  renderProjectMeta(project);
  renderFileTabs(project);
  renderFileDrawer(project);
  renderSource(project);
  proofSourceInspection.observe();
  renderMode();
  // Metadata below consumes semanticRenderModel. Derive it from this committed
  // frame before laying out its observations, then repaint at the final size.
  // Both passes display the same frame; neither executes/advances the world.
  drawPreview();
  renderViewPolicy(project);
  renderSemanticObservations(project);
  renderRunMetadata(project);
  renderRunArchiveHistory();
  renderProjectControls();
  renderServerProject();
  renderPortableLineage(project);
  renderPortableExportStatus();
  renderPortableImportPreview();
  renderPortableMetadataEditor();
  renderDiagnostics();
  els.root.dataset.projectId = project.id;
  els.root.dataset.singleFile = project.files.length === 1 ? "true" : "false";
  // Authored side docks change the available width. Resize and paint only
  // after their layout and all mode controls have been installed.
  drawPreview();
  // Logical execution owns these labels; ordinary metadata must not overwrite them.
  renderExecutionControls();
}

async function selectProjectInputContract(project) {
  if(!project.logicalInput||!workbenchLogicalPresentationState.logicalPanel)return;
  const readContract=async name=>{
    const response=await fetch(new URL(`../../samples/${name}`,import.meta.url));
    if(!response.ok)throw new Error(`E_LOGICAL_INPUT_CONTRACT_LOAD: ${name} (${response.status})`);
    return response.text();
  };
  const [bindingJson,actionsJson]=await Promise.all([readContract(project.logicalInput.binding),readContract(project.logicalInput.actions)]);
  await workbenchLogicalPresentationState.logicalPanel.logicalExecution.selectContract({bindingJson,actionsJson,capacity:project.logicalInput.capacity,reveal:false});
}

async function selectProject(projectId, { preserveServerSession = false, selectedFilePath = "" } = {}) {
  // Only built-in lesson metadata may open a document. Imported projects do
  // not obtain navigation behavior, and the active execution remains intact.
  const documentLesson = FIRST_STEPS_PROJECTS.find(item => item.id === projectId && item.documentOnly === true);
  if (documentLesson) {
    const target = new URL(documentLesson.course.asset_root + documentLesson.course.student_doc, window.location.href);
    if (target.origin !== window.location.origin) throw new Error("E_COURSE_DOCUMENT_ORIGIN");
    window.open(target.href, "_blank", "noopener");
    return;
  }
  state.projectEditGeneration += 1;
  closeProjectEditDialog();
  closeCompletionPopup();
  runtimeBridge.cancelExecution("프로젝트를 바꿔 이전 실행을 중지했습니다.");
  workbenchHostExecutionState.hostStopNotice = "";
  const project = allProjects().find((item) => item.id === projectId) ?? PROJECTS[0];
  const supersededRun = activeRun();
  const inFlightStep = state.livePump.inFlightPromise;
  clearTimeout(state.projectControlsRefreshTimer);
  cancelLivePump("project_switch");
  applyExecutionEvent(EXECUTION_EVENTS.PROJECT_SWITCHED);
  state.runtimeBusy = true;
  renderAll();
  try {
    if (workbenchLogicalPresentationState.logicalPanel) await workbenchLogicalPresentationState.logicalPanel.logicalExecution.resetForProject();
    if (inFlightStep) await inFlightStep.catch(() => false);
    try {
      await finalizeRunRecord(supersededRun, "stopped");
    } catch (archiveErr) {
      pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
    }
    notebookHost?.invalidate();
    state.projectId = project.id;
    clearDiagnostics();
    state.projectControlsGeneration += 1;
    state.projectControls = emptyProjectControlModel(project.id, "loading");
    state.activeFilePath = project.files.some(file => file.path === selectedFilePath)
      ? selectedFilePath : localProjectEntryPath(project);
    state.fileFilter = "";
    els.fileSearch.value = "";
    state.projectLoadError = "";
    if(project.nativeAuthoredFiles&&await nativeSourceAvailable()) {
      // Reuse the already authored project files verbatim. Do not wrap or
      // rewrite a one-file learner program in JavaScript.
      const declaredSource=localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {});
      project.files=project.nativeAuthoredFiles;project.nativeSaveOnFirstUse=true;
      project.portableSnapshot={source_profile_identity:workbenchSourceProfile(project),local_package_payload:{
        kind:'seamgrim.local_project_tree.v0',root_path:declaredSource?.root_path ?? 'root.ddn',entry_path:declaredSource?.entry_path ?? 'main.ddn',
        files:project.files.map(file=>({path:file.path,role:file.role}))}};
      state.activeFilePath=project.files.some(file=>file.path===selectedFilePath)
        ?selectedFilePath:localProjectEntryPath(project);
    }
    await loadProjectFiles(project);
    state.pendingRecipe = null;
    state.nextInput = null;
    state.selectedRunArchive = null;
    state.runs = [];
    state.activeRunId = "";
    state.cursorIndex = -1;
    state.wasm = null;
    resetProjectExecutionState({ keepImportedRecord: false });
    state.projectExecution.lockManuallyEdited = false;
    const portableControls = project.portableSnapshot?.controls_snapshot;
    state.viewWorkspace = portableControls && typeof portableControls === "object"
      ? hydrateProductViewWorkspace(
          project,
          portableControls.view_workspace,
          portableControls.selected_view ?? portableControls.view_role ?? "",
          "portable_project",
        )
      : createViewWorkspace(project);
    state.persistenceStatus = null;
    state.lastPortableExport = null;
    state.portableMetadataDraft = metadataDraftFromProject(project);
    if (!preserveServerSession) {
      state.capabilitySession = null;
      clearWorkbenchCapabilitySession();
      state.serverProject = emptyServerProjectState();
      state.runArchives = [];
    }
    await refreshProjectControls(project);
    await selectProjectInputContract(project);
  } catch (error) {
    state.projectLoadError = String(error?.message ?? error);
    state.fileTexts.clear();
    state.runs = [];
    state.activeRunId = "";
    state.cursorIndex = -1;
    state.wasm = null;
    state.projectControls = emptyProjectControlModel(project.id, "error");
    pushDiagnostic("E_PROJECT_FILE_LOAD", state.projectLoadError, "error");
    if (!els.projectVerificationDrawer.open) els.projectVerificationDrawer.showModal();
  } finally {
    state.runtimeBusy = false;
    renderAll();
  }
  editRecoveryGuard.offer();
  setTimeout(() => refreshProjectGraph().catch(() => {}), 0);
}

async function createOneFileDdnProject() {
  const draft = readOneFileDraft();
  workbenchProjectRegistryState.customOneFileProject = buildOneFileProject(draft);
  await selectProject(ONE_FILE_DDN_PROJECT_ID);
  const source = state.fileTexts.get("lesson.ddn") ?? defaultOneFileDdnSource();
  writeOneFileDraft(source);
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "local_recovery",
    server_saved: false,
    local_saved: true,
    user_label: "새 DDN local recovery 준비",
  };
  pushDiagnostic(
    draft ? "W_ONE_FILE_DDN_RECOVERED" : "W_ONE_FILE_DDN_CREATED",
    draft ? "이 기기의 임시 원문을 복구했습니다." : "새 한 파일 DDN을 만들었습니다.",
  );
  renderAll();
}

function selectFile(path) {
  closeCompletionPopup();
  state.activeFilePath = path;
  renderAll();
}

document.querySelectorAll("[data-global-entry]").forEach((button) => {
  button.addEventListener("click", () => {
    state.globalEntry = button.dataset.globalEntry;
    if (["first-steps", "official-collection", "my-work"].includes(state.globalEntry)) {
      state.projectFilter = "";
      els.projectSearch.value = "";
      setProjectRailOpen(true);
    }
    renderGlobalNav();
    renderProjects();
    if (state.globalEntry === "my-work") {
      refreshMyWorkIndex().catch((err) => {
        pushDiagnostic("E_MY_WORK_REFRESH_FAILED", String(err?.message ?? err), "error");
        renderAll();
      });
    }
  });
});

els.projectRailToggle?.addEventListener("click", () => {
  setProjectRailOpen(els.root.dataset.projectRailOpen !== "true");
});

for (const [name, offset] of [["previous", -1], ["next", 1]]) {
  document.querySelector(`[data-testid='first-steps-${name}']`)?.addEventListener("click", () => {
    const index = FIRST_STEPS_PROJECTS.findIndex(project => project.id === state.projectId);
    const next = FIRST_STEPS_PROJECTS[index + offset];
    if (index >= 0 && next) void selectProject(next.id);
  });
}

document.querySelectorAll("button[data-mode]").forEach((button) => {
  button.addEventListener("click", () => {
    state.mode = button.dataset.mode;
    els.layout.style.removeProperty("--source-width");
    els.layout.style.removeProperty("--bogae-width");
    document.querySelector("[data-testid='panel-divider']").setAttribute("aria-valuenow", state.mode === "bogae-main" ? "40" : "60");
    renderAll();
  });
});

document.querySelectorAll("[data-view-action]").forEach((button) => {
  button.addEventListener("click", () => {
    const project = currentProject();
    const capabilities = viewRoleCapabilities(project, state.semanticViewPayload, state.semanticRenderModel);
    const action = button.dataset.viewAction;
    if (action === "focus-primary") state.viewWorkspace = focusPrimaryView(state.viewWorkspace);
    if (action === "add-side") state.viewWorkspace = openSideView(state.viewWorkspace, project, capabilities);
    if (action === "toggle-overlay") state.viewWorkspace = toggleOverlayView(state.viewWorkspace, project, capabilities);
    if (action === "open-magnifier") state.viewWorkspace = openMagnifierView(state.viewWorkspace, project, capabilities);
    renderAll();
  });
});

els.canvas?.addEventListener("click", (event) => selectPrimaryCanvasTarget(event));
els.bogaeSourceAccess?.addEventListener("click", () => {
  state.mode = "ddn-main";
  renderAll();
  els.sourceEditor?.focus();
});

els.projectSearch.addEventListener("input", () => {
  state.projectFilter = els.projectSearch.value;
  renderProjects();
  if (state.globalEntry === "my-work") renderMyWork();
});

document.querySelector("[data-testid='new-ddn-project']")?.addEventListener("click", () => {
  els.projectCreateDialog?.showModal();
  els.projectCreateName?.focus();
});

function closeProjectCreateDialog() {
  els.projectCreateDialog?.close();
}

document.querySelector("[data-testid='project-create-close']")?.addEventListener("click", closeProjectCreateDialog);
document.querySelector("[data-testid='project-create-cancel']")?.addEventListener("click", closeProjectCreateDialog);
els.projectCreateForm?.addEventListener("submit", (event) => {
  event.preventDefault();
  const data = new FormData(els.projectCreateForm);
  const name = String(data.get("project-name") ?? "").trim();
  const template = String(data.get("project-template") ?? "physics");
  if (!name) {
    els.projectCreateName?.focus();
    return;
  }
  createNativeProject(name, template)
    .then(closeProjectCreateDialog)
    .catch((err) => {
      pushDiagnostic("E_NATIVE_PROJECT_CREATE_FAILED", String(err?.message ?? err), "error");
      renderAll();
    });
});

els.fileSearch.addEventListener("input", () => {
  state.fileFilter = els.fileSearch.value;
  renderFileDrawer(currentProject());
});

function reportProjectEditError(err) {
  const diagnostic = projectEditDiagnostic(err);
  pushDiagnostic(diagnostic.code || "E_PROJECT_EDIT_FAILED", diagnostic.message || String(err), "error");
  renderAll();
}

document.querySelector("[data-testid='file-add']")?.addEventListener("click", () => {
  const path = globalThis.prompt?.("추가할 DDN 파일 경로", "app/새파일.ddn");
  if (!path) return;
  requestProjectEdit({ kind: "add_file", path, utf8_text: "# 새 DDN 파일\n", role: "ddn_source" }).catch(reportProjectEditError);
});

document.querySelector("[data-testid='module-add']")?.addEventListener("click", () => {
  const path = globalThis.prompt?.("추가할 모듈 경로", "lib/새모듈.ddn");
  if (!path) return;
  const moduleName = globalThis.prompt?.("모듈 이름", path.split("/").at(-1)?.replace(/\.ddn$/u, "") || "새모듈");
  if (!moduleName) return;
  requestProjectEdit({ kind: "add_module", path, module_name: moduleName }).catch(reportProjectEditError);
});
document.querySelector("[data-testid='gaji-bundle-open']")?.addEventListener("click", openGajiBundleDialog);
document.querySelector("[data-testid='gaji-bundle-close']")?.addEventListener("click", closeGajiBundleDialog);
document.querySelector("[data-testid='gaji-bundle-cancel']")?.addEventListener("click", closeGajiBundleDialog);
els.gajiBundleForm?.addEventListener("submit", (event) => {
  event.preventDefault();
  previewGajiBundle().catch((err) => {
    workbenchGajiPresentationState.pendingGajiBundle = null;
    els.gajiBundlePublish.disabled = true;
    els.gajiBundleStatus.textContent = String(err?.message ?? err);
    pushDiagnostic("E_GAJI_EXTRACTION_PLAN", String(err?.message ?? err), "error");
  });
});
els.gajiBundlePublish?.addEventListener("click", () => {
  publishGajiBundle().catch((err) => {
    els.gajiBundleStatus.textContent = String(err?.message ?? err);
    pushDiagnostic("E_GAJI_PUBLISH", String(err?.message ?? err), "error");
    renderAll();
  });
});
document.querySelector("[data-testid='gaji-shelf-open']")?.addEventListener("click", () => {
  const menu = document.querySelector("[data-testid='project-more-menu']");
  if (menu) menu.open = false;
  openGajiShelf().catch(reportGajiShelfError);
});
document.querySelector("[data-testid='gaji-shelf-close']")?.addEventListener("click", () => els.gajiShelfDialog?.close?.());
els.gajiShelfSearch?.addEventListener("input", () => refreshGajiShelf().catch(reportGajiShelfError));
document.querySelectorAll("[data-testid='gaji-profile-filters'] input").forEach((node) => {
  node.addEventListener("change", () => refreshGajiShelf().catch(reportGajiShelfError));
});
els.gajiInstallAlias?.addEventListener("input", () => {
  if (!workbenchGajiPresentationState.pendingGajiInstall) return;
  const rowButton = document.querySelector(`.gaji-shelf-row[data-gaji-package-sha256="${workbenchGajiPresentationState.pendingGajiInstall.row.package_sha256}"]`);
  previewShelfInstall(workbenchGajiPresentationState.pendingGajiInstall.row, rowButton).catch(reportGajiShelfError);
});
els.gajiInstallApply?.addEventListener("click", () => applyShelfInstall().catch(reportGajiShelfError));
els.gajiInstallRemove?.addEventListener("click", () => removeShelfInstall().catch(reportGajiShelfError));
els.gajiRemixStart?.addEventListener("click", () => {
  try { startShelfRemix(); } catch (err) { reportGajiShelfError(err); }
});
els.gajiRemixPublish?.addEventListener("click", () => publishShelfRemix().catch(reportGajiShelfError));
document.querySelector("[data-testid='gaji-store-export']")?.addEventListener("click", () => {
  try { exportSelectedGajiArtifact(); } catch (err) { reportGajiShelfError(err); }
});
document.querySelector("[data-testid='gaji-store-import']")?.addEventListener("click", () => document.querySelector("[data-testid='gaji-store-import-input']")?.click());
document.querySelector("[data-testid='gaji-store-import-input']")?.addEventListener("change", (event) => {
  importGajiArtifactFile(event.target.files?.[0]).catch(reportGajiShelfError).finally(() => { event.target.value = ""; });
});

els.fileList.addEventListener("click", (event) => {
  const button = event.target?.closest?.("[data-file-action]");
  if (!button) return;
  event.preventDefault();
  event.stopPropagation();
  const path = String(button.dataset.filePath || "");
  const action = String(button.dataset.fileAction || "");
  let operation = null;
  if (action === "rename") {
    const toPath = globalThis.prompt?.("바꿀 파일 경로", path);
    if (toPath && toPath !== path) operation = { kind: "rename_file", from_path: path, to_path: toPath };
  } else if (action === "duplicate") {
    const toPath = globalThis.prompt?.("복제할 파일 경로", path.replace(/\.ddn$/u, "_복사.ddn"));
    if (toPath) operation = { kind: "duplicate_file", from_path: path, to_path: toPath };
  } else if (action === "set_entry") {
    operation = { kind: "set_entry", path };
  } else if (action === "delete") {
    operation = { kind: "delete_file", path };
  }
  if (operation) requestProjectEdit(operation).catch(reportProjectEditError);
});

document.querySelector("[data-testid='project-edit-undo']")?.addEventListener("click", async () => {
  const index = state.projectEditUndo.findIndex((entry) => entry.projectId === state.projectId);
  const previous = state.projectEditUndo[index];
  if (!previous) {
    pushDiagnostic("W_PROJECT_EDIT_UNDO_EMPTY", "되돌릴 구조 변경이 없습니다.");
    renderAll();
    return;
  }
  try { await applyVirtualProjectSource(previous.source, { rememberUndo: false }); }
  catch(error) { reportProjectEditError(error); return; }
  state.projectEditUndo.splice(index, 1);
  pushDiagnostic("W_PROJECT_EDIT_UNDONE", "최근 구조 변경을 되돌렸습니다.");
  renderAll();
});

function closeProjectEditDialog() {
  state.pendingProjectEdit = null;
  els.projectEditDialog?.close();
}
document.querySelector("[data-testid='project-edit-close']")?.addEventListener("click", closeProjectEditDialog);
document.querySelector("[data-testid='project-edit-cancel']")?.addEventListener("click", closeProjectEditDialog);
document.querySelector("[data-testid='project-edit-apply']")?.addEventListener("click", () => {
  applyPendingProjectEdit().catch(reportProjectEditError);
});

els.sourceEditor.addEventListener("input", (event) => {
  const editedPath = activeFile().path;
  const nextSource = els.sourceEditor.value;
  // 다중행 붙여넣기가 같은 전체 값을 여러 input 이벤트로 전달해도
  // 실제 변경 한 번만 처리한다. 게시 이력·세계 입력은 바꾸지 않는다.
  if (state.fileTexts.get(editedPath) === nextSource) return;
  state.completionDocumentVersion += 1;
  state.completionGeneration += 1;
  closeCompletionPopup();
  state.fileTexts.set(editedPath, nextSource);
  if (editedPath === "ddn.lock") {
    state.projectExecution.lockManuallyEdited = true;
    state.projectExecution.lockTrust = emptyProjectLockTrust();
  } else if (state.projectExecution?.lockTrust?.verified_lock_sha256) {
    state.projectExecution.lockTrust = {
      ...state.projectExecution.lockTrust,
      local_non_lock_edit_after_verified: true,
    };
  }
  state.lastPortableExport = null;
  syncNativeProjectFromState();
  state.projectControlsGeneration += 1;
  state.projectControls = emptyProjectControlModel(state.projectId, "loading");
  state.pendingRecipe = null;
  renderProjectControls();
  scheduleProjectControlsRefresh();
  clearTimeout(state.projectGraphRefreshTimer);
  state.projectGraphRefreshTimer = setTimeout(() => refreshProjectGraph().catch(() => {}), 250);
  markCurrentProjectExecutionStale("source_edit");
  if (state.projectExecution?.failure) {
    state.projectExecution.failure = null;
  }
  if (currentProject().id === ONE_FILE_DDN_PROJECT_ID && activeFile().path === "lesson.ddn") {
    writeOneFileDraft(els.sourceEditor.value);
    state.persistenceStatus = {
      schema: "seamgrim.workbench.persistence_status.v1",
      mode: "local_recovery",
      server_saved: false,
      local_saved: true,
      user_label: "새 DDN local recovery 저장됨",
    };
  } else if (workbenchProjectRegistryState.nativeProjects.some((project) => project.id === currentProject().id)) {
    state.persistenceStatus = {
      schema: "seamgrim.workbench.persistence_status.v1",
      mode: "local_recovery",
      server_saved: false,
      local_saved: true,
      user_label: `${currentProject().title} local recovery 저장됨`,
    };
  }
  refreshSourceReadiness(currentProject());
  renderProjectMeta(currentProject());
  renderDiagnostics();
  renderSemanticObservations(currentProject());
  renderRunMetadata(currentProject());
  renderExecutionControls();
  drawPreview();
  // Suggestions are explicit only; ordinary edits never reopen the popup.
});

els.sourceEditor.addEventListener("blur", (event) => {
  // A completion button keeps its pending click. Other controls and project
  // changes invalidate the request as well as the already displayed menu.
  if (!els.completionPopup?.contains(event.relatedTarget)) closeCompletionPopup();
});

els.sourceEditor.addEventListener("compositionstart", () => {
  state.completionComposing = true;
  closeCompletionPopup();
});
els.sourceEditor.addEventListener("compositionend", () => {
  state.completionComposing = false;
  closeCompletionPopup();
});
document.querySelector('[data-testid="completion-request"]').addEventListener("click", () => {
  els.sourceEditor.focus();
  requestCompletion();
});
els.sourceEditor.addEventListener("scroll", closeCompletionPopup);
window.addEventListener("resize", closeCompletionPopup);
els.sourceEditor.addEventListener("keydown", (event) => {
  if (event.isComposing || event.keyCode === 229 || state.completionComposing) {
    closeCompletionPopup();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && !event.altKey && event.code === "Space") {
    event.preventDefault();
    requestCompletion();
    return;
  }
  if (event.key === "Escape") {
    if (!els.completionPopup?.hidden) event.preventDefault();
    closeCompletionPopup();
    return;
  }
  if (["Enter", "Tab"].includes(event.key) && !(event.ctrlKey || event.metaKey)) {
    closeCompletionPopup();
    return; // Preserve normal newline/focus; never accept a suggestion implicitly.
  }
  if (els.completionPopup?.hidden || !state.completionItems.length) return;
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const delta = event.key === "ArrowDown" ? 1 : -1;
    state.completionSelectedIndex = (state.completionSelectedIndex + delta + state.completionItems.length)
      % state.completionItems.length;
    renderCompletionPopup();
  } else if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    selectCompletion();
  } else if (["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].includes(event.key)) {
    closeCompletionPopup();
  }
});

document.querySelector("[data-testid='spatial-fit']").addEventListener("click", () => {
  fitSpatialHistory(spatialView);
  drawPreview();
});
document.querySelector("[data-testid='spatial-reset']").addEventListener("click", () => {
  resetSpatialRange(spatialView);
  drawPreview();
});
document.querySelector("[data-testid='spatial-trail']").addEventListener("click", (event) => {
  workbenchSpatialPresentationState.showSpatialTrail = !workbenchSpatialPresentationState.showSpatialTrail;
  event.currentTarget.setAttribute("aria-pressed", String(workbenchSpatialPresentationState.showSpatialTrail));
  drawPreview();
});

const panelDivider = document.querySelector("[data-testid='panel-divider']");
const panelLayout = panelDivider.parentElement;
function setSourcePanelWidth(percent) {
  const width = Math.max(25, Math.min(75, percent));
  panelLayout.style.setProperty("--source-width", `${width}fr`);
  panelLayout.style.setProperty("--bogae-width", `${100 - width}fr`);
  panelDivider.setAttribute("aria-valuenow", String(Math.round(width)));
  els.ratioLabel.textContent = `원문 ${Math.round(width)} / 보개 ${Math.round(100 - width)}`;
  drawPreview();
}
panelDivider.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  event.preventDefault();
  panelDivider.setPointerCapture(event.pointerId);
});
panelDivider.addEventListener("pointermove", (event) => {
  if (!panelDivider.hasPointerCapture(event.pointerId)) return;
  const rect = panelLayout.getBoundingClientRect();
  if (rect.width > 0) setSourcePanelWidth(100 * (event.clientX - rect.left) / rect.width);
});
panelDivider.addEventListener("pointerup", (event) => {
  if (panelDivider.hasPointerCapture(event.pointerId)) panelDivider.releasePointerCapture(event.pointerId);
});
panelDivider.addEventListener("keydown", (event) => {
  if (!["ArrowLeft", "ArrowRight", "Home"].includes(event.key)) return;
  event.preventDefault();
  const current = 100 * panelLayout.querySelector(".source-panel").getBoundingClientRect().width
    / panelLayout.getBoundingClientRect().width;
  setSourcePanelWidth(event.key === "Home" ? (state.mode === "bogae-main" ? 40 : 60)
    : current + (event.key === "ArrowRight" ? 5 : -5));
});
window.addEventListener("resize", () => drawPreview());
// Side docks and wrapped controls can resize the canvas without a window
// resize. Repaint the committed frame at the settled display dimensions;
// never advance the world or rerun source in response to layout changes.
const bogaeCanvasResizeObserver = new ResizeObserver(() => {
  const rect = els.canvas.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0) return;
  const width = Math.max(320, Math.floor(rect.width));
  const height = Math.max(240, Math.floor(rect.height));
  if (els.canvas.width !== width || els.canvas.height !== height) drawPreview();
});
bogaeCanvasResizeObserver.observe(els.canvas);

loadNativeProjectCatalog();
loadPortableProjectCatalog();
state.capabilitySession = captureWorkbenchCapabilitySession() ?? readWorkbenchCapabilitySession();
await selectProject(state.projectId, { preserveServerSession: Boolean(state.capabilitySession?.project_id) });
try {
  await openProjectFromServerSession();
} catch (err) {
  pushDiagnostic(err?.payload?.code ?? "E_PROJECT_SERVER_OPEN", err?.message ?? "서버 프로젝트 열기 실패", "error");
  renderAll();
}

async function refreshProjectGraph() {
  const source = currentVirtualProjectSource();
  if (!source) {
    state.projectGraph = null;
    renderProjectGraph();
    return null;
  }
  try {
    const graph = await runtimeBridge.projectGraph(source);
    const symbols = await runtimeBridge.projectSymbols(source);
    state.projectGraph = {
      graph,
      symbols,
      lockStatus: source.files.some((file) => file.path === "ddn.lock") ? "있음/실행 시 검증" : "없음",
      error: null,
    };
  } catch (err) {
    state.projectGraph = { graph: null, symbols: null, lockStatus: "확인 실패", error: projectEditDiagnostic(err) };
  }
  renderProjectGraph();
  return state.projectGraph;
}

async function prepareCurrentProjectLink(projectSource) {
  return runtimeBridge.prepareProjectLink(projectSource, "", true, true);
}

function projectEditDiagnostic(err) {
  const text = String(err?.message ?? err ?? "project edit 실패");
  const start = text.indexOf("{");
  if (start >= 0) {
    try { return JSON.parse(text.slice(start)); } catch (_) { /* use fallback */ }
  }
  return { code: "E_PROJECT_EDIT_FAILED", message: text };
}

function requireProjectEditContext(context) {
  if (context.projectId !== state.projectId || context.generation !== state.projectEditGeneration
      || context.sourceJson !== JSON.stringify(currentVirtualProjectSource())) {
    throw new Error("E_PROJECT_EDIT_SOURCE_CONFLICT: 작품이나 원문이 바뀌어 구조 변경을 취소했습니다");
  }
}

async function requestProjectEdit(operation) {
  const source = currentVirtualProjectSource();
  if (!source || !workbenchProjectRegistryState.nativeProjects.some((item) => item.id === state.projectId)) {
    throw new Error("네이티브 로컬 프로젝트에서만 구조를 편집할 수 있습니다");
  }
  const context = {
    projectId: state.projectId,
    generation: ++state.projectEditGeneration,
    sourceJson: JSON.stringify(source),
  };
  closeProjectEditDialog();
  const beforeHash = await runtimeBridge.projectSourceSha256(source);
  requireProjectEditContext(context);
  const plan = await runtimeBridge.planProjectEdit({
    schema: "ddn.local_project_edit_plan.v1",
    source,
    operation,
    expected_source_sha256: beforeHash,
  });
  requireProjectEditContext(context);
  state.pendingProjectEdit = { ...context, plan };
  els.projectEditSummary.textContent = `${plan.operation} · 파일 ${plan.affected_files?.length ?? 0}개 · 참조 ${plan.affected_references?.length ?? 0}개 · lock ${plan.lock_stale ? "stale→적용 시 갱신" : "유지"}`;
  els.projectEditPreview.textContent = (plan.patches ?? []).map((patch) => (
    `${patch.path} [${patch.reason}]\n- ${patch.before || "(없음)"}\n+ ${patch.after || "(삭제)"}`
  )).join("\n\n") || "변경 patch 없음";
  els.projectEditDialog?.showModal();
  document.querySelector("[data-testid='project-edit-apply']")?.focus();
  return plan;
}

async function applyVirtualProjectSource(source, { rememberUndo = true } = {}) {
  const currentProjectValue=currentProject();
  const authored=Boolean(currentProjectValue.authoredArtifact?.source_snapshot?.root_path);
  const context={projectId:state.projectId,generation:state.projectEditGeneration,sourceJson:JSON.stringify(currentVirtualProjectSource())};
  if(authored) {
    source=await prepareAuthoredGajiDraft({artifact:currentProjectValue.authoredArtifact,source,bridge:runtimeBridge,
      assertCurrent:()=>requireProjectEditContext(context)});
    requireProjectEditContext(context);
  }
  const projects=authored?workbenchProjectRegistryState.receivedPortableProjects:workbenchProjectRegistryState.nativeProjects;
  const index=projects.findIndex(item=>item.id===state.projectId);
  if(index<0)throw new Error("현재 편집할 프로젝트를 찾지 못했습니다");
  if (rememberUndo) {
    state.projectEditUndo = [{ projectId: state.projectId, source: currentVirtualProjectSource() }, ...state.projectEditUndo].slice(0, 10);
  }
  state.projectEditGeneration += 1;
  closeProjectEditDialog();
  const current = projects[index];
  const files = source.files.map((file) => ({ path: file.path, role: file.role, fallback: file.utf8_text }));
  const updated = {
    ...current,
    files,
    portableSnapshot: {
      ...current.portableSnapshot,
      local_package_payload: {
        ...current.portableSnapshot.local_package_payload,
        root_path: source.root_path,
        entry_path: source.entry_path,
        files: source.files,
      },
    },
  };
  projects[index] = updated;
  state.fileTexts = new Map(source.files.map((file) => [file.path, file.utf8_text]));
  if (!source.files.some((file) => file.path === state.activeFilePath)) state.activeFilePath = source.entry_path;
  if(!authored)writeNativeProjectCatalog();
  state.lastPortableExport = null;
  state.pendingRecipe = null;
  state.nextInput = null;
  state.selectedRunArchive = null;
  state.projectControlsGeneration += 1;
  state.projectControls = emptyProjectControlModel(state.projectId, "loading");
  if(authored) {
    state.completionDocumentVersion+=1;state.completionGeneration+=1;closeCompletionPopup();
    markCurrentProjectExecutionStale("source_edit");proofSourceInspection.invalidate();
    refreshSourceReadiness(currentProject());
  }else resetProjectExecutionState({ keepImportedRecord: false });
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "local_recovery",
    server_saved: false,
    local_saved: !authored,
    user_label: authored?"가지 변경을 적용했습니다. 원문을 저장한 뒤 새 실행을 시작하세요.":"프로젝트 구조 변경 local recovery 저장됨",
  };
  renderAll();
  scheduleProjectControlsRefresh();
  refreshProjectGraph().catch(() => {});
}

async function applyPendingProjectEdit() {
  const pending = state.pendingProjectEdit;
  const plan = pending?.plan;
  if (!plan?.atomic_apply) throw new Error("원자 적용 가능한 계획이 아닙니다");
  requireProjectEditContext(pending);
  const currentHash = await runtimeBridge.projectSourceSha256(currentVirtualProjectSource());
  requireProjectEditContext(pending);
  if (currentHash !== plan.before_source_sha256) {
    throw new Error("E_PROJECT_EDIT_SOURCE_CONFLICT: 미리보기 뒤 원문이 바뀌어 적용을 취소했습니다");
  }
  let sourceToApply = plan.project_source;
  if (plan.lock_stale && sourceToApply.files.some((file) => file.path === "ddn.lock")) {
    const sourceWithoutLock = {
      ...sourceToApply,
      files: sourceToApply.files.filter((file) => file.path !== "ddn.lock"),
    };
    const lockText = `${(await runtimeBridge.projectGajiLock(sourceWithoutLock)).trimEnd()}\n`;
    sourceToApply = {
      ...sourceToApply,
      files: sourceToApply.files.map((file) => file.path === "ddn.lock"
        ? { path: "ddn.lock", role: "lock", utf8_text: lockText }
        : file),
    };
  }
  requireProjectEditContext(pending);
  if (state.pendingProjectEdit !== pending) {
    throw new Error("E_PROJECT_EDIT_SOURCE_CONFLICT: 구조 변경 미리보기가 바뀌어 적용을 취소했습니다");
  }
  await applyVirtualProjectSource(sourceToApply);
  state.pendingProjectEdit = null;
  els.projectEditDialog?.close();
  pushDiagnostic("W_PROJECT_EDIT_APPLIED", "검토한 구조 변경을 원자적으로 적용했습니다.");
  renderAll();
}

async function refreshGeneratedProjectLockForRun(projectSource) {
  if (!projectSource || state.projectExecution?.lockManuallyEdited) return projectSource;
  const files = Array.isArray(projectSource.files) ? projectSource.files : [];
  const lockIndex = files.findIndex((file) => String(file?.path ?? "") === "ddn.lock");
  if (lockIndex < 0) return projectSource;
  const lockTrust = state.projectExecution?.lockTrust ?? emptyProjectLockTrust();
  if (!lockTrust.local_non_lock_edit_after_verified || !lockTrust.verified_lock_sha256) {
    return projectSource;
  }
  const currentLockText = String(files[lockIndex]?.utf8_text ?? "");
  if (!currentLockText.includes(String(lockTrust.verified_lock_sha256))) {
    return projectSource;
  }
  const sourceWithoutLock = {
    ...projectSource,
    files: files.filter((file) => String(file?.path ?? "") !== "ddn.lock"),
  };
  const lockPayload = await runtimeBridge.projectGajiLock(sourceWithoutLock, { allowMissing: true });
  if (lockPayload === null) return projectSource;
  const lockText = `${lockPayload.trimEnd()}\n`;
  const nextFiles = files.map((file, index) => index === lockIndex
    ? {
        path: "ddn.lock",
        role: "lock",
        utf8_text: lockText,
      }
    : file);
  state.fileTexts.set("ddn.lock", lockText);
  state.projectExecution.lockTrust = emptyProjectLockTrust();
  return {
    ...projectSource,
    files: nextFiles,
  };
}

function renderExplorationPresets(project) {
  const panel=document.querySelector('[data-testid="exploration-presets"]');
  const spec=project.exploration;
  panel.hidden=!spec;
  if(!spec)return;
  const initialSource=activeRuntimeSourceText(project);
  const buttons=panel.querySelector('[data-testid="exploration-preset-buttons"]');
  if(buttons.dataset.projectId!==project.id) {
    buttons.replaceChildren();buttons.dataset.projectId=project.id;
    for(const preset of spec.presets) {
      const button=document.createElement('button');button.type='button';button.textContent=preset.label;
      button.addEventListener('click',()=>{
        if(els.sourceEditor.readOnly)return;
        try{
          const source=applyExplorationPreset(els.sourceEditor.value,spec.parameters,preset.values);
          els.sourceEditor.value=source;els.sourceEditor.dispatchEvent(new Event('input',{bubbles:true}));
          pushDiagnostic('W_EXPLORATION_SOURCE_EDIT','상황의 초기값을 원문에 반영했습니다. ▶로 다시 계산하세요.');
        }catch(error){pushDiagnostic('E_EXPLORATION_PRESET',String(error.message),'error');renderAll();}
      });
      buttons.append(button);
    }
    // Initial-condition authoring only (A3-P5). Sliding edits the visible
    // source; it does not advance a madi or inject a hidden live event.
    for (const [name,rule] of Object.entries(spec.parameters)) {
      if(rule.hidden)continue;
      const label=document.createElement('label');label.style.cssText='display:inline-flex;align-items:center;gap:4px;margin:4px 8px 4px 0';
      const value=document.createElement('output'), slider=document.createElement('input');slider.type='range';
      slider.min=String(rule.min);slider.max=String(rule.max);slider.step=String(rule.integer?1:(rule.step ?? 0.001));
      slider.dataset.explorationParameter=name;slider.setAttribute('aria-label',`${name} 초기값`);
      const escaped=name.replace(/[.*+?^${}()|[\]\\]/g,'\\$&');
      const match=new RegExp(`^${escaped}\\s*:=\\s*(-?[0-9]+(?:\\.[0-9]+)?)\\s*\\.`,'m').exec(initialSource);
      slider.value=match?.[1] ?? String(rule.min);value.textContent=slider.value;
      slider.addEventListener('input',()=>{
        if(els.sourceEditor.readOnly)return;
        try {
          const source=applyExplorationPreset(els.sourceEditor.value,spec.parameters,{[name]:Number(slider.value)});
          value.textContent=slider.value;els.sourceEditor.value=source;
          els.sourceEditor.dispatchEvent(new Event('input',{bubbles:true}));
        } catch(error){pushDiagnostic('E_EXPLORATION_PRESET',String(error.message),'error');renderAll();}
      });
      label.append(document.createTextNode(name),slider,value);buttons.append(label);
    }
  }
  for(const button of buttons.querySelectorAll('button'))button.disabled=els.sourceEditor.readOnly;
  for(const slider of buttons.querySelectorAll('input')) {
    slider.disabled=els.sourceEditor.readOnly;
    const name=slider.dataset.explorationParameter;
    const escaped=name.replace(/[.*+?^${}()|[\]\\]/g,'\\$&');
    const match=new RegExp(`^${escaped}\\s*:=\\s*(-?[0-9]+(?:\\.[0-9]+)?)\\s*\\.`,'m').exec(initialSource);
    if(match && document.activeElement!==slider) {
      slider.value=match[1];
      slider.nextElementSibling.textContent=slider.value;
    }
  }
  panel.querySelector('p').textContent=spec.guide;
}

function applySetupRecipe() {
  const model = state.projectControls;
  if (model?.project_id !== state.projectId || !Array.isArray(model.controls) || !model.controls.length) {
    pushDiagnostic("E_SETUP_CONTROL_UNAVAILABLE", "현재 프로젝트가 선언한 매김 조절이 없습니다.", "error");
    renderAll();
    return null;
  }
  const values = { ...model.control_values };
  for (const input of els.setupControlList?.querySelectorAll?.("input[data-control-name]") ?? []) {
    const value = Number(input.value);
    if (Number.isFinite(value)) values[String(input.dataset.controlName)] = value;
  }
  const recipe = buildSetupRecipe(model, values);
  if (!recipe) return null;
  state.recipeVersion += 1;
  state.projectControls.control_values = { ...recipe.values };
  state.pendingRecipe = recipe;
  pushDiagnostic("W_SETUP_NEW_RECIPE", "저장한 조절값은 다음 실행부터 적용됩니다.");
  renderAll();
  return recipe;
}

function setNextAction(actionId) {
  const action = actionById(state.projectControls, actionId);
  if (!action || state.projectControls?.project_id !== state.projectId) return null;
  const input = action.input_snapshot;
  const binding = inputSnapshotRequestBinding();
  state.actionSeq += 1;
  state.nextInput = {
    input_id: `input_${String(state.actionSeq).padStart(6, "0")}`,
    project_id: state.projectId,
    action_id: action.action_id,
    action_map_hash: String(state.projectControls.action_map?.action_map_hash ?? ""),
    intent: input.intent,
    keys: input.keys,
    lastKey: input.last_key,
    px: input.pointer_x,
    py: input.pointer_y,
    dt: input.dt,
    dtText: input.dt_text,
    ...binding,
    input_binding_hash: inputSnapshotRequestHash(binding),
  };
  renderAll();
  return state.nextInput;
}

document.addEventListener("keydown", (event) => {
  if (event.altKey || event.ctrlKey || event.metaKey || event.repeat) return;
  const target = event.target;
  if (!(target instanceof Element) || !target.closest("#canvas-bogae, [data-testid='action-controls']")) return;
  const action = state.projectControls?.actions?.find((candidate) => candidate.input_snapshot?.last_key === event.key);
  if (!action) return;
  event.preventDefault();
  setNextAction(action.action_id);
});

async function applyRunRecipeToWasm(wasm, recipe = null) {
  const controls = Array.isArray(recipe?.controls) ? recipe.controls : [];
  for (const control of controls) {
    await wasm.setParam({ key: control.name, value: control.value });
  }
  return controls.length;
}

function attemptOverwriteActiveRun() {
  pushDiagnostic("E_RUN_IMMUTABLE_OVERWRITE_REJECTED", "기존 run record는 덮어쓸 수 없습니다.", "error");
  return false;
}

function attemptReuseRunId(runId = state.activeRunId) {
  const exists = state.runs.some((run) => run.run_id === runId);
  if (exists) {
    pushDiagnostic("E_RUN_ID_REUSE_REJECTED", "같은 run id는 다시 쓸 수 없습니다.", "error");
    return false;
  }
  return true;
}

function attemptMutateParentHash() {
  const run = activeRun();
  if (run?.parent_run_id) {
    pushDiagnostic("E_PARENT_HASH_MUTATION_REJECTED", "parent state_hash는 branch 생성 뒤 바꿀 수 없습니다.", "error");
    return false;
  }
  pushDiagnostic("W_PARENT_HASH_MUTATION_NO_PARENT", "parent가 없는 run입니다.");
  return false;
}

function buildProjectSnapshot() {
  const project = currentProject();
  const payload = {
    schema: "seamgrim.approved_workbench.project_snapshot.v1",
    project_id: project.id,
    source_profile_identity: workbenchSourceProfile(project),
    active_file_path: state.activeFilePath,
    mode: state.mode,
    view_role: legacySelectedView(state.viewWorkspace),
    view_workspace: serializeViewWorkspace(state.viewWorkspace),
    recipe_version: state.recipeVersion,
    files: project.files.map((file) => ({
      path: file.path,
      role: file.role,
      text: state.fileTexts.get(file.path) ?? "",
    })),
    virtual_project_source_hash: currentVirtualProjectSourceHash(project),
    project_execution: {
      current_state: state.projectExecution?.current?.state ?? "",
      current_source_hash: state.projectExecution?.current?.source_hash ?? "",
      previous_success_source_hash: state.projectExecution?.previousSuccess?.source_hash ?? "",
      failure_source_hash: state.projectExecution?.failure?.source_hash ?? "",
      imported_record_source_hash: state.projectExecution?.importedRecord?.source_hash ?? "",
    },
    runs: state.runs.map((run) => ({
      run_id: run.run_id,
      parent_run_id: run.parent_run_id,
      parent_madi: run.parent_madi,
      parent_state_hash: run.parent_state_hash,
      frame_count: run.frames.length,
      latest_state_hash: run.frames.at(-1)?.state_hash ?? "",
      recipe_hash: run.recipe_hash,
      source_hash: run.source_hash,
    })),
  };
  const canonical = stableCanonicalJson(payload);
  return {
    ...payload,
    canonical_hash: `candidate:fnv1a32:${stableHash(canonical)}`,
  };
}

function hydrateProductViewWorkspace(project, snapshot, legacySelection, sourceLabel) {
  const result = hydrateViewWorkspaceWithMigration(project, snapshot, legacySelection);
  if (result.migration) {
    pushDiagnostic(
      result.migration.code,
      `${result.migration.message} (${sourceLabel})`,
    );
  }
  return result.workspace;
}

function workbenchProjectFiles(project = currentProject()) {
  return project.files.map((file) => ({
    path: file.path,
    role: file.role,
    text: state.fileTexts.get(file.path) ?? "",
  }));
}

async function sha256HexUtf8(text = "") {
  const subtle = globalThis?.crypto?.subtle;
  if (!subtle || typeof subtle.digest !== "function") return "";
  const digest = await subtle.digest("SHA-256", new TextEncoder().encode(String(text ?? "")));
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function currentLocalProjectTreePayload(project = currentProject()) {
  const payload = localProjectTreePayloadFromSnapshot(project.portableSnapshot ?? {});
  if (!payload) return null;
  const files = [];
  for (const file of Array.isArray(payload.files) ? payload.files : []) {
    const pathValue = String(file?.path ?? "").trim();
    if (!pathValue) continue;
    const text = String(state.fileTexts.get(pathValue) ?? file?.utf8_text ?? file?.text ?? "");
    files.push({
      ...file,
      path: pathValue,
      role: String(file?.role ?? "ddn_source").trim() || "ddn_source",
      utf8_text: text,
      byte_length: new TextEncoder().encode(text).length,
      sha256: await sha256HexUtf8(text),
    });
  }
  const evidence = localProjectRunEvidenceFromProject(project);
  if (evidence) {
    const projectGraphSha256 = String(evidence.execution_preparation?.project_graph_sha256 || payload.project_graph_sha256 || "");
    const lockSha256 = String(evidence.execution_preparation?.lock_sha256 || evidence.run_lineage?.lock_hash || payload.lock_sha256 || "");
    return {
      ...payload,
      files,
      project_graph_sha256: projectGraphSha256,
      lock_sha256: lockSha256,
      bounded_state_sha256: evidence.bounded_state_sha256,
      runtime_state_hash: evidence.runtime_state_hash,
      summary_sha256: evidence.summary_sha256,
      output_rows: evidence.output_rows,
      project_observation_trace: evidence.project_observation_trace ?? {
        schema: "ddn.local_project.observation_trace.v1",
        trace_sha256: evidence.trace_sha256,
        rows: evidence.observation_rows,
      },
      execution_preparation: evidence.execution_preparation,
      run_lineage: evidence.run_lineage,
      frontdoor: evidence.frontdoor,
      state_hash_formula_change: false,
    };
  }
  return {
    ...payload,
    files,
  };
}

function currentProjectControlSnapshot() {
  const model = state.projectControls?.project_id === state.projectId
    ? state.projectControls
    : emptyProjectControlModel(state.projectId);
  return {
    authoring_plan_hash: String(model.authoring_plan_hash ?? ""),
    authoring_source: String(model.source ?? ""),
    controls: (Array.isArray(model.controls) ? model.controls : []).map((control) => ({
      name: control.name,
      value: Number(model.control_values?.[control.name] ?? control.value),
    })),
    action_map: model.action_map
      ? {
          schema: model.action_map.schema,
          project_id: model.action_map.project_id,
          actions: model.action_map.actions.map((action) => ({
            action_id: action.action_id,
            label: action.label,
            glyph: action.glyph,
            input_snapshot: { ...action.input_snapshot },
          })),
        }
      : null,
  };
}

async function buildServerProjectArtifact() {
  const project = currentProject();
  const localProjectTreePayload = await currentLocalProjectTreePayload(project);
  return buildWorkbenchProjectArtifact({
    projectId: project.id,
    projectName: project.title,
    files: workbenchProjectFiles(project),
    controlsSnapshot: {
      selected_view: legacySelectedView(state.viewWorkspace),
      view_workspace: serializeViewWorkspace(state.viewWorkspace),
      run_mode: state.mode,
      recipe_version: state.recipeVersion,
      ...currentProjectControlSnapshot(),
    },
    localPackagePayload: localProjectTreePayload ?? project.portableSnapshot?.local_package_payload ?? null,
    lineage: currentPortableLineage(project),
    metadata: currentPortableMetadata(project),
    sourceProfileIdentity: workbenchSourceProfile(project),
  });
}

function buildPortableProjectInput(project = currentProject()) {
  const metadata = currentPortableMetadata(project);
  const sourceText = activeSourceText();
  return {
    projectName: metadata.title || project.title,
    ddnText: sourceText,
    lessonPlanText: String(project.lessonPlanText ?? project.portableSnapshot?.lesson_plan_text ?? ""),
    sourceLessonId: project.id === ONE_FILE_DDN_PROJECT_ID ? "custom" : project.id,
    controlsSnapshot: {
      selected_view: legacySelectedView(state.viewWorkspace),
      view_workspace: serializeViewWorkspace(state.viewWorkspace),
      run_mode: state.mode,
      recipe_version: state.recipeVersion,
      ...currentProjectControlSnapshot(),
      view_authoring: {
        primary_view: project.primaryView,
        view_grade: project.viewGrade,
      },
    },
    localPackagePayload: project.portableSnapshot?.local_package_payload ?? null,
    curriculumInquiryArtifact: project.portableSnapshot?.curriculum_inquiry_artifact ?? null,
    curriculumArtifactBundle: project.portableSnapshot?.curriculum_artifact_bundle ?? null,
    curriculumModelComparison: project.portableSnapshot?.curriculum_model_comparison ?? null,
    curriculumSemanticScene: project.portableSnapshot?.curriculum_semantic_scene ?? null,
    lineage: {
      ...(project.portableSnapshot?.lineage ?? {}),
      ...currentPortableLineage(project),
    },
    metadata,
  };
}

function applyProjectSnapshot(snapshot = {}) {
  const sourceLessonId = String(snapshot.source_lesson_id ?? "").trim();
  const localProjectPayload = localProjectTreePayloadFromSnapshot(snapshot);
  const known = allProjects().find((project) => project.id === sourceLessonId);
  let project = known ?? currentProject();
  if (!localProjectPayload) {
    // A saved document cannot borrow the built-in lesson's version merely
    // because its source_lesson_id happens to name that lesson.
    project = portableProjectFromArtifact({ project_snapshot: snapshot }, {
      catalogId: `server_document_${sourceLessonId || "custom"}`,
    });
    workbenchProjectRegistryState.receivedPortableProjects = [project, ...workbenchProjectRegistryState.receivedPortableProjects.filter((row) => row.id !== project.id)];
  }
  if (localProjectPayload) {
    const hashSource = String(snapshot.project_graph_sha256 ?? localProjectPayload.project_graph_sha256 ?? sourceLessonId ?? "").trim() || "server";
    const hashPrefix = hashSource.slice(0, 12) || "server";
    project = {
      id: sourceLessonId && sourceLessonId !== "custom" ? sourceLessonId : `server_local_project_${hashPrefix}`,
      title: String(snapshot.project_name ?? "저장된 로컬 프로젝트"),
      kind: "저장된 로컬 프로젝트",
      summary: "서버 snapshot에서 복원한 여러 파일 DDN 프로젝트입니다.",
      tags: ["저장", "로컬 프로젝트"],
      primaryView: known?.primaryView ?? "line",
      viewGrade: "portable-verified",
      viewPolicy: known?.viewPolicy ?? {
        primary: "복원된 DDN 보개",
        side: "복원된 관찰값",
        overlay: "복원된 보기 후보",
        magnifier: "복원된 보기 확대",
      },
      portableMetadata: snapshot.metadata ?? {},
      portableSnapshot: snapshot,
      portableArtifact: null,
      files: filesFromLocalProjectPayload(localProjectPayload, snapshot),
    };
    workbenchProjectRegistryState.receivedPortableProjects = [
      project,
      ...workbenchProjectRegistryState.receivedPortableProjects.filter((row) => row.id !== project.id),
    ];
  }
  const supersededRun = activeRun();
  cancelLivePump("project_snapshot_replaced");
  applyExecutionEvent(EXECUTION_EVENTS.PROJECT_SWITCHED);
  state.runtimeBusy = false;
  state.runs = [];
  state.activeRunId = "";
  state.cursorIndex = -1;
  state.wasm = null;
  finalizeRunRecord(supersededRun, "stopped").catch((archiveErr) => {
    pushDiagnostic("E_RUN_ARCHIVE_FINALIZE_FAILED", String(archiveErr?.message ?? archiveErr), "error");
  });
  notebookHost?.invalidate();
  state.projectId = project.id;
  state.activeFilePath = localProjectPayload
    ? String(localProjectPayload.entry_path ?? project.files[0]?.path ?? "lesson.ddn")
    : "lesson.ddn";
  state.fileTexts.clear();
  for (const file of project.files) {
    state.fileTexts.set(
      file.path,
      localProjectPayload
        ? String(file.fallback ?? "")
        : file.path === "lesson.ddn"
          ? String(snapshot.ddn_text ?? "")
          : String(file.fallback ?? ""),
    );
  }
  state.loadedFileTexts = new Map(state.fileTexts);
  state.mode = ["ddn-main", "bogae-main", "bogae-full"].includes(snapshot.controls_snapshot?.run_mode)
    ? snapshot.controls_snapshot.run_mode
    : "ddn-main";
  state.viewWorkspace = hydrateProductViewWorkspace(
    project,
    snapshot.controls_snapshot?.view_workspace,
    snapshot.controls_snapshot?.selected_view ?? snapshot.controls_snapshot?.view_role ?? "",
    "project_snapshot",
  );
  state.pendingRecipe = null;
  state.nextInput = null;
  state.projectControlsGeneration += 1;
  state.projectControls = emptyProjectControlModel(project.id, "loading");
  scheduleProjectControlsRefresh();
  resetProjectExecutionState({ keepImportedRecord: false });
  seedImportedProjectExecutionRecord(project);
}

async function refreshServerRevisions() {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  const projectId = state.serverProject.projectId || session?.project_id || "";
  const capability = session?.capability || state.serverProject.editCapability || state.serverProject.viewCapability || "";
  if (!projectId || !capability) {
    state.serverProject.revisions = [];
    renderServerProject();
    return [];
  }
  const payload = await listServerRevisions({ projectId, capability });
  state.serverProject.revisions = Array.isArray(payload.revisions) ? payload.revisions : [];
  state.serverProject.role = String(payload.capability_role ?? state.serverProject.role ?? "");
  renderServerProject();
  return state.serverProject.revisions;
}

async function recordServerConflict(err, artifact) {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  const projectId = state.serverProject.projectId || session?.project_id || "";
  let head = null;
  if (projectId && session?.capability) {
    try {
      head = await openServerProject({ projectId, capability: session.capability });
      updateServerStateFromPayload(head, {
        editCapability: session.role === "edit" ? session.capability : "",
        viewCapability: session.role === "view" ? session.capability : "",
      });
      await refreshServerRevisions();
    } catch (_) {
      // The original conflict diagnostic remains the user-facing result.
    }
  }
  state.serverProject.conflict = {
    schema: "seamgrim.workbench.project_conflict.v1",
    message: err?.message || "다른 창에서 먼저 저장되었습니다.",
    serverRevisionId: String(head?.head_revision_id ?? state.serverProject.headRevisionId ?? ""),
    serverContentSha256: String(head?.content_sha256 ?? state.serverProject.contentSha256 ?? ""),
    localParentRevisionId: String(state.persistenceStatus?.revision_id ?? state.serverProject.headRevisionId ?? ""),
    localPackageJson: stableCanonicalJson(artifact),
  };
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "server_conflict",
    server_saved: false,
    local_saved: false,
    revision_id: state.serverProject.headRevisionId,
    user_label: "다른 창에서 먼저 저장됨",
  };
  els.saveStatus.textContent = "다른 창에서 먼저 저장됨";
  pushDiagnostic(err?.payload?.code ?? "E_PROJECT_REVISION_CONFLICT", state.serverProject.conflict.message, "error");
  renderAll();
  return state.serverProject.conflict;
}

async function restoreProjectRevision(sourceRevisionId = "") {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  const projectId = state.serverProject.projectId || session?.project_id || "";
  if (!projectId || !session?.capability || session.role !== "edit") {
    pushDiagnostic("E_PROJECT_RESTORE_PERMISSION", "편집 권한이 있어야 리비전을 복원할 수 있습니다.", "error");
    return null;
  }
  const payload = await restoreServerRevision({
    projectId,
    capability: session.capability,
    sourceRevisionId,
    expectedParentRevisionId: state.serverProject.headRevisionId,
  });
  updateServerStateFromPayload(payload, { editCapability: session.capability });
  state.serverProject.conflict = null;
  if (payload.project_snapshot) {
    applyProjectSnapshot(payload.project_snapshot);
  }
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "server_restore",
    server_saved: true,
    local_saved: false,
    revision_id: state.serverProject.headRevisionId,
    user_label: `복원됨 ${state.serverProject.headRevisionId}`,
  };
  els.saveStatus.textContent = `복원됨 ${state.serverProject.headRevisionId}`;
  await refreshServerRevisions();
  pushDiagnostic("W_PROJECT_REVISION_RESTORED", `${sourceRevisionId} 새 revision으로 복원 완료`);
  renderAll();
  return payload;
}

async function cloneCurrentServerProject() {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  const projectId = state.serverProject.projectId || session?.project_id || "";
  if (!projectId || !session?.capability) {
    pushDiagnostic("E_PROJECT_CLONE_NO_CAPABILITY", "복제할 서버 프로젝트 권한이 없습니다.", "error");
    return null;
  }
  const payload = await cloneServerProject({
    projectId,
    capability: session.capability,
    title: `${currentProject().title} 복제본`,
  });
  updateServerStateFromPayload(payload, {
    editCapability: payload.edit_capability,
    viewCapability: payload.view_capability,
  });
  state.serverProject.conflict = null;
  state.runArchives = [];
  if (payload.project_snapshot) {
    applyProjectSnapshot(payload.project_snapshot);
  }
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "server_project",
    server_saved: true,
    local_saved: false,
    revision_id: state.serverProject.headRevisionId,
    user_label: `복제본 저장 ${state.serverProject.headRevisionId}`,
  };
  els.saveStatus.textContent = describePersistenceStatus(state.persistenceStatus);
  renderAll();
  await refreshServerRevisions();
  await refreshRunArchives();
  pushDiagnostic("W_PROJECT_CLONED", `복제본 ${state.serverProject.projectId} 생성 완료`);
  renderAll();
  return payload;
}

async function cloneCurrentPortableProject() {
  const source = currentProject();
  if (!source.portableArtifact?.content_sha256) {
    pushDiagnostic("E_PORTABLE_CLONE_NO_ORIGIN", "복제할 받은 프로젝트가 없습니다.", "error");
    return null;
  }
  const hashPrefix = String(source.portableArtifact.content_sha256).slice(0, 12);
  const cloneId = `portable_${hashPrefix}_clone_${String(Date.now()).slice(-6)}`;
  const snapshot = {
    ...(source.portableSnapshot ?? {}),
    lineage: {
      ...(source.portableSnapshot?.lineage ?? {}),
      origin_kind: "portable_clone",
      origin_content_sha256: source.portableArtifact.content_sha256,
    },
  };
  const clone = {
    ...source,
    id: cloneId,
    title: `${source.title} 내 버전`,
    kind: "내 작업",
    tags: Array.from(new Set([...(source.tags ?? []), "내 작업", "리믹스"])),
    portableSnapshot: snapshot,
    portableMetadata: {
      ...(source.portableMetadata ?? {}),
      title: `${source.title} 내 버전`,
      origin_hash_prefix: hashPrefix,
      trust_badge: "local_remix",
    },
  };
  workbenchProjectRegistryState.receivedPortableProjects = [clone, ...workbenchProjectRegistryState.receivedPortableProjects];
  await selectProject(clone.id);
  seedImportedProjectExecutionRecord(clone);
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "portable_local_clone",
    server_saved: false,
    local_saved: true,
    user_label: `받은 프로젝트 내 버전 ${hashPrefix}`,
  };
  pushDiagnostic("W_PORTABLE_PROJECT_CLONED", "받은 프로젝트의 내 버전을 만들었습니다.");
  renderAll();
  return clone;
}

async function openProjectFromServerSession() {
  const session = state.capabilitySession ?? readWorkbenchCapabilitySession();
  if (!session?.project_id || !session?.capability) return null;
  if (!/^sgp_[A-Za-z0-9]{12,24}$/.test(session.project_id)) return null;
  const payload = await openServerProject({
    projectId: session.project_id,
    capability: session.capability,
  });
  updateServerStateFromPayload(payload, {
    editCapability: session.role === "edit" ? session.capability : "",
    viewCapability: session.role === "view" ? session.capability : "",
  });
  if (payload.project_snapshot) {
    applyProjectSnapshot(payload.project_snapshot);
  }
  state.persistenceStatus = {
    schema: "seamgrim.workbench.persistence_status.v1",
    mode: "server_project",
    server_saved: true,
    local_saved: false,
    revision_id: state.serverProject.headRevisionId,
    user_label: `서버 프로젝트 다시 열림 ${state.serverProject.headRevisionId}`,
  };
  els.saveStatus.textContent = describePersistenceStatus(state.persistenceStatus);
  await refreshServerRevisions();
  await refreshRunArchives();
  pushDiagnostic("W_PROJECT_SERVER_OPENED", `server project ${state.serverProject.projectId} 다시 열림`);
  renderAll();
  return payload;
}

function openProgressiveDialog(dialog) {
  const menu = document.querySelector("[data-testid='project-more-menu']");
  if (menu) menu.open = false;
  if (dialog && !dialog.open) dialog.showModal();
}

document.querySelector("[data-testid='project-storage-open']")?.addEventListener("click", () => openProgressiveDialog(els.projectStorageDrawer));

document.querySelector("[data-testid='authored-project-open']")?.addEventListener("click", () => openProgressiveDialog(document.querySelector('[data-testid="authored-project-drawer"]')));
document.querySelector('[data-authored-close]')?.addEventListener('click',()=>{
  const dialog=document.querySelector('[data-testid="authored-project-drawer"]');
  if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().started)workbenchLogicalPresentationState.logicalPanel.logicalExecution.closeViewDialog(dialog);
  else dialog.close();
});
const authoredProjectPanel = installAuthoredProjectPanel({
  root:document.querySelector('[data-testid="authored-project-drawer"]'),bridge:runtimeBridge,host:hostBridge,
  saveRuntime:saveCurrentRuntimeProject,
  reopenRuntime:reopenCurrentRuntimeProject,
  ownsLogicalSource:()=>Boolean(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.ownsCurrentSource()),
  exportRuntime:exportCurrentRuntimeProject,
  saveProject:saveProjectSnapshot,
  reopenProject:reopenProjectSnapshot,
  capture:()=>({
    selected_project:state.projectId,
    source_profile:workbenchSourceProfile(currentProject()),
    source_snapshot:{entry_path:localProjectEntryPath(currentProject()),files:currentProject().files.map(file=>({path:file.path,text:state.fileTexts.get(file.path)??''})),
      ...((currentProject().authoredArtifact?.source_snapshot.root_path ?? localProjectTreePayloadFromSnapshot(currentProject().portableSnapshot ?? {})?.root_path)
        ? {root_path:currentProject().authoredArtifact?.source_snapshot.root_path ?? localProjectTreePayloadFromSnapshot(currentProject().portableSnapshot ?? {}).root_path}:{}),
    },
  }),
  apply:async (artifact,projectContext={})=>{
    const id=`authored_${artifact.authored_identity.logical_dependency_id}`;
    // Display context only. The admitted artifact remains the sole source of
    // authored identity, source bytes and registration; never infer them here.
    const savedTitle=readMyWorkIndex().find(row=>row.project_id===artifact.authored_identity.logical_dependency_id)?.title;
    const previous=workbenchProjectRegistryState.receivedPortableProjects.find(row=>row.id===id);
    const title=[projectContext.title,savedTitle,previous?.title].find(value=>typeof value==='string'&&value.trim())??'명시 작품';
    const snapshot=artifact.source_snapshot;
    let files;
    if(snapshot.root_path) {
      // Read the same Rust-owned role projection used by saved-source admission.
      // This JSON supplies no execution or storage permit; those gates still run.
      const context=()=>JSON.stringify([state.projectId,state.projectEditGeneration,
        state.sourceAnalysisGeneration,state.completionGeneration,state.activeFilePath,workbenchSourceProfile(currentProject())]);
      const expectedContext=context();
      const projected=JSON.parse(await runtimeBridge.authoredProjectOperation({
        operation:'source_project',text:JSON.stringify(artifact),
      }));
      if(context()!==expectedContext)throw new Error('E_PROJECT_EDIT_SOURCE_CONFLICT: 작품이나 원문이 바뀌어 구조 변경을 취소했습니다');
      if(projected.schema!=='ddn.virtual_project_source.v1'||projected.root_path!==snapshot.root_path
        ||projected.entry_path!==snapshot.entry_path||!Array.isArray(projected.files)
        ||projected.files.length!==snapshot.files.length
        ||!projected.files.every((file,index)=>file.path===snapshot.files[index].path
          &&file.utf8_text===snapshot.files[index].text&&typeof file.role==='string'&&file.role.length>0)) {
        throw new Error('E_PROJECT_RUNTIME_SOURCE_SUPPLY: 원자료와 파일 역할 공급이 다릅니다');
      }
      files=projected.files.map(file=>({path:file.path,role:file.role,fallback:file.utf8_text}));
    }else files=snapshot.files.map(file=>({path:file.path,role:'main',fallback:file.text}));
    const project={id,title,kind:'내 작업',summary:'명시 정체를 가진 원문입니다. 등록한 보개짜임은 저장된 정체로 실행합니다.',tags:['내 작업'],primaryView:'line',viewGrade:'output-only',viewPolicy:{primary:'원문 결과',side:'',overlay:'',magnifier:''},sourceProfileIdentity:artifact.source_profile,files,authoredArtifact:artifact};
    if(projectContext.logicalInput) {project.logicalInput={...projectContext.logicalInput};}
    if(snapshot.root_path) project.portableSnapshot={source_profile_identity:artifact.source_profile,local_package_payload:{kind:'seamgrim.local_project_tree.v0',root_path:snapshot.root_path,entry_path:snapshot.entry_path,files:files.map(file=>({path:file.path,role:file.role,utf8_text:file.fallback}))}};
    workbenchProjectRegistryState.receivedPortableProjects=[project,...workbenchProjectRegistryState.receivedPortableProjects.filter(row=>row.id!==id)];
    // An acknowledged update of this same work must not navigate from the
    // edited source to the start file. A different work still opens its entry.
    const selectedFilePath = state.projectId === id ? state.activeFilePath : "";
    await selectProject(id, { selectedFilePath });
    // A restored source owns a fresh check context even when its bytes match.
    proofSourceInspection.invalidate();
    document.querySelector('[data-testid="bogae-entry"]').value='';
    // Derive expected state from admitted bytes, never from the mutable editor
    // after selectProject's awaits. The panel verifies before publishing.
    return {selected_project:id,source_profile:artifact.source_profile,source_snapshot:artifact.source_snapshot};
  },
});
document.querySelector("[data-testid='first-steps-close']")?.addEventListener("click", () => {
  const guide = document.querySelector("[data-testid='first-steps-guide']");
  if (guide) guide.open = false;
  document.querySelector("[data-testid='first-steps-open']")?.focus();
});
document.addEventListener("keydown", (event) => {
  const guide = document.querySelector("[data-testid='first-steps-guide']");
  if (event.key === "Escape" && guide?.open) {
    guide.open = false;
    document.querySelector("[data-testid='first-steps-open']")?.focus();
  }
});
document.querySelector("[data-testid='project-history-open']")?.addEventListener("click", () => {
  openProgressiveDialog(els.projectHistoryDrawer);
  refreshRunArchives().catch((err) => {
    pushDiagnostic("E_RUN_ARCHIVE_LIST_FAILED", String(err?.message ?? err), "error");
  });
});
document.querySelector("[data-testid='project-verification-open']")?.addEventListener("click", () => openProgressiveDialog(els.projectVerificationDrawer));
document.querySelector("[data-testid='portable-project-settings']")?.addEventListener("click", () => openProgressiveDialog(els.portableDialog));
document.querySelector("[data-testid='project-storage-close']")?.addEventListener("click", () => els.projectStorageDrawer?.close());
document.querySelector("[data-testid='project-history-close']")?.addEventListener("click", () => els.projectHistoryDrawer?.close());
document.querySelector("[data-testid='project-verification-close']")?.addEventListener("click", () => els.projectVerificationDrawer?.close());
document.querySelector("[data-testid='portable-project-dialog-close']")?.addEventListener("click", () => els.portableDialog?.close());
els.runArchiveList?.addEventListener("click", (event) => {
  const button = event.target?.closest?.("button[data-server-run-id]");
  if (!button) return;
  openRunArchiveDetail(String(button.dataset.serverRunId)).catch((err) => {
    pushDiagnostic("E_RUN_ARCHIVE_OPEN_FAILED", String(err?.message ?? err), "error");
  });
});
document.querySelector("[data-testid='project-save']")?.addEventListener("click", () => {
  if (state.persistenceOperation?.phase === "saving") return;
  saveProjectSnapshot().catch(() => renderAll());
});
document.querySelector("[data-testid='project-reopen']")?.addEventListener("click", () => {
  reopenProjectSnapshot().catch(() => renderAll());
});
document.addEventListener("click", (event) => {
  const target = event.target?.nodeType === Node.TEXT_NODE ? event.target.parentElement : event.target;
  if (!target?.closest?.("[data-testid='portable-project-export']")) return;
  event.preventDefault();
  exportPortableProject().catch(() => renderAll());
});
document.querySelector("[data-testid='portable-project-export-confirm']")?.addEventListener("click", () => {
  exportPortableProject().catch(() => renderAll());
});
els.portableImport?.addEventListener("click", () => {
  els.portableImportInput?.click();
});
const sourceFileInput=document.querySelector('[data-testid="source-file-input"]');
document.querySelector('[data-testid="source-file-open"]')?.addEventListener('click',()=>sourceFileInput.click());
let sourceFileRequest=0;
sourceFileInput?.addEventListener('change',async()=>{
  const file=sourceFileInput.files?.[0];if(!file)return;
  const request=++sourceFileRequest;
  const prior={project:state.projectId,generation:state.sourceAnalysisGeneration,text:els.sourceEditor.value};
  try{
    const opened=await readDdnSourceFile(file);
    if(request!==sourceFileRequest || state.projectId!==prior.project
      || state.sourceAnalysisGeneration!==prior.generation || els.sourceEditor.value!==prior.text)
      throw new Error('E_SOURCE_FILE_STALE: 파일을 읽는 동안 원문이 바뀌었습니다. 다시 여세요.');
    // This ID is an in-memory UI locator, never a permanent authored identity.
    const id='opened_v25_source';
    const project={...buildOneFileProject(opened.source),id,title:opened.name,
      kind:'v25 한 파일',summary:'v25로 명시하여 연 원문입니다. 열기만으로 실행하거나 영구 정체를 만들지 않습니다.',
      sourceProfileIdentity:SOURCE_ANALYSIS_PROFILE_IDENTITY,
      files:[{path:opened.name,role:'main',fallback:opened.source}]};
    workbenchProjectRegistryState.receivedPortableProjects=[project,...workbenchProjectRegistryState.receivedPortableProjects.filter(row=>row.id!==id)];
    await selectProject(id);
  }catch(error){
    const diagnostic=projectDiagnosticFromError(error,'E_SOURCE_FILE_OPEN');
    pushDiagnostic(diagnostic.code,diagnostic.message,'error');renderAll();
  }finally{if(request===sourceFileRequest)sourceFileInput.value='';}
});
els.portableImportInput?.addEventListener("change", async () => {
  const file = els.portableImportInput.files?.[0] ?? null;
  if(!file)return;
  try {
    if(file.size>64*1024*1024)throw new Error('저장 자료가 64MiB를 넘습니다.');
    const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(await file.arrayBuffer());
    const schema=JSON.parse(text)?.schema;
    if(schema==='ddn.authored_project_artifact.v1'||schema==='ddn.authored_project_artifact.v2') {
      await workbenchAuthoredPanelAccess.current.importFile(file);
      els.saveStatus.textContent='작품 자료를 열었습니다. 저장·다시 열기는 위쪽 단추를 사용하세요. 실행은 직접 시작하세요.';
      els.portableImportInput.value='';return;
    }
  }catch(error) {
    els.portableImportInput.value='';
    pushDiagnostic('E_PROJECT_IMPORT',sourceStorageErrorMessage(error),'error');renderAll();return;
  }
  openProgressiveDialog(els.portableDialog);
  previewPortableProjectFile(file).catch(() => renderAll());
});
els.portableOpen?.addEventListener("click", () => {
  openPendingPortableProject()
    .then(() => els.portableDialog?.close())
    .catch(() => renderAll());
});
els.portableCancel?.addEventListener("click", () => {
  state.pendingPortableImport = null;
  renderPortableImportPreview();
  els.portableImport?.focus?.();
});
[
  els.portableMetaTitle,
  els.portableMetaSummary,
  els.portableMetaTags,
  els.portableMetaSubject,
  els.portableMetaLevel,
  els.portableMetaRequiredViews,
  els.portableMetaRequiredGaji,
].forEach((node) => {
  node?.addEventListener("input", updatePortableMetadataDraftFromInputs);
});
els.portableMetaContentKind?.addEventListener("change", updatePortableMetadataDraftFromInputs);
els.projectRevisionList?.addEventListener("click", (event) => {
  const button = event.target?.closest?.("[data-project-restore-revision]");
  if (!button) return;
  restoreProjectRevision(button.dataset.projectRestoreRevision).catch((err) => {
    pushDiagnostic(err?.payload?.code ?? "E_PROJECT_RESTORE_FAILED", err?.message ?? "리비전 복원 실패", "error");
    renderAll();
  });
});
document.querySelector("[data-testid='project-conflict-copy-local']")?.addEventListener("click", async () => {
  const text = String(state.serverProject.conflict?.localPackageJson ?? "");
  try {
    await hostBridge.writeClipboardText(text);
  } catch (_) {
    // Clipboard is optional in headless and local file contexts.
  }
  els.saveStatus.textContent = "이 기기에 변경 내용 보존됨";
  pushDiagnostic("W_PROJECT_LOCAL_PACKAGE_PRESERVED", "충돌한 로컬 패키지를 화면에 보존했습니다.");
});
els.projectClone?.addEventListener("click", () => {
  const project = currentProject();
  const action = project.portableArtifact?.content_sha256
    ? cloneCurrentPortableProject()
    : cloneCurrentServerProject();
  action.catch((err) => {
    pushDiagnostic(err?.payload?.code ?? "E_PROJECT_CLONE_FAILED", err?.message ?? "프로젝트 복제 실패", "error");
    renderAll();
  });
});
els.myWorkList?.addEventListener("click", (event) => {
  const visibility = event.target?.closest?.("[data-my-work-visibility]");
  if (visibility) {
    event.preventDefault();
    const store = visibility.dataset.myWorkVisibility;
    if (store === "toggle") state.showHiddenMyWork = !state.showHiddenMyWork;
    else {
      const projectId = visibility.dataset.projectId;
      const rows = store === "portable" ? readPortableProjectCatalog() : readMyWorkIndex();
      const row = rows.find((item) => String(item.project_id || "") === projectId);
      if (!row) return;
      row.hidden_from_list = !row.hidden_from_list;
      if (store === "portable") writePortableProjectCatalog(rows);
      else { writeMyWorkIndex(rows); state.myWorkRows = []; }
    }
    renderMyWork();
    return;
  }
  const link = event.target?.closest?.("[data-testid='my-work-item']");
  if (!link) return;
  event.preventDefault();
  const portableProjectId = String(link.dataset.portableProjectId || "");
  if (portableProjectId) {
    selectProject(portableProjectId).then(() => {
      state.persistenceStatus = {
        schema: "seamgrim.workbench.persistence_status.v1",
        mode: "portable_catalog_reopen",
        server_saved: false,
        local_saved: true,
        user_label: `받은 프로젝트 다시 열림 ${portableProjectId}`,
      };
      pushDiagnostic("W_PORTABLE_PROJECT_CATALOG_OPENED", "내 작업에서 받은 프로젝트를 다시 열었습니다.");
      renderAll();
    }).catch((err) => {
      pushDiagnostic("E_PORTABLE_PROJECT_CATALOG_OPEN_FAILED", String(err?.message ?? err), "error");
      renderAll();
    });
    return;
  }
  const projectId = String(link.dataset.projectId || "");
  const row = readMyWorkIndex().find((item) => String(item.project_id || "") === projectId);
  if(row?.origin_kind==='native_source') {
    workbenchAuthoredPanelAccess.current.reopenNativeLocator(row.native_source_locator,{title:row.title,logicalInput:row.native_input_profile,expectedNativeProjectId:projectId}).then(async result=>{
      await selectProjectInputContract(currentProject());
      state.persistenceStatus={schema:'seamgrim.workbench.persistence_status.v1',mode:'source_only',server_saved:false,local_saved:true,user_label:result.message};
      renderAll();
    }).catch(error=>{pushDiagnostic('E_NATIVE_WORK_REOPEN_FAILED',sourceStorageErrorMessage(error),'error');renderAll();});
    return;
  }
  const localProject = workbenchProjectRegistryState.nativeProjects.find((item) => item.id === projectId);
  if (localProject) {
    selectProject(projectId).then(() => {
      state.persistenceStatus = {
        schema: "seamgrim.workbench.persistence_status.v1",
        mode: "local_recovery",
        server_saved: false,
        local_saved: true,
        user_label: `로컬 프로젝트 다시 열림 ${projectId}`,
      };
      pushDiagnostic("W_NATIVE_PROJECT_REOPENED", "내 작업에서 로컬 프로젝트를 다시 열었습니다.");
      renderAll();
    });
    return;
  }
  const capability = String(row?.edit_capability || row?.view_capability || "");
  if (!row || !capability) {
    pushDiagnostic("E_MY_WORK_CAPABILITY_MISSING", "내 작업 항목의 권한 정보를 찾지 못했습니다.", "error");
    renderAll();
    return;
  }
  state.capabilitySession = writeWorkbenchCapabilitySession({
    projectId,
    role: row.edit_capability ? "edit" : "view",
    capability,
  }) ?? state.capabilitySession;
  openProjectFromServerSession().catch((err) => {
    pushDiagnostic(err?.payload?.code ?? "E_MY_WORK_OPEN_FAILED", err?.message ?? "내 작업 열기 실패", "error");
    renderAll();
  });
});

window.__SEAMGRIM_APPROVED_WORKBENCH__ = createApprovedWorkbenchApi({ activeFrame, analyzeSourceReadiness, applySourceReadinessFixIt, attemptMutateParentHash, attemptOverwriteActiveRun, attemptReuseRunId, branchFromCursor, buildCurrentPortableProjectArtifact, buildProjectSnapshot, buildServerProjectArtifact, cloneCurrentPortableProject, currentPortableLineage, currentProject, els, exportPortableProject, getSourceReadinessCandidate, openPendingPortableProject, openRunArchiveDetail, pauseRun, previewPortableProjectFile, projectStorageKey, readMyWorkIndex, readPortableProjectCatalog, reopenProjectSnapshot, resumeRun, saveProjectSnapshot, sourceReadinessSnapshot, startRun, state, stepForward, stopRun, summarizeRunArchiveBoundary, workbenchProjectRegistryState });
const multilangDialog = document.querySelector('[data-testid="multilang-dialog"]');
function readNotebookEquationContext() {
  return { projectId: currentProject().id, path: activeFile().path,
    generation: state.sourceAnalysisGeneration, sourceText: els.sourceEditor.value };
}
function assertNotebookEquationContext(expected) {
  const current = readNotebookEquationContext();
  if (!expected || ['projectId', 'path', 'generation', 'sourceText'].some(key => current[key] !== expected[key])) {
    throw new Error('작품·파일·원문 판본이 바뀌어 풀이 원문을 적용하지 않았습니다.');
  }
  if (els.sourceEditor.readOnly) throw new Error('현재 원문은 편집할 수 없습니다.');
  return current;
}
function applyNotebookEquationSource(expected, sourceText) {
  assertNotebookEquationContext(expected);
  if (typeof sourceText !== 'string') throw new TypeError('전체 DDN 원문이 필요합니다.');
  if (sourceText !== expected.sourceText) {
    els.sourceEditor.value = sourceText;
    els.sourceEditor.dispatchEvent(new Event('input', { bubbles: true }));
  }
  return readNotebookEquationContext();
}
async function saveNotebookEquationSource(expected) {
  assertNotebookEquationContext(expected);
  await saveProjectSnapshot();
  const current = readNotebookEquationContext();
  if (current.path !== expected.path || current.sourceText !== expected.sourceText
      || state.persistenceOperation?.phase !== 'saved' || state.persistenceOperation?.outcome !== 'source_only') {
    throw new Error('현재 풀이 원문의 Native 저장 완료를 확인하지 못했습니다.');
  }
  const artifactText = workbenchAuthoredPanelAccess.current.getArtifactText();
  const artifact = JSON.parse(artifactText || 'null');
  const locator = localStorage.getItem('ddn.authored_project.last.v1');
  const owner = JSON.parse(locator || 'null');
  const file = artifact?.source_snapshot?.files?.find(item => item.path === current.path);
  if (file?.text !== current.sourceText || !owner?.native_source_project_id
      || artifact?.authored_identity?.logical_dependency_id !== owner.native_source_project_id
      || currentProject().authoredArtifact?.authored_identity?.logical_dependency_id !== owner.native_source_project_id) {
    throw new Error('저장된 원문·작품 정체·저장 위치가 일치하지 않습니다.');
  }
  return { context: current, artifactText, locator };
}
notebookHost = installNotebookHost({
  root: document.querySelector('[data-testid="math-notebook-dialog"]'),
  opener: document.querySelector('[data-testid="math-notebook-open"]'),
  runtimeBridge,
  readEquationContext: readNotebookEquationContext,
  applyEquationSource: applyNotebookEquationSource,
  saveEquationSource: saveNotebookEquationSource,
  readSource: () => {
    const raw = workbenchAuthoredPanelAccess.current?.getArtifactText() || '';
    if (!raw) return '';
    const artifact = JSON.parse(raw), project = currentProject();
    if (project.authoredArtifact?.authored_identity?.logical_dependency_id !== artifact.authored_identity?.logical_dependency_id) return '';
    const files = artifact.source_snapshot?.files;
    if (!Array.isArray(files) || files.length !== project.files.length
        || files.some(file => state.fileTexts.get(file.path) !== file.text)) {
      throw new Error('수정한 작품 원문을 먼저 저장하세요.');
    }
    return raw;
  },
  readLocator: () => localStorage.getItem('ddn.authored_project.last.v1') ?? '',
  rememberLocator: locator => localStorage.setItem('ddn.authored_project.last.v1', locator),
  applyReopenedLocator: locator => workbenchAuthoredPanelAccess.current.reopenNativeLocator(locator),
  readSavedSources: () => readMyWorkIndex().filter(row => row.origin_kind === 'native_source' && row.native_source_locator),
  rememberSavedSource: (slot, locator) => {
    const owner=JSON.parse(locator);
    rememberMyWorkProject({projectId:owner.native_source_project_id,title:`탐구 실험 ${slot}`,source:'native_source',nativeLocator:locator});
  },
});
notebookHost.connectProducer(createUniformMotionProvider(runtimeBridge));
window.addEventListener('pagehide', () => notebookHost.dispose(), { once: true });
const multilangHost = installMultilangHost({
  root:multilangDialog.querySelector('[data-testid="multilang-root"]'),runtimeBridge,
  readContext:() => {
    const project=currentProject();
    return {projectId:project.id,path:activeFile().path,profile:workbenchSourceProfile(project),
      running:Boolean(state.runtimeBusy||state.livePump?.in_flight||state.runtimeStatus==='running'),
      artifactText:workbenchAuthoredPanelAccess.current.getRunProjectArtifact()||workbenchAuthoredPanelAccess.current.getRunArtifact(),
      locator:localStorage.getItem('ddn.authored_project.last.v1')??'',
      sourceSnapshot:{entry_path:localProjectEntryPath(project),
        files:project.files.map(file=>({path:file.path,text:state.fileTexts.get(file.path)??''})),
        ...((project.authoredArtifact?.source_snapshot.root_path??localProjectTreePayloadFromSnapshot(project.portableSnapshot??{})?.root_path)
          ?{root_path:project.authoredArtifact?.source_snapshot.root_path??localProjectTreePayloadFromSnapshot(project.portableSnapshot??{}).root_path}:{})}};
  },
});
document.querySelector('[data-testid="multilang-open"]').addEventListener('click',()=>{
  if(state.runtimeBusy||state.livePump?.in_flight||state.runtimeStatus==='running'){
    pushDiagnostic('E_MULTILANG_PAUSE_REQUIRED','현재 실행을 일시정지한 뒤 다국어 소비 패널을 여세요.','warning');return;
  }
  if(!multilangDialog.open)multilangDialog.show();
  void multilangHost.loadCurrent();
});
document.querySelector('[data-testid="multilang-close"]').addEventListener('click',()=>multilangDialog.close());
window.addEventListener('pagehide',()=>multilangHost.dispose(),{once:true});
workbenchLogicalPresentationState.logicalPanel = createLogicalPresentationPanel({ installLogicalInputPanel, activeSourceText, workbenchAuthoredPanelAccess, nativeSourceAvailable, nativeSourceRequest, renderExecutionControls, els, workbenchLogicalPresentationState, state, renderViewPolicy, currentProject, drawPreview, renderSemanticObservations, createNativeProject });
document.querySelector('[data-testid="logical-record-open"]').addEventListener('click', () => {
  const drawer = document.querySelector('[data-testid="logical-record-drawer"]');
  workbenchLogicalPresentationState.logicalPanel.open = true;
  if (!drawer.open) drawer.show();
});
document.querySelector('[data-testid="logical-record-close"]').addEventListener('click', () => {
  const dialog=document.querySelector('[data-testid="logical-record-drawer"]');
  if(workbenchLogicalPresentationState.logicalPanel?.logicalExecution.status().started)workbenchLogicalPresentationState.logicalPanel.logicalExecution.closeViewDialog(dialog);
  else dialog.close();
});
document.querySelector('[data-testid="project-files-open"]').addEventListener('click', () => openProgressiveDialog(document.querySelector('[data-testid="project-files-dialog"]')));
document.querySelector('[data-testid="project-files-close"]').addEventListener('click', () => document.querySelector('[data-testid="project-files-dialog"]').close());

document.querySelector('[data-testid="first-steps-input-contract"]').addEventListener('click',async()=>{
  const projectId=state.projectId,row=currentProject().course;
  if (!row?.binding_file) return;
  const read=async file=>{
    const response=await fetch(new URL(`${row.asset_root}${file}`,location.href));
    if(!response.ok)throw new Error(`E_FIRST_STEPS_ASSET: 학습 입력 자료를 읽을 수 없습니다 (${response.status}).`);
    return response.text();
  };
  try {
    const [bindingJson,actionsJson]=await Promise.all([read(row.binding_file),read('입력/action_map.json')]);
    if(state.projectId!==projectId)throw new Error('E_FIRST_STEPS_CHANGED: 학습 원문이 바뀌었습니다. 다시 선택하세요.');
    await workbenchLogicalPresentationState.logicalPanel.logicalExecution.selectContract({bindingJson,actionsJson,capacity:128});
  } catch(error){pushDiagnostic('E_FIRST_STEPS_INPUT',String(error),'error');renderAll();}
});
