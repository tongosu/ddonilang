#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionNode {
    pub action_id: String,
    pub total_cost: i64,
    pub steps: u32,
}

pub fn pick_best_action(mut actions: Vec<ActionNode>) -> Option<ActionNode> {
    actions.sort_by(|a, b| {
        (
            a.total_cost,
            a.steps as i64,
            a.action_id.as_str(),
        )
            .cmp(&(
                b.total_cost,
                b.steps as i64,
                b.action_id.as_str(),
            ))
    });
    actions.into_iter().next()
}

use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::seulgi::goal::{GoalCondition, TargetState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub id: u64,
    pub name: String,
    pub preconditions: Vec<(String, String)>,
    pub effects: Vec<(String, String)>,
    pub cost: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub goal_id: u64,
    pub actions: Vec<Action>,
    pub total_cost: u16,
    pub madi_count: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldState {
    pub variables: BTreeMap<String, String>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            variables: BTreeMap::new(),
        }
    }

    pub fn apply_action(&self, action: &Action) -> WorldState {
        let mut new_state = self.clone();
        for (key, value) in &action.effects {
            new_state.variables.insert(key.clone(), value.clone());
        }
        new_state
    }

    pub fn satisfies_preconditions(&self, action: &Action) -> bool {
        action.preconditions.iter().all(|(key, value)| {
            self.variables
                .get(key)
                .map(|v| v == value)
                .unwrap_or(false)
        })
    }

    pub fn satisfies_goal(&self, goal: &TargetState) -> bool {
        match &goal.condition {
            GoalCondition::StateEquals { key, value } => {
                self.variables
                    .get(key)
                    .map(|v| v == value)
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    fn key(&self) -> Vec<(String, String)> {
        self.variables
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[derive(Clone, Debug)]
struct Node {
    state: WorldState,
    state_key: Vec<(String, String)>,
    path: Vec<Action>,
    g_cost: u16,
    h_cost: u16,
}

impl Node {
    fn new(state: WorldState, g_cost: u16, h_cost: u16) -> Self {
        let state_key = state.key();
        Self {
            state,
            state_key,
            path: Vec::new(),
            g_cost,
            h_cost,
        }
    }

    fn f_cost(&self) -> u16 {
        self.g_cost.saturating_add(self.h_cost)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.state_key == other.state_key
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .f_cost()
            .cmp(&self.f_cost())
            .then_with(|| self.state_key.cmp(&other.state_key))
    }
}

pub struct GoapPlanner;

impl GoapPlanner {
    pub fn plan(
        initial_state: &WorldState,
        goal: &TargetState,
        available_actions: &[Action],
    ) -> Result<Plan, PlanError> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set: BTreeSet<Vec<(String, String)>> = BTreeSet::new();

        let h_cost = Self::heuristic(initial_state, goal);
        let mut start_node = Node::new(initial_state.clone(), 0, h_cost);
        start_node.path = Vec::new();
        open_set.push(start_node);

        while let Some(current) = open_set.pop() {
            if current.state.satisfies_goal(goal) {
                let mut actions = current.path.clone();
                actions.sort_by_key(|action| action.id);
                return Ok(Plan {
                    goal_id: goal.goal_id,
                    actions,
                    total_cost: current.g_cost,
                    madi_count: (current.path.len() as u16).saturating_add(1),
                });
            }

            if closed_set.contains(&current.state_key) {
                continue;
            }
            closed_set.insert(current.state_key.clone());

            let mut applicable_actions: Vec<_> = available_actions
                .iter()
                .filter(|action| current.state.satisfies_preconditions(action))
                .collect();
            applicable_actions.sort_by_key(|action| action.id);

            for action in applicable_actions {
                let next_state = current.state.apply_action(action);
                let next_key = next_state.key();

                if closed_set.contains(&next_key) {
                    continue;
                }

                let new_g_cost = current.g_cost.saturating_add(action.cost);
                let new_h_cost = Self::heuristic(&next_state, goal);

                let mut next_node = Node::new(next_state, new_g_cost, new_h_cost);
                next_node.state_key = next_key;
                let mut next_path = current.path.clone();
                next_path.push(action.clone());
                next_node.path = next_path;
                open_set.push(next_node);
            }
        }

        Err(PlanError::NoSolution)
    }

    pub fn validate_plan(plan: &Plan, initial_state: &WorldState, goal: &TargetState) -> bool {
        let mut state = initial_state.clone();
        for action in &plan.actions {
            if !state.satisfies_preconditions(action) {
                return false;
            }
            state = state.apply_action(action);
        }
        state.satisfies_goal(goal)
    }

    pub fn visualize_plan(plan: &Plan) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Plan (Cost: {}, Madi: {})\n",
            plan.total_cost, plan.madi_count
        ));
        out.push_str(&"=".repeat(40));
        out.push('\n');
        for (idx, action) in plan.actions.iter().enumerate() {
            out.push_str(&format!(
                "{:2}. {} (cost: {})\n",
                idx + 1,
                action.name,
                action.cost
            ));
        }
        out
    }

    fn heuristic(state: &WorldState, goal: &TargetState) -> u16 {
        match &goal.condition {
            GoalCondition::StateEquals { key, value } => {
                if state.variables.get(key).map(|v| v == value).unwrap_or(false) {
                    0
                } else {
                    100
                }
            }
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    NoSolution,
}

pub fn plan_detjson(plan: &Plan) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_str(&mut out, "schema", "goap.plan.v1", true);
    push_kv_num(&mut out, "goal_id", plan.goal_id, false);
    push_kv_num(&mut out, "total_cost", plan.total_cost as u64, false);
    push_kv_num(&mut out, "madi_count", plan.madi_count as u64, false);
    out.push_str(",\"actions\":[");
    for (idx, action) in plan.actions.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&action_detjson(action));
    }
    out.push_str("]}");
    out
}

