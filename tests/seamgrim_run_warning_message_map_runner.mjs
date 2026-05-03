import path from "node:path";
import { fileURLToPath } from "node:url";
import { pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, "..");

const runModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
const runModuleUrl = pathToFileURL(runModulePath).href;
const { mapParseWarningToUserMessage, buildWarningShortcutHint } = await import(runModuleUrl);
const warningContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_warning_contract.js");
const warningContractModuleUrl = pathToFileURL(warningContractModulePath).href;
const {
  buildWarningShortcutHint: buildWarningShortcutHintContract,
  buildWarningPrimaryUserMessage,
  mapParseWarningToUserMessage: mapParseWarningToUserMessageContract,
  resolveUserWarningCause,
  resolveRuntimeGuideText,
} = await import(warningContractModuleUrl);
const warningPanelContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_warning_panel_contract.js");
const warningPanelContractModuleUrl = pathToFileURL(warningPanelContractModulePath).href;
const { buildWarningPanelViewModel } = await import(warningPanelContractModuleUrl);
const studioEditRunContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/studio_edit_run_contract.js");
const studioEditRunContractModuleUrl = pathToFileURL(studioEditRunContractModulePath).href;
const {
  buildAutofixResultContract,
  buildStudioEditorReadinessModel,
  STUDIO_READINESS_STAGE,
} = await import(studioEditRunContractModuleUrl);
const runtimeHintContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_runtime_hint_contract.js");
const runtimeHintContractModuleUrl = pathToFileURL(runtimeHintContractModulePath).href;
const { buildRuntimeHintViewModel } = await import(runtimeHintContractModuleUrl);
const execStatusContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_exec_status_contract.js");
const execStatusContractModuleUrl = pathToFileURL(execStatusContractModulePath).href;
const { buildRunExecStatusViewModel } = await import(execStatusContractModuleUrl);
const actionRailContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_action_rail_contract.js");
const actionRailContractModuleUrl = pathToFileURL(actionRailContractModulePath).href;
const { buildRunActionRailViewModel } = await import(actionRailContractModuleUrl);
const platformServerAdapterContractPath = path.resolve(
  root,
  "solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js",
);
const platformServerAdapterContractUrl = pathToFileURL(platformServerAdapterContractPath).href;
const { PlatformServerAdapterErrorCode, PlatformServerAdapterUiAction } = await import(platformServerAdapterContractUrl);
const observeActionContractModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/run_observe_action_contract.js");
const observeActionContractModuleUrl = pathToFileURL(observeActionContractModulePath).href;
const {
  OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT,
  buildObserveActionPlan,
  normalizeObserveAction,
} = await import(observeActionContractModuleUrl);

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const mappingCases = [
  { code: "E_WASM_DIRECT_ONLY_FALLBACK_BLOCKED", expected: "WASM 직접 실행 전용 모드로 서버 보조 실행이 차단되었습니다. 진단 모드로 전환하거나 입력을 점검하세요." },
  { code: "E_BLOCK_HEADER_COLON_FORBIDDEN", expected: "블록 헤더에는 ':'를 쓰지 않습니다. 예: 채비 {}" },
  { code: "E_BLOCK_HEADER_HASH_FORBIDDEN", expected: "active 실행선에서는 # 헤더를 사용할 수 없습니다. 설정 {}와 매김 {}를 사용하세요." },
  { code: "E_IMPORT_ALIAS_DUPLICATE", expected: "import 별칭이 중복되었습니다. 별칭 이름을 고유하게 바꿔 주세요." },
  { code: "E_IMPORT_ALIAS_RESERVED", expected: "예약된 이름은 import 별칭으로 사용할 수 없습니다." },
  { code: "E_IMPORT_PATH_INVALID", expected: "import 경로 형식이 올바르지 않습니다. 경로 표기를 확인해 주세요." },
  { code: "E_IMPORT_VERSION_CONFLICT", expected: "동일 패키지의 import 버전이 충돌합니다. 버전을 하나로 맞춰 주세요." },
  { code: "E_EXPORT_BLOCK_DUPLICATE", expected: "export 블록이 중복되었습니다. 하나만 유지해 주세요." },
  { code: "E_RECEIVE_OUTSIDE_IMJA", expected: "받기(수신) 구문은 임자 블록 안에서만 사용할 수 있습니다." },
  { code: "E_PARSE_CALL_PIN_DUPLICATE", expected: "동일한 핀(pin)이 중복되었습니다. 핀 이름을 고유하게 맞춰 주세요." },
];

