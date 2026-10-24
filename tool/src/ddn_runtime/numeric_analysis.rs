//! Unit suffixes and numerical analysis; unchanged implementations.
use super::*;
// F5-GA[NUMERIC_ARGS] Each numeric argument reader rejects its wrong exact arity before indexing this immutable slice: List1, quantile2/3 (index2 only under len3), derivative4, integral/bisection5, polynomial/inequality2. No helper mutates the slice between guard and access.

pub(super) fn apply_suffix_value(value: Value, suffix: &AtSuffix) -> Result<Value, EvalError> {
    match suffix {
        AtSuffix::Unit(unit) => apply_unit_suffix(value, unit),
        AtSuffix::Asset(path) => {
            let handle = gate0_registry::resolve_asset_handle(path)?;
            Ok(Value::ResourceHandle(handle))
        }
    }
}

pub(super) fn apply_unit_suffix(value: Value, unit: &str) -> Result<Value, EvalError> {
    let Some(spec) = unit_spec_from_symbol(unit) else {
        return Err(format!("단위 '{}'를 알 수 없습니다", unit).into());
    };
    match value {
        Value::Fixed64(v) => Ok(Value::Unit(UnitValue::try_from_spec(v, spec).map_err(unit_error)?)),
        Value::Unit(existing) => {
            if existing.dim != spec.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: existing.dim,
                    right: spec.dim,
                }));
            }
            Ok(Value::Unit(existing))
        }
        _ => Err("단위는 수치에만 붙일 수 있습니다".to_string().into()),
    }
}

pub(super) fn parse_resource_unit_tag(tag: &str) -> Option<ddonirang_core::UnitSpec> {
    let (name, unit) = tag.rsplit_once('@')?;
    if name.is_empty() {
        return None;
    }
    unit_spec_from_symbol(unit)
}

