# BRIEF: 모호성 강제(E_CALL_TAIL_AMBIGUOUS)의 실제 제품 경로 검증

> 작성: Claude (2026-07-06) / 실행: Codex / 리뷰: Claude
> 성격: 진단 전용 — 코드 수정 금지. Q18과 같은 방법론(사실+파일:행 근거, 추측 금지).
> 배경: `pack/lang_kernel_v1_conformance/cases/stem_alias_ambiguous.ddn`(`계산:셈씨`와 `계산하:셈씨`를 같은 스코프에 선언 후 `() 계산하기.` 호출)이 실제 `teul-cli run`에서 오류 없이 정상 실행됨. SSOT §11.3.1("모호성은 추측 금지, 후보 2개 이상이면 오류")과 어긋나는 것으로 보임. `lang/src/parser.rs:5736`의 `resolve_call_tail_candidates`(E_CALL_TAIL_AMBIGUOUS 로직)는 Q18이 이미 "teul-cli run이 실제로 쓰는 경로가 아니다"라고 확인한 바 있음.

## 질문 (사실만, 추측 금지)

1. `tools/teul-cli/src/lang/parser.rs`(실제 `teul-cli run`이 쓰는 파서)에 `계산하기` 같은 호출 표면을 어간+꼬리로 분해하는 로직이 존재하는가? 존재한다면 파일:행. 존재한다면 후보가 여러 개일 때(예: `계산`+`하기`도 되고 `계산하`+`기`도 되는 경우) 어떻게 처리하는가 — ①첫 매치를 그냥 쓰는가 ②둘 다 시도해서 성공하는 쪽을 쓰는가 ③다른 방식인가?
2. `stem_alias_ambiguous.ddn`을 실제로 재실행해, `() 계산하기.`가 정확히 어느 씨앗(`계산` 또는 `계산하`)에 dispatch됐는지 확인하라. 두 씨앗이 각각 다른 값을 돌려주게 만들어(예: 1 돌려줘 vs 2 돌려줘, 이미 그렇게 되어 있음) 어느 쪽이 호출됐는지 실행 결과(`state_hash`/`trace_hash`가 아니라 실제 반환값을 stdout에 보이도록 케이스를 임시로 조정해도 됨 — 단 원본 파일은 건드리지 말고 scratch 사본으로)로 확정하라.
3. `E_CALL_TAIL_AMBIGUOUS`라는 오류 코드 문자열이 `tools/teul-cli/src/**/*.rs` 전체에 실제로 존재하는가? 존재한다면 어디서 발생 조건을 검사하는지(파일:행), 그 조건이 이 케이스에서 왜 발동 안 했는지.
4. SSOT §11.3.1("모호성은 추측 금지")이 실제로 강제되는 다른 사례(다른 팩/골든 테스트)가 있는지 저장소 전체에서 검색하라 — 있다면 그 사례와 이번 케이스의 차이가 뭔지.

## 결론 요구사항

다음 중 어느 쪽인지 근거와 함께 명시: **(A) 실제 제품 파서에도 모호성 검사가 있으나 이 케이스의 조건을 못 미침(설계상 이 경우는 모호성으로 안 침) / (B) 실제 제품 파서에 모호성 검사 자체가 없음(SSOT MUST 미구현) / (C) 다른 이유**.

## 수용 기준

- [ ] 4개 질문 전부 파일:행 근거 또는 "찾지 못함"
- [ ] 실제 dispatch 대상 씨앗을 실행으로 확정(추측 금지)
- [ ] 코드 수정 없음, `git status --short` 깨끗(scratch 사본은 저장소 밖 또는 명시된 out/ 경로에만)
- [ ] 결론 A/B/C 중 하나 + 근거

## 금지 사항

코드 수정 / golden 갱신 / 범위 밖 조사. `codex/queue-20260706` 브랜치에 커밋 1개.

## 보고 형식

이 파일 하단 `## 실행 보고`.

## 실행 보고

- 실행일: 2026-07-06
- 브랜치: `codex/queue-20260706`
- 성격: 진단 전용. 코드 수정/golden 갱신/네트워크 없음.
- scratch 경로: `I:/home/urihanl/ddn/codex/out/queue-20260706/q20-ambiguity/`

### 재현

원본 케이스:

```powershell
I:/home/urihanl/ddn/codex/target/debug/teul-cli.exe run pack/lang_kernel_v1_conformance/cases/stem_alias_ambiguous.ddn
```

결과:

```text
state_hash=blake3:f83311d1881e4ab122c25dd8c3a9e30349858a9cac5f6066f609c52c4ed9db45
trace_hash=blake3:87696537a1552c09a4cd26b2cc45298e8a2952224f974f1692dfdc8f9663c438
```

`E_CALL_TAIL_AMBIGUOUS` 없이 정상 종료한다. 원본은 반환값을 stdout에 보이지 않으므로 scratch 사본에서 호출 결과를 변수에 담아 출력했다.

