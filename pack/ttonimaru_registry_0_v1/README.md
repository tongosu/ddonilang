# ttonimaru_registry_0_v1

`카-0` Ttonimaru platform charter v1 schema/checker fixture pack.

This pack fixes the minimum `ddn.ttonimaru.platform_charter.v1` contract for
the platform shell. It is schema and contract evidence only; it does not claim
server, database, public registry, runtime truth, replay, or state hash
ownership completion.

Validation:

- `python tests/run_ttonimaru_registry_0_check.py`
- `python tests/run_ttonimaru_registry_0_check.py --file pack/ttonimaru_registry_0_v1/valid/valid_platform_charter.detjson`
- `python tests/run_ttonimaru_registry_0_check.py --dir pack/ttonimaru_registry_0_v1/valid`
- `python tests/run_ttonimaru_registry_0_platform_contract_check.py`

Fixture layout:

- `valid/*.detjson`: independent charter documents that must pass.
- `invalid/*.detjson`: charter documents that must fail with the top-level
  `expected_error` value.

Key boundaries:

- Ttonimaru is a platform shell and does not own runtime truth.
- `runtime_truth_owner`, `state_hash_owner`, and `replay_owner` must be `false`.
- Machine object kinds follow `platform_contract.js`; use `artifact`, not
  `published_artifact`.
- Public artifacts are immutable snapshots pinned to a revision. Draft changes
  do not auto-reflect to public artifacts.
- Public API v1 is read-only/add-only for this charter; authoring mutation stays
  under internal v0.
