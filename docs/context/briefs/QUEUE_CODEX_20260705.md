# CODEX 작업 큐 (2026-07-05) — 순차 자율 실행

> 운영 모델: 사용자가 큐를 쌓아두면 Codex가 **위에서부터 순차 실행**, Claude는 **사후 커밋 리뷰** (동기 승인 대기 없음).
> 이 파일이 Codex의 단일 진입점이다. 미션/브리프 파일의 "Claude 리뷰 통과 전 착수 금지" 문구는 이 큐 프로토콜로 대체된다 (2026-07-05 개정).

## 프로토콜 (모든 작업 공통)

1. **브랜치**: 시작 전 `codex/queue-20260705` 브랜치 생성(있으면 재사용). **main 직접 커밋 금지.**
2. **작업 1건 = 커밋 1개**: 각 작업 완료 시 자기 검증(수용 기준) 통과를 확인한 후 브랜치에 커밋. 메시지: `[Q<번호>] <요약>` + 검증 결과 요약.
3. **자기 검증 실패 시**: 그 작업의 변경을 `git restore`로 원복하고, 실패 사유를 해당 브리프의 실행 보고에 기록한 뒤, **의존 관계 없는 다음 작업으로 진행**. 의존 작업(아래 표기)은 건너뛴다.
4. **보고**: 각 작업의 브리프/미션 파일 실행 보고 절에 기록 (기존 형식 유지). 보고 파일 수정도 해당 커밋에 포함.
5. **공통 금지 (불변)**: main 커밋 / push / golden --update / 파일 삭제 / allowlist 변경 / 네트워크 / 브리프 범위 밖 수정.
5-1. 브리프/보고서 신규 파일은 `docs/`가 gitignore이므로 `git add -f`로 커밋에 포함한다 (기존 관례 — Q1~Q5에서 확립).
6. 큐 소진 시: 이 파일 하단에 `## 큐 완료 보고` 추가하고 종료.

## 큐 (위에서부터 순차)

| # | 작업 | 명세 위치 | 성격 | 의존 |
|---|---|---|---|---|
| Q1 | SSOT 언어 인벤토리 추출 (미션 단계 B) | MISSION_20260705_LONG_TRACK_CONSOLIDATION_V1.md §단계 B | 보고서 생성 (저위험) | — |
| Q2 | Fixed64 포화 계수기 (커널 파일럿) | BRIEF_20260705_KERNEL_SATURATION_COUNTER_V1.md | 런타임 계측 (자기 검증 엄격: 비트 동일) | — |
| Q3 | 깨진 체크 목록화·분류 (미션 단계 C) | 미션 §단계 C — 대상은 단계 A 보고의 기존FAIL 포함 전수 재집계 (약 90건 예상) | 보고서 생성 | — |
| Q4 | UI/러너 의존성 지도 (미션 단계 D) | 미션 §단계 D | 보고서 생성 | — |
| Q5 | 포화 실태 전 팩 집계 (미션 단계 E) | 미션 §단계 E — 착수 조건 개정: "Claude 패치 제공" 대신 **Q2 자기 검증 통과** | 보고서 생성 | Q2 |

## 개정 메모
- 미션 단계 E의 선행 조건은 이 큐에서 "Q2 자기 검증 통과"로 대체한다 (Claude 사후 리뷰는 E 결과를 의사결정에 쓰기 전에 수행).
- Q2가 실패하면 Q5는 건너뛰고 보고에 사유 기록.

## 추가 큐 (2026-07-04 Claude 감사 후 — Codex 자기 감사 의견 채택)

| # | 작업 | 명세 | 의존 |
|---|---|---|---|
| Q6 | **[AUDIT-FIX]** 감사 보정 4건 — 기존 커밋 수정 없이 브랜치 위에 커밋 1개 | 아래 상세 | — |
| Q7 | Q5 재실행 (보정된 계측으로) — `SATURATION_AUDIT_V2.md` | Q6 | Q6 |

### Q6 상세 ([AUDIT-FIX] 커밋 1개)
1. **계측 명확화**: `DDN_SATURATION_AUDIT` 설정 시 **count=0도 stderr 출력** (`saturation_audit count=0`). 미설정 시 출력 없음(불변). 추가: 오류 종료 경로에서도 출력되도록 훅 위치 점검 — 불가하면 보고에 한계 명시.
2. **Q3 보고 문구 보정**: "모든 FAIL 체크" → "누락 .md 필수 참조 기반 FAIL 후보 (정적 분석)", 처분 "삭제" → "삭제 심사 후보". 표 데이터는 불변.
3. **Q4 보고 문구 보정**: "제품" → "제품 도달 또는 제품 시작점 가정 (ui/screens/*.js 시작점 포함)" 판정 기준 명시.
4. **Q2 브리프 실행 보고에 보정 내역 추가.**
- 검증: 미설정 경로 비트 동일 재확인 (pack golden 2건 + core_lang PASS), 설정 시 count=0 팩에서 `count=0` 한 줄 출력 확인, cargo test PASS.

