# Proof Certificate V1 Runtime Emit

## Stable Contract

- 목적:
  - `teul-cli run --proof-out`가 기존 `ddn.proof.detjson.v0` 옆에 실제 `proof_certificate_v1` runtime sidecar를 함께 쓰는 최소 emit 경로를 고정한다.
  - 이 문서는 test-only 후보 fixture를 그대로 재사용하지 않고, 실제 실행 출력에서 바로 얻을 수 있는 runtime candidate/artifact shape만 검증한다.
- 대상 surface:
  - `pack/age4_proof_detjson_smoke_v1/input.ddn`
  - `pack/age4_proof_detjson_smoke_v1/input_abort.ddn`
  - `pack/proof_certificate_v1_runtime_emit_v1/README.md`
  - `tests/proof_certificate_v1_draft_contract/README.md`
  - `tests/proof_certificate_v1_promotion/README.md`
  - `tests/proof_certificate_v1_family/README.md`
  - `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
  - `python tests/run_proof_certificate_v1_runtime_emit_pack_check.py`
  - `proof_certificate_v1_runtime_emit_selftest`

## Runtime Sidecars

- candidate:
  - schema: `ddn.proof_certificate_v1_runtime_candidate.v1`
  - naming: `<proof-out stem>.proof_certificate_v1_candidate.detjson`
- artifact:
  - schema: `ddn.proof_certificate_v1_runtime_draft_artifact.v1`
  - naming: `<proof-out stem>.proof_certificate_v1_draft_artifact.detjson`

## Runtime Candidate Fields

- `schema`
- `source_proof_path`
- `source_proof_schema`
- `source_proof_kind`
- `profile`
- `cert_manifest_schema`
- `cert_algo`
- `verified`
- `contract_diag_count`
- `proof_subject_hash`
- `canonical_body_hash`
- `proof_runtime_hash`
- `solver_translation_hash`
- `state_hash`
- `trace_hash`

## Runtime Artifact Fields

- `schema`
- `source_proof_path`
- `profile`
- `shared_shell_key_count`
- `state_delta_key_count`
- `candidate_manifest`
- `shared_shell`
- `state_delta`

## Consumer Surface

- `pack/age4_proof_detjson_smoke_v1/README.md`
- `pack/proof_certificate_v1_runtime_emit_v1/README.md`
- `tests/proof_certificate_v1_signed_emit/README.md`
- `tests/proof_certificate_v1_signed_contract/README.md`
- `tests/proof_certificate_v1_draft_contract/README.md`
- `tests/proof_certificate_v1_promotion/README.md`
- `tests/proof_certificate_v1_family/README.md`
- `python tests/run_proof_certificate_v1_runtime_emit_selftest.py`
- `python tests/run_proof_certificate_v1_runtime_emit_pack_check.py`
- `python tests/run_proof_certificate_v1_signed_emit_selftest.py`
- `python tests/run_proof_certificate_v1_signed_contract_selftest.py`
- `python tests/run_proof_certificate_v1_family_selftest.py`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