pub(super) fn unit_value_from_value(value: &Value) -> Result<UnitValue, EvalError> {
    match value {
        Value::Fixed64(v) => Ok(UnitValue {
            value: *v,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Ok(*unit),
        Value::Pack(_) => {
            if let Some(value) = exact_dataset_scalar_unit_value(value) {
                return Ok(value);
            }
            let Some(v) = numeric_pack_approx(value) else {
                return Err("수치가 필요합니다".to_string().into());
            };
            Ok(UnitValue {
                value: v,
                dim: UnitDim::NONE,
            })
        }
        _ => Err("수치가 필요합니다".to_string().into()),
    }
}

pub(super) fn numeric_value(value: &Value) -> Option<UnitValue> {
    match value {
        Value::Fixed64(v) => Some(UnitValue {
            value: *v,
            dim: UnitDim::NONE,
        }),
        Value::Unit(unit) => Some(*unit),
        Value::Pack(_) => exact_dataset_scalar_unit_value(value).or_else(|| {
            numeric_pack_approx(value).map(|v| UnitValue {
                value: v,
                dim: UnitDim::NONE,
            })
        }),
        _ => None,
    }
}

pub(super) fn exact_dataset_scalar_unit_value(value: &Value) -> Option<UnitValue> {
    let Value::Pack(fields) = value else {
        return None;
    };
    if fields.len() != 7 {
        return None;
    }
    let kind = match fields.get("kind")? {
        Value::String(value) => value.as_str(),
        _ => return None,
    };
    if !matches!(kind, "integer" | "fixed64") {
        return None;
    }
    let raw = match fields.get("value")? {
        Value::String(value) => value,
        _ => return None,
    };
    let parsed = Fixed64::parse_decimal_exact(raw)?;
    let dim = match fields.get("unit")? {
        Value::None => UnitDim::NONE,
        Value::String(unit) if unit == "mol" => UnitDim::AMOUNT_OF_SUBSTANCE,
        _ => return None,
    };
    match fields.get("name")? {
        Value::String(name) if !name.is_empty() => {}
        _ => return None,
    }
    match fields.get("source_locator")? {
        Value::String(locator) if !locator.is_empty() => {}
        _ => return None,
    }
    match fields.get("content_sha256")? {
        Value::String(hash)
            if hash.len() == 64
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) => {}
        _ => return None,
    }
    match fields.get("row_source_ordinal")? {
        Value::String(value) if value.parse::<u64>().is_ok() => {}
        _ => return None,
    }
    Some(UnitValue { value: parsed, dim })
}

pub(super) fn unit_value_to_value(value: UnitValue) -> Value {
    if value.is_dimensionless() {
        Value::Fixed64(value.value)
    } else {
        Value::Unit(value)
    }
}

pub(super) fn unit_error(err: UnitError) -> EvalError {
    EvalError::from(err)
}

pub(super) fn fixed64_floor(value: Fixed64) -> Fixed64 {
    Fixed64::from_i64(value.int_part())
}

pub(super) fn fixed64_to_f64(value: Fixed64) -> f64 {
    value.raw_i64() as f64 / Fixed64::SCALE_F64
}

pub(super) fn fixed64_from_host_f64_checked(value: f64) -> Result<Fixed64, ArithmeticFaultKind> {
    let raw = value * Fixed64::SCALE_F64;
    let rounded = raw.round_ties_even();
    if !rounded.is_finite() || rounded < i64::MIN as f64 || rounded >= i64::MAX as f64 {
        return Err(ArithmeticFaultKind::Overflow);
    }
    Ok(Fixed64::from_raw_i64(rounded as i64))
}

pub(super) fn fixed64_ceil(value: Fixed64) -> Result<Fixed64, ArithmeticFaultKind> {
    if value.frac_part() == 0 {
        Ok(Fixed64::from_raw_i64(
            value.raw_i64() & !((1_i64 << Fixed64::FRAC_BITS) - 1),
        ))
    } else {
        Fixed64::from_raw_i64(value.raw_i64() & !((1_i64 << Fixed64::FRAC_BITS) - 1))
            .checked_add(Fixed64::ONE)
    }
}

pub(super) fn fixed64_round_even(value: Fixed64) -> Result<Fixed64, ArithmeticFaultKind> {
    let raw = value.raw_i64() as i128;
    if raw == 0 {
        return Ok(Fixed64::ZERO);
    }
    let sign = if raw < 0 { -1 } else { 1 };
    let abs = raw.abs();
    let frac_mask = (1_i128 << Fixed64::FRAC_BITS) - 1;
    let int_part = abs >> Fixed64::FRAC_BITS;
    let frac = abs & frac_mask;
    let half = 1_i128 << (Fixed64::FRAC_BITS - 1);
    let mut rounded = int_part;
    if frac > half || (frac == half && (int_part & 1) != 0) {
        rounded += 1;
    }
    let raw = (rounded * sign) << Fixed64::FRAC_BITS;
    i64::try_from(raw)
        .map(Fixed64::from_raw_i64)
        .map_err(|_| ArithmeticFaultKind::Overflow)
}

pub(super) fn fixed64_abs(value: Fixed64) -> Result<Fixed64, ArithmeticFaultKind> {
    let raw = value.raw_i64();
    if raw == i64::MIN {
        return Err(ArithmeticFaultKind::Overflow);
    }
    Ok(Fixed64::from_raw_i64(raw.abs()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PercentileMode {
    Linear,
    NearestRank,
}

pub(super) fn expect_numeric_list_arg(
    args: &[Value],
    func_name: &str,
    work_cost: &RefCell<WorkCostMeter>,
) -> Result<Vec<UnitValue>, EvalError> {
    if args.len() != 1 {
        return Err(format!("{func_name}는 인자 1개를 받습니다")
            .to_string()
            .into());
    }
    let Value::List(items) = &args[0] else {
        return Err(format!("{func_name}는 차림 인자를 받습니다")
            .to_string()
            .into());
    };
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        work_cost.borrow_mut().charge_element_visits(1)?;
        values.push(unit_value_from_value(item)?);
    }
    if let Some(head) = values.first() {
        for item in values.iter().skip(1) {
            if item.dim != head.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: head.dim,
                    right: item.dim,
                }));
            }
        }
    }
    Ok(values)
}

