//! Value text, canonical spelling and resource conversion; unchanged bodies.
use super::{
    ASSERTION_RESOURCE_KIND,
    Assertion,
    BTreeMap,
    BTreeSet,
    EvalError,
    FORMULA_RESOURCE_KIND,
    Fixed64,
    Formula,
    FormulaDialect,
    Lexer,
    MapEntry,
    NUMERIC_KIND_BIG_INT,
    NUMERIC_KIND_FACTOR,
    NUMERIC_KIND_RATIONAL,
    RELATION_KIND_EQUATION,
    RELATION_KIND_FIELD,
    RELATION_LEFT_FIELD,
    RELATION_RIGHT_FIELD,
    RELATION_SOLVE_BINDINGS_FIELD,
    RELATION_SOLVE_REASON_FIELD,
    RELATION_SOLVE_RESULT_FAILURE,
    RELATION_SOLVE_RESULT_KIND_FIELD,
    RELATION_SOLVE_RESULT_SUCCESS,
    RELATION_SOLVE_VALUE_FIELD,
    RELATION_SOLVE_VAR_FIELD,
    ResourceHandle,
    ResourceMapEntry,
    ResourceValue,
    STATE_MACHINE_RESOURCE_KIND,
    STD_BLOCK_PIECE_KIND,
    STD_FALLING_PIECE_KIND,
    STD_GRID_GAME_HOLD_KIND,
    STD_GRID_GAME_HOLD_SWAP_KIND,
    STD_GRID_GAME_NEXT_PIECE_KIND,
    STD_GRID_GAME_ROTATION_TRY_KIND,
    STD_GRID_GAME_SCORE_KIND,
    STD_GRID_GAME_SESSION_KIND,
    STD_GRID_GAME_STATE_KIND,
    STD_GRID_GAME_TICK_KIND,
    STD_GRID_GAME_VIEW_SUMMARY_KIND,
    STD_GRID_KIND,
    STD_GRID_LINE_CLEAR_KIND,
    STD_INPUT_MAP_KIND,
    STD_RANDOM_BAG_DRAW_KIND,
    STD_RANDOM_BAG_KIND,
    StateMachine,
    StateTransition,
    TemplateFormat,
    TemplatePart,
    TokenKind,
    Value,
    extract_assertion_body_source,
    format_formula_body,
    map_key_canon,
    numeric_pack_kind,
    pow,
    token_observation_name,
    type_mismatch_error,
    unit_error,
    unit_spec_from_symbol,
};

pub(crate) fn value_to_string(value: &Value) -> String {
    format_value_text(value, false)
}

pub(crate) fn value_to_display_text(value: &Value) -> String {
    format_value_text(value, true)
}

pub(super) fn format_value_text(value: &Value, display_only: bool) -> String {
    // Restored saved values can be much deeper than the host call stack.
    // Preserve existing display spelling/rounding, but traverse containers
    // and the existing equation/solver carriers through heap jobs.
    use std::borrow::Cow;
    enum Job<'a> { Value(&'a Value, bool), Text(Cow<'a, str>) }
    let mut jobs = vec![Job::Value(value, display_only)];
    let mut out = String::new();
    fn fields<'a>(jobs: &mut Vec<Job<'a>>, items: &'a BTreeMap<String, Value>, display: bool, separator: &'static str) {
        jobs.push(Job::Text(Cow::Borrowed(")")));
        for (index, (name, value)) in items.iter().enumerate().rev() {
            jobs.push(Job::Value(value, display));
            jobs.push(Job::Text(Cow::Borrowed(separator)));
            jobs.push(Job::Text(Cow::Borrowed(name)));
            if index != 0 { jobs.push(Job::Text(Cow::Borrowed(", "))); }
        }
    }
    while let Some(job) = jobs.pop() {
        match job {
            Job::Text(text) => out.push_str(&text),
            Job::Value(Value::SumBranch {owner,branch,payload,..},display) => {
                out.push_str(owner);out.push_str(".$");out.push_str(branch);
                if payload.len()==1 {
                    out.push('(');jobs.push(Job::Text(Cow::Borrowed(")")));jobs.push(Job::Value(payload.values().next().expect("one payload"),display));
                } else if !payload.is_empty() {
                    out.push('(');fields(&mut jobs,payload,display,"=");
                }
            }
            Job::Value(Value::List(items), display) => {
                out.push_str("차림["); jobs.push(Job::Text(Cow::Borrowed("]")));
                for (index, value) in items.iter().enumerate().rev() {
                    jobs.push(Job::Value(value, display));
                    if index != 0 { jobs.push(Job::Text(Cow::Borrowed(", "))); }
                }
            }
            Job::Value(Value::Tuple(items), display) => {
                out.push('(');jobs.push(Job::Text(Cow::Borrowed(")")));
                if items.len()==1 {jobs.push(Job::Text(Cow::Borrowed(",")));}
                for (index,value) in items.iter().enumerate().rev() {
                    jobs.push(Job::Value(value,display));
                    if index!=0 {jobs.push(Job::Text(Cow::Borrowed(", ")));}
                }
            }
            // Historically Record children use value_to_string, not the
            // display-only six-place policy. Do not change that contract.
            Job::Value(Value::Record { fields: items, .. }, _) => {
                out.push('('); fields(&mut jobs, items, false, ": ");
            }
            Job::Value(value @ Value::Pack(items), display) if numeric_pack_kind(value).is_none() => {
                if matches!(items.get(RELATION_KIND_FIELD), Some(Value::String(kind)) if kind == RELATION_KIND_EQUATION) {
                    for (name, separator) in [(RELATION_RIGHT_FIELD, Some(" =:= ")), (RELATION_LEFT_FIELD, None)] {
                        jobs.push(match items.get(name) { Some(value) => Job::Value(value, display),
                            None => Job::Text(Cow::Borrowed("[수식]")) });
                        if let Some(text) = separator { jobs.push(Job::Text(Cow::Borrowed(text))); }
                    }
                    continue;
                }
                if matches!(items.get(RELATION_SOLVE_RESULT_KIND_FIELD), Some(Value::String(kind)) if kind == RELATION_SOLVE_RESULT_SUCCESS) {
                    if let (Some(Value::String(variable)), Some(value)) = (items.get(RELATION_SOLVE_VAR_FIELD), items.get(RELATION_SOLVE_VALUE_FIELD)) {
                        out.push_str(&format!("#성공(미지수=\"{variable}\", 값="));
                        jobs.push(Job::Text(Cow::Borrowed(")"))); jobs.push(Job::Value(value, display));
                        continue;
                    }
                    if let Some(Value::Pack(bindings)) = items.get(RELATION_SOLVE_BINDINGS_FIELD) {
                        out.push_str("#성공(해=("); jobs.push(Job::Text(Cow::Borrowed(")")));
                        fields(&mut jobs, bindings, display, "="); continue;
                    }
                }
                if matches!(items.get(RELATION_SOLVE_RESULT_KIND_FIELD), Some(Value::String(kind)) if kind == RELATION_SOLVE_RESULT_FAILURE) {
                    let reason = match items.get(RELATION_SOLVE_REASON_FIELD) { Some(Value::String(text)) => text.as_str(), _ => "?" };
                    out.push_str(&format!("#실패(사유=\"{reason}\")")); continue;
                }
                out.push('('); fields(&mut jobs, items, display, ": ");
            }
            Job::Value(value, display) => out.push_str(&format_value_leaf_text(value, display)),
        }
    }
    out
}

