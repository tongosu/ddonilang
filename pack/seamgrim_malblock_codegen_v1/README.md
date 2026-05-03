# pack/seamgrim_malblock_codegen_v1

ROADMAP_V2 `라-1` 말블록 기본 팔레트 + block->DDN codegen evidence pack.

이 pack은 broad block editor screen smoke가 아니라 targeted codegen smoke다.
말블록 팔레트에서 만든 block tree가 DDN으로 encode되고, 생성된 DDN이 `teul-cli canon`과 최소 `run` smoke를 통과하는지 확인한다.

검증:

```powershell
python tests/run_seamgrim_malblock_codegen_check.py
```

보조 검증:

```powershell
python tests/run_block_editor_roundtrip_check.py
python tests/run_block_editor_choose_exhaustive_check.py
```
