# studio_local_release_rehearsal_check_v1

This pack records the local evidence and product UI behavior check for `STUDIO_LOCAL_RELEASE_REHEARSAL_CHECK_V1`.

It checks local release rehearsal inputs without approving a release, executing a release, generating archives, generating publication checksums, signing artifacts, certifying LTS status, running benchmarks, uploading files, publishing registry rows, creating public links, enabling installs, emitting publication snapshots, creating a GitHub Release, syncing cloud state, adding accounts, changing permissions, or changing runtime behavior.

Progress:

- 작업 단위: 6/6 = 100% (`닫힘-동작`)
- 전체 초장기 계획: 18/18 = 100%
- 현재 스테이지: post-super-long follow-up 3/8 = 38%
- ROADMAP_V2 product behavior baseline: 90/90 = 100%

Verification:

```powershell
python tests/run_pack_golden.py studio_local_release_rehearsal_check_v1
node tests/studio_local_release_rehearsal_check_runner.mjs
python tests/run_studio_local_release_rehearsal_check.py
```
