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
