# gogae9_w92_aot_compiler_v2

- 상태: W92 착수용 최소 계약 pack
- 기준: `docs/ssot/walks/gogae9/w92_aot_compiler_v2/README.md`
- Pack ID: `pack/gogae9_w92_aot_compiler_v2`

## 목적
- AOT v2 본 구현 이전에 `bench_cases`/`golden` 문서 구조와 상호 정합을 고정한다.
- parity(`interp_state_hash == aot_state_hash`) + speedup 하한(`>=20.0`) 표면을 미리 봉인한다.

## 구성
- `intent.md`
- `bench_cases.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c01_interp_aot_hash_parity/input.ddn`

## 검증
- `python tests/run_w92_aot_pack_check.py --pack pack/gogae9_w92_aot_compiler_v2`
- `python tests/run_pack_golden.py gogae9_w92_aot_compiler_v2`

## 상위 Family
- `tests/gate0_family/README.md`
- `python tests/run_gate0_family_selftest.py`

## 비고
- 이 pack은 AOT 코드 생성 자체를 검증하지 않는다.
- AOT 본체 구현은 W92 PR-92A/92B 단계에서 `teul-cli build/run/bench` 실경로로 확장한다.
