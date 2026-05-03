# release notes 2026-02-11 (नेपाली)

> historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## सारांश

यो release AGE2 Open policy hardening र open.net/open.ffi/open.gpu minimum schema/runtime API मा केन्द्रित थियो।

## मुख्य बुँदा

- age_target < AGE2 हुँदा 'open=record|replay' रोकिन्छ।
- '--unsafe-open' explicit bypass का रूपमा थपियो।
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## behavior change

open=record|replay age_target >= AGE2 मा मात्र; --unsafe-open explicit bypass हो।

## historical test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## current status pointer

current Seamgrim/WASM/current-line status का लागि यस folder को QUICKSTART.md र DEV_STRUCTURE.md प्रयोग गर्नुहोस्।
