import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const commonPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/wasm_page_common.js");
  const mod = await import(pathToFileURL(commonPath).href);
  const { selectSpace2dPrimitiveItems } = mod;

  assert(
    typeof selectSpace2dPrimitiveItems === "function",
    "export missing: selectSpace2dPrimitiveItems",
  );

  const drawOnly = [{ kind: "line", x1: 0, y1: 0, x2: 1, y2: 1 }];
  const shapeOnly = [{ kind: "circle", x: 0, y: 0, r: 1 }];

  const shapesModeFallback = selectSpace2dPrimitiveItems({
    drawlist: drawOnly,
    shapes: [],
    hasShapesField: false,
    sourceMode: "shapes",
  });
  assert(
    Array.isArray(shapesModeFallback) && shapesModeFallback.length === 1 && shapesModeFallback[0].kind === "line",
    "shapes mode must fallback to drawlist",
  );

  const drawlistModeFallback = selectSpace2dPrimitiveItems({
    drawlist: [],
    shapes: shapeOnly,
    hasShapesField: true,
    sourceMode: "drawlist",
  });
  assert(
    Array.isArray(drawlistModeFallback) &&
      drawlistModeFallback.length === 1 &&
      drawlistModeFallback[0].kind === "circle",
    "drawlist mode must fallback to shapes",
  );

  const autoModeFallback = selectSpace2dPrimitiveItems({
    drawlist: drawOnly,
    shapes: [],
    hasShapesField: true,
    sourceMode: "auto",
  });
  assert(
    Array.isArray(autoModeFallback) && autoModeFallback.length === 1 && autoModeFallback[0].kind === "line",
    "auto mode must fallback to drawlist when shapes empty",
  );

  console.log("seamgrim space2d primitive source runner ok");
}

main().catch((err) => {
  console.error(String(err?.stack || err));
  process.exit(1);
});
