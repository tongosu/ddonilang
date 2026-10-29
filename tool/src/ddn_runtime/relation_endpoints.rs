//! Equation endpoints and boundary/range reports; unchanged implementations.
// 패닉 감사 F5(가): values의 직접 인덱스는 각 함수 입구의 정확한 길이(1/2/3)
// 검사 뒤 같은 불변 slice에서만 읽는다. 잘못된 길이는 기존 오류로 먼저 반환한다.
// count field의 expect는 같은 불변 BTreeMap에서 Some(Fixed64)를 확인한 분기에만 있다.
// relations[0]은 relations.len()==1인 match 분기에만 있다.
// ep_ 변수 검색은 i+6<=bytes.len() 안에서 바이트를 읽고, ASCII ep_와 숫자 셋을
// 모두 확인한 뒤 같은 text의 6바이트를 자른다. 따라서 양 끝은 UTF-8 문자 경계다.
// 이 경계 설명은 후속 계산의 산술 넘침·재귀·할당 상한을 보장하지 않는다.
use super::{
    value_to_string,
    BTreeMap,
    BTreeSet,
    BigInt,
    BigRational,
    EvalError,
    Fixed64,
    Formula,
    FormulaDialect,
    Ordering,
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
    Rc,
    RefCell,
    UnitDim,
    UnitValue,
    Value,
    WorkCostMeter,
    analyze_formula_for_transform,
    exact_cmp,
    exact_numeric_from_value,
    fixed_value,
    format_formula_expr,
    make_big_int_pack_from_bigint,
    make_rational_pack_from_big_rational,
    numeric_pack_approx,
    parse_bigint_text,
    parse_formula_value,
    unit_value_to_value,
};

pub(super) fn make_relation_pack(left: Formula, right: Formula) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_KIND_FIELD.to_string(),
        Value::String(RELATION_KIND_EQUATION.to_string()),
    );
    fields.insert(RELATION_LEFT_FIELD.to_string(), Value::Formula(left));
    fields.insert(RELATION_RIGHT_FIELD.to_string(), Value::Formula(right));
    Value::Pack(fields)
}

pub(super) fn eval_endpoint_relation_list(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err("endpoint connect relation 1개가 필요합니다"
            .to_string()
            .into());
    }
    let mut items = Vec::new();
    flatten_endpoint_relation_value(&values[0], &mut items)?;
    Ok(Value::List(items))
}

pub(super) fn eval_endpoint_relation_normalize(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err("endpoint connect relation 1개가 필요합니다"
            .to_string()
            .into());
    }
    let mut items = Vec::new();
    flatten_endpoint_relation_value(&values[0], &mut items)?;
    let count = items.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_relation_flat_set".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(count));
    fields.insert("관계들".to_string(), Value::List(items));
    Ok(Value::Pack(fields))
}

struct EndpointFormulaRelationBridge {
    relations: Vec<Value>,
    mappings: Vec<Value>,
}

pub(super) fn eval_endpoint_formula_relation_list(values: &[Value]) -> Result<Value, EvalError> {
    let bridge = endpoint_formula_relation_bridge(values)?;
    Ok(Value::List(bridge.relations))
}

pub(super) fn eval_endpoint_formula_relation_set(values: &[Value]) -> Result<Value, EvalError> {
    let bridge = endpoint_formula_relation_bridge(values)?;
    let relation_count = bridge.relations.len() as i64;
    let mapping_count = bridge.mappings.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_formula_relation_set".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(relation_count));
    fields.insert("변수개수".to_string(), fixed_value(mapping_count));
    fields.insert("관계들".to_string(), Value::List(bridge.relations));
    fields.insert("변수사상".to_string(), Value::List(bridge.mappings));
    Ok(Value::Pack(fields))
}

struct EndpointBoundaryValueInjection {
    relations: Vec<Value>,
    injected_relations: Vec<Value>,
    injected_values: Vec<Value>,
    mappings: Vec<Value>,
}

pub(super) fn eval_endpoint_boundary_value_relation_list(values: &[Value]) -> Result<Value, EvalError> {
    let injected = endpoint_boundary_value_injection(values)?;
    Ok(Value::List(injected.injected_relations))
}

pub(super) fn eval_endpoint_boundary_value_injection(values: &[Value]) -> Result<Value, EvalError> {
    let injected = endpoint_boundary_value_injection(values)?;
    let relation_count = injected.relations.len() as i64;
    let mapping_count = injected.mappings.len() as i64;
    let injected_count = injected.injected_values.len() as i64;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_formula_relation_set_with_values".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(relation_count));
    fields.insert("변수개수".to_string(), fixed_value(mapping_count));
    fields.insert("주입개수".to_string(), fixed_value(injected_count));
    fields.insert("관계들".to_string(), Value::List(injected.relations));
    fields.insert("변수사상".to_string(), Value::List(injected.mappings));
    fields.insert(
        "주입값들".to_string(),
        Value::List(injected.injected_values),
    );
    Ok(Value::Pack(fields))
}

pub(super) fn eval_endpoint_explicit_solve_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 2 {
        return Err(
            "endpoint connect relation과 boundary value 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let formula_set = eval_endpoint_formula_relation_set(&[values[0].clone()])?;
    let solve_source = match &values[1] {
        Value::List(items) if items.is_empty() => formula_set,
        Value::List(_) => {
            eval_endpoint_boundary_value_injection(&[formula_set, values[1].clone()])?
        }
        _ => eval_endpoint_boundary_value_injection(&[formula_set, values[1].clone()])?,
    };
    let formula_fields = expect_endpoint_formula_relation_mapping_source(&solve_source)?;
    let relation_list = endpoint_relation_list_field(formula_fields, "관계들")?.to_vec();
    let relation_arg = Value::List(relation_list);
    let solve_result = match expect_equation_relations(&[relation_arg])
        .and_then(|relations| eval_relation_solve_result_with_meter(&relations, work_cost))
    {
        Ok(result) => result,
        Err(_) => make_relation_solve_failure("unsupported"),
    };
    eval_endpoint_solve_result_remap(&[solve_source, solve_result])
}

pub(super) fn eval_endpoint_explicit_solve_range_violation_list_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 3 {
        return Err(
            "endpoint connect relation, boundary value 차림, range 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let solve_result = eval_endpoint_explicit_solve_with_meter(
        &[values[0].clone(), values[1].clone()],
        work_cost,
    )?;
    eval_endpoint_boundary_range_violation_list(&[solve_result, values[2].clone()])
}

pub(super) fn eval_endpoint_explicit_solve_range_check_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 3 {
        return Err(
            "endpoint connect relation, boundary value 차림, range 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let solve_result = eval_endpoint_explicit_solve_with_meter(
        &[values[0].clone(), values[1].clone()],
        work_cost,
    )?;
    let range_check =
        eval_endpoint_boundary_range_check(&[solve_result.clone(), values[2].clone()])?;
    let Value::Pack(solve_pack) = &solve_result else {
        return Err("endpoint solve result 묶음이 필요합니다".to_string().into());
    };
    let Value::Pack(range_pack) = &range_check else {
        return Err("endpoint range check 묶음이 필요합니다".to_string().into());
    };
    let solve_kind = endpoint_relation_string_field(solve_pack, "풀이결과종류")?.to_string();
    let check_kind = endpoint_relation_string_field(range_pack, "검사결과")?.to_string();
    let violation_count = range_pack
        .get("위반개수")
        .cloned()
        .ok_or_else(|| EvalError::from("endpoint range check 위반개수가 필요합니다".to_string()))?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_check".to_string()),
    );
    fields.insert("풀이결과".to_string(), solve_result);
    fields.insert("범위검사".to_string(), range_check);
    fields.insert("풀이결과종류".to_string(), Value::String(solve_kind));
    fields.insert("검사결과".to_string(), Value::String(check_kind));
    fields.insert("위반개수".to_string(), violation_count);
    Ok(Value::Pack(fields))
}

struct EndpointSolveRangeReport {
    check: Value,
    rows: Vec<Value>,
    solve_kind: String,
    check_kind: String,
    value_count: usize,
    missing_count: usize,
    range_count: usize,
    violation_count: usize,
}

pub(super) fn eval_endpoint_solve_range_report_rows_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    let report = endpoint_solve_range_report_with_meter(values, work_cost)?;
    Ok(Value::List(report.rows))
}