pub(super) fn expect_quantile_args(
    args: &[Value],
    work_cost: &RefCell<WorkCostMeter>,
) -> Result<(Vec<UnitValue>, Fixed64, PercentileMode), EvalError> {
    if args.len() != 2 && args.len() != 3 {
        return Err("분위수는 인자 2개 또는 3개를 받습니다".to_string().into());
    }
    let values = expect_numeric_list_arg(&args[..1], "분위수", work_cost)?;
    let percentile = unit_value_from_value(&args[1])?;
    if !percentile.is_dimensionless() {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: percentile.dim,
            right: UnitDim::NONE,
        }));
    }
    if percentile.value.raw_i64() < 0 || percentile.value.raw_i64() > Fixed64::ONE.raw_i64() {
        return Err("분위수 p는 0..1 범위여야 합니다".to_string().into());
    }
    let mode = if args.len() == 3 {
        parse_percentile_mode(&args[2])?
    } else {
        PercentileMode::Linear
    };
    Ok((values, percentile.value, mode))
}

pub(super) fn parse_percentile_mode(value: &Value) -> Result<PercentileMode, EvalError> {
    let Value::String(mode) = value else {
        return Err("분위수 mode는 글이어야 합니다".to_string().into());
    };
    match mode.trim() {
        "선형보간" => Ok(PercentileMode::Linear),
        "최근순위" => Ok(PercentileMode::NearestRank),
        _ => Err("분위수 mode는 선형보간 또는 최근순위여야 합니다"
            .to_string()
            .into()),
    }
}

pub(super) fn fixed64_to_nonnegative_index(value: Fixed64) -> Result<usize, EvalError> {
    if value.raw_i64() < 0 || value.frac_part() != 0 {
        return Err("분위수 인덱스는 0 이상의 정수여야 합니다"
            .to_string()
            .into());
    }
    Ok(value.int_part() as usize)
}

pub(super) fn nearest_rank_index(p: Fixed64, len: usize) -> Result<usize, EvalError> {
    if len == 0 {
        return Err("분위수 입력 목록이 비어 있습니다".to_string().into());
    }
    let p_raw = p.raw_i64() as i128;
    let n = len as i128;
    let scale = Fixed64::ONE_RAW as i128;
    let rank = if p_raw <= 0 {
        1
    } else {
        ((p_raw * n) + (scale - 1)) / scale
    };
    let clamped_rank = rank.clamp(1, n);
    Ok((clamped_rank - 1) as usize)
}

pub(super) struct NumericFormulaPrepared {
    pub(super) parsed: ParsedFormula,
    pub(super) var_name: String,
}

pub(super) fn expect_numeric_derivative_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String, UnitValue, UnitValue), EvalError> {
    if args.len() != 4 {
        return Err(format!("{label}는 인자 4개를 받습니다").to_string().into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(format!("{label}는 수식값 인자가 필요합니다")
                .to_string()
                .into())
        }
    };
    let raw_var = match &args[1] {
        Value::String(text) => text.clone(),
        _ => {
            return Err(format!("{label} 변수 이름이 글이어야 합니다")
                .to_string()
                .into())
        }
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(
            format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다")
                .to_string()
                .into(),
        );
    }
    let point = unit_value_from_value(&args[2])?;
    let step = unit_value_from_value(&args[3])?;
    if point.dim != step.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: point.dim,
            right: step.dim,
        }));
    }
    if step.value.raw_i64() == 0 {
        return Err(
            format!("E_CALC_NUMERIC_BAD_STEP: {label} 스텝은 0이 될 수 없습니다")
                .to_string()
                .into(),
        );
    }
    Ok((formula, var_name, point, step))
}

pub(super) fn expect_numeric_integral_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String, UnitValue, UnitValue, UnitValue), EvalError> {
    if args.len() != 5 {
        return Err(format!("{label}는 인자 5개를 받습니다").to_string().into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(format!("{label}는 수식값 인자가 필요합니다")
                .to_string()
                .into())
        }
    };
    let raw_var = match &args[1] {
        Value::String(text) => text.clone(),
        _ => {
            return Err(format!("{label} 변수 이름이 글이어야 합니다")
                .to_string()
                .into())
        }
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(
            format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다")
                .to_string()
                .into(),
        );
    }
    let start = unit_value_from_value(&args[2])?;
    let end = unit_value_from_value(&args[3])?;
    let step = unit_value_from_value(&args[4])?;
    if start.dim != end.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: start.dim,
            right: end.dim,
        }));
    }
    if start.dim != step.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: start.dim,
            right: step.dim,
        }));
    }
    if step.value.raw_i64() <= 0 {
        return Err(
            format!("E_CALC_NUMERIC_BAD_STEP: {label} 스텝은 0보다 커야 합니다")
                .to_string()
                .into(),
        );
    }
    Ok((formula, var_name, start, end, step))
}

