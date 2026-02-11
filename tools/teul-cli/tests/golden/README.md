# AGE01 Golden Tests Pack (WALK01~WALK11) — v1

이 묶음은 `AGE01_WALK01-11_IMPLEMENTATION_PLAN_SSOTv20.md`의 골든 테스트를 실제 파일 구조로 구현한 것입니다.

- 문서/예시의 **정본 출력 우선** 규칙에 맞춰, `canon` 테스트는 `teul-cli canon --emit ddn` 출력만 비교합니다.
- `run` 테스트는 `보여주기` 출력(줄 배열)과 `state_hash/trace_hash`를 비교합니다.
  - 필요 시 `bogae_hash`도 비교할 수 있습니다.

## 1) 실행

```bash
python3 run_golden.py --teul-cli teul-cli --walk 1
python3 run_golden.py --teul-cli teul-cli --walk 1-2
python3 run_golden.py --teul-cli teul-cli --walk 8 --madi 5
```

## 2) 블레스(해시 채우기)

일부 테스트는 `state_hash` 또는 `trace_hash`가 `"ANY"` 입니다.

```bash
python3 run_golden.py --teul-cli teul-cli --walk 1 --bless
```

- `"ANY"` → 실제 값(`blake3:...`)으로 채워집니다.
- `"SAME_AS:..."`, `"DIFFERS_FROM:..."`는 그대로 둡니다.

## 3) 테스트 스펙(.dtest.json)

각 테스트 폴더:
- `main.ddn`
- `test.dtest.json`

### 주요 필드
- `name`: 테스트 ID
- `walk`: 걸음 번호
- `command`: `run` | `canon` | `preproc` | `pipeline`
- `args`:
  - `madi`: tick 수(없으면 1)
  - `seed`: `"blake3:..."` 문자열(없으면 `"0x0"`)
  - `cli`: CLI 추가 인자 리스트(예: `["--trace-json","trace.json"]`)
- `expect`:
  - `exit`: 종료 코드(기본 0)
  - `stdout`: 줄 배열(보여주기 출력만 비교; 해시 라인은 자동 제거)
  - `stderr_contains`: stderr에 포함되어야 하는 문자열 배열
  - `state_hash` / `trace_hash` / `bogae_hash`:
    - `"ANY"` / `"ABSENT"` / `"blake3:..."` / `"SAME_AS:<name>"` / `"DIFFERS_FROM:<name>"`
  - `files_exist`: 생성되어야 하는 파일 목록

--- 