pub(super) fn eval_endpoint_solve_range_report_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    let report = endpoint_solve_range_report_with_meter(values, work_cost)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_report".to_string()),
    );
    fields.insert("검사".to_string(), report.check);
    fields.insert("풀이결과종류".to_string(), Value::String(report.solve_kind));
    fields.insert("검사결과".to_string(), Value::String(report.check_kind));
    fields.insert("행개수".to_string(), fixed_value(report.rows.len() as i64));
    fields.insert("값개수".to_string(), fixed_value(report.value_count as i64));
    fields.insert(
        "누락개수".to_string(),
        fixed_value(report.missing_count as i64),
    );
    fields.insert(
        "범위개수".to_string(),
        fixed_value(report.range_count as i64),
    );
    fields.insert(
        "위반개수".to_string(),
        fixed_value(report.violation_count as i64),
    );
    fields.insert("행들".to_string(), Value::List(report.rows));
    Ok(Value::Pack(fields))
}

pub(super) fn eval_endpoint_solve_range_text_report(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
        ));
    }
    Ok(Value::String(endpoint_solve_range_text_table(&values[0])?))
}

pub(super) fn eval_endpoint_explicit_solve_range_text_report_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    let report = eval_endpoint_solve_range_report_with_meter(values, work_cost)?;
    Ok(Value::String(endpoint_solve_range_text_table(&report)?))
}

pub(super) fn eval_endpoint_solve_range_case_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    endpoint_solve_range_case_result_with_meter(&values[0], work_cost)
}

pub(super) fn eval_endpoint_solve_range_case_suite_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let Value::List(cases) = &values[0] else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    };
    let mut results = Vec::new();
    let mut passed = 0usize;
    for case in cases {
        let result = endpoint_solve_range_case_result_with_meter(case, work_cost)?;
        let Value::Pack(result_pack) = &result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_malformed_case",
            ));
        };
        let pass = match result_pack.get("통과여부") {
            Some(Value::Bool(pass)) => *pass,
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_malformed_case",
                ))
            }
        };
        if pass {
            passed += 1;
        }
        results.push(result);
    }
    Ok(endpoint_solve_range_case_suite_pack(results, passed))
}

pub(super) fn eval_endpoint_solve_range_case_suite_text(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
        ));
    }
    Ok(Value::String(endpoint_solve_range_case_suite_text(
        &values[0],
    )?))
}

pub(super) fn eval_endpoint_solve_range_case_suite_detail_text(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
        ));
    }
    Ok(Value::String(endpoint_solve_range_case_suite_detail_text(
        &values[0],
    )?))
}

pub(super) fn eval_endpoint_solve_range_case_suite_run_detail_text_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let suite = eval_endpoint_solve_range_case_suite_with_meter(values, work_cost)?;
    Ok(Value::String(endpoint_solve_range_case_suite_detail_text(
        &suite,
    )?))
}

pub(super) fn eval_endpoint_solve_range_case_suite_summary(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
        ));
    }
    endpoint_solve_range_case_suite_summary(&values[0])
}

pub(super) fn eval_endpoint_solve_range_case_suite_run_summary_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let suite = eval_endpoint_solve_range_case_suite_with_meter(values, work_cost)?;
    endpoint_solve_range_case_suite_summary(&suite)
}

pub(super) fn eval_endpoint_solve_range_case_suite_check(values: &[Value]) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
        ));
    }
    endpoint_solve_range_case_suite_check(&values[0])
}

pub(super) fn eval_endpoint_solve_range_case_suite_run_check_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    if values.len() != 1 {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    }
    let summary = eval_endpoint_solve_range_case_suite_run_summary_with_meter(values, work_cost)?;
    endpoint_solve_range_case_suite_check(&summary)
}

fn endpoint_solve_range_case_result_with_meter(
    case: &Value,
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    let Value::Pack(case_pack) = case else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    };
    let name = endpoint_relation_string_field(case_pack, "이름")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_malformed_case"))?
        .to_string();
    let relation = case_pack
        .get("이음관계")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case"))?;
    let values = case_pack
        .get("값들")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case"))?;
    let ranges = case_pack
        .get("범위들")
        .cloned()
        .ok_or_else(|| endpoint_case_suite_error("connect_case_suite_malformed_case"))?;
    let expected = match case_pack.get("기대검사결과") {
        Some(Value::String(value)) if value == "통과" || value == "실패" => value.clone(),
        Some(_) => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_invalid_expected_result",
            ))
        }
        None => "통과".to_string(),
    };
    let report =
        eval_endpoint_solve_range_report_with_meter(&[relation, values, ranges], work_cost)?;
    let Value::Pack(report_pack) = &report else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_malformed_case",
        ));
    };
    let actual = endpoint_relation_string_field(report_pack, "검사결과")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_malformed_case"))?
        .to_string();
    let text = endpoint_solve_range_text_table(&report)?;
    let passed = expected == actual;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_result".to_string()),
    );
    fields.insert("이름".to_string(), Value::String(name));
    fields.insert("기대검사결과".to_string(), Value::String(expected));
    fields.insert("실제검사결과".to_string(), Value::String(actual));
    fields.insert("통과여부".to_string(), Value::Bool(passed));
    fields.insert("보고서".to_string(), report);
    fields.insert("문자표".to_string(), Value::String(text));
    Ok(Value::Pack(fields))
}

pub(super) fn endpoint_solve_range_case_suite_pack(results: Vec<Value>, passed: usize) -> Value {
    let total = results.len();
    let failed = total.saturating_sub(passed);
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_suite".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(total as i64));
    fields.insert("통과개수".to_string(), fixed_value(passed as i64));
    fields.insert("실패개수".to_string(), fixed_value(failed as i64));
    fields.insert("전체통과".to_string(), Value::Bool(failed == 0));
    fields.insert("결과들".to_string(), Value::List(results));
    Value::Pack(fields)
}

fn endpoint_solve_range_case_suite_text(suite: &Value) -> Result<String, EvalError> {
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
        ));
    };
    if endpoint_relation_kind(suite_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_text_expected_suite",
        ));
    }
    let results = endpoint_relation_list_field(suite_pack, "결과들")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
    let mut lines = vec!["이름\t기대\t실제\t통과".to_string()];
    for result in results {
        let Value::Pack(result_pack) = result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_text_expected_suite",
            ));
        };
        if endpoint_relation_kind(result_pack)
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?
            != "endpoint_solve_range_case_result"
        {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_text_expected_suite",
            ));
        }
        let name = endpoint_relation_string_field(result_pack, "이름")
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
        let expected = endpoint_relation_string_field(result_pack, "기대검사결과")
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
        let actual = endpoint_relation_string_field(result_pack, "실제검사결과")
            .map_err(|_| endpoint_case_suite_error("connect_case_suite_text_expected_suite"))?;
        let passed = match result_pack.get("통과여부") {
            Some(Value::Bool(true)) => "참",
            Some(Value::Bool(false)) => "거짓",
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_text_expected_suite",
                ))
            }
        };
        lines.push(format!("{name}\t{expected}\t{actual}\t{passed}"));
    }
    Ok(lines.join("\n"))
}

fn endpoint_solve_range_case_suite_detail_text(suite: &Value) -> Result<String, EvalError> {
    let summary = endpoint_solve_range_case_suite_text(suite)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite"))?;
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
        ));
    };
    if endpoint_relation_kind(suite_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite"))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_expected_suite",
        ));
    }
    let results = endpoint_relation_list_field(suite_pack, "결과들")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_expected_suite"))?;
    let mut sections = vec![summary];
    for result in results {
        sections.push(endpoint_solve_range_case_detail_section(result)?);
    }
    Ok(sections.join("\n\n"))
}

fn endpoint_solve_range_case_detail_section(result: &Value) -> Result<String, EvalError> {
    let Value::Pack(result_pack) = result else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_malformed_case_result",
        ));
    };
    if endpoint_relation_kind(result_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result"))?
        != "endpoint_solve_range_case_result"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_detail_malformed_case_result",
        ));
    }
    let name = endpoint_relation_string_field(result_pack, "이름").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    let expected = endpoint_relation_string_field(result_pack, "기대검사결과").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    let actual = endpoint_relation_string_field(result_pack, "실제검사결과").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    let passed = match result_pack.get("통과여부") {
        Some(Value::Bool(true)) => "참",
        Some(Value::Bool(false)) => "거짓",
        _ => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_detail_malformed_case_result",
            ))
        }
    };
    let text = endpoint_relation_string_field(result_pack, "문자표").map_err(|_| {
        endpoint_case_suite_error("connect_case_suite_detail_malformed_case_result")
    })?;
    Ok(format!(
        "## {name}\n기대\t{expected}\n실제\t{actual}\n통과\t{passed}\n{text}"
    ))
}

