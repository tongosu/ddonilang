import { instantiateBlock, renderExprNodeText } from "./ddn_block_codec.js";

function defaultCreateElement(tagName) {
  if (typeof document !== "undefined" && typeof document.createElement === "function") {
    return document.createElement(tagName);
  }
  return {
    tagName: String(tagName ?? "").toUpperCase(),
    className: "",
    textContent: "",
    dataset: {},
    children: [],
    value: "",
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    replaceChildren(...items) {
      this.children = items;
    },
    addEventListener() {},
  };
}

function clearNode(node) {
  if (!node) return;
  if (typeof node.replaceChildren === "function") {
    node.replaceChildren();
    return;
  }
  if (Array.isArray(node.children)) {
    node.children = [];
  }
}

function ensureArray(value) {
  return Array.isArray(value) ? value : [];
}

function ensureObject(value) {
  return value && typeof value === "object" ? value : {};
}

function summarizeExprNode(node, depth = 0, out = []) {
  if (!node || typeof node !== "object") return out;
  out.push({
    depth,
    kind: String(node.kind ?? ""),
    text: String(node.text ?? ""),
  });
  Object.values(ensureObject(node.inputs)).forEach((children) => {
    ensureArray(children).forEach((child) => summarizeExprNode(child, depth + 1, out));
  });
  return out;
}

function summarizeBlockTree(block, depth = 0, out = []) {
  if (!block || typeof block !== "object") return out;
  const exprEntries = Object.entries(ensureObject(block.exprs));
  out.push({
    id: String(block.id ?? ""),
    kind: String(block.kind ?? ""),
    depth,
    expr_keys: exprEntries.map(([key]) => String(key)),
    expr_kinds: exprEntries.flatMap(([, expr]) => summarizeExprNode(expr).map((item) => item.kind)),
  });
  Object.values(ensureObject(block.inputs)).forEach((children) => {
    ensureArray(children).forEach((child) => summarizeBlockTree(child, depth + 1, out));
  });
  return out;
}

function summarizeBlockExprTree(block, out = []) {
  if (!block || typeof block !== "object") return out;
  Object.values(ensureObject(block.exprs)).forEach((expr) => {
    summarizeExprNode(expr, 0, out);
  });
  Object.values(ensureObject(block.inputs)).forEach((children) => {
    ensureArray(children).forEach((child) => summarizeBlockExprTree(child, out));
  });
  return out;
}

function findBlockById(blocks, blockId) {
  const targetId = String(blockId ?? "");
  for (const block of ensureArray(blocks)) {
    if (String(block?.id ?? "") === targetId) {
      return block;
    }
    const nested = Object.values(ensureObject(block?.inputs)).flatMap((children) => ensureArray(children));
    const hit = findBlockById(nested, targetId);
    if (hit) return hit;
  }
  return null;
}

function findExprNodeByPath(node, exprPath = []) {
  let current = node;
  const path = Array.isArray(exprPath) ? exprPath : [];
  if (!current || typeof current !== "object") return null;
  for (let index = 0; index < path.length; index += 2) {
    const inputKey = String(path[index] ?? "").trim();
    const itemIndex = Number(path[index + 1] ?? 0);
    const children = ensureArray(current?.inputs?.[inputKey]);
    current = children[itemIndex] ?? null;
    if (!current) return null;
  }
  return current;
}

function refreshExprNodeText(node) {
  if (!node || typeof node !== "object") return "";
  Object.values(ensureObject(node.inputs)).forEach((children) => {
    ensureArray(children).forEach((child) => {
      refreshExprNodeText(child);
    });
  });
  node.text = renderExprNodeText(node);
  return node.text;
}

export class DdnBlockEngine {
  constructor({ paletteEl = null, canvasEl = null, createElement = defaultCreateElement } = {}) {
    this.paletteEl = paletteEl;
    this.canvasEl = canvasEl;
    this.createElement = typeof createElement === "function" ? createElement : defaultCreateElement;
    this.onChange = null;
    this._palette = null;
    this._blocks = [];
    this.lastPaletteSummary = null;
    this.lastCanvasSummary = null;
  }

  init(palette, blocks) {
    this._palette = palette;
    this._blocks = ensureArray(blocks);
    this.render();
  }

  getBlocks() {
    return JSON.parse(JSON.stringify(this._blocks));
  }

  setBlocks(blocks) {
    this._blocks = ensureArray(blocks);
    this.render();
  }

  appendPaletteBlock(blockDef) {
    const next = instantiateBlock(blockDef);
    this._blocks.push(next);
    this.render();
    this.onChange?.(this.getBlocks());
    return next;
  }

