# intent — gogae9_w92_aot_compiler_v2

목표:
- W92 AOT v2 착수 시점의 최소 계약을 팩으로 고정한다.
- 실행 동치(parity)와 성능 하한(speedup>=20.0)을 동일 문서 구조로 기록한다.

범위:
- 실제 LLVM AOT 빌드/실행 구현은 아직 포함하지 않는다.
- 본 팩은 `bench_cases.json`/`golden.detjson` 형식 계약과 상호 정합만 검증한다.

완료 조건:
- `tests/run_w92_aot_pack_check.py`가 PASS여야 한다.
- `run_pack_golden.py` 기본 루프에 편입 가능한 `golden.jsonl`을 제공한다.