pub(super) fn endpoint_solve_range_case_suite_summary(suite: &Value) -> Result<Value, EvalError> {
    let Value::Pack(suite_pack) = suite else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
        ));
    };
    if endpoint_relation_kind(suite_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_summary_expected_suite"))?
        != "endpoint_solve_range_case_suite"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_summary_expected_suite",
        ));
    }
    let results = endpoint_relation_list_field(suite_pack, "결과들")
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_summary_expected_suite"))?;

    let mut pass_names = Vec::new();
    let mut fail_names = Vec::new();
    let mut expected_fail_actual_pass = Vec::new();
    let mut expected_pass_actual_fail = Vec::new();

    for result in results {
        let Value::Pack(result_pack) = result else {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_summary_malformed_case_result",
            ));
        };
        if endpoint_relation_kind(result_pack).map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
        })? != "endpoint_solve_range_case_result"
        {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_summary_malformed_case_result",
            ));
        }
        let name = endpoint_relation_string_field(result_pack, "이름")
            .map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
            })?
            .to_string();
        let expected =
            endpoint_relation_string_field(result_pack, "기대검사결과").map_err(|_| {
                endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
            })?;
        let actual = endpoint_relation_string_field(result_pack, "실제검사결과").map_err(|_| {
            endpoint_case_suite_error("connect_case_suite_summary_malformed_case_result")
        })?;
        let passed = match result_pack.get("통과여부") {
            Some(Value::Bool(pass)) => *pass,
            _ => {
                return Err(endpoint_case_suite_error(
                    "connect_case_suite_summary_malformed_case_result",
                ))
            }
        };
        if passed {
            pass_names.push(name.clone());
        } else {
            fail_names.push(name.clone());
        }
        if expected == "실패" && actual == "통과" {
            expected_fail_actual_pass.push(name.clone());
        }
        if expected == "통과" && actual == "실패" {
            expected_pass_actual_fail.push(name);
        }
    }

    let total = results.len();
    let passed = pass_names.len();
    let failed = fail_names.len();
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_suite_summary".to_string()),
    );
    fields.insert("개수".to_string(), fixed_value(total as i64));
    fields.insert("통과개수".to_string(), fixed_value(passed as i64));
    fields.insert("실패개수".to_string(), fixed_value(failed as i64));
    fields.insert("전체통과".to_string(), Value::Bool(failed == 0));
    fields.insert(
        "통과케이스들".to_string(),
        Value::List(pass_names.into_iter().map(Value::String).collect()),
    );
    fields.insert(
        "실패케이스들".to_string(),
        Value::List(fail_names.into_iter().map(Value::String).collect()),
    );
    fields.insert(
        "기대실패통과케이스들".to_string(),
        Value::List(
            expected_fail_actual_pass
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    fields.insert(
        "기대통과실패케이스들".to_string(),
        Value::List(
            expected_pass_actual_fail
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    Ok(Value::Pack(fields))
}

pub(super) fn endpoint_solve_range_case_suite_check(summary: &Value) -> Result<Value, EvalError> {
    let Value::Pack(summary_pack) = summary else {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
        ));
    };
    if endpoint_relation_kind(summary_pack)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_check_expected_summary"))?
        != "endpoint_solve_range_case_suite_summary"
    {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_expected_summary",
        ));
    }

    let overall = match summary_pack.get("전체통과") {
        Some(Value::Bool(value)) => *value,
        _ => {
            return Err(endpoint_case_suite_error(
                "connect_case_suite_check_malformed_summary",
            ))
        }
    };
    let count = endpoint_case_suite_check_count(summary_pack, "개수")?;
    let passed = endpoint_case_suite_check_count(summary_pack, "통과개수")?;
    let failed = endpoint_case_suite_check_count(summary_pack, "실패개수")?;
    let fail_cases = endpoint_case_suite_check_string_list(summary_pack, "실패케이스들")?;
    let expected_fail_actual_pass =
        endpoint_case_suite_check_string_list(summary_pack, "기대실패통과케이스들")?;
    let expected_pass_actual_fail =
        endpoint_case_suite_check_string_list(summary_pack, "기대통과실패케이스들")?;

    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_range_case_suite_check".to_string()),
    );
    fields.insert(
        "판정".to_string(),
        Value::String(if overall { "통과" } else { "실패" }.to_string()),
    );
    fields.insert("전체통과".to_string(), Value::Bool(overall));
    fields.insert("개수".to_string(), count);
    fields.insert("통과개수".to_string(), passed);
    fields.insert("실패개수".to_string(), failed);
    fields.insert("실패케이스들".to_string(), Value::List(fail_cases));
    fields.insert(
        "기대실패통과케이스들".to_string(),
        Value::List(expected_fail_actual_pass),
    );
    fields.insert(
        "기대통과실패케이스들".to_string(),
        Value::List(expected_pass_actual_fail),
    );
    fields.insert("요약".to_string(), summary.clone());
    Ok(Value::Pack(fields))
}

fn endpoint_case_suite_check_count(
    fields: &BTreeMap<String, Value>,
    name: &str,
) -> Result<Value, EvalError> {
    match fields.get(name) {
        Some(Value::Fixed64(_)) => Ok(fields.get(name).expect("count field").clone()),
        _ => Err(endpoint_case_suite_error(
            "connect_case_suite_check_malformed_summary",
        )),
    }
}

fn endpoint_case_suite_check_string_list(
    fields: &BTreeMap<String, Value>,
    name: &str,
) -> Result<Vec<Value>, EvalError> {
    let list = endpoint_relation_list_field(fields, name)
        .map_err(|_| endpoint_case_suite_error("connect_case_suite_check_malformed_summary"))?;
    if list.iter().any(|item| !matches!(item, Value::String(_))) {
        return Err(endpoint_case_suite_error(
            "connect_case_suite_check_malformed_summary",
        ));
    }
    Ok(list.to_vec())
}

fn endpoint_case_suite_error(marker: &'static str) -> EvalError {
    EvalError::from(marker.to_string())
}

fn endpoint_solve_range_text_table(report: &Value) -> Result<String, EvalError> {
    let Value::Pack(report_pack) = report else {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
        ));
    };
    if endpoint_relation_kind(report_pack).map_err(|_| {
        endpoint_report_text_error("connect_report_text_expected_solve_range_report")
    })? != "endpoint_solve_range_report"
    {
        return Err(endpoint_report_text_error(
            "connect_report_text_expected_solve_range_report",
        ));
    }
    let rows = endpoint_relation_list_field(report_pack, "행들")
        .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
    let mut lines = vec!["변수\t경로\t값상태\t값\t범위상태\t하한\t상한\t위반".to_string()];
    for row in rows {
        let Value::Pack(row_pack) = row else {
            return Err(endpoint_report_text_error(
                "connect_report_text_malformed_row",
            ));
        };
        if endpoint_relation_kind(row_pack)
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?
            != "endpoint_solve_range_report_row"
        {
            return Err(endpoint_report_text_error(
                "connect_report_text_malformed_row",
            ));
        }
        let variable = endpoint_relation_string_field(row_pack, "변수")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let path = endpoint_relation_string_field(row_pack, "경로")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let value_status = endpoint_relation_string_field(row_pack, "값상태")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let range_status = endpoint_relation_string_field(row_pack, "범위상태")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let value = row_pack.get("값").map(value_to_string).unwrap_or_default();
        let lower = row_pack
            .get("하한")
            .map(value_to_string)
            .unwrap_or_default();
        let upper = row_pack
            .get("상한")
            .map(value_to_string)
            .unwrap_or_default();
        let violations = endpoint_relation_list_field(row_pack, "위반들")
            .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?;
        let mut reasons = Vec::new();
        for violation in violations {
            let Value::Pack(violation_pack) = violation else {
                return Err(endpoint_report_text_error(
                    "connect_report_text_malformed_row",
                ));
            };
            reasons.push(
                endpoint_relation_string_field(violation_pack, "이유")
                    .map_err(|_| endpoint_report_text_error("connect_report_text_malformed_row"))?
                    .to_string(),
            );
        }
        lines.push(format!(
            "{variable}\t{path}\t{value_status}\t{value}\t{range_status}\t{lower}\t{upper}\t{}",
            reasons.join("|")
        ));
    }
    Ok(lines.join("\n"))
}

