export function makeDiag({
  level = "ERROR",
  code = "E_UNKNOWN",
  message = "",
  where = null,
  details = null,
} = {}) {
  return {
    level: String(level || "ERROR").toUpperCase(),
    code: String(code || "E_UNKNOWN"),
    message: String(message ?? ""),
    where: where ?? undefined,
    details: details ?? undefined,
  };
}

export function formatDiagText(diag) {
  const d = diag && typeof diag === "object" ? diag : makeDiag({ message: String(diag ?? "") });
  const parts = [];
  if (d.level) parts.push(`[${d.level}]`);
  if (d.code) parts.push(d.code);
  if (d.message) parts.push(String(d.message));
  if (d.where) parts.push(`@${String(d.where)}`);
  const head = parts.join(" ");
  if (d.details === undefined || d.details === null || d.details === "") {
    return head;
  }
  const detailText =
    typeof d.details === "string"
      ? d.details
      : (() => {
          try {
            return JSON.stringify(d.details);
          } catch (_) {
            return String(d.details);
          }
        })();
  return `${head}\n${detailText}`;
}

