# Flow-hook phase separation verification v1

## Scope

- 목적: SSOT_LANG §7.3 `흐름씨-훅 위상 분리`의 P1~P6가 현재 제품 경로(`tools/teul-cli`)에서 실제로 집행되는지 확인한다.
- 제한: 진단 전용. 코드, 팩, golden, checker는 수정하지 않았다.
- 실행 경로: `cargo run --manifest-path tools/teul-cli/Cargo.toml -- run <case>/input.ddn`.
- 대상 팩: `pack/lang_flow_hook_interaction_v1`.

## Executive Finding

현재 제품 teul-cli 경로에서는 `<<-` 흐름씨 표면이 런타임까지 도달하지 않는다. `lang/src`에는 `DoubleArrow` 토큰이 있지만, 제품 실행 경로가 쓰는 `tools/teul-cli/src/lang/token.rs`에는 `DoubleArrow`가 없고, `tools/teul-cli/src/lang/lexer.rs`도 `<<-`를 단일 토큰으로 읽지 않는다. 따라서 네 팩 케이스는 모두 `E_PARSE_EXPECTED_EXPR`로 끝났고, P1~P6의 fixed-point/충돌/cycle/state_hash 의미는 현재 제품 코드에서 집행된다고 볼 수 없다.

## Source Evidence

| claim | evidence |
|---|---|
| SSOT §7.3은 ordinary assignment -> flow fixed-point -> tail-phase hook 순서와 P1~P6를 MUST로 둔다. | `docs/ssot/ssot/SSOT_LANG_v24.12.9.md:2986`, `:2991`, `:2992`, `:2993`, `:2997`-`:3002` |
| 제품 run 경로는 `parse_program_for_runtime_with_mode`를 호출한다. | `tools/teul-cli/src/cli/run.rs:4318` |
| 제품 frontdoor는 `tools/teul-cli`의 `Lexer::tokenize`와 `Parser::parse_with_default_root_mode`를 사용한다. | `tools/teul-cli/src/cli/frontdoor_parse.rs:28`-`:33` |
| 제품 토큰 enum에는 `Arrow`와 `SignalArrow`는 있으나 `DoubleArrow`가 없다. | `tools/teul-cli/src/lang/token.rs:42`-`:78` |
| 제품 lexer는 `<-`만 `TokenKind::Arrow`로 읽고, `<<-` 분기가 없다. | `tools/teul-cli/src/lang/lexer.rs:308`-`:317` |
| 제품 parser의 assignment 분기는 `Arrow`, `Equal`, `PlusArrow`, `MinusArrow`만 받는다. | `tools/teul-cli/src/lang/parser.rs:689`-`:803` |
| runtime tick loop는 `every_hooks` -> `every_n_hooks` -> `becomes_hooks` -> `while_hooks` 순으로 훅을 실행하며 별도 flow fixed-point pass가 없다. | `tools/teul-cli/src/runtime/eval.rs:735`-`:811` |
| runtime `Stmt::Assign`은 표현식을 평가해 state에 직접 set/deferred 기록만 한다. flow assignment 변종이 없다. | `tools/teul-cli/src/runtime/eval.rs:921`-`:956` |
| 지원 진단 문자열은 팩/체커/SSOT에는 있지만 `tools/teul-cli/src`에는 없다. | `rg E_FLOW_MULTIPLE_SOURCE_CONFLICT/E_FLOW_CIRCULAR_REFERENCE tools/teul-cli/src` 결과 없음 |
| pack README도 `evidence_tier: docs_first`, `closure_claim: no`, 실제 runtime fixed-point/conflict/cycle은 별도 closure 대상이라고 적는다. | `pack/lang_flow_hook_interaction_v1/README.md:5`-`:10` |
| pack checker는 contract/expected 파일 구조를 검사할 뿐 DDN product run을 하지 않는다. | `tests/run_lang_flow_hook_interaction_pack_check.py:40`-`:100` |
| `lang/src`의 별도 lexer에는 `DoubleArrow`가 존재한다. 이는 제품 teul-cli parser 집행과 분리해서 봐야 한다. | `lang/src/lexer.rs:61`-`:65`, `:211`-`:218` |

## Product Case Runs

