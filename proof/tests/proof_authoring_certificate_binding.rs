use ddonirang_proof::{
    build_definedness_report, check_definedness_certificate, seal_definedness_receipt,
    PublishEvidence,
};

fn checked_pair() -> (
    ddonirang_symbolic::DefinednessProofProblem,
    ddonirang_symbolic::DefinednessCertificate,
) {
    let closure =
        ddonirang_symbolic::DeclaredAssumptionClosure::from_declared_nonzero(&["x"]).unwrap();
    ddonirang_symbolic::create_definedness_certificate("x^2/x", &closure).unwrap()
}

#[test]
fn report_rejects_certificate_changed_after_check() {
    let (problem, mut certificate) = checked_pair();
    let checked = check_definedness_certificate(&problem, &certificate).unwrap();
    let original_report = build_definedness_report(&certificate, &checked).unwrap();
    let original = certificate.clone();
    certificate.result = "x + 1".to_string();
    assert_eq!(
        build_definedness_report(&certificate, &checked).unwrap_err(),
        "E_PATHIR_CERTIFICATE_IDENTITY_INTERNAL"
    );
    // 복원된 원 증표는 기존 결과와 같은 보고서를 만든다.
    assert_eq!(build_definedness_report(&original, &checked).unwrap(), original_report);
}

#[test]
fn refreshed_forgery_cannot_reuse_old_check() {
    let (problem, mut certificate) = checked_pair();
    let checked = check_definedness_certificate(&problem, &certificate).unwrap();
    certificate.result = "x + 1".to_string();
    certificate.certificate_identity =
        ddonirang_symbolic::recompute_definedness_certificate_identity(&certificate);
    assert_eq!(
        build_definedness_report(&certificate, &checked).unwrap_err(),
        "E_PATHIR_REPORT_CERTIFICATE_BINDING_INTERNAL"
    );
    assert_eq!(
        check_definedness_certificate(&problem, &certificate).unwrap_err(),
        "E_PATHIR_REWRITE_SEMANTICS_MISMATCH_INTERNAL"
    );
}

#[test]
fn receipt_rechecks_certificate_content_even_with_original_report() {
    let (problem, mut certificate) = checked_pair();
    let checked = check_definedness_certificate(&problem, &certificate).unwrap();
    let report = build_definedness_report(&certificate, &checked).unwrap();
    let original = certificate.clone();
    let seal = |certificate: &ddonirang_symbolic::DefinednessCertificate| {
        seal_definedness_receipt(
            "source", &problem, certificate, &checked, &report, "state", "state",
            &report.result_hash,
            PublishEvidence {
                check_completed_before_publish: true,
                result_publish_count: 1,
                state_publish_count: 0,
                outbox_release_count: 0,
            },
            "proof-authoring-test",
        )
    };
    let original_receipt = seal(&original).unwrap();
    // result와 저장된 certificate identity는 그대로 두고 provider만 바꾼다.
    certificate.provider_identity =
        ddonirang_symbolic::ProviderIdentity::internal_candidate("unsupported-provider");
    assert_eq!(seal(&certificate).unwrap_err(), "E_PATHIR_CERTIFICATE_IDENTITY_INTERNAL");
    assert_eq!(seal(&original).unwrap(), original_receipt);
}
