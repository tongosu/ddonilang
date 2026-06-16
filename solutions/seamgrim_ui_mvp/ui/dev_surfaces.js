function setWindowPayload(dataKey, textKey, formatter, row) {
  try {
    window[dataKey] = row;
    window[textKey] = row && formatter ? formatter(row) : "";
  } catch (_) {
    // ignore browser instrumentation errors
  }
}

function renderSurface({ dataKey, textKey, formatter, render, elementId, row }) {
  const payload = row && typeof row === "object" ? row : null;
  setWindowPayload(dataKey, textKey, formatter, payload);
  if (typeof render === "function") {
    render(document.getElementById(elementId), payload);
  }
}

function ensureDevSurfaceDom() {
  if (document.getElementById("dev-surface-root")) return true;
  const template = document.getElementById("dev-surface-template");
  const catalog = document.querySelector("#screen-browse .catalog-body");
  if (!template || !template.content || !catalog) return false;
  const root = document.createElement("div");
  root.id = "dev-surface-root";
  root.className = "dev-surface-root";
  root.appendChild(template.content.cloneNode(true));
  catalog.after(root);
  return true;
}

async function optionalImport(specifier) {
  try {
    return await import(specifier);
  } catch (err) {
    try {
      console.warn("dev surface skipped", specifier, err);
    } catch (_) {
      // ignore narrow test-console failures
    }
    return null;
  }
}

async function mountGenericSurface(definition) {
  const module = await optionalImport(definition.module);
  if (!module) return false;
  const build = module[definition.build];
  const render = module[definition.render];
  const formatter = module[definition.formatter];
  if (typeof build !== "function" || typeof render !== "function") return false;
  renderSurface({
    dataKey: definition.dataKey,
    textKey: definition.textKey,
    formatter,
    render,
    elementId: definition.elementId,
    row: build(definition.args(module)),
  });
  return true;
}

