# release notes 2026-02-11 (తెలుగు)

> historical release note. current public entry points '../../README.md', '../../QUICKSTART.md', '../../DDONIRANG_DEV_STRUCTURE.md'.

## సారాంశం

ఈ release AGE2 Open policy hardening మరియు open.net/open.ffi/open.gpu minimum schema/runtime API పై కేంద్రీకృతమైంది.

## ముఖ్యాంశాలు

- age_target < AGE2 అయితే 'open=record|replay' నిరోధించబడుతుంది.
- '--unsafe-open' explicit bypass గా జోడించబడింది.
- open log schemas:
  - 'open.net.v1'
  - 'open.ffi.v1'
  - 'open.gpu.v1'
- packs:
  - 'pack/open_net_record_replay'
  - 'pack/open_ffi_record_replay'
  - 'pack/open_gpu_record_replay'

## behavior change

open=record|replay age_target >= AGE2 లో మాత్రమే; --unsafe-open explicit bypass.

## historical test command

~~~sh
python tests/run_pack_golden.py open_net_record_replay open_ffi_record_replay open_gpu_record_replay
~~~

## current status pointer

current Seamgrim/WASM/current-line status కోసం ఈ folder లో QUICKSTART.md మరియు DEV_STRUCTURE.md వాడండి.
