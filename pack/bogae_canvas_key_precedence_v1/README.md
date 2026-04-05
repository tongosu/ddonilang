# D-PACK: bogae_canvas_key_precedence_v1

## 목적
- 보개 캔버스 키가 동시에 주어질 때 정본 `보개_그림판_가로/세로`가 legacy `bogae_canvas_w/h`보다 우선하는지 검증한다.

## 구성
- `input.ddn`: 정본/legacy canvas key를 함께 넣고 목록 기반 rect 1개를 그리는 입력
- `tests/README.md`: 수동 실행 가이드

## DoD(최소)
- 출력 `bogae_hash`가 정본 canvas 크기 기준으로 결정적으로 재현된다.

## Contract Pointer
- 상위 alias 계약면: `tests/bogae_shape_alias_contract/README.md`
- 검증: `python tests/run_bogae_shape_alias_contract_selftest.py`
