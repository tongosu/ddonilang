# BRIEF: 가나다 로드맵 재검증 — 2순위(다/마/하/라 줄기)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/roadmap/PLAN_GANADA_REVERIFICATION_20260706.md` — Tier1 브리프와 동일 방법론, 착수는 Tier1 완료 후 권장(순서 무관하게 병렬 가능하면 병렬도 무방).
> 성격: 진단·감사 전용. golden 갱신/코드 수정/삭제 금지.

## 대상

`ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md` §5 중 **다줄기(수학·심볼릭·증명), 마줄기(셈그림 교과), 하줄기(교육·교재), 라줄기(말블록·편집환경)** 각 6마루(총 24칸).

## 작업 (Tier1 브리프와 동일 절차)

1. 완료 근거 문자열 추출
2. 대응 pack/checker/suite 존재 확인(검색 전수)
3. 실제 실행 → PASS/FAIL 기록
4. PASS 시 내용 표본 확인(placeholder 여부)
5. 재판정: `증거없음 / 존재하나FAIL / 존재+PASS이나형식뿐 / 진짜닫힘`

## 특히 확인할 것

- 마-1("대표 차시 첫실행: 포물선/진자/수요공급")과 마-2("교과 pack 닫힘")는 이번 대화에서 이미 "edu_p1_constant_accel/lesson.ddn이 `보개로 그려.` 한 줄 placeholder"임을 확인한 바 있다. 이 재검증에서 다른 대표 차시(진자/수요공급)도 같은 문제인지 확인할 것 — 특히 커널 스펙 논의에서 나온 "대표 경험 1호(포물선) lesson"이 지금 실제로 어떤 상태인지 정확히 표본 확인.
- 라줄기는 §6 우선순위표에서 "1위"로 지정된 줄기(`말블록 기본 팔레트 + block→DDN codegen`)다 — 실제로 동작하는 자산인지 특히 꼼꼼히 확인.

## 산출물

`docs/context/reports/GANADA_REVERIFICATION_TIER2_V1.md`
스키마: Tier1과 동일.

## 수용 기준

- [ ] 24칸 전수
- [ ] 마-1/마-2는 실제 lesson 파일 내용까지 열어 확인(placeholder 여부 명시)
- [ ] `git status --short` 깨끗

## 금지 사항

golden 갱신 / 코드 수정 / 삭제 / 범위 밖 조사. `codex/queue-20260706` 브랜치에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 산출물: `docs/context/reports/GANADA_REVERIFICATION_TIER2_V1.md`
- 대상: 다/마/하/라 줄기 24칸 전수
- 실행 로그:
  - 체커: `I:/home/urihanl/ddn/codex/out/queue-20260706/q24-ganada-tier2/`
  - 보조 pack golden/템플릿: `I:/home/urihanl/ddn/codex/out/queue-20260706/q24-ganada-tier2/pack_golden/`
  - 직접 lesson 실행: `I:/home/urihanl/ddn/codex/out/queue-20260706/q24-ganada-tier2/lesson_run/`
- 체커 결과: 35개 실행, PASS 6개(`ma5_lts_boundary`, `ha0`, `ha1`, `ha2`, `ha3`, `la0_docs`), FAIL 29개
- 재판정: `진짜닫힘` 0칸, `존재하나FAIL` 20칸, `존재+PASS이나형식뿐` 4칸
- 마-1/마-2 내용 확인: `pack/edu_p1_constant_accel/lesson.ddn`, `pack/edu_e1_supply_demand_tax/lesson.ddn`, `pack/edu_s1_function_graph/lesson.ddn`는 `보개로 그려.` 한 줄 placeholder. `edu_seamgrim_rep_phys_projectile_xy_v1`, `edu_seamgrim_rep_econ_supply_demand_tax_v1`는 실질 모델이나 `golden.jsonl` 없음. seed lesson 2개는 실질 모델이나 `E_LEGACY_RANGE_SYNTAX`로 직접 실행 FAIL.
- 금지사항 준수: golden 갱신 없음, 코드 수정 없음, 파일 삭제 없음.
