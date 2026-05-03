import { findPaletteBlock, SEAMGRIM_PALETTE } from "./seamgrim_palette.js";
import { RPGBOX_PALETTE } from "./rpgbox_palette.js";

let nextBlockId = 1;

function createBlockId(prefix = "b") {
  const token = String(prefix ?? "b").trim() || "b";
  const id = `${token}_${String(nextBlockId).padStart(4, "0")}`;
  nextBlockId += 1;
  return id;
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function normalizeFieldValue(value) {
  if (value === undefined || value === null) return "";
  return String(value);
}

function replaceTemplateFields(template, fieldValues) {
  let out = String(template ?? "");
  Object.entries(fieldValues).forEach(([key, value]) => {
    out = out.replaceAll(`{${key}}`, normalizeFieldValue(value));
  });
  return out;
}

function indentLines(text, prefix = "  ") {
  return String(text ?? "")
    .split(/\r?\n/)
    .map((line) => (line.trim() ? `${prefix}${line}` : line))
    .join("\n");
}

function renderChildStatements(blocks) {
  const lines = encodeBlocksToDdn(blocks).trimEnd();
  if (!lines) return "  # 비어 있음";
  return indentLines(lines, "  ");
}

function createRawBlock(rawText, { id, source = "raw_fallback", label = "원시" } = {}) {
  return {
    id: String(id ?? createBlockId("raw")),
    kind: "raw",
    label: String(label ?? "원시"),
    fields: {},
    exprs: {},
    inputs: {},
    template: "",
    source: String(source ?? "raw_fallback"),
    rawText: String(rawText ?? ""),
  };
}

export function instantiateBlock(blockDef, overrides = {}) {
  const def = blockDef && typeof blockDef === "object" ? blockDef : {};
  const fieldDefs = Array.isArray(def.fields) ? def.fields : [];
  const inputDefs = Array.isArray(def.inputs) ? def.inputs : [];
  const fields = {};
  fieldDefs.forEach((field) => {
    const fieldId = String(field?.id ?? "").trim();
    if (!fieldId) return;
    fields[fieldId] = normalizeFieldValue(overrides?.fields?.[fieldId] ?? field?.default ?? "");
  });
  const inputs = {};
  inputDefs.forEach((input) => {
    const inputId = String(input?.id ?? "").trim();
    if (!inputId) return;
    inputs[inputId] = Array.isArray(overrides?.inputs?.[inputId]) ? cloneJson(overrides.inputs[inputId]) : [];
  });
  return {
    id: String(overrides?.id ?? createBlockId(def.kind || "block")),
    kind: String(def.kind ?? "raw"),
    label: String(def.label ?? def.kind ?? "raw"),
    fields,
    exprs: overrides?.exprs && typeof overrides.exprs === "object" ? cloneJson(overrides.exprs) : {},
    inputs,
    template: String(def.template ?? "{text}"),
    source: String(overrides?.source ?? "palette"),
    rawText: String(overrides?.rawText ?? ""),
  };
}

function encodeSingleBlock(block) {
  if (!block || typeof block !== "object") return "";
  if (String(block.kind ?? "") === "raw") {
    return String(block.rawText ?? "").trim();
  }
  if (String(block.kind ?? "") === "bogae_madang_block") {
    return renderRawBodyBlock("보개마당", block.fields?.body);
  }
  if (String(block.kind ?? "") === "exec_policy_block") {
    return renderRawBodyBlock("실행정책", block.fields?.body);
  }
  if (String(block.kind ?? "") === "jjaim_block") {
    return renderRawBodyBlock("짜임", block.fields?.body);
  }
  if (String(block.kind ?? "") === "seed_def") {
    const params = String(block.fields?.params ?? "").trim();
    const name = String(block.fields?.name ?? "").trim();
    const kind = String(block.fields?.kind ?? "").trim();
    const body = renderChildStatements(block.inputs?.body ?? []);
    const head = params ? `${params} ${name}:${kind}` : `${name}:${kind}`;
    return `${head} = {\n${body}\n}.`.trim();
  }
  if (String(block.kind ?? "") === "receive_hook_plan") {
    return `# plan ${block.fields?.scope ?? ""} ${block.fields?.kind ?? ""}\n# ${block.fields?.body_canon ?? ""}`.trim();
  }
  if (String(block.kind ?? "") === "choose_branch") {
    const cond = String(block.fields?.cond ?? "");
    const body = renderChildStatements(block.inputs?.body ?? []);
    return `${cond}: {\n${body}\n}`.trim();
  }
  if (String(block.kind ?? "") === "choose_else") {
    const branches = renderChildStatements(block.inputs?.branches ?? []);
    const elseLines = encodeBlocksToDdn(block.inputs?.else ?? []).trimEnd();
    const elseBody = elseLines ? indentLines(elseLines, "    ") : "    # 비어 있음";
    return `고르기:\n${branches}\n  아니면: {\n${elseBody}\n  }.`.trim();
  }
  if (String(block.kind ?? "") === "choose_exhaustive") {
    const branches = renderChildStatements(block.inputs?.branches ?? []);
    return `고르기:\n${branches}\n  모든 경우 다룸.`.trim();
  }
  if (String(block.kind ?? "") === "contract_guard") {
    const cond = String(block.fields?.cond ?? "");
    const contractKind = String(block.fields?.contract_kind ?? "pre");
    const mode = String(block.fields?.mode ?? "abort");
    const keyword = contractKind === "post" ? "다짐하고" : "바탕으로";
    const modeSuffix = mode === "alert" ? "(알림)" : "";
    const elseLines = encodeBlocksToDdn(block.inputs?.else ?? []).trimEnd();
    const elseBody = elseLines ? indentLines(elseLines, "  ") : "  # 비어 있음";
    const thenLines = encodeBlocksToDdn(block.inputs?.then ?? []).trimEnd();
    let rendered = `${cond} ${keyword}${modeSuffix} 아니면 {\n${elseBody}\n}`;
    if (thenLines) {
      rendered += ` 맞으면 {\n${indentLines(thenLines, "  ")}\n}`;
    }
    rendered += ".";
    return rendered.trim();
  }
  if (String(block.kind ?? "") === "prompt_choose") {
    const branches = renderChildStatements(block.inputs?.branches ?? []);
    const elseLines = encodeBlocksToDdn(block.inputs?.else ?? []).trimEnd();
    let rendered = `??:\n${branches}`;
    if (elseLines) {
      rendered += `\n  ??: {\n${indentLines(elseLines, "    ")}\n  }`;
    }
    return rendered.trim();
  }
  if (String(block.kind ?? "") === "receive_block") {
    const kind = String(block.fields?.kind ?? "").trim();
    const binding = String(block.fields?.binding ?? "").trim();
    const condition = String(block.fields?.condition ?? "").trim();
    const body = renderChildStatements(block.inputs?.body ?? []);
    let rendered = "";
    if (binding) {
      rendered += `(${binding}`;
      if (condition) {
        rendered += ` ${condition}`;
      }
      rendered += ")인 ";
    }
    rendered += kind ? `${kind}${pickObjectParticle(kind)}` : "알림을";
    rendered += ` 받으면 {\n${body}\n}.`;
    return rendered.trim();
  }
  if (String(block.kind ?? "") === "send_signal") {
    const sender = String(block.fields?.sender ?? "").trim();
    const payload = String(block.fields?.payload ?? "").trim();
    const receiver = String(block.fields?.receiver ?? "").trim();
    if (!sender) {
      return `${payload} ~~> ${receiver}.`.trim();
    }
    return `(${sender})의 ${payload} ~~> ${receiver}.`.trim();
  }
  const fieldValues = { ...(block.fields && typeof block.fields === "object" ? block.fields : {}) };
  Object.entries(block.inputs && typeof block.inputs === "object" ? block.inputs : {}).forEach(([key, value]) => {
    fieldValues[key] = renderChildStatements(value);
  });
  return replaceTemplateFields(block.template, fieldValues).trim();
}

function pickObjectParticle(text) {
  const value = String(text ?? "").trim();
  if (!value) return "을";
  const last = value.charCodeAt(value.length - 1);
  if (last < 0xac00 || last > 0xd7a3) {
    return "을";
  }
  return (last - 0xac00) % 28 === 0 ? "를" : "을";
}

function renderRawBodyBlock(keyword, body) {
  const raw = String(body ?? "");
  return `${keyword} {${raw}}.`.trim();
}

export function renderExprNodeText(node) {
  const kind = String(node?.kind ?? "").trim();
  const text = String(node?.text ?? "").trim();
  const fields = node?.fields && typeof node.fields === "object" ? node.fields : {};
  const inputs = node?.inputs && typeof node.inputs === "object" ? node.inputs : {};
  const renderInputList = (key) =>
    (Array.isArray(inputs[key]) ? inputs[key] : []).map((child) => renderExprNodeText(child)).filter(Boolean);

  if (!kind) return text;
  if (kind === "manual_text") return text;
  if (kind === "literal") return String(fields.value ?? text);
  if (kind === "resource") return String(fields.name ?? text);
  if (kind === "path") return String(fields.path ?? text);
  if (kind === "binding") {
    const valueText = renderInputList("value")[0] ?? String(fields.value ?? text);
    return `${String(fields.name ?? "").trim()}=${valueText}`.trim();
  }
  if (kind === "pack") {
    return `(${renderInputList("bindings").join(", ")})`;
  }
  if (kind === "call") {
    const name = String(fields.name ?? "").trim();
    const args = renderInputList("args");
    if (!args.length) {
      return `() ${name}`.trim();
    }
    if (args.length === 1) {
      const only = String(args[0] ?? "").trim();
      const head = only.startsWith("(") && only.endsWith(")") ? only : `(${only})`;
      return `${head} ${name}`.trim();
    }
    return `(${args.join(", ")}) ${name}`.trim();
  }
  if (kind === "call_in") {
    const name = String(fields.name ?? "").trim();
    const bindings = renderInputList("bindings");
    return `${bindings.length ? `(${bindings.join(", ")})` : "()"} ${name}`.trim();
  }
  if (kind === "field_access") {
    const target = renderInputList("target")[0] ?? "";
    return `${target}.${String(fields.field ?? "").trim()}`.trim();
  }
  return text || String(fields.value ?? "");
}

export function encodeBlocksToDdn(blocks) {
  const list = Array.isArray(blocks) ? blocks : [];
  const rendered = list.map((block) => encodeSingleBlock(block)).filter(Boolean);
  if (!rendered.length) return "";
  return `${rendered.join("\n\n")}\n`;
}

export function decodeAlrimPlanToBlocks(alrimPlan) {
  const handlers = Array.isArray(alrimPlan?.handlers) ? alrimPlan.handlers : [];
  return handlers.map((handler) => ({
    id: createBlockId("plan"),
    kind: "receive_hook_plan",
    label: "알림 계획",
    fields: {
      order: String(handler?.order ?? ""),
      kind: String(handler?.kind ?? ""),
      scope: String(handler?.scope ?? ""),
      body_canon: String(handler?.body_canon ?? "").trim(),
    },
    inputs: {},
    template: "",
    source: "wasm_alrim_plan",
    rawText: "",
  }));
}

function decodeBlockEditorPlanNode(node, palette) {
  const kind = String(node?.kind ?? "").trim();
  if (!kind || kind === "raw") {
    return createRawBlock(String(node?.raw_text ?? ""), {
      source: "wasm_block_editor_plan",
    });
  }
  const hit = findBlockDefinition(kind, palette);
  if (!hit) {
    return createRawBlock(String(node?.raw_text ?? ""), {
      source: "wasm_block_editor_plan",
      label: `미지원:${kind}`,
    });
  }
  const inputOverrides = {};
  Object.entries(node?.inputs && typeof node.inputs === "object" ? node.inputs : {}).forEach(([key, value]) => {
    inputOverrides[key] = (Array.isArray(value) ? value : []).map((child) =>
      decodeBlockEditorPlanNode(child, palette),
    );
  });
  return instantiateBlock(hit.block, {
    source: "wasm_block_editor_plan",
    fields: node?.fields && typeof node.fields === "object" ? node.fields : {},
    exprs: node?.exprs && typeof node.exprs === "object" ? node.exprs : {},
    inputs: inputOverrides,
  });
}

export function decodeBlockEditorPlanToBlocks(blockPlan, palette = SEAMGRIM_PALETTE) {
  const blocks = Array.isArray(blockPlan?.blocks)
    ? blockPlan.blocks.map((node) => decodeBlockEditorPlanNode(node, palette))
    : [];
  return {
    blocks,
    errors: [],
    palette,
    blockPlan,
  };
}

function resolvePalette(mode = "seamgrim") {
  return String(mode ?? "").trim() === "rpg" ? RPGBOX_PALETTE : SEAMGRIM_PALETTE;
}

export async function decodeDdnToBlocks(ddnSource, { canon = null, mode = "seamgrim" } = {}) {
  const sourceText = String(ddnSource ?? "");
  const palette = resolvePalette(mode);
  if (String(mode ?? "").trim() !== "rpg") {
    if (!canon || typeof canon.canonBlockEditorPlan !== "function") {
      return {
        blocks: [createRawBlock(sourceText)],
        errors: [{ message: "canonBlockEditorPlan 미주입" }],
        palette,
        blockPlan: null,
      };
    }
    try {
      const blockPlan = await canon.canonBlockEditorPlan(sourceText);
      return decodeBlockEditorPlanToBlocks(blockPlan, palette);
    } catch (error) {
      return {
        blocks: [createRawBlock(sourceText)],
        errors: [{ message: String(error?.message ?? error) }],
        palette,
        blockPlan: null,
      };
    }
  }
  if (!canon || typeof canon.canonAlrimPlan !== "function") {
    return {
      blocks: [createRawBlock(sourceText)],
      errors: [{ message: "canonAlrimPlan 미주입" }],
      palette,
      alrimPlan: null,
    };
  }
  try {
    const alrimPlan = await canon.canonAlrimPlan(sourceText);
    return {
      blocks: decodeAlrimPlanToBlocks(alrimPlan),
      errors: [],
      palette,
      alrimPlan,
    };
  } catch (error) {
    return {
      blocks: [createRawBlock(sourceText)],
      errors: [{ message: String(error?.message ?? error) }],
      palette,
      alrimPlan: null,
    };
  }
}

export function findBlockDefinition(kind, palette = RPGBOX_PALETTE) {
  return findPaletteBlock(palette, kind);
}

export function buildPaletteBlock(kind, overrides = {}, palette = RPGBOX_PALETTE) {
  const hit = findBlockDefinition(kind, palette);
  if (!hit) {
    throw new Error(`palette block 정의 누락: ${kind}`);
  }
  return instantiateBlock(hit.block, overrides);
}
