# lang_flow_hook_interaction_v1

흐름씨(`<<-`)와 tail-phase hook ordering에 대한 supporting contract pack이다.

- evidence_tier: `runtime`
- closure_claim: `yes`
- current status:
  - `c01`, `c02`는 `teul-cli run --summary-json`으로 runtime state를 검증한다.
  - `c03`, `c04`는 제품 runtime error code를 검증한다.
  - 흐름씨 fixed-point / conflict / cycle / tail-phase hook ordering은 제품 경로에서 닫혔다.
  - `c02`의 `after_tick_2`는 tick index 2 도달 상태이므로 checker는 `--madi 3`으로 검증한다.

검증:

```bash
python tests/run_lang_flow_hook_interaction_pack_check.py
```
