# Development structure (Aymara)

> Aka qillqata qallta localización ukhamawa. Comandos ukat file sutinakax canonical qhiparaki.

Aka qillqatax público localización resumen ukhamawa. Canonical detalle file '../../DDONIRANG_DEV_STRUCTURE.md' ukawa.

## Core layers

| Layer | Path | Lurawi |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | CLI apayaña ukat uñakipaña |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace ukat Bogae views |
| tests | 'tests/' | integration/product tests |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': maya mantaña chiqapa
- 'ui/screens/run.js': run screen ukat current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror resumen
- 'tools/ddn_exec_server.py': local static server ukat helper API

## Runtime principio

- DDN runtime, packs, state hashes, mirror/replay records ukaw truth katuyi.
- Bogae ukax view layer; runtime truth janiw katuykiti.
- Python/JS orchestration ukat UI lurapxaspawa, ukampis language semantics test-only lowering ukamp jan lantintañawa.

## Jichha evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
