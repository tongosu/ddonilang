# studio_benchmark_lts_matrix_v1

This pack records the local evidence for `STUDIO_BENCHMARK_LTS_MATRIX_V1`.

It defines the Studio benchmark/LTS candidate gate matrix without running benchmarks, publishing a performance baseline, certifying LTS status, approving a release, executing a release, uploading files, publishing registry rows, creating public links, enabling installs, emitting publication snapshots, creating a GitHub Release, generating archives, syncing cloud state, adding accounts, changing permissions, or changing product/runtime behavior.

Verification:

```powershell
python tests/run_pack_golden.py studio_benchmark_lts_matrix_v1
python tests/run_studio_benchmark_lts_matrix_check.py
```
