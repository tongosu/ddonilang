use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const FACTOR_JOB_SCHEMA: &str = "ddn.numeric.factor_job.v1";
pub const FACTOR_RESULT_SCHEMA: &str = "ddn.numeric.factor_result.v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactorTerm {
    pub prime: String,
    pub exponent: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactorJob {
    pub schema: String,
    pub input: String,
    pub sign: i8,
    pub remaining: String,
    pub cursor: String,
    pub factors: Vec<FactorTerm>,
    pub status: String,
    pub route: String,
    pub budget_ops_total: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactorResult {
    pub schema: String,
    pub input: String,
    pub status: String,
    pub canonical: Option<String>,
    pub factors: Vec<FactorTerm>,
    pub route: String,
    pub certificate: FactorCertificate,
    pub job_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactorCertificate {
    pub schema: String,
    pub kind: String,
    pub product_matches_input: bool,
    pub prime_checks: Vec<PrimeCheck>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrimeCheck {
    pub value: String,
    pub method: String,
    pub checked_divisors_through: String,
    pub pass: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepOutcome {
    pub job: FactorJob,
    pub result: FactorResult,
}

pub fn new_factor_job(input: &str) -> Result<FactorJob, String> {
    let parsed = parse_bigint(input)?;
    if parsed.is_zero() {
        return Err("E_NUMERIC_FACTOR_ZERO 0은 곱수 분해 대상이 될 수 없습니다".to_string());
    }
    let sign = match parsed.sign() {
        Sign::Minus => -1,
        Sign::NoSign => 0,
        Sign::Plus => 1,
    };
    let remaining = parsed.abs().to_string();
    let mut job = FactorJob {
        schema: FACTOR_JOB_SCHEMA.to_string(),
        input: parsed.to_string(),
        sign,
        remaining,
        cursor: "2".to_string(),
        factors: Vec::new(),
        status: "running".to_string(),
        route: "deterministic:trial-resumable".to_string(),
        budget_ops_total: 0,
        last_error: None,
    };
    normalize_trivial_one(&mut job)?;
    Ok(job)
}

pub fn step_factor_job(mut job: FactorJob, budget_ops: u64) -> Result<StepOutcome, String> {
    validate_schema(&job)?;
    if job.status == "done" || job.status == "blocked" {
        let result = build_result(&job)?;
        return Ok(StepOutcome { job, result });
    }
    let mut remaining = parse_bigint(&job.remaining)?;
    let mut cursor = parse_bigint(&job.cursor)?;
    let two = BigInt::from(2u8);
    if cursor < two {
        cursor = two.clone();
    }

    let mut used = 0u64;
    while used < budget_ops && remaining > BigInt::one() {
        if &cursor * &cursor > remaining {
            push_factor_bigint(&mut job.factors, remaining.clone(), 1);
            remaining = BigInt::one();
            break;
        }
        if (&remaining % &cursor).is_zero() {
            let mut exponent = 0u32;
            while (&remaining % &cursor).is_zero() {
                remaining /= &cursor;
                exponent += 1;
            }
            push_factor_bigint(&mut job.factors, cursor.clone(), exponent);
        } else if cursor == two {
            cursor = BigInt::from(3u8);
        } else {
            cursor += 2u8;
        }
        used += 1;
    }

    job.remaining = remaining.to_string();
    job.cursor = cursor.to_string();
    job.budget_ops_total = job.budget_ops_total.saturating_add(used);
    if remaining == BigInt::one() {
        job.status = "done".to_string();
        job.route = "deterministic:trial-complete".to_string();
    } else {
        job.status = "running".to_string();
    }
    job.factors
        .sort_by(|a, b| numeric_text_cmp(&a.prime, &b.prime));
    let result = build_result(&job)?;
    Ok(StepOutcome { job, result })
}

pub fn complete_factor(input: &str) -> Result<StepOutcome, String> {
    let mut outcome = StepOutcome {
        job: new_factor_job(input)?,
        result: pending_result(input)?,
    };
    while outcome.job.status == "running" {
        outcome = step_factor_job(outcome.job, 250_000)?;
    }
    Ok(outcome)
}

pub fn factor_job_from_json(text: &str) -> Result<FactorJob, String> {
    serde_json::from_str(text).map_err(|e| format!("E_NUMERIC_FACTOR_JOB_PARSE {e}"))
}

pub fn to_detjson<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

pub fn factor_canonical_from_terms(sign: i8, factors: &[FactorTerm]) -> String {
    if factors.is_empty() {
        return if sign < 0 { "-1" } else { "1" }.to_string();
    }
    let body = factors
        .iter()
        .map(|term| {
            if term.exponent == 1 {
                term.prime.clone()
            } else {
                format!("{}^{}", term.prime, term.exponent)
            }
        })
        .collect::<Vec<_>>()
        .join(" * ");
    if sign < 0 {
        format!("-{body}")
    } else {
        body
    }
}

fn pending_result(input: &str) -> Result<FactorResult, String> {
    let job = new_factor_job(input)?;
    build_result(&job)
}

fn build_result(job: &FactorJob) -> Result<FactorResult, String> {
    let canonical = if job.status == "done" {
        Some(factor_canonical_from_terms(job.sign, &job.factors))
    } else {
        None
    };
    let certificate = build_certificate(job)?;
    let job_hash = format!("sha256:{}", sha256_hex(to_detjson(job)?.as_bytes()));
    Ok(FactorResult {
        schema: FACTOR_RESULT_SCHEMA.to_string(),
        input: job.input.clone(),
        status: job.status.clone(),
        canonical,
        factors: job.factors.clone(),
        route: job.route.clone(),
        certificate,
        job_hash,
    })
}

fn build_certificate(job: &FactorJob) -> Result<FactorCertificate, String> {
    let product_matches_input = if job.status == "done" {
        factor_product(job)? == parse_bigint(&job.input)?.abs()
    } else {
        false
    };
    let mut prime_checks = Vec::new();
    if job.status == "done" {
        for factor in &job.factors {
            let value = parse_bigint(&factor.prime)?;
            let pass = is_prime_by_trial(&value);
            let checked = integer_sqrt_floor(&value).to_string();
            prime_checks.push(PrimeCheck {
                value: factor.prime.clone(),
                method: "trial_division_complete".to_string(),
                checked_divisors_through: checked,
                pass,
            });
        }
    }
    Ok(FactorCertificate {
        schema: "ddn.numeric.factor_certificate.v1".to_string(),
        kind: if job.status == "done" {
            "complete_trial_certificate".to_string()
        } else {
            "pending_resume_certificate".to_string()
        },
        product_matches_input,
        prime_checks,
    })
}

fn validate_schema(job: &FactorJob) -> Result<(), String> {
    if job.schema != FACTOR_JOB_SCHEMA {
        return Err(format!("E_NUMERIC_FACTOR_JOB_SCHEMA {}", job.schema));
    }
    Ok(())
}

fn normalize_trivial_one(job: &mut FactorJob) -> Result<(), String> {
    if parse_bigint(&job.remaining)? == BigInt::one() {
        job.status = "done".to_string();
        job.route = "deterministic:trivial".to_string();
    }
    Ok(())
}

fn parse_bigint(input: &str) -> Result<BigInt, String> {
    let trimmed = input.trim().replace('_', "");
    BigInt::parse_bytes(trimmed.as_bytes(), 10)
        .ok_or_else(|| format!("E_NUMERIC_BIGINT_PARSE {input}"))
}

fn push_factor_bigint(out: &mut Vec<FactorTerm>, prime: BigInt, exponent: u32) {
    let key = prime.to_string();
    if let Some(existing) = out.iter_mut().find(|term| term.prime == key) {
        existing.exponent = existing.exponent.saturating_add(exponent);
    } else {
        out.push(FactorTerm {
            prime: key,
            exponent,
        });
    }
}

fn factor_product(job: &FactorJob) -> Result<BigInt, String> {
    let mut out = BigInt::one();
    for term in &job.factors {
        let prime = parse_bigint(&term.prime)?;
        for _ in 0..term.exponent {
            out *= &prime;
        }
    }
    Ok(out)
}

fn is_prime_by_trial(value: &BigInt) -> bool {
    let two = BigInt::from(2u8);
    if value < &two {
        return false;
    }
    if value == &two {
        return true;
    }
    if (value % &two).is_zero() {
        return false;
    }
    let mut divisor = BigInt::from(3u8);
    while &divisor * &divisor <= *value {
        if (value % &divisor).is_zero() {
            return false;
        }
        divisor += 2u8;
    }
    true
}

fn integer_sqrt_floor(value: &BigInt) -> BigInt {
    if value <= &BigInt::zero() {
        return BigInt::zero();
    }
    let mut low = BigInt::zero();
    let mut high = value.clone();
    let one = BigInt::one();
    while low <= high {
        let mid: BigInt = (&low + &high) >> 1;
        let square = &mid * &mid;
        if square <= *value {
            low = &mid + &one;
        } else if mid.is_zero() {
            break;
        } else {
            high = &mid - &one;
        }
    }
    high
}

fn numeric_text_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    match (parse_bigint(a), parse_bigint(b)) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => a.cmp(b),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn exact_universe_hash(fields: BTreeMap<String, String>) -> String {
    let text = serde_json::to_string(&fields).unwrap_or_default();
    format!("sha256:{}", sha256_hex(text.as_bytes()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactText {
    pub num: String,
    pub den: String,
    pub kind: Option<String>,
}

pub fn normalize_rational_text(
    num: &str,
    den: &str,
    kind: Option<String>,
) -> Result<ExactText, String> {
    let n = parse_bigint(num)?;
    let d = parse_bigint(den)?;
    if d.is_zero() {
        return Err("E_NUMERIC_RATIONAL_ZERO_DENOMINATOR".to_string());
    }
    let ratio = BigRational::new(n, d);
    Ok(exact_text_from_ratio(ratio, kind))
}

pub fn exact_binary_text(
    op: &str,
    left: &ExactText,
    right: &ExactText,
) -> Result<ExactText, String> {
    let l = exact_ratio(left)?;
    let r = exact_ratio(right)?;
    let factor_pair = left.kind.as_deref() == Some("곱수") && right.kind.as_deref() == Some("곱수");
    let ratio = match op {
        "+" => l + r,
        "-" => l - r,
        "*" => l * r,
        "/" => {
            if r.is_zero() {
                return Err("E_NUMERIC_DIV_ZERO".to_string());
            }
            l / r
        }
        "%" => {
            if !l.is_integer() || !r.is_integer() {
                return Err("E_NUMERIC_MOD_INTEGER_REQUIRED".to_string());
            }
            let lhs = l.to_integer();
            let rhs = r.to_integer();
            if rhs.is_zero() {
                return Err("E_NUMERIC_DIV_ZERO".to_string());
            }
            BigRational::from_integer(lhs % rhs)
        }
        _ => return Err(format!("E_NUMERIC_EXACT_OP {op}")),
    };
    let kind = if factor_pair && matches!(op, "*" | "/") && ratio.is_integer() {
        Some("곱수".to_string())
    } else if factor_pair && matches!(op, "+" | "-") {
        Some("나눔수".to_string())
    } else if op == "/" || !ratio.is_integer() {
        Some("나눔수".to_string())
    } else {
        Some("큰바른수".to_string())
    };
    Ok(exact_text_from_ratio(ratio, kind))
}

pub fn exact_compare_text(op: &str, left: &ExactText, right: &ExactText) -> Result<bool, String> {
    let ord = exact_ratio(left)?.cmp(&exact_ratio(right)?);
    Ok(match op {
        "==" => ord == std::cmp::Ordering::Equal,
        "!=" => ord != std::cmp::Ordering::Equal,
        "<" => ord == std::cmp::Ordering::Less,
        "<=" => ord != std::cmp::Ordering::Greater,
        ">" => ord == std::cmp::Ordering::Greater,
        ">=" => ord != std::cmp::Ordering::Less,
        _ => return Err(format!("E_NUMERIC_EXACT_COMPARE {op}")),
    })
}

pub fn factor_canon_text_unbounded(raw: &str) -> Result<String, String> {
    let out = complete_factor(raw)?;
    out.result
        .canonical
        .ok_or_else(|| "E_NUMERIC_FACTOR_CANON_PENDING".to_string())
}

fn exact_ratio(value: &ExactText) -> Result<BigRational, String> {
    let num = parse_bigint(&value.num)?;
    let den = parse_bigint(&value.den)?;
    if den.is_zero() {
        return Err("E_NUMERIC_RATIONAL_ZERO_DENOMINATOR".to_string());
    }
    Ok(BigRational::new(num, den))
}

fn exact_text_from_ratio(value: BigRational, kind: Option<String>) -> ExactText {
    ExactText {
        num: value.numer().to_string(),
        den: value.denom().to_string(),
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_kernel_factor_complete_large_power() {
        let input: BigInt = (BigInt::from(1u8) << 520) * BigInt::from(9u8);
        let out = complete_factor(&input.to_string()).expect("factor complete");
        assert_eq!(out.result.status, "done");
        assert_eq!(out.result.canonical.as_deref(), Some("2^520 * 3^2"));
        assert!(out.result.certificate.product_matches_input);
        assert!(out
            .result
            .certificate
            .prime_checks
            .iter()
            .all(|row| row.pass));
    }

    #[test]
    fn numeric_kernel_factor_resume_matches_complete() {
        let input = "7429";
        let complete = complete_factor(input).expect("complete");
        let mut step = step_factor_job(new_factor_job(input).expect("job"), 1).expect("step");
        while step.job.status == "running" {
            step = step_factor_job(step.job, 1).expect("resume");
        }
        assert_eq!(step.result.canonical, complete.result.canonical);
        assert_eq!(step.result.certificate.product_matches_input, true);
    }

    #[test]
    fn numeric_kernel_hard_composite_can_remain_running() {
        let input = "100160063";
        let out = step_factor_job(new_factor_job(input).expect("job"), 2).expect("step");
        assert_eq!(out.result.status, "running");
        assert!(out.result.canonical.is_none());
    }
}
