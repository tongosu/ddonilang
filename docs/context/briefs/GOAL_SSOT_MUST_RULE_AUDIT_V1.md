# GOAL: SSOT_LANG MUST 규칙 179건 전수 실코드 검증 V1 (Codex Goal 모드)

> 작성: Claude (2026-07-06) / 실행: Codex(Goal 자율 루프) / 리뷰: Claude
> 성격: **진단/조사 전용.** 코드, pack, golden, SSOT 수정 전혀 없음.
> 동기: 이번 세션에서 개별적으로 찾은 것들 — `V18-00C`(모호성 검사 미구현, Q20), `§7.3`(흐름씨 미구현, GOAL-B), `설정.누리`(빈약, D48 정정), `CALL-TAIL-RESOLVE-01`(예약어만 부분구현) — 전부 "SSOT는 MUST라고 잠갔는데 코드엔 없다"는 같은 패턴이었다. 낱개로 우연히 찾는 대신, `SSOT_LANG_v24.12.9.md`의 MUST 태그 179개 섹션 전체를 체계적으로 훑는다.

## 방법론 (GOAL-B와 동일 수준의 엄격함 요구)

각 MUST 섹션에 대해:
1. 섹션 제목/규칙 ID, 줄 범위를 기록.
2. 핵심 주장(무엇을 강제하는지) 1~2줄 요약.
3. 실제 제품 코드(`tools/teul-cli/src/**`, `core/src/**`, 필요시 `tool/src/**`)에서 이 규칙을 실행하는 근거를 찾는다 — 에러 코드 문자열, 함수명, 파서 분기 등을 `rg`로 검색하고 실제로 파일을 열어 확인한다.
4. 참조된 pack이 있으면 존재 여부 + golden 유무(`ls pack/<name>`, `README.md`의 `evidence_tier`) 확인.
5. **판정**: `구현됨`(제품 코드에 실제 집행 근거 있음, file:line 인용) / `부분구현`(일부만, 어디까지인지 명시) / `미구현`(product 코드에서 못 찾음, 검색식 명시) / `확인불가`(모호하거나 판단 근거 부족, 이유 명시).
6. **"확인 안 하고 구현됨이라고 쓰지 마라"** — GOAL-B처럼 실제로 `rg`/코드 읽기/가능하면 실행까지 해서 근거를 남겨야 한다. "아마 될 것 같다"는 판정 불가.

## 범위 분할 (4개 티어, 각자 독립 진행 가능)

- **Tier1**: `SSOT_LANG_v24.12.9.md` 1~2690행 (MUST 섹션 약 45개, 어휘/호출/블록헤더/임자/알림 계열)
- **Tier2**: 2690~3979행 (약 45개, 타입 시스템/고름씨/우선순위 계열)
- **Tier3**: 3979~5406행 (약 45개, 계산/보개/단위 계열)
- **Tier4**: 5406행~끝(8151행) (약 44개, 나머지)

각 티어 시작 전에 `grep -nE "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`로 그 티어 범위의 정확한 섹션 목록을 직접 뽑아서 빠짐없이 커버하라(위 줄번호는 근사치).

## 산출물

`docs/context/reports/SSOT_MUST_RULE_AUDIT_TIER{1,2,3,4}_V1.md` (티어당 1개 파일):

| 규칙 ID/제목 | 줄 범위 | 핵심 주장 | 판정 | 근거(file:line 또는 검색식) |
|---|---|---|---|---|

마지막에 티어별 요약(구현됨/부분구현/미구현/확인불가 건수)도 포함.

## 진행 규칙

1. Tier1 → Tier4 순서(또는 병렬 가능하면 병렬)로 진행. 하나가 너무 오래 걸리면 그 티어 부분 결과만 커밋하고 다음으로.
2. 각 티어 완료 시 `codex/queue-20260706` 브랜치에 커밋 1개(`[GOAL-MUSTAUDIT-T1]` ~ `[GOAL-MUSTAUDIT-T4]`).
3. **코드/pack/golden/SSOT 수정 절대 없음** — 이건 순수 조사다. 미구현을 발견해도 그 자리에서 고치지 마라.
4. 판정이 애매한 섹션(예: 문서 자체가 모순되거나 범위가 불명확)은 무리해서 확정하지 말고 `확인불가`로 표시 + 이유 기록.
5. 각 티어 완료 후 `python tests/run_ci_sanity_gate.py --profile core_lang` PASS 확인(수정이 없으니 당연히 영향 없어야 하지만, 확인 자체가 "코드 안 건드렸다"는 증거가 된다).

## 완료 조건

