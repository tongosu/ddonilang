# Development structure (Telugu)

> Starter localized guide; commands and file names stay canonical.

This is a localized public summary. The canonical detailed file is '../../DDONIRANG_DEV_STRUCTURE.md'.

## Core layers

| Layer | Path | Role |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | command-line execution and checks |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace and Bogae views |
| tests | 'tests/' | integration and product checks |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': single entry point
- 'ui/screens/run.js': run screen and current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': local static server and helper API

## Runtime principle

- DDN runtime, packs, state hashes, and mirror/replay records own truth.
- Bogae is a view layer and must not own runtime truth.
- Python/JS may orchestrate checks and UI, but they must not replace language/runtime semantics with test-only lowering.

## Current evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product stabilization smoke
- Bogae madi/graph UI checks