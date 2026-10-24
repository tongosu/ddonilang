//! Formula parsing, symbolic transforms and value evaluation; pure moved implementation.
// 패닉 감사 F5(가): 아래 직접 인덱스/expect의 경계는 입력과 같은 불변 자료에서 검증한다.
// - parse_formula_value/parse_formula_with_vars: find_top_level_eq는 같은 tokens의
//   enumerate 인덱스를 반환한다. 양끝 거부 뒤 분할하며 lhs.len()==1 뒤에만 lhs[0]을 읽는다.
// - expect_* 인자 변환: 비어 있음/정확한 길이를 먼저 검사한다. transform의 두 번째
//   값은 길이 1 조기 반환 뒤 길이 2인 분기에서만 읽는다.
// - 관계/묶음 추출: values.len()==1 또는 relations.len()==1 검사를 통과한 뒤
//   같은 불변 slice/소유 vector의 첫 원소를 읽으며 그 사이 길이 변경은 없다.
// - 변수 선택의 iter().next().expect는 불변 BTreeSet의 len()==1 분기에만 있다.
// - eval_formula_func: 단항은 private expect_args(1)의 길이 검사를 소비한다.
//   이항/삼항은 각각 길이 2/3 거부 검사 뒤 같은 소유 args의 인덱스를 읽는다.
// 이 설명은 인덱스/expect 경계만 다룬다. 산술 넘침과 재귀/자원 상한의 안전성은
// 별도 검사 대상이며 이 주석이나 길이 검사를 그 보장으로 사용하지 않는다.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use ddonirang_core::{Fixed64, UnitValue, UnitDim, UnitError};
use ddonirang_lang::{Formula, FormulaDialect};
use ddonirang_lang::runtime::Value;
use crate::work_cost::WorkCostMeter;
use super::{EvalError, checked_unit_mul, checked_unit_add, checked_unit_sub, unit_value_from_value, unit_error, fixed64_floor, fixed64_to_f64, fixed64_from_host_f64_checked, fixed64_ceil, fixed64_round_even, fixed64_abs, fixed64_sqrt, unit_powi, exact_numeric_from_value, expect_single_equation_relation, relation_formula_text, RELATION_LEFT_FIELD, RELATION_RIGHT_FIELD};

#[derive(Debug, Clone)]
pub(super) struct ParsedFormula {
    pub(super) expr: FormulaExpr,
    pub(super) vars: BTreeSet<String>,
}

#[derive(Default)]
pub(super) struct FormulaTransformOptions {
    pub(super) var_name: Option<String>,
    pub(super) order: Option<i64>,
    pub(super) include_const: Option<bool>,
}

pub(super) struct FormulaAnalysis {
    pub(super) assign_name: Option<String>,
    pub(super) expr: FormulaExpr,
    pub(super) vars: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub(super) enum FormulaExpr {
    Number(UnitValue),
    Var(String),
    Func {
        name: String,
        args: Vec<FormulaExpr>,
    },
    Unary {
        op: FormulaOp,
        expr: Box<FormulaExpr>,
    },
    Binary {
        op: FormulaOp,
        left: Box<FormulaExpr>,
        right: Box<FormulaExpr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) enum FormulaOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone)]
enum FormulaToken {
    Number(Fixed64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Eq,
    Comma,
}

struct FormulaParser {
    tokens: Vec<FormulaToken>,
    pos: usize,
    dialect: FormulaDialect,
    vars: BTreeSet<String>,
}

impl FormulaParser {
    fn new(tokens: Vec<FormulaToken>, dialect: FormulaDialect) -> Self {
        Self {
            tokens,
            pos: 0,
            dialect,
            vars: BTreeSet::new(),
        }
    }

