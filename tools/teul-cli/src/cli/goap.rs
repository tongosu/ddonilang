use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ddonirang_core::seulgi::goal::{GoalCondition, TargetState};
use ddonirang_core::seulgi::goap::{plan_detjson, Action, GoapPlanner, PlanError, WorldState};
use serde_json::Value;

use super::detjson::write_text;

pub fn run_plan(input: &Path, out: Option<&Path>) -> Result<(), String> {
    let raw = fs::read_to_string(input).map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;

    let initial_state = parse_initial_state(&value)?;
    let goal = parse_goal(&value)?;
    let actions = parse_actions(&value)?;

    let plan = match GoapPlanner::plan(&initial_state, &goal, &actions) {
        Ok(plan) => plan_detjson(&plan),
        Err(PlanError::NoSolution) => {
            format!(
                "{{\"schema\":\"goap.plan.v1\",\"goal_id\":{},\"total_cost\":0,\"madi_count\":0,\"actions\":[],\"error\":\"NoSolution\"}}",
                goal.goal_id
            )
        }
    };

    if let Some(path) = out {
        write_text(path, &format!("{}\n", plan))?;
    } else {
        println!("{}", plan);
    }
    Ok(())
}

fn parse_initial_state(value: &Value) -> Result<WorldState, String> {
    let initial = value
        .get("initial_state")
        .ok_or_else(|| "E_GOAP_INPUT initial_state 없음".to_string())?;
    let obj = initial
        .as_object()
        .ok_or_else(|| "E_GOAP_INPUT initial_state 객체 필요".to_string())?;
    let mut variables = BTreeMap::new();
    for (key, val) in obj {
        let v = val
            .as_str()
            .ok_or_else(|| "E_GOAP_INPUT initial_state 값은 문자열이어야 함".to_string())?;
        variables.insert(key.clone(), v.to_string());
    }
    Ok(WorldState { variables })
}

fn parse_goal(value: &Value) -> Result<TargetState, String> {
    let goal = value
        .get("goal")
        .ok_or_else(|| "E_GOAP_INPUT goal 없음".to_string())?;
    let obj = goal
        .as_object()
        .ok_or_else(|| "E_GOAP_INPUT goal 객체 필요".to_string())?;
    let key = obj
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_GOAP_INPUT goal.key 없음".to_string())?;
    let value = obj
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "E_GOAP_INPUT goal.value 없음".to_string())?;
    let goal_id = obj.get("goal_id").and_then(|v| v.as_u64()).unwrap_or(0);
    let agent_id = obj.get("agent_id").and_then(|v| v.as_u64()).unwrap_or(0);
    let priority = obj.get("priority").and_then(|v| v.as_u64()).unwrap_or(128) as u8;

    Ok(TargetState {
        agent_id,
        goal_id,
        condition: GoalCondition::StateEquals {
            key: key.to_string(),
            value: value.to_string(),
        },
        priority,
    })
}

fn parse_actions(value: &Value) -> Result<Vec<Action>, String> {
    let list = value
        .get("actions")
        .ok_or_else(|| "E_GOAP_INPUT actions 없음".to_string())?;
    let arr = list
        .as_array()
        .ok_or_else(|| "E_GOAP_INPUT actions 배열 필요".to_string())?;
    let mut actions = Vec::new();
    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| "E_GOAP_INPUT action 객체 필요".to_string())?;
        let id = obj
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "E_GOAP_INPUT action.id 없음".to_string())?;
        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "E_GOAP_INPUT action.name 없음".to_string())?;
        let cost = obj
            .get("cost")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| "E_GOAP_INPUT action.cost 없음".to_string())?;
        let preconditions = parse_pairs(obj.get("preconditions"))?;
        let effects = parse_pairs(obj.get("effects"))?;
        actions.push(Action {
            id,
            name: name.to_string(),
            preconditions,
            effects,
            cost: cost as u16,
        });
    }
    Ok(actions)
}

fn parse_pairs(value: Option<&Value>) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let Some(value) = value else {
        return Ok(out);
    };
    let list = value
        .as_array()
        .ok_or_else(|| "E_GOAP_INPUT preconditions/effects 배열 필요".to_string())?;
    for item in list {
        let pair = item
            .as_array()
            .ok_or_else(|| "E_GOAP_INPUT 조건은 [key, value] 배열".to_string())?;
        if pair.len() != 2 {
            return Err("E_GOAP_INPUT 조건은 [key, value] 2요소".to_string());
        }
        let key = pair[0]
            .as_str()
            .ok_or_else(|| "E_GOAP_INPUT 조건 key 문자열 필요".to_string())?;
        let value = pair[1]
            .as_str()
            .ok_or_else(|| "E_GOAP_INPUT 조건 value 문자열 필요".to_string())?;
        out.push((key.to_string(), value.to_string()));
    }
    Ok(out)
}
