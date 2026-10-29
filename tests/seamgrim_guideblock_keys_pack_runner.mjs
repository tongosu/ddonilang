import fs from "fs";
import path from "path";
import { spawnSync } from "child_process";
import { pathToFileURL } from "url";

function parseArgs(argv) {
  const out = {
    packRoot: "pack/guideblock_keys_basics",
    caseFile: "",
    jsonOut: "",
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

function readJson(pathValue) {
  return JSON.parse(fs.readFileSync(pathValue, "utf-8"));
}

function normalizeObject(obj) {
  const out = {};
  const source = obj && typeof obj === "object" ? obj : {};
  for (const key of Object.keys(source).sort()) {
    out[key] = source[key];
  }
  return out;
}

function trimText(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").trim();
}

function parseGoldenCases(packRoot) {
  const goldenPath = path.join(packRoot, "golden.jsonl");
  if (!fs.existsSync(goldenPath)) {
    return { ok: false, reason: `missing_golden:${goldenPath}`, rows: [] };
  }
  const rows = [];
  const lines = fs.readFileSync(goldenPath, "utf-8").split(/\r?\n/u);
  for (let idx = 0; idx < lines.length; idx += 1) {
    const line = String(lines[idx] ?? "").trim();
    if (!line) continue;
    let row;
    try {
      row = JSON.parse(line);
    } catch (err) {
      return { ok: false, reason: `golden_parse_failed:line=${idx + 1}:${err}`, rows: [] };
    }
    const rel = String(row?.guideblock_case ?? "").trim();
    if (!rel) {
      return { ok: false, reason: `golden_case_missing:line=${idx + 1}`, rows: [] };
    }
    const casePath = path.resolve(packRoot, rel);
    if (!casePath.startsWith(path.resolve(packRoot))) {
      return { ok: false, reason: `golden_case_outside_pack:line=${idx + 1}:${rel}`, rows: [] };
    }
    if (!fs.existsSync(casePath)) {
      return { ok: false, reason: `golden_case_missing_file:line=${idx + 1}:${rel}`, rows: [] };
    }
    rows.push({ line: idx + 1, rel, casePath });
  }
  if (!rows.length) {
    return { ok: false, reason: "golden_empty", rows: [] };
  }
  return { ok: true, reason: "", rows };
}

function resolveSingleCasePath(packRoot, caseFile) {
  const target = path.resolve(packRoot, caseFile);
  if (!target.startsWith(path.resolve(packRoot))) {
    return { ok: false, reason: `invalid_case_file:${caseFile}`, rows: [] };
  }
  if (!fs.existsSync(target)) {
    return { ok: false, reason: `missing_case_file:${caseFile}`, rows: [] };
  }
  return { ok: true, reason: "", rows: [{ line: 1, rel: caseFile, casePath: target }] };
}

function runToolParser(root, inputPath) {
  const py = String(process.env.PYTHON ?? "python").trim() || "python";
  const proc = spawnSync(
    py,
    ["tests/seamgrim_guideblock_tool_parse.py", "--input", inputPath],
    {
      cwd: root,
      encoding: "utf-8",
      env: {
        ...process.env,
        PYTHONUTF8: "1",
        PYTHONIOENCODING: "utf-8",
      },
    },
  );
  if (proc.status !== 0) {
    const detail = String(proc.stderr ?? "").trim() || String(proc.stdout ?? "").trim() || `returncode=${proc.status}`;
    return { ok: false, detail, parsed: null };
  }
  try {
    const parsed = JSON.parse(String(proc.stdout ?? ""));
    return { ok: true, detail: "", parsed };
  } catch (err) {
    return { ok: false, detail: `tool_parse_json_failed:${err}`, parsed: null };
  }
}

async function main() {
  const root = process.cwd();
  const args = parseArgs(process.argv.slice(2));
  const packRoot = path.resolve(root, args.packRoot);
  const guideMetaPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/components/guide_meta.js");
  const { parseGuideMetaHeader } = await import(pathToFileURL(guideMetaPath).href);

  const report = {
    schema: "ddn.seamgrim.guideblock_keys_pack_report.v1",
    generated_at_utc: new Date().toISOString(),
    ok: false,
    pack_root: packRoot,
    cases: [],
    failure_digest: [],
  };

  const scan = args.caseFile ? resolveSingleCasePath(packRoot, args.caseFile) : parseGoldenCases(packRoot);
  if (!scan.ok) {
    report.failure_digest.push(scan.reason);
    if (args.jsonOut) {
      fs.mkdirSync(path.dirname(args.jsonOut), { recursive: true });
      fs.writeFileSync(args.jsonOut, JSON.stringify(report, null, 2) + "\n", "utf-8");
    }
    if (!args.quiet) {
      console.log(`guideblock keys pack failed: ${scan.reason}`);
    }
    return 1;
  }

  for (const row of scan.rows) {
    const caseDoc = readJson(row.casePath);
    const caseId = String(caseDoc?.case_id ?? path.basename(path.dirname(row.casePath))).trim();
    const schema = String(caseDoc?.schema ?? "");
    if (schema !== "ddn.seamgrim.guideblock_case.v1") {
      const detail = `check=${caseId} invalid_schema:${schema}`;
      report.cases.push({ case_id: caseId, ok: false, detail, case_path: row.casePath });
      report.failure_digest.push(detail);
      continue;
    }
    const inputRel = String(caseDoc?.input ?? "").trim();
    if (!inputRel) {
      const detail = `check=${caseId} missing_input`;
      report.cases.push({ case_id: caseId, ok: false, detail, case_path: row.casePath });
      report.failure_digest.push(detail);
      continue;
    }
    const inputPath = path.resolve(path.dirname(row.casePath), inputRel);
    if (!fs.existsSync(inputPath)) {
      const detail = `check=${caseId} missing_input_file:${inputRel}`;
      report.cases.push({ case_id: caseId, ok: false, detail, case_path: row.casePath });
      report.failure_digest.push(detail);
      continue;
    }

    const expect = caseDoc?.expect && typeof caseDoc.expect === "object" ? caseDoc.expect : {};
    const expectedMeta = expect?.meta && typeof expect.meta === "object" ? expect.meta : {};
    const expectedBodyStartsWith = String(expect?.body_starts_with ?? "").trim();

    const inputText = fs.readFileSync(inputPath, "utf-8");
    const jsParsed = parseGuideMetaHeader(inputText);
    const toolResult = runToolParser(root, inputPath);
    if (!toolResult.ok) {
      const detail = `check=${caseId} tool_parser_failed:${toolResult.detail}`;
      report.cases.push({ case_id: caseId, ok: false, detail, case_path: row.casePath });
      report.failure_digest.push(detail);
      continue;
    }
    const pyParsed = toolResult.parsed ?? {};

    const jsMeta = normalizeObject(jsParsed?.meta ?? {});
    const pyMeta = normalizeObject(pyParsed?.meta ?? {});
    const jsBody = trimText(jsParsed?.body ?? "");
    const pyBody = trimText(pyParsed?.body ?? "");
    const expectedBodyOk = !expectedBodyStartsWith || jsBody.startsWith(expectedBodyStartsWith);

    let metaExpectOk = true;
    const mismatches = [];
    for (const [key, expectedValue] of Object.entries(expectedMeta)) {
      const jsValue = String(jsMeta[key] ?? "");
      if (jsValue !== String(expectedValue ?? "")) {
        metaExpectOk = false;
        mismatches.push(`meta.${key}:expected=${expectedValue}:actual=${jsValue}`);
      }
    }

    const parityMetaOk = JSON.stringify(jsMeta) === JSON.stringify(pyMeta);
    const parityBodyOk = jsBody === pyBody;
    const caseOk = metaExpectOk && expectedBodyOk && parityMetaOk && parityBodyOk;
    const detail = [
      `expected_meta_ok=${Number(metaExpectOk)}`,
      `expected_body_ok=${Number(expectedBodyOk)}`,
      `parity_meta_ok=${Number(parityMetaOk)}`,
      `parity_body_ok=${Number(parityBodyOk)}`,
      mismatches.length ? `mismatch=${mismatches.join(",")}` : "",
    ]
      .filter(Boolean)
      .join(" ");

    if (!caseOk) {
      report.failure_digest.push(`check=${caseId} ${detail}`);
      if (!args.quiet) {
        console.log(`check=${caseId} ${detail}`);
      }
    } else if (!args.quiet) {
      console.log(`[guideblock-pack] case=${caseId} ok=1`);
    }

    report.cases.push({
      case_id: caseId,
      case_path: row.casePath,
      input_path: inputPath,
      ok: caseOk,
      detail,
      js_meta: jsMeta,
      py_meta: pyMeta,
    });
  }

  const failed = report.cases.filter((item) => !item.ok);
  report.ok = failed.length === 0;
  if (args.jsonOut) {
    fs.mkdirSync(path.dirname(args.jsonOut), { recursive: true });
    fs.writeFileSync(args.jsonOut, JSON.stringify(report, null, 2) + "\n", "utf-8");
    if (!args.quiet) {
      console.log(`[guideblock-pack] report=${args.jsonOut}`);
    }
  }
  if (!report.ok) {
    if (!args.quiet) {
      console.log(`guideblock keys pack failed: failed_cases=${failed.length}`);
    }
    return 1;
  }
  if (!args.quiet) {
    console.log(`[guideblock-pack] ok cases=${report.cases.length}`);
  }
  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((err) => {
    console.error(String(err?.stack ?? err));
    process.exit(1);
  });