fn action_detjson(action: &Action) -> String {
    let mut out = String::new();
    out.push('{');
    push_kv_num(&mut out, "id", action.id, true);
    push_kv_str(&mut out, "name", &action.name, false);
    push_kv_num(&mut out, "cost", action.cost as u64, false);

    out.push_str(",\"preconditions\":[");
    let mut prec = action.preconditions.clone();
    prec.sort();
    for (idx, (key, value)) in prec.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('[');
        push_string(&mut out, key);
        out.push(',');
        push_string(&mut out, value);
        out.push(']');
    }
    out.push(']');

    out.push_str(",\"effects\":[");
    let mut effects = action.effects.clone();
    effects.sort();
    for (idx, (key, value)) in effects.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('[');
        push_string(&mut out, key);
        out.push(',');
        push_string(&mut out, value);
        out.push(']');
    }
    out.push(']');

    out.push('}');
    out
}

fn push_kv_str(out: &mut String, key: &str, value: &str, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    out.push_str(&escape_json(value));
    out.push('"');
}

fn push_kv_num(out: &mut String, key: &str, value: u64, first: bool) {
    if !first {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn push_string(out: &mut String, value: &str) {
    out.push('"');
    out.push_str(&escape_json(value));
    out.push('"');
}

fn escape_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_plan() {
        let mut initial = WorldState::new();
        initial
            .variables
            .insert("has_key".to_string(), "false".to_string());
        initial
            .variables
            .insert("door_open".to_string(), "false".to_string());

        let pickup_key = Action {
            id: 1,
            name: "Pick up key".to_string(),
            preconditions: vec![],
            effects: vec![("has_key".to_string(), "true".to_string())],
            cost: 1,
        };
        let unlock = Action {
            id: 2,
            name: "Unlock door".to_string(),
            preconditions: vec![("has_key".to_string(), "true".to_string())],
            effects: vec![("door_open".to_string(), "true".to_string())],
            cost: 1,
        };

        let goal = TargetState {
            agent_id: 1,
            goal_id: 42,
            condition: GoalCondition::StateEquals {
                key: "door_open".to_string(),
                value: "true".to_string(),
            },
            priority: 128,
        };

        let plan = GoapPlanner::plan(&initial, &goal, &[pickup_key, unlock]).unwrap();
        assert_eq!(plan.total_cost, 2);
        assert_eq!(plan.actions.len(), 2);
        assert!(GoapPlanner::validate_plan(&plan, &initial, &goal));
    }
}
