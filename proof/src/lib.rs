use num_bigint::{BigInt, Sign};
use num_traits::One;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const VERIFY_REPORT_SCHEMA: &str = "ddn.proof.verify_report.v1";
pub const DEFINEDNESS_CHECKER_ID: &str = "ddn.proof.pathir-definedness-checker.v1";
pub const DEFINEDNESS_RECEIPT_PROFILE_ID: &str = "ddn.proof.pathir-definedness-receipt.v1";
pub const DEFINEDNESS_INTERNAL_REQUEST_SCHEMA_VERSION: &str =
    "ddn.proof.pathir-definedness.internal-request.v1";
pub const DEFINEDNESS_CHECK_EXECUTION_CONTRACT_ID: &str =
    "ddn.proof.pathir-definedness-check-execution.v1";
pub const DEFINEDNESS_REPORT_PROFILE_ID: &str = "ddn.proof.pathir-definedness-report.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckerIdentity(String);

impl CheckerIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn internal_candidate(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckExecutionIdentity(String);

impl CheckExecutionIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckResultIdentity(String);

impl CheckResultIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportIdentity(String);

impl ReportIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiptIdentity(String);

impl ReceiptIdentity {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinednessCheckResult {
    pub proof_problem_identity: ddonirang_symbolic::ProofProblemIdentity,
    pub declared_assumption_closure_identity: String,
    pub transformation_obligation_set_identity: String,
    pub certificate_identity: ddonirang_symbolic::CertificateIdentity,
    pub provider_identity: ddonirang_symbolic::ProviderIdentity,
    pub checker_identity: CheckerIdentity,
    pub kernel_profile_identity: ddonirang_symbolic::KernelProfileIdentity,
    pub check_execution_identity: CheckExecutionIdentity,
    pub verdict: bool,
    pub check_result_identity: CheckResultIdentity,
}

/// Reversible internal typed ingress. This is deliberately not serialized and
/// does not choose the OPEN public registry, CLI option or WASM field spelling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinednessCheckRequest {
    schema_version: String,
    source_snapshot_identity: String,
    source: String,
    declared_assumption_closure: ddonirang_symbolic::DeclaredAssumptionClosure,
    provider_identity: ddonirang_symbolic::ProviderIdentity,
    checker_identity: CheckerIdentity,
    kernel_profile_identity: ddonirang_symbolic::KernelProfileIdentity,
    state_hash: String,
    provenance: String,
}

impl DefinednessCheckRequest {
    pub fn internal_candidate(
        source_snapshot_identity: &str,
        source: &str,
        declared_assumption_closure: ddonirang_symbolic::DeclaredAssumptionClosure,
        state_hash: &str,
        provenance: &str,
    ) -> Self {
        Self {
            schema_version: DEFINEDNESS_INTERNAL_REQUEST_SCHEMA_VERSION.to_string(),
            source_snapshot_identity: source_snapshot_identity.to_string(),
            source: source.to_string(),
            declared_assumption_closure,
            provider_identity: ddonirang_symbolic::ProviderIdentity::internal_candidate(
                ddonirang_symbolic::DEFINEDNESS_PROVIDER_ID,
            ),
            checker_identity: CheckerIdentity::internal_candidate(DEFINEDNESS_CHECKER_ID),
            kernel_profile_identity: ddonirang_symbolic::KernelProfileIdentity::internal_candidate(
                ddonirang_symbolic::DEFINEDNESS_PROFILE_ID,
            ),
            state_hash: state_hash.to_string(),
            provenance: provenance.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinednessReport {
    pub proof_problem_identity: ddonirang_symbolic::ProofProblemIdentity,
    pub certificate_identity: ddonirang_symbolic::CertificateIdentity,
    pub check_execution_identity: CheckExecutionIdentity,
    pub check_result_identity: CheckResultIdentity,
    pub result_hash: String,
    pub verdict: bool,
    pub report_identity: ReportIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishEvidence {
    pub check_completed_before_publish: bool,
    pub result_publish_count: u32,
    pub state_publish_count: u32,
    pub outbox_release_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinednessReceipt {
    pub source_snapshot_identity: String,
    pub proof_problem_identity: ddonirang_symbolic::ProofProblemIdentity,
    pub certificate_identity: ddonirang_symbolic::CertificateIdentity,
    pub check_execution_identity: CheckExecutionIdentity,
    pub check_result_identity: CheckResultIdentity,
    pub report_identity: ReportIdentity,
    pub provider_identity: ddonirang_symbolic::ProviderIdentity,
    pub checker_identity: CheckerIdentity,
    pub kernel_profile_identity: ddonirang_symbolic::KernelProfileIdentity,
    pub pre_state_hash: String,
    pub post_state_hash: String,
    pub result_hash: String,
    pub publish_evidence: PublishEvidence,
    pub provenance: String,
    pub receipt_identity: ReceiptIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefinednessExecution {
    pub problem: ddonirang_symbolic::DefinednessProofProblem,
    pub certificate: ddonirang_symbolic::DefinednessCertificate,
    pub check_result: DefinednessCheckResult,
    pub report: DefinednessReport,
    pub receipt: DefinednessReceipt,
}

fn recompute_check_execution_identity(
    problem: &ddonirang_symbolic::DefinednessProofProblem,
    certificate: &ddonirang_symbolic::DefinednessCertificate,
    checker_identity: &CheckerIdentity,
) -> CheckExecutionIdentity {
    CheckExecutionIdentity(format!(
        "sha256:{}",
        sha256_hex(
            format!(
                "definedness-check-execution\n{}\n{}\n{}\n{}\n{}\n{}",
                DEFINEDNESS_CHECK_EXECUTION_CONTRACT_ID,
                problem.proof_problem_identity.as_str(),
                certificate.certificate_identity.as_str(),
                problem.provider_identity.as_str(),
                checker_identity.as_str(),
                problem.kernel_profile_identity.as_str(),
            )
            .as_bytes()
        )
    ))
}

fn recompute_check_result_identity(
    problem: &ddonirang_symbolic::DefinednessProofProblem,
    certificate: &ddonirang_symbolic::DefinednessCertificate,
    checker_identity: &CheckerIdentity,
    check_execution_identity: &CheckExecutionIdentity,
    verdict: bool,
) -> CheckResultIdentity {
    CheckResultIdentity(format!(
        "sha256:{}",
        sha256_hex(
            format!(
                "definedness-check-result\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                problem.proof_problem_identity.as_str(),
                problem.declared_assumption_closure.identity(),
                problem.transformation_obligations.identity(),
                certificate.certificate_identity.as_str(),
                problem.provider_identity.as_str(),
                checker_identity.as_str(),
                problem.kernel_profile_identity.as_str(),
                check_execution_identity.as_str(),
                if verdict { "pass" } else { "fail" },
            )
            .as_bytes()
        )
    ))
}

fn recompute_report_identity(
    check_result: &DefinednessCheckResult,
    result_hash: &str,
) -> ReportIdentity {
    ReportIdentity(format!(
        "sha256:{}",
        sha256_hex(
            format!(
                "definedness-report\n{}\n{}\n{}\n{}\n{}\n{}",
                DEFINEDNESS_REPORT_PROFILE_ID,
                check_result.proof_problem_identity.as_str(),
                check_result.certificate_identity.as_str(),
                check_result.check_execution_identity.as_str(),
                check_result.check_result_identity.as_str(),
                result_hash,
            )
            .as_bytes()
        )
    ))
}

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

    #[test]
    fn proof_rejects_unconditional_definedness_losing_equivalence() {
        let unsafe_certificate = json!({
            "schema": "ddn.symbolic.equivalence_certificate.v1",
            "lhs": "x^2/x",
            "rhs": "x",
            "lhs_canonical": "x",
            "rhs_canonical": "x",
            "equivalent": true,
            "method": "mathir_polynomial_normal_form",
            "certificate_hash": "historical-unsafe-custody"
        });
        let error = verify_value(&unsafe_certificate)
            .expect_err("definedness obligation 없는 과거 certificate는 fail-closed해야 함");
        assert!(error.starts_with("E_SYMBOLIC_DEFINEDNESS_OBLIGATION_REQUIRED"));
    }

    fn checked_definedness_pair(
        source: &str,
        evidence: &[&str],
    ) -> (
        ddonirang_symbolic::DefinednessProofProblem,
        ddonirang_symbolic::DefinednessCertificate,
    ) {
        let closure =
            ddonirang_symbolic::DeclaredAssumptionClosure::from_declared_nonzero(evidence).unwrap();
        ddonirang_symbolic::create_definedness_certificate(source, &closure).unwrap()
    }

    fn seal_test_receipt(
        publish_evidence: PublishEvidence,
        pre_state_hash: &str,
        post_state_hash: &str,
    ) -> Result<DefinednessReceipt, String> {
        let (problem, certificate) = checked_definedness_pair("x^2/x", &["x"]);
        let checked = check_definedness_certificate(&problem, &certificate).unwrap();
        let report = build_definedness_report(&certificate, &checked).unwrap();
        seal_definedness_receipt(
            "sha256:source",
            &problem,
            &certificate,
            &checked,
            &report,
            pre_state_hash,
            post_state_hash,
            &report.result_hash,
            publish_evidence,
            "test",
        )
    }

    #[test]
    fn p1_checked_definedness_certificate_checker_and_receipt_are_bound() {
        for (source, evidence) in [("x^2/x", "x"), ("(x^2-1)/(x-1)", "x-1")] {
            let (problem, certificate) = checked_definedness_pair(source, &[evidence]);
            let checked = check_definedness_certificate(&problem, &certificate).unwrap();
            let report = build_definedness_report(&certificate, &checked).unwrap();
            let publish = PublishEvidence {
                check_completed_before_publish: true,
                result_publish_count: 1,
                state_publish_count: 0,
                outbox_release_count: 0,
            };
            let first = seal_definedness_receipt(
                "sha256:source-snapshot",
                &problem,
                &certificate,
                &checked,
                &report,
                "sha256:unchanged-state",
                "sha256:unchanged-state",
                &report.result_hash,
                publish.clone(),
                "fixed-test-provenance",
            )
            .unwrap();
            let second = seal_definedness_receipt(
                "sha256:source-snapshot",
                &problem,
                &certificate,
                &checked,
                &report,
                "sha256:unchanged-state",
                "sha256:unchanged-state",
                &report.result_hash,
                publish,
                "fixed-test-provenance",
            )
            .unwrap();
            assert_eq!(first.receipt_identity, second.receipt_identity);
            assert!(checked.verdict);
        }
    }

    #[test]
    fn p1_checker_rejects_cross_closure_and_recomputed_forgery() {
        let (problem_x, certificate_x) = checked_definedness_pair("x^2/x", &["x"]);
        let (problem_xy, _) = checked_definedness_pair("x^2/x", &["x", "y"]);
        assert!(check_definedness_certificate(&problem_xy, &certificate_x)
            .unwrap_err()
            .starts_with("E_PATHIR_CERTIFICATE_PROBLEM_MISMATCH_INTERNAL"));

        // P1-M08: even a self-consistent forged certificate identity cannot
        // turn a prover-selected wrong result into checker PASS.
        let mut forged_result = certificate_x.clone();
        forged_result.result = "x + 1".to_string();
        forged_result.certificate_identity =
            ddonirang_symbolic::recompute_definedness_certificate_identity(&forged_result);
        assert_eq!(
            check_definedness_certificate(&problem_x, &forged_result).unwrap_err(),
            "E_PATHIR_REWRITE_SEMANTICS_MISMATCH_INTERNAL"
        );

        // Byte/content mutation without an identity refresh is rejected first.
        let mut byte_mutation = certificate_x.clone();
        byte_mutation.result.push(' ');
        assert_eq!(
            check_definedness_certificate(&problem_x, &byte_mutation).unwrap_err(),
            "E_PATHIR_CERTIFICATE_IDENTITY_INTERNAL"
        );
        check_definedness_certificate(&problem_x, &certificate_x).unwrap();
    }

    #[test]
    fn p1_profile_check_result_and_publish_mutations_fail_closed() {
        let (problem, certificate) = checked_definedness_pair("x^2/x", &["x"]);

        // P1-M09: provider/checker/profile changes do not inherit PASS.
        let mut wrong_profile = certificate.clone();
        wrong_profile.kernel_profile_identity =
            ddonirang_symbolic::KernelProfileIdentity::internal_candidate("wrong-profile");
        wrong_profile.certificate_identity =
            ddonirang_symbolic::recompute_definedness_certificate_identity(&wrong_profile);
        assert_eq!(
            check_definedness_certificate(&problem, &wrong_profile).unwrap_err(),
            "E_PATHIR_CERTIFICATE_PROFILE_MISMATCH_INTERNAL"
        );

        let checked = check_definedness_certificate(&problem, &certificate).unwrap();
        let report = build_definedness_report(&certificate, &checked).unwrap();
        let publish = PublishEvidence {
            check_completed_before_publish: true,
            result_publish_count: 1,
            state_publish_count: 0,
            outbox_release_count: 0,
        };

        let mut wrong_provider = checked.clone();
        wrong_provider.provider_identity =
            ddonirang_symbolic::ProviderIdentity::internal_candidate("wrong-provider");
        assert_eq!(
            seal_definedness_receipt(
                "sha256:source",
                &problem,
                &certificate,
                &wrong_provider,
                &report,
                "sha256:unchanged-state",
                "sha256:unchanged-state",
                &report.result_hash,
                publish.clone(),
                "test",
            )
            .unwrap_err(),
            "E_PATHIR_RECEIPT_CHECK_BINDING_INTERNAL"
        );

        let mut wrong_checker = checked.clone();
        wrong_checker.checker_identity = CheckerIdentity::internal_candidate("wrong-checker");
        assert_eq!(
            seal_definedness_receipt(
                "sha256:source",
                &problem,
                &certificate,
                &wrong_checker,
                &report,
                "sha256:unchanged-state",
                "sha256:unchanged-state",
                &report.result_hash,
                publish.clone(),
                "test",
            )
            .unwrap_err(),
            "E_PATHIR_RECEIPT_CHECK_BINDING_INTERNAL"
        );

        let mut wrong_execution = checked.clone();
        wrong_execution.check_execution_identity.0 = "sha256:wrong-execution".to_string();
        wrong_execution.check_result_identity = recompute_check_result_identity(
            &problem,
            &certificate,
            &wrong_execution.checker_identity,
            &wrong_execution.check_execution_identity,
            wrong_execution.verdict,
        );
        assert_eq!(
            seal_definedness_receipt(
                "sha256:source",
                &problem,
                &certificate,
                &wrong_execution,
                &report,
                "sha256:unchanged-state",
                "sha256:unchanged-state",
                &report.result_hash,
                publish.clone(),
                "test",
            )
            .unwrap_err(),
            "E_PATHIR_CHECK_EXECUTION_IDENTITY_INTERNAL"
        );

        // P1-M10: a receipt cannot omit or forge the check-result identity.
        let mut forged_check = checked.clone();
        forged_check.check_result_identity.0 = "sha256:forged".to_string();
        assert_eq!(
            seal_definedness_receipt(
                "sha256:source",
                &problem,
                &certificate,
                &forged_check,
                &report,
                "sha256:unchanged-state",
                "sha256:unchanged-state",
                &report.result_hash,
                publish.clone(),
                "test",
            )
            .unwrap_err(),
            "E_PATHIR_CHECK_RESULT_IDENTITY_INTERNAL"
        );

        // P1-M13: publication before checker completion is never receipted.
        let early_publish = PublishEvidence {
            check_completed_before_publish: false,
            result_publish_count: 1,
            state_publish_count: 1,
            outbox_release_count: 1,
        };
        assert_eq!(
            seal_definedness_receipt(
                "sha256:source",
                &problem,
                &certificate,
                &checked,
                &report,
                "sha256:pre",
                "sha256:post",
                &report.result_hash,
                early_publish,
                "test",
            )
            .unwrap_err(),
            "E_PATHIR_PUBLISH_BEFORE_CHECK_INTERNAL"
        );

        seal_definedness_receipt(
            "sha256:source",
            &problem,
            &certificate,
            &checked,
            &report,
            "sha256:unchanged-state",
            "sha256:unchanged-state",
            &report.result_hash,
            publish,
            "test",
        )
        .unwrap();
    }

    #[test]
    fn p1_receipt_rejects_result_publish_count_other_than_one() {
        for result_publish_count in [0, 2] {
            let error = seal_test_receipt(
                PublishEvidence {
                    check_completed_before_publish: true,
                    result_publish_count,
                    state_publish_count: 0,
                    outbox_release_count: 0,
                },
                "sha256:unchanged-state",
                "sha256:unchanged-state",
            )
            .unwrap_err();
            assert_eq!(error, "E_PATHIR_RESULT_PUBLISH_COUNT_INTERNAL");
        }
    }

    #[test]
    fn p1_receipt_rejects_state_publish() {
        let error = seal_test_receipt(
            PublishEvidence {
                check_completed_before_publish: true,
                result_publish_count: 1,
                state_publish_count: 1,
                outbox_release_count: 0,
            },
            "sha256:unchanged-state",
            "sha256:unchanged-state",
        )
        .unwrap_err();
        assert_eq!(error, "E_PATHIR_STATE_PUBLISH_INTERNAL");
    }

    #[test]
    fn p1_receipt_rejects_outbox_release() {
        let error = seal_test_receipt(
            PublishEvidence {
                check_completed_before_publish: true,
                result_publish_count: 1,
                state_publish_count: 0,
                outbox_release_count: 1,
            },
            "sha256:unchanged-state",
            "sha256:unchanged-state",
        )
        .unwrap_err();
        assert_eq!(error, "E_PATHIR_OUTBOX_RELEASE_INTERNAL");
    }

    #[test]
    fn p1_receipt_rejects_pure_profile_state_change() {
        let error = seal_test_receipt(
            PublishEvidence {
                check_completed_before_publish: true,
                result_publish_count: 1,
                state_publish_count: 0,
                outbox_release_count: 0,
            },
            "sha256:pre-state",
            "sha256:changed-state",
        )
        .unwrap_err();
        assert_eq!(error, "E_PATHIR_PURE_PROOF_STATE_MUTATION_INTERNAL");
    }

    #[test]
    fn p1_receipt_rejects_checker_incomplete_single_axis() {
        let error = seal_test_receipt(
            PublishEvidence {
                check_completed_before_publish: false,
                result_publish_count: 1,
                state_publish_count: 0,
                outbox_release_count: 0,
            },
            "sha256:unchanged-state",
            "sha256:unchanged-state",
        )
        .unwrap_err();
        assert_eq!(error, "E_PATHIR_PUBLISH_BEFORE_CHECK_INTERNAL");
    }

    #[test]
    fn p1_receipt_rejects_reviewer_compound_partial_publish_counterexample() {
        let error = seal_test_receipt(
            PublishEvidence {
                check_completed_before_publish: true,
                result_publish_count: 1,
                state_publish_count: 1,
                outbox_release_count: 1,
            },
            "sha256:pre-state",
            "sha256:changed-state",
        )
        .unwrap_err();
        assert_eq!(error, "E_PATHIR_STATE_PUBLISH_INTERNAL");
    }

    #[test]
    fn p1_internal_checked_request_binds_execution_report_and_receipt_chain() {
        let closure =
            ddonirang_symbolic::DeclaredAssumptionClosure::from_declared_nonzero(&["x"]).unwrap();
        let request = DefinednessCheckRequest::internal_candidate(
            "sha256:source-snapshot",
            "x^2/x",
            closure,
            "sha256:unchanged-state",
            "fixed-test-provenance",
        );

        let first = execute_definedness_checked_internal(&request).unwrap();
        let second = execute_definedness_checked_internal(&request).unwrap();

        assert_eq!(
            first.problem.proof_problem_identity,
            second.problem.proof_problem_identity
        );
        assert_eq!(
            first.certificate.certificate_identity,
            second.certificate.certificate_identity
        );
        assert_eq!(
            first.check_result.check_execution_identity,
            second.check_result.check_execution_identity
        );
        assert_eq!(
            first.check_result.check_result_identity,
            second.check_result.check_result_identity
        );
        assert_eq!(first.report.report_identity, second.report.report_identity);
        assert_eq!(
            first.receipt.receipt_identity,
            second.receipt.receipt_identity
        );
        assert_eq!(first.receipt.report_identity, first.report.report_identity);
        assert_eq!(first.receipt.publish_evidence.result_publish_count, 1);
        assert_eq!(first.receipt.publish_evidence.state_publish_count, 0);
        assert_eq!(first.receipt.publish_evidence.outbox_release_count, 0);
    }

    #[test]
    fn p1_internal_request_unknown_version_and_assumption_loss_fail_closed() {
        let closure =
            ddonirang_symbolic::DeclaredAssumptionClosure::from_declared_nonzero(&["x"]).unwrap();
        let mut request = DefinednessCheckRequest::internal_candidate(
            "sha256:source-snapshot",
            "x^2/x",
            closure,
            "sha256:unchanged-state",
            "fixed-test-provenance",
        );

        // P1-M11: an unknown internal request version cannot fall back.
        request.schema_version = "unknown-internal-version".to_string();
        assert_eq!(
            execute_definedness_checked_internal(&request).unwrap_err(),
            "E_PATHIR_REQUEST_SCHEMA_VERSION_INTERNAL"
        );

        // The same typed ingress without the checked assumption cannot reach
        // checker/report/receipt success. Public frontdoor M12 remains blocked
        // until its exact physical registry is Owner-approved.
        let missing = DefinednessCheckRequest::internal_candidate(
            "sha256:source-snapshot",
            "x^2/x",
            ddonirang_symbolic::DeclaredAssumptionClosure::empty(),
            "sha256:unchanged-state",
            "fixed-test-provenance",
        );
        assert!(execute_definedness_checked_internal(&missing)
            .unwrap_err()
            .contains("nonzero(x)"));
    }

    #[test]
    fn p1_diagnostic_text_is_not_an_assumption_ingress() {
        // P1-M14: an internal diagnostic string is data, not checked evidence.
        let error = ddonirang_symbolic::DeclaredAssumptionClosure::from_declared_nonzero(&[
            "E_SYMBOLIC_DEFINEDNESS_OBLIGATION_REQUIRED_INTERNAL nonzero(x)",
        ])
        .unwrap_err();
        assert!(error.starts_with("E_SYMBOLIC_PARSE"), "{error}");
    }
}

pub fn check_definedness_certificate(
    problem: &ddonirang_symbolic::DefinednessProofProblem,
    certificate: &ddonirang_symbolic::DefinednessCertificate,
) -> Result<DefinednessCheckResult, String> {
    if !problem.declared_assumption_closure.identity_is_valid() {
        return Err("E_PATHIR_ASSUMPTION_CLOSURE_IDENTITY_INTERNAL".to_string());
    }
    if !problem.transformation_obligations.identity_is_valid() {
        return Err("E_PATHIR_OBLIGATION_SET_IDENTITY_INTERNAL".to_string());
    }
    if problem.proof_problem_identity
        != ddonirang_symbolic::recompute_definedness_problem_identity(problem)
    {
        return Err("E_PATHIR_PROOF_PROBLEM_IDENTITY_INTERNAL".to_string());
    }
    if problem.provider_identity.as_str() != ddonirang_symbolic::DEFINEDNESS_PROVIDER_ID {
        return Err("E_PATHIR_PROVIDER_IDENTITY_INTERNAL".to_string());
    }
    if problem.kernel_profile_identity.as_str() != ddonirang_symbolic::DEFINEDNESS_PROFILE_ID {
        return Err("E_PATHIR_KERNEL_PROFILE_IDENTITY_INTERNAL".to_string());
    }
    if certificate.proof_problem_identity != problem.proof_problem_identity {
        return Err("E_PATHIR_CERTIFICATE_PROBLEM_MISMATCH_INTERNAL".to_string());
    }
    if certificate.declared_assumption_closure_identity
        != problem.declared_assumption_closure.identity()
    {
        return Err("E_PATHIR_CERTIFICATE_CLOSURE_MISMATCH_INTERNAL".to_string());
    }
    if certificate.transformation_obligation_set_identity
        != problem.transformation_obligations.identity()
    {
        return Err("E_PATHIR_CERTIFICATE_OBLIGATION_MISMATCH_INTERNAL".to_string());
    }
    if certificate.provider_identity != problem.provider_identity {
        return Err("E_PATHIR_CERTIFICATE_PROVIDER_MISMATCH_INTERNAL".to_string());
    }
    if certificate.kernel_profile_identity != problem.kernel_profile_identity {
        return Err("E_PATHIR_CERTIFICATE_PROFILE_MISMATCH_INTERNAL".to_string());
    }
    if certificate.certificate_identity
        != ddonirang_symbolic::recompute_definedness_certificate_identity(certificate)
    {
        return Err("E_PATHIR_CERTIFICATE_IDENTITY_INTERNAL".to_string());
    }

    // The checker deliberately recomputes the PathIR/type/definedness relation.
    // It never consumes a prover boolean or trusts the prover's canonical result.
    ddonirang_symbolic::independently_check_definedness_rewrite(problem, &certificate.result)?;

    let checker_identity = CheckerIdentity(DEFINEDNESS_CHECKER_ID.to_string());
    let check_execution_identity =
        recompute_check_execution_identity(problem, certificate, &checker_identity);
    let check_result_identity = recompute_check_result_identity(
        problem,
        certificate,
        &checker_identity,
        &check_execution_identity,
        true,
    );
    Ok(DefinednessCheckResult {
        proof_problem_identity: problem.proof_problem_identity.clone(),
        declared_assumption_closure_identity: problem
            .declared_assumption_closure
            .identity()
            .to_string(),
        transformation_obligation_set_identity: problem
            .transformation_obligations
            .identity()
            .to_string(),
        certificate_identity: certificate.certificate_identity.clone(),
        provider_identity: problem.provider_identity.clone(),
        checker_identity,
        kernel_profile_identity: problem.kernel_profile_identity.clone(),
        check_execution_identity,
        verdict: true,
        check_result_identity,
    })
}

pub fn build_definedness_report(
    certificate: &ddonirang_symbolic::DefinednessCertificate,
    check_result: &DefinednessCheckResult,
) -> Result<DefinednessReport, String> {
    if !check_result.verdict {
        return Err("E_PATHIR_CHECK_FAILURE_NOT_REPORTABLE_INTERNAL".to_string());
    }
    if check_result.certificate_identity != certificate.certificate_identity {
        return Err("E_PATHIR_REPORT_CERTIFICATE_BINDING_INTERNAL".to_string());
    }
    let result_hash = format!(
        "sha256:{}",
        sha256_hex(format!("definedness-result\n{}", certificate.result).as_bytes())
    );
    let report_identity = recompute_report_identity(check_result, &result_hash);
    Ok(DefinednessReport {
        proof_problem_identity: check_result.proof_problem_identity.clone(),
        certificate_identity: certificate.certificate_identity.clone(),
        check_execution_identity: check_result.check_execution_identity.clone(),
        check_result_identity: check_result.check_result_identity.clone(),
        result_hash,
        verdict: true,
        report_identity,
    })
}

pub fn seal_definedness_receipt(
    source_snapshot_identity: &str,
    problem: &ddonirang_symbolic::DefinednessProofProblem,
    certificate: &ddonirang_symbolic::DefinednessCertificate,
    check_result: &DefinednessCheckResult,
    report: &DefinednessReport,
    pre_state_hash: &str,
    post_state_hash: &str,
    result_hash: &str,
    publish_evidence: PublishEvidence,
    provenance: &str,
) -> Result<DefinednessReceipt, String> {
    if !check_result.verdict {
        return Err("E_PATHIR_CHECK_FAILURE_NOT_RECEIPTABLE_INTERNAL".to_string());
    }
    if check_result.proof_problem_identity != problem.proof_problem_identity
        || check_result.certificate_identity != certificate.certificate_identity
        || check_result.provider_identity != problem.provider_identity
        || check_result.kernel_profile_identity != problem.kernel_profile_identity
        || check_result.checker_identity.as_str() != DEFINEDNESS_CHECKER_ID
    {
        return Err("E_PATHIR_RECEIPT_CHECK_BINDING_INTERNAL".to_string());
    }
    if check_result.check_result_identity
        != recompute_check_result_identity(
            problem,
            certificate,
            &check_result.checker_identity,
            &check_result.check_execution_identity,
            check_result.verdict,
        )
    {
        return Err("E_PATHIR_CHECK_RESULT_IDENTITY_INTERNAL".to_string());
    }
    if check_result.check_execution_identity
        != recompute_check_execution_identity(problem, certificate, &check_result.checker_identity)
    {
        return Err("E_PATHIR_CHECK_EXECUTION_IDENTITY_INTERNAL".to_string());
    }
    if report.proof_problem_identity != problem.proof_problem_identity
        || report.certificate_identity != certificate.certificate_identity
        || report.check_execution_identity != check_result.check_execution_identity
        || report.check_result_identity != check_result.check_result_identity
        || !report.verdict
        || report.report_identity != recompute_report_identity(check_result, &report.result_hash)
    {
        return Err("E_PATHIR_REPORT_BINDING_INTERNAL".to_string());
    }
    let expected_result_hash = format!(
        "sha256:{}",
        sha256_hex(format!("definedness-result\n{}", certificate.result).as_bytes())
    );
    if report.result_hash != expected_result_hash || result_hash != expected_result_hash {
        return Err("E_PATHIR_REPORT_RESULT_IDENTITY_INTERNAL".to_string());
    }
    if !publish_evidence.check_completed_before_publish {
        return Err("E_PATHIR_PUBLISH_BEFORE_CHECK_INTERNAL".to_string());
    }
    if publish_evidence.result_publish_count != 1 {
        return Err("E_PATHIR_RESULT_PUBLISH_COUNT_INTERNAL".to_string());
    }
    if publish_evidence.state_publish_count != 0 {
        return Err("E_PATHIR_STATE_PUBLISH_INTERNAL".to_string());
    }
    if publish_evidence.outbox_release_count != 0 {
        return Err("E_PATHIR_OUTBOX_RELEASE_INTERNAL".to_string());
    }
    if pre_state_hash != post_state_hash {
        return Err("E_PATHIR_PURE_PROOF_STATE_MUTATION_INTERNAL".to_string());
    }
    for (field, value) in [
        ("source_snapshot_identity", source_snapshot_identity),
        ("pre_state_hash", pre_state_hash),
        ("post_state_hash", post_state_hash),
        ("result_hash", result_hash),
        ("provenance", provenance),
    ] {
        if value.trim().is_empty() {
            return Err(format!("E_PATHIR_RECEIPT_FIELD_INTERNAL {field}"));
        }
    }
    let receipt_identity = ReceiptIdentity(format!(
        "sha256:{}",
        sha256_hex(
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                DEFINEDNESS_RECEIPT_PROFILE_ID,
                source_snapshot_identity,
                problem.proof_problem_identity.as_str(),
                certificate.certificate_identity.as_str(),
                check_result.check_execution_identity.as_str(),
                check_result.check_result_identity.as_str(),
                report.report_identity.as_str(),
                problem.provider_identity.as_str(),
                check_result.checker_identity.as_str(),
                problem.kernel_profile_identity.as_str(),
                pre_state_hash,
                post_state_hash,
                result_hash,
                publish_evidence.result_publish_count,
                publish_evidence.state_publish_count,
                publish_evidence.outbox_release_count,
                provenance,
            )
            .as_bytes()
        )
    ));
    Ok(DefinednessReceipt {
        source_snapshot_identity: source_snapshot_identity.to_string(),
        proof_problem_identity: problem.proof_problem_identity.clone(),
        certificate_identity: certificate.certificate_identity.clone(),
        check_execution_identity: check_result.check_execution_identity.clone(),
        check_result_identity: check_result.check_result_identity.clone(),
        report_identity: report.report_identity.clone(),
        provider_identity: problem.provider_identity.clone(),
        checker_identity: check_result.checker_identity.clone(),
        kernel_profile_identity: problem.kernel_profile_identity.clone(),
        pre_state_hash: pre_state_hash.to_string(),
        post_state_hash: post_state_hash.to_string(),
        result_hash: result_hash.to_string(),
        publish_evidence,
        provenance: provenance.to_string(),
        receipt_identity,
    })
}

/// Runs the full checked-assumption product seam without serializing the OPEN
/// public wire. No result/state/outbox evidence is created before checker PASS.
pub fn execute_definedness_checked_internal(
    request: &DefinednessCheckRequest,
) -> Result<DefinednessExecution, String> {
    if request.schema_version != DEFINEDNESS_INTERNAL_REQUEST_SCHEMA_VERSION {
        return Err("E_PATHIR_REQUEST_SCHEMA_VERSION_INTERNAL".to_string());
    }
    for (field, value) in [
        (
            "source_snapshot_identity",
            request.source_snapshot_identity.as_str(),
        ),
        ("source", request.source.as_str()),
        ("state_hash", request.state_hash.as_str()),
        ("provenance", request.provenance.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("E_PATHIR_REQUEST_FIELD_INTERNAL {field}"));
        }
    }
    if request.provider_identity.as_str() != ddonirang_symbolic::DEFINEDNESS_PROVIDER_ID {
        return Err("E_PATHIR_REQUEST_PROVIDER_IDENTITY_INTERNAL".to_string());
    }
    if request.checker_identity.as_str() != DEFINEDNESS_CHECKER_ID {
        return Err("E_PATHIR_REQUEST_CHECKER_IDENTITY_INTERNAL".to_string());
    }
    if request.kernel_profile_identity.as_str() != ddonirang_symbolic::DEFINEDNESS_PROFILE_ID {
        return Err("E_PATHIR_REQUEST_KERNEL_PROFILE_IDENTITY_INTERNAL".to_string());
    }

    let (problem, certificate) = ddonirang_symbolic::create_definedness_certificate(
        &request.source,
        &request.declared_assumption_closure,
    )?;
    let check_result = check_definedness_certificate(&problem, &certificate)?;
    if check_result.provider_identity != request.provider_identity
        || check_result.checker_identity != request.checker_identity
        || check_result.kernel_profile_identity != request.kernel_profile_identity
    {
        return Err("E_PATHIR_REQUEST_EXECUTION_IDENTITY_INTERNAL".to_string());
    }
    let report = build_definedness_report(&certificate, &check_result)?;
    let publish_evidence = PublishEvidence {
        check_completed_before_publish: true,
        result_publish_count: 1,
        state_publish_count: 0,
        outbox_release_count: 0,
    };
    let receipt = seal_definedness_receipt(
        &request.source_snapshot_identity,
        &problem,
        &certificate,
        &check_result,
        &report,
        &request.state_hash,
        &request.state_hash,
        &report.result_hash,
        publish_evidence,
        &request.provenance,
    )?;
    Ok(DefinednessExecution {
        problem,
        certificate,
        check_result,
        report,
        receipt,
    })
}
