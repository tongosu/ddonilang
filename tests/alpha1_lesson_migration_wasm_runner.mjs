// Review-only: feeds unchanged migrated DDN bytes to the actual product WASM.
// This does not introduce a product replay format or source lowering.
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Worker, isMainThread, parentPort, workerData } from 'node:worker_threads';
import { createHash } from 'node:crypto';

if (!isMainThread) {
  try {
    const wasm = await import(pathToFileURL(path.join(workerData.wasmDir, 'ddonirang_tool.js')).href);
    await wasm.default({ module_or_path: fs.readFileSync(path.join(workerData.wasmDir, 'ddonirang_tool_bg.wasm')) });
    const source = fs.readFileSync(workerData.sourcePath, 'utf8');
    const result = JSON.parse(wasm.wasm_run_summary_with_source_profile_json(source, workerData.sourcePath, 3, 'v1-core-v25'));
    parentPort.postMessage({ status: 'EXECUTED', state_hash: result.state_hash, outputs: result.output_log_texts });
  } catch (error) {
    parentPort.postMessage({ status: 'REJECTED', error: String(error) });
  }
} else {
  const [reviewDir, wasmDirectory] = process.argv.slice(2);
  if (!reviewDir || !wasmDirectory) throw new Error('usage: runner <immutable-review-dir> <product-wasm-dir>');
  const output = path.join(reviewDir, 'wasm-parity.json');
  if (fs.existsSync(output)) throw new Error('existing evidence is immutable; choose a successor review directory');
  const report = JSON.parse(fs.readFileSync(path.join(reviewDir, 'report.json'), 'utf8'));
  const candidates = report.rows.filter(row => row.cli?.returncode === 0);
  let next = 0;
  const rows = [];
  async function check(row) {
    const sourcePath = path.resolve(reviewDir, 'candidates', row.path);
    const actualHash = createHash('sha256').update(fs.readFileSync(sourcePath)).digest('hex');
    if (actualHash !== row.output_sha256) throw new Error(`candidate bytes changed: ${row.path}`);
    return await new Promise(resolve => {
      const worker = new Worker(new URL(import.meta.url), { workerData: {
        wasmDir: path.resolve(wasmDirectory), sourcePath,
      } });
      let settled = false;
      const finish = result => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        void worker.terminate();
        resolve({ path: row.path, source_sha256: row.output_sha256, ...result });
      };
      const timer = setTimeout(() => finish({ status: 'NOT_RUN_ENVIRONMENT', error: 'review worker host timeout; not a language failure' }), 30000);
      worker.once('message', result => finish({ ...result, parity: result.status === 'EXECUTED'
        && result.state_hash === row.cli.state_hash && JSON.stringify(result.outputs) === JSON.stringify(row.cli.outputs) }));
      worker.once('error', error => finish({ status: 'NOT_RUN_ENVIRONMENT', error: String(error) }));
      worker.once('exit', code => { if (!settled) finish({ status: 'NOT_RUN_ENVIRONMENT', error: `worker exited ${code}` }); });
    });
  }
  await Promise.all(Array.from({ length: 4 }, async () => {
    while (next < candidates.length) rows.push(await check(candidates[next++]));
  }));
  rows.sort((a, b) => a.path.localeCompare(b.path));
  const summary = { count: rows.length, matched: rows.filter(row => row.parity).length,
    rejected: rows.filter(row => row.status === 'REJECTED').length,
    not_run: rows.filter(row => row.status === 'NOT_RUN_ENVIRONMENT').length,
    product_promotion: false, input_membership_sha256: report.input_membership_sha256,
    wasm_sha256: createHash('sha256')
      .update(fs.readFileSync(path.join(wasmDirectory, 'ddonirang_tool_bg.wasm'))).digest('hex') };
  fs.writeFileSync(output, JSON.stringify({ ...summary, rows }, null, 2));
  console.log(JSON.stringify(summary));
  if (summary.matched !== summary.count) process.exitCode = 1;
}
