# D-PACK: block_editor_roundtrip_v1

## 목적
- SSOT `PLN-20260322-BLOCK-EDITOR-ROUNDTRIP-01`의 기본 roundtrip equality를 고정한다.
- `DDN source -> canon DDN -> flat_json -> block_editor_plan -> block_tree -> canon DDN` 경로에서 canon bytes가 동일해야 한다.

## 구성
- `fixtures/source.ddn`: raw fallback 없이 block editor plan이 구조적으로 소비하는 기본 셈그림 예제.
- `expected/block_editor_roundtrip.detjson`: canon equality, flat_json 요약, block kind 통계를 고정한 결과.

## 검증
- `node --no-warnings tests/block_editor_roundtrip_runner.mjs pack/block_editor_roundtrip_v1`
- `python tests/run_block_editor_roundtrip_check.py`