  updateField(blockId, fieldId, value) {
    const block = findBlockById(this._blocks, blockId);
    if (!block) return false;
    if (!block.fields || typeof block.fields !== "object") {
      block.fields = {};
    }
    block.fields[String(fieldId ?? "")] = String(value ?? "");
    this.render();
    this.onChange?.(this.getBlocks());
    return true;
  }

  updateExprText(blockId, exprKey, value) {
    const block = findBlockById(this._blocks, blockId);
    if (!block) return false;
    if (!block.fields || typeof block.fields !== "object") {
      block.fields = {};
    }
    if (!block.exprs || typeof block.exprs !== "object") {
      block.exprs = {};
    }
    const key = String(exprKey ?? "").trim();
    const nextValue = String(value ?? "");
    if (!key) return false;
    block.fields[key] = nextValue;
    block.exprs[key] = {
      kind: "manual_text",
      text: nextValue,
      fields: {},
      inputs: {},
    };
    this.render();
    this.onChange?.(this.getBlocks());
    return true;
  }

  updateExprNodeField(blockId, exprKey, exprPath, fieldKey, value) {
    const block = findBlockById(this._blocks, blockId);
    if (!block) return false;
    const key = String(exprKey ?? "").trim();
    const targetField = String(fieldKey ?? "").trim();
    if (!key || !targetField) return false;
    const rootExpr = block?.exprs?.[key];
    const targetExpr = findExprNodeByPath(rootExpr, exprPath);
    if (!targetExpr) return false;
    if (!targetExpr.fields || typeof targetExpr.fields !== "object") {
      targetExpr.fields = {};
    }
    targetExpr.fields[targetField] = String(value ?? "");
    refreshExprNodeText(rootExpr);
    if (!block.fields || typeof block.fields !== "object") {
      block.fields = {};
    }
    block.fields[key] = String(rootExpr.text ?? "");
    this.render();
    this.onChange?.(this.getBlocks());
    return true;
  }

  removeBlock(blockId) {
    const before = this._blocks.length;
    this._blocks = this._blocks.filter((item) => String(item?.id ?? "") !== String(blockId ?? ""));
    if (this._blocks.length === before) return false;
    this.render();
    this.onChange?.(this.getBlocks());
    return true;
  }

  render() {
    this._renderPalette();
    this._renderCanvas();
  }

  _renderPalette() {
    const categories = ensureArray(this._palette?.categories);
    this.lastPaletteSummary = {
      category_count: categories.length,
      category_ids: categories.map((category) => String(category?.id ?? "")),
      block_kinds: categories.flatMap((category) =>
        ensureArray(category?.blocks).map((block) => String(block?.kind ?? "")),
      ),
    };
    clearNode(this.paletteEl);
    if (!this.paletteEl) return;
    categories.forEach((category) => {
      const label = this.createElement("div");
      label.className = "block-category__label";
      label.textContent = String(category?.label ?? category?.id ?? "");
      this.paletteEl.appendChild(label);
      ensureArray(category?.blocks).forEach((blockDef) => {
        const button = this.createElement("button");
        button.className = "block-category__item";
        button.textContent = String(blockDef?.label ?? blockDef?.kind ?? "");
        button.dataset = { kind: String(blockDef?.kind ?? "") };
        if (typeof button.addEventListener === "function") {
          button.addEventListener("click", () => {
            this.appendPaletteBlock(blockDef);
          });
        }
        this.paletteEl.appendChild(button);
      });
    });
  }

  _renderCanvas() {
    const blocks = ensureArray(this._blocks);
    const blockTree = blocks.flatMap((block) => summarizeBlockTree(block));
    const exprTree = blocks.flatMap((block) => summarizeBlockExprTree(block));
    this.lastCanvasSummary = {
      block_count: blocks.length,
      block_kinds: blocks.map((block) => String(block?.kind ?? "")),
      block_ids: blocks.map((block) => String(block?.id ?? "")),
      total_block_count: blockTree.length,
      total_expr_count: exprTree.length,
      nested_blocks: blockTree,
      expr_nodes: exprTree,
    };
    clearNode(this.canvasEl);
    if (!this.canvasEl) return;
    blocks.forEach((block) => {
      this.canvasEl.appendChild(this._renderBlockCard(block, 0));
    });
  }

