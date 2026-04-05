# gogae9_w94_social_sim

- 상태: W94 사회 시뮬 최소 계약 (deterministic)
- 기준: `docs/ssot/walks/gogae9/w94_social_sim/README.md`
- Pack ID: `pack/gogae9_w94_social_sim`

## 계약 요약
- 입력: `social.world.ddn` (JSON 직렬화 문서)
- 실행: `teul-cli social simulate --input ... --out ...`
- 출력: `social_report.detjson` + `social_report_hash` + `final_state_hash`
- 결정성: 동일 입력 2회 실행 시 hash가 동일해야 한다.

## 파일
- `intent.md`
- `social_cases.json`
- `golden.detjson`
- `golden.jsonl`
- `inputs/c00_contract_anchor/{input.ddn,expected_canon.ddn}`
- `inputs/c01_inequality/social.world.ddn`
- `inputs/c02_conflict/social.world.ddn`
- `inputs/c03_harmony/social.world.ddn`

## 상위 Family
- `tests/gate0_family/README.md`
- `python tests/run_gate0_family_selftest.py`
