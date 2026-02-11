# Release Notes — 2026-02-11

## 요약
이번 릴리스는 AGE2(Open) 기반의 open 정책 강화와 open.net/ffi/gpu 최소 스키마/런타임 API를 확정하고, 이에 대한 pack 검증을 추가했다.

## 주요 변경
- age_target 기반 open 차단: `age_target<AGE2`에서는 `open=record|replay`가 금지된다.
- 우회 옵션: `teul-cli run --unsafe-open` 추가.
- 새 open API:
  - `열림.네트워크.요청.`
  - `열림.호스트FFI.호출.`
  - `열림.GPU.실행.`
- open.log v1 스키마 추가:
  - `open.net.v1`
  - `open.ffi.v1`
  - `open.gpu.v1`
- pack 추가:
  - `pack/open_net_record_replay`
  - `pack/open_ffi_record_replay`
  - `pack/open_gpu_record_replay`

## 동작 변경(주의)
- `open=record|replay`는 `age_target>=AGE2`에서만 허용된다.
- 기존 pack/open_* 테스트는 `--unsafe-open`을 통해 우회 실행된다.

## 사용 예시
```bash
teul-cli run path/to/input.ddn --open record --open-log open.log.jsonl --unsafe-open
teul-cli run path/to/input.ddn --open replay --open-log open.log.jsonl --unsafe-open
```

## open.log 스키마 요약
```json
{ "schema": "open.net.v1", "url": "...", "method": "GET", "body": "...", "text": "..." }
{ "schema": "open.ffi.v1", "name": "func", "args": ["..."], "result": "..." }
{ "schema": "open.gpu.v1", "kernel": "k", "payload": "...", "result": "..." }
```

## 제한 사항
- open.net/ffi/gpu의 실제 외부 호출은 런타임이 수행하지 않는다.
- record 모드에서는 응답/결과를 호출 인자로 제공해야 한다.

## 테스트
- PASS: `python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay`

## GitHub 업로드 범위
- 코드 + `publish/` 문서만 업로드한다.
