# BRIEF: 가나다 로드맵 재검증 — 1순위(가/나/타 줄기)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/roadmap/PLAN_GANADA_REVERIFICATION_20260706.md`
> 성격: 진단·감사 전용. golden 갱신/코드 수정/삭제 금지.
> 배경: `docs/context/roadmap/ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md`가 15줄기×6마루 90칸 전부를 "닫힘-동작"으로 표기하고 있으나, 이번 주 실제 검증(Q18/Q20)이 가줄기 2/4마루 주장을 반증함. 신뢰할 수 없는 것으로 판단, 전면 재검증 착수.

## 대상

`ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md`의 §5(줄기별 상세 체크표) 중 **가줄기, 나줄기, 타줄기** 각 6마루(총 18칸).

## 작업 (18칸 각각에 대해)

1. 해당 마루 행의 "완료 근거" 열 문자열(예: `core smoke pack`, `std_core, std_grid, std_input_map smoke`, `registry check pack`)을 그대로 가져온다.
2. 그 이름에 정확히 또는 유사하게 대응하는 pack/checker/suite/테스트가 저장소에 실제로 존재하는지 확인한다(`pack/`, `tests/`, `docs/ssot/pack/` 등 전체에서 검색). 존재하지 않으면 "증거 없음"으로 기록하고 다음 칸으로.
3. 존재하면 실제로 실행한다(`python tests/run_pack_golden.py <이름>` 또는 해당 체커 스크립트). 실제 PASS/FAIL을 기록한다.
4. PASS라면 그 pack/checker의 실제 내용을 열어 마일스톤 설명과 부합하는 실질적 내용인지 표본 확인한다(placeholder 1줄짜리나 스켈레톤뿐이면 "형식만 존재"로 별도 표기).
5. 위 결과를 종합해 재판정 상태를 매긴다: `증거없음 / 존재하나FAIL / 존재+PASS이나형식뿐 / 진짜닫힘`.

## 특히 확인할 것 (이미 이번 주 반증된 항목이라 정확히 재확인)

- 가-2("대표 문법 pack 닫힘: 채비/훅/조건/임자/계약") — Q18/Q20이 이미 반증(사용자 지정 씨앗 bare 호출 미배선, 모호성 검사 부재)한 사실을 이 재검증 결과에도 반영/교차 확인할 것.
- 가-4("package/gaji 연결: gaji metadata/lock/package, registry shell pack") — 이 이름의 pack이 실제 존재/PASS하는지, Q21(로컬 레지스트리 랜딩 감사, 진행 중이면 그 결과와 교차 확인)과 일치하는지.

## 산출물

`docs/context/reports/GANADA_REVERIFICATION_TIER1_V1.md`
스키마: `| 줄기-마루 | 원문 완료근거 | 존재확인 | 실행결과 | 표본 내용확인 | 재판정 상태 |`

## 수용 기준

- [ ] 18칸(가6+나6+타6) 전수, 빠짐없이 재판정
- [ ] 모든 "PASS" 판정은 실제 실행 로그 첨부(추정 금지)
- [ ] 가-2/가-4는 기존 Q18/Q20/Q21 결과와 교차 확인 문장 포함
- [ ] `git status --short` 깨끗(golden 갱신/코드 수정 없음)

## 금지 사항

golden 갱신 / 코드 수정 / 삭제 / 범위 밖(다른 줄기) 조사. `codex/queue-20260706` 브랜치에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 산출물: `docs/context/reports/GANADA_REVERIFICATION_TIER1_V1.md`
- 실행 로그:
  - 체커: `I:/home/urihanl/ddn/codex/out/queue-20260706/q23-ganada-tier1/`
  - 보조 pack golden: `I:/home/urihanl/ddn/codex/out/queue-20260706/q23-ganada-tier1/pack_golden/`
- 범위: 가줄기 6칸, 나줄기 6칸, 타줄기 6칸 총 18칸 전수.
- 실행 결과:
  - roadmap_v2 관련 체커 28개 실행: PASS 2개(`ga4`, `na4`), FAIL 26개.
  - 존재하는 roadmap_v2 marker/reconciliation 팩 21개 `run_pack_golden.py` 보조 실행: 전부 PASS.
- 재판정:
  - `진짜닫힘`: 0칸.
  - `존재하나FAIL`: 16칸.
  - `존재+PASS이나형식뿐`: 2칸(가-4, 나-4).
- 필수 교차 확인:
  - 가-2는 Q18/Q20 결과와 교차 확인했다. bare 사용자 씨앗 호출 미배선 및 `E_CALL_TAIL_AMBIGUOUS` 미강제 때문에 대표 문법 pack 닫힘으로 볼 수 없다.
  - 가-4는 Q21 결과와 교차 확인했다. local registry minimum은 부분 착지이나 `gaji/` 30개 전체 discover/install/publish 닫힘은 아니다.
- 자기 검증:
  - `git status --short` 확인 시 실행 부산물로 인한 워킹 트리 변경 없음.
  - golden 갱신, 코드 수정, 삭제 없음.
