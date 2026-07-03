# question_card_author_tool_share_v1

This pack closes ROADMAP_V2 `거-4` as a behavior-closed author tool share UI.

The scope is local and explicit: it records a question card template registry, author tool manifest, lesson template, review template, and handoff bundle. It does not publish to a registry, change accounts or permissions, sync to cloud storage, call an AI service, write files, or change parser/runtime semantics.

## Checks

- `python tests/run_pack_golden.py question_card_author_tool_share_v1`
- `node tests/question_card_author_tool_share_runner.mjs`
- `python tests/run_roadmap_v2_geo4_author_tool_share_check.py`
