import { BrowserHostAdapter } from "./browser_host_adapter.js";

export const HOST_BRIDGE_V1_SCHEMA = "seamgrim.host_bridge.v1";

export const HOST_BRIDGE_V1_OPERATIONS = Object.freeze([
  "readTextFile",
  "readFileBytes",
  "downloadTextFile",
  "downloadFileBytes",
  "writeClipboardText",
]);

export class HostBridgeV1 {
  #adapter;

  constructor({ adapter = new BrowserHostAdapter() } = {}) {
    if (!adapter || typeof adapter !== "object") {
      throw new TypeError("host adapter must be an object");
    }
    this.schema = HOST_BRIDGE_V1_SCHEMA;
    this.#adapter = adapter;
  }

  readTextFile(file) { return this.#adapter.readTextFile(file); }
  readFileBytes(file) { return this.#adapter.readFileBytes(file); }
  downloadTextFile(options) { return this.#adapter.downloadTextFile(options); }
  downloadFileBytes(options) { return this.#adapter.downloadFileBytes(options); }
  writeClipboardText(text) { return this.#adapter.writeClipboardText(text); }
}

export function createHostBridgeV1(options = {}) {
  return new HostBridgeV1(options);
}