- 4개 티어 보고서 전부 생성, 179개 섹션 전수 커버(빠짐없음 — 각 티어 서두에 다룬 섹션 수/전체 대상 수 명시)
- 이 브리프 파일 하단에 티어별 `## 실행 보고 T1`~`T4`, 마지막 `## Goal 종료 보고`(전체 판정 집계: 구현됨/부분구현/미구현/확인불가 총계, 가장 심각해 보이는 미구현 항목 상위 10개 하이라이트)

## 금지 사항

SSOT/코드/pack/golden/checker 수정 전혀 없음. 발견한 문제를 그 자리에서 고치려 하지 마라 — 수리는 이후 Claude가 설계해서 별도로 위임한다. main 직접 커밋/push/네트워크 금지.

## 실행 보고 T1

- 실행일: 2026-07-06
- 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 1~2689행, MUST 헤더 시작 행 `<2690`.
- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`.
- 커버: Tier1 대상 44개 전부.
- 산출물: `docs/context/reports/SSOT_MUST_RULE_AUDIT_TIER1_V1.md`.
- 판정 집계: 구현됨 14 / 부분구현 18 / 미구현 12 / 확인불가 0.
- 주요 미구현 후보: `CALL-TAIL-RESOLVE-01`, `X/X하` 이름 충돌 금지, TERM-MAP 단일 소스/기계 추출, `해보고`/`그것`, 단자음 예약 식별자, `->` 우향 대입, `이름씨`/`나`/`고름씨` 타입군.
- 검증:
  - `git diff --check` PASS.
  - Tier1 보고서 표 행 수 44개 확인.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
- `core_lang` 실행 중 갱신된 기존 open 로그 2개(`pack/open_bundle_artifact/run_bundle/geoul.diag.jsonl`, `pack/open_end_to_end/open.log.jsonl`)는 이번 진단 산출물이 아니므로 diff 확인 후 원래 상태로 되돌렸다.
- 코드/pack/golden/SSOT 수정 없음.

## 실행 보고 T3

- 실행일: 2026-07-06
- 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 3979~5405행, MUST 헤더 시작 행 `>=3979 && <5406`.
- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`.
- 커버: Tier3 대상 45개 전부.
- 산출물: `docs/context/reports/SSOT_MUST_RULE_AUDIT_TIER3_V1.md`.
- 판정 집계: 구현됨 1 / 부분구현 31 / 미구현 13 / 확인불가 0.
- 주요 미구현 후보: 다중 디스덧댐, `늘지켜보고`/`덧댐거부`, 말결 `$...` 표면과 결합 규칙, 누리 query 표면 `임자들`/`모두`, `~에 따라`/`없으면`, MathIR v1/De Bruijn/DetBin.
- 검증:
  - `git diff --check` PASS.
  - Tier3 보고서 표 행 수 45개 확인.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
- `core_lang` 실행 중 갱신된 기존 open 로그 2개(`pack/open_bundle_artifact/run_bundle/geoul.diag.jsonl`, `pack/open_end_to_end/open.log.jsonl`)는 이번 진단 산출물이 아니므로 diff 확인 후 원래 상태로 되돌렸다.
- 코드/pack/golden/SSOT 수정 없음.

## 실행 보고 T2

- 실행일: 2026-07-06
- 범위: `docs/ssot/ssot/SSOT_LANG_v24.12.9.md` 2690~3978행, MUST 헤더 시작 행 `>=2690 && <3979`.
- 대상 추출: `rg -n "^#{2,4} .*(MUST)" docs/ssot/ssot/SSOT_LANG_v24.12.9.md`.
- 커버: Tier2 대상 45개 전부.
- 산출물: `docs/context/reports/SSOT_MUST_RULE_AUDIT_TIER2_V1.md`.
- 판정 집계: 구현됨 9 / 부분구현 25 / 미구현 11 / 확인불가 0.
- 주요 미구현 후보: 사용자 정의 이음씨/기호 이음씨/우선순위 registry, `이전값보기`, `<<-` 흐름씨 fixed-point, 조사/핀 바인딩 모호성 LSP QuickFix, `기/하기` 어간 trigger와 call-tail 후보 생성/모호성 오류, 숫자 literal 일반 suffix 분리.
- 검증:
  - `git diff --check` PASS.
  - Tier2 보고서 표 행 수 45개 확인.
  - `python tests/run_ci_sanity_gate.py --profile core_lang` PASS.
- `core_lang` 실행 중 갱신된 기존 open 로그 2개(`pack/open_bundle_artifact/run_bundle/geoul.diag.jsonl`, `pack/open_end_to_end/open.log.jsonl`)는 이번 진단 산출물이 아니므로 diff 확인 후 원래 상태로 되돌렸다.
- 코드/pack/golden/SSOT 수정 없음.
