# seamgrim_graph_autorender_v1

`그래프_` / `보개_그래프_` 접두어 자동 그래프 렌더 계약 검증 팩.

검증:

```bash
python tests/run_seamgrim_graph_autorender.py
# 또는 전체 골든 게이트
python tests/run_pack_golden.py --all
```

검증 내용:

- value 리소스에 접두어 키가 있으면 `seamgrim.graph.v0`가 생성된다.
- 시리즈 이름은 원본 태그를 유지한다.
- 점열 파싱은 `짝맞춤{"x"=>...,"y"=>...}` 패턴을 따른다.

`golden.jsonl` 케이스 키(autorender 전용):

- `fixture`: 입력 detjson 경로
- `expected_graph`: 기대 그래프 JSON 경로
- `prefer_patch`(선택): patch 우선 경로 사용 여부
