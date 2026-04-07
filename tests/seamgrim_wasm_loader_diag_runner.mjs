import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function expectEnsureFailure(loader, sourceText, expectedCode, label) {
  let failed = false;
  try {
    await loader.ensure(sourceText);
  } catch (_) {
    failed = true;
  }
  assert(failed, `${label}: ensure must fail`);
  assert(
    loader.getLastInitDiag?.()?.code === expectedCode,
    `${label}: init diag mismatch (${JSON.stringify(loader.getLastInitDiag?.() ?? null)})`,
  );
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/wasm_page_common.js");
  const common = await import(pathToFileURL(modulePath).href);
  const { createWasmLoader } = common;
  assert(typeof createWasmLoader === "function", "createWasmLoader export missing");

  const sourceText = "매틱:움직씨 = { x <- 1. }.";

  const loadFailLoader = createWasmLoader({
    moduleFactory: async () => {
      throw new Error("module-load-fail");
    },
  });
  await expectEnsureFailure(
    loadFailLoader,
    sourceText,
    "E_WASM_LOADER_MODULE_LOAD_FAILED",
    "module-load-fail",
  );

  const initFailLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {
        throw new Error("module-init-fail");
      },
      wasm_build_info() {
        return "";
      },
      DdnWasmVm: function DdnWasmVm() {},
    }),
  });
  await expectEnsureFailure(
    initFailLoader,
    sourceText,
    "E_WASM_LOADER_MODULE_INIT_FAILED",
    "module-init-fail",
  );

  const exportMissingLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "";
      },
    }),
  });
  await expectEnsureFailure(
    exportMissingLoader,
    sourceText,
    "E_WASM_LOADER_EXPORT_MISSING",
    "export-missing",
  );

  const wrapperLoadFailLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "";
      },
      DdnWasmVm: function DdnWasmVm(_source) {
        this.get_build_info = () => "vm-build-info";
      },
    }),
    wrapperFactory: async () => {
      throw new Error("wrapper-load-fail");
    },
  });
  await expectEnsureFailure(
    wrapperLoadFailLoader,
    sourceText,
    "E_WASM_LOADER_WRAPPER_LOAD_FAILED",
    "wrapper-load-fail",
  );

  const wrapperExportMissingLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "";
      },
      DdnWasmVm: function DdnWasmVm(_source) {
        this.get_build_info = () => "vm-build-info";
      },
    }),
    wrapperFactory: async () => ({}),
  });
  await expectEnsureFailure(
    wrapperExportMissingLoader,
    sourceText,
    "E_WASM_LOADER_WRAPPER_EXPORT_MISSING",
    "wrapper-export-missing",
  );

  const vmConstructFailLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "";
      },
      DdnWasmVm: function DdnWasmVm(_source) {
        throw new Error("vm-construct-fail");
      },
    }),
    wrapperFactory: async () => ({
      DdnWasmVmClient: class DdnWasmVmClient {
        constructor(vm) {
          this.vm = vm;
        }
        updateLogic(_text) {}
      },
    }),
  });
  await expectEnsureFailure(
    vmConstructFailLoader,
    sourceText,
    "E_WASM_LOADER_VM_CONSTRUCT_FAILED",
    "vm-construct-fail",
  );

  const clientConstructFailLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "";
      },
      DdnWasmVm: function DdnWasmVm(_source) {
        this.get_build_info = () => "vm-build-info";
      },
    }),
    wrapperFactory: async () => ({
      DdnWasmVmClient: class DdnWasmVmClient {
        constructor(_vm) {
          throw new Error("client-construct-fail");
        }
      },
    }),
  });
  await expectEnsureFailure(
    clientConstructFailLoader,
    sourceText,
    "E_WASM_LOADER_CLIENT_CONSTRUCT_FAILED",
    "client-construct-fail",
  );

  const updateLogicFailLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "";
      },
      DdnWasmVm: function DdnWasmVm() {
        this.get_build_info = () => "vm-build-info";
      },
    }),
    wrapperFactory: async () => ({
      DdnWasmVmClient: class DdnWasmVmClient {
        constructor(vm) {
          this.vm = vm;
        }
        updateLogic(_text) {
          throw new Error("client-updateLogic-fail");
        }
      },
    }),
  });
  await expectEnsureFailure(
    updateLogicFailLoader,
    sourceText,
    "E_WASM_LOADER_CLIENT_UPDATELOGIC_FAILED",
    "client-updateLogic-fail",
  );

  const successLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        return "wasm-build";
      },
      DdnWasmVm: function DdnWasmVm(_source) {
        this.get_build_info = () => "vm-build-info";
      },
    }),
    wrapperFactory: async () => ({
      DdnWasmVmClient: class DdnWasmVmClient {
        constructor(vm) {
          this.vm = vm;
        }
        updateLogic(_text) {}
      },
    }),
  });
  const client = await successLoader.ensure(sourceText);
  assert(client && typeof client === "object", "success loader: client object expected");
  assert(successLoader.getLastInitDiag?.() === null, "success loader: init diag should be cleared");

  const buildInfoFallbackLoader = createWasmLoader({
    moduleFactory: async () => ({
      default() {},
      wasm_build_info() {
        throw new Error("wasm-build-info-fail");
      },
      DdnWasmVm: function DdnWasmVm(_source) {
        this.get_build_info = () => "vm-only-build";
      },
    }),
    wrapperFactory: async () => ({
      DdnWasmVmClient: class DdnWasmVmClient {
        constructor(vm) {
          this.vm = vm;
        }
        updateLogic(_text) {}
      },
    }),
  });
  const buildInfoFallbackClient = await buildInfoFallbackLoader.ensure(sourceText);
  assert(
    buildInfoFallbackClient && typeof buildInfoFallbackClient === "object",
    "build-info fallback loader: client object expected",
  );
  assert(
    buildInfoFallbackLoader.getLastBuildInfo?.() === "vm-only-build",
    `build-info fallback loader: expected vm-only-build, got ${String(buildInfoFallbackLoader.getLastBuildInfo?.())}`,
  );
  assert(
    buildInfoFallbackLoader.getLastBuildInfoDiag?.() === null,
    "build-info fallback loader: build info diag should be cleared after vm fallback success",
  );

  console.log("seamgrim wasm loader diag runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
