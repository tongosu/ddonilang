# Release notes 2026-02-11 (Aymara)

> Nayra release note. Jichha público mantañanakax '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## Resumen

Aka release AGE2 Open policy hardening ukat open.net/open.ffi/open.gpu minimum schema/runtime API ukanakaru uñtatawa.

## Jach'a mayjt'awinaka

- age_target < AGE2 ukax 'open=record|replay' jark'iwa.
- '--unsafe-open' qhana bypass ukham yapxatata.
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Behavior mayjt'awi

open=record|replay ukax age_target >= AGE2 ukaki; --unsafe-open ukax qhana bypass.

## Nayra test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Jichha estado uñacht'awi

Jichha Seamgrim/WASM/current-line estado uñjañatakix aka folder QUICKSTART.md ukat DEV_STRUCTURE.md apnaqam.
