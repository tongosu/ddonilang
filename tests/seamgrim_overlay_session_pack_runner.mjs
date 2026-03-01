import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function parseArgs(argv) {
  const out = {
    packRoot: "pack/seamgrim_overlay_session_roundtrip_v0",
    jsonOut: "",
    caseFile: "",
    quiet: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const key = argv[i];
    if (key === "--pack-root" && i + 1 < argv.length) {
      out.packRoot = String(argv[i + 1]);
      i += 1;
      continue;
    }
    if (key === "--json-out" && i + 1 < argv.length) {
      out.jsonOut = String(argv[i + 1]);
      i += 1;
      continue;
    }
    if (key === "--case-file" && i + 1 < argv.length) {
      out.caseFile = String(argv[i + 1]);
      i += 1;
      continue;
    }
    if (key === "--quiet") {
      out.quiet = true;
    }
  }
  return out;
}

function loadJsonFile(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf-8"));
}

function collectCasePaths(packRoot) {
  if (!fs.existsSync(packRoot)) {
    return { ok: false, casePaths: [], reason: `missing pack root: ${packRoot}` };
  }
  const casePaths = [];
  const children = fs
    .readdirSync(packRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name.startsWith("c"))
    .map((entry) => entry.name)
    .sort();
  for (const name of children) {
    const casePath = path.join(packRoot, name, "case.detjson");
    if (!fs.existsSync(casePath)) {
      return { ok: false, casePaths: [], reason: `missing pack case: ${casePath}` };
    }
    casePaths.push(casePath);
  }
  if (!casePaths.length) {
    return { ok: false, casePaths: [], reason: `missing pack case directories under: ${packRoot}` };
  }
  return { ok: true, casePaths, reason: "" };
}

function resolveSingleCasePath(packRoot, caseFile) {
  const target = path.resolve(packRoot, caseFile);
  if (!target.startsWith(packRoot)) {
    return { ok: false, casePaths: [], reason: `invalid case-file path: ${caseFile}` };
  }
  if (!fs.existsSync(target)) {
    return { ok: false, casePaths: [], reason: `missing pack case: ${target}` };
  }
  return { ok: true, casePaths: [target], reason: "" };
}

function toRuntimeRun(raw) {
  const row = raw && typeof raw === "object" ? raw : {};
  return {
    id: row.id ?? "",
    label: row.label ?? "",
    visible: row.visible ?? true,
    layerIndex: Number.isFinite(row.layer_index) ? row.layer_index : 0,
    compareRole: row.compare_role ?? null,
    source: row.source ?? {},
    inputs: row.inputs ?? {},
    graph: row.graph ?? null,
    space2d: row.space2d ?? null,
    textDoc: row.text_doc ?? null,
  };
}

function normalizeExpected(expect) {
  const row = expect && typeof expect === "object" ? expect : {};
  return {
    enabled: Boolean(row.enabled),
    baselineId: typeof row.baseline_id === "string" ? row.baseline_id : null,
    variantId: typeof row.variant_id === "string" ? row.variant_id : null,
    droppedVariant: Boolean(row.dropped_variant),
    dropCode: typeof row.drop_code === "string" ? row.drop_code : "",
    blockReasonContains: typeof row.block_reason_contains === "string" ? row.block_reason_contains : "",
  };
}

function normalizeActual(actual) {
  const row = actual && typeof actual === "object" ? actual : {};
  return {
    enabled: Boolean(row.enabled),
    baselineId: typeof row.baselineId === "string" ? row.baselineId : null,
    variantId: typeof row.variantId === "string" ? row.variantId : null,
    droppedVariant: Boolean(row.droppedVariant),
    dropCode: typeof row.dropCode === "string" ? row.dropCode : "",
    blockReason: typeof row.blockReason === "string" ? row.blockReason : "",
  };
}