pub(super) fn expect_numeric_bisection_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String, UnitValue, UnitValue, usize), EvalError> {
    if args.len() != 5 {
        return Err(format!("{label}는 인자 5개를 받습니다").to_string().into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(format!("{label}는 수식값 인자가 필요합니다")
                .to_string()
                .into())
        }
    };
    let raw_var = match &args[1] {
        Value::String(text) => text.clone(),
        _ => {
            return Err(format!("{label} 변수 이름이 글이어야 합니다")
                .to_string()
                .into())
        }
    };
    let var_name = raw_var.trim().trim_start_matches('#').to_string();
    if var_name.is_empty() {
        return Err(
            format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다")
                .to_string()
                .into(),
        );
    }
    let lower = unit_value_from_value(&args[2])?;
    let upper = unit_value_from_value(&args[3])?;
    let iterations = unit_value_from_value(&args[4])?;
    if lower.dim != upper.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: lower.dim,
            right: upper.dim,
        }));
    }
    if iterations.dim != UnitDim::NONE {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: iterations.dim,
            right: UnitDim::NONE,
        }));
    }
    let iteration_count = fixed64_to_nonnegative_index(iterations.value)?;
    if iteration_count == 0 {
        return Err(
            "E_CALC_NUMERIC_BAD_ITERATION: 수치해.이분법 반복횟수는 1 이상이어야 합니다"
                .to_string()
                .into(),
        );
    }
    Ok((formula, var_name, lower, upper, iteration_count))
}

pub(super) fn expect_polynomial_solve_args(
    args: &[Value],
    label: &'static str,
) -> Result<(Formula, String), EvalError> {
    if args.len() != 2 {
        return Err(format!("{label}는 인자 2개를 받습니다").into());
    }
    let formula = match &args[0] {
        Value::Formula(value) => value.clone(),
        _ => return Err(format!("{label} 첫 번째 인자는 수식값이어야 합니다").into()),
    };
    let var_name = match &args[1] {
        Value::String(value) => value.trim().trim_start_matches('#').to_string(),
        _ => return Err(format!("{label} 두 번째 인자는 변수 글이어야 합니다").into()),
    };
    if var_name.is_empty() {
        return Err(format!("E_CALC_NUMERIC_BAD_VAR: {label} 변수 이름이 비어 있습니다").into());
    }
    Ok((formula, var_name))
}

pub(super) fn prepare_numeric_formula(
    formula: &Formula,
    var_name: &str,
    label: &'static str,
) -> Result<NumericFormulaPrepared, EvalError> {
    if !matches!(formula.dialect, FormulaDialect::Ascii) {
        return Err(format!(
            "E_CALC_NUMERIC_DIALECT_UNSUPPORTED: {label}는 #ascii 수식만 지원합니다"
        )
        .to_string()
        .into());
    }
    ensure_formula_ident(var_name, &formula.dialect, label)?;
    let parsed = parse_formula_value(formula)?;
    if !parsed.vars.contains(var_name) {
        return Err(format!(
            "E_CALC_NUMERIC_FREEVAR_NOT_FOUND: {label} 변수 '{}'가 수식에 없습니다",
            var_name
        )
        .to_string()
        .into());
    }
    if parsed.vars.len() != 1 {
        return Err(
            format!("E_CALC_NUMERIC_FREEVAR_AMBIGUOUS: {label} 변수 이름이 여러 개입니다")
                .to_string()
                .into(),
        );
    }
    Ok(NumericFormulaPrepared {
        parsed,
        var_name: var_name.to_string(),
    })
}

