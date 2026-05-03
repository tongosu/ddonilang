# Entwicklungsstruktur (Deutsch)

> Starter-Lokalisierung. Befehle und Dateinamen bleiben canonical.

Dies ist eine lokalisierte öffentliche Zusammenfassung. Die detaillierte canonical Datei ist '../../DDONIRANG_DEV_STRUCTURE.md'.

## Kernschichten

| Schicht | Pfad | Rolle |
| --- | --- | --- |
| core | 'core/' | deterministischer Engine-Kern |
| lang | 'lang/' | Grammatik, Parser, canonicalization |
| tool | 'tool/' | runtime/tool Implementierung |
| CLI | 'tools/teul-cli/' | CLI-Ausführung und Checks |
| packs | 'pack/' | ausführbare pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | Web-workspace und Bogae views |
| tests | 'tests/' | Integrations- und Produkttests |
| publish | 'publish/' | öffentliche Dokumente |

## Seamgrim workspace V2

- 'ui/index.html': einziger Einstiegspunkt
- 'ui/screens/run.js': Ausführungsansicht und current-line Ausführung
- 'ui/components/bogae.js': Bogae-Rendering für console/graph/space2d/grid
- 'ui/seamgrim_runtime_state.js': madi, runtime state und mirror summary
- 'tools/ddn_exec_server.py': lokaler statischer Server und Hilfs-API

## Runtime-Prinzip

- DDN runtime, packs, state hashes und mirror/replay records besitzen die truth.
- Bogae ist eine view layer und besitzt keine runtime truth.
- Python/JS dürfen Checks und UI orchestrieren, aber Sprachsemantik nicht durch test-only lowering ersetzen.

## Aktuelle evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
