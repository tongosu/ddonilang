# age4_proof_detjson_smoke_v1

`--proof-out`으로 최소 `ddn.proof.detjson.v0` 증명서를 발급하는 AGE4 회귀 팩.

검증:

- `python tests/run_pack_golden.py age4_proof_detjson_smoke_v1`
- `python tests/run_pack_golden.py --update age4_proof_detjson_smoke_v1`
- proof artifact/cert contract:
  - `tests/proof_artifact_certificate_contract/README.md`
  - `python tests/run_proof_artifact_certificate_contract_selftest.py`
- proof artifact cert bridge pack:
  - `pack/age4_proof_artifact_cert_subject_v1/README.md`
  - `python tests/run_w95_cert_pack_check.py --pack pack/age4_proof_artifact_cert_subject_v1`
- proof certificate runtime emit:
  - `tests/proof_certificate_v1_runtime_emit/README.md`
  - `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
- proof certificate signed emit:
  - `tests/proof_certificate_v1_signed_emit/README.md`
  - `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
- proof certificate signed emit profile parity:
  - `tests/proof_certificate_v1_signed_emit_profiles/README.md`
  - `python tests/run_proof_certificate_v1_signed_emit_profile_selftest.py`
- proof certificate bundle verify:
  - `tests/proof_certificate_v1_verify_bundle/README.md`
  - `python tests/run_proof_certificate_v1_verify_bundle_selftest.py`
- proof certificate verify report:
  - `tests/proof_certificate_v1_verify_report/README.md`
  - `python tests/run_proof_certificate_v1_verify_report_selftest.py`
- proof certificate consumer contract:
  - `tests/proof_certificate_v1_consumer_contract/README.md`
  - `python tests/run_proof_certificate_v1_consumer_contract_selftest.py`
  - `python tests/run_proof_certificate_v1_consumer_transport_contract_selftest.py`
- proof certificate signed contract:
  - `tests/proof_certificate_v1_signed_contract/README.md`
  - `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- proof certificate promotion:
  - `tests/proof_certificate_v1_promotion/README.md`
  - `python tests/run_proof_certificate_v1_promotion_selftest.py`
- proof certificate family:
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_family_selftest.py`
  - `tests/proof_certificate_family/README.md`
  - `python tests/run_proof_certificate_family_selftest.py`
