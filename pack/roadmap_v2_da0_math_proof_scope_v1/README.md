# roadmap_v2_da0_math_proof_scope_v1

ROADMAP_V2 `다-0` math/proof 범위 확정 정합화 pack이다.

이 pack은 새 math runtime, parser, symbolic/proof surface, 제품 UI 변경을 주장하지 않는다. 이미 닫힌 `다-1`~`다-5`의 exact/vector/function, solve/symbolic/proof, graph/view, share, LTS regression evidence를 `다-0` 범위 확정 좌표에 연결한다.

대표 검증:

```sh
python tests/run_pack_golden.py roadmap_v2_da0_math_proof_scope_v1
python tests/run_pack_golden.py math_vector_minimum_first_run_v1 math_calculus_v1 formula_relation_solve_v1 relation_solve_system_2x2_v1 symbolic_relation_canon_v1 symbolic_ddn_formula_bridge_v1 symbolic_rational_expr_v1 symbolic_diff_integral_v1 proof_ddn_relation_bridge_v1 proof_relation_equivalence_v1 proof_runtime_smoke_v1 proof_guard_tick_v1 proof_guard_rollback_v1 proof_alert_continue_v1
python tests/run_roadmap_v2_da0_math_proof_scope_check.py
```