pub(super) fn format_value_leaf_text(value: &Value, display_only: bool) -> String {
    match value {
        Value::Rational(value) => format!("{}/{}", value.ratio().numer(), value.ratio().denom()),
        Value::SumBranch {..} => unreachable!("Sum payloads use the heap formatter"),
        Value::Tuple(_) => map_key_canon(value),
        Value::Tensor(_) => map_key_canon(value),
        Value::Record { fields, .. } => format!("({})", fields.iter()
            .map(|(name, value)| format!("{name}: {}", value_to_string(value)))
            .collect::<Vec<_>>().join(", ")),
        Value::String(s) => s.clone(),
        Value::RegisteredAtom(atom) => ResourceValue::RegisteredAtom(atom.clone()).canon_key(),
        Value::IdentityReference(value) => ResourceValue::IdentityReference(value.clone()).canon_key(),
        Value::NominalValueV1(value) => ResourceValue::NominalValueV1(value.clone()).canon_key(),
        Value::FrozenSubjectViewV1(_) => "<임자 읽기값: 읽기 전용>".into(),
        Value::SubjectHandleV1(value) => ResourceValue::SubjectHandleV1(value.clone()).canon_key(),
        Value::Bytes(bytes) => bytes.canonical_literal(),
        Value::Fixed64(n) => if display_only { n.display_rounded_six() }
            else { format_fixed64_for_display(*n) },
        Value::Unit(unit) => {
            let suffix = unit
                .display_symbol()
                .map(|s| s.to_string())
                .unwrap_or_else(|| unit.dim.format());
            format!("{}@{}", unit.value, suffix)
        }
        Value::Bool(b) => {
            if *b {
                "참".to_string()
            } else {
                "거짓".to_string()
            }
        }
        Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
        Value::View(view) => match view.without_time() { ddonirang_lang::runtime::ViewValue::ResultTable => "<보기값:결과표>", _ => "<보기값:장면>" }.to_string(),
        Value::SceneElement(_) => "<장면요소값>".to_string(),
        Value::Scene(_) => "<장면값>".to_string(),
        Value::BogaeRelation(relation) => format!("<{}>", relation.kind_name()),
        Value::BogaeComposition(_) => "<보개짜임값>".to_string(),
        Value::TimePolicy(policy) => policy.canonical_text(),
        Value::None => String::new(),
        Value::List(items) => {
            let rendered = items
                .iter()
                .map(|value| format_value_text(value, display_only))
                .collect::<Vec<_>>()
                .join(", ");
            format!("차림[{rendered}]")
        }
        Value::Set(_) => "[모음]".to_string(),
        Value::Map(_) => "[짝맞춤]".to_string(),
        Value::Pack(items) => {
            if let Some(kind) = numeric_pack_kind(value) {
                return match kind {
                    NUMERIC_KIND_BIG_INT => match items.get("값") {
                        Some(Value::String(text)) => text.clone(),
                        _ => "[큰바른수]".to_string(),
                    },
                    NUMERIC_KIND_RATIONAL => {
                        let num = items
                            .get("분자")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        let den = items
                            .get("분모")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("{num}/{den}")
                    }
                    NUMERIC_KIND_FACTOR => items
                        .get("표기")
                        .and_then(|v| match v {
                            Value::String(text) => Some(text.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "[곱수]".to_string()),
                    _ => "[수형식]".to_string(),
                };
            }
            if let Some(Value::String(kind)) = items.get(RELATION_KIND_FIELD) {
                if kind == RELATION_KIND_EQUATION {
                    let lhs = items
                        .get(RELATION_LEFT_FIELD)
                        .map(|value| format_value_text(value, display_only))
                        .unwrap_or_else(|| "[수식]".to_string());
                    let rhs = items
                        .get(RELATION_RIGHT_FIELD)
                        .map(|value| format_value_text(value, display_only))
                        .unwrap_or_else(|| "[수식]".to_string());
                    return format!("{lhs} =:= {rhs}");
                }
            }
            if let Some(Value::String(kind)) = items.get(RELATION_SOLVE_RESULT_KIND_FIELD) {
                if kind == RELATION_SOLVE_RESULT_SUCCESS {
                    if let (Some(Value::String(variable)), Some(value)) = (
                        items.get(RELATION_SOLVE_VAR_FIELD),
                        items.get(RELATION_SOLVE_VALUE_FIELD),
                    ) {
                        return format!(
                            "#성공(미지수=\"{variable}\", 값={})",
                            format_value_text(value, display_only)
                        );
                    }
                    if let Some(Value::Pack(bindings)) = items.get(RELATION_SOLVE_BINDINGS_FIELD) {
                        let rendered = bindings
                            .iter()
                            .map(|(key, value)| format!("{key}={}", format_value_text(value, display_only)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        return format!("#성공(해=({rendered}))");
                    }
                }
                if kind == RELATION_SOLVE_RESULT_FAILURE {
                    let reason = items
                        .get(RELATION_SOLVE_REASON_FIELD)
                        .and_then(|v| match v {
                            Value::String(text) => Some(text.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "?".to_string());
                    return format!("#실패(사유=\"{reason}\")");
                }
            }
            let mut out = String::from("(");
            let mut first = true;
            for (key, value) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(key);
                out.push_str(": ");
                out.push_str(&format_value_text(value, display_only));
            }
            out.push(')');
            out
        }
        Value::Assertion(assertion) => assertion.canon.clone(),
        Value::StateMachine(machine) => state_machine_canon(machine),
        Value::Formula(formula) => formula_display_string(formula),
        Value::Template(template) => template.raw.clone(),
        Value::Regex(regex) => {
            if regex.flags.is_empty() {
                format!("정규식{{\"{}\"}}", regex.pattern)
            } else {
                format!("정규식{{\"{}\", \"{}\"}}", regex.pattern, regex.flags)
            }
        }
        Value::Lambda(lambda) => format!("<씨앗#{}>", lambda.id),
    }
}

fn format_fixed64_for_display(value: Fixed64) -> String {
    let raw = value.raw_i64();
    if raw == 0 {
        return "0".to_string();
    }
    let negative = raw < 0;
    let abs = if raw == i64::MIN {
        (i64::MAX as i128 + 1) as i128
    } else {
        raw.abs() as i128
    };
    let int_part = abs >> Fixed64::FRAC_BITS;
    let mut frac = abs & ((1_i128 << Fixed64::FRAC_BITS) - 1);
    let mut out = int_part.to_string();
    if frac != 0 {
        let mut digits = String::new();
        for _ in 0..10 {
            frac *= 10;
            let digit = (frac >> Fixed64::FRAC_BITS) as u8;
            frac &= (1_i128 << Fixed64::FRAC_BITS) - 1;
            digits.push((b'0' + digit) as char);
        }
        while digits.ends_with('0') {
            digits.pop();
        }
        if !digits.is_empty() {
            out.push('.');
            out.push_str(&digits);
        }
    }
    if negative {
        out.insert(0, '-');
    }
    out
}

pub(super) fn state_machine_canon(machine: &StateMachine) -> String {
    let mut out = String::from("상태머신{ ");
    out.push_str(&machine.states.join(", "));
    out.push_str(" 으로 이뤄짐. ");
    out.push_str(&machine.initial);
    out.push_str(" 으로 시작.");
    for transition in &machine.transitions {
        out.push(' ');
        out.push_str(&transition.from);
        out.push_str(" 에서 ");
        out.push_str(&transition.to);
        out.push_str(" 으로");
        if let Some(guard_name) = &transition.guard_name {
            out.push_str(" 걸러서 ");
            out.push_str(guard_name);
        }
        if let Some(action_name) = &transition.action_name {
            out.push_str(" 하고 ");
            out.push_str(action_name);
        }
        out.push('.');
    }
    for check in &machine.on_transition_checks {
        out.push(' ');
        out.push_str("바뀔때마다 ");
        out.push_str(check);
        out.push_str(" 살피기.");
    }
    out.push_str(" }");
    out
}

fn state_machine_resource_value(machine: &StateMachine) -> ResourceValue {
    let mut converted = BTreeMap::new();
    let insert = |map: &mut BTreeMap<String, ResourceMapEntry>, key: &str, value: ResourceValue| {
        let entry_key = ResourceValue::String(key.to_string());
        map.insert(
            entry_key.canon_key(),
            ResourceMapEntry {
                key: entry_key,
                value,
            },
        );
    };
    insert(
        &mut converted,
        "__ddn_kind",
        ResourceValue::String(STATE_MACHINE_RESOURCE_KIND.to_string()),
    );
    insert(
        &mut converted,
        "states",
        ResourceValue::List(
            machine
                .states
                .iter()
                .map(|state| ResourceValue::String(state.clone()))
                .collect(),
        ),
    );
    insert(
        &mut converted,
        "initial",
        ResourceValue::String(machine.initial.clone()),
    );
    insert(
        &mut converted,
        "checks",
        ResourceValue::List(
            machine
                .on_transition_checks
                .iter()
                .map(|check| ResourceValue::String(check.clone()))
                .collect(),
        ),
    );
    insert(
        &mut converted,
        "transitions",
        ResourceValue::List(
            machine
                .transitions
                .iter()
                .map(|transition| {
                    let mut transition_map = BTreeMap::new();
                    let from_key = ResourceValue::String("from".to_string());
                    transition_map.insert(
                        from_key.canon_key(),
                        ResourceMapEntry {
                            key: from_key,
                            value: ResourceValue::String(transition.from.clone()),
                        },
                    );
                    let to_key = ResourceValue::String("to".to_string());
                    transition_map.insert(
                        to_key.canon_key(),
                        ResourceMapEntry {
                            key: to_key,
                            value: ResourceValue::String(transition.to.clone()),
                        },
                    );
                    if let Some(guard_name) = &transition.guard_name {
                        let guard_key = ResourceValue::String("guard".to_string());
                        transition_map.insert(
                            guard_key.canon_key(),
                            ResourceMapEntry {
                                key: guard_key,
                                value: ResourceValue::String(guard_name.clone()),
                            },
                        );
                    }
                    if let Some(action_name) = &transition.action_name {
                        let action_key = ResourceValue::String("action".to_string());
                        transition_map.insert(
                            action_key.canon_key(),
                            ResourceMapEntry {
                                key: action_key,
                                value: ResourceValue::String(action_name.clone()),
                            },
                        );
                    }
                    ResourceValue::Map(transition_map)
                })
                .collect(),
        ),
    );
    ResourceValue::Map(converted)
}

fn assertion_resource_value(assertion: &Assertion) -> ResourceValue {
    let mut converted = BTreeMap::new();
    let insert = |map: &mut BTreeMap<String, ResourceMapEntry>, key: &str, value: ResourceValue| {
        let entry_key = ResourceValue::String(key.to_string());
        map.insert(
            entry_key.canon_key(),
            ResourceMapEntry {
                key: entry_key,
                value,
            },
        );
    };
    insert(
        &mut converted,
        "__ddn_kind",
        ResourceValue::String(ASSERTION_RESOURCE_KIND.to_string()),
    );
    insert(
        &mut converted,
        "canon",
        ResourceValue::String(assertion.canon.clone()),
    );
    ResourceValue::Map(converted)
}

fn resource_map_lookup<'a>(
    entries: &'a BTreeMap<String, ResourceMapEntry>,
    key: &str,
) -> Option<&'a ResourceValue> {
    entries.values().find_map(|entry| match &entry.key {
        ResourceValue::String(value) if value == key => Some(&entry.value),
        _ => None,
    })
}

pub(super) fn resource_map_to_state_machine(
    entries: &BTreeMap<String, ResourceMapEntry>,
) -> Option<StateMachine> {
    let kind = match resource_map_lookup(entries, "__ddn_kind")? {
        ResourceValue::String(value) => value,
        _ => return None,
    };
    if kind != STATE_MACHINE_RESOURCE_KIND {
        return None;
    }
    let states = match resource_map_lookup(entries, "states")? {
        ResourceValue::List(items) => items
            .iter()
            .map(|item| match item {
                ResourceValue::String(value) => Some(value.clone()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    let initial = match resource_map_lookup(entries, "initial")? {
        ResourceValue::String(value) => value.clone(),
        _ => return None,
    };
    let checks = match resource_map_lookup(entries, "checks") {
        Some(value) => match value {
            ResourceValue::List(items) => items
                .iter()
                .map(|item| match item {
                    ResourceValue::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()?,
            _ => return None,
        },
        None => Vec::new(),
    };
    let transitions = match resource_map_lookup(entries, "transitions")? {
        ResourceValue::List(items) => items
            .iter()
            .map(|item| match item {
                ResourceValue::Map(map) => {
                    let from = match resource_map_lookup(&map, "from")? {
                        ResourceValue::String(value) => value.clone(),
                        _ => return None,
                    };
                    let to = match resource_map_lookup(&map, "to")? {
                        ResourceValue::String(value) => value.clone(),
                        _ => return None,
                    };
                    let guard_name = match resource_map_lookup(&map, "guard") {
                        Some(ResourceValue::String(value)) => Some(value.clone()),
                        Some(_) => return None,
                        None => None,
                    };
                    let action_name = match resource_map_lookup(&map, "action") {
                        Some(ResourceValue::String(value)) => Some(value.clone()),
                        Some(_) => return None,
                        None => None,
                    };
                    Some(StateTransition {
                        from,
                        to,
                        guard_name,
                        action_name,
                    })
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    Some(StateMachine {
        states,
        initial,
        transitions,
        on_transition_checks: checks,
    })
}

fn resource_map_to_assertion(entries: &BTreeMap<String, ResourceMapEntry>) -> Option<Assertion> {
    let kind = match resource_map_lookup(entries, "__ddn_kind")? {
        ResourceValue::String(value) => value,
        _ => return None,
    };
    if kind != ASSERTION_RESOURCE_KIND {
        return None;
    }
    let canon = match resource_map_lookup(entries, "canon")? {
        ResourceValue::String(value) => value.clone(),
        _ => return None,
    };
    let body_source = extract_assertion_body_source(&canon)?;
    Some(Assertion { body_source, canon })
}

fn formula_resource_value(formula: &Formula) -> ResourceValue {
    let mut converted = BTreeMap::new();
    let insert = |map: &mut BTreeMap<String, ResourceMapEntry>, key: &str, value: ResourceValue| {
        let entry_key = ResourceValue::String(key.to_string());
        map.insert(
            entry_key.canon_key(),
            ResourceMapEntry {
                key: entry_key,
                value,
            },
        );
    };
    insert(
        &mut converted,
        "__ddn_kind",
        ResourceValue::String(FORMULA_RESOURCE_KIND.to_string()),
    );
    insert(
        &mut converted,
        "raw",
        ResourceValue::String(formula.raw.clone()),
    );
    let dialect = match &formula.dialect {
        FormulaDialect::Ascii => "ascii".to_string(),
        FormulaDialect::Ascii1 => "ascii1".to_string(),
        FormulaDialect::Latex => "latex".to_string(),
        FormulaDialect::Other(value) => format!("other:{value}"),
    };
    insert(&mut converted, "dialect", ResourceValue::String(dialect));
    insert(
        &mut converted,
        "explicit_tag",
        ResourceValue::Bool(formula.explicit_tag),
    );
    ResourceValue::Map(converted)
}

pub(crate) fn formula_from_resource_value(value: &ResourceValue) -> Option<Formula> {
    match value {
        ResourceValue::Map(entries) => resource_map_to_formula(entries),
        _ => None,
    }
}

pub(crate) fn formula_summary_parts(formula: &Formula) -> (String, String) {
    let raw = format_formula_body(&formula.raw, &formula.dialect)
        .unwrap_or_else(|_| formula.raw.trim().to_string());
    (formula_dialect_tag(&formula.dialect), raw)
}

pub(crate) fn formula_display_string(formula: &Formula) -> String {
    let (tag, raw) = formula_summary_parts(formula);
    if formula.explicit_tag {
        format!("({tag}) 수식{{ {raw} }}")
    } else {
        format!("수식{{ {raw} }}")
    }
}

fn formula_dialect_tag(dialect: &FormulaDialect) -> String {
    match dialect {
        FormulaDialect::Ascii => "#ascii".to_string(),
        FormulaDialect::Ascii1 => "#ascii1".to_string(),
        FormulaDialect::Latex => "#latex".to_string(),
        FormulaDialect::Other(value) => format!("#{value}"),
    }
}

fn resource_map_to_formula(entries: &BTreeMap<String, ResourceMapEntry>) -> Option<Formula> {
    let kind = match resource_map_lookup(entries, "__ddn_kind")? {
        ResourceValue::String(value) => value,
        _ => return None,
    };
    if kind != FORMULA_RESOURCE_KIND {
        return None;
    }
    let raw = match resource_map_lookup(entries, "raw")? {
        ResourceValue::String(value) => value.clone(),
        _ => return None,
    };
    let dialect = match resource_map_lookup(entries, "dialect")? {
        ResourceValue::String(value) if value == "ascii" => FormulaDialect::Ascii,
        ResourceValue::String(value) if value == "ascii1" => FormulaDialect::Ascii1,
        ResourceValue::String(value) if value == "latex" => FormulaDialect::Latex,
        ResourceValue::String(value) => FormulaDialect::Other(
            value
                .strip_prefix("other:")
                .unwrap_or(value.as_str())
                .to_string(),
        ),
        _ => return None,
    };
    let explicit_tag = match resource_map_lookup(entries, "explicit_tag") {
        Some(ResourceValue::Bool(value)) => *value,
        Some(_) => return None,
        None => false,
    };
    Some(Formula {
        raw,
        dialect,
        explicit_tag,
    })
}

pub(super) fn runtime_value_contains_tuple(root: &Value) -> bool {
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        match value {
            Value::Tuple(_) | Value::Rational(_) => return true,
            Value::List(items) => pending.extend(items.iter()),
            Value::Pack(fields) | Value::Set(fields) | Value::Record { fields, .. } => pending.extend(fields.values()),
            Value::Map(items) => for item in items.values() { pending.push(&item.key); pending.push(&item.value); },
            _ => {},
        }
    }
    false
}

pub(crate) fn resource_value_from_typed_value(value: &Value, kind: &ddonirang_core::platform::RecordFieldType) -> Result<ResourceValue, EvalError> {
    resource_value_from_typed_value_with_output(value,kind,None)
}
pub(crate) fn resource_value_from_typed_value_with_output(value:&Value,kind:&ddonirang_core::platform::RecordFieldType,
    output:Option<&crate::registered_atom_contract::output_evidence::OutputValueContextV1<'_>>)->Result<ResourceValue,EvalError>{
    if value.contains_transient_subject_view(){return Err("E_VC04_READ_SNAPSHOT_ESCAPE: 임자 읽기값에는 저장 형식이 없습니다".to_string().into());}
    use ddonirang_core::platform::RecordFieldType as R;
    enum Job<'a> { Pair(&'a Value, &'a R), Tuple(&'a [R], usize), List(usize), Record(&'a BTreeMap<String, R>) }
    let unbound = || EvalError::from("E_TUPLE_PERSISTENCE_UNBOUND: 위치별 씨와 평가값의 결속이 다릅니다".to_owned());
    let mut pending = vec![Job::Pair(value, kind)];
    let mut built = Vec::<ResourceValue>::new();
    while let Some(job) = pending.pop() {
        match job {
            Job::Pair(value,R::RegisteredAtomV1)=>{
                let context=output.ok_or_else(unbound)?;
                built.push(context.registered_atom(value).map_err(EvalError::from)?);
            },
            Job::Pair(Value::RegisteredAtom(_),_) if output.is_some()=>return Err(unbound()),
            Job::Pair(Value::Rational(value), R::Named { name }) if name == "나눔수" => {
                built.push(ResourceValue::RationalV1(value.clone()));
            }
            Job::Pair(Value::Rational(_), _) => return Err(unbound()),
            Job::Pair(Value::Tuple(values), R::Tuple { items: types }) => {
                if values.len() != types.len() { return Err(unbound()); }
                pending.push(Job::Tuple(types, values.len()));
                pending.extend(values.iter().zip(types).rev().map(|(value, kind)| Job::Pair(value, kind)));
            }
            Job::Pair(Value::List(values), R::Applied { name, args }) if name == "차림" && args.len() == 1 => {
                pending.push(Job::List(values.len()));
                pending.extend(values.iter().rev().map(|value| Job::Pair(value, &args[0])));
            }
            Job::Pair(Value::Record { field_types, fields }, R::Record { fields: types }) => {
                if types != field_types || !types.keys().eq(fields.keys()) { return Err(unbound()); }
                pending.push(Job::Record(types));
                pending.extend(fields.values().zip(types.values()).rev().map(|(value, kind)| Job::Pair(value, kind)));
            }
            Job::Pair(Value::Tuple(_), _) | Job::Pair(_, R::Tuple { .. }) => return Err(unbound()),
            Job::Pair(value, kind) => {
                let converted = resource_value_from_value(value)?;
                crate::unit_wire::validate_saved_value_type(&converted, kind).map_err(|_| unbound())?;
                built.push(converted);
            }
            Job::Tuple(types, n) => {
                let items = built.split_off(built.len() - n);
                built.push(ResourceValue::Tuple { element_types: types.to_vec(), items });
            }
            Job::List(n) => { let items = built.split_off(built.len() - n); built.push(ResourceValue::List(items)); }
            Job::Record(types) => {
                let items = built.split_off(built.len() - types.len());
                built.push(ResourceValue::Record { field_types: types.clone(), fields: types.keys().cloned().zip(items).collect() });
            }
        }
    }
    built.pop().ok_or_else(unbound)
}

pub(super) fn resource_value_from_value(value: &Value) -> Result<ResourceValue, EvalError> {
    enum Job<'a> {
        Value(&'a Value), List(usize), Set(Vec<String>), Map(Vec<String>), Pack(Vec<String>),
        Record(&'a BTreeMap<String, ddonirang_core::platform::RecordFieldType>, Vec<String>),
    }
    struct Built(Vec<ResourceValue>);
    impl Drop for Built {
        fn drop(&mut self) { for value in self.0.drain(..) { value.drop_stack_safe(); } }
    }
    let mut built = Built(Vec::new());
    let mut pending = vec![Job::Value(value)];
    while let Some(job) = pending.pop() {
        let value = match job {
            Job::Value(value) => value,
            Job::List(count) => {
                let values = built.0.split_off(built.0.len() - count);
                built.0.push(ResourceValue::List(values)); continue;
            }
            Job::Set(keys) => {
                let values = built.0.split_off(built.0.len() - keys.len());
                built.0.push(ResourceValue::Set(keys.into_iter().zip(values).collect())); continue;
            }
            Job::Record(field_types, keys) => {
                let values = built.0.split_off(built.0.len() - keys.len());
                built.0.push(ResourceValue::Record { field_types: field_types.clone(), fields: keys.into_iter().zip(values).collect() }); continue;
            }
            Job::Map(keys) => {
                let values = built.0.split_off(built.0.len() - keys.len() * 2);
                let mut values = values.into_iter();
                let entries = keys.into_iter().map(|index| (index, ResourceMapEntry {
                    key: values.next().expect("map key"), value: values.next().expect("map value"),
                })).collect();
                built.0.push(ResourceValue::Map(entries)); continue;
            }
            Job::Pack(keys) => {
                let values = built.0.split_off(built.0.len() - keys.len());
                let entries = keys.into_iter().zip(values).map(|(key, value)| {
                    let key = ResourceValue::String(key);
                    (key.canon_key(), ResourceMapEntry { key, value })
                }).collect();
                built.0.push(ResourceValue::Map(entries)); continue;
            }
        };
        let converted = match value {
        Value::FrozenSubjectViewV1(_) => Err("E_VC04_READ_SNAPSHOT_ESCAPE: 임자 읽기값에는 저장 형식이 없습니다".to_string().into()),
        Value::Tuple(_) => Err("E_TUPLE_PERSISTENCE_UNBOUND: 위치형 값꾸러미의 저장 계약이 결속되지 않았습니다".to_string().into()),
        Value::Record { field_types, fields } => {
            ddonirang_lang::runtime::validate_record_value(value).map_err(|detail|
                type_mismatch_error("묶음 복원", &detail.expected, &detail.actual))?;
            pending.push(Job::Record(field_types, fields.keys().cloned().collect()));
            pending.extend(fields.values().rev().map(Job::Value));
            continue;
        }
        Value::Tensor(value) => crate::unit_wire::tensor_to_resource(value).map_err(EvalError::from),
        Value::Rational(_) => Err("E_RATIONAL_SOURCE_TYPE_REQUIRED: 나눔수 저장에는 원문에 결속된 씨가 필요합니다".to_string().into()),
        Value::None => Ok(ResourceValue::None),
        Value::Bool(value) => Ok(ResourceValue::Bool(*value)),
        Value::Fixed64(value) => Ok(ResourceValue::Fixed64(*value)),
        Value::Unit(value) => Ok(ResourceValue::Unit(*value)),
        Value::String(value) => Ok(ResourceValue::String(value.clone())),
        Value::RegisteredAtom(atom) => Ok(ResourceValue::RegisteredAtom(atom.clone())),
        Value::IdentityReference(value) => Ok(ResourceValue::IdentityReference(value.clone())),
        Value::NominalValueV1(value) => Ok(ResourceValue::NominalValueV1(value.clone())),
        Value::SubjectHandleV1(value) => Ok(ResourceValue::SubjectHandleV1(value.clone())),
        Value::Bytes(bytes) => Ok(ResourceValue::Bytes(bytes.clone())),
        Value::ResourceHandle(value) => Ok(ResourceValue::ResourceHandle(*value)),
        Value::List(items) => {
            pending.push(Job::List(items.len()));
            pending.extend(items.iter().rev().map(Job::Value));
            continue;
        }
        Value::Set(items) => {
            pending.push(Job::Set(items.keys().cloned().collect()));
            pending.extend(items.values().rev().map(Job::Value));
            continue;
        }
        Value::Map(entries) => {
            pending.push(Job::Map(entries.keys().cloned().collect()));
            for entry in entries.values().rev() {
                pending.push(Job::Value(&entry.value));
                pending.push(Job::Value(&entry.key));
            }
            continue;
        }
        Value::Pack(entries) => {
            pending.push(Job::Pack(entries.keys().cloned().collect()));
            pending.extend(entries.values().rev().map(Job::Value));
            continue;
        }
        Value::Assertion(assertion) => Ok(assertion_resource_value(assertion)),
        Value::StateMachine(machine) => Ok(state_machine_resource_value(machine)),
        Value::Formula(formula) => Ok(formula_resource_value(formula)),
        Value::Template(_) => Err("글무늬는 자원에 대입할 수 없습니다".to_string().into()),
        Value::Regex(_) => Err("정규식은 자원에 대입할 수 없습니다".to_string().into()),
        Value::Lambda(_) => Err("씨앗은 자원에 대입할 수 없습니다".to_string().into()),
        Value::SumBranch {..} => Err("고름씨 갈래값은 자원에 대입할 수 없습니다".to_string().into()),
        Value::View(_) => Err("보기값은 세계 자원에 대입할 수 없습니다".to_string().into()),
        Value::SceneElement(_) | Value::Scene(_) => Err("장면 값은 세계 자원에 대입할 수 없습니다".to_string().into()),
        Value::BogaeRelation(_) => Err("보개관계값은 세계 자원에 대입할 수 없습니다".to_string().into()),
        Value::BogaeComposition(_) => Err("보개짜임값은 세계 자원에 대입할 수 없습니다".to_string().into()),
        Value::TimePolicy(_) => Err("보개시간값은 세계 자원에 대입할 수 없습니다".to_string().into()),
        }?;
        built.0.push(converted);
    }
    Ok(built.0.pop().expect("saved root"))
}

pub(super) fn public_observation_type(value: &Value) -> (&'static str, String) {
    match value {
        Value::FrozenSubjectViewV1(_) => ("transient_subject_view",String::new()),
        Value::Rational(_) => ("rational_v1", String::new()),
        Value::SumBranch {..} => ("sum_branch",String::new()),
        Value::Tuple(_) => ("tuple", String::new()),
        Value::Tensor(_) => ("basic_tensor_v1", String::new()),
        Value::None => ("none", String::new()),
        Value::Bool(_) => ("bool", String::new()),
        Value::Fixed64(_) => ("fixed64", String::new()),
        Value::Unit(unit) => (
            "unit",
            unit.display_symbol()
                .map(str::to_string)
                .unwrap_or_else(|| unit.dim.format()),
        ),
        Value::String(_) => ("string", String::new()),
        Value::RegisteredAtom(_) => ("registered_atom_v1", String::new()),
        Value::IdentityReference(_) => ("identity_reference_v1",String::new()),
        Value::NominalValueV1(_) => ("nominal_value_v1",String::new()),
        Value::SubjectHandleV1(_) => ("subject_handle_v1",String::new()),
        Value::Bytes(_) => ("bytes", String::new()),
        Value::ResourceHandle(_) => ("resource_handle", String::new()),
        Value::List(_) => ("list", String::new()),
        Value::Set(_) => ("set", String::new()),
        Value::Map(_) => ("map", String::new()),
        Value::Pack(_) | Value::Record { .. } => ("pack", String::new()),
        Value::Assertion(_) => ("assertion", String::new()),
        Value::StateMachine(_) => ("state_machine", String::new()),
        Value::Formula(_) => ("formula", String::new()),
        Value::Template(_) => ("template", String::new()),
        Value::Regex(_) => ("regex", String::new()),
        Value::Lambda(_) => ("lambda", String::new()),
        Value::View(_) => ("view", String::new()),
        Value::SceneElement(_) => ("scene_element", String::new()),
        Value::Scene(_) => ("scene", String::new()),
        Value::BogaeRelation(_) => ("bogae_relation", String::new()),
        Value::BogaeComposition(_) => ("bogae_composition", String::new()),
        Value::TimePolicy(_) => ("bogae_time", String::new()),
    }
}

pub(super) struct DatasetPublicObservationScalar {
    pub(super) name: String,
    pub(super) dtype: String,
    pub(super) unit: String,
    pub(super) value_canon: String,
    pub(super) value_text: String,
    pub(super) dataset_content_sha256: String,
    pub(super) row_source_ordinal: u64,
}

pub(super) fn dataset_public_observation_scalar(value: &Value) -> Option<DatasetPublicObservationScalar> {
    let Value::Pack(fields) = value else {
        return None;
    };
    if fields.len() != 7 {
        return None;
    }
    let name = match fields.get("name")? {
        Value::String(value) if !value.is_empty() => value.clone(),
        _ => return None,
    };
    let kind = match fields.get("kind")? {
        Value::String(value) => value.as_str(),
        _ => return None,
    };
    let unit = match fields.get("unit")? {
        Value::String(value) => value.clone(),
        Value::None => String::new(),
        _ => return None,
    };
    let _source_locator = match fields.get("source_locator")? {
        Value::String(value) if !value.is_empty() => value,
        _ => return None,
    };
    let content_sha256 = match fields.get("content_sha256")? {
        Value::String(value)
            if value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) =>
        {
            value.clone()
        }
        _ => return None,
    };
    let row_source_ordinal = match fields.get("row_source_ordinal")? {
        Value::String(value) => value.parse::<u64>().ok()?,
        _ => return None,
    };
    let raw = fields.get("value")?;
    let (value_canon, value_text) = match (kind, raw) {
        ("integer" | "fixed64", Value::String(value)) => (value.clone(), value.clone()),
        ("string", Value::String(value)) => {
            (format!("\"{}\"", escape_canon_string(value)), value.clone())
        }
        ("boolean", Value::Bool(value)) => {
            let text = if *value { "참" } else { "거짓" }.to_string();
            (text.clone(), text)
        }
        _ => return None,
    };
    Some(DatasetPublicObservationScalar {
        name,
        dtype: kind.to_string(),
        unit,
        value_canon,
        value_text,
        dataset_content_sha256: content_sha256,
        row_source_ordinal,
    })
}

pub(super) fn canonical_dataset_observation_name(name: &str) -> Result<String, EvalError> {
    let tokens = Lexer::new(name)
        .tokenize()
        .map_err(|err| format!("E_DATASET_OBSERVATION_NAME: {}", err.message))?;
    if tokens.len() != 2 || !matches!(tokens[1].kind, TokenKind::Eof) {
        return Err(format!("E_DATASET_OBSERVATION_NAME: 자료 열 이름 {name}").into());
    }
    token_observation_name(&tokens[0], name)
        .ok_or_else(|| format!("E_DATASET_OBSERVATION_NAME: 자료 열 이름 {name}").into())
}

pub(crate) fn value_from_resource_value(value: &ResourceValue) -> Value {
    // Private callers consume resources already admitted by their codec or
    // runner boundary. Untrusted NuriWorld values use the fallible path below.
    try_value_from_resource_value(value).expect("resource admitted before runtime conversion")
}

pub(super) fn try_value_from_resource_value(value: &ResourceValue) -> Result<Value, String> {
    enum Job<'a> {
        Tuple(usize),
        Value(&'a ResourceValue), List(usize), Set(Vec<String>), Map(Vec<String>),
        Pack(Vec<String>), Record(&'a BTreeMap<String, ddonirang_core::platform::RecordFieldType>, Vec<String>),
    }
    struct Built(Vec<Value>);
    impl Drop for Built {
        fn drop(&mut self) {
            for value in self.0.drain(..) { value.drop_saved_tree_stack_safe(); }
        }
    }
    fn take_fields(keys: Vec<String>, built: &mut Vec<Value>) -> BTreeMap<String, Value> {
        let items = built.split_off(built.len() - keys.len());
        let mut fields = BTreeMap::new();
        for (key, value) in keys.into_iter().zip(items) {
            if let Some(previous) = fields.insert(key, value) { previous.drop_saved_tree_stack_safe(); }
        }
        fields
    }
    let mut pending = vec![Job::Value(value)];
    let mut built = Built(Vec::new());
    while let Some(job) = pending.pop() {
        let value = match job {
            Job::Value(value) => value,
            Job::Tuple(count) => {
                let items = built.0.split_off(built.0.len() - count);
                built.0.push(Value::Tuple(items)); continue;
            }
            Job::List(count) => {
                let items = built.0.split_off(built.0.len() - count);
                built.0.push(Value::List(items)); continue;
            }
            Job::Set(keys) => {
                let fields = take_fields(keys, &mut built.0);
                built.0.push(Value::Set(fields)); continue;
            }
            Job::Pack(keys) => {
                let fields = take_fields(keys, &mut built.0);
                built.0.push(Value::Pack(fields)); continue;
            }
            Job::Record(field_types, keys) => {
                let fields = take_fields(keys, &mut built.0);
                built.0.push(Value::Record { field_types: field_types.clone(), fields }); continue;
            }
            Job::Map(keys) => {
                let items = built.0.split_off(built.0.len() - keys.len() * 2);
                let mut items = items.into_iter();
                let converted = keys.into_iter().map(|key| (key, MapEntry {
                    key: items.next().expect("map key"), value: items.next().expect("map value"),
                })).collect();
                built.0.push(Value::Map(converted)); continue;
            }
        };
        let converted = match value {
        ResourceValue::RationalV1(value) => Value::Rational(value.clone()),
        ResourceValue::NominalValueV1(value) => Value::NominalValueV1(value.clone()),
        ResourceValue::SubjectHandleV1(value) => Value::SubjectHandleV1(value.clone()),
        ResourceValue::SubjectPayloadV1(_) => return Err(
            "E_RESOURCE_SNAPSHOT_INVALID: Subject payload is field05-only, not an ordinary resource".into()),
        ResourceValue::Tuple { items, .. } => {
            pending.push(Job::Tuple(items.len()));
            pending.extend(items.iter().rev().map(Job::Value));
            continue;
        }
        ResourceValue::BasicTensorV1(carrier) => Value::Tensor(
            crate::unit_wire::tensor_from_carrier(carrier)
                .map_err(|cause| format!("E_RESOURCE_SNAPSHOT_INVALID: {cause}"))?),
        ResourceValue::Record { field_types, fields } => {
            pending.push(Job::Record(field_types, fields.keys().cloned().collect()));
            pending.extend(fields.values().rev().map(Job::Value));
            continue;
        },
        ResourceValue::None => Value::None,
        ResourceValue::Bool(value) => Value::Bool(*value),
        ResourceValue::Fixed64(value) => Value::Fixed64(*value),
        ResourceValue::Unit(value) => Value::Unit(*value),
        ResourceValue::String(value) => Value::String(value.clone()),
        ResourceValue::RegisteredAtom(atom) => Value::RegisteredAtom(atom.clone()),
        ResourceValue::IdentityReference(value) => Value::IdentityReference(value.clone()),
        ResourceValue::Bytes(bytes) => Value::Bytes(bytes.clone()),
        ResourceValue::ResourceHandle(value) => Value::ResourceHandle(*value),
        ResourceValue::List(items) => {
            pending.push(Job::List(items.len()));
            pending.extend(items.iter().rev().map(Job::Value));
            continue;
        },
        ResourceValue::Set(items) => {
            pending.push(Job::Set(items.keys().cloned().collect()));
            pending.extend(items.values().rev().map(Job::Value));
            continue;
        }
        ResourceValue::Map(entries) => {
            if let Some(assertion) = resource_map_to_assertion(entries) {
                built.0.push(Value::Assertion(assertion)); continue;
            }
            if let Some(machine) = resource_map_to_state_machine(entries) {
                built.0.push(Value::StateMachine(machine)); continue;
            }
            if let Some(formula) = resource_map_to_formula(entries) {
                built.0.push(Value::Formula(formula)); continue;
            }
            // These existing standard-library records already carry their kind
            // in the committed resource map. Restore their Pack representation
            // structurally; otherwise a session accepted in one tick becomes an
            // incompatible ordinary Map at the next tick.
            let is_grid_record = entries.values().any(|entry| match (&entry.key, &entry.value) {
                (ResourceValue::String(key), ResourceValue::String(kind)) =>
                    (key == "__kind" && [STD_GRID_KIND, STD_INPUT_MAP_KIND].contains(&kind.as_str()))
                    || (key == "__종류" && [STD_BLOCK_PIECE_KIND, STD_RANDOM_BAG_KIND,
                STD_RANDOM_BAG_DRAW_KIND, STD_GRID_GAME_STATE_KIND, STD_GRID_LINE_CLEAR_KIND,
                STD_FALLING_PIECE_KIND, STD_GRID_GAME_SCORE_KIND, STD_GRID_GAME_SESSION_KIND,
                STD_GRID_GAME_TICK_KIND, STD_GRID_GAME_NEXT_PIECE_KIND, STD_GRID_GAME_VIEW_SUMMARY_KIND,
                STD_GRID_GAME_HOLD_KIND, STD_GRID_GAME_HOLD_SWAP_KIND, STD_GRID_GAME_ROTATION_TRY_KIND].contains(&kind.as_str())),
                _ => false,
            });
            if is_grid_record
                && entries.values().all(|entry| matches!(entry.key, ResourceValue::String(_))) {
                let keys = entries.values().map(|entry| match &entry.key {
                    ResourceValue::String(key) => key.clone(), _ => unreachable!("string keys"),
                }).collect();
                pending.push(Job::Pack(keys));
                pending.extend(entries.values().rev().map(|entry| Job::Value(&entry.value)));
                continue;
            }
            pending.push(Job::Map(entries.keys().cloned().collect()));
            for entry in entries.values().rev() {
                pending.push(Job::Value(&entry.value));
                pending.push(Job::Value(&entry.key));
            }
            continue;
        }
        };
        built.0.push(converted);
    }
    Ok(built.0.pop().expect("restored root"))
}


fn escape_canon_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

pub(super) fn value_canon(value: &Value) -> String {
    match value {
        Value::FrozenSubjectViewV1(value) => format!("transient-subject-view:{:?}",value.canonical_body()),
        Value::Rational(value) => format!("나눔수{{{}/{}}}", value.ratio().numer(), value.ratio().denom()),
        Value::SumBranch {..} => map_key_canon(value),
        Value::Tuple(_) => map_key_canon(value),
        Value::Tensor(_) => map_key_canon(value),
        Value::None => "없음".to_string(),
        Value::Bool(true) => "참".to_string(),
        Value::Bool(false) => "거짓".to_string(),
        Value::Fixed64(n) => n.canonical_decimal_text(),
        Value::Unit(unit) => {
            let suffix = unit
                .display_symbol()
                .map(|s| s.to_string())
                .unwrap_or_else(|| unit.dim.format());
            format!("{}@{}", unit.value.canonical_decimal_text(), suffix)
        }
        Value::String(s) => format!("\"{}\"", escape_canon_string(s)),
        Value::RegisteredAtom(atom) => ResourceValue::RegisteredAtom(atom.clone()).canon_key(),
        Value::IdentityReference(value) => ResourceValue::IdentityReference(value.clone()).canon_key(),
        Value::NominalValueV1(value) => ResourceValue::NominalValueV1(value.clone()).canon_key(),
        Value::SubjectHandleV1(value) => ResourceValue::SubjectHandleV1(value.clone()).canon_key(),
        Value::Bytes(bytes) => bytes.canonical_literal(),
        Value::View(view) => match view.without_time() { ddonirang_lang::runtime::ViewValue::ResultTable => "<보기값:결과표>", _ => "<보기값:장면>" }.to_string(),
        Value::SceneElement(_) => "<장면요소값>".to_string(),
        Value::Scene(_) => "<장면값>".to_string(),
        Value::BogaeRelation(relation) => format!("<{}>", relation.kind_name()),
        Value::BogaeComposition(_) => "<보개짜임값>".to_string(),
        Value::TimePolicy(policy) => policy.canonical_text(),
        Value::ResourceHandle(handle) => format!("자원#{}", handle.to_hex()),
        Value::List(items) => {
            let mut out = String::from("차림[");
            let mut first = true;
            for item in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&value_canon(item));
            }
            out.push(']');
            out
        }
        Value::Set(items) => {
            let mut out = String::from("모음{");
            let mut first = true;
            for (_canon_key, value) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&value_canon(value));
            }
            out.push('}');
            out
        }
        Value::Map(entries) => {
            let mut out = String::from("짝맞춤{");
            let mut first = true;
            for (_canon_key, entry) in entries {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(&value_canon(&entry.key));
                out.push_str("=>");
                out.push_str(&value_canon(&entry.value));
            }
            out.push('}');
            out
        }
        Value::Pack(items) | Value::Record { fields: items, .. } => {
            if let Some(kind) = numeric_pack_kind(value) {
                return match kind {
                    NUMERIC_KIND_BIG_INT => {
                        let body = items
                            .get("값")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("큰바른수{{{body}}}")
                    }
                    NUMERIC_KIND_RATIONAL => {
                        let num = items
                            .get("분자")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        let den = items
                            .get("분모")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("나눔수{{{num}/{den}}}")
                    }
                    NUMERIC_KIND_FACTOR => {
                        let body = items
                            .get("표기")
                            .and_then(|v| match v {
                                Value::String(text) => Some(text.clone()),
                                _ => None,
                            })
                            .unwrap_or_else(|| "?".to_string());
                        format!("곱수{{{body}}}")
                    }
                    _ => "수형식{}".to_string(),
                };
            }
            let mut out = String::from("묶음{");
            let mut first = true;
            for (key, value) in items {
                if !first {
                    out.push_str(", ");
                }
                first = false;
                out.push_str(key);
                out.push('=');
                out.push_str(&value_canon(value));
            }
            out.push('}');
            out
        }
        Value::Assertion(assertion) => assertion.canon.clone(),
        Value::StateMachine(machine) => state_machine_canon(machine),
        Value::Formula(formula) => formula.raw.clone(),
        Value::Template(template) => template.raw.clone(),
        Value::Regex(regex) => {
            if regex.flags.is_empty() {
                format!("정규식{{\"{}\"}}", regex.pattern)
            } else {
                format!("정규식{{\"{}\", \"{}\"}}", regex.pattern, regex.flags)
            }
        }
        Value::Lambda(lambda) => format!("<씨앗#{}>", lambda.id),
    }
}

pub(super) fn render_template(
    template: &ddonirang_lang::Template,
    pack: &BTreeMap<String, Value>,
) -> Result<String, EvalError> {
    let mut required = BTreeSet::new();
    for part in &template.parts {
        let TemplatePart::Placeholder(placeholder) = part else {
            continue;
        };
        if let Some(root) = placeholder.path.first() {
            required.insert(root.clone());
        }
    }
    let provided: BTreeSet<String> = pack.keys().cloned().collect();
    let missing: Vec<String> = required.difference(&provided).cloned().collect();
    if !missing.is_empty() {
        return Err(format!("채우기: 주입 키가 누락되었습니다: {}", missing.join(", ")).into());
    }
    let extra: Vec<String> = provided.difference(&required).cloned().collect();
    if !extra.is_empty() {
        return Err(format!("채우기: 주입 키가 여분입니다: {}", extra.join(", ")).into());
    }
    let mut out = String::new();
    for part in &template.parts {
        match part {
            TemplatePart::Text(text) => out.push_str(text),
            TemplatePart::Placeholder(placeholder) => {
                let value = resolve_template_value(pack, &placeholder.path)?;
                let rendered = format_template_value(value, placeholder.format.as_ref())?;
                out.push_str(&rendered);
            }
        }
    }
    Ok(out)
}

fn resolve_template_value<'a>(
    pack: &'a BTreeMap<String, Value>,
    path: &[String],
) -> Result<&'a Value, EvalError> {
    let Some(root) = path.first() else {
        return Err("글무늬 자리표시자 경로가 비었습니다".to_string().into());
    };
    let mut current = pack
        .get(root)
        .ok_or_else(|| format!("채우기: 키 '{}'가 없습니다", root))?;
    if matches!(current, Value::None) {
        return Err(format!("채우기: 키 '{}' 값이 없습니다", root).into());
    }
    let mut current_path = root.clone();
    for seg in path.iter().skip(1) {
        let next_path = format!("{}.{}", current_path, seg);
        let Value::Pack(map) = current else {
            return Err(format!("채우기: '{}'는 묶음이 아닙니다", current_path).into());
        };
        current = map
            .get(seg)
            .ok_or_else(|| format!("채우기: 필드 '{}'가 없습니다", next_path))?;
        if matches!(current, Value::None) {
            return Err(format!("채우기: 필드 '{}' 값이 없습니다", next_path).into());
        }
        current_path = next_path;
    }
    Ok(current)
}

fn format_template_value(
    value: &Value,
    format: Option<&TemplateFormat>,
) -> Result<String, EvalError> {
    let Some(format) = format else {
        if matches!(value, Value::None) {
            return Err("채우기: 자리표시자 값이 없습니다".to_string().into());
        }
        return Ok(value_to_string(value));
    };
    let (number, suffix) = match value {
        Value::Fixed64(v) => {
            if format.unit.is_some() {
                return Err("글무늬 포맷 단위는 단위 값에만 적용됩니다"
                    .to_string()
                    .into());
            }
            (*v, None)
        }
        Value::Unit(unit) => {
            if let Some(unit_str) = &format.unit {
                let Some(spec) = unit_spec_from_symbol(unit_str) else {
                    return Err(format!("글무늬 포맷 단위 '{}'를 알 수 없습니다", unit_str).into());
                };
                let converted = unit.to_unit(spec).map_err(unit_error)?;
                (converted, Some(unit_str.clone()))
            } else {
                let symbol = unit
                    .display_symbol()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| unit.dim.format());
                (unit.value, Some(symbol))
            }
        }
        Value::None => {
            return Err("채우기: 자리표시자 값이 없습니다".to_string().into());
        }
        _ => {
            return Err("글무늬 포맷은 수치에만 적용됩니다".to_string().into());
        }
    };
    let precision = format.precision.unwrap_or(0);
    let mut out = format_fixed64_with_precision(number, precision);
    if let Some(width) = format.width {
        out = apply_width(out, width, format.zero_pad);
    }
    if let Some(unit) = suffix {
        out.push('@');
        out.push_str(&unit);
    }
    Ok(out)
}

fn format_fixed64_with_precision(value: Fixed64, precision: u8) -> String {
    let raw = value.raw_i64();
    let negative = raw < 0;
    let abs_raw = if raw == i64::MIN {
        (i64::MAX as i128) + 1
    } else {
        raw.abs() as i128
    };
    let scale = 10i128.pow(precision as u32);
    let den = 1i128 << 32;
    let num = abs_raw * scale;
    let mut q = num / den;
    let r = num % den;
    let half = den / 2;
    if r > half || (r == half && (q & 1) == 1) {
        q += 1;
    }
    let int_part = q / scale;
    let frac_part = q % scale;
    let mut out = if precision == 0 {
        int_part.to_string()
    } else {
        format!(
            "{}.{:0width$}",
            int_part,
            frac_part,
            width = precision as usize
        )
    };
    if negative {
        out.insert(0, '-');
    }
    out
}

fn apply_width(value: String, width: usize, zero_pad: bool) -> String {
    let len = value.len();
    if len >= width {
        return value;
    }
    let pad_len = width - len;
    if zero_pad {
        if value.starts_with('-') {
            let digits = value[1..].to_string();
            format!("-{}{}", "0".repeat(pad_len), digits)
        } else {
            format!("{}{}", "0".repeat(pad_len), value)
        }
    } else {
        format!("{}{}", " ".repeat(pad_len), value)
    }
}

