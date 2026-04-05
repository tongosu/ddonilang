# gaji_registry_report_provenance_v1

`gaji registry verify/audit-verify` report provenance를 relative path 기준으로 고정하는 pack.

## 계약
- `verify report`는 `source_hash/source_provenance`를 포함해야 한다.
- `source_provenance`는 `fixtures/registry.index.json`, `fixtures/ddn.lock`의 상대 경로와 raw-byte `sha256`을 기록해야 한다.
- `audit verify report`는 `out/registry.audit.jsonl`의 상대 경로와 raw-byte `sha256`을 기록해야 한다.
- `publish`는 `--at 2026-03-23T00:00:00Z` 고정값으로 실행해 audit row와 `last_hash`를 결정적으로 유지한다.

## 구성
- `fixtures/registry.index.json`
- `fixtures/ddn.lock`
- `expected/registry.verify.report.json`
- `expected/registry.audit.verify.report.json`