for (const row of mappingCases) {
  const actual = mapParseWarningToUserMessage(row.code, "");
  assert(actual === row.expected, `mapping mismatch: code=${row.code} actual=${actual}`);
  assert(
    actual === mapParseWarningToUserMessageContract(row.code, ""),
    `mapping contract mismatch: code=${row.code}`,
  );
}

const ddnShortcut = buildWarningShortcutHint([{ code: "E_BLOCK_HEADER_COLON_FORBIDDEN", message: "" }]);
assert(ddnShortcut.includes("권장: Alt+D"), "shortcut hint should prefer DDN for syntax/header errors");
assert(
  ddnShortcut === buildWarningShortcutHintContract([{ code: "E_BLOCK_HEADER_COLON_FORBIDDEN", message: "" }]),
  "shortcut hint should match warning contract module",
);

const inspectorShortcut = buildWarningShortcutHint([{ code: "E_IMPORT_ALIAS_DUPLICATE", message: "" }]);
assert(inspectorShortcut.includes("권장: Alt+I"), "shortcut hint should prefer inspector for import/semantic errors");

const unknownCodeMessage = mapParseWarningToUserMessage("E_UNKNOWN_TEST_CODE", "");
assert(unknownCodeMessage === "문법/실행 입력을 점검해 주세요.", "unknown code should map to default guide message");
assert(
  unknownCodeMessage === mapParseWarningToUserMessageContract("E_UNKNOWN_TEST_CODE", ""),
  "unknown code should match warning contract mapping",
);

