# paper1_canonize_agglutinative_ast_v1

교착어 조사 어순이 다른 두 입력이 동일한 정본 AST detjson으로 수렴하는지 검증하는 D-PACK.

검증 항목:

- `A를 B에 더하기`와 `B에 A를 더하기` 표면 입력이 동일 정본 AST JSON으로 수렴
- 두 케이스 출력이 바이트 단위로 완전히 동일
- success artifact에 불필요한 tail-less lint warning이 섞이지 않도록 입력 표면은 tailed call(`더하기.`)로 고정

실행:

- `python tests/run_canon_ast_dpack.py paper1_canonize_agglutinative_ast_v1`
- 갱신: `python tests/run_canon_ast_dpack.py --update paper1_canonize_agglutinative_ast_v1`