function normalizeExpectedUiLayout(layout) {
  if (!layout || typeof layout !== "object") return null;
  return {
    screen_mode: typeof layout.screen_mode === "string" ? layout.screen_mode : "",
    workspace_mode: typeof layout.workspace_mode === "string" ? layout.workspace_mode : "",
    main_tab: typeof layout.main_tab === "string" ? layout.main_tab : "",
    active_view: typeof layout.active_view === "string" ? layout.active_view : "",
  };
}

function normalizeExpectedViewCombo(viewCombo) {
  if (!viewCombo || typeof viewCombo !== "object") return null;
  return {
    enabled: Boolean(viewCombo.enabled),
    layout: typeof viewCombo.layout === "string" ? viewCombo.layout : "",
    overlay_order: typeof viewCombo.overlay_order === "string" ? viewCombo.overlay_order : "",
  };
}

function compareRow(expected, actual) {
  const checks = [];
  checks.push(expected.enabled === actual.enabled);
  checks.push(expected.baselineId === actual.baselineId);
  checks.push(expected.variantId === actual.variantId);
  checks.push(expected.droppedVariant === actual.droppedVariant);
  checks.push(expected.dropCode === actual.dropCode);
  if (expected.blockReasonContains) {
    checks.push(actual.blockReason.includes(expected.blockReasonContains));
  }
  return checks.every(Boolean);
}

