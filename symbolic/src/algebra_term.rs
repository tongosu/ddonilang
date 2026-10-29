//! Exact Q arithmetic for source-bound algebra candidates.
//! Parameter slots are local indices, NOT declaration identities or evidence.
//! The source owner/checker must bind every slot to its actual lexical binder.
//! No parser, provider registration, serialized certificate or success flag.
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QTerm {
    Integer(i64),
    Parameter(usize),
    Add(Box<QTerm>, Box<QTerm>),
    Subtract(Box<QTerm>, Box<QTerm>),
    Multiply(Box<QTerm>, Box<QTerm>),
    /// Only a nonzero integer literal denominator is admitted by this slice.
    DivideByInteger(Box<QTerm>, i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizeError { UnboundParameter, ZeroDenominator, BudgetExceeded }

/// Arithmetic normal form only. Equality is not a checked proof and cannot
/// authenticate the source, assumptions, rule, or definition dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizedQ(BTreeMap<Vec<usize>, BigRational>);

struct Budget { nodes: usize, products: usize }
impl Budget {
    fn visit(&mut self, depth: usize) -> Result<(), NormalizeError> {
        if depth > 64 || self.nodes == 0 { return Err(NormalizeError::BudgetExceeded); }
        self.nodes -= 1;
        Ok(())
    }
    fn product(&mut self) -> Result<(), NormalizeError> {
        if self.products == 0 { return Err(NormalizeError::BudgetExceeded); }
        self.products -= 1;
        Ok(())
    }
}

fn insert(out: &mut BTreeMap<Vec<usize>, BigRational>, key: Vec<usize>, value: BigRational)
    -> Result<(), NormalizeError> {
    if key.len() > 64 { return Err(NormalizeError::BudgetExceeded); }
    let value = out.get(&key).cloned().unwrap_or_else(BigRational::zero) + value;
    if value.numer().bits() > 4096 || value.denom().bits() > 4096 {
        return Err(NormalizeError::BudgetExceeded);
    }
    if value.is_zero() { out.remove(&key); } else { out.insert(key, value); }
    if out.len() > 4096 { return Err(NormalizeError::BudgetExceeded); }
    Ok(())
}

/// Normalize a typed candidate, with bounded exact arithmetic. A failure is
/// unknown/unsupported, never a mathematical counterexample or a proof.
pub fn normalize(term: &QTerm, parameter_count: usize) -> Result<NormalizedQ, NormalizeError> {
    fn walk(term: &QTerm, count: usize, depth: usize, budget: &mut Budget)
        -> Result<NormalizedQ, NormalizeError> {
        budget.visit(depth)?;
        let mut out = BTreeMap::new();
        match term {
            QTerm::Integer(value) => {
                insert(&mut out, vec![], BigRational::from_integer(BigInt::from(*value)))?;
            }
            QTerm::Parameter(slot) => {
                if *slot >= count { return Err(NormalizeError::UnboundParameter); }
                insert(&mut out, vec![*slot], BigRational::one())?;
            }
            QTerm::DivideByInteger(value, denominator) => {
                if *denominator == 0 { return Err(NormalizeError::ZeroDenominator); }
                let value = walk(value, count, depth + 1, budget)?;
                let divisor = BigRational::from_integer(BigInt::from(*denominator));
                for (key, coefficient) in value.0 {
                    insert(&mut out, key, coefficient / &divisor)?;
                }
            }
            QTerm::Add(left, right) | QTerm::Subtract(left, right) => {
                out = walk(left, count, depth + 1, budget)?.0;
                // Both operands are inspected even if a surrounding term is zero.
                for (key, coefficient) in walk(right, count, depth + 1, budget)?.0 {
                    insert(&mut out, key, if matches!(term, QTerm::Subtract(..)) {
                        -coefficient
                    } else { coefficient })?;
                }
            }
            QTerm::Multiply(left, right) => {
                let left = walk(left, count, depth + 1, budget)?;
                let right = walk(right, count, depth + 1, budget)?;
                for (a, x) in &left.0 {
                    for (b, y) in &right.0 {
                        budget.product()?;
                        let mut key = a.clone(); key.extend(b); key.sort_unstable();
                        insert(&mut out, key, x * y)?;
                    }
                }
            }
        }
        Ok(NormalizedQ(out))
    }
    walk(term, parameter_count, 0, &mut Budget { nodes: 4096, products: 16384 })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn add(a: QTerm, b: QTerm) -> QTerm { QTerm::Add(Box::new(a), Box::new(b)) }
    fn sub(a: QTerm, b: QTerm) -> QTerm { QTerm::Subtract(Box::new(a), Box::new(b)) }
    fn mul(a: QTerm, b: QTerm) -> QTerm { QTerm::Multiply(Box::new(a), Box::new(b)) }
    fn div(a: QTerm, b: i64) -> QTerm { QTerm::DivideByInteger(Box::new(a), b) }

    #[test]
    fn algebra_term_pythagorean_expansion_and_exact_halves() {
        let a = QTerm::Parameter(0); let b = QTerm::Parameter(1);
        let square = mul(add(a.clone(), b.clone()), add(a.clone(), b.clone()));
        let cross = mul(QTerm::Integer(2), mul(a.clone(), b.clone()));
        let expanded = add(add(mul(a.clone(), a.clone()), cross.clone()), mul(b.clone(), b.clone()));
        assert_eq!(normalize(&square, 2), normalize(&expanded, 2));
        assert_ne!(normalize(&square, 2).unwrap(), normalize(&sub(expanded, cross.clone()), 2).unwrap());
        assert_eq!(normalize(&mul(QTerm::Integer(4), div(mul(a, b), 2)), 2), normalize(&cross, 2));
        assert_eq!(normalize(&div(QTerm::Integer(2), 6), 0), normalize(&div(QTerm::Integer(1), 3), 0));
    }

    #[test]
    fn algebra_term_sides_cannot_be_replaced_by_equal_residuals() {
        let x = QTerm::Parameter(0); let shifted = add(x.clone(), QTerm::Integer(1));
        assert_eq!(normalize(&sub(x.clone(), x.clone()), 1), normalize(&sub(shifted.clone(), shifted.clone()), 1));
        assert_ne!(normalize(&x, 1).unwrap(), normalize(&shifted, 1).unwrap());
        assert_ne!(normalize(&QTerm::Parameter(0), 2).unwrap(), normalize(&QTerm::Parameter(1), 2).unwrap());
    }

    #[test]
    fn algebra_term_zero_does_not_hide_invalid_subterms() {
        assert_eq!(normalize(&mul(QTerm::Integer(0), div(QTerm::Integer(1), 0)), 0), Err(NormalizeError::ZeroDenominator));
        assert_eq!(normalize(&sub(QTerm::Parameter(1), QTerm::Parameter(1)), 1), Err(NormalizeError::UnboundParameter));
        assert_eq!(normalize(&div(QTerm::Integer(i64::MIN), i64::MIN), 0), normalize(&QTerm::Integer(1), 0));
    }

    #[test]
    fn algebra_term_resource_exhaustion_is_not_inequality() {
        let mut term = QTerm::Integer(1);
        for _ in 0..66 { term = add(term, QTerm::Integer(1)); }
        assert_eq!(normalize(&term, 0), Err(NormalizeError::BudgetExceeded));
    }
}


/// Arithmetic only: rejects syntactic variables before cancellation and keeps
/// all original definedness/budget checks. No evidence or provider is issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReciprocalError { Arithmetic(NormalizeError), NotClosed, Zero }
pub fn closed_reciprocal(term:&QTerm)->Result<QTerm,ReciprocalError> {
    use num_traits::ToPrimitive;
    fn closed(t:&QTerm,depth:usize,remaining:&mut usize)->Result<bool,ReciprocalError> {
        if depth>64 || *remaining==0 {return Err(ReciprocalError::Arithmetic(NormalizeError::BudgetExceeded));}
        *remaining-=1;
        Ok(match t {
            QTerm::Parameter(_)=>false,
            QTerm::Integer(_)=>true,
            QTerm::DivideByInteger(a,_)=>closed(a,depth+1,remaining)?,
            QTerm::Add(a,b)|QTerm::Subtract(a,b)|QTerm::Multiply(a,b)=>{
                let a=closed(a,depth+1,remaining)?;let b=closed(b,depth+1,remaining)?;a&&b
            }
        })
    }
    if !closed(term,0,&mut 4096)? {return Err(ReciprocalError::NotClosed);}
    let n=normalize(term,0).map_err(ReciprocalError::Arithmetic)?;
    let c=n.0.get(&vec![]).cloned().unwrap_or_else(BigRational::zero);
    if c.is_zero(){return Err(ReciprocalError::Zero);}
    let numerator=c.denom().to_i64().ok_or(ReciprocalError::Arithmetic(NormalizeError::BudgetExceeded))?;
    let denominator=c.numer().to_i64().ok_or(ReciprocalError::Arithmetic(NormalizeError::BudgetExceeded))?;
    Ok(QTerm::DivideByInteger(Box::new(QTerm::Integer(numerator)),denominator))
}

#[cfg(test)]
mod reciprocal_tests {
 use super::*;
 #[test] fn closed_reciprocal_keeps_zero_variables_and_limits_distinct(){
  let c=QTerm::DivideByInteger(Box::new(QTerm::Integer(-3)),2);
  let r=closed_reciprocal(&c).unwrap();
  assert_eq!(normalize(&QTerm::Multiply(Box::new(c),Box::new(r)),0),normalize(&QTerm::Integer(1),0));
  assert_eq!(closed_reciprocal(&QTerm::Subtract(Box::new(QTerm::Integer(2)),Box::new(QTerm::Integer(2)))),Err(ReciprocalError::Zero));
  let t=QTerm::Add(Box::new(QTerm::Subtract(Box::new(QTerm::Parameter(0)),Box::new(QTerm::Parameter(0)))),Box::new(QTerm::Integer(1)));
  assert_eq!(closed_reciprocal(&t),Err(ReciprocalError::NotClosed));
  let bad=QTerm::Multiply(Box::new(QTerm::Integer(0)),Box::new(QTerm::DivideByInteger(Box::new(QTerm::Integer(1)),0)));
  assert_eq!(closed_reciprocal(&bad),Err(ReciprocalError::Arithmetic(NormalizeError::ZeroDenominator)));
  let large=QTerm::Multiply(Box::new(QTerm::Integer(i64::MAX)),Box::new(QTerm::Integer(2)));
  assert_eq!(closed_reciprocal(&large),Err(ReciprocalError::Arithmetic(NormalizeError::BudgetExceeded)));
 }
}
