# Seamgrim Runtime Fallback Surface Contract

## Stable Contract

- 목적:
  - seamgrim runtime fallback 계열의 핵심 surface를 `lesson path fallback + shape fallback mode + motion/projectile fallback + runtime fallback metrics + runtime fallback policy` 기준으로 고정한다.
- compared surface:
  - `tests/run_seamgrim_lesson_path_fallback_check.py`
  - `tests/run_seamgrim_shape_fallback_mode_check.py`
  - `tests/run_seamgrim_motion_projectile_fallback_check.py`
  - `tests/run_seamgrim_runtime_fallback_metrics_check.py`
  - `tests/run_seamgrim_runtime_fallback_policy_check.py`
- pinned rules:
  - `lesson_path_fallback`는 project-prefixed host/path candidate fallback token을 유지한다.
  - `shape_fallback_mode`는 strict default + opt-in fallback gate를 유지한다.
  - `motion_projectile_fallback`는 x/y 관찰치 fallback smoke를 유지한다.
  - `runtime_fallback_metrics`는 metrics report를 생성하고 fallback/native 집계를 통과한다.
  - `runtime_fallback_policy`는 metrics 기반 max fallback ratio 정책을 통과한다.

## Checks

- direct runner:
  - `python tests/run_seamgrim_runtime_fallback_surface_contract_check.py --out build/tmp/seamgrim_runtime_fallback_surface_contract.detjson`
- direct selftest:
  - `python tests/run_seamgrim_runtime_fallback_surface_contract_selftest.py`

## Parent Family

- `tests/seamgrim_guard_surface_family/README.md`
- `python tests/run_seamgrim_guard_surface_family_selftest.py`
