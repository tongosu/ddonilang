import path from "path";
import { pathToFileURL } from "url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/inspector_contract.js");
  const mod = await import(pathToFileURL(modulePath).href);
  const { buildInspectorReport, formatInspectorReportText } = mod;

  assert(typeof buildInspectorReport === "function", "inspector export: buildInspectorReport");
  assert(typeof formatInspectorReportText === "function", "inspector export: formatInspectorReportText");

  const okReport = buildInspectorReport({
    lesson: {
      id: "l1",
      title: "수요곡선",
      description: "기본 설명",
      ddnMetaHeader: { name: "ddn_name", desc: "ddn_desc", required_views: ["2d", "graph", "2d"] },
    },
    lastRuntimeHash: "sha256:state",
    parseWarnings: [{ code: "W_TEST", message: "warn" }],
    runtimeTickCounter: 5,
    runtimeTimeValue: 0.5,
    playbackPaused: false,
    playbackSpeed: 1,
    lastExecPathHint: "실행 경로: wasm(strict)",
    lastRuntimeDerived: {
      views: {
        graph: { schema: "seamgrim.graph.v0" },
      },
    },
    sceneSummary: {
      schema: "seamgrim.scene.v0",
      hashes: { input_hash: "sha256:in", result_hash: "sha256:out" },
      layers: [{ id: "run:l1", label: "run", update: "append", tick: 4, points: 5 }],
    },
    snapshotV0: { schema: "seamgrim.snapshot.v0" },
    sessionV0: { schema: "seamgrim.session.v0" },
  });
  assert(okReport.bridge_check.ok === true, "inspector bridge: ok case");
  assert(okReport.view_contract?.strict === true, "inspector view source strict: ok case");
  assert(Array.isArray(okReport.logs) && okReport.logs.length >= 1, "inspector logs: warning included");
  const okText = formatInspectorReportText(okReport);
  assert(okText.includes("연결 확인: 정상"), "inspector text: bridge ok");
  assert(okText.includes("DDN 메타 이름: ddn_name"), "inspector text: ddn meta");
  assert(okText.includes("DDN 필수 보기: 보개, 그래프"), "inspector text: ddn required views");
  assert(okText.includes("보기 엄격성: 정상"), "inspector text: view source strict ok");

  const sourceFailReport = buildInspectorReport({
    lesson: { id: "l3" },
    lastRuntimeHash: "sha256:view-source-fail",
    parseWarnings: [],
    lastRuntimeDerived: {
      graphSource: "observation-fallback",
      views: {
        graph: { schema: "seamgrim.graph.v0" },
        contract: {
          schema: "seamgrim.view_contract.v1",
          source: "runtime",
          families: ["graph"],
          by_family: {
            graph: { source: "observation-fallback", schema: "seamgrim.graph.v0", available: true },
          },
        },
      },
    },
    sceneSummary: { schema: "seamgrim.scene.v0", hashes: { result_hash: "sha256:out" } },
    snapshotV0: { schema: "seamgrim.snapshot.v0" },
    sessionV0: { schema: "seamgrim.session.v0" },
  });
  assert(sourceFailReport.view_contract?.strict === false, "inspector view source strict: fail case");
  assert(
    Array.isArray(sourceFailReport.view_contract?.non_strict_families) &&
      sourceFailReport.view_contract.non_strict_families.includes("graph"),
    "inspector view source strict: non strict families",
  );
  const sourceFailText = formatInspectorReportText(sourceFailReport);
  assert(sourceFailText.includes("보기 엄격성: 실패"), "inspector text: view source strict fail");

  const failReport = buildInspectorReport({
    lesson: { id: "l2" },
    lastRuntimeHash: "",
    parseWarnings: [],
    lastExecPathHint: "실행 실패: wasm/server 모두 실패",
    lastRuntimeDerived: { views: { graph: { schema: "wrong.schema" } } },
    sceneSummary: { schema: "seamgrim.scene.v0" },
    snapshotV0: { schema: "seamgrim.snapshot.v0" },
    sessionV0: { schema: "wrong.session" },
  });
  assert(failReport.bridge_check.ok === false, "inspector bridge: fail case");
  assert(
    failReport.logs.some((row) => String(row?.code ?? "").trim() === "E_BRIDGE_CHECK"),
    "inspector logs: bridge error",
  );
  const failText = formatInspectorReportText(failReport);
  assert(failText.includes("연결 확인: 실패"), "inspector text: bridge fail");

  console.log("seamgrim inspector contract runner ok");
}

main().catch((error) => {
  console.error(error?.stack ?? String(error));
  process.exit(1);
});
