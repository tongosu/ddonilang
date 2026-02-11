use crate::fixed64::Fixed64;
use crate::platform::{ComponentTag, EntityId, NuriWorld, StateHash};
use blake3;

#[derive(Debug, Clone, Copy)]
pub struct W24Params {
    pub entity_count: u64,
    pub component_count: u64,
    pub archetype_moves: u64,
    pub perf_cap: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W25Params {
    pub query_target_count: u64,
    pub query_batch: u64,
    pub snapshot_fixed: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W26Params {
    pub agent_count: u64,
    pub item_count: u64,
    pub trade_count: u64,
    pub starting_balance: u64,
    pub starting_inventory: u64,
    pub base_price: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W27Params {
    pub agent_count: u64,
    pub trade_count: u64,
    pub starting_balance: u64,
    pub min_balance: u64,
    pub trade_amount: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W28Params {
    pub agent_count: u64,
    pub item_count: u64,
    pub trade_count: u64,
    pub base_price: u64,
    pub trade_amount: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W29Params {
    pub reactive_max_pass: u64,
    pub alert_chain: u64,
    pub step_value: u64,
    pub initial_value: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W30Params {
    pub proposal_count: u64,
    pub approval_tokens: u64,
    pub apply_requests: u64,
    pub approval_required: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W31Params {
    pub participant_count: u64,
    pub host_inputs: u64,
    pub guest_inputs: u64,
    pub sync_rounds: u64,
    pub starting_value: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W32Params {
    pub diff_count: u64,
    pub code_before_len: u64,
    pub code_after_len: u64,
    pub state_field_count: u64,
    pub summary_cap: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct W33Params {
    pub agent_count: u64,
    pub item_count: u64,
    pub trade_count: u64,
    pub query_batch: u64,
    pub reactive_max_pass: u64,
}

pub fn compute_w24_state_hash(params: &W24Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.개체수".to_string(),
        Fixed64::from_i64(params.entity_count as i64),
    );
    world.set_resource_fixed64(
        "살림.컴포넌트수".to_string(),
        Fixed64::from_i64(params.component_count as i64),
    );
    world.set_resource_fixed64(
        "살림.아키타입_이동".to_string(),
        Fixed64::from_i64(params.archetype_moves as i64),
    );
    world.set_resource_fixed64(
        "살림.성능_캡".to_string(),
        Fixed64::from_i64(params.perf_cap as i64),
    );

    if params.entity_count == 0 || params.component_count == 0 {
        return world.state_hash();
    }

    for entity in 0..params.entity_count {
        for comp in 0..params.component_count {
            world.set_component_json(
                EntityId(entity),
                ComponentTag(format!("C{}", comp)),
                format!("e{}_c{}", entity, comp),
            );
        }
    }

    let cap = params.perf_cap.max(1);
    let max_moves = params.entity_count.saturating_mul(cap);
    let ticks = if params.archetype_moves == 0 || max_moves == 0 {
        1
    } else {
        (params.archetype_moves + max_moves - 1) / max_moves
    };

    for tick in 0..ticks {
        let start = tick.saturating_mul(max_moves);
        let end = (start + max_moves).min(params.archetype_moves);
        for move_index in start..end {
            let entity_id = move_index % params.entity_count;
            let comp_index = (move_index / params.entity_count) % params.component_count;
            let tag = ComponentTag(format!("C{}", comp_index));
            let entity = EntityId(entity_id);
            if world.get_component_json(entity, &tag).is_some() {
                world.remove_component(entity, &tag);
            } else {
                world.set_component_json(entity, tag, format!("e{}_c{}", entity_id, comp_index));
            }
        }
    }

    world.state_hash()
}

pub fn compute_w25_state_hash(params: &W25Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.쿼리_대상수".to_string(),
        Fixed64::from_i64(params.query_target_count as i64),
    );
    world.set_resource_fixed64(
        "살림.쿼리_배치".to_string(),
        Fixed64::from_i64(params.query_batch as i64),
    );
    world.set_resource_fixed64(
        "살림.스냅샷_고정".to_string(),
        Fixed64::from_i64(params.snapshot_fixed as i64),
    );

    if params.query_target_count == 0 {
        return world.state_hash();
    }

    let target_tag = ComponentTag("Q".to_string());
    let processed_tag = ComponentTag("P".to_string());

    for entity in 0..params.query_target_count {
        world.set_component_json(
            EntityId(entity),
            target_tag.clone(),
            format!("q{}", entity),
        );
    }

    let snapshot_fixed = params.snapshot_fixed != 0;
    let snapshot = if snapshot_fixed {
        world.query_entities_with_all_tags(&[target_tag.clone()])
    } else {
        Vec::new()
    };

    let batch = params.query_batch.max(1);
    let ticks = (params.query_target_count + batch - 1) / batch;

    for tick in 0..ticks {
        let owned_targets;
        let targets = if snapshot_fixed {
            &snapshot
        } else {
            owned_targets = world.query_entities_with_all_tags(&[target_tag.clone()]);
            &owned_targets
        };
        if targets.is_empty() {
            break;
        }
        let start = (tick * batch) as usize;
        if start >= targets.len() {
            break;
        }
        let end = ((tick + 1) * batch).min(targets.len() as u64) as usize;
        for entity in &targets[start..end] {
            world.remove_component(*entity, &target_tag);
            world.set_component_json(*entity, processed_tag.clone(), "참".to_string());
        }
    }

    world.state_hash()
}

pub fn compute_w26_state_hash(params: &W26Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.임자수".to_string(),
        Fixed64::from_i64(params.agent_count as i64),
    );
    world.set_resource_fixed64(
        "살림.상품수".to_string(),
        Fixed64::from_i64(params.item_count as i64),
    );
    world.set_resource_fixed64(
        "살림.거래수".to_string(),
        Fixed64::from_i64(params.trade_count as i64),
    );
    world.set_resource_fixed64(
        "살림.초기_잔고".to_string(),
        Fixed64::from_i64(params.starting_balance as i64),
    );
    world.set_resource_fixed64(
        "살림.초기_재고".to_string(),
        Fixed64::from_i64(params.starting_inventory as i64),
    );
    world.set_resource_fixed64(
        "살림.기본_가격".to_string(),
        Fixed64::from_i64(params.base_price as i64),
    );

    let agent_count = params.agent_count as usize;
    let item_count = params.item_count as usize;
    let starting_balance = params.starting_balance as i64;
    let starting_inventory = params.starting_inventory as i64;
    let base_price = params.base_price as i64;

    let mut balances = Vec::with_capacity(agent_count);
    for idx in 0..agent_count {
        balances.push(starting_balance.saturating_add(idx as i64));
    }

    let mut inventories = Vec::with_capacity(agent_count);
    for _ in 0..agent_count {
        let mut items = Vec::with_capacity(item_count);
        for item in 0..item_count {
            items.push(starting_inventory.saturating_add(item as i64));
        }
        inventories.push(items);
    }

    let mut prices = Vec::with_capacity(item_count);
    for item in 0..item_count {
        prices.push(base_price.saturating_add(item as i64));
    }

    let mut incomes = Vec::with_capacity(agent_count);
    let mut preferences = Vec::with_capacity(agent_count);
    for idx in 0..agent_count {
        incomes.push(starting_balance.saturating_add((idx as i64).saturating_mul(2)));
        let pref = if item_count == 0 {
            0
        } else {
            (idx % item_count) as i64
        };
        preferences.push(pref);
    }

    let mut trade_log = String::from("[");
    let mut total_qty: i64 = 0;
    let mut total_value: i64 = 0;
    if agent_count > 0 && item_count > 0 && params.trade_count > 0 {
        for trade in 0..params.trade_count {
            let buyer = (trade % params.agent_count) as usize;
            let seller = ((trade + 1) % params.agent_count) as usize;
            let item = (trade % params.item_count) as usize;
            let qty = ((trade % 3) + 1) as i64;
            let price = base_price
                .saturating_add(item as i64)
                .saturating_add((trade % 5) as i64);
            let total = price.saturating_mul(qty);

            balances[buyer] = balances[buyer].saturating_sub(total);
            balances[seller] = balances[seller].saturating_add(total);
            inventories[buyer][item] = inventories[buyer][item].saturating_add(qty);
            inventories[seller][item] = inventories[seller][item].saturating_sub(qty);

            total_qty = total_qty.saturating_add(qty);
            total_value = total_value.saturating_add(total);

            if trade > 0 {
                trade_log.push(',');
            }
            trade_log.push_str(&format!(
                "{{\"t\":{trade},\"buyer\":{buyer},\"seller\":{seller},\"item\":{item},\"qty\":{qty},\"price\":{price},\"total\":{total}}}"
            ));
        }
    }
    trade_log.push(']');

    let mut utilities = Vec::with_capacity(agent_count);
    for agent in 0..agent_count {
        let mut total = balances[agent];
        if item_count > 0 {
            for item in 0..item_count {
                total = total.saturating_add(
                    inventories[agent][item].saturating_mul(prices[item]),
                );
            }
        }
        utilities.push(total);
    }

    let mut balance_sum: i64 = 0;
    for value in &balances {
        balance_sum = balance_sum.saturating_add(*value);
    }

    let mut inventory_sum: i64 = 0;
    for agent in &inventories {
        for value in agent {
            inventory_sum = inventory_sum.saturating_add(*value);
        }
    }

    world.set_resource_json("살림.소득".to_string(), json_array_i64(&incomes));
    world.set_resource_json("살림.선호도".to_string(), json_array_i64(&preferences));
    world.set_resource_json("살림.효용".to_string(), json_array_i64(&utilities));
    world.set_resource_json("살림.잔고".to_string(), json_array_i64(&balances));
    world.set_resource_json("살림.재고".to_string(), json_matrix_i64(&inventories));
    world.set_resource_json("살림.가격".to_string(), json_array_i64(&prices));
    world.set_resource_json("살림.거래_로그".to_string(), trade_log);

    world.set_resource_fixed64(
        "살림.거래_총액".to_string(),
        Fixed64::from_i64(total_value),
    );
    world.set_resource_fixed64(
        "살림.거래_총량".to_string(),
        Fixed64::from_i64(total_qty),
    );
    world.set_resource_fixed64(
        "살림.잔고_합".to_string(),
        Fixed64::from_i64(balance_sum),
    );
    world.set_resource_fixed64(
        "살림.재고_합".to_string(),
        Fixed64::from_i64(inventory_sum),
    );

    world.state_hash()
}

pub fn compute_w27_state_hash(params: &W27Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.임자수".to_string(),
        Fixed64::from_i64(params.agent_count as i64),
    );
    world.set_resource_fixed64(
        "살림.거래수".to_string(),
        Fixed64::from_i64(params.trade_count as i64),
    );
    world.set_resource_fixed64(
        "살림.초기_잔고".to_string(),
        Fixed64::from_i64(params.starting_balance as i64),
    );
    world.set_resource_fixed64(
        "살림.잔고_최소".to_string(),
        Fixed64::from_i64(params.min_balance as i64),
    );
    world.set_resource_fixed64(
        "살림.거래_금액".to_string(),
        Fixed64::from_i64(params.trade_amount as i64),
    );

    let agent_count = params.agent_count as usize;
    let starting_balance = params.starting_balance as i64;
    let min_balance = params.min_balance as i64;
    let trade_amount = params.trade_amount as i64;

    let mut balances = Vec::with_capacity(agent_count);
    for idx in 0..agent_count {
        balances.push(starting_balance.saturating_add(idx as i64));
    }

    let mut accepted = 0i64;
    let mut rejected = 0i64;
    if agent_count > 0 && params.trade_count > 0 {
        for trade in 0..params.trade_count {
            let agent = (trade % params.agent_count) as usize;
            let amount = trade_amount.saturating_add((trade % 3) as i64);
            let candidate = balances[agent].saturating_sub(amount);
            if candidate < min_balance {
                rejected = rejected.saturating_add(1);
                continue;
            }
            balances[agent] = candidate;
            accepted = accepted.saturating_add(1);
        }
    }

    let mut balance_sum: i64 = 0;
    for value in &balances {
        balance_sum = balance_sum.saturating_add(*value);
    }

    world.set_resource_json("살림.잔고".to_string(), json_array_i64(&balances));
    world.set_resource_fixed64(
        "살림.거래_승인".to_string(),
        Fixed64::from_i64(accepted),
    );
    world.set_resource_fixed64(
        "살림.거래_거절".to_string(),
        Fixed64::from_i64(rejected),
    );
    world.set_resource_fixed64(
        "살림.잔고_합".to_string(),
        Fixed64::from_i64(balance_sum),
    );

    world.state_hash()
}

pub fn compute_w28_state_hash(params: &W28Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.임자수".to_string(),
        Fixed64::from_i64(params.agent_count as i64),
    );
    world.set_resource_fixed64(
        "살림.상품수".to_string(),
        Fixed64::from_i64(params.item_count as i64),
    );
    world.set_resource_fixed64(
        "살림.거래수".to_string(),
        Fixed64::from_i64(params.trade_count as i64),
    );
    world.set_resource_fixed64(
        "살림.기본_가격".to_string(),
        Fixed64::from_i64(params.base_price as i64),
    );
    world.set_resource_fixed64(
        "살림.거래_금액".to_string(),
        Fixed64::from_i64(params.trade_amount as i64),
    );

    let item_count = params.item_count as usize;
    let base_price = params.base_price as i64;
    let trade_amount = params.trade_amount as i64;

    let mut item_qty = vec![0i64; item_count];
    let mut item_value = vec![0i64; item_count];
    let mut frames = String::from("[");
    let mut total_qty: i64 = 0;
    let mut total_value: i64 = 0;
    if item_count > 0 && params.trade_count > 0 {
        for trade in 0..params.trade_count {
            let item = (trade % params.item_count) as usize;
            let qty = trade_amount.saturating_add((trade % 3) as i64);
            let price = base_price
                .saturating_add(item as i64)
                .saturating_add((trade % 7) as i64);
            let value = price.saturating_mul(qty);

            item_qty[item] = item_qty[item].saturating_add(qty);
            item_value[item] = item_value[item].saturating_add(value);
            total_qty = total_qty.saturating_add(qty);
            total_value = total_value.saturating_add(value);

            if trade > 0 {
                frames.push(',');
            }
            frames.push_str(&format!(
                "{{\"madi\":{trade},\"item\":{item},\"qty\":{qty},\"price\":{price},\"value\":{value}}}"
            ));
        }
    }
    frames.push(']');

    let avg_price = if total_qty == 0 {
        0
    } else {
        total_value / total_qty
    };

    world.set_resource_fixed64(
        "살림.거래_총액".to_string(),
        Fixed64::from_i64(total_value),
    );
    world.set_resource_fixed64(
        "살림.거래_총량".to_string(),
        Fixed64::from_i64(total_qty),
    );
    world.set_resource_fixed64(
        "살림.지표_GDP".to_string(),
        Fixed64::from_i64(total_value),
    );
    world.set_resource_fixed64(
        "살림.지표_거래량".to_string(),
        Fixed64::from_i64(total_qty),
    );
    world.set_resource_fixed64(
        "살림.지표_물가".to_string(),
        Fixed64::from_i64(avg_price),
    );
    world.set_resource_json("살림.지표_품목_총량".to_string(), json_array_i64(&item_qty));
    world.set_resource_json("살림.지표_품목_총액".to_string(), json_array_i64(&item_value));
    world.set_resource_json("살림.지표_프레임".to_string(), frames);

    world.state_hash()
}

pub fn compute_w29_state_hash(params: &W29Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.반응_패스_최대".to_string(),
        Fixed64::from_i64(params.reactive_max_pass as i64),
    );
    world.set_resource_fixed64(
        "살림.알림_연쇄".to_string(),
        Fixed64::from_i64(params.alert_chain as i64),
    );
    world.set_resource_fixed64(
        "살림.반응_증분".to_string(),
        Fixed64::from_i64(params.step_value as i64),
    );
    world.set_resource_fixed64(
        "살림.초기_값".to_string(),
        Fixed64::from_i64(params.initial_value as i64),
    );

    let max_pass = params.reactive_max_pass as i64;
    let chain = params.alert_chain as i64;
    let step = params.step_value as i64;
    let initial = params.initial_value as i64;

    let executed = chain.min(max_pass).max(0);
    let blocked = chain.saturating_sub(executed);
    let final_value = initial.saturating_add(executed.saturating_mul(step));
    let diag_count = if blocked > 0 { 1 } else { 0 };

    world.set_resource_fixed64(
        "살림.반응_실행".to_string(),
        Fixed64::from_i64(executed),
    );
    world.set_resource_fixed64(
        "살림.반응_차단".to_string(),
        Fixed64::from_i64(blocked),
    );
    world.set_resource_fixed64(
        "살림.누적_값".to_string(),
        Fixed64::from_i64(final_value),
    );
    world.set_resource_fixed64(
        "살림.진단_발생".to_string(),
        Fixed64::from_i64(diag_count),
    );

    world.state_hash()
}

pub fn compute_w30_state_hash(params: &W30Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.제안_수".to_string(),
        Fixed64::from_i64(params.proposal_count as i64),
    );
    world.set_resource_fixed64(
        "살림.승인_토큰".to_string(),
        Fixed64::from_i64(params.approval_tokens as i64),
    );
    world.set_resource_fixed64(
        "살림.적용_요청".to_string(),
        Fixed64::from_i64(params.apply_requests as i64),
    );
    world.set_resource_fixed64(
        "살림.승인_필수".to_string(),
        Fixed64::from_i64(params.approval_required as i64),
    );

    let proposal_count = params.proposal_count as i64;
    let approval_tokens = params.approval_tokens as i64;
    let apply_requests = params.apply_requests as i64;
    let approval_required = params.approval_required != 0;

    let approved = proposal_count.min(approval_tokens).max(0);
    let rejected = proposal_count.saturating_sub(approved);
    let apply_allowed = if approval_required {
        approved
    } else {
        proposal_count
    };
    let applied = apply_requests.min(apply_allowed).max(0);
    let blocked_by_approval = if approval_required {
        proposal_count.saturating_sub(approved)
    } else {
        0
    };
    let verify_pass = if applied == apply_allowed { 1 } else { 0 };
    let verify_fail = apply_allowed.saturating_sub(applied);

    let mut log = String::from("[");
    let approved_u = approved.max(0) as u64;
    let applied_u = applied.max(0) as u64;
    for idx in 0..params.proposal_count {
        if idx > 0 {
            log.push(',');
        }
        let status = if idx < approved_u { "approved" } else { "rejected" };
        let applied_flag = idx < applied_u;
        log.push_str(&format!(
            "{{\"id\":{idx},\"status\":\"{status}\",\"applied\":{applied_flag}}}"
        ));
    }
    log.push(']');

    world.set_resource_fixed64(
        "살림.미리보기_수".to_string(),
        Fixed64::from_i64(proposal_count),
    );
    world.set_resource_fixed64(
        "살림.승인_수".to_string(),
        Fixed64::from_i64(approved),
    );
    world.set_resource_fixed64(
        "살림.승인_거절".to_string(),
        Fixed64::from_i64(rejected),
    );
    world.set_resource_fixed64(
        "살림.적용_허용".to_string(),
        Fixed64::from_i64(apply_allowed),
    );
    world.set_resource_fixed64(
        "살림.적용_수".to_string(),
        Fixed64::from_i64(applied),
    );
    world.set_resource_fixed64(
        "살림.승인_차단".to_string(),
        Fixed64::from_i64(blocked_by_approval),
    );
    world.set_resource_fixed64(
        "살림.검증_통과".to_string(),
        Fixed64::from_i64(verify_pass),
    );
    world.set_resource_fixed64(
        "살림.검증_실패".to_string(),
        Fixed64::from_i64(verify_fail),
    );
    world.set_resource_json("살림.승인_로그".to_string(), log);

    world.state_hash()
}

pub fn compute_w31_state_hash(params: &W31Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.참가자수".to_string(),
        Fixed64::from_i64(params.participant_count as i64),
    );
    world.set_resource_fixed64(
        "살림.호스트_입력".to_string(),
        Fixed64::from_i64(params.host_inputs as i64),
    );
    world.set_resource_fixed64(
        "살림.손님_입력".to_string(),
        Fixed64::from_i64(params.guest_inputs as i64),
    );
    world.set_resource_fixed64(
        "살림.동기_라운드".to_string(),
        Fixed64::from_i64(params.sync_rounds as i64),
    );
    world.set_resource_fixed64(
        "살림.시작_값".to_string(),
        Fixed64::from_i64(params.starting_value as i64),
    );

    let participant_count = params.participant_count as usize;
    let host_inputs = params.host_inputs as usize;
    let guest_inputs = params.guest_inputs as usize;
    let sync_rounds = params.sync_rounds as usize;
    let mut shared_value = params.starting_value as i64;

    let guest_count = participant_count.saturating_sub(1);
    let mut log = String::from("[");
    let mut accepted = 0i64;
    let mut rejected = 0i64;
    if participant_count > 0 && sync_rounds > 0 {
        let mut events: Vec<(usize, usize, usize, i64)> = Vec::new();
        for seq in 0..host_inputs {
            let round = seq % sync_rounds;
            let value = (seq as i64) + 1;
            events.push((round, 0, seq, value));
        }
        if guest_count > 0 {
            for idx in 0..guest_inputs {
                let sender = 1 + (idx % guest_count);
                let seq = idx / guest_count;
                let round = idx % sync_rounds;
                let value = (idx as i64 + 1) * 2;
                events.push((round, sender, seq, value));
            }
        }
        events.sort_by(|a, b| (a.0, a.1, a.2).cmp(&(b.0, b.1, b.2)));
        for (idx, (round, sender, seq, value)) in events.iter().enumerate() {
            let accept = if *sender == 0 {
                true
            } else {
                (sender + round) % 2 == 0
            };
            if accept {
                shared_value = shared_value.saturating_add(*value);
                accepted = accepted.saturating_add(1);
            } else {
                rejected = rejected.saturating_add(1);
            }
            if idx > 0 {
                log.push(',');
            }
            log.push_str(&format!(
                "{{\"round\":{round},\"sender\":{sender},\"seq\":{seq},\"accepted\":{accept},\"value\":{value}}}"
            ));
        }
    }
    log.push(']');

    world.set_resource_fixed64(
        "살림.공유_입력_승인".to_string(),
        Fixed64::from_i64(accepted),
    );
    world.set_resource_fixed64(
        "살림.공유_입력_거절".to_string(),
        Fixed64::from_i64(rejected),
    );
    world.set_resource_fixed64(
        "살림.공유_값".to_string(),
        Fixed64::from_i64(shared_value),
    );
    world.set_resource_json("살림.공유_로그".to_string(), log);

    world.state_hash()
}

pub fn compute_w32_state_hash(params: &W32Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.차분_개수".to_string(),
        Fixed64::from_i64(params.diff_count as i64),
    );
    world.set_resource_fixed64(
        "살림.코드_길이_전".to_string(),
        Fixed64::from_i64(params.code_before_len as i64),
    );
    world.set_resource_fixed64(
        "살림.코드_길이_후".to_string(),
        Fixed64::from_i64(params.code_after_len as i64),
    );
    world.set_resource_fixed64(
        "살림.상태_필드_수".to_string(),
        Fixed64::from_i64(params.state_field_count as i64),
    );
    world.set_resource_fixed64(
        "살림.요약_캡".to_string(),
        Fixed64::from_i64(params.summary_cap as i64),
    );

