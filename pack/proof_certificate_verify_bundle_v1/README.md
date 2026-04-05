# proof_certificate_verify_bundle_v1

`proof certificate verify bundle` consumer surface를 clean/abort 두 profile의 relative path 실행으로 고정하는 pack.

## 계약
- `teul-cli cert verify-proof-certificate --in <bundle>`가 clean/abort 두 profile에서 결정적인 stdout을 재생성해야 한다.
- `keygen --seed` 고정값과 relative path 실행으로 `cert_subject_hash/cert_pubkey`까지 expected stdout과 exact match여야 한다.
- tampered bundle verify는 반드시 `E_PROOF_CERT_VERIFY_SUBJECT_HASH_MISMATCH`로 실패해야 한다.

## 구성
- `fixtures/input.ddn`
- `fixtures/input_abort.ddn`
- `expected/clean.stdout.txt`
- `expected/abort.stdout.txt`
