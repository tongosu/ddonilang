# relation_solve_wasm_cli_parity_v2

- 목적: `방정식풀기` v2 surface가 CLI/tool/WASM에서 같은 relation result pack을 내는지 고정한다.

## representative parity set

- `c01_quadratic_success`
- `c02_system_2x2_success`
- `c03_unsupported_higher_degree`
- `c04_unsupported_quadratic`

## note

- direct formula solve representative와 DDN bridge representative를 이미 고정한 뒤,
  이 pack은 같은 bounded subset이 CLI/tool/WASM rail에서 같은 의미 결과를 내는지 다시 잠근다.
- `unsupported_higher_degree`도 CLI/tool/WASM parity rail에 포함해, representative 4케이스 전체가 어떤 rail에서 검문되는지 같은 case id 기준으로 읽히게 한다.
