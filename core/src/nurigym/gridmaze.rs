use crate::fixed64::Fixed64;

#[derive(Clone, Debug)]
pub struct GridMazeConfig {
    pub width: i64,
    pub height: i64,
    pub max_steps: u64,
    pub goal_reward: Fixed64,
    pub step_cost: Fixed64,
    pub obstacle_cost: Fixed64,
}

#[derive(Clone, Debug)]
pub struct GridMazeLayout {
    pub width: i64,
    pub height: i64,
    pub goal: (i64, i64),
    pub obstacles: Vec<(i64, i64)>,
}

#[derive(Clone, Debug)]
pub struct GridMazeState {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Debug)]
pub struct GridMazeStep {
    pub observation: [Fixed64; 2],
    pub action: i64,
    pub reward: Fixed64,
    pub next_observation: [Fixed64; 2],
    pub done: bool,
}

#[derive(Clone, Debug)]
pub struct GridMazeEnv {
    state: GridMazeState,
    config: GridMazeConfig,
    obstacles: Vec<(i64, i64)>,
    goal: (i64, i64),
    done: bool,
}

impl GridMazeConfig {
    pub fn default_v1() -> Self {
        Self {
            width: 4,
            height: 4,
            max_steps: 100,
            goal_reward: Fixed64::from_i64(10),
            step_cost: Fixed64::ONE,
            obstacle_cost: Fixed64::from_i64(2),
        }
    }
}

impl GridMazeState {
    pub fn seeded(seed: u64, config: &GridMazeConfig) -> Self {
        let base = splitmix64(seed);
        let x = (base & 0xFFFF) as i64 % config.width.max(1);
        let y = ((base >> 16) & 0xFFFF) as i64 % config.height.max(1);
        Self { x, y }
    }

    pub fn observation(&self) -> [Fixed64; 2] {
        [Fixed64::from_i64(self.x), Fixed64::from_i64(self.y)]
    }
}

impl GridMazeEnv {
    pub fn new(seed: u64, config: GridMazeConfig) -> Self {
        let layout = layout_for_seed(seed);
        Self::new_with_layout(seed, config, layout)
    }

    pub fn new_with_layout(seed: u64, config: GridMazeConfig, layout: GridMazeLayout) -> Self {
        let layout = normalize_layout(layout);
        let mut config = config;
        config.width = layout.width;
        config.height = layout.height;
        let mut state = GridMazeState::seeded(seed, &config);
        if is_obstacle_at(&state, &layout.obstacles) || is_goal(&state, layout.goal) {
            state = GridMazeState { x: 0, y: 0 };
        }
        Self {
            state,
            config,
            obstacles: layout.obstacles,
            goal: layout.goal,
            done: false,
        }
    }

    pub fn is_done(&self) -> bool {
        self.done || is_goal(&self.state, self.goal)
    }

    pub fn step(&mut self, action: i64) -> Result<GridMazeStep, String> {
        if self.is_done() {
            return Err("E_NURIGYM_DONE gridmaze already done".to_string());
        }
        let action = normalize_action(action)?;
        let obs = self.state.observation();
        let hit_obstacle = apply_step(&mut self.state, action, &self.config, &self.obstacles);
        let next_obs = self.state.observation();
        let done_after = is_goal(&self.state, self.goal);
        let reward = if done_after {
            self.config.goal_reward
        } else if hit_obstacle {
            -self.config.obstacle_cost
        } else {
            -self.config.step_cost
        };
        self.done = done_after;
        Ok(GridMazeStep {
            observation: obs,
            action,
            reward,
            next_observation: next_obs,
            done: done_after,
        })
    }
}

pub fn run_episode(seed: u64, actions: &[i64], max_steps: Option<u64>) -> Result<Vec<GridMazeStep>, String> {
    if actions.is_empty() {
        return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
    }

    let mut config = GridMazeConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = limit;
    }

    let mut env = GridMazeEnv::new(seed, config.clone());
    let mut steps = Vec::new();

    for (idx, raw_action) in actions.iter().enumerate() {
        if idx as u64 >= config.max_steps {
            break;
        }
        if env.is_done() {
            break;
        }
        let mut step = env.step(*raw_action)?;
        let last_step = (idx as u64 + 1) >= config.max_steps;
        if last_step && !step.done {
            step.done = true;
            step.reward = Fixed64::ZERO;
        }
        steps.push(step);
        if steps.last().map(|s| s.done).unwrap_or(false) {
            break;
        }
    }

    Ok(steps)
}

