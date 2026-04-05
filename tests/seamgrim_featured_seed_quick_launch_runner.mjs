import path from "path";
import fs from "fs/promises";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function loadModuleFromPath(modulePath) {
  const source = await fs.readFile(modulePath, "utf8");
  const encoded = Buffer.from(String(source), "utf8").toString("base64");
  const moduleUrl = `data:text/javascript;base64,${encoded}`;
  return import(moduleUrl);
}

async function main() {
  const root = process.cwd();
  const modulePath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/featured_seed_quick_launch.js");
  const catalogPath = path.resolve(root, "solutions/seamgrim_ui_mvp/ui/featured_seed_catalog.js");
  const mod = await loadModuleFromPath(modulePath);
  const catalogMod = await loadModuleFromPath(catalogPath);
  const {
    resolveAvailableFeaturedSeedIds,
    pickNextFeaturedSeedLaunch,
    shouldTriggerFeaturedSeedQuickLaunch,
    shouldTriggerFeaturedSeedQuickPreset,
  } = mod;
  const FEATURED = Array.isArray(catalogMod?.FEATURED_SEED_IDS) ? [...catalogMod.FEATURED_SEED_IDS] : [];

  const REQUIRED_FEATURED = [
    "bio_sir_transition_visual_seed_v2",
    "econ_tax_shock_supply_demand_seed_v1",
    "econ_inventory_price_feedback_seed_v2",
    "transport_signal_traffic_seed_v1",
    "queue_mm1_seed_v1",
    "physics_projectile_drag_seed_v1",
    "physics_spring_damper_seed_v1",
    "physics_heat_diffusion_seed_v1",
    "ecology_forest_fire_ca_seed_v1",
    "swarm_boids_alignment_seed_v1",
  ];
  assert(FEATURED.length >= REQUIRED_FEATURED.length, "featured seed minimum count");
  REQUIRED_FEATURED.forEach((seedId) => {
    assert(FEATURED.includes(seedId), `featured seed required id missing: ${seedId}`);
  });
  assert(new Set(FEATURED).size === FEATURED.length, "featured seed duplicate ids");

  const lessonsById = new Map(FEATURED.map((seedId) => [seedId, { id: seedId }]));

  const available = resolveAvailableFeaturedSeedIds(FEATURED, lessonsById);
  assert(Array.isArray(available) && available.length === FEATURED.length, "available ids count");
  assert(available[0] === "bio_sir_transition_visual_seed_v2", "available order keep 0");
  assert(available[2] === "econ_inventory_price_feedback_seed_v2", "available order keep 2");
  const deduped = resolveAvailableFeaturedSeedIds([...FEATURED, FEATURED[0]], lessonsById);
  assert(Array.isArray(deduped) && deduped.length === FEATURED.length, "available ids dedupe count");
  assert(deduped[0] === FEATURED[0], "available ids dedupe order keep 0");

  const pickFromCurrent = pickNextFeaturedSeedLaunch({
    featuredSeedIds: FEATURED,
    lessonsById,
    currentLessonId: "econ_tax_shock_supply_demand_seed_v1",
    cursor: -1,
  });
  assert(pickFromCurrent.nextId === "econ_inventory_price_feedback_seed_v2", "current-id next pick");
  assert(pickFromCurrent.nextCursor === 2, "current-id cursor");

  const pickWrap = pickNextFeaturedSeedLaunch({
    featuredSeedIds: FEATURED,
    lessonsById,
    currentLessonId: "",
    cursor: FEATURED.length - 1,
  });
  assert(pickWrap.nextId === "bio_sir_transition_visual_seed_v2", "cursor wrap pick");
  assert(pickWrap.nextCursor === 0, "cursor wrap nextCursor");

  const pickFromInvalidCursor = pickNextFeaturedSeedLaunch({
    featuredSeedIds: FEATURED,
    lessonsById,
    currentLessonId: "bio_sir_transition_visual_seed_v2",
    cursor: 999,
  });
  assert(pickFromInvalidCursor.nextId === "econ_tax_shock_supply_demand_seed_v1", "invalid cursor fallback to current");
  assert(pickFromInvalidCursor.nextCursor === 1, "invalid cursor nextCursor");

  const subsetLessonsById = new Map([
    ["econ_tax_shock_supply_demand_seed_v1", { id: "econ_tax_shock_supply_demand_seed_v1" }],
  ]);
  const subsetPick = pickNextFeaturedSeedLaunch({
    featuredSeedIds: FEATURED,
    lessonsById: subsetLessonsById,
    currentLessonId: "",
    cursor: -1,
  });
  assert(Array.isArray(subsetPick.availableIds) && subsetPick.availableIds.length === 1, "subset available filtered");
  assert(subsetPick.nextId === "econ_tax_shock_supply_demand_seed_v1", "subset pick id");
  assert(subsetPick.nextCursor === 0, "subset pick cursor");

  const emptyPick = pickNextFeaturedSeedLaunch({
    featuredSeedIds: FEATURED,
    lessonsById: new Map(),
    currentLessonId: "",
    cursor: -1,
  });
  assert(Array.isArray(emptyPick.availableIds) && emptyPick.availableIds.length === 0, "empty available ids");
  assert(emptyPick.nextId === "", "empty nextId");
  assert(emptyPick.nextCursor === -1, "empty nextCursor");

  assert(
    shouldTriggerFeaturedSeedQuickLaunch({ altKey: true, ctrlKey: false, metaKey: false, shiftKey: false, repeat: false, code: "Digit6", key: "6" }) === true,
    "hotkey basic trigger",
  );
  assert(
    shouldTriggerFeaturedSeedQuickLaunch({ altKey: true, ctrlKey: false, metaKey: false, shiftKey: false, repeat: false, code: "KeyA", key: "a" }) === false,
    "hotkey wrong key blocked",
  );
  assert(
    shouldTriggerFeaturedSeedQuickLaunch({ altKey: true, ctrlKey: true, metaKey: false, shiftKey: false, repeat: false, code: "Digit6", key: "6" }) === false,
    "hotkey ctrl-modifier blocked",
  );
  assert(
    shouldTriggerFeaturedSeedQuickLaunch({ altKey: true, ctrlKey: false, metaKey: false, shiftKey: false, repeat: true, code: "Digit6", key: "6" }) === false,
    "hotkey repeat blocked",
  );
  assert(
    shouldTriggerFeaturedSeedQuickLaunch(
      { altKey: true, ctrlKey: false, metaKey: false, shiftKey: false, repeat: false, code: "Digit6", key: "6" },
      { isEditableTarget: true },
    ) === false,
    "hotkey editable target blocked",
  );
  assert(
    shouldTriggerFeaturedSeedQuickPreset(
      { altKey: true, ctrlKey: false, metaKey: false, shiftKey: true, repeat: false, code: "Digit6", key: "6" },
      { isEditableTarget: false, isBrowseScreen: true },
    ) === true,
    "preset hotkey basic trigger",
  );
  assert(
    shouldTriggerFeaturedSeedQuickPreset(
      { altKey: true, ctrlKey: false, metaKey: false, shiftKey: true, repeat: false, code: "Digit6", key: "6" },
      { isEditableTarget: false, isBrowseScreen: false },
    ) === false,
    "preset hotkey non-browse blocked",
  );
  assert(
    shouldTriggerFeaturedSeedQuickPreset(
      { altKey: true, ctrlKey: false, metaKey: false, shiftKey: false, repeat: false, code: "Digit6", key: "6" },
      { isEditableTarget: false, isBrowseScreen: true },
    ) === false,
    "preset hotkey shift-required",
  );
  assert(
    shouldTriggerFeaturedSeedQuickPreset(
      { altKey: true, ctrlKey: false, metaKey: false, shiftKey: true, repeat: true, code: "Digit6", key: "6" },
      { isEditableTarget: false, isBrowseScreen: true },
    ) === false,
    "preset hotkey repeat blocked",
  );
  assert(
    shouldTriggerFeaturedSeedQuickPreset(
      { altKey: true, ctrlKey: false, metaKey: false, shiftKey: true, repeat: false, code: "Digit6", key: "6" },
      { isEditableTarget: true, isBrowseScreen: true },
    ) === false,
    "preset hotkey editable target blocked",
  );

  console.log("seamgrim featured seed quick launch runner ok");
}

main().catch((err) => {
  const msg = String(err?.message ?? err);
  console.error(`check=featured_seed_quick_launch detail=${msg}`);
  process.exit(1);
});
