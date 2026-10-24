const STORE_KEY = "seamgrim.local_gotgan_store.v1";

export function emptyLocalGotganStore() {
  return { schema: "ddn.local_gotgan_store.v1", artifacts: [] };
}

export function loadLocalGotganStore(storage = globalThis.localStorage) {
  const raw = storage?.getItem?.(STORE_KEY);
  if (!raw) return emptyLocalGotganStore();
  try {
    const parsed = JSON.parse(raw);
    return parsed?.schema === "ddn.local_gotgan_store.v1" && Array.isArray(parsed.artifacts)
      ? parsed
      : emptyLocalGotganStore();
  } catch {
    return emptyLocalGotganStore();
  }
}

export async function publishToLocalGotgan(runtime, artifact, storage = globalThis.localStorage) {
  if (typeof runtime?.publishLocalGotgan !== "function") {
    throw new Error("E_GOTGAN_WASM_REQUIRED 공통 Rust/WASM 발행 기능이 필요합니다");
  }
  const request = {
    schema: "ddn.local_gotgan_publish_request.v1",
    store: loadLocalGotganStore(storage),
    artifact,
  };
  const store = await runtime.publishLocalGotgan(request);
  storage?.setItem?.(STORE_KEY, JSON.stringify(store));
  return store;
}

export async function searchLocalGotgan(runtime, { query = "", profiles = [] } = {}, storage = globalThis.localStorage) {
  if (typeof runtime?.searchLocalGotgan !== "function") {
    throw new Error("E_GOTGAN_WASM_REQUIRED 공통 Rust/WASM 검색 기능이 필요합니다");
  }
  return runtime.searchLocalGotgan({
    schema: "ddn.local_gotgan_search_request.v1",
    store: loadLocalGotganStore(storage),
    query,
    profiles,
  });
}

export function exportLocalGotganArtifact(artifact) {
  return `${JSON.stringify(artifact, null, 2)}\n`;
}

export async function importLocalGotganArtifact(runtime, text, storage = globalThis.localStorage) {
  const artifact = JSON.parse(text);
  return publishToLocalGotgan(runtime, artifact, storage);
}
