# PROPOSAL_FLOW_HOOK_IMPLEMENTATION_DECISIONS (D51~D58)

> 작성: Claude (2026-07-06) / 상태: 결정 완료(설계), 구현은 Q13-18 게이트 대상
> 근거: GOAL-B(`docs/context/reports/FLOW_HOOK_PHASE_SEPARATION_VERIFICATION_V1.md`) 실측 + 후속 구현 설계 검토(codex 의견 경유)
> 성격: §7.3 흐름씨-훅 위상 분리(SSOT_LANG:2986, MUST) P1~P6를 실제로 구현할 때 따를 8개 설계 결정. **이 문서 자체는 구현이 아니다** — 구현 착수는 Q13-18 게이트 개방 여부를 별도로 결정한 뒤.

## 배경

GOAL-B가 `<<-`가 제품 파서(`tools/teul-cli/src/lang/lexer.rs`)에 전혀 없음을 확인했다. P1~P6를 실제로 구현하기 전에, 8개 설계 갈림길에서 결정을 내려야 나중에 되돌아가는 일이 없다.

## 결정

### D51 — `<<-` 구현 대상 확정
**결정:** 이번 커널 게이트(Q13-18)의 실 구현 대상으로 확정한다. "MUST지만 v다음"으로 미루지 않는다. 흐름씨는 시간/상태 모델의 핵심이라 v1 "동결된 계약"에서 뺄 이유가 없다.
**선행 조건:** D58(red conformance pack)을 먼저 만든다 — 구현 전에 현재 실패를 공식 문서로 고정.

### D52 — AST: 별도 `Stmt::FlowAssign` 노드
**결정:** 기존 `Stmt::Assign`(이미 `deferred: bool` 보유, 미루기/deferred-apply 용도라 무관)에 flag를 얹지 않고 별도 `Stmt::FlowAssign { target, expr, span }`를 신설한다.
**근거:** ordinary assignment와 흐름씨는 실행 시점·재계산 방식이 근본적으로 다르다. 같은 노드에 bool로 얹으면 P3~P5 분석 코드가 두 의미론을 매번 분기해야 해서 장기적으로 더 지저분해진다.

### D53 — `<<-` 허용 위치: top-level/매마디 전용, 훅 본문 안 선언 금지
**결정:** `<<-` 선언은 `(매마디)마다` 블록 또는 top-level에서만 허용한다. `될때`/`인 동안` 훅 본문 안에서 새 `<<-` 규칙을 선언하는 것은 금지(파스 에러 또는 별도 진단).
**중요한 구분(SSOT 정정 필요):** 이 결정은 "훅 본문 안에서 `<<-`로 새 흐름 규칙을 선언하는 것"만 막는다. 기존 SSOT P2("hook body 안에서 흐름 관련 값을 바꿔도... 다음 마디에서만 반영")가 이미 허용하는 "훅 본문에서 ordinary assignment(`<-`)로 흐름 대상 변수를 바꾸는 것"은 계속 허용되고 계속 지연 반영된다. SSOT §7.3에 이 구분을 명시적으로 추가해야 한다.
**근거:** 흐름 규칙 집합이 정적으로 고정되면(D56과 연결) fixed-point 그래프를 마디 시작 전에 한 번에 수집할 수 있어 구현/검증이 크게 단순해진다.

### D54 — Fixed-point 의미: DAG 위상 재계산(1-pass), 반복 수렴 아님
**결정:** v1의 "흐름씨 fixed-point"는 **DAG 위상 정렬 기반 1-pass 재계산**으로 구현한다. 진짜 수치적 반복 수렴(damping/iteration cap/허용오차)이 아니다.
**근거:** P3(다중 출처=병합 아니라 하드 에러)와 P4(순환=fatal, 감쇠 없음)는 애초에 "여러 번 돌려서 수렴시키는" 모델이 아니라 "정확히 하나의 위상 정렬 순서로 한 번에 계산되는" 모델을 전제한다.
**SSOT 보강 필요:** 기존 "fixed-point" 용어는 유지(문서 연속성)하되, "v1은 DAG 위상 재계산 방식으로 구현한다"는 구현 방식 보강 문구를 추가한다.

### D55 — `이전값보기` 저장 범위: 참조된 대상만
**결정:** 모든 변수의 이전 값을 저장하지 않는다. `이전값보기`가 실제로 참조하는 변수만 마디 종료 시점에 스냅샷한다.
**근거:** D56(정적 선언)과 맞물리면 canon 시점에 참조 대상을 전부 알 수 있어 범위를 정확히 좁힐 수 있다. 전체 스냅샷은 불필요한 메모리/구현 비용이다.

### D56 — P3/P4 진단은 정적(canon-time), 동적 흐름 규칙 v1 배제
**결정:** 흐름씨 규칙은 v1에서 전부 정적으로 선언되어야 한다(조건부/동적 생성 금지). `E_FLOW_MULTIPLE_SOURCE_CONFLICT`/`E_FLOW_CIRCULAR_REFERENCE`는 canon/정적 분석 단계에서 진단한다.
**근거:** D53(선언 위치 제한)과 직결 — 선언이 top-level/매마디에서만 정적으로 이뤄지면 전체 규칙 집합이 canon 시점에 이미 확정된다.

### D57 — P6 hash 경계 문구 보강
**결정:** SSOT §7.3 P6에 "훅 body의 상태 변경(state 값 자체)은 `state_hash`에 포함되고, '훅이 발화했다'는 사실 자체(메타 이벤트)만 제외된다"를 명시적으로 추가한다. 기존 규칙 변경 아님, 오독 방지용 문구 보강.

### D58 — pack 분리: 기존은 contract-skeleton, 신규 runtime pack은 구현 후
**결정:** `pack/lang_flow_hook_interaction_v1`은 그대로 두되(README는 이미 정직하게 `docs_first`/`closure_claim: no`로 표기되어 있어 pack 자체는 문제없음), **`PROPOSAL_SSOT_V24_0_9_POST_CLARIFICATION_EXECUTION_SYNC_20260412.md:52`의 "evidence로 잠그는 작업"이라는 문구를 다운그레이드**한다(실제로는 이 pack의 4개 케이스 전부 파싱 단계에서 실패해 관계 자체에 도달하지 못함, GOAL-B 실측).
**후속(구현 후):** `lang_flow_hook_interaction_runtime_v1`을 별도 runner-backed red/green pack으로 신설한다. 이번 결정에는 포함하지 않음.

## 구현 착수 시 권장 순서(D51 승인 시)

`FlowAssign AST 신설 → 정적 흐름 규칙 수집(canon) → DAG 위상 정렬/순환·충돌 진단 → tail-phase 훅 순서 정렬(런타임) → runner-backed pack(D58 후속)`

이 순서는 Q13-18 게이트가 열릴 때 그대로 적용한다. **이 문서 자체는 구현 착수를 승인하지 않는다** — 게이트 개방은 별도 결정.

## 즉시 실행 가능한 항목(게이트 무관)

- D58의 "proposal 문구 다운그레이드" — 코드/의미론 변경 없는 순수 문서 정정이라 Q13-18 게이트와 무관. 별도 브리프로 지금 Codex에 위임(`BRIEF_FLOW_HOOK_PACK_WORDING_FIX_V1.md`).
