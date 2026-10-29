const fs = require("fs");
const path = require("path");
const childProcess = require("child_process");

// The editor checks and runs with the one supported product profile; the
// language server reports the same shared analysis as `ddonilang 검사`.
const SOURCE_PROFILE = "v1-core-v25";

let activeClient = null;
let startupErrorShown = false;
let statusItem = null;
let runChannel = null;
let activeRun = null;

function isLocalAbsolutePath(candidate) {
  if (typeof candidate !== "string" || candidate.length === 0) {
    return false;
  }
  if (!path.isAbsolute(candidate)) {
    return false;
  }
  return process.platform !== "win32" || !candidate.startsWith("\\\\");
}

function isRegularExecutable(candidate) {
  if (!isLocalAbsolutePath(candidate)) {
    return false;
  }
  try {
    const stat = fs.lstatSync(candidate);
    return stat.isFile() && !stat.isSymbolicLink();
  } catch (_) {
    return false;
  }
}

function executableName(base = "ddonirang-lsp") {
  return process.platform === "win32" ? `${base}.exe` : base;
}

function boundedLocalCandidates(extensionPath, workspaceFolders, base = "ddonirang-lsp") {
  const result = [];
  const seen = new Set();
  const add = (candidate) => {
    if (!seen.has(candidate)) {
      seen.add(candidate);
      result.push(candidate);
    }
  };
  // A self-contained extension folder carries its binaries in `server/`.
  add(path.resolve(extensionPath, "server", executableName(base)));

  const roots = [];
  for (const folder of workspaceFolders || []) {
    if (folder && typeof folder.fsPath === "string") {
      roots.push(folder.fsPath);
    }
  }
  roots.push(path.resolve(extensionPath, "..", ".."));
  for (const root of roots) {
    for (const profile of ["debug", "release"]) {
      add(path.resolve(root, "target", profile, executableName(base)));
    }
  }
  return result;
}

function resolveServerExecutable({
  configuredPath,
  autoDiscover,
  extensionPath,
  workspaceFolders,
}) {
  if (typeof configuredPath === "string" && configuredPath.trim() !== "") {
    const exactPath = path.normalize(configuredPath.trim());
    return isRegularExecutable(exactPath) ? exactPath : null;
  }
  if (autoDiscover !== true) {
    return null;
  }
  for (const candidate of boundedLocalCandidates(extensionPath, workspaceFolders)) {
    if (isRegularExecutable(candidate)) {
      return candidate;
    }
  }
  return null;
}

// The run command uses the canonical CLI `ddonilang`: an exact configured
// path, else the binary beside the language server, else the same bounded
// local locations. No PATH search.
function resolveCliExecutable({
  configuredPath,
  autoDiscover,
  serverExecutable,
  extensionPath,
  workspaceFolders,
}) {
  if (typeof configuredPath === "string" && configuredPath.trim() !== "") {
    const exactPath = path.normalize(configuredPath.trim());
    return isRegularExecutable(exactPath) ? exactPath : null;
  }
  if (autoDiscover !== true) {
    return null;
  }
  const candidates = [];
  if (serverExecutable) {
    candidates.push(path.join(path.dirname(serverExecutable), executableName("ddonilang")));
  }
  candidates.push(...boundedLocalCandidates(extensionPath, workspaceFolders, "ddonilang"));
  return candidates.find(isRegularExecutable) || null;
}

function buildRunInvocation(cliExecutable, filePath, madi) {
  const args = ["실행", filePath, "--source-profile", SOURCE_PROFILE];
  if (Number.isInteger(madi) && madi > 0) {
    args.push("--마디수", String(madi));
  }
  return { command: cliExecutable, args, cwd: path.dirname(filePath) };
}

// Only `.ddn` text is DDN source. `.ddl`/`.옮김` share the language id but
// are not analyzed, and an unsaved buffer has no file to run.
function documentSurface(document) {
  if (!document || document.languageId !== "ddonilang") {
    return "other";
  }
  if (document.uri.scheme !== "file") {
    return "unsaved";
  }
  return document.uri.fsPath.endsWith(".ddn") ? "source" : "unanalyzed";
}

