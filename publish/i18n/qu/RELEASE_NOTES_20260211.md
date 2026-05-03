# Release notes 2026-02-11 (Runasimi)

> Ñawpa release note. Kunan público yaykuna '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## Resumen

Kay release AGE2 Open policy hardening, open.net/open.ffi/open.gpu minimum schema/runtime API nisqaman qhawarqan.

## Hatun willakuykuna

- age_target < AGE2 kaptinqa 'open=record|replay' hark'asqa.
- '--unsafe-open' qhapaq bypass hina yapasqa.
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Behavior tikray

open=record|replay age_target >= AGE2 kaptinlla; --unsafe-open qhapaq bypass.

## Ñawpa test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Kunan estado ñiqichay

Kunan Seamgrim/WASM/current-line estado qhawanapaq kay folder QUICKSTART.md, DEV_STRUCTURE.md apaykachay.