    fn parse_expr(&mut self) -> Result<FormulaExpr, EvalError> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<FormulaExpr, EvalError> {
        let mut expr = self.parse_mul_div()?;
        loop {
            if self.consume(&FormulaToken::Plus) {
                let rhs = self.parse_mul_div()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Add,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if self.consume(&FormulaToken::Minus) {
                let rhs = self.parse_mul_div()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Sub,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_mul_div(&mut self) -> Result<FormulaExpr, EvalError> {
        let mut expr = self.parse_pow()?;
        loop {
            if self.consume(&FormulaToken::Star) {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Mul,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if self.consume(&FormulaToken::Slash) {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Div,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if self.consume(&FormulaToken::Percent) {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Mod,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            if matches!(self.dialect, FormulaDialect::Ascii1) && self.next_starts_factor() {
                let rhs = self.parse_pow()?;
                expr = FormulaExpr::Binary {
                    op: FormulaOp::Mul,
                    left: Box::new(expr),
                    right: Box::new(rhs),
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_pow(&mut self) -> Result<FormulaExpr, EvalError> {
        let mut expr = self.parse_unary()?;
        if self.consume(&FormulaToken::Caret) {
            let rhs = self.parse_pow()?;
            expr = FormulaExpr::Binary {
                op: FormulaOp::Pow,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<FormulaExpr, EvalError> {
        if self.consume(&FormulaToken::Plus) {
            return self.parse_unary();
        }
        if self.consume(&FormulaToken::Minus) {
            let expr = self.parse_unary()?;
            return Ok(FormulaExpr::Unary {
                op: FormulaOp::Sub,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<FormulaExpr, EvalError> {
        let Some(token) = self.next() else {
            return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
        };
        match token {
            FormulaToken::Number(value) => Ok(FormulaExpr::Number(UnitValue {
                value,
                dim: UnitDim::NONE,
            })),
            FormulaToken::Ident(name) => {
                if self.consume(&FormulaToken::LParen) {
                    if self.consume(&FormulaToken::RParen) {
                        return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
                    }
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_expr()?);
                        if self.consume(&FormulaToken::Comma) {
                            continue;
                        }
                        if self.consume(&FormulaToken::RParen) {
                            break;
                        }
                        return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
                    }
                    Ok(FormulaExpr::Func { name, args })
                } else {
                    self.vars.insert(name.clone());
                    Ok(FormulaExpr::Var(name))
                }
            }
            FormulaToken::LParen => {
                let expr = self.parse_expr()?;
                if !self.consume(&FormulaToken::RParen) {
                    return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
                }
                Ok(expr)
            }
            _ => Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string())),
        }
    }

    fn consume(&mut self, expected: &FormulaToken) -> bool {
        if let Some(token) = self.peek() {
            if std::mem::discriminant(token) == std::mem::discriminant(expected) {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn next(&mut self) -> Option<FormulaToken> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn peek(&self) -> Option<&FormulaToken> {
        self.tokens.get(self.pos)
    }

    fn next_starts_factor(&self) -> bool {
        matches!(
            self.peek(),
            Some(FormulaToken::Number(_))
                | Some(FormulaToken::Ident(_))
                | Some(FormulaToken::LParen)
        )
    }
}

pub(super) fn parse_formula_value(formula: &Formula) -> Result<ParsedFormula, EvalError> {
    let tokens = tokenize_formula(&formula.raw, &formula.dialect)?;
    let eq_index = find_top_level_eq(&tokens)?;
    let (expr_tokens, vars) = if let Some(eq_index) = eq_index {
        if eq_index == 0 || eq_index + 1 >= tokens.len() {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        let lhs = &tokens[..eq_index];
        let rhs = tokens[eq_index + 1..].to_vec();
        if lhs.len() != 1 {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        match &lhs[0] {
            FormulaToken::Ident(_) => {}
            _ => {
                return Err(EvalError::Message(
                    "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
                ))
            }
        }
        (rhs, BTreeSet::new())
    } else {
        (tokens, BTreeSet::new())
    };
    let mut parser = FormulaParser::new(expr_tokens, formula.dialect.clone());
    parser.vars = vars;
    let expr = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
    }
    Ok(ParsedFormula {
        expr,
        vars: parser.vars,
    })
}

fn parse_formula_with_vars(
    body: &str,
    dialect: &FormulaDialect,
) -> Result<(Option<String>, FormulaExpr, BTreeSet<String>), EvalError> {
    let tokens = tokenize_formula(body, dialect)?;
    let eq_index = find_top_level_eq(&tokens)?;
    let (expr_tokens, assign_name) = if let Some(eq_index) = eq_index {
        if eq_index == 0 || eq_index + 1 >= tokens.len() {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        let lhs = &tokens[..eq_index];
        if lhs.len() != 1 {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        }
        let FormulaToken::Ident(name) = &lhs[0] else {
            return Err(EvalError::Message(
                "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
            ));
        };
        (tokens[eq_index + 1..].to_vec(), Some(name.clone()))
    } else {
        (tokens, None)
    };
    let mut parser = FormulaParser::new(expr_tokens, dialect.clone());
    let expr = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
        return Err(EvalError::Message("FATAL:FORMULA_SYNTAX".to_string()));
    }
    Ok((assign_name, expr, parser.vars))
}

pub(super) fn analyze_formula_for_transform(formula: &Formula) -> Result<FormulaAnalysis, EvalError> {
    let (assign_name, expr, mut vars) = parse_formula_with_vars(&formula.raw, &formula.dialect)?;
    if let Some(name) = &assign_name {
        vars.remove(name);
    }
    Ok(FormulaAnalysis {
        assign_name,
        expr,
        vars,
    })
}

pub(super) fn format_formula_body(body: &str, dialect: &FormulaDialect) -> Result<String, EvalError> {
    let (assign_name, expr, _) = parse_formula_with_vars(body, dialect)?;
    let expr_text = format_formula_expr(&expr, 0);
    if let Some(assign) = assign_name {
        Ok(format!("{} = {}", assign, expr_text))
    } else {
        Ok(expr_text)
    }
}

pub(super) fn format_formula_expr(expr: &FormulaExpr, parent_prec: u8) -> String {
    match expr {
        FormulaExpr::Number(value) => value.value.canonical_decimal_text(),
        FormulaExpr::Var(name) => name.clone(),
        FormulaExpr::Func { name, args } => {
            let rendered = args
                .iter()
                .map(|arg| format_formula_expr(arg, 0))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, rendered)
        }
        FormulaExpr::Unary {
            op: FormulaOp::Sub,
            expr,
        } => {
            let inner = format_formula_expr(expr, 3);
            format!("-{}", inner)
        }
        FormulaExpr::Unary { expr, .. } => format_formula_expr(expr, parent_prec),
        FormulaExpr::Binary { op, left, right } => {
            let (prec, op_str, spaced, right_assoc) = match op {
                FormulaOp::Add => (1, "+", true, false),
                FormulaOp::Sub => (1, "-", true, false),
                FormulaOp::Mul => (2, "*", false, false),
                FormulaOp::Div => (2, "/", false, false),
                FormulaOp::Mod => (2, "%", false, false),
                FormulaOp::Pow => (3, "^", false, true),
            };
            let (left_prec, right_prec) = if right_assoc {
                (prec + 1, prec)
            } else {
                (prec, prec + 1)
            };
            let left_str = format_formula_expr(left, left_prec);
            let right_str = format_formula_expr(right, right_prec);
            let joined = if spaced {
                format!("{} {} {}", left_str, op_str, right_str)
            } else {
                format!("{}{}{}", left_str, op_str, right_str)
            };
            if prec < parent_prec {
                format!("({})", joined)
            } else {
                joined
            }
        }
    }
}

pub(super) fn expect_formula_transform(
    values: &[Value],
    label: &'static str,
) -> Result<(Formula, FormulaTransformOptions), EvalError> {
    if values.is_empty() || values.len() > 2 {
        return Err(EvalError::Message(
            "formula[, variable|options]".to_string(),
        ));
    }
    let formula = match &values[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{}는 수식값 인자가 필요합니다",
                label
            )))
        }
    };
    if values.len() == 1 {
        return Ok((formula, FormulaTransformOptions::default()));
    }
    let options = parse_formula_transform_arg(&values[1], label)?;
    Ok((formula, options))
}

pub(super) fn expect_single_formula(values: &[Value], label: &'static str) -> Result<Formula, EvalError> {
    if values.len() != 1 {
        return Err(EvalError::Message(format!(
            "{label}는 수식값 인자 1개를 받습니다"
        )));
    }
    match &values[0] {
        Value::Formula(value) => Ok(value.clone()),
        _ => Err(EvalError::Message(format!(
            "{label}는 수식값 인자가 필요합니다"
        ))),
    }
}

pub(super) fn expect_two_formulas(
    values: &[Value],
    label: &'static str,
) -> Result<(Formula, Formula), EvalError> {
    if values.len() != 2 {
        return Err(EvalError::Message(format!(
            "{label}는 수식값 인자 2개를 받습니다"
        )));
    }
    let left = match &values[0] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{label} 왼쪽은 수식값이어야 합니다"
            )))
        }
    };
    let right = match &values[1] {
        Value::Formula(value) => value.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{label} 오른쪽은 수식값이어야 합니다"
            )))
        }
    };
    Ok((left, right))
}

fn parse_formula_transform_arg(
    value: &Value,
    label: &'static str,
) -> Result<FormulaTransformOptions, EvalError> {
    match value {
        Value::String(_) => Ok(FormulaTransformOptions {
            var_name: Some(parse_formula_var_name(value, label)?),
            ..FormulaTransformOptions::default()
        }),
        Value::Pack(pack) => parse_formula_transform_pack(pack, label),
        _ => Err(EvalError::Message(
            "string variable or pack options".to_string(),
        )),
    }
}

fn parse_formula_transform_pack(
    pack: &BTreeMap<String, Value>,
    label: &'static str,
) -> Result<FormulaTransformOptions, EvalError> {
    let mut options = FormulaTransformOptions::default();
    for key in pack.keys() {
        match key.as_str() {
            "변수" | "차수" | "상수포함" => {}
            _ => {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: {} 옵션을 지원하지 않습니다",
                    key
                )))
            }
        }
    }

    if let Some(value) = pack.get("변수") {
        if !matches!(value, Value::None) {
            options.var_name = Some(parse_formula_var_name(value, label)?);
        }
    }
    if let Some(value) = pack.get("차수") {
        if !matches!(value, Value::None) {
            let order = expect_int_strict(value)?;
            if order < 1 {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_BAD_ORDER: {} 차수는 1 이상이어야 합니다",
                    label
                )));
            }
            options.order = Some(order);
        }
    }
    if let Some(value) = pack.get("상수포함") {
        if !matches!(value, Value::None) {
            let include = match value {
                Value::Bool(flag) => *flag,
                _ => {
                    return Err(EvalError::Message(
                        "상수포함은 참/거짓이어야 합니다".to_string(),
                    ))
                }
            };
            options.include_const = Some(include);
        }
    }
    Ok(options)
}

