# D-PACK: block_editor_raw_fallback_v1

## 목적
- SSOT `PLN-20260322-BLOCK-EDITOR-ROUNDTRIP-01`의 raw fallback 보존 규칙을 고정한다.
- `ddn.block_editor_plan.v1`가 아직 구조화하지 않은 문형을 `raw_block`으로 유지하면서도 canon bytes equality를 깨지 않는지 검증한다.

## 구성
- `fixtures/source.ddn`: 현재 구조화되지 않은 `매김` 확장 필드(`설명`)를 포함한 입력.
- `expected/block_editor_roundtrip.detjson`: `raw_block_count >= 1`과 canon equality를 함께 고정한 결과.

## 검증
- `node --no-warnings tests/block_editor_roundtrip_runner.mjs pack/block_editor_raw_fallback_v1`
- `python tests/run_block_editor_roundtrip_check.py`