| case | contract expectation | exit | actual product result | judgment |
|---|---|---:|---|---|
| `c01_flow_then_hook_snapshot` | ok / 흐름씨 뒤 훅 스냅샷 | 1 | `E_PARSE_EXPECTED_EXPR` at `input.ddn:14:8` | FAIL: `<<-` parse 단계에서 차단 |
| `c02_no_same_tick_refire` | ok / 같은마디 재발화 금지 | 1 | `E_PARSE_EXPECTED_EXPR` at `input.ddn:12:6` | FAIL: `<<-` parse 단계에서 차단 |
| `c03_multiple_flow_source_conflict` | diag / E_FLOW_MULTIPLE_SOURCE_CONFLICT | 1 | `E_PARSE_EXPECTED_EXPR` at `input.ddn:11:8` | FAIL: `<<-` parse 단계에서 차단 |
| `c04_flow_cycle_fatal` | diag / E_FLOW_CIRCULAR_REFERENCE | 1 | `E_PARSE_EXPECTED_EXPR` at `input.ddn:7:6` | FAIL: `<<-` parse 단계에서 차단 |

지원 체커 `python tests/run_lang_flow_hook_interaction_pack_check.py`는 PASS했다. 다만 이 체커는 `contract.detjson`/`expected.json`의 문서 계약을 검사하는 docs-first checker이며, 위 DDN 입력을 제품 런타임으로 실행하지 않는다.

## P1-P6 Enforcement Table

| point | SSOT requirement | current product status | evidence |
|---|---|---|---|
| P1 | 훅은 흐름씨 fixed-point 이전에 실행되지 않는다. | 미집행. 제품 tick loop에 fixed-point 단계가 없고 `<<-`가 parse되지 않는다. | `eval.rs:735`-`:811`, `parser.rs:689`-`:803` |
| P2 | hook body는 같은 마디의 흐름씨 fixed-point를 재발화하지 않는다. | 미검증/미집행. flow phase 자체가 없어서 재발화 금지 의미가 런타임에 닿지 않는다. | c02 direct run `E_PARSE_EXPECTED_EXPR`; `eval.rs:773`-`:811` |
| P3 | 같은 마디 같은 변수의 다중 흐름씨 규칙 충돌은 `E_FLOW_MULTIPLE_SOURCE_CONFLICT`. | 미집행. c03은 기대 진단이 아니라 parse error로 실패하고, 해당 error code emission point가 제품 코드에 없다. | c03 direct run; `rg` 결과; `contract.detjson`만 기대값 보유 |
| P4 | 흐름씨 dependency cycle은 `E_FLOW_CIRCULAR_REFERENCE` FATAL. | 미집행. c04는 기대 진단이 아니라 parse error로 실패하고, 해당 error code emission point가 제품 코드에 없다. | c04 direct run; `rg` 결과; `contract.detjson`만 기대값 보유 |
| P5 | 같은 변수에 ordinary assignment와 흐름씨가 모두 있으면 ordinary assignment 후 흐름씨 재계산. | 미집행. 제품 runtime은 일반 `Stmt::Assign` state set/deferred만 처리하며 flow assignment와 재계산 pass가 없다. | `eval.rs:921`-`:956` |
| P6 | 흐름씨 fixed-point 결과는 `state_hash`에 포함되고 훅 발화 사실 자체는 state_hash 입력이 아니다. | 부분 기반만 있음. `state_hash`는 `State` encoding을 해시하지만, fixed-point 결과를 만드는 제품 단계가 없어 P6 flow 의미는 검증 불가. | `tools/teul-cli/src/core/hash.rs:8`-`:10`; flow fixed-point pass 부재 |

## Conclusion

- SSOT/contract pack 수준의 문서 계약은 존재한다.
- 제품 teul-cli 경로의 `<<-` surface, flow fixed-point pass, P3/P4 진단 emission point는 현재 랜딩되어 있지 않다.
- 따라서 §7.3 P1~P6는 현재 제품 실행 기준으로 behavior-closed가 아니다. 다음 작업이 필요하다면 별도 구현 브리프에서 lexer/parser AST/runtime fixed-point/diagnostic/state_hash 회귀를 다뤄야 하며, 이 보고서는 설계나 수리를 수행하지 않았다.
