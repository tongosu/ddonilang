# 릴리즈 노트 2026-02-11 (한국어)

> 역사적 릴리즈 노트입니다. 현재 공개 진입점은 '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'입니다.

## 요약

이번 릴리스는 AGE2 Open 정책 강화와 open.net/open.ffi/open.gpu 최소 스키마 및 runtime API에 초점을 둡니다.

## 주요 변경

- age_target < AGE2에서는 'open=record|replay'가 차단됩니다.
- '--unsafe-open' 우회 옵션이 추가되었습니다.
- open log 스키마:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- pack:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## 동작 변경

open=record|replay는 age_target >= AGE2에서만 허용됩니다. --unsafe-open을 쓰면 명시적으로 우회합니다.

## 당시 테스트 명령

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## 현재 상태 안내

현재 셈그림/WASM/current-line 상태는 이 폴더의 QUICKSTART.md와 DEV_STRUCTURE.md를 보세요.