const GENERIC_SURFACES = [
  {
    module: "./studio_teacher_feedback_surface_preview.js",
    build: "buildTeacherFeedbackSurfacePreview",
    formatter: "formatTeacherFeedbackSurfacePreviewText",
    render: "renderTeacherFeedbackSurfacePreview",
    elementId: "teacher-feedback-preview-panel",
    dataKey: "__SEAMGRIM_TEACHER_FEEDBACK_SURFACE_PREVIEW__",
    textKey: "__SEAMGRIM_TEACHER_FEEDBACK_SURFACE_PREVIEW_TEXT__",
    args: (m) => ({ seedRows: m.DEFAULT_TEACHER_FEEDBACK_SEED_ROWS }),
  },
  {
    module: "./studio_classroom_operations_panel_preview.js",
    build: "buildClassroomOperationsPanelPreview",
    formatter: "formatClassroomOperationsPanelPreviewText",
    render: "renderClassroomOperationsPanelPreview",
    elementId: "classroom-operations-panel-preview",
    dataKey: "__SEAMGRIM_CLASSROOM_OPERATIONS_PANEL_PREVIEW__",
    textKey: "__SEAMGRIM_CLASSROOM_OPERATIONS_PANEL_PREVIEW_TEXT__",
    args: (m) => ({ triageRows: m.DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_ROWS }),
  },
  {
    module: "./studio_benchmark_baseline_local_snapshot.js",
    build: "buildBenchmarkBaselineLocalSnapshot",
    formatter: "formatBenchmarkBaselineLocalSnapshotText",
    render: "renderBenchmarkBaselineLocalSnapshot",
    elementId: "benchmark-baseline-local-snapshot",
    dataKey: "__SEAMGRIM_BENCHMARK_BASELINE_LOCAL_SNAPSHOT__",
    textKey: "__SEAMGRIM_BENCHMARK_BASELINE_LOCAL_SNAPSHOT_TEXT__",
    args: (m) => ({
      plannedInputs: m.DEFAULT_BENCHMARK_BASELINE_INPUTS,
      panelRows: m.DEFAULT_BENCHMARK_CLASSROOM_PANEL_ROWS,
    }),
  },
  {
    module: "./studio_release_review_packet_dashboard.js",
    build: "buildReleaseReviewPacketDashboard",
    formatter: "formatReleaseReviewPacketDashboardText",
    render: "renderReleaseReviewPacketDashboard",
    elementId: "release-review-packet-dashboard",
    dataKey: "__SEAMGRIM_RELEASE_REVIEW_PACKET_DASHBOARD__",
    textKey: "__SEAMGRIM_RELEASE_REVIEW_PACKET_DASHBOARD_TEXT__",
    args: (m) => ({
      snapshotRows: m.DEFAULT_RELEASE_REVIEW_SNAPSHOT_ROWS,
      reviewMaterials: m.DEFAULT_RELEASE_REVIEW_MATERIALS,
    }),
  },
  {
    module: "./studio_lesson_publication_review_surface.js",
    build: "buildLessonPublicationReviewSurface",
    formatter: "formatLessonPublicationReviewSurfaceText",
    render: "renderLessonPublicationReviewSurface",
    elementId: "lesson-publication-review-surface",
    dataKey: "__SEAMGRIM_LESSON_PUBLICATION_REVIEW_SURFACE__",
    textKey: "__SEAMGRIM_LESSON_PUBLICATION_REVIEW_SURFACE_TEXT__",
    args: (m) => ({
      reviewGates: m.DEFAULT_LESSON_PUBLICATION_REVIEW_GATES,
      dashboardRows: m.DEFAULT_LESSON_PUBLICATION_DASHBOARD_ROWS,
      candidateIds: m.DEFAULT_LESSON_PUBLICATION_CANDIDATE_IDS,
    }),
  },
  {
    module: "./studio_ma3_regression_gate_matrix.js",
    build: "buildMa3RegressionGateMatrix",
    formatter: "formatMa3RegressionGateMatrixText",
    render: "renderMa3RegressionGateMatrix",
    elementId: "ma3-regression-gate-matrix",
    dataKey: "__SEAMGRIM_MA3_REGRESSION_GATE_MATRIX__",
    textKey: "__SEAMGRIM_MA3_REGRESSION_GATE_MATRIX_TEXT__",
    args: (m) => ({ evidenceRows: m.DEFAULT_MA3_REGRESSION_GATE_EVIDENCE }),
  },
  {
    module: "./studio_ma3_next_queue_coordinate_lock.js",
    build: "buildMa3NextQueueCoordinateLock",
    formatter: "formatMa3NextQueueCoordinateLockText",
    render: "renderMa3NextQueueCoordinateLock",
    elementId: "ma3-next-queue-coordinate-lock",
    dataKey: "__SEAMGRIM_MA3_NEXT_QUEUE_COORDINATE_LOCK__",
    textKey: "__SEAMGRIM_MA3_NEXT_QUEUE_COORDINATE_LOCK_TEXT__",
    args: (m) => ({ lockRows: m.DEFAULT_MA3_NEXT_QUEUE_LOCK_ROWS }),
  },
  {
    module: "./studio_operations_preview_stage_closure.js",
    build: "buildOperationsPreviewStageClosure",
    formatter: "formatOperationsPreviewStageClosureText",
    render: "renderOperationsPreviewStageClosure",
    elementId: "operations-preview-stage-closure",
    dataKey: "__SEAMGRIM_OPERATIONS_PREVIEW_STAGE_CLOSURE__",
    textKey: "__SEAMGRIM_OPERATIONS_PREVIEW_STAGE_CLOSURE_TEXT__",
    args: (m) => ({ closureRows: m.DEFAULT_OPERATIONS_PREVIEW_STAGE_CLOSURE_ROWS }),
  },
  {
    module: "./studio_productization_stage_rebase.js",
    build: "buildProductizationStageRebase",
    formatter: "formatProductizationStageRebaseText",
    render: "renderProductizationStageRebase",
    elementId: "productization-stage-rebase",
    dataKey: "__SEAMGRIM_PRODUCTIZATION_STAGE_REBASE__",
    textKey: "__SEAMGRIM_PRODUCTIZATION_STAGE_REBASE_TEXT__",
    args: (m) => ({ rebaseRows: m.DEFAULT_PRODUCTIZATION_STAGE_REBASE_ROWS }),
  },
  {
    module: "./seamgrim_numeric_track_consolidation.js",
    build: "buildSeamgrimNumericTrackConsolidation",
    formatter: "formatSeamgrimNumericTrackConsolidationText",
    render: "renderSeamgrimNumericTrackConsolidation",
    elementId: "seamgrim-numeric-track-consolidation",
    dataKey: "__SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION__",
    textKey: "__SEAMGRIM_NUMERIC_TRACK_CONSOLIDATION_TEXT__",
    args: (m) => ({ consolidationRows: m.DEFAULT_NUMERIC_TRACK_CONSOLIDATION_ROWS }),
  },
  {
    module: "./studio_numeric_report_workflow_stage.js",
    build: "buildNumericReportWorkflowStage",
    formatter: "formatNumericReportWorkflowStageText",
    render: "renderNumericReportWorkflowStage",
    elementId: "numeric-report-workflow-stage",
    dataKey: "__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_STAGE__",
    textKey: "__SEAMGRIM_NUMERIC_REPORT_WORKFLOW_STAGE_TEXT__",
    args: (m) => ({ workflowRows: m.DEFAULT_NUMERIC_REPORT_WORKFLOW_STAGE_ROWS }),
  },
  {
    module: "./studio_numeric_result_report_stage.js",
    build: "buildNumericResultReportStage",
    formatter: "formatNumericResultReportStageText",
    render: "renderNumericResultReportStage",
    elementId: "numeric-result-report-stage",
    dataKey: "__SEAMGRIM_NUMERIC_RESULT_REPORT_STAGE__",
    textKey: "__SEAMGRIM_NUMERIC_RESULT_REPORT_STAGE_TEXT__",
    args: (m) => ({ resultRows: m.DEFAULT_NUMERIC_RESULT_REPORT_STAGE_ROWS }),
  },
  {
    module: "./studio_productization_stage_closure.js",
    build: "buildProductizationStageClosure",
    formatter: "formatProductizationStageClosureText",
    render: "renderProductizationStageClosure",
    elementId: "productization-stage-closure",
    dataKey: "__SEAMGRIM_PRODUCTIZATION_STAGE_CLOSURE__",
    textKey: "__SEAMGRIM_PRODUCTIZATION_STAGE_CLOSURE_TEXT__",
    args: (m) => ({ closureRows: m.DEFAULT_PRODUCTIZATION_STAGE_CLOSURE_ROWS }),
  },
  {
    module: "./studio_post_super_long_rebase.js",
    build: "buildPostSuperLongRebase",
    formatter: "formatPostSuperLongRebaseText",
    render: "renderPostSuperLongRebase",
    elementId: "post-super-long-rebase",
    dataKey: "__SEAMGRIM_POST_SUPER_LONG_REBASE__",
    textKey: "__SEAMGRIM_POST_SUPER_LONG_REBASE_TEXT__",
    args: (m) => ({ followupRows: m.DEFAULT_POST_SUPER_LONG_REBASE_ROWS }),
  },
  {
    module: "./studio_public_release_approval_recheck.js",
    build: "buildPublicReleaseApprovalRecheck",
    formatter: "formatPublicReleaseApprovalRecheckText",
    render: "renderPublicReleaseApprovalRecheck",
    elementId: "public-release-approval-recheck",
    dataKey: "__SEAMGRIM_PUBLIC_RELEASE_APPROVAL_RECHECK__",
    textKey: "__SEAMGRIM_PUBLIC_RELEASE_APPROVAL_RECHECK_TEXT__",
    args: (m) => ({ approvalRows: m.DEFAULT_PUBLIC_RELEASE_APPROVAL_RECHECK_ROWS }),
  },
  {
    module: "./studio_local_release_rehearsal_check.js",
    build: "buildLocalReleaseRehearsalCheck",
    formatter: "formatLocalReleaseRehearsalCheckText",
    render: "renderLocalReleaseRehearsalCheck",
    elementId: "local-release-rehearsal-check",
    dataKey: "__SEAMGRIM_LOCAL_RELEASE_REHEARSAL_CHECK__",
    textKey: "__SEAMGRIM_LOCAL_RELEASE_REHEARSAL_CHECK_TEXT__",
    args: (m) => ({ rehearsalRows: m.DEFAULT_LOCAL_RELEASE_REHEARSAL_ROWS }),
  },
  {
    module: "./studio_publication_artifact_dry_run.js",
    build: "buildPublicationArtifactDryRun",
    formatter: "formatPublicationArtifactDryRunText",
    render: "renderPublicationArtifactDryRun",
    elementId: "publication-artifact-dry-run",
    dataKey: "__SEAMGRIM_PUBLICATION_ARTIFACT_DRY_RUN__",
    textKey: "__SEAMGRIM_PUBLICATION_ARTIFACT_DRY_RUN_TEXT__",
    args: (m) => ({ artifactRows: m.DEFAULT_PUBLICATION_ARTIFACT_DRY_RUN_ROWS }),
  },
  {
    module: "./studio_teacher_feedback_loop_seed.js",
    build: "buildTeacherFeedbackLoopSeed",
    formatter: "formatTeacherFeedbackLoopSeedText",
    render: "renderTeacherFeedbackLoopSeed",
    elementId: "teacher-feedback-loop-seed",
    dataKey: "__SEAMGRIM_TEACHER_FEEDBACK_LOOP_SEED__",
    textKey: "__SEAMGRIM_TEACHER_FEEDBACK_LOOP_SEED_TEXT__",
    args: (m) => ({ seedRows: m.DEFAULT_TEACHER_FEEDBACK_LOOP_SEED_ROWS }),
  },
  {
    module: "./studio_classroom_operations_triage.js",
    build: "buildClassroomOperationsTriage",
    formatter: "formatClassroomOperationsTriageText",
    render: "renderClassroomOperationsTriage",
    elementId: "classroom-operations-triage",
    dataKey: "__SEAMGRIM_CLASSROOM_OPERATIONS_TRIAGE__",
    textKey: "__SEAMGRIM_CLASSROOM_OPERATIONS_TRIAGE_TEXT__",
    args: (m) => ({ triageRows: m.DEFAULT_CLASSROOM_OPERATIONS_TRIAGE_UI_ROWS }),
  },
  {
    module: "./studio_benchmark_baseline_prep_dry_run.js",
    build: "buildBenchmarkBaselinePrepDryRun",
    formatter: "formatBenchmarkBaselinePrepDryRunText",
    render: "renderBenchmarkBaselinePrepDryRun",
    elementId: "benchmark-baseline-prep-dry-run",
    dataKey: "__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN__",
    textKey: "__SEAMGRIM_BENCHMARK_BASELINE_PREP_DRY_RUN_TEXT__",
    args: (m) => ({ inputRows: m.DEFAULT_BENCHMARK_BASELINE_PREP_INPUT_ROWS }),
  },
  {
    module: "./studio_next_roadmap_v2_coordinate_lock.js",
    build: "buildNextRoadmapV2CoordinateLock",
    formatter: "formatNextRoadmapV2CoordinateLockText",
    render: "renderNextRoadmapV2CoordinateLock",
    elementId: "next-roadmap-v2-coordinate-lock",
    dataKey: "__SEAMGRIM_NEXT_ROADMAP_V2_COORDINATE_LOCK__",
    textKey: "__SEAMGRIM_NEXT_ROADMAP_V2_COORDINATE_LOCK_TEXT__",
    args: (m) => ({ decisions: m.DEFAULT_NEXT_ROADMAP_V2_COORDINATE_LOCK_DECISIONS }),
  },
  {
    module: "./studio_ma3_next_development_queue_rebase.js",
    build: "buildMa3NextDevelopmentQueueRebase",
    formatter: "formatMa3NextDevelopmentQueueRebaseText",
    render: "renderMa3NextDevelopmentQueueRebase",
    elementId: "ma3-next-development-queue-rebase",
    dataKey: "__SEAMGRIM_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE__",
    textKey: "__SEAMGRIM_MA3_NEXT_DEVELOPMENT_QUEUE_REBASE_TEXT__",
    args: (m) => ({ queueRows: m.DEFAULT_MA3_NEXT_DEVELOPMENT_QUEUE_ROWS }),
  },
  {
    module: "./free_lab_experiment_report.js",
    build: "buildFreeLabExperimentReport",
    formatter: "formatFreeLabExperimentReportText",
    render: "renderFreeLabExperimentReport",
    elementId: "free-lab-experiment-report",
    dataKey: "__SEAMGRIM_FREE_LAB_EXPERIMENT_REPORT__",
    textKey: "__SEAMGRIM_FREE_LAB_EXPERIMENT_REPORT_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_FREE_LAB_EXPERIMENT_REPORT_ROWS }),
  },
  {
    module: "./free_lab_ui_pack.js",
    build: "buildFreeLabUiPack",
    formatter: "formatFreeLabUiPackText",
    render: "renderFreeLabUiPack",
    elementId: "free-lab-ui-pack",
    dataKey: "__SEAMGRIM_FREE_LAB_UI_PACK__",
    textKey: "__SEAMGRIM_FREE_LAB_UI_PACK_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_FREE_LAB_UI_ROWS }),
  },
  {
    module: "./free_lab_share_pack.js",
    build: "buildFreeLabSharePack",
    formatter: "formatFreeLabSharePackText",
    render: "renderFreeLabSharePack",
    elementId: "free-lab-share-pack",
    dataKey: "__SEAMGRIM_FREE_LAB_SHARE_PACK__",
    textKey: "__SEAMGRIM_FREE_LAB_SHARE_PACK_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_FREE_LAB_SHARE_ROWS }),
  },
  {
    module: "./free_lab_research_workflow.js",
    build: "buildFreeLabResearchWorkflow",
    formatter: "formatFreeLabResearchWorkflowText",
    render: "renderFreeLabResearchWorkflow",
    elementId: "free-lab-research-workflow",
    dataKey: "__SEAMGRIM_FREE_LAB_RESEARCH_WORKFLOW__",
    textKey: "__SEAMGRIM_FREE_LAB_RESEARCH_WORKFLOW_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_FREE_LAB_RESEARCH_WORKFLOW_ROWS }),
  },
  {
    module: "./rpg_story_package.js",
    build: "buildRpgStoryPackage",
    formatter: "formatRpgStoryPackageText",
    render: "renderRpgStoryPackage",
    elementId: "rpg-story-package",
    dataKey: "__SEAMGRIM_RPG_STORY_PACKAGE__",
    textKey: "__SEAMGRIM_RPG_STORY_PACKAGE_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_RPG_STORY_PACKAGE_ROWS }),
  },
  {
    module: "./rpg_engine_adapter_lts.js",
    build: "buildRpgEngineAdapterLts",
    formatter: "formatRpgEngineAdapterLtsText",
    render: "renderRpgEngineAdapterLts",
    elementId: "rpg-engine-adapter-lts",
    dataKey: "__SEAMGRIM_RPG_ENGINE_ADAPTER_LTS__",
    textKey: "__SEAMGRIM_RPG_ENGINE_ADAPTER_LTS_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_RPG_ENGINE_ADAPTER_LTS_ROWS }),
  },
  {
    module: "./ttonimaru_publication_read_api.js",
    build: "buildTtonimaruPublicationReadApi",
    formatter: "formatTtonimaruPublicationReadApiText",
    render: "renderTtonimaruPublicationReadApi",
    elementId: "ttonimaru-publication-read-api",
    dataKey: "__TTONIMARU_PUBLICATION_READ_API__",
    textKey: "__TTONIMARU_PUBLICATION_READ_API_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TTONIMARU_PUBLICATION_READ_API_ROWS }),
  },
  {
    module: "./ttonimaru_project_share_ui.js",
    build: "buildTtonimaruProjectShareUi",
    formatter: "formatTtonimaruProjectShareUiText",
    render: "renderTtonimaruProjectShareUi",
    elementId: "ttonimaru-project-share-ui",
    dataKey: "__TTONIMARU_PROJECT_SHARE_UI__",
    textKey: "__TTONIMARU_PROJECT_SHARE_UI_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TTONIMARU_PROJECT_SHARE_ROWS }),
  },
  {
    module: "./ttonimaru_public_registry_seed.js",
    build: "buildTtonimaruPublicRegistrySeed",
    formatter: "formatTtonimaruPublicRegistrySeedText",
    render: "renderTtonimaruPublicRegistrySeed",
    elementId: "ttonimaru-public-registry-seed",
    dataKey: "__TTONIMARU_PUBLIC_REGISTRY_SEED__",
    textKey: "__TTONIMARU_PUBLIC_REGISTRY_SEED_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TTONIMARU_PUBLIC_REGISTRY_SEED_ROWS }),
  },
  {
    module: "./ttonimaru_platform_hardening.js",
    build: "buildTtonimaruPlatformHardening",
    formatter: "formatTtonimaruPlatformHardeningText",
    render: "renderTtonimaruPlatformHardening",
    elementId: "ttonimaru-platform-hardening",
    dataKey: "__TTONIMARU_PLATFORM_HARDENING__",
    textKey: "__TTONIMARU_PLATFORM_HARDENING_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TTONIMARU_PLATFORM_HARDENING_ROWS }),
  },
  {
    module: "./toolchain_diagnostic_ui_lsp.js",
    build: "buildToolchainDiagnosticUiLsp",
    formatter: "formatToolchainDiagnosticUiLspText",
    render: "renderToolchainDiagnosticUiLsp",
    elementId: "toolchain-diagnostic-ui-lsp",
    dataKey: "__TOOLCHAIN_DIAGNOSTIC_UI_LSP__",
    textKey: "__TOOLCHAIN_DIAGNOSTIC_UI_LSP_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TOOLCHAIN_DIAGNOSTIC_UI_LSP_ROWS }),
  },
  {
    module: "./toolchain_registry_verification.js",
    build: "buildToolchainRegistryVerification",
    formatter: "formatToolchainRegistryVerificationText",
    render: "renderToolchainRegistryVerification",
    elementId: "toolchain-registry-verification",
    dataKey: "__TOOLCHAIN_REGISTRY_VERIFICATION__",
    textKey: "__TOOLCHAIN_REGISTRY_VERIFICATION_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TOOLCHAIN_REGISTRY_VERIFICATION_ROWS }),
  },
  {
    module: "./toolchain_benchmark_lts.js",
    build: "buildToolchainBenchmarkLts",
    formatter: "formatToolchainBenchmarkLtsText",
    render: "renderToolchainBenchmarkLts",
    elementId: "toolchain-benchmark-lts",
    dataKey: "__TOOLCHAIN_BENCHMARK_LTS__",
    textKey: "__TOOLCHAIN_BENCHMARK_LTS_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_TOOLCHAIN_BENCHMARK_LTS_ROWS }),
  },
  {
    module: "./social_world_bridge_pack.js",
    build: "buildSocialWorldBridgePack",
    formatter: "formatSocialWorldBridgePackText",
    render: "renderSocialWorldBridgePack",
    elementId: "social-world-bridge-pack",
    dataKey: "__SOCIAL_WORLD_BRIDGE_PACK__",
    textKey: "__SOCIAL_WORLD_BRIDGE_PACK_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_SOCIAL_WORLD_BRIDGE_ROWS }),
  },
  {
    module: "./social_world_policy_ghost_ui.js",
    build: "buildSocialWorldPolicyGhostUi",
    formatter: "formatSocialWorldPolicyGhostUiText",
    render: "renderSocialWorldPolicyGhostUi",
    elementId: "social-world-policy-ghost-ui",
    dataKey: "__SOCIAL_WORLD_POLICY_GHOST_UI__",
    textKey: "__SOCIAL_WORLD_POLICY_GHOST_UI_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_SOCIAL_WORLD_POLICY_GHOST_ROWS }),
  },
  {
    module: "./social_world_template_registry.js",
    build: "buildSocialWorldTemplateRegistry",
    formatter: "formatSocialWorldTemplateRegistryText",
    render: "renderSocialWorldTemplateRegistry",
    elementId: "social-world-template-registry",
    dataKey: "__SOCIAL_WORLD_TEMPLATE_REGISTRY__",
    textKey: "__SOCIAL_WORLD_TEMPLATE_REGISTRY_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_SOCIAL_WORLD_TEMPLATE_REGISTRY_ROWS }),
  },
  {
    module: "./social_world_lts_readiness.js",
    build: "buildSocialWorldLtsReadiness",
    formatter: "formatSocialWorldLtsReadinessText",
    render: "renderSocialWorldLtsReadiness",
    elementId: "social-world-lts-readiness",
    dataKey: "__SOCIAL_WORLD_LTS_READINESS__",
    textKey: "__SOCIAL_WORLD_LTS_READINESS_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_SOCIAL_WORLD_LTS_ROWS }),
  },
  {
    module: "./education_assessment_pack.js",
    build: "buildEducationAssessmentPack",
    formatter: "formatEducationAssessmentPackText",
    render: "renderEducationAssessmentPack",
    elementId: "education-assessment-pack",
    dataKey: "__EDUCATION_ASSESSMENT_PACK__",
    textKey: "__EDUCATION_ASSESSMENT_PACK_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_EDUCATION_ASSESSMENT_ROWS }),
  },
  {
    module: "./education_classroom_ui_pack.js",
    build: "buildEducationClassroomUiPack",
    formatter: "formatEducationClassroomUiPackText",
    render: "renderEducationClassroomUiPack",
    elementId: "education-classroom-ui-pack",
    dataKey: "__EDUCATION_CLASSROOM_UI_PACK__",
    textKey: "__EDUCATION_CLASSROOM_UI_PACK_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_EDUCATION_CLASSROOM_UI_ROWS }),
  },
  {
    module: "./education_publication_pack.js",
    build: "buildEducationPublicationPack",
    formatter: "formatEducationPublicationPackText",
    render: "renderEducationPublicationPack",
    elementId: "education-publication-pack",
    dataKey: "__EDUCATION_PUBLICATION_PACK__",
    textKey: "__EDUCATION_PUBLICATION_PACK_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_EDUCATION_PUBLICATION_ROWS }),
  },
  {
    module: "./education_operations_lts.js",
    build: "buildEducationOperationsLts",
    formatter: "formatEducationOperationsLtsText",
    render: "renderEducationOperationsLts",
    elementId: "education-operations-lts",
    dataKey: "__EDUCATION_OPERATIONS_LTS__",
    textKey: "__EDUCATION_OPERATIONS_LTS_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_EDUCATION_OPERATIONS_LTS_ROWS }),
  },
  {
    module: "./question_card_smoke.js",
    build: "buildQuestionCardSmoke",
    formatter: "formatQuestionCardSmokeText",
    render: "renderQuestionCardSmoke",
    elementId: "question-card-smoke",
    dataKey: "__QUESTION_CARD_SMOKE__",
    textKey: "__QUESTION_CARD_SMOKE_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_QUESTION_CARD_SMOKE_ROWS }),
  },
  {
    module: "./question_card_validation.js",
    build: "buildQuestionCardValidation",
    formatter: "formatQuestionCardValidationText",
    render: "renderQuestionCardValidation",
    elementId: "question-card-validation",
    dataKey: "__QUESTION_CARD_VALIDATION__",
    textKey: "__QUESTION_CARD_VALIDATION_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_QUESTION_CARD_VALIDATION_ROWS }),
  },
  {
    module: "./question_card_dev_assist.js",
    build: "buildQuestionCardDevAssist",
    formatter: "formatQuestionCardDevAssistText",
    render: "renderQuestionCardDevAssist",
    elementId: "question-card-dev-assist",
    dataKey: "__QUESTION_CARD_DEV_ASSIST__",
    textKey: "__QUESTION_CARD_DEV_ASSIST_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_QUESTION_CARD_DEV_ASSIST_ROWS }),
  },
  {
    module: "./question_card_author_tool_share.js",
    build: "buildQuestionCardAuthorToolShare",
    formatter: "formatQuestionCardAuthorToolShareText",
    render: "renderQuestionCardAuthorToolShare",
    elementId: "question-card-author-tool-share",
    dataKey: "__QUESTION_CARD_AUTHOR_TOOL_SHARE__",
    textKey: "__QUESTION_CARD_AUTHOR_TOOL_SHARE_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_QUESTION_CARD_AUTHOR_TOOL_SHARE_ROWS }),
  },
  {
    module: "./question_card_workflow_hardening.js",
    build: "buildQuestionCardWorkflowHardening",
    formatter: "formatQuestionCardWorkflowHardeningText",
    render: "renderQuestionCardWorkflowHardening",
    elementId: "question-card-workflow-hardening",
    dataKey: "__QUESTION_CARD_WORKFLOW_HARDENING__",
    textKey: "__QUESTION_CARD_WORKFLOW_HARDENING_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_QUESTION_CARD_WORKFLOW_HARDENING_ROWS }),
  },
  {
    module: "./seulgi_proposal_ui.js",
    build: "buildSeulgiProposalUi",
    formatter: "formatSeulgiProposalUiText",
    render: "renderSeulgiProposalUi",
    elementId: "seulgi-proposal-ui",
    dataKey: "__SEULGI_PROPOSAL_UI__",
    textKey: "__SEULGI_PROPOSAL_UI_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_SEULGI_PROPOSAL_UI_ROWS }),
  },
  {
    module: "./seulgi_replay_safe_workflow.js",
    build: "buildSeulgiReplaySafeWorkflow",
    formatter: "formatSeulgiReplaySafeWorkflowText",
    render: "renderSeulgiReplaySafeWorkflow",
    elementId: "seulgi-replay-safe-workflow",
    dataKey: "__SEULGI_REPLAY_SAFE_WORKFLOW__",
    textKey: "__SEULGI_REPLAY_SAFE_WORKFLOW_TEXT__",
    args: (m) => ({ rows: m.DEFAULT_SEULGI_REPLAY_SAFE_ROWS }),
  },
];

