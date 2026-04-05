# pack/block_codec_rpg_alrimsi_v1

RPG 박스 block editor 최소 smoke.

- `RPGBOX_PALETTE` 카테고리/블록 정의
- `ddn_block_codec.js`의 block → DDN 인코딩
- `wasm_canon_alrim_plan` 기반 receive handler decode
- `RpgBoxScreen`의 block editor 동기화

검증 명령:

```bash
node tests/rpgbox_block_runner.mjs pack/block_codec_rpg_alrimsi_v1
python tests/run_rpgbox_block_smoke_check.py
```
