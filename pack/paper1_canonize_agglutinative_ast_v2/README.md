# paper1_canonize_agglutinative_ast_v2

논문 1의 교착어 AST 증빙을 단일 D-PACK으로 묶은 v2 pack.

검증 항목:

- `c01_ab`, `c02_ba`, `c04_alias_acceptance`는 동일한 정본 AST detjson으로 수렴한다.
- `c03_positional`은 독립 expected로 성공하며 `binding_reason = "positional"`을 유지한다.
- `c05_conflict_warning`는 shared-josa를 `값:핀~조사`로 고정한 성공 입력이며 `W_CALL_JOSA_CONFLICT_FIXED` warning을 남긴다.
- `c06_ambiguous_reject`는 `E_PARSE_CALL_PIN_DUPLICATE` stderr로 거절된다.
- `c07_shared_josa_ambiguous_reject`도 warning이 아니라 `E_PARSE_CALL_JOSA_AMBIGUOUS` stderr로 거절된다.
- pack 내부 동치 검증은 `equivalence_groups`로만 강제한다.

실행:

- `python tests/run_canon_ast_dpack.py paper1_canonize_agglutinative_ast_v2`
- 갱신: `python tests/run_canon_ast_dpack.py --update paper1_canonize_agglutinative_ast_v2`

참고:

- `c05_conflict_warning`는 ambiguous josa를 성공 경로에서 `값:핀` 고정으로 푼 현재 구현 warning 표면을 봉인한다.
- 여전히 warning 없이 성공하는 bare reorder(`c02_ba`)와는 별도 경계다.
- 기존 `*_v1`, `*_positional_v1`, `*_alias_acceptance_v1`, `*_negative_v1` pack은 호환용으로 유지한다.
