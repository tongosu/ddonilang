# dev_structure (sym3)

> compact localized set :: commands + file names stay canonical

localized public summary; canonical detail = '../../DDONIRANG_DEV_STRUCTURE.md'.

## core_layers

| layer | path | role |
| --- | --- | --- |
| core | 'core/' | det_engine_core |
| lang | 'lang/' | grammar/parser/canon |
| tool | 'tool/' | runtime_tool_impl |
| CLI | 'tools/teul-cli/' | CLI_run_checks |
| packs | 'pack/' | runnable_pack_evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web_workspace+Bogae_views |
| tests | 'tests/' | integration_product_tests |
| publish | 'publish/' | public_docs |

## Seamgrim_workspace_V2

- 'ui/index.html': single_entry
- 'ui/screens/run.js': run_screen + current_line
- 'ui/components/bogae.js': console/graph/space2d/grid render
- 'ui/seamgrim_runtime_state.js': madi + runtime_state + mirror_summary
- 'tools/ddn_exec_server.py': local_static_server + helper_API

## runtime_principle

- DDN_runtime+packs+state_hash+mirror/replay own truth.
- Bogae == view_layer; not runtime_truth.
- Python/JS == orchestration/UI only; no test_only_lowering semantics.

## current_evidence

- CLI_WASM_runtime_parity
- vol4_raw_currentline_parity
- Seamgrim_product_smoke
- Bogae_madi_graph_UI_checks
