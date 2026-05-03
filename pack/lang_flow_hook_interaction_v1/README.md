# lang_flow_hook_interaction_v1

흐름씨(`<<-`)와 tail-phase hook ordering에 대한 supporting contract pack이다.

- evidence_tier: `docs_first`
- closure_claim: `no`
- current status:
  - `c01`, `c02`는 existing snapshot input으로 남긴다.
  - `c03`, `c04`는 diag contract를 고정한다.
  - 실제 runtime fixed-point / conflict / cycle implementation은 아직 별도 closure 대상이다.

검증:

```bash
python tests/run_lang_flow_hook_interaction_pack_check.py
```
