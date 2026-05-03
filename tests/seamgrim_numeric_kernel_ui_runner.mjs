import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const index = fs.readFileSync(path.join(root, "solutions/seamgrim_ui_mvp/ui/index.html"), "utf8");
const runJs = fs.readFileSync(path.join(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js"), "utf8");
const worker = fs.readFileSync(
  path.join(root, "solutions/seamgrim_ui_mvp/ui/runtime/numeric_kernel_worker.js"),
  "utf8",
);

function mustContain(text, needle, label) {
  if (!text.includes(needle)) {
    throw new Error(`missing ${label}: ${needle}`);
  }
}

mustContain(index, "run-numeric-kernel-panel", "numeric kernel panel");
mustContain(index, "수 작업", "numeric kernel label");
mustContain(runJs, "updateNumericKernelPanel", "numeric panel renderer");
mustContain(runJs, "numeric:factor:", "numeric factor diag filter");
mustContain(worker, "numeric-factor-step", "numeric worker message");

console.log("seamgrim numeric kernel ui runner ok");
