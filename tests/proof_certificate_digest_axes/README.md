# Proof Certificate Digest Axes

## Stable Contract

- 목적:
  - `저장형 증명값` 해시 규칙을 아직 정본화하지 않은 상태에서, 현재 proof artifact/cert 아래층에 이미 존재하는 digest 축이 무엇인지 먼저 고정한다.
  - 이 문서는 최종 해시 알고리즘을 선언하지 않고, clean/abort proof pair에서 **같아야 하는 digest**와 **달라야 하는 digest**를 분리한다.
- pack 계약:
  - `pack/proof_certificate_digest_axes_v1/README.md`
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/expected/clean.proof.detjson`
  - `pack/age4_proof_detjson_smoke_v1/expected/abort.proof.detjson`
  - `pack/age4_proof_artifact_cert_subject_v1/cert_cases.json`
  - `tests/proof_artifact_certificate_contract/README.md`
- selftest:
  - `python tests/run_proof_certificate_digest_axes_selftest.py`
  - `proof_certificate_digest_axes_selftest`

## Matrix

| axis | clean vs abort | meaning |
| --- | --- | --- |
| `subject_hash(sha256(file bytes))` | different | cert layer는 proof artifact 전체 바이트를 subject로 본다 |
| `canonical_body_hash` | different | 본문 정본 digest는 성공/abort 본문 차이를 반영한다 |
| `state_hash` | different | 실행 결과 상태 차이를 반영한다 |
| `trace_hash` | different | trace 차이를 반영한다 |
| `proof_runtime_hash` | same | 최소 smoke pair에서는 runtime proof summary 축이 같다 |
| `solver_translation_hash` | same | 최소 smoke pair에서는 solver translation 축이 같다 |

## Consumer Surface

- `tests/proof_certificate_candidate_manifest/README.md`
- `tests/proof_certificate_v1_promotion_candidate/README.md`
- `tests/proof_artifact_certificate_contract/README.md`
- `pack/age4_proof_artifact_cert_subject_v1/README.md`
- `python tests/run_proof_certificate_digest_axes_selftest.py`
- `python tests/run_proof_certificate_candidate_manifest_selftest.py`
- `python tests/run_proof_certificate_v1_promotion_candidate_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
