# lang_chaebi_scope_v1

`채비 {}` current-line scope rule을 runner/checker로 고정하는 support pack이다.

- evidence_tier: `runner_fill`
- closure_claim: `no`
- checks:
  - top-level `채비 {}` 허용
  - loop/hook/순회 body 안 `채비 {}` = `E_CHAEBI_IN_LOOP`
  - top-level redundant reassignment = `W_CHAEBI_REDUNDANT_TOP_REASSIGN`

실행:

```bash
python tests/run_lang_chaebi_scope_pack_check.py
python tests/run_pack_golden.py lang_chaebi_scope_v1
```
