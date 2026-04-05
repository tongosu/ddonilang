# W95 cert pack intent

- 대상: `teul-cli cert keygen|sign|verify`
- 핵심 계약:
  - 동일 입력 + 동일 seed keygen -> 동일 subject_hash/signature
  - proof 변조(1바이트) 시 verify가 실패해야 한다.
  - 실패 시 `E_CERT_VERIFY_FAIL` 코드를 노출해야 한다.
