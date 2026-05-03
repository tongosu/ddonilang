import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function estimateBogaeFrameRect({
  layoutWidth,
  editorRatio,
  splitterWidth = 6,
  bounds,
  aspect = 16 / 9,
}) {
  const width = Math.max(0, Number(layoutWidth) || 0);
  const ratio = Math.max(0, Math.min(1, Number(editorRatio) || 0));
  const splitter = Math.max(0, Number(splitterWidth) || 0);
  const visualWidth = Math.max(0, width - splitter - (width * ratio));
  const maxWidth = Number(bounds?.bogaeFrameMaxWidthPx ?? 0);
  const frameWidth = maxWidth > 0 ? Math.min(visualWidth, maxWidth) : visualWidth;
  const frameHeight = frameWidth > 0 ? frameWidth / aspect : 0;
  return { visualWidth, frameWidth, frameHeight };
}

async function main() {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const runModulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/screens/run.js");
  const indexHtmlPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/index.html");
  const stylesPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/styles.css");
  const runMod = await import(pathToFileURL(runModulePath).href);
  const {
    resolveStudioLayoutBounds,
    resolveBogaeToolbarCompact,
    resolveRunMainControlLabels,
    resolveRunEngineModeFromDdnText,
  } = runMod;

  assert(typeof resolveStudioLayoutBounds === "function", "studio layout contract: resolveStudioLayoutBounds export missing");
  assert(typeof resolveBogaeToolbarCompact === "function", "studio layout contract: resolveBogaeToolbarCompact export missing");
  assert(typeof resolveRunMainControlLabels === "function", "studio layout contract: resolveRunMainControlLabels export missing");
  assert(typeof resolveRunEngineModeFromDdnText === "function", "studio layout contract: resolveRunEngineModeFromDdnText export missing");

  const [indexHtml, stylesCss] = await Promise.all([
    fs.readFile(indexHtmlPath, "utf8"),
    fs.readFile(stylesPath, "utf8"),
  ]);

  assert(indexHtml.includes('class="bogae-frame"'), "studio layout contract: bogae frame wrapper missing");
  assert(indexHtml.includes('aria-label="한마디씩"'), "studio layout contract: step aria-label token missing");
  assert(
    (indexHtml.match(/<span class="brand">셈그림<\/span>/g) ?? []).length >= 4,
    "studio layout contract: shell brand should be visible on browse/editor/block/run title lines",
  );
  assert(!indexHtml.includes("bogae-toolbar"), "studio layout contract: legacy bogae toolbar DOM should be removed");
  assert(indexHtml.includes('class="run-control-bar"'), "studio layout contract: common run control bar missing");
  assert(
    indexHtml.indexOf('class="run-control-bar"') > indexHtml.indexOf('class="run-topbar"')
      && indexHtml.indexOf('class="run-control-bar"') < indexHtml.indexOf('class="run-layout"'),
    "studio layout contract: run control bar should sit between title bar and layout",
  );
  assert(
    !/data-studio-view-mode=["'](?:analyze|full)["'][\s\S]{0,240}\.run-control-bar[\s\S]{0,120}display\s*:\s*none/.test(stylesCss),
    "studio layout contract: expanded modes must not hide the common run control bar",
  );
  assert(stylesCss.includes(".bogae-frame {"), "studio layout contract: bogae frame style missing");
  assert(stylesCss.includes("aspect-ratio: 16 / 9;"), "studio layout contract: 16:9 aspect token missing");
  assert(stylesCss.includes(".run-control-bar--compact {"), "studio layout contract: compact run control token missing");
  assert(
    /\.run-main\s*>\s*\.run-topbar\s*\{[^}]*gap:\s*8px;[^}]*padding:\s*10px\s+14px;[^}]*min-height:\s*var\(--topbar-height\);/s.test(stylesCss),
    "studio layout contract: run title bar should use the same shell topbar sizing",
  );
  assert(
    /\.run-main\s*>\s*\.run-topbar\s+\.brand\s*\{[^}]*font-size:\s*18px;/s.test(stylesCss),
    "studio layout contract: run title brand should match shell brand size",
  );
  assert(
    /\.subpanel\s*\{[^}]*min-height\s*:\s*300px;/s.test(stylesCss),
    "studio layout contract: subpanel min-height token missing",
  );
  assert(
    stylesCss.includes('.run-layout--studio-full .subpanel[data-panel-open="0"] .subpanel-tab-panel'),
    "studio layout contract: full mode slideout close token missing",
  );
  assert(
    stylesCss.includes("grid-template-columns: minmax(0, 2fr) minmax(300px, 1fr);"),
    "studio layout contract: analyze mode should split bogae and tabs as 2/3 + 1/3",
  );
  assert(
    stylesCss.includes("grid-template-columns: minmax(0, 1fr) 42px;"),
    "studio layout contract: full mode should give bogae the full visual width with overlay tabs",
  );
  assert(
    /\.run-view-mode-group\s*\{[^}]*flex:\s*0 0 auto;[^}]*flex-wrap:\s*nowrap;[^}]*white-space:\s*nowrap;/s.test(stylesCss),
    "studio layout contract: view mode button group should stay one row",
  );
  assert(
    /\.run-view-mode-btn\s*\{[^}]*flex:\s*0 0 auto;[^}]*white-space:\s*nowrap;/s.test(stylesCss),
    "studio layout contract: view mode buttons should not wrap vertically",
  );
  assert(
    /\.run-layout--studio-analyze\s+\.bogae-area\s*\{[^}]*grid-column:\s*1;[^}]*border-right:\s*1px\s+solid\s+var\(--line\);/s.test(stylesCss),
    "studio layout contract: analyze bogae area should stay in the left two-thirds",
  );
  assert(
    runMod && /const panelOpen = mode === STUDIO_VIEW_MODE_FULL \? this\.fullModePanelOpen : true;/.test(await fs.readFile(runModulePath, "utf8")),
    "studio layout contract: analyze mode should keep the right panel open while full mode uses the slideout panel",
  );
  assert(
    /\.run-layout--studio-analyze\s+\.bogae-frame,\s*\.run-layout--studio-full\s+\.bogae-frame\s*\{[^}]*width:\s*100%;[^}]*height:\s*100%;[^}]*aspect-ratio:\s*auto;/s.test(stylesCss),
    "studio layout contract: expanded bogae frame should fill full area instead of 16:9 letterboxing",
  );
  assert(
    /\.run-layout--studio-analyze\s+\.bogae-area,\s*\.run-layout--studio-full\s+\.bogae-area\s*\{[^}]*align-items:\s*stretch;[^}]*justify-content:\s*stretch;[^}]*width:\s*100%;[^}]*height:\s*100%;[^}]*overflow:\s*auto;/s.test(stylesCss),
    "studio layout contract: studio bogae area should stretch and allow full-width frame",
  );
  assert(
    /\.run-layout--studio-analyze\s+\.bogae-frame,\s*\.run-layout--studio-full\s+\.bogae-frame\s*\{[^}]*max-width:\s*none;[^}]*margin:\s*0;[^}]*align-self:\s*stretch;[^}]*justify-self:\s*stretch;/s.test(stylesCss),
    "studio layout contract: studio bogae frame should not center-clamp itself",
  );
  assert(
    /#screen-run\[data-studio-view-mode="analyze"\]\s+\.bogae-frame,\s*#screen-run\[data-studio-view-mode="full"\]\s+\.bogae-frame\s*\{[^}]*position:\s*absolute\s*!important;[^}]*inset:\s*0\s*!important;[^}]*width:\s*100%\s*!important;[^}]*height:\s*100%\s*!important;[^}]*aspect-ratio:\s*auto\s*!important;/s.test(stylesCss),
    "studio layout contract: expanded view final override should force full-frame bogae",
  );
  assert(
    /#screen-run\[data-studio-view-mode="analyze"\]\s+\.run-visual-column\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*2fr\)\s*minmax\(300px,\s*1fr\)\s*!important;/s.test(stylesCss),
    "studio layout contract: analyze mode final override should split bogae and tabs",
  );
  assert(
    /#screen-run\[data-studio-view-mode="full"\]\s+\.run-visual-column\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s*42px\s*!important;/s.test(stylesCss),
    "studio layout contract: full mode final override should keep a narrow tab strip",
  );
  assert(
    /#screen-run\[data-studio-view-mode="full"\]\s+\.subpanel\s*\{[^}]*display:\s*flex\s*!important;[^}]*width:\s*42px\s*!important;[^}]*overflow:\s*visible\s*!important;/s.test(stylesCss),
    "studio layout contract: full mode should keep a narrow slideout tab strip",
  );
  assert(
    /#screen-run\[data-studio-view-mode="analyze"\]\s+\.bogae-area\s*\{[^}]*grid-column:\s*1\s*!important;[^}]*width:\s*100%\s*!important;[^}]*height:\s*100%\s*!important;[^}]*overflow:\s*hidden\s*!important;/s.test(stylesCss),
    "studio layout contract: analyze final override should force bogae into the left pane",
  );
  assert(
    /#screen-run\[data-studio-view-mode="full"\]\s+\.bogae-area\s*\{[^}]*grid-column:\s*1\s*\/\s*-1\s*!important;[^}]*width:\s*100%\s*!important;[^}]*height:\s*100%\s*!important;[^}]*overflow:\s*hidden\s*!important;/s.test(stylesCss),
    "studio layout contract: full final override should force full-area bogae",
  );
  assert(
    stylesCss.includes(".run-visual-column.run-visual-column--scroll-fallback {"),
    "studio layout contract: visual fallback scroll token missing",
  );
  assert(
    resolveBogaeToolbarCompact({ toolbarWidth: 780, thresholdPx: 860 }) === true,
    "studio layout contract: compact threshold should turn on under narrow toolbar",
  );
  assert(
    resolveBogaeToolbarCompact({ toolbarWidth: 940, thresholdPx: 860 }) === false,
    "studio layout contract: compact threshold should turn off when width recovers",
  );
  const labelsWide = resolveRunMainControlLabels({ isPaused: false, compact: false });
  const labelsCompact = resolveRunMainControlLabels({ isPaused: false, compact: true });
  assert(labelsWide.execute === "▶ 작업실에서 실행", "studio layout contract: wide execute label");
  assert(labelsCompact.execute === "▶ 실행", "studio layout contract: compact execute label");
  assert(
    resolveRunEngineModeFromDdnText("x <- 1.\nx 보여주기.") === "oneshot",
    "studio layout contract: scalar show should be oneshot",
  );
  assert(
    resolveRunEngineModeFromDdnText("(시작)할때 {\n  x <- 1.\n}.") === "oneshot",
    "studio layout contract: start hook only should be oneshot",
  );
  assert(
    resolveRunEngineModeFromDdnText("(매마디)마다 {\n  x <- x + 1.\n}.") === "live",
    "studio layout contract: every madi should be live",
  );

  const bounds = resolveStudioLayoutBounds({
    layoutWidth: 1280,
    layoutHeight: 820,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 0,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  assert(bounds.hasConstraintOverflow === false, "studio layout contract: unexpected overflow in roomy viewport");
  const frame = estimateBogaeFrameRect({
    layoutWidth: 1280,
    editorRatio: 0.64,
    splitterWidth: 6,
    bounds,
  });
  assert(frame.frameWidth > 0 && frame.frameHeight > 0, "studio layout contract: frame size missing");
  assert(Math.abs((frame.frameWidth / frame.frameHeight) - (16 / 9)) < 1e-9, "studio layout contract: frame ratio drift");
  const subpanelHeight = bounds.availableVisualHeight - frame.frameHeight;
  assert(subpanelHeight >= 300 - 1e-6, "studio layout contract: subpanel height below 300");

  const boundsWithBanner = resolveStudioLayoutBounds({
    layoutWidth: 1280,
    layoutHeight: 820,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 36,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  const frameWithBanner = estimateBogaeFrameRect({
    layoutWidth: 1280,
    editorRatio: 0.64,
    splitterWidth: 6,
    bounds: boundsWithBanner,
  });
  assert(
    frameWithBanner.frameHeight <= frame.frameHeight + 1e-9,
    "studio layout contract: banner should not increase frame height",
  );

  const boundsHeightOverflow = resolveStudioLayoutBounds({
    layoutWidth: 920,
    layoutHeight: 300,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 24,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  assert(boundsHeightOverflow.hasHeightOverflow === true, "studio layout contract: height overflow missing");
  assert(boundsHeightOverflow.hasConstraintOverflow === true, "studio layout contract: overflow aggregate missing");

  const boundsWidthOverflow = resolveStudioLayoutBounds({
    layoutWidth: 380,
    layoutHeight: 760,
    splitterWidth: 6,
    toolbarHeight: 40,
    errorBannerHeight: 0,
    minVisualWidth: 420,
    subpanelMinHeight: 300,
    bogaeAspectRatio: 16 / 9,
  });
  assert(boundsWidthOverflow.hasWidthOverflow === true, "studio layout contract: width overflow missing");
  assert(boundsWidthOverflow.hasConstraintOverflow === true, "studio layout contract: width overflow aggregate missing");

  console.log("seamgrim studio layout contract runner ok");
}

main().catch((error) => {
  console.error(String(error?.stack ?? error));
  process.exit(1);
});
