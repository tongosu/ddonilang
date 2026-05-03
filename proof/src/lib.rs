use num_bigint::{BigInt, Sign};
use num_traits::One;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const VERIFY_REPORT_SCHEMA: &str = "ddn.proof.verify_report.v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyReport {
    pub schema: String,
    pub valid: bool,
    pub kind: String,
    pub detail: String,
    pub report_hash: String,
}

pub fn verify_json_text(text: &str) -> Result<VerifyReport, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| format!("E_PROOF_JSON_PARSE {e}"))?;
    verify_value(&value)
}

pub fn verify_value(value: &Value) -> Result<VerifyReport, String> {
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let (valid, kind, detail) = match schema {
        "ddn.proof.symbolic_rewrite.v1" => verify_symbolic_rewrite(value)?,
        "ddn.symbolic.equivalence_certificate.v1" => verify_symbolic_equivalence(value)?,
        "ddn.symbolic.relation_equivalence_certificate.v1" => verify_relation_equivalence(value)?,
        "ddn.symbolic.relation_solve_consistency_certificate.v1" => {
            verify_relation_solve_consistency(value)?
        }
        "ddn.proof.numeric_factor_certificate.v1" => verify_numeric_factor(value)?,
        "ddn.numeric.factor_result.v1" => verify_numeric_factor_result(value)?,
        "ddn.proof.seum_bridge.v1" => verify_seum_bridge(value)?,
        other => (
            false,
            "unsupported".to_string(),
            format!("E_PROOF_SCHEMA_UNSUPPORTED {other}"),
        ),
    };
    Ok(report(valid, kind, detail))
}

pub fn to_detjson<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|e| e.to_string())
}

fn verify_symbolic_equivalence(value: &Value) -> Result<(bool, String, String), String> {
    let lhs = required_str(value, "lhs")?;
    let rhs = required_str(value, "rhs")?;
    let expected = value
        .get("equivalent")
        .and_then(Value::as_bool)
        .ok_or_else(|| "E_PROOF_FIELD equivalent".to_string())?;
    let actual = ddonirang_symbolic::equivalent(lhs, rhs)?;
    Ok((
        actual == expected,
        "symbolic_equivalence".to_string(),
        format!("equivalent={actual} expected={expected}"),
    ))
}

fn verify_symbolic_rewrite(value: &Value) -> Result<(bool, String, String), String> {
    let steps = value
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| "E_PROOF_FIELD steps".to_string())?;
    for (idx, step) in steps.iter().enumerate() {
        let from = required_str(step, "from")?;
        let to = required_str(step, "to")?;
        if !ddonirang_symbolic::equivalent(from, to)? {
            return Ok((
                false,
                "symbolic_rewrite".to_string(),
                format!("step {idx} is not equivalent"),
            ));
        }
    }
    Ok((
        true,
        "symbolic_rewrite".to_string(),
        format!("steps={}", steps.len()),
    ))
}

fn verify_relation_equivalence(value: &Value) -> Result<(bool, String, String), String> {
    let first = value
        .get("first")
        .ok_or_else(|| "E_PROOF_FIELD first".to_string())?;
    let second = value
        .get("second")
        .ok_or_else(|| "E_PROOF_FIELD second".to_string())?;
    let first_lhs = required_str(first, "lhs")?;
    let first_rhs = required_str(first, "rhs")?;
    let second_lhs = required_str(second, "lhs")?;
    let second_rhs = required_str(second, "rhs")?;
    let expected = value
        .get("equivalent")
        .and_then(Value::as_bool)
        .ok_or_else(|| "E_PROOF_FIELD equivalent".to_string())?;
    let actual =
        ddonirang_symbolic::relation_equivalent(first_lhs, first_rhs, second_lhs, second_rhs)?;
    Ok((
        actual == expected,
        "relation_equivalence".to_string(),
        format!("equivalent={actual} expected={expected}"),
    ))
}

