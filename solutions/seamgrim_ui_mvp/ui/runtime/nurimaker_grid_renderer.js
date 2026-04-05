const DEFAULT_TILE_PALETTE = Object.freeze({
  0: "#f6f1e8",
  1: "#2e3440",
  2: "#67b99a",
  3: "#f4b860",
  4: "#7aa2f7",
});

function toFiniteInt(rawValue, fallback = 0, min = 0) {
  const value = Number(rawValue);
  if (!Number.isFinite(value)) return fallback;
  return Math.max(min, Math.trunc(value));
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function normalizePalette(rawPalette) {
  const source = rawPalette && typeof rawPalette === "object" ? rawPalette : {};
  const out = { ...DEFAULT_TILE_PALETTE };
  Object.entries(source).forEach(([key, value]) => {
    const color = String(value ?? "").trim();
    if (!color) return;
    out[String(key)] = color;
  });
  return out;
}

export function normalizeGridState(rawGridState, fallback = {}) {
  const source = rawGridState && typeof rawGridState === "object" ? rawGridState : {};
  const gridW = toFiniteInt(source.gridW ?? source.grid_w ?? fallback.gridW ?? fallback.grid_w, 1, 1);
  const gridH = toFiniteInt(source.gridH ?? source.grid_h ?? fallback.gridH ?? fallback.grid_h, 1, 1);
  const tileSize = toFiniteInt(
    source.tileSize ?? source.tile_size ?? fallback.tileSize ?? fallback.tile_size,
    24,
    4,
  );
  const rawTiles = Array.isArray(source.tiles) ? source.tiles : [];
  const tiles = [];
  for (let row = 0; row < gridH; row += 1) {
    const sourceRow = Array.isArray(rawTiles[row]) ? rawTiles[row] : [];
    const nextRow = [];
    for (let col = 0; col < gridW; col += 1) {
      nextRow.push(toFiniteInt(sourceRow[col], 0, 0));
    }
    tiles.push(nextRow);
  }
  return {
    schema: "seamgrim.nurimaker.grid_state.v1",
    gridW,
    gridH,
    tileSize,
    tiles,
    palette: normalizePalette(source.palette ?? fallback.palette),
    background: String(source.background ?? fallback.background ?? "#fdf8ef"),
    gridLineColor: String(source.gridLineColor ?? source.grid_line_color ?? fallback.gridLineColor ?? "#d8cfbf"),
  };
}

function normalizeActor(actor) {
  const source = actor && typeof actor === "object" ? actor : {};
  const id = String(source.id ?? source.name ?? "").trim();
  if (!id) return null;
  return {
    id,
    row: toFiniteInt(source.row, 0, 0),
    col: toFiniteInt(source.col, 0, 0),
    fill: String(source.fill ?? "#e85d75"),
    label: String(source.label ?? id.slice(0, 1).toUpperCase()),
  };
}

function normalizeActorList(rawActorList) {
  return (Array.isArray(rawActorList) ? rawActorList : []).map(normalizeActor).filter(Boolean);
}

function summarizeTileCounts(tiles) {
  const counts = {};
  (Array.isArray(tiles) ? tiles : []).forEach((row) => {
    (Array.isArray(row) ? row : []).forEach((value) => {
      const key = String(toFiniteInt(value, 0, 0));
      counts[key] = Number(counts[key] ?? 0) + 1;
    });
  });
  return Object.fromEntries(
    Object.entries(counts).sort((a, b) => Number(a[0]) - Number(b[0])),
  );
}

function nonZeroTileCells(tiles) {
  const out = [];
  (Array.isArray(tiles) ? tiles : []).forEach((row, rowIndex) => {
    (Array.isArray(row) ? row : []).forEach((value, colIndex) => {
      const kind = toFiniteInt(value, 0, 0);
      if (kind === 0) return;
      out.push({ row: rowIndex, col: colIndex, kind });
    });
  });
  return out;
}

function defaultCanvasRect(canvas) {
  const width = Number(canvas?.width ?? 0) || 1;
  const height = Number(canvas?.height ?? 0) || 1;
  return {
    left: 0,
    top: 0,
    width,
    height,
  };
}

export class NurimakerGridRenderer {
  constructor(canvas, options = {}) {
    this.canvas = canvas ?? null;
    this.options = { ...options };
    this.gridState = normalizeGridState(options.gridState, options);
    this.actorList = normalizeActorList(options.actorList);
    this.clickHandlers = new Set();
    this.lastRenderSummary = null;
    this.handleCanvasClick = this.handleCanvasClick.bind(this);
    if (this.canvas && typeof this.canvas.addEventListener === "function") {
      this.canvas.addEventListener("click", this.handleCanvasClick);
    }
  }

  destroy() {
    if (this.canvas && typeof this.canvas.removeEventListener === "function") {
      this.canvas.removeEventListener("click", this.handleCanvasClick);
    }
    this.clickHandlers.clear();
  }

  onCellClick(callback) {
    if (typeof callback !== "function") {
      return () => {};
    }
    this.clickHandlers.add(callback);
    return () => {
      this.clickHandlers.delete(callback);
    };
  }

  setPalette(palette) {
    this.gridState = {
      ...this.gridState,
      palette: normalizePalette(palette),
    };
    return this.render();
  }

  renderGrid(gridState) {
    this.gridState = normalizeGridState(gridState, this.gridState);
    return this.render();
  }

  renderActors(actorList) {
    this.actorList = normalizeActorList(actorList);
    return this.render();
  }

  getGridState() {
    return cloneJson(this.gridState);
  }

  getActorList() {
    return cloneJson(this.actorList);
  }

  pickCellFromClientPoint(clientX, clientY) {
    const gridState = this.gridState;
    if (!this.canvas || !gridState) return null;
    const rect =
      typeof this.canvas.getBoundingClientRect === "function"
        ? this.canvas.getBoundingClientRect()
        : defaultCanvasRect(this.canvas);
    const width = Number(rect?.width ?? 0) || Number(this.canvas?.width ?? 0) || 1;
    const height = Number(rect?.height ?? 0) || Number(this.canvas?.height ?? 0) || 1;
    const left = Number(rect?.left ?? 0) || 0;
    const top = Number(rect?.top ?? 0) || 0;
    const localX = ((Number(clientX) - left) / width) * (gridState.gridW * gridState.tileSize);
    const localY = ((Number(clientY) - top) / height) * (gridState.gridH * gridState.tileSize);
    const col = Math.floor(localX / gridState.tileSize);
    const row = Math.floor(localY / gridState.tileSize);
    if (row < 0 || col < 0 || row >= gridState.gridH || col >= gridState.gridW) {
      return null;
    }
    return { row, col };
  }

  handleCanvasClick(event) {
    const hit = this.pickCellFromClientPoint(event?.clientX ?? 0, event?.clientY ?? 0);
    if (!hit) return;
    this.clickHandlers.forEach((callback) => {
      callback(hit);
    });
  }

  render() {
    const gridState = this.gridState;
    const canvas = this.canvas;
    if (!canvas || typeof canvas.getContext !== "function") {
      this.lastRenderSummary = {
        schema: "seamgrim.nurimaker.grid_render.v1",
        grid_w: gridState.gridW,
        grid_h: gridState.gridH,
        tile_size: gridState.tileSize,
        tile_counts: summarizeTileCounts(gridState.tiles),
        non_zero_cells: nonZeroTileCells(gridState.tiles),
        actor_cells: this.actorList.map((actor) => ({
          id: actor.id,
          row: actor.row,
          col: actor.col,
        })),
      };
      return this.lastRenderSummary;
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("NurimakerGridRenderer: 2d context를 가져오지 못했습니다.");
    }

    const width = gridState.gridW * gridState.tileSize;
    const height = gridState.gridH * gridState.tileSize;
    canvas.width = width;
    canvas.height = height;

    if (typeof ctx.clearRect === "function") {
      ctx.clearRect(0, 0, width, height);
    }
    ctx.fillStyle = gridState.background;
    if (typeof ctx.fillRect === "function") {
      ctx.fillRect(0, 0, width, height);
    }

    let tileDrawCount = 0;
    let actorDrawCount = 0;
    for (let row = 0; row < gridState.gridH; row += 1) {
      for (let col = 0; col < gridState.gridW; col += 1) {
        const kind = toFiniteInt(gridState.tiles[row]?.[col], 0, 0);
        const fill = String(gridState.palette[String(kind)] ?? DEFAULT_TILE_PALETTE[0]);
        const x = col * gridState.tileSize;
        const y = row * gridState.tileSize;
        ctx.fillStyle = fill;
        ctx.fillRect(x, y, gridState.tileSize, gridState.tileSize);
        if (typeof ctx.strokeRect === "function") {
          ctx.strokeStyle = gridState.gridLineColor;
          ctx.strokeRect(x, y, gridState.tileSize, gridState.tileSize);
        }
        tileDrawCount += 1;
      }
    }

    this.actorList.forEach((actor) => {
      if (actor.row >= gridState.gridH || actor.col >= gridState.gridW) return;
      const inset = Math.max(2, Math.floor(gridState.tileSize * 0.18));
      const x = actor.col * gridState.tileSize + inset;
      const y = actor.row * gridState.tileSize + inset;
      const size = Math.max(2, gridState.tileSize - inset * 2);
      ctx.fillStyle = actor.fill;
      ctx.fillRect(x, y, size, size);
      if (typeof ctx.fillText === "function") {
        ctx.fillStyle = "#111111";
        ctx.font = `${Math.max(8, Math.floor(gridState.tileSize * 0.42))}px sans-serif`;
        ctx.fillText(String(actor.label), x + Math.max(1, Math.floor(size * 0.22)), y + Math.max(10, Math.floor(size * 0.72)));
      }
      actorDrawCount += 1;
    });

    this.lastRenderSummary = {
      schema: "seamgrim.nurimaker.grid_render.v1",
      canvas_width: width,
      canvas_height: height,
      grid_w: gridState.gridW,
      grid_h: gridState.gridH,
      tile_size: gridState.tileSize,
      tile_counts: summarizeTileCounts(gridState.tiles),
      non_zero_cells: nonZeroTileCells(gridState.tiles),
      actor_cells: this.actorList.map((actor) => ({
        id: actor.id,
        row: actor.row,
        col: actor.col,
      })),
      palette_keys: Object.keys(gridState.palette).sort((a, b) => Number(a) - Number(b)),
      draw_calls: {
        background: 1,
        tiles: tileDrawCount,
        actors: actorDrawCount,
      },
    };
    return this.lastRenderSummary;
  }
}

export { DEFAULT_TILE_PALETTE };
