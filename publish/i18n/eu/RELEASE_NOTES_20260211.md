# Release oharrak 2026-02-11 (Euskara)

> Ohar historikoa da. Uneko sarrera publikoak '../../README.md', '../../QUICKSTART.md' eta '../../DDONIRANG_DEV_STRUCTURE.md' dira.

## Laburpena

Release honek AGE2 Open policy hardening eta open.net/open.ffi/open.gpu minimum schema/runtime APIak landu zituen.

## Nabarmenak

- age_target < AGE2 denean 'open=record|replay' blokeatzen da.
- '--unsafe-open' bypass esplizitu gisa gehitu da.
- open log schemak:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packak:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Portaera aldaketa

open=record|replay age_target >= AGE2 denean bakarrik onartzen da; --unsafe-open bypass esplizitua da.

## Test komando historikoa

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Uneko egoeraren lotura

Uneko Seamgrim/WASM/current-line egoerarako erabili folder honetako QUICKSTART.md eta DEV_STRUCTURE.md.
