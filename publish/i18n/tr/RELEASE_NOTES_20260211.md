# Release notları 2026-02-11 (Türkçe)

> Tarihsel release notudur. Güncel genel giriş noktaları '../../README.md', '../../QUICKSTART.md' ve '../../DDONIRANG_DEV_STRUCTURE.md' dosyalarıdır.

## Özet

Bu release AGE2 Open politika güçlendirmesi ve open.net/open.ffi/open.gpu minimum schema/runtime API'lerine odaklandı.

## Öne çıkanlar

- age_target < AGE2 iken 'open=record|replay' engellenir.
- '--unsafe-open' açık bypass olarak eklendi.
- open log schema'ları:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- pack'ler:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## Davranış değişikliği

open=record|replay yalnızca age_target >= AGE2 için izinlidir; --unsafe-open açık bypass sağlar.

## Tarihsel test komutu

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## Güncel durum yönlendirmesi

Güncel Seamgrim/WASM/current-line durumu için bu klasördeki QUICKSTART.md ve DEV_STRUCTURE.md dosyalarını kullanın.