fn verify_relation_solve_consistency(value: &Value) -> Result<(bool, String, String), String> {
    let equations = value
        .get("equations")
        .and_then(Value::as_array)
        .ok_or_else(|| "E_PROOF_FIELD equations".to_string())?;
    let bindings_value = value
        .get("bindings")
        .and_then(Value::as_object)
        .ok_or_else(|| "E_PROOF_FIELD bindings".to_string())?;
    let expected = value
        .get("consistent")
        .and_then(Value::as_bool)
        .ok_or_else(|| "E_PROOF_FIELD consistent".to_string())?;
    let mut parsed_equations = Vec::new();
    for equation in equations {
        let lhs = required_str(equation, "lhs")?;
        let rhs = required_str(equation, "rhs")?;
        parsed_equations.push((lhs.to_string(), rhs.to_string()));
    }
    let mut bindings = std::collections::BTreeMap::new();
    for (name, raw_binding) in bindings_value {
        let numerator = required_str(raw_binding, "numerator")?;
        let denominator = required_str(raw_binding, "denominator")?;
        bindings.insert(
            name.clone(),
            ddonirang_symbolic::SolveBinding {
                numerator: numerator.to_string(),
                denominator: denominator.to_string(),
            },
        );
    }
    let actual = ddonirang_symbolic::relation_system_holds(&parsed_equations, &bindings)?;
    Ok((
        actual == expected,
        "relation_solve_consistency".to_string(),
        format!(
            "consistent={actual} expected={expected} equations={}",
            parsed_equations.len()
        ),
    ))
}

fn verify_seum_bridge(value: &Value) -> Result<(bool, String, String), String> {
    let claims = value
        .get("claims")
        .and_then(Value::as_array)
        .ok_or_else(|| "E_PROOF_FIELD claims".to_string())?;
    for (idx, claim) in claims.iter().enumerate() {
        let report = verify_value(claim)?;
        if !report.valid {
            return Ok((
                false,
                "seum_bridge".to_string(),
                format!("claim {idx} failed: {}", report.detail),
            ));
        }
    }
    Ok((
        true,
        "seum_bridge".to_string(),
        format!("claims={}", claims.len()),
    ))
}

fn verify_numeric_factor(value: &Value) -> Result<(bool, String, String), String> {
    let input = required_str(value, "input")?;
    let factors = value
        .get("factors")
        .and_then(Value::as_array)
        .ok_or_else(|| "E_PROOF_FIELD factors".to_string())?;
    verify_factor_terms(input, factors)
}

fn verify_numeric_factor_result(value: &Value) -> Result<(bool, String, String), String> {
    let input = required_str(value, "input")?;
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if status != "done" {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            format!("status={status}"),
        ));
    }
    let factors = value
        .get("factors")
        .and_then(Value::as_array)
        .ok_or_else(|| "E_PROOF_FIELD factors".to_string())?;
    let (product_valid, _, product_detail) = verify_factor_terms(input, factors)?;
    if !product_valid {
        return Ok((false, "numeric_factor_result".to_string(), product_detail));
    }
    let route = value
        .get("route")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if route.trim().is_empty() {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            "missing route".to_string(),
        ));
    }
    let job_hash = value
        .get("job_hash")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !job_hash.starts_with("sha256:") {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            "missing job_hash".to_string(),
        ));
    }
    // Proof-side verifier stops at route/hash/certificate consistency.
    // Pollard/Rho-style factor discovery strategy and bounded equation solve stay outside this line.
    let certificate = value
        .get("certificate")
        .ok_or_else(|| "E_PROOF_FIELD certificate".to_string())?;
    if certificate.get("schema").and_then(Value::as_str)
        != Some("ddn.numeric.factor_certificate.v1")
    {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            "bad certificate schema".to_string(),
        ));
    }
    if certificate
        .get("product_matches_input")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            "certificate product mismatch".to_string(),
        ));
    }
    let Some(prime_checks) = certificate.get("prime_checks").and_then(Value::as_array) else {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            "missing prime_checks".to_string(),
        ));
    };
    if prime_checks.len() != factors.len() {
        return Ok((
            false,
            "numeric_factor_result".to_string(),
            "prime_checks length mismatch".to_string(),
        ));
    }
    for check in prime_checks {
        if check.get("pass").and_then(Value::as_bool) != Some(true) {
            return Ok((
                false,
                "numeric_factor_result".to_string(),
                "prime check failed".to_string(),
            ));
        }
        if check
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .is_empty()
        {
            return Ok((
                false,
                "numeric_factor_result".to_string(),
                "prime check method missing".to_string(),
            ));
        }
    }
    Ok((
        true,
        "numeric_factor_result".to_string(),
        format!("route={route} prime_checks={}", prime_checks.len()),
    ))
}