async function main() {
  const root = process.cwd();
  const args = parseArgs(process.argv.slice(2));
  const packRoot = path.resolve(root, args.packRoot);
  const contractPath = path.resolve(root, "tests/contracts/overlay_session_contract.mjs");
  const contract = await import(pathToFileURL(contractPath).href);
  const {
    resolveOverlayCompareFromSession,
    buildOverlayCompareSessionPayload,
    resolveSessionUiLayoutFromPayload,
    buildSessionUiLayoutPayload,
    resolveSessionViewComboFromPayload,
    buildSessionViewComboPayload,
  } = contract;
  const scan = args.caseFile ? resolveSingleCasePath(packRoot, args.caseFile) : collectCasePaths(packRoot);

  const report = {
    schema: "ddn.seamgrim.overlay_session_pack_report.v1",
    generated_at_utc: new Date().toISOString(),
    ok: false,
    pack_root: packRoot,
    cases: [],
    failure_digest: [],
  };

  if (!scan.ok) {
    report.failure_digest.push(scan.reason);
    if (args.jsonOut) {
      fs.mkdirSync(path.dirname(args.jsonOut), { recursive: true });
      fs.writeFileSync(args.jsonOut, JSON.stringify(report, null, 2) + "\n", "utf-8");
    }
    if (!args.quiet) {
      console.log(`overlay session pack failed: ${scan.reason}`);
    }
    return 1;
  }

  for (const casePath of scan.casePaths) {
    const raw = loadJsonFile(casePath);
    const caseId = String(raw.case_id ?? path.basename(path.dirname(casePath)));
    const runsRaw = Array.isArray(raw?.session_in?.runs) ? raw.session_in.runs : [];
    const runs = runsRaw.map((row) => toRuntimeRun(row));
    const compareInput = raw?.session_in?.compare ?? {};
    const uiLayoutInput = raw?.session_in?.ui_layout ?? {};
    const viewComboInput = raw?.session_in?.view_combo ?? {};
    const expected = normalizeExpected(raw?.expect ?? {});
    const expectedUiLayout = normalizeExpectedUiLayout(raw?.expect?.ui_layout);
    const expectedViewCombo = normalizeExpectedViewCombo(raw?.expect?.view_combo);
    const resolved = resolveOverlayCompareFromSession({
      runs,
      compare: compareInput,
    });
    const actual = normalizeActual(resolved);
    const resolvedUiLayout = resolveSessionUiLayoutFromPayload(uiLayoutInput);
    const actualUiLayout = buildSessionUiLayoutPayload(resolvedUiLayout);
    const resolvedViewCombo = resolveSessionViewComboFromPayload(viewComboInput);
    const actualViewCombo = buildSessionViewComboPayload(resolvedViewCombo);
    const roundtripCompare = buildOverlayCompareSessionPayload({
      enabled: resolved.enabled,
      baselineId: resolved.baselineId,
      variantId: resolved.variantId,
    });
    const expectedRoundtrip = {
      enabled: expected.enabled,
      baseline_id: expected.baselineId,
      variant_id: expected.variantId,
    };
    const rowOk = compareRow(expected, actual);
    const roundtripOk = JSON.stringify(roundtripCompare) === JSON.stringify(expectedRoundtrip);
    const uiLayoutOk = expectedUiLayout ? JSON.stringify(actualUiLayout) === JSON.stringify(expectedUiLayout) : true;
    const viewComboOk = expectedViewCombo
      ? JSON.stringify(actualViewCombo) === JSON.stringify(expectedViewCombo)
      : true;
    const caseOk = rowOk && roundtripOk && uiLayoutOk && viewComboOk;
    const detail = `enabled=${Number(actual.enabled)} baseline=${actual.baselineId ?? "-"} variant=${
      actual.variantId ?? "-"
    } dropped=${Number(actual.droppedVariant)} drop_code=${actual.dropCode || "-"} layout=${actualUiLayout.screen_mode}/${actualUiLayout.workspace_mode}/${actualUiLayout.main_tab}/${actualUiLayout.active_view} combo=${Number(actualViewCombo.enabled)}/${actualViewCombo.layout}/${actualViewCombo.overlay_order}`;
    if (!caseOk) {
      const line = `check=${caseId} ${detail} expected_enabled=${Number(expected.enabled)} expected_baseline=${
        expected.baselineId ?? "-"
      } expected_variant=${expected.variantId ?? "-"} expected_drop_code=${expected.dropCode || "-"} expected_layout=${
        expectedUiLayout
          ? `${expectedUiLayout.screen_mode}/${expectedUiLayout.workspace_mode}/${expectedUiLayout.main_tab}/${expectedUiLayout.active_view}`
          : "-"
      } expected_combo=${
        expectedViewCombo
          ? `${Number(expectedViewCombo.enabled)}/${expectedViewCombo.layout}/${expectedViewCombo.overlay_order}`
          : "-"
      } reason=${actual.blockReason || "-"}`;
      report.failure_digest.push(line);
      if (!args.quiet) {
        console.log(line);
      }
    } else if (!args.quiet) {
      console.log(`[overlay-session-pack] case=${caseId} ok=1 baseline=${actual.baselineId ?? "-"} variant=${actual.variantId ?? "-"}`);
    }
    report.cases.push({
      case_id: caseId,
      case_path: casePath,
      ok: caseOk,
      expected,
      actual,
      roundtrip_compare_actual: roundtripCompare,
      roundtrip_compare_expected: expectedRoundtrip,
      expected_ui_layout: expectedUiLayout,
      actual_ui_layout: actualUiLayout,
      expected_view_combo: expectedViewCombo,
      actual_view_combo: actualViewCombo,
      detail,
    });
  }

  const failed = report.cases.filter((row) => !row.ok);
  report.ok = failed.length === 0;
  if (args.jsonOut) {
    fs.mkdirSync(path.dirname(args.jsonOut), { recursive: true });
    fs.writeFileSync(args.jsonOut, JSON.stringify(report, null, 2) + "\n", "utf-8");
    if (!args.quiet) {
      console.log(`[overlay-session-pack] report=${args.jsonOut}`);
    }
  }
  if (!report.ok) {
    if (!args.quiet) {
      console.log(`overlay session pack failed: failed_cases=${failed.length}`);
    }
    return 1;
  }
  if (!args.quiet) {
    console.log(`[overlay-session-pack] ok cases=${report.cases.length}`);
  }
  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error(String(err?.stack ?? err));
    process.exit(1);
  });
