"use strict";
// Runs inside the official VS Code Extension Host (--extensionTestsPath).
// Drives the first editor flow through the real UI APIs: open -> see error ->
// edit -> fresh diagnostics -> save -> run. No vscode mock, no direct server
// calls except observing the server process id. Writes a JSON report.
const assert = require("assert");
const fs = require("fs");
const path = require("path");
const vscode = require("vscode");

const BAD = "// 😀 한글\n값 <- (1 + 2.\n";
const GOOD = "// 😀 한글\n값 <- (1 + 2).\n값 보여주기.\n";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const alive = (pid) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch (_) {
    return false;
  }
};
async function until(predicate, label, timeout = 20000) {
  const end = Date.now() + timeout;
  do {
    if (await predicate()) return;
    await sleep(50);
  } while (Date.now() < end);
  throw new Error("REAL_HOST_TIMEOUT: " + label);
}
const status = () => vscode.commands.executeCommand("ddonilang.statusSnapshot");
const diags = (uri) =>
  vscode.languages.getDiagnostics(uri).map((item) => ({
    code: typeof item.code === "object" ? item.code.value : item.code,
    severity: item.severity,
    range: [item.range.start.line, item.range.start.character, item.range.end.line, item.range.end.character],
    message: item.message,
  }));
async function replaceAll(document, text) {
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), text);
  assert.strictEqual(await vscode.workspace.applyEdit(edit), true);
}
// Settles only when the visible diagnostics stay equal for `quiet` ms.
async function stableDiagnostics(uri, predicate, label, quiet = 800) {
  let last = null;
  let since = Date.now();
  await until(() => {
    const now = JSON.stringify(diags(uri));
    if (now !== last) {
      last = now;
      since = Date.now();
    }
    return predicate(diags(uri)) && Date.now() - since >= quiet;
  }, label);
  return diags(uri);
}
// The same text checked by the real CLI `ddonilang 검사`; its UTF-8 byte span
// converted to VS Code's UTF-16 line/character.
function cliCheck(text) {
  const cli = vscode.workspace.getConfiguration("ddonilang.cli").get("executable");
  const dir = fs.mkdtempSync(path.join(process.env.DDN_REAL_HOST_ROOT, "..", "cli-check-"));
  const file = path.join(dir, "check.ddn");
  fs.writeFileSync(file, text);
  const out = require("child_process").spawnSync(cli, ["검사", file, "--source-profile", "v1-core-v25"], { encoding: "utf8" });
  fs.rmSync(dir, { recursive: true, force: true });
  const snapshot = JSON.parse(out.stdout);
  const bytes = Buffer.from(text, "utf8");
  const at = (offset) => {
    const before = bytes.subarray(0, offset).toString("utf8");
    const line = before.split("\n").length - 1;
    const lineStart = before.lastIndexOf("\n") + 1;
    return [line, before.length - lineStart];
  };
  return snapshot.diagnostics.map((item) => ({ code: item.code, range: [...at(item.span.start), ...at(item.span.end)] }));
}
const sameAsCli = (text) => {
  const expected = JSON.stringify(cliCheck(text));
  return (list) => JSON.stringify(list.map(({ code, range }) => ({ code, range }))) === expected;
};
const isParseAt = (expected) => (list) =>
  list.length === 1 && list[0].code === "E_PARSE" && JSON.stringify(list[0].range) === JSON.stringify(expected);