fn parse_formula_var_name(value: &Value, label: &'static str) -> Result<String, EvalError> {
    let raw = match value {
        Value::String(text) => text.clone(),
        _ => {
            return Err(EvalError::Message(format!(
                "{} 변수 이름이 글이어야 합니다",
                label
            )))
        }
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 비어 있습니다",
            label
        )));
    }
    Ok(trimmed.trim_start_matches('#').to_string())
}

pub(super) fn expect_int_strict(value: &Value) -> Result<i64, EvalError> {
    match value {
        Value::Fixed64(n) => {
            if n.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(n.int_part())
        }
        Value::Unit(unit) if unit.is_dimensionless() => {
            if unit.value.frac_part() != 0 {
                return Err("정수 값이 필요합니다".to_string().into());
            }
            Ok(unit.value.int_part())
        }
        _ => Err("정수 값이 필요합니다".to_string().into()),
    }
}

pub(super) fn transform_formula_value(
    formula: Formula,
    options: FormulaTransformOptions,
    call_name: &'static str,
    label: &'static str,
) -> Result<Formula, EvalError> {
    let dialect = formula.dialect.clone();
    if !matches!(dialect, FormulaDialect::Ascii) {
        return Err(EvalError::Message(format!(
            "{}는 #ascii 수식만 지원합니다",
            label
        )));
    }

    let analysis = analyze_formula_for_transform(&formula)?;
    let vars = analysis.vars.clone();
    let needs_var = matches!(call_name, "diff" | "int");
    let var_name = if needs_var {
        let var_name = match options.var_name {
            Some(name) => {
                if !vars.contains(&name) {
                    return Err(EvalError::Message(format!(
                        "E_CALC_FREEVAR_NOT_FOUND: {} 변수 '{}'가 수식에 없습니다",
                        label, name
                    )));
                }
                name
            }
            None => infer_single_var(&vars, label)?,
        };
        ensure_formula_ident(&var_name, &dialect, label)?;
        var_name
    } else {
        String::new()
    };

    if call_name == "diff" && options.include_const.is_some() {
        return Err(EvalError::Message(
            "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 미분하기는 상수포함을 지원하지 않습니다"
                .to_string(),
        ));
    }
    if call_name == "int" && options.order.is_some() {
        return Err(EvalError::Message(
            "E_CALC_TRANSFORM_UNSUPPORTED_OPTION: 적분하기는 차수를 지원하지 않습니다".to_string(),
        ));
    }

    let expr_text = format_formula_expr(&analysis.expr, 0);
    let expr_text = match call_name {
        "simplify" => ddonirang_symbolic::simplify(&expr_text),
        "expand" => ddonirang_symbolic::expand(&expr_text),
        "factor" => ddonirang_symbolic::factor(&expr_text),
        "diff" => {
            let mut out = expr_text;
            let order = options.order.unwrap_or(1);
            for _ in 0..order {
                out = ddonirang_symbolic::diff(&out, &var_name)
                    .map_err(|err| symbolic_formula_error(call_name, err))?;
            }
            Ok(out)
        }
        "int" => ddonirang_symbolic::integrate(&expr_text, &var_name),
        _ => Err(format!("E_SYMBOLIC_UNKNOWN_TRANSFORM {call_name}")),
    };
    let mut expr_text = expr_text.map_err(|err| symbolic_formula_error(call_name, err))?;
    if call_name == "int" && options.include_const.unwrap_or(false) {
        expr_text = format!("{} + C", expr_text);
    }

    let body = if let Some(assign) = analysis.assign_name {
        format!("{} = {}", assign, expr_text)
    } else {
        expr_text
    };
    let formatted = format_formula_body(&body, &dialect)?;
    Ok(Formula {
        raw: formatted,
        dialect,
        explicit_tag: formula.explicit_tag,
    })
}

pub(super) fn symbolic_formula_error(call_name: &str, message: String) -> EvalError {
    let label = match call_name {
        "simplify" => "정리하기",
        "expand" => "전개하기",
        "factor" => "인수분해하기",
        "diff" => "미분하기",
        "int" => "적분하기",
        "equiv" => "동치인가",
        _ => "수식변환",
    };
    EvalError::Message(format!(
        "E_SYMBOLIC_FORMULA_UNSUPPORTED: {label} ({message})"
    ))
}

pub(super) fn symbolic_formulas_equivalent(left: &Formula, right: &Formula) -> Result<bool, EvalError> {
    if !matches!(left.dialect, FormulaDialect::Ascii)
        || !matches!(right.dialect, FormulaDialect::Ascii)
    {
        return Err(EvalError::Message(
            "동치인가는 #ascii 수식만 지원합니다".to_string(),
        ));
    }
    let left_analysis = analyze_formula_for_transform(left)?;
    let right_analysis = analyze_formula_for_transform(right)?;
    let left_text = format_formula_expr(&left_analysis.expr, 0);
    let right_text = format_formula_expr(&right_analysis.expr, 0);
    ddonirang_symbolic::equivalent(&left_text, &right_text)
        .map_err(|err| symbolic_formula_error("equiv", err))
}

pub(super) fn verify_symbolic_assertion(left: &Formula, right: &Formula) -> Result<bool, String> {
    if !matches!(left.dialect, FormulaDialect::Ascii)
        || !matches!(right.dialect, FormulaDialect::Ascii)
    {
        return Err("세움 symbolic bridge는 #ascii 수식만 지원합니다".to_string());
    }
    let left_analysis = analyze_formula_for_transform(left).map_err(|err| err.message())?;
    let right_analysis = analyze_formula_for_transform(right).map_err(|err| err.message())?;
    let left_text = format_formula_expr(&left_analysis.expr, 0);
    let right_text = format_formula_expr(&right_analysis.expr, 0);
    let cert = ddonirang_symbolic::prove_equivalent(&left_text, &right_text)
        .map_err(|err| symbolic_formula_error("equiv", err).message())?;
    let cert_value = serde_json::to_value(&cert)
        .map_err(|err| format!("proof certificate serialize failed: {err}"))?;
    let report = ddonirang_proof::verify_value(&cert_value)
        .map_err(|err| format!("proof verify failed: {err}"))?;
    Ok(report.valid && cert.equivalent)
}

pub(super) fn parse_symbolic_equivalence_assertion(raw: &str) -> Option<(Formula, Formula)> {
    let body = trim_seum_wrapper(raw.trim());
    let eq_idx = find_top_level_equivalence_eq(body)?;
    let left = parse_formula_literal_text(body[..eq_idx].trim())?;
    let right = parse_formula_literal_text(body[eq_idx + 1..].trim())?;
    Some((left, right))
}

pub(super) fn parse_symbolic_relation_assertion(raw: &str) -> Option<(Formula, Formula)> {
    let body = trim_seum_wrapper(raw.trim());
    let eq_idx = body.find("=:=")?;
    let left = parse_formula_literal_text(body[..eq_idx].trim())?;
    let right = parse_formula_literal_text(body[eq_idx + 3..].trim())?;
    Some((left, right))
}

fn trim_seum_wrapper(raw: &str) -> &str {
    let trimmed = raw.trim();
    let Some(rest) = trimmed.strip_prefix("세움") else {
        return trimmed;
    };
    let rest = rest.trim_start();
    if !rest.starts_with('{') || !rest.ends_with('}') {
        return trimmed;
    }
    rest[1..rest.len().saturating_sub(1)].trim()
}

fn find_top_level_equivalence_eq(raw: &str) -> Option<usize> {
    let mut brace_depth = 0i32;
    let mut paren_depth = 0i32;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '=' if brace_depth == 0 && paren_depth == 0 => return Some(idx),
            _ => {}
        }
    }
    None
}

