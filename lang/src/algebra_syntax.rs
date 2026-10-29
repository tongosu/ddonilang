//! Conversion from the existing expression parser's AST. No second parser,
//! name lookup, binder identity, arithmetic evaluation or evidence issuance.
use crate::{ast::*, ParseError};

fn reject(span: Span, detail: &str) -> ParseError {
    ParseError {span, message: format!("E_PROOF_ALGEBRA_TERM: {detail}")}
}

fn integer(expr: &Expr) -> Option<i64> {
    if let Some(literal) = &expr.numeric_literal_source {
        if !literal.is_decimal() { return literal.raw().parse().ok(); }
    }
    if expr.source_unary_minus {
        if let ExprKind::Infix {right,..} = &expr.kind {
            return integer(right)?.checked_neg();
        }
    }
    None
}

pub(crate) fn term(expr: &Expr) -> Result<AlgebraTermSyntax, ParseError> {
    fn visit(expr: &Expr, depth: usize, remaining: &mut usize) -> Result<AlgebraTermSyntax, ParseError> {
        if depth > 64 || *remaining == 0 { return Err(ParseError{span:expr.span,message:"E_PROOF_BUDGET: 정적 항 구성 한도를 넘었습니다".into()}); }
        *remaining -= 1;
        let kind = if let Some(n) = integer(expr) { AlgebraTermKind::Integer(n) }
        else { match &expr.kind {
            ExprKind::Var(name) => AlgebraTermKind::Name(name.clone()),
            ExprKind::Infix {left,op,right} => {
                let left = if expr.source_unary_minus {
                    AlgebraTermSyntax {span:left.span,kind:AlgebraTermKind::Integer(0)}
                } else { visit(left, depth+1, remaining)? };
                match op.as_str() {
                    "+" => AlgebraTermKind::Add(Box::new(left),Box::new(visit(right,depth+1,remaining)?)),
                    "-" => AlgebraTermKind::Subtract(Box::new(left),Box::new(visit(right,depth+1,remaining)?)),
                    "*" => AlgebraTermKind::Multiply(Box::new(left),Box::new(visit(right,depth+1,remaining)?)),
                    "/" => {
                        let denominator=integer(right).ok_or_else(||reject(right.span,"분모에는 0이 아닌 정수 상수만 쓸 수 있습니다"))?;
                        if denominator==0 {return Err(reject(right.span,"분모가 0인 정적 항은 허용하지 않습니다"));}
                        AlgebraTermKind::DivideByInteger(Box::new(left),denominator)
                    }
                    _ => return Err(reject(expr.span,"지원하는 정적 항은 더하기·빼기·곱하기·정수 상수 나눗셈입니다")),
                }
            }
            _ => return Err(reject(expr.span,"정확 정수 상수와 명시된 나눔수 변수의 정적 항이 필요합니다")),
        }};
        Ok(AlgebraTermSyntax {span:expr.span,kind})
    }
    visit(expr,0,&mut 4096)
}
