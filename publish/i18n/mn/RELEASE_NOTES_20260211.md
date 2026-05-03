# Release notes 2026-02-11 (Монгол)

> Түүхэн release note. Одоогийн нийтийн эхлэл нь '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## Хураангуй

Энэ release AGE2 Open policy hardening болон open.net/open.ffi/open.gpu minimum schema/runtime API дээр төвлөрсөн.

## Гол өөрчлөлтүүд

- age_target < AGE2 үед 'open=record|replay' хоригдоно.
- '--unsafe-open' ил bypass байдлаар нэмэгдсэн.
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Behavior change

open=record|replay нь age_target >= AGE2 үед зөвшөөрөгдөнө; --unsafe-open нь ил bypass.

## Түүхэн test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Одоогийн төлөв рүү заавар

Одоогийн Seamgrim/WASM/current-line төлөвийг энэ folder-ийн QUICKSTART.md ба DEV_STRUCTURE.md-ээс харна уу.
