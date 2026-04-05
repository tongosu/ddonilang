import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/play_output_contract.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    normalizePlayTab,
    resolvePlayTabVisibility,
    resolvePreferredPlayTab,
    resolvePlayActiveTab,
  } = mod;

  assert(typeof normalizePlayTab === "function", "play output contract: normalizePlayTab export");
  assert(typeof resolvePlayTabVisibility === "function", "play output contract: resolvePlayTabVisibility export");
  assert(typeof resolvePreferredPlayTab === "function", "play output contract: resolvePreferredPlayTab export");
  assert(typeof resolvePlayActiveTab === "function", "play output contract: resolvePlayActiveTab export");

  assert(normalizePlayTab("obs") === "obs", "normalize play tab: obs");
  assert(normalizePlayTab("unknown") === "diag", "normalize play tab: fallback");

  const emptyVisible = resolvePlayTabVisibility({});
  assert(emptyVisible.diag === true, "play visibility: diag always visible");
  assert(emptyVisible.obs === false, "play visibility: obs hidden when empty");
  assert(emptyVisible.mirror === false, "play visibility: mirror hidden when empty");

  const fullVisible = resolvePlayTabVisibility({
    hasDiagnostics: true,
    hasObservation: true,
    hasMirror: true,
  });
  assert(fullVisible.diag === true, "play visibility: diag visible");
  assert(fullVisible.obs === true, "play visibility: obs visible");
  assert(fullVisible.mirror === true, "play visibility: mirror visible");

  assert(resolvePreferredPlayTab({ hasError: true, hasObservation: true }) === "diag", "play preferred: error => diag");
  assert(resolvePreferredPlayTab({ hasDiagnostics: true, hasObservation: true }) === "obs", "play preferred: output beats warn");
  assert(resolvePreferredPlayTab({ hasMirror: true }) === "mirror", "play preferred: mirror fallback");

  assert(
    resolvePlayActiveTab("obs", { hasObservation: false, hasMirror: true }, { preserveCurrent: true }) === "mirror",
    "play active: switch away from hidden obs",
  );
  assert(
    resolvePlayActiveTab("mirror", { hasObservation: true, hasMirror: true }, { preserveCurrent: true }) === "mirror",
    "play active: preserve visible current tab",
  );
  assert(
    resolvePlayActiveTab("diag", { hasDiagnostics: false, hasObservation: true, hasMirror: true }) === "obs",
    "play active: prefer obs when available",
  );

  console.log("seamgrim play output contract ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
