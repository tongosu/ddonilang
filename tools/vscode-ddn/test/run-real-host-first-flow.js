// Launches the installed official VS Code with an isolated, throwaway profile
// and runs test/real-host-first-flow.test.js inside its Extension Host.
//   node test/run-real-host-first-flow.js --code <Code.exe> --lsp <ddonirang-lsp> --cli <ddonilang> --out <dir> [--extension <dir>]
// Nothing is downloaded; the user's normal profile and extensions are untouched.
const assert = require("assert");
const childProcess = require("child_process");
const fs = require("fs");
const path = require("path");

function option(name) {
  const index = process.argv.indexOf(`--${name}`);
  assert.ok(index > 0 && process.argv[index + 1], `--${name} is required`);
  return path.resolve(process.argv[index + 1]);
}

const code = option("code");
const lsp = option("lsp");
const cli = option("cli");
const out = option("out");
for (const file of [code, lsp, cli]) {
  assert.ok(fs.statSync(file).isFile(), `not a file: ${file}`);
}
const runRoot = path.join(out, `host-${Date.now()}`);
const userData = path.join(runRoot, "user-data");
const extensionsDir = path.join(runRoot, "extensions");
const workspace = path.join(runRoot, "workspace");
for (const dir of [path.join(userData, "User"), extensionsDir, workspace]) {
  fs.mkdirSync(dir, { recursive: true });
}
fs.writeFileSync(
  path.join(userData, "User", "settings.json"),
  JSON.stringify(
    {
      "ddonilang.lsp.executable": lsp,
      "ddonilang.cli.executable": cli,
      "security.workspace.trust.enabled": false,
      "update.mode": "none",
      "extensions.autoCheckUpdates": false,
      "extensions.autoUpdate": false,
      "telemetry.telemetryLevel": "off",
      "files.eol": "\n",
    },
    null,
    2,
  ),
);

// --extension <dir> runs an assembled candidate folder instead of this one.
const extensionIndex = process.argv.indexOf("--extension");
const extensionRoot =
  extensionIndex > 0 ? path.resolve(process.argv[extensionIndex + 1]) : path.resolve(__dirname, "..");
const args = [
  `--extensionDevelopmentPath=${extensionRoot}`,
  `--extensionTestsPath=${path.join(__dirname, "real-host-first-flow.test.js")}`,
  `--user-data-dir=${userData}`,
  `--extensions-dir=${extensionsDir}`,
  "--disable-extensions",
  "--disable-workspace-trust",
  "--skip-welcome",
  "--skip-release-notes",
  "--new-window",
  workspace,
];
console.log(`REAL_HOST_RUN_ROOT=${runRoot}`);
// A shell started from inside VS Code inherits ELECTRON_RUN_AS_NODE, which
// would make Code.exe behave as plain Node and reject every option.
const env = { ...process.env, DDN_REAL_HOST_ROOT: workspace };
delete env.ELECTRON_RUN_AS_NODE;
const child = childProcess.spawn(code, args, {
  env,
  stdio: ["ignore", "pipe", "pipe"],
});
const log = fs.createWriteStream(path.join(runRoot, "host.log"));
child.stdout.on("data", (chunk) => {
  log.write(chunk);
  process.stdout.write(chunk.toString("utf8").split("\n").filter((line) => line.includes("REAL_HOST")).map((line) => line + "\n").join(""));
});
child.stderr.on("data", (chunk) => log.write(chunk));
child.on("close", (status) => {
  log.end();
  const reportPath = path.join(runRoot, "real-host-first-flow-result.json");
  const report = fs.existsSync(reportPath) ? JSON.parse(fs.readFileSync(reportPath, "utf8")) : null;
  console.log(`REAL_HOST_EXIT=${status}`);
  console.log(`REAL_HOST_STATUS=${report ? report.status : "NO_REPORT"}`);
  console.log(`REAL_HOST_REPORT=${reportPath}`);
  process.exitCode = status === 0 && report && report.status === "PASS_ACTUAL_HOST_FIRST_FLOW" ? 0 : 1;
});
