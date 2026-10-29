import path from "path";
import { pathToFileURL } from "url";

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/first_run_catalog.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const payload = {
    path_text: String(mod.SEAMGRIM_FIRST_RUN_PATH_TEXT ?? ""),
    estimated_minutes: Number(mod.SEAMGRIM_FIRST_RUN_ESTIMATED_MINUTES ?? 0),
    steps: Array.isArray(mod.SEAMGRIM_FIRST_RUN_STEPS)
      ? mod.SEAMGRIM_FIRST_RUN_STEPS.map((step) => ({
          id: String(step?.id ?? ""),
          title: String(step?.title ?? ""),
          target_kind: String(step?.targetKind ?? ""),
          target_id: String(step?.targetId ?? ""),
        }))
      : [],
  };
  process.stdout.write(`${JSON.stringify(payload)}\n`);
}

main().catch((err) => {
  const msg = String(err?.message ?? err);
  process.stderr.write(`check=seamgrim_first_run_catalog detail=${msg}\n`);
  process.exit(1);
});