fn verify_factor_terms(input: &str, factors: &[Value]) -> Result<(bool, String, String), String> {
    let target = parse_bigint(input)?.abs();
    let mut product = BigInt::one();
    for item in factors {
        let prime = required_str(item, "prime")?;
        let exponent = item
            .get("exponent")
            .and_then(Value::as_u64)
            .ok_or_else(|| "E_PROOF_FIELD exponent".to_string())?;
        let factor = parse_bigint(prime)?;
        if factor <= BigInt::one() {
            return Ok((
                false,
                "numeric_factor".to_string(),
                format!("non-factor {factor}"),
            ));
        }
        for _ in 0..exponent {
            product *= &factor;
        }
    }
    Ok((
        product == target,
        "numeric_factor".to_string(),
        format!("product_matches_input={}", product == target),
    ))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("E_PROOF_FIELD {key}"))
}

fn parse_bigint(input: &str) -> Result<BigInt, String> {
    let trimmed = input.trim().replace('_', "");
    BigInt::parse_bytes(trimmed.as_bytes(), 10)
        .ok_or_else(|| format!("E_PROOF_BIGINT_PARSE {input}"))
}

fn report(valid: bool, kind: String, detail: String) -> VerifyReport {
    let base = format!("{VERIFY_REPORT_SCHEMA}\n{valid}\n{kind}\n{detail}");
    VerifyReport {
        schema: VERIFY_REPORT_SCHEMA.to_string(),
        valid,
        kind,
        detail,
        report_hash: format!("sha256:{}", sha256_hex(base.as_bytes())),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

trait BigIntAbs {
    fn abs(self) -> Self;
}

impl BigIntAbs for BigInt {
    fn abs(self) -> Self {
        match self.sign() {
            Sign::Minus => -self,
            Sign::NoSign | Sign::Plus => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn proof_verifies_symbolic_equivalence_certificate() {
        let cert = ddonirang_symbolic::prove_equivalent("(x+1)^2", "x^2+2*x+1").unwrap();
        let value = serde_json::to_value(cert).unwrap();
        assert!(verify_value(&value).unwrap().valid);
    }

    #[test]
    fn proof_verifies_symbolic_rewrite_steps() {
        let value = json!({
            "schema": "ddn.proof.symbolic_rewrite.v1",
            "steps": [
                {"from": "(x+1)^2", "to": "x^2 + 2*x + 1"}
            ]
        });
        assert!(verify_value(&value).unwrap().valid);
    }

    #[test]
    fn proof_verifies_numeric_factor_product() {
        let value = json!({
            "schema": "ddn.proof.numeric_factor_certificate.v1",
            "input": "91",
            "factors": [
                {"prime": "7", "exponent": 1},
                {"prime": "13", "exponent": 1}
            ]
        });
        assert!(verify_value(&value).unwrap().valid);
    }

    #[test]
    fn proof_verifies_relation_equivalence_certificate() {
        let cert = ddonirang_symbolic::prove_relation_equivalent("x + 1", "0", "x", "-1").unwrap();
        let value = serde_json::to_value(cert).unwrap();
        let report = verify_value(&value).unwrap();
        assert!(report.valid);
        assert_eq!(report.kind, "relation_equivalence");
    }

    #[test]
    fn proof_verifies_relation_solve_consistency_certificate() {
        let mut bindings = std::collections::BTreeMap::new();
        bindings.insert(
            "x".to_string(),
            ddonirang_symbolic::SolveBinding {
                numerator: "2".to_string(),
                denominator: "1".to_string(),
            },
        );
        let cert = ddonirang_symbolic::prove_relation_solution_consistency(
            &[("2*x + 3".to_string(), "7".to_string())],
            &bindings,
        )
        .unwrap();
        let value = serde_json::to_value(cert).unwrap();
        let report = verify_value(&value).unwrap();
        assert!(report.valid);
        assert_eq!(report.kind, "relation_solve_consistency");
    }
}
