# roadmap_v2_cha0_rpg_seed_reassessment_v1

ROADMAP_V2 `차-0` RPG action/story seed 재평가 pack이다.

이 pack은 새 RPG 제품 동작이나 parser/runtime semantics를 추가하지 않는다. 기존 seed pack과 `차-1`~`차-5` 제품/런타임 검증을 묶어 `차-0` action/story 설계가 현재 행렬에서 `닫힘-동작`으로 재분류될 수 있음을 고정한다.

## Verification

```powershell
python tests/run_pack_golden.py roadmap_v2_cha0_rpg_seed_reassessment_v1
python tests/run_roadmap_v2_cha0_rpg_seed_reassessment_check.py
```