pub fn run_episode_with_layout(
    seed: u64,
    actions: &[i64],
    max_steps: Option<u64>,
    layout: GridMazeLayout,
) -> Result<Vec<GridMazeStep>, String> {
    if actions.is_empty() {
        return Err("E_NURIGYM_ACTIONS actions must not be empty".to_string());
    }

    let mut config = GridMazeConfig::default_v1();
    if let Some(limit) = max_steps {
        config.max_steps = limit;
    }

    let mut env = GridMazeEnv::new_with_layout(seed, config.clone(), layout);
    let mut steps = Vec::new();

    for (idx, raw_action) in actions.iter().enumerate() {
        if idx as u64 >= config.max_steps {
            break;
        }
        if env.is_done() {
            break;
        }
        let mut step = env.step(*raw_action)?;
        let last_step = (idx as u64 + 1) >= config.max_steps;
        if last_step && !step.done {
            step.done = true;
            step.reward = Fixed64::ZERO;
        }
        steps.push(step);
        if steps.last().map(|s| s.done).unwrap_or(false) {
            break;
        }
    }

    Ok(steps)
}

fn normalize_action(action: i64) -> Result<i64, String> {
    match action {
        -2 | -1 | 1 | 2 => Ok(action),
        _ => Err(format!("E_NURIGYM_ACTION_INVALID action={} (expected -2,-1,1,2)", action)),
    }
}

fn apply_step(
    state: &mut GridMazeState,
    action: i64,
    config: &GridMazeConfig,
    obstacles: &[(i64, i64)],
) -> bool {
    let mut x = state.x;
    let mut y = state.y;
    match action {
        -1 => x -= 1,
        1 => x += 1,
        -2 => y += 1,
        2 => y -= 1,
        _ => {}
    }
    let max_x = config.width.max(1) - 1;
    let max_y = config.height.max(1) - 1;
    if x < 0 {
        x = 0;
    }
    if x > max_x {
        x = max_x;
    }
    if y < 0 {
        y = 0;
    }
    if y > max_y {
        y = max_y;
    }
    let next = GridMazeState { x, y };
    if is_obstacle_at(&next, obstacles) {
        return true;
    }
    state.x = x;
    state.y = y;
    false
}

fn is_goal(state: &GridMazeState, goal: (i64, i64)) -> bool {
    state.x == goal.0 && state.y == goal.1
}

fn is_obstacle_at(state: &GridMazeState, obstacles: &[(i64, i64)]) -> bool {
    obstacles
        .iter()
        .any(|(ox, oy)| *ox == state.x && *oy == state.y)
}

fn layout_for_seed(seed: u64) -> GridMazeLayout {
    match seed % 3 {
        0 => GridMazeLayout {
            width: 4,
            height: 4,
            goal: (3, 3),
            obstacles: vec![(1, 2), (2, 1)],
        },
        1 => GridMazeLayout {
            width: 5,
            height: 4,
            goal: (4, 3),
            obstacles: vec![(1, 1), (2, 1), (3, 1), (3, 2)],
        },
        _ => GridMazeLayout {
            width: 5,
            height: 5,
            goal: (4, 4),
            obstacles: vec![(2, 0), (2, 1), (2, 2), (1, 3), (3, 3)],
        },
    }
}

fn normalize_layout(layout: GridMazeLayout) -> GridMazeLayout {
    let width = layout.width.max(1);
    let height = layout.height.max(1);
    let goal = (
        clamp_i64(layout.goal.0, 0, width - 1),
        clamp_i64(layout.goal.1, 0, height - 1),
    );
    let mut obstacles = Vec::new();
    for (x, y) in layout.obstacles {
        let ox = clamp_i64(x, 0, width - 1);
        let oy = clamp_i64(y, 0, height - 1);
        if (ox, oy) == goal {
            continue;
        }
        obstacles.push((ox, oy));
    }
    obstacles.sort_unstable();
    obstacles.dedup();
    GridMazeLayout {
        width,
        height,
        goal,
        obstacles,
    }
}

fn clamp_i64(value: i64, min: i64, max: i64) -> i64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
