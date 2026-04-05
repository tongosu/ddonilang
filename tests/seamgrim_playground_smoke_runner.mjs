#!/usr/bin/env node
/**
 * seamgrim_playground_smoke_runner.mjs
 *
 * 셈그림 플레이그라운드 핵심 흐름 스모크 러너 (WASM-only, 서버 불필요)
 *
 * 검증 항목 (케이스별):
 *   1. wasm_canon_maegim_plan() → 매김{} 컨트롤 추출
 *   2. wasm_canon_flat_json()   → 구성 flat JSON 생성
 *   3. DdnWasmVm 생성 + 틱 실행 → 결정론 확인 (2회 실행 동일 결과)
 *   4. 관찰값 채널 존재 확인
 *   5. 파라미터 오버라이드 → 다른 state_hash 생성 확인
 *
 * 사용법:
 *   node --no-warnings tests/seamgrim_playground_smoke_runner.mjs
 *   node --no-warnings tests/seamgrim_playground_smoke_runner.mjs pack/seamgrim_playground_smoke_v1
 *   node --no-warnings tests/seamgrim_playground_smoke_runner.mjs pack/seamgrim_playground_smoke_v1 --update
 */

import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const ROOT_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const UI_DIR = path.join(ROOT_DIR, "solutions", "seamgrim_ui_mvp", "ui");
const WASM_DIR = path.join(UI_DIR, "wasm");
const DEFAULT_PLAYGROUND_SMOKE_PACK_DIR = path.join(ROOT_DIR, "pack", "seamgrim_playground_smoke_v1");

// ── WASM 모듈 초기화 ─────────────────────────────────────────────────────────

async function loadWasmModule() {
  const wasmModUrl = pathToFileURL(path.join(WASM_DIR, "ddonirang_tool.js")).href;
  const wasmBytes = await fs.readFile(path.join(WASM_DIR, "ddonirang_tool_bg.wasm"));
  const mod = await import(wasmModUrl);
  if (typeof mod.default === "function") {
    await mod.default({ module_or_path: wasmBytes });
  }
  return mod;
}

async function loadWrappers() {
  const wrapperUrl = pathToFileURL(path.join(UI_DIR, "wasm_ddn_wrapper.js")).href;
  const runtimeStateUrl = pathToFileURL(path.join(UI_DIR, "seamgrim_runtime_state.js")).href;
  const canonRuntimeUrl = pathToFileURL(path.join(UI_DIR, "runtime", "wasm_canon_runtime.js")).href;
  const controlParserUrl = pathToFileURL(path.join(UI_DIR, "components", "control_parser.js")).href;

  const [wrapper, , canonRuntimeMod, controlParserMod] = await Promise.all([
    import(wrapperUrl),
    import(runtimeStateUrl),
    import(canonRuntimeUrl),
    import(controlParserUrl),
  ]);
  return { wrapper, canonRuntimeMod, controlParserMod };
}

// ── 메타 헤더 제거 (VM 입력용) ───────────────────────────────────────────────

/**
 * 채비 선언의 "매김 { ... }." 어노테이션 블록을 제거하고 "." 로 교체.
 * lang/src/parser.rs 의 DdnWasmVm 파서가 매김{} 미지원이므로
 * VM 실행 전 전처리로 제거한다. canon 추출은 이미 완료 후라 정보 손실 없음.
 *
 * 예:
 *   "g: 수 = (9.8) 매김 { 범위: 1..20. 간격: 0.1. }."
 *   → "g: 수 = (9.8)."
 */
function stripMaegimAnnotations(text) {
  let result = "";
  let i = 0;
  while (i < text.length) {
    const maegimIdx = text.indexOf("매김", i);
    if (maegimIdx === -1) {
      result += text.slice(i);
      break;
    }
    let j = maegimIdx + "매김".length;
    // 공백 건너뜀
    while (j < text.length && " \t\n\r".includes(text[j])) j++;
    if (j < text.length && text[j] === "{") {
      // 매김 앞까지 복사
      result += text.slice(i, maegimIdx);
      // 중괄호 매칭으로 블록 끝 탐색
      let depth = 0;
      let k = j;
      while (k < text.length) {
        if (text[k] === "{") { depth++; k++; }
        else if (text[k] === "}") {
          depth--;
          k++;
          if (depth === 0) {
            // } 이후 공백 건너뜀
            while (k < text.length && " \t\n\r".includes(text[k])) k++;
            // 선언 종결자 . 건너뜀
            if (k < text.length && text[k] === ".") k++;
            break;
          }
        } else { k++; }
      }
      result += "."; // 선언 종결자 복원
      i = k;
    } else {
      // 매김이지만 { 없음 — 그대로 포함
      result += text.slice(i, j);
      i = j;
    }
  }
  return result;
}