  _renderBlockCard(block, depth = 0) {
    const card = this.createElement("div");
    card.className = `ddn-block ddn-block--${String(block?.kind ?? "unknown")}`;
    card.dataset = {
      ...(card.dataset ?? {}),
      depth: String(depth),
      blockId: String(block?.id ?? ""),
    };

    const header = this.createElement("div");
    header.className = "ddn-block__header";
    header.textContent = String(block?.label ?? block?.kind ?? "");
    card.appendChild(header);

    const fieldsWrap = this.createElement("div");
    fieldsWrap.className = "ddn-block__fields";
    Object.entries(ensureObject(block?.fields)).forEach(([key, value]) => {
      const row = this.createElement("label");
      row.className = "ddn-block__field";
      row.textContent = `${key}: ${String(value ?? "")}`;
      fieldsWrap.appendChild(row);
    });
    if (fieldsWrap.children?.length) {
      card.appendChild(fieldsWrap);
    }

    const exprEntries = Object.entries(ensureObject(block?.exprs));
    if (exprEntries.length) {
      const exprWrap = this.createElement("div");
      exprWrap.className = "ddn-block__exprs";
      exprEntries.forEach(([key, expr]) => {
        const slot = this.createElement("div");
        slot.className = "ddn-block__expr-slot";
        const label = this.createElement("div");
        label.className = "ddn-block__expr-label";
        label.textContent = `${String(key)} 식`;
        slot.appendChild(label);
        const input = this.createElement("input");
        input.className = "ddn-block__expr-input";
        input.value = String(block?.fields?.[key] ?? expr?.text ?? "");
        if (typeof input.addEventListener === "function") {
          const commit = (event) => {
            this.updateExprText(block?.id, key, event?.target?.value ?? input.value ?? "");
          };
          input.addEventListener("change", commit);
          input.addEventListener("input", commit);
        }
        slot.appendChild(input);
        slot.appendChild(this._renderExprNode(expr, block?.id, key, [], 0));
        exprWrap.appendChild(slot);
      });
      card.appendChild(exprWrap);
    }

    const inputEntries = Object.entries(ensureObject(block?.inputs));
    if (inputEntries.length) {
      const inputsWrap = this.createElement("div");
      inputsWrap.className = "ddn-block__inputs";
      inputEntries.forEach(([key, value]) => {
        const children = ensureArray(value);
        if (!children.length) return;
        const lane = this.createElement("div");
        lane.className = "ddn-block__input-lane";
        const label = this.createElement("div");
        label.className = "ddn-block__input-label";
        label.textContent = String(key);
        lane.appendChild(label);
        children.forEach((child) => {
          lane.appendChild(this._renderBlockCard(child, depth + 1));
        });
        inputsWrap.appendChild(lane);
      });
      if (inputsWrap.children?.length) {
        card.appendChild(inputsWrap);
      }
    }

    return card;
  }

  _renderExprNode(node, blockId, exprKey, exprPath = [], depth = 0) {
    const wrap = this.createElement("div");
    wrap.className = "ddn-expr";
    wrap.dataset = {
      ...(wrap.dataset ?? {}),
      depth: String(depth),
      kind: String(node?.kind ?? ""),
    };

    const head = this.createElement("div");
    head.className = "ddn-expr__head";
    head.textContent = `${String(node?.kind ?? "expr")}: ${String(node?.text ?? "")}`;
    wrap.appendChild(head);

    Object.entries(ensureObject(node?.fields)).forEach(([key, value]) => {
      const row = this.createElement("label");
      row.className = "ddn-expr__field";
      const input = this.createElement("input");
      input.className = "ddn-expr__field-input";
      input.value = String(value ?? "");
      if (typeof input.addEventListener === "function") {
        const commit = (event) => {
          this.updateExprNodeField(blockId, exprKey, exprPath, key, event?.target?.value ?? input.value ?? "");
        };
        input.addEventListener("change", commit);
        input.addEventListener("input", commit);
      }
      row.textContent = `${String(key)}: `;
      row.appendChild(input);
      wrap.appendChild(row);
    });

    Object.entries(ensureObject(node?.inputs)).forEach(([key, value]) => {
      const children = ensureArray(value);
      if (!children.length) return;
      const lane = this.createElement("div");
      lane.className = "ddn-expr__input";
      const label = this.createElement("div");
      label.className = "ddn-expr__input-label";
      label.textContent = String(key);
      lane.appendChild(label);
      children.forEach((child, childIndex) => {
        lane.appendChild(this._renderExprNode(child, blockId, exprKey, [...exprPath, key, childIndex], depth + 1));
      });
      wrap.appendChild(lane);
    });

    return wrap;
  }
}
