// Pure first-flow helpers: which documents are checked/run, the exact run
// command, CLI discovery, and the status shown for each server state.
const assert = require("assert");
const fs = require("fs");
const os = require("os");
const path = require("path");

const extension = require("../extension");

const exe = (base) => (process.platform === "win32" ? `${base}.exe` : base);
const doc = (scheme, fsPath, languageId = "ddonilang") => ({ languageId, uri: { scheme, fsPath } });
const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "ddonilang-first-flow-"));

try {
  assert.strictEqual(extension.documentSurface(doc("file", "C:\\작업\\한글.ddn")), "source");
  assert.strictEqual(extension.documentSurface(doc("file", "C:\\작업\\번역.ddl")), "unanalyzed");
  assert.strictEqual(extension.documentSurface(doc("file", "C:\\작업\\번역.옮김")), "unanalyzed");
  assert.strictEqual(extension.documentSurface(doc("untitled", "Untitled-1")), "unsaved");
  assert.strictEqual(extension.documentSurface(doc("file", "C:\\a.ddn", "plaintext")), "other");
  assert.strictEqual(extension.documentSurface(undefined), "other");

  const file = path.join(tempRoot, "한글 폴더", "😀.ddn");
  assert.deepStrictEqual(extension.buildRunInvocation("C:\\bin\\ddonilang.exe", file, 0), {
    command: "C:\\bin\\ddonilang.exe",
    args: ["실행", file, "--source-profile", "v1-core-v25"],
    cwd: path.dirname(file),
  });
  assert.deepStrictEqual(extension.buildRunInvocation("x", file, 30).args.slice(4), ["--마디수", "30"]);

  // CLI: exact configured path, else beside the server, else bounded discovery.
  const serverDir = path.join(tempRoot, "server-dir");
  fs.mkdirSync(serverDir);
  const server = path.join(serverDir, exe("ddonirang-lsp"));
  const besideServer = path.join(serverDir, exe("ddonilang"));
  fs.writeFileSync(server, "placeholder");
  fs.writeFileSync(besideServer, "placeholder");
  const base = { autoDiscover: true, extensionPath: path.join(tempRoot, "ext"), workspaceFolders: [] };
  assert.strictEqual(extension.resolveCliExecutable({ ...base, configuredPath: "", serverExecutable: server }), besideServer);
  assert.strictEqual(
    extension.resolveCliExecutable({ ...base, configuredPath: path.join(tempRoot, "missing"), serverExecutable: server }),
    null,
  );
  assert.strictEqual(
    extension.resolveCliExecutable({ ...base, autoDiscover: false, configuredPath: "", serverExecutable: server }),
    null,
  );
  const bundled = path.join(tempRoot, "ext", "server", exe("ddonirang-lsp"));
  fs.mkdirSync(path.dirname(bundled), { recursive: true });
  fs.writeFileSync(bundled, "placeholder");
  assert.strictEqual(extension.resolveServerExecutable({ ...base, configuredPath: "" }), bundled);

  // The status never presents a stopped server or an unanalyzed file as checked.
  assert.match(extension.statusView("stopped", "source").text, /중지됨/);
  assert.strictEqual(extension.statusView("stopped", "source").command, "ddonilang.restartLanguageServer");
  assert.match(extension.statusView("running", "unanalyzed").text, /진단 미지원/);
  assert.match(extension.statusView("running", "unsaved").text, /저장 전/);
  const checked = extension.statusView("running", "source");
  assert.match(checked.tooltip, /ddonilang 검사 --source-profile v1-core-v25/);
  assert.match(checked.tooltip, /보장하지는 않습니다/);
  assert.strictEqual(checked.command, "ddonilang.runCurrentFile");

  console.log("first-flow tests: PASS");
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