```ddn
계산:셈씨 = {
  1 돌려줘.
}

계산하:셈씨 = {
  2 돌려줘.
}

결과 <- () 계산하기.
결과 보여주기.
```

실행 결과:

```text
1
state_hash=blake3:b49b9ae8740e22ff4c91d06be5e25bfcc18c267a81b68b110bc6927af8bf0878
trace_hash=blake3:560566c2c9a068201e5aab5e1ccf4e3fa9b4feaedb1532f91d82c8bc3ff2a108
```

따라서 `() 계산하기.`는 `계산하`가 아니라 `계산` 씨앗으로 dispatch된다. 보조 scratch에서 `계산하:셈씨`만 남기면 같은 표면이 `2`를 출력해, 런타임이 `하기` 실패 후 `기` 후보로 넘어갈 수 있음도 확인했다.

### 질문별 답

1. `tools/teul-cli/src/lang/parser.rs`에 `계산하기`를 어간+꼬리 후보 전체로 분해하고 모호성을 검사하는 로직은 찾지 못했다. 실제 제품 파서는 `() 이름` 표면을 `parse_zero_arg_prefix_call_stmt()`에서 `Expr::Call { name, args: Vec::new() }`로 만든다(`tools/teul-cli/src/lang/parser.rs:887`, `:894`, `:897`, `:901`). `parse_call_name()`은 `Ident`와 점 segment를 문자열로 합칠 뿐, 꼬리 후보를 만들지 않는다(`tools/teul-cli/src/lang/parser.rs:5588`, `:5591`, `:5601`, `:5612`).
2. 실제 dispatch 대상은 `계산`이다. scratch 사본에서 `결과 <- () 계산하기.` 뒤 `결과 보여주기.`를 실행하자 stdout이 `1`이었다. `계산:셈씨`는 `1`, `계산하:셈씨`는 `2`를 돌려주므로 dispatch 대상은 `계산`으로 확정된다.
3. `E_CALL_TAIL_AMBIGUOUS` 문자열은 `tools/teul-cli/src/**/*.rs`에 없다. `rg -n 'E_CALL_TAIL_AMBIGUOUS' tools/teul-cli/src -g '*.rs'` 결과는 `NO_MATCH tools/teul-cli/src`였다. 반면 `lang/src/parser.rs`에는 후보를 모아 2개 이상이면 오류를 내는 별도 로직이 있다(`lang/src/parser.rs:5736`, `:5759`, `:5769`, `:5774`, `:6188`). 이 경로는 Q18에서 확인한 대로 `teul-cli run` 제품 경로가 아니다.
4. 저장소 전체 검색에서 `E_CALL_TAIL_AMBIGUOUS`는 `lang/src/parser.rs`와 문서/보고/팩 설명에만 나타났다. 실제 pack golden이나 `tools/teul-cli` 제품 테스트에서 이 오류를 강제하는 사례는 찾지 못했다. `pack/lang_kernel_v1_conformance/README.md`도 현재 제품 run 경로에서 `stem_alias_ambiguous`가 모호성 오류 없이 PASS한다고 명시한다.

### 실제 원인

파서는 `() 계산하기.`를 꼬리 분석 없이 `Expr::Call(name="계산하기")`로 만든다. 그 뒤 런타임 `eval_call()`이 사용자 씨앗 직접 매치를 먼저 시도하고(`tools/teul-cli/src/runtime/eval.rs:2215`), 실패하면 꼬리 목록을 순서대로 strip한다(`tools/teul-cli/src/runtime/eval.rs:2219`, `:2220`, `:2221`). 목록 순서는 `["하면서", "면서", "하기", "기", "하고", "고", "하면", "면"]`이며, `계산하기`에서 `하기`를 먼저 떼면 `계산`이 된다. `계산` 씨앗을 찾는 즉시 `eval_user_seed()`로 반환한다(`tools/teul-cli/src/runtime/eval.rs:2222`, `:2223`, `:2224`). 그래서 `기`를 떼어 `계산하` 후보도 가능한지까지 세지 않는다.

### 결론

**B) 실제 제품 파서/제품 실행 경로에 모호성 검사 자체가 없음(SSOT MUST 미구현).**

엄밀히는 제품 파서가 아니라 제품 런타임이 꼬리 제거 dispatch를 수행하며, 이 런타임 로직이 "후보 전체 수집 후 0개/2개 이상 진단" 구조가 아니라 "정해진 꼬리 순서에서 첫 성공 후보 즉시 실행" 구조다. 따라서 `계산`과 `계산하`가 동시에 존재해도 `계산하기`는 `하기` 우선으로 `계산`에 dispatch되고 `E_CALL_TAIL_AMBIGUOUS`가 발동할 지점이 없다.

### 상태 확인

- Q20 자체 코드 수정 없음.
- scratch 파일은 저장소 밖 `out/` 경로에만 작성.
- 작업 중 `git status --short`에는 Q20 착수 전부터 존재하던 `docs/context/briefs/QUEUE_CODEX_20260706.md` 수정만 남아 있었다.
