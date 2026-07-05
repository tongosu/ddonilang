# MA_LESSON_REPAIR_MAEGIM_MIGRATION_V1

작성일: 2026-07-06
브랜치: `codex/queue-20260706`
범위: 기존 seed lesson 2개의 legacy `// 범위(...)` 주석을 `채비 {}` 안의 `매김 {}` 문법으로 이전. 코드, 파서, 런타임 수정 없음.

## 대상

| lesson | 변경 |
|---|---|
| `solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn` | `(시작)할때` 안의 8개 기본 파라미터 range 주석을 `채비 {}`의 `매김 { 범위: .. 간격: .. }` 선언으로 이전 |
| `solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_tax_shock_supply_demand_seed_v1/lesson.ddn` | `채비:`의 10개 range 주석을 정본 `채비 {}` + `매김 {}` 선언으로 이전하고, 동일 기본값을 중복 대입하던 top-level 초기화는 `채비` 기본값으로 흡수 |

모델 로직인 `(매마디)마다`의 방정식, 조건, 출력/모양 생성은 변경하지 않았다.

## 실행 로그

- 로그 경로: `I:/home/urihanl/ddn/codex/out/queue-20260706/ma-lesson-repair/maegim-migration/`
- `physics_pendulum_seed_v1.teul-cli-run.log`
- `econ_tax_shock_supply_demand_seed_v1.teul-cli-run.log`

## 검증

| 명령 | 결과 | hash |
|---|---|---|
| `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- run solutions/seamgrim_ui_mvp/seed_lessons_v1/physics_pendulum_seed_v1/lesson.ddn` | PASS(exit 0), `E_LEGACY_RANGE_SYNTAX` 없음 | `state_hash=blake3:2b1ff7b1613e69d02ead6a19a849dfe1973936e83ba86b51461583b3f4622ee8`, `trace_hash=blake3:920df98797acef337592f26017c38fb848e5497996dc2af3366f54c24efb79cd` |
| `cargo run -q --manifest-path tools/teul-cli/Cargo.toml -- run solutions/seamgrim_ui_mvp/seed_lessons_v1/econ_tax_shock_supply_demand_seed_v1/lesson.ddn` | PASS(exit 0), `E_LEGACY_RANGE_SYNTAX` 없음 | `state_hash=blake3:80eb2fcf529cef317c961bdedaa1e240299232a17ac0c2ff207ee57f553d0faa`, `trace_hash=blake3:ac5459cc2375efce297364c8cd9a7ec0c9e47def568cece06a5df07dbd60aa99` |
| `rg -n "//\\s*범위|범위\\(" <두 lesson 파일>` | PASS: 출력 없음 | legacy range 주석 0건 |

## 비범위

- 새 lesson/pack 추가 없음
- parser/runtime/tool 코드 수정 없음
- golden 갱신 없음
- 모델 계산식, 조건식, 출력 계열 수정 없음
