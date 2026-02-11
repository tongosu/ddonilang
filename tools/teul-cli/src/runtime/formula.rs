use std::collections::{BTreeMap, BTreeSet};

use crate::core::fixed64::Fixed64;
use crate::core::value::Quantity;
use crate::core::unit::UnitDim;
use crate::lang::ast::FormulaDialect;

#[derive(Debug)]
pub enum FormulaError {
    Parse(String),
    Undefined(String),
    ExtUnsupported { name: String },
    #[allow(dead_code)]
    IdentNotAscii1,
    UnitMismatch,
    DivZero,
}

#[derive(Clone, Debug)]
enum Token {
    Number(Fixed64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Equal,
    LParen,
    RParen,
    Comma,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, Debug)]
enum Expr {
    Number(Fixed64),
    Var(String),
    Call { name: String, args: Vec<Expr> },
    UnaryNeg(Box<Expr>),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Clone, Debug)]
enum FormulaAst {
    Expr(Expr),
    Assign { name: String, expr: Expr },
}

pub fn format_formula_body(body: &str, dialect: FormulaDialect) -> Result<String, FormulaError> {
    let ast = parse_formula(body, dialect)?;
    Ok(format_formula(&ast))
}

pub struct FormulaAnalysis {
    pub assign_name: Option<String>,
    pub expr_text: String,
    pub vars: BTreeSet<String>,
}

pub fn analyze_formula(
    body: &str,
    dialect: FormulaDialect,
) -> Result<FormulaAnalysis, FormulaError> {
    let ast = parse_formula(body, dialect)?;
    let (assign_name, expr) = match ast {
        FormulaAst::Expr(expr) => (None, expr),
        FormulaAst::Assign { name, expr } => (Some(name), expr),
    };
    let expr_text = format_expr(&expr, 0);
    let mut vars = BTreeSet::new();
    collect_vars(&expr, &mut vars);
    Ok(FormulaAnalysis {
        assign_name,
        expr_text,
        vars,
    })
}

pub fn eval_formula_body(
    body: &str,
    dialect: FormulaDialect,
    bindings: &BTreeMap<String, Quantity>,
) -> Result<Quantity, FormulaError> {
    let ast = parse_formula(body, dialect)?;
    let expr = match ast {
        FormulaAst::Expr(expr) => expr,
        FormulaAst::Assign { expr, .. } => expr,
    };
    eval_expr(&expr, bindings)
}

fn parse_formula(body: &str, dialect: FormulaDialect) -> Result<FormulaAst, FormulaError> {
    let tokens = FormulaLexer::new(body, dialect).tokenize()?;
    let mut parser = FormulaParser::new(tokens, dialect);
    let ast = parser.parse_assignment()?;
    parser.expect_end()?;
    Ok(ast)
}

fn format_formula(ast: &FormulaAst) -> String {
    match ast {
        FormulaAst::Expr(expr) => format_expr(expr, 0),
        FormulaAst::Assign { name, expr } => format!("{} = {}", name, format_expr(expr, 0)),
    }
}

