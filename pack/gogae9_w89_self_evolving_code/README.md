# gogae9_w89_self_evolving_code

- 상태: 제품 CLI 검증 팩 (SSOT v20.3.1)
- 기준: `docs/ssot/walks/gogae9/w89_self_evolving_code/README.md`
- Pack ID: `pack/gogae9_w89_self_evolving_code`

## 범위
- `teul-cli evolve run|emit` 제품 경로를 사용한다.
- `evolve_spec.json`의 정본 AST 후보에서 결정적 변이 5종을 평가한다.
- `generated.ddn`는 CLI가 정본화한 결과만 저장한다.
- `evolve_meta.json`은 seed, score, best hash, final state hash를 고정한다.
