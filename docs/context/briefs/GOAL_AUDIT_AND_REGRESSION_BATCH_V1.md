# GOAL: 감사+회귀 확인 배치 V1 (Codex Goal 모드용)

> 작성: Claude (2026-07-06) / 실행: Codex(Goal 자율 루프) / 리뷰: Claude
> 성격: **전부 진단/검증 전용.** 설계 판단이 필요 없는, 이미 완료 기준이 정해진 작업만 묶었다. Plan/Act/Test/Review/Iterate를 반복하되, 아래 제약을 벗어나는 판단이 필요한 지점에 도달하면 **자율 진행을 멈추고 그 지점까지의 결과만 보고**하라 — 임의로 설계/삭제/수정 결정을 내리지 마라.

## 이 배치에 포함하지 않은 것(참고)

D49(`설정.누리` 스펙), D46 세부 설계(`슬기.의도{}`), Q19 "통합"/"보류" 199+99건 분류 — 이것들은 설계 판단이 필요해서 Goal 자율 루프에 안 맞는다(완료 기준을 미리 다 못 박아둘 수 없음). 이번 배치에는 넣지 않는다.

---

## 항목 A — DEV_SUMMARY 아카이브 분리 회귀 확인

**배경:** Q32가 `DEV_SUMMARY.md`를 9,084줄로 줄이고 나머지를 `DEV_SUMMARY_ARCHIVE_20260706.md`로 옮겼다. 이 파일을 참조하는 체커 184개(`pack/` 3, `publish/` 1, `tests/` 180)를 찾았지만 수정은 안 했다. 그중 일부가 **분리로 인해 archive로 넘어간 특정 문자열을 요구**하고 있었다면, 지금 조용히 FAIL로 바뀌었을 수 있다 — 아직 아무도 확인 안 했다.

**Outcome(결과):** 184개 참조 체커 전부가 분리 전과 동일한 PASS/FAIL 상태를 유지하고 있음을 실측으로 확인한다. 분리로 인해 새로 FAIL된 것이 있다면 정확히 몇 개, 어느 파일인지 특정한다.

**Verification(검증 근거):**
- 184개 파일 각각 실행 가능한 것(`tests/run_*.py`)은 개별 실행해 PASS/FAIL 기록.
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS(회귀 없음 확인).
- 새로 FAIL로 바뀐 게 있으면, 그 원인이 정말 DEV_SUMMARY 분리 때문인지(요구 문자열이 archive로 넘어갔는지) `rg`로 원문 위치를 archive/본문 양쪽에서 확인해 근거를 첨부.

**Constraints(제약):**
- 코드/체커 파일 수정 금지 — 이번 항목은 **진단만**. 새로 FAIL 발견해도 그 자리에서 고치지 마라. 별도 보고로 남겨라.
- `DEV_SUMMARY.md`/`DEV_SUMMARY_ARCHIVE_20260706.md` 내용 수정 금지.

**산출물:** `docs/context/reports/DEV_SUMMARY_SPLIT_REGRESSION_CHECK_V1.md` — 184개 표(파일 | 분리전 추정상태 | 분리후 실측상태 | 일치여부), 새 FAIL 목록+원인.

---

## 항목 B — §7.3 흐름씨-훅 위상 분리 실코드 검증

**배경:** SSOT `SSOT_LANG_v24.12.9.md:2988`의 §7.3 "흐름씨-훅 위상 분리"는 MUST 규칙이다: 한 마디 안에서 `[1단계] ordinary assignment → [2단계] 흐름씨(<<-) fixed-point → [3단계] tail-phase 훅(될때/인 동안/일때)` 순서, P1~P6(같은-마디 재발화 금지/다중출처충돌=E_FLOW_MULTIPLE_SOURCE_CONFLICT/순환참조=E_FLOW_CIRCULAR_REFERENCE/ordinary assignment 우선/state_hash 경계). 대응 pack `lang_flow_hook_interaction_v1`이 있지만(`c01_흐름씨_후_훅_스냅샷.ddn`, `c02_같은마디_재발화_금지.ddn`), Claude가 확인한 바로는 `E_FLOW_MULTIPLE_SOURCE_CONFLICT`/`E_FLOW_CIRCULAR_REFERENCE` 에러 코드가 `tools/teul-cli/src`, `lang/src` 어디에도 없다(정적 검색 0건). golden 실행 결과도 없어 보인다.

**Outcome(결과):** P1~P6 각각에 대해 "실제로 강제됨(실행 근거 있음)" 또는 "강제 안 됨(pack은 있으나 검사 코드 없음)"으로 확정 판정한다. `흐름씨(<<-)` 문법 자체가 파서/런타임에 존재하는지부터 확인한다(이게 없으면 P1~P6 전부 자동으로 "강제 안 됨"이 된다).

**Verification(검증 근거):**
- `<<-` 토큰 파서 지원 여부(`tools/teul-cli/src/lang/parser.rs`에서 실측).
- `E_FLOW_MULTIPLE_SOURCE_CONFLICT`/`E_FLOW_CIRCULAR_REFERENCE` 런타임 발화 지점(`tools/teul-cli/src/runtime/eval.rs`)의 파일:행.
- `pack/lang_flow_hook_interaction_v1`의 case 파일(`.ddn`)을 실제로 `cargo run --manifest-path tools/teul-cli/Cargo.toml -- run <case>.ddn`으로 실행해 보고, 기대한 대로 동작/실패하는지(또는 아예 파싱조차 안 되는지) 실측.
- 이번 세션에서 이미 확인한 [1단계]→[2단계]→[3단계] 실제 tick 루프(`tools/teul-cli/src/runtime/eval.rs`의 `every_hooks`/`becomes_hooks`/`while_hooks` 처리부, 대략 733~830행 구간)와 §7.3 문서 순서를 줄 단위로 대조해, 실제 코드가 몇 단계 체계인지(흐름씨 fixed-point 단계 자체가 있는지) 판정.

