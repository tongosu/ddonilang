# AGE4 Proof Transport Contract

## 목적
- `AGE4 proof` child artifact가 aggregate, summary, failure handoff, report-index 소비층까지 끊기지 않고 전달되는지 빠르게 훑기 위한 요약 문서다.
- 개별 child/aggregate selftest를 하나로 묶는 번들 selftest와, 그 번들 자체의 stable contract 요약을 함께 둔다.

## 번들 범위

| 단계 | selftest | 의미 |
| --- | --- | --- |
| proof summary | `run_proof_artifact_digest_selftest.py` | proof detjson 요약/다이제스트 기초 계약 |
| child report | `run_age4_proof_artifact_report_selftest.py` | `ddn.age4.proof_artifact_report.v1` 생성 계약 |
| aggregate combine | `run_ci_combine_reports_age4_selftest.py` | child proof report가 aggregate `age4` row에 접히는 계약 |
| aggregate compact | `run_ci_aggregate_status_line_selftest.py` | `age4_proof_*`가 aggregate status compact line에 노출되는 계약 |
| summary body | `run_ci_gate_summary_report_check_selftest.py` | `ci_gate_summary.txt` 본문에 proof snapshot이 드러나는 계약 |
| emit artifacts | `run_ci_emit_artifacts_check_selftest.py` | summary/brief/triage 소비층 parity 계약 |
| final emitter | `run_ci_final_line_emitter_check.py` | final line, `ci_fail_brief.txt`, `ci_fail_triage.detjson` 소비 계약 |
| report index | `run_ci_gate_report_index_check_selftest.py` | report-index가 final/result/failure proof snapshot parity를 읽는 계약 |

## Stable Contract

- bundle `checks_text`: `proof_artifact_digest,proof_artifact_report,aggregate_combine,aggregate_status_line,gate_summary_report,emit_artifacts,final_line_emitter,report_index`
- bundle selftest: `run_age4_proof_transport_contract_selftest.py`
- summary selftest: `run_age4_proof_transport_contract_summary_selftest.py`
- sanity steps:
  - `age4_proof_transport_contract_selftest`
  - `age4_proof_transport_contract_summary_selftest`
- direct/consumer surface:
  - `ddn.proof_artifact_summary.v1`
  - `ddn.age4.proof_artifact_report.v1`
  - aggregate `age4` row
  - aggregate status line
  - `ci_gate_summary.txt`
  - `ci_fail_brief.txt`
  - `ci_fail_triage.detjson`
  - `ci_gate_report_index`
- required proof snapshot keys:
  - `age4_proof_ok`
  - `age4_proof_failed_criteria`
  - `age4_proof_failed_preview`
  - `age4_proof_summary_hash`

## 참고
- transport bundle self-check: `python tests/run_age4_proof_transport_contract_selftest.py`
- stable contract summary self-check: `python tests/run_age4_proof_transport_contract_summary_selftest.py`
- `core_lang/full` sanity step: `age4_proof_transport_contract_selftest`
- `core_lang/full` sanity step: `age4_proof_transport_contract_summary_selftest`
- `ci_sanity_gate --json-out` progress schema: `ddn.ci.age4_proof_transport_contract_selftest.progress.v1`
- `ci_sanity_gate` stdout token: `age4_proof_transport_contract_selftest_completed_checks`, `...total_checks`, `...checks_text`, `...last_completed_probe`
