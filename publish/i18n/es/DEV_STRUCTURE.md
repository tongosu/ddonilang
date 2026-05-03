# Estructura de desarrollo (Español)

> Traducción starter. Los comandos y nombres de archivo se mantienen canonical.

Este es un resumen público localizado. El archivo canonical detallado es '../../DDONIRANG_DEV_STRUCTURE.md'.

## Capas principales

| Capa | Ruta | Rol |
| --- | --- | --- |
| core | 'core/' | núcleo de motor determinista |
| lang | 'lang/' | gramática, parser, canonicalization |
| tool | 'tool/' | implementación runtime/tool |
| CLI | 'tools/teul-cli/' | ejecución y checks de CLI |
| packs | 'pack/' | pack evidence ejecutable |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | workspace web y vistas Bogae |
| tests | 'tests/' | tests de integración y producto |
| publish | 'publish/' | documentos públicos |

## Seamgrim workspace V2

- 'ui/index.html': punto de entrada único
- 'ui/screens/run.js': pantalla de ejecución y ejecución current-line
- 'ui/components/bogae.js': render de Bogae console/graph/space2d/grid
- 'ui/seamgrim_runtime_state.js': madi, estado runtime y resumen mirror
- 'tools/ddn_exec_server.py': servidor estático local y API auxiliar

## Principio de runtime

- DDN runtime, packs, state hashes y registros mirror/replay poseen la truth.
- Bogae es una view layer y no posee runtime truth.
- Python/JS puede orquestar checks y UI, pero no debe reemplazar la semántica del lenguaje con test-only lowering.

## Evidence actual

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
