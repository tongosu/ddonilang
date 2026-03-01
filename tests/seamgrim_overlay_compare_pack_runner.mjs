import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function parseArgs(argv) {
  const out = {
    packRoot: "pack/seamgrim_overlay_param_compare_v0",
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

function normalizeExpectedOk(expect) {
  if (typeof expect?.overlay_ok === "boolean") return expect.overlay_ok;
  if (typeof expect?.ok === "boolean") return expect.ok;
  return null;
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

async function main() {
  const root = process.cwd();
  const args = parseArgs(process.argv.slice(2));
  const packRoot = path.resolve(root, args.packRoot);
  const contractPath = path.resolve(root, "tests/contracts/overlay_compare_contract.mjs");
  const contract = await import(pathToFileURL(contractPath).href);
  const { canOverlayCompareRuns } = contract;
  const scan = args.caseFile ? resolveSingleCasePath(packRoot, args.caseFile) : collectCasePaths(packRoot);

  const report = {
    schema: "ddn.seamgrim.overlay_compare_pack_report.v1",
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
      console.log(`overlay compare pack failed: ${scan.reason}`);
    }
    return 1;
  }

  for (const casePath of scan.casePaths) {
    const raw = loadJsonFile(casePath);
    const caseId = String(raw.case_id ?? path.basename(path.dirname(casePath)));
    const expect = raw.expect ?? {};
    const expectedOk = normalizeExpectedOk(expect);
    if (typeof expectedOk !== "boolean") {
      const msg = `check=${caseId} missing=expect.overlay_ok`;
      report.failure_digest.push(msg);
      report.cases.push({
        case_id: caseId,
        case_path: casePath,
        ok: false,
        detail: msg,
      });
      continue;
    }
    const actual = canOverlayCompareRuns(raw.baseline, raw.variant);
    const expectedCode = typeof expect.code === "string" ? expect.code : "";
    const reasonContains = typeof expect.reason_contains === "string" ? expect.reason_contains : "";
    const okMatch = Boolean(actual.ok) === expectedOk;
    const codeMatch = expectedCode ? String(actual.code ?? "") === expectedCode : true;
    const reasonMatch = reasonContains ? String(actual.reason ?? "").includes(reasonContains) : true;
    const caseOk = okMatch && codeMatch && reasonMatch;
    const detail = `expected_ok=${Number(expectedOk)} actual_ok=${Number(Boolean(actual.ok))} expected_code=${
      expectedCode || "-"
    } actual_code=${String(actual.code ?? "-")}`;
    if (!caseOk) {
      const line = `check=${caseId} ${detail} reason=${String(actual.reason ?? "-")}`;
      report.failure_digest.push(line);
      if (!args.quiet) {
        console.log(line);
      }
    } else {
      if (!args.quiet) {
        console.log(`[overlay-pack] case=${caseId} ok=1 code=${String(actual.code ?? "-")}`);
      }
    }
    report.cases.push({
      case_id: caseId,
      case_path: casePath,
      ok: caseOk,
      expected_ok: expectedOk,
      expected_code: expectedCode || null,
      reason_contains: reasonContains || null,
      actual,
      detail,
    });
  }

  const failed = report.cases.filter((row) => !row.ok);
  report.ok = failed.length === 0;
  if (args.jsonOut) {
    fs.mkdirSync(path.dirname(args.jsonOut), { recursive: true });
    fs.writeFileSync(args.jsonOut, JSON.stringify(report, null, 2) + "\n", "utf-8");
    if (!args.quiet) {
      console.log(`[overlay-pack] report=${args.jsonOut}`);
    }
  }
  if (!report.ok) {
    if (!args.quiet) {
      console.log(`overlay compare pack failed: failed_cases=${failed.length}`);
    }
    return 1;
  }
  if (!args.quiet) {
    console.log(`[overlay-pack] ok cases=${report.cases.length}`);
  }
  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error(String(err?.stack ?? err));
    process.exit(1);
  });
