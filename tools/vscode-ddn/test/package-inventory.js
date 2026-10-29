const assert = require("assert");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const expectedFiles = [
  "README.md",
  "extension.js",
  "language-configuration.json",
  "package-lock.json",
  "package.json",
  "snippets/ddn.code-snippets",
  "syntaxes/ddn.tmLanguage.json",
].sort();
const requiredIgnoreRules = ["node_modules/**", "test/**", "legacy/**", "quickfix/**"];

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

const ignore = fs.readFileSync(path.join(root, ".vscodeignore"), "utf8").split(/\r?\n/).filter(Boolean);
for (const rule of requiredIgnoreRules) {
  assert.ok(ignore.includes(rule), `missing .vscodeignore rule: ${rule}`);
}

const entries = expectedFiles.map((relativePath) => {
  const absolutePath = path.join(root, relativePath);
  assert.ok(fs.existsSync(absolutePath), `missing package file: ${relativePath}`);
  const bytes = fs.readFileSync(absolutePath);
  return { path: relativePath.replaceAll(path.sep, "/"), bytes: bytes.length, sha256: sha256(bytes) };
});

const forbidden = entries.filter(({ path: relativePath }) =>
  /^(node_modules|test|legacy|quickfix|work|docs|backup|secret)(\/|$)/.test(relativePath),
);
assert.deepStrictEqual(forbidden, []);

const manifest = JSON.stringify({ files: entries, forbiddenFileCount: forbidden.length });
console.log(`PACKAGE_FILE_COUNT=${entries.length}`);
console.log(`PACKAGE_FORBIDDEN_FILE_COUNT=${forbidden.length}`);
console.log(`PACKAGE_MANIFEST_SHA256=${sha256(Buffer.from(manifest, "utf8"))}`);
console.log(manifest);
