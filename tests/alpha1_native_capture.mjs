import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';

// Verification evidence only. No product frontdoor reads these files.
// stdout bytes are never dropped, compacted or rewritten. The process deadline
// and stderr buffer remain the caller's existing values.
export function captureNative(executable, args, options, directory, root) {
  if (!directory) return spawnSync(executable, args, options);
  const relative = path.relative(path.resolve(root), path.resolve(directory));
  assert(relative.startsWith('..' + path.sep) || path.isAbsolute(relative),
    'Native stdout evidence must stay outside the product checkout');
  fs.mkdirSync(directory, {recursive:true});
  const filename = path.join(directory, `stdout-${process.pid}-${captureNative.sequence++}.json`);
  const descriptor = fs.openSync(filename, 'wx');
  let result;
  try {
    result = spawnSync(executable, args, {...options, stdio:['ignore', descriptor, 'pipe']});
  } finally { fs.closeSync(descriptor); }
  const bytes = fs.readFileSync(filename);
  result.stdout = bytes.toString('utf8');
  result.stdoutEvidence = {path:filename, bytes:bytes.length,
    sha256:createHash('sha256').update(bytes).digest('hex')};
  return result;
}
captureNative.sequence = 0;
