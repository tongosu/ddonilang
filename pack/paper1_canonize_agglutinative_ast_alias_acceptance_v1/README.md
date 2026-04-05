# paper1_canonize_agglutinative_ast_alias_acceptance_v1

논문 1의 조사 별칭 입력이 정본 AST로 수렴하는지 검증하는 D-PACK.

참고:

- 디렉터리 이름은 초기 immediate phase 이름을 유지하지만, 현재 출력은 alias acceptance를 넘어 alias normalization까지 포함한다.

검증 항목:

- `(3를, 1에) 더하.` 입력이 성공적으로 정본 AST detjson으로 직렬화된다.
- `를`가 정본 AST와 `normalized_n1`에서 canonical `을`로 수렴한다.
- 현재 expected 출력은 `paper1_canonize_agglutinative_ast_v1`와 바이트 단위로 동일하다.
- success artifact에 warning이 남지 않도록 입력 표면은 `더하기.`로 고정한다.

실행:

- `python tests/run_canon_ast_dpack.py paper1_canonize_agglutinative_ast_alias_acceptance_v1`
- 갱신: `python tests/run_canon_ast_dpack.py --update paper1_canonize_agglutinative_ast_alias_acceptance_v1`

참고:

- 별도 alias normalization pack 없이도 이 pack 자체가 alias normalization 수렴을 보여준다.