async function mountFreeLabFirstRun(openStudioWithDdnText) {
  const module = await optionalImport("./free_lab_first_run.js");
  if (!module) return false;
  const row = module.buildFreeLabFirstRun({ rows: module.DEFAULT_FREE_LAB_FIRST_RUN_ROWS });
  setWindowPayload("__SEAMGRIM_FREE_LAB_FIRST_RUN__", "__SEAMGRIM_FREE_LAB_FIRST_RUN_TEXT__", module.formatFreeLabFirstRunText, row);
  module.renderFreeLabFirstRun(document.getElementById("free-lab-first-run"), row, {
    onOpenFirstRun: (firstRun) => {
      try {
        window.__SEAMGRIM_FREE_LAB_LAST_OPEN__ = {
          work_item: firstRun?.work_item ?? "",
          source_id: firstRun?.source_id ?? "",
          source_label: firstRun?.source_label ?? "",
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      if (typeof openStudioWithDdnText === "function") {
        void openStudioWithDdnText(firstRun?.ddn_template ?? "", {
          launchKind: "free_lab_first_run",
          autoExecute: false,
          sourceId: firstRun?.source_id ?? "ddn:free_lab:first_run",
          title: firstRun?.source_label ?? "자유 실험 첫실행",
          description: "자유 실험실 첫실행 초안",
        });
      }
    },
  });
  return true;
}

async function mountRpgAuthoring(openStudioWithDdnText) {
  const module = await optionalImport("./rpg_box_authoring_ui.js");
  if (!module) return false;
  const row = module.buildRpgBoxAuthoringUi({ rows: module.DEFAULT_RPG_BOX_AUTHORING_ROWS });
  setWindowPayload("__SEAMGRIM_RPG_BOX_AUTHORING_UI__", "__SEAMGRIM_RPG_BOX_AUTHORING_UI_TEXT__", module.formatRpgBoxAuthoringUiText, row);
  module.renderRpgBoxAuthoringUi(document.getElementById("rpg-box-authoring-ui"), row, {
    onOpenPlaytest: (authoring) => {
      try {
        window.__SEAMGRIM_RPG_BOX_LAST_OPEN__ = {
          work_item: authoring?.work_item ?? "",
          source_id: "ddn:rpg_box:authoring_ui",
          source_label: "RPG Box authoring UI",
        };
      } catch (_) {
        // ignore browser instrumentation errors
      }
      if (typeof openStudioWithDdnText === "function") {
        void openStudioWithDdnText(authoring?.playtest_ddn ?? "", {
          launchKind: "rpg_box_authoring_ui",
          autoExecute: false,
          sourceId: "ddn:rpg_box:authoring_ui",
          title: "RPG Box authoring UI",
          description: "RPG Box / 누리메이커 playtest 초안",
        });
      }
    },
  });
  return true;
}

export async function mountDevSurfaces({ openStudioWithDdnText = null } = {}) {
  if (!ensureDevSurfaceDom()) return false;
  await Promise.all([
    ...GENERIC_SURFACES.map((definition) => mountGenericSurface(definition)),
    mountFreeLabFirstRun(openStudioWithDdnText),
    mountRpgAuthoring(openStudioWithDdnText),
  ]);
  return true;
}