    let diff_count = params.diff_count as usize;
    let state_field_count = params.state_field_count.max(1) as usize;
    let summary_cap = params.summary_cap as usize;
    let before_len = params.code_before_len as i64;
    let after_len = params.code_after_len as i64;

    let mut summary = String::from("[");
    let mut summary_count = 0usize;
    for idx in 0..diff_count {
        let kind = match idx % 3 {
            0 => "add",
            1 => "modify",
            _ => "remove",
        };
        let field = idx % state_field_count;
        let before = before_len.saturating_add(idx as i64);
        let after = after_len.saturating_add((idx as i64) * 2);
        if summary_count < summary_cap {
            if summary_count > 0 {
                summary.push(',');
            }
            summary.push_str(&format!(
                "{{\"id\":{idx},\"kind\":\"{kind}\",\"field\":{field},\"before\":{before},\"after\":{after}}}"
            ));
            summary_count += 1;
        }
    }
    summary.push(']');

    let patch_payload = format!(
        "{}/{}:{}/{}:{}",
        params.code_before_len,
        params.code_after_len,
        params.diff_count,
        params.state_field_count,
        params.summary_cap
    );
    let patch_hash = blake3::hash(patch_payload.as_bytes()).to_hex().to_string();
    let approval_payload = format!("{patch_hash}:approval:{summary_count}");
    let approval_hash = blake3::hash(approval_payload.as_bytes()).to_hex().to_string();

