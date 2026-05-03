# Notas de release 2026-02-11 (Español)

> Nota histórica de release. Las entradas públicas actuales son '../../README.md', '../../QUICKSTART.md' y '../../DDONIRANG_DEV_STRUCTURE.md'.

## Resumen

Esta release se centró en endurecer la política AGE2 Open y en los minimum schemas/runtime APIs de open.net/open.ffi/open.gpu.

## Cambios principales

- 'open=record|replay' se bloquea cuando 'age_target < AGE2'.
- Se añadió '--unsafe-open' como bypass explícito.
- schemas de open log:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Cambio de comportamiento

open=record|replay solo se permite con age_target >= AGE2; --unsafe-open es un bypass explícito.

## Comando histórico de test

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Puntero de estado actual

Para el estado actual de Seamgrim/WASM/current-line, usa QUICKSTART.md y DEV_STRUCTURE.md en esta carpeta.