fn endpoint_report_text_error(marker: &'static str) -> EvalError {
    EvalError::from(marker.to_string())
}

fn endpoint_solve_range_report_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<EndpointSolveRangeReport, EvalError> {
    if values.len() != 3 {
        return Err(
            "endpoint connect relation, boundary value 차림, range 차림이 필요합니다"
                .to_string()
                .into(),
        );
    }
    let check = eval_endpoint_explicit_solve_range_check_with_meter(values, work_cost)?;
    let Value::Pack(check_pack) = &check else {
        return Err("endpoint solve range check 묶음이 필요합니다"
            .to_string()
            .into());
    };
    let solve_result = check_pack
        .get("풀이결과")
        .cloned()
        .ok_or_else(|| EvalError::from("endpoint solve result가 필요합니다".to_string()))?;
    let range_check = check_pack
        .get("범위검사")
        .cloned()
        .ok_or_else(|| EvalError::from("endpoint range check가 필요합니다".to_string()))?;
    let solve_pack = expect_endpoint_solve_result_value(&solve_result)?;
    let Value::Pack(range_pack) = &range_check else {
        return Err("endpoint range check 묶음이 필요합니다".to_string().into());
    };
    let solve_kind = endpoint_relation_string_field(solve_pack, "풀이결과종류")?.to_string();
    let check_kind = endpoint_relation_string_field(range_pack, "검사결과")?.to_string();
    let mappings = endpoint_relation_list_field(solve_pack, "변수사상")?;
    let value_by_path = endpoint_solve_result_values_by_path(solve_pack)?;
    let ranges = endpoint_range_bounds(&values[2])?;
    let mut range_by_path: BTreeMap<String, (Option<Value>, Option<Value>)> = BTreeMap::new();
    for range in &ranges {
        range_by_path.insert(range.path.clone(), (range.min.clone(), range.max.clone()));
    }
    let mut violations_by_path: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let violations = endpoint_relation_list_field(range_pack, "위반들")?;
    for violation in violations {
        let Value::Pack(violation_pack) = violation else {
            return Err("endpoint range violation 묶음이 필요합니다"
                .to_string()
                .into());
        };
        let path = endpoint_relation_string_field(violation_pack, "경로")?;
        violations_by_path
            .entry(path.to_string())
            .or_default()
            .push(violation.clone());
    }

    let mut rows = Vec::new();
    let mut value_count = 0usize;
    let mut missing_count = 0usize;
    for mapping in mappings {
        let Value::Pack(mapping_pack) = mapping else {
            return Err("endpoint variable mapping 묶음이 필요합니다"
                .to_string()
                .into());
        };
        let variable = endpoint_relation_string_field(mapping_pack, "변수")?;
        let path = endpoint_relation_string_field(mapping_pack, "경로")?;
        let row_violations = violations_by_path.get(path).cloned().unwrap_or_default();
        let mut fields = BTreeMap::new();
        fields.insert(
            "__이음관계종류".to_string(),
            Value::String("endpoint_solve_range_report_row".to_string()),
        );
        fields.insert("변수".to_string(), Value::String(variable.to_string()));
        fields.insert("경로".to_string(), Value::String(path.to_string()));
        if let Some(value) = value_by_path.get(path) {
            fields.insert("값상태".to_string(), Value::String("값있음".to_string()));
            fields.insert("값".to_string(), value.clone());
            value_count += 1;
        } else {
            fields.insert("값상태".to_string(), Value::String("누락".to_string()));
            missing_count += 1;
        }
        if let Some((min, max)) = range_by_path.get(path) {
            fields.insert(
                "범위상태".to_string(),
                Value::String(if row_violations.is_empty() {
                    "통과".to_string()
                } else {
                    "실패".to_string()
                }),
            );
            if let Some(min) = min {
                fields.insert("하한".to_string(), min.clone());
            }
            if let Some(max) = max {
                fields.insert("상한".to_string(), max.clone());
            }
        } else {
            fields.insert(
                "범위상태".to_string(),
                Value::String("범위없음".to_string()),
            );
        }
        fields.insert(
            "위반개수".to_string(),
            fixed_value(row_violations.len() as i64),
        );
        fields.insert("위반들".to_string(), Value::List(row_violations));
        rows.push(Value::Pack(fields));
    }

    Ok(EndpointSolveRangeReport {
        check,
        rows,
        solve_kind,
        check_kind,
        value_count,
        missing_count,
        range_count: ranges.len(),
        violation_count: violations.len(),
    })
}

#[cfg(test)]
pub(super) fn eval_endpoint_explicit_solve(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_explicit_solve_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_explicit_solve_range_check(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_explicit_solve_range_check_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_solve_range_report(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_solve_range_report_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_solve_range_report_rows(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_solve_range_report_rows_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_explicit_solve_range_text_report(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_explicit_solve_range_text_report_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_solve_range_case(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_solve_range_case_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_solve_range_case_suite(values: &[Value]) -> Result<Value, EvalError> {
    eval_endpoint_solve_range_case_suite_with_meter(values, None)
}

#[cfg(test)]
pub(super) fn eval_endpoint_solve_range_case_suite_run_detail_text(
    values: &[Value],
) -> Result<Value, EvalError> {
    eval_endpoint_solve_range_case_suite_run_detail_text_with_meter(values, None)
}

pub(super) fn eval_endpoint_solve_result_value_list(values: &[Value]) -> Result<Value, EvalError> {
    let remapped = endpoint_solve_result_remap(values)?;
    Ok(Value::List(remapped.values))
}

pub(super) fn eval_endpoint_solve_result_remap(values: &[Value]) -> Result<Value, EvalError> {
    let remapped = endpoint_solve_result_remap(values)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_solve_result".to_string()),
    );
    fields.insert("풀이결과종류".to_string(), Value::String(remapped.kind));
    fields.insert("값들".to_string(), Value::List(remapped.values));
    fields.insert(
        "누락변수들".to_string(),
        Value::List(remapped.missing_variables),
    );
    fields.insert("변수사상".to_string(), Value::List(remapped.mappings));
    fields.insert("원래풀이".to_string(), remapped.original);
    Ok(Value::Pack(fields))
}

struct EndpointSolveResultRemap {
    kind: String,
    values: Vec<Value>,
    missing_variables: Vec<Value>,
    mappings: Vec<Value>,
    original: Value,
}

#[derive(Clone, Copy)]
struct EndpointUnitSeed {
    dim: UnitDim,
}

pub(super) struct EndpointBoundaryNumber {
    value: Value,
    pub(super) formula_text: String,
    unit_dim: Option<UnitDim>,
    unit_symbol: Option<String>,
}

fn endpoint_solve_result_remap(values: &[Value]) -> Result<EndpointSolveResultRemap, EvalError> {
    if values.len() != 2 {
        return Err(
            "endpoint formula relation set과 relation solve result가 필요합니다"
                .to_string()
                .into(),
        );
    }
    let formula_set = expect_endpoint_formula_relation_mapping_source(&values[0])?;
    let mappings_list = endpoint_relation_list_field(formula_set, "변수사상")?;
    let mut mappings = Vec::new();
    let mut ordered_mapping = Vec::new();
    for item in mappings_list {
        let Value::Pack(mapping_pack) = item else {
            return Err("endpoint variable mapping 묶음이 필요합니다"
                .to_string()
                .into());
        };
        let variable = endpoint_relation_string_field(mapping_pack, "변수")?;
        let path = endpoint_relation_string_field(mapping_pack, "경로")?;
        if ordered_mapping
            .iter()
            .any(|(known, _): &(String, String)| known == variable)
        {
            return Err("endpoint variable mapping 변수는 유일해야 합니다"
                .to_string()
                .into());
        }
        ordered_mapping.push((variable.to_string(), path.to_string()));
        mappings.push(item.clone());
    }
    let unit_components = endpoint_unit_components(formula_set, &ordered_mapping)?;

    let solve_result = expect_relation_solve_result(&values[1])?;
    let original = values[1].clone();
    let result_kind = relation_solve_result_kind(solve_result)?;
    if result_kind != RELATION_SOLVE_RESULT_SUCCESS {
        return Ok(EndpointSolveResultRemap {
            kind: "실패".to_string(),
            values: Vec::new(),
            missing_variables: ordered_mapping
                .iter()
                .map(|(variable, _)| Value::String(variable.clone()))
                .collect(),
            mappings,
            original,
        });
    }

    let bindings = match solve_result.get(RELATION_SOLVE_BINDINGS_FIELD) {
        Some(Value::Pack(bindings)) => bindings,
        _ => {
            return Err("relation solve result 해 묶음이 필요합니다"
                .to_string()
                .into())
        }
    };
    for variable in bindings.keys() {
        if !ordered_mapping
            .iter()
            .any(|(mapped_variable, _)| mapped_variable == variable)
        {
            return Err("endpoint_variable_mapping relation solve binding이 endpoint 변수사상에 포함되어야 합니다"
                .to_string()
                .into());
        }
    }

    let mut remapped_values = Vec::new();
    let mut missing_variables = Vec::new();
    for (variable, path) in &ordered_mapping {
        if let Some(value) = bindings.get(variable) {
            let value = match unit_components.get(variable) {
                Some(seed) => {
                    let raw = endpoint_solve_numeric_value(value)?;
                    unit_value_to_value(UnitValue {
                        value: raw,
                        dim: seed.dim,
                    })
                }
                None => value.clone(),
            };
            let mut item = BTreeMap::new();
            item.insert("변수".to_string(), Value::String(variable.clone()));
            item.insert("경로".to_string(), Value::String(path.clone()));
            item.insert("값".to_string(), value);
            remapped_values.push(Value::Pack(item));
        } else {
            missing_variables.push(Value::String(variable.clone()));
        }
    }
    let kind = if missing_variables.is_empty() {
        "성공"
    } else {
        "부분성공"
    };
    Ok(EndpointSolveResultRemap {
        kind: kind.to_string(),
        values: remapped_values,
        missing_variables,
        mappings,
        original,
    })
}

fn expect_endpoint_formula_relation_set(
    value: &Value,
) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into());
    };
    match fields.get("__이음관계종류") {
        Some(Value::String(kind)) if kind == "endpoint_formula_relation_set" => Ok(fields),
        _ => Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into()),
    }
}

fn expect_endpoint_formula_relation_mapping_source(
    value: &Value,
) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into());
    };
    match fields.get("__이음관계종류") {
        Some(Value::String(kind))
            if kind == "endpoint_formula_relation_set"
                || kind == "endpoint_formula_relation_set_with_values" =>
        {
            Ok(fields)
        }
        _ => Err("endpoint_formula_relation_set 묶음이 필요합니다"
            .to_string()
            .into()),
    }
}