### Q7 상세 (Q5 재실행)
- 보정된 계측으로 826개 팩 재집계 → `docs/context/reports/SATURATION_AUDIT_V2.md`
- V1과의 차이: 모든 팩에 count 로그가 명시되므로 "로그 없음=0" 추정 제거. **FAIL 팩도 "실행 실패 + count=N 확인"으로 구분 기재.**
- FAIL 98개 + TIMEOUT 1개는 별도 열로: `실패 단계(스키마/골든 불일치/타임아웃)` 1차 분류 (수리 금지 — 분류만).

## 2차 큐 (2026-07-04 기획 — Q7 이후 순차)

| # | 작업 | 성격 | 착수 조건 |
|---|---|---|---|
| Q8 | FAIL 팩 심층 분류 | 보고서 | Q7 완료 |
| Q9 | run_pack_golden `--all` 견고화 | 테스트 인프라 수리 | — |
| Q10 | SSOT 인벤토리 v1.1 (Claude 교정 3건 반영) | 보고서 수정 | — |
| Q11 | 테스트 부산물 gitignore 등재 | 설정 1건 | — |
| Q13~Q18 | 커널 구현 시리즈 (아래) | 런타임 구현 | **⛔ 게이트 — 착수 조건 충족 전 절대 시작 금지** |

### Q8 상세 — FAIL 팩 심층 분류
- Q7에서 확인된 FAIL/TIMEOUT 팩 각각을 개별 재현하고 첫 오류를 채집해 `docs/context/reports/PACK_FAIL_TRIAGE_V1.md` 생성:
  `| 팩 | 첫 오류 요지 | 분류(골든 stale/스키마 오류/특수 러너 필요/입력 오류/타임아웃) | 처분 제안(갱신 후보/수리 후보/삭제 심사 후보/특수러너 확인) | 증빙 로그 경로 |`
- 수리·갱신 실행 금지. 분류의 근거 로그를 빌드 디렉터리에 보존.

### Q9 상세 — `--all` 견고화
- 문제: `python tests/run_pack_golden.py --all`이 `pack/external_intent_boundary_v1` golden.jsonl 스키마 오류에서 전체 중단.
- 수리 명세: 팩 하나의 로드/스키마 실패가 전체 실행을 중단시키지 않고 해당 팩을 FAIL로 기록 후 계속. 끝에 `총/PASS/FAIL` 요약 출력. 실패 존재 시 종료 코드 1. **단일 팩 호출 경로 동작 불변.**
- 수용: `--all` 완주 증빙, external_intent_boundary_v1이 FAIL로 보고됨, `run_pack_golden.py gate0_contract_abort_statehash_v1 bogae_asset_manifest_v1` PASS(회귀 없음), 수정 파일은 tests/_run_pack_golden_impl.py(+wrapper) 이내.

### Q10 상세 — 인벤토리 v1.1
- 미션 단계 B 리뷰의 교정 3건만 반영: ①수/셈수 행의 정본-별칭 관계를 SSOT 문면 기준으로 정리 ②셈씨 코드 근거 보완(canon.rs/parser.rs — 예: parser.rs:4355) ③큰바른수/곱수 상태를 `⚠️부분(SSOT designed, 코드 구성자 존재)`으로 완화.
- 수용: 해당 행 외 diff 없음.

### Q11 상세 — 부산물 gitignore
- `.gitignore`에 `pack/_tmp_age5_surface_selftest_contract/`, `pack/_tmp_age5_surface_selftest_warning/` 추가 (sanity gate가 재생성하는 selftest 부산물 — 삭제해도 되살아남 확인됨).
- 수용: core_lang sanity gate 실행 후 `git status --short` 깨끗. 이 브리프에 한해 .gitignore 수정 허용 (그 외 설정 변경 금지 유지).

### Q13~Q18 — 커널 구현 시리즈 (전부 게이트 잠김)
공통 착수 조건: **①Claude가 해당 항목의 적합성 케이스(레드)를 커밋 ②큐 파일에 Claude가 "Q1N 게이트 해제" 기록.** 조건 미충족 시 건너뛰고 보고에 "게이트 잠김" 기록.

| # | 작업 | 개별 조건 |
|---|---|---|
| Q13 | 시간 내장: `마디사이`/`지금` (커널 스펙 §4) | — |
| Q14 | 벡터2 = 텐서 (2,) 특수형: 산술+@단위+`.x/.y` 설탕 (스펙 §2.1) | — |
| Q15 | `움직임{}` 설탕 + 펼쳐보기 (스펙 §7 전개 규칙) | Q13 (마디사이 의존) |
| Q16 | 조건문 정본화기 출력 전환 (`~인 동안`/`만약~이면`) + 기존 팩·lesson 일괄 재정본화 | — |
| Q17 | 오버플로 포화→오류 전환 (E_NUM_OVERFLOW + fix-it) | Q7에서 포화 0 재확정 + Claude 틱 처리 의미론 확정 |
| Q18 | 승인된 삭제 실행 (깨진 체크·고아 모듈·FAIL 팩 처분) | **사용자 승인 목록** 커밋 후 |

