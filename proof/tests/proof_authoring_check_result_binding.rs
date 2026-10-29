use ddonirang_proof::{
    build_definedness_report, check_definedness_certificate, seal_definedness_receipt,
    PublishEvidence,
};

fn pair(
    source: &str,
    assumptions: &[&str],
) -> (
    ddonirang_symbolic::DefinednessProofProblem,
    ddonirang_symbolic::DefinednessCertificate,
) {
    let closure =
        ddonirang_symbolic::DeclaredAssumptionClosure::from_declared_nonzero(assumptions).unwrap();
    ddonirang_symbolic::create_definedness_certificate(source, &closure).unwrap()
}

#[test]
fn issued_result_remains_readable_and_usable() {
    let (problem, certificate) = pair("x^2/x", &["x"]);
    let checked = check_definedness_certificate(&problem, &certificate).unwrap();
    assert!(checked.verdict());
    assert_eq!(
        checked.proof_problem_identity(),
        &problem.proof_problem_identity
    );
    assert_eq!(
        checked.certificate_identity(),
        &certificate.certificate_identity
    );
    let report = build_definedness_report(&certificate, &checked).unwrap();
    assert_eq!(
        report,
        build_definedness_report(&certificate, &checked.clone()).unwrap()
    );
    let receipt = seal_definedness_receipt(
        "source",
        &problem,
        &certificate,
        &checked,
        &report,
        "state",
        "state",
        &report.result_hash,
        PublishEvidence {
            check_completed_before_publish: true,
            result_publish_count: 1,
            state_publish_count: 0,
            outbox_release_count: 0,
        },
        "test",
    )
    .unwrap();
    assert_eq!(
        receipt.certificate_identity,
        certificate.certificate_identity
    );
}

#[test]
fn issued_result_cannot_be_transplanted_to_another_problem_or_closure() {
    let (problem, certificate) = pair("x^2/x", &["x"]);
    let checked = check_definedness_certificate(&problem, &certificate).unwrap();
    for (other_problem, other_certificate) in [
        pair("y^2/y", &["y"]),
        pair("x^2/x", &["x", "y"]),
        pair("(x^2-1)/(x-1)", &["x-1"]),
    ] {
        check_definedness_certificate(&other_problem, &other_certificate).unwrap();
        assert_eq!(
            build_definedness_report(&other_certificate, &checked).unwrap_err(),
            "E_PATHIR_REPORT_CERTIFICATE_BINDING_INTERNAL"
        );
    }
}

#[test]
fn recomputed_forgery_and_unknown_profile_still_require_real_checker() {
    let (problem, certificate) = pair("x^2/x", &["x"]);
    let checked = check_definedness_certificate(&problem, &certificate).unwrap();
    let mut forged = certificate.clone();
    forged.result = "x + 1".into();
    forged.certificate_identity =
        ddonirang_symbolic::recompute_definedness_certificate_identity(&forged);
    assert_eq!(
        check_definedness_certificate(&problem, &forged).unwrap_err(),
        "E_PATHIR_REWRITE_SEMANTICS_MISMATCH_INTERNAL"
    );
    assert!(build_definedness_report(&forged, &checked).is_err());
    let mut unsupported = certificate;
    unsupported.kernel_profile_identity =
        ddonirang_symbolic::KernelProfileIdentity::internal_candidate("unknown");
    unsupported.certificate_identity =
        ddonirang_symbolic::recompute_definedness_certificate_identity(&unsupported);
    assert_eq!(
        check_definedness_certificate(&problem, &unsupported).unwrap_err(),
        "E_PATHIR_CERTIFICATE_PROFILE_MISMATCH_INTERNAL"
    );
    assert!(build_definedness_report(&unsupported, &checked).is_err());
}
