# D-PACK: diag_contract_seulgi_hook

## 목적
- 계약 위반 시 슬기 훅 기록 이벤트가 geoul.diag.jsonl에 남는지 확인한다.

## 구성
- input.ddn
- tests/README.md
- `tools/teul-cli/tests/golden/W107/README.md` 인덱스
- `python tools/teul-cli/tests/run_w107_golden_index_selfcheck.py` self-check
- `python tests/run_w107_progress_contract_selftest.py` transport bundle self-check
- `python tests/run_ci_sanity_gate.py --profile core_lang` 내 `w107_golden_index_selfcheck`
- `python tests/run_ci_sanity_gate.py --profile core_lang` 내 `w107_progress_contract_selftest`
- progress snapshot schema: `ddn.ci.w107_golden_index_selfcheck.progress.v1`
- progress snapshot schema: `ddn.ci.w107_progress_contract_selftest.progress.v1`
- sanity stdout token: `w107_golden_index_selfcheck_active_cases`, `w107_golden_index_selfcheck_inactive_cases`, `w107_golden_index_selfcheck_index_codes`
- sanity stdout token: `w107_progress_contract_selftest_completed_checks`, `w107_progress_contract_selftest_total_checks`, `w107_progress_contract_selftest_checks_text`

## DoD(최소)
- 계약 diag와 함께 hook 기록(event_kind/hook_name/hook_input_ref)이 남는다.
- hook 기록에는 선택된 정책(`hook_policy`)이 함께 남는다.
- hook 기록에는 AGE2 실행 준비용 inline payload(`hook_input.code/file/line/col/message/fault_id/contract_kind/mode`)가 포함된다.
- `슬기훅정책: 실행` + AGE2 이상에서는 `hook_result` 이벤트가 추가로 남고, builtin text suggestion 2건을 포함한다.
- 같은 `hook_result`는 상수 거짓 계약식(`0`, `거짓`, `1 == 2`, `1 != 1`, `(1 < 0)` 등)에 대해 draft `replace_block` patch candidate 1건을 함께 남긴다.
- 같은 `hook_result`는 직전 상수 대입이 보이는 엄격 부등호 실패(`x <- 5.` 다음 `x > 10`)에 대해서도 관측값 기준 완화 patch 후보(`x >= 5`)를 남길 수 있다.
- 같은 `hook_result`는 직전 상수 대입이 보이는 동치 실패(`x <- 5.` 다음 `x == 10`)에 대해서도 관측값 기준 동치 patch 후보(`x == 5`)를 남길 수 있다.
- 위 관측값 역추적은 괄호 친 상수 대입(`x <- (5).`, `x <- (-5).`)도 읽는다.
- 위 관측값 역추적은 plain negative 대입(`x <- -5.`)도 읽는다.
- 위 관측값 역추적은 unary plus 대입(`x <- +5.`, `x <- (+5).`)도 읽는다.
- 위 관측값 역추적은 기본/명시 루트 경로 대입(`x <- +5.`, `공.속도.x <- +5.`, `x <- +5.`, `공.속도.x <- +5.`)도 bare 계약식(`x == 10`, `공.속도.x == 10`)과 연결해 읽는다.
- 위 관측값 역추적 helper는 typed lhs(`x:수`, `공.속도.x:수`)와 ``/`` prefix를 벗겨 bare 대상(`x`, `공.속도.x`)으로 정규화한다.
- 같은 helper 기준에서 bare leaf target(`x`)은 이미 최종 대상이므로, source 루트 객체 재대입(`살림 <- ...`, `바탕 <- ...`)은 `x`의 상위 경로로 보지 않는다.
- 기본 루트 계약식(`{ x == 10 }`)도 같은 정규화 규칙으로 patch 후보(`{ x == 5 }`)를 받을 수 있다.
- 기본 루트 nested 계약식(`{ 공.속도.x == 10 }`)도 같은 정규화 규칙으로 patch 후보(`{ 공.속도.x == 5 }`)를 받을 수 있다.
- 명시 루트 계약식(`{ x == 10 }`)도 같은 정규화 규칙으로 patch 후보(`{ x == 5 }`)를 받을 수 있다.
- 명시 루트 nested 계약식(`{ 공.속도.x == 10 }`)도 같은 정규화 규칙으로 patch 후보(`{ 공.속도.x == 5 }`)를 받을 수 있다.
- cross-root 조합(`x <- +5.` 뒤 `{ x == 10 }`, `x <- +5.` 뒤 `{ x == 10 }`)도 같은 bare 대상 정규화로 patch 후보를 받을 수 있다.
- nested cross-root 조합(`공.속도.x <- +5.` 뒤 `{ 공.속도.x == 10 }`, `공.속도.x <- +5.` 뒤 `{ 공.속도.x == 10 }`)도 같은 bare 대상 정규화로 patch 후보를 받을 수 있다.
- nested cross-root 조합도 source 쪽 최신 간접 대입(`공.속도.x <- y.`, `공.속도.x <- y.`)이 있으면 더 오래된 하위 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- nested cross-root 조합도 source 쪽 최신 상위 경로 재대입(`공 <- ...`, `공 <- ...`)이 있으면 더 오래된 하위 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- nested cross-root 조합은 source 쪽 sibling 경로 재대입(`공.다른 <- 7.`, `공.다른 <- 7.`)이 와도 대상 경로를 덮지 않으면 patch 후보를 유지한다.
- 기본 루트 계약식 target도 같은 rooted target에 최신 간접 대입(`x <- y.`)이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 기본 루트 nested 계약식 target도 같은 rooted nested target에 최신 간접 대입(`공.속도.x <- y.`)이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 명시 루트 계약식 target도 같은 rooted target에 최신 간접 대입(`x <- y.`)이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 명시 루트 nested 계약식 target도 같은 rooted nested target에 최신 간접 대입(`공.속도.x <- y.`)이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 기본 루트 nested 계약식 target도 같은 rooted nested target에 최신 상위 경로 재대입(`공 <- ...`)이 있으면 더 오래된 하위 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 명시 루트 nested 계약식 target도 같은 rooted nested target에 최신 상위 경로 재대입(`공 <- ...`)이 있으면 더 오래된 하위 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 기본 루트 nested 계약식 target은 sibling 경로 재대입(`공.다른 <- 7.`)이 와도 대상 경로를 덮지 않으면 patch 후보를 유지한다.
- 명시 루트 nested 계약식 target은 sibling 경로 재대입(`공.다른 <- 7.`)이 와도 대상 경로를 덮지 않으면 patch 후보를 유지한다.
- 관측값 경로가 계약식 대상과 다르면(`y <- +5.` 뒤 `x == 10`) patch 후보를 만들지 않는다.
- 같은 대상이라도 비상수/간접 대입(`x <- y.`)은 관측값 후보로 승격하지 않는다.
- 같은 대상에 최신 비상수 대입이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 같은 대상의 최신 계산식 대입이 정수 상수식(`x <- 2 + 3.`, `x <- (2 + 3).`)이면 계산 결과를 관측값으로 승격해 patch 후보를 만든다.
- 같은 `` explicit root 대상에 최신 비상수 대입(`x <- y.`)이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 같은 nested path 대상(`공.속도.x <- y.`)에 최신 비상수 대입이 있으면 더 오래된 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 같은 nested path 대상의 최신 괄호 계산식 대입(`공.속도.x <- (2 + 3).`)도 계산 결과를 관측값으로 승격해 patch 후보를 만든다.
- 같은 nested path 대상에 최신 상위 경로 재대입(`공 <- ("속도", ("x", 7) 짝맞춤) 짝맞춤.`)이 있으면 더 오래된 하위 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 같은 `` explicit root nested target에 최신 상위 경로 재대입(`공 <- ("속도", ("x", 7) 짝맞춤) 짝맞춤.`)이 있으면 더 오래된 하위 상수 대입으로 되감아 patch 후보를 만들지 않는다.
- 같은 nested target이라도 sibling 경로 재대입(`공.다른 <- 7.`)은 대상 경로를 덮지 않으므로 기존 하위 상수 관측값 patch 후보를 막지 않는다.
- 같은 `` explicit root nested target이라도 sibling 경로 재대입(`공.다른 <- 7.`)은 대상 경로를 덮지 않으므로 기존 하위 상수 관측값 patch 후보를 막지 않는다.
- descendant 경로 재대입(`공.속도.x.세부 <- 7.`)은 helper 기준으로 대상 leaf `공.속도.x`의 최신 대입으로 보지 않는다. 실제 실행 표면에서는 leaf int child write가 타입 불일치로 막힌다.
- 같은 식별자에 상수 대입이 여러 번 있으면 가장 최근 상수 대입을 우선 사용한다.
