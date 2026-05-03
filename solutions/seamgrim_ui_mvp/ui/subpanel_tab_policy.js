export const SUBPANEL_TAB = Object.freeze({
  MAEGIM: "console",
  OUTPUT: "output",
  MIRROR: "mirror",
  GRAPH: "graph",
  OVERLAY: "overlay",
});

export const SUBPANEL_TAB_LABEL = Object.freeze({
  [SUBPANEL_TAB.MAEGIM]: "매김",
  [SUBPANEL_TAB.OUTPUT]: "결과표",
  [SUBPANEL_TAB.MIRROR]: "거울",
  [SUBPANEL_TAB.GRAPH]: "그래프",
  [SUBPANEL_TAB.OVERLAY]: "겹보기",
});

export function resolveSubpanelTabs(_primaryFamily = "sim") {
  return [SUBPANEL_TAB.MAEGIM, SUBPANEL_TAB.OUTPUT, SUBPANEL_TAB.MIRROR, SUBPANEL_TAB.GRAPH, SUBPANEL_TAB.OVERLAY];
}

export function resolveGraphTabMode(primaryFamily = "sim") {
  return "graph";
}
