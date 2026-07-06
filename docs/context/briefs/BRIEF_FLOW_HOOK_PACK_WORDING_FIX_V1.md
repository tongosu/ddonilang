# BRIEF: 흐름씨-훅 pack 문구 정정 (D58, 문서만)

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 근거: `docs/context/proposals/PROPOSAL_FLOW_HOOK_IMPLEMENTATION_DECISIONS_D51_D58_20260706.md`(D58), GOAL-B 실측(`docs/context/reports/FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md`)
> 성격: **순수 문서 정정.** 코드/pack/의미론 변경 없음. Q13-18 게이트와 무관 — 지금 바로 실행 가능.

## 배경

`docs/context/proposals/PROPOSAL_SSOT_V24_0_9_POST_CLARIFICATION_EXECUTION_SYNC_20260412.md:52`에 다음 문구가 있다:

> `lang_flow_hook_interaction_v1` 를 supporting pack/checker immediate target 으로 명시한다.
> 이것은 `ordinary assignment -> 흐름씨 fixed-point -> tail-phase hook` 관계를 **evidence로 잠그는 작업**이지, 새 surface 승격이 아니다.

이 문구는 실제보다 강하다. GOAL-B 실측 결과, 이 pack의 케이스 4개(`c01`~`c04`) 전부 제품 파서에서 `E_PARSE_EXPECTED_EXPR`로 실패한다 — `<<-` 자체가 파싱 안 되어 관계 자체에 도달하지 못한다. pack README(`pack/lang_flow_hook_interaction_v1/README.md`)는 이미 정직하게 `evidence_tier: docs_first`/`closure_claim: no`를 표기하고 있다 — pack 자체는 문제없다. 고쳐야 할 건 이 proposal 문서의 문구뿐이다.

## 작업

1. `docs/context/proposals/PROPOSAL_SSOT_V24_0_9_POST_CLARIFICATION_EXECUTION_SYNC_20260412.md:52`의 "evidence로 잠그는 작업" 문구를 다음으로 교체:
   > 이것은 `ordinary assignment -> 흐름씨 fixed-point -> tail-phase hook` 관계의 **contract/expected 구조를 문서로 고정하는 작업**(docs-first skeleton)이지, 실제 runtime 관계를 evidence로 닫는 작업이 아니다. 실제 제품 실행 검증은 GOAL-B(`FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md`, 2026-07-06) 참고 — 현재 `<<-`는 제품 파서(`tools/teul-cli/src/lang/lexer.rs`)에 없어 4개 케이스 전부 파싱 단계에서 실패한다.
2. 이 문서에 유사한 과장 문구가 다른 곳에도 있는지(`rg -n "evidence.*잠그"` 등으로) 확인하고, 있으면 같은 방식으로 정정. 없으면 위 1건만 수정.
3. `pack/lang_flow_hook_interaction_v1/README.md`는 이미 정직하므로 **수정하지 않는다.**

## 검증

- 수정 전/후 diff가 문구 정정만이고 다른 내용(다른 절)은 안 건드렸는지 확인.
- `python tests/run_ci_sanity_gate.py --profile core_lang` PASS(회귀 없음 — 애초에 문서만 바꾸므로 당연히 영향 없어야 함).

## 수용 기준

- [ ] L52 문구 정정 완료, GOAL-B 보고서 참조 포함
- [ ] 다른 과장 문구 발견 시 함께 정정, 없으면 없다고 보고
- [ ] `pack/lang_flow_hook_interaction_v1/README.md` 수정 없음
- [ ] 코드/checker/golden 변경 없음

## 금지 사항

코드 수정 없음. pack README 수정 없음(이미 정직함). 다른 proposal 문서의 무관한 내용 수정 없음. main 직접 커밋 금지, `codex/queue-20260706` 브랜치에 커밋.

## 보고 형식

이 파일 하단 `## 실행 보고`: 수정한 파일:행, 추가로 발견한 과장 문구(있으면), 검증 결과.
