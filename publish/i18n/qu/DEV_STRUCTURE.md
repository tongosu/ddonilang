# Development structure (Runasimi)

> Kay qillqa starter localizaciónmi. Comandos hinallataq file sutikuna canonical kachkan.

Kayqa público localización resumenmi. Canonical detalle file '../../DDONIRANG_DEV_STRUCTURE.md'.

## Core layers

| Layer | Path | Ruray |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | CLI purichiy chaymanta qhaway |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace chaymanta Bogae views |
| tests | 'tests/' | integration/product tests |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': huk yaykuna
- 'ui/screens/run.js': run screen chaymanta current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror resumen
- 'tools/ddn_exec_server.py': local static server chaymanta helper API

## Runtime principio

- DDN runtime, packs, state hashes, mirror/replay records truth hap'in.
- Bogae view layermi; runtime truth mana hap'inchu.
- Python/JS orchestration/UI rurayta atin, ichaqa language semantics test-only loweringwan mana tikranachu.

## Kunan evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
