#!/usr/bin/env node

import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT = path.resolve(__dirname, "..");
const clientUrl = pathToFileURL(path.resolve(ROOT, "solutions/seamgrim_ui_mvp/ui/project_service_client.js")).href;
const contractUrl = pathToFileURL(path.resolve(ROOT, "solutions/seamgrim_ui_mvp/ui/platform_server_adapter_contract.js")).href;

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const {
  PROJECT_CAPABILITY_SESSION_KEY,
  buildProjectCapabilityFragment,
  captureProjectCapabilityFromLocation,
  mapProjectApiStatusToAdapterCode,
  parseProjectCapabilityFragment,
  readProjectCapabilitySession,
  requestProjectApi,
} = await import(clientUrl);
const { PlatformServerAdapterErrorCode } = await import(contractUrl);

const fragment = buildProjectCapabilityFragment({
  projectId: "sgp_client",
  role: "edit",
  capability: "raw-edit-capability",
});
assert(fragment.startsWith("#"), "fragment starts with hash");
assert(!fragment.includes("?"), "capability must not use query string");
const parsed = parseProjectCapabilityFragment(fragment);
assert(parsed.project_id === "sgp_client", "parse project id");
assert(parsed.role === "edit", "parse role");
assert(parsed.capability === "raw-edit-capability", "parse capability");

const storage = new Map();
const sessionStorage = {
  setItem(key, value) { storage.set(key, value); },
  getItem(key) { return storage.get(key) ?? null; },
};
const replaced = [];
const captured = captureProjectCapabilityFromLocation({
  location: {
    hash: fragment,
    pathname: "/solutions/seamgrim_ui_mvp/ui/index.html",
    search: "?first_delight=pendulum",
  },
  sessionStorage,
  history: {
    replaceState(_state, _title, url) { replaced.push(url); },
  },
});
assert(captured.capability === "raw-edit-capability", "captured raw capability for session");
assert(replaced[0] === "/solutions/seamgrim_ui_mvp/ui/index.html?first_delight=pendulum", "fragment scrubbed");
const session = readProjectCapabilitySession({ sessionStorage });
assert(session.project_id === "sgp_client", "session project id");
assert(session.capability === "raw-edit-capability", "session capability");
assert(storage.get(PROJECT_CAPABILITY_SESSION_KEY).includes("raw-edit-capability"), "session stores page-local capability");

let observedRequest = null;
const payload = await requestProjectApi("/api/v1/projects/sgp_client", {
  capability: "raw-edit-capability",
  fetchImpl: async (url, init) => {
    observedRequest = { url, init };
    return {
      ok: true,
      status: 200,
      async json() {
        return { project_id: "sgp_client", capability_role: "edit" };
      },
    };
  },
});
assert(payload.project_id === "sgp_client", "request payload");
assert(observedRequest.init.headers.Authorization === "Bearer raw-edit-capability", "authorization header");
assert(!String(observedRequest.url).includes("raw-edit-capability"), "raw capability not in URL");

try {
  await requestProjectApi("/api/v1/projects/sgp_client", {
    fetchImpl: async () => ({
      ok: false,
      status: 409,
      async json() {
        return { code: "E_PROJECT_REVISION_CONFLICT", user_message: "충돌" };
      },
    }),
  });
  throw new Error("expected requestProjectApi conflict error");
} catch (err) {
  assert(err.status === 409, "error status");
  assert(err.adapterCode === PlatformServerAdapterErrorCode.VALIDATION_FAILED, "409 adapter mapping");
}

assert(mapProjectApiStatusToAdapterCode(401) === PlatformServerAdapterErrorCode.AUTH_REQUIRED, "401 mapping");
assert(mapProjectApiStatusToAdapterCode(403) === PlatformServerAdapterErrorCode.PERMISSION_DENIED, "403 mapping");
assert(mapProjectApiStatusToAdapterCode(413) === PlatformServerAdapterErrorCode.VALIDATION_FAILED, "413 mapping");

console.log("seamgrim project service client runner ok");
