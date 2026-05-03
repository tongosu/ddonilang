# relation_solve_ddn_bridge_v2

- 목적: DDN runtime rail에서 `=:= relation` input surface와 `방정식풀기` bounded subset bridge를 representative case 단위로 고정한다.

## current-line boundary

- `=:= relation`
  - minimum_landed 쪽 relation surface
- `방정식풀기`
  - bounded subset solve bridge
  - representative subset:
    - 2차식
    - 2x2 exact system

## representative rows

| row | input | expected surface | unsupported reason | parity source |
| --- | --- | --- | --- | --- |
| `c01_quadratic_success` | `input_quadratic_success.ddn` | `#성공(미지수="x", 값=2)` | - | `direct_formula_bridge`, `cli_tool_wasm` |
| `c02_system_2x2_success` | `input_system_2x2_success.ddn` | `#성공(해=(x=3, y=2))` | - | `direct_formula_bridge`, `cli_tool_wasm` |
| `c03_unsupported_higher_degree` | `input_unsupported_higher_degree.ddn` | `#실패(사유="unsupported")` | `higher_degree` | `cli_tool_wasm` |
| `c04_unsupported_quadratic` | `input_unsupported_quadratic.ddn` | `#실패(사유="unsupported")` | `unsupported_quadratic` | `direct_formula_bridge`, `cli_tool_wasm` |

## parity rails

| rail | representative pack | representative ids |
| --- | --- | --- |
| direct formula/system | `formula_relation_solve_quadratic_v1`, `relation_solve_system_2x2_v1` | `c01_quadratic_success`, `c02_system_2x2_success`, `c04_unsupported_quadratic` |
| DDN bridge | `relation_solve_ddn_bridge_v2` | `c01_quadratic_success`, `c02_system_2x2_success`, `c03_unsupported_higher_degree`, `c04_unsupported_quadratic` |
| CLI/tool/WASM parity | `relation_solve_wasm_cli_parity_v2` | `c01_quadratic_success`, `c02_system_2x2_success`, `c03_unsupported_higher_degree`, `c04_unsupported_quadratic` |

## note

- `formula_relation_solve_*` family는 symbolic-side direct solve representative다.
- `relation_solve_ddn_bridge_v2`는 같은 bounded subset을 DDN runtime surface에서 다시 잠그는 bridge다.
- `relation_solve_wasm_cli_parity_v2`는 같은 bounded subset을 CLI/tool/WASM rail에서 다시 잠그는 parity representative다.
- representative 4케이스는 direct formula/system golden, DDN bridge stdout, CLI/tool/WASM parity rail 중 어디에서 다시 검문되는지 `parity source`로 함께 적는다.
- proof check는 solver 본체가 아니라 solve consistency line에 남는다.

## non-goals

- generalized solver
- full DAE
- theorem prover화된 solve expansion
