use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ddonirang_core::Fixed64;
use serde::Deserialize;
use serde_json::Value as JsonValue;

use super::detjson::write_text;

#[derive(Deserialize)]
struct RewardCheckInput {
    schema: Option<String>,
    script: String,
    cases: Vec<RewardCaseInput>,
}

#[derive(Deserialize)]
struct RewardCaseInput {
    realm_id: u64,
    step: u64,
    vars: BTreeMap<String, JsonValue>,
}

#[derive(Clone)]
struct RewardCase {
    realm_id: u64,
    step: u64,
    vars: BTreeMap<String, Fixed64>,
}

#[derive(Clone)]
struct RewardEntry {
    realm_id: u64,
    step: u64,
    reward: Fixed64,
}

#[derive(Clone)]
struct RealmSummary {
    step_count: u64,
    total: Fixed64,
}

pub fn run_reward_check(input: &Path, out_dir: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(input)
        .map_err(|e| format!("E_REWARD_INPUT_READ {} {}", input.display(), e))?;
    let input: RewardCheckInput =
        serde_json::from_str(&text).map_err(|e| format!("E_REWARD_INPUT_PARSE {}", e))?;
    if let Some(schema) = input.schema.as_deref() {
        if schema != "reward.check.v1" {
            return Err(format!("E_REWARD_SCHEMA {}", schema));
        }
    }

    let expr = RewardExpr::parse(&input.script)?;
    let mut cases = Vec::with_capacity(input.cases.len());
    for (idx, item) in input.cases.into_iter().enumerate() {
        let mut vars = BTreeMap::new();
        for (key, value) in item.vars {
            let fixed = parse_fixed64_value(&value)
                .map_err(|e| format!("E_REWARD_VAR {} {}", key, e))?;
            vars.insert(key, fixed);
        }
        cases.push((idx, RewardCase {
            realm_id: item.realm_id,
            step: item.step,
            vars,
        }));
    }

    cases.sort_by(|(a_idx, a), (b_idx, b)| {
        (a.realm_id, a.step, *a_idx).cmp(&(b.realm_id, b.step, *b_idx))
    });

    let mut entries = Vec::with_capacity(cases.len());
    for (_, case) in cases {
        let value = expr.eval(&case.vars)?;
        let reward = match value {
            RewardValue::Num(value) => value,
            RewardValue::Bool(_) => {
                return Err("E_REWARD_RESULT_TYPE 보상식 결과는 수여야 합니다".to_string());
            }
        };
        entries.push(RewardEntry {
            realm_id: case.realm_id,
            step: case.step,
            reward,
        });
    }

    let mut summaries: BTreeMap<u64, RealmSummary> = BTreeMap::new();
    let mut total = Fixed64::ZERO;
    for entry in &entries {
        let summary = summaries
            .entry(entry.realm_id)
            .or_insert(RealmSummary {
                step_count: 0,
                total: Fixed64::ZERO,
            });
        summary.step_count = summary.step_count.saturating_add(1);
        summary.total = summary.total.saturating_add(entry.reward);
        total = total.saturating_add(entry.reward);
    }

    let log_detjson = build_reward_log(&entries);
    let report_detjson = build_reward_report(&summaries, total);

    if let Some(out_dir) = out_dir {
        fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
        let log_path = out_dir.join("reward.log.detjson");
        let report_path = out_dir.join("reward.report.detjson");
        write_text(&log_path, &log_detjson)?;
        write_text(&report_path, &report_detjson)?;
    }

    println!("{}", log_detjson);
    println!("{}", report_detjson);
    Ok(())
}

fn build_reward_log(entries: &[RewardEntry]) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"reward.log.v1\",\"entries\":[");
    for (idx, entry) in entries.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("{\"realm_id\":");
        out.push_str(&entry.realm_id.to_string());
        out.push_str(",\"step\":");
        out.push_str(&entry.step.to_string());
        out.push_str(",\"reward_raw\":\"");
        out.push_str(&fixed64_raw_string(entry.reward));
        out.push_str("\"}");
    }
    out.push_str("]}");
    out
}

fn build_reward_report(summaries: &BTreeMap<u64, RealmSummary>, total: Fixed64) -> String {
    let mut out = String::new();
    out.push_str("{\"schema\":\"reward.report.v1\",\"realm_count\":");
    out.push_str(&summaries.len().to_string());
    out.push_str(",\"total_raw\":\"");
    out.push_str(&fixed64_raw_string(total));
    out.push_str("\",\"realms\":[");
    let mut first = true;
    for (realm_id, summary) in summaries {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str("{\"realm_id\":");
        out.push_str(&realm_id.to_string());
        out.push_str(",\"step_count\":");
        out.push_str(&summary.step_count.to_string());
        out.push_str(",\"total_raw\":\"");
        out.push_str(&fixed64_raw_string(summary.total));
        out.push_str("\"}");
    }
    out.push_str("]}");
    out
}

