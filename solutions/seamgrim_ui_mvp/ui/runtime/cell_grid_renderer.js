const CELL_GRID_SCHEMA = "seamgrim.cell_grid_render.v1";
const CELL_GRID_RENDERER = "cell_grid";
const DEFAULT_TILE_TEXT_MAP = Object.freeze({});

function toFiniteInt(rawValue, fallback = 0, min = 0) {
  const value = Number(rawValue);
  if (!Number.isFinite(value)) return fallback;
  return Math.max(min, Math.trunc(value));
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function normalizeCell(cell) {
  const source = cell && typeof cell === "object" ? cell : {};
  return {
    row: toFiniteInt(source.row, 0, 0),
    col: toFiniteInt(source.col, 0, 0),
    text: String(source.text ?? ""),
    ...(source.fg ? { fg: String(source.fg) } : {}),
    ...(source.bg ? { bg: String(source.bg) } : {}),
    ...(source.tag ? { tag: String(source.tag) } : {}),
    ...(source.source ? { source: String(source.source) } : {}),
    ...(source.tile_kind !== undefined ? { tile_kind: toFiniteInt(source.tile_kind, 0, 0) } : {}),
    ...(source.actor_id ? { actor_id: String(source.actor_id) } : {}),
  };
}

function normalizeFamily(rawFamily) {
  const family = String(rawFamily ?? "").trim().toLowerCase();
  return family === "grid2d" ? "grid2d" : "console";
}

function normalizeRole(rawRole, family) {
  const role = String(rawRole ?? "").trim().toLowerCase();
  if (role === "world_grid" || role === "output") return role;
  return family === "grid2d" ? "world_grid" : "output";
}

export function normalizeCellGridRender(raw) {
  const source = raw && typeof raw === "object" ? raw : {};
  const family = normalizeFamily(source.family);
  const role = normalizeRole(source.role, family);
  const rows = toFiniteInt(source.rows, 1, 1);
  const cols = toFiniteInt(source.cols, 1, 1);
  return {
    schema: CELL_GRID_SCHEMA,
    family,
    renderer: CELL_GRID_RENDERER,
    role,
    rows,
    cols,
    cells: (Array.isArray(source.cells) ? source.cells : []).map(normalizeCell),
    meta: source.meta && typeof source.meta === "object" ? cloneJson(source.meta) : {},
  };
}

export function consoleLinesToCellGrid(lines = [], options = {}) {
  const allLines = Array.isArray(lines) ? lines.map((line) => String(line ?? "")) : [];
  const rows = Math.max(1, toFiniteInt(options.rows, allLines.length || 1, 1));
  const cols = Math.max(
    1,
    toFiniteInt(
      options.cols,
      allLines.reduce((max, line) => Math.max(max, line.length), 1),
      1,
    ),
  );
  const visibleLines = allLines.length > rows ? allLines.slice(-rows) : allLines;
  const cells = [];
  visibleLines.forEach((line, row) => {
    let col = 0;
    for (const ch of line) {
      if (col >= cols) break;
      cells.push({ row, col, text: ch, source: "console_line" });
      col += 1;
    }
  });
  return normalizeCellGridRender({
    family: "console",
    role: "output",
    rows,
    cols,
    cells,
    meta: {
      lines: allLines,
      visible_lines: visibleLines,
      empty_text: String(options.emptyText ?? "콘솔 출력 없음"),
    },
  });
}

function normalizeGridSource(fixtureOrGridState, options = {}) {
  const source = fixtureOrGridState && typeof fixtureOrGridState === "object" ? fixtureOrGridState : {};
  const grid = source.grid && typeof source.grid === "object" ? source.grid : source;
  const actors = Array.isArray(source.actors)
    ? source.actors
    : Array.isArray(source.actorList)
      ? source.actorList
      : Array.isArray(options.actorList)
        ? options.actorList
        : [];
  return { grid, actors };
}

export function grid2dFixtureToCellGrid(fixtureOrGridState, options = {}) {
  const { grid, actors } = normalizeGridSource(fixtureOrGridState, options);
  const rows = toFiniteInt(grid.gridH ?? grid.grid_h, 1, 1);
  const cols = toFiniteInt(grid.gridW ?? grid.grid_w, 1, 1);
  const rawTiles = Array.isArray(grid.tiles) ? grid.tiles : [];
  const tileTextMap = options.tileTextMap && typeof options.tileTextMap === "object"
    ? options.tileTextMap
    : DEFAULT_TILE_TEXT_MAP;
  const cells = [];
  for (let row = 0; row < rows; row += 1) {
    const sourceRow = Array.isArray(rawTiles[row]) ? rawTiles[row] : [];
    for (let col = 0; col < cols; col += 1) {
      const kind = toFiniteInt(sourceRow[col], 0, 0);
      const mapped = Object.prototype.hasOwnProperty.call(tileTextMap, String(kind))
        ? tileTextMap[String(kind)]
        : kind;
      cells.push({
        row,
        col,
        text: String(mapped ?? ""),
        source: "tile",
        tag: kind === 0 ? "floor" : "tile",
        tile_kind: kind,
      });
    }
  }
  actors.forEach((actor) => {
    if (!actor || typeof actor !== "object") return;
    const id = String(actor.id ?? actor.name ?? "").trim();
    if (!id) return;
    cells.push({
      row: toFiniteInt(actor.row, 0, 0),
      col: toFiniteInt(actor.col, 0, 0),
      text: String(actor.label ?? id.slice(0, 1).toUpperCase()),
      source: "actor",
      tag: "actor",
      actor_id: id,
      fg: String(actor.fill ?? ""),
    });
  });
  return normalizeCellGridRender({
    family: "grid2d",
    role: "world_grid",
    rows,
    cols,
    cells,
    meta: {
      grid: cloneJson(grid),
      actors: cloneJson(actors),
    },
  });
}

export function summarizeCellGridRender(cellGrid, { sampleLimit = 12 } = {}) {
  const normalized = normalizeCellGridRender(cellGrid);
  const limit = Math.max(0, toFiniteInt(sampleLimit, 12, 0));
  return {
    cell_grid_schema: normalized.schema,
    cell_grid_family: normalized.family,
    cell_grid_renderer: normalized.renderer,
    cell_grid_role: normalized.role,
    cell_grid_cell_count: normalized.cells.length,
    cell_grid_sample_cells: normalized.cells.slice(0, limit).map((cell) => ({ ...cell })),
  };
}

function resolveConsoleDisplaySize(canvas) {
  const rect = typeof canvas.getBoundingClientRect === "function" ? canvas.getBoundingClientRect() : null;
  const parentRect =
    typeof canvas.parentElement?.getBoundingClientRect === "function"
      ? canvas.parentElement.getBoundingClientRect()
      : null;
  return {
    width: Math.max(
      1,
      Math.round(Number(canvas.clientWidth || rect?.width || canvas.parentElement?.clientWidth || parentRect?.width || 640)),
    ),
    height: Math.max(
      1,
      Math.round(Number(canvas.clientHeight || rect?.height || canvas.parentElement?.clientHeight || parentRect?.height || 360)),
    ),
  };
}

function renderConsoleCellGrid(canvas, cellGrid) {
  const { width: displayW, height: displayH } = resolveConsoleDisplaySize(canvas);
  if (canvas.width !== displayW || canvas.height !== displayH) {
    canvas.width = displayW;
    canvas.height = displayH;
  }
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  const W = canvas.width;
  const H = canvas.height;
  const fontPx = Math.max(11, Math.min(16, Math.floor(W / 58)));
  const cellW = Math.round(fontPx * 0.62);
  const cellH = Math.round(fontPx * 1.75);
  const cols = Math.floor(W / cellW);
  const rows = Math.floor(H / cellH);

  ctx.fillStyle = "#080c18";
  ctx.fillRect(0, 0, W, H);
  ctx.strokeStyle = "rgba(30, 70, 140, 0.28)";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  for (let c = 0; c <= cols; c += 1) {
    ctx.moveTo(c * cellW, 0);
    ctx.lineTo(c * cellW, H);
  }
  for (let r = 0; r <= rows; r += 1) {
    ctx.moveTo(0, r * cellH);
    ctx.lineTo(W, r * cellH);
  }
  ctx.stroke();

  ctx.font = `${fontPx}px "Courier New", Courier, monospace`;
  ctx.textBaseline = "middle";
  const allLines = Array.isArray(cellGrid.meta?.lines) ? cellGrid.meta.lines.map((line) => String(line ?? "")) : [];
  const visibleLines = allLines.length > rows ? allLines.slice(-rows) : allLines;
  for (let rowIdx = 0; rowIdx < visibleLines.length; rowIdx += 1) {
    const line = visibleLines[rowIdx];
    const cellY = rowIdx * cellH + cellH / 2;
    let colIdx = 0;
    for (let i = 0; i < line.length && colIdx < cols; i += 1) {
      const ch = line[i];
      const cellX = colIdx * cellW + 1;
      if ((ch >= "0" && ch <= "9") || ch === ".") {
        ctx.fillStyle = "#7dd3fc";
      } else if (ch === "=" || ch === ":" || ch === "[" || ch === "]") {
        ctx.fillStyle = "#475569";
      } else if (ch === " ") {
        colIdx += 1;
        continue;
      } else {
        ctx.fillStyle = "#cbd5e1";
      }
      ctx.fillText(ch, cellX, cellY);
      const charW = ctx.measureText(ch).width;
      colIdx += charW > cellW * 1.3 ? 2 : 1;
    }
  }
  if (allLines.length === 0) {
    ctx.fillStyle = "rgba(71, 85, 105, 0.6)";
    ctx.font = `${fontPx + 1}px "Courier New", Courier, monospace`;
    ctx.textBaseline = "middle";
    const msg = String(cellGrid.meta?.empty_text ?? "콘솔 출력 없음");
    const msgW = ctx.measureText(msg).width;
    ctx.fillText(msg, (W - msgW) / 2, H / 2);
  }
  return { canvas_width: W, canvas_height: H, cols, rows };
}

function renderGrid2dCellGrid(canvas, cellGrid, options = {}) {
  const gridState = options.gridState ?? cellGrid.meta?.grid ?? {};
  const actorList = Array.isArray(options.actorList) ? options.actorList : Array.isArray(cellGrid.meta?.actors) ? cellGrid.meta.actors : [];
  const gridW = Number(gridState.gridW ?? gridState.grid_w ?? cellGrid.cols);
  const gridH = Number(gridState.gridH ?? gridState.grid_h ?? cellGrid.rows);
  const tileSize = Number(gridState.tileSize ?? gridState.tile_size ?? 24);
  const width = gridW * tileSize;
  const height = gridH * tileSize;
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  if (typeof ctx.clearRect === "function") {
    ctx.clearRect(0, 0, width, height);
  }
  ctx.fillStyle = String(gridState.background ?? "#fdf8ef");
  if (typeof ctx.fillRect === "function") {
    ctx.fillRect(0, 0, width, height);
  }
  const rawTiles = Array.isArray(gridState.tiles) ? gridState.tiles : [];
  const palette = gridState.palette && typeof gridState.palette === "object" ? gridState.palette : {};
  let tileDrawCount = 0;
  let actorDrawCount = 0;
  for (let row = 0; row < gridH; row += 1) {
    for (let col = 0; col < gridW; col += 1) {
      const kind = toFiniteInt(rawTiles[row]?.[col], 0, 0);
      const fill = String(palette[String(kind)] ?? palette[0] ?? "#f6f1e8");
      const x = col * tileSize;
      const y = row * tileSize;
      ctx.fillStyle = fill;
      ctx.fillRect(x, y, tileSize, tileSize);
      if (typeof ctx.strokeRect === "function") {
        ctx.strokeStyle = String(gridState.gridLineColor ?? gridState.grid_line_color ?? "#d8cfbf");
        ctx.strokeRect(x, y, tileSize, tileSize);
      }
      tileDrawCount += 1;
    }
  }
  actorList.forEach((actor) => {
    if (actor.row >= gridH || actor.col >= gridW) return;
    const inset = Math.max(2, Math.floor(tileSize * 0.18));
    const x = actor.col * tileSize + inset;
    const y = actor.row * tileSize + inset;
    const size = Math.max(2, tileSize - inset * 2);
    ctx.fillStyle = actor.fill;
    ctx.fillRect(x, y, size, size);
    if (typeof ctx.fillText === "function") {
      ctx.fillStyle = "#111111";
      ctx.font = `${Math.max(8, Math.floor(tileSize * 0.42))}px sans-serif`;
      ctx.fillText(String(actor.label), x + Math.max(1, Math.floor(size * 0.22)), y + Math.max(10, Math.floor(size * 0.72)));
    }
    actorDrawCount += 1;
  });
  return { canvas_width: width, canvas_height: height, tiles: tileDrawCount, actors: actorDrawCount };
}

export function renderCellGridCanvas2d(canvas, rawCellGrid, options = {}) {
  if (!canvas || typeof canvas.getContext !== "function") return null;
  const cellGrid = normalizeCellGridRender(rawCellGrid);
  if (cellGrid.family === "grid2d") {
    return renderGrid2dCellGrid(canvas, cellGrid, options);
  }
  return renderConsoleCellGrid(canvas, cellGrid);
}

export { CELL_GRID_SCHEMA, CELL_GRID_RENDERER };
