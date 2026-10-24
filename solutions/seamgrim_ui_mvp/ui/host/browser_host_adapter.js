function requireMethod(owner, name, label) {
  const operation = owner?.[name];
  if (typeof operation !== "function") {
    throw new TypeError(`${label} is unavailable`);
  }
  return operation.bind(owner);
}

export class BrowserHostAdapter {
  #documentRef;
  #navigatorRef;

  constructor({ documentRef = globalThis.document, navigatorRef = globalThis.navigator } = {}) {
    this.#documentRef = documentRef;
    this.#navigatorRef = navigatorRef;
  }

  readTextFile(file) {
    return requireMethod(file, "text", "browser text file read")();
  }

  readFileBytes(file) {
    return requireMethod(file, "arrayBuffer", "browser binary file read")();
  }

  downloadTextFile({ text = "", filename = "download.txt", mimeType = "text/plain;charset=utf-8" } = {}) {
    const createElement = requireMethod(this.#documentRef, "createElement", "browser file download");
    const appendChild = requireMethod(this.#documentRef?.body, "appendChild", "browser file download body");
    const link = createElement("a");
    link.href = `data:${mimeType},${encodeURIComponent(String(text ?? ""))}`;
    link.download = filename;
    link.rel = "noopener";
    link.style.display = "none";
    appendChild(link);
    requireMethod(link, "click", "browser file download link")();
    requireMethod(link, "remove", "browser file download cleanup")();
  }

  writeClipboardText(text) {
    return requireMethod(this.#navigatorRef?.clipboard, "writeText", "browser clipboard write")(text);
  }

  downloadFileBytes({ bytes, filename = "input.tape" } = {}) {
    const url = URL.createObjectURL(new Blob([bytes], { type: "application/octet-stream" }));
    const link = this.#documentRef.createElement("a");
    try {
      link.href = url;
      link.download = filename;
      link.style.display = "none";
      this.#documentRef.body.appendChild(link);
      link.click();
    } finally {
      link.remove();
      setTimeout(() => URL.revokeObjectURL(url), 0);
    }
  }
}