fn parse_formula_literal_text(raw: &str) -> Option<Formula> {
    let text = raw.trim().trim_end_matches('.').trim();
    let dialect = if text.contains("#ascii1") {
        FormulaDialect::Ascii1
    } else if text.contains("#ascii") {
        FormulaDialect::Ascii
    } else if text.contains("#latex") {
        FormulaDialect::Latex
    } else {
        return None;
    };
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(Formula {
        raw: text[start + 1..end].trim().to_string(),
        dialect,
        explicit_tag: true,
    })
}

fn solve_binding_from_value(value: &Value) -> Result<ddonirang_symbolic::SolveBinding, EvalError> {
    let Some(exact) = exact_numeric_from_value(value)? else {
        return Err("세움 relation binding은 exact numeric이어야 합니다"
            .to_string()
            .into());
    };
    Ok(ddonirang_symbolic::SolveBinding {
        numerator: exact.value.numer().to_string(),
        denominator: exact.value.denom().to_string(),
    })
}

pub(super) fn solve_bindings_from_hashmap(
    bindings: &HashMap<String, Value>,
) -> Result<BTreeMap<String, ddonirang_symbolic::SolveBinding>, EvalError> {
    let mut out = BTreeMap::new();
    for (name, value) in bindings {
        out.insert(name.clone(), solve_binding_from_value(value)?);
    }
    Ok(out)
}

pub(super) fn eval_symbolic_proof_tactic_with_meter(
    values: &[Value],
    work_cost: Option<&Rc<RefCell<WorkCostMeter>>>,
) -> Result<BTreeMap<String, Value>, EvalError> {
    if values.len() == 1 {
        let Value::Pack(pack) = &values[0] else {
            return Err(EvalError::Message(
                "증명하기는 묶음 인자가 필요합니다".to_string(),
            ));
        };
        if let Some(relation_value) = find_pack_value(pack, &["관계", "relation", "식", "equation"])
        {
            let relations = extract_relation_texts_from_value(relation_value)?;
            if let Some(bindings_value) = find_pack_value(pack, &["해", "bindings", "값들"]) {
                let Value::Pack(bindings_pack) = bindings_value else {
                    return Err("증명하기 해는 묶음이어야 합니다".to_string().into());
                };
                let bindings = solve_bindings_from_pack(bindings_pack)?;
                if let Some(work_cost) = work_cost {
                    work_cost
                        .borrow_mut()
                        .charge_solver_evaluations(relations.len() as u64)?;
                }
                let verified = ddonirang_symbolic::relation_system_holds(&relations, &bindings)
                    .map_err(|err| symbolic_formula_error("equiv", err))?;
                let mut fields = BTreeMap::new();
                fields.insert("검증".to_string(), Value::Bool(verified));
                fields.insert(
                    "종류".to_string(),
                    Value::String("relation_solve_consistency".to_string()),
                );
                fields.insert(
                    "관계수".to_string(),
                    Value::Fixed64(Fixed64::from_i64(relations.len() as i64)),
                );
                return Ok(fields);
            }
            if relations.len() != 1 {
                return Err("증명하기 relation proof는 단일 관계만 지원합니다"
                    .to_string()
                    .into());
            }
            if let Some(work_cost) = work_cost {
                work_cost.borrow_mut().charge_solver_evaluations(1)?;
            }
            let cert = ddonirang_symbolic::prove_equivalent(&relations[0].0, &relations[0].1)
                .map_err(|err| symbolic_formula_error("equiv", err))?;
            let cert_value = serde_json::to_value(&cert).map_err(|err| {
                EvalError::Message(format!("proof certificate serialize failed: {err}"))
            })?;
            let report = ddonirang_proof::verify_value(&cert_value)
                .map_err(|err| EvalError::Message(format!("proof verify failed: {err}")))?;
            let mut fields = BTreeMap::new();
            fields.insert(
                "검증".to_string(),
                Value::Bool(report.valid && cert.equivalent),
            );
            fields.insert(
                "종류".to_string(),
                Value::String("relation_equivalence".to_string()),
            );
            fields.insert("증거".to_string(), Value::String(cert.certificate_hash));
            fields.insert("왼쪽정본".to_string(), Value::String(cert.lhs_canonical));
            fields.insert("오른쪽정본".to_string(), Value::String(cert.rhs_canonical));
            return Ok(fields);
        }
    }

    let (left, right) = if values.len() == 2 {
        expect_two_formulas(values, "증명하기")?
    } else {
        if values.len() != 1 {
            return Err(EvalError::Message(
                "증명하기는 증명요청 묶음 1개를 받습니다".to_string(),
            ));
        }
        let Value::Pack(pack) = &values[0] else {
            return Err(EvalError::Message(
                "증명하기는 묶음 인자가 필요합니다".to_string(),
            ));
        };
        let left = find_pack_formula(pack, &["왼쪽", "left", "lhs"])
            .ok_or_else(|| EvalError::Message("증명하기 왼쪽 수식이 없습니다".to_string()))?;
        let right = find_pack_formula(pack, &["오른쪽", "right", "rhs"])
            .ok_or_else(|| EvalError::Message("증명하기 오른쪽 수식이 없습니다".to_string()))?;
        (left, right)
    };
    if !matches!(left.dialect, FormulaDialect::Ascii)
        || !matches!(right.dialect, FormulaDialect::Ascii)
    {
        return Err(EvalError::Message(
            "증명하기는 #ascii 수식만 지원합니다".to_string(),
        ));
    }
    let left_analysis = analyze_formula_for_transform(&left)?;
    let right_analysis = analyze_formula_for_transform(&right)?;
    let left_text = format_formula_expr(&left_analysis.expr, 0);
    let right_text = format_formula_expr(&right_analysis.expr, 0);
    if let Some(work_cost) = work_cost {
        work_cost.borrow_mut().charge_solver_evaluations(1)?;
    }
    let cert = ddonirang_symbolic::prove_equivalent(&left_text, &right_text)
        .map_err(|err| symbolic_formula_error("equiv", err))?;
    let cert_value = serde_json::to_value(&cert)
        .map_err(|err| EvalError::Message(format!("proof certificate serialize failed: {err}")))?;
    let report = ddonirang_proof::verify_value(&cert_value)
        .map_err(|err| EvalError::Message(format!("proof verify failed: {err}")))?;
    let mut fields = BTreeMap::new();
    fields.insert(
        "검증".to_string(),
        Value::Bool(report.valid && cert.equivalent),
    );
    fields.insert(
        "종류".to_string(),
        Value::String("symbolic_equivalence".to_string()),
    );
    fields.insert("증거".to_string(), Value::String(cert.certificate_hash));
    fields.insert("왼쪽정본".to_string(), Value::String(cert.lhs_canonical));
    fields.insert("오른쪽정본".to_string(), Value::String(cert.rhs_canonical));
    Ok(fields)
}

#[cfg(test)]
fn eval_symbolic_proof_tactic(values: &[Value]) -> Result<BTreeMap<String, Value>, EvalError> {
    eval_symbolic_proof_tactic_with_meter(values, None)
}

fn find_pack_value<'a>(pack: &'a BTreeMap<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(value) = pack.get(*key) {
            return Some(value);
        }
    }
    None
}

fn find_pack_formula(pack: &BTreeMap<String, Value>, keys: &[&str]) -> Option<Formula> {
    keys.iter().find_map(|key| match pack.get(*key) {
        Some(Value::Formula(formula)) => Some(formula.clone()),
        _ => None,
    })
}

