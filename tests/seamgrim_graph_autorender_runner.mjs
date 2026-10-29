#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function canonical(data) {
  return JSON.stringify(data, Object.keys(data).sort());
}

async function main() {
  const packDirArg = process.argv[2];
  const fixtureRel = process.argv[3];
  const preferPatchRaw = process.argv[4] ?? "false";
  if (!packDirArg || !fixtureRel) {
    throw new Error("usage: node seamgrim_graph_autorender_runner.mjs <pack_dir> <fixture_rel> [prefer_patch]");
  }

  const packDir = path.resolve(packDirArg);
  const fixturePath = path.join(packDir, fixtureRel);
  const fixture = JSON.parse(await fs.readFile(fixturePath, "utf8"));
  const preferPatch = String(preferPatchRaw).toLowerCase() === "true";

  const moduleUrl = pathToFileURL(
    path.resolve("solutions/seamgrim_ui_mvp/ui/graph_autorender.js"),
  ).href;
  const mod = await import(moduleUrl);
  const graph = mod.buildGraphFromValueResources(fixture.state, preferPatch);

  process.stdout.write(`${JSON.stringify(graph, null, 2)}\n`);
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