pub(super) fn eval_polynomial_solve_result_with_meter(
    args: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<Value, EvalError> {
    let (formula, var_name) = expect_polynomial_solve_args(args, "다항식.풀기")?;
    let _prepared = prepare_numeric_formula(&formula, &var_name, "다항식.풀기")?;
    let zero = Formula {
        raw: "0".to_string(),
        dialect: formula.dialect.clone(),
        explicit_tag: formula.explicit_tag,
    };
    let mut relation_value = make_relation_pack(formula, zero);
    let Value::Pack(relation) = &mut relation_value else {
        return Err("다항식.풀기 relation pack 생성 실패".to_string().into());
    };
    eval_relation_solve_result_with_meter(&[std::mem::take(relation)], work_cost)
}

#[cfg(test)]
pub(super) fn eval_polynomial_solve_result(args: &[Value]) -> Result<Value, EvalError> {
    eval_polynomial_solve_result_with_meter(args, None)
}

#[derive(Clone, Copy)]
pub(super) struct LinearInequalityBound {
    pub(super) value: Fixed64,
    pub(super) inclusive: bool,
}

#[derive(Default)]
pub(super) struct LinearInequalityInterval {
    pub(super) lower: Option<LinearInequalityBound>,
    pub(super) upper: Option<LinearInequalityBound>,
    pub(super) empty: bool,
}

pub(super) fn tighten_linear_lower(interval: &mut LinearInequalityInterval, value: Fixed64, inclusive: bool) {
    let next = LinearInequalityBound { value, inclusive };
    match interval.lower {
        None => interval.lower = Some(next),
        Some(current)
            if value > current.value
                || (value == current.value && !inclusive && current.inclusive) =>
        {
            interval.lower = Some(next);
        }
        _ => {}
    }
}

pub(super) fn tighten_linear_upper(interval: &mut LinearInequalityInterval, value: Fixed64, inclusive: bool) {
    let next = LinearInequalityBound { value, inclusive };
    match interval.upper {
        None => interval.upper = Some(next),
        Some(current)
            if value < current.value
                || (value == current.value && !inclusive && current.inclusive) =>
        {
            interval.upper = Some(next);
        }
        _ => {}
    }
}

pub(super) fn linear_zero_condition_satisfies(compare: &str, rhs: Fixed64) -> bool {
    let zero = Fixed64::ZERO;
    match compare {
        "<=" | "이하" => zero <= rhs,
        "<" | "미만" => zero < rhs,
        ">=" | "이상" => zero >= rhs,
        ">" | "초과" => zero > rhs,
        _ => false,
    }
}

pub(super) fn apply_linear_inequality_constraint(
    interval: &mut LinearInequalityInterval,
    slope: Fixed64,
    rhs: Fixed64,
    compare: &str,
) -> Result<(), EvalError> {
    if slope.to_raw() == 0 {
        if !linear_zero_condition_satisfies(compare, rhs) {
            interval.empty = true;
        }
        return Ok(());
    }
    let bound = rhs.try_div(slope).map_err(|_| {
        "E_LINEAR_INEQUALITY_DIV_ZERO: 선형부등식 경계 계산 중 0으로 나눌 수 없습니다".to_string()
    })?;
    let inclusive = matches!(compare, "<=" | ">=" | "이하" | "이상");
    let slope_positive = slope.to_raw() > 0;
    match (compare, slope_positive) {
        ("<=" | "<" | "이하" | "미만", true) | (">=" | ">" | "이상" | "초과", false) => {
            tighten_linear_upper(interval, bound, inclusive)
        }
        ("<=" | "<" | "이하" | "미만", false) | (">=" | ">" | "이상" | "초과", true) => {
            tighten_linear_lower(interval, bound, inclusive)
        }
        _ => {
            return Err(
                "E_LINEAR_INEQUALITY_BAD_COMPARE: 비교는 이하, 미만, 이상, 초과 중 하나여야 합니다"
                    .to_string()
                    .into(),
            )
        }
    }
    Ok(())
}

pub(super) fn linear_inequality_pack_string_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<String, EvalError> {
    match fields.get(field) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(format!("선형부등식 조건 {field} 필드는 글이어야 합니다").into()),
        None => Err(format!("선형부등식 조건에 {field} 필드가 필요합니다").into()),
    }
}