fn solve_bindings_from_pack(
    pack: &BTreeMap<String, Value>,
) -> Result<BTreeMap<String, ddonirang_symbolic::SolveBinding>, EvalError> {
    let mut out = BTreeMap::new();
    for (name, value) in pack {
        out.insert(name.clone(), solve_binding_from_value(value)?);
    }
    Ok(out)
}

fn extract_relation_texts_from_value(value: &Value) -> Result<Vec<(String, String)>, EvalError> {
    match value {
        Value::Pack(_) => {
            let relation = expect_single_equation_relation(value)?;
            let left = match relation.get(RELATION_LEFT_FIELD) {
                Some(Value::Formula(formula)) => relation_formula_text(formula)?,
                _ => return Err("equation relation이 필요합니다".to_string().into()),
            };
            let right = match relation.get(RELATION_RIGHT_FIELD) {
                Some(Value::Formula(formula)) => relation_formula_text(formula)?,
                _ => return Err("equation relation이 필요합니다".to_string().into()),
            };
            Ok(vec![(left, right)])
        }
        Value::List(list) => {
            let mut out = Vec::new();
            for item in list {
                out.extend(extract_relation_texts_from_value(item)?);
            }
            Ok(out)
        }
        _ => Err("equation relation이 필요합니다".to_string().into()),
    }
}

fn diff_formula_expr(
    expr: &FormulaExpr,
    var: &str,
    label: &'static str,
) -> Result<FormulaExpr, EvalError> {
    use FormulaExpr::*;
    use FormulaOp::*;

    let d = match expr {
        Number(_) => num_zero(),
        Var(name) => {
            if name == var {
                num_one()
            } else {
                num_zero()
            }
        }
        Unary { op: Sub, expr } => unary_sub(diff_formula_expr(expr, var, label)?),
        Unary { expr, .. } => diff_formula_expr(expr, var, label)?,
        Binary {
            op: Add,
            left,
            right,
        } => add(
            diff_formula_expr(left, var, label)?,
            diff_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Sub,
            left,
            right,
        } => sub(
            diff_formula_expr(left, var, label)?,
            diff_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Mul,
            left,
            right,
        } => {
            let dl = diff_formula_expr(left, var, label)?;
            let dr = diff_formula_expr(right, var, label)?;
            add(
                mul(dl, right.as_ref().clone()),
                mul(left.as_ref().clone(), dr),
            )
        }
        Binary {
            op: Div,
            left,
            right,
        } => {
            let dl = diff_formula_expr(left, var, label)?;
            let dr = diff_formula_expr(right, var, label)?;
            let num = sub(
                mul(dl, right.as_ref().clone()),
                mul(left.as_ref().clone(), dr),
            );
            let denom = pow(right.as_ref().clone(), num_i(2));
            div(num, denom)
        }
        Binary {
            op: Pow,
            left,
            right,
        } => {
            if let Some(exp) = number_as_int(right) {
                if exp == 0 {
                    num_zero()
                } else {
                    let coeff = num_i(exp);
                    let pow_expr = pow(left.as_ref().clone(), num_i(exp - 1));
                    let dl = diff_formula_expr(left, var, label)?;
                    mul(coeff, mul(pow_expr, dl))
                }
            } else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 거듭제곱 미분을 지원하지 않습니다",
                    label
                )));
            }
        }
        Binary { op: Mod, .. } => {
            return Err(EvalError::Message(format!(
                "E_CALC_TRANSFORM_UNSUPPORTED: {}는 나머지 미분을 지원하지 않습니다",
                label
            )));
        }
        Func { name, args } => {
            let Some(first) = args.first() else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 빈 함수 미분을 지원하지 않습니다",
                    label
                )));
            };
            let du = diff_formula_expr(first, var, label)?;
            match name.as_str() {
                "sin" => mul(func("cos", vec![first.clone()]), du),
                "cos" => mul(unary_sub(func("sin", vec![first.clone()])), du),
                "tan" => {
                    let cos_u = func("cos", vec![first.clone()]);
                    let denom = pow(cos_u, num_i(2));
                    div(du, denom)
                }
                "exp" => mul(func("exp", vec![first.clone()]), du),
                "ln" => div(du, first.clone()),
                "log10" => {
                    let denom = mul(first.clone(), num_ln10()?);
                    div(du, denom)
                }
                "log2" => {
                    let denom = mul(first.clone(), num_ln2()?);
                    div(du, denom)
                }
                "sqrt" => {
                    let denom = mul(num_i(2), func("sqrt", vec![first.clone()]));
                    div(du, denom)
                }
                _ => {
                    return Err(EvalError::Message(format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED: {}는 함수 '{}' 미분을 지원하지 않습니다",
                        label, name
                    )))
                }
            }
        }
    };
    Ok(d)
}

fn integrate_formula_expr(
    expr: &FormulaExpr,
    var: &str,
    label: &'static str,
) -> Result<FormulaExpr, EvalError> {
    use FormulaExpr::*;
    use FormulaOp::*;

    if !expr_contains_var(expr, var) {
        return Ok(mul(expr.clone(), Var(var.to_string())));
    }

    let out = match expr {
        Number(_) => mul(expr.clone(), Var(var.to_string())),
        Var(name) => {
            if name == var {
                div(pow(Var(var.to_string()), num_i(2)), num_i(2))
            } else {
                mul(expr.clone(), Var(var.to_string()))
            }
        }
        Unary { op: Sub, expr } => unary_sub(integrate_formula_expr(expr, var, label)?),
        Unary { expr, .. } => integrate_formula_expr(expr, var, label)?,
        Binary {
            op: Add,
            left,
            right,
        } => add(
            integrate_formula_expr(left, var, label)?,
            integrate_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Sub,
            left,
            right,
        } => sub(
            integrate_formula_expr(left, var, label)?,
            integrate_formula_expr(right, var, label)?,
        ),
        Binary {
            op: Mul,
            left,
            right,
        } => {
            if !expr_contains_var(left, var) {
                mul(
                    left.as_ref().clone(),
                    integrate_formula_expr(right, var, label)?,
                )
            } else if !expr_contains_var(right, var) {
                mul(
                    right.as_ref().clone(),
                    integrate_formula_expr(left, var, label)?,
                )
            } else if let Some(exp) = detect_var_power(expr, var) {
                let next = exp + 1;
                let num = pow(Var(var.to_string()), num_i(next));
                div(num, num_i(next))
            } else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 곱셈 적분을 지원하지 않습니다",
                    label
                )));
            }
        }
        Binary {
            op: Pow,
            left,
            right,
        } => {
            if let (FormulaExpr::Var(name), Some(exp)) = (&**left, number_as_int(right)) {
                if name == var && exp != -1 {
                    let next = exp + 1;
                    let num = pow(Var(var.to_string()), num_i(next));
                    div(num, num_i(next))
                } else {
                    return Err(EvalError::Message(format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED: {}는 거듭제곱 적분을 지원하지 않습니다",
                        label
                    )));
                }
            } else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 거듭제곱 적분을 지원하지 않습니다",
                    label
                )));
            }
        }
        Binary { op: Div, .. } | Binary { op: Mod, .. } => {
            return Err(EvalError::Message(format!(
                "E_CALC_TRANSFORM_UNSUPPORTED: {}는 나눗셈 적분을 지원하지 않습니다",
                label
            )));
        }
        Func { name, args } => {
            let Some(first) = args.first() else {
                return Err(EvalError::Message(format!(
                    "E_CALC_TRANSFORM_UNSUPPORTED: {}는 빈 함수 적분을 지원하지 않습니다",
                    label
                )));
            };
            match name.as_str() {
                "sin" => unary_sub(func("cos", vec![first.clone()])),
                "cos" => func("sin", vec![first.clone()]),
                "exp" => func("exp", vec![first.clone()]),
                _ => {
                    return Err(EvalError::Message(format!(
                        "E_CALC_TRANSFORM_UNSUPPORTED: {}는 함수 '{}' 적분을 지원하지 않습니다",
                        label, name
                    )))
                }
            }
        }
    };
    Ok(out)
}

