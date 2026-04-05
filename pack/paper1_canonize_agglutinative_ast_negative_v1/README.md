# paper1_canonize_agglutinative_ast_negative_v1

논문 1의 바인딩 우선순위 Rule 4를 현재 구현 표면으로 봉인하는 negative D-PACK.

검증 항목:

- `(3을, 1을) 더하.` 입력이 성공 AST를 만들지 않고 결정적으로 실패한다.
- 현재 구현의 실제 실패 표면은 가상 `E_ROLE_BINDING_AMBIGUOUS` JSON이 아니라 parse 오류 stderr다.
- 동일 입력은 항상 `E_PARSE_CALL_PIN_DUPLICATE` / `핀 '왼'에 인자가 중복되었습니다` 표면으로 수렴한다.
- `(1을) 이동.`처럼 같은 조사가 여러 핀 후보에 걸치는 입력도 warning이 아니라 parse 오류 stderr로 수렴한다.
- shared-josa ambiguity는 항상 `E_PARSE_CALL_JOSA_AMBIGUOUS` / `조사 '을'가 모호합니다. 값:핀 또는 ~조사로 고정하세요` 표면으로 고정된다.

실행:

- `python tests/run_canon_ast_dpack.py paper1_canonize_agglutinative_ast_negative_v1`
- 갱신: `python tests/run_canon_ast_dpack.py --update paper1_canonize_agglutinative_ast_negative_v1`

참고:

- 이 pack은 success용 `canon.ddn/ast.detjson/ast_hash.txt` 표준 산출물 대신 실제 stderr golden을 저장한다.
- success warning 경계는 `paper1_canonize_agglutinative_ast_v2`의 `c05_conflict_warning`가 담당한다.
- 이 pack은 warning으로 해결되지 않는 deterministic error surface만 봉인한다.