pub(super) fn linear_inequality_pack_formula_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<Formula, EvalError> {
    match fields.get(field) {
        Some(Value::Formula(value)) => Ok(value.clone()),
        Some(_) => Err(format!("선형부등식 조건 {field} 필드는 수식값이어야 합니다").into()),
        None => Err(format!("선형부등식 조건에 {field} 필드가 필요합니다").into()),
    }
}

pub(super) fn linear_inequality_pack_unit_field(
    fields: &BTreeMap<String, Value>,
    field: &str,
) -> Result<UnitValue, EvalError> {
    match fields.get(field) {
        Some(value) => unit_value_from_value(value),
        None => Err(format!("선형부등식 조건에 {field} 필드가 필요합니다").into()),
    }
}

pub(super) fn eval_linear_inequality_solve(
    args: &[Value],
    work_cost: &Rc<RefCell<WorkCostMeter>>,
) -> Result<Value, EvalError> {
    if args.len() != 2 {
        return Err("선형부등식.풀기는 인자 2개를 받습니다".to_string().into());
    }
    let conditions = match &args[0] {
        Value::List(items) => items,
        _ => {
            return Err("선형부등식.풀기 첫 번째 인자는 조건 차림이어야 합니다"
                .to_string()
                .into())
        }
    };
    let var_name = match &args[1] {
        Value::String(value) => value.trim().trim_start_matches('#').to_string(),
        _ => {
            return Err("선형부등식.풀기 두 번째 인자는 변수 글이어야 합니다"
                .to_string()
                .into())
        }
    };
    if var_name.is_empty() {
        return Err("E_LINEAR_INEQUALITY_BAD_VAR: 변수 이름이 비어 있습니다"
            .to_string()
            .into());
    }

    let mut interval = LinearInequalityInterval::default();
    for condition in conditions {
        let Value::Pack(fields) = condition else {
            return Err("선형부등식 조건은 묶음이어야 합니다".to_string().into());
        };
        let formula = linear_inequality_pack_formula_field(fields, "식")?;
        let compare = linear_inequality_pack_string_field(fields, "비교")?;
        let boundary = linear_inequality_pack_unit_field(fields, "경계")?;
        let prepared = prepare_numeric_formula(&formula, &var_name, "선형부등식.풀기")?;
        let y0 = eval_numeric_formula_at_metered(
            &prepared,
            UnitValue {
                value: Fixed64::from_i64(0),
                dim: UnitDim::NONE,
            },
            "선형부등식.풀기",
            work_cost,
        )?;
        let y1 = eval_numeric_formula_at_metered(
            &prepared,
            UnitValue {
                value: Fixed64::from_i64(1),
                dim: UnitDim::NONE,
            },
            "선형부등식.풀기",
            work_cost,
        )?;
        let y2 = eval_numeric_formula_at_metered(
            &prepared,
            UnitValue {
                value: Fixed64::from_i64(2),
                dim: UnitDim::NONE,
            },
            "선형부등식.풀기",
            work_cost,
        )?;
        for rhs in [y1, y2, boundary] {
            if y0.dim != rhs.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: y0.dim,
                    right: rhs.dim,
                }));
            }
        }
        let slope01 = y1.value.checked_sub(y0.value)?;
        let slope12 = y2.value.checked_sub(y1.value)?;
        if slope01 != slope12 {
            return Err(
                "E_LINEAR_INEQUALITY_NONLINEAR: V1은 1변수 선형 부등식만 지원합니다"
                    .to_string()
                    .into(),
            );
        }
        let rhs = boundary.value.checked_sub(y0.value)?;
        apply_linear_inequality_constraint(&mut interval, slope01, rhs, compare.trim())?;
    }

    if let (Some(lower), Some(upper)) = (interval.lower, interval.upper) {
        if lower.value > upper.value
            || (lower.value == upper.value && (!lower.inclusive || !upper.inclusive))
        {
            interval.empty = true;
        }
    }

    let mut fields = BTreeMap::new();
    fields.insert(
        "__종류".to_string(),
        Value::String("linear_inequality_solution".to_string()),
    );
    fields.insert("변수".to_string(), Value::String(var_name));
    fields.insert(
        "제약개수".to_string(),
        unit_value_to_value(UnitValue {
            value: Fixed64::from_i64(conditions.len() as i64),
            dim: UnitDim::NONE,
        }),
    );
    if interval.empty {
        fields.insert("상태".to_string(), Value::String("공집합".to_string()));
    } else {
        fields.insert(
            "상태".to_string(),
            Value::String(
                if interval.lower.is_none() && interval.upper.is_none() {
                    "전체"
                } else {
                    "구간"
                }
                .to_string(),
            ),
        );
        if let Some(lower) = interval.lower {
            fields.insert(
                "하한".to_string(),
                unit_value_to_value(UnitValue {
                    value: lower.value,
                    dim: UnitDim::NONE,
                }),
            );
            fields.insert("하한포함".to_string(), Value::Bool(lower.inclusive));
        }
        if let Some(upper) = interval.upper {
            fields.insert(
                "상한".to_string(),
                unit_value_to_value(UnitValue {
                    value: upper.value,
                    dim: UnitDim::NONE,
                }),
            );
            fields.insert("상한포함".to_string(), Value::Bool(upper.inclusive));
        }
    }
    Ok(Value::Pack(fields))
}