fn endpoint_boundary_value_injection(
    values: &[Value],
) -> Result<EndpointBoundaryValueInjection, EvalError> {
    if values.len() != 2 {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_expected_formula_set",
        ));
    }
    let formula_set = expect_endpoint_formula_relation_set(&values[0]).map_err(|_| {
        endpoint_boundary_value_error("connect_boundary_value_expected_formula_set")
    })?;
    let original_relations = endpoint_relation_list_field(formula_set, "관계들")?;
    let mappings_list = endpoint_relation_list_field(formula_set, "변수사상")?;

    let mut path_to_variable = BTreeMap::new();
    let mut mappings = Vec::new();
    for item in mappings_list {
        let Value::Pack(mapping_pack) = item else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_malformed_item",
            ));
        };
        let variable = endpoint_relation_string_field(mapping_pack, "변수")?;
        let path = endpoint_relation_string_field(mapping_pack, "경로")?;
        path_to_variable.insert(path.to_string(), variable.to_string());
        mappings.push(item.clone());
    }

    let Value::List(value_items) = &values[1] else {
        return Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
        ));
    };
    let mut seen_paths = BTreeSet::new();
    let mut injected_relations = Vec::new();
    let mut injected_values = Vec::new();
    for item in value_items {
        let Value::Pack(value_pack) = item else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_malformed_item",
            ));
        };
        let path = endpoint_boundary_value_path(value_pack)?;
        if !seen_paths.insert(path.to_string()) {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_duplicate_path",
            ));
        }
        let Some(variable) = path_to_variable.get(path) else {
            return Err(endpoint_boundary_value_error(
                "connect_boundary_value_unknown_path",
            ));
        };
        let boundary = endpoint_boundary_value_number(value_pack)?;
        injected_relations.push(make_relation_pack(
            endpoint_formula_value(variable),
            endpoint_formula_value(&boundary.formula_text),
        ));
        let mut injected = BTreeMap::new();
        injected.insert("변수".to_string(), Value::String(variable.clone()));
        injected.insert("경로".to_string(), Value::String(path.to_string()));
        injected.insert("값".to_string(), boundary.value);
        if let Some(dim) = boundary.unit_dim {
            injected.insert("단위차원".to_string(), Value::String(dim.format()));
        }
        if let Some(symbol) = boundary.unit_symbol {
            injected.insert("단위기호".to_string(), Value::String(symbol));
        }
        injected_values.push(Value::Pack(injected));
    }

    let mut relations = original_relations.to_vec();
    relations.extend(injected_relations.clone());
    Ok(EndpointBoundaryValueInjection {
        relations,
        injected_relations,
        injected_values,
        mappings,
    })
}

fn endpoint_boundary_value_path(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get("경로") {
        Some(Value::String(path)) => Ok(path.as_str()),
        _ => Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
        )),
    }
}

pub(super) fn endpoint_boundary_value_number(
    fields: &BTreeMap<String, Value>,
) -> Result<EndpointBoundaryNumber, EvalError> {
    match fields.get("값") {
        Some(Value::Fixed64(value)) => Ok(EndpointBoundaryNumber {
            value: Value::Fixed64(*value),
            formula_text: value.canonical_decimal_text(),
            unit_dim: None,
            unit_symbol: None,
        }),
        Some(Value::Unit(unit)) => Ok(EndpointBoundaryNumber {
            value: Value::Unit(*unit),
            formula_text: unit.value.canonical_decimal_text(),
            unit_dim: (!unit.is_dimensionless()).then_some(unit.dim),
            unit_symbol: (!unit.is_dimensionless()).then(|| {
                unit.display_symbol()
                    .map(str::to_string)
                    .unwrap_or_else(|| unit.dim.format())
            }),
        }),
        Some(_) => Err(endpoint_boundary_value_error(
            "connect_boundary_value_non_numeric",
        )),
        None => Err(endpoint_boundary_value_error(
            "connect_boundary_value_malformed_item",
        )),
    }
}

fn endpoint_unit_components(
    formula_set: &BTreeMap<String, Value>,
    ordered_mapping: &[(String, String)],
) -> Result<BTreeMap<String, EndpointUnitSeed>, EvalError> {
    let kind = match formula_set.get("__이음관계종류") {
        Some(Value::String(kind)) => kind.as_str(),
        _ => "",
    };
    if kind != "endpoint_formula_relation_set_with_values" {
        return Ok(BTreeMap::new());
    }

    let known_variables: BTreeSet<String> = ordered_mapping
        .iter()
        .map(|(variable, _)| variable.clone())
        .collect();
    let mut parent: BTreeMap<String, String> = known_variables
        .iter()
        .map(|variable| (variable.clone(), variable.clone()))
        .collect();

    if let Ok(relations) = endpoint_relation_list_field(formula_set, "관계들") {
        for relation in relations {
            let Value::Pack(relation_pack) = relation else {
                continue;
            };
            let mut vars = BTreeSet::new();
            if let Some(Value::Formula(left)) = relation_pack.get(RELATION_LEFT_FIELD) {
                for variable in endpoint_variables_in_formula(&relation_formula_text(left)?) {
                    if known_variables.contains(&variable) {
                        vars.insert(variable);
                    }
                }
            }
            if let Some(Value::Formula(right)) = relation_pack.get(RELATION_RIGHT_FIELD) {
                for variable in endpoint_variables_in_formula(&relation_formula_text(right)?) {
                    if known_variables.contains(&variable) {
                        vars.insert(variable);
                    }
                }
            }
            let mut vars = vars.into_iter();
            if let Some(first) = vars.next() {
                for variable in vars {
                    endpoint_union(&mut parent, &first, &variable);
                }
            }
        }
    }

    let mut component_units: BTreeMap<String, UnitDim> = BTreeMap::new();
    if let Some(Value::List(injected)) = formula_set.get("주입값들") {
        for item in injected {
            let Value::Pack(pack) = item else {
                continue;
            };
            let variable = endpoint_relation_string_field(pack, "변수")?;
            let Some(Value::Unit(unit)) = pack.get("값") else {
                continue;
            };
            if unit.is_dimensionless() {
                continue;
            }
            let root = endpoint_find(&mut parent, variable);
            if let Some(existing) = component_units.get(&root).copied() {
                if existing != unit.dim {
                    let marker = if endpoint_currency_pair(existing, unit.dim) {
                        "connect_unit_boundary_incompatible_unit"
                    } else {
                        "connect_unit_boundary_dim_conflict"
                    };
                    return Err(endpoint_boundary_value_error(marker));
                }
            } else {
                component_units.insert(root, unit.dim);
            }
        }
    }

    let mut by_variable = BTreeMap::new();
    for variable in known_variables {
        let root = endpoint_find(&mut parent, &variable);
        if let Some(dim) = component_units.get(&root).copied() {
            by_variable.insert(variable, EndpointUnitSeed { dim });
        }
    }
    Ok(by_variable)
}

