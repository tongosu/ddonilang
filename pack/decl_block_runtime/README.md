# decl_block_runtime

- 목적: `그릇채비`/`붙박이마련` 선언 규칙과 `붙박이` 재대입 오류를 Gate0(ddonirang-tool) 테스트로 고정한다.
- 테스트 실행: `cargo run -p ddonirang-tool -- test pack/decl_block_runtime/golden`
- 구성:
  - scripts/decl_block_ok.ddn: `그릇채비`에서 `=` 사용 금지 오류 케이스.
  - scripts/const_reassign_error.ddn: `붙박이` 재대입 오류 케이스.
