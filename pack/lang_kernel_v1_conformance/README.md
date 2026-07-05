# lang_kernel_v1_conformance

LANG_KERNEL_V1 제안 스펙 §14의 적합성 팩 골격이다.

- 근거 브리프: `docs/context/briefs/BRIEF_LANG_KERNEL_CONFORMANCE_PACK_V1.md`
- 목적: 0705 큐 Q13~Q18 커널 구현 시리즈의 게이트 해제 전, 현재 제품 CLI 동작을 red/green 기준선으로 고정한다.
- 실행 경로: `python tests/run_pack_golden.py lang_kernel_v1_conformance`
- 실제 캡처 로그: `I:/home/urihanl/ddn/codex/out/queue-20260706/q-conformance/case_results.json`
- 2026-07-06 Q18 진단 반영: bare `이름꼬리.` 호출은 제품 실행 파서에서 `살림.이름꼬리` 경로로 내려가므로, 그룹1 형태론 호출 케이스 3건의 호출부를 `() 이름꼬리.` 접두 호출로 정정했다.

## 케이스

| id | 그룹 | 예상 | 실제 | 비고 |
|---|---|---|---|---|
| `stem_alias_dop_dou` | 형태론 회귀 | 그린 예상 | PASS | Q18 반영: `() 돕기.` 접두 호출로 정정 후 예상대로. |
| `stem_alias_ambiguous` | 형태론 회귀 | 레드 예상(E_CALL_TAIL_AMBIGUOUS) | PASS | 예상과 다름: `() 계산하기.` 접두 호출로 정정해도 현재 제품 run 경로는 모호성 오류 없이 실행. |
| `tail_equiv_gi_hagi` | 형태론 회귀 | 그린 예상 | PASS | Q18 반영: `() 회복기.`/`() 회복하기.` 접두 호출로 정정 후 동일 출력. |
| `condition_new_surface` | 형태론 회귀 | 그린 예상 | PASS | 예상대로. |
| `time_madisai_undefined` | 시간 | 레드 예상 | FAIL(E_RUNTIME_UNDEFINED) | 예상대로. |
| `time_jigeum_undefined` | 시간 | 레드 예상 | FAIL(E_RUNTIME_UNDEFINED) | 예상대로. |
| `vector2_construct_undefined` | 벡터2/텐서 | 레드 예상 | FAIL(E_PARSE_UNEXPECTED_TOKEN) | 예상대로. |
| `tensor_existing_baseline` | 벡터2/텐서 | 그린 예상 | PASS | 예상대로. |
| `option_syntax_undefined` | Option/Result | 레드 예상 | FAIL(E_LEX_UNEXPECTED_CHAR) | 예상대로. |
| `result_matum_undefined` | Option/Result | 레드 예상 | FAIL(E_PARSE_UNEXPECTED_TOKEN) | 예상대로. |
| `value_ref_tail_undefined` | Option/Result | 레드 예상 | FAIL(E_RUNTIME_UNDEFINED) | 예상대로. |
| `overflow_saturate_current` | 오버플로 | 현재 그린(전환 후 의도적 파손) | PASS | 현재 포화 기준선. E_NUM_OVERFLOW 전환 시 의도적으로 깨져야 함. |
| `relative_clause_undefined` | 포함절 | 레드 예상 | FAIL(E_LEX_BAD_IDENT_START) | 예상대로. |

## 주의

- `value_ref_tail_undefined`는 브리프의 12개 기본 케이스와 별도로 요구된 SSOT 개정 대기 레드 케이스라 총 케이스 수는 13개다.
- `stem_alias_dop_dou`, `tail_equiv_gi_hagi`는 Q18 진단 뒤 호출부를 접두 호출로 정정해 제품 run 경로에서 PASS한다.
- `stem_alias_ambiguous`는 접두 호출로 정정해도 현재 제품 run 경로에서 `E_CALL_TAIL_AMBIGUOUS`가 나지 않고 PASS한다. 브리프 기대와 달라 golden은 임의 갱신하지 않았고 추가 진단이 필요하다.
- `overflow_saturate_current`는 현재 `2147483647.9999999997`로 포화되는 기준선이다. D4 전환 후 `E_NUM_OVERFLOW` 정책이 들어오면 이 케이스는 의도적으로 깨지고 갱신되어야 한다.