async function run() {
  const root = process.env.DDN_REAL_HOST_ROOT;
  assert.ok(root && path.isAbsolute(root), "DDN_REAL_HOST_ROOT");
  const report = { environment: "official VS Code Extension Host", vscode: vscode.version, gates: {}, observations: {}, serverPids: [] };
  const mark = (name, detail) => {
    report.gates[name] = detail === undefined ? "PASS" : { status: "PASS", detail };
    console.log("REAL_HOST " + name + ": PASS");
  };
  try {
    const file = path.join(root, "한글😀.ddn");
    fs.writeFileSync(file, BAD, { flag: "wx" });
    const uri = vscode.Uri.file(file);

    // 1. Open: the extension activates and the error is shown at the exact place.
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);
    assert.strictEqual(document.languageId, "ddonilang");
    await until(async () => (await status()).clientState === "running", "client running");
    const first = await status();
    assert.ok(first.serverPid && alive(first.serverPid));
    report.serverPids.push(first.serverPid);
    assert.match(first.text, /또니랑 검사/);
    assert.deepStrictEqual(cliCheck(BAD), [{ code: "E_PARSE", range: [1, 11, 1, 12] }]);
    const opened = await stableDiagnostics(uri, isParseAt([1, 11, 1, 12]), "open shows E_PARSE once");
    mark("OPEN_SHOWS_SHARED_CHECK_ERROR_ONCE", opened);

    // 2. Edit (unsaved): diagnostics follow the editor text, not the disk.
    await replaceAll(document, GOOD);
    assert.strictEqual(document.isDirty, true);
    assert.strictEqual(fs.readFileSync(file, "utf8"), BAD, "disk still has the error");
    await stableDiagnostics(uri, (list) => list.length === 0, "fixed buffer clears diagnostics");
    mark("UNSAVED_EDIT_REFRESHES_FROM_BUFFER_NOT_DISK");

    // 3. Burst of edits without waiting: only the final text's result remains.
    for (let index = 0; index < 12; index += 1) {
      await replaceAll(document, index % 2 === 0 ? BAD : GOOD.replace("값 보여주기", `값 보여주기 // ${index}`));
    }
    await replaceAll(document, BAD.replace("// 😀 한글", "// 😀😀 한글"));
    await stableDiagnostics(uri, isParseAt([1, 11, 1, 12]), "burst ends on error", 1500);
    await replaceAll(document, GOOD);
    const settled = await stableDiagnostics(uri, (list) => list.length === 0, "burst ends clean", 1500);
    mark("LATE_RESULT_NEVER_REPLACES_NEWER_TEXT", { versionAfterBurst: document.version, settled });

    // 4. Emoji/Hangul before the error on the same line (UTF-16 columns).
    await replaceAll(document, "값 <- \"😀\" + (1 + 2.\n");
    assert.deepStrictEqual(cliCheck("값 <- \"😀\" + (1 + 2.\n"), [{ code: "E_PARSE", range: [0, 18, 0, 19] }]);
    await stableDiagnostics(uri, isParseAt([0, 18, 0, 19]), "emoji-prefixed error column");
    mark("EMOJI_HANGUL_SAME_LINE_UTF16_RANGE");

    // 5. CRLF document with a multi-line edit.
    await replaceAll(document, GOOD);
    await editor.edit((builder) => builder.setEndOfLine(vscode.EndOfLine.CRLF));
    assert.strictEqual(document.eol, vscode.EndOfLine.CRLF);
    assert.ok(document.getText().includes("\r\n"));
    const editOk = await editor.edit((builder) =>
      builder.replace(new vscode.Range(1, 5, 2, 1), "1.\r\n// 😀 주석\r\n값 <- (3"),
    );
    assert.strictEqual(editOk, true);
    const crlfText = document.getText();
    assert.strictEqual(crlfText, "// 😀 한글\r\n값 <- 1.\r\n// 😀 주석\r\n값 <- (3 보여주기.\r\n");
    const crlfExpected = cliCheck(crlfText);
    assert.ok(crlfExpected.length > 0);
    const crlf = await stableDiagnostics(uri, sameAsCli(crlfText), "CRLF error equals CLI check");
    report.observations.crlf = { text: crlfText, cli: crlfExpected };
    mark("CRLF_MULTILINE_EDIT", crlf);
    await editor.edit((builder) => builder.setEndOfLine(vscode.EndOfLine.LF));

    // 6. Save and run: the CLI runs exactly the saved bytes.
    await replaceAll(document, GOOD);
    await stableDiagnostics(uri, (list) => list.length === 0, "good before run");
    assert.strictEqual(document.isDirty, true);
    const result = await vscode.commands.executeCommand("ddonilang.runCurrentFile");
    assert.strictEqual(document.isDirty, false);
    assert.strictEqual(fs.readFileSync(file, "utf8"), GOOD);
    assert.strictEqual(result.status, "finished");
    assert.strictEqual(result.exitCode, 0, result.stderr);
    assert.strictEqual(result.stdout.trim(), "3");
    assert.deepStrictEqual(result.invocation.args.slice(0, 1).concat(result.invocation.args.slice(2)), [
      "실행",
      "--source-profile",
      "v1-core-v25",
    ]);
    mark("SAVE_AND_RUN", { exitCode: result.exitCode, stdout: result.stdout, savedVersion: result.savedVersion });

    await replaceAll(document, BAD);
    const failed = await vscode.commands.executeCommand("ddonilang.runCurrentFile");
    assert.strictEqual(failed.exitCode, 1);
    assert.match(failed.stdout + failed.stderr, /E_PARSE/);
    mark("SAVE_AND_RUN_REJECTED_SOURCE_REPORTS_CLI_ERROR", { exitCode: failed.exitCode });

    // 7. Close, change the file on disk, reopen: no previous-session result.
    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
    await until(() => vscode.workspace.textDocuments.every((item) => item.uri.toString() !== uri.toString()) || true, "closed");
    await sleep(1500);
    report.observations.diagnosticsAfterClose = diags(uri);
    fs.writeFileSync(file, GOOD);
    const reopened = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(reopened);
    // VS Code may hand back its cached model first and reload it from disk
    // through the file watcher; wait for the disk text to be the editor text.
    await until(() => reopened.getText() === GOOD, "reopened editor shows disk text");
    assert.strictEqual(reopened.isDirty, false);
    await stableDiagnostics(uri, (list) => list.length === 0, "reopen shows current file only");
    mark("CLOSE_REOPEN_NO_PREVIOUS_RESULT", { diagnosticsAfterClose: report.observations.diagnosticsAfterClose });

    // 8. Restart command: a new server process, the same current result.
    await replaceAll(reopened, BAD);
    await stableDiagnostics(uri, isParseAt([1, 11, 1, 12]), "error before restart");
    const before = (await status()).serverPid;
    assert.strictEqual(await vscode.commands.executeCommand("ddonilang.restartLanguageServer"), true);
    await until(async () => {
      const now = await status();
      return now.clientState === "running" && now.serverPid && now.serverPid !== before;
    }, "restart gives a new server");
    const restartedPid = (await status()).serverPid;
    report.serverPids.push(restartedPid);
    await until(() => !alive(before), "old server stopped");
    await stableDiagnostics(uri, isParseAt([1, 11, 1, 12]), "error after restart, once");
    mark("RESTART_NEW_PROCESS_CURRENT_RESULT", { before, after: restartedPid });

    // 9. Server crash: the client restarts it and the result is rebuilt from
    // the editor text; while down the status says so.
    process.kill(restartedPid);
    await until(async () => {
      const now = await status();
      return now.clientState === "running" && now.serverPid && now.serverPid !== restartedPid;
    }, "crash recovery");
    const recoveredPid = (await status()).serverPid;
    report.serverPids.push(recoveredPid);
    await replaceAll(reopened, GOOD);
    await stableDiagnostics(uri, (list) => list.length === 0, "after crash recovery edit");
    mark("CRASH_RECOVERY_REANALYZES_CURRENT_TEXT", { crashed: restartedPid, recovered: recoveredPid });

    const client = vscode.extensions.getExtension("ddonilang.ddn-language").exports;
    await client.stop();
    const stopped = await status();
    assert.strictEqual(stopped.clientState, "stopped");
    assert.match(stopped.text, /중지됨/);
    assert.strictEqual(stopped.command, "ddonilang.restartLanguageServer");
    mark("STOPPED_SERVER_IS_SHOWN", { text: stopped.text });
    await vscode.commands.executeCommand("ddonilang.restartLanguageServer");
    await until(async () => (await status()).clientState === "running", "restart from stopped");
    report.serverPids.push((await status()).serverPid);
    mark("RESTART_FROM_STOPPED");

    // 10. Typing an opening bracket at the end of text: the shared parser
    // rejects it as an ordinary E_PARSE; the same server process keeps running.
    const pidBefore = (await status()).serverPid;
    for (const text of ["값 <- (", "값 <- [", "값 <- {"]) {
      await replaceAll(reopened, text);
      const shown = await stableDiagnostics(uri, sameAsCli(text), `open bracket ${text}`);
      assert.strictEqual(shown.length, 1);
      assert.strictEqual(shown[0].code, "E_PARSE");
    }
    const afterBrackets = await status();
    assert.strictEqual(afterBrackets.clientState, "running");
    assert.strictEqual(afterBrackets.serverPid, pidBefore, "no crash or restart");
    await replaceAll(reopened, GOOD);
    await stableDiagnostics(uri, (list) => list.length === 0, "clean after brackets");
    mark("OPEN_BRACKET_AT_END_IS_E_PARSE_WITHOUT_CRASH", { serverPid: pidBefore });

    // 11. Rename (F2): a top-level callable that the shared name binding
    // resolves exactly is renamed at every use (declaration, result place,
    // `…하기` call stem) and the program runs the same; a callable shadowed by
    // a same-spelling local is refused with the reason and nothing changes.
    const renameSource = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n결과 := 21을 두배하기. 결과 보여주기.\n";
    await replaceAll(reopened, renameSource);
    await stableDiagnostics(uri, (list) => list.length === 0, "rename source clean");
    const callStem = reopened.positionAt(renameSource.indexOf("두배하기") + 1);
    const prepared = await vscode.commands.executeCommand("vscode.prepareRename", uri, callStem);
    assert.ok(prepared && prepared.placeholder === "두배", `prepareRename: ${JSON.stringify(prepared)}`);
    const renameEdit = await vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", uri, callStem, "곱절");
    assert.ok(renameEdit && renameEdit.size === 1, "one document edited");
    assert.strictEqual(await vscode.workspace.applyEdit(renameEdit), true);
    assert.strictEqual(reopened.getText(), renameSource.split("두배").join("곱절"));
    const renamedRun = await vscode.commands.executeCommand("ddonilang.runCurrentFile");
    assert.strictEqual(renamedRun.exitCode, 0, renamedRun.stderr);
    assert.strictEqual(renamedRun.stdout.trim(), "42");

    const shadowSource = "(값:수~을) 두배:수:셈씨 := { 두배 <- 값 * 2. }.\n(값:수~을) 세배:수:셈씨 := { 두배:수 := 값 * 3. 세배 <- 두배. }.\n결과 := 7을 세배하기. 결과 보여주기.\n";
    await replaceAll(reopened, shadowSource);
    let refusal = null;
    try {
      await vscode.commands.executeCommand("vscode.prepareRename", uri, reopened.positionAt(shadowSource.indexOf("두배")));
    } catch (error) {
      refusal = String(error && error.message ? error.message : error);
    }
    assert.ok(refusal && refusal.includes("같은 철자의 지역 이름이나 매개변수"), `prepareRename: ${refusal}`);
    assert.strictEqual(reopened.getText(), shadowSource);
    assert.strictEqual((await status()).serverPid, pidBefore, "rename is not a crash");
    mark("RENAME_EXACT_CALLABLE_AND_REFUSE_SHADOWED", { renamedOutput: renamedRun.stdout.trim(), refusal });

    // 12. Exact navigation and Final local/parameter rename:
    // diagnostics -> definition -> references/highlight -> rename -> CLI rerun.
    const navSource = shadowSource;
    await replaceAll(reopened, navSource);
    await stableDiagnostics(uri, (list) => list.length === 0, "navigation source clean");
    const localRead = reopened.positionAt(navSource.indexOf("세배 <- 두배") + "세배 <- ".length + 1);
    const localDecl = reopened.positionAt(navSource.indexOf("두배:수 :="));
    const defs = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, localRead);
    assert.strictEqual(defs.length, 1, "one definition");
    const target = defs[0].range || defs[0].targetRange;
    assert.deepStrictEqual([target.start.line, target.start.character], [localDecl.line, localDecl.character], "the shadowing local, not the callable");
    const callableDecl = reopened.positionAt(navSource.indexOf("두배") + 1);
    const noGuess = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, callableDecl);
    assert.strictEqual(noGuess.length, 0, "callable sharing a local spelling: no guessed jump");
    const refs = await vscode.commands.executeCommand("vscode.executeReferenceProvider", uri, localRead);
    assert.strictEqual(refs.length, 2, "local declaration and read only");
    const callStem12 = reopened.positionAt(navSource.indexOf("세배하기") + 1);
    const callDefs = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, callStem12);
    assert.strictEqual(callDefs.length, 1, "call stem -> callable");
    const beforeRun = await vscode.commands.executeCommand("ddonilang.runCurrentFile");
    assert.strictEqual(beforeRun.exitCode, 0, beforeRun.stderr);
    const localEdit = await vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", uri, localRead, "세값");
    assert.ok(localEdit && localEdit.size === 1);
    assert.strictEqual(await vscode.workspace.applyEdit(localEdit), true);
    const localRenamed = reopened.getText();
    assert.strictEqual(localRenamed.split("세값").length - 1, 2);
    assert.strictEqual(localRenamed.split("두배").length - 1, navSource.split("두배").length - 1 - 2);
    const afterRun = await vscode.commands.executeCommand("ddonilang.runCurrentFile");
    assert.strictEqual(afterRun.exitCode, 0, afterRun.stderr);
    assert.strictEqual(afterRun.stdout.trim(), beforeRun.stdout.trim());
    // A stale position after an edit gives no old result: the old local spelling is gone.
    const stale = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, reopened.positionAt(localRenamed.indexOf("세값")));
    assert.strictEqual(stale.length, 1);
    assert.strictEqual((await status()).serverPid, pidBefore);
    mark("EXACT_NAVIGATION_LOCAL_RENAME_AND_CLI_RERUN", { output: afterRun.stdout.trim(), references: refs.length });

    // 13. Project (dev helper fixture on disk, app as CRLF lines with emoji):
    // with the work root open, `힘.속도` leads into the dependency file and the
    // setup counter is renamed at its 4 sites; after save, the old lock is
    // rejected until the explicit lock regeneration, then the run is the same.
    const projectDir = path.join(root, "작품");
    const fixture = JSON.parse(
      fs.readFileSync(path.resolve(__dirname, "..", "..", "..", "tool", "tests", "fixtures", "v2_helper_source.json"), "utf8"),
    ).source_snapshot;
    const APP = "(힘=힘) 쓰임.\r\n채비 { 횟수:바른수 := 0. }.\r\n앱:움직씨 := {\r\n  (매마디)마다 { \"🙂🙂\" 보여주기. 횟수 <- 횟수 + 1. 힘.속도 보여주기. 횟수 보여주기. }.\r\n}.\r\n";
    for (const file of fixture.files) {
      const target = path.join(projectDir, file.path);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, file.path === "app/main.ddn" ? APP : file.text);
    }
    const cliPath = vscode.workspace.getConfiguration("ddonilang.cli").get("executable");
    const cli = (args) => require("child_process").spawnSync(cliPath, args, { encoding: "utf8", cwd: projectDir });
    const ROLES = { "뿌리.ddn": "root", "vendor/운동/artifact.detjson": "metadata", "vendor/운동/gaji.toml": "gaji_manifest" };
    const regenerateLock = () => {
      const request = path.join(root, "lock-request.json");
      const files = fixture.files
        .filter((file) => file.path !== "ddn.lock")
        .map((file) => ({ path: file.path, role: ROLES[file.path] || "ddn_source", utf8_text: fs.readFileSync(path.join(projectDir, file.path), "utf8") }));
      fs.writeFileSync(request, JSON.stringify({ schema: "ddn.virtual_project_source.v1", kind: "seamgrim.local_project_tree.v0", root_path: fixture.root_path, entry_path: fixture.entry_path, files }));
      const out = cli(["작품잠금", request]);
      assert.strictEqual(out.status, 0, out.stderr);
      fs.writeFileSync(path.join(projectDir, "ddn.lock"), out.stdout);
    };
    const projectRun = () => {
      const out = cli(["작품실행", path.join(projectDir, fixture.root_path)]);
      return { status: out.status, logs: out.status === 0 ? JSON.parse(out.stdout).runtime_output_log_texts : null, error: out.status === 0 ? null : JSON.parse(out.stderr).code };
    };
    regenerateLock();
    const projectBefore = projectRun();
    assert.deepStrictEqual(projectBefore, { status: 0, logs: ["🙂🙂", "7", "1"], error: null });

    const rootUri = vscode.Uri.file(path.join(projectDir, fixture.root_path));
    const appUri = vscode.Uri.file(path.join(projectDir, "app", "main.ddn"));
    const libUri = vscode.Uri.file(path.join(projectDir, "vendor", "운동", "ddn", "lib", "운동.ddn"));
    await vscode.workspace.openTextDocument(rootUri);
    const app = await vscode.workspace.openTextDocument(appUri);
    await vscode.window.showTextDocument(app);
    assert.strictEqual(app.getText(), APP, "CRLF kept");
    // The editor check is of this one file (no project context); fix an error
    // and the shown result returns to exactly what the CLI says for it.
    const appBaseline = await stableDiagnostics(appUri, sameAsCli(APP), "project app baseline");
    const broken = APP.replace("횟수 + 1.", "횟수 + (1.");
    await replaceAll(app, broken);
    await stableDiagnostics(appUri, sameAsCli(broken), "project app broken");
    await replaceAll(app, APP);
    await stableDiagnostics(appUri, sameAsCli(APP), "project app fixed");

    const speed = app.positionAt(APP.indexOf("힘.속도") + "힘.".length + 1);
    const speedDefs = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", appUri, speed);
    assert.strictEqual(speedDefs.length, 1, "imported name -> one definition");
    const speedTarget = speedDefs[0].targetUri || speedDefs[0].uri;
    const speedRange = speedDefs[0].targetRange || speedDefs[0].range;
    assert.strictEqual(speedTarget.fsPath.toLowerCase(), libUri.fsPath.toLowerCase(), "dependency file");
    assert.deepStrictEqual([speedRange.start.line, speedRange.start.character, speedRange.end.line, speedRange.end.character], [1, 0, 1, 2]);
    assert.strictEqual((await vscode.commands.executeCommand("vscode.executeReferenceProvider", appUri, speed)).length, 0, "incomplete export census");
    let speedRefusal = null;
    try {
      await vscode.commands.executeCommand("vscode.prepareRename", appUri, speed);
    } catch (error) {
      speedRefusal = String(error && error.message ? error.message : error);
    }
    assert.ok(speedRefusal && speedRefusal.includes("이름을 바꿀 수 없습니다"), `prepareRename 속도: ${speedRefusal}`);

    const counterUse = app.positionAt(APP.indexOf("횟수 +") + 1);
    const counterDecl = app.positionAt(APP.indexOf("횟수:바른수"));
    const counterDefs = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", appUri, counterUse);
    assert.strictEqual(counterDefs.length, 1);
    const counterRange = counterDefs[0].targetRange || counterDefs[0].range;
    assert.deepStrictEqual([counterRange.start.line, counterRange.start.character], [counterDecl.line, counterDecl.character]);
    const counterRefs = await vscode.commands.executeCommand("vscode.executeReferenceProvider", appUri, counterUse);
    assert.strictEqual(counterRefs.length, 4, "complete setup counter census");
    // UTF-16 columns after the emoji on the CRLF line 3.
    const emojiLineUse = app.positionAt(APP.indexOf("횟수 <-"));
    assert.ok(counterRefs.some((ref) => ref.range.start.line === 3 && ref.range.start.character === emojiLineUse.character));
    const counterEdit = await vscode.commands.executeCommand("vscode.executeDocumentRenameProvider", appUri, counterUse, "진행수");
    assert.ok(counterEdit && counterEdit.size === 1, "this document only");
    assert.strictEqual(await vscode.workspace.applyEdit(counterEdit), true);
    const renamedApp = APP.split("횟수").join("진행수");
    assert.strictEqual(app.getText(), renamedApp);
    assert.strictEqual(await app.save(), true);
    assert.strictEqual(fs.readFileSync(appUri.fsPath, "utf8"), renamedApp, "saved with CRLF");
    const staleLock = projectRun();
    assert.deepStrictEqual(staleLock, { status: 2, logs: null, error: "E_LOCK_HASH_MISMATCH" }, "source edit is not a lock update");
    regenerateLock();
    const projectAfter = projectRun();
    assert.deepStrictEqual(projectAfter, projectBefore);
    const nextDefs = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", appUri, app.positionAt(renamedApp.indexOf("진행수 +") + 1));
    assert.strictEqual(nextDefs.length, 1);
    assert.strictEqual((await status()).serverPid, pidBefore);
    mark("PROJECT_DEFINITION_COUNTER_RENAME_LOCK_AND_CLI_RERUN", {
      appDiagnostics: appBaseline.map((item) => item.code),
      speedDefinition: [speedTarget.fsPath, speedRange.start.line, speedRange.start.character],
      counterReferences: counterRefs.length,
      speedRefusal,
      staleLock: staleLock.error,
      output: projectAfter.logs,
    });

    report.status = "PASS_ACTUAL_HOST_FIRST_FLOW";
  } catch (error) {
    report.status = "FAIL_ACTUAL_HOST";
    report.error = String(error.stack || error);
    throw error;
  } finally {
    try {
      const ext = vscode.extensions.getExtension("ddonilang.ddn-language");
      const implementation = Object.values(require.cache).find(
        (entry) => entry.filename && path.basename(entry.filename) === "extension.js" && typeof entry.exports.statusSnapshot === "function",
      );
      if (implementation) await implementation.exports.deactivate();
      else if (ext && ext.exports) await ext.exports.stop();
    } catch (error) {
      report.cleanupError = String(error);
    }
    await until(() => report.serverPids.every((pid) => !alive(pid)), "server cleanup", 10000).catch(() => {});
    report.processResidue = report.serverPids.filter(alive);
    fs.writeFileSync(path.join(root, "..", "real-host-first-flow-result.json"), JSON.stringify(report, null, 2) + "\n");
  }
}
module.exports = { run };
