# education_curriculum_5_v1

ROADMAP_V2 `하-5` education operations LTS evidence for `HA5_EDUCATION_OPERATIONS_LTS_V1`.

- Work item: `HA5_EDUCATION_OPERATIONS_LTS_V1`
- Coordinate: `하-5`
- Closure tier: `닫힘-동작`
- Scope: local submission versioning, assessment archive, curriculum version lock, local LTS gate, operations handoff.
- Non-claims: remote LTS certification, live submission, gradebook write, student personal data collection, remote classroom sync, release execution, registry publish, account permission change, state hash participation.

Validation:

```powershell
python tests/run_pack_golden.py education_curriculum_5_v1
node tests/education_operations_lts_runner.mjs
python tests/run_roadmap_v2_ha5_education_operations_lts_check.py
```
