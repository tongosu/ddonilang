import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { performance } from "node:perf_hooks";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const viewports = ["1440x900", "1024x768", "778x912", "390x844"];
const results = [];
for (const viewport of viewports) {
  for (let repetition = 1; repetition <= 3; repetition += 1) {
    const started = performance.now();
    const result = spawnSync(process.execPath, ["tests/seamgrim_gaji_bundle_runner.mjs"], {
      cwd: root, env: { ...process.env, GAJI_VIEWPORT: viewport }, encoding: "utf8", timeout: 180_000,
    });
    assert.equal(result.status, 0, `${viewport} repetition ${repetition} failed\n${result.stdout}\n${result.stderr}`);
    assert.match(result.stdout, new RegExp(`seamgrim_gaji_bundle ${viewport.replace("x", "x")}: ok`));
    results.push({ viewport, repetition, elapsed_ms: Math.round(performance.now() - started) });
  }
}
assert.equal(results.length, 12);
console.log(JSON.stringify({ schema: "ddn.gaji_four_viewport_circulation.v1", runs: results, all_ok: true }));
console.log("gaji_four_viewport: ok");
