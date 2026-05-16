import { RunScreen } from "../solutions/seamgrim_ui_mvp/ui/screens/run.js";

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  const screen = new RunScreen({});
  const captured = [];
  screen.screenVisible = true;
  screen.lesson = { id: "live", ddnText: "" };
  screen.sourceKind = "scratch";
  screen.liveReplDebounceMs = 20;
  screen.runDdnPreviewEl = { value: "첫째" };
  screen.enqueueRunRequest = (request) => {
    captured.push({ ...request });
    return request;
  };

  if (!screen.scheduleLiveReplRestart("첫째")) {
    throw new Error("first schedule failed");
  }
  screen.runDdnPreviewEl.value = "둘째";
  screen.scheduleLiveReplRestart("둘째");
  await wait(60);

  if (captured.length !== 1) {
    throw new Error(`expected one debounced run, got ${captured.length}`);
  }
  if (captured[0].sourceText !== "둘째") {
    throw new Error(`latest source not used: ${captured[0].sourceText}`);
  }
  if (captured[0].launchKind !== "live_repl") {
    throw new Error(`launch kind mismatch: ${captured[0].launchKind}`);
  }

  screen.screenVisible = false;
  if (screen.scheduleLiveReplRestart("셋째")) {
    throw new Error("hidden screen should not schedule live repl");
  }

  console.log("[seamgrim-live-repl] ok");
}

main().catch((error) => {
  console.error(`[seamgrim-live-repl] fail ${String(error?.message ?? error)}`);
  process.exit(1);
});