**Constraints(제약):**
- 코드 수정, pack 생성, golden 갱신 전혀 없음 — 순수 조사.
- P1~P6 중 "강제 안 됨"으로 확인된 항목을 그 자리에서 구현하려 하지 마라 — 이건 언어 코어 의미론 변경이라 Claude 설계가 먼저 필요하다(가줄기 커널 게이트, Q13-18과 같은 범주).

**산출물:** `docs/context/reports/FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md` — P1~P6 판정표(근거 파일:행 포함), `<<-` 파서 지원 여부, pack 실행 결과.

---

## 공통 진행 규칙 (Goal 자율 루프용)

1. 항목 A → 항목 B 순서로 진행하되, 하나가 막히면(예: 필요한 파일이 없음, 판단이 모호함) 그 항목만 부분 보고로 남기고 다음 항목으로 넘어가라. 전체를 막지 마라.
2. 각 항목 완료 시 `codex/queue-20260706` 브랜치에 커밋 1개(`[GOAL-A] ...`, `[GOAL-B] ...`).
3. 매 커밋 전 자기 검증(위 Verification 항목)을 실제로 실행하고 결과를 커밋 메시지/브리프 실행 보고에 남겨라 — "돌렸다고 주장"이 아니라 실제 출력을 인용하라.
4. 이 브리프 파일 하단에 항목별 `## 실행 보고 A`, `## 실행 보고 B`를 추가하라.
5. 두 항목 다 끝나면 이 파일 최하단에 `## Goal 종료 보고`(무엇을 완료했는지, 무엇을 못 했는지, 다음에 뭘 봐야 하는지)를 남겨라.
6. main 직접 커밋/push/네트워크 사용 금지는 기존 큐 프로토콜과 동일하게 유지한다.

## 실행 보고 A

- 항목: DEV_SUMMARY 아카이브 분리 회귀 확인.
- 기준선: `26d8d2b` (Q31, `DEV_SUMMARY_ARCHIVE_20260706.md` 분리 직전).
- 현 지점: `d05c7ff` (`codex/queue-20260706`).
- 참조 수집: `DEV_SUMMARY.md`, `DEV_SUMMARY`, `Development Summary`, `또니랑 Codex 개발 요약` 토큰 기준.
- 실측 범위: 참조 파일 184개 = `tests/run_*.py` 180개 + 비실행 참조 4개.
- 결과: 새 FAIL 0개. 동일 결과 167개, 기준선 FAIL -> 현 PASS 13개.
- 검증: `python tests/run_ci_sanity_gate.py --profile core_lang` PASS (`ci_sanity_status=pass`).
- 산출물: `docs/context/reports/DEV_SUMMARY_SPLIT_REGRESSION_CHECK_V1.md`.

## 실행 보고 B

- 항목: SSOT §7.3 흐름씨-훅 위상 분리(P1~P6) 제품 코드 검증.
- 검증 대상: `tools/teul-cli/src/cli/frontdoor_parse.rs`, `tools/teul-cli/src/lang/{token,lexer,parser}.rs`, `tools/teul-cli/src/runtime/eval.rs`, `pack/lang_flow_hook_interaction_v1`.
- 직접 실행: `cargo run --manifest-path tools/teul-cli/Cargo.toml -- run pack/lang_flow_hook_interaction_v1/cases/<case>/input.ddn` 4건.
- 결과: 4건 모두 `E_PARSE_EXPECTED_EXPR`로 실패. 제품 teul-cli 경로에서 `<<-`가 단일 흐름씨 토큰/문장으로 파싱되지 않아 fixed-point runtime 단계까지 도달하지 않는다.
- 지원 체커: `python tests/run_lang_flow_hook_interaction_pack_check.py` PASS. 단, docs-first contract/expected 파일 검사이며 제품 실행 검사는 아니다.
- 판단: P1~P6는 현재 제품 실행 기준 behavior-closed가 아니다. 코드 수정 없이 진단 보고만 수행했다.
- 산출물: `docs/context/reports/FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md`.

## Goal 종료 보고

- 항목 A: 완료. DEV_SUMMARY split 이후 새 FAIL 0개, `core_lang` PASS.
- 항목 B: 완료. §7.3 P1~P6 제품 코드 실측 결과, teul-cli 제품 경로는 `<<-` 미랜딩으로 런타임 의미를 집행하지 못함을 확인.
- 커밋(원본 `codex/queue-20260706` 브랜치, main 병합 시 리포트 파일만 반영하고 브리프 본문은 이 파일로 재합류): `[GOAL-A] DEV_SUMMARY split regression check`(`545d3db`), `[GOAL-B] Flow hook phase separation verification`(`fc3e484`).
- 금지 준수: main 직접 커밋/push 없음, 네트워크 없음, 코드/팩/golden/checker 수정 없음.
- 참고: 실행 당시 Codex 로컬 체크아웃이 이 브리프 파일의 최신 버전(Outcome/Verification/Constraints 포함)을 갖고 있지 않아, 실행 보고만 담긴 축약본을 별도로 커밋했다. Claude가 병합 시 원본 스펙에 실행 보고를 재합류했다.
