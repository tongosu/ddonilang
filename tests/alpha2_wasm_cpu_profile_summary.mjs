// Diagnostic helper only: never a product input/replay format or closure gate.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';

const [wasmPath, profilePath] = process.argv.slice(2);
assert.ok(wasmPath && profilePath, 'Usage: node <helper> <symbolized.wasm> <same-run.cpuprofile>');
const wasm = fs.readFileSync(wasmPath);
const profileBytes = fs.readFileSync(profilePath);
const sections = WebAssembly.Module.customSections(new WebAssembly.Module(wasm), 'name');
const names = new Map();
for (const section of sections) {
  const bytes = new Uint8Array(section);
  let cursor = 0;
  const uint = () => {
    let value = 0;
    for (let shift = 0; shift <= 28; shift += 7) {
      assert.ok(cursor < bytes.length, 'Truncated name section');
      const byte = bytes[cursor++];
      value += (byte & 127) * 2 ** shift;
      if (!(byte & 128)) return value;
    }
    throw new Error('Invalid name-section integer');
  };
  while (cursor < bytes.length) {
    const kind = bytes[cursor++];
    const length = uint();
    const end = cursor + length;
    assert.ok(end <= bytes.length, 'Invalid name subsection length');
    if (kind === 1) {
      const count = uint();
      for (let n = 0; n < count; n++) {
        const index = uint();
        const size = uint();
        assert.ok(cursor + size <= end, 'Truncated function name');
        names.set(index, new TextDecoder('utf-8', {fatal:true}).decode(bytes.subarray(cursor, cursor + size)));
        cursor += size;
      }
      assert.equal(cursor, end);
    }
    cursor = end;
  }
}
const totals = new Map();
for (const node of JSON.parse(profileBytes).nodes) {
  const original = node.callFrame.functionName;
  const index = /^wasm-function\[(\d+)\]$/.exec(original);
  const name = index ? names.get(Number(index[1])) ?? original : original || '(anonymous)';
  totals.set(name, (totals.get(name) ?? 0) + (node.hitCount ?? 0));
}
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
console.log(JSON.stringify({
  scope: 'Observational CPU self-samples only; caller must bind profile to this exact WASM run. No cross-binary index mapping.',
  wasm_sha256: sha(wasm), profile_sha256: sha(profileBytes), named_functions:names.size,
  hottest:[...totals].sort((a,b) => b[1]-a[1]).slice(0,30).map(([name,self_samples]) => ({name,self_samples})),
}, null, 2));
