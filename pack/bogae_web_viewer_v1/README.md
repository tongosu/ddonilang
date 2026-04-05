# pack/bogae_web_viewer_v1

evidence_tier: golden_closed

Bogae Web Viewer v1 스모크.

- 목표:
  - web viewer 산출물 경로를 생성한다.
  - `bogae_hash`가 결정적으로 출력된다.

## 입력
- `input.ddn`

## 골든
- `golden.jsonl`

## Family Pointer
- 상위 family: `tests/bogae_alias_viewer_family/README.md`
- 검증: `python tests/run_bogae_alias_viewer_family_selftest.py`