fn endpoint_variables_in_formula(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 6 <= bytes.len() {
        if &bytes[i..i + 3] == b"ep_"
            && bytes[i + 3].is_ascii_digit()
            && bytes[i + 4].is_ascii_digit()
            && bytes[i + 5].is_ascii_digit()
        {
            out.push(text[i..i + 6].to_string());
            i += 6;
        } else {
            i += 1;
        }
    }
    out
}

fn endpoint_find(parent: &mut BTreeMap<String, String>, variable: &str) -> String {
    let current = parent
        .get(variable)
        .cloned()
        .unwrap_or_else(|| variable.to_string());
    if current == variable {
        return current;
    }
    let root = endpoint_find(parent, &current);
    parent.insert(variable.to_string(), root.clone());
    root
}

fn endpoint_union(parent: &mut BTreeMap<String, String>, left: &str, right: &str) {
    let left_root = endpoint_find(parent, left);
    let right_root = endpoint_find(parent, right);
    if left_root != right_root {
        parent.insert(right_root, left_root);
    }
}

fn endpoint_currency_pair(left: UnitDim, right: UnitDim) -> bool {
    (left == UnitDim::KRW && right == UnitDim::USD)
        || (left == UnitDim::USD && right == UnitDim::KRW)
}

fn endpoint_solve_numeric_value(value: &Value) -> Result<Fixed64, EvalError> {
    match value {
        Value::Fixed64(raw) => Ok(*raw),
        Value::Unit(unit) if unit.is_dimensionless() => Ok(unit.value),
        _ => numeric_pack_approx(value)
            .ok_or_else(|| endpoint_boundary_value_error("connect_boundary_value_non_numeric")),
    }
}

fn endpoint_boundary_value_error(marker: &'static str) -> EvalError {
    marker.to_string().into()
}

struct EndpointRangeBound {
    path: String,
    min: Option<Value>,
    max: Option<Value>,
    min_inclusive: bool,
    max_inclusive: bool,
}

struct EndpointBoundaryRangeCheck {
    violations: Vec<Value>,
    range_count: usize,
}

pub(super) fn eval_endpoint_boundary_range_violation_list(values: &[Value]) -> Result<Value, EvalError> {
    let checked = endpoint_boundary_range_check(values)?;
    Ok(Value::List(checked.violations))
}

pub(super) fn eval_endpoint_boundary_range_check(values: &[Value]) -> Result<Value, EvalError> {
    let checked = endpoint_boundary_range_check(values)?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "__이음관계종류".to_string(),
        Value::String("endpoint_range_check".to_string()),
    );
    fields.insert(
        "검사결과".to_string(),
        Value::String(if checked.violations.is_empty() {
            "통과".to_string()
        } else {
            "실패".to_string()
        }),
    );
    fields.insert(
        "범위개수".to_string(),
        fixed_value(checked.range_count as i64),
    );
    fields.insert(
        "위반개수".to_string(),
        fixed_value(checked.violations.len() as i64),
    );
    fields.insert("위반들".to_string(), Value::List(checked.violations));
    Ok(Value::Pack(fields))
}

fn endpoint_boundary_range_check(
    values: &[Value],
) -> Result<EndpointBoundaryRangeCheck, EvalError> {
    if values.len() != 2 {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
        ));
    }
    let solve_result = expect_endpoint_solve_result_value(&values[0])?;
    let known_paths = endpoint_solve_result_mapping_paths(solve_result)?;
    let value_by_path = endpoint_solve_result_values_by_path(solve_result)?;
    let ranges = endpoint_range_bounds(&values[1])?;

    let mut violations = Vec::new();
    for range in &ranges {
        if !known_paths.contains(&range.path) {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_unknown_path",
            ));
        }
        let Some(value) = value_by_path.get(&range.path) else {
            let mut fields = endpoint_range_violation_base(&range.path, "missing_value");
            if let Some(min) = &range.min {
                fields.insert("하한".to_string(), min.clone());
            }
            if let Some(max) = &range.max {
                fields.insert("상한".to_string(), max.clone());
            }
            violations.push(Value::Pack(fields));
            continue;
        };
        if let Some(min) = &range.min {
            let ord = endpoint_compare_range_numbers(value, min)?;
            let below = if range.min_inclusive {
                ord == std::cmp::Ordering::Less
            } else {
                ord != std::cmp::Ordering::Greater
            };
            if below {
                let mut fields = endpoint_range_violation_base(&range.path, "below_min");
                fields.insert("값".to_string(), value.clone());
                fields.insert("하한".to_string(), min.clone());
                if let Some(max) = &range.max {
                    fields.insert("상한".to_string(), max.clone());
                }
                violations.push(Value::Pack(fields));
                continue;
            }
        }
        if let Some(max) = &range.max {
            let ord = endpoint_compare_range_numbers(value, max)?;
            let above = if range.max_inclusive {
                ord == std::cmp::Ordering::Greater
            } else {
                ord != std::cmp::Ordering::Less
            };
            if above {
                let mut fields = endpoint_range_violation_base(&range.path, "above_max");
                fields.insert("값".to_string(), value.clone());
                if let Some(min) = &range.min {
                    fields.insert("하한".to_string(), min.clone());
                }
                fields.insert("상한".to_string(), max.clone());
                violations.push(Value::Pack(fields));
            }
        }
    }
    Ok(EndpointBoundaryRangeCheck {
        violations,
        range_count: ranges.len(),
    })
}

fn expect_endpoint_solve_result_value(
    value: &Value,
) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
        ));
    };
    match fields.get("__이음관계종류") {
        Some(Value::String(kind)) if kind == "endpoint_solve_result" => Ok(fields),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_expected_solve_result",
        )),
    }
}

fn endpoint_solve_result_mapping_paths(
    solve_result: &BTreeMap<String, Value>,
) -> Result<BTreeSet<String>, EvalError> {
    let mappings = endpoint_relation_list_field(solve_result, "변수사상")?;
    let mut out = BTreeSet::new();
    for item in mappings {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
            ));
        };
        out.insert(endpoint_relation_string_field(pack, "경로")?.to_string());
    }
    Ok(out)
}

fn endpoint_solve_result_values_by_path(
    solve_result: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, EvalError> {
    let values = endpoint_relation_list_field(solve_result, "값들")?;
    let mut out = BTreeMap::new();
    for item in values {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
            ));
        };
        let path = endpoint_relation_string_field(pack, "경로")?;
        let Some(value) = pack.get("값") else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_expected_solve_result",
            ));
        };
        out.insert(path.to_string(), value.clone());
    }
    Ok(out)
}

fn endpoint_range_bounds(value: &Value) -> Result<Vec<EndpointRangeBound>, EvalError> {
    let Value::List(items) = value else {
        return Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
        ));
    };
    let mut seen_paths = BTreeSet::new();
    let mut ranges = Vec::new();
    for item in items {
        let Value::Pack(pack) = item else {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_malformed_item",
            ));
        };
        let path = endpoint_range_path(pack)?.to_string();
        if !seen_paths.insert(path.clone()) {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_duplicate_path",
            ));
        }
        let min = endpoint_optional_range_number(pack, "최소")?;
        let max = endpoint_optional_range_number(pack, "최대")?;
        if min.is_none() && max.is_none() {
            return Err(endpoint_boundary_range_error(
                "connect_boundary_range_malformed_item",
            ));
        }
        let min_inclusive = endpoint_optional_bool(pack, "최소포함", true)?;
        let max_inclusive = endpoint_optional_bool(pack, "최대포함", true)?;
        ranges.push(EndpointRangeBound {
            path,
            min,
            max,
            min_inclusive,
            max_inclusive,
        });
    }
    Ok(ranges)
}

