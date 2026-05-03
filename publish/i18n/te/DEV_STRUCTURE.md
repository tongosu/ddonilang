# development structure (తెలుగు)

> ఇది starter localization. Commands మరియు file names canonical గానే ఉంటాయి.

ఇది public localized summary. canonical వివర file '../../DDONIRANG_DEV_STRUCTURE.md'.

## core layers

| layer | path | పాత్ర |
| --- | --- | --- |
| core | 'core/' | deterministic engine core |
| lang | 'lang/' | grammar, parser, canonicalization |
| tool | 'tool/' | runtime/tool implementation |
| CLI | 'tools/teul-cli/' | CLI execution మరియు checks |
| packs | 'pack/' | runnable pack evidence |
| Seamgrim | 'solutions/seamgrim_ui_mvp/' | web workspace మరియు Bogae views |
| tests | 'tests/' | integration/product tests |
| publish | 'publish/' | public documents |

## Seamgrim workspace V2

- 'ui/index.html': ఒక entry point
- 'ui/screens/run.js': run screen మరియు current-line execution
- 'ui/components/bogae.js': console/graph/space2d/grid Bogae rendering
- 'ui/seamgrim_runtime_state.js': madi, runtime state, mirror summary
- 'tools/ddn_exec_server.py': local static server మరియు helper API

## runtime principle

- DDN runtime, packs, state hashes, mirror/replay records truth కలిగి ఉంటాయి.
- Bogae view layer; runtime truth కలిగి ఉండదు.
- Python/JS orchestration మరియు UI చేయగలవు; language semantics ను test-only lowering తో మార్చకూడదు.

## current evidence

- CLI/WASM runtime parity
- Vol4 raw current-line bundle parity
- Seamgrim product smoke
- Bogae madi/graph UI checks
