# development structure (தமிழ்)

> இது starter localization. Commands மற்றும் file names canonical ஆகவே இருக்கும்.

இது public localized summary. canonical விவர file '../../DDONIRANG_DEV_STRUCTURE.md'.

## core layers

| layer | path | பங்கு |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | CLI execution மற்றும் checks |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace மற்றும் Bogae views |
| tests | 'tests/' | integration/product tests |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': ஒற்றை entry point
- 'ui/screens/run.js': run screen மற்றும் current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': local static server மற்றும் helper API

## runtime principle

- DDN runtime, packs, state hashes, mirror/replay records truth வைத்திருக்கும்.
- Bogae view layer; runtime truth வைத்திருக்காது.
- Python/JS orchestration மற்றும் UI செய்யலாம்; ஆனால் language semantics ஐ test-only lowering ஆக மாற்றக்கூடாது.

## current evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