pub(super) fn eval_numeric_formula_at(
    prepared: &NumericFormulaPrepared,
    x: UnitValue,
    label: &'static str,
) -> Result<UnitValue, EvalError> {
    let mut env = BTreeMap::new();
    env.insert(prepared.var_name.clone(), unit_value_to_value(x));
    eval_formula_expr(&prepared.parsed.expr, &env).map_err(|err| {
        EvalError::Message(format!(
            "E_CALC_NUMERIC_EVAL_FAIL: {label} 수식 평가 실패 ({err})"
        ))
    })
}

pub(super) fn eval_numeric_formula_at_metered(
    prepared: &NumericFormulaPrepared,
    x: UnitValue,
    label: &'static str,
    work_cost: &Rc<RefCell<WorkCostMeter>>,
) -> Result<UnitValue, EvalError> {
    work_cost.borrow_mut().charge_solver_evaluations(1)?;
    work_cost.borrow_mut().enter_call()?;
    let result = eval_numeric_formula_at(prepared, x, label);
    work_cost.borrow_mut().exit_call();
    result
}

pub(super) fn numeric_trapezoid_integral(
    prepared: &NumericFormulaPrepared,
    start: UnitValue,
    end: UnitValue,
    segments: usize,
    work_cost: &Rc<RefCell<WorkCostMeter>>,
) -> Result<UnitValue, EvalError> {
    if segments == 0 {
        return Err("E_CALC_NUMERIC_BAD_STEP: 적분.사다리꼴 구간 수가 0입니다"
            .to_string()
            .into());
    }
    let interval = end.value.checked_sub(start.value)?;
    let seg = Fixed64::from_i64(segments as i64);
    let h = interval.try_div(seg)?;

    let mut y_dim: Option<UnitDim> = None;
    let mut sum_raw = Fixed64::ZERO;
    let two = Fixed64::from_i64(2);
    for idx in 0..=segments {
        let idx_raw = Fixed64::from_i64(idx as i64);
        let x = UnitValue {
            value: start.value.checked_add(h.checked_mul(idx_raw)?)?,
            dim: start.dim,
        };
        let y = eval_numeric_formula_at_metered(prepared, x, "적분.사다리꼴", work_cost)?;
        if let Some(dim) = y_dim {
            if y.dim != dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: dim,
                    right: y.dim,
                }));
            }
        } else {
            y_dim = Some(y.dim);
        }
        let weight = if idx == 0 || idx == segments {
            Fixed64::from_i64(1)
        } else {
            two
        };
        sum_raw = sum_raw.checked_add(y.value.checked_mul(weight)?)?;
    }
    let approx = sum_raw.checked_mul(h)?.try_div(Fixed64::from_i64(2))?;
    Ok(UnitValue {
        value: approx,
        dim: y_dim.unwrap_or(UnitDim::NONE).add(start.dim),
    })
}

