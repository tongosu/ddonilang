# release_notes_20260211 (sym3)

> historical note; current entries: '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## summary

AGE2 Open hardening + open.net/open.ffi/open.gpu minimum schema/runtime API.

## highlights

- age_target<AGE2 blocks 'open=record|replay'.
- '--unsafe-open' added as explicit bypass.
- open_log_schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## behavior_change

open=record|replay only when age_target>=AGE2; --unsafe-open = explicit bypass.

## historical_test_command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## current_status_pointer

current Seamgrim/WASM/current-line => QUICKSTART.md + DEV_STRUCTURE.md.
