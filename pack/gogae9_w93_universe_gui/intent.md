# intent — gogae9_w93_universe_gui

목표:
- W93(우주 만들기 GUI) 착수 전, pack/unpack 결정성 계약을 팩 파일로 고정한다.
- 동일 source의 2회 pack 해시 일치와 roundtrip/state_hash 재현 요구를 정적 계약으로 점검한다.

범위:
- 실제 `teul-cli universe pack/unpack` 실행은 포함하지 않는다.
- 본 팩은 `universe_cases.json`/`golden.detjson`/`golden.jsonl` 구조 정합을 우선 검증한다.

완료 조건:
- `tests/run_w93_universe_pack_check.py` PASS
- `tests/run_pack_golden.py gogae9_w93_universe_gui` PASS
