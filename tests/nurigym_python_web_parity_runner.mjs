#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { pathToFileURL } from "url";

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

async function main() {
  const root = process.cwd();
  const packDir = path.join(root, "pack", "nurigym_python_web_parity_v1");
  const cases = readJson(path.join(packDir, "cases.detjson"));
  const modulePath = path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "nurigym_python_web_parity.js");
  const parity = await import(pathToFileURL(modulePath).href);
  if (typeof parity.compareNuriGymDatasetParity !== "function") {
    throw new Error("compareNuriGymDatasetParity export missing");
  }

  const rows = [];
  for (const testCase of cases.cases ?? []) {
    const inputPath = path.join(root, testCase.input);
    const datasetPath = path.join(root, testCase.dataset);
    const result = parity.compareNuriGymDatasetParity({
      input: readJson(inputPath),
      datasetJsonl: fs.readFileSync(datasetPath, "utf8"),
    });
    rows.push({
      id: testCase.id,
      input: testCase.input,
      dataset: testCase.dataset,
      ...result,
    });
  }

  const report = {
    schema: "ddn.nurigym.python_web_parity.report.v1",
    ok: rows.every((row) => row.ok),
    case_count: rows.length,
    cases: rows,
  };
  process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  if (!report.ok) process.exit(1);
}

main().catch((err) => {
  console.error(err?.stack ?? String(err));
  process.exit(1);
});
