# seamgrim_graph_autorender_v1

`그래프_` / `보개_그래프_` 접두어 자동 그래프 렌더 계약 검증 팩.

검증:

```bash
python tests/run_seamgrim_graph_autorender.py
```

검증 내용:

- value 리소스에 접두어 키가 있으면 `seamgrim.graph.v0`가 생성된다.
- 시리즈 이름은 원본 태그를 유지한다.
- 점열 파싱은 `짝맞춤{"x"=>...,"y"=>...}` 패턴을 따른다.