function stripMetaHeader(text) {
  const lines = String(text ?? "")
    .replace(/\r\n/g, "\n")
    .replace(/\r/g, "\n")
    .split("\n");
  let idx = 0;
  while (idx < lines.length) {
    const trimmed = lines[idx].replace(/^[ \t\uFEFF]+/, "");
    if (!trimmed) { idx++; continue; }
    if (trimmed.startsWith("#") && trimmed.includes(":")) { idx++; continue; }
    break;
  }
  return lines.slice(idx).join("\n");
}

// ── VM 틱 실행 ───────────────────────────────────────────────────────────────

function runTicks(wasmMod, wrapper, sourceText, maxTicks, paramOverride) {
  const source = stripMaegimAnnotations(stripMetaHeader(sourceText)).trim();
  const vm = new wasmMod.DdnWasmVm(source);
  const client = new wrapper.DdnWasmVmClient(vm);
  try {
    client.resetParsed(false);

    // 파라미터 오버라이드 적용
    if (paramOverride && typeof paramOverride === "object") {
      for (const [key, val] of Object.entries(paramOverride)) {
        try {
          client.setParamParsed(key, val);
        } catch (_) { /* 오버라이드 실패 무시 */ }
      }
    }

    const rows = [];
    for (let i = 0; i < maxTicks; i++) {
      const result = client.stepOneParsed();
      if (!result) break;
      rows.push({
        tick: Number(result.tick_id ?? -1),
        state_hash: String(result.state_hash ?? ""),
      });
    }

    // 관찰 채널 수집 (마지막 row의 columns 기준)
    let channels = [];
    try {
      const colData = client.columnsParsed();
      channels = (colData?.columns ?? []).map((c) => String(c?.key ?? "")).filter(Boolean);
    } catch (_) { /* 무시 */ }

    return { rows, channels };
  } finally {
    if (typeof vm.free === "function") vm.free();
  }
}

// ── 케이스 실행 ──────────────────────────────────────────────────────────────

async function runCase(caseSpec, packDir, wasmMod, wrapper, canonRuntime, controlParser) {
  const { id, ddn_path, expect_slider_count_min, expect_maegim_count,
          expect_channels_any, max_ticks, param_override } = caseSpec;

  const lessonPath = path.resolve(packDir, ddn_path);
  const sourceText = await fs.readFile(lessonPath, "utf8");

  // 1. wasm_canon_maegim_plan
  let maegimPlan = null;
  let maegimOk = false;
  let maegimError = null;
  try {
    maegimPlan = await canonRuntime.canonMaegimPlan(sourceText);
    maegimOk = maegimPlan?.schema === "ddn.maegim_control_plan.v1";
  } catch (e) {
    maegimError = String(e?.message ?? e);
  }
  const maegimCount = Array.isArray(maegimPlan?.controls) ? maegimPlan.controls.length : 0;

  // 2. wasm_canon_flat_json
  let flatOk = false;
  let flatError = null;
  try {
    const flat = await canonRuntime.canonFlatJson(sourceText);
    flatOk = flat?.schema === "ddn.guseong_flatten_plan.v1";
  } catch (e) {
    flatError = String(e?.message ?? e);
  }

  // 3. 슬라이더 스펙 (buildControlSpecsFromDdn — 구형 //범위 + 신형 매김{} 모두 처리)
  let sliderCount = 0;
  let sliderSource = "none";
  try {
    const maegimJson = maegimOk ? JSON.stringify(maegimPlan) : "";
    const parsed = controlParser.buildControlSpecsFromDdn(sourceText, { maegimControlJson: maegimJson });
    sliderCount = Array.isArray(parsed?.specs) ? parsed.specs.length : 0;
    sliderSource = String(parsed?.source ?? "none");
  } catch (_) { /* 무시 */ }

  // 4. 틱 실행 (1회차)
  let ticksOk = false;
  let tickError = null;
  let rowsA = [];
  let channelsFound = [];
  try {
    const res = runTicks(wasmMod, wrapper, sourceText, max_ticks, null);
    rowsA = res.rows;
    channelsFound = res.channels;
    ticksOk = rowsA.length > 0;
  } catch (e) {
    tickError = String(e?.message ?? e);
  }

  // 5. 결정론 확인 (2회차)
  let deterministicOk = false;
  if (ticksOk) {
    try {
      const resB = runTicks(wasmMod, wrapper, sourceText, max_ticks, null);
      deterministicOk = JSON.stringify(rowsA) === JSON.stringify(resB.rows);
    } catch (_) { /* 무시 */ }
  }

  // 6. 파라미터 오버라이드 → 다른 hash
  let paramOverrideOk = null;
  let hashBase = rowsA.length > 0 ? rowsA[rowsA.length - 1].state_hash : "";
  if (param_override && ticksOk) {
    try {
      const resOverride = runTicks(wasmMod, wrapper, sourceText, max_ticks, param_override);
      const hashOverride = resOverride.rows.length > 0
        ? resOverride.rows[resOverride.rows.length - 1].state_hash
        : "";
      paramOverrideOk = hashOverride !== "" && hashOverride !== hashBase;
    } catch (_) {
      paramOverrideOk = false;
    }
  }

  // 검증 결과
  const sliderOk = sliderCount >= (expect_slider_count_min ?? 0);
  const maegimCountOk = expect_maegim_count === null || expect_maegim_count === undefined
    ? true
    : maegimCount === expect_maegim_count;
  const channelsOk = !Array.isArray(expect_channels_any) || expect_channels_any.length === 0
    ? true
    : expect_channels_any.some((ch) => channelsFound.includes(ch));

  const allOk = maegimOk && flatOk && ticksOk && deterministicOk && sliderOk && maegimCountOk && channelsOk
    && (paramOverrideOk === null || paramOverrideOk === true);

  return {
    id,
    ok: allOk,
    maegim_ok: maegimOk,
    maegim_error: maegimError,
    maegim_count: maegimCount,
    maegim_count_ok: maegimCountOk,
    flat_ok: flatOk,
    flat_error: flatError,
    slider_count: sliderCount,
    slider_source: sliderSource,
    slider_ok: sliderOk,
    ticks_run: rowsA.length,
    ticks_ok: ticksOk,
    tick_error: tickError,
    deterministic_ok: deterministicOk,
    channels_found: channelsFound,
    channels_ok: channelsOk,
    state_hash_final: hashBase,
    param_override_ok: paramOverrideOk,
  };
}

