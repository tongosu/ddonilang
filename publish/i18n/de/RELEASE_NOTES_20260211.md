# Release notes 2026-02-11 (Deutsch)

> Historische Release-Notiz. Aktuelle öffentliche Einstiegspunkte sind '../../README.md', '../../QUICKSTART.md' und '../../DDONIRANG_DEV_STRUCTURE.md'.

## Zusammenfassung

Diese Release fokussierte AGE2 Open policy hardening sowie minimum schemas/runtime APIs für open.net/open.ffi/open.gpu.

## Highlights

- 'open=record|replay' wird blockiert, wenn 'age_target < AGE2' ist.
- '--unsafe-open' wurde als expliziter Bypass hinzugefügt.
- open-log-Schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Verhaltensänderung

open=record|replay ist nur mit age_target >= AGE2 erlaubt; --unsafe-open ist ein expliziter Bypass.

## Historischer Testbefehl

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Aktueller Statushinweis

Für den aktuellen Seamgrim/WASM/current-line Status QUICKSTART.md und DEV_STRUCTURE.md in diesem Ordner nutzen.
