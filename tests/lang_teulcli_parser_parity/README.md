# Lang Teul-CLI Parser Parity

## Stable Contract

- 목적:
  - `tool canon` frontdoor와 `teul-cli canon` frontdoor가 현재 고정한 parser/canon surface 4건에서 같은 accept/canonicalization/warning을 보인다는 점을 고정한다.
  - 이 문서는 parser/canon surface의 shared parity 4건을 기록한다.
- compared frontdoors:
  - `tool/src/main.rs`
  - `tools/teul-cli/src/cli/canon.rs`
  - `tools/teul-cli/src/canon.rs`
- shared parity:
  - `채비:` 블록 헤더는 두 frontdoor 모두 입력으로 수용하고 `채비 {`로 정본화한다.
  - `보개장면`은 두 frontdoor 모두 `보개마당`으로 정본화하고 `W_BOGAE_MADANG_ALIAS_DEPRECATED` 경고를 남긴다.
  - `구성`은 두 frontdoor 모두 `짜임`으로 정본화하고 `W_JJAIM_ALIAS_DEPRECATED` 경고를 남긴다.
  - `채비` 선언값의 `조건 { ... }` alias는 두 frontdoor 모두 `매김`으로 수용해 정본화한다.
- selftest:
  - `python tests/run_lang_teulcli_parser_parity_selftest.py`
  - `python tests/run_lang_surface_family_selftest.py`
  - `python tests/run_lang_surface_family_contract_selftest.py`
  - `python tests/run_lang_surface_family_contract_summary_selftest.py`
  - `python tests/run_ci_sanity_gate.py --profile core_lang`

## Cases

- parity:
  - `block_header_colon_deprecated`
  - `bogae_madang_alias_parity`
  - `jjaim_alias_warning_parity`
  - `decl_maegim_alias_parity`

## Consumer Surface

- upstream family:
  - `tests/lang_surface_family/README.md`
  - `python tests/run_lang_surface_family_selftest.py`
- direct selftest:
  - `python tests/run_lang_teulcli_parser_parity_selftest.py`
