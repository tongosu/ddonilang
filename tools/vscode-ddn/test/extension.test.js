const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");

const extension = require("../extension");
const manifest = require("../package.json");

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ddonilang-vscode-"));

try {
  assert.strictEqual(manifest.main, "./extension.js");
  assert.deepStrictEqual(manifest.activationEvents, [
    "onLanguage:ddonilang",
    "onCommand:ddonilang.runCurrentFile",
    "onCommand:ddonilang.restartLanguageServer",
  ]);
  assert.strictEqual(manifest.dependencies["vscode-languageclient"], "8.1.0");
  assert.deepStrictEqual(manifest.contributes.languages[0].extensions, [".ddn", ".ddl", ".옮김"]);
  assert.deepStrictEqual(extension.documentSelector(), [
    { scheme: "file", language: "ddonilang" },
    { scheme: "untitled", language: "ddonilang" },
  ]);

  const workspaceRoot = path.join(tempRoot, "workspace");
  const debugDir = path.join(workspaceRoot, "target", "debug");
  fs.mkdirSync(debugDir, { recursive: true });
  const discovered = path.join(debugDir, process.platform === "win32" ? "ddonirang-lsp.exe" : "ddonirang-lsp");
  fs.writeFileSync(discovered, "placeholder");

  assert.strictEqual(
    extension.resolveServerExecutable({
      configuredPath: "",
      autoDiscover: true,
      extensionPath: path.join(workspaceRoot, "tools", "vscode-ddn"),
      workspaceFolders: [{ fsPath: workspaceRoot }],
    }),
    discovered,
  );
  assert.strictEqual(
    extension.resolveServerExecutable({
      configuredPath: path.join(workspaceRoot, "missing-lsp"),
      autoDiscover: true,
      extensionPath: path.join(workspaceRoot, "tools", "vscode-ddn"),
      workspaceFolders: [{ fsPath: workspaceRoot }],
    }),
    null,
  );
  assert.strictEqual(
    extension.resolveServerExecutable({
      configuredPath: "relative-lsp",
      autoDiscover: true,
      extensionPath: path.join(workspaceRoot, "tools", "vscode-ddn"),
      workspaceFolders: [{ fsPath: workspaceRoot }],
    }),
    null,
  );
  assert.strictEqual(
    extension.resolveServerExecutable({
      configuredPath: "",
      autoDiscover: false,
      extensionPath: path.join(workspaceRoot, "tools", "vscode-ddn"),
      workspaceFolders: [{ fsPath: workspaceRoot }],
    }),
    null,
  );

  const directory = path.join(tempRoot, "directory");
  fs.mkdirSync(directory);
  assert.strictEqual(extension.isRegularExecutable(directory), false);

  const symlink = path.join(tempRoot, "symlink");
  try {
    fs.symlinkSync(discovered, symlink, "file");
    assert.strictEqual(extension.isRegularExecutable(symlink), false);
  } catch (error) {
    if (error.code !== "EPERM" && error.code !== "EACCES") {
      throw error;
    }
  }

  console.log("extension tests: PASS");
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