fn fixed64_raw_string(value: Fixed64) -> String {
    value.raw_i64().to_string()
}

fn parse_fixed64_value(value: &JsonValue) -> Result<Fixed64, String> {
    match value {
        JsonValue::String(text) => parse_fixed64_string(text),
        _ => Err("보상 변수 값은 문자열이어야 합니다".to_string()),
    }
}

fn parse_fixed64_string(input: &str) -> Result<Fixed64, String> {
    let trimmed = input.trim();
    if let Some(raw) = trimmed.strip_prefix("raw:") {
        let raw_value = raw
            .trim()
            .parse::<i64>()
            .map_err(|_| format!("Fixed64 raw 변환 실패: {}", input))?;
        return Ok(Fixed64::from_raw_i64(raw_value));
    }
    parse_fixed64_decimal(trimmed)
}

fn parse_fixed64_decimal(input: &str) -> Result<Fixed64, String> {
    let text = input.trim();
    if text.is_empty() {
        return Err("Fixed64 문자열이 비었습니다".to_string());
    }
    let mut sign = 1i128;
    let mut raw_text = text;
    if let Some(rest) = raw_text.strip_prefix('-') {
        sign = -1;
        raw_text = rest;
    } else if let Some(rest) = raw_text.strip_prefix('+') {
        raw_text = rest;
    }
    let mut parts = raw_text.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    let frac_part = parts.next().unwrap_or("");

    let int_value = if int_part.is_empty() {
        0i128
    } else {
        if !int_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Fixed64 정수부 형식 오류: {}", input));
        }
        int_part
            .parse::<i128>()
            .map_err(|_| format!("Fixed64 정수부 변환 실패: {}", input))?
    };

    let frac_value = if frac_part.is_empty() {
        0i128
    } else {
        if !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Fixed64 소수부 형식 오류: {}", input));
        }
        frac_part
            .parse::<i128>()
            .map_err(|_| format!("Fixed64 소수부 변환 실패: {}", input))?
    };

    let scale = 10i128.pow(frac_part.len() as u32);
    let frac_raw = if frac_part.is_empty() {
        0i128
    } else {
        (frac_value * (1i128 << 32)) / scale
    };

    let raw = (int_value << 32) + frac_raw;
    let signed = raw.saturating_mul(sign);
    let clamped = clamp_i128_to_i64(signed);
    Ok(Fixed64::from_raw_i64(clamped))
}

fn clamp_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

#[derive(Clone, Debug)]
enum RewardExpr {
    Number(Fixed64),
    Var(String),
    Unary(UnaryOp, Box<RewardExpr>),
    Binary(BinaryOp, Box<RewardExpr>, Box<RewardExpr>),
    Compare(CompareOp, Box<RewardExpr>, Box<RewardExpr>),
    Call(String, Vec<RewardExpr>),
}

#[derive(Clone, Debug)]
enum UnaryOp {
    Plus,
    Minus,
}

#[derive(Clone, Debug)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug)]
enum RewardValue {
    Num(Fixed64),
    Bool(bool),
}

impl RewardExpr {
    fn parse(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = RewardParser::new(tokens);
        let expr = parser.parse_expr()?;
        parser.expect(Token::Eof)?;
        Ok(expr)
    }