fn expr_contains_var(expr: &FormulaExpr, var: &str) -> bool {
    match expr {
        FormulaExpr::Var(name) => name == var,
        FormulaExpr::Number(_) => false,
        FormulaExpr::Func { args, .. } => args.iter().any(|arg| expr_contains_var(arg, var)),
        FormulaExpr::Unary { expr, .. } => expr_contains_var(expr, var),
        FormulaExpr::Binary { left, right, .. } => {
            expr_contains_var(left, var) || expr_contains_var(right, var)
        }
    }
}

fn detect_var_power(expr: &FormulaExpr, var: &str) -> Option<i64> {
    match expr {
        FormulaExpr::Var(name) if name == var => Some(1),
        FormulaExpr::Binary {
            op: FormulaOp::Pow,
            left,
            right,
        } => {
            if let FormulaExpr::Var(name) = &**left {
                if name == var {
                    return number_as_int(right);
                }
            }
            None
        }
        FormulaExpr::Binary {
            op: FormulaOp::Mul,
            left,
            right,
        } => {
            if let (Some(a), Some(b)) = (detect_var_power(left, var), detect_var_power(right, var))
            {
                return Some(a + b);
            }
            if let (Some(a), FormulaExpr::Var(name)) = (detect_var_power(left, var), &**right) {
                if name == var {
                    return Some(a + 1);
                }
            }
            if let (FormulaExpr::Var(name), Some(b)) = (&**left, detect_var_power(right, var)) {
                if name == var {
                    return Some(b + 1);
                }
            }
            None
        }
        _ => None,
    }
}

fn number_as_int(expr: &FormulaExpr) -> Option<i64> {
    match expr {
        FormulaExpr::Number(value) if value.dim == UnitDim::NONE => {
            if value.value.frac_part() == 0 {
                Some(value.value.int_part())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn num_zero() -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_i64(0),
        dim: UnitDim::NONE,
    })
}

fn num_one() -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_i64(1),
        dim: UnitDim::NONE,
    })
}

fn num_i(value: i64) -> FormulaExpr {
    FormulaExpr::Number(UnitValue {
        value: Fixed64::from_i64(value),
        dim: UnitDim::NONE,
    })
}

pub(super) fn num_ln10() -> Result<FormulaExpr, EvalError> {
    Ok(FormulaExpr::Number(UnitValue {
        value: fixed64_from_host_f64_checked(libm::log(10.0))?,
        dim: UnitDim::NONE,
    }))
}

pub(super) fn num_ln2() -> Result<FormulaExpr, EvalError> {
    Ok(FormulaExpr::Number(UnitValue {
        value: fixed64_from_host_f64_checked(libm::log(2.0))?,
        dim: UnitDim::NONE,
    }))
}

pub(super) fn func(name: &str, args: Vec<FormulaExpr>) -> FormulaExpr {
    FormulaExpr::Func {
        name: name.to_string(),
        args,
    }
}

fn unary_sub(expr: FormulaExpr) -> FormulaExpr {
    FormulaExpr::Unary {
        op: FormulaOp::Sub,
        expr: Box::new(expr),
    }
}

pub(super) fn add(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) {
        return right;
    }
    if is_zero(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.add(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Add,
        left: Box::new(left),
        right: Box::new(right),
    }
}

pub(super) fn sub(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.sub(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Sub,
        left: Box::new(left),
        right: Box::new(right),
    }
}

pub(super) fn mul(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) || is_zero(&right) {
        return num_zero();
    }
    if is_one(&left) {
        return right;
    }
    if is_one(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        return FormulaExpr::Number(l.mul(r));
    }
    FormulaExpr::Binary {
        op: FormulaOp::Mul,
        left: Box::new(left),
        right: Box::new(right),
    }
}

pub(super) fn div(left: FormulaExpr, right: FormulaExpr) -> FormulaExpr {
    if is_zero(&left) {
        return num_zero();
    }
    if is_one(&right) {
        return left;
    }
    if let (Some(l), Some(r)) = (as_number(&left), as_number(&right)) {
        if let Ok(out) = l.div(r) {
            return FormulaExpr::Number(out);
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Div,
        left: Box::new(left),
        right: Box::new(right),
    }
}

pub(super) fn pow(base: FormulaExpr, exp: FormulaExpr) -> FormulaExpr {
    if let Some(e) = number_as_int(&exp) {
        if e == 0 {
            return num_one();
        }
        if e == 1 {
            return base;
        }
    }
    FormulaExpr::Binary {
        op: FormulaOp::Pow,
        left: Box::new(base),
        right: Box::new(exp),
    }
}

fn as_number(expr: &FormulaExpr) -> Option<UnitValue> {
    if let FormulaExpr::Number(value) = expr {
        Some(*value)
    } else {
        None
    }
}

pub(super) fn is_zero(expr: &FormulaExpr) -> bool {
    matches!(expr, FormulaExpr::Number(value) if value.value.raw_i64() == 0)
}

fn is_one(expr: &FormulaExpr) -> bool {
    matches!(expr, FormulaExpr::Number(value) if value.dim == UnitDim::NONE && value.value.raw_i64() == Fixed64::from_i64(1).raw_i64())
}

fn infer_single_var(vars: &BTreeSet<String>, label: &'static str) -> Result<String, EvalError> {
    match vars.len() {
        1 => Ok(vars.iter().next().unwrap().to_string()),
        0 => Err(EvalError::Message(format!(
            "E_CALC_FREEVAR_AMBIGUOUS: {}는 변수 이름을 지정해야 합니다",
            label
        ))),
        _ => Err(EvalError::Message(format!(
            "E_CALC_FREEVAR_AMBIGUOUS: {} 변수 이름이 여러 개입니다",
            label
        ))),
    }
}

pub(super) fn ensure_formula_ident(
    name: &str,
    dialect: &FormulaDialect,
    label: &'static str,
) -> Result<(), EvalError> {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 비어 있습니다",
            label
        )));
    };
    if !first.is_ascii_alphabetic() {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 올바르지 않습니다: {}",
            label, name
        )));
    }
    let valid = match dialect {
        FormulaDialect::Ascii => chars.all(|ch| ch.is_ascii_alphanumeric()),
        FormulaDialect::Ascii1 => chars.all(|ch| ch.is_ascii_digit()),
        _ => false,
    };
    if !valid {
        return Err(EvalError::Message(format!(
            "{} 변수 이름이 올바르지 않습니다: {}",
            label, name
        )));
    }
    Ok(())
}

