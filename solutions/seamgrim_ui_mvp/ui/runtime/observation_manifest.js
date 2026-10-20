const SCHEMA = "ddn.observation_manifest.v0";
const VERSION = "20.6.33";

function asObject(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return {};
  return value;
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function normalizePragmaKind(kind) {
  const raw = String(kind ?? "").trim();
  if (!raw) return "기타";
  return raw;
}

function classifyRoleByName(name) {
  const key = String(name ?? "").trim();
  if (!key) return "상태";
  if (
    key === "__tick__" ||
    key === "tick_id" ||
    key === "tick" ||
    key === "state_hash" ||
    key === "view_hash" ||
    key === "frame_id" ||
    key === "tick_time_ms"
  ) {
    return "파생";
  }
  if (key.startsWith("보개_") || key.startsWith("__view_")) {
    return "뷰";
  }
  return "상태";
}

function normalizeNode(entry) {
  const obj = asObject(entry);
  const name = String(obj.key ?? obj.name ?? "").trim();
  if (!name) return null;
  return {
    name,
    dtype: String(obj.dtype ?? "unknown"),
    role: String(obj.role ?? classifyRoleByName(name)),
    unit: typeof obj.unit === "string" ? obj.unit : undefined,
    pragma_refs: [],
  };
}

function collectMentionedNodeNamesFromPragma(pragma) {
  const names = [];
  const args = asObject(pragma?.args);
  Object.values(args).forEach((raw) => {
    const text = String(raw ?? "").trim();
    if (!text) return;
    // Keep Gate0 simple: comma/space separated identifiers.
    text
      .split(/[,\s]+/)
      .map((token) => token.trim())
      .filter(Boolean)
      .forEach((token) => {
        if (/^[\[\]().]+$/.test(token)) return;
        names.push(token);
      });
  });
  return names;
}

export function buildObservationManifest({
  channels = [],
  pragmas = [],
  version = VERSION,
} = {}) {
  const inputChannels = asArray(channels);
  const nodes = [];
  const nodeMap = new Map();

  inputChannels.forEach((entry) => {
    const node = normalizeNode(entry);
    if (!node) return;
    nodeMap.set(node.name, node);
    nodes.push(node);
  });

  const pragmaList = asArray(pragmas);
  pragmaList.forEach((pragma) => {
    const kind = normalizePragmaKind(pragma?.kind);
    const mentioned = collectMentionedNodeNamesFromPragma(pragma);
    mentioned.forEach((name) => {
      const node = nodeMap.get(name);
      if (!node) return;
      if (!node.pragma_refs.includes(kind)) {
        node.pragma_refs.push(kind);
      }
    });
  });

  return {
    schema: SCHEMA,
    version: String(version || VERSION),
    nodes: nodes.map((node) => {
      if (!node.pragma_refs.length) {
        const { pragma_refs: _unused, ...rest } = node;
        return rest;
      }
      return node;
    }),
  };
}
