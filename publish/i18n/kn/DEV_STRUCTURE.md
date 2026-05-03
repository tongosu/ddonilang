# development structure (ಕನ್ನಡ)

> ಇದು starter localization. Commands ಮತ್ತು file names canonical ಆಗಿಯೇ ಉಳಿಯುತ್ತವೆ.

ಇದು public localized summary. canonical ವಿವರವಾದ file '../../DDONIRANG_DEV_STRUCTURE.md'.

## core layers

| layer | path | ಪಾತ್ರ |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | CLI execution ಮತ್ತು checks |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace ಮತ್ತು Bogae views |
| tests | 'tests/' | integration/product tests |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': ಒಂದು entry point
- 'ui/screens/run.js': run screen ಮತ್ತು current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': local static server ಮತ್ತು helper API

## runtime principle

- DDN runtime, packs, state hashes, mirror/replay records truth ಹೊಂದಿವೆ.
- Bogae view layer; runtime truth ಹೊಂದುವುದಿಲ್ಲ.
- Python/JS orchestration ಮತ್ತು UI ಮಾಡಬಹುದು; language semantics ಅನ್ನು test-only lowering ಮೂಲಕ ಬದಲಿಸಬಾರದು.

## current evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