    fn eval(&self, vars: &BTreeMap<String, Fixed64>) -> Result<RewardValue, String> {
        match self {
            RewardExpr::Number(value) => Ok(RewardValue::Num(*value)),
            RewardExpr::Var(name) => vars
                .get(name)
                .copied()
                .map(RewardValue::Num)
                .ok_or_else(|| format!("E_REWARD_VAR_MISSING {}", name)),
            RewardExpr::Unary(op, expr) => {
                let value = expr.eval(vars)?;
                let num = match value {
                    RewardValue::Num(num) => num,
                    RewardValue::Bool(_) => {
                        return Err("E_REWARD_TYPE 단항 연산은 수에만 허용됩니다".to_string());
                    }
                };
                match op {
                    UnaryOp::Plus => Ok(RewardValue::Num(num)),
                    UnaryOp::Minus => Ok(RewardValue::Num(-num)),
                }
            }
            RewardExpr::Binary(op, left, right) => {
                let lhs = left.eval(vars)?;
                let rhs = right.eval(vars)?;
                let (lhs, rhs) = match (lhs, rhs) {
                    (RewardValue::Num(lhs), RewardValue::Num(rhs)) => (lhs, rhs),
                    _ => {
                        return Err("E_REWARD_TYPE 산술 연산은 수에만 허용됩니다".to_string());
                    }
                };
                let value = match op {
                    BinaryOp::Add => lhs + rhs,
                    BinaryOp::Sub => lhs - rhs,
                    BinaryOp::Mul => lhs * rhs,
                    BinaryOp::Div => lhs.try_div(rhs).map_err(|_| {
                        "E_REWARD_DIV0 보상식에서 0으로 나눌 수 없습니다".to_string()
                    })?,
                };
                Ok(RewardValue::Num(value))
            }
            RewardExpr::Compare(op, left, right) => {
                let lhs = left.eval(vars)?;
                let rhs = right.eval(vars)?;
                let (lhs, rhs) = match (lhs, rhs) {
                    (RewardValue::Num(lhs), RewardValue::Num(rhs)) => (lhs, rhs),
                    _ => {
                        return Err("E_REWARD_TYPE 비교 연산은 수에만 허용됩니다".to_string());
                    }
                };
                let result = match op {
                    CompareOp::Eq => lhs == rhs,
                    CompareOp::Ne => lhs != rhs,
                    CompareOp::Lt => lhs < rhs,
                    CompareOp::Le => lhs <= rhs,
                    CompareOp::Gt => lhs > rhs,
                    CompareOp::Ge => lhs >= rhs,
                };
                Ok(RewardValue::Bool(result))
            }
            RewardExpr::Call(name, args) => match name.as_str() {
                "min" => {
                    if args.len() != 2 {
                        return Err("E_REWARD_CALL min은 인자 2개가 필요합니다".to_string());
                    }
                    let a = value_as_num(args[0].eval(vars)?)?;
                    let b = value_as_num(args[1].eval(vars)?)?;
                    Ok(RewardValue::Num(if a <= b { a } else { b }))
                }
                "max" => {
                    if args.len() != 2 {
                        return Err("E_REWARD_CALL max는 인자 2개가 필요합니다".to_string());
                    }
                    let a = value_as_num(args[0].eval(vars)?)?;
                    let b = value_as_num(args[1].eval(vars)?)?;
                    Ok(RewardValue::Num(if a >= b { a } else { b }))
                }
                "clamp" => {
                    if args.len() != 3 {
                        return Err("E_REWARD_CALL clamp는 인자 3개가 필요합니다".to_string());
                    }
                    let value = value_as_num(args[0].eval(vars)?)?;
                    let min = value_as_num(args[1].eval(vars)?)?;
                    let max = value_as_num(args[2].eval(vars)?)?;
                    if min > max {
                        return Err("E_REWARD_CALL clamp의 min은 max 이하여야 합니다".to_string());
                    }
                    let clamped = if value < min {
                        min
                    } else if value > max {
                        max
                    } else {
                        value
                    };
                    Ok(RewardValue::Num(clamped))
                }
                "if" => {
                    if args.len() != 3 {
                        return Err("E_REWARD_CALL if는 인자 3개가 필요합니다".to_string());
                    }
                    let cond_value = args[0].eval(vars)?;
                    let cond = match cond_value {
                        RewardValue::Bool(value) => value,
                        RewardValue::Num(value) => value.raw_i64() != 0,
                    };
                    if cond {
                        Ok(RewardValue::Num(value_as_num(args[1].eval(vars)?)?))
                    } else {
                        Ok(RewardValue::Num(value_as_num(args[2].eval(vars)?)?))
                    }
                }
                _ => Err(format!("E_REWARD_CALL_UNKNOWN {}", name)),
            },
        }
    }
}

