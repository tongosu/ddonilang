# STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1

Date: 2026-06-07

## Summary

`STUDIO_PUBLIC_RELEASE_APPROVAL_RECHECK_V1` closes the second item in the post-super-long Studio follow-up plan.

This stage rechecks the public release approval boundary after the completed super-long Studio plan and post-super-long rebase. It confirms that the exact approval phrase remains required, that ordinary next-development requests are not approval, and that the current development request is not release approval.

Primary coordinate: `마-3` — Studio public release approval readiness recheck.

Support coordinate: `타-3` — checker/approval-chain evidence.

No release approval, release execution, LTS certification, benchmark execution, performance baseline, GitHub Release, public upload, registry publication, public link creation, package install enablement, publication snapshot emission, archive generation, checksum generation for publication, cloud sync, account setup, permission system, result replay, parser/frontdoor grammar, DDN runtime surface, stdlib surface, lesson schema, active allowlist, or `docs/ssot/**` content changes are made.

This revision adds a local Studio product UI panel for the approval recheck boundary.

## Recheck Scope

Recheck schema: `ddn.studio.public_release_approval_recheck.v1`.

Required approval phrase: `STUDIO_PUBLIC_RELEASE_EXECUTION_V1 실행을 승인합니다`.

The recheck records:

- source post-super-long rebase: `pack/studio_post_super_long_rebase_v1/post_super_long_rebase.detjson`;
- source approval continuity: `pack/studio_release_approval_packet_continuity_v1/continuity.detjson`;
- source approval chain closure: `pack/studio_release_approval_chain_closure_v1/closure.detjson`;
- generic next-development requests are not approval;
- current development request is not release approval;
- next state remains `AWAIT_EXPLICIT_RELEASE_APPROVAL`;
- release execution remains blocked until exact explicit approval.

## Product Changes

- Adds `buildPublicReleaseApprovalRecheck`.
- Adds `formatPublicReleaseApprovalRecheckText`.
- Adds `renderPublicReleaseApprovalRecheck`.
- Adds a Studio catalog panel for the approval recheck rows.
- Shows the exact required approval phrase in the UI.
- Shows the current state as `AWAIT_EXPLICIT_RELEASE_APPROVAL`.
- Adds deterministic copy text for the approval recheck artifact.

## Evidence

- `pack/studio_public_release_approval_recheck_v1`
- `pack/studio_public_release_approval_recheck_v1/public_release_approval_recheck.detjson`
- `tests/studio_public_release_approval_recheck_runner.mjs`
- `tests/run_studio_public_release_approval_recheck_check.py`
- `docs/studio/PUBLIC_RELEASE_APPROVAL_RECHECK_V1.md`

## Progress Accounting

- 작업 단위: 6/6 = 100% (`닫힘-동작`)
- 기획: 1/1 = 100%
- approval rows: 5/5 = 100%
- approval recheck stages: 6/6 = 100%
- 전체 초장기 계획: 18/18 = 100%
- 현재 스테이지: post-super-long follow-up 2/8 = 25%
- ROADMAP_V2 product behavior baseline: 90/90 = 100%

## Verification

```powershell
python -m py_compile tests/run_studio_public_release_approval_recheck_check.py
node tests/studio_public_release_approval_recheck_runner.mjs
python tests/run_pack_golden.py studio_public_release_approval_recheck_v1
python tests/run_studio_public_release_approval_recheck_check.py
python tests/run_studio_post_super_long_rebase_check.py
python tests/run_studio_release_approval_packet_continuity_check.py
python tests/run_seamgrim_product_stabilization_smoke_check.py
git diff --check
git status --short -- docs/ssot
```

## Boundaries

- No release approval.
- No release execution.
- No LTS certification.
- No benchmark execution.
- No performance baseline.
- No GitHub Release creation.
- No public upload.
- No registry publication.
- No public link creation.
- No package install enablement.
- No publication snapshot emission.
- No archive generation.
- No checksum generation for publication.
- No cloud sync.
- No account setup.
- No permission system.
- No result replay.
- No lesson schema change.
- No active allowlist mutation.
- No DDN runtime claim.
- No parser/frontdoor grammar change.
- No stdlib surface change.
- `docs/ssot/**` remains unchanged.

## Next

The next recommended item is `STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1`.