## 큐 완료 보고
(큐 소진 시 여기에 기록)

### Codex 실행 완료 보고

- 브랜치: `codex/queue-20260705`
- main 직접 커밋: 없음
- push: 없음
- golden 갱신: 없음
- 네트워크 사용: 없음
- Q1 완료: `0704081` `[Q1] SSOT 언어 인벤토리 추출`
  - 산출물: `docs/context/reports/SSOT_LANG_INVENTORY_V1.md`
  - 검증: 본표 100행, SSOT_LANG 절 반영/소거 449행, SSOT 근거 누락 0, 모순 1건 표기
- Q2 완료: `ca7541c` `[Q2] Fixed64 포화 계수기 추가`
  - 산출물: `tools/teul-cli/src/core/fixed64.rs`, `tools/teul-cli/src/main.rs`, Q2 브리프 실행 보고
  - 검증: `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS, 관련 pack golden PASS, `python tests/run_ci_sanity_gate.py --profile core_lang` PASS, `DDN_SATURATION_AUDIT=1` 환경 core_lang PASS, 포화 재현 count=1 확인
- Q3 완료: `b708863` `[Q3] 깨진 체크 목록화`
  - 산출물: `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`
  - 검증: `tests/run_*.py` 1080개 정적 분석, FAIL 후보 173개, 고유 누락 문서 176개, 전부 git 이력 0행
- Q4 완료: `74d595f` `[Q4] UI 러너 의존성 지도 작성`
  - 산출물: `docs/context/reports/UI_RUNNER_DEPENDENCY_MAP_V1.md`
  - 검증: UI 최상위 JS 102개 + `tests/*.mjs` 191개 = 표 293행, 제품 91 / 러너전용 10 / 고아 1, 러너전용 189 / 고아 2
- Q5 완료: `7f9acab` `[Q5] 포화 실태 팩 집계`
  - 산출물: `docs/context/reports/SATURATION_AUDIT_V1.md`
  - 검증: 826개 golden 팩 독립 실행, PASS 727 / FAIL 98 / TIMEOUT 1, 포화 발생 팩 0, 총 포화 발생 0
- 최종 상태: 큐 Q1-Q5 전부 커밋 완료. 이 완료 보고 커밋 전 `git status --short` 출력 없음.

### Codex 추가 큐 실행 완료 보고

- 브랜치: `codex/queue-20260705`
- main 직접 커밋: 없음
- push: 없음
- golden 갱신: 없음
- 네트워크 사용: 없음
- 파일 삭제: 없음
- Q6 완료: `91d8a31` `[Q6] 감사 보정 반영`
  - 산출물: `tools/teul-cli/src/main.rs`, `docs/context/reports/BROKEN_CHECKS_AUDIT_V1.md`, `docs/context/reports/UI_RUNNER_DEPENDENCY_MAP_V1.md`, `docs/context/briefs/MISSION_20260705_LONG_TRACK_CONSOLIDATION_V1.md`, `docs/context/briefs/BRIEF_20260705_KERNEL_SATURATION_COUNTER_V1.md`
  - 보정: `DDN_SATURATION_AUDIT` 설정 시 count 0도 stderr 출력, 오류 종료 경로 audit helper 적용, Q3/Q4 보고 문구 보정, Q2 브리프 보정 보고 추가
  - 검증: `cargo test --manifest-path tools/teul-cli/Cargo.toml` PASS(1091 tests), `python tests/run_pack_golden.py gate0_contract_abort_statehash_v1 bogae_asset_manifest_v1` PASS, `python tests/run_ci_sanity_gate.py --profile core_lang` PASS, `DDN_SATURATION_AUDIT=1` count 0 성공/오류 종료 출력 확인, `git diff --check` PASS
- Q7 완료: `d347e7c` `[Q7] 포화 실태 팩 재집계`
  - 산출물: `docs/context/reports/SATURATION_AUDIT_V2.md`
  - 집계 원본: `I:/home/urihanl/ddn/codex/build/q7_saturation_audit/per_pack_results_v2.detjson`
  - 실행 로그: `I:/home/urihanl/ddn/codex/build/q7_saturation_audit/per_pack_results_v2.log`, `I:/home/urihanl/ddn/codex/build/q7_saturation_audit/raw_pack_logs/`
  - 검증: 826개 추적 pack golden 독립 실행, PASS 727 / FAIL 98 / TIMEOUT 1, 실패 단계 스키마 27 / 골든 불일치 71 / 타임아웃 1, 포화 발생 팩 0, 총 포화 발생 0, 보고서 표 826행, `git diff --check` PASS
- 최종 상태: 추가 큐 Q6-Q7 전부 커밋 완료. 기존 미추적 selftest 부산물 `pack/_tmp_age5_surface_selftest_contract/`, `pack/_tmp_age5_surface_selftest_warning/`는 삭제 금지 지침에 따라 그대로 둠.
