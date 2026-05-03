# release notes 2026-02-11 (ಕನ್ನಡ)

> historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## ಸಾರಾಂಶ

ಈ release AGE2 Open policy hardening ಮತ್ತು open.net/open.ffi/open.gpu minimum schema/runtime API ಮೇಲೆ ಕೇಂದ್ರೀಕರಿಸಿತು.

## ಮುಖ್ಯಾಂಶಗಳು

- age_target < AGE2 ಆಗಿದ್ದರೆ 'open=record|replay' ತಡೆಹಿಡಿಯಲಾಗುತ್ತದೆ.
- '--unsafe-open' explicit bypass ಆಗಿ ಸೇರಿಸಲಾಗಿದೆ.
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## behavior change

open=record|replay age_target >= AGE2 ಆಗಿರುವಾಗ ಮಾತ್ರ; --unsafe-open explicit bypass.

## historical test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## current status pointer

current Seamgrim/WASM/current-line status ಗಾಗಿ ಈ folder ನ QUICKSTART.md ಮತ್ತು DEV_STRUCTURE.md ಬಳಸಿ.
