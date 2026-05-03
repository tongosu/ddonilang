# development structure (नेपाली)

> यो starter localization हो। Commands र file names canonical नै रहन्छन्।

यो public localized summary हो। canonical विस्तृत file '../../DDONIRANG_DEV_STRUCTURE.md' हो।

## core layers

| layer | path | भूमिका |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | CLI execution र checks |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace र Bogae views |
| tests | 'tests/' | integration/product tests |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': एकल entry point
- 'ui/screens/run.js': run screen र current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': local static server र helper API

## runtime principle

- DDN runtime, packs, state hashes, mirror/replay records ले truth राख्छन्।
- Bogae view layer हो; runtime truth राख्दैन।
- Python/JS orchestration र UI का लागि हुन सक्छ, तर language semantics लाई test-only lowering ले बदल्नु हुँदैन।

## current evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
