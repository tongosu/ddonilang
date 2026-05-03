# 开发结构 (中文)

> 这是 starter 本地化文档。命令和文件名保持 canonical 写法。

这是公开用本地化摘要。详细 canonical 文档是 '../../DDONIRANG_DEV_STRUCTURE.md'。

## 核心层

| 层 | 路径 | 角色 |
| --- | --- | --- |
| core | 'core/' | 确定性引擎核心 |
| lang | 'lang/' | 语法、解析器、canonicalization |
| tool | 'tool/' | runtime/tool 实现 |
| CLI | 'tools/teul-cli/' | CLI 执行与检查 |
| packs | 'pack/' | 可执行 pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web 工作室与 Bogae 视图 |
| tests | 'tests/' | 集成和产品测试 |
| publish | 'publish/' | 公开文档 |

## Seamgrim workspace V2

- 'ui/index.html': 单一入口
- 'ui/screens/run.js': 运行画面与 current-line 执行
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae 渲染
- 'ui/seamgrim_runtime_state.js': madi、runtime state、mirror 摘要
- 'tools/ddn_exec_server.py': 本地静态服务器与辅助 API

## Runtime 原则

- DDN runtime、packs、state hashes、mirror/replay records 拥有 truth。
- Bogae 是 view layer，不拥有 runtime truth。
- Python/JS 可以做 orchestration 和 UI，但不能用 test-only lowering 取代语言语义。

## 当前 evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
