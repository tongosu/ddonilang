# GitHub Release Draft — 2026-02-11

## Highlights
- AGE2(Open) 정책 강화: `age_target<AGE2`에서 open 모드 차단, `--unsafe-open` 우회 추가.
- open.net/ffi/gpu 런타임 API 및 open.log v1 스키마 확정.
- open.net/ffi/gpu record/replay pack 추가.

## Breaking/Behavior Changes
- `open=record|replay`는 `age_target>=AGE2`에서만 허용됨.

## CLI
- `--unsafe-open`: age_target 제한 우회

## Packs
- `pack/open_net_record_replay`
- `pack/open_ffi_record_replay`
- `pack/open_gpu_record_replay`

## Docs
- `publish/QUICKSTART.md` 사용법 보강
- `publish/DOWNLOADS.md` 배포/패키징 규칙 보강

## Assets
- `ddonirang-tool-0.1.0-windows-x64.zip`
- `SHA256SUMS.txt`

SHA256
```
6174AA313EEAB0E1AAB7FAAF7B15DB867A3E33E7C0A0694AD880F8FC9E15C8A4  ddonirang-tool-0.1.0-windows-x64.zip
```

## Tests
- `python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay`