const primaryWarningMessage = buildWarningPrimaryUserMessage([
  { code: "E_PARSE_EXPECTED_EXPR", message: "식이 필요한 위치입니다." },
]);
assert(primaryWarningMessage === "식이 필요한 위치입니다.", "warning primary message should read user text");
const mappedPrimaryWarningMessage = buildWarningPrimaryUserMessage([
  { code: "E_PARSE_EXPECTED_RPAREN", technical_message: "missing )" },
]);
assert(mappedPrimaryWarningMessage === "닫는 괄호 ')'가 필요합니다.", "warning primary message should map technical warning");
assert(
  resolveUserWarningCause({ code: "E_PARSE_EXPECTED_RPAREN", technical_message: "missing )" }) === "닫는 괄호 ')'가 필요합니다.",
  "warning cause should use shared mapping",
);
const runtimeGuideWithWarning = resolveRuntimeGuideText({
  warnings: [{ code: "E_PARSE_EXPECTED_EXPR", message: "식이 필요한 위치입니다." }],
  observeGuideText: "권장: 관찰 탭 확인",
});
assert(
  runtimeGuideWithWarning.startsWith("문제: 식이 필요한 위치입니다."),
  "runtime guide should prioritize warning primary message",
);
const runtimeHintModel = buildRuntimeHintViewModel({
  tickCount: 3,
  controlCount: 2,
  runtimeGuideText: "문제: 식이 필요한 위치입니다.",
  execPathHint: "실행 경로: wasm(strict)",
  shapeMode: "native",
  parseWarningSummary: "점검 필요: 1건",
  viewFamilies: ["graph", "table"],
  previewSummary: "그래프+표",
  nonStrictFamilies: ["graph"],
  observationSummary: "t=0.2 theta=0.1",
});
assert(runtimeHintModel.status === "error", "runtime hint model should mark error on warning");
assert(runtimeHintModel.text.includes("3마디"), "runtime hint model should include tick count");
assert(!runtimeHintModel.text.includes("매김 조절: 2개"), "runtime hint model should keep control count out of top status");
assert(!runtimeHintModel.text.includes("실행 경로: wasm(strict)"), "runtime hint model should keep normal exec path out of top status");
assert(!runtimeHintModel.text.includes("보기: graph"), "runtime hint model should keep view family out of top status");
assert(!runtimeHintModel.text.includes("보기소스경고"), "runtime hint model should keep view source warning out of top status");
assert(!runtimeHintModel.text.includes("대표보기:"), "runtime hint model should keep preview summary out of top status");
assert(runtimeHintModel.text.includes("점검 필요:"), "runtime hint model should prioritize user-friendly problem text");
const runtimeFailureHintModel = buildRuntimeHintViewModel({
  tickCount: 3,
  execPathHint: "실행 실패(step): 문법 문제",
});
assert(runtimeFailureHintModel.text.includes("실행 실패"), "runtime hint model should keep failure path visible");
const execStatusModel = buildRunExecStatusViewModel({
  warnings: [{ code: "E_PARSE_EXPECTED_EXPR", message: "식이 필요한 위치입니다." }],
  execPathHint: "실행 경로: wasm(strict)",
  runtimeHintText: runtimeHintModel.text,
  parseWarningSummary: "점검 필요: 1건",
  viewSourceStrictness: { strict: false, nonStrictFamilies: ["graph"] },
});
assert(execStatusModel.userStatusText.startsWith("문제:"), "exec status model should prioritize user warning");
assert(execStatusModel.status === "error", "exec status model should mark error");
assert(execStatusModel.techSummaryText.includes("점검 필요"), "exec status model should include parse summary");
assert(String(execStatusModel.techBodyText ?? "").trim().length > 0, "exec status model should include technical hint body");
const execStatusModelIdle = buildRunExecStatusViewModel({
  warnings: [],
  execPathHint: "",
  runtimeHintText: "3마디 · 매김 조절: 2개",
  parseWarningSummary: "",
  viewSourceStrictness: { strict: true, nonStrictFamilies: [] },
});
assert(execStatusModelIdle.showTechnical === false, "exec status model should hide technical details when nothing technical exists");
assert(String(execStatusModelIdle.techBodyText ?? "") === "", "exec status model should clear technical body when idle");
const actionRailModelWarn = buildRunActionRailViewModel({
  warnings: [{ code: "E_BLOCK_HEADER_COLON_FORBIDDEN", message: "블록 헤더에는 ':'를 쓰지 않습니다. 예: 채비 {}" }],
  onboardingProfile: "",
  onboardingStatusText: "온보딩: 선택 대기",
});
assert(actionRailModelWarn.statusLevel === "warn", "action rail model should show warn on warnings");
assert(actionRailModelWarn.actions.openDdn.recommended === true, "action rail model should recommend ddn on syntax warning");
const actionRailModelIdle = buildRunActionRailViewModel({
  warnings: [],
  onboardingProfile: "",
  onboardingStatusText: "온보딩: 선택 대기",
});
assert(actionRailModelIdle.actions.onboardStudent.recommended === true, "action rail model should recommend student on idle");
const actionRailModelPlatformAuth = buildRunActionRailViewModel({
  warnings: [],
  onboardingProfile: "",
  onboardingStatusText: "온보딩: 선택 대기",
  platformErrorCode: PlatformServerAdapterErrorCode.AUTH_REQUIRED,
  platformActionRail: [PlatformServerAdapterUiAction.LOGIN, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
});
assert(actionRailModelPlatformAuth.statusLevel === "warn", "action rail model should warn on platform auth error");
assert(
  actionRailModelPlatformAuth.actions.openInspector.recommended === true,
  "action rail model should recommend inspector on platform auth error",
);
const actionRailModelPlatformValidation = buildRunActionRailViewModel({
  warnings: [],
  onboardingProfile: "",
  onboardingStatusText: "온보딩: 선택 대기",
  platformErrorCode: PlatformServerAdapterErrorCode.VALIDATION_FAILED,
  platformActionRail: [PlatformServerAdapterUiAction.FIX_INPUT, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
});
assert(
  actionRailModelPlatformValidation.actions.openDdn.recommended === true,
  "action rail model should recommend ddn on platform validation error",
);

const warningModel = buildWarningPanelViewModel({
  warnings: [
    {
      code: "E_RUNTIME_EXEC_FAILED",
      technical_code: "E_RUNTIME_EXEC_FAILED",
      message: "실행에 실패했습니다.",
      technical_message: "runtime failed",
      span: { line: 2, column: 5 },
    },
  ],
  lastWarningSignature: "",
});
assert(warningModel.hasWarnings === true, "warning panel model should be warning state");
assert(warningModel.panelLevel === "error", "warning panel model should mark error level");
assert(warningModel.primaryAction.kind === "retry", "warning panel model should expose recovery-first primary action");
assert(warningModel.userCategory === "실행입력", "warning panel model should expose user category");
assert(warningModel.ddnAction.recommended === true, "warning panel model should still keep ddn helper action");
assert(warningModel.shortcutsText.includes("권장: Alt+D"), "warning panel model should build ddn shortcut hint");
assert(warningModel.techBodyText.includes("(L2:C5)"), "warning panel model should include span location");
const platformWarningModelInspector = buildWarningPanelViewModel({
  warnings: [],
  lastWarningSignature: "",
  platformErrorCode: PlatformServerAdapterErrorCode.AUTH_REQUIRED,
  platformActionRail: [PlatformServerAdapterUiAction.LOGIN],
});
assert(
  platformWarningModelInspector.primaryAction.label === "거울에서 원인 확인",
  "warning panel model should use shared inspector CTA label",
);
const platformWarningModelAuth = buildWarningPanelViewModel({
  warnings: [],
  lastWarningSignature: "",
  platformErrorCode: PlatformServerAdapterErrorCode.AUTH_REQUIRED,
  platformActionRail: [PlatformServerAdapterUiAction.LOGIN, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
});
assert(platformWarningModelAuth.hasWarnings === true, "platform warning model should show warning state");
assert(platformWarningModelAuth.panelLevel === "warn", "platform warning model should mark warn level");
assert(
  platformWarningModelAuth.inspectorAction.recommended === true,
  "platform warning model should recommend inspector for auth error",
);
assert(
  platformWarningModelAuth.codes.includes(PlatformServerAdapterErrorCode.AUTH_REQUIRED),
  "platform warning model should include auth code",
);
assert(
  platformWarningModelAuth.platformLoginAction.hidden === false &&
    platformWarningModelAuth.platformLoginAction.recommended === true,
  "platform warning model should surface login action on auth error",
);
assert(
  platformWarningModelAuth.platformOpenLocalSaveAction.hidden === false,
  "platform warning model should surface local save action on auth error",
);
const platformWarningModelValidation = buildWarningPanelViewModel({
  warnings: [],
  lastWarningSignature: "",
  platformErrorCode: PlatformServerAdapterErrorCode.VALIDATION_FAILED,
  platformActionRail: [PlatformServerAdapterUiAction.FIX_INPUT, PlatformServerAdapterUiAction.OPEN_LOCAL_SAVE],
});
assert(
  platformWarningModelValidation.ddnAction.recommended === true,
  "platform warning model should recommend ddn for validation error",
);
assert(
  platformWarningModelValidation.primaryAction.label === "DDN 바로 수정",
  "warning panel model should use shared ddn CTA label",
);
assert(
  platformWarningModelValidation.shortcutsText.includes("권장: Alt+D"),
  "platform warning model should expose ddn shortcut for validation error",
);
assert(
  platformWarningModelValidation.platformOpenLocalSaveAction.hidden === false,
  "platform warning model should surface local save action on validation error",
);
assert(
  platformWarningModelValidation.platformLoginAction.hidden === true,
  "platform warning model should hide login action when not requested",
);

const clearModel = buildWarningPanelViewModel({ warnings: [], lastWarningSignature: "old" });
assert(clearModel.hasWarnings === false, "warning panel model should clear state");
assert(clearModel.signatureChanged === true, "warning panel model should mark clear transition");
assert(clearModel.platformOpenLocalSaveAction.hidden === true, "clear model should hide platform actions");

assert(
  normalizeObserveAction(OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT) === OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT,
  "observe action contract should normalize known action",
);
const actionPlan = buildObserveActionPlan(OBSERVE_ACTION_OPEN_DDN_OBSERVE_OUTPUT, { observeToken: "table.row" });
assert(actionPlan?.kind === "open-ddn-token", "observe action contract should build plan kind");
assert(actionPlan?.token === "table.row", "observe action contract should carry token payload");
assert(buildObserveActionPlan("unknown-action", {}) === null, "observe action contract should reject unknown action");

const readinessReady = buildStudioEditorReadinessModel({
  sourceText: "채비 { 값:수 <- 1. }.",
  canonDiagCode: "",
  canonDiagMessage: "",
  autofixAvailable: false,
});
assert(readinessReady.stage === STUDIO_READINESS_STAGE.READY, "studio readiness should be ready on clean source");

const readinessAutofix = buildStudioEditorReadinessModel({
  sourceText: "채비: { 값:수 <- 1. }.",
  canonDiagCode: "",
  canonDiagMessage: "",
  autofixAvailable: true,
});
assert(readinessAutofix.stage === STUDIO_READINESS_STAGE.AUTOFIX, "studio readiness should flag autofix stage");

const readinessBlocked = buildStudioEditorReadinessModel({
  sourceText: "",
  canonDiagCode: "",
  canonDiagMessage: "",
  autofixAvailable: false,
});
assert(readinessBlocked.stage === STUDIO_READINESS_STAGE.BLOCKED, "studio readiness should block empty source");

const autofixContract = buildAutofixResultContract(
  {
    changed: true,
    stats: {
      setup_colon_rewrites: 1,
    },
  },
  { sourceTextAfter: "채비 { 값:수 <- (1). }." },
);
assert(autofixContract.changed === true, "autofix contract should expose changed flag");
assert(Array.isArray(autofixContract.applied_rules) && autofixContract.applied_rules.length > 0, "autofix contract should list applied rules");

console.log("seamgrim run warning message map runner ok");
