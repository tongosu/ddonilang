# nurigym_shared_sync_action_pipeline_v1

evidence_tier: runtime_pack
closure_claim: no
machine_checkable: yes

목적: shared sync action pipeline의 add-only provenance 필드와 status 4종(`맞음/가만히/잘림/거부`)을 케이스 기반으로 고정한다.

케이스:
- `input_valid_noop.json`: `맞음/가만히`
- `input_clip.json`: out-of-range `clip` -> `잘림`
- `input_reject.json`: out-of-range `reject` -> `거부`
