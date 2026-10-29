import path from "node:path";
import { pathToFileURL } from "node:url";

const packageRoot = String(process.env.DDN_LOCAL_PLAYWRIGHT_NODE_MODULES ?? "").trim();

export async function resolve(specifier, context, nextResolve) {
  if (packageRoot && specifier === "playwright") {
    return {
      shortCircuit: true,
      url: pathToFileURL(path.join(packageRoot, "playwright", "index.mjs")).href,
    };
  }
  if (packageRoot && specifier === "playwright-core") {
    return {
      shortCircuit: true,
      url: pathToFileURL(path.join(packageRoot, "playwright-core", "index.mjs")).href,
    };
  }
  return nextResolve(specifier, context);
}