function statusView(clientState, surface) {
  if (clientState === "starting") {
    return { text: "$(sync~spin) 또니랑 시작 중", tooltip: "또니랑 언어 서버를 시작하고 있습니다.", command: undefined };
  }
  if (clientState !== "running") {
    return {
      text: "$(error) 또니랑 서버 중지됨",
      tooltip: "또니랑 언어 서버가 멈춰 진단이 갱신되지 않습니다. 눌러서 다시 시작합니다.",
      command: "ddonilang.restartLanguageServer",
    };
  }
  if (surface === "source") {
    return {
      text: "$(check) 또니랑 검사",
      tooltip:
        `편집 중인 내용을 \`ddonilang 검사 --source-profile ${SOURCE_PROFILE}\`와 같은 공통 분석으로 확인합니다. ` +
        "오류가 없어도 실행 결과까지 보장하지는 않습니다. 눌러서 저장하고 실행합니다.",
      command: "ddonilang.runCurrentFile",
    };
  }
  if (surface === "unsaved") {
    return { text: "$(circle-slash) 또니랑: 저장 전 문서", tooltip: "`.ddn` 파일로 저장해야 검사·실행할 수 있습니다.", command: undefined };
  }
  return { text: "$(circle-slash) 또니랑: 진단 미지원", tooltip: "이 파일 형식(.ddl, .옮김)은 아직 진단하지 않습니다.", command: undefined };
}

function documentSelector() {
  return [
    { scheme: "file", language: "ddonilang" },
    { scheme: "untitled", language: "ddonilang" },
  ];
}

function configuration(vscode, section = "ddonilang.lsp") {
  return vscode.workspace.getConfiguration(section);
}

function clientStateName(state) {
  // vscode-languageclient State: Stopped = 1, Running = 2, Starting = 3.
  return state === 2 ? "running" : state === 3 ? "starting" : "stopped";
}

function refreshStatus(vscode) {
  if (!statusItem) {
    return;
  }
  const editor = vscode.window.activeTextEditor;
  const surface = documentSurface(editor && editor.document);
  if (surface === "other") {
    statusItem.hide();
    return;
  }
  const view = statusView(activeClient ? clientStateName(activeClient.state) : "stopped", surface);
  statusItem.text = view.text;
  statusItem.tooltip = view.tooltip;
  statusItem.command = view.command;
  statusItem.show();
}

function serverExecutable(vscode, context) {
  const settings = configuration(vscode);
  return resolveServerExecutable({
    configuredPath: settings.get("executable", ""),
    autoDiscover: settings.get("autoDiscover", true),
    extensionPath: context.extensionPath,
    workspaceFolders: vscode.workspace.workspaceFolders,
  });
}

// Read-only view of what the user currently sees, for diagnostics and tests.
function statusSnapshot() {
  return {
    clientState: activeClient ? clientStateName(activeClient.state) : "stopped",
    serverPid: activeClient && activeClient._serverProcess ? activeClient._serverProcess.pid : null,
    text: statusItem ? statusItem.text : null,
    tooltip: statusItem ? statusItem.tooltip : null,
    command: statusItem ? statusItem.command : null,
  };
}

async function restartLanguageServer(vscode) {
  if (!activeClient) {
    return false;
  }
  if (activeClient.isRunning()) {
    await activeClient.stop();
  }
  await activeClient.start();
  refreshStatus(vscode);
  return true;
}