fn value_as_num(value: RewardValue) -> Result<Fixed64, String> {
    match value {
        RewardValue::Num(num) => Ok(num),
        RewardValue::Bool(_) => Err("E_REWARD_TYPE 수식 인자는 수여야 합니다".to_string()),
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(String),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Eof,
}

struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    input: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            input,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut out = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                out.push(token);
                break;
            }
            out.push(token);
        }
        Ok(out)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_ws();
        if self.pos >= self.chars.len() {
            return Ok(Token::Eof);
        }
        let ch = self.chars[self.pos];
        match ch {
            '+' => {
                self.pos += 1;
                Ok(Token::Plus)
            }
            '-' => {
                self.pos += 1;
                Ok(Token::Minus)
            }
            '*' => {
                self.pos += 1;
                Ok(Token::Star)
            }
            '/' => {
                self.pos += 1;
                Ok(Token::Slash)
            }
            '(' => {
                self.pos += 1;
                Ok(Token::LParen)
            }
            ')' => {
                self.pos += 1;
                Ok(Token::RParen)
            }
            ',' => {
                self.pos += 1;
                Ok(Token::Comma)
            }
            '=' => {
                if self.peek_next('=') {
                    self.pos += 2;
                    Ok(Token::EqEq)
                } else {
                    Err(self.error_at("= 는 == 비교로만 허용됩니다"))
                }
            }
            '!' => {
                if self.peek_next('=') {
                    self.pos += 2;
                    Ok(Token::NotEq)
                } else {
                    Err(self.error_at("! 는 != 비교로만 허용됩니다"))
                }
            }
            '<' => {
                if self.peek_next('=') {
                    self.pos += 2;
                    Ok(Token::Le)
                } else {
                    self.pos += 1;
                    Ok(Token::Lt)
                }
            }
            '>' => {
                if self.peek_next('=') {
                    self.pos += 2;
                    Ok(Token::Ge)
                } else {
                    self.pos += 1;
                    Ok(Token::Gt)
                }
            }
            _ => {
                if ch.is_ascii_digit() {
                    self.read_number()
                } else if ch.is_ascii_alphabetic() || ch == '_' {
                    self.read_ident()
                } else {
                    Err(self.error_at("허용되지 않은 문자입니다"))
                }
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let start = self.pos;
        let mut seen_dot = false;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_digit() {
                self.pos += 1;
                continue;
            }
            if ch == '.' && !seen_dot {
                seen_dot = true;
                self.pos += 1;
                continue;
            }
            break;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        Ok(Token::Number(text))
    }

    fn read_ident(&mut self) -> Result<Token, String> {
        let start = self.pos;
        self.pos += 1;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.pos += 1;
                continue;
            }
            break;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        Ok(Token::Ident(text))
    }

    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() {
            if self.chars[self.pos].is_whitespace() {
                self.pos += 1;
                continue;
            }
            break;
        }
    }

    fn peek_next(&self, expected: char) -> bool {
        self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == expected
    }

    fn error_at(&self, message: &str) -> String {
        format!("E_REWARD_LEX {}: {}", self.input, message)
    }
}

struct RewardParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl RewardParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_expr(&mut self) -> Result<RewardExpr, String> {
        self.parse_compare()
    }

    fn parse_compare(&mut self) -> Result<RewardExpr, String> {
        let mut expr = self.parse_add()?;
        let op = match self.peek() {
            Token::EqEq => Some(CompareOp::Eq),
            Token::NotEq => Some(CompareOp::Ne),
            Token::Lt => Some(CompareOp::Lt),
            Token::Le => Some(CompareOp::Le),
            Token::Gt => Some(CompareOp::Gt),
            Token::Ge => Some(CompareOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.next();
            let rhs = self.parse_add()?;
            expr = RewardExpr::Compare(op, Box::new(expr), Box::new(rhs));
            if matches!(
                self.peek(),
                Token::EqEq | Token::NotEq | Token::Lt | Token::Le | Token::Gt | Token::Ge
            ) {
                return Err("E_REWARD_PARSE 비교 연산자는 연쇄로 사용할 수 없습니다".to_string());
            }
        }
        Ok(expr)
    }

    fn parse_add(&mut self) -> Result<RewardExpr, String> {
        let mut expr = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus => Some(BinaryOp::Add),
                Token::Minus => Some(BinaryOp::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.next();
                let rhs = self.parse_mul()?;
                expr = RewardExpr::Binary(op, Box::new(expr), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_mul(&mut self) -> Result<RewardExpr, String> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => Some(BinaryOp::Mul),
                Token::Slash => Some(BinaryOp::Div),
                _ => None,
            };
            if let Some(op) = op {
                self.next();
                let rhs = self.parse_unary()?;
                expr = RewardExpr::Binary(op, Box::new(expr), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<RewardExpr, String> {
        match self.peek() {
            Token::Plus => {
                self.next();
                let expr = self.parse_unary()?;
                Ok(RewardExpr::Unary(UnaryOp::Plus, Box::new(expr)))
            }
            Token::Minus => {
                self.next();
                let expr = self.parse_unary()?;
                Ok(RewardExpr::Unary(UnaryOp::Minus, Box::new(expr)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<RewardExpr, String> {
        match self.next() {
            Token::Number(text) => {
                let value = parse_fixed64_decimal(&text)
                    .map_err(|e| format!("E_REWARD_PARSE {}", e))?;
                Ok(RewardExpr::Number(value))
            }
            Token::Ident(name) => {
                if matches!(self.peek(), Token::LParen) {
                    self.next();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        loop {
                            let expr = self.parse_expr()?;
                            args.push(expr);
                            if matches!(self.peek(), Token::Comma) {
                                self.next();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(RewardExpr::Call(name, args))
                } else {
                    Ok(RewardExpr::Var(name))
                }
            }
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            token => Err(format!("E_REWARD_PARSE 예상치 못한 토큰 {:?}", token)),
        }
    }

    fn next(&mut self) -> Token {
        let token = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof)
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let token = self.next();
        if token == expected {
            Ok(())
        } else {
            Err(format!(
                "E_REWARD_PARSE 예상 {:?} 실제 {:?}",
                expected, token
            ))
        }
    }
}
