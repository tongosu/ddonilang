const TOAST_HOST_CLASS = "ui-toast-host";
const TOAST_CLASS = "ui-toast";

let toastHost = null;

function ensureToastHost() {
  if (toastHost) return toastHost;
  if (typeof document === "undefined" || typeof document.createElement !== "function") return null;
  const body = document.body;
  if (!body || typeof body.appendChild !== "function") return null;
  const host = document.createElement("div");
  host.className = TOAST_HOST_CLASS;
  host.setAttribute?.("aria-live", "polite");
  host.setAttribute?.("aria-atomic", "false");
  body.appendChild(host);
  toastHost = host;
  return host;
}

export function showGlobalToast(message, { kind = "info", durationMs = 1200 } = {}) {
  const host = ensureToastHost();
  if (!host || typeof host.appendChild !== "function") return false;
  const text = String(message ?? "").trim();
  if (!text) return false;
  const node = document.createElement("div");
  node.className = `${TOAST_CLASS} ${TOAST_CLASS}-${String(kind ?? "info").trim() || "info"}`;
  node.textContent = text;
  host.appendChild(node);
  const ttl = Math.max(300, Number.isFinite(Number(durationMs)) ? Math.trunc(Number(durationMs)) : 1200);
  window.setTimeout(() => {
    try {
      host.removeChild(node);
    } catch (_) {
      // ignore toast removal errors
    }
  }, ttl);
  return true;
}
