export function formatObserveFamilyName(family) {
  const code = String(family ?? "").trim().toLowerCase();
  if (code === "space2d") return "보개";
  if (code === "graph") return "그래프";
  if (code === "table") return "표";
  if (code === "text") return "설명";
  if (code === "structure") return "구조";
  return code || "-";
}

export function buildObserveFamilyActionHint({ family = "", available = false, strict = true } = {}) {
  const code = String(family ?? "").trim().toLowerCase();
  if (!available) {
    return "권장: lesson의 보임 {}에 이 보기 family를 추가하세요.";
  }
  if (!strict) {
    return "권장: 구조 출력 소스(strict)로 전환하세요.";
  }
  if (code === "space2d") {
    return "권장: 카드 클릭 후 보개를 이동/확대해 상태 변화를 관찰하세요.";
  }
  if (code === "graph") {
    return "권장: 카드 클릭 후 x/y축과 슬라이더를 조정해 추세를 확인하세요.";
  }
  if (code === "table") {
    return "권장: 카드 클릭 후 열/행 변화를 비교하세요.";
  }
  if (code === "text") {
    return "권장: 카드 클릭 후 설명 패널에서 핵심 요약을 확인하세요.";
  }
  if (code === "structure") {
    return "권장: 카드 클릭 후 노드/링크 관계를 점검하세요.";
  }
  return "권장: 카드 클릭 후 관찰 패널에서 값을 점검하세요.";
}

export function summarizeObserveFamilyMetric(family, runtimeView, metricReaders = {}) {
  const code = String(family ?? "").trim().toLowerCase();
  if (code === "space2d") {
    return String(metricReaders?.space2d?.(runtimeView) ?? "보개 출력 없음");
  }
  if (code === "graph") {
    return String(metricReaders?.graph?.(runtimeView) ?? "그래프 출력 없음");
  }
  if (code === "table") {
    return String(metricReaders?.table?.(runtimeView) ?? "표 출력 없음");
  }
  if (code === "text") {
    return String(metricReaders?.text?.(runtimeView) ?? "설명 출력 없음");
  }
  if (code === "structure") {
    return String(metricReaders?.structure?.(runtimeView) ?? "구조 출력 없음");
  }
  return "출력 없음";
}
