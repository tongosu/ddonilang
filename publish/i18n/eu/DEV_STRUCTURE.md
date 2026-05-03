# Garapen egitura (Euskara)

> Hasierako lokalizazioa da. Komandoak eta fitxategi-izenak canonical geratzen dira.

Hau lokalizatutako laburpen publikoa da. Xehetasun canonicala '../../DDONIRANG_DEV_STRUCTURE.md' da.

## Geruza nagusiak

| Geruza | Bidea | Rola |
| --- | --- | --- |
| core | 'core/' | motor deterministaren muina |
| lang | 'lang/' | gramatika, parserra, canonicalization |
| tool | 'tool/' | runtime/tresna inplementazioa |
| CLI | 'tools/teul-cli/' | CLI exekuzioa eta egiaztapenak |
| packs | 'pack/' | exekutagarri pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace eta Bogae ikuspegiak |
| tests | 'tests/' | integrazio eta produktu probak |
| publish | 'publish/' | dokumentu publikoak |

## Seamgrim workspace V2

- 'ui/index.html': sarrera puntu bakarra
- 'ui/screens/run.js': exekuzio pantaila eta current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror laburpena
- 'tools/ddn_exec_server.py': zerbitzari estatiko lokala eta helper API

## Runtime printzipioa

- DDN runtime, packs, state hashes eta mirror/replay erregistroek truth dute.
- Bogae view layer da eta ez du runtime truth jabetzen.
- Python/JS orkestrazio eta UI-rako erabil daitezke, baina ez dute hizkuntza esanahia test-only lowering bidez ordezkatu behar.

## Uneko evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