fn tokenize_formula(body: &str, dialect: &FormulaDialect) -> Result<Vec<FormulaToken>, EvalError> {
    let mut tokens = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        match ch {
            '0'..='9' => {
                let mut raw = String::new();
                let mut seen_dot = false;
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        raw.push(next);
                        chars.next();
                        continue;
                    }
                    if next == '.' && !seen_dot {
                        seen_dot = true;
                        raw.push(next);
                        chars.next();
                        continue;
                    }
                    break;
                }
                let value = Fixed64::parse_decimal(&raw)
                    .ok_or_else(|| EvalError::Message("FATAL:FORMULA_TOKEN_INVALID".to_string()))?;
                tokens.push(FormulaToken::Number(value));
            }
            'a'..='z' | 'A'..='Z' => {
                let ident = read_formula_ident(&mut chars, dialect)?;
                if matches!(dialect, FormulaDialect::Ascii1) {
                    tokens.extend(
                        split_ascii1_formula_ident(&ident)
                            .into_iter()
                            .map(FormulaToken::Ident),
                    );
                } else {
                    tokens.push(FormulaToken::Ident(ident));
                }
            }
            '+' => {
                chars.next();
                tokens.push(FormulaToken::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(FormulaToken::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(FormulaToken::Star);
            }
            '/' => {
                chars.next();
                tokens.push(FormulaToken::Slash);
            }
            '%' => {
                chars.next();
                tokens.push(FormulaToken::Percent);
            }
            ',' => {
                chars.next();
                tokens.push(FormulaToken::Comma);
            }
            '^' => {
                chars.next();
                tokens.push(FormulaToken::Caret);
            }
            '(' => {
                chars.next();
                tokens.push(FormulaToken::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(FormulaToken::RParen);
            }
            '=' => {
                chars.next();
                tokens.push(FormulaToken::Eq);
            }
            _ => {
                return Err(EvalError::Message(
                    "FATAL:FORMULA_TOKEN_INVALID".to_string(),
                ))
            }
        }
    }
    Ok(tokens)
}

fn read_formula_ident<I>(
    chars: &mut std::iter::Peekable<I>,
    dialect: &FormulaDialect,
) -> Result<String, EvalError>
where
    I: Iterator<Item = char>,
{
    let Some(first) = chars.next() else {
        return Err(EvalError::Message(
            "FATAL:FORMULA_TOKEN_INVALID".to_string(),
        ));
    };
    let mut ident = String::new();
    ident.push(first);
    if matches!(dialect, FormulaDialect::Ascii1) {
        while let Some(&next) = chars.peek() {
            if next.is_ascii_alphanumeric() {
                ident.push(next);
                chars.next();
                continue;
            }
            if next == '_' {
                return Err(EvalError::Message("FATAL:FORMULA_ASCII1_VAR".to_string()));
            }
            break;
        }
        return Ok(ident);
    }
    while let Some(&next) = chars.peek() {
        if next.is_ascii_alphanumeric() || next == '_' {
            ident.push(next);
            chars.next();
        } else {
            break;
        }
    }
    Ok(ident)
}

fn split_ascii1_formula_ident(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = name.chars().peekable();
    while let Some(first) = chars.next() {
        let mut ident = String::new();
        ident.push(first);
        while matches!(chars.peek(), Some(ch) if ch.is_ascii_digit()) {
            if let Some(ch) = chars.next() {
                ident.push(ch);
            }
        }
        out.push(ident);
    }
    out
}

fn find_top_level_eq(tokens: &[FormulaToken]) -> Result<Option<usize>, EvalError> {
    let mut depth = 0i32;
    let mut eq_index = None;
    for (idx, token) in tokens.iter().enumerate() {
        match token {
            FormulaToken::LParen => depth += 1,
            FormulaToken::RParen => depth -= 1,
            FormulaToken::Eq if depth == 0 => {
                if eq_index.is_some() {
                    return Err(EvalError::Message(
                        "FATAL:FORMULA_EQUATION_UNSUPPORTED".to_string(),
                    ));
                }
                eq_index = Some(idx);
            }
            _ => {}
        }
    }
    Ok(eq_index)
}

pub(super) fn eval_formula_expr(
    expr: &FormulaExpr,
    env: &BTreeMap<String, Value>,
) -> Result<UnitValue, EvalError> {
    match expr {
        FormulaExpr::Number(value) => Ok(*value),
        FormulaExpr::Var(name) => {
            if let Some(value) = env.get(name) {
                return unit_value_from_value(value);
            }
            if name == "pi" {
                return Ok(UnitValue {
                    value: fixed64_from_host_f64_checked(std::f64::consts::PI)?,
                    dim: UnitDim::NONE,
                });
            }
            if name == "e" {
                return Ok(UnitValue {
                    value: fixed64_from_host_f64_checked(std::f64::consts::E)?,
                    dim: UnitDim::NONE,
                });
            }
            Err(EvalError::Message(format!(
                "풀기: 키 '{}'가 없습니다",
                name
            )))
        }
        FormulaExpr::Func { name, args } => {
            let mut evaluated = Vec::with_capacity(args.len());
            for arg in args {
                evaluated.push(eval_formula_expr(arg, env)?);
            }
            eval_formula_func(name, evaluated)
        }
        FormulaExpr::Unary { op, expr } => {
            let value = eval_formula_expr(expr, env)?;
            match op {
                FormulaOp::Sub => Ok(UnitValue {
                    value: Fixed64::ZERO.checked_sub(value.value)?,
                    dim: value.dim,
                }),
                _ => Ok(value),
            }
        }
        FormulaExpr::Binary { op, left, right } => {
            let l = eval_formula_expr(left, env)?;
            let r = eval_formula_expr(right, env)?;
            match op {
                FormulaOp::Add => checked_unit_add(l, r),
                FormulaOp::Sub => checked_unit_sub(l, r),
                FormulaOp::Mul => checked_unit_mul(l, r),
                FormulaOp::Div => l.div(r).map_err(unit_error),
                FormulaOp::Mod => {
                    if l.dim != r.dim {
                        return Err(unit_error(UnitError::DimensionMismatch {
                            left: l.dim,
                            right: r.dim,
                        }));
                    }
                    let raw_r = r.value.raw_i64();
                    if raw_r == 0 {
                        return Err(EvalError::Message("FATAL:FORMULA_MOD_ZERO".to_string()));
                    }
                    Ok(UnitValue {
                        value: Fixed64::from_raw_i64(l.value.raw_i64().checked_rem(raw_r)
                            .ok_or(ddonirang_core::ArithmeticFaultKind::Overflow)?),
                        dim: l.dim,
                    })
                }
                FormulaOp::Pow => eval_formula_pow(l, r),
            }
        }
    }
}

