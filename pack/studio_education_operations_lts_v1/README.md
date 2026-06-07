# studio_education_operations_lts_v1

This pack records the local evidence for `STUDIO_EDUCATION_OPERATIONS_LTS_V1`.

It defines the Studio education-operations LTS readiness envelope without certifying LTS status, running benchmarks, publishing a performance baseline, approving a release, executing a release, uploading files, publishing registry rows, creating public links, enabling installs, emitting publication snapshots, creating a GitHub Release, generating archives, syncing cloud state, adding accounts, changing permissions, or changing product/runtime behavior.

Verification:

```powershell
python tests/run_pack_golden.py studio_education_operations_lts_v1
python tests/run_studio_education_operations_lts_check.py
```