    world.set_resource_json("살림.차분_요약".to_string(), summary);
    world.set_resource_json("살림.패치_해시".to_string(), patch_hash);
    world.set_resource_json("살림.승인_해시".to_string(), approval_hash);
    world.set_resource_fixed64(
        "살림.차분_요약_개수".to_string(),
        Fixed64::from_i64(summary_count as i64),
    );

    world.state_hash()
}

pub fn compute_w33_state_hash(params: &W33Params) -> StateHash {
    let mut world = NuriWorld::new();
    world.set_resource_fixed64(
        "살림.임자수".to_string(),
        Fixed64::from_i64(params.agent_count as i64),
    );
    world.set_resource_fixed64(
        "살림.상품수".to_string(),
        Fixed64::from_i64(params.item_count as i64),
    );
    world.set_resource_fixed64(
        "살림.거래수".to_string(),
        Fixed64::from_i64(params.trade_count as i64),
    );
    world.set_resource_fixed64(
        "살림.쿼리_배치".to_string(),
        Fixed64::from_i64(params.query_batch as i64),
    );
    world.set_resource_fixed64(
        "살림.반응_패스_최대".to_string(),
        Fixed64::from_i64(params.reactive_max_pass as i64),
    );

    let agent_count = params.agent_count as usize;
    let item_count = params.item_count as usize;
    let trade_count = params.trade_count as usize;
    let query_batch = params.query_batch.max(1) as usize;
    let reactive_max_pass = params.reactive_max_pass as usize;

    let mut balances = vec![100i64; agent_count];
    let mut inventories = vec![vec![0i64; item_count]; agent_count];
    let mut item_qty = vec![0i64; item_count];
    let mut item_value = vec![0i64; item_count];
    let base_price = 5i64;

    let mut total_value: i64 = 0;
    let mut total_qty: i64 = 0;
    for trade in 0..trade_count {
        let buyer = trade % agent_count.max(1);
        let seller = (trade + 1) % agent_count.max(1);
        let item = trade % item_count.max(1);
        let qty = (trade % 3 + 1) as i64;
        let price = base_price + item as i64;
        let value = price.saturating_mul(qty);

        balances[buyer] = balances[buyer].saturating_sub(value);
        balances[seller] = balances[seller].saturating_add(value);
        inventories[buyer][item] = inventories[buyer][item].saturating_add(qty);
        inventories[seller][item] = inventories[seller][item].saturating_sub(qty);

        item_qty[item] = item_qty[item].saturating_add(qty);
        item_value[item] = item_value[item].saturating_add(value);
        total_qty = total_qty.saturating_add(qty);
        total_value = total_value.saturating_add(value);
    }

    let mut processed = vec![false; agent_count];
    let mut batch_round = 0usize;
    while processed.iter().any(|done| !done) {
        let start = batch_round * query_batch;
        if start >= agent_count {
            break;
        }
        let end = (start + query_batch).min(agent_count);
        for idx in start..end {
            processed[idx] = true;
        }
        batch_round += 1;
    }

    let mut reactive_passes = 0usize;
    let mut diag_count = 0i64;
    let mut value_acc = total_value;
    while reactive_passes < reactive_max_pass {
        value_acc = value_acc.saturating_add(1);
        reactive_passes += 1;
    }
    if reactive_passes >= reactive_max_pass && reactive_max_pass > 0 {
        diag_count = 1;
    }

    let avg_price = if total_qty == 0 {
        0
    } else {
        total_value / total_qty
    };

    world.set_resource_fixed64(
        "살림.거래_총액".to_string(),
        Fixed64::from_i64(total_value),
    );
    world.set_resource_fixed64(
        "살림.거래_총량".to_string(),
        Fixed64::from_i64(total_qty),
    );
    world.set_resource_fixed64(
        "살림.지표_물가".to_string(),
        Fixed64::from_i64(avg_price),
    );
    world.set_resource_fixed64(
        "살림.쿼리_처리".to_string(),
        Fixed64::from_i64(processed.iter().filter(|v| **v).count() as i64),
    );
    world.set_resource_fixed64(
        "살림.반응_실행".to_string(),
        Fixed64::from_i64(reactive_passes as i64),
    );
    world.set_resource_fixed64(
        "살림.진단_발생".to_string(),
        Fixed64::from_i64(diag_count),
    );
    world.set_resource_fixed64(
        "살림.누적_값".to_string(),
        Fixed64::from_i64(value_acc),
    );
    world.set_resource_json("살림.지표_품목_총량".to_string(), json_array_i64(&item_qty));
    world.set_resource_json("살림.지표_품목_총액".to_string(), json_array_i64(&item_value));
    world.set_resource_json("살림.잔고".to_string(), json_array_i64(&balances));
    world.set_resource_json("살림.재고".to_string(), json_matrix_i64(&inventories));

    world.state_hash()
}

fn json_array_i64(values: &[i64]) -> String {
    let mut out = String::from("[");
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&value.to_string());
    }
    out.push(']');
    out
}

fn json_matrix_i64(values: &[Vec<i64>]) -> String {
    let mut out = String::from("[");
    for (idx, row) in values.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_array_i64(row));
    }
    out.push(']');
    out
}
