#!/usr/bin/env node

import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function pointXY(point) {
  if (Array.isArray(point)) {
    return { x: Number(point[0]), y: Number(point[1]) };
  }
  return { x: Number(point?.x), y: Number(point?.y) };
}

function graphWithObjectPoints(graph) {
  return {
    ...graph,
    series: (Array.isArray(graph?.series) ? graph.series : []).map((row) => ({
      ...row,
      points: (Array.isArray(row?.points) ? row.points : []).map(pointXY),
    })),
  };
}

async function main() {
  const root = process.cwd();
  const uiDir = path.join(root, "solutions", "seamgrim_ui_mvp", "ui");
  const wasmModule = await import(pathToFileURL(path.join(uiDir, "wasm", "ddonirang_tool.js")).href);
  const wrapper = await import(pathToFileURL(path.join(uiDir, "wasm_ddn_wrapper.js")).href);
  const runtimeState = await import(pathToFileURL(path.join(uiDir, "seamgrim_runtime_state.js")).href);
  const graphAutorender = await import(pathToFileURL(path.join(uiDir, "graph_autorender.js")).href);
  const graphPreview = await import(pathToFileURL(path.join(uiDir, "components", "graph_preview.js")).href);
  const runMod = await import(pathToFileURL(path.join(uiDir, "screens", "run.js")).href);
  const wasmBytes = await fs.readFile(path.join(uiDir, "wasm", "ddonirang_tool_bg.wasm"));
  if (typeof wasmModule.default === "function") {
    await wasmModule.default({ module_or_path: wasmBytes });
  }

  const source = `
설정 {
  제목: 보개_마디수_그래프.
  설명: 마디수_40_보임_그래프_확인.
  마디수: 40.
}.

채비 {
  t: 수 <- 0.
  현재인덱스: 수 <- 0.
}.

(매마디)마다 {
  t <- t + 1.
  현재인덱스 <- 현재인덱스 + 1.
  보임 {
    x축: t.
    값: 현재인덱스.
  }.
}.

보개로 그려.
`;

  const vm = new wasmModule.DdnWasmVm(source);
  const client = new wrapper.DdnWasmVmClient(vm);
  try {
    assert(client.configuredMadi() === 40, `configuredMadi mismatch: ${client.configuredMadi()}`);
    let state = client.getStateParsed();
    for (let idx = 0; idx < client.configuredMadi(); idx += 1) {
      state = client.stepOneParsed();
    }
    const rows = runtimeState.extractObservationOutputRowsFromState(state);
    assert(rows.length >= 80, `expected output rows, got ${rows.length}`);
    assert(String(rows.at(-2)?.key ?? "") === "x축", "last x row key mismatch");
    assert(String(rows.at(-2)?.value ?? "") === "40", "last x row value mismatch");
    assert(String(rows.at(-1)?.key ?? "") === "값", "last value row key mismatch");
    assert(String(rows.at(-1)?.value ?? "") === "40", "last value row value mismatch");

    const graph = graphAutorender.buildGraphFromValueResources(state);
    assert(graph?.schema === "seamgrim.graph.v0", "graph schema missing");
    const series = Array.isArray(graph?.series) ? graph.series : [];
    assert(series.length === 1, `expected one series, got ${series.length}`);
    assert(Array.isArray(series[0]?.points), "graph points missing");
    assert(series[0].points.length === 40, `expected 40 graph points, got ${series[0].points.length}`);
    const firstPoint = pointXY(series[0].points[0]);
    const lastPoint = pointXY(series[0].points[39]);
    assert(firstPoint.x === 1 && firstPoint.y === 1, "first point mismatch");
    assert(lastPoint.x === 40 && lastPoint.y === 40, "last point mismatch");

    const html = graphPreview.buildGraphPreviewHtml(graphWithObjectPoints(graph));
    assert(String(html).includes("<svg"), "graph preview svg missing");
    assert(String(html).length > 200, "graph preview appears blank");

    const legacyRepresentative = `설정 {
  title: rep-cs-linear-search-timeline.
  desc: representative cs hybrid lesson.
}.

채비 {
  데이터길이: 수 <- (12) 매김 { 범위: 4..40. 간격: 1. }.
  목표인덱스: 수 <- (7) 매김 { 범위: 0..39. 간격: 1. }.
  최대마디: 수 <- (40) 매김 { 범위: 4..120. 간격: 1. }.
}.

(시작)할때 {
  t <- 0.
  현재인덱스 <- 0.
  비교횟수 <- 0.
  찾음 <- 0.
}.

(매마디)마다 {
  { t < 최대마디 }인것 일때 {
    { 찾음 < 1 }인것 일때 {
      { 현재인덱스 < 데이터길이 }인것 일때 {
        비교횟수 <- 비교횟수 + 1.
        { 현재인덱스 >= 목표인덱스 }인것 일때 {
          찾음 <- 1.
        }.
        { 찾음 < 1 }인것 일때 {
          현재인덱스 <- 현재인덱스 + 1.
        }.
      }.
    }.
    t <- t + 1.
  }.
}.

보개로 그려.`;
    const fixedRepresentative = runMod.applyLegacyAutofixToDdn(legacyRepresentative).text;
    assert(runMod.readConfiguredMadiFromDdnText(fixedRepresentative) === 40, "legacy representative 마디수 must be detected");
    assert(runMod.resolveRunEngineModeFromDdnText(fixedRepresentative) === "live", "legacy representative must run as live");
    const captureGraph = runMod.resolveLiveRunCaptureGraph({
      runtimeGraph: {
        series: [{ id: "현재", points: [{ x: 2, y: 2 }] }],
        meta: { source: "observation-fallback" },
      },
      runtimeGraphSource: "observation-fallback",
      fallbackGraph: {
        series: [{ id: "값", points: [{ x: 1, y: 1 }, { x: 2, y: 2 }] }],
      },
    });
    assert(
      Array.isArray(captureGraph?.series?.[0]?.points) && captureGraph.series[0].points.length === 2,
      "live capture should prefer accumulated timeline graph over one-frame fallback graph",
    );
    const liveVm = new wasmModule.DdnWasmVm(fixedRepresentative);
    const liveClient = new wrapper.DdnWasmVmClient(liveVm);
    try {
      liveClient.updateLogicWithMode(fixedRepresentative, "strict");
      const firstState = liveClient.stepOneParsed();
      const firstRows = runtimeState.extractObservationOutputRowsFromState(firstState);
      const firstObservation = runtimeState.extractObservationChannelsFromState(firstState);
      const firstViews = runMod.mergeRuntimeViewsWithObservationOutputFallback(
        firstState,
        runtimeState.extractStructuredViewsFromState(firstState, {
          preferPatch: false,
          allowObservationOutputFallback: false,
        }),
      );
      const firstMainVisual = runMod.resolveRunMainVisualMode({
        views: firstViews,
        observation: firstObservation,
        outputRows: firstRows,
      });
      assert(firstRows.some((row) => String(row.key) === "t" && String(row.value) === "1"), "live first frame row missing");
      assert(firstMainVisual.mode !== "none", "live first frame must produce a visible bogae mode");
      const secondState = liveClient.stepOneParsed();
      const secondRows = runtimeState.extractObservationOutputRowsFromState(secondState);
      assert(secondRows.some((row) => String(row.key) === "t" && String(row.value) === "2"), "live second frame row missing");
      assert(JSON.stringify(firstRows) !== JSON.stringify(secondRows), "live rows must change per frame");
    } finally {
      if (typeof liveVm.free === "function") liveVm.free();
    }
    console.log("seamgrim bogae madi graph ui ok");
  } finally {
    if (typeof vm.free === "function") {
      vm.free();
    }
  }
}

main().catch((err) => {
  console.error(err?.stack ?? String(err));
  process.exit(1);
});
