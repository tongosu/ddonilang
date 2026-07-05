# BRIEF: 가나다 로드맵 재검증 — 3순위(바/사/아/자/차/카/파/거 줄기)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/roadmap/PLAN_GANADA_REVERIFICATION_20260706.md` — Tier1/2와 동일 방법론.
> 성격: 진단·감사 전용. golden 갱신/코드 수정/삭제 금지.

## 대상

`ROADMAP_V2_GANADA_MILESTONE_MATRIX_20260426.md` §5 중 **바(자유실험실), 사(보개·공간·게임), 아(누리짐), 자(AI·슬기), 차(RPG), 카(또니마루·공유), 파(경제사회), 거(AI개발보조)** 각 6마루(총 48칸).

## 작업 (Tier1/2와 동일 절차)

1. 완료 근거 문자열 추출
2. 대응 pack/checker/suite 존재 확인(검색 전수)
3. 실제 실행 → PASS/FAIL 기록
4. PASS 시 내용 표본 확인(placeholder 여부)
5. 재판정: `증거없음 / 존재하나FAIL / 존재+PASS이나형식뿐 / 진짜닫힘`

## 특히 확인할 것

- 아줄기(누리짐)는 메모리 기록상 "아-1 닫힘, 아-2 진행중(CartPole/Pendulum expected hash stale 문제 잔존)"이었던 트랙이다 — 로드맵 문서는 이걸 6마루까지 전부 닫힘으로 표기하는데 실제 상태와 맞는지 특히 확인.
- 48칸으로 양이 많으니, 시간이 부족하면 0~2마루(씨앗/첫실행/닫힘)를 우선하고 3~5마루(작업실/나눔/단단)는 다음 배치로 미뤄도 된다 — 그 경우 어디까지 했는지 명확히 보고.

## 산출물

`docs/context/reports/GANADA_REVERIFICATION_TIER3_V1.md`
스키마: Tier1/2와 동일.

## 수용 기준

- [ ] 48칸 전수(또는 명시된 부분 완료 + 남은 범위 표기)
- [ ] `git status --short` 깨끗

## 금지 사항

golden 갱신 / 코드 수정 / 삭제 / 범위 밖 조사. `codex/queue-20260706` 브랜치에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 산출물: `docs/context/reports/GANADA_REVERIFICATION_TIER3_V1.md`
- 대상: 바/사/아/자/차/카/파/거 줄기 48칸 전수
- 실행 로그:
  - 체커: `I:/home/urihanl/ddn/codex/out/queue-20260706/q25-ganada-tier3/`
  - 실패 tail: `I:/home/urihanl/ddn/codex/out/queue-20260706/q25-ganada-tier3/failure_tails.txt`
  - 보조 pack golden: `I:/home/urihanl/ddn/codex/out/queue-20260706/q25-ganada-tier3/pack_golden/`
- 체커 결과: 56개 실행, PASS 9개, FAIL 47개
- 보조 pack golden 결과: roadmap_v2 대응 pack 22개 실행, 22개 PASS
- 재판정: `진짜닫힘` 8칸(바-1~바-5, 자-1, 자-2, 자-4), `존재+PASS이나형식뿐` 2칸(자-0, 파-0), `존재하나FAIL` 38칸
- 아줄기 확인: 아-1/아-2는 expected refresh 및 bandit/gridworld/cartpole/pendulum 관련 누락으로 FAIL. 현재 로드맵의 아-0~아-5 전부 `닫힘-동작` 표기는 재실행 결과와 맞지 않는다.
- 금지사항 준수: golden 갱신 없음, 코드 수정 없음, 파일 삭제 없음.
