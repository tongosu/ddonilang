# Compound Update Reject Contract

## Stable Contract
- 이 계약면은 AGE1 복합 갱신 문법의 거부 경계를 고정한다.
- canonical surface:
  - `+<-`
  - `-<-`
- forbidden surface:
  - `+=`
  - `-=`
- source of truth in repo:
  - `docs/status/AGE1_PENDING.md`
  - `pack/compound_update_basics/README.md`
  - `pack/compound_update_basics/golden.jsonl`
  - `tools/teul-cli/src/canon.rs`
  - `tools/teul-cli/src/lang/parser.rs`
- reject rule:
  - `+=`/`-=`는 canon에서 `E_CANON_UNSUPPORTED_COMPOUND_UPDATE`로 거부된다.
  - `+=`/`-=`는 run(parse)에서 `E_PARSE_UNEXPECTED_TOKEN`으로 거부된다.
  - 안내 문구는 `+=/-=는 미지원입니다. +<-/ -<-를 사용하세요.`와 `'+<-' 또는 '-<-' (+=/-=는 미지원)`를 유지한다.

## Checks
- upstream family: `tests/lang_surface_family/README.md`
- `python tests/run_compound_update_reject_contract_selftest.py`
- `python tests/run_lang_surface_family_selftest.py`
- `python tests/run_pack_golden.py compound_update_basics`
- `python tests/run_ci_sanity_gate.py --profile core_lang`
