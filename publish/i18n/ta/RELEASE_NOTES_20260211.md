# release notes 2026-02-11 (தமிழ்)

> historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## சுருக்கம்

இந்த release AGE2 Open policy hardening மற்றும் open.net/open.ffi/open.gpu minimum schema/runtime API மீது கவனம் வைத்தது.

## முக்கிய மாற்றங்கள்

- age_target < AGE2 என்றால் 'open=record|replay' தடுக்கப்படும்.
- '--unsafe-open' explicit bypass ஆக சேர்க்கப்பட்டது.
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## behavior change

open=record|replay age_target >= AGE2 என்றால் மட்டும்; --unsafe-open explicit bypass.

## historical test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## current status pointer

current Seamgrim/WASM/current-line status க்கு இந்த folder இல் QUICKSTART.md மற்றும் DEV_STRUCTURE.md பயன்படுத்தவும்.