fn endpoint_range_path(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get("경로") {
        Some(Value::String(path)) => Ok(path.as_str()),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
        )),
    }
}

fn endpoint_optional_range_number(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<Option<Value>, EvalError> {
    match fields.get(field) {
        Some(Value::Fixed64(_)) | Some(Value::Unit(_)) => Ok(fields.get(field).cloned()),
        Some(_) => Err(endpoint_boundary_range_error(
            "connect_boundary_range_non_numeric",
        )),
        None => Ok(None),
    }
}

fn endpoint_optional_bool(
    fields: &BTreeMap<String, Value>,
    field: &str,
    default: bool,
) -> Result<bool, EvalError> {
    match fields.get(field) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(endpoint_boundary_range_error(
            "connect_boundary_range_malformed_item",
        )),
        None => Ok(default),
    }
}

fn endpoint_compare_range_numbers(
    left: &Value,
    right: &Value,
) -> Result<std::cmp::Ordering, EvalError> {
    let left_exact = exact_numeric_from_value(left)?;
    let right_exact = exact_numeric_from_value(right)?;
    if let (Some(left), Some(right)) = (left_exact, right_exact) {
        return Ok(exact_cmp(&left, &right));
    }
    let left = endpoint_range_numeric_unit(left)?;
    let right = endpoint_range_numeric_unit(right)?;
    if left.dim != right.dim {
        let marker = if endpoint_currency_pair(left.dim, right.dim) {
            "connect_boundary_range_incompatible_unit"
        } else {
            "connect_boundary_range_dim_conflict"
        };
        return Err(endpoint_boundary_range_error(marker));
    }
    Ok(left.value.cmp(&right.value))
}

fn endpoint_range_numeric_unit(value: &Value) -> Result<UnitValue, EvalError> {
    match value {
        Value::Fixed64(raw) => Ok(UnitValue {
            value: *raw,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Ok(*unit),
        _ => Err(endpoint_boundary_range_error(
            "connect_boundary_range_non_numeric",
        )),
    }
}

fn endpoint_range_violation_base(path: &str, reason: &str) -> BTreeMap<String, Value> {
    let mut fields = BTreeMap::new();
    fields.insert("경로".to_string(), Value::String(path.to_string()));
    fields.insert("이유".to_string(), Value::String(reason.to_string()));
    fields
}

fn endpoint_boundary_range_error(marker: &'static str) -> EvalError {
    marker.to_string().into()
}

fn expect_relation_solve_result(value: &Value) -> Result<&BTreeMap<String, Value>, EvalError> {
    let Value::Pack(fields) = value else {
        return Err("relation solve result 묶음이 필요합니다".to_string().into());
    };
    relation_solve_result_kind(fields)?;
    Ok(fields)
}

fn relation_solve_result_kind(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get(RELATION_SOLVE_RESULT_KIND_FIELD) {
        Some(Value::String(kind)) => Ok(kind.as_str()),
        _ => Err("relation solve result kind가 필요합니다".to_string().into()),
    }
}

fn endpoint_formula_relation_bridge(
    values: &[Value],
) -> Result<EndpointFormulaRelationBridge, EvalError> {
    if values.len() != 1 {
        return Err("endpoint connect relation 1개가 필요합니다"
            .to_string()
            .into());
    }
    let mut flat_relations = Vec::new();
    flatten_endpoint_relation_value(&values[0], &mut flat_relations)?;

    let mut variables: BTreeMap<String, String> = BTreeMap::new();
    let mut mappings = Vec::new();
    let mut relation_packs = Vec::new();
    for relation in flat_relations {
        let Value::Pack(fields) = &relation else {
            return Err("endpoint connect relation 묶음이 필요합니다"
                .to_string()
                .into());
        };
        match endpoint_relation_kind(&fields)? {
            "endpoint_equality" => {
                let left = endpoint_formula_variable_for_field(
                    &fields,
                    "왼쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                let right = endpoint_formula_variable_for_field(
                    &fields,
                    "오른쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                relation_packs.push(make_relation_pack(
                    endpoint_formula_value(&left),
                    endpoint_formula_value(&right),
                ));
            }
            "endpoint_flow" => {
                let convention = endpoint_relation_string_field(&fields, "부호규약")?;
                if convention != "left_plus_right_zero" {
                    return Err(
                        "endpoint flow는 left_plus_right_zero 부호규약만 방정식화할 수 있습니다"
                            .to_string()
                            .into(),
                    );
                }
                let left = endpoint_formula_variable_for_field(
                    &fields,
                    "왼쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                let right = endpoint_formula_variable_for_field(
                    &fields,
                    "오른쪽",
                    &mut variables,
                    &mut mappings,
                )?;
                relation_packs.push(make_relation_pack(
                    endpoint_formula_value(&format!("{left} + {right}")),
                    endpoint_formula_value("0"),
                ));
            }
            "endpoint_carried_property" => {
                return Err("endpoint_carried_property는 방정식화 대상이 아닙니다"
                    .to_string()
                    .into());
            }
            _ => {
                return Err("지원하지 않는 endpoint relation은 방정식화할 수 없습니다"
                    .to_string()
                    .into());
            }
        }
    }

    Ok(EndpointFormulaRelationBridge {
        relations: relation_packs,
        mappings,
    })
}

fn flatten_endpoint_relation_value(value: &Value, out: &mut Vec<Value>) -> Result<(), EvalError> {
    let Value::Pack(fields) = value else {
        return Err("endpoint connect relation 묶음이 필요합니다"
            .to_string()
            .into());
    };
    match endpoint_relation_kind(fields)? {
        "endpoint_equality" | "endpoint_flow" | "endpoint_carried_property" => {
            out.push(value.clone());
            Ok(())
        }
        "endpoint_relation_set" => {
            let relations = endpoint_relation_list_field(fields, "관계들")?;
            for item in relations {
                flatten_endpoint_relation_value(item, out)?;
            }
            Ok(())
        }
        "endpoint_relation_flat_set" => {
            let relations = endpoint_relation_list_field(fields, "관계들")?;
            for item in relations {
                flatten_endpoint_relation_value(item, out)?;
            }
            Ok(())
        }
        "endpoint_statement_set" => {
            let statements = endpoint_relation_list_field(fields, "이음들")?;
            for item in statements {
                flatten_endpoint_relation_value(item, out)?;
            }
            Ok(())
        }
        _ => Err("지원하지 않는 endpoint connect relation입니다"
            .to_string()
            .into()),
    }
}

fn endpoint_formula_variable_for_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
    variables: &mut BTreeMap<String, String>,
    mappings: &mut Vec<Value>,
) -> Result<String, EvalError> {
    let path = endpoint_relation_string_field(fields, field)?;
    if let Some(variable) = variables.get(path) {
        return Ok(variable.clone());
    }
    let variable = format!("ep_{:03}", variables.len() + 1);
    variables.insert(path.to_string(), variable.clone());
    let mut mapping = BTreeMap::new();
    mapping.insert("변수".to_string(), Value::String(variable.clone()));
    mapping.insert("경로".to_string(), Value::String(path.to_string()));
    mappings.push(Value::Pack(mapping));
    Ok(variable)
}

fn endpoint_relation_string_field<'a>(
    fields: &'a BTreeMap<String, Value>,
    field: &str,
) -> Result<&'a str, EvalError> {
    match fields.get(field) {
        Some(Value::String(value)) => Ok(value.as_str()),
        _ => Err(format!("endpoint connect relation field '{field}' 글이 필요합니다").into()),
    }
}

fn endpoint_formula_value(body: &str) -> Formula {
    Formula {
        raw: body.to_string(),
        dialect: FormulaDialect::Ascii,
        explicit_tag: true,
    }
}

fn endpoint_relation_kind(fields: &BTreeMap<String, Value>) -> Result<&str, EvalError> {
    match fields.get("__이음관계종류") {
        Some(Value::String(kind)) => Ok(kind.as_str()),
        _ => Err("endpoint connect relation kind가 필요합니다"
            .to_string()
            .into()),
    }
}

