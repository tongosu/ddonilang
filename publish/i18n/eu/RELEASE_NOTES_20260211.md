# Release notes 2026-02-11 (Basque)

> Historical release note. Current public entry points are '../../README.md', '../../QUICKSTART.md', and '../../DDONIRANG_DEV_STRUCTURE.md'.

## Summary

This release focused on AGE2 Open policy hardening and minimum schemas/runtime APIs for open.net, open.ffi, and open.gpu.

## Highlights

- 'open=record|replay' is blocked when 'age_target < AGE2'.
- '--unsafe-open' was added as an explicit bypass.
- open log schemas were added:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- Packs were added:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Behavior change

'open=record|replay' is allowed only for 'age_target >= AGE2' unless '--unsafe-open' is used.

## Historical test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Current status pointer

For current Seamgrim/WASM/current-line status, use 'QUICKSTART.md' and 'DEV_STRUCTURE.md' in this language folder.