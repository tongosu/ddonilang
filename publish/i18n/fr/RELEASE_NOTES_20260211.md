# Notes de release 2026-02-11 (Français)

> Note de release historique. Les points d'entrée publics actuels sont '../../README.md', '../../QUICKSTART.md' et '../../DDONIRANG_DEV_STRUCTURE.md'.

## Résumé

Cette release a renforcé la politique AGE2 Open et les minimum schemas/runtime APIs de open.net/open.ffi/open.gpu.

## Points forts

- 'open=record|replay' est bloqué quand 'age_target < AGE2'.
- '--unsafe-open' a été ajouté comme contournement explicite.
- schemas open log :
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs :
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Changement de comportement

open=record|replay est autorisé seulement avec age_target >= AGE2 ; --unsafe-open est un contournement explicite.

## Commande de test historique

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Pointeur d'état actuel

Pour l'état actuel Seamgrim/WASM/current-line, utiliser QUICKSTART.md et DEV_STRUCTURE.md dans ce dossier.
