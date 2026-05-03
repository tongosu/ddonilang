use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const MATHIR_SCHEMA: &str = "ddn.symbolic.mathir.v1";
pub const EQUIV_CERT_SCHEMA: &str = "ddn.symbolic.equivalence_certificate.v1";
pub const RELATION_EQUIV_CERT_SCHEMA: &str = "ddn.symbolic.relation_equivalence_certificate.v1";
pub const RELATION_SOLVE_SCHEMA: &str = "ddn.symbolic.relation_solve.v1";
pub const RELATION_SOLVE_CONSISTENCY_CERT_SCHEMA: &str =
    "ddn.symbolic.relation_solve_consistency_certificate.v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MathIr {
    pub schema: String,
    pub source: String,
    pub canonical: String,
    pub variables: Vec<String>,
    pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EquivalenceCertificate {
    pub schema: String,
    pub lhs: String,
    pub rhs: String,
    pub lhs_canonical: String,
    pub rhs_canonical: String,
    pub equivalent: bool,
    pub method: String,
    pub certificate_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationEquationSpec {
    pub lhs: String,
    pub rhs: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationEquivalenceCertificate {
    pub schema: String,
    pub first: RelationEquationSpec,
    pub second: RelationEquationSpec,
    pub first_canonical: String,
    pub second_canonical: String,
    pub equivalent: bool,
    pub method: String,
    pub certificate_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolveBinding {
    pub numerator: String,
    pub denominator: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationSolveConsistencyCertificate {
    pub schema: String,
    pub equations: Vec<RelationEquationSpec>,
    pub bindings: BTreeMap<String, SolveBinding>,
    pub consistent: bool,
    pub method: String,
    pub certificate_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationSolveOutcome {
    Solution(BTreeMap<String, SolveBinding>),
    NoSolution,
    NonUnique,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Monomial(Vec<(String, u32)>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial {
    terms: BTreeMap<Monomial, BigRational>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr {
    Num(BigRational),
    Var(String),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
}

pub fn mathir(input: &str) -> Result<MathIr, String> {
    let poly = parse_polynomial(input)?;
    let canonical = poly.to_canonical();
    let variables = poly.variables();
    let hash = format!(
        "sha256:{}",
        sha256_hex(format!("{MATHIR_SCHEMA}\n{canonical}").as_bytes())
    );
    Ok(MathIr {
        schema: MATHIR_SCHEMA.to_string(),
        source: input.to_string(),
        canonical,
        variables,
        hash,
    })
}

pub fn canon(input: &str) -> Result<String, String> {
    Ok(mathir(input)?.canonical)
}

pub fn simplify(input: &str) -> Result<String, String> {
    canon(input)
}

pub fn expand(input: &str) -> Result<String, String> {
    canon(input)
}

pub fn factor(input: &str) -> Result<String, String> {
    let poly = parse_polynomial(input)?;
    if poly.is_zero() {
        return Ok("0".to_string());
    }
    if let Some(text) = factor_difference_of_squares(&poly) {
        return Ok(text);
    }
    Ok(poly.to_canonical())
}

pub fn diff(input: &str, var: &str) -> Result<String, String> {
    Ok(parse_polynomial(input)?.differentiate(var).to_canonical())
}

pub fn integrate(input: &str, var: &str) -> Result<String, String> {
    Ok(parse_polynomial(input)?.integrate(var).to_canonical())
}

pub fn equivalent(lhs: &str, rhs: &str) -> Result<bool, String> {
    Ok(parse_polynomial(lhs)? == parse_polynomial(rhs)?)
}

pub fn prove_equivalent(lhs: &str, rhs: &str) -> Result<EquivalenceCertificate, String> {
    let lhs_canonical = canon(lhs)?;
    let rhs_canonical = canon(rhs)?;
    let equivalent = lhs_canonical == rhs_canonical;
    let base = format!("{EQUIV_CERT_SCHEMA}\n{lhs_canonical}\n{rhs_canonical}\n{equivalent}");
    Ok(EquivalenceCertificate {
        schema: EQUIV_CERT_SCHEMA.to_string(),
        lhs: lhs.to_string(),
        rhs: rhs.to_string(),
        lhs_canonical,
        rhs_canonical,
        equivalent,
        method: "mathir_polynomial_normal_form".to_string(),
        certificate_hash: format!("sha256:{}", sha256_hex(base.as_bytes())),
    })
}

pub fn relation_canon(lhs: &str, rhs: &str) -> Result<String, String> {
    let left = parse_polynomial(lhs)?;
    let right = parse_polynomial(rhs)?;
    Ok(left.sub(&right).to_canonical())
}

pub fn relation_equivalent(
    first_lhs: &str,
    first_rhs: &str,
    second_lhs: &str,
    second_rhs: &str,
) -> Result<bool, String> {
    Ok(relation_canon(first_lhs, first_rhs)? == relation_canon(second_lhs, second_rhs)?)
}

pub fn prove_relation_equivalent(
    first_lhs: &str,
    first_rhs: &str,
    second_lhs: &str,
    second_rhs: &str,
) -> Result<RelationEquivalenceCertificate, String> {
    let first_canonical = relation_canon(first_lhs, first_rhs)?;
    let second_canonical = relation_canon(second_lhs, second_rhs)?;
    let equivalent = first_canonical == second_canonical;
    let base = format!(
        "{RELATION_EQUIV_CERT_SCHEMA}\n{first_canonical}\n{second_canonical}\n{equivalent}"
    );
    Ok(RelationEquivalenceCertificate {
        schema: RELATION_EQUIV_CERT_SCHEMA.to_string(),
        first: RelationEquationSpec {
            lhs: first_lhs.to_string(),
            rhs: first_rhs.to_string(),
        },
        second: RelationEquationSpec {
            lhs: second_lhs.to_string(),
            rhs: second_rhs.to_string(),
        },
        first_canonical,
        second_canonical,
        equivalent,
        method: "mathir_relation_normal_form".to_string(),
        certificate_hash: format!("sha256:{}", sha256_hex(base.as_bytes())),
    })
}

pub fn solve_relation_equation(lhs: &str, rhs: &str) -> Result<RelationSolveOutcome, String> {
    let left = parse_polynomial(lhs)?;
    let right = parse_polynomial(rhs)?;
    solve_relation_polynomial(&left.sub(&right))
}

pub fn solve_relation_system(
    equations: &[(String, String)],
) -> Result<RelationSolveOutcome, String> {
    if equations.len() != 2 {
        return Err(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 2식 2미지수 exact system만 지원합니다"
                .to_string(),
        );
    }
    let first = parse_polynomial(&equations[0].0)?.sub(&parse_polynomial(&equations[0].1)?);
    let second = parse_polynomial(&equations[1].0)?.sub(&parse_polynomial(&equations[1].1)?);
    solve_linear_system_2x2(&first, &second)
}

pub fn solve_linear_equation(lhs: &str, rhs: &str) -> Result<RelationSolveOutcome, String> {
    solve_relation_equation(lhs, rhs)
}

pub fn relation_holds(
    lhs: &str,
    rhs: &str,
    bindings: &BTreeMap<String, SolveBinding>,
) -> Result<bool, String> {
    let left = parse_polynomial(lhs)?;
    let right = parse_polynomial(rhs)?;
    let env = solve_bindings_to_rationals(bindings)?;
    Ok(left.sub(&right).evaluate(&env).is_zero())
}

pub fn relation_system_holds(
    equations: &[(String, String)],
    bindings: &BTreeMap<String, SolveBinding>,
) -> Result<bool, String> {
    for (lhs, rhs) in equations {
        if !relation_holds(lhs, rhs, bindings)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn prove_relation_solution_consistency(
    equations: &[(String, String)],
    bindings: &BTreeMap<String, SolveBinding>,
) -> Result<RelationSolveConsistencyCertificate, String> {
    let consistent = relation_system_holds(equations, bindings)?;
    let equation_specs = equations
        .iter()
        .map(|(lhs, rhs)| RelationEquationSpec {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
        })
        .collect::<Vec<_>>();
    let equations_text = equation_specs
        .iter()
        .map(|eq| format!("{} =:= {}", eq.lhs, eq.rhs))
        .collect::<Vec<_>>()
        .join("\n");
    let bindings_text = bindings
        .iter()
        .map(|(name, value)| format!("{name}={}/{}", value.numerator, value.denominator))
        .collect::<Vec<_>>()
        .join("\n");
    let base = format!(
        "{RELATION_SOLVE_CONSISTENCY_CERT_SCHEMA}\n{equations_text}\n{bindings_text}\n{consistent}"
    );
    Ok(RelationSolveConsistencyCertificate {
        schema: RELATION_SOLVE_CONSISTENCY_CERT_SCHEMA.to_string(),
        equations: equation_specs,
        bindings: bindings.clone(),
        consistent,
        method: "mathir_relation_substitution".to_string(),
        certificate_hash: format!("sha256:{}", sha256_hex(base.as_bytes())),
    })
}

pub fn to_detjson<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

fn parse_polynomial(input: &str) -> Result<Polynomial, String> {
    Parser::new(input)
        .parse()
        .and_then(|expr| expr_to_poly(&expr))
}

fn solve_relation_polynomial(poly: &Polynomial) -> Result<RelationSolveOutcome, String> {
    let vars = poly.variables();
    match vars.len() {
        0 | 1 => solve_univariate_relation(poly, vars.first().map(String::as_str).unwrap_or("x")),
        _ => Err(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 여러 미지수 단일 관계 풀기는 아직 지원하지 않습니다"
                .to_string(),
        ),
    }
}

fn solve_univariate_relation(
    poly: &Polynomial,
    variable: &str,
) -> Result<RelationSolveOutcome, String> {
    let mut constant = BigRational::zero();
    let mut linear_coeff = BigRational::zero();
    let mut quadratic_coeff = BigRational::zero();

    for (mono, coeff) in &poly.terms {
        let degree: u32 = mono.0.iter().map(|(_, exp)| *exp).sum();
        match degree {
            0 => constant += coeff.clone(),
            1 => {
                if mono.0.len() != 1 || mono.0[0].0 != variable || mono.0[0].1 != 1 {
                    return Err(
                        "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 선형 단항식만 지원합니다"
                            .to_string(),
                    );
                }
                linear_coeff += coeff.clone();
            }
            2 => {
                if mono.0.len() != 1 || mono.0[0].0 != variable || mono.0[0].1 != 2 {
                    return Err(
                        "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 단일 미지수 2차식까지만 지원합니다"
                            .to_string(),
                    );
                }
                quadratic_coeff += coeff.clone();
            }
            _ => {
                return Err(
                    "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 단일 미지수 2차식까지만 지원합니다"
                        .to_string(),
                );
            }
        }
    }

    if quadratic_coeff.is_zero() {
        return solve_linear_coeffs(variable, linear_coeff, constant);
    }
    solve_quadratic_coeffs(variable, quadratic_coeff, linear_coeff, constant)
}

fn solve_linear_coeffs(
    variable: &str,
    linear_coeff: BigRational,
    constant: BigRational,
) -> Result<RelationSolveOutcome, String> {
    if linear_coeff.is_zero() {
        return Ok(if constant.is_zero() {
            RelationSolveOutcome::NonUnique
        } else {
            RelationSolveOutcome::NoSolution
        });
    }
    let solution = -constant / linear_coeff;
    Ok(single_solution(variable, solution))
}

fn solve_quadratic_coeffs(
    variable: &str,
    quadratic_coeff: BigRational,
    linear_coeff: BigRational,
    constant: BigRational,
) -> Result<RelationSolveOutcome, String> {
    let four = BigRational::from_integer(BigInt::from(4u8));
    let discriminant =
        linear_coeff.clone() * linear_coeff.clone() - four * quadratic_coeff.clone() * constant;
    if discriminant.is_negative() {
        return Err(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE exact rational quadratic root만 지원합니다"
                .to_string(),
        );
    }
    let Some(sqrt_discriminant) = rational_perfect_square(&discriminant) else {
        return Err(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE exact rational quadratic root만 지원합니다"
                .to_string(),
        );
    };
    let two = BigRational::from_integer(BigInt::from(2u8));
    let denom = two * quadratic_coeff;
    let root1 = (-linear_coeff.clone() + sqrt_discriminant.clone()) / denom.clone();
    let root2 = (-linear_coeff - sqrt_discriminant) / denom;
    if root1 == root2 {
        Ok(single_solution(variable, root1))
    } else {
        Ok(RelationSolveOutcome::NonUnique)
    }
}

fn solve_linear_system_2x2(
    first: &Polynomial,
    second: &Polynomial,
) -> Result<RelationSolveOutcome, String> {
    let mut vars = first.variables();
    for var in second.variables() {
        if !vars.contains(&var) {
            vars.push(var);
        }
    }
    vars.sort();
    if vars.len() != 2 {
        return Err(
            "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 2식 2미지수 exact system만 지원합니다"
                .to_string(),
        );
    }
    let (a1, b1, c1) = extract_linear_system_coeffs(first, &vars)?;
    let (a2, b2, c2) = extract_linear_system_coeffs(second, &vars)?;
    let det = a1.clone() * b2.clone() - a2.clone() * b1.clone();
    if det.is_zero() {
        let dependent = a1.clone() * c2.clone() == a2.clone() * c1.clone()
            && b1.clone() * c2.clone() == b2.clone() * c1.clone();
        return Ok(if dependent {
            RelationSolveOutcome::NonUnique
        } else {
            RelationSolveOutcome::NoSolution
        });
    }
    let x = (b1.clone() * c2.clone() - b2.clone() * c1.clone()) / det.clone();
    let y = (c1 * a2 - a1 * c2) / det;
    let mut bindings = BTreeMap::new();
    bindings.insert(vars[0].clone(), binding_from_rational(x));
    bindings.insert(vars[1].clone(), binding_from_rational(y));
    Ok(RelationSolveOutcome::Solution(bindings))
}

fn extract_linear_system_coeffs(
    poly: &Polynomial,
    vars: &[String],
) -> Result<(BigRational, BigRational, BigRational), String> {
    let mut constant = BigRational::zero();
    let mut first = BigRational::zero();
    let mut second = BigRational::zero();
    for (mono, coeff) in &poly.terms {
        let degree: u32 = mono.0.iter().map(|(_, exp)| *exp).sum();
        match degree {
            0 => constant += coeff.clone(),
            1 => {
                if mono.0.len() != 1 || mono.0[0].1 != 1 {
                    return Err(
                        "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 2식 2미지수 선형계만 지원합니다"
                            .to_string(),
                    );
                }
                if mono.0[0].0 == vars[0] {
                    first += coeff.clone();
                } else if mono.0[0].0 == vars[1] {
                    second += coeff.clone();
                } else {
                    return Err(
                        "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 2식 2미지수 선형계만 지원합니다"
                            .to_string(),
                    );
                }
            }
            _ => {
                return Err(
                    "E_SYMBOLIC_UNSUPPORTED_RELATION_SOLVE 2식 2미지수 선형계만 지원합니다"
                        .to_string(),
                );
            }
        }
    }
    Ok((first, second, constant))
}

fn single_solution(variable: &str, value: BigRational) -> RelationSolveOutcome {
    let mut bindings = BTreeMap::new();
    bindings.insert(variable.to_string(), binding_from_rational(value));
    RelationSolveOutcome::Solution(bindings)
}

fn binding_from_rational(value: BigRational) -> SolveBinding {
    SolveBinding {
        numerator: value.numer().to_string(),
        denominator: value.denom().to_string(),
    }
}

fn parse_bigint(raw: &str) -> Result<BigInt, String> {
    let normalized = raw.trim().replace('_', "");
    BigInt::parse_bytes(normalized.as_bytes(), 10)
        .ok_or_else(|| format!("E_SYMBOLIC_BAD_INTEGER {raw}"))
}

fn solve_bindings_to_rationals(
    bindings: &BTreeMap<String, SolveBinding>,
) -> Result<BTreeMap<String, BigRational>, String> {
    let mut out = BTreeMap::new();
    for (name, binding) in bindings {
        let num = parse_bigint(&binding.numerator)?;
        let den = parse_bigint(&binding.denominator)?;
        if den.is_zero() {
            return Err("E_SYMBOLIC_BAD_BINDING_ZERO_DEN".to_string());
        }
        out.insert(name.clone(), BigRational::new(num, den));
    }
    Ok(out)
}

fn rational_perfect_square(value: &BigRational) -> Option<BigRational> {
    if value.is_negative() {
        return None;
    }
    let num = bigint_perfect_square(value.numer())?;
    let den = bigint_perfect_square(value.denom())?;
    Some(BigRational::new(num, den))
}

fn bigint_perfect_square(value: &BigInt) -> Option<BigInt> {
    if value.is_negative() {
        return None;
    }
    if value.is_zero() {
        return Some(BigInt::zero());
    }
    let mut low = BigInt::zero();
    let mut high = BigInt::one();
    while &high * &high < *value {
        high <<= 1usize;
    }
    while low <= high {
        let mid = (&low + &high) >> 1usize;
        let square = &mid * &mid;
        if square == *value {
            return Some(mid);
        }
        if square < *value {
            low = mid + BigInt::one();
        } else {
            if mid.is_zero() {
                break;
            }
            high = mid - BigInt::one();
        }
    }
    None
}

impl Polynomial {
    fn zero() -> Self {
        Self {
            terms: BTreeMap::new(),
        }
    }

    fn one() -> Self {
        Self::constant(BigRational::one())
    }

    fn constant(value: BigRational) -> Self {
        if value.is_zero() {
            return Self::zero();
        }
        let mut terms = BTreeMap::new();
        terms.insert(Monomial(Vec::new()), value);
        Self { terms }
    }

    fn var(name: &str) -> Self {
        let mut terms = BTreeMap::new();
        terms.insert(Monomial(vec![(name.to_string(), 1)]), BigRational::one());
        Self { terms }
    }

    fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    fn add(&self, rhs: &Self) -> Self {
        let mut out = self.terms.clone();
        for (mono, coeff) in &rhs.terms {
            *out.entry(mono.clone()).or_insert_with(BigRational::zero) += coeff.clone();
        }
        Self { terms: out }.normalized()
    }

    fn neg(&self) -> Self {
        Self {
            terms: self
                .terms
                .iter()
                .map(|(m, c)| (m.clone(), -c.clone()))
                .collect(),
        }
    }

    fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.neg())
    }

    fn mul(&self, rhs: &Self) -> Self {
        let mut out: BTreeMap<Monomial, BigRational> = BTreeMap::new();
        for (lm, lc) in &self.terms {
            for (rm, rc) in &rhs.terms {
                let mono = multiply_monomial(lm, rm);
                *out.entry(mono).or_insert_with(BigRational::zero) += lc.clone() * rc.clone();
            }
        }
        Self { terms: out }.normalized()
    }

    fn div_poly(&self, rhs: &Self) -> Result<Self, String> {
        if rhs.is_zero() {
            return Err("E_SYMBOLIC_DIV_ZERO 0으로 나눌 수 없습니다".to_string());
        }
        if let Some(denom) = rhs.constant_value() {
            return Ok(Self {
                terms: self
                    .terms
                    .iter()
                    .map(|(m, c)| (m.clone(), c.clone() / denom.clone()))
                    .collect(),
            }
            .normalized());
        }

        let mut remainder = self.clone();
        let mut quotient = Polynomial::zero();
        let (denom_mono, denom_coeff) = rhs
            .leading_term()
            .ok_or_else(|| "E_SYMBOLIC_DIV_ZERO 0으로 나눌 수 없습니다".to_string())?;
        while !remainder.is_zero() {
            let Some((rem_mono, rem_coeff)) = remainder.leading_term() else {
                break;
            };
            let Some(term_mono) = divide_monomial(&rem_mono, &denom_mono) else {
                return Err("E_SYMBOLIC_UNSUPPORTED_RATIONAL_REMAINDER 유리식 나머리가 있는 나눗셈은 아직 지원하지 않습니다".to_string());
            };
            let term_coeff = rem_coeff / denom_coeff.clone();
            let term = Polynomial {
                terms: BTreeMap::from([(term_mono, term_coeff)]),
            };
            quotient = quotient.add(&term);
            remainder = remainder.sub(&term.mul(rhs));
        }
        Ok(quotient.normalized())
    }

    fn leading_term(&self) -> Option<(Monomial, BigRational)> {
        self.terms
            .iter()
            .max_by(|(am, _), (bm, _)| monomial_sort_key(am).cmp(&monomial_sort_key(bm)))
            .map(|(m, c)| (m.clone(), c.clone()))
    }

    fn pow_u32(&self, exponent: u32) -> Self {
        let mut out = Polynomial::one();
        for _ in 0..exponent {
            out = out.mul(self);
        }
        out
    }

    fn constant_value(&self) -> Option<BigRational> {
        if self.terms.len() != 1 {
            return None;
        }
        self.terms.get(&Monomial(Vec::new())).cloned()
    }

    fn evaluate(&self, bindings: &BTreeMap<String, BigRational>) -> BigRational {
        let mut out = BigRational::zero();
        for (mono, coeff) in &self.terms {
            let mut term = coeff.clone();
            for (name, exp) in &mono.0 {
                let Some(value) = bindings.get(name) else {
                    return BigRational::zero();
                };
                let mut power = BigRational::one();
                for _ in 0..*exp {
                    power *= value.clone();
                }
                term *= power;
            }
            out += term;
        }
        out
    }

    fn normalized(mut self) -> Self {
        self.terms.retain(|_, coeff| !coeff.is_zero());
        self
    }

    fn differentiate(&self, var: &str) -> Self {
        let mut out = BTreeMap::new();
        for (mono, coeff) in &self.terms {
            let mut parts = mono.0.clone();
            let Some(idx) = parts.iter().position(|(name, _)| name == var) else {
                continue;
            };
            let exponent = parts[idx].1;
            let next_coeff = coeff.clone() * BigRational::from_integer(BigInt::from(exponent));
            if exponent == 1 {
                parts.remove(idx);
            } else {
                parts[idx].1 -= 1;
            }
            out.insert(Monomial(parts), next_coeff);
        }
        Self { terms: out }.normalized()
    }

    fn integrate(&self, var: &str) -> Self {
        let mut out = BTreeMap::new();
        for (mono, coeff) in &self.terms {
            let mut parts = mono.0.clone();
            match parts.iter().position(|(name, _)| name == var) {
                Some(idx) => parts[idx].1 += 1,
                None => parts.push((var.to_string(), 1)),
            }
            parts.sort_by(|a, b| a.0.cmp(&b.0));
            let exponent = parts
                .iter()
                .find(|(name, _)| name == var)
                .map(|(_, exp)| *exp)
                .unwrap_or(1);
            let denom = BigRational::from_integer(BigInt::from(exponent));
            out.insert(Monomial(parts), coeff.clone() / denom);
        }
        Self { terms: out }.normalized()
    }

    fn variables(&self) -> Vec<String> {
        let mut vars = BTreeSet::new();
        for mono in self.terms.keys() {
            for (name, _) in &mono.0 {
                vars.insert(name.clone());
            }
        }
        vars.into_iter().collect()
    }

    fn to_canonical(&self) -> String {
        if self.terms.is_empty() {
            return "0".to_string();
        }
        let mut terms = self.terms.iter().collect::<Vec<_>>();
        terms.sort_by(|(am, _), (bm, _)| monomial_sort_key(bm).cmp(&monomial_sort_key(am)));
        let mut out = String::new();
        for (idx, (mono, coeff)) in terms.into_iter().enumerate() {
            let negative = coeff.is_negative();
            let abs = coeff.abs();
            let body = format_term(mono, &abs);
            if idx == 0 {
                if negative {
                    out.push('-');
                }
                out.push_str(&body);
            } else {
                out.push_str(if negative { " - " } else { " + " });
                out.push_str(&body);
            }
        }
        out
    }
}

fn expr_to_poly(expr: &Expr) -> Result<Polynomial, String> {
    match expr {
        Expr::Num(value) => Ok(Polynomial::constant(value.clone())),
        Expr::Var(name) => Ok(Polynomial::var(name)),
        Expr::Neg(inner) => Ok(expr_to_poly(inner)?.neg()),
        Expr::Add(left, right) => Ok(expr_to_poly(left)?.add(&expr_to_poly(right)?)),
        Expr::Sub(left, right) => Ok(expr_to_poly(left)?.sub(&expr_to_poly(right)?)),
        Expr::Mul(left, right) => Ok(expr_to_poly(left)?.mul(&expr_to_poly(right)?)),
        Expr::Div(left, right) => expr_to_poly(left)?.div_poly(&expr_to_poly(right)?),
        Expr::Pow(base, exp) => {
            let exponent = expr_to_poly(exp)?.constant_value().ok_or_else(|| {
                "E_SYMBOLIC_BAD_POWER 지수는 0 이상의 정수여야 합니다".to_string()
            })?;
            if !exponent.is_integer() || exponent.is_negative() {
                return Err("E_SYMBOLIC_BAD_POWER 지수는 0 이상의 정수여야 합니다".to_string());
            }
            let value = exponent
                .to_integer()
                .to_u32()
                .ok_or_else(|| "E_SYMBOLIC_BAD_POWER 지수가 너무 큽니다".to_string())?;
            Ok(expr_to_poly(base)?.pow_u32(value))
        }
    }
}

fn multiply_monomial(left: &Monomial, right: &Monomial) -> Monomial {
    let mut map: BTreeMap<String, u32> = BTreeMap::new();
    for (name, exp) in left.0.iter().chain(right.0.iter()) {
        *map.entry(name.clone()).or_insert(0) += *exp;
    }
    Monomial(map.into_iter().filter(|(_, exp)| *exp > 0).collect())
}

fn divide_monomial(left: &Monomial, right: &Monomial) -> Option<Monomial> {
    let mut map: BTreeMap<String, i64> = BTreeMap::new();
    for (name, exp) in &left.0 {
        *map.entry(name.clone()).or_insert(0) += i64::from(*exp);
    }
    for (name, exp) in &right.0 {
        *map.entry(name.clone()).or_insert(0) -= i64::from(*exp);
    }
    if map.values().any(|exp| *exp < 0) {
        return None;
    }
    Some(Monomial(
        map.into_iter()
            .filter_map(|(name, exp)| (exp > 0).then_some((name, exp as u32)))
            .collect(),
    ))
}

fn monomial_sort_key(mono: &Monomial) -> (u32, String) {
    let degree = mono.0.iter().map(|(_, exp)| *exp).sum::<u32>();
    let text = mono
        .0
        .iter()
        .map(|(name, exp)| format!("{name}^{exp}"))
        .collect::<Vec<_>>()
        .join("*");
    (degree, text)
}

fn format_term(mono: &Monomial, coeff: &BigRational) -> String {
    let mono_text = format_monomial(mono);
    if mono_text.is_empty() {
        return format_rational(coeff);
    }
    if coeff == &BigRational::one() {
        mono_text
    } else {
        format!("{}*{}", format_rational(coeff), mono_text)
    }
}

fn format_monomial(mono: &Monomial) -> String {
    mono.0
        .iter()
        .map(|(name, exp)| {
            if *exp == 1 {
                name.clone()
            } else {
                format!("{name}^{exp}")
            }
        })
        .collect::<Vec<_>>()
        .join("*")
}

fn format_rational(value: &BigRational) -> String {
    if value.is_integer() {
        value.to_integer().to_string()
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn factor_difference_of_squares(poly: &Polynomial) -> Option<String> {
    if poly.terms.len() != 2 {
        return None;
    }
    let mut square: Option<(String, BigRational)> = None;
    let mut constant: Option<BigRational> = None;
    for (mono, coeff) in &poly.terms {
        if mono.0.len() == 1 && mono.0[0].1 == 2 && coeff == &BigRational::one() {
            square = Some((mono.0[0].0.clone(), coeff.clone()));
        } else if mono.0.is_empty() {
            constant = Some(coeff.clone());
        }
    }
    let (var, _) = square?;
    let c = constant?;
    if c == BigRational::from_integer(BigInt::from(-1)) {
        return Some(format!("({var} - 1)*({var} + 1)"));
    }
    None
}

struct Parser<'a> {
    input: &'a str,
    chars: Vec<char>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<Expr, String> {
        let expr = self.parse_add()?;
        self.skip_ws();
        if self.pos != self.chars.len() {
            return Err(format!(
                "E_SYMBOLIC_PARSE unexpected '{}'",
                self.chars[self.pos]
            ));
        }
        Ok(expr)
    }

    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_mul()?;
        loop {
            self.skip_ws();
            if self.eat('+') {
                expr = Expr::Add(Box::new(expr), Box::new(self.parse_mul()?));
            } else if self.eat('-') {
                expr = Expr::Sub(Box::new(expr), Box::new(self.parse_mul()?));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_pow()?;
        loop {
            self.skip_ws();
            if self.eat('*') {
                expr = Expr::Mul(Box::new(expr), Box::new(self.parse_pow()?));
            } else if self.eat('/') {
                expr = Expr::Div(Box::new(expr), Box::new(self.parse_pow()?));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_pow(&mut self) -> Result<Expr, String> {
        let base = self.parse_unary()?;
        self.skip_ws();
        if self.eat('^') {
            let exp = self.parse_pow()?;
            return Ok(Expr::Pow(Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat('-') {
            return Ok(Expr::Neg(Box::new(self.parse_unary()?)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat('(') {
            let expr = self.parse_add()?;
            self.skip_ws();
            if !self.eat(')') {
                return Err("E_SYMBOLIC_PARSE expected ')'".to_string());
            }
            return Ok(expr);
        }
        if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return self.parse_number();
        }
        if self
            .peek()
            .map(|c| c == '_' || c.is_alphabetic())
            .unwrap_or(false)
        {
            return self.parse_ident();
        }
        Err(format!("E_SYMBOLIC_PARSE bad primary in '{}'", self.input))
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let int = BigInt::parse_bytes(text.as_bytes(), 10)
            .ok_or_else(|| format!("E_SYMBOLIC_NUMBER {text}"))?;
        Ok(Expr::Num(BigRational::from_integer(int)))
    }

    fn parse_ident(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        while self
            .peek()
            .map(|c| c == '_' || c.is_alphanumeric())
            .unwrap_or(false)
        {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        Ok(Expr::Var(text))
    }

    fn skip_ws(&mut self) {
        while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbolic_simplifies_polynomial() {
        assert_eq!(simplify("x + x + 2").unwrap(), "2*x + 2");
    }

    #[test]
    fn symbolic_expands_square() {
        assert_eq!(expand("(x + 1)^2").unwrap(), "x^2 + 2*x + 1");
    }

    #[test]
    fn symbolic_factors_difference_of_squares() {
        assert_eq!(factor("x^2 - 1").unwrap(), "(x - 1)*(x + 1)");
    }

    #[test]
    fn symbolic_diff_and_integral() {
        assert_eq!(diff("x^3 + 2*x", "x").unwrap(), "3*x^2 + 2");
        assert_eq!(integrate("2*x", "x").unwrap(), "x^2");
    }

    #[test]
    fn symbolic_equivalence_normal_form() {
        assert!(equivalent("(x + 1)^2", "x^2 + 2*x + 1").unwrap());
    }

    #[test]
    fn symbolic_solves_linear_equation() {
        let solved = solve_linear_equation("2*x + 3", "7").unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(
            "x".to_string(),
            SolveBinding {
                numerator: "2".to_string(),
                denominator: "1".to_string(),
            },
        );
        assert_eq!(solved, RelationSolveOutcome::Solution(expected));
    }

    #[test]
    fn symbolic_linear_equation_reports_no_solution_and_non_unique() {
        assert_eq!(
            solve_linear_equation("x + 1", "x + 2").unwrap(),
            RelationSolveOutcome::NoSolution
        );
        assert_eq!(
            solve_linear_equation("x + 1", "x + 1").unwrap(),
            RelationSolveOutcome::NonUnique
        );
    }

    #[test]
    fn symbolic_quadratic_relation_reports_single_root_and_non_unique() {
        let solved = solve_relation_equation("x^2 - 4*x + 4", "0").unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(
            "x".to_string(),
            SolveBinding {
                numerator: "2".to_string(),
                denominator: "1".to_string(),
            },
        );
        assert_eq!(solved, RelationSolveOutcome::Solution(expected));
        assert_eq!(
            solve_relation_equation("x^2 - 5*x + 6", "0").unwrap(),
            RelationSolveOutcome::NonUnique
        );
    }

    #[test]
    fn symbolic_solves_linear_system_2x2() {
        let solved = solve_relation_system(&[
            ("x + y".to_string(), "5".to_string()),
            ("x - y".to_string(), "1".to_string()),
        ])
        .unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(
            "x".to_string(),
            SolveBinding {
                numerator: "3".to_string(),
                denominator: "1".to_string(),
            },
        );
        expected.insert(
            "y".to_string(),
            SolveBinding {
                numerator: "2".to_string(),
                denominator: "1".to_string(),
            },
        );
        assert_eq!(solved, RelationSolveOutcome::Solution(expected));
    }

    #[test]
    fn symbolic_relation_holds_with_exact_bindings() {
        let mut bindings = BTreeMap::new();
        bindings.insert(
            "x".to_string(),
            SolveBinding {
                numerator: "3".to_string(),
                denominator: "1".to_string(),
            },
        );
        bindings.insert(
            "y".to_string(),
            SolveBinding {
                numerator: "2".to_string(),
                denominator: "1".to_string(),
            },
        );
        assert!(relation_holds("x + y", "5", &bindings).unwrap());
        assert!(relation_system_holds(
            &[
                ("x + y".to_string(), "5".to_string()),
                ("x - y".to_string(), "1".to_string()),
            ],
            &bindings,
        )
        .unwrap());
        assert!(!relation_holds("x + y", "6", &bindings).unwrap());
    }

    #[test]
    fn symbolic_relation_canon_and_equivalence_work() {
        assert_eq!(relation_canon("x + 1", "0").unwrap(), "x + 1");
        assert!(relation_equivalent("x + 1", "0", "x", "-1").unwrap());
        assert!(!relation_equivalent("x + 1", "0", "x", "1").unwrap());
    }

    #[test]
    fn symbolic_relation_proof_helpers_emit_expected_flags() {
        let cert = prove_relation_equivalent("x + 1", "0", "x", "-1").unwrap();
        assert!(cert.equivalent);
        let mut bindings = BTreeMap::new();
        bindings.insert(
            "x".to_string(),
            SolveBinding {
                numerator: "2".to_string(),
                denominator: "1".to_string(),
            },
        );
        let consistency = prove_relation_solution_consistency(
            &[("2*x + 3".to_string(), "7".to_string())],
            &bindings,
        )
        .unwrap();
        assert!(consistency.consistent);
    }
}
