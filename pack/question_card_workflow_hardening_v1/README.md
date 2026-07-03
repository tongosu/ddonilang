# question_card_workflow_hardening_v1

This pack closes ROADMAP_V2 `거-5` as a behavior-closed AI workflow hardening UI.

The scope is local and explicit: it records a manual approval gate, workflow replay packet, append-only audit preview, local rollback plan, and local LTS gate. It does not execute AI calls, auto-apply patches, write files, publish to registries, sync cloud state, execute releases, certify LTS status, or change parser/runtime semantics.

## Checks

- `python tests/run_pack_golden.py question_card_workflow_hardening_v1`
- `node tests/question_card_workflow_hardening_runner.mjs`
- `python tests/run_roadmap_v2_geo5_ai_workflow_hardening_check.py`
