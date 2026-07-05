# MA_LESSON_REPAIR_GOLDEN_V1

작성일: 2026-07-06
브랜치: `codex/queue-20260706`
범위: 기존 실질 lesson pack 2개에 `golden.jsonl` 추가. lesson 내용, 코드, 파서, 런타임 수정 없음.

## 대상

| pack | 변경 | 근거 |
|---|---|---|
| `pack/edu_seamgrim_rep_phys_projectile_xy_v1` | `golden.jsonl` 추가 | `teul-cli run pack/edu_seamgrim_rep_phys_projectile_xy_v1/lesson.ddn` 실제 출력 |
| `pack/edu_seamgrim_rep_econ_supply_demand_tax_v1` | `golden.jsonl` 추가 | `teul-cli run pack/edu_seamgrim_rep_econ_supply_demand_tax_v1/lesson.ddn` 실제 출력 |

## 실행 로그

- 직접 실행 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/ma-lesson-repair/golden/`
- `edu_seamgrim_rep_phys_projectile_xy_v1.teul-cli-run.log`
- `edu_seamgrim_rep_econ_supply_demand_tax_v1.teul-cli-run.log`
- `edu_seamgrim_rep_phys_projectile_xy_v1.pack-golden.log`
- `edu_seamgrim_rep_econ_supply_demand_tax_v1.pack-golden.log`

## 실제 출력값

| pack | state_hash | trace_hash | bogae_hash |
|---|---|---|---|
| `edu_seamgrim_rep_phys_projectile_xy_v1` | `blake3:349745dfcb0a6944acaa02db955721c8833bd6508cecd5f4ebbd7862ec67f8a3` | `blake3:073ca02588c60314761aa6df83e866fcfd055117615d581e98ade471b2f1050f` | `blake3:46356cd030f74fa13a0fcbfb1de09e372c21aa7408031c06fb3199aabbaa8bde` |
| `edu_seamgrim_rep_econ_supply_demand_tax_v1` | `blake3:2eb6105578f83c34b8017fea29a297736792e22bda6aa86b365e1346d5e4c3e6` | `blake3:f254d2cbb90f8e8134eaa17978310bd95ce785f519338a52f6a60357ce981890` | `blake3:46356cd030f74fa13a0fcbfb1de09e372c21aa7408031c06fb3199aabbaa8bde` |

## 검증

| 명령 | 결과 |
|---|---|
| `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- run pack/edu_seamgrim_rep_phys_projectile_xy_v1/lesson.ddn` | PASS(exit 0) |
| `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- run pack/edu_seamgrim_rep_econ_supply_demand_tax_v1/lesson.ddn` | PASS(exit 0) |
| `python tests/run_pack_golden.py edu_seamgrim_rep_phys_projectile_xy_v1` | PASS |
| `python tests/run_pack_golden.py edu_seamgrim_rep_econ_supply_demand_tax_v1` | PASS |

메모: 현 `run_pack_golden.py`는 `run` 명령 stdout에서 `state_hash`와 `trace_hash`를 비교 대상에서 제외한다. 따라서 `golden.jsonl`은 실제 출력의 `bogae_hash` 라인을 고정하고, 전체 원출력은 위 로그와 표에 남겼다.
