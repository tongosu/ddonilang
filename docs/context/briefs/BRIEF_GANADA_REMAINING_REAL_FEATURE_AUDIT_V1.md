# BRIEF: 가나다 나머지 8줄기 실기능 감사 (문서 체커 아님)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/roadmap/GANADA_MATRIX_CORRECTED_20260706.md`
> 성격: 진단/조사 전용. 코드 수정 없음.

## 배경

Tier1~3 재검증(Q23~25)이 이미 90칸 전체를 실행했지만, 확인한 건 "로드맵 재판정 체커가 PASS/FAIL하는가"였다. 다/라/사/차/카/파/거/아 8개 줄기(48칸)의 FAIL 대부분은 "완료 문서(.md)가 없어서"였는데, **이게 실제 기능도 없다는 뜻인지, 아니면 기능은 있는데 그냥 문서만 안 썼다는 뜻인지는 아직 아무도 확인 안 했다**(`GANADA_MATRIX_CORRECTED_20260706.md`의 "아직 모른다는 뜻" 문구가 이 공백을 가리킴).

## 작업

8개 줄기 각각의 6칸(줄기-0~5)에 대해, 로드맵 문서가 아니라 **실제 코드/pack**을 확인하라:

1. 각 칸이 가리키는 pack(`ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md`에서 원래 칸별로 지정한 pack 이름)이 실제로 존재하는지.
2. 존재하면 `python tests/run_pack_golden.py <pack>`을 실제로 돌려서 PASS/FAIL을 실측하라(로드맵 재판정 체커 말고 pack 자체의 golden).
3. pack이 없으면, 관련 기능이 다른 이름의 pack/코드로 실제로 존재하는지 `rg`로 찾아보라(예: 사-2 "sprite/grid2d"라면 `rg -l "grid2d\|sprite"`로 관련 코드 존재 여부 확인).
4. 각 칸에 대해 "실제 코드/pack 존재+동작", "이름만 다른 실제 자산 존재(찾음)", "진짜 없음" 셋 중 하나로 판정하라. 마-줄기에서 이미 발견됐던 패턴(placeholder만 있거나, 실질 자산인데 golden만 없거나)과 비슷한 게 있는지 특히 주의해서 보라.

## 검증 방법

- 정적 분석 + `python tests/run_pack_golden.py <pack>` 실제 실행. 코드 수정, golden 갱신, pack 생성 전혀 없음.

## 산출물

`docs/context/reports/GANADA_REMAINING_TRACKS_REAL_FEATURE_AUDIT_V1.md`:
- 48칸 표: 줄기-마루 | 원래 지정 pack | pack 존재 여부 | golden 실행 결과 | 판정(실제동작/이름만다른자산발견/진짜없음) | 비고

양이 많으면 다/라 먼저(우선순위 1위였던 라-줄기 포함) 하고 나머지는 별도 보고해도 된다.

## 수용 기준

- [ ] 48칸(또는 진행한 만큼) 전부 실측 기반 판정
- [ ] "실제동작"/"이름만다른자산발견" 판정에는 반드시 실행 로그/명령 근거 첨부
- [ ] 코드/golden/pack 변경 없음

## 금지 사항

코드 수정, golden 갱신, pack 생성 없음. main 직접 커밋 금지.

## 보고 형식

이 파일 하단 `## 실행 보고`: 처리한 줄기 수, 판정별 집계, 산출물 경로.