pub(super) fn fixed64_sign(value: Fixed64) -> i8 {
    match value.raw_i64().cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

pub(super) fn numeric_bisection_root(
    prepared: &NumericFormulaPrepared,
    mut lower: UnitValue,
    mut upper: UnitValue,
    iterations: usize,
    work_cost: &Rc<RefCell<WorkCostMeter>>,
) -> Result<(UnitValue, UnitValue, usize), EvalError> {
    if upper.value < lower.value {
        std::mem::swap(&mut lower, &mut upper);
    }

    let mut f_lower = eval_numeric_formula_at_metered(prepared, lower, "수치해.이분법", work_cost)?;
    let mut f_upper = eval_numeric_formula_at_metered(prepared, upper, "수치해.이분법", work_cost)?;
    if f_lower.dim != f_upper.dim {
        return Err(unit_error(UnitError::DimensionMismatch {
            left: f_lower.dim,
            right: f_upper.dim,
        }));
    }
    if f_lower.value.raw_i64() == 0 {
        return Ok((
            lower,
            UnitValue {
                value: Fixed64::ZERO,
                dim: f_lower.dim,
            },
            0,
        ));
    }
    if f_upper.value.raw_i64() == 0 {
        return Ok((
            upper,
            UnitValue {
                value: Fixed64::ZERO,
                dim: f_upper.dim,
            },
            0,
        ));
    }
    if fixed64_sign(f_lower.value) == fixed64_sign(f_upper.value) {
        return Err(
            "E_CALC_NUMERIC_BRACKET_SIGN: 수치해.이분법 하한/상한 함수값의 부호가 달라야 합니다"
                .to_string()
                .into(),
        );
    }

    let two = Fixed64::from_i64(2);
    let mut best_root = lower;
    let mut best_residual = f_lower;
    let mut used_iterations = 0_usize;
    for idx in 1..=iterations {
        let mid_value = lower.value.checked_add(upper.value)?.try_div(two)?;
        let mid = UnitValue {
            value: mid_value,
            dim: lower.dim,
        };
        let f_mid = eval_numeric_formula_at_metered(prepared, mid, "수치해.이분법", work_cost)?;
        if f_lower.dim != f_mid.dim {
            return Err(unit_error(UnitError::DimensionMismatch {
                left: f_lower.dim,
                right: f_mid.dim,
            }));
        }
        best_root = mid;
        best_residual = UnitValue {
            value: fixed64_abs(f_mid.value)?,
            dim: f_mid.dim,
        };
        used_iterations = idx;

        if f_mid.value.raw_i64() == 0 {
            break;
        }
        if fixed64_sign(f_lower.value) != fixed64_sign(f_mid.value) {
            upper = mid;
            f_upper = f_mid;
        } else {
            lower = mid;
            f_lower = f_mid;
        }
        if f_lower.dim != f_upper.dim {
            return Err(unit_error(UnitError::DimensionMismatch {
                left: f_lower.dim,
                right: f_upper.dim,
            }));
        }
    }

    Ok((best_root, best_residual, used_iterations))
}

pub(super) fn fixed64_sqrt(value: Fixed64) -> Option<Fixed64> {
    if value.raw_i64() < 0 {
        return None;
    }
    let scaled = (value.raw_i64() as i128) << Fixed64::FRAC_BITS;
    let root = int_sqrt(scaled);
    let capped = if root > i64::MAX as i128 {
        i64::MAX
    } else {
        root as i64
    };
    Some(Fixed64::from_raw_i64(capped))
}

pub(super) fn int_sqrt(value: i128) -> i128 {
    if value <= 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) >> 1;
    while y < x {
        x = y;
        y = (x + value / x) >> 1;
    }
    x
}

pub(super) fn unit_powi(base: UnitValue, exp: i64) -> Result<UnitValue, EvalError> {
    if exp == 0 {
        return Ok(UnitValue {
            value: Fixed64::from_i64(1),
            dim: UnitDim::NONE,
        });
    }
    if exp == i64::MIN {
        return Err(ArithmeticFaultKind::Overflow.into());
    }
    let mut result = base;
    let abs_exp = exp.unsigned_abs();
    for _ in 1..abs_exp {
        result = UnitValue {
            value: result.value.checked_mul(base.value)?,
            dim: result.dim.add(base.dim),
        };
    }
    if exp < 0 {
        let one = UnitValue {
            value: Fixed64::from_i64(1),
            dim: UnitDim::NONE,
        };
        return one.div(result).map_err(unit_error);
    }
    Ok(result)
}

