# seamgrim_intro_exec_blocky_v1

라-1 교재 1권 입문 rail을 block editor palette data로 조립하고 DDN으로 생성하는 기준 pack이다.

필수 조건:

- generated DDN은 `teul-cli canon --emit ddn`과 `teul-cli run`을 통과한다.
- generated DDN은 `seamgrim_wasm_cli_runtime_parity_runner.mjs`로 WASM direct runtime을 통과한다.
- `raw_block_count`는 모든 케이스에서 0이다.

