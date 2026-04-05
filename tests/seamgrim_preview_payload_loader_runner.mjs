import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/preview_payload_loader.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const {
    fetchPreviewPayload,
    resolvePreviewHtmlFromDescriptors,
  } = mod;

  assert(typeof fetchPreviewPayload === "function", "preview payload loader export: fetchPreviewPayload");
  assert(typeof resolvePreviewHtmlFromDescriptors === "function", "preview payload loader export: resolvePreviewHtmlFromDescriptors");

  let fetchCount = 0;
  const fetchImpl = async (url) => {
    fetchCount += 1;
    const text = String(url ?? "");
    if (text.includes("/preview/graph.json")) {
      return {
        ok: true,
        async json() {
          return { schema: "seamgrim.graph.v0", series: [{ id: "y", points: [{ x: 0, y: 1 }, { x: 1, y: 2 }] }] };
        },
      };
    }
    if (text.includes("/preview/text.md")) {
      return {
        ok: true,
        async text() {
          return "# 설명\n- 요약";
        },
      };
    }
    return { ok: false };
  };

  const cache = new Map();
  const graphDescriptor = { family: "graph", mode: "json", candidates: ["/preview/graph.json"] };
  const payload0 = await fetchPreviewPayload(graphDescriptor, { cache, fetchImpl });
  assert(payload0?.schema === "seamgrim.graph.v0", "preview payload loader: json payload");
  const payload1 = await fetchPreviewPayload(graphDescriptor, { cache, fetchImpl });
  assert(payload1?.schema === "seamgrim.graph.v0", "preview payload loader: cached payload");
  assert(fetchCount === 1, "preview payload loader: cache avoids duplicate fetch");

  const resolved = await resolvePreviewHtmlFromDescriptors(
    [
      { family: "graph", mode: "json", candidates: ["/preview/graph.json"] },
      { family: "text", mode: "text", candidates: ["/preview/text.md"] },
    ],
    {
      cache,
      fetchImpl,
      renderPreview: ({ family, payload, text }) => {
        if (family === "graph" && payload?.schema === "seamgrim.graph.v0") {
          return "<div>graph preview</div>";
        }
        if (family === "text" && text) {
          return "<div>text preview</div>";
        }
        return "";
      },
    },
  );
  assert(resolved?.descriptor?.family === "graph", "preview payload loader: first renderable descriptor chosen");
  assert(resolved?.html === "<div>graph preview</div>", "preview payload loader: html returned");

  console.log("seamgrim preview payload loader runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack ?? err));
  process.exit(1);
});