pub(super) fn eval_formula_func(name: &str, args: Vec<UnitValue>) -> Result<UnitValue, EvalError> {
    let expect_args = |expected: usize| -> Result<Vec<UnitValue>, EvalError> {
        if args.len() != expected {
            return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
        }
        Ok(args.clone())
    };

    match name {
        "sin" | "cos" | "tan" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !(arg.is_dimensionless() || arg.dim == UnitDim::ANGLE) {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::ANGLE,
                }));
            }
            let angle = fixed64_to_f64(arg.value);
            let out = match name {
                "sin" => libm::sin(angle),
                "cos" => libm::cos(angle),
                _ => libm::tan(angle),
            };
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "asin" | "acos" | "atan" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "asin" => libm::asin(value),
                "acos" => libm::acos(value),
                _ => libm::atan(value),
            };
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "atan2" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let y = args[0];
            let x = args[1];
            if !y.is_dimensionless() || !x.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: y.dim,
                    right: UnitDim::NONE,
                }));
            }
            let out = libm::atan2(fixed64_to_f64(y.value), fixed64_to_f64(x.value));
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "sinh" | "cosh" | "tanh" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "sinh" => libm::sinh(value),
                "cosh" => libm::cosh(value),
                _ => libm::tanh(value),
            };
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "asinh" | "acosh" | "atanh" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "asinh" => libm::asinh(value),
                "acosh" => libm::acosh(value),
                _ => libm::atanh(value),
            };
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "abs" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_abs(arg.value)?,
                dim: arg.dim,
            });
        }
        "sign" => {
            let args = expect_args(1)?;
            let arg = args[0];
            let raw = arg.value.raw_i64();
            let sign = if raw > 0 {
                1
            } else if raw < 0 {
                -1
            } else {
                0
            };
            return Ok(UnitValue {
                value: Fixed64::from_i64(sign),
                dim: UnitDim::NONE,
            });
        }
        "floor" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_floor(arg.value),
                dim: arg.dim,
            });
        }
        "ceil" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_ceil(arg.value)?,
                dim: arg.dim,
            });
        }
        "round" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: fixed64_round_even(arg.value)?,
                dim: arg.dim,
            });
        }
        "trunc" => {
            let args = expect_args(1)?;
            let arg = args[0];
            return Ok(UnitValue {
                value: Fixed64::from_i64(arg.value.int_part()),
                dim: arg.dim,
            });
        }
        "fract" => {
            let args = expect_args(1)?;
            let arg = args[0];
            let floor = fixed64_floor(arg.value);
            let frac = arg.value - floor;
            return Ok(UnitValue {
                value: frac,
                dim: arg.dim,
            });
        }
        "sqrt" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let Some(value) = fixed64_sqrt(arg.value) else {
                return Err(EvalError::Message("FATAL:FORMULA_SQRT_INVALID".to_string()));
            };
            return Ok(UnitValue {
                value,
                dim: UnitDim::NONE,
            });
        }
        "cbrt" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(libm::cbrt(value))?,
                dim: UnitDim::NONE,
            });
        }
        "powi" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let base = args[0];
            let exp = args[1];
            if !exp.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: exp.dim,
                    right: UnitDim::NONE,
                }));
            }
            let exp_i = exp.value.int_part();
            return unit_powi(base, exp_i);
        }
        "pow" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let base = args[0];
            let exp = args[1];
            if !exp.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: exp.dim,
                    right: UnitDim::NONE,
                }));
            }
            if exp.value.frac_part() == 0 {
                return unit_powi(base, exp.value.int_part());
            }
            if !base.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: base.dim,
                    right: UnitDim::NONE,
                }));
            }
            let out = libm::pow(fixed64_to_f64(base.value), fixed64_to_f64(exp.value));
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "exp" | "ln" | "log10" | "log2" => {
            let args = expect_args(1)?;
            let arg = args[0];
            if !arg.is_dimensionless() {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: arg.dim,
                    right: UnitDim::NONE,
                }));
            }
            let value = fixed64_to_f64(arg.value);
            let out = match name {
                "exp" => libm::exp(value),
                "ln" => libm::log(value),
                "log10" => libm::log10(value),
                _ => libm::log2(value),
            };
            return Ok(UnitValue {
                value: fixed64_from_host_f64_checked(out)?,
                dim: UnitDim::NONE,
            });
        }
        "min" | "max" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let a = args[0];
            let b = args[1];
            if a.dim != b.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: a.dim,
                    right: b.dim,
                }));
            }
            let pick = if name == "min" {
                if a.value <= b.value {
                    a
                } else {
                    b
                }
            } else if a.value >= b.value {
                a
            } else {
                b
            };
            return Ok(pick);
        }
        "clamp" => {
            if args.len() != 3 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let x = args[0];
            let lo = args[1];
            let hi = args[2];
            if x.dim != lo.dim || x.dim != hi.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: x.dim,
                    right: lo.dim,
                }));
            }
            let mut out = x;
            if out.value < lo.value {
                out = lo;
            }
            if out.value > hi.value {
                out = hi;
            }
            return Ok(out);
        }
        "mod" => {
            if args.len() != 2 {
                return Err(EvalError::Message("FATAL:FORMULA_FUNC_ARG".to_string()));
            }
            let a = args[0];
            let b = args[1];
            if a.dim != b.dim {
                return Err(unit_error(UnitError::DimensionMismatch {
                    left: a.dim,
                    right: b.dim,
                }));
            }
            let raw_b = b.value.raw_i64();
            if raw_b == 0 {
                return Err(EvalError::Message("FATAL:FORMULA_MOD_ZERO".to_string()));
            }
            let raw = a.value.raw_i64().checked_rem(raw_b)
                .ok_or(ddonirang_core::ArithmeticFaultKind::Overflow)?;
            return Ok(UnitValue {
                value: Fixed64::from_raw_i64(raw),
                dim: a.dim,
            });
        }
        _ => {}
    }
    Err(EvalError::Message(format!(
        "FATAL:FORMULA_FUNC_UNKNOWN:{}",
        name
    )))
}

pub(super) fn eval_formula_pow(base: UnitValue, exponent: UnitValue) -> Result<UnitValue, EvalError> {
    if exponent.dim != UnitDim::NONE {
        return Err(EvalError::Message("FATAL:FORMULA_POW_INVALID".to_string()));
    }
    if exponent.value.frac_part() != 0 {
        return Err(EvalError::Message("FATAL:FORMULA_POW_INVALID".to_string()));
    }
    let exp = exponent.value.int_part();
    if exp < 0 {
        return Err(EvalError::Message("FATAL:FORMULA_POW_INVALID".to_string()));
    }
    if exp == 0 {
        return Ok(UnitValue {
            value: Fixed64::from_i64(1),
            dim: UnitDim::NONE,
        });
    }
    let mut result = base;
    for _ in 1..exp {
        result = UnitValue {
            value: result.value.checked_mul(base.value)?,
            dim: result.dim.add(base.dim),
        };
    }
    Ok(result)
}

#[cfg(test)]
mod remainder_panic_regression {
    use super::*;

    fn quantity(raw: i64) -> UnitValue {
        UnitValue { value: Fixed64::from_raw_i64(raw), dim: UnitDim::NONE }
    }

    fn operator(left: i64, right: i64) -> Result<UnitValue, EvalError> {
        eval_formula_expr(&FormulaExpr::Binary {
            op: FormulaOp::Mod,
            left: Box::new(FormulaExpr::Number(quantity(left))),
            right: Box::new(FormulaExpr::Number(quantity(right))),
        }, &BTreeMap::new())
    }

    #[test]
    fn minimum_raw_remainder_rejects_with_existing_typed_overflow() {
        for result in [operator(i64::MIN, -1),
            eval_formula_func("mod", vec![quantity(i64::MIN), quantity(-1)])] {
            assert!(matches!(result, Err(EvalError::ArithmeticFault(
                ddonirang_core::ArithmeticFaultKind::Overflow))));
        }
    }

    #[test]
    fn ordinary_raw_remainders_and_existing_zero_errors_are_preserved() {
        for (left, right) in [(5, 2), (-5, 2), (5, -2), (-5, -2), (i64::MIN, 1)] {
            assert_eq!(operator(left, right).unwrap().value.raw_i64(), left % right);
            assert_eq!(eval_formula_func("mod", vec![quantity(left), quantity(right)])
                .unwrap().value.raw_i64(), left % right);
        }
        for result in [operator(1, 0), eval_formula_func("mod", vec![quantity(1), quantity(0)])] {
            assert!(matches!(result, Err(EvalError::Message(ref text))
                if text == "FATAL:FORMULA_MOD_ZERO"));
        }
    }
}
