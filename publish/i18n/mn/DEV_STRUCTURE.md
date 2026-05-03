# Хөгжүүлэлтийн бүтэц (Монгол)

> Starter орчуулга. Команд ба файлын нэрс canonical хэвээр байна.

Энэ бол олон хэлний нийтийн товч хураангуй. Дэлгэрэнгүй canonical файл нь '../../DDONIRANG_DEV_STRUCTURE.md'.

## Үндсэн давхаргууд

| Давхарга | Зам | Үүрэг |
| --- | --- | --- |
| core | 'core/' | детерминист engine core |
| lang | 'lang/' | дүрэм, parser, canonicalization |
| tool | 'tool/' | runtime/tool хэрэгжилт |
| CLI | 'tools/teul-cli/' | CLI ажиллуулах ба шалгах |
| packs | 'pack/' | ажиллах pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace ба Bogae view |
| tests | 'tests/' | integration ба product tests |
| publish | 'publish/' | нийтийн баримтууд |

## Seamgrim workspace V2

- 'ui/index.html': нэг орох цэг
- 'ui/screens/run.js': ажиллуулах дэлгэц ба current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': локал static server ба helper API

## Runtime зарчим

- DDN runtime, packs, state hashes, mirror/replay records truth-ийг эзэмшинэ.
- Bogae нь view layer бөгөөд runtime truth эзэмшихгүй.
- Python/JS orchestration ба UI хийж болно, харин хэлний утгыг test-only lowering-оор орлож болохгүй.

## Одоогийн evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