// ── 메인 ─────────────────────────────────────────────────────────────────────

async function main() {
  const args = process.argv.slice(2);
  const packArg = args.find((a) => !a.startsWith("--"));
  const updateMode = args.includes("--update");
  const packDir = packArg ? path.resolve(packArg) : DEFAULT_PLAYGROUND_SMOKE_PACK_DIR;
  if (!packArg) {
    process.stderr.write(`[info] pack_dir 미지정: 기본 경로 사용 (${packDir})\n`);
  }
  const fixturePath = path.join(packDir, "fixture.json");
  const fixture = JSON.parse(await fs.readFile(fixturePath, "utf8"));

  if (fixture.schema !== "ddn.seamgrim_playground_smoke.v1") {
    throw new Error(`fixture schema 불일치: ${fixture.schema}`);
  }

  // 모듈 로드
  const [wasmMod, { wrapper, canonRuntimeMod, controlParserMod }] = await Promise.all([
    loadWasmModule(),
    loadWrappers(),
  ]);

  const canonRuntime = await canonRuntimeMod.createWasmCanon({
    wasmUrl: pathToFileURL(path.join(WASM_DIR, "ddonirang_tool.js")).href,
    initInput: await fs.readFile(path.join(WASM_DIR, "ddonirang_tool_bg.wasm")),
    cacheBust: "smoke",
  });

  if (typeof controlParserMod.buildControlSpecsFromDdn !== "function") {
    throw new Error("buildControlSpecsFromDdn export 누락");
  }

  // 케이스 실행
  const cases = Array.isArray(fixture.cases) ? fixture.cases : [];
  const results = [];
  for (const caseSpec of cases) {
    const result = await runCase(caseSpec, packDir, wasmMod, wrapper, canonRuntime, controlParserMod);
    results.push(result);
  }

  const allOk = results.every((r) => r.ok);
  const output = {
    schema: "ddn.seamgrim_playground_smoke_result.v1",
    all_ok: allOk,
    case_count: results.length,
    pass_count: results.filter((r) => r.ok).length,
    cases: results,
  };

  const outputPath = path.join(packDir, "expected", "playground_smoke.detjson");

  if (updateMode) {
    await fs.mkdir(path.dirname(outputPath), { recursive: true });
    await fs.writeFile(outputPath, JSON.stringify(output, null, 2) + "\n", "utf8");
    process.stdout.write(`[update] ${outputPath}\n`);
    return;
  }

  // 검증 모드
  process.stdout.write(JSON.stringify({
    pack_id: "seamgrim_playground_smoke_v1",
    outputs: {
      "expected/playground_smoke.detjson": output,
    },
  }, null, 2) + "\n");

  if (!allOk) {
    const failed = results.filter((r) => !r.ok).map((r) => r.id);
    process.stderr.write(`실패 케이스: ${failed.join(", ")}\n`);
    process.exit(1);
  }
}

main().catch((err) => {
  process.stderr.write(`${String(err?.stack ?? err)}\n`);
  process.exit(1);
});