fn endpoint_relation_list_field<'a>(
    fields: &'a BTreeMap<String, Value>,
    field: &str,
) -> Result<&'a [Value], EvalError> {
    match fields.get(field) {
        Some(Value::List(items)) => Ok(items),
        _ => Err(format!("endpoint connect relation field '{field}' 차림이 필요합니다").into()),
    }
}

fn make_relation_solve_success_bindings(bindings: BTreeMap<String, Value>) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
        Value::String(RELATION_SOLVE_RESULT_SUCCESS.to_string()),
    );
    if bindings.len() == 1 {
        if let Some((variable, value)) = bindings.iter().next() {
            fields.insert(
                RELATION_SOLVE_VAR_FIELD.to_string(),
                Value::String(variable.clone()),
            );
            fields.insert(RELATION_SOLVE_VALUE_FIELD.to_string(), value.clone());
        }
    }
    fields.insert(
        RELATION_SOLVE_BINDINGS_FIELD.to_string(),
        Value::Pack(bindings),
    );
    Value::Pack(fields)
}

fn make_relation_solve_failure(reason: &str) -> Value {
    let mut fields = BTreeMap::new();
    fields.insert(
        RELATION_SOLVE_RESULT_KIND_FIELD.to_string(),
        Value::String(RELATION_SOLVE_RESULT_FAILURE.to_string()),
    );
    fields.insert(
        RELATION_SOLVE_REASON_FIELD.to_string(),
        Value::String(reason.to_string()),
    );
    Value::Pack(fields)
}

pub(super) fn expect_single_equation_relation(value: &Value) -> Result<BTreeMap<String, Value>, EvalError> {
    let Value::Pack(pack) = value else {
        return Err("방정식 관계가 필요합니다".to_string().into());
    };
    match pack.get(RELATION_KIND_FIELD) {
        Some(Value::String(kind)) if kind == RELATION_KIND_EQUATION => {}
        _ => return Err("방정식 관계가 필요합니다".to_string().into()),
    }
    match (
        pack.get(RELATION_LEFT_FIELD),
        pack.get(RELATION_RIGHT_FIELD),
    ) {
        (Some(Value::Formula(_)), Some(Value::Formula(_))) => Ok(pack.clone()),
        _ => Err("방정식 관계가 필요합니다".to_string().into()),
    }
}

pub(super) fn expect_equation_relations(values: &[Value]) -> Result<Vec<BTreeMap<String, Value>>, EvalError> {
    if values.len() != 1 {
        return Err("방정식 관계 1개가 필요합니다".to_string().into());
    }
    match &values[0] {
        Value::Pack(_) => Ok(vec![expect_single_equation_relation(&values[0])?]),
        Value::List(items) => {
            if items.len() != 2 {
                return Err("방정식 관계 차림은 2개여야 합니다".to_string().into());
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(expect_single_equation_relation(item)?);
            }
            Ok(out)
        }
        _ => Err("방정식 관계가 필요합니다".to_string().into()),
    }
}

pub(super) fn relation_formula_text(formula: &Formula) -> Result<String, EvalError> {
    if !matches!(
        formula.dialect,
        FormulaDialect::Ascii | FormulaDialect::Ascii1
    ) {
        return Err(EvalError::Message(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE #ascii 수식만 지원합니다".to_string(),
        ));
    }
    let analysis = analyze_formula_for_transform(formula)?;
    Ok(match analysis.assign_name {
        Some(assign_name) => format!(
            "({assign_name}) - ({})",
            format_formula_expr(&analysis.expr, 0)
        ),
        None => format_formula_expr(&analysis.expr, 0),
    })
}

fn relation_binding_value_from_text(
    binding: &ddonirang_symbolic::SolveBinding,
) -> Result<Value, EvalError> {
    let num = parse_bigint_text(&binding.numerator, "relation solve numerator")?;
    let den = parse_bigint_text(&binding.denominator, "relation solve denominator")?;
    Ok(if den == BigInt::from(1u8) {
        make_big_int_pack_from_bigint(&num)
    } else {
        make_rational_pack_from_big_rational(&BigRational::new(num, den))
    })
}

pub(super) fn eval_relation_solve_result_with_meter(
    relations: &[BTreeMap<String, Value>],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    let outcome = match relations.len() {
        1 => {
            let left = match relations[0].get(RELATION_LEFT_FIELD) {
                Some(Value::Formula(formula)) => formula,
                _ => return Err("방정식 관계가 필요합니다".to_string().into()),
            };
            let right = match relations[0].get(RELATION_RIGHT_FIELD) {
                Some(Value::Formula(formula)) => formula,
                _ => return Err("방정식 관계가 필요합니다".to_string().into()),
            };
            let left_text = relation_formula_text(left)?;
            let right_text = relation_formula_text(right)?;
            if let Some(work_cost) = work_cost {
                work_cost.borrow_mut().charge_solver_evaluations(1)?;
            }
            ddonirang_symbolic::solve_relation_equation(&left_text, &right_text)
        }
        2 => {
            let pairs = relations
                .iter()
                .map(|relation| {
                    let left = match relation.get(RELATION_LEFT_FIELD) {
                        Some(Value::Formula(formula)) => formula,
                        _ => return Err("방정식 관계가 필요합니다".to_string().into()),
                    };
                    let right = match relation.get(RELATION_RIGHT_FIELD) {
                        Some(Value::Formula(formula)) => formula,
                        _ => return Err("방정식 관계가 필요합니다".to_string().into()),
                    };
                    Ok((relation_formula_text(left)?, relation_formula_text(right)?))
                })
                .collect::<Result<Vec<_>, EvalError>>()?;
            if let Some(work_cost) = work_cost {
                work_cost.borrow_mut().charge_solver_evaluations(1)?;
            }
            ddonirang_symbolic::solve_relation_system(&pairs)
        }
        _ => Err("E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE relation solve arity".to_string()),
    };
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(err) if err.starts_with("E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE") => {
            return Ok(make_relation_solve_failure("unsupported"));
        }
        Err(err) => return Err(EvalError::Message(err)),
    };

    Ok(match outcome {
        ddonirang_symbolic::RelationSolveOutcome::Solution(solution) => {
            let mut bindings = BTreeMap::new();
            let mut exact_bindings = BTreeMap::new();
            for (variable, binding) in solution {
                if relations.len() == 1 {
                    let numerator = parse_bigint_text(&binding.numerator, "relation solve numerator")?;
                    let denominator = parse_bigint_text(&binding.denominator, "relation solve denominator")?;
                    if denominator == BigInt::from(0u8) {
                        return Err("방정식 해의 분모는 0일 수 없습니다".to_string().into());
                    }
                    exact_bindings.insert(variable.clone(), make_rational_pack_from_big_rational(
                        &BigRational::new(numerator, denominator)));
                }
                bindings.insert(variable, relation_binding_value_from_text(&binding)?);
            }
            let mut result = make_relation_solve_success_bindings(bindings);
            if relations.len() == 1 {
                let Value::Pack(fields) = &mut result else { unreachable!() };
                for (source_field, result_field) in [(RELATION_LEFT_FIELD, "왼쪽입력"), (RELATION_RIGHT_FIELD, "오른쪽입력")] {
                    let Some(Value::Formula(formula)) = relations[0].get(source_field) else { unreachable!() };
                    // Use precisely the parser and required-key set used by 풀기;
                    // do not discard a variable merely because it cancels.
                    let parsed = parse_formula_value(formula)?;
                    let mut injection = BTreeMap::new();
                    for name in parsed.vars {
                        let value = exact_bindings.get(&name).ok_or_else(||
                            EvalError::from(format!("검산 입력에 필요한 해가 없습니다: {name}")))?;
                        injection.insert(name, value.clone());
                    }
                    fields.insert(result_field.into(), Value::Pack(injection));
                }
            }
            result
        }
        ddonirang_symbolic::RelationSolveOutcome::NoSolution => {
            make_relation_solve_failure("no_solution")
        }
        ddonirang_symbolic::RelationSolveOutcome::NonUnique => {
            make_relation_solve_failure("non_unique")
        }
    })
}

#[cfg(test)]
pub(super) fn eval_relation_solve_result(relations: &[BTreeMap<String, Value>]) -> Result<Value, EvalError> {
    eval_relation_solve_result_with_meter(relations, None)
}

