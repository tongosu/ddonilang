const MASK64 = (1n << 64n) - 1n;

function splitmix64(seed) {
  let x = (BigInt(seed) + 0x9e3779b97f4a7c15n) & MASK64;
  let z = x;
  z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & MASK64;
  z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & MASK64;
  return z ^ (z >> 31n);
}

function normalizeRows(jsonlText) {
  return String(jsonlText ?? "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => JSON.parse(line));
}

function normalizeLayout(layout) {
  const width = Math.max(1, Number(layout?.width ?? 4));
  const height = Math.max(1, Number(layout?.height ?? 4));
  const clamp = (value, min, max) => Math.max(min, Math.min(max, Number(value ?? 0)));
  const goalRaw = Array.isArray(layout?.goal) ? layout.goal : [width - 1, height - 1];
  const goal = [clamp(goalRaw[0], 0, width - 1), clamp(goalRaw[1], 0, height - 1)];
  const seen = new Set();
  const obstacles = [];
  for (const raw of Array.isArray(layout?.obstacles) ? layout.obstacles : []) {
    if (!Array.isArray(raw)) continue;
    const next = [clamp(raw[0], 0, width - 1), clamp(raw[1], 0, height - 1)];
    if (next[0] === goal[0] && next[1] === goal[1]) continue;
    const key = `${next[0]},${next[1]}`;
    if (seen.has(key)) continue;
    seen.add(key);
    obstacles.push(next);
  }
  obstacles.sort((a, b) => (a[0] - b[0]) || (a[1] - b[1]));
  return { width, height, goal, obstacles };
}

function defaultGridmazeLayout(seed) {
  const idx = Number(BigInt(seed) % 3n);
  if (idx === 0) {
    return normalizeLayout({ width: 4, height: 4, goal: [3, 3], obstacles: [[1, 2], [2, 1]] });
  }
  if (idx === 1) {
    return normalizeLayout({
      width: 5,
      height: 4,
      goal: [4, 3],
      obstacles: [[1, 1], [2, 1], [3, 1], [3, 2]],
    });
  }
  return normalizeLayout({
    width: 5,
    height: 5,
    goal: [4, 4],
    obstacles: [[2, 0], [2, 1], [2, 2], [1, 3], [3, 3]],
  });
}

function selectGridmazeLayout(input) {
  const layouts = Array.isArray(input.gridmaze_layouts) ? input.gridmaze_layouts : [];
  if (layouts.length === 0) return defaultGridmazeLayout(input.seed ?? 0);
  const index = Number(BigInt(input.seed ?? 0) % BigInt(layouts.length));
  return normalizeLayout(layouts[index]);
}

function isObstacle(state, obstacles) {
  return obstacles.some(([x, y]) => x === state.x && y === state.y);
}

function isGoal(state, goal) {
  return state.x === goal[0] && state.y === goal[1];
}

function seededGridmazeState(seed, layout) {
  const base = splitmix64(seed);
  let x = Number(base & 0xffffn) % layout.width;
  let y = Number((base >> 16n) & 0xffffn) % layout.height;
  const state = { x, y };
  if (isObstacle(state, layout.obstacles) || isGoal(state, layout.goal)) {
    x = 0;
    y = 0;
  }
  return { x, y };
}

function applyGridmazeStep(state, action, layout) {
  let x = state.x;
  let y = state.y;
  if (action === -1) x -= 1;
  if (action === 1) x += 1;
  if (action === -2) y += 1;
  if (action === 2) y -= 1;
  x = Math.max(0, Math.min(layout.width - 1, x));
  y = Math.max(0, Math.min(layout.height - 1, y));
  const next = { x, y };
  if (isObstacle(next, layout.obstacles)) {
    return { state: { ...state }, hitObstacle: true };
  }
  return { state: next, hitObstacle: false };
}

function simulateGridmaze(input) {
  const layout = selectGridmazeLayout(input);
  const actions = Array.isArray(input.actions) ? input.actions.map(Number) : [];
  const maxSteps = Number(input.max_steps ?? 100);
  let state = seededGridmazeState(input.seed ?? 0, layout);
  const rows = [];
  for (let idx = 0; idx < actions.length && idx < maxSteps; idx += 1) {
    if (isGoal(state, layout.goal)) break;
    const action = actions[idx];
    if (![-2, -1, 1, 2].includes(action)) {
      throw new Error(`invalid gridmaze action: ${action}`);
    }
    const observation = [state.x, state.y];
    const step = applyGridmazeStep(state, action, layout);
    state = step.state;
    let done = isGoal(state, layout.goal);
    let reward = done ? 10 : (step.hitObstacle ? -2 : -1);
    const lastStep = idx + 1 >= maxSteps;
    if (lastStep && !done) {
      done = true;
      reward = 0;
    }
    rows.push({ action, observation, reward, next_observation: [state.x, state.y], done });
    if (done) break;
  }
  return rows;
}

function simulateBandit(input) {
  const actions = Array.isArray(input.actions) ? input.actions.map(Number) : [];
  const maxSteps = Number(input.max_steps ?? 100);
  const preferred = (splitmix64(input.seed ?? 0) & 1n) === 0n ? -1 : 1;
  const rows = [];
  let pulls = 0;
  for (let idx = 0; idx < actions.length && idx < maxSteps; idx += 1) {
    const action = actions[idx];
    if (![-1, 1].includes(action)) {
      throw new Error(`invalid bandit action: ${action}`);
    }
    const observation = [pulls, preferred];
    const reward = action === preferred ? 1 : 0;
    pulls += 1;
    rows.push({
      action,
      observation,
      reward,
      next_observation: [pulls, preferred],
      done: pulls >= maxSteps,
    });
    if (pulls >= maxSteps) break;
  }
  return rows;
}

function datasetStepProjection(row) {
  return {
    action: Number(row?.action?.value),
    observation: Array.isArray(row?.observation?.values) ? row.observation.values.map(Number) : [],
    reward: Number(row?.reward),
    next_observation: Array.isArray(row?.next_observation?.values)
      ? row.next_observation.values.map(Number)
      : [],
    done: Boolean(row?.done),
  };
}

export function simulateNuriGymEpisode(input) {
  const envId = String(input?.env_id ?? "");
  if (envId === "nurigym.gridmaze2d") return simulateGridmaze(input);
  if (envId === "nurigym.bandit1d") return simulateBandit(input);
  throw new Error(`unsupported web parity env_id: ${envId}`);
}

export function compareNuriGymDatasetParity({ input, datasetJsonl }) {
  const rows = normalizeRows(datasetJsonl);
  const header = rows[0] ?? {};
  const datasetSteps = rows.slice(1).map(datasetStepProjection);
  const webSteps = simulateNuriGymEpisode(input);
  const failures = [];
  if (header.schema !== "nurigym.dataset.v1") {
    failures.push(`bad dataset schema: ${header.schema}`);
  }
  if (header.env_id !== input.env_id) {
    failures.push(`env mismatch: ${header.env_id} != ${input.env_id}`);
  }
  if (datasetSteps.length !== webSteps.length) {
    failures.push(`step count mismatch: ${datasetSteps.length} != ${webSteps.length}`);
  }
  const limit = Math.min(datasetSteps.length, webSteps.length);
  for (let idx = 0; idx < limit; idx += 1) {
    const left = JSON.stringify(datasetSteps[idx]);
    const right = JSON.stringify(webSteps[idx]);
    if (left !== right) failures.push(`step ${idx} mismatch: ${left} != ${right}`);
  }
  return {
    schema: "ddn.nurigym.python_web_parity.case.v1",
    ok: failures.length === 0,
    env_id: input.env_id,
    dataset_count: datasetSteps.length,
    web_count: webSteps.length,
    failures,
  };
}