fn format_expr(expr: &Expr, parent_prec: u8) -> String {
    match expr {
        Expr::Number(value) => value.format(),
        Expr::Var(name) => name.clone(),
        Expr::Call { name, args } => {
            let rendered = args
                .iter()
                .map(|arg| format_expr(arg, 0))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, rendered)
        }
        Expr::UnaryNeg(inner) => {
            let rendered = format_expr(inner, 3);
            format!("-{}", rendered)
        }
        Expr::Binary { op, left, right } => {
            let (prec, op_str, spaced, right_assoc) = match op {
                BinOp::Add => (1, "+", true, false),
                BinOp::Sub => (1, "-", true, false),
                BinOp::Mul => (2, "*", false, false),
                BinOp::Div => (2, "/", false, false),
                BinOp::Pow => (3, "^", false, true),
            };
            let (left_prec, right_prec) = if right_assoc {
                (prec + 1, prec)
            } else {
                (prec, prec + 1)
            };
            let left_str = format_expr(left, left_prec);
            let right_str = format_expr(right, right_prec);
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

fn collect_vars(expr: &Expr, vars: &mut BTreeSet<String>) {
    match expr {
        Expr::Var(name) => {
            vars.insert(name.clone());
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_vars(arg, vars);
            }
        }
        Expr::UnaryNeg(inner) => collect_vars(inner, vars),
        Expr::Binary { left, right, .. } => {
            collect_vars(left, vars);
            collect_vars(right, vars);
        }
        Expr::Number(_) => {}
    }
}

fn eval_expr(expr: &Expr, bindings: &BTreeMap<String, Quantity>) -> Result<Quantity, FormulaError> {
    match expr {
        Expr::Number(value) => Ok(Quantity::new(*value, UnitDim::zero())),
        Expr::Var(name) => bindings
            .get(name)
            .cloned()
            .ok_or_else(|| FormulaError::Undefined(name.clone())),
        Expr::Call { name, .. } => Err(FormulaError::ExtUnsupported { name: name.clone() }),
        Expr::UnaryNeg(inner) => {
            let value = eval_expr(inner, bindings)?;
            Ok(Quantity::new(
                Fixed64::from_raw(value.raw.raw().saturating_neg()),
                value.dim,
            ))
        }
        Expr::Binary { op, left, right } => {
            let left_val = eval_expr(left, bindings)?;
            let right_val = eval_expr(right, bindings)?;
            match op {
                BinOp::Add => {
                    if left_val.dim != right_val.dim {
                        return Err(FormulaError::UnitMismatch);
                    }
                    Ok(Quantity::new(
                        left_val.raw.saturating_add(right_val.raw),
                        left_val.dim,
                    ))
                }
                BinOp::Sub => {
                    if left_val.dim != right_val.dim {
                        return Err(FormulaError::UnitMismatch);
                    }
                    Ok(Quantity::new(
                        left_val.raw.saturating_sub(right_val.raw),
                        left_val.dim,
                    ))
                }
                BinOp::Mul => Ok(Quantity::new(
                    left_val.raw.saturating_mul(right_val.raw),
                    left_val.dim.add(right_val.dim),
                )),
                BinOp::Div => {
                    let raw = left_val
                        .raw
                        .checked_div(right_val.raw)
                        .ok_or(FormulaError::DivZero)?;
                    Ok(Quantity::new(raw, left_val.dim.add(right_val.dim.scale(-1))))
                }
                BinOp::Pow => {
                    if !right_val.dim.is_dimensionless() {
                        return Err(FormulaError::UnitMismatch);
                    }
                    let exp = fixed64_to_i32_exponent(right_val.raw)?;
                    if exp < 0 && left_val.raw.raw() == 0 {
                        return Err(FormulaError::DivZero);
                    }
                    let raw = left_val.raw.powi(exp);
                    let dim = left_val.dim.scale(exp);
                    Ok(Quantity::new(raw, dim))
                }
            }
        }
    }
}

fn fixed64_to_i32_exponent(value: Fixed64) -> Result<i32, FormulaError> {
    let raw = value.raw();
    let frac_mask = (1_i64 << Fixed64::SCALE_BITS) - 1;
    if raw & frac_mask != 0 {
        return Err(FormulaError::Parse(
            "거듭제곱 지수는 정수여야 합니다".to_string(),
        ));
    }
    let exp = raw >> Fixed64::SCALE_BITS;
    if exp < i32::MIN as i64 || exp > i32::MAX as i64 {
        return Err(FormulaError::Parse(
            "거듭제곱 지수가 범위를 벗어났습니다".to_string(),
        ));
    }
    Ok(exp as i32)
}

struct FormulaLexer {
    chars: Vec<char>,
    pos: usize,
    dialect: FormulaDialect,
}

impl FormulaLexer {
    fn new(body: &str, dialect: FormulaDialect) -> Self {
        Self {
            chars: body.chars().collect(),
            pos: 0,
            dialect,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, FormulaError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if let Token::End = token {
                tokens.push(Token::End);
                break;
            }
            match token {
                Token::Ident(name) if self.dialect == FormulaDialect::Ascii1 && name.len() > 1 => {
                    for ch in name.chars() {
                        tokens.push(Token::Ident(ch.to_string()));
                    }
                }
                other => tokens.push(other),
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, FormulaError> {
        self.skip_ws();
        let Some(ch) = self.peek() else {
            return Ok(Token::End);
        };

        let token = match ch {
            '0'..='9' => self.read_number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.read_ident(),
            '+' => {
                self.pos += 1;
                Token::Plus
            }
            '-' => {
                self.pos += 1;
                Token::Minus
            }
            '*' => {
                self.pos += 1;
                Token::Star
            }
            '/' => {
                self.pos += 1;
                Token::Slash
            }
            '^' => {
                self.pos += 1;
                Token::Caret
            }
            '=' => {
                self.pos += 1;
                Token::Equal
            }
            '(' => {
                self.pos += 1;
                Token::LParen
            }
            ')' => {
                self.pos += 1;
                Token::RParen
            }
            ',' => {
                self.pos += 1;
                Token::Comma
            }
            _ => {
                return Err(FormulaError::Parse(format!("unexpected char: {}", ch)));
            }
        };
        Ok(token)
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, FormulaError> {
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.peek() == Some('.') {
            text.push('.');
            self.pos += 1;
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    text.push(ch);
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        let value = Fixed64::parse_literal(&text)
            .ok_or_else(|| FormulaError::Parse("bad number".to_string()))?;
        Ok(Token::Number(value))
    }

    fn read_ident(&mut self) -> Token {
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                self.pos += 1;
            } else {
                break;
            }
        }
        Token::Ident(text)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
}

struct FormulaParser {
    tokens: Vec<Token>,
    pos: usize,
    dialect: FormulaDialect,
}

impl FormulaParser {
    fn new(tokens: Vec<Token>, dialect: FormulaDialect) -> Self {
        Self { tokens, pos: 0, dialect }
    }

    fn parse_assignment(&mut self) -> Result<FormulaAst, FormulaError> {
        if let (Token::Ident(name), Token::Equal) = (self.peek().clone(), self.peek_n(1).clone()) {
            self.pos += 2;
            let expr = self.parse_expr()?;
            return Ok(FormulaAst::Assign { name, expr });
        }
        let expr = self.parse_expr()?;
        Ok(FormulaAst::Expr(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_term()?;
        loop {
            let op = match self.peek() {
                Token::Plus => Some(BinOp::Add),
                Token::Minus => Some(BinOp::Sub),
                _ => None,
            };
            let Some(op) = op else { break };
            self.pos += 1;
            let right = self.parse_term()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, FormulaError> {
        let mut expr = self.parse_power()?;
        loop {
            let mut implicit = false;
            let op = match self.peek() {
                Token::Star => Some(BinOp::Mul),
                Token::Slash => Some(BinOp::Div),
                _ => {
                    if self.dialect == FormulaDialect::Ascii1 && self.peek_starts_primary() {
                        implicit = true;
                        Some(BinOp::Mul)
                    } else {
                        None
                    }
                }
            };
            let Some(op) = op else { break };
            if !implicit {
                self.pos += 1;
            }
            let right = self.parse_power()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expr, FormulaError> {
        let left = self.parse_unary()?;
        if let Token::Caret = self.peek() {
            self.pos += 1;
            let right = self.parse_power()?;
            return Ok(Expr::Binary {
                op: BinOp::Pow,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, FormulaError> {
        match self.peek() {
            Token::Plus => {
                self.pos += 1;
                return self.parse_unary();
            }
            Token::Minus => {
                self.pos += 1;
                let expr = self.parse_unary()?;
                return Ok(Expr::UnaryNeg(Box::new(expr)));
            }
            _ => {}
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, FormulaError> {
        match self.peek().clone() {
            Token::Number(value) => {
                self.pos += 1;
                Ok(Expr::Number(value))
            }
            Token::Ident(name) => {
                self.pos += 1;
                if self.dialect == FormulaDialect::Ascii && matches!(self.peek(), Token::LParen) {
                    return self.parse_call(name);
                }
                Ok(Expr::Var(name))
            }
            Token::LParen => {
                self.pos += 1;
                let expr = self.parse_expr()?;
                match self.peek() {
                    Token::RParen => {
                        self.pos += 1;
                        Ok(expr)
                    }
                    _ => Err(FormulaError::Parse("expected ')'".to_string())),
                }
            }
            Token::End => Err(FormulaError::Parse("unexpected end".to_string())),
            other => Err(FormulaError::Parse(format!("unexpected token: {:?}", other))),
        }
    }

    fn parse_call(&mut self, name: String) -> Result<Expr, FormulaError> {
        if !matches!(self.peek(), Token::LParen) {
            return Ok(Expr::Var(name));
        }
        self.pos += 1;
        let mut args = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                let expr = self.parse_expr()?;
                args.push(expr);
                if matches!(self.peek(), Token::Comma) {
                    self.pos += 1;
                    continue;
                }
                break;
            }
        }
        match self.peek() {
            Token::RParen => self.pos += 1,
            _ => return Err(FormulaError::Parse("expected ')'".to_string())),
        }
        self.validate_reserved_call(&name, &args)?;
        Ok(Expr::Call { name, args })
    }

    fn validate_reserved_call(&self, name: &str, args: &[Expr]) -> Result<(), FormulaError> {
        match name {
            "sum" | "prod" => {
                if args.len() != 4 {
                    return Err(FormulaError::Parse(format!(
                        "{} 호출은 sum(index, from, to, body) 형태여야 합니다",
                        name
                    )));
                }
                self.require_ident(&args[0], "index")?;
            }
            "diff" => {
                if args.len() != 2 && args.len() != 3 {
                    return Err(FormulaError::Parse(
                        "diff 호출은 diff(expr, var[, order]) 형태여야 합니다".to_string(),
                    ));
                }
                self.require_ident(&args[1], "var")?;
                if args.len() == 3 {
                    self.require_positive_int_literal(&args[2], "order")?;
                }
            }
            "int" => {
                if args.len() != 2 && args.len() != 4 {
                    return Err(FormulaError::Parse(
                        "int 호출은 int(expr, var[, from, to]) 형태여야 합니다".to_string(),
                    ));
                }
                self.require_ident(&args[1], "var")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn require_ident(&self, expr: &Expr, label: &str) -> Result<(), FormulaError> {
        match expr {
            Expr::Var(_) => Ok(()),
            _ => Err(FormulaError::Parse(format!(
                "예약 호출 {} 인자는 식별자여야 합니다",
                label
            ))),
        }
    }

    fn require_positive_int_literal(&self, expr: &Expr, label: &str) -> Result<(), FormulaError> {
        let Expr::Number(value) = expr else {
            return Err(FormulaError::Parse(format!(
                "예약 호출 {} 인자는 정수 리터럴이어야 합니다",
                label
            )));
        };
        let raw = value.raw();
        let frac_mask = (1_i64 << Fixed64::SCALE_BITS) - 1;
        if raw & frac_mask != 0 {
            return Err(FormulaError::Parse(format!(
                "예약 호출 {} 인자는 정수 리터럴이어야 합니다",
                label
            )));
        }
        let int_value = raw >> Fixed64::SCALE_BITS;
        if int_value <= 0 {
            return Err(FormulaError::Parse(format!(
                "예약 호출 {} 인자는 양의 정수여야 합니다",
                label
            )));
        }
        Ok(())
    }

    fn expect_end(&mut self) -> Result<(), FormulaError> {
        match self.peek() {
            Token::End => Ok(()),
            other => Err(FormulaError::Parse(format!("unexpected token: {:?}", other))),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::End)
    }

    fn peek_n(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or(&Token::End)
    }

    fn peek_starts_primary(&self) -> bool {
        matches!(self.peek(), Token::Number(_) | Token::Ident(_) | Token::LParen)
    }
}