// Saves the active `.ddn` editor, then runs the saved file with the CLI. The
// run sees exactly the saved bytes; it never runs unsaved editor text.
async function runCurrentFile(vscode, context) {
  const editor = vscode.window.activeTextEditor;
  const document = editor && editor.document;
  if (documentSurface(document) !== "source") {
    void vscode.window.showWarningMessage("또니랑 실행: 활성 편집기가 저장된 .ddn 파일이 아닙니다.");
    return { status: "rejected", reason: "not-a-saved-ddn-file" };
  }
  if (document.isDirty && !(await document.save())) {
    void vscode.window.showErrorMessage("또니랑 실행: 저장하지 못해 실행하지 않았습니다.");
    return { status: "rejected", reason: "save-failed" };
  }
  const cliSettings = configuration(vscode, "ddonilang.cli");
  const cli = resolveCliExecutable({
    configuredPath: cliSettings.get("executable", ""),
    autoDiscover: configuration(vscode).get("autoDiscover", true),
    serverExecutable: serverExecutable(vscode, context),
    extensionPath: context.extensionPath,
    workspaceFolders: vscode.workspace.workspaceFolders,
  });
  if (!cli) {
    void vscode.window.showErrorMessage(
      "또니랑 실행: ddonilang 실행 파일을 찾지 못했습니다. ddonilang.cli.executable에 로컬 경로를 지정하세요.",
    );
    return { status: "rejected", reason: "cli-not-found" };
  }
  const madi = configuration(vscode, "ddonilang.run").get("madi", 0);
  const invocation = buildRunInvocation(cli, document.uri.fsPath, madi);
  if (activeRun) {
    activeRun.kill();
  }
  runChannel.clear();
  runChannel.show(true);
  runChannel.appendLine(`▶ ${path.basename(invocation.args[1])} 실행 (저장본, 편집기 판 ${document.version})`);
  return new Promise((resolve) => {
    let stdout = "";
    let stderr = "";
    const child = childProcess.spawn(invocation.command, invocation.args, { cwd: invocation.cwd, windowsHide: true });
    activeRun = child;
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString("utf8");
      runChannel.append(chunk.toString("utf8"));
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString("utf8");
      runChannel.append(chunk.toString("utf8"));
    });
    const finish = (exitCode, error) => {
      if (activeRun === child) {
        activeRun = null;
      }
      runChannel.appendLine(error ? `■ 실행 실패: ${error.message}` : `■ 종료 코드 ${exitCode}`);
      resolve({ status: error ? "failed" : "finished", exitCode, stdout, stderr, invocation, savedVersion: document.version });
    };
    child.on("error", (error) => finish(null, error));
    child.on("close", (code) => finish(code, null));
  });
}

async function activate(context) {
  if (activeClient) {
    return activeClient;
  }

  const vscode = require("vscode");
  const { LanguageClient } = require("vscode-languageclient/node");
  const executable = serverExecutable(vscode, context);

  if (vscode.window.createStatusBarItem && !statusItem) {
    statusItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    runChannel = vscode.window.createOutputChannel("또니랑 실행");
    context.subscriptions.push(
      statusItem,
      runChannel,
      vscode.commands.registerCommand("ddonilang.restartLanguageServer", () => restartLanguageServer(vscode)),
      vscode.commands.registerCommand("ddonilang.runCurrentFile", () => runCurrentFile(vscode, context)),
      vscode.commands.registerCommand("ddonilang.statusSnapshot", () => statusSnapshot()),
      vscode.window.onDidChangeActiveTextEditor(() => refreshStatus(vscode)),
    );
  }

  if (!executable) {
    if (!startupErrorShown) {
      startupErrorShown = true;
      void vscode.window.showErrorMessage(
        "Ddonilang LSP를 시작할 수 없습니다. ddonilang.lsp.executable에 로컬 실행 파일 경로를 지정하거나 autoDiscover를 확인하세요.",
      );
    }
    refreshStatus(vscode);
    return null;
  }

  const serverOptions = {
    command: executable,
    args: [],
    options: { cwd: path.dirname(executable) },
  };
  const clientOptions = {
    documentSelector: documentSelector(),
    synchronize: { configurationSection: "ddonilang.lsp" },
    outputChannelName: "또니랑 언어 서버",
  };

  activeClient = new LanguageClient(
    "ddonilangLsp",
    "Ddonilang Language Server",
    serverOptions,
    clientOptions,
  );
  context.subscriptions.push(activeClient);
  if (activeClient.onDidChangeState) {
    context.subscriptions.push(activeClient.onDidChangeState(() => refreshStatus(vscode)));
  }

  try {
    await activeClient.start();
    refreshStatus(vscode);
    return activeClient;
  } catch (error) {
    const failedClient = activeClient;
    activeClient = null;
    failedClient.dispose();
    refreshStatus(vscode);
    if (!startupErrorShown) {
      startupErrorShown = true;
      void vscode.window.showErrorMessage(
        `Ddonilang LSP 시작에 실패했습니다: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
    return null;
  }
}

async function deactivate() {
  if (activeRun) {
    activeRun.kill();
    activeRun = null;
  }
  if (!activeClient) {
    return undefined;
  }
  const client = activeClient;
  activeClient = null;
  return client.stop();
}

module.exports = {
  activate,
  deactivate,
  boundedLocalCandidates,
  buildRunInvocation,
  documentSelector,
  documentSurface,
  isRegularExecutable,
  resolveCliExecutable,
  resolveServerExecutable,
  statusSnapshot,
  statusView,
};
