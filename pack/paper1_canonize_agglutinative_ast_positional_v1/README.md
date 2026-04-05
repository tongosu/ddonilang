# paper1_canonize_agglutinative_ast_positional_v1

논문 1의 바인딩 우선순위 Rule 2를 즉시 단계로 검증하는 D-PACK.

검증 항목:

- 조사 생략 입력 `(3, 1) 더하.`가 구성별 위치 기본값으로 해석된다.
- 호출 인자 AST에 `binding_reason = "positional"`과 `josa = null`이 고정된다.
- success artifact에 warning이 남지 않도록 입력 표면은 `더하기.`로 고정한다.

실행:

- `python tests/run_canon_ast_dpack.py paper1_canonize_agglutinative_ast_positional_v1`
- 갱신: `python tests/run_canon_ast_dpack.py --update paper1_canonize_agglutinative_ast_positional_v1`

참고:

- 현재 canon D-PACK 러너는 pack 내부 모든 출력의 바이트 동일성을 강제하므로, 이 케이스는 독립 pack으로 분리한다.
